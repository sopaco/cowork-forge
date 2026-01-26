
/// 文件上下文摘要 - 用于批次间传递
#[derive(Debug, Clone)]
pub struct FileContext {
    /// 文件路径
    pub path: String,
    
    /// 文件摘要描述
    pub summary: String,
    
    /// 导出的符号/函数/类型
    pub exports: Vec<String>,
    
    /// 导入的依赖
    pub imports: Vec<String>,
    
    /// 关键类型定义
    pub key_types: Vec<String>,
}

/// 批次上下文 - 包含已完成文件的详细信息
#[derive(Debug, Clone)]
pub struct BatchContext {
    /// 已完成的文件上下文
    pub completed_files: Vec<FileContext>,
}

impl BatchContext {
    pub fn new() -> Self {
        Self {
            completed_files: Vec::new(),
        }
    }
    
    /// 添加文件上下文
    pub fn add_file(&mut self, context: FileContext) {
        self.completed_files.push(context);
    }
    
    /// 生成简洁的上下文摘要（用于嵌入 instruction）
    pub fn generate_summary(&self) -> String {
        if self.completed_files.is_empty() {
            return String::new();
        }
        
        let mut lines = vec![
            "**📚 Previously Generated Files:**".to_string(),
            "".to_string(),
        ];
        
        for file in &self.completed_files {
            lines.push(format!("### {}", file.path));
            lines.push(format!("- Summary: {}", file.summary));
            
            if !file.exports.is_empty() {
                lines.push(format!("- Exports: {}", file.exports.join(", ")));
            }
            
            if !file.key_types.is_empty() {
                lines.push(format!("- Key Types: {}", file.key_types.join(", ")));
            }
            
            lines.push("".to_string());
        }
        
        lines.push("**IMPORTANT**: Ensure consistency with these files (naming, types, imports).".to_string());
        lines.push("".to_string());
        
        lines.join("\n")
    }
}

/// 文件摘要生成器
pub struct FileSummaryGenerator;

impl FileSummaryGenerator {
    /// 从文件内容生成上下文摘要
    pub fn generate(path: &str, content: &str, lang: &str) -> FileContext {
        match lang {
            "rust" => Self::generate_rust_context(path, content),
            "python" => Self::generate_python_context(path, content),
            "javascript" | "typescript" => Self::generate_js_context(path, content),
            "html" => Self::generate_html_context(path, content),
            _ => Self::generate_generic_context(path, content),
        }
    }
    
    /// Rust 文件摘要
    fn generate_rust_context(path: &str, content: &str) -> FileContext {
        let mut exports = Vec::new();
        let mut imports = Vec::new();
        let mut key_types = Vec::new();
        
        // 提取 pub struct/enum/fn
        for line in content.lines() {
            let trimmed = line.trim();
            
            // pub struct Xxx
            if trimmed.starts_with("pub struct ") {
                if let Some(name) = trimmed.strip_prefix("pub struct ").and_then(|s| s.split_whitespace().next()) {
                    exports.push(name.trim_end_matches('{').trim().to_string());
                    key_types.push(format!("struct {}", name.trim_end_matches('{').trim()));
                }
            }
            
            // pub enum Xxx
            if trimmed.starts_with("pub enum ") {
                if let Some(name) = trimmed.strip_prefix("pub enum ").and_then(|s| s.split_whitespace().next()) {
                    exports.push(name.trim_end_matches('{').trim().to_string());
                    key_types.push(format!("enum {}", name.trim_end_matches('{').trim()));
                }
            }
            
            // pub fn xxx
            if trimmed.starts_with("pub fn ") {
                if let Some(name) = trimmed.strip_prefix("pub fn ").and_then(|s| s.split('(').next()) {
                    exports.push(format!("{}()", name.trim()));
                }
            }
            
            // use xxx;
            if trimmed.starts_with("use ") && trimmed.ends_with(';') {
                if let Some(import) = trimmed.strip_prefix("use ").and_then(|s| s.strip_suffix(';')) {
                    imports.push(import.trim().to_string());
                }
            }
        }
        
        let summary = if !exports.is_empty() {
            format!("Rust module with {} public items", exports.len())
        } else {
            "Rust source file".to_string()
        };
        
        FileContext {
            path: path.to_string(),
            summary,
            exports,
            imports,
            key_types,
        }
    }
    
    /// Python 文件摘要
    fn generate_python_context(path: &str, content: &str) -> FileContext {
        let mut exports = Vec::new();
        let mut imports = Vec::new();
        let mut key_types = Vec::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            
            // class Xxx:
            if trimmed.starts_with("class ") {
                if let Some(name) = trimmed.strip_prefix("class ").and_then(|s| s.split(':').next()) {
                    let class_name = name.split('(').next().unwrap_or(name).trim().to_string();
                    exports.push(class_name.clone());
                    key_types.push(format!("class {}", class_name));
                }
            }
            
            // def xxx():
            if trimmed.starts_with("def ") && !trimmed.starts_with("def _") {
                if let Some(name) = trimmed.strip_prefix("def ").and_then(|s| s.split('(').next()) {
                    exports.push(format!("{}()", name.trim()));
                }
            }
            
            // import/from xxx import
            if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
                imports.push(trimmed.to_string());
            }
        }
        
        let summary = format!("Python module with {} exports", exports.len());
        
        FileContext {
            path: path.to_string(),
            summary,
            exports,
            imports,
            key_types,
        }
    }
    
    /// JavaScript/TypeScript 文件摘要
    fn generate_js_context(path: &str, content: &str) -> FileContext {
        let mut exports = Vec::new();
        let mut imports = Vec::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            
            // export function xxx
            if trimmed.starts_with("export function ") {
                if let Some(name) = trimmed.strip_prefix("export function ").and_then(|s| s.split('(').next()) {
                    exports.push(format!("{}()", name.trim()));
                }
            }
            
            // export class Xxx
            if trimmed.starts_with("export class ") {
                if let Some(name) = trimmed.strip_prefix("export class ").and_then(|s| s.split_whitespace().next()) {
                    exports.push(name.trim().to_string());
                }
            }
            
            // export const xxx
            if trimmed.starts_with("export const ") {
                if let Some(name) = trimmed.strip_prefix("export const ").and_then(|s| s.split('=').next()) {
                    exports.push(name.trim().to_string());
                }
            }
            
            // import xxx from
            if trimmed.starts_with("import ") {
                imports.push(trimmed.to_string());
            }
        }
        
        let summary = format!("JavaScript module with {} exports", exports.len());
        
        FileContext {
            path: path.to_string(),
            summary,
            exports,
            imports,
            key_types: Vec::new(),
        }
    }
    
    /// HTML 文件摘要
    fn generate_html_context(path: &str, content: &str) -> FileContext {
        let mut key_types = Vec::new();
        
        // 提取 id 和 class
        let mut ids = Vec::new();
        let mut classes = Vec::new();
        
        for line in content.lines() {
            // id="xxx"
            if let Some(start) = line.find("id=\"") {
                if let Some(end) = line[start + 4..].find('"') {
                    ids.push(line[start + 4..start + 4 + end].to_string());
                }
            }
            
            // class="xxx"
            if let Some(start) = line.find("class=\"") {
                if let Some(end) = line[start + 7..].find('"') {
                    let class_str = &line[start + 7..start + 7 + end];
                    for cls in class_str.split_whitespace() {
                        if !classes.contains(&cls.to_string()) {
                            classes.push(cls.to_string());
                        }
                    }
                }
            }
        }
        
        if !ids.is_empty() {
            key_types.push(format!("IDs: {}", ids.join(", ")));
        }
        
        if !classes.is_empty() {
            key_types.push(format!("Classes: {}", classes.iter().take(10).cloned().collect::<Vec<_>>().join(", ")));
        }
        
        FileContext {
            path: path.to_string(),
            summary: "HTML document".to_string(),
            exports: Vec::new(),
            imports: Vec::new(),
            key_types,
        }
    }
    
    /// 通用文件摘要
    fn generate_generic_context(path: &str, content: &str) -> FileContext {
        let lines = content.lines().count();
        
        FileContext {
            path: path.to_string(),
            summary: format!("File with {} lines", lines),
            exports: Vec::new(),
            imports: Vec::new(),
            key_types: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_context_generation() {
        let rust_code = r#"
use serde::{Serialize, Deserialize};

pub struct TodoItem {
    pub id: String,
    pub title: String,
}

pub enum Status {
    Active,
    Done,
}

pub fn create_todo(title: String) -> TodoItem {
    TodoItem { id: uuid::new_v4(), title }
}
"#;
        
        let context = FileSummaryGenerator::generate("todo.rs", rust_code, "rust");
        
        assert_eq!(context.exports.len(), 3);  // TodoItem, Status, create_todo
        assert!(context.exports.contains(&"TodoItem".to_string()));
        assert!(context.exports.contains(&"Status".to_string()));
        assert!(context.exports.contains(&"create_todo()".to_string()));
        assert!(context.imports.len() > 0);
    }
    
    #[test]
    fn test_batch_context_summary() {
        let mut batch_ctx = BatchContext::new();
        
        batch_ctx.add_file(FileContext {
            path: "todo.rs".to_string(),
            summary: "Todo data model".to_string(),
            exports: vec!["TodoItem".to_string(), "create_todo()".to_string()],
            imports: vec!["serde::Serialize".to_string()],
            key_types: vec!["struct TodoItem".to_string()],
        });
        
        let summary = batch_ctx.generate_summary();
        assert!(summary.contains("todo.rs"));
        assert!(summary.contains("TodoItem"));
        assert!(summary.contains("consistency"));
    }
}
