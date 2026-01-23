use crate::artifacts::*;

/// TodoList 管理器 - 负责更新和追踪 TodoList 状态
pub struct TodoListManager;

impl TodoListManager {
    /// 根据执行结果更新 TodoList 状态
    pub fn update_from_execution(
        todo_list: &mut TodoList,
        _changes: &[Change],
        successful_files: &[String],
        failed_files: &[String],
    ) {
        for todo_item in &mut todo_list.items {
            // 检查这个 Todo 相关的文件是否都已成功生成
            let all_files_successful = todo_item.related_files.iter()
                .all(|file| successful_files.contains(file));
            
            let some_files_failed = todo_item.related_files.iter()
                .any(|file| failed_files.contains(file));
            
            // 根据文件生成情况更新状态
            if some_files_failed {
                todo_item.status = TodoStatus::Blocked {
                    reason: format!("Some related files failed to generate: {:?}", 
                        todo_item.related_files.iter()
                            .filter(|f| failed_files.contains(f))
                            .collect::<Vec<_>>())
                };
            } else if all_files_successful && !todo_item.related_files.is_empty() {
                // 所有相关文件都成功生成
                match &todo_item.status {
                    TodoStatus::Pending | TodoStatus::InProgress => {
                        todo_item.status = TodoStatus::Completed;
                    }
                    _ => {}  // 保持现有状态
                }
            } else if todo_item.related_files.iter().any(|file| successful_files.contains(file)) {
                // 部分文件生成成功
                match &todo_item.status {
                    TodoStatus::Pending => {
                        todo_item.status = TodoStatus::InProgress;
                    }
                    _ => {}
                }
            }
        }
    }
    
    /// 从 CheckReport 验证 TodoList 完成度
    pub fn verify_from_check(
        todo_list: &mut TodoList,
        check_report: &CheckReport,
    ) {
        // 构建失败文件列表
        let failed_files: Vec<String> = check_report.issues.iter()
            .filter(|issue| issue.sev == "error")
            .filter_map(|issue| {
                // 从 issue.id 提取文件路径
                if issue.id.starts_with("ISSUE-FILE-") {
                    Some(issue.id.strip_prefix("ISSUE-FILE-").unwrap_or("").to_string())
                } else if issue.id.starts_with("ISSUE-EMPTY-") {
                    Some(issue.id.strip_prefix("ISSUE-EMPTY-").unwrap_or("").to_string())
                } else {
                    None
                }
            })
            .collect();
        
        for todo_item in &mut todo_list.items {
            // 如果相关文件有验证失败，标记为 Blocked
            let has_failed_files = todo_item.related_files.iter()
                .any(|file| failed_files.contains(file));
            
            if has_failed_files {
                todo_item.status = TodoStatus::Blocked {
                    reason: format!("Verification failed for: {:?}",
                        todo_item.related_files.iter()
                            .filter(|f| failed_files.contains(f))
                            .collect::<Vec<_>>())
                };
            }
        }
    }
    
    /// 生成 TodoList 状态报告
    pub fn generate_status_report(todo_list: &TodoList) -> TodoStatusReport {
        let mut total = 0;
        let mut pending = 0;
        let mut in_progress = 0;
        let mut completed = 0;
        let mut blocked = 0;
        
        for item in &todo_list.items {
            total += 1;
            match &item.status {
                TodoStatus::Pending => pending += 1,
                TodoStatus::InProgress => in_progress += 1,
                TodoStatus::Completed => completed += 1,
                TodoStatus::Blocked { .. } => blocked += 1,
            }
        }
        
        let completion_percentage = if total > 0 {
            (completed as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        TodoStatusReport {
            total,
            pending,
            in_progress,
            completed,
            blocked,
            completion_percentage,
        }
    }
    
    /// 打印 TodoList 状态
    pub fn print_status(todo_list: &TodoList) {
        let report = Self::generate_status_report(todo_list);
        
        println!("\n╔═══════════════════════════════════════╗");
        println!("║   TodoList 状态                       ║");
        println!("╚═══════════════════════════════════════╝");
        println!("总任务数: {}", report.total);
        println!("✅ 已完成: {}", report.completed);
        println!("🔄 进行中: {}", report.in_progress);
        println!("⏳ 待开始: {}", report.pending);
        println!("🚫 阻塞: {}", report.blocked);
        println!("完成度: {:.1}%", report.completion_percentage);
        println!();
        
        // 显示阻塞的任务
        if report.blocked > 0 {
            println!("⚠️  阻塞的任务:");
            for item in &todo_list.items {
                if let TodoStatus::Blocked { reason } = &item.status {
                    println!("  - {}: {}", item.id, item.description);
                    println!("    原因: {}", reason);
                }
            }
            println!();
        }
        
        // 显示已完成的任务
        if report.completed > 0 {
            println!("✅ 已完成的任务:");
            for item in &todo_list.items {
                if matches!(item.status, TodoStatus::Completed) {
                    println!("  - {}: {}", item.id, item.description);
                }
            }
            println!();
        }
    }
}

/// TodoList 状态报告
#[derive(Debug, Clone)]
pub struct TodoStatusReport {
    pub total: usize,
    pub pending: usize,
    pub in_progress: usize,
    pub completed: usize,
    pub blocked: usize,
    pub completion_percentage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_from_execution() {
        let mut todo_list = TodoList {
            items: vec![
                TodoItem {
                    id: "TODO-001".to_string(),
                    description: "Implement login".to_string(),
                    status: TodoStatus::Pending,
                    related_requirements: vec!["REQ-001".to_string()],
                    related_files: vec!["login.rs".to_string(), "session.rs".to_string()],
                    verification_method: "unit_test".to_string(),
                },
            ],
        };
        
        let successful_files = vec!["login.rs".to_string(), "session.rs".to_string()];
        let failed_files = vec![];
        
        TodoListManager::update_from_execution(
            &mut todo_list,
            &[],
            &successful_files,
            &failed_files,
        );
        
        assert!(matches!(todo_list.items[0].status, TodoStatus::Completed));
    }
    
    #[test]
    fn test_status_report() {
        let todo_list = TodoList {
            items: vec![
                TodoItem {
                    id: "TODO-001".to_string(),
                    description: "Task 1".to_string(),
                    status: TodoStatus::Completed,
                    related_requirements: vec![],
                    related_files: vec![],
                    verification_method: "test".to_string(),
                },
                TodoItem {
                    id: "TODO-002".to_string(),
                    description: "Task 2".to_string(),
                    status: TodoStatus::InProgress,
                    related_requirements: vec![],
                    related_files: vec![],
                    verification_method: "test".to_string(),
                },
                TodoItem {
                    id: "TODO-003".to_string(),
                    description: "Task 3".to_string(),
                    status: TodoStatus::Pending,
                    related_requirements: vec![],
                    related_files: vec![],
                    verification_method: "test".to_string(),
                },
            ],
        };
        
        let report = TodoListManager::generate_status_report(&todo_list);
        
        assert_eq!(report.total, 3);
        assert_eq!(report.completed, 1);
        assert_eq!(report.in_progress, 1);
        assert_eq!(report.pending, 1);
        assert!((report.completion_percentage - 33.333333333333336).abs() < 1e-9);
    }
}
