# Artifacts 验证

## 概述

Artifacts 验证是 Cowork Forge Pipeline 层的关键机制，用于确保每个阶段都生成了必需的制品。通过自动检查文件是否存在、格式是否正确、内容是否完整，Artifacts 验证机制能够在早期发现问题，避免进入错误的下一个阶段，提高开发流程的可靠性和效率。

### Artifacts 的作用

Artifacts 是各阶段的主要输出，用于：

1. **阶段间传递**：作为下一个阶段的输入
2. **质量保证**：证明阶段已完成且输出正确
3. **版本管理**：作为迭代的版本化制品
4. **用户审查**：供用户通过 HITL 审查
5. **文档归档**：作为项目的完整文档

### 验证目标

1. **存在性验证**：确认 Artifacts 文件已生成
2. **格式验证**：确认文件格式正确
3. **内容验证**：确认内容完整且有意义
4. **一致性验证**：确认与其他 Artifacts 一致

## Artifacts 类型

### 文档 Artifacts

| Artifact | 阶段 | 文件路径 | 说明 |
|----------|------|---------|------|
| **idea.md** | Idea | `artifacts/idea.md` | 创意文档 |
| **prd.md** | PRD | `artifacts/prd.md` | 产品需求文档 |
| **design.md** | Design | `artifacts/design.md` | 设计文档 |
| **plan.md** | Plan | `artifacts/plan.md` | 实施计划 |
| **delivery.md** | Delivery | `artifacts/delivery.md` | 交付报告 |

### 结构化数据 Artifacts

| Artifact | 阶段 | 文件路径 | 说明 |
|----------|------|---------|------|
| **requirements.json** | PRD | `data/requirements.json` | 需求列表 |
| **feature_list.json** | PRD | `data/feature_list.json` | 功能列表 |
| **design_spec.json** | Design | `data/design_spec.json` | 设计规范 |
| **implementation_plan.json** | Plan | `data/implementation_plan.json` | 实施计划 |
| **code_metadata.json** | Coding | `data/code_metadata.json` | 代码元数据 |

### 代码 Artifacts

| Artifact | 阶段 | 文件路径 | 说明 |
|----------|------|---------|------|
| **源代码** | Coding | `workspace/src/*` | 源代码文件 |
| **组件** | Coding | `workspace/components/*` | 组件文件 |
| **测试** | Coding | `workspace/tests/*` | 测试文件 |
| **配置** | Coding | `workspace/config/*` | 配置文件 |

## 验证机制

### 验证流程

```
Agent 执行完成
    ↓
返回 StageResult::Success
    ↓
Pipeline 接收结果
    ↓
检查 Artifacts 是否生成
    ↓
    ├─→ 存在 → 继续
    └─→ 不存在 → 自动重试（最多 3 次）
    ↓
重试后仍失败 → 阶段失败
```

### 验证实现

```rust
impl IterationExecutor {
    /// 检查 Artifacts 是否存在
    async fn check_artifact_exists(
        &self,
        stage_name: &str,
        workspace: &Path,
    ) -> bool {
        let iteration_dir = workspace.parent().unwrap_or(workspace);
        let artifacts_dir = iteration_dir.join("artifacts");
        let data_dir = iteration_dir.join("data");

        match stage_name {
            "idea" => {
                // 检查 idea.md 是否存在
                artifacts_dir.join("idea.md").exists()
            }
            "prd" => {
                // 检查 prd.md 或 requirements.json 是否存在
                artifacts_dir.join("prd.md").exists() ||
                data_dir.join("requirements.json").exists()
            }
            "design" => {
                // 检查 design.md 或 design_spec.json 是否存在
                artifacts_dir.join("design.md").exists() ||
                data_dir.join("design_spec.json").exists()
            }
            "plan" => {
                // 检查 plan.md 或 implementation_plan.json 是否存在
                artifacts_dir.join("plan.md").exists() ||
                data_dir.join("implementation_plan.json").exists()
            }
            "coding" => {
                // 检查 workspace 中是否有代码文件
                self.check_code_files_exist(workspace)
            }
            "delivery" => {
                // 检查 delivery.md 是否存在
                artifacts_dir.join("delivery.md").exists()
            }
            _ => true,
        }
    }

    /// 检查代码文件是否存在
    fn check_code_files_exist(&self, workspace: &Path) -> bool {
        let code_extensions = [
            "rs", "js", "jsx", "ts", "tsx",
            "py", "java", "go", "cpp", "c", "h"
        ];

        for entry in walkdir::WalkDir::new(workspace)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if let Some(ext) = entry.path().extension() {
                if ext.to_str().map(|e| code_extensions.contains(&e)).unwrap_or(false) {
                    return true;
                }
            }
        }

        false
    }
}
```

### 阶段验证规则

| 阶段 | 必需 Artifacts | 可选 Artifacts | 验证逻辑 |
|------|---------------|---------------|---------|
| **Idea** | idea.md | 无 | 检查 idea.md 存在 |
| **PRD** | prd.md | requirements.json | 检查 prd.md 或 requirements.json 存在 |
| **Design** | design.md | design_spec.json | 检查 design.md 或 design_spec.json 存在 |
| **Plan** | plan.md | implementation_plan.json | 检查 plan.md 或 implementation_plan.json 存在 |
| **Coding** | 源代码文件 | code_metadata.json | 检查 workspace 中有代码文件 |
| **Check** | 无 | 无 | 不需要验证 |
| **Delivery** | delivery.md | 无 | 检查 delivery.md 存在 |

## 自动重试机制

### 重试流程

```
阶段执行
    ↓
验证 Artifacts
    ↓
    ├─→ 验证成功 → 继续
    └─→ 验证失败
        ↓
    重试计数器 +1
        ↓
    重试次数 < 3？
        ├─→ Yes → 等待 2s → 重新执行阶段
        └─→ No → 标记阶段失败
```

### 重试实现

```rust
impl Pipeline {
    /// 带重试执行阶段
    async fn execute_stage_with_retry(
        &self,
        context: &ExecutionContext,
        stage: Stage,
    ) -> Result<StageResult, CoworkError> {
        let max_retries = 3;
        let mut retry_count = 0;
        let mut last_error = None;

        loop {
            // 执行阶段
            let result = self.stage_executor.execute(context, stage).await;

            match result {
                Ok(stage_result) => {
                    // 验证 Artifacts
                    let artifact_exists = self.check_artifact_exists(
                        &stage.to_string(),
                        &context.workspace
                    ).await;

                    if artifact_exists {
                        // 验证成功，返回结果
                        return Ok(stage_result);
                    } else {
                        // Artifacts 未生成，记录错误
                        last_error = Some(format!(
                            "Artifacts not generated for stage '{}'",
                            stage
                        ));
                    }
                }
                Err(error) => {
                    // 执行错误，记录错误
                    last_error = Some(error.to_string());
                }
            }

            // 检查重试次数
            retry_count += 1;
            if retry_count >= max_retries {
                // 超过最大重试次数，返回错误
                return Err(CoworkError::ArtifactsNotFound(
                    last_error.unwrap_or_else(|| "Unknown error".to_string())
                ));
            }

            // 等待后重试（指数退避）
            let delay = Duration::from_secs(2 * retry_count as u64);
            tokio::time::sleep(delay).await;

            // 记录重试
            self.log_retry(context, &stage, retry_count, &last_error).await;
        }
    }
}
```

### 重试配置

| 配置项 | 值 | 说明 |
|-------|---|------|
| **最大重试次数** | 3 次 | 阶段失败后的最大重试次数 |
| **重试间隔** | 2s, 4s, 6s | 指数退避策略 |
| **重试等待** | 支持 | 支持等待后重试 |
| **重试日志** | 记录 | 记录每次重试信息 |

## 格式验证

### Markdown 文件格式验证

```rust
/// 验证 Markdown 文件格式
fn validate_markdown_format(path: &Path) -> Result<(), String> {
    // 1. 读取文件
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // 2. 检查文件大小
    if content.len() < 100 {
        return Err("File is too short (minimum 100 characters)".to_string());
    }

    // 3. 检查基本 Markdown 结构
    if !content.contains("#") {
        return Err("Missing markdown headers".to_string());
    }

    // 4. 检查空行过多
    let empty_lines = content.lines().filter(|l| l.trim().is_empty()).count();
    let total_lines = content.lines().count();
    if empty_lines as f64 / total_lines as f64 > 0.5 {
        return Err("Too many empty lines".to_string());
    }

    Ok(())
}
```

### JSON 文件格式验证

```rust
/// 验证 JSON 文件格式
fn validate_json_format(path: &Path) -> Result<(), String> {
    // 1. 读取文件
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // 2. 解析 JSON
    let value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    // 3. 检查是否为对象
    if !value.is_object() {
        return Err("JSON must be an object".to_string());
    }

    // 4. 检查是否为空对象
    if value.as_object().unwrap().is_empty() {
        return Err("JSON object is empty".to_string());
    }

    Ok(())
}
```

### 代码文件格式验证

```rust
/// 验证代码文件格式
fn validate_code_file(path: &Path) -> Result<(), String> {
    // 1. 读取文件
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // 2. 检查文件大小
    if content.len() < 10 {
        return Err("File is too short".to_string());
    }

    // 3. 检查基本代码结构
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| "Missing file extension".to_string())?;

    match ext {
        "rs" => {
            // Rust 文件验证
            if !content.contains("fn ") && !content.contains("struct ") && !content.contains("impl ") {
                return Err("Rust file missing function, struct, or impl".to_string());
            }
        }
        "js" | "jsx" | "ts" | "tsx" => {
            // JavaScript/TypeScript 文件验证
            if !content.contains("function ") && !content.contains("=>") && !content.contains("class ") {
                return Err("JavaScript/TypeScript file missing function, arrow, or class".to_string());
            }
        }
        "py" => {
            // Python 文件验证
            if !content.contains("def ") && !content.contains("class ") {
                return Err("Python file missing function or class".to_string());
            }
        }
        _ => {
            // 其他文件类型，不做验证
        }
    }

    Ok(())
}
```

## 内容验证

### Idea.md 内容验证

```rust
/// 验证 idea.md 内容
fn validate_idea_content(content: &str) -> Result<(), String> {
    // 1. 检查必需的章节
    let required_sections = [
        "# 项目背景",
        "# 核心功能",
        "# 目标用户",
        "# 技术方向",
    ];

    for section in &required_sections {
        if !content.contains(section) {
            return Err(format!("Missing required section: {}", section));
        }
    }

    // 2. 检查内容长度
    if content.len() < 500 {
        return Err("idea.md is too short (minimum 500 characters)".to_string());
    }

    Ok(())
}
```

### PRD.md 内容验证

```rust
/// 验证 prd.md 内容
fn validate_prd_content(content: &str) -> Result<(), String> {
    // 1. 检查必需的章节
    let required_sections = [
        "# 产品概述",
        "# 功能需求",
        "# 非功能需求",
        "# 验收标准",
    ];

    for section in &required_sections {
        if !content.contains(section) {
            return Err(format!("Missing required section: {}", section));
        }
    }

    // 2. 检查内容长度
    if content.len() < 1000 {
        return Err("prd.md is too short (minimum 1000 characters)".to_string());
    }

    // 3. 检查是否有需求列表
    if !content.contains("- ") && !content.contains("* ") {
        return Err("prd.md missing requirement list".to_string());
    }

    Ok(())
}
```

## 验证结果处理

### 成功处理

```rust
impl Pipeline {
    /// 处理验证成功
    async fn handle_validation_success(
        &self,
        context: &ExecutionContext,
        stage: Stage,
        result: StageResult,
    ) -> Result<(), CoworkError> {
        // 1. 记录成功
        self.log_stage_success(context, &stage).await;

        // 2. 更新迭代状态
        self.iteration_store.update_current_stage(
            &context.iteration_id,
            stage
        )?;

        // 3. 保存 Artifacts 元数据
        self.save_artifacts_metadata(context, &stage).await?;

        Ok(())
    }
}
```

### 失败处理

```rust
impl Pipeline {
    /// 处理验证失败
    async fn handle_validation_failure(
        &self,
        context: &ExecutionContext,
        stage: Stage,
        error: CoworkError,
    ) -> Result<(), CoworkError> {
        // 1. 记录失败
        self.log_stage_failure(context, &stage, &error).await;

        // 2. 保存错误信息
        self.save_error(context, &stage, &error).await;

        // 3. 通知用户
        self.interaction.show_message(
            MessageLevel::Error,
            format!("Stage '{}' failed: {}", stage, error)
        ).await;

        // 4. 决定是否继续或停止
        match stage {
            Stage::Idea | Stage::Prd | Stage::Design | Stage::Plan => {
                // 关键阶段失败，停止迭代
                Err(error)
            }
            Stage::Coding | Stage::Check | Stage::Delivery => {
                // 非关键阶段失败，可以继续
                Ok(())
            }
        }
    }
}
```

## 最佳实践

### 1. 明确定义验证规则

```rust
// ✅ 好的做法：明确定义验证规则
const IDEA_REQUIRED_SECTIONS: &[&str] = &[
    "# 项目背景",
    "# 核心功能",
    "# 目标用户",
];

fn validate_idea(content: &str) -> Result<(), String> {
    for section in IDEA_REQUIRED_SECTIONS {
        if !content.contains(section) {
            return Err(format!("Missing section: {}", section));
        }
    }
    Ok(())
}

// ❌ 不好的做法：模糊的验证
fn validate_idea(content: &str) -> Result<(), String> {
    if content.len() < 100 {
        return Err("Content too short".to_string());
    }
    Ok(())
}
```

### 2. 提供详细的错误信息

```rust
// ✅ 好的做法：详细的错误信息
Err(format!(
    "Missing required sections in prd.md. Required: {:?}, Found: {:?}",
    required_sections,
    found_sections
))

// ❌ 不好的做法：模糊的错误信息
Err("Invalid prd.md".to_string())
```

### 3. 支持增量验证

```rust
// ✅ 好的做法：支持增量验证
fn validate_prd_incremental(
    old_content: &str,
    new_content: &str,
) -> Result<(), String> {
    // 只验证新增或修改的部分
    let diff = compute_diff(old_content, new_content);
    validate_prd_section(&diff)
}

// ❌ 不好的做法：总是验证全部内容
fn validate_prd(content: &str) -> Result<(), String> {
    validate_prd_full(content)
}
```

### 4. 记录验证历史

```rust
// ✅ 好的做法：记录验证历史
pub struct ValidationHistory {
    pub timestamp: DateTime<Utc>,
    pub stage: Stage,
    pub result: ValidationResult,
    pub error: Option<String>,
}

self.validation_history.push(ValidationHistory {
    timestamp: Utc::now(),
    stage,
    result: ValidationResult::Failed,
    error: Some(error.to_string()),
});
```

## 验证配置

### 验证级别

```rust
pub enum ValidationLevel {
    /// 基本验证：只检查文件是否存在
    Basic,
    /// 格式验证：检查文件格式是否正确
    Format,
    /// 内容验证：检查文件内容是否完整
    Content,
    /// 严格验证：检查所有方面
    Strict,
}

impl Default for ValidationLevel {
    fn default() -> Self {
        ValidationLevel::Format
    }
}
```

### 验证配置

```rust
pub struct ValidationConfig {
    pub level: ValidationLevel,
    pub max_retries: u32,
    pub retry_delay_seconds: u64,
    pub enable_content_validation: bool,
    pub enable_format_validation: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            level: ValidationLevel::Format,
            max_retries: 3,
            retry_delay_seconds: 2,
            enable_content_validation: true,
            enable_format_validation: true,
        }
    }
}
```

## 相关文档

- [架构概览](./overview.md)
- [Pipeline 流程](./pipeline.md)
- [文件安全机制](./file-security.md)
- [Agent 系统](./agent-system.md)
- [迭代架构](./iteration-architecture.md)