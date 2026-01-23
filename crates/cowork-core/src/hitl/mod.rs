use anyhow::Result;
use dialoguer::{Confirm, Input, Editor};
use serde::Serialize;
use std::fmt::Display;

/// Human-in-the-Loop 控制器
pub struct HitlController;

impl HitlController {
    pub fn new() -> Self {
        Self
    }

    /// 获取用户输入
    pub fn input(&self, prompt: &str) -> Result<String> {
        let input: String = Input::new()
            .with_prompt(prompt)
            .interact_text()?;
        Ok(input)
    }

    /// 确认（是/否）
    pub fn confirm(&self, prompt: &str) -> Result<bool> {
        let confirmed = Confirm::new()
            .with_prompt(prompt)
            .default(true)
            .interact()?;
        Ok(confirmed)
    }

    /// 让用户在编辑器中审查和修改 JSON 内容
    /// 
    /// 返回值：
    /// - Ok(Some(modified_json)) - 用户修改了内容
    /// - Ok(None) - 用户接受原内容
    /// - Err(_) - 发生错误
    pub fn review_and_edit_json<T>(&self, title: &str, data: &T) -> Result<Option<String>>
    where
        T: Serialize,
    {
        println!("\n📝 请审查 {} 的内容", title);
        
        // 转换为格式化的 JSON
        let json_str = serde_json::to_string_pretty(data)?;
        
        // 显示摘要
        let line_count = json_str.lines().count();
        println!("  内容预览（共 {} 行）：", line_count);
        println!("  ────────────────────────────────────────");
        for (i, line) in json_str.lines().take(10).enumerate() {
            println!("  {}: {}", i + 1, line);
        }
        if line_count > 10 {
            println!("  ... ({} 行省略)", line_count - 10);
        }
        println!("  ────────────────────────────────────────\n");

        // 询问用户是否要编辑
        let should_edit = Confirm::new()
            .with_prompt("是否需要修改此内容？")
            .default(false)
            .interact()?;

        if !should_edit {
            return Ok(None);
        }

        // 打开编辑器
        println!("📝 打开编辑器...（保存并关闭编辑器以提交修改）");
        let edited = Editor::new()
            .require_save(true)
            .edit(&json_str)?;

        match edited {
            Some(text) if text.trim() != json_str.trim() => {
                // 验证 JSON 格式
                match serde_json::from_str::<serde_json::Value>(&text) {
                    Ok(_) => {
                        println!("✅ JSON 格式验证通过");
                        Ok(Some(text))
                    }
                    Err(e) => {
                        println!("❌ JSON 格式错误: {}", e);
                        let retry = Confirm::new()
                            .with_prompt("是否重新编辑？")
                            .default(true)
                            .interact()?;
                        
                        if retry {
                            self.review_and_edit_json(title, data)
                        } else {
                            println!("⚠️  放弃修改，使用原始内容");
                            Ok(None)
                        }
                    }
                }
            }
            _ => {
                println!("ℹ️  内容未修改");
                Ok(None)
            }
        }
    }

    /// 简化版：让用户确认内容并选择是否修改
    pub fn review<T>(&self, title: &str, data: &T) -> Result<bool>
    where
        T: Serialize + Display,
    {
        println!("\n┌─────────────────────────────────────────┐");
        println!("│ 审查: {}                            ", title);
        println!("└─────────────────────────────────────────┘");
        println!("{}", data);
        println!();

        let approved = Confirm::new()
            .with_prompt("是否接受此结果？")
            .default(true)
            .interact()?;

        Ok(approved)
    }

    /// 让用户提供反馈意见
    pub fn collect_feedback(&self, prompt: &str) -> Result<String> {
        println!("\n💬 {}", prompt);
        
        let feedback = Editor::new()
            .require_save(false)
            .edit("")?
            .unwrap_or_default();

        Ok(feedback.trim().to_string())
    }

    /// 让用户提供反馈意见（带默认值）
    pub fn collect_feedback_with_default(&self, prompt: &str, default: &str) -> Result<String> {
        println!("\n💬 {}", prompt);
        println!("(当前内容已预填充，可直接保存或修改)");
        
        let feedback = Editor::new()
            .require_save(false)
            .edit(default)?
            .unwrap_or_else(|| default.to_string());

        Ok(feedback.trim().to_string())
    }

    /// 显示选项菜单并让用户选择
    pub fn select(&self, prompt: &str, options: &[&str]) -> Result<usize> {
        use dialoguer::Select;
        
        let selection = Select::new()
            .with_prompt(prompt)
            .items(options)
            .default(0)
            .interact()?;

        Ok(selection)
    }
}

impl Default for HitlController {
    fn default() -> Self {
        Self::new()
    }
}
