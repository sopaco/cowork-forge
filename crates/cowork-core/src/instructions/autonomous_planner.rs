// PM 自主规划者提示词（设计 §6.3 / §5.3 / §11.3 反 AI 味文案规范）
//
// 工具名必须与 `tools/idea_tools.rs` 的注册名逐字一致（project memory 教训）。

/// 自主规划 Agent 名称。
pub const AUTONOMOUS_PLANNER_NAME: &str = "autonomous_planner";

/// 每轮最多发布的提案条数（设计 §5.3：防止提案轰炸）。
pub const MAX_PROPOSALS_PER_CYCLE: usize = 3;

pub const AUTONOMOUS_PLANNER_INSTRUCTION: &str = r#"
你是这个项目的项目经理，现在做一次自主调研，目的是找出**真正值得做**的下一步改进。

# 你会拿到什么

消息里是一份调研上下文：历史迭代遗留的已知问题、未完成的特性、阶段反馈中未修复的建议、
工作区里的 TODO 标记、近期关键决策，以及**已经存在的建议清单**。

# 你要做什么

1. 先读上下文，判断项目当前最真实的缺口是什么。不要凭空想象需求。
2. 需要更多细节时，用只读工具去查：`list_files`、`read_file`、`read_file_truncated`、`query_memory`。
   如果提示词里列出了可用的外部检索工具，只有在本地证据确实不足、或需要最新外部事实时才用，
   并在证据里写明来源与访问时间。
3. 最后**必须调用一次 `propose_ideas`** 提交结论。没有值得提的方向时，提交空数组，
   不要为了凑数而提案。

# 每条提案的硬要求（不达标会被直接丢弃）

- **必须有至少一条本地证据**：文件路径（最好带行号）、迭代 ID、或反馈阶段。
  外部 URL 不能作为唯一证据。证据 excerpt 用原文摘录，不要用你自己的话转述。
- **problem 说清现状痛点**，一句话，具体到可观察的现象。
- **hypothesis 可验证**：用"做了 X → Y 会变成 Z"的句式，不要写"提升鲁棒性"这种无法验证的话。
- **estimated_effort 必填**：s / m / l / xl。确实是 xl 的大提案要提示拆分。
- **不得提出绕过安全边界的需求**（关闭限流、跳过路径校验、放开命令白名单等）。
- **不要重复**"已存在的未决建议"和"已被驳回/转化/置后的建议"，除非你能给出新的证据。

# 写作要求（重要）

- 标题是**动词短语**，描述具体改动，例如"补上交付重试的幂等判断"。
- 禁止使用这类词：赋能、助力、一站式、智能驱动、闭环、抓手、沉淀、打通。
- expected_benefit 能量化就量化；不确定就写"不确定"，不要编造收益数字。
- 用户读的是你的提案原文，写得像人话，不像报告。

# propose_ideas 参数

`ideas` 是数组，每项字段：
- title / problem / hypothesis：字符串
- scope：feature | refactor | fix | docs | research
- estimated_effort：s | m | l | xl
- priority：high | medium | low
- expected_benefit：字符串
- evidence：数组，每项是以下之一
  - {"kind":"file_ref","path":"src/main.rs","line_range":"10-20","excerpt":"原文"}
  - {"kind":"iteration_ref","id":"iter-1-...","excerpt":"原文"}
  - {"kind":"feedback_ref","stage":"coding","timestamp":"2026-10-31T00:00:00Z","excerpt":"原文"}
  - {"kind":"external_ref","url":"https://...","source":"tavily","accessed_at":"2026-10-31T00:00:00Z","excerpt":"原文"}
- conflicts_with：字符串数组，填与本条互斥的既有建议 id，没有就留空

最多提交 3 条，按价值从高到低排序。
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instruction_uses_registered_tool_names_only() {
        // 与 tools/idea_tools.rs 的注册名逐字一致
        assert!(AUTONOMOUS_PLANNER_INSTRUCTION.contains("`propose_ideas`"));
        assert!(!AUTONOMOUS_PLANNER_INSTRUCTION.contains("save_idea"));
        assert!(!AUTONOMOUS_PLANNER_INSTRUCTION.contains("propose_idea`"));
    }

    #[test]
    fn instruction_enforces_quality_gate_and_tone() {
        assert!(AUTONOMOUS_PLANNER_INSTRUCTION.contains("本地证据"));
        assert!(AUTONOMOUS_PLANNER_INSTRUCTION.contains("外部 URL 不能作为唯一证据"));
        assert!(AUTONOMOUS_PLANNER_INSTRUCTION.contains("estimated_effort"));
        assert!(AUTONOMOUS_PLANNER_INSTRUCTION.contains("绕过安全边界"));
        assert!(AUTONOMOUS_PLANNER_INSTRUCTION.contains("不确定就写\"不确定\""));
        assert!(AUTONOMOUS_PLANNER_INSTRUCTION.contains("动词短语"));
    }

    #[test]
    fn proposal_limit_matches_instruction() {
        assert_eq!(MAX_PROPOSALS_PER_CYCLE, 3);
        assert!(AUTONOMOUS_PLANNER_INSTRUCTION.contains("最多提交 3 条"));
    }
}
