use adk_rust::prelude::*;
use adk_rust::AdkError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::path::Path;

/// 文件读取参数
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct ReadFileParams {
    /// 文件路径（相对或绝对路径）
    pub path: String,
}

/// 文件写入参数
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct WriteFileParams {
    /// 文件路径
    pub path: String,
    /// 文件内容
    pub content: String,
}

/// 目录列表参数
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct ListDirParams {
    /// 目录路径
    pub path: String,
    /// 是否递归列出子目录
    #[serde(default)]
    pub recursive: bool,
    /// 是否包含隐藏文件（默认不包含）
    #[serde(default)]
    pub include_hidden: bool,
}

/// 文件存在检查参数
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct FileExistsParams {
    /// 文件路径
    pub path: String,
}

/// 创建目录参数
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct CreateDirParams {
    /// 目录路径
    pub path: String,
    /// 是否创建父目录
    #[serde(default)]
    pub recursive: bool,
}

/// 读取文件范围参数
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct ReadFileRangeParams {
    /// 文件路径
    pub path: String,
    /// 起始行号（1-based，包含）
    pub start_line: usize,
    /// 结束行号（1-based，包含）。如果省略，读到文件末尾
    #[serde(default)]
    pub end_line: Option<usize>,
}

/// 替换文件行范围参数
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct ReplaceLineRangeParams {
    /// 文件路径
    pub path: String,
    /// 起始行号（1-based，包含）
    pub start_line: usize,
    /// 结束行号（1-based，包含）
    pub end_line: usize,
    /// 新内容（多行文本）
    pub new_content: String,
}

/// 插入行参数
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct InsertLinesParams {
    /// 文件路径
    pub path: String,
    /// 在此行号之后插入（1-based）。0 表示在文件开头插入
    pub after_line: usize,
    /// 要插入的内容
    pub content: String,
}

/// 删除行范围参数
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct DeleteLineRangeParams {
    /// 文件路径
    pub path: String,
    /// 起始行号（1-based，包含）
    pub start_line: usize,
    /// 结束行号（1-based，包含）
    pub end_line: usize,
}

/// 追加到文件参数
#[derive(JsonSchema, Serialize, Deserialize)]
pub struct AppendToFileParams {
    /// 文件路径
    pub path: String,
    /// 要追加的内容
    pub content: String,
}

/// 检查文件名是否为隐藏文件
#[cfg(test)]
pub(crate) fn is_hidden_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

/// 构建 gitignore walker
pub(crate) fn build_gitignore_walker(root: &str, recursive: bool, include_hidden: bool) -> ignore::Walk {
    let mut builder = ignore::WalkBuilder::new(root);
    
    // 设置最大深度
    if !recursive {
        builder.max_depth(Some(1));
    }
    
    // 控制是否包含隐藏文件
    if !include_hidden {
        builder.hidden(false); // 排除隐藏文件
    } else {
        builder.hidden(true); // 包含隐藏文件
    }
    
    // 始终遵循 .gitignore 规则
    builder.git_ignore(true);
    builder.git_global(true);
    builder.git_exclude(true);
    
    // 不遵循符号链接（避免循环）
    builder.follow_links(false);
    
    // 🔧 额外过滤：排除常见依赖目录和构建输出（即使没有 .gitignore）
    // 这些目录通常包含大量文件但对代码生成无意义
    builder.filter_entry(|entry| {
        let path = entry.path();
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        // 排除常见依赖和构建目录
        let excluded_dirs = [
            "node_modules",    // Node.js
            "target",          // Rust
            "dist",            // 构建输出
            "build",           // 构建输出
            "out",             // 构建输出
            ".next",           // Next.js
            ".nuxt",           // Nuxt.js
            ".venv",           // Python
            "venv",            // Python
            "env",             // Python
            "__pycache__",     // Python
            "vendor",          // 多种语言
            ".tox",            // Python
            ".pytest_cache",   // Python
            ".mypy_cache",     // Python
            "coverage",        // 测试覆盖率
            ".coverage",       // 测试覆盖率
            "htmlcov",         // 测试覆盖率
            "bower_components", // Bower
            "jspm_packages",   // JSPM
            ".gradle",         // Gradle
            ".mvn",            // Maven
            "Pods",            // CocoaPods
            ".cargo",          // Rust (local cache)
        ];
        
        !excluded_dirs.contains(&file_name)
    });
    
    builder.build()
}

/// 文件工具集合
pub struct FileToolsBundle {
    pub read_file: Arc<FunctionTool>,
    pub write_file: Arc<FunctionTool>,
    pub list_dir: Arc<FunctionTool>,
    pub file_exists: Arc<FunctionTool>,
    pub create_dir: Arc<FunctionTool>,
    // 增量编辑工具
    pub read_file_range: Arc<FunctionTool>,
    pub replace_line_range: Arc<FunctionTool>,
    pub insert_lines: Arc<FunctionTool>,
    pub delete_line_range: Arc<FunctionTool>,
    pub append_to_file: Arc<FunctionTool>,
}

/// 创建文件操作工具集
pub fn create_file_tools() -> FileToolsBundle {
    // 1. 读取文件工具
    let read_file = Arc::new(
        FunctionTool::new(
            "read_file",
            "Read the contents of a file. Returns the file content as a string.",
            |_ctx, args| async move {
                let params: ReadFileParams = serde_json::from_value(args)
                    .map_err(|e| AdkError::Tool(format!("Invalid parameters: {}", e)))?;

                match std::fs::read_to_string(&params.path) {
                    Ok(content) => Ok(json!({
                        "success": true,
                        "path": params.path,
                        "content": content,
                        "size": content.len()
                    })),
                    Err(e) => Err(AdkError::Tool(format!(
                        "Failed to read file '{}': {}",
                        params.path, e
                    ))),
                }
            },
        )
        .with_parameters_schema::<ReadFileParams>(),
    );

    // 2. 写入文件工具
    let write_file = Arc::new(
        FunctionTool::new(
            "write_file",
            "Write content to a file. Creates the file if it doesn't exist, overwrites if it does.",
            |_ctx, args| async move {
                let params: WriteFileParams = serde_json::from_value(args)
                    .map_err(|e| AdkError::Tool(format!("Invalid parameters: {}", e)))?;

                // 确保父目录存在
                if let Some(parent) = Path::new(&params.path).parent() {
                    if !parent.exists() {
                        std::fs::create_dir_all(parent).map_err(|e| {
                            AdkError::Tool(format!(
                                "Failed to create parent directories: {}",
                                e
                            ))
                        })?;
                    }
                }

                match std::fs::write(&params.path, &params.content) {
                    Ok(_) => Ok(json!({
                        "success": true,
                        "path": params.path,
                        "bytes_written": params.content.len()
                    })),
                    Err(e) => Err(AdkError::Tool(format!(
                        "Failed to write file '{}': {}",
                        params.path, e
                    ))),
                }
            },
        )
        .with_parameters_schema::<WriteFileParams>(),
    );

    // 3. 列出目录工具（使用 ignore crate 处理 .gitignore）
    let list_dir = Arc::new(
        FunctionTool::new(
            "list_directory",
            "List files and directories in a directory. Automatically respects .gitignore rules and excludes hidden files by default. Use include_hidden=true to show hidden files.",
            |_ctx, args| async move {
                let params: ListDirParams = serde_json::from_value(args)
                    .map_err(|e| AdkError::Tool(format!("Invalid parameters: {}", e)))?;

                let mut entries = Vec::new();
                
                // 使用 ignore crate 构建 walker（自动处理 .gitignore）
                let walker = build_gitignore_walker(&params.path, params.recursive, params.include_hidden);

                for result in walker {
                    match result {
                        Ok(entry) => {
                            let path = entry.path();
                            
                            // 跳过根目录自身
                            if path == Path::new(&params.path) {
                                continue;
                            }
                            
                            let path_str = path.to_string_lossy().to_string();
                            let is_dir = path.is_dir();
                            let is_file = path.is_file();
                            
                            let size = if is_file {
                                std::fs::metadata(path).ok().map(|m| m.len()).unwrap_or(0)
                            } else {
                                0
                            };

                            entries.push(json!({
                                "path": path_str,
                                "is_dir": is_dir,
                                "is_file": is_file,
                                "size": size
                            }));
                        }
                        Err(e) => {
                            // 记录错误但继续处理其他文件
                            tracing::warn!("Error walking directory: {}", e);
                        }
                    }
                }

                Ok(json!({
                    "success": true,
                    "path": params.path,
                    "count": entries.len(),
                    "entries": entries,
                    "note": "Hidden files and .gitignore patterns are excluded by default"
                }))
            },
        )
        .with_parameters_schema::<ListDirParams>(),
    );

    // 4. 检查文件是否存在工具
    let file_exists = Arc::new(
        FunctionTool::new(
            "file_exists",
            "Check if a file or directory exists.",
            |_ctx, args| async move {
                let params: FileExistsParams = serde_json::from_value(args)
                    .map_err(|e| AdkError::Tool(format!("Invalid parameters: {}", e)))?;

                let path = Path::new(&params.path);
                let exists = path.exists();
                let is_dir = path.is_dir();
                let is_file = path.is_file();

                Ok(json!({
                    "path": params.path,
                    "exists": exists,
                    "is_dir": is_dir,
                    "is_file": is_file
                }))
            },
        )
        .with_parameters_schema::<FileExistsParams>(),
    );

    // 5. 创建目录工具
    let create_dir = Arc::new(
        FunctionTool::new(
            "create_directory",
            "Create a directory. Can create parent directories if recursive is true.",
            |_ctx, args| async move {
                let params: CreateDirParams = serde_json::from_value(args)
                    .map_err(|e| AdkError::Tool(format!("Invalid parameters: {}", e)))?;

                let result = if params.recursive {
                    std::fs::create_dir_all(&params.path)
                } else {
                    std::fs::create_dir(&params.path)
                };

                match result {
                    Ok(_) => Ok(json!({
                        "success": true,
                        "path": params.path
                    })),
                    Err(e) => Err(AdkError::Tool(format!(
                        "Failed to create directory '{}': {}",
                        params.path, e
                    ))),
                }
            },
        )
        .with_parameters_schema::<CreateDirParams>(),
    );

    // 6. 读取文件范围工具（用于大文件）
    let read_file_range = Arc::new(
        FunctionTool::new(
            "read_file_range",
            "Read a specific range of lines from a file. Useful for large files to avoid context overflow. Line numbers are 1-based.",
            |_ctx, args| async move {
                let params: ReadFileRangeParams = serde_json::from_value(args)
                    .map_err(|e| AdkError::Tool(format!("Invalid parameters: {}", e)))?;

                let content = std::fs::read_to_string(&params.path)
                    .map_err(|e| AdkError::Tool(format!("Failed to read file '{}': {}", params.path, e)))?;

                let lines: Vec<&str> = content.lines().collect();
                let total_lines = lines.len();

                if params.start_line < 1 || params.start_line > total_lines {
                    return Err(AdkError::Tool(format!(
                        "Invalid start_line: {} (file has {} lines)",
                        params.start_line, total_lines
                    )));
                }

                let start_idx = params.start_line - 1;
                let end_idx = match params.end_line {
                    Some(end) if end > 0 => end.min(total_lines),
                    _ => total_lines,
                };

                let selected_lines = &lines[start_idx..end_idx];
                let selected_content = selected_lines.join("\n");

                Ok(json!({
                    "success": true,
                    "path": params.path,
                    "start_line": params.start_line,
                    "end_line": end_idx,
                    "total_lines": total_lines,
                    "content": selected_content,
                    "lines_read": selected_lines.len()
                }))
            },
        )
        .with_parameters_schema::<ReadFileRangeParams>(),
    );

    // 7. 替换行范围工具
    let replace_line_range = Arc::new(
        FunctionTool::new(
            "replace_line_range",
            "Replace a range of lines in a file with new content. Useful for modifying specific sections without rewriting the entire file. Line numbers are 1-based.",
            |_ctx, args| async move {
                let params: ReplaceLineRangeParams = serde_json::from_value(args)
                    .map_err(|e| AdkError::Tool(format!("Invalid parameters: {}", e)))?;

                let content = std::fs::read_to_string(&params.path)
                    .map_err(|e| AdkError::Tool(format!("Failed to read file '{}': {}", params.path, e)))?;

                let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
                let total_lines = lines.len();

                if params.start_line < 1 || params.start_line > total_lines {
                    return Err(AdkError::Tool(format!("Invalid start_line: {}", params.start_line)));
                }
                if params.end_line < params.start_line || params.end_line > total_lines {
                    return Err(AdkError::Tool(format!("Invalid end_line: {}", params.end_line)));
                }

                // 替换指定范围
                let start_idx = params.start_line - 1;
                let end_idx = params.end_line;
                
                let new_lines: Vec<String> = params.new_content.lines().map(|s| s.to_string()).collect();
                lines.splice(start_idx..end_idx, new_lines.clone());

                let new_content = lines.join("\n");
                std::fs::write(&params.path, new_content)
                    .map_err(|e| AdkError::Tool(format!("Failed to write file: {}", e)))?;

                Ok(json!({
                    "success": true,
                    "path": params.path,
                    "replaced_lines": format!("{}-{}", params.start_line, params.end_line),
                    "new_line_count": new_lines.len(),
                    "total_lines_after": lines.len()
                }))
            },
        )
        .with_parameters_schema::<ReplaceLineRangeParams>(),
    );

    // 8. 插入行工具
    let insert_lines = Arc::new(
        FunctionTool::new(
            "insert_lines",
            "Insert new lines after a specific line number. Line numbers are 1-based. Use after_line=0 to insert at the beginning.",
            |_ctx, args| async move {
                let params: InsertLinesParams = serde_json::from_value(args)
                    .map_err(|e| AdkError::Tool(format!("Invalid parameters: {}", e)))?;

                let content = std::fs::read_to_string(&params.path)
                    .map_err(|e| AdkError::Tool(format!("Failed to read file '{}': {}", params.path, e)))?;

                let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
                let total_lines = lines.len();

                if params.after_line > total_lines {
                    return Err(AdkError::Tool(format!(
                        "Invalid after_line: {} (file has {} lines)",
                        params.after_line, total_lines
                    )));
                }

                let new_lines: Vec<String> = params.content.lines().map(|s| s.to_string()).collect();
                let insert_idx = params.after_line;
                
                for (i, line) in new_lines.iter().enumerate() {
                    lines.insert(insert_idx + i, line.clone());
                }

                let new_content = lines.join("\n");
                std::fs::write(&params.path, new_content)
                    .map_err(|e| AdkError::Tool(format!("Failed to write file: {}", e)))?;

                Ok(json!({
                    "success": true,
                    "path": params.path,
                    "inserted_after_line": params.after_line,
                    "lines_inserted": new_lines.len(),
                    "total_lines_after": lines.len()
                }))
            },
        )
        .with_parameters_schema::<InsertLinesParams>(),
    );

    // 9. 删除行范围工具
    let delete_line_range = Arc::new(
        FunctionTool::new(
            "delete_line_range",
            "Delete a range of lines from a file. Line numbers are 1-based.",
            |_ctx, args| async move {
                let params: DeleteLineRangeParams = serde_json::from_value(args)
                    .map_err(|e| AdkError::Tool(format!("Invalid parameters: {}", e)))?;

                let content = std::fs::read_to_string(&params.path)
                    .map_err(|e| AdkError::Tool(format!("Failed to read file '{}': {}", params.path, e)))?;

                let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
                let total_lines = lines.len();

                if params.start_line < 1 || params.start_line > total_lines {
                    return Err(AdkError::Tool(format!("Invalid start_line: {}", params.start_line)));
                }
                if params.end_line < params.start_line || params.end_line > total_lines {
                    return Err(AdkError::Tool(format!("Invalid end_line: {}", params.end_line)));
                }

                let start_idx = params.start_line - 1;
                let end_idx = params.end_line;
                let deleted_count = end_idx - start_idx;
                
                lines.drain(start_idx..end_idx);

                let new_content = lines.join("\n");
                std::fs::write(&params.path, new_content)
                    .map_err(|e| AdkError::Tool(format!("Failed to write file: {}", e)))?;

                Ok(json!({
                    "success": true,
                    "path": params.path,
                    "deleted_lines": format!("{}-{}", params.start_line, params.end_line),
                    "lines_deleted": deleted_count,
                    "total_lines_after": lines.len()
                }))
            },
        )
        .with_parameters_schema::<DeleteLineRangeParams>(),
    );

    // 10. 追加到文件工具
    let append_to_file = Arc::new(
        FunctionTool::new(
            "append_to_file",
            "Append content to the end of a file. Adds a newline before the content if the file doesn't end with one.",
            |_ctx, args| async move {
                let params: AppendToFileParams = serde_json::from_value(args)
                    .map_err(|e| AdkError::Tool(format!("Invalid parameters: {}", e)))?;

                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&params.path)
                    .map_err(|e| AdkError::Tool(format!("Failed to open file '{}': {}", params.path, e)))?;

                use std::io::Write;
                
                // 如果文件不为空且不以换行结尾，先加个换行
                let metadata = file.metadata()
                    .map_err(|e| AdkError::Tool(format!("Failed to get metadata: {}", e)))?;
                
                if metadata.len() > 0 {
                    write!(file, "\n")
                        .map_err(|e| AdkError::Tool(format!("Failed to write newline: {}", e)))?;
                }

                write!(file, "{}", params.content)
                    .map_err(|e| AdkError::Tool(format!("Failed to append content: {}", e)))?;

                Ok(json!({
                    "success": true,
                    "path": params.path,
                    "bytes_appended": params.content.len()
                }))
            },
        )
        .with_parameters_schema::<AppendToFileParams>(),
    );

    FileToolsBundle {
        read_file,
        write_file,
        list_dir,
        file_exists,
        create_dir,
        read_file_range,
        replace_line_range,
        insert_lines,
        delete_line_range,
        append_to_file,
    }
}
