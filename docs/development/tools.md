# 工具系统

## 概述

Cowork Forge 的工具系统基于权限最小化原则，为 Agent 提供专用工具，确保安全性和可控性。每个工具都有明确的职责、严格的输入验证和完善的错误处理机制。

### 设计原则

1. **权限最小化**：只提供完成任务所需的最小工具集
2. **专用工具**：每个工具只完成特定功能
3. **安全优先**：所有工具都经过安全验证
4. **类型安全**：使用强类型确保参数正确性
5. **错误处理**：提供清晰的错误信息

## 工具分类

### 1. Artifact 工具

用于保存和加载各阶段的文档制品。

| 工具 | 功能 | 使用阶段 |
|------|------|---------|
| `SaveIdeaTool` | 保存 idea.md | Idea |
| `SavePrdDocTool` | 保存 prd.md | PRD |
| `SaveDesignDocTool` | 保存 design.md | Design |
| `SavePlanDocTool` | 保存 plan.md | Plan |
| `SaveDeliveryReportTool` | 保存 delivery.md | Delivery |
| `LoadIdeaTool` | 加载 idea.md | PRD, Delivery |
| `LoadPrdDocTool` | 加载 prd.md | Design, Plan, Delivery |
| `LoadDesignDocTool` | 加载 design.md | Delivery |

### 2. 文件操作工具

用于在 workspace 中操作代码文件。

| 工具 | 功能 | 使用阶段 |
|------|------|---------|
| `ReadFileTool` | 读取文件 | Coding, Check |
| `WriteFileTool` | 写入文件 | Coding |
| `ListFilesTool` | 列出文件 | Coding, Check, Delivery |

### 3. 命令执行工具

用于运行构建和测试命令。

| 工具 | 功能 | 使用阶段 |
|------|------|---------|
| `RunCommandTool` | 执行命令 | Coding, Check |

### 4. 数据管理工具

用于管理需求、功能、设计等结构化数据。

| 工具 | 功能 | 使用阶段 |
|------|------|---------|
| `CreateRequirementTool` | 创建需求 | PRD |
| `AddFeatureTool` | 添加功能 | PRD |
| `UpdateRequirementTool` | 更新需求 | PRD |
| `DeleteRequirementTool` | 删除需求 | PRD |
| `GetRequirementsTool` | 获取需求 | PRD, Design, Check |
| `CreateDesignComponentTool` | 创建设计组件 | Design |
| `GetDesignTool` | 获取设计 | Design, Check |
| `CreateTaskTool` | 创建任务 | Plan |
| `GetPlanTool` | 获取计划 | Coding, Check |
| `UpdateTaskStatusTool` | 更新任务状态 | Coding |
| `UpdateFeatureStatusTool` | 更新功能状态 | Coding |

### 5. 验证工具

用于质量检查和验证。

| 工具 | 功能 | 使用阶段 |
|------|------|---------|
| `CheckTestsTool` | 检查测试 | Coding, Check |
| `CheckLintTool` | 检查代码质量 | Check |
| `CheckFeatureCoverageTool` | 检查功能覆盖度 | Check |
| `CheckTaskDependenciesTool` | 检查任务依赖 | Check |
| `CheckDataFormatTool` | 检查数据格式 | Check |

### 6. 交互工具

用于人机交互和反馈。

| 工具 | 功能 | 使用阶段 |
|------|------|---------|
| `ProvideFeedbackTool` | 提供反馈 | 所有 Critic |
| `GotoStageTool` | 跳转到阶段 | Check |

### 7. 部署工具

用于代码部署。

| 工具 | 功能 | 使用阶段 |
|------|------|---------|
| `CopyWorkspaceToProjectTool` | 复制代码到项目根 | Delivery |

### 8. 记忆工具

用于查询和保存记忆。

| 工具 | 功能 | 使用阶段 |
|------|------|---------|
| `QueryMemoryTool` | 查询记忆 | 所有阶段 |
| `SaveInsightTool` | 保存洞见 | 所有阶段 |
| `SaveLearningTool` | 保存学习 | Design, Plan, Coding, Delivery |
| `SaveIssueTool` | 保存问题 | Design, Plan, Coding, Delivery |

## 工具接口

### Tool Trait

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    /// 工具名称
    fn name(&self) -> &str;

    /// 工具描述
    fn description(&self) -> &str;

    /// 工具参数模式
    fn parameters(&self) -> serde_json::Value;

    /// 执行工具
    async fn execute(
        &self,
        ctx: Arc<dyn ToolContext>,
        args: Value,
    ) -> adk_core::Result<Value>;
}
```

### ToolContext

```rust
#[async_trait]
pub trait ToolContext: Send + Sync {
    /// 获取迭代 ID
    fn get_iteration_id(&self) -> Result<String, CoworkError>;

    /// 获取工作空间路径
    fn get_workspace_path(&self) -> Result<PathBuf, CoworkError>;

    /// 记录日志
    async fn log(&self, level: LogLevel, message: &str);
}
```

## 文件工具实现

### ReadFileTool

```rust
pub struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read a file from the iteration workspace"
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Relative path to the file"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(
        &self,
        _ctx: Arc<dyn ToolContext>,
        args: Value,
    ) -> adk_core::Result<Value> {
        let path = get_required_string_param(&args, "path")?;
        let iteration_id = get_iteration_id()?;
        let workspace_dir = IterationStore::new().workspace_path(&iteration_id)?;
        let safe_path = validate_path_security_within_workspace(path, &workspace_dir)?;
        let full_path = workspace_dir.join(&safe_path);

        let content = fs::read_to_string(&full_path)
            .map_err(|e| adk_core::Error::Io(e.to_string()))?;

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
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file in the iteration workspace"
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Relative path to the file"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write"
                }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(
        &self,
        _ctx: Arc<dyn ToolContext>,
        args: Value,
    ) -> adk_core::Result<Value> {
        let path = get_required_string_param(&args, "path")?;
        let content = get_required_string_param(&args, "content")?;
        let iteration_id = get_iteration_id()?;
        let workspace_dir = IterationStore::new().workspace_path(&iteration_id)?;
        let safe_path = validate_path_security_within_workspace(path, &workspace_dir)?;
        let full_path = workspace_dir.join(&safe_path);

        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| adk_core::Error::Io(e.to_string()))?;
        }

        fs::write(&full_path, content)
            .map_err(|e| adk_core::Error::Io(e.to_string()))?;

        Ok(json!({
            "path": path,
            "size": content.len(),
            "status": "success"
        }))
    }
}
```

## 数据工具实现

### CreateRequirementTool

```rust
pub struct CreateRequirementTool;

#[async_trait]
impl Tool for CreateRequirementTool {
    fn name(&self) -> &str {
        "create_requirement"
    }

    fn description(&self) -> &str {
        "Create a new requirement"
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "title": {
                    "type": "string",
                    "description": "Requirement title"
                },
                "description": {
                    "type": "string",
                    "description": "Requirement description"
                },
                "priority": {
                    "type": "string",
                    "enum": ["Critical", "High", "Medium", "Low"],
                    "description": "Requirement priority"
                },
                "acceptance_criteria": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "List of acceptance criteria"
                }
            },
            "required": ["title", "description", "priority"]
        })
    }

    async fn execute(
        &self,
        _ctx: Arc<dyn ToolContext>,
        args: Value,
    ) -> adk_core::Result<Value> {
        let title = get_required_string_param(&args, "title")?;
        let description = get_required_string_param(&args, "description")?;
        let priority_str = get_required_string_param(&args, "priority")?;
        let acceptance_criteria = args.get("acceptance_criteria")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>())
            .unwrap_or_default();

        let priority = match priority_str.as_str() {
            "Critical" => Priority::Critical,
            "High" => Priority::High,
            "Medium" => Priority::Medium,
            "Low" => Priority::Low,
            _ => return Err(adk_core::Error::Validation("Invalid priority".to_string())),
        };

        let iteration_id = get_iteration_id()?;
        let iteration_store = IterationStore::new();

        let requirement = Requirement {
            id: format!("req-{}", uuid::Uuid::new_v4()),
            title,
            description,
            priority,
            status: RequirementStatus::Draft,
            acceptance_criteria,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        iteration_store.add_requirement(&iteration_id, &requirement)?;

        Ok(json!({
            "id": requirement.id,
            "status": "created"
        }))
    }
}
```

### GetRequirementsTool

```rust
pub struct GetRequirementsTool;

#[async_trait]
impl Tool for GetRequirementsTool {
    fn name(&self) -> &str {
        "get_requirements"
    }

    fn description(&self) -> &str {
        "Get all requirements for the current iteration"
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    async fn execute(
        &self,
        _ctx: Arc<dyn ToolContext>,
        _args: Value,
    ) -> adk_core::Result<Value> {
        let iteration_id = get_iteration_id()?;
        let iteration_store = IterationStore::new();
        let requirements = iteration_store.get_requirements(&iteration_id)?;

        Ok(json!(requirements))
    }
}
```

## 验证工具实现

### CheckTestsTool

```rust
pub struct CheckTestsTool;

#[async_trait]
impl Tool for CheckTestsTool {
    fn name(&self) -> &str {
        "check_tests"
    }

    fn description(&self) -> &str {
        "Check if tests pass"
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "test_command": {
                    "type": "string",
                    "description": "Command to run tests"
                }
            },
            "required": ["test_command"]
        })
    }

    async fn execute(
        &self,
        _ctx: Arc<dyn ToolContext>,
        args: Value,
    ) -> adk_core::Result<Value> {
        let test_command = get_required_string_param(&args, "test_command")?;
        let iteration_id = get_iteration_id()?;
        let workspace_dir = IterationStore::new().workspace_path(&iteration_id)?;

        let output = Command::new("sh")
            .arg("-c")
            .arg(&test_command)
            .current_dir(&workspace_dir)
            .output()
            .map_err(|e| adk_core::Error::Io(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let passed = output.status.success();

        Ok(json!({
            "passed": passed,
            "exit_code": output.status.code(),
            "stdout": stdout,
            "stderr": stderr
        }))
    }
}
```

## 部署工具实现

### CopyWorkspaceToProjectTool

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
        let confirm = args.get("confirm")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !confirm {
            return Err(adk_core::Error::Validation(
                "Copy operation requires confirm=true".to_string()
            ));
        }

        let iteration_id = get_iteration_id()?;
        let workspace_dir = IterationStore::new().workspace_path(&iteration_id)?;
        let project_root = std::env::current_dir()
            .map_err(|e| adk_core::Error::Io(e.to_string()))?;

        let extensions_to_copy = vec![
            ".html", ".htm", ".css", ".js", ".jsx", ".ts", ".tsx",
            ".json", ".md", ".txt", ".svg", ".png", ".jpg", ".jpeg",
        ];

        let mut copied_files = Vec::new();
        for entry in walkdir::WalkDir::new(&workspace_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let src_path = entry.path();
            let rel_path = src_path.strip_prefix(&workspace_dir)
                .map_err(|e| adk_core::Error::Io(e.to_string()))?;

            let should_copy = src_path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| extensions_to_copy.contains(&format!(".{}", ext)))
                .unwrap_or(false);

            if should_copy {
                let dest_path = project_root.join(rel_path);

                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| adk_core::Error::Io(e.to_string()))?;
                }

                fs::copy(&src_path, &dest_path)
                    .map_err(|e| adk_core::Error::Io(e.to_string()))?;

                copied_files.push(rel_path.to_string_lossy().to_string());
            }
        }

        Ok(json!({
            "status": "success",
            "copied_files": copied_files,
            "count": copied_files.len()
        }))
    }
}
```

## 工具配置

### 工具注册

```rust
// 在 agents/mod.rs 中注册工具
.tool(Arc::new(SaveIdeaTool))
.tool(Arc::new(LoadIdeaTool))
.tool(Arc::new(CreateRequirementTool))
.tool(Arc::new(GetRequirementsTool))
.tool(Arc::new(CheckTestsTool))
.tool(Arc::new(CopyWorkspaceToProjectTool))
```

### 工具权限

| 工具类型 | 使用阶段 | 权限级别 |
|---------|---------|---------|
| Artifact 工具 | 所有阶段 | 低（只读写 artifacts） |
| 文件工具 | Coding, Check | 中（读写 workspace） |
| 命令工具 | Coding, Check | 高（执行命令） |
| 数据工具 | 所有阶段 | 低（读写 data） |
| 验证工具 | Coding, Check | 低（只读） |
| 部署工具 | Delivery | 高（修改项目根） |

## 工具最佳实践

### 1. 参数验证

```rust
// ✅ 好的做法：严格验证参数
let path = get_required_string_param(&args, "path")?;
if path.is_empty() {
    return Err(adk_core::Error::Validation("Path cannot be empty".to_string()));
}

// ❌ 不好的做法：不验证参数
let path = args.get("path").unwrap().as_str().unwrap();
```

### 2. 错误处理

```rust
// ✅ 好的做法：提供清晰的错误信息
fs::write(&full_path, content)
    .map_err(|e| adk_core::Error::Io(format!("Failed to write file {}: {}", path, e)))?;

// ❌ 不好的做法：模糊的错误信息
fs::write(&full_path, content).map_err(|e| adk_core::Error::Io(e.to_string()))?;
```

### 3. 安全验证

```rust
// ✅ 好的做法：验证路径安全性
let safe_path = validate_path_security_within_workspace(path, &workspace_dir)?;
let full_path = workspace_dir.join(&safe_path);

// ❌ 不好的做法：直接使用用户输入
let full_path = PathBuf::from(path);
```

## 相关文档

- [架构概览](../architecture/overview.md)
- [Agent 系统](../architecture/agent-system.md)
- [文件安全机制](../architecture/file-security.md)
- [领域模型](./domain.md)