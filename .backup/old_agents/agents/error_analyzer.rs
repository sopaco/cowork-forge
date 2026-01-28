use std::collections::HashMap;

use crate::artifacts::*;

/// 错误分析器 - 从 CheckReport 中提取关键信息
pub struct ErrorAnalyzer;

impl ErrorAnalyzer {
    /// 分析检查报告，提取受影响的文件和错误摘要
    pub fn analyze(check_report: &CheckReport) -> ErrorAnalysis {
        let mut affected_files: HashMap<String, Vec<String>> = HashMap::new();
        let mut error_count_by_severity: HashMap<String, usize> = HashMap::new();
        
        // 遍历所有 issues
        for issue in &check_report.issues {
            // 统计严重程度
            *error_count_by_severity.entry(issue.sev.clone()).or_insert(0) += 1;
            
            // 从 issue.id 中提取文件路径
            // 例如: "ISSUE-FILE-app.rs" -> "app.rs"
            //      "ISSUE-COMPILE-RUST" -> 影响所有文件
            //      "ISSUE-SYNTAX-PY-main.py" -> "main.py"
            let mut extracted_files: Vec<String> = Vec::new();

            let file_path = Self::extract_file_path(&issue.id);
            if !file_path.is_empty() {
                extracted_files.push(file_path);
            }

            // 对验证命令失败的 issue，尝试从 fix_hint 文本中提取文件路径（跨语言）
            if issue.id.starts_with("ISSUE-VERIFY-") {
                let more = Self::extract_files_from_text(&issue.fix_hint);
                for f in more {
                    if !extracted_files.contains(&f) {
                        extracted_files.push(f);
                    }
                }
            }

            for f in extracted_files {
                if f.is_empty() {
                    continue;
                }
                affected_files
                    .entry(f.clone())
                    .or_insert_with(Vec::new)
                    .push(format!("[{}] {}", issue.sev, issue.desc));
            }
        }
        
        // 生成摘要
        let total_errors = check_report.issues.len();
        let critical_errors = error_count_by_severity.get("error").copied().unwrap_or(0);
        let warnings = error_count_by_severity.get("warning").copied().unwrap_or(0);
        
        let summary = if total_errors == 0 {
            "All checks passed".to_string()
        } else {
            format!(
                "{} total issues ({} errors, {} warnings)",
                total_errors, critical_errors, warnings
            )
        };
        
        // 提取详细错误信息（用于传递给重试）
        let detailed_errors = check_report.issues.iter()
            .filter(|issue| issue.sev == "error")
            .map(|issue| format!("- {}: {}\n  Fix hint: {}", issue.id, issue.desc, issue.fix_hint))
            .collect::<Vec<_>>()
            .join("\n\n");
        
        ErrorAnalysis {
            affected_files: affected_files.keys().cloned().collect(),
            error_details_by_file: affected_files,
            summary,
            detailed_errors,
            has_critical_errors: critical_errors > 0,
        }
    }
    
    /// 从 issue ID 中提取文件路径
    fn extract_file_path(issue_id: &str) -> String {
        // ISSUE-FILE-app.rs -> app.rs
        if issue_id.starts_with("ISSUE-FILE-") {
            return issue_id.strip_prefix("ISSUE-FILE-").unwrap_or("").to_string();
        }
        
        // ISSUE-EMPTY-src/main.rs -> src/main.rs
        if issue_id.starts_with("ISSUE-EMPTY-") {
            return issue_id.strip_prefix("ISSUE-EMPTY-").unwrap_or("").to_string();
        }
        
        // ISSUE-TODO-app.js -> app.js
        if issue_id.starts_with("ISSUE-TODO-") {
            return issue_id.strip_prefix("ISSUE-TODO-").unwrap_or("").to_string();
        }
        
        // ISSUE-SYNTAX-PY-main.py -> main.py
        if issue_id.starts_with("ISSUE-SYNTAX-PY-") {
            return issue_id.strip_prefix("ISSUE-SYNTAX-PY-").unwrap_or("").to_string();
        }

        // ISSUE-COMPILE-RUST -> 空（影响多个文件）
        String::new()
    }
    
    /// 从任意错误文本中提取文件路径（跨语言，适用于验证命令输出）
    pub fn extract_files_from_text(text: &str) -> Vec<String> {
        let mut files = Vec::new();

        // Generic: path.ext:line:col
        for line in text.lines() {
            if let Some((maybe_path, _rest)) = line.split_once(':') {
                if Self::looks_like_path(maybe_path) {
                    let p = maybe_path.trim().replace('\\', "/");
                    if !files.contains(&p) {
                        files.push(p);
                    }
                }
            }
        }

        // Rust style: --> src/main.rs:42:5
        for line in text.lines() {
            if let Some(pos) = line.find(" --> ") {
                let path_part = &line[pos + 5..];
                if let Some(colon_pos) = path_part.find(':') {
                    let p = path_part[..colon_pos].trim().replace('\\', "/");
                    if !files.contains(&p) {
                        files.push(p);
                    }
                }
            }
        }

        // Python style: File "main.py", line 10
        for line in text.lines() {
            if line.contains("File \"") {
                if let Some(start) = line.find("File \"") {
                    let rest = &line[start + 6..];
                    if let Some(end) = rest.find('"') {
                        let p = rest[..end].trim().replace('\\', "/");
                        if !files.contains(&p) {
                            files.push(p);
                        }
                    }
                }
            }
        }

        files
    }

    fn looks_like_path(s: &str) -> bool {
        let s = s.trim();
        if s.is_empty() {
            return false;
        }
        // must contain a dot extension and a slash-like separator
        let has_ext = s.rsplit_once('.').is_some();
        let has_sep = s.contains('/') || s.contains('\\');
        has_ext && has_sep
    }

    /// 从编译错误中智能提取文件路径
    pub fn extract_files_from_compilation_errors(stderr: &str) -> Vec<String> {
        let mut files = Vec::new();
        
        // Rust: error[E0XXX]: ... --> src/main.rs:42:5
        for line in stderr.lines() {
            if line.contains(" --> ") {
                if let Some(pos) = line.find(" --> ") {
                    let path_part = &line[pos + 5..];
                    if let Some(colon_pos) = path_part.find(':') {
                        let file_path = path_part[..colon_pos].trim().to_string();
                        if !files.contains(&file_path) {
                            files.push(file_path);
                        }
                    }
                }
            }
        }
        
        // Python: File "main.py", line 10
        for line in stderr.lines() {
            if line.contains("File \"") {
                if let Some(start) = line.find("File \"") {
                    let rest = &line[start + 6..];
                    if let Some(end) = rest.find('"') {
                        let file_path = rest[..end].to_string();
                        if !files.contains(&file_path) {
                            files.push(file_path);
                        }
                    }
                }
            }
        }
        
        files
    }
}

/// 错误分析结果
#[derive(Debug, Clone)]
pub struct ErrorAnalysis {
    /// 受影响的文件列表
    pub affected_files: Vec<String>,
    
    /// 每个文件的详细错误
    pub error_details_by_file: HashMap<String, Vec<String>>,
    
    /// 错误摘要
    pub summary: String,
    
    /// 详细错误信息（用于传递给 Agent）
    pub detailed_errors: String,
    
    /// 是否有严重错误
    pub has_critical_errors: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_file_path() {
        assert_eq!(ErrorAnalyzer::extract_file_path("ISSUE-FILE-app.rs"), "app.rs");
        assert_eq!(ErrorAnalyzer::extract_file_path("ISSUE-EMPTY-src/main.rs"), "src/main.rs");
        assert_eq!(ErrorAnalyzer::extract_file_path("ISSUE-TODO-index.html"), "index.html");
        assert_eq!(ErrorAnalyzer::extract_file_path("ISSUE-COMPILE-RUST"), "");
    }
    
    #[test]
    fn test_extract_files_from_compilation_errors() {
        let rust_error = r#"
error[E0425]: cannot find value `x` in this scope
 --> src/main.rs:42:5
  |
42 |     x + 1
  |     ^ not found in this scope

error[E0308]: mismatched types
 --> src/lib.rs:10:20
  |
10 |     let y: i32 = "hello";
   |                  ^^^^^^^ expected `i32`, found `&str`
"#;
        
        let files = ErrorAnalyzer::extract_files_from_compilation_errors(rust_error);
        assert_eq!(files, vec!["src/main.rs", "src/lib.rs"]);
    }
}
