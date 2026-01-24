<p align="center">
  <img height="200" src="./assets/blend_banner.png">
</p>

<h1 align="center">Cowork Forge</h1>

<p align="center">
    <a href="./README.md">English</a>
    |
    <a href="./README_zh.md">中文</a>
</p>

<p align="center">
    <strong>🤖 基于 AI 多智能体架构的自动化软件开发框架 🤖</strong>
</p>
<p align="center">使用 Rust 构建，Cowork Forge 通过协调智能 AI 智能体，实现从需求到交付的完整软件开发生命周期自动化。</p>

<p align="center">
  <a href="https://github.com/sopaco/cowork-forge/tree/main/litho.docs"><img alt="Litho Docs" src="https://img.shields.io/badge/Litho-Docs-green?logo=Gitbook&color=%23008a60">
  <a href="https://github.com/sopaco/cowork-forge"><img alt="GitHub Actions Workflow Status" src="https://img.shields.io/github/actions/workflow/status/sopaco/cowork-forge/rust.yml?label=Build">
  <a href="./LICENSE"><img alt="MIT" src="https://img.shields.io/badge/license-MIT-blue.svg?label=LICENSE" />
</p>

<hr />

# 👋 什么是 Cowork Forge？

<strong>Cowork Forge</strong> 是一个完整的、生产就绪的框架，通过智能多智能体协作实现软件开发自动化。它超越了简单的代码生成，提供了一个全面的系统，协调专业的 AI 智能体处理软件开发生命周期的每个阶段。

由 Rust 和 LLM 驱动，Cowork Forge 协调 8 个专业智能体协同工作，将你的想法转化为生产就绪的代码。从需求收集和 PRD 生成，到技术设计、实施计划、编码和质量验证——Cowork Forge 全部管理，并在关键决策点进行人工验证。

<p align="center">
  <strong>让你的开发工作流程通过像真实开发团队一样思考、规划和协作的 AI 智能体得到变革。</strong>
</p>

<div style="text-align: center; margin: 30px 0;">
  <table style="width: 100%; border-collapse: collapse; margin: 0 auto;">
    <tr>
      <th style="width: 50%; padding: 15px; background-color: #f8f9fa; border: 1px solid #e9ecef; text-align: center; font-weight: bold; color: #495057;">传统开发模式</th>
      <th style="width: 50%; padding: 15px; background-color: #f8f9fa; border: 1px solid #e9ecef; text-align: center; font-weight: bold; color: #495057;">使用 Cowork Forge 的开发模式</th>
    </tr>
    <tr>
      <td style="padding: 15px; border: 1px solid #e9ecef; vertical-align: top;">
        <p style="font-size: 14px; color: #6c757d; margin-bottom: 10px;"><strong>手动、碎片化的流程</strong></p>
        <ul style="font-size: 13px; color: #6c757d; line-height: 1.6;">
          <li>手动需求收集和文档编写</li>
          <li>设计、编码和测试使用分离的工具</li>
          <li>耗时的代码审查和迭代</li>
          <li>知识孤岛和沟通鸿沟</li>
          <li>重复的样板代码编写</li>
        </ul>
      </td>
      <td style="padding: 15px; border: 1px solid #e9ecef; vertical-align: top;">
        <p style="font-size: 14px; color: #6c757d; margin-bottom: 10px;"><strong>AI 驱动的协作开发</strong></p>
        <ul style="font-size: 13px; color: #6c757d; line-height: 1.6;">
          <li>自动化需求分析和 PRD 生成</li>
          <li>集成智能体处理设计、编码和验证</li>
          <li>智能代码规划，支持增量更新</li>
          <li>所有开发阶段共享上下文</li>
          <li>关键决策点的人工验证</li>
        </ul>
      </td>
    </tr>
  </table>
</div>

<hr />

# 😺 为什么选择 Cowork Forge？

- <strong>端到端自动化：</strong> 从想法到交付，Cowork Forge 通过协调的 AI 智能体自动化整个软件开发生命周期。
- <strong>多智能体协作：</strong> 8 个专业智能体协同工作，每个都在其领域带来专业知识——需求、设计、规划、编码和验证。
- <strong>人机协作：</strong> 关键决策点包含人工验证，在保持自动化效率的同时确保质量和控制。
- <strong>增量代码更新：</strong> 智能代码分析实现有针对性的更新，无需重新生成整个代码库，保留你的自定义内容。
- <strong>多语言支持：</strong> 内置支持 Rust、Python、JavaScript/TypeScript 等，并自动检测项目类型。
- <strong>安全优先：</strong> 多层安全检查防止危险命令并确保安全的代码执行。
- <strong>高性能：</strong> 使用 Rust 构建，确保速度、内存安全和高效的资源利用。

# 🏆 Cowork Forge 与竞品对比

Cowork Forge 在 AI 开发工具领域通过其独特的多智能体架构和全面的工作流覆盖而脱颖而出。

## 核心能力对比

| 能力 | Cowork Forge | GitHub Copilot | Cursor AI | Aider |
|------------|------------------------|----------------|-----------|-------|
| **端到端工作流** | ✅ 完整（想法→交付） | ❌ 仅代码补全 | ❌ 专注于代码编辑 | ❌ 仅代码辅助 |
| **多智能体架构** | ✅ 8 个专业智能体 | ❌ 单一模型 | ❌ 单一模型 | ❌ 单一模型 |
| **PRD 生成** | ✅ 自动化 | ❌ N/A | ❌ N/A | ❌ N/A |
| **技术设计** | ✅ C4 架构文档 | ❌ N/A | ❌ N/A | ❌ N/A |
| **实施计划** | ✅ 任务分解和里程碑 | ❌ N/A | ❌ N/A | ❌ N/A |
| **增量更新** | ✅ 智能增量分析 | ❌ N/A | ❌ 有限 | ❌ 有限 |
| **多语言支持** | ✅ Rust、Python、JS/TS | ✅ 多种语言 | ✅ 多种语言 | ✅ 多种语言 |
| **人机协作** | ✅ 关键决策点 | ❌ N/A | ❌ 有限 | ❌ 有限 |
| **自动化验证** | ✅ 构建/测试集成 | ❌ N/A | ❌ N/A | ❌ N/A |
| **安全检查** | ✅ 多层安全 | ❌ N/A | ❌ 基础 | ❌ 基础 |
| **工件存储** | ✅ 版本化工件 | ❌ N/A | ❌ N/A | ❌ N/A |
| **开源** | ✅ MIT 许可证 | ❌ 专有 | ❌ 专有 | ✅ MIT 许可证 |
| **自托管** | ✅ 本地执行 | ❌ 仅云端 | ❌ 仅云端 | ✅ 本地执行 |

## 核心差异化优势

### 1. 完整的开发生命周期
与仅协助编写单行代码的代码补全工具不同，Cowork Forge 管理整个软件开发过程——从初始想法收集到最终交付。这种全面的方法确保所有阶段的一致性和可追溯性。

### 2. 多智能体协作
Cowork Forge 的 8 个专业智能体像真实开发团队一样协同工作：
- <strong>需求采集智能体</strong>: 结构化用户需求
- <strong>PRD 智能体</strong>: 生成全面的 PRD
- <strong>设计智能体</strong>: 创建技术架构（C4 模型）
- <strong>计划智能体</strong>: 分解实施任务
- <strong>编码阶段智能体</strong>: 规划和执行代码变更
- <strong>检查智能体</strong>: 验证代码质量并运行测试
- <strong>反馈智能体</strong>: 分析用户反馈并确定重做范围
- <strong>交付智能体</strong>: 生成交付报告

### 3. 人机协作验证
关键输出需要人工确认才能继续，确保：
- 准确的需求捕获
- 合理的技术决策
- 可行的实施计划
- 安全的代码变更

这种自动化与人工控制的平衡使 Cowork Forge 区别于完全自主的工具。

### 4. 增量代码更新
当需求或设计变更时，Cowork Forge 智能识别受影响的文件并仅更新必要的内容——保留你的自定义内容，避免完全重新生成。

### 5. 内置安全
多层安全检查防止：
- 危险命令执行（rm -rf、sudo 等）
- 未授权的文件系统访问
- 恶意代码注入
- 资源耗尽


❤️ 喜欢 <strong>Cowork Forge</strong>？给它加星 🌟 或 [赞助我](https://github.com/sponsors/sopaco)！❤️

# 🌠 功能与特性

- <strong>8 阶段开发工作流：</strong> 涵盖需求采集 → PRD 生成 → 技术设计 → 实施计划 → 编码 → 检查 → 反馈 → 交付的完整工作流。
- <strong>专业 AI 智能体：</strong> 每个阶段由具有领域特定专业知识和工具的专用智能体处理。
- <strong>智能代码规划：</strong> 分析项目结构、依赖关系，生成精确的代码变更计划。
- <strong>增量代码更新：</strong> 智能增量分析只更新受影响的文件，保留现有修改。
- <strong>自动化质量验证：</strong> 多语言构建/测试集成，包含全面的错误分析和报告。
- <strong>人机协作验证：</strong> 关键输出（PRD、设计、计划）需要人工确认才能继续。
- <strong>基于工件的存储：</strong> 所有阶段输出的版本化存储，使用 JSON 和 Markdown 格式。
- <strong>待办事项列表管理：</strong> 自动任务跟踪，包含状态推断和进度报告。
- <strong>多语言项目支持：</strong> 自动检测和处理 Rust、Python、JavaScript/TypeScript 项目。
- <strong>安全与防护：</strong> 命令验证、路径访问控制和看门狗监控，确保安全执行。

# 🏗️ 架构

Cowork Forge 采用模块化、领域驱动的架构构建：

```mermaid
graph TB
    subgraph "CLI 层"
        CLI[cowork-cli]
    end
    
    subgraph "核心层"
        Orch[编排器]
        Exec[阶段执行器]
    end
    
    subgraph "智能体层"
        IDEA[需求采集智能体]
        PRD[PRD 智能体]
        DESIGN[设计智能体]
        PLAN[计划智能体]
        CODING[编码阶段智能体]
        CHECK[检查智能体]
        FEEDBACK[反馈智能体]
        DELIVERY[交付智能体]
    end
    
    subgraph "基础设施层"
        TOOLS[工具]
        VERIFY[验证]
        MEMORY[内存]
        HITL[HITL]
        CONFIG[配置]
    end
    
    subgraph "外部"
        LLM[OpenAI LLM]
        FS[文件系统]
        CMD[命令行]
    end
    
    CLI --> Orch
    Orch --> Exec
    Exec --> IDEA
    Exec --> PRD
    Exec --> DESIGN
    Exec --> PLAN
    Exec --> CODING
    Exec --> CHECK
    Exec --> FEEDBACK
    Exec --> DELIVERY
    
    IDEA --> TOOLS
    CODING --> TOOLS
    CHECK --> TOOLS
    
    CHECK --> VERIFY
    CODING --> VERIFY
    
    IDEA --> MEMORY
    PRD --> MEMORY
    DESIGN --> MEMORY
    PLAN --> MEMORY
    CODING --> MEMORY
    CHECK --> MEMORY
    FEEDBACK --> MEMORY
    DELIVERY --> MEMORY
    
    Exec --> HITL
    
    TOOLS --> FS
    TOOLS --> CMD
    
    IDEA -.-> LLM
    PRD -.-> LLM
    DESIGN -.-> LLM
    CODING -.-> LLM
```

## 核心组件

### 编排器
中央协调器，管理会话生命周期、阶段依赖和工作流执行。

### 阶段执行器
为所有智能体提供统一的执行框架，具有一致的错误处理和状态管理。

### AI 智能体
八个专业智能体，每个负责开发生命周期的特定阶段。

### 工具模块
安全的文件操作和命令执行，包含安全检查和资源限制。

### 验证模块
项目类型检测、代码验证和全面的错误分析。

### HITL 控制器
管理人机协作交互，包括内容审查和编辑。

### 工件存储
所有阶段输出的版本化存储，使用 JSON 和 Markdown 格式。

# 🧠 工作原理

Cowork Forge 使用由 `Orchestrator` 编排的复杂多阶段工作流：

```mermaid
sequenceDiagram
    participant User as 用户
    participant CLI as Cowork Forge CLI
    participant Orch as 编排器
    participant Agents as AI 智能体
    participant LLM as OpenAI LLM
    participant FS as 文件系统
    participant CMD as 命令行

    User->>CLI: 提供想法/需求
    CLI->>Orch: 启动新会话
    Orch->>Agents: 执行需求采集智能体
    Agents->>LLM: 结构化需求
    LLM-->>Agents: 返回需求规格
    Agents->>User: HITL 验证
    User-->>Agents: 确认/编辑
    
    loop 对于每个阶段
        Orch->>Agents: 执行下一个智能体
        Agents->>LLM: 生成阶段输出
        LLM-->>Agents: 返回结果
        
        alt 关键阶段
            Agents->>User: HITL 验证
            User-->>Agents: 确认/编辑
        end
        
        alt 编码阶段
            Agents->>FS: 读取项目文件
            Agents->>LLM: 规划代码变更
            LLM-->>Agents: 返回代码计划
            Agents->>User: HITL 验证
            User-->>Agents: 确认计划
            Agents->>FS: 写入代码变更
            Agents->>CMD: 运行构建/测试
            CMD-->>Agents: 返回结果
        end
    end
    
    Orch->>Agents: 执行交付智能体
    Agents->>User: 展示交付报告
```

# 🖥 快速开始

### 前置要求
- [**Rust**](https://www.rust-lang.org)（版本 1.70 或更高）
- [**OpenAI API 密钥**](https://platform.openai.com/) 用于 LLM 访问
- Git 和语言特定的构建工具（cargo、npm、pip 等）

### 安装

从源代码构建：

```sh
# 克隆仓库
git clone https://github.com/sopaco/cowork-forge.git
cd cowork-forge

# 构建项目
cargo build --release

# CLI 二进制文件将位于：
# target/release/cowork
```

### 配置

Cowork Forge 使用 `config.toml` 文件进行配置。在项目目录中创建一个或使用 `--config` 指定路径：

```toml
# -----------------------------------------------------------------------------
# OpenAI LLM 配置
# -----------------------------------------------------------------------------
[llm]
api_base_url = "https://api.openai.com/v1"
api_key = "sk-your-openai-api-key"
model = "gpt-4o"
temperature = 0.7
max_tokens = 8192

# -----------------------------------------------------------------------------
# 工作区配置
# -----------------------------------------------------------------------------
[workspace]
project_root = "."  # 项目的根目录
artifacts_dir = ".cowork/artifacts"  # 存储生成的工件的目录

# -----------------------------------------------------------------------------
# HITL（人机协作）配置
# -----------------------------------------------------------------------------
[hitl]
enabled = true
editor_command = "code"  # 用于验证的外部编辑器

# -----------------------------------------------------------------------------
# 安全配置
# -----------------------------------------------------------------------------
[safety]
block_dangerous_commands = true
allowed_build_tools = ["cargo", "npm", "pip", "make", "mvn", "gradle"]

# -----------------------------------------------------------------------------
# 日志配置
# -----------------------------------------------------------------------------
[logging]
level = "info"
file = ".cowork/logs/cowork.log"
```

# 🚀 使用

### 启动新的开发会话

```sh
# 使用想法启动新会话
cowork start "构建任务管理应用的 REST API"

# 使用配置文件启动
cowork start "创建 Web 仪表板" --config ./config.toml

# 恢复现有会话
cowork resume <session-id>
```

### 会话工作流

当你启动会话时，Cowork Forge 将引导你完成 8 阶段工作流：

1. **需求采集**: 你的想法被结构化为正式规范
2. **PRD 生成**: 创建全面的产品需求文档
3. **技术设计**: C4 架构图和技术规范
4. **实施计划**: 任务分解，包含里程碑和依赖关系
5. **编码**: 代码被规划和执行，支持增量更新
6. **检查**: 构建、测试和质量验证
7. **反馈**: 审查结果并在需要时请求更改
8. **交付**: 最终交付报告，包含实施摘要

在每个关键阶段，你将被提示在继续之前审查和确认输出。

### 示例会话流程

```sh
$ cowork start "构建文件转换的 CLI 工具"

[阶段 1/8] 需求采集智能体
正在分析你的需求...
生成的需求规格: "用于在格式之间转换文件的命令行工具"

你想要：
  [1] 接受并继续
  [2] 编辑规范
  [3] 重新生成
> 1

[阶段 2/8] PRD 生成智能体
正在生成产品需求文档...
创建了包含 12 个需求和 5 个用户故事的 PRD

在以下位置审查 PRD: .cowork/artifacts/session-001/prd.md
你想要：
  [1] 接受并继续
  [2] 编辑 PRD
  [3] 重新生成
> 1

[阶段 3/8] 设计智能体
正在创建技术架构...
生成了 C4 系统上下文和容器图

在以下位置审查设计: .cowork/artifacts/session-001/design.md
你想要：
  [1] 接受并继续
  [2] 编辑设计
  [3] 重新生成
> 1

... （继续完成所有 8 个阶段）

[交付] 会话完成！
交付报告: .cowork/artifacts/session-001/delivery.md

摘要:
- 12 个需求已实现
- 4 个模块已创建
- 15 个测试用例已添加
- 构建: 通过
- 测试: 15/15 通过
```

### 管理会话

```sh
# 列出所有会话
cowork list

# 查看会话状态
cowork status <session-id>

# 查看会话的工件
cowork artifacts <session-id>

# 删除会话
cowork delete <session-id>
```

### 配置管理

```sh
# 显示当前配置
cowork config show

# 更新配置
cowork config set llm.model gpt-4-turbo
cowork config set hitl.enabled false
```

# 🔒 安全

Cowork Forge 实现多层安全：

1. **命令验证**: 危险命令在执行前被阻止
2. **路径访问控制**: 限制对敏感系统目录的访问
3. **构建工具白名单**: 只有授权的开发工具才能执行
4. **超时控制**: 防止长时间运行的命令导致资源耗尽
5. **看门狗监控**: 检测并防止智能体偏离目标

# 🤝 贡献

我们欢迎各种形式的贡献！通过 [GitHub Issues](https://github.com/sopaco/cowork-forge/issues) 报告错误或提交功能请求。

### 开发流程
1. Fork 此项目
2. 创建功能分支（`git checkout -b feature/amazing-feature`）
3. 提交你的更改（`git commit -m 'Add some amazing feature'`）
4. 推送到分支（`git push origin feature/amazing-feature`）
5. 创建 Pull Request

### 运行测试

```sh
# 运行所有测试
cargo test

# 运行带覆盖率的测试
cargo test --all-features

# 运行特定模块的测试
cargo test -p cowork-core
```

# 📚 文档

全面的文档可在 [litho.docs](./litho.docs/) 目录中找到：

- [项目概述](./litho.docs/1、项目概述.md) - 系统上下文和架构
- [架构概述](./litho.docs/2、架构概览.md) - 详细的架构文档
- [核心工作流](./litho.docs/3、工作流程.md) - 工作流和流程文档
- [领域模块](./litho.docs/4、深入探索/) - 深入的领域分析

# 🪪 许可证

本项目采用 **MIT 许可证**。详见 [LICENSE](LICENSE) 文件。

# 🙏 致谢

- 使用 [Rust](https://www.rust-lang.org/) 构建
- 由 [OpenAI](https://openai.com/) GPT 模型驱动
- 灵感来自现代软件开发实践和 AI 智能体研究

# 📬 联系方式

- **GitHub**: [sopaco/cowork-forge](https://github.com/sopaco/cowork-forge)
- **Issues**: [GitHub Issues](https://github.com/sopaco/cowork-forge/issues)

---

**通过 Cowork Forge 改变你的开发工作流程——协作式软件开发的未来。** 🚀
