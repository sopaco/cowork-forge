# 文件安全机制

## 概述

Cowork Forge 的文件安全机制确保所有文件操作都在受控的环境中进行，防止恶意代码访问系统敏感文件，保护用户数据安全。通过 Workspace 隔离、路径验证、权限控制等多层安全措施，Cowork Forge 提供了一个安全可靠的开发环境。

### 安全目标

1. **Workspace 隔离**：所有文件操作限制在 iteration workspace 内
2. **路径安全验证**：拒绝绝对路径和父目录访问
3. **权限最小化**：只提供完成任务所需的最小权限
4. **命令安全**：阻止危险命令执行
5. **数据保护**：保护用户数据和项目文件

### 安全层次

```
┌─────────────────────────────────────────────────────────┐
│                  应用层安全                              │
│  - 权限最小化                                            │
│  - 工具专用化                                            │
│  - 输入验证                                              │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                  文件操作层安全                          │
│  - Workspace 隔离                                        │
│  - 路径验证                                              │
│  - 相对路径限制                                          │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                  命令执行层安全                          │
│  - 命令白名单                                            │
│  - 超时控制                                              │
│  - 阻塞检测                                              │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                  系统层安全                              │
│  - 文件系统权限                                          │
│  - 进程隔离                                              │
│  - 资源限制                                              │
└─────────────────────────────────────────────────────────┘
```

## Workspace 隔离

### Workspace 目录结构

每个迭代都有独立的 Workspace 目录：

```
.cowork-v2/
└── iterations/
    └── {iteration_id}/
        └── workspace/              # 代码工作空间
            ├── src/               # 源代码
            ├── components/        # 组件
            ├── tests/             # 测试
            ├── docs/              # 文档
            └── config/            # 配置
```

### Workspace 路径获取

```rust
use std::path::PathBuf;

impl IterationStore {
    /// 获取迭代 Workspace 路径
    pub fn workspace_path(&self, iteration_id: &str) -> Result<PathBuf, CoworkError> {
        let iteration_dir = self.iterations_dir.join(iteration_id);
        let workspace_dir = iteration_dir.join("workspace");

        if !workspace_dir.exists() {
            fs::create_dir_all(&workspace_dir)?;
        }

        Ok(workspace_dir)
    }
}
```

### Workspace 隔离原理

1. **独立目录**：每个迭代有独立的 Workspace 目录
2. **相对路径**：所有文件操作使用相对路径
3. **路径验证**：验证路径是否在 Workspace 内
4. **沙箱执行**：命令在 Workspace 目录内执行

## 路径验证机制

### 验证原则

路径验证遵循以下原则：

- ✅ 接受相对路径：`src/index.html`
- ❌ 拒绝绝对路径：`/tmp/file.txt`
- ❌ 拒绝父目录访问：`../config.toml`
- ✅ 自动拼接到 Workspace：`.cowork-v2/iterations/{id}/workspace/src/index.html`

### 路径安全验证函数

```rust
use std::path::{Path, PathBuf};

/// 验证路径安全性（相对于 workspace）
fn validate_path_security_within_workspace(
    path: &str,
    workspace_dir: &Path,
) -> Result<PathBuf, String> {
    // 1. 解析路径
    let path_obj = Path::new(path);

    // 2. 拒绝绝对路径
    if path_obj.is_absolute() {
        return Err("Absolute paths are not allowed. Please use relative paths.".to_string());
    }

    // 3. 拒绝父目录访问
    if path.contains("..") {
        return Err("Parent directory access (..) is not allowed for security reasons.".to_string());
    }

    // 4. 构造完整路径
    let full_path = workspace_dir.join(path);

    // 5. 规范化路径（解析 . 和 ..）
    let canonical_path = full_path
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize path: {}", e))?;

    // 6. 规范化 workspace 目录
    let canonical_workspace = workspace_dir
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize workspace: {}", e))?;

    // 7. 验证路径在 workspace 目录内
    if !canonical_path.starts_with(&canonical_workspace) {
        return Err(format!(
            "Path escapes workspace directory. Path: {:?}, Workspace: {:?}",
            canonical_path, canonical_workspace
        ));
    }

    Ok(canonical_path)
}
```

### 路径验证示例

```rust
let workspace_dir = PathBuf::from(".cowork-v2/iterations/iter-1/workspace");

// ✅ 有效路径
assert!(validate_path_security_within_workspace("src/index.html", &workspace_dir).is_ok());
assert!(validate_path_security_within_workspace("components/Button.tsx", &workspace_dir).is_ok());

// ❌ 无效路径
assert!(validate_path_security_within_workspace("/etc/passwd", &workspace_dir).is_err());
assert!(validate_path_security_within_workspace("../config.toml", &workspace_dir).is_err());
assert!(validate_path_security_within_workspace("../../malicious.txt", &workspace_dir).is_err());
```

## 文件工具安全实现

### ReadFileTool

```rust
pub struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        // 1. 获取参数
        let path = get_required_string_param(&args, "path")?;

        // 2. 获取 iteration workspace 路径
        let iteration_id = get_iteration_id()?;
        let iteration_store = IterationStore::new();
        let workspace_dir = iteration_store.workspace_path(&iteration_id)?;

        // 3. 验证路径安全性
        let safe_path = validate_path_security_within_workspace(path, &workspace_dir)
            .map_err(|e| adk_core::Error::Validation(e))?;

        // 4. 构造完整路径
        let full_path = workspace_dir.join(&safe_path);

        // 5. 读取文件
        let content = fs::read_to_string(&full_path)
            .map_err(|e| adk_core::Error::Io(e.to_string()))?;

        // 6. 返回结果
        Ok(json!({
            "path": path,
            "content": content,
            "size": content.len()
        }))
    }
}
```

### WriteFileTool

```rust
pub struct WriteFileTool;

#[async_trait]
impl Tool for WriteFileTool {
    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        // 1. 获取参数
        let path = get_required_string_param(&args, "path")?;
        let content = get_required_string_param(&args, "content")?;

        // 2. 获取 iteration workspace 路径
        let iteration_id = get_iteration_id()?;
        let iteration_store = IterationStore::new();
        let workspace_dir = iteration_store.workspace_path(&iteration_id)?;

        // 3. 验证路径安全性
        let safe_path = validate_path_security_within_workspace(path, &workspace_dir)
            .map_err(|e| adk_core::Error::Validation(e))?;

        // 4. 构造完整路径
        let full_path = workspace_dir.join(&safe_path);

        // 5. 创建父目录（如果不存在）
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| adk_core::Error::Io(e.to_string()))?;
        }

        // 6. 写入文件
        fs::write(&full_path, content)
            .map_err(|e| adk_core::Error::Io(e.to_string()))?;

        // 7. 返回结果
        Ok(json!({
            "path": path,
            "size": content.len(),
            "status": "success"
        }))
    }
}
```

### ListFilesTool

```rust
pub struct ListFilesTool;

#[async_trait]
impl Tool for ListFilesTool {
    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        // 1. 获取参数
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        // 2. 获取 iteration workspace 路径
        let iteration_id = get_iteration_id()?;
        let iteration_store = IterationStore::new();
        let workspace_dir = iteration_store.workspace_path(&iteration_id)?;

        // 3. 验证路径安全性
        let safe_path = validate_path_security_within_workspace(path, &workspace_dir)
            .map_err(|e| adk_core::Error::Validation(e))?;

        // 4. 构造完整路径
        let full_path = workspace_dir.join(&safe_path);

        // 5. 列出文件
        let mut files = Vec::new();
        let entries = fs::read_dir(&full_path)
            .map_err(|e| adk_core::Error::Io(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| adk_core::Error::Io(e.to_string()))?;
            let file_name = entry.file_name()
                .to_string_lossy()
                .to_string();

            let metadata = entry.metadata()
                .map_err(|e| adk_core::Error::Io(e.to_string()))?;

            files.push(json!({
                "name": file_name,
                "is_dir": metadata.is_dir(),
                "size": metadata.len()
            }));
        }

        // 6. 返回结果
        Ok(json!({
            "path": path,
            "files": files
        }))
    }
}
```

## 命令安全机制

### RunCommandTool

```rust
pub struct RunCommandTool;

#[async_trait]
impl Tool for RunCommandTool {
    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        // 1. 获取参数
        let command = get_required_string_param(&args, "command")?;
        let args_list = args.get("args")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();

        // 2. 验证命令安全性
        validate_command_security(&command)?;

        // 3. 获取 iteration workspace 路径
        let iteration_id = get_iteration_id()?;
        let iteration_store = IterationStore::new();
        let workspace_dir = iteration_store.workspace_path(&iteration_id)?;

        // 4. 构造命令
        let mut cmd = Command::new(command);
        cmd.args(args_list);
        cmd.current_dir(&workspace_dir);

        // 5. 设置超时
        let timeout = Duration::from_secs(30);

        // 6. 执行命令
        let output = tokio::time::timeout(
            timeout,
            tokio::process::Command::from(cmd).output()
        )
        .await
        .map_err(|_| adk_core::Error::Timeout("Command execution timed out".to_string()))?
        .map_err(|e| adk_core::Error::Io(e.to_string()))?;

        // 7. 返回结果
        Ok(json!({
            "exit_code": output.status.code(),
            "stdout": String::from_utf8_lossy(&output.stdout).to_string(),
            "stderr": String::from_utf8_lossy(&output.stderr).to_string(),
            "success": output.status.success()
        }))
    }
}
```

### 命令安全验证

```rust
/// 验证命令安全性
fn validate_command_security(command: &str) -> Result<(), String> {
    // 危险命令黑名单
    const DANGEROUS_COMMANDS: &[&str] = &[
        "rm", "del", "rmdir", "format",
        "sudo", "su", "doas",
        "chmod", "chown", "chgrp",
        "mkfs", "fdisk", "parted",
        "dd", "shred", "wipe",
        "crontab", "at",
        "systemctl", "service",
        "iptables", "ufw", "firewall-cmd",
        "useradd", "userdel", "usermod",
        "groupadd", "groupdel",
        "passwd",
    ];

    // 检查是否为危险命令
    if DANGEROUS_COMMANDS.contains(&command) {
        return Err(format!(
            "Command '{}' is not allowed for security reasons.",
            command
        ));
    }

    // 检查是否包含危险参数
    const DANGEROUS_PATTERNS: &[&str] = &[
        " -rf ", " -fr ", " / ", " --force",
        " --recursive", " --all", " --no-preserve-root",
    ];

    for pattern in DANGEROUS_PATTERNS {
        if command.contains(pattern) {
            return Err(format!(
                "Command contains dangerous pattern '{}'.",
                pattern
            ));
        }
    }

    Ok(())
}
```

### 命令白名单

```rust
/// 允许的命令列表
const ALLOWED_COMMANDS: &[&str] = &[
    // 构建工具
    "cargo", "npm", "bun", "yarn", "pnpm",
    "rustc", "gcc", "clang", "g++",
    "make", "cmake", "ninja",
    "vite", "webpack", "rollup",

    // 测试工具
    "pytest", "jest", "vitest", "mocha",
    "cargo test", "npm test",

    // 代码质量工具
    "clippy", "eslint", "prettier",
    "rustfmt", "black",

    // 文档工具
    "mdbook", "typedoc", "rustdoc",

    // 其他工具
    "git", "ls", "dir", "type", "cat",
    "echo", "printf",
];
```

## 权限控制

### 权限矩阵

| 阶段 | ReadFile | WriteFile | ListFiles | RunCommand | 说明 |
|------|----------|-----------|-----------|------------|------|
| **Idea** | ❌ | ❌ | ❌ | ❌ | 不需要文件操作 |
| **PRD** | ❌ | ❌ | ❌ | ❌ | 只操作结构化数据 |
| **Design** | ❌ | ❌ | ❌ | ❌ | 只操作结构化数据 |
| **Plan** | ❌ | ❌ | ❌ | ❌ | 只操作结构化数据 |
| **Coding** | ✅ | ✅ | ✅ | ✅ | 需要完整的文件操作 |
| **Check** | ✅ | ❌ | ✅ | ✅ | 需要读取和验证 |
| **Delivery** | ❌ | ❌ | ✅ | ❌ | 只需要列出文件 |

### 权限最小化原则

```rust
/// 根据阶段获取允许的工具
pub fn get_allowed_tools(stage: Stage) -> Vec<String> {
    match stage {
        Stage::Idea => vec![
            "SaveIdeaTool".to_string(),
            "QueryMemoryTool".to_string(),
            "SaveInsightTool".to_string(),
        ],
        Stage::Prd => vec![
            "LoadIdeaTool".to_string(),
            "CreateRequirementTool".to_string(),
            "AddFeatureTool".to_string(),
            "SavePrdDocTool".to_string(),
        ],
        Stage::Coding => vec![
            "ReadFileTool".to_string(),
            "WriteFileTool".to_string(),
            "ListFilesTool".to_string(),
            "RunCommandTool".to_string(),
            "UpdateTaskStatusTool".to_string(),
        ],
        // ... 其他阶段
    }
}
```

## Artifact 工具安全

### Artifact 工具路径限制

```rust
/// Artifact 工具只能操作 artifacts 目录
fn validate_artifact_path(
    iteration_id: &str,
    artifact_name: &str,
) -> Result<PathBuf, String> {
    // 1. 获取 artifacts 目录
    let iteration_store = IterationStore::new();
    let iteration_dir = iteration_store.iterations_dir.join(iteration_id);
    let artifacts_dir = iteration_dir.join("artifacts");

    // 2. 验证文件名
    if !is_valid_artifact_name(artifact_name) {
        return Err(format!(
            "Invalid artifact name: '{}'. Valid names are: idea.md, prd.md, design.md, plan.md, delivery.md",
            artifact_name
        ));
    }

    // 3. 构造完整路径
    let artifact_path = artifacts_dir.join(artifact_name);

    // 4. 验证路径在 artifacts 目录内
    let canonical_path = artifact_path.canonicalize()
        .unwrap_or(artifact_path.clone());
    let canonical_artifacts = artifacts_dir.canonicalize()
        .unwrap_or(artifacts_dir.clone());

    if !canonical_path.starts_with(&canonical_artifacts) {
        return Err("Artifact path escapes artifacts directory".to_string());
    }

    Ok(artifact_path)
}

/// 验证 Artifact 名称
fn is_valid_artifact_name(name: &str) -> bool {
    matches!(
        name,
        "idea.md" | "prd.md" | "design.md" | "plan.md" | "delivery.md"
    )
}
```

### SaveIdeaTool 示例

```rust
pub struct SaveIdeaTool;

#[async_trait]
impl Tool for SaveIdeaTool {
    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        // 1. 获取参数
        let content = get_required_string_param(&args, "content")?;

        // 2. 获取 iteration ID
        let iteration_id = get_iteration_id()?;

        // 3. 验证并获取 Artifact 路径
        let artifact_path = validate_artifact_path(&iteration_id, "idea.md")
            .map_err(|e| adk_core::Error::Validation(e))?;

        // 4. 写入文件
        fs::write(&artifact_path, content)
            .map_err(|e| adk_core::Error::Io(e.to_string()))?;

        // 5. 返回结果
        Ok(json!({
            "artifact": "idea.md",
            "status": "saved",
            "path": artifact_path.to_string_lossy().to_string()
        }))
    }
}
```

## 部署安全

### CopyWorkspaceToProjectTool

```rust
pub struct CopyWorkspaceToProjectTool;

#[async_trait]
impl Tool for CopyWorkspaceToProjectTool {
    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
        // 1. 获取确认参数
        let confirm = args.get("confirm")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !confirm {
            return Err(adk_core::Error::Validation(
                "Copy operation requires confirm=true".to_string()
            ));
        }

        // 2. 获取 iteration workspace 路径
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
            }
        }

        // 6. 返回结果
        Ok(json!({
            "status": "success",
            "copied_files": copied_files,
            "count": copied_files.len()
        }))
    }
}
```

## 安全最佳实践

### 1. 始终验证路径

```rust
// ❌ 不好的做法
let path = args.get("path").unwrap();
let full_path = PathBuf::from(path);
fs::write(&full_path, content)?;

// ✅ 好的做法
let path = get_required_string_param(&args, "path")?;
let safe_path = validate_path_security_within_workspace(path, &workspace_dir)?;
let full_path = workspace_dir.join(&safe_path);
fs::write(&full_path, content)?;
```

### 2. 使用白名单而非黑名单

```rust
// ❌ 不好的做法
if command.contains("rm") {
    return Err("Command not allowed");
}

// ✅ 好的做法
const ALLOWED_COMMANDS: &[&str] = &["cargo", "npm", "git", "pytest"];
if !ALLOWED_COMMANDS.contains(&command) {
    return Err("Command not allowed");
}
```

### 3. 设置超时

```rust
// ❌ 不好的做法
let output = Command::new(command).output()?;

// ✅ 好的做法
let timeout = Duration::from_secs(30);
let output = tokio::time::timeout(
    timeout,
    tokio::process::Command::new(command).output()
).await?;
```

### 4. 验证输入

```rust
// ❌ 不好的做法
let filename = args.get("filename").unwrap();
fs::write(filename, content)?;

// ✅ 好的做法
let filename = get_required_string_param(&args, "filename")?;
if !is_valid_filename(filename) {
    return Err("Invalid filename");
}
fs::write(&safe_path, content)?;
```

### 5. 记录所有操作

```rust
// 记录文件操作
log::info!("Writing file: {:?}, size: {}", path, content.len());

// 记录命令执行
log::info!("Executing command: {}, args: {:?}", command, args);

// 记录安全事件
log::warn!("Path validation failed: {}", error);
```

## 安全检查清单

在开发和维护 Cowork Forge 时，请确保：

- ✅ 所有文件操作都通过验证
- ✅ 所有命令执行都通过白名单验证
- ✅ 所有工具都遵循权限最小化原则
- ✅ 所有输入都经过验证和清理
- ✅ 所有操作都有适当的超时限制
- ✅ 所有敏感操作都有日志记录
- ✅ 所有错误都妥善处理
- ✅ 所有用户数据都得到保护

## 相关文档

- [架构概览](./overview.md)
- [Agent 系统](./agent-system.md)
- [工具系统](../development/tools.md)
- [Pipeline 流程](./pipeline.md)
- [Artifacts 验证](./artifacts-validation.md)