# 快速入门指南

本指南将帮助您在5分钟内快速上手 Cowork Forge，体验AI驱动的软件开发全流程。

## 系统要求

- **操作系统**: Windows 10/11, macOS 10.15+, 或 Linux (Ubuntu 20.04+)
- **内存**: 最少 4GB RAM (推荐 8GB+)
- **存储**: 至少 2GB 可用空间
- **网络**: 稳定的互联网连接 (用于LLM服务)

## 安装步骤

### 1. 克隆项目

```bash
git clone https://github.com/sopaco/cowork-forge.git
cd cowork-forge
```

### 2. 安装 Rust 依赖

```bash
# 确保已安装 Rust (推荐使用 rustup)
rustup --version

# 安装项目依赖并构建
cargo build --release
```

### 3. 配置 LLM 服务

创建配置文件 `config.toml`：

```toml
[llm]
api_base_url = "https://api.openai.com/v1"
api_key = "your-api-key-here"
model_name = "gpt-4"

# 可选：嵌入模型配置
[embedding]
api_base_url = "https://api.openai.com/v1"
api_key = "your-api-key-here"
model_name = "text-embedding-ada-002"
```

**支持的LLM提供商**:
- OpenAI (GPT-3.5, GPT-4)
- Anthropic Claude
- Google Gemini
- 本地模型 (Ollama, LM Studio)
- 自定义API (需兼容OpenAI格式)

### 4. 验证安装

```bash
# 检查CLI是否正常工作
cargo run -- --help

# 预期输出：
# cowork 
# AI-powered software development system - Iteration Architecture
#
# USAGE:
#     cowork [OPTIONS] <COMMAND>
#
# COMMANDS:
#     iter     Create and execute a new iteration
#     list     List all iterations
#     show     Show iteration details
#     continue Continue a paused iteration
#     init     Initialize a new project
#     status   Show project status
#     delete   Delete an iteration
```

## 创建第一个项目

### 1. 初始化项目

```bash
# 在新目录中初始化项目
mkdir my-first-project
cd my-first-project
cargo run -- init --name "我的第一个项目"
```

系统将创建项目结构：
```
my-first-project/
├── .cowork-v2/
│   ├── project.json
│   └── memory/
└── config.toml
```

### 2. 创建第一个迭代

```bash
# 创建一个简单的Web应用迭代
cargo run -- iter "创建待办事项Web应用" --description "一个简单的待办事项管理应用，支持添加、删除和标记完成任务"
```

### 3. 观察执行过程

系统将自动执行以下流程：

1. **Idea阶段** - 捕获和结构化创意
2. **PRD阶段** - 创建产品需求文档
3. **Design阶段** - 设计系统架构
4. **Plan阶段** - 制定开发计划
5. **Coding阶段** - 生成代码实现
6. **Check阶段** - 检查代码质量
7. **Delivery阶段** - 生成交付报告

### 4. 人机交互体验

在关键阶段，系统会请求您的确认：

```bash
=== Idea 阶段完成 ===

📝 创意文档已生成: .cowork-v2/iterations/iter-1-1234567890/artifacts/idea.md

请选择操作:
[1] 查看内容
[2] 编辑内容
[3] 继续
[4] 提供反馈

请输入选择 (1-4): 
```

**推荐操作**:
1. 选择 `1` 查看生成的内容
2. 根据需要选择 `2` 进行编辑
3. 确认无误后选择 `3` 继续下一阶段

## 查看结果

### 1. 检查迭代状态

```bash
# 查看项目状态
cargo run -- status

# 查看迭代列表
cargo run -- list

# 查看当前迭代详情
cargo run -- show
```

### 2. 探索生成的制品

```bash
# 查看生成的文件结构
tree .cowork-v2

# 预期结构：
# .cowork-v2/
# ├── iterations/
# │   └── iter-1-1234567890/
# │       ├── artifacts/
# │       │   ├── idea.md      # 创意文档
# │       │   ├── prd.md       # 需求文档
# │       │   ├── design.md    # 设计文档
# │       │   ├── plan.md      # 开发计划
# │       │   └── delivery.md  # 交付报告
# │       └── iteration.json   # 迭代元数据
# └── memory/
#     ├── decisions/           # 关键决策
#     ├── patterns/           # 设计模式
#     └── context/            # 项目上下文
```

### 3. 运行生成的应用

如果成功生成了Web应用，可以按照生成的说明运行：

```bash
# 按照delivery.md中的说明运行应用
cd path/to/generated/app
npm install
npm run dev
```

## 创建演化迭代

现在让我们为基础应用添加新功能：

```bash
# 添加用户认证功能
cargo run -- iter "添加用户认证" --description "实现用户注册、登录和会话管理" --base "iter-1-1234567890"
```

注意观察系统如何：
1. 自动检测这是功能扩展，选择合适的起始阶段
2. 继承现有的设计和代码
3. 基于现有内容进行增量开发

## 故障排除

### 常见问题

#### 1. LLM API调用失败

**错误**: `LLM generation failed: API request failed`

**解决方案**:
- 检查 `config.toml` 中的 API 密钥是否正确
- 确认网络连接正常
- 验证 API 端点是否有效
- 检查API配额是否充足

#### 2. 编译错误

**错误**: `cargo build` 失败

**解决方案**:
- 确保使用最新版的 Rust: `rustup update`
- 清理编译缓存: `cargo clean`
- 安装缺少的系统依赖

#### 3. 权限错误

**错误**: `Permission denied` 或权限相关错误

**解决方案**:
- Windows: 以管理员身份运行终端
- macOS/Linux: 检查文件权限: `ls -la`
- 确保对项目目录有读写权限

### 获取帮助

1. 查看详细错误日志:
   ```bash
   RUST_LOG=debug cargo run -- [命令]
   ```

2. 检查配置有效性:
   ```bash
   # 使用CLI检查配置
   cargo run -- config check
   ```

3. 查看所有可用命令:
   ```bash
   cargo run -- --help
   ```

## 下一步

恭喜您已经完成 Cowork Forge 的快速入门！接下来建议：

1. 阅读[核心概念文档](./core-concepts.md)深入理解系统设计
2. 学习[迭代管理高级技巧](../features/iteration-management.md)
3. 探索[人机协作最佳实践](../features/hitl-features.md)
4. 尝试更复杂的[实战案例](./web-app-case.md)

## 参考资源

- [项目主页](https://github.com/sopaco/cowork-forge)
- [API文档](../architecture/)
- [示例项目](https://github.com/sopaco/cowork-forge-examples)
- [社区讨论](https://github.com/sopaco/cowork-forge/discussions)