// Rate-limited LLM wrapper
use adk_core::{Llm, LlmRequest, LlmResponseStream};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// A wrapper around any Llm implementation that adds rate limiting
/// by introducing a delay before each API call.
pub struct RateLimitedLlm {
    inner: Arc<dyn Llm>,
    delay_ms: u64,
}

impl RateLimitedLlm {
    /// Create a new rate-limited LLM wrapper
    ///
    /// # Arguments
    /// * `inner` - The underlying LLM implementation
    /// * `delay_ms` - Delay in milliseconds before each API call
    pub fn new(inner: Arc<dyn Llm>, delay_ms: u64) -> Self {
        Self { inner, delay_ms }
    }

    /// Create with 2-second delay (for <30 calls per minute limit)
    pub fn with_default_delay(inner: Arc<dyn Llm>) -> Self {
        Self::new(inner, 2000) // 2 seconds = 2000ms
    }
}

#[async_trait]
impl Llm for RateLimitedLlm {
    fn name(&self) -> &str {
        self.inner.name()
    }

    async fn generate_content(
        &self,
        req: LlmRequest,
        stream: bool,
    ) -> adk_core::Result<LlmResponseStream> {
        // Wait before making the API call
        sleep(Duration::from_millis(self.delay_ms)).await;
        
        // Delegate to the inner LLM
        self.inner.generate_content(req, stream).await
    }
}

