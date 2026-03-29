# AI友好的项目上下文方法论

## 核心理念

让Coding Agent理解项目，不需要每次都消耗大量Token从头阅读代码。

### 关键矛盾

| 需求 | 约束 |
|------|------|
| 信息足够让AI理解代码 | 文档不能太细，维护成本高 |
| 保持准确性 | 更新本身消耗Token |
| 覆盖关键决策 | 不能每次都从头读代码 |

### 解决思路：分层 + 衰减

```
高价值/低变化 → 详细文档，长期有效
高价值/高变化 → 简要索引，指向代码
低价值/低变化 → 不记录
低价值/高变化 → 不记录（AI直接读代码）
```

---

## 设计原则

### 1. 记录"为什么"，不记录"是什么"

AI可以从代码读出"是什么"，但无法推断"为什么"。

```
❌ 不记录：
struct Iteration {
    id: String,
    number: u32,
    ...
}

✅ 记录：
为什么LoopAgent用max_iterations=1？
因为SequentialAgent有bug，exit_loop会提前终止整个链。
```

### 2. 删除精确引用

精确行号容易过时，用概念定位代替。

```
❌ 位置: crates/cowork-core/src/pipeline/executor/mod.rs:72

✅ 入口函数: IterationExecutor::execute()
   AI可以通过grep自己定位
```

### 3. 合并相关内容

减少文件跳转，降低认知负担。

```
合并前：
- domains/pipeline.md (Pipeline)
- domains/agents.md (Agents)
- domains/interaction.md (Interaction)

合并后：
- domains/pipeline.md (Pipeline + Agents)
- core/constraints.md (约束 + Interaction规则)
```

### 4. 精简工具列表

工具名已经足够说明功能。

```
❌ | ReadFileTool | 读取文件内容 |
   | WriteFileTool | 写入文件内容 |

✅ File Tools: ReadFile, WriteFile, ListFiles
```

---

## 文档结构

### 最小化结构

```
.ai-context/
├── manifest.yaml          # 索引 + 维护指南
├── project.snapshot       # 项目结构 + 存储 + 配置
├── architecture-decisions.md  # 反直觉设计决策
├── core/
│   ├── modules.map        # 模块依赖 + 导航
│   └── constraints.md     # 安全约束 + 限制规则
├── domains/
│   ├── pipeline.md        # 流程 + Agent
│   ├── domain-logic.md    # 核心实体关系
│   └── tools.md           # 工具分类
└── prompts/
    └── coding-context.md  # 代码风格
```

### Token预算

| 文件 | 预算 | 用途 |
|------|------|------|
| project.snapshot | ~800 | 项目概览 |
| modules.map | ~800 | 导航定位 |
| constraints.md | ~400 | 约束规则 |
| pipeline.md | ~500 | 流程理解 |
| domain-logic.md | ~400 | 实体关系 |
| tools.md | ~400 | 工具索引 |
| architecture-decisions.md | ~500 | 决策理解 |
| **总计** | **~4000** | |

---

## 维护策略

### 需要更新 vs 不需要更新

| 变化类型 | 是否更新文档 |
|---------|-------------|
| 新增工具 | ✅ 更新 tools.md |
| 新增Stage | ✅ 更新 pipeline.md |
| 新增模块 | ✅ 更新 modules.map |
| 新增约束 | ✅ 更新 constraints.md |
| 架构决策 | ✅ 更新 architecture-decisions.md |
| struct字段变化 | ❌ 不更新 |
| 函数签名变化 | ❌ 不更新 |
| 重构代码 | ❌ 不更新 |
| 行号变化 | ❌ 不记录 |

### 更新指令

```
用户: "Update .ai-context because I added a new tool"
用户: "Update .ai-context because I made an architecture decision"
```

### 变更检测

Agent可在会话启动时检测：
```bash
git diff --name-only HEAD~10
```

检测到关键目录变更时提醒用户更新文档。

---

## 实践检查清单

### 创建AI上下文时

- [ ] 是否只记录"为什么"而非"是什么"？
- [ ] 是否删除了精确行号引用？
- [ ] 是否合并了相关内容？
- [ ] 是否删除了struct字段定义？
- [ ] 总Token是否控制在4000以内？

### 维护AI上下文时

- [ ] 是否只在高价值变化时更新？
- [ ] 是否避免了结构体字段的同步？
- [ ] 是否保持了"指向代码"的原则？

---

## 效果评估

| 准确度 | 效果 |
|--------|------|
| 70% | Agent能理解项目结构 |
| 80% | Agent能找到关键代码 |
| 90% | Agent能遵循核心约束 |
| 100% | 不现实，维护成本过高 |

**核心目标**：让Agent快速定位，需要精确信息时自己去读代码。
