// Idea 聚合 —— 自主规划产出的迭代提案（设计 §5.2 / §9）
//
// Idea 有"提议 → 决策 → 转迭代"的长生命周期与可追溯决策记录，
// 与被消费即销毁的 `Feedback` 语义不同，因此是独立的领域聚合。
//
// schema 一次定型：字段按语义必需定义，不写兼容垫片。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::data::models::Priority;

/// 影响范围。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdeaScope {
    /// 新增功能。
    Feature,
    /// 重构。
    Refactor,
    /// 修复。
    Fix,
    /// 文档。
    Docs,
    /// 调研。
    Research,
}

impl IdeaScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Feature => "feature",
            Self::Refactor => "refactor",
            Self::Fix => "fix",
            Self::Docs => "docs",
            Self::Research => "research",
        }
    }

    /// 低风险范围（设计 §8-6：自动执行仅对文档/调研类开放，且默认关）。
    pub fn is_low_risk(&self) -> bool {
        matches!(self, Self::Docs | Self::Research)
    }
}

/// 预估工作量档位（不造精确人日假象）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effort {
    S,
    M,
    L,
    Xl,
}

impl Effort {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::S => "s",
            Self::M => "m",
            Self::L => "l",
            Self::Xl => "xl",
        }
    }
}

/// Idea 生命周期状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdeaStatus {
    /// 草稿（未通过质量门槛或超出每轮发布上限，留在池中）。
    Draft,
    /// 已发布，等待人决策。
    Proposed,
    /// 已采纳。
    Accepted,
    /// 已驳回。
    Rejected,
    /// 已改写（仅保存修改，未转迭代）。
    Rewritten,
    /// 已置后。
    Deferred,
    /// 被更好的 idea 取代。
    Superseded,
    /// 已转为迭代。
    Converted,
}

impl IdeaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Proposed => "proposed",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Rewritten => "rewritten",
            Self::Deferred => "deferred",
            Self::Superseded => "superseded",
            Self::Converted => "converted",
        }
    }

    /// 状态抑制（设计 §5.3）：这些状态下的 idea 同证据不再重提。
    pub fn is_suppressing(&self) -> bool {
        matches!(
            self,
            Self::Rejected | Self::Accepted | Self::Converted | Self::Deferred | Self::Superseded
        )
    }

    /// 仍在主列表中、可被人决策。
    pub fn is_open(&self) -> bool {
        matches!(self, Self::Proposed | Self::Rewritten)
    }
}

/// Idea 来源。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdeaSource {
    /// 调研发现。
    Research,
    /// 用户主动输入。
    UserInput,
    /// PM 推导。
    PmDerived,
}

impl IdeaSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Research => "research",
            Self::UserInput => "user_input",
            Self::PmDerived => "pm_derived",
        }
    }
}

/// 证据可信度（设计 §5.4）：本地文件可复现=高，记忆/反馈=中，外部不可复现=中低。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Credibility {
    Low,
    Mid,
    High,
}

impl Credibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Mid => "mid",
            Self::High => "high",
        }
    }
}

/// 证据来源（设计 §5.2）：保证"证据可回放"，excerpt 为原文摘录而非 LLM 转述。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Evidence {
    FileRef {
        path: String,
        line_range: Option<String>,
        excerpt: String,
    },
    IterationRef {
        id: String,
        excerpt: String,
    },
    FeedbackRef {
        stage: String,
        timestamp: DateTime<Utc>,
        excerpt: String,
    },
    ExternalRef {
        url: String,
        /// 外部来源标识，如 `tavily` / `deepwiki`。
        source: String,
        accessed_at: DateTime<Utc>,
        excerpt: String,
    },
}

impl Evidence {
    /// 原文摘录（进 prompt 前需截断，见 `truncated_excerpt`）。
    pub fn excerpt(&self) -> &str {
        match self {
            Self::FileRef { excerpt, .. }
            | Self::IterationRef { excerpt, .. }
            | Self::FeedbackRef { excerpt, .. }
            | Self::ExternalRef { excerpt, .. } => excerpt,
        }
    }

    /// 是否为本地证据（文件/迭代/反馈）。外部 URL 不算本地证据（设计 §5.3）。
    pub fn is_local(&self) -> bool {
        !matches!(self, Self::ExternalRef { .. })
    }

    pub fn credibility(&self) -> Credibility {
        match self {
            Self::FileRef { .. } => Credibility::High,
            Self::IterationRef { .. } | Self::FeedbackRef { .. } => Credibility::Mid,
            Self::ExternalRef { .. } => Credibility::Low,
        }
    }

    /// 用于"证据集合是否发生变化"的比对待键。
    pub fn identity_key(&self) -> String {
        match self {
            Self::FileRef { path, line_range, .. } => {
                format!("file:{}:{}", path, line_range.clone().unwrap_or_default())
            }
            Self::IterationRef { id, .. } => format!("iteration:{}", id),
            Self::FeedbackRef { stage, timestamp, .. } => {
                format!("feedback:{}:{}", stage, timestamp.timestamp())
            }
            Self::ExternalRef { url, .. } => format!("external:{}", url),
        }
    }

    /// 展示用定位文本（文件路径/迭代号/阶段/URL）。
    pub fn locator(&self) -> String {
        match self {
            Self::FileRef { path, line_range, .. } => match line_range {
                Some(range) if !range.is_empty() => format!("{}:{}", path, range),
                _ => path.clone(),
            },
            Self::IterationRef { id, .. } => id.clone(),
            Self::FeedbackRef { stage, .. } => format!("反馈 · {}", stage),
            Self::ExternalRef { url, .. } => url.clone(),
        }
    }

    pub fn external_source(&self) -> Option<&str> {
        match self {
            Self::ExternalRef { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// 改写内容（设计 §5.2 `IdeaRewrite`）。同一形状也用于保存改写前原文。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdeaRewrite {
    pub title: String,
    pub problem: String,
    pub hypothesis: String,
    pub scope: IdeaScope,
}

/// 决策动作。
///
/// 前四类（Accept/Reject/Rewrite/Defer）是**人工**决策门的四个动作；
/// `Supersede` 是 Agent 侧的提案维护动作（证据变化导致原提案过时），
/// 不进入人工决策面板。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionAction {
    Accept,
    Reject,
    Rewrite,
    Defer,
    Supersede,
}

impl DecisionAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Accept => "accept",
            Self::Reject => "reject",
            Self::Rewrite => "rewrite",
            Self::Defer => "defer",
            Self::Supersede => "supersede",
        }
    }

    /// 是否为人工决策门的四类动作之一。
    pub fn is_human_action(&self) -> bool {
        matches!(
            self,
            Self::Accept | Self::Reject | Self::Rewrite | Self::Defer
        )
    }
}

/// 决策审计（设计 §5.2）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdeaDecision {
    pub action: DecisionAction,
    /// 决策人，当前恒为 `"user"`（预留 `"auto"`）。
    pub by: String,
    pub comment: String,
    /// 改写后的内容（仅 Rewrite / 改写并采纳时存在）。
    pub rewritten: Option<IdeaRewrite>,
    pub decided_at: DateTime<Utc>,
    pub deferred_until: Option<DateTime<Utc>>,
}

/// Idea 聚合本体。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Idea {
    pub id: String,
    pub title: String,
    /// 解决什么问题（现状痛点，一句话说人话）。
    pub problem: String,
    /// 做了之后会怎样（可验证假设）。
    pub hypothesis: String,
    pub scope: IdeaScope,
    pub evidence: Vec<Evidence>,
    pub estimated_effort: Effort,
    pub priority: Priority,
    pub status: IdeaStatus,
    pub source: IdeaSource,
    /// 去重指纹（title + problem 规范化哈希）。
    pub fingerprint: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// 最近一次决策（完整历史见 `autonomy/decisions.log.jsonl`）。
    pub decision: Option<IdeaDecision>,
    /// 采纳后关联的迭代。
    pub derived_iteration_id: Option<String>,
    /// 被哪条 idea 取代。
    pub superseded_by: Option<String>,
    /// 预期收益（一句话；证据不足时写"不确定"，不编造收益）。
    pub expected_benefit: String,
    /// 与本条冲突的其他 idea id（同一模块两种改法，界面提示二选一）。
    pub conflicts_with: Vec<String>,
    /// 改写前的原文留底。
    pub original: Option<IdeaRewrite>,
}

/// LLM/用户输入侧的提案形状：不含 id、状态、指纹与时间戳。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdeaDraft {
    pub title: String,
    pub problem: String,
    pub hypothesis: String,
    pub scope: IdeaScope,
    pub evidence: Vec<Evidence>,
    pub estimated_effort: Effort,
    pub priority: Priority,
    pub expected_benefit: String,
    pub conflicts_with: Vec<String>,
}

impl IdeaDraft {
    pub fn into_idea(self, id: String, source: IdeaSource) -> Idea {
        let now = Utc::now();
        let fingerprint = compute_fingerprint(&self.title, &self.problem);
        Idea {
            id,
            title: self.title,
            problem: self.problem,
            hypothesis: self.hypothesis,
            scope: self.scope,
            evidence: self.evidence,
            estimated_effort: self.estimated_effort,
            priority: self.priority,
            status: IdeaStatus::Draft,
            source,
            fingerprint,
            created_at: now,
            updated_at: now,
            decision: None,
            derived_iteration_id: None,
            superseded_by: None,
            expected_benefit: self.expected_benefit,
            conflicts_with: self.conflicts_with,
            original: None,
        }
    }
}

impl Idea {
    /// 证据集合指纹（判断"证据是否变化"，变化时才允许重提被驳回的 idea）。
    pub fn evidence_key(&self) -> String {
        let mut keys: Vec<String> = self.evidence.iter().map(|e| e.identity_key()).collect();
        keys.sort();
        keys.join("|")
    }

    /// 本地证据条数。
    pub fn local_evidence_count(&self) -> usize {
        self.evidence.iter().filter(|e| e.is_local()).count()
    }

    /// 整条 idea 的可信度 = 其证据中的最高档。
    pub fn credibility(&self) -> Credibility {
        self.evidence
            .iter()
            .map(|e| e.credibility())
            .max()
            .unwrap_or(Credibility::Low)
    }

    /// 外部佐证是否缺失（没有任何外部证据时用于界面标注）。
    pub fn lacks_external_evidence(&self) -> bool {
        !self
            .evidence
            .iter()
            .any(|e| matches!(e, Evidence::ExternalRef { .. }))
    }

    /// 质量门槛（设计 §5.3）。返回全部不达标原因，空向量表示通过。
    ///
    /// ① 有明确 problem 与 ≥1 条本地证据（外部 URL 不算唯一证据）
    /// ② hypothesis 可验证（非空且含结果指向）
    /// ③ scope 不越权（不提出绕过安全边界的需求）
    /// ④ estimated_effort 由类型保证必填
    pub fn quality_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();

        if self.title.trim().is_empty() {
            issues.push("标题不能为空".to_string());
        }
        if self.problem.trim().is_empty() {
            issues.push("problem 不能为空：必须说清现状痛点".to_string());
        }
        if self.hypothesis.trim().is_empty() {
            issues.push("hypothesis 不能为空：必须是可验证的假设".to_string());
        }
        if self.local_evidence_count() == 0 {
            issues.push(
                "至少需要 1 条本地证据（文件/迭代/反馈），外部 URL 不能作为唯一证据".to_string(),
            );
        }

        let haystack = format!(
            "{} {} {}",
            self.title, self.problem, self.hypothesis
        )
        .to_lowercase();
        const FORBIDDEN: [&str; 8] = [
            "绕过限流",
            "关闭限流",
            "bypass rate limit",
            "disable rate limit",
            "绕过路径校验",
            "跳过路径校验",
            "disable sandbox",
            "rm -rf",
        ];
        for needle in FORBIDDEN {
            if haystack.contains(needle) {
                issues.push(format!("scope 越权：提案涉及绕过安全边界（命中 \"{}\"）", needle));
                break;
            }
        }

        issues
    }

    pub fn to_summary(&self) -> IdeaSummary {
        IdeaSummary {
            id: self.id.clone(),
            title: self.title.clone(),
            problem: self.problem.clone(),
            hypothesis: self.hypothesis.clone(),
            scope: self.scope,
            estimated_effort: self.estimated_effort,
            priority: self.priority,
            status: self.status,
            source: self.source,
            credibility: self.credibility(),
            evidence_count: self.evidence.len(),
            // 首条证据的定位文本（卡片上展示"证据在哪"用）
            evidence_hint: self.evidence.first().map(|e| e.locator()),
            expected_benefit: self.expected_benefit.clone(),
            derived_iteration_id: self.derived_iteration_id.clone(),
            deferred_until: self.decision.as_ref().and_then(|d| d.deferred_until),
            conflicts_with: self.conflicts_with.clone(),
            updated_at: self.updated_at,
        }
    }

    /// 改写：保留原文，更新内容，重算指纹。
    pub fn apply_rewrite(&mut self, rewrite: IdeaRewrite) {
        self.original = Some(IdeaRewrite {
            title: self.title.clone(),
            problem: self.problem.clone(),
            hypothesis: self.hypothesis.clone(),
            scope: self.scope,
        });
        self.title = rewrite.title;
        self.problem = rewrite.problem;
        self.hypothesis = rewrite.hypothesis;
        self.scope = rewrite.scope;
        self.fingerprint = compute_fingerprint(&self.title, &self.problem);
        self.updated_at = Utc::now();
    }
}

/// 列表行/卡片用的轻量视图。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdeaSummary {
    pub id: String,
    pub title: String,
    pub problem: String,
    pub hypothesis: String,
    pub scope: IdeaScope,
    pub estimated_effort: Effort,
    pub priority: Priority,
    pub status: IdeaStatus,
    pub source: IdeaSource,
    pub credibility: Credibility,
    pub evidence_count: usize,
    /// 首条证据的定位文本（文件路径/迭代号/反馈阶段/URL）。
    pub evidence_hint: Option<String>,
    pub expected_benefit: String,
    pub derived_iteration_id: Option<String>,
    pub deferred_until: Option<DateTime<Utc>>,
    pub conflicts_with: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

/// 规范化文本：小写、去空白与标点，只保留字母数字与 CJK 字符。
fn normalize_for_fingerprint(text: &str) -> String {
    text.chars()
        .flat_map(|c| c.to_lowercase())
        .filter(|c| c.is_alphanumeric())
        .collect()
}

/// FNV-1a 64 位哈希（自实现，保证跨进程/跨版本确定，用于去重指纹）。
fn fnv1a_64(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

/// 去重指纹：title + problem 规范化后的稳定哈希。
pub fn compute_fingerprint(title: &str, problem: &str) -> String {
    let normalized = format!(
        "{}|{}",
        normalize_for_fingerprint(title),
        normalize_for_fingerprint(problem)
    );
    format!("{:016x}", fnv1a_64(normalized.as_bytes()))
}

/// 进 prompt 前的证据摘录截断（设计 §6.3：单条 excerpt ≤ 500 字符，UTF-8 安全）。
pub fn truncated_excerpt(text: &str, max_chars: usize) -> String {
    if text.chars().count() > max_chars {
        format!(
            "{}…[已截断]",
            text.chars().take(max_chars).collect::<String>()
        )
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn local_evidence() -> Evidence {
        Evidence::FileRef {
            path: "src/main.rs".to_string(),
            line_range: Some("10-20".to_string()),
            excerpt: "fn main() { retry(); }".to_string(),
        }
    }

    fn external_evidence() -> Evidence {
        Evidence::ExternalRef {
            url: "https://example.com/advisory".to_string(),
            source: "tavily".to_string(),
            accessed_at: Utc::now(),
            excerpt: "security advisory".to_string(),
        }
    }

    fn draft() -> IdeaDraft {
        IdeaDraft {
            title: "补上交付重试的幂等判断".to_string(),
            problem: "交付失败后重试会重复生成交付文档".to_string(),
            hypothesis: "补上幂等判断后，重试将只补缺口".to_string(),
            scope: IdeaScope::Fix,
            evidence: vec![local_evidence()],
            estimated_effort: Effort::M,
            priority: Priority::High,
            expected_benefit: "重试不再产生重复文档".to_string(),
            conflicts_with: vec![],
        }
    }

    #[test]
    fn fingerprint_is_stable_and_ignores_formatting() {
        let a = compute_fingerprint("补上重试幂等", "重试会重复生成文档");
        let b = compute_fingerprint("补上重试幂等", "重试会重复生成文档");
        assert_eq!(a, b);

        // 大小写与空白差异不应改变指纹
        let c = compute_fingerprint("Retry  Idempotency", "Dup   Docs");
        let d = compute_fingerprint("retry idempotency", "dup docs");
        assert_eq!(c, d);

        let different = compute_fingerprint("另一个标题", "另一个问题");
        assert_ne!(a, different);
    }

    #[test]
    fn quality_gate_requires_local_evidence() {
        let mut idea = draft().into_idea("idea-1".to_string(), IdeaSource::Research);
        assert!(idea.quality_issues().is_empty(), "{:?}", idea.quality_issues());

        idea.evidence = vec![external_evidence()];
        let issues = idea.quality_issues();
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("本地证据"), "{}", issues[0]);
    }

    #[test]
    fn quality_gate_rejects_security_bypass() {
        let mut input = draft();
        input.hypothesis = "绕过限流后即可无限调用".to_string();
        let idea = input.into_idea("idea-2".to_string(), IdeaSource::Research);
        assert!(
            idea.quality_issues().iter().any(|i| i.contains("越权")),
            "{:?}",
            idea.quality_issues()
        );
    }

    #[test]
    fn quality_gate_rejects_empty_problem_and_hypothesis() {
        let mut input = draft();
        input.problem = "  ".to_string();
        input.hypothesis = String::new();
        let idea = input.into_idea("idea-3".to_string(), IdeaSource::Research);
        let issues = idea.quality_issues();
        assert_eq!(issues.len(), 2, "{:?}", issues);
    }

    #[test]
    fn credibility_is_highest_of_evidence() {
        let mut idea = draft().into_idea("idea-4".to_string(), IdeaSource::Research);
        assert_eq!(idea.credibility(), Credibility::High);

        idea.evidence = vec![external_evidence()];
        assert_eq!(idea.credibility(), Credibility::Low);
        assert!(idea.lacks_external_evidence() == false);

        idea.evidence.push(local_evidence());
        assert_eq!(idea.credibility(), Credibility::High);
    }

    #[test]
    fn rewrite_preserves_original_and_refreshes_fingerprint() {
        let mut idea = draft().into_idea("idea-5".to_string(), IdeaSource::Research);
        let before = idea.fingerprint.clone();
        let original_title = idea.title.clone();

        idea.apply_rewrite(IdeaRewrite {
            title: "改成只修复文档写入".to_string(),
            problem: "文档写入缺少存在性检查".to_string(),
            hypothesis: "补检查后可避免覆盖".to_string(),
            scope: IdeaScope::Fix,
        });

        assert_ne!(idea.fingerprint, before);
        assert_eq!(idea.original.as_ref().expect("original").title, original_title);
        assert_eq!(idea.title, "改成只修复文档写入");
    }

    #[test]
    fn status_suppression_matches_design() {
        assert!(IdeaStatus::Rejected.is_suppressing());
        assert!(IdeaStatus::Converted.is_suppressing());
        assert!(IdeaStatus::Deferred.is_suppressing());
        assert!(!IdeaStatus::Proposed.is_suppressing());
        assert!(IdeaStatus::Proposed.is_open());
        assert!(IdeaStatus::Rewritten.is_open());
        assert!(!IdeaStatus::Converted.is_open());
    }

    #[test]
    fn low_risk_scope_is_docs_and_research_only() {
        assert!(IdeaScope::Docs.is_low_risk());
        assert!(IdeaScope::Research.is_low_risk());
        assert!(!IdeaScope::Feature.is_low_risk());
        assert!(!IdeaScope::Fix.is_low_risk());
    }

    #[test]
    fn idea_serde_round_trip() {
        let mut idea = draft().into_idea("idea-6".to_string(), IdeaSource::Research);
        idea.evidence.push(external_evidence());
        idea.decision = Some(IdeaDecision {
            action: DecisionAction::Defer,
            by: "user".to_string(),
            comment: "下周再说".to_string(),
            rewritten: None,
            decided_at: Utc::now(),
            deferred_until: Some(Utc::now()),
        });

        let json = serde_json::to_string_pretty(&idea).expect("serialize");
        let back: Idea = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, idea);
        assert!(json.contains("\"kind\": \"file_ref\""));
        assert!(json.contains("\"kind\": \"external_ref\""));
        assert!(json.contains("\"status\": \"draft\""));
    }

    #[test]
    fn excerpt_truncation_is_utf8_safe() {
        let long = "汉".repeat(600);
        let truncated = truncated_excerpt(&long, 500);
        assert_eq!(truncated.chars().count(), 500 + "…[已截断]".chars().count());
        assert_eq!(truncated_excerpt("短文本", 500), "短文本");
    }
}
