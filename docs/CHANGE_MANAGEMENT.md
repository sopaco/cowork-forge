# Cowork 需求变更和技术调整指南

## 概述

Cowork 已经内置了完整的**需求变更**和**迭代优化**机制，可以在任何阶段调整方向。

---

## 🎯 核心机制

### 1. **Feedback Loop（反馈循环）**

Cowork 在完成 Check 阶段后，会自动进入 Feedback 阶段，此时你可以：
- 提出需求变更
- 调整技术方案
- 要求重做某些阶段

### 2. **Delta + Rerun 机制**

```json
{
  "delta": [
    {
      "target_stage": "requirements",
      "change": "增加用户头像上传功能"
    },
    {
      "target_stage": "design", 
      "change": "改用 React 而不是 Vue"
    }
  ],
  "rerun": [
    {
      "stage": "requirements",
      "reason": "需求增加了新功能"
    },
    {
      "stage": "design",
      "reason": "技术栈调整"
    }
  ]
}
```

**工作原理：**
- `delta`: 记录具体的变更内容
- `rerun`: 指定需要重新执行的阶段
- 系统会**自动级联重跑**后续阶段

---

## 📋 操作方式

### 方式一：等待 Feedback 阶段（推荐）

**流程：**
1. Cowork 完成 Coding → Check 后，会进入 Feedback 阶段
2. FeedbackAgent 会问你："是否满意？有什么需要调整的？"
3. 你回复你的需求变更或技术调整
4. FeedbackAgent 自动生成 `delta` 和 `rerun`
5. 系统自动重跑相关阶段

**示例对话：**

```
[Feedback 阶段]
FeedbackAgent: 
"项目已完成初步实现，请review以下内容：
 - 功能实现: ✅ 用户登录、注册
 - 技术栈: Node.js + Express + SQLite
 - 验证结果: ✅ 所有测试通过
 
 您是否满意？有什么需要调整的吗？"

用户回复:
"我希望：
1. 增加用户头像上传功能
2. 数据库从 SQLite 改为 PostgreSQL
3. 前端增加一个用户管理界面"

FeedbackAgent 分析后生成:
{
  "delta": [
    {"target_stage": "requirements", "change": "增加 REQ-004: 用户头像上传"},
    {"target_stage": "design", "change": "数据库改为 PostgreSQL"},
    {"target_stage": "design", "change": "增加前端用户管理界面"}
  ],
  "rerun": [
    {"stage": "requirements", "reason": "新增功能需求"},
    {"stage": "design", "reason": "数据库技术栈调整"}
  ]
}

系统自动执行:
→ 重跑 Requirements（生成新 PRD，包含头像上传需求）
→ 重跑 Design（调整为 PostgreSQL + 前端界面设计）
→ 重跑 Plan（更新实施计划）
→ 重跑 Coding（生成新代码）
→ 重跑 Check（验证新实现）
→ 再次进入 Feedback
```

---

### 方式二：使用 `cowork modify` 命令（⭐ 新增！）

**适用场景：**
- Feedback 阶段已退出
- 会话已完成所有阶段
- 随时想修改需求/技术方案

**命令格式：**

```bash
# 交互式输入修改内容
cowork modify <session-id>

# 直接指定修改内容
cowork modify <session-id> --change "增加用户头像上传功能，改用 PostgreSQL"
```

**示例操作：**

```bash
# 1. 查看会话状态
$ cowork inspect e41a1d0c

📊 会话信息:
  创建时间: 2026-01-23 09:00:00
  已完成阶段: [IdeaIntake, Requirements, Design, Plan, Coding, Check, Feedback, Delivery]
  
💡 提示:
  ✅ 所有阶段已完成！

# 2. 修改需求
$ cowork modify e41a1d0c

🔧 修改会话: e41a1d0c

📊 当前会话状态:
  创建时间: 2026-01-23 09:00:00
  已完成阶段: [IdeaIntake, Requirements, Design, Plan, Coding, Check, Feedback, Delivery]
  Feedback 迭代次数: 0/20

请描述您的修改需求（可以是需求变更、技术调整等）:
修改内容: 增加用户头像上传功能，数据库改为 PostgreSQL

🔄 正在处理修改请求...

╔═══════════════════════════════════════╗
║   🔄 处理修改请求                      ║
╚═══════════════════════════════════════╝

📝 分析结果:
  修改项: 2 个
  需要重跑: 2 个阶段
  - 修改 Requirements: 增加用户头像上传功能
  - 修改 Design: 数据库改为 PostgreSQL
  - 重跑 Requirements: 新增功能需求
  - 重跑 Design: 数据库技术栈调整

🔄 开始从 Requirements 阶段重新执行...

→ 重跑 Requirements
→ 重跑 Design
→ 重跑 Plan
→ 重跑 Coding
→ 重跑 Check
→ 进入 Feedback

✅ 修改完成！
```

---

### 方式三：手动干预（高级）

如果你想更精细地控制修改流程：

#### Step 1: 查看当前状态

```bash
cowork inspect <session-id>
```

#### Step 2: 手动修改 Artifact（可选）

```bash
# 编辑 PRD
vim .cowork/<session-id>/artifacts/requirements_<artifact-id>.json
```

修改示例：
```json
{
  "reqs": [
    {
      "id": "REQ-001",
      "desc": "用户登录",
      ...
    },
    {
      "id": "REQ-004",  // 新增
      "pri": "p1",
      "type": "func",
      "desc": "用户头像上传功能",
      "deps": ["REQ-001"],
      "ac": [
        "支持 jpg/png 格式",
        "图片大小不超过 2MB"
      ]
    }
  ]
}
```

#### Step 3: 使用 modify 命令触发重跑

```bash
cowork modify <session-id> --change "应用手动修改的 PRD"
```

---

## 🔄 自动级联重跑规则

当你标记某个阶段需要 `rerun` 时，系统会**自动重跑后续所有阶段**：

| 重跑阶段 | 自动级联重跑 |
|----------|-------------|
| Requirements | → Design → Plan → Coding → Check → Feedback |
| Design | → Plan → Coding → Check → Feedback |
| Plan | → Coding → Check → Feedback |
| Coding | → Check → Feedback |

**示例：**
```json
{
  "rerun": [
    {"stage": "design", "reason": "技术栈从 SQLite 改为 PostgreSQL"}
  ]
}
```

**执行流程：**
```
1. ✅ 保留 Requirements（不变）
2. 🔄 重跑 Design → 生成新的技术方案（PostgreSQL）
3. 🔄 重跑 Plan → 更新实施计划
4. 🔄 重跑 Coding → 生成适配 PostgreSQL 的代码
5. 🔄 重跑 Check → 验证新实现
6. 🔄 进入 Feedback → 再次确认
```

---

## 📝 实际案例

### 案例 1: Feedback 阶段没有反馈就退出了

**场景：**
用户在 Feedback 阶段没有输入反馈，直接退出了程序。现在想增加密码找回功能。

**操作：**
```bash
# 方法 1: 使用 modify 命令
$ cowork modify e41a1d0c --change "增加密码找回功能（邮箱验证码方式）"

# 方法 2: 交互式输入
$ cowork modify e41a1d0c
请描述您的修改需求: 增加密码找回功能（邮箱验证码方式）
```

**系统执行：**
```
FeedbackAgent 分析 → 生成 delta + rerun
→ 重跑 Requirements（增加 REQ-005: 密码找回）
→ 重跑 Design（增加邮件服务设计）
→ 重跑 Plan → Coding → Check
→ 进入 Feedback（再次确认）
```

---

### 案例 2: 会话已完成，想调整技术方案

**场景：**
项目已完成并交付，但用户发现 SQLite 不支持高并发，想改用 PostgreSQL。

**操作：**
```bash
$ cowork inspect e41a1d0c
✅ 所有阶段已完成！

$ cowork modify e41a1d0c --change "数据库从 SQLite 改为 PostgreSQL，需要支持高并发"
```

**系统执行：**
```
FeedbackAgent 分析 → delta: [Design 修改]
→ 重跑 Design（PostgreSQL 方案）
→ 重跑 Plan → Coding → Check
→ 进入 Feedback
```

---

### 案例 3: 增加多个需求

**场景：**
用户想一次性增加多个功能。

**操作：**
```bash
$ cowork modify e41a1d0c --change "
1. 增加用户头像上传功能
2. 增加用户个人资料编辑
3. 增加邮件通知功能
"
```

**系统执行：**
```
FeedbackAgent 分析 → delta: [Requirements 增加 3 个需求]
→ 重跑 Requirements（新 PRD 包含 REQ-004, REQ-005, REQ-006）
→ 重跑 Design → Plan → Coding → Check
→ 进入 Feedback
```

---

## 🚀 高级功能

### 1. **最大迭代次数控制**

Cowork 默认最大 Feedback 迭代次数为 **20 次**：

```rust
// SessionMeta 中
pub feedback_iterations: usize,  // 当前迭代次数
pub max_feedback_iterations: usize = 20,  // 最大迭代次数
```

如果超过 20 次，系统会拒绝继续修改：
```
Error: 已达到最大 Feedback 迭代次数 (20)，无法继续修改
```

---

### 2. **查看所有 Artifacts**

```bash
# 查看会话状态
cowork inspect <session-id>

# 导出所有 markdown 报告
cowork export <session-id>

# 查看具体 artifact
cat .cowork/<session-id>/artifacts/requirements_<id>.json | jq
cat .cowork/<session-id>/artifacts/design_<id>.md
```

---

### 3. **命令对比**

| 命令 | 用途 | 适用场景 |
|------|------|----------|
| `cowork resume <session-id>` | 恢复中断的会话 | 程序异常退出、还有阶段未完成 |
| `cowork modify <session-id>` | 修改需求/技术方案 | Feedback 阶段已退出、会话已完成、随时想调整 |
| `cowork inspect <session-id>` | 查看会话状态 | 了解当前进度、已完成的阶段 |
| `cowork export <session-id>` | 导出所有报告 | 获取最终交付物、查看历史版本 |

---

## ⚠️ 注意事项

### 1. **增量修改 vs 全量重做**

- **增量修改**：只改少量需求/技术细节 → 只重跑相关阶段
- **全量重做**：需求完全变化 → 建议创建新 session

### 2. **Artifact 版本管理**

每次重跑会生成**新的 Artifact ID**：
```
requirements_abc123.json  (v1)
requirements_def456.json  (v2, 包含新需求)
```

旧版本保留，可以对比查看变化。

### 3. **手动修改风险**

直接编辑 JSON 需要注意：
- ✅ 保持 JSON schema 正确
- ✅ 确保 ID 唯一性（REQ-001, REQ-002...）
- ✅ 更新 `artifact_id` 和 `ts` 时间戳（可选）
- ❌ 不要破坏 JSON 结构

---

## 📊 完整流程图

```
┌─────────────────────────────────────────────────┐
│     用户在任何时候想修改需求/技术方案             │
└───────────────┬─────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────────────────┐
│         执行 cowork modify <session-id>          │
│    （可选：--change "修改内容"）                  │
└───────────────┬─────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────────────────┐
│   系统检查会话状态                                │
│   - 会话是否存在？                                │
│   - Feedback 迭代次数是否超限？                   │
└───────────────┬─────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────────────────┐
│   FeedbackAgent 分析修改内容                      │
│   生成 delta + rerun                             │
└───────────────┬─────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────────────────┐
│   Orchestrator 自动级联重跑                       │
│   → Requirements (如果需要)                      │
│   → Design (如果需要)                            │
│   → Plan                                         │
│   → Coding                                       │
│   → Check                                        │
│   → Feedback (再次确认)                          │
└───────────────┬─────────────────────────────────┘
                │
                ▼
      ┌─────────┴─────────┐
      │                   │
  满意？              不满意？
      │                   │
      ▼                   ▼
   Delivery          继续迭代
   (交付)       (cowork modify 再次调整)
```

---

## 🎁 总结

### ✅ Cowork 已经支持：

1. **需求变更** - 随时使用 `cowork modify` 命令
2. **技术调整** - 修改 Design，自动级联更新后续阶段
3. **迭代优化** - 最多 20 次 Feedback 循环
4. **手动干预** - 可直接编辑 Artifact JSON
5. **阶段恢复** - 随时从任意阶段继续

### 📝 操作方式：

**方式一：Feedback 阶段（自动）**
```
等待系统进入 Feedback 阶段 → 输入修改需求 → 自动处理
```

**方式二：modify 命令（主动）⭐**
```bash
cowork modify <session-id>  # 交互式
cowork modify <session-id> --change "修改内容"  # 命令行
```

**方式三：手动编辑（高级）**
```bash
vim .cowork/<session-id>/artifacts/requirements_xxx.json
cowork modify <session-id> --change "应用手动修改"
```

### 🚀 最佳实践：

- **小步快跑**：每次只调整 1-2 个点，避免一次性大改
- **优先 modify 命令**：比手动编辑更安全、更智能
- **保留历史**：所有 Artifact 都有版本，可以随时回溯

---

**Feedback 阶段退出了也没关系！随时使用 `cowork modify` 命令重新调整需求。** 🎉
