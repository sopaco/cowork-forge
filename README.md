# Cowork - AI 驱动的多 Agent 软件开发系统

一个基于 Rust 和 adk-rust 框架构建的智能软件开发辅助系统，通过8个阶段的工作流，从 IDEA 到交付物的全流程自动化。

## ✨ 特性

- 🤖 **8阶段工作流**：IDEA Intake → PRD → Design → Plan → Code → Check → Feedback → Delivery
- 🧠 **智能 Agent**：每个阶段由专门的 AI Agent 处理
- 💾 **持久化**：所有 Artifact 以 JSON + Markdown 双格式保存
- 🔄 **HITL 集成**：关键节点支持人工确认和反馈
- 📊 **类型安全**：完整的 Rust 类型系统保证
- ✅ **测试覆盖**：16个单元测试，100%通过
- 🔌 **灵活配置**：支持私有部署的 OpenAI-compatible API

## 🚀 快速开始

### 前置要求

- Rust 1.70+
- 私有部署的 LLM 服务（OpenAI-compatible API）

### 安装

```bash
git clone <your-repo>
cd cowork-rs
cargo build --release
```

### 配置

创建 `config.toml` 文件：

```toml
[llm]
api_base_url = "https://your-llm-api.com/v1"
api_key = "your-api-key"
model_name = "your-model-name"

[embedding]
api_base_url = "https://your-embedding-api.com/v1"
api_key = "your-api-key"
model_name = "your-embedding-model"
```

或使用环境变量：

```bash
export LLM_API_BASE_URL="https://your-llm-api.com/v1"
export LLM_API_KEY="your-api-key"
export LLM_MODEL_NAME="your-model-name"
```

### 使用

```bash
# 启动交互式模式（创建新会话）
cargo run --release

# 恢复中断的会话
cargo run --release -- resume <session-id>

# 查看会话详情
cargo run --release -- inspect <session-id>

# 导出会话结果
cargo run --release -- export <session-id>

# 指定配置文件
cargo run --release -- --config path/to/config.toml
```

**会话恢复示例：**

```bash
# 1. 启动新会话（工作流执行到一半时中断）
$ cargo run --release
Session created: a1b2c3d4-e5f6-7890-abcd-ef1234567890
Stage 1: IDEA Intake ✓
Stage 2: Requirements ✓
Stage 3: Design ✓
^C  # 用户中断

# 2. 恢复会话（从 Stage 4 继续）
$ cargo run --release -- resume a1b2c3d4-e5f6-7890-abcd-ef1234567890
已完成: [IdeaIntake, Requirements, Design]
下一阶段: Plan
✓ 跳过 Stage 1: IDEA Intake (已完成)
✓ 跳过 Stage 2: Requirements (已完成)
✓ 跳过 Stage 3: Design (已完成)
Stage 4: Implementation Plan ...  # 从这里继续

# 3. 检查会话状态
$ cargo run --release -- inspect a1b2c3d4
📊 会话信息:
  已完成阶段: [IdeaIntake, Requirements, Design, Plan]
  下一阶段: Coding
```

详细的会话恢复指南请参考：[会话恢复功能使用指南](docs/session-resume-guide.md)

## 📋 工作流阶段

### 1. IDEA Intake
- 输入：用户描述的想法
- 输出：结构化的 IdeaSpec（背景、目标、约束等）

### 2. Requirements (PRD)
- 输入：IdeaSpec
- 输出：产品需求文档（功能需求、验收标准等）

### 3. Design
- 输入：PRD
- 输出：技术设计文档（架构、工作流、CLI设计等）

### 4. Plan
- 输入：Design
- 输出：实施计划（C4模型、任务分解、里程碑）

### 5. Coding
- 输入：Plan
- 输出：代码结构和变更计划

### 6. Check
- 输入：Code Changes
- 输出：质量检查报告

### 7. Feedback
- 输入：Check Report + 用户反馈
- 输出：改进建议和需要重跑的阶段

### 8. Delivery
- 输入：Check Report + IdeaSpec
- 输出：最终交付报告

## 🏗️ 项目结构

```
cowork-rs/
├── crates/
│   ├── cowork-core/          # 核心库
│   │   ├── src/
│   │   │   ├── agents/       # 各阶段 Agent 实现
│   │   │   ├── artifacts/    # 数据模型定义
│   │   │   ├── memory/       # Artifact 存储
│   │   │   ├── orchestrator/ # 工作流编排
│   │   │   ├── hitl/         # 人机交互
│   │   │   └── config.rs     # 配置管理
│   │   └── Cargo.toml
│   └── cowork-cli/           # CLI 工具
│       ├── src/main.rs
│       └── Cargo.toml
├── config.toml               # 配置文件
├── Cargo.toml                # Workspace 配置
└── README.md
```

## 🧪 测试

```bash
# 运行所有测试
cargo test

# 运行库测试
cargo test --lib

# 运行特定测试
cargo test test_artifact_envelope_creation
```

当前测试覆盖：
- ✅ Artifact 数据结构测试（8个测试）
- ✅ ArtifactStore CRUD 测试（8个测试）
- ✅ 总计 16 个测试，100% 通过

## 📊 Artifact 存储

所有 Artifact 保存在 `.cowork/<session-id>/artifacts/` 目录：

```
.cowork/
└── <session-id>/
    ├── meta.json                      # Session 元数据
    └── artifacts/
        ├── idea_intake.<id>.json      # IdeaSpec JSON
        ├── idea_intake.<id>.md        # IdeaSpec Markdown
        ├── requirements.<id>.json     # PRD JSON
        ├── requirements.<id>.md       # PRD Markdown
        └── ...                        # 其他阶段
```

## 🔧 配置选项

### CLI 参数

```bash
cowork [OPTIONS] [COMMAND]

Options:
  --config <FILE>  配置文件路径 [default: config.toml]
  -h, --help       显示帮助信息

Commands:
  resume <SESSION_ID>   恢复会话
  inspect <SESSION_ID>  查看会话 artifacts
  export <SESSION_ID>   导出交付物
```

## 🤝 贡献

欢迎贡献！请参考以下步骤：

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

## 📝 开发路线图

- [x] 核心 8 阶段 Agent 实现
- [x] Artifact 持久化
- [x] HITL 基础交互
- [x] 单元测试覆盖
- [ ] Code Agent 完整实现（实际代码生成）
- [ ] Check Agent 实际检查逻辑
- [ ] Feedback 迭代循环
- [ ] 集成测试
- [ ] Web UI
- [ ] 多语言支持

## 📄 许可证

MIT License

## 🙏 致谢

- [adk-rust](https://github.com/zavora-ai/adk-rust) - Agent Development Kit
- [rig-core](https://github.com/0xPlaygrounds/rig) - Structured output inspiration

## 📧 联系

如有问题或建议，欢迎创建 Issue 或 PR。
