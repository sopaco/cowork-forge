# 部署机制

## 概述

Cowork Forge 的部署机制确保代码从 iteration workspace 安全地复制到项目根目录。部署只在 Delivery 阶段执行，遵循严格的文件过滤规则，只复制必要的源代码文件，避免复制临时文件和构建产物。

### 部署的目标

1. **代码同步**：将生成的代码同步到项目根目录
2. **文件过滤**：只复制必要的源代码文件
3. **安全保护**：避免覆盖用户的重要文件
4. **版本管理**：保留部署历史记录
5. **可回滚**：支持回滚到之前的部署版本

### 部署特点

- **阶段隔离**：只在 Delivery 阶段执行
- **文件过滤**：根据扩展名过滤文件
- **用户确认**：需要用户明确确认
- **增量更新**：只复制修改过的文件
- **冲突检测**：检测文件冲突并提示用户

## 部署流程

### 完整流程

```
Delivery Agent 执行
    ↓
生成 delivery.md
    ↓
调用 copy_workspace_to_project(confirm=true)
    ↓
验证确认参数
    ↓
获取 workspace 路径
    ↓
获取项目根目录
    ↓
遍历 workspace
    ↓
    ├─→ 检查文件扩展名
    ├─→ 决定是否复制
    └─→ 创建目标目录
    ↓
复制文件
    ↓
记录复制日志
    ↓
返回结果
```

### 部署实现

```rust
pub struct CopyWorkspaceToProjectTool;

#[async_trait]
impl Tool for CopyWorkspaceToProjectTool {
    fn name(&self) -> &str {
        "copy_workspace_to_project"
    }

    fn description(&self) -> &str {
        "Copy workspace files to project root directory"
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "confirm": {
                    "type": "boolean",
                    "description": "Confirm the copy operation"
                }
            },
            "required": ["confirm"]
        })
    }

    async fn execute(
        &self,
        _ctx: Arc<dyn ToolContext>,
        args: Value,
    ) -> adk_core::Result<Value> {
        // 1. 验证确认参数
        let confirm = args.get("confirm")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !confirm {
            return Err(adk_core::Error::Validation(
                "Copy operation requires confirm=true".to_string()
            ));
        }

        // 2. 获取 workspace 路径
        let iteration_id = get_iteration_id()?;
        let iteration_store = IterationStore::new();
        let workspace_dir = iteration_store.workspace_path(&iteration_id)?;

        // 3. 获取项目根目录
        let project_root = std::env::current_dir()
            .map_err(|e| adk_core::Error::Io(e.to_string()))?;

        // 4. 定义允许复制的文件扩展名
        let extensions_to_copy = vec![
            ".html", ".htm", ".css", ".js", ".jsx", ".ts", ".tsx",
            ".json", ".md", ".txt", ".svg", ".png", ".jpg", ".jpeg",
            ".woff", ".woff2", ".ttf", ".eot",
        ];

        // 5. 遍历 workspace 并复制文件
        let mut copied_files = Vec::new();
        let mut skipped_files = Vec::new();

        for entry in walkdir::WalkDir::new(&workspace_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let src_path = entry.path();
            let rel_path = src_path.strip_prefix(&workspace_dir)
                .map_err(|e| adk_core::Error::Io(e.to_string()))?;

            // 检查文件扩展名
            let should_copy = src_path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| extensions_to_copy.contains(&format!(".{}", ext)))
                .unwrap_or(false);

            if should_copy {
                let dest_path = project_root.join(rel_path);

                // 创建父目录
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| adk_core::Error::Io(e.to_string()))?;
                }

                // 复制文件
                fs::copy(&src_path, &dest_path)
                    .map_err(|e| adk_core::Error::Io(e.to_string()))?;

                copied_files.push(rel_path.to_string_lossy().to_string());
            } else {
                skipped_files.push(rel_path.to_string_lossy().to_string());
            }
        }

        // 6. 返回结果
        Ok(json!({
            "status": "success",
            "copied_files": copied_files,
            "skipped_files": skipped_files,
            "copied_count": copied_files.len(),
            "skipped_count": skipped_files.len()
        }))
    }
}
```

## 文件过滤

### 允许的文件扩展名

| 分类 | 扩展名 | 说明 |
|------|-------|------|
| **HTML** | .html, .htm | HTML 文件 |
| **CSS** | .css | CSS 样式文件 |
| **JavaScript** | .js, .jsx | JavaScript 文件 |
| **TypeScript** | .ts, .tsx | TypeScript 文件 |
| **JSON** | .json | JSON 配置文件 |
| **Markdown** | .md | Markdown 文档 |
| **文本** | .txt | 文本文件 |
| **图像** | .svg, .png, .jpg, .jpeg | 图像文件 |
| **字体** | .woff, .woff2, .ttf, .eot | 字体文件 |

### 拒绝的文件类型

| 分类 | 扩展名 | 原因 |
|------|-------|------|
| **编译产物** | .o, .obj, .exe, .dll, .so | 编译产物，不需要复制 |
| **依赖** | node_modules/, target/, dist/ | 依赖目录，不应该复制 |
| **临时文件** | .tmp, .swp, .bak | 临时文件 |
| **系统文件** | .DS_Store, Thumbs.db | 系统文件 |
| **配置文件** | .git/, .gitignore | Git 配置 |
| **日志** | *.log | 日志文件 |

### 文件过滤规则

```rust
/// 检查文件是否应该被复制
fn should_copy_file(path: &Path, extensions_to_copy: &[String]) -> bool {
    // 1. 检查是否为文件
    if !path.is_file() {
        return false;
    }

    // 2. 检查扩展名
    if let Some(ext) = path.extension() {
        let ext_str = format!(".{}", ext.to_str().unwrap_or(""));
        return extensions_to_copy.contains(&ext_str);
    }

    false
}

/// 检查路径是否应该被跳过
fn should_skip_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_string();

    // 跳过的目录和文件
    const SKIP_PATTERNS: &[&str] = &[
        "node_modules",
        "target",
        "dist",
        "build",
        ".git",
        ".DS_Store",
        "Thumbs.db",
        ".swp",
        ".tmp",
    ];

    for pattern in SKIP_PATTERNS {
        if path_str.contains(pattern) {
            return true;
        }
    }

    false
}
```

## 部署配置

### 配置结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub enabled: bool,                    // 是否启用部署
    pub require_confirm: bool,            // 是否需要确认
    pub overwrite: bool,                  // 是否覆盖现有文件
    pub backup_before_deploy: bool,       // 部署前是否备份
    pub create_deployment_log: bool,      // 是否创建部署日志
    pub allowed_extensions: Vec<String>,  // 允许的扩展名
    pub skip_patterns: Vec<String>,       // 跳过的模式
}

impl Default for DeploymentConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            require_confirm: true,
            overwrite: true,
            backup_before_deploy: true,
            create_deployment_log: true,
            allowed_extensions: vec![
                ".html".to_string(),
                ".htm".to_string(),
                ".css".to_string(),
                ".js".to_string(),
                ".jsx".to_string(),
                ".ts".to_string(),
                ".tsx".to_string(),
                ".json".to_string(),
                ".md".to_string(),
            ],
            skip_patterns: vec![
                "node_modules".to_string(),
                "target".to_string(),
                ".git".to_string(),
            ],
        }
    }
}
```

### 配置文件

```toml
[deployment]
enabled = true
require_confirm = true
overwrite = true
backup_before_deploy = true
create_deployment_log = true

allowed_extensions = [
    ".html", ".htm", ".css",
    ".js", ".jsx", ".ts", ".tsx",
    ".json", ".md"
]

skip_patterns = [
    "node_modules",
    "target",
    "dist",
    ".git"
]
```

## 部署历史

### 部署记录

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRecord {
    pub id: String,                      // 部署 ID
    pub iteration_id: String,            // 迭代 ID
    pub timestamp: DateTime<Utc>,        // 部署时间
    pub copied_files: Vec<String>,       // 复制的文件
    pub skipped_files: Vec<String>,      // 跳过的文件
    pub status: DeploymentStatus,        // 部署状态
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStatus {
    Success,     // 成功
    Partial,     // 部分成功
    Failed,      // 失败
    RolledBack,  // 已回滚
}
```

### 保存部署记录

```rust
impl DeploymentManager {
    /// 保存部署记录
    pub async fn save_deployment_record(
        &self,
        iteration_id: &str,
        copied_files: Vec<String>,
        skipped_files: Vec<String>,
    ) -> Result<DeploymentRecord, CoworkError> {
        let record = DeploymentRecord {
            id: format!("deploy-{}", uuid::Uuid::new_v4()),
            iteration_id: iteration_id.to_string(),
            timestamp: Utc::now(),
            copied_files,
            skipped_files,
            status: DeploymentStatus::Success,
        };

        // 保存到文件
        let deployment_dir = PathBuf::from(".cowork-v2/deployments");
        fs::create_dir_all(&deployment_dir)?;

        let record_file = deployment_dir.join(&record.id).with_extension("json");
        let content = serde_json::to_string_pretty(&record)?;
        fs::write(record_file, content)?;

        // 保存索引
        let index_file = deployment_dir.join("index.json");
        let mut index: Vec<DeploymentRecord> = if index_file.exists() {
            let content = fs::read_to_string(&index_file)?;
            serde_json::from_str(&content)?
        } else {
            Vec::new()
        };

        index.push(record.clone());
        index.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        let index_content = serde_json::to_string_pretty(&index)?;
        fs::write(index_file, index_content)?;

        Ok(record)
    }
}
```

## 备份机制

### 备份实现

```rust
impl DeploymentManager {
    /// 部署前备份
    pub async fn backup_before_deploy(
        &self,
        project_root: &Path,
        files_to_copy: &[String],
    ) -> Result<String, CoworkError> {
        if !self.config.backup_before_deploy {
            return Ok("".to_string());
        }

        // 创建备份目录
        let backup_dir = project_root.join(".cowork-v2/backups");
        fs::create_dir_all(&backup_dir)?;

        let backup_id = format!("backup-{}", Utc::now().timestamp());
        let backup_path = backup_dir.join(&backup_id);
        fs::create_dir_all(&backup_path)?;

        // 复制文件到备份目录
        for file in files_to_copy {
            let src_path = project_root.join(file);
            if src_path.exists() {
                let dest_path = backup_path.join(file);
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&src_path, &dest_path)?;
            }
        }

        Ok(backup_id)
    }

    /// 回滚部署
    pub async fn rollback(
        &self,
        backup_id: &str,
        project_root: &Path,
    ) -> Result<(), CoworkError> {
        let backup_dir = project_root.join(".cowork-v2/backups");
        let backup_path = backup_dir.join(backup_id);

        if !backup_path.exists() {
            return Err(CoworkError::BackupNotFound(backup_id.to_string()));
        }

        // 从备份目录恢复文件
        for entry in walkdir::WalkDir::new(&backup_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let src_path = entry.path();
            let rel_path = src_path.strip_prefix(&backup_path)?;
            let dest_path = project_root.join(rel_path);

            fs::copy(&src_path, &dest_path)?;
        }

        Ok(())
    }
}
```

## 冲突检测

### 冲突检测实现

```rust
impl DeploymentManager {
    /// 检测文件冲突
    pub async fn detect_conflicts(
        &self,
        workspace_dir: &Path,
        project_root: &Path,
    ) -> Result<Vec<FileConflict>, CoworkError> {
        let mut conflicts = Vec::new();

        for entry in walkdir::WalkDir::new(workspace_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let src_path = entry.path();
            let rel_path = src_path.strip_prefix(workspace_dir)?;
            let dest_path = project_root.join(rel_path);

            if dest_path.exists() {
                // 文件已存在，检查是否有修改
                let src_content = fs::read_to_string(src_path)?;
                let dest_content = fs::read_to_string(&dest_path)?;

                if src_content != dest_content {
                    conflicts.push(FileConflict {
                        path: rel_path.to_string_lossy().to_string(),
                        workspace_hash: calculate_hash(&src_content),
                        project_hash: calculate_hash(&dest_content),
                    });
                }
            }
        }

        Ok(conflicts)
    }
}

#[derive(Debug, Clone)]
pub struct FileConflict {
    pub path: String,
    pub workspace_hash: String,
    pub project_hash: String,
}

fn calculate_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
```

## 部署最佳实践

### 1. 总是要求确认

```rust
// ✅ 好的做法：要求确认
let confirm = args.get("confirm")
    .and_then(|v| v.as_bool())
    .unwrap_or(false);

if !confirm {
    return Err(adk_core::Error::Validation("Requires confirm=true".to_string()));
}

// ❌ 不好的做法：不要求确认
// 直接复制文件
```

### 2. 提供详细的日志

```rust
// ✅ 好的做法：记录详细信息
log::info!("Deployment started for iteration: {}", iteration_id);
log::info!("Copied {} files", copied_files.len());
log::info!("Skipped {} files", skipped_files.len());
for file in &copied_files {
    log::debug!("Copied: {}", file);
}

// ❌ 不好的做法：不记录日志
// 静默执行
```

### 3. 备份后再部署

```rust
// ✅ 好的做法：先备份
let backup_id = self.backup_before_deploy(&project_root, &copied_files).await?;
self.copy_files(&workspace_dir, &project_root).await?;

// ❌ 不好的做法：不备份
self.copy_files(&workspace_dir, &project_root).await?;
```

### 4. 检测冲突

```rust
// ✅ 好的做法：检测冲突
let conflicts = self.detect_conflicts(&workspace_dir, &project_root).await?;
if !conflicts.is_empty() {
    return Err(CoworkError::DeploymentConflicts(conflicts));
}

// ❌ 不好的做法：不检测冲突
// 直接覆盖
```

## 相关文档

- [架构概览](../architecture/overview.md)
- [Pipeline 流程](../architecture/pipeline.md)
- [文件安全机制](../architecture/file-security.md)
- [迭代架构](../architecture/iteration-architecture.md)