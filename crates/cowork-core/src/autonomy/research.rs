// ResearchAgent 的证据收集（设计 §5.1 / §6.2 第 2 步 / §6.3 上下文预算）
//
// 这一层**不调 LLM**：按 research 配置从本地工作区、迭代知识、阶段反馈、
// 未决需求里收集证据并压缩成有上限的上下文。LLM 只在 controller 里
// 基于这份上下文做缺口分析与提案，避免把全量文件塞进 prompt。

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::data::models::{FeatureStatus, FeedbackType};
use crate::domain::idea::{truncated_excerpt, Idea, IdeaStatus};
use crate::persistence::{IterationStore, MemoryStore, ProjectStore};

use super::config::AutonomyResearchConfig;
use super::cycle::CycleSnapshot;

/// 上下文预算常量（设计 §6.3：只喂摘要与增量，不喂全量文件）。
pub const MAX_KNOWN_ISSUES: usize = 12;
pub const MAX_PENDING_FEATURES: usize = 12;
pub const MAX_FEEDBACK_ITEMS: usize = 12;
pub const MAX_WORKSPACE_FILES: usize = 60;
pub const MAX_TODO_MARKERS: usize = 20;
pub const MAX_SCAN_FILES: usize = 40;
pub const MAX_FILE_BYTES: usize = 200_000;
pub const MAX_RECENT_DECISIONS: usize = 8;
pub const EXCERPT_CHARS: usize = 300;
/// prompt 总字符上限（超出则按优先级截断）。
pub const DEFAULT_PROMPT_BUDGET_CHARS: usize = 12_000;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnownIssue {
    pub iteration_id: String,
    pub issue: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PendingFeature {
    pub iteration_id: String,
    pub feature_id: String,
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeedbackItem {
    pub iteration_id: String,
    pub stage: String,
    pub details: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TodoMarker {
    pub path: String,
    pub line: usize,
    pub text: String,
}

/// 一轮调研的上下文（落进 cycle 快照的 evidence_digest 由它派生）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResearchContext {
    pub project_name: String,
    pub tech_stack: Vec<String>,
    pub known_issues: Vec<KnownIssue>,
    pub pending_features: Vec<PendingFeature>,
    pub feedback_items: Vec<FeedbackItem>,
    pub workspace_files: Vec<String>,
    pub todo_markers: Vec<TodoMarker>,
    pub recent_decisions: Vec<String>,
    /// 未决 idea（标题级），提示词据此避免重复提案。
    pub open_idea_lines: Vec<String>,
    /// 已被驳回/转化/置后的 idea（状态抑制，除非证据变化否则不重提）。
    pub suppressed_idea_lines: Vec<String>,
    pub previous_cycle_note: Option<String>,
    /// 外部检索降级标记（由 controller 在外查失败后写入）。
    pub degraded: bool,
    pub degraded_reason: Option<String>,
}

impl ResearchContext {
    /// 本地证据是否足够支撑提案（设计 §5.4 判定信号①：三处都查不到才允许外查）。
    pub fn has_local_evidence(&self) -> bool {
        !(self.known_issues.is_empty()
            && self.pending_features.is_empty()
            && self.feedback_items.is_empty()
            && self.todo_markers.is_empty())
    }

    /// 调研输入的摘要指纹：下一轮据此判断"有没有新东西"。
    pub fn evidence_digest(&self) -> String {
        let mut material = String::new();
        for item in &self.known_issues {
            material.push_str(&format!("issue:{}:{}\n", item.iteration_id, item.issue));
        }
        for item in &self.pending_features {
            material.push_str(&format!("feature:{}:{}\n", item.feature_id, item.status));
        }
        for item in &self.feedback_items {
            material.push_str(&format!(
                "feedback:{}:{}:{}\n",
                item.stage,
                item.timestamp.timestamp(),
                item.details
            ));
        }
        for item in &self.todo_markers {
            material.push_str(&format!("todo:{}:{}\n", item.path, item.line));
        }
        for item in &self.open_idea_lines {
            material.push_str(&format!("open:{}\n", item));
        }
        crate::domain::idea::compute_fingerprint("research", &material)
    }

    pub fn to_prompt(&self) -> String {
        self.to_prompt_with_budget(DEFAULT_PROMPT_BUDGET_CHARS)
    }

    /// 组装调研上下文 prompt，严格受字符预算约束。
    ///
    /// 段内条目超限即丢弃并追加"…（其余 N 条从略）"，绝不把整段撑爆。
    pub fn to_prompt_with_budget(&self, max_chars: usize) -> String {
        let mut out = String::new();
        push_line(&mut out, &format!("项目：{}", self.project_name), max_chars);
        if !self.tech_stack.is_empty() {
            push_line(
                &mut out,
                &format!("技术栈：{}", self.tech_stack.join(" / ")),
                max_chars,
            );
        }

        push_section(
            &mut out,
            "历史迭代遗留的已知问题",
            &self
                .known_issues
                .iter()
                .map(|i| format!("- [{}] {}", i.iteration_id, i.issue))
                .collect::<Vec<_>>(),
            max_chars,
        );
        push_section(
            &mut out,
            "未完成的特性（与 PRD/计划对账）",
            &self
                .pending_features
                .iter()
                .map(|f| format!("- {}（{}，{}）", f.name, f.feature_id, f.status))
                .collect::<Vec<_>>(),
            max_chars,
        );
        push_section(
            &mut out,
            "阶段反馈中未修复的建议",
            &self
                .feedback_items
                .iter()
                .map(|f| format!("- [{}] {}", f.stage, f.details))
                .collect::<Vec<_>>(),
            max_chars,
        );
        push_section(
            &mut out,
            "工作区里的 TODO / FIXME 标记",
            &self
                .todo_markers
                .iter()
                .map(|t| format!("- {}:{} {}", t.path, t.line, t.text))
                .collect::<Vec<_>>(),
            max_chars,
        );
        push_section(
            &mut out,
            "近期关键决策",
            &self
                .recent_decisions
                .iter()
                .map(|d| format!("- {}", d))
                .collect::<Vec<_>>(),
            max_chars,
        );
        push_section(
            &mut out,
            "工作区文件（部分）",
            &self
                .workspace_files
                .iter()
                .map(|f| format!("- {}", f))
                .collect::<Vec<_>>(),
            max_chars,
        );
        push_section(
            &mut out,
            "已存在的未决建议（不要重复提出）",
            &self.open_idea_lines,
            max_chars,
        );
        push_section(
            &mut out,
            "已被驳回/转化/置后的建议（除非证据变化，否则不要重提）",
            &self.suppressed_idea_lines,
            max_chars,
        );
        if let Some(note) = &self.previous_cycle_note {
            push_line(&mut out, &format!("上一轮结论：{}", note), max_chars);
        }
        if self.degraded {
            push_line(
                &mut out,
                &format!(
                    "外部检索不可用（{}），本轮只用本地证据。",
                    self.degraded_reason.clone().unwrap_or_else(|| "原因未知".to_string())
                ),
                max_chars,
            );
        }
        out
    }
}

fn push_line(out: &mut String, line: &str, max_chars: usize) {
    if out.chars().count() >= max_chars {
        return;
    }
    out.push_str(line);
    out.push('\n');
}

fn push_section(out: &mut String, title: &str, lines: &[String], max_chars: usize) {
    if lines.is_empty() || out.chars().count() >= max_chars {
        return;
    }
    out.push_str(&format!("\n## {}\n", title));
    let mut omitted = 0usize;
    for line in lines {
        if out.chars().count() + line.chars().count() + 1 > max_chars {
            omitted += 1;
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    if omitted > 0 {
        out.push_str(&format!("…（其余 {} 条从略）\n", omitted));
    }
}

/// 按 research 配置收集证据。
///
/// `existing_ideas` 用于未决/抑制清单；`previous_cycle` 用于增量提示。
pub fn collect_research(
    config: &AutonomyResearchConfig,
    existing_ideas: &[Idea],
    previous_cycle: Option<&CycleSnapshot>,
) -> anyhow::Result<ResearchContext> {
    let mut ctx = ResearchContext::default();

    let project = ProjectStore::new().load()?.ok_or_else(|| anyhow::anyhow!("项目未初始化"))?;
    ctx.project_name = project.name.clone();
    ctx.tech_stack = project.metadata.tech_stack.clone();

    let iteration_store = IterationStore::new();
    let iterations = iteration_store.load_all().unwrap_or_default();

    // 记忆库：已知问题 + 近期决策
    if config.memory {
        let memory = MemoryStore::new().load_project_memory().unwrap_or_default();
        let mut knowledge: Vec<_> = memory.iteration_knowledge.values().collect();
        knowledge.sort_by_key(|k| k.iteration_number);
        for item in knowledge.into_iter().rev() {
            for issue in &item.known_issues {
                if ctx.known_issues.len() >= MAX_KNOWN_ISSUES {
                    break;
                }
                ctx.known_issues.push(KnownIssue {
                    iteration_id: item.iteration_id.clone(),
                    issue: truncated_excerpt(issue, EXCERPT_CHARS),
                });
            }
        }
        for decision in memory.decisions.iter().rev().take(MAX_RECENT_DECISIONS) {
            ctx.recent_decisions
                .push(truncated_excerpt(&decision.title, EXCERPT_CHARS));
        }
    }

    // 本地工作区 + 未完成特性 + 阶段反馈
    let previous_iteration_id = crate::persistence::get_iteration_id();
    for iteration in iterations.iter().rev().take(3) {
        if config.local {
            crate::persistence::set_iteration_id(iteration.id.clone());
            if let Ok(features) = crate::persistence::load_feature_list() {
                for feature in features.features {
                    if ctx.pending_features.len() >= MAX_PENDING_FEATURES {
                        break;
                    }
                    if matches!(feature.status, FeatureStatus::Completed) {
                        continue;
                    }
                    ctx.pending_features.push(PendingFeature {
                        iteration_id: iteration.id.clone(),
                        feature_id: feature.id.clone(),
                        name: truncated_excerpt(&feature.name, EXCERPT_CHARS),
                        status: format!("{:?}", feature.status),
                    });
                }
            }
        }

        if config.feedback {
            crate::persistence::set_iteration_id(iteration.id.clone());
            if let Ok(history) = crate::persistence::load_feedback_history() {
                for feedback in history.feedbacks.iter().rev() {
                    if ctx.feedback_items.len() >= MAX_FEEDBACK_ITEMS {
                        break;
                    }
                    if !matches!(
                        feedback.feedback_type,
                        FeedbackType::Suggestion
                            | FeedbackType::QualityIssue
                            | FeedbackType::MissingRequirement
                            | FeedbackType::ArchitectureIssue
                    ) {
                        continue;
                    }
                    ctx.feedback_items.push(FeedbackItem {
                        iteration_id: iteration.id.clone(),
                        stage: feedback.stage.clone(),
                        details: truncated_excerpt(&feedback.details, EXCERPT_CHARS),
                        timestamp: feedback.timestamp,
                    });
                }
            }
        }

        // 只扫最近一个迭代的工作区，避免重复劳动
        if config.local && ctx.workspace_files.is_empty() {
            let workspace = iteration_store.workspace_path(&iteration.id)?;
            if workspace.exists() {
                let (files, markers) = scan_workspace(&workspace);
                ctx.workspace_files = files;
                ctx.todo_markers = markers;
            }
        }
    }
    // 恢复全局当前迭代 ID：调研不得污染执行侧的单例假设
    match previous_iteration_id {
        Some(id) => crate::persistence::set_iteration_id(id),
        None => crate::persistence::clear_iteration_id(),
    }

    for idea in existing_ideas {
        let line = format!("- [{}] {}", idea.id, idea.title);
        if idea.status == IdeaStatus::Proposed || idea.status == IdeaStatus::Rewritten {
            ctx.open_idea_lines.push(line);
        } else if idea.status.is_suppressing() {
            ctx.suppressed_idea_lines.push(line);
        }
    }
    ctx.previous_cycle_note = previous_cycle.and_then(|c| c.note.clone());

    Ok(ctx)
}

const SCANNED_EXTENSIONS: [&str; 14] = [
    "rs", "ts", "tsx", "js", "jsx", "mjs", "py", "go", "java", "vue", "toml", "md", "json", "css",
];

/// 扫描工作区：文件清单 + TODO/FIXME/HACK 标记（带数量与体积上限）。
pub fn scan_workspace(workspace: &Path) -> (Vec<String>, Vec<TodoMarker>) {
    let mut files = Vec::new();
    let mut markers = Vec::new();
    let mut scanned = 0usize;

    let walker = ignore::WalkBuilder::new(workspace)
        .max_depth(Some(4))
        .standard_filters(true)
        .build();

    for entry in walker.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let relative = path
            .strip_prefix(workspace)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_lowercase();
        if !SCANNED_EXTENSIONS.contains(&extension.as_str()) {
            continue;
        }

        if files.len() < MAX_WORKSPACE_FILES {
            files.push(relative.clone());
        }

        if scanned >= MAX_SCAN_FILES || markers.len() >= MAX_TODO_MARKERS {
            continue;
        }
        scanned += 1;

        let Ok(metadata) = std::fs::metadata(path) else {
            continue;
        };
        if metadata.len() as usize > MAX_FILE_BYTES {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        for (index, line) in content.lines().enumerate() {
            if markers.len() >= MAX_TODO_MARKERS {
                break;
            }
            let trimmed = line.trim();
            if trimmed.contains("TODO") || trimmed.contains("FIXME") || trimmed.contains("HACK") {
                markers.push(TodoMarker {
                    path: relative.clone(),
                    line: index + 1,
                    text: truncated_excerpt(trimmed, EXCERPT_CHARS),
                });
            }
        }
    }

    (files, markers)
}

/// 外部检索的选择建议（设计 §5.4：时效性事实→tavily，仓库文档→deepwiki）。
pub fn preferred_external_server(query: &str) -> &'static str {
    let lower = query.to_lowercase();
    if lower.contains("github") || lower.contains("文档") || lower.contains("api") {
        "deepwiki"
    } else {
        "tavily"
    }
}

/// 外部证据摘录进 prompt 前的统一截断（≤500 字符）。
pub fn external_excerpt(text: &str) -> String {
    truncated_excerpt(text, 500)
}

/// 工作区路径的便捷包装（供 controller 记录步骤详情）。
pub fn latest_workspace_path() -> anyhow::Result<Option<PathBuf>> {
    let store = IterationStore::new();
    let mut iterations = store.load_all().unwrap_or_default();
    iterations.sort_by_key(|i| i.number);
    Ok(iterations
        .last()
        .map(|i| store.workspace_path(&i.id))
        .transpose()?
        .filter(|p| p.exists()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::PoisonError;
    use tempfile::TempDir;

    fn ctx_with_many_items(count: usize) -> ResearchContext {
        ResearchContext {
            project_name: "demo".to_string(),
            known_issues: (0..count)
                .map(|i| KnownIssue {
                    iteration_id: "iter-1".to_string(),
                    issue: format!("问题 {}", i),
                })
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn prompt_budget_truncates_and_marks_omissions() {
        let ctx = ctx_with_many_items(50);
        let prompt = ctx.to_prompt_with_budget(200);
        assert!(prompt.chars().count() <= 220, "实际长度 {}", prompt.chars().count());
        assert!(prompt.contains("从略"), "超预算必须标注省略: {}", prompt);

        let full = ctx_with_many_items(2).to_prompt_with_budget(10_000);
        assert!(full.contains("问题 0"));
        assert!(full.contains("问题 1"));
        assert!(!full.contains("从略"));
    }

    #[test]
    fn digest_changes_with_evidence() {
        let a = ctx_with_many_items(1).evidence_digest();
        let b = ctx_with_many_items(2).evidence_digest();
        assert_ne!(a, b, "证据变化必须改变 digest（增量调研依据）");
        assert_eq!(a, ctx_with_many_items(1).evidence_digest());
    }

    #[test]
    fn local_evidence_detection() {
        assert!(!ResearchContext::default().has_local_evidence());
        assert!(ctx_with_many_items(1).has_local_evidence());
    }

    #[test]
    fn scan_workspace_collects_todo_markers_with_limits() {
        let temp = TempDir::new().expect("temp dir");
        let root = temp.path();
        std::fs::create_dir_all(root.join("src")).expect("mkdir");
        std::fs::write(
            root.join("src").join("main.rs"),
            "fn main() {}\n// TODO: 补上重试幂等\n// FIXME: 错误处理\n",
        )
        .expect("write rs");
        std::fs::write(root.join("src").join("notes.txt"), "TODO: 不应被扫描").expect("write txt");

        let (files, markers) = scan_workspace(root);
        assert_eq!(files, vec!["src/main.rs".to_string()]);
        assert_eq!(markers.len(), 2);
        assert_eq!(markers[0].path, "src/main.rs");
        assert_eq!(markers[0].line, 2);
        assert!(markers[0].text.contains("TODO"));
        assert!(markers[1].text.contains("FIXME"));
    }

    #[test]
    fn external_server_selection() {
        assert_eq!(preferred_external_server("某个库的最新版本"), "tavily");
        assert_eq!(preferred_external_server("这个 GitHub 仓库的 API 文档"), "deepwiki");
        assert_eq!(external_excerpt(&"汉".repeat(600)).chars().count(), 500 + 6);
    }

    #[test]
    fn workspace_global_iteration_id_is_restored_after_research() {
        let _t = crate::persistence::WORKSPACE_TEST_LOCK
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let temp = TempDir::new().expect("temp dir");
        crate::persistence::set_workspace_path(temp.path().to_path_buf());
        crate::persistence::init_project_structure("demo").expect("init");
        ProjectStore::new()
            .save(&crate::domain::Project::new("demo"))
            .expect("save project");

        crate::persistence::set_iteration_id("iter-original".to_string());
        let config = AutonomyResearchConfig {
            local: true,
            memory: true,
            feedback: true,
            external_mcp: super::super::config::ExternalMcpPolicy::AsNeeded,
        };
        let ctx = collect_research(&config, &[], None).expect("collect");
        assert_eq!(ctx.project_name, "demo");
        assert_eq!(
            crate::persistence::get_iteration_id().as_deref(),
            Some("iter-original"),
            "调研不得污染全局当前迭代 ID"
        );
    }
}
