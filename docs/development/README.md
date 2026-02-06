# 开发指南

本目录包含 Cowork Forge 项目的开发文档，为希望贡献代码、扩展功能或深入理解系统实现的开发者提供详细指导。

## 开发环境搭建

- [环境准备](./environment-setup.md) - 开发环境依赖和工具链配置
- [项目结构](./project-structure.md) - 代码组织和架构说明
- [构建系统](./build-system.md) - Cargo 工作空间和构建流程

## 核心开发指南

- [添加新Agent](./adding-agents.md) - 创建和集成新的角色Agent
- [扩展开发阶段](./extending-stages.md) - 添加自定义开发阶段
- [开发工具集](./tool-development.md) - 创建自定义Agent工具
- [记忆系统扩展](./memory-extensions.md) - 扩展项目记忆功能

## 内部架构深析

- [领域模型](../architecture/domain-model.md) - 核心领域模型设计
- [事件系统](../architecture/event-system.md) - 事件驱动架构实现
- [持久化层](../architecture/storage-design.md) - 数据存储和持久化机制

## 贡献指南

- [代码规范](./coding-standards.md) - 编码风格和最佳实践
- [提交规范](./commit-guidelines.md) - 提交信息和PR流程
- [测试策略](./testing-strategy.md) - 测试策略和质量保证
- [发布流程](./release-process.md) - 版本发布和变更日志

## 开发资源

### 必读文档

1. **[迭代架构设计](../architecture/iteration-architecture.md)** - 理解系统核心设计理念
2. **[Agent系统](../architecture/agent-system.md)** - 掌握Agent协作机制
3. **[领域模型](../architecture/domain-model.md)** - 了解核心领域实体

### 关键目录

```
crates/
├── cowork-core/     # 核心业务逻辑
│   ├── src/
│   │   ├── agents/  # Agent实现
│   │   ├── domain/  # 领域模型
│   │   ├── pipeline/ # 迭代执行管道
│   │   └── tools/   # Agent工具集
├── cowork-cli/      # 命令行接口
└── cowork-gui/      # 桌面GUI应用
```

### 开发工具

- **IDE配置**: [VS Code配置](./ide-setup.md)
- **调试技巧**: [调试指南](./debugging.md)
- **性能分析**: [性能优化](./performance.md)

## 开发流程

### 1. 环境准备

```bash
# 克隆仓库
git clone https://github.com/sopaco/cowork-forge.git
cd cowork-forge

# 安装开发依赖
cargo install cargo-watch cargo-flamegraph

# 设置pre-commit钩子
./scripts/setup-dev.sh
```

### 2. 开发工作流

```bash
# 创建功能分支
git checkout -b feature/your-feature-name

# 开发过程中持续测试
cargo watch -x 'test --workspace'

# 构建并运行
cargo run --bin cowork -- init --name "test-project"

# 运行特定测试
cargo test -p cowork-core agent_system
```

### 3. 提交代码

```bash
# 代码格式化和检查
cargo fmt
cargo clippy -- -D warnings

# 运行完整测试套件
cargo test --workspace

# 提交代码
git add .
git commit -m "feat: add new agent type"
git push origin feature/your-feature-name
```

## 社区参与

### 参与方式

- **Bug报告**: 通过[GitHub Issues](https://github.com/sopaco/cowork-forge/issues)提交
- **功能建议**: 在[Discussions](https://github.com/sopaco/cowork-forge/discussions)中讨论
- **代码贡献**: 提交Pull Request
- **文档改进**: 直接提交文档相关PR

### 沟通渠道

- **GitHub Discussions**: 功能讨论和问答
- **开发者邮件列表**: [dev@cowork-forge.org](mailto:dev@cowork-forge.org)
- **技术社区**: [Discord服务器](https://discord.gg/cowork-forge)

## 开发路线图

### 当前开发重点

- [ ] GUI界面增强和可视化
- [ ] 测试系统和质量保证完善
- [ ] 版本控制集成
- [ ] 插件系统开发

### 长期目标

- [ ] 分布式Agent协作
- [ ] 多用户实时协作
- [ ] 企业级功能支持
- [ ] 云端服务集成

## 获取帮助

### 文档资源

- [API参考文档](../architecture/)
- [示例代码](https://github.com/sopaco/cowork-forge-examples)
- [常见问题](../user-guide/troubleshooting.md)

### 技术支持

- **代码问题**: 创建GitHub Issue
- **设计讨论**: 在Discussions中发起
- **紧急问题**: 联系核心维护者

## 版本兼容性

| Cowork Forge版本 | Rust最低版本 | 说明 |
|------------------|-------------|------|
| 2.0.x | 1.70+ | 当前稳定版 |
| 2.1.x (开发中) | 1.72+ | 包含新功能 |

## 开发许可证

本项目的开发遵循 [MIT License](../LICENSE)，贡献者需要签署[贡献者许可协议(CLA)](https://cla-assistant.io/sopaco/cowork-forge)。

感谢您对 Cowork Forge 项目的关注和贡献！