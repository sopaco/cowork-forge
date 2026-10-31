// Idea store —— Idea 聚合的版本化持久化 + 追加式决策审计（设计 §7.2 / §9）
//
// 布局：
//   .cowork-v2/ideas/{idea_id}.json          一 idea 一文件（含最近一次 IdeaDecision）
//   .cowork-v2/autonomy/decisions.log.jsonl  追加式审计（撤销也追加，不改写历史）

use std::io::Write;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::idea::{DecisionAction, Idea};

use super::{from_versioned_json, get_cowork_dir, to_versioned_json};

/// 一条决策审计记录。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionLogEntry {
    pub entry_id: String,
    pub ts: DateTime<Utc>,
    pub idea_id: String,
    pub action: DecisionAction,
    pub by: String,
    pub comment: String,
    pub resulting_iteration_id: Option<String>,
    /// 若本条为撤销记录，该字段指向被撤销的原始决策 `entry_id`。
    /// 撤销以追加新记录表达，绝不改写历史条目。
    pub reversed_by: Option<String>,
}

impl DecisionLogEntry {
    pub fn new(
        idea_id: &str,
        action: DecisionAction,
        by: &str,
        comment: &str,
        resulting_iteration_id: Option<String>,
    ) -> Self {
        Self {
            entry_id: format!("dec-{}", uuid::Uuid::new_v4()),
            ts: Utc::now(),
            idea_id: idea_id.to_string(),
            action,
            by: by.to_string(),
            comment: comment.to_string(),
            resulting_iteration_id,
            reversed_by: None,
        }
    }

    /// 生成一条撤销记录，指向 `original`。
    pub fn reversal_of(original: &DecisionLogEntry, by: &str, comment: &str) -> Self {
        Self {
            entry_id: format!("dec-{}", uuid::Uuid::new_v4()),
            ts: Utc::now(),
            idea_id: original.idea_id.clone(),
            action: original.action,
            by: by.to_string(),
            comment: comment.to_string(),
            resulting_iteration_id: None,
            reversed_by: Some(original.entry_id.clone()),
        }
    }

    pub fn is_reversal(&self) -> bool {
        self.reversed_by.is_some()
    }
}

/// Idea store for persistence
pub struct IdeaStore;

impl IdeaStore {
    pub fn new() -> Self {
        Self
    }

    pub fn save(&self, idea: &Idea) -> anyhow::Result<()> {
        let path = self.idea_file_path(&idea.id)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, to_versioned_json(idea)?)?;
        Ok(())
    }

    pub fn load(&self, idea_id: &str) -> anyhow::Result<Idea> {
        let path = self.idea_file_path(idea_id)?;
        if !path.exists() {
            anyhow::bail!("Idea not found: {}", idea_id);
        }
        let content = std::fs::read_to_string(&path)?;
        from_versioned_json(&content, &path.display().to_string())
    }

    pub fn exists(&self, idea_id: &str) -> bool {
        self.idea_file_path(idea_id).map(|p| p.exists()).unwrap_or(false)
    }

    pub fn delete(&self, idea_id: &str) -> anyhow::Result<()> {
        let path = self.idea_file_path(idea_id)?;
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// 读取全部 idea，按创建时间升序。
    pub fn load_all(&self) -> anyhow::Result<Vec<Idea>> {
        let dir = self.ideas_dir()?;
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut ideas = Vec::new();
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                // 与 IterationStore 一致：坏文件显式报错，不静默跳过
                let content = std::fs::read_to_string(&path)?;
                let idea: Idea = from_versioned_json(&content, &path.display().to_string())?;
                ideas.push(idea);
            }
        }
        ideas.sort_by_key(|i| i.created_at);
        Ok(ideas)
    }

    /// 分配下一个 idea id：`idea-{project}-{seq}`，序号取当前最大值 + 1。
    pub fn next_idea_id(&self, project_name: &str) -> anyhow::Result<String> {
        let slug: String = project_name
            .chars()
            .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
            .collect();
        let slug = slug.trim_matches('-');
        let slug = if slug.is_empty() { "project" } else { slug };

        let existing = self.load_all()?;
        let mut max_seq = 0u32;
        for idea in existing {
            if let Some(tail) = idea.id.rsplit('-').next()
                && let Ok(seq) = tail.parse::<u32>()
            {
                max_seq = max_seq.max(seq);
            }
        }
        Ok(format!("idea-{}-{}", slug, max_seq + 1))
    }

    /// 追加一条决策审计（jsonl，一行一条）。
    pub fn append_decision(&self, entry: &DecisionLogEntry) -> anyhow::Result<()> {
        let path = self.decision_log_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::OpenOptions::new().create(true).append(true).open(&path)?;
        let line = serde_json::to_string(entry)?;
        writeln!(file, "{}", line)?;
        Ok(())
    }

    /// 读取全部审计记录（按文件顺序，即时间顺序）。
    pub fn load_decision_log(&self) -> anyhow::Result<Vec<DecisionLogEntry>> {
        let path = self.decision_log_path()?;
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&path)?;
        let mut entries = Vec::new();
        for (index, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let entry: DecisionLogEntry = serde_json::from_str(line).map_err(|e| {
                anyhow::anyhow!("{}: 第 {} 行审计记录无法解析: {}", path.display(), index + 1, e)
            })?;
            entries.push(entry);
        }
        Ok(entries)
    }

    /// 某条 idea 的决策时间线（含撤销记录）。
    pub fn decisions_for(&self, idea_id: &str) -> anyhow::Result<Vec<DecisionLogEntry>> {
        Ok(self
            .load_decision_log()?
            .into_iter()
            .filter(|e| e.idea_id == idea_id)
            .collect())
    }

    fn ideas_dir(&self) -> anyhow::Result<PathBuf> {
        Ok(get_cowork_dir()?.join("ideas"))
    }

    fn idea_file_path(&self, idea_id: &str) -> anyhow::Result<PathBuf> {
        Ok(self.ideas_dir()?.join(format!("{}.json", idea_id)))
    }

    fn decision_log_path(&self) -> anyhow::Result<PathBuf> {
        Ok(get_cowork_dir()?.join("autonomy").join("decisions.log.jsonl"))
    }
}

impl Default for IdeaStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::idea::{Effort, IdeaDraft, IdeaSource};
    use crate::persistence::WORKSPACE_TEST_LOCK as TEST_LOCK;
    use std::sync::PoisonError;
    use tempfile::TempDir;

    fn sample_idea(id: &str) -> Idea {
        IdeaDraft {
            title: "补上重试幂等".to_string(),
            problem: "重试会重复生成文档".to_string(),
            hypothesis: "补判断后只补缺口".to_string(),
            scope: crate::domain::idea::IdeaScope::Fix,
            evidence: vec![crate::domain::idea::Evidence::FileRef {
                path: "src/main.rs".to_string(),
                line_range: None,
                excerpt: "fn retry() {}".to_string(),
            }],
            estimated_effort: Effort::S,
            priority: crate::data::models::Priority::High,
            expected_benefit: "不再重复".to_string(),
            conflicts_with: vec![],
        }
        .into_idea(id.to_string(), IdeaSource::Research)
    }

    #[test]
    fn idea_save_load_round_trip_and_load_all() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let temp = TempDir::new().expect("temp dir");
        crate::persistence::set_workspace_path(temp.path().to_path_buf());

        let store = IdeaStore::new();
        assert!(store.load_all().expect("load all").is_empty());

        let idea = sample_idea("idea-demo-1");
        store.save(&idea).expect("save");
        assert!(store.exists("idea-demo-1"));
        assert_eq!(store.load("idea-demo-1").expect("load"), idea);
        assert_eq!(store.load_all().expect("load all").len(), 1);

        let path = temp.path().join(".cowork-v2").join("ideas").join("idea-demo-1.json");
        assert!(path.exists());
        let raw = std::fs::read_to_string(path).expect("read raw");
        assert!(raw.contains("\"schema_version\": 1"));

        store.delete("idea-demo-1").expect("delete");
        assert!(!store.exists("idea-demo-1"));
        assert!(store.load("idea-demo-1").is_err());
    }

    #[test]
    fn next_idea_id_increments_and_slugs_project_name() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let temp = TempDir::new().expect("temp dir");
        crate::persistence::set_workspace_path(temp.path().to_path_buf());

        let store = IdeaStore::new();
        assert_eq!(store.next_idea_id("My App").expect("id"), "idea-my-app-1");
        store.save(&sample_idea("idea-my-app-1")).expect("save");
        assert_eq!(store.next_idea_id("My App").expect("id"), "idea-my-app-2");
    }

    #[test]
    fn decision_log_is_append_only_and_undo_appends_reversal() {
        let _t = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let temp = TempDir::new().expect("temp dir");
        crate::persistence::set_workspace_path(temp.path().to_path_buf());

        let store = IdeaStore::new();
        assert!(store.load_decision_log().expect("empty log").is_empty());

        let accept = DecisionLogEntry::new(
            "idea-demo-1",
            DecisionAction::Accept,
            "user",
            "值得做",
            Some("iter-2-1".to_string()),
        );
        store.append_decision(&accept).expect("append accept");

        let undo = DecisionLogEntry::reversal_of(&accept, "user", "误操作");
        store.append_decision(&undo).expect("append undo");

        let log = store.load_decision_log().expect("load log");
        assert_eq!(log.len(), 2, "撤销必须追加而非改写历史");
        assert_eq!(log[0].entry_id, accept.entry_id);
        assert!(log[0].reversed_by.is_none());
        assert_eq!(log[1].reversed_by.as_deref(), Some(accept.entry_id.as_str()));
        assert!(log[1].is_reversal());
        assert_eq!(log[1].action, DecisionAction::Accept);
        assert_eq!(log[0].resulting_iteration_id.as_deref(), Some("iter-2-1"));

        let timeline = store.decisions_for("idea-demo-1").expect("timeline");
        assert_eq!(timeline.len(), 2);
        assert!(store.decisions_for("idea-other").expect("other").is_empty());

        let raw = std::fs::read_to_string(
            temp.path().join(".cowork-v2").join("autonomy").join("decisions.log.jsonl"),
        )
        .expect("read log");
        assert_eq!(raw.lines().count(), 2, "jsonl 一行一条");
    }
}
