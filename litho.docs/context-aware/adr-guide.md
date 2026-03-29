# Architecture Decisions Record (ADR) 编写指南

## 什么是ADR？

记录**反直觉的设计决策**——那些AI无法从代码推断的"为什么"。

## 什么需要记录？

| 需要记录 | 不需要记录 |
|---------|-----------|
| 违反常见模式 | 遵循最佳实践 |
| 有非显而易见的原因 | 原因显而易见 |
| 会让人困惑的设计 | 直观的设计 |
| 有替代方案但放弃了 | 没有其他选择 |

## ADR模板

```markdown
## ADR-XXX: [简短标题]

**Decision**: 做了什么决定

**Reason**: 为什么这样做（最重要）

**Impact**: 影响哪些代码/模块

**Do not**: 什么不要做（可选）

**Limitation**: 有什么限制（可选）
```

## 示例

### ✅ 好的ADR

```markdown
## ADR-001: LoopAgent max_iterations=1

**Decision**: 所有Actor-Critic循环使用max_iterations=1

**Reason**: SequentialAgent有bug，exit_loop()会终止整个链而非仅循环。
用max_iterations=1让LoopAgent自然完成。

**Impact**: 所有critical stages (PRD, Design, Plan, Coding)

**Do not**: 修改此参数或迁移到SequentialAgent
```

### ❌ 不需要记录的

```markdown
## ADR-XXX: 使用async/await

**Decision**: 使用async/await处理异步

**Reason**: 这是Rust标准做法

→ 这是最佳实践，不需要记录
```

## 示例场景

### 场景1：性能换简洁

```markdown
## ADR-XXX: JSON存储而非SQLite

**Decision**: 用JSON文件存储数据

**Reason**: 简化调试、易于检查、无外部依赖。数据量小、无并发需求。

**Limitation**: 不适合大规模并发场景
```

### 场景2：Bug workaround

```markdown
## ADR-XXX: 两步知识提升

**Decision**: Insight→Decision需要两个工具

**Reason**: 并非所有insight都应成为decision。
中间步骤让人review确保质量。

**Impact**: memory tools, knowledge workflow
```

### 场景3：安全权衡

```markdown
## ADR-XXX: HITL超时默认Pass

**Decision**: HITL确认超时时默认继续(Pass)

**Reason**: 比无限阻塞好。用户可用PM Agent回退。

**Impact**: Stage执行, GUI超时处理
```

## 维护原则

1. **只加不减**：ADR是历史记录，不建议删除
2. **简短有力**：每个ADR不超过100字
3. **聚焦原因**：Reason字段最重要
4. **保持数量少**：一个项目通常只有5-15个ADR
