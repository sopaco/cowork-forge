# 边界接口报告

## CLI 接口

Cowork Forge 通过 `cowork` CLI 二进制提供完整的命令行界面。所有的子命令和参数在 `crates/cowork-cli/src/main.rs:12-117` 使用 clap derive 宏定义。

### 命令：cowork init
- **描述**：初始化一个新项目，创建 `.cowork-v2/` 目录结构
- **参数**：
  | 参数 | 类型 | 是否必须 | 默认值 | 说明 |
  |------|------|---------|-------|------|
  | `-n, --name` | String | 否 | - | 项目名称，如果不指定则交互式输入 |
- **使用示例**：
  ```bash
  cowork init --name "My Project"
  ```

### 命令：cowork iter
- **描述**：创建并执行一个新的迭代——这是最核心的命令
- **参数**：
  | 参数 | 类型 | 是否必须 | 默认值 | 含义解读 |
  |------|------|---------|-------|---------|
  | `title` | String | 是 | - | 迭代标题/想法描述——这就是你的"需求"，用自然语言描述你想要的 |
  | `-d, --description` | String | 否 | - | 更详细的迭代描述，如果想法很复杂可以在这里展开说明 |
  | `-b, --base` | String | 否 | - | 基础迭代 ID，用于创建演化迭代（继承前一个迭代的代码或制品） |
  | `-i, --inherit` | String | 否 | `"full"` | 继承模式：`none`（全新开始）、`full`（复制代码+制品）、`partial`（只复制制品，重新生成代码） |
- **使用示例**：
  ```bash
  cowork iter --project "my-project" "Build a REST API for task management"
  cowork iter --project "my-project" --base iter-2 --inherit partial "Add user authentication"
  ```

### 命令：cowork list
- **描述**：列出项目的所有迭代
- **参数**：
  | 参数 | 类型 | 是否必须 | 默认值 | 说明 |
  |------|------|---------|-------|------|
  | `-a, --all` | bool | 否 | false | 显示所有迭代，包括已完成的 |
- **使用示例**：
  ```bash
  cowork list
  cowork list --all
  ```

### 命令：cowork show
- **描述**：显示指定迭代的详细信息
- **参数**：
  | 参数 | 类型 | 是否必须 | 默认值 | 说明 |
  |------|------|---------|-------|------|
  | `iteration_id` | String | 否 | 当前迭代 | 要查看的迭代 ID |
- **使用示例**：
  ```bash
  cowork show
  cowork show iter-1-1234567890
  ```

### 命令：cowork continue
- **描述**：继续执行已暂停的迭代
- **参数**：
  | 参数 | 类型 | 是否必须 | 默认值 | 说明 |
  |------|------|---------|-------|------|
  | `iteration_id` | String | 否 | 当前迭代 | 要继续的迭代 ID |
- **使用示例**：
  ```bash
  cowork continue
  ```

### 命令：cowork status
- **描述**：显示项目当前状态
- **参数**：无
- **使用示例**：
  ```bash
  cowork status
  ```

### 命令：cowork delete
- **描述**：删除指定迭代
- **参数**：
  | 参数 | 类型 | 是否必须 | 默认值 | 说明 |
  |------|------|---------|-------|------|
  | `iteration_id` | String | 是 | - | 要删除的迭代 ID |
- **使用示例**：
  ```bash
  cowork delete iter-1-1234567890
  ```

### 命令：cowork import
- **描述**：导入已有项目——分析现有代码结构并反向工程生成文档
- **参数**：
  | 参数 | 类型 | 是否必须 | 默认值 | 含义解读 |
  |------|------|---------|-------|---------|
  | `path` | String | 是 | - | 已有项目的目录路径 |
  | `-n, --name` | String | 否 | 目录名 | 在 Cowork Forge 中使用的项目名称 |
  | `--idea` | bool | 否 | true | 是否生成 idea.md |
  | `--prd` | bool | 否 | true | 是否生成 prd.md |
  | `--design` | bool | 否 | true | 是否生成 design.md |
  | `--plan` | bool | 否 | true | 是否生成 plan.md |
  | `--template-only` | bool | 否 | false | 仅使用模板生成（不使用 LLM） |
- **使用示例**：
  ```bash
  cowork import /path/to/existing/project
  cowork import /path/to/project --template-only
  ```

### 命令：cowork config
- **描述**：配置 LLM 设置（API 地址、密钥、模型等）
- **参数**：无（交互式配置）
- **使用示例**：
  ```bash
  cowork config
  ```

## 配置结构

配置文件位于系统应用数据目录的 `config.toml`：

| 配置项 | 类型 | 含义解读 |
|-------|------|---------|
| `[llm].api_base_url` | String | LLM API 地址——决定用哪家服务商，默认是 OpenAI |
| `[llm].api_key` | String | API 密钥——敏感信息，建议用环境变量替代硬编码 |
| `[llm].model_name` | String | 使用的模型名——如 `gpt-5` |
| `[embedding].api_base_url` | String | 嵌入模型 API 地址（可选） |
| `[coding_agent].enabled` | bool | 是否启用外部编码 Agent |
| `[coding_agent].agent_type` | String | 外部 Agent 类型：opencode/iflow/codex/gemini/claude |
| `[coding_agent].command` | String | 启动外部 Agent 的命令 |
| `[coding_agent].transport` | String | 通信方式：stdio 或 websocket |

**示例配置**：
```toml
[llm]
api_base_url = "https://api.openai.com/v1"
api_key = "sk-your-openai-api-key"
model_name = "gpt-5"

[embedding]
api_base_url = "https://your-embedding-api.com/v1"
api_key = "your-embedding-api-key"
model_name = "text-embedding-3-small"

[coding_agent]
enabled = true
agent_type = "opencode"
command = "bun"
args = ["x", "opencode-ai", "acp"]
transport = "stdio"
```

## 集成建议

**通过 CLI 集成到 CI/CD**：
```bash
# 在 CI 流水线中初始化项目和创建迭代
cowork init --name "My Project"
cowork iter --project "my-project" "Auto-generated feature from CI"
```

**通过外部 Agent 协议集成**：
Cowork Forge 支持通过 ACP（Agent Client Protocol）集成外部编码 Agent。配置 `[coding_agent]` 部分后，Coding 阶段将自动调用外部 Agent 而非内置 Agent。

**通过 Integration Hook 集成**：
配置 Integration 定义后，可以在特定阶段完成后触发 webhook 调用外部系统（如部署平台、需求管理系统）。
