# Cowork 严重问题修复说明

## 🚨 发现的严重问题

### 问题描述

用户使用 `cowork modify` 命令想要"把页面改为中文"，但系统：

1. **错误地创建新文件而非修改现有文件**
   - 显示的变更方案是 `create` 而不是 `modify`
   - 这些文件之前已经存在

2. **完全忽略了用户的修改意图**
   - 用户说"改为中文"
   - 但系统完全重新生成了项目，变成了另一个完全不相干的项目

3. **用户数据被覆盖**
   - 原有的项目文件被新生成的内容完全覆盖
   - 这是**数据丢失**级别的严重问题

---

## 🔍 根本原因分析

### 问题 1：CodePlanner 缺少 Feedback 上下文

**代码位置**：`crates/cowork-core/src/agents/code_planner.rs:41-47`

```rust
pub async fn execute(
    &self, 
    session_id: &str,
    prd_artifact: &PRDArtifact,
    design_artifact: &DesignDocArtifact,
    plan_artifact: &PlanArtifact    // ❌ 缺少 feedback_context 参数
) -> Result<CodeChangeArtifact>
```

**问题**：
- CodePlanner 只接收 PRD、Design、Plan 三个 artifact
- 用户通过 `cowork modify` 提交的修改需求（"改为中文"）被 FeedbackAgent 分析后**没有传递给 CodePlanner**
- CodePlanner 以为是第一次生成代码，所以全部标记为 `create`

**执行流程（错误的）**：
```
用户: "改为中文"
  ↓
FeedbackAgent 分析: delta=[Coding], rerun=[Coding]
  ↓
Orchestrator 重跑 Coding 阶段
  ↓
CodePlanner.execute(prd, design, plan)  // ❌ 没有 feedback 信息
  ↓
CodePlanner: "我要按照 PRD 从头生成代码" // ❌ 不知道这是修改
  ↓
生成的 changes: [create index.html, create styles.css, create scripts.js]
  ↓
CodeExecutor 执行: 覆盖所有文件 // 🔥 数据丢失！
```

---

### 问题 2：CodePlanner 没有检测现有文件

**代码位置**：`crates/cowork-core/src/agents/code_planner.rs:88-189`

```rust
async fn analyze_project_structure(&self, session_id: &str) -> Result<String> {
    // ...
    "existing_files": ["list of important files"],  // ✅ 有这个字段
    // ...
}
```

**问题**：
- `analyze_project_structure` 确实会分析现有文件
- 但是 `generate_code_plan` 在生成 `changes` 时**没有利用这个信息**
- 没有逻辑判断"如果文件已存在，应该是 modify 而非 create"

---

### 问题 3：modify_and_rerun 没有保存修改意图

**代码位置**：`crates/cowork-core/src/orchestrator/mod.rs:900-988`

```rust
pub async fn modify_and_rerun(
    &self,
    session_id: &str,
    modification: &str,  // ✅ 有用户的修改意图
    model_config: &ModelConfig,
) -> Result<()> {
    // ...
    let feedback_artifact = feedback_agent.execute(/*...*/).await?;
    
    // ❌ feedback_artifact 保存后，没有传递给后续的 CodePlanner
    
    self.run_workflow_from_stage(session_id, model_config, Some(earliest_stage)).await?;
    // ↑ 这里 CodePlanner 不知道有 feedback
}
```

---

## 🛠️ 修复方案

### 方案 A：在 SessionMeta 中保存修改意图（推荐）

**优点**：
- 简单直接
- 不需要修改 CodePlanner 的接口
- 所有 Agent 都能访问到修改意图

**实现**：

1. **修改 SessionMeta 结构**

```rust
pub struct SessionMeta {
    pub session_id: String,
    pub created_at: DateTime<Utc>,
    pub current_stage: Option<Stage>,
    pub stage_status: HashMap<Stage, StageStatus>,
    pub completed_stages: Vec<Stage>,
    pub feedback_iterations: usize,
    pub max_feedback_iterations: usize,
    
    // 🆕 新增：保存用户的修改需求上下文
    #[serde(default)]
    pub modification_context: Option<String>,
}
```

2. **在 modify_and_rerun 中保存**

```rust
// 在 modify_and_rerun 方法中
meta.modification_context = Some(modification.to_string());
self.save_session_meta(&meta)?;
```

3. **在 CodePlanner 中读取**

```rust
// 在 CodePlanner.execute 中
let meta = self.load_session_meta(session_id)?;
let modification_hint = meta.modification_context.as_deref();

let context = if let Some(modification) = modification_hint {
    format!(
        "**IMPORTANT: This is a MODIFICATION task, not creating from scratch!**\n\
         User wants: {}\n\
         Please MODIFY existing files instead of creating new ones.\n\n\
         {}",
        modification,
        normal_context
    )
} else {
    normal_context
};
```

4. **CodePlanner 的 prompt 需要强调**

```rust
r#"You are a code change planner.

**CRITICAL RULES:**
1. Check the project_context for "existing_files"
2. If a file already exists, use "kind": "modify", NOT "create"
3. If user provides modification instructions, RESPECT them
4. Do NOT regenerate the entire project unless explicitly asked

**Modification Instructions (if any):**
{modification_context}

**Project Analysis:**
{project_context}

Based on the above, generate a code change plan..."#
```

---

### 方案 B：修改 CodePlanner 接口（更彻底）

```rust
pub async fn execute(
    &self, 
    session_id: &str,
    prd_artifact: &PRDArtifact,
    design_artifact: &DesignDocArtifact,
    plan_artifact: &PlanArtifact,
    feedback_context: Option<&str>,  // 🆕 新增
) -> Result<CodeChangeArtifact>
```

**缺点**：
- 需要修改所有调用 CodePlanner.execute 的地方（约 6 处）
- 比较繁琐

---

### 方案 C：CodePlanner 自动读取最新的 Feedback artifact

```rust
// 在 CodePlanner.execute 中
let latest_feedback = self.store.list(session_id)?
    .iter()
    .filter(|a| a.stage == Stage::Feedback)
    .max_by_key(|a| a.created_at);

if let Some(feedback_meta) = latest_feedback {
    let feedback_artifact = self.load_artifact::<FeedbackArtifact>(...)?;
    // 使用 feedback_artifact.data.delta 中的信息
}
```

**缺点**：
- 逻辑复杂
- 可能读取到旧的 Feedback

---

## 🚀 推荐实施步骤

### 立即修复（方案 A）

1. **修改 `SessionMeta` 结构**
   - 添加 `modification_context: Option<String>`

2. **修改 `modify_and_rerun` 方法**
   - 保存用户的修改意图到 meta

3. **修改 `CodePlanner` 的 prompt**
   - 读取 `modification_context`
   - 强调"这是修改任务，检查现有文件"

4. **修改 `generate_code_plan` 的逻辑**
   - 根据 `existing_files` 判断是 `create` 还是 `modify`

5. **添加安全检查**
   - 如果 `kind` 是 `create` 但文件已存在，警告用户

---

## ⚠️ 临时缓解措施

在修复完成前，用户可以：

1. **不要使用 `cowork modify`，而是手动编辑文件**
2. **在使用 `cowork modify` 前备份项目**
   ```bash
   cp -r project project.backup
   ```
3. **使用 Git 版本控制**
   ```bash
   git init
   git add .
   git commit -m "Before cowork modify"
   # 如果修改出错，可以回退
   git reset --hard HEAD
   ```

---

## 📊 影响范围

### 受影响的命令
- `cowork modify` - **完全不可用**
- Feedback 阶段的自动重跑 - **部分问题**

### 受影响的用户场景
- ❌ 需求微调（改文字、改颜色等）
- ❌ 技术方案调整后的代码更新
- ❌ Bug 修复后的重新生成

---

## 🎯 修复目标

修复后的正确流程：

```
用户: "改为中文"
  ↓
FeedbackAgent 分析: delta=[Coding], rerun=[Coding]
  ↓
Orchestrator: meta.modification_context = "改为中文"
  ↓
CodePlanner.execute() 读取 meta.modification_context
  ↓
CodePlanner: "用户要改为中文，检查现有文件..."
  ↓
生成的 changes: [modify index.html (改为中文), modify styles.css, ...]
  ↓
CodeExecutor: 只修改必要的内容，不覆盖整个文件 ✅
```

---

## 🔥 优先级：P0（最高）

**原因**：
- 会导致用户数据丢失
- 完全破坏了 `cowork modify` 的核心功能
- 影响所有使用 Feedback 功能的用户

**建议**：
- 立即修复
- 添加回归测试
- 在修复前暂时禁用或警告 `cowork modify` 命令
