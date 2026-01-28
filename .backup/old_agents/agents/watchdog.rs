/// WatchDog Agent - 监控执行 Agent 的行为，防止跑偏
/// 
/// 核心功能：
/// 1. 每隔 N 次工具调用，提醒 Agent 当前目标
/// 2. 记录检查次数和历史
/// 3. 生成目标提醒消息
pub struct WatchDogAgent {
    /// 用户原始需求
    original_requirements: String,
    
    /// 当前阶段目标
    current_objective: String,
    
    /// 检查间隔（每 N 次工具调用检查一次）
    check_interval: usize,
    
    /// 已检查次数
    check_count: usize,
}

impl WatchDogAgent {
    /// 创建新的 WatchDog Agent
    /// 
    /// # 参数
    /// - `original_requirements`: 用户的原始需求描述
    /// - `current_objective`: 当前阶段的具体目标
    /// - `check_interval`: 每隔多少次工具调用进行一次检查
    pub fn new(original_requirements: String, current_objective: String, check_interval: usize) -> Self {
        tracing::info!(
            "WatchDog initialized: interval={}, objective={}",
            check_interval,
            &current_objective
        );
        
        Self {
            original_requirements,
            current_objective,
            check_interval,
            check_count: 0,
        }
    }
    
    /// 检查是否需要注入提醒
    /// 
    /// # 参数
    /// - `tool_call_count`: 当前工具调用总次数
    /// 
    /// # 返回
    /// 如果需要提醒返回 true
    pub fn should_remind(&self, tool_call_count: usize) -> bool {
        tool_call_count > 0 && tool_call_count % self.check_interval == 0
    }
    
    /// 生成提醒消息
    /// 
    /// # 返回
    /// 格式化的提醒消息，包含原始需求和当前目标
    pub fn generate_reminder(&mut self) -> String {
        self.check_count += 1;
        
        let reminder = format!(
            r#"⚠️  **WatchDog 提醒 #{count}**

📋 **原始需求**:
{requirements}

🎯 **当前目标**:
{objective}

❓ **自检问题**:
1. 你当前的行为是否偏离了原始需求？
2. 你是否在做不必要的工作？
3. 你是否遗漏了关键需求？

✅ **继续执行**，但请保持专注于目标。"#,
            count = self.check_count,
            requirements = self.original_requirements,
            objective = self.current_objective
        );
        
        tracing::info!("WatchDog reminder generated (#{}))", self.check_count);
        
        reminder
    }
    
    /// 更新当前目标
    /// 
    /// 用于在执行过程中切换不同的子任务目标
    /// 
    /// # 参数
    /// - `new_objective`: 新的目标描述
    pub fn update_objective(&mut self, new_objective: String) {
        tracing::info!(
            "WatchDog objective updated: {} -> {}",
            &self.current_objective,
            &new_objective
        );
        self.current_objective = new_objective;
    }
    
    /// 重置检查计数器
    /// 
    /// 用于在开始新的阶段时重置统计
    pub fn reset_check_count(&mut self) {
        tracing::info!("WatchDog check count reset (was: {})", self.check_count);
        self.check_count = 0;
    }
    
    /// 获取统计信息
    pub fn stats(&self) -> WatchDogStats {
        WatchDogStats {
            check_count: self.check_count,
            check_interval: self.check_interval,
        }
    }
}

/// WatchDog 统计信息
#[derive(Debug, Clone)]
pub struct WatchDogStats {
    pub check_count: usize,
    pub check_interval: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_watchdog_should_remind() {
        let watchdog = WatchDogAgent::new(
            "Build a web app".to_string(),
            "Generate HTML files".to_string(),
            3
        );
        
        assert!(!watchdog.should_remind(0));
        assert!(!watchdog.should_remind(1));
        assert!(!watchdog.should_remind(2));
        assert!(watchdog.should_remind(3));
        assert!(!watchdog.should_remind(4));
        assert!(!watchdog.should_remind(5));
        assert!(watchdog.should_remind(6));
    }
    
    #[test]
    fn test_watchdog_generate_reminder() {
        let mut watchdog = WatchDogAgent::new(
            "Build a web app".to_string(),
            "Generate HTML files".to_string(),
            3
        );
        
        let reminder1 = watchdog.generate_reminder();
        assert!(reminder1.contains("WatchDog 提醒 #1"));
        assert!(reminder1.contains("Build a web app"));
        assert!(reminder1.contains("Generate HTML files"));
        
        let reminder2 = watchdog.generate_reminder();
        assert!(reminder2.contains("WatchDog 提醒 #2"));
    }
    
    #[test]
    fn test_watchdog_update_objective() {
        let mut watchdog = WatchDogAgent::new(
            "Build a web app".to_string(),
            "Generate HTML files".to_string(),
            3
        );
        
        watchdog.update_objective("Generate CSS files".to_string());
        
        let reminder = watchdog.generate_reminder();
        assert!(reminder.contains("Generate CSS files"));
        assert!(!reminder.contains("Generate HTML files"));
    }
}
