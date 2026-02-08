# 领域模型

## 概述

Cowork Forge 的领域模型基于领域驱动设计（DDD）原则，将业务领域的核心概念抽象为清晰的实体和值对象。通过良好的领域模型设计，Cowork Forge 能够准确表达业务逻辑，简化复杂度，提高代码的可维护性和可扩展性。

### 领域模型的作用

1. **业务逻辑封装**：将业务规则封装在领域模型中
2. **数据一致性**：通过领域模型保证数据的一致性
3. **代码可读性**：使用业务术语提高代码可读性
4. **测试友好**：领域模型易于单元测试
5. **演进支持**：支持业务需求的演进和变化

### 核心实体

```
┌──────────────┐
│   Project    │
│  项目实体    │
└──────┬───────┘
       │ 1..*
       ↓
┌──────────────┐
│  Iteration   │
│  迭代实体    │
└──────┬───────┘
       │ 1..*
       ↓
┌──────────────┐
│   Memory     │
│  记忆实体    │
└──────────────┘
```

## Project 实体

### 定义

Project 是项目的根实体，代表一个完整的软件开发项目。

```rust
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,                  // 项目唯一标识
    pub name: String,                // 项目名称
    pub description: String,         // 项目描述
    pub created_at: DateTime<Utc>,   // 创建时间
    pub updated_at: DateTime<Utc>,   // 更新时间
}

impl Project {
    /// 创建新项目
    pub fn new(name: String, description: String) -> Self {
        let id = format!("proj-{}", uuid::Uuid::new_v4());
        let now = Utc::now();

        Self {
            id,
            name,
            description,
            created_at: now,
            updated_at: now,
        }
    }

    /// 更新项目信息
    pub fn update(&mut self, name: Option<String>, description: Option<String>) {
        if let Some(name) = name {
            self.name = name;
        }
        if let Some(description) = description {
            self.description = description;
        }
        self.updated_at = Utc::now();
    }
}
```

### 业务规则

1. **项目名称**：不能为空，长度 1-100 字符
2. **项目描述**：不能为空，长度 1-500 字符
3. **项目 ID**：自动生成，格式为 `proj-{uuid}`
4. **时间戳**：创建和更新时间自动管理

### 项目状态

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectStatus {
    Active,     // 活跃
    Archived,   // 已归档
    Deleted,    // 已删除
}
```

## Iteration 实体

### 定义

Iteration 是迭代实体，代表一个完整的开发周期。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Iteration {
    pub id: String,                    // 迭代唯一标识
    pub project_id: String,            // 所属项目 ID
    pub name: String,                  // 迭代名称
    pub description: String,           // 迭代描述
    pub iteration_type: IterationType, // 迭代类型
    pub base_iteration_id: Option<String>, // 基础迭代 ID
    pub inheritance_mode: InheritanceMode, // 继承模式
    pub current_stage: Stage,          // 当前阶段
    pub status: IterationStatus,       // 迭代状态
    pub created_at: DateTime<Utc>,     // 创建时间
    pub updated_at: DateTime<Utc>,     // 更新时间
    pub started_at: Option<DateTime<Utc>>,   // 开始时间
    pub completed_at: Option<DateTime<Utc>>, // 完成时间
    pub change_description: String,    // 变更描述
}

impl Iteration {
    /// 创建 Genesis 迭代
    pub fn new_genesis(
        project_id: String,
        name: String,
        description: String,
    ) -> Self {
        let id = format!("iter-{}", chrono::Utc::now().timestamp());
        let now = Utc::now();

        Self {
            id,
            project_id,
            name,
            description,
            iteration_type: IterationType::Genesis,
            base_iteration_id: None,
            inheritance_mode: InheritanceMode::None,
            current_stage: Stage::Idea,
            status: IterationStatus::Draft,
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
            change_description: String::new(),
        }
    }

    /// 创建 Evolution 迭代
    pub fn new_evolution(
        project_id: String,
        base_iteration_id: String,
        name: String,
        description: String,
        change_description: String,
        inheritance_mode: InheritanceMode,
        start_stage: Stage,
    ) -> Self {
        let id = format!("iter-{}", chrono::Utc::now().timestamp());
        let now = Utc::now();

        Self {
            id,
            project_id,
            name,
            description,
            iteration_type: IterationType::Evolution,
            base_iteration_id: Some(base_iteration_id),
            inheritance_mode,
            current_stage: start_stage,
            status: IterationStatus::Draft,
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
            change_description,
        }
    }

    /// 启动迭代
    pub fn start(&mut self) -> Result<(), CoworkError> {
        if self.status != IterationStatus::Draft {
            return Err(CoworkError::InvalidIterationState(
                "Cannot start iteration: not in Draft state".to_string()
            ));
        }

        self.status = IterationStatus::Running;
        self.started_at = Some(Utc::now());
        self.updated_at = Utc::now();

        Ok(())
    }

    /// 暂停迭代
    pub fn pause(&mut self) -> Result<(), CoworkError> {
        if self.status != IterationStatus::Running {
            return Err(CoworkError::InvalidIterationState(
                "Cannot pause iteration: not in Running state".to_string()
            ));
        }

        self.status = IterationStatus::Paused;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// 继续迭代
    pub fn resume(&mut self) -> Result<(), CoworkError> {
        if self.status != IterationStatus::Paused {
            return Err(CoworkError::InvalidIterationState(
                "Cannot resume iteration: not in Paused state".to_string()
            ));
        }

        self.status = IterationStatus::Running;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// 完成迭代
    pub fn complete(&mut self) -> Result<(), CoworkError> {
        if self.status != IterationStatus::Running {
            return Err(CoworkError::InvalidIterationState(
                "Cannot complete iteration: not in Running state".to_string()
            ));
        }

        self.status = IterationStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();

        Ok(())
    }

    /// 标记迭代失败
    pub fn fail(&mut self) -> Result<(), CoworkError> {
        if self.status != IterationStatus::Running {
            return Err(CoworkError::InvalidIterationState(
                "Cannot fail iteration: not in Running state".to_string()
            ));
        }

        self.status = IterationStatus::Failed;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// 更新当前阶段
    pub fn advance_stage(&mut self, stage: Stage) -> Result<(), CoworkError> {
        if self.status != IterationStatus::Running {
            return Err(CoworkError::InvalidIterationState(
                "Cannot advance stage: iteration not running".to_string()
            ));
        }

        self.current_stage = stage;
        self.updated_at = Utc::now();

        Ok(())
    }
}
```

### 迭代类型

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IterationType {
    Genesis,   // 起源迭代
    Evolution, // 演化迭代
}
```

### 继承模式

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InheritanceMode {
    None,     // 不继承
    Full,     // 完全继承
    Partial,  // 部分继承
}
```

### 阶段

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Stage {
    Idea,     // 创意阶段
    Prd,      // 需求阶段
    Design,   // 设计阶段
    Plan,     // 计划阶段
    Coding,   // 编码阶段
    Check,    // 检查阶段
    Delivery, // 交付阶段
}

impl Stage {
    /// 获取下一个阶段
    pub fn next(&self) -> Option<Stage> {
        match self {
            Stage::Idea => Some(Stage::Prd),
            Stage::Prd => Some(Stage::Design),
            Stage::Design => Some(Stage::Plan),
            Stage::Plan => Some(Stage::Coding),
            Stage::Coding => Some(Stage::Check),
            Stage::Check => Some(Stage::Delivery),
            Stage::Delivery => None,
        }
    }

    /// 获取上一个阶段
    pub fn prev(&self) -> Option<Stage> {
        match self {
            Stage::Idea => None,
            Stage::Prd => Some(Stage::Idea),
            Stage::Design => Some(Stage::Prd),
            Stage::Plan => Some(Stage::Design),
            Stage::Coding => Some(Stage::Plan),
            Stage::Check => Some(Stage::Coding),
            Stage::Delivery => Some(Stage::Check),
        }
    }
}
```

### 迭代状态

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IterationStatus {
    Draft,     // 草稿
    Running,   // 运行中
    Paused,    // 已暂停
    Completed, // 已完成
    Failed,    // 失败
}
```

### 状态转换规则

``┌─────────┐
│  Draft  │
└────┬────┘
     │ start()
     ↓
┌─────────┐
│ Running │
└────┬────┘
     │ pause()
     ↓
┌─────────┐
│ Paused  │
└────┬────┘
     │ resume()
     ↓
┌─────────┐
│ Running │
└────┬────┘
     │
     ├─→ complete() ─→ ┌────────────┐
     │                  │ Completed  │
     │                  └────────────┘
     │
     └─→ fail() ──────→ ┌────────────┐
                        │   Failed    │
                        └────────────┘
```

## Memory 实体

### 定义

Memory 是记忆实体，用于存储项目或迭代的关键决策和学习内容。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,                  // 记忆唯一标识
    pub project_id: Option<String>,  // 所属项目 ID（可选）
    pub iteration_id: Option<String>, // 所属迭代 ID（可选）
    pub memory_type: MemoryType,     // 记忆类型
    pub content: String,             // 记忆内容
    pub tags: Vec<String>,           // 标签
    pub created_at: DateTime<Utc>,   // 创建时间
    pub updated_at: DateTime<Utc>,   // 更新时间
}

impl Memory {
    /// 创建项目级记忆
    pub fn new_project_memory(
        project_id: String,
        memory_type: MemoryType,
        content: String,
    ) -> Self {
        let id = format!("mem-{}", uuid::Uuid::new_v4());
        let now = Utc::now();

        Self {
            id,
            project_id: Some(project_id),
            iteration_id: None,
            memory_type,
            content,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// 创建迭代级记忆
    pub fn new_iteration_memory(
        project_id: String,
        iteration_id: String,
        memory_type: MemoryType,
        content: String,
    ) -> Self {
        let id = format!("mem-{}", uuid::Uuid::new_v4());
        let now = Utc::now();

        Self {
            id,
            project_id: Some(project_id),
            iteration_id: Some(iteration_id),
            memory_type,
            content,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// 添加标签
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    /// 更新内容
    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }
}
```

### 记忆类型

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryType {
    Decision,   // 决策
    Learning,   // 学习
    Issue,      // 问题
    Insight,    // 洞见
    Pattern,    // 模式
}
```

## 数据模型

### Requirements（需求）

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: String,              // 需求 ID
    pub title: String,           // 需求标题
    pub description: String,     // 需求描述
    pub priority: Priority,      // 优先级
    pub status: RequirementStatus, // 状态
    pub acceptance_criteria: Vec<String>, // 验收标准
    pub created_at: DateTime<Utc>, // 创建时间
    pub updated_at: DateTime<Utc>, // 更新时间
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Critical, // 关键
    High,     // 高
    Medium,   // 中
    Low,      // 低
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequirementStatus {
    Draft,      // 草稿
    Approved,   // 已批准
    InProgress, // 进行中
    Completed,  // 已完成
    Rejected,   // 已拒绝
}
```

### Feature（功能）

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub id: String,              // 功能 ID
    pub requirement_id: String,  // 关联需求 ID
    pub title: String,           // 功能标题
    pub description: String,     // 功能描述
    pub status: FeatureStatus,   // 状态
    pub created_at: DateTime<Utc>, // 创建时间
    pub updated_at: DateTime<Utc>, // 更新时间
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureStatus {
    Planned,    // 已计划
    InProgress, // 进行中
    Completed,  // 已完成
    Deferred,   // 已延期
}
```

### DesignComponent（设计组件）

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignComponent {
    pub id: String,              // 组件 ID
    pub name: String,            // 组件名称
    pub description: String,     // 组件描述
    pub component_type: ComponentType, // 组件类型
    pub dependencies: Vec<String>, // 依赖组件
    pub interfaces: Vec<Interface>, // 接口定义
    pub created_at: DateTime<Utc>, // 创建时间
    pub updated_at: DateTime<Utc>, // 更新时间
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentType {
    Module,     // 模块
    Class,      // 类
    Function,   // 函数
    Interface,  // 接口
    Service,    // 服务
    Database,   // 数据库
    Api,        // API
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interface {
    pub name: String,            // 接口名称
    pub input_type: String,      // 输入类型
    pub output_type: String,     // 输出类型
    pub description: String,     // 接口描述
}
```

### Task（任务）

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,              // 任务 ID
    pub title: String,           // 任务标题
    pub description: String,     // 任务描述
    pub priority: Priority,      // 优先级
    pub status: TaskStatus,      // 状态
    pub dependencies: Vec<String>, // 依赖任务
    pub estimated_hours: Option<f64>, // 预估工时
    pub created_at: DateTime<Utc>, // 创建时间
    pub updated_at: DateTime<Utc>, // 更新时间
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,       // 待办
    InProgress, // 进行中
    Completed,  // 已完成
    Blocked,    // 已阻塞
}
```

## 关系和约束

### Project - Iteration 关系

```
Project (1) ─────── (0..*) Iteration
```

**约束**：
- 一个迭代必须属于一个项目
- 一个项目可以有零个或多个迭代
- 删除项目时，不能删除已完成的迭代

### Iteration - Memory 关系

```
Iteration (1) ─────── (0..*) Memory
```

**约束**：
- 一个记忆可以属于一个项目或一个迭代
- 项目级记忆对所有迭代可见
- 迭代级记忆只对当前迭代可见

### Requirement - Feature 关系

```
Requirement (1) ─────── (0..*) Feature
```

**约束**：
- 一个功能必须关联一个需求
- 一个需求可以有零个或多个功能

### DesignComponent 依赖关系

```
DesignComponent ─────── (0..*) DesignComponent
```

**约束**：
- 组件依赖不能形成循环
- 组件依赖必须先定义

### Task 依赖关系

```
Task ─────── (0..*) Task
```

**约束**：
- 任务依赖不能形成循环
- 依赖任务必须先完成

## 持久化

### 项目存储

```rust
use std::path::PathBuf;

pub struct ProjectStore {
    projects_dir: PathBuf,
}

impl ProjectStore {
    pub fn new(base_dir: PathBuf) -> Self {
        let projects_dir = base_dir.join(".cowork-v2/projects");
        fs::create_dir_all(&projects_dir).unwrap();
        Self { projects_dir }
    }

    /// 保存项目
    pub fn save(&self, project: &Project) -> Result<(), CoworkError> {
        let project_file = self.projects_dir.join(&project.id).with_extension("json");
        let content = serde_json::to_string_pretty(project)?;
        fs::write(project_file, content)?;
        Ok(())
    }

    /// 加载项目
    pub fn load(&self, project_id: &str) -> Result<Project, CoworkError> {
        let project_file = self.projects_dir.join(project_id).with_extension("json");
        let content = fs::read_to_string(project_file)?;
        let project: Project = serde_json::from_str(&content)?;
        Ok(project)
    }

    /// 列出所有项目
    pub fn list(&self) -> Result<Vec<Project>, CoworkError> {
        let mut projects = Vec::new();

        for entry in fs::read_dir(&self.projects_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                let content = fs::read_to_string(&path)?;
                let project: Project = serde_json::from_str(&content)?;
                projects.push(project);
            }
        }

        Ok(projects)
    }
}
```

### 迭代存储

```rust
pub struct IterationStore {
    iterations_dir: PathBuf,
}

impl IterationStore {
    pub fn new() -> Self {
        let iterations_dir = PathBuf::from(".cowork-v2/iterations");
        fs::create_dir_all(&iterations_dir).unwrap();
        Self { iterations_dir }
    }

    /// 保存迭代
    pub fn save(&self, iteration: &Iteration) -> Result<(), CoworkError> {
        let iteration_dir = self.iterations_dir.join(&iteration.id);
        fs::create_dir_all(&iteration_dir)?;

        let iteration_file = iteration_dir.join("iteration.json");
        let content = serde_json::to_string_pretty(iteration)?;
        fs::write(iteration_file, content)?;

        Ok(())
    }

    /// 加载迭代
    pub fn load(&self, iteration_id: &str) -> Result<Iteration, CoworkError> {
        let iteration_dir = self.iterations_dir.join(iteration_id);
        let iteration_file = iteration_dir.join("iteration.json");
        let content = fs::read_to_string(iteration_file)?;
        let iteration: Iteration = serde_json::from_str(&content)?;
        Ok(iteration)
    }

    /// 列出项目的所有迭代
    pub fn list_by_project(&self, project_id: &str) -> Result<Vec<Iteration>, CoworkError> {
        let mut iterations = Vec::new();

        for entry in fs::read_dir(&self.iterations_dir)? {
            let entry = entry?;
            let iteration_dir = entry.path();
            let iteration_file = iteration_dir.join("iteration.json");

            if iteration_file.exists() {
                let content = fs::read_to_string(&iteration_file)?;
                let iteration: Iteration = serde_json::from_str(&content)?;

                if iteration.project_id == project_id {
                    iterations.push(iteration);
                }
            }
        }

        Ok(iterations)
    }
}
```

### 记忆存储

```rust
pub struct MemoryStore {
    memory_dir: PathBuf,
}

impl MemoryStore {
    pub fn new() -> Self {
        let memory_dir = PathBuf::from(".cowork-v2/memory");
        fs::create_dir_all(&memory_dir).unwrap();

        // 创建项目级和迭代级记忆目录
        fs::create_dir_all(memory_dir.join("project"))?;
        fs::create_dir_all(memory_dir.join("iterations"))?;

        Self { memory_dir }
    }

    /// 保存记忆
    pub fn save(&self, memory: &Memory) -> Result<(), CoworkError> {
        let memory_file = match memory.iteration_id {
            Some(iteration_id) => {
                let iteration_dir = self.memory_dir.join("iterations").join(&iteration_id);
                fs::create_dir_all(&iteration_dir)?;
                iteration_dir.join(&memory.id).with_extension("json")
            }
            None => {
                self.memory_dir.join("project").join(&memory.id).with_extension("json")
            }
        };

        let content = serde_json::to_string_pretty(memory)?;
        fs::write(memory_file, content)?;

        Ok(())
    }

    /// 查询记忆
    pub fn query(
        &self,
        project_id: Option<&str>,
        iteration_id: Option<&str>,
        memory_type: Option<MemoryType>,
    ) -> Result<Vec<Memory>, CoworkError> {
        let mut memories = Vec::new();

        // 搜索项目级记忆
        if let Some(pid) = project_id {
            let project_dir = self.memory_dir.join("project");
            if project_dir.exists() {
                for entry in fs::read_dir(&project_dir)? {
                    let entry = entry?;
                    let memory_file = entry.path();
                    let content = fs::read_to_string(&memory_file)?;
                    let memory: Memory = serde_json::from_str(&content)?;

                    if memory.project_id.as_deref() == Some(pid) {
                        if memory_type.is_none() || memory.memory_type == memory_type.unwrap() {
                            memories.push(memory);
                        }
                    }
                }
            }
        }

        // 搜索迭代级记忆
        if let Some(iid) = iteration_id {
            let iteration_dir = self.memory_dir.join("iterations").join(iid);
            if iteration_dir.exists() {
                for entry in fs::read_dir(&iteration_dir)? {
                    let entry = entry?;
                    let memory_file = entry.path();
                    let content = fs::read_to_string(&memory_file)?;
                    let memory: Memory = serde_json::from_str(&content)?;

                    if memory.iteration_id.as_deref() == Some(iid) {
                        if memory_type.is_none() || memory.memory_type == memory_type.unwrap() {
                            memories.push(memory);
                        }
                    }
                }
            }
        }

        Ok(memories)
    }
}
```

## 领域服务

### 迭代服务

```rust
pub struct IterationService {
    iteration_store: IterationStore,
    project_store: Arc<ProjectStore>,
}

impl IterationService {
    pub fn new(project_store: Arc<ProjectStore>) -> Self {
        Self {
            iteration_store: IterationStore::new(),
            project_store,
        }
    }

    /// 创建 Genesis 迭代
    pub fn create_genesis_iteration(
        &self,
        project_id: &str,
        name: String,
        description: String,
    ) -> Result<Iteration, CoworkError> {
        // 验证项目存在
        let _project = self.project_store.load(project_id)?;

        // 创建迭代
        let iteration = Iteration::new_genesis(
            project_id.to_string(),
            name,
            description,
        );

        // 保存迭代
        self.iteration_store.save(&iteration)?;

        Ok(iteration)
    }

    /// 创建 Evolution 迭代
    pub fn create_evolution_iteration(
        &self,
        project_id: &str,
        base_iteration_id: &str,
        name: String,
        description: String,
        change_description: String,
        inheritance_mode: InheritanceMode,
        start_stage: Stage,
    ) -> Result<Iteration, CoworkError> {
        // 验证项目存在
        let _project = self.project_store.load(project_id)?;

        // 验证基础迭代存在
        let _base_iteration = self.iteration_store.load(base_iteration_id)?;

        // 创建迭代
        let iteration = Iteration::new_evolution(
            project_id.to_string(),
            base_iteration_id.to_string(),
            name,
            description,
            change_description,
            inheritance_mode,
            start_stage,
        );

        // 保存迭代
        self.iteration_store.save(&iteration)?;

        Ok(iteration)
    }
}
```

## 相关文档

- [架构概览](../architecture/overview.md)
- [迭代架构](../architecture/iteration-architecture.md)
- [Pipeline 流程](../architecture/pipeline.md)
- [工具系统](./tools.md)