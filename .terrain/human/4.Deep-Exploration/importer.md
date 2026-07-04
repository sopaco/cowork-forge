# Importer 领域

**模块路径**：`crates/cowork-core/src/importer/`
**生成日期**：2026-07-05

---

## 概述

Importer 是 Cowork Forge 的"考古学家"。它能让已有项目进入 Cowork Forge 的迭代体系——自动分析项目结构、检测技术栈、读取配置和关键文件，然后用 LLM 综合成标准文档（idea.md、prd.md、design.md、plan.md）。

---

## 核心功能点

1. **项目导入配置**——`ImportConfig` 定义导入参数（生成哪些文档、是否使用 LLM）。`crates/cowork-core/src/importer/import_config.rs`
2. **项目分析器**——扫描目录结构、检测技术栈。`crates/cowork-core/src/importer/project_analyzer.rs`
3. **制品生成器**——LLM 将分析结果综合成文档。`crates/cowork-core/src/importer/artifact_generator.rs`

---

## 关键组件

| 组件/类型 | 文件路径 | 核心职责 |
|---------|---------|---------|
| `ImportConfig` | `crates/cowork-core/src/importer/import_config.rs` | 导入参数配置 |
| `ImportResult` | `crates/cowork-core/src/importer/mod.rs` | 导入结果 |
| `ProjectAnalysis` | `crates/cowork-core/src/importer/mod.rs` | 项目分析结果 |

---

## 与其他模块的交互

| 交互模块 | 方向 | 说明 |
|---------|------|------|
| domain | 依赖 | 创建 Project/Iteration |
| persistence | 依赖 | 保存 Project/Iteration |
