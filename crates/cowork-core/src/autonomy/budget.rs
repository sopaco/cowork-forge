// BudgetGuard —— 自主循环的 LLM/MCP 调用记账与限额（设计 §8-2 / §5.4）
//
// 叠加在全局限流（30 req/min 共享令牌桶）之上，不替代也不绕过限流：
// 限流管"速率"，本模块管"自主循环能吃多少"。超限则本轮跳过并记录，不报错。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::config::AutonomyBudget;

/// 单轮 cycle 的调用预算记账。
#[derive(Debug, Clone)]
pub struct BudgetGuard {
    llm_limit: u32,
    mcp_limit: u32,
    llm_used: u32,
    mcp_used: u32,
}

impl BudgetGuard {
    pub fn new(budget: &AutonomyBudget) -> Self {
        Self {
            llm_limit: budget.max_llm_calls_per_cycle,
            mcp_limit: budget.max_mcp_calls_per_cycle,
            llm_used: 0,
            mcp_used: 0,
        }
    }

    pub fn can_call_llm(&self) -> bool {
        self.llm_used < self.llm_limit
    }

    pub fn can_call_mcp(&self) -> bool {
        self.mcp_used < self.mcp_limit
    }

    /// 记账一次 LLM 调用；预算不足返回 false 且不记账。
    pub fn consume_llm(&mut self) -> bool {
        if !self.can_call_llm() {
            return false;
        }
        self.llm_used += 1;
        true
    }

    /// 记账一次 MCP 调用；预算不足返回 false 且不记账。
    pub fn consume_mcp(&mut self) -> bool {
        if !self.can_call_mcp() {
            return false;
        }
        self.mcp_used += 1;
        true
    }

    pub fn llm_used(&self) -> u32 {
        self.llm_used
    }

    pub fn mcp_used(&self) -> u32 {
        self.mcp_used
    }

    pub fn llm_limit(&self) -> u32 {
        self.llm_limit
    }

    pub fn mcp_limit(&self) -> u32 {
        self.mcp_limit
    }

    pub fn llm_remaining(&self) -> u32 {
        self.llm_limit.saturating_sub(self.llm_used)
    }
}

/// 每日周期预算（持久化于 `autonomy/budget.json`，跨天自动归零）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyBudget {
    /// 日期键（UTC `YYYY-MM-DD`）。
    pub day: String,
    pub cycles_used: u32,
}

impl DailyBudget {
    pub fn for_day(now: DateTime<Utc>) -> Self {
        Self {
            day: day_key(now),
            cycles_used: 0,
        }
    }

    /// 跨天则归零重置（设计 §8-5：单日预算超限次日恢复）。
    pub fn rollover(&mut self, now: DateTime<Utc>) {
        let today = day_key(now);
        if self.day != today {
            self.day = today;
            self.cycles_used = 0;
        }
    }

    pub fn is_exhausted(&self, limit: u32) -> bool {
        self.cycles_used >= limit
    }

    pub fn consume(&mut self) {
        self.cycles_used = self.cycles_used.saturating_add(1);
    }

    pub fn remaining(&self, limit: u32) -> u32 {
        limit.saturating_sub(self.cycles_used)
    }
}

pub fn day_key(now: DateTime<Utc>) -> String {
    now.format("%Y-%m-%d").to_string()
}

/// 面板用的预算总览。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BudgetStatus {
    pub llm_used: u32,
    pub llm_limit: u32,
    pub mcp_used: u32,
    pub mcp_limit: u32,
    pub daily_cycles_used: u32,
    pub daily_cycle_limit: u32,
    pub day: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn budget() -> AutonomyBudget {
        AutonomyBudget {
            max_llm_calls_per_cycle: 2,
            max_cycles_per_day: 3,
            max_mcp_calls_per_cycle: 1,
        }
    }

    #[test]
    fn llm_budget_stops_at_limit() {
        let mut guard = BudgetGuard::new(&budget());
        assert!(guard.can_call_llm());
        assert!(guard.consume_llm());
        assert!(guard.consume_llm());
        assert!(!guard.can_call_llm());
        assert!(!guard.consume_llm(), "超限不得记账");
        assert_eq!(guard.llm_used(), 2);
        assert_eq!(guard.llm_remaining(), 0);
    }

    #[test]
    fn mcp_budget_is_counted_independently() {
        let mut guard = BudgetGuard::new(&budget());
        assert!(guard.consume_mcp());
        assert!(!guard.consume_mcp());
        assert_eq!(guard.mcp_used(), 1);
        assert!(guard.can_call_llm(), "MCP 耗尽不应影响 LLM 预算");
        assert!(guard.consume_llm());
    }

    #[test]
    fn daily_budget_rolls_over_across_days() {
        let day1 = Utc.with_ymd_and_hms(2026, 10, 31, 23, 0, 0).unwrap();
        let mut daily = DailyBudget::for_day(day1);
        daily.consume();
        daily.consume();
        daily.consume();
        assert!(daily.is_exhausted(3));
        assert_eq!(daily.remaining(3), 0);

        // 同一天内不重置
        daily.rollover(day1);
        assert!(daily.is_exhausted(3));

        // 跨天自动归零恢复
        let day2 = Utc.with_ymd_and_hms(2026, 11, 1, 0, 0, 1).unwrap();
        daily.rollover(day2);
        assert_eq!(daily.day, "2026-11-01");
        assert_eq!(daily.cycles_used, 0);
        assert!(!daily.is_exhausted(3));
        assert_eq!(daily.remaining(3), 3);
    }
}
