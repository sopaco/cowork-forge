# AI上下文模板

## 项目快照模板 (project.snapshot)

```markdown
# [项目名] - AI Context Snapshot

## PROJECT OVERVIEW

name: [项目名]
type: [rust-workspace | monorepo | single-app]
version: [版本]
description: [一句话描述]

## STRUCTURE

### [核心模块名]
path: [路径]
key_modules:
  - [模块1]: [作用]
  - [模块2]: [作用]

### [CLI/前端等其他模块]
path: [路径]

---

## STORAGE STRUCTURE

[存储目录结构，简化版]

| Store | File | Purpose |
|-------|------|---------|
| [Store名] | [文件路径] | [用途] |

---

## CONFIGURATION

[配置文件位置和关键字段]

---

## EXTERNAL INTEGRATIONS

[外部集成，如API、SDK等]

---
**Note**: For detailed implementations, see corresponding source files.
```

---

## 模块地图模板 (modules.map)

```markdown
# Module Dependency Map
# Note: For detailed function signatures, grep the code directly

## ARCHITECTURE LAYERS

```
[ASCII架构图，展示分层]
```

## MODULE DEPENDENCY FLOW

```
[依赖流向，一行表示]
```

## KEY TRAITS

| Trait | Location | Purpose |
|-------|----------|---------|
| [Trait名] | [位置] | [用途] |

## CORE FLOWS

### [流程名]
```
[流程步骤]
```

## MODULE FILES REFERENCE

### [模块名]
`[路径]`
- [文件1] - [作用]
- [文件2] - [作用]

## QUICK NAVIGATION BY TASK

| Task | Location |
|------|----------|
| [任务描述] | [文件路径] |
```

---

## 约束模板 (constraints.md)

```markdown
# Constraints & Boundaries

## Security Constraints

| Constraint | Rule |
|------------|------|
| [约束名] | [规则] |

## Rate Limiting

| Resource | Limit |
|----------|-------|
| [资源名] | [限制] |

## [业务] Constraints

| Constraint | Value |
|------------|-------|
| [约束] | [值] |

## Error Handling

| Type | Behavior |
|------|----------|
| [错误类型] | [处理方式] |
```

---

## 领域文档模板 (domains/*.md)

```markdown
# [领域名] Domain

## Core Concepts

| Concept | Location | Purpose |
|---------|----------|---------|
| [概念] | [位置] | [用途] |

## Relationships

```
[实体关系图]
```

## [关键机制]

| Type | Description |
|------|-------------|
| [类型] | [描述] |

## Code Locations

| Component | Location |
|-----------|----------|
| [组件] | [位置] |

---
**Note**: For details, read source files directly.
```

---

## 工具索引模板 (tools.md)

```markdown
# Tools Domain

## Tool Categories

| Category | File | Key Tools |
|----------|------|-----------|
| [分类] | [文件] | [工具列表] |

## Security

[安全约束摘要]

## Location

[工具目录路径]
```

---

## Manifest模板 (manifest.yaml)

```yaml
# AI Context Manifest
# Version: [版本]
# Project: [项目名]

manifest_version: "[版本]"
project:
  name: [项目名]
  version: [版本]
  language: [语言]

documents:
  - path: [文件路径]
    purpose: [用途]
    tokens: ~[估算]

usage:
  coding_context:
    - [文件列表]
  
  debug_context:
    - [文件列表]

total_tokens: ~[总计]

# =============================================================================
# MAINTENANCE GUIDE
# =============================================================================
#
# | Code Change              | Update File            |
# |--------------------------|------------------------|
# | [变更类型]               | [更新文件]             |
#
# | NO UPDATE: [不需要更新的情况] |
#
# Command: "Update .ai-context because I [action] [thing]"
# =============================================================================
```
