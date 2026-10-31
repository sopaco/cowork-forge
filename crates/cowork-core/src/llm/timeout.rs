// LLM 请求超时装饰器
//
// 与限流器同构：只包裹 inner，语义透明（`name` / `schema_adapter` /
// `uses_interactions_api` 全部透传），不改变 provider 行为。
//
// 为什么需要这一层：上游网关"连接建立后连响应头都不回"时，reqwest
// 默认不设请求超时，`generate_content` 会永久 pending。调用方看到一个
// 永不 settle 的 future，只能靠杀进程恢复。超时必须由**我们**兜底，
// 不能指望上游。
//
// 两层超时各司其职：
// - `request_timeout`：从发起请求到拿到响应流的最长等待（覆盖"无响应"）
// - `stream_idle_timeout`：两个流事件之间的最长静默（覆盖"说了一半就断"）

use adk_core::{AdkError, ErrorCategory, ErrorComponent, Llm, LlmRequest, LlmResponseStream};
use adk_core::SchemaAdapter;
use async_trait::async_trait;
use futures::{StreamExt, stream};
use std::sync::Arc;
use std::time::Duration;

/// 默认请求超时：覆盖连接建立后上游不返回任何字节的情况。
///
/// 取 10 分钟而非更短：调研类请求的 prompt 很长，推理型模型的
/// "思考阶段"可以对前端表现为长时间无字节输出（首字节前不算流事件）。
/// 120s 会把这类**正常但慢**的调用误杀成失败，这里只兜底"真挂死"。
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(600);

/// 默认流空闲超时：覆盖响应流中途静默。
///
/// 与请求超时同值——同一个模型的推理停顿不该因"是否已吐出首字节"
/// 而被两套标准判定。
pub const DEFAULT_STREAM_IDLE_TIMEOUT: Duration = Duration::from_secs(600);

/// 给任意 [`Llm`] 叠加请求超时与流空闲超时。
pub struct TimeoutLlm {
    inner: Arc<dyn Llm>,
    request_timeout: Duration,
    stream_idle_timeout: Duration,
}

impl TimeoutLlm {
    pub fn new(inner: Arc<dyn Llm>, request_timeout: Duration, stream_idle_timeout: Duration) -> Self {
        Self {
            inner,
            request_timeout,
            stream_idle_timeout,
        }
    }

    /// 默认超时配置（600s / 600s）。
    pub fn with_defaults(inner: Arc<dyn Llm>) -> Self {
        Self::new(inner, DEFAULT_REQUEST_TIMEOUT, DEFAULT_STREAM_IDLE_TIMEOUT)
    }
}

/// 超时错误：归类为 Model/Timeout，带可重试提示，错误文案直接进日志与界面。
fn timeout_error(timeout: Duration) -> AdkError {
    AdkError::new(
        ErrorComponent::Model,
        ErrorCategory::Timeout,
        "model.client.timeout",
        format!(
            "模型在 {} 秒内没有返回响应，已中止本次调用（请检查模型是否可用、网关是否通畅）",
            timeout.as_secs()
        ),
    )
}

/// 给响应流补逐块空闲超时：超时后追加一条错误再结束，避免消费方永久等待。
fn with_idle_timeout(
    inner: LlmResponseStream,
    idle: Duration,
) -> LlmResponseStream {
    Box::pin(stream::unfold(
        (inner, false),
        move |(mut stream, finished)| async move {
            if finished {
                return None;
            }
            match tokio::time::timeout(idle, stream.next()).await {
                Ok(Some(item)) => Some((item, (stream, false))),
                Ok(None) => None,
                // 静默超时：吐一条错误，然后终止流
                Err(_) => Some((Err(timeout_error(idle)), (stream, true))),
            }
        },
    ))
}

#[async_trait]
impl Llm for TimeoutLlm {
    fn name(&self) -> &str {
        self.inner.name()
    }

    async fn generate_content(
        &self,
        req: LlmRequest,
        stream: bool,
    ) -> Result<LlmResponseStream, AdkError> {
        // 先落到局部变量：临时值活到语句结束，超时分支才能安全丢弃 future
        let attempt =
            tokio::time::timeout(self.request_timeout, self.inner.generate_content(req, stream))
                .await;

        match attempt {
            Ok(Ok(response)) => Ok(with_idle_timeout(response, self.stream_idle_timeout)),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(timeout_error(self.request_timeout)),
        }
    }

    // 装饰器必须完全透明：schema 归一化与交互式 API 判定交回真实 provider，
    // 否则 OpenAI 等 provider 的专用 schema 适配会被静默替换成通用实现。
    fn schema_adapter(&self) -> &dyn SchemaAdapter {
        self.inner.schema_adapter()
    }

    fn uses_interactions_api(&self) -> bool {
        self.inner.uses_interactions_api()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adk_core::LlmResponse;
    use futures::stream as futures_stream;

    /// 永远不返回的模型：模拟上游"连上就不回包"。
    struct HangingLlm;

    #[async_trait]
    impl Llm for HangingLlm {
        fn name(&self) -> &str {
            "hanging"
        }

        async fn generate_content(
            &self,
            _req: LlmRequest,
            _stream: bool,
        ) -> Result<LlmResponseStream, AdkError> {
            futures::future::pending::<()>().await;
            unreachable!("pending never resolves")
        }
    }

    /// 先吐一条再永久静默的模型：模拟流中途卡死。
    struct StallingLlm;

    #[async_trait]
    impl Llm for StallingLlm {
        fn name(&self) -> &str {
            "stalling"
        }

        async fn generate_content(
            &self,
            _req: LlmRequest,
            _stream: bool,
        ) -> Result<LlmResponseStream, AdkError> {
            // 一条正常响应之后永远 pending
            let s = futures_stream::iter(vec![Ok(LlmResponse::default())])
                .chain(futures_stream::pending());
            Ok(Box::pin(s))
        }
    }

    fn empty_request() -> LlmRequest {
        LlmRequest {
            model: "demo".to_string(),
            contents: Vec::new(),
            config: None,
            tools: Default::default(),
            previous_response_id: None,
        }
    }

    #[tokio::test]
    async fn hanging_request_times_out_instead_of_hanging_forever() {
        let llm = TimeoutLlm::new(
            Arc::new(HangingLlm),
            Duration::from_millis(50),
            Duration::from_millis(50),
        );

        let err = match llm.generate_content(empty_request(), true).await {
            Err(e) => e,
            Ok(_) => panic!("挂死的请求必须超时报错，而不是永久等待"),
        };
        assert_eq!(err.category, ErrorCategory::Timeout);
        assert_eq!(err.code, "model.client.timeout");
        assert!(err.message.contains("没有返回响应"), "{}", err.message);
    }

    #[tokio::test]
    async fn stalled_stream_yields_error_and_terminates() {
        let llm = TimeoutLlm::new(
            Arc::new(StallingLlm),
            Duration::from_secs(5),
            Duration::from_millis(50),
        );

        let mut stream = llm
            .generate_content(empty_request(), true)
            .await
            .expect("首包正常");

        // 第一条正常返回
        let first = stream.next().await.expect("first item");
        assert!(first.is_ok());

        // 之后静默 → 补一条超时错误
        let second = stream.next().await.expect("timeout item");
        assert_eq!(
            second.expect_err("静默后必须是错误").category,
            ErrorCategory::Timeout
        );

        // 错误之后流结束，不会一直转圈
        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn non_timeout_errors_pass_through_unchanged() {
        struct FailingLlm;

        #[async_trait]
        impl Llm for FailingLlm {
            fn name(&self) -> &str {
                "failing"
            }

            async fn generate_content(
                &self,
                _req: LlmRequest,
                _stream: bool,
            ) -> Result<LlmResponseStream, AdkError> {
                Err(AdkError::new(
                    ErrorComponent::Model,
                    ErrorCategory::Unauthorized,
                    "model.test.unauthorized",
                    "bad key",
                ))
            }
        }

        let llm = TimeoutLlm::with_defaults(Arc::new(FailingLlm));
        let err = match llm.generate_content(empty_request(), true).await {
            Err(e) => e,
            Ok(_) => panic!("上游错误必须原样透传，不能被当成成功"),
        };
        assert_eq!(err.category, ErrorCategory::Unauthorized);
        assert_eq!(err.code, "model.test.unauthorized");
    }
}
