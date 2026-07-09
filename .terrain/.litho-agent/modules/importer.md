# importer 模块深度报告

## 这个模块在做什么

Importer 是 Cowork Forge 的"考古学家"——当你想把一个已有的项目纳入 Cowork Forge 管理时，它会自动分析项目结构、检测技术栈、读取关键配置和代码，然后用 LLM 把所有这些信息综合成结构化的文档产出（idea.md、prd.md、design.md、plan.md）。它让用户不需要从头开始，而是可以把现有项目"带进"Cowork Forge 的迭代体系。

## 核心功能点

1. **项目导入配置**——定义导入参数和选项（`ImportConfig`），包括哪些文档需要生成、是否使用 LLM、项目名称。代码位置：`crates/cowork-core/src/importer/import_config.rs`
2. **项目分析器**——`ProjectAnalyzer` 扫描目录结构、检测技术栈、提取关键信息。代码位置：`crates/cowork-core/src/importer/project_analyzer.rs`
3. **制品生成器**——`ArtifactGenerator` 使用 LLM 将分析结果综合成标准文档产出。代码位置：`crates/cowork-core/src/importer/artifact_generator.rs`

## 关键组件

| 组件/类型 | 文件路径 | 一句话职责 |
|---------|---------|----------|
| `ImportConfig` | `crates/cowork-core/src/importer/import_config.rs` | 导入参数配置 |
| `ImportResult` | `crates/cowork-core/src/importer/mod.rs` | 导入结果，包含生成的制品和导入预览 |
| `ProjectAnalysis` | `crates/cowork-core/src/importer/mod.rs` | 项目分析结果，包含结构和技术栈信息 |

## 与其他模块的交互

| 交互模块 | 方向 | 说明 |
|---------|------|------|
| domain | 依赖 | 导入器创建 Project 和 Iteration 实体 |
| persistence | 依赖 | 导入器需要保存 Project 和 Iteration |

## 跨模块协作场景

**在导入流程中**：CLI 的 `import` 命令调用 Importer → 分析项目结构（`project_analyzer.rs`）→ 生成文档制品（`artifact_generator.rs`）→ 创建 Project 和 Genesis Iteration → 保存到 persistence。
