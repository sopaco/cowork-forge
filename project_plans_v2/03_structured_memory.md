# 方案 03: 双层架构记忆系统

**版本**: 1.0  
**创建日期**: 2026-02-02  
**Phase**: Phase 0.5  
**工作量**: 24h  
**优先级**: P0 (必须)

---

## 📋 概述

### 1.1 需求背景

**当前问题**:
- Cowork 覆盖项目全生命周期，但项目经验无法有效沉淀
- Agent 无法基于历史经验做决策
- 缺少项目记忆和知识复用能力
- 与普通 AI IDE 相比，缺乏差异化优势

**目标**:
- 构建双层架构记忆系统（主 Memory + Session Memory）
- Agent 能够查询历史经验，基于历史做决策
- 项目经验能够沉淀和复用
- 保持简单性和可控性

### 1.2 设计原则

- ✅ 系统定义结构，保持可控性
- ✅ Agent 只需查询索引，负担轻
- ✅ 简约且立体的索引（时间、需求、状态）
- ✅ 易于维护和扩展

---

## 🏗️ 技术方案

### 2.1 双层架构

```
┌─────────────────────────────────────────────────────────┐
│                     记忆系统架构                          │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  主 Memory (Project Memory)                              │
│  ├─ 项目级关键决策                                       │
│  ├─ 跨 session 的经验                                    │
│  ├─ 通用的模式和最佳实践                                 │
│  └─ 项目时间线                                          │
│                                                          │
│  Session Memory (Session 维度)                          │
│  ├─ 该 session 的所有决策                               │
│  ├─ 该 session 的所有经验                               │
│  ├─ 该 session 的详细记录                               │
│  └─ 该 session 的状态变化                               │
│                                                          │
│  记忆索引 (JSON)                                         │
│  ├─ 主 Memory 索引                                      │
│  ├─ Session Memory 索引                                 │
│  └─ 多维查询支持                                        │
│                                                          │
│  Agent 工具 (5个)                                        │
│  ├─ QueryMemoryIndexTool                               │
│  ├─ LoadMemoryDetailTool                               │
│  ├─ SaveSessionMemoryTool                              │
│  ├─ PromoteToProjectMemoryTool                         │
│  └─ GetMemoryContextTool                                │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### 2.2 存储结构

```
.cowork/
├── memory/
│   ├── project_memory.json           # 主 Memory（索引 + 摘要）
│   ├── project_memory/
│   │   ├── decisions/                # 项目级决策详情
│   │   ├── experiences/              # 项目级经验详情
│   │   └── patterns/                 # 通用模式详情
│   ├── sessions/
│   │   ├── session-xxx.json          # Session Memory（索引 + 摘要）
│   │   └── sessions/
│   │       └── session-xxx/
│   │           ├── decisions/        # Session 决策详情
│   │           ├── experiences/      # Session 经验详情
│   │           └── records/          # Session 详细记录
│   └── timeline.json                # 项目时间线
```

### 2.3 主 Memory 索引

```json
{
  "project_id": "project-001",
  "project_name": "我的项目",
  "schema_version": "1.0",
  "created_at": "2026-02-01T10:00:00Z",
  "updated_at": "2026-02-02T16:00:00Z",
  
  "key_decisions": [
    {
      "id": "DEC-001",
      "title": "选择使用 React 作为前端框架",
      "category": "技术选型",
      "summary": "经过对 React、Vue、Svelte 的对比，选择 React 因为团队熟悉且生态丰富",
      "session_id": "session-001",
      "stage": "design",
      "created_at": "2026-02-02T10:30:00Z",
      "impact": "high",
      "status": "implemented",
      "file": "project_memory/decisions/DEC-001.md",
      "tags": ["前端", "React", "技术选型"]
    }
  ],
  
  "key_experiences": [...],
  "patterns": [...],
  "timeline": [...],
  "statistics": {
    "total_decisions": 2,
    "total_experiences": 1,
    "total_patterns": 1,
    "total_sessions": 2
  }
}
```

### 2.4 Session Memory 索引

```json
{
  "session_id": "session-001",
  "session_type": "new",
  "session_description": "创建新的 Web 应用",
  "schema_version": "1.0",
  "created_at": "2026-02-02T10:00:00Z",
  "updated_at": "2026-02-02T16:00:00Z",
  "status": "completed",
  
  "overview": {
    "stages_completed": ["idea", "prd", "design", "plan", "coding"],
    "key_achievements": [
      "完成了需求分析",
      "设计了系统架构",
      "实现了核心功能"
    ],
    "challenges_faced": [
      "技术选型的权衡",
      "架构复杂度的控制"
    ]
  },
  
  "decisions": [...],
  "experiences": [...],
  "records": [...]
}
```

---

## 🛠️ Agent 工具（5个）

### 3.1 QueryMemoryIndexTool

```rust
#[tauri::command]
pub async fn query_memory_index(
    query_type: String,  // all | project | session
    category: String,    // decision | experience | pattern | record
    stage: Option<String>,
    limit: i64,
) -> Result<MemoryIndexResult, String> {
    let memory_dir = get_memory_dir()?;
    let mut results = vec![];
    
    // 查询逻辑...
    
    Ok(MemoryIndexResult {
        results,
        total: results.len(),
    })
}
```

### 3.2 LoadMemoryDetailTool

```rust
#[tauri::command]
pub async fn load_memory_detail(
    memory_id: String,
    file_path: String,
) -> Result<MemoryDetailResult, String> {
    let memory_dir = get_memory_dir()?;
    let full_path = memory_dir.join(&file_path);
    
    let content = std::fs::read_to_string(&full_path)?;
    
    Ok(MemoryDetailResult {
        memory_id,
        content,
        file: file_path,
    })
}
```

### 3.3 SaveSessionMemoryTool

```rust
#[tauri::command]
pub async fn save_session_memory(
    memory_type: String,  // decision | experience | record
    title: String,
    summary: String,
    content: String,
    stage: String,
    category: String,
    is_project_level: bool,
) -> Result<MemorySaveResult, String> {
    let session_id = get_current_session_id()?;
    let memory_id = format!("{}-{:04}", 
        memory_type.chars().next().unwrap_or('X'),
        chrono::Utc::now().timestamp() % 10000
    );
    
    // 保存到 Session Memory
    let memory_dir = get_memory_dir()?;
    let session_dir = memory_dir.join("sessions").join(&session_id);
    let type_dir = session_dir.join(&format!("{}s", memory_type));
    std::fs::create_dir_all(&type_dir)?;
    
    let file_path = type_dir.join(format!("{}.md", memory_id));
    std::fs::write(&file_path, content)?;
    
    // 更新索引
    update_session_index(&session_id, &memory_id, &title, &summary)?;
    
    Ok(MemorySaveResult {
        memory_id,
        file: file_path.to_string_lossy().to_string(),
        message: "Session 记忆已保存",
    })
}
```

### 3.4 PromoteToProjectMemoryTool

```rust
#[tauri::command]
pub async fn promote_to_project_memory(
    memory_id: String,
    reason: String,
) -> Result<PromoteResult, String> {
    // 读取 Session Memory
    let session_memory = load_session_memory_by_id(&memory_id)?;
    
    // 创建项目级 ID
    let project_memory_id = format!("DEC-{:04}", chrono::Utc::now().timestamp() % 10000);
    
    // 复制到项目 Memory
    let project_dir = get_memory_dir()?.join("project_memory").join("decisions");
    std::fs::create_dir_all(&project_dir)?;
    
    let session_dir = get_memory_dir()?.join("sessions").join(&session_memory.session_id);
    let old_file = session_dir.join("decisions").join(format!("{}.md", memory_id));
    let new_file = project_dir.join(format!("{}.md", project_memory_id));
    
    std::fs::copy(&old_file, &new_file)?;
    
    // 更新项目 Memory 索引
    update_project_index(&project_memory_id, &memory_id, &reason)?;
    
    Ok(PromoteResult {
        project_memory_id,
        file: new_file.to_string_lossy().to_string(),
        message: "已提升到项目级记忆",
    })
}
```

### 3.5 GetMemoryContextTool

```rust
#[tauri::command]
pub async fn get_memory_context() -> Result<MemoryContextResult, String> {
    let memory_dir = get_memory_dir()?;
    
    // 加载主 Memory
    let project_memory = load_project_memory(&memory_dir)?;
    
    // 加载当前 Session Memory
    let session_id = get_current_session_id()?;
    let session_memory = load_session_memory(&session_id)?;
    
    Ok(MemoryContextResult {
        project_memory: MemoryContextProject {
            total_decisions: project_memory.key_decisions.len(),
            total_experiences: project_memory.key_experiences.len(),
            key_decisions: project_memory.key_decisions.iter()
                .take(5)
                .map(|d| (d.id.clone(), d.title.clone()))
                .collect(),
        },
        session_memory: MemoryContextSession {
            session_id: session_id.clone(),
            status: session_memory.status,
            stages_completed: session_memory.overview.stages_completed,
            current_stage: "design", // 从上下文获取
            decisions: session_memory.decisions.len(),
            experiences: session_memory.experiences.len(),
        },
        context: MemoryContextInfo {
            current_time: chrono::Utc::now().to_rfc3339(),
            project_age: "1天",
            session_age: "6小时",
        },
    })
}
```

---

## 🧠 Agent 指令

```
# 记忆系统使用指南

## 你可以做什么

### 1. 查询记忆索引
使用 `query_memory_index` 工具查询记忆索引，获取基本信息：
- 决策列表（包含时间、需求、状态）
- 经验列表
- 模式列表

参数：
- query_type: all | project | session
- category: decision | experience | pattern | record
- stage: 阶段过滤
- limit: 返回数量限制

### 2. 读取记忆详情
使用 `load_memory_detail` 工具读取记忆的详细内容。

### 3. 保存 Session 记忆
使用 `save_session_memory` 工具保存当前 Session 的记忆：
- 决策（decision）
- 经验（experience）
- 记录（record）

### 4. 提升到项目级
使用 `promote_to_project_memory` 工具将有价值的记忆提升到项目级。

### 5. 获取记忆上下文
使用 `get_memory_context` 工具获取当前的记忆上下文。

## 何时使用记忆

### 做决策前
1. 查询项目级决策（query_memory_index, query_type="project", category="decision"）
2. 查询当前 Session 决策（query_memory_index, query_type="session", category="decision"）
3. 读取相关决策详情（load_memory_detail）
4. 基于历史经验做决策
5. 保存决策（save_session_memory）

### 遇到问题时
1. 查询项目级经验（query_memory_index, category="experience"）
2. 查询当前 Session 经验
3. 读取相关经验详情
4. 应用经验
5. 保存新经验

## 注意事项

- 只保存重要的、有价值的记忆
- 保持摘要简洁准确
- 详细内容放在 Markdown 文件中
- 有价值的经验可以提升到项目级
```

---

## 📅 实施计划

### 4.1 任务分解 (24h)

#### 数据结构定义 (4h)
- [ ] 主 Memory 数据结构 (1h)
- [ ] Session Memory 数据结构 (1h)
- [ ] 记忆索引数据结构 (1h)
- [ ] 时间线数据结构 (1h)

#### 记忆工具实现 (12h)
- [ ] QueryMemoryIndexTool (3h)
- [ ] LoadMemoryDetailTool (2h)
- [ ] SaveSessionMemoryTool (3h)
- [ ] PromoteToProjectMemoryTool (2h)
- [ ] GetMemoryContextTool (2h)

#### 存储管理 (4h)
- [ ] 文件系统结构 (1h)
- [ ] JSON 索引管理 (2h)
- [ ] Markdown 文件读写 (1h)

#### Agent 集成 (2h)
- [ ] 记忆工具注册 (1h)
- [ ] 记忆指令配置 (1h)

#### 用户界面 (2h)
- [ ] 简化的记忆浏览器 (1h)
- [ ] 查询功能 (1h)

---

## 🎯 验收标准

### 功能验收
- ✅ Agent 能够查询历史记忆（时间、需求、状态）
- ✅ 基于历史经验做决策
- ✅ 避免重复错误
- ✅ 积累项目知识
- ✅ 支持记忆提升到项目级

### 技术验收
- ✅ 结构清晰，易于维护
- ✅ 索引高效，查询快速
- ✅ 可控性强，质量有保证
- ✅ 立体结构，信息丰富
- ✅ Agent 负担轻，使用简单

---

**文档版本**: 1.0  
**创建时间**: 2026-02-02  
**Phase**: Phase 0.5  
**工作量**: 24h