use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::path::{Path, PathBuf};

use crate::artifacts::Stage;

#[cfg(test)]
mod tests;

/// Artifact 存储接口（简化为直接使用 FileArtifactStore）
pub struct ArtifactStore {
    store: FileArtifactStore,
}

impl ArtifactStore {
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            store: FileArtifactStore::new(base_dir),
        }
    }

    /// 写入 artifact（json + md）
    pub fn put<T: Serialize>(&self, session_id: &str, stage: Stage, artifact: &T) -> Result<String> {
        self.store.put(session_id, stage, artifact)
    }

    /// 读取 artifact（json）
    pub fn get<T: DeserializeOwned>(&self, session_id: &str, artifact_id: &str) -> Result<T> {
        self.store.get(session_id, artifact_id)
    }

    /// 列出 session 的所有 artifacts
    pub fn list(&self, session_id: &str) -> Result<Vec<ArtifactMeta>> {
        self.store.list(session_id)
    }

    /// 检查 session 是否存在
    pub fn session_exists(&self, session_id: &str) -> bool {
        self.store.session_exists(session_id)
    }
}

#[derive(Debug, Clone)]
pub struct ArtifactMeta {
    pub artifact_id: String,
    pub stage: Stage,
    pub path_json: PathBuf,
    pub path_md: PathBuf,
}

/// 默认的文件存储实现
struct FileArtifactStore {
    base_dir: PathBuf,
}

impl FileArtifactStore {
    fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    fn session_dir(&self, session_id: &str) -> PathBuf {
        self.base_dir.join(session_id)
    }

    fn artifacts_dir(&self, session_id: &str) -> PathBuf {
        self.session_dir(session_id).join("artifacts")
    }

    fn artifact_path(&self, session_id: &str, stage: Stage, artifact_id: &str, ext: &str) -> PathBuf {
        self.artifacts_dir(session_id)
            .join(format!("{}.{}.{}", stage.as_str(), artifact_id, ext))
    }

    fn put<T: Serialize>(&self, session_id: &str, stage: Stage, artifact: &T) -> Result<String> {
        use std::fs;

        let artifacts_dir = self.artifacts_dir(session_id);
        fs::create_dir_all(&artifacts_dir)?;

        // Extract artifact_id from the artifact (assuming it has a meta field)
        let json_str = serde_json::to_string_pretty(artifact)?;
        let json_value: serde_json::Value = serde_json::from_str(&json_str)?;
        let artifact_id = json_value["meta"]["artifact_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing artifact_id in meta"))?
            .to_string();

        // Write JSON
        let json_path = self.artifact_path(session_id, stage, &artifact_id, "json");
        fs::write(&json_path, json_str)?;

        // Write MD (minimal template)
        let md_content = self.generate_markdown(&json_value)?;
        let md_path = self.artifact_path(session_id, stage, &artifact_id, "md");
        fs::write(&md_path, md_content)?;

        tracing::info!("Artifact saved: {}", artifact_id);
        Ok(artifact_id)
    }

    fn get<T: DeserializeOwned>(&self, session_id: &str, artifact_id: &str) -> Result<T> {
        use std::fs;

        // Find the artifact by scanning the artifacts directory
        let artifacts_dir = self.artifacts_dir(session_id);
        for entry in fs::read_dir(&artifacts_dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.contains(artifact_id) && name.ends_with(".json") {
                    let content = fs::read_to_string(&path)?;
                    return Ok(serde_json::from_str(&content)?);
                }
            }
        }

        anyhow::bail!("Artifact not found: {}", artifact_id)
    }

    fn list(&self, session_id: &str) -> Result<Vec<ArtifactMeta>> {
        use std::fs;

        let artifacts_dir = self.artifacts_dir(session_id);
        if !artifacts_dir.exists() {
            return Ok(Vec::new());
        }

        let mut artifacts = Vec::new();
        for entry in fs::read_dir(&artifacts_dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".json") {
                    // Parse: <stage>.<artifact_id>.json
                    let parts: Vec<&str> = name.rsplitn(3, '.').collect();
                    if parts.len() == 3 {
                        let artifact_id = parts[1].to_string();
                        let stage_str = parts[2];
                        if let Some(stage) = self.parse_stage(stage_str) {
                            let path_json = path.clone();
                            let path_md = path.with_extension("md");
                            artifacts.push(ArtifactMeta {
                                artifact_id,
                                stage,
                                path_json,
                                path_md,
                            });
                        }
                    }
                }
            }
        }

        Ok(artifacts)
    }

    fn session_exists(&self, session_id: &str) -> bool {
        self.session_dir(session_id).exists()
    }

    fn parse_stage(&self, s: &str) -> Option<Stage> {
        match s {
            "idea_intake" => Some(Stage::IdeaIntake),
            "requirements" => Some(Stage::Requirements),
            "design" => Some(Stage::Design),
            "plan" => Some(Stage::Plan),
            "coding" => Some(Stage::Coding),
            "check" => Some(Stage::Check),
            "feedback" => Some(Stage::Feedback),
            "delivery" => Some(Stage::Delivery),
            _ => None,
        }
    }

    fn generate_markdown(&self, json: &serde_json::Value) -> Result<String> {
        let mut md = String::new();

        // Meta
        if let Some(meta) = json.get("meta") {
            md.push_str("# Artifact\n\n");
            md.push_str(&format!("- **Session ID**: {}\n", meta["session_id"].as_str().unwrap_or("")));
            md.push_str(&format!("- **Artifact ID**: {}\n", meta["artifact_id"].as_str().unwrap_or("")));
            md.push_str(&format!("- **Stage**: {}\n", meta["stage"].as_str().unwrap_or("")));
            md.push_str(&format!("- **Version**: {}\n", meta["v"].as_u64().unwrap_or(0)));
            md.push_str(&format!("- **Timestamp**: {}\n", meta["ts"].as_i64().unwrap_or(0)));
            md.push_str("\n");
        }

        // Summary
        if let Some(summary) = json.get("summary").and_then(|s| s.as_array()) {
            md.push_str("## Summary\n\n");
            for item in summary {
                if let Some(s) = item.as_str() {
                    md.push_str(&format!("- {}\n", s));
                }
            }
            md.push_str("\n");
        }

        // Data (simplified representation)
        if let Some(data) = json.get("data") {
            md.push_str("## Data\n\n");
            md.push_str("```json\n");
            md.push_str(&serde_json::to_string_pretty(data)?);
            md.push_str("\n```\n");
        }

        Ok(md)
    }
}
