# Cowork Forge - AI驱动的软件开发系统

## 项目概述

Cowork Forge 是一个基于"迭代架构"的 AI 驱动软件开发系统，专为中小型项目全链路开发而设计。它通过全角色Agent协作和人机交互(HITL)机制，实现从创意原型到工具应用的高效智能开发。

## 核心特性

### 🔄 迭代驱动架构
- **起源迭代(Genesis)**：从零开始的新项目开发
- **演化迭代(Evolution)**：基于现有迭代的功能扩展和优化
- **智能继承机制**：支持完全继承、部分继承和自定义起始阶段

### 🤖 全角色Agent系统
- **产品Agent**：需求分析和产品规划
- **设计Agent**：架构设计和系统规划
- **开发Agent**：代码实现和功能开发
- **测试Agent**：代码检查和质量保证
- **交付Agent**：文档生成和项目交付

### 🛠️ 阶段式开发流程
1. **Idea** - 创意捕获和问题定义
2. **PRD** - 产品需求文档编写
3. **Design** - 系统设计和架构规划
4. **Plan** - 开发计划和任务分解
5. **Coding** - 代码实现和功能开发
6. **Check** - 代码检查和质量验证
7. **Delivery** - 文档生成和项目交付

### 🧠 智能记忆系统
- **项目级记忆**：跨迭代沉淀关键决策和设计模式
- **迭代级记忆**：当前迭代的洞察、问题和学习
- **智能查询**：基于关键词的上下文检索

### 👥 人机协作(HITL)
- 关键节点确认机制
- 多轮反馈和修订支持
- 灵活的人机切换模式

## 文档导航

- [架构设计](./architecture/) - 系统架构和设计原理
- [功能特性](./features/) - 详细功能介绍和使用方法
- [用户指南](./user-guide/) - 快速入门和使用教程
- [开发指南](./development/) - 开发环境搭建和贡献指南
- [评估报告](./evaluation/) - 项目评估和改进建议

## 适用场景

### 高度适用
- 快速原型开发和MVP验证
- 工具类应用和内部系统开发
- 个人开发者和小型团队项目
- 需要快速迭代的创意项目

### 可能受限
- 大型企业级复杂系统
- 需要严格合规流程的开发
- 大型团队协作开发
- 对性能有极致要求的系统

## 技术栈

- **后端**：Rust + Tauri
- **AI框架**：adk-rust (Agent Development Kit)
- **交互界面**：CLI + GUI (Tauri)
- **存储**：基于文件系统的持久化
- **LLM集成**：支持多种外部模型

## 快速开始

1. 克隆项目仓库
```bash
git clone https://github.com/sopaco/cowork-forge.git
cd cowork-forge
```

2. 安装依赖并构建
```bash
cargo build
```

3. 配置LLM（编辑config.toml）
```toml
[llm]
api_base_url = "https://api.example.com"
api_key = "your-api-key"
model_name = "gpt-4"
```

4. 创建新项目
```bash
cargo run -- init --name "my-project"
```

5. 开始迭代开发
```bash
cargo run -- iter "我的第一个功能" --description "实现用户登录功能"
```

## 许可证

MIT License - 详见 [LICENSE](../LICENSE) 文件