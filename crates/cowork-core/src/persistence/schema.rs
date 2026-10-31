// Persistence schema versioning
//
// 一次定型策略：所有 .cowork-v2 JSON 文档在落盘时携带顶层 `schema_version`（数值），
// 读取时严格校验。无法识别的版本直接报错，不做迁移、不做静默降级。

use anyhow::{bail, Context, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

/// 当前持久化 schema 版本。任何不兼容的格式变更都必须提升此版本号。
pub const SCHEMA_VERSION: u32 = 1;

/// 将文档序列化为 pretty JSON，并在顶层写入 `schema_version`。
///
/// 扁平信封（不包裹 `data` 字段），保持既有 JSON 文件风格不变。
pub fn to_versioned_json<T: Serialize>(value: &T) -> Result<String> {
    let mut root = serde_json::to_value(value).context("Failed to serialize persisted document")?;
    let obj = root
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("Persisted document must serialize to a JSON object"))?;
    obj.insert("schema_version".to_string(), Value::from(SCHEMA_VERSION));
    serde_json::to_string_pretty(&root).context("Failed to serialize versioned document")
}

/// 解析 [`to_versioned_json`] 写入的文档，强制校验 schema 版本。
///
/// `source` 用于错误信息（通常是文件名）。版本缺失或无法识别时返回明确错误，
/// 不做任何迁移或降级。
pub fn from_versioned_json<T: DeserializeOwned>(content: &str, source: &str) -> Result<T> {
    let mut root: Value =
        serde_json::from_str(content).with_context(|| format!("Failed to parse {} as JSON", source))?;
    let obj = root
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("{}: expected a JSON object", source))?;

    match obj.remove("schema_version") {
        Some(Value::Number(n)) if n.as_u64() == Some(u64::from(SCHEMA_VERSION)) => {}
        Some(other) => bail!(
            "{}: unsupported schema_version {} (this build only supports {}); \
             automatic migration is not supported",
            source,
            other,
            SCHEMA_VERSION
        ),
        None => bail!(
            "{}: missing schema_version field (expected {}); \
             the file was not written by this storage layer",
            source,
            SCHEMA_VERSION
        ),
    }

    serde_json::from_value(root).with_context(|| format!("Failed to deserialize {}", source))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct SampleDoc {
        name: String,
        count: u32,
    }

    fn sample() -> SampleDoc {
        SampleDoc {
            name: "demo".to_string(),
            count: 7,
        }
    }

    #[test]
    fn round_trip_preserves_payload() {
        let json = to_versioned_json(&sample()).expect("serialize");
        assert!(json.contains("\"schema_version\": 1"));
        let back: SampleDoc = from_versioned_json(&json, "sample.json").expect("deserialize");
        assert_eq!(back, sample());
    }

    #[test]
    fn missing_version_is_rejected() {
        let err = from_versioned_json::<SampleDoc>(r#"{"name":"demo","count":7}"#, "sample.json")
            .expect_err("must reject missing schema_version");
        let msg = format!("{:#}", err);
        assert!(msg.contains("missing schema_version"), "unexpected error: {}", msg);
        assert!(msg.contains("sample.json"), "unexpected error: {}", msg);
    }

    #[test]
    fn unknown_version_is_rejected() {
        let err = from_versioned_json::<SampleDoc>(
            r#"{"schema_version":99,"name":"demo","count":7}"#,
            "sample.json",
        )
        .expect_err("must reject unknown schema_version");
        let msg = format!("{:#}", err);
        assert!(msg.contains("unsupported schema_version 99"), "unexpected error: {}", msg);
        assert!(msg.contains("sample.json"), "unexpected error: {}", msg);
        assert!(msg.contains("migration is not supported"), "unexpected error: {}", msg);
    }

    #[test]
    fn non_object_document_is_rejected() {
        let err = from_versioned_json::<SampleDoc>(r#"[1,2,3]"#, "sample.json")
            .expect_err("must reject non-object document");
        assert!(format!("{:#}", err).contains("expected a JSON object"));
    }
}
