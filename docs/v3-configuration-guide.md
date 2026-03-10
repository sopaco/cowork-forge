# V3 配置系统使用指南

## 概述

V3 版本引入了数据驱动的配置系统，将原本硬编码的 Agent、Stage、Flow、Skill 和 Integration 定义转为可配置的 JSON 格式。这使系统更加灵活、可扩展，用户可以自定义开发流程而无需修改代码。

## 配置架构

配置系统由以下核心组件构成：

```
┌─────────────────────────────────────────────────────────────┐
│                    Config Registry                          │
│  (全局配置注册表，管理所有配置定义)                            │
├─────────────────────────────────────────────────────────────┤
│  Agents │ Stages │ Flows │ Skills │ Integrations            │
│  (智能体)│ (阶段) │ (流程)│ (技能) │ (外部集成)               │
└─────────────────────────────────────────────────────────────┘
```

## 1. Agent 配置

Agent（智能体）是执行具体任务的 AI 实体。每个 Agent 有自己的指令（Instruction）和工具集（Tools）。

### 配置结构

```json
{
  "id": "idea_agent",
  "name": "Idea Agent",
  "description": "捕获并结构化初始项目想法",
  "version": "1.0.0",
  "agent_type": "simple",
  "instruction": "builtin://idea_agent",
  "tools": [
    { "tool_id": "save_idea" },
    { "tool_id": "query_memory" }
  ],
  "skills": [],
  "model": {
    "temperature": 0.7,
    "max_tokens": null
  },
  "include_contents": "none",
  "tags": ["built-in", "ideation"]
}
```

### 字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | string | ✅ | 唯一标识符 |
| `name` | string | ✅ | 显示名称 |
| `description` | string | ❌ | 功能描述 |
| `version` | string | ❌ | 版本号（语义化版本） |
| `agent_type` | enum | ❌ | `simple` 或 `loop`（默认 simple） |
| `instruction` | string | ✅ | 指令引用（见下文） |
| `tools` | array | ❌ | 工具引用列表 |
| `skills` | array | ❌ | 技能引用列表 |
| `model` | object | ❌ | 模型配置 |
| `tags` | array | ❌ | 标签（用于分类） |

### 指令引用格式

Instruction 支持三种引用方式：

```json
// 1. 内置指令（推荐）
"instruction": "builtin://idea_agent"

// 2. 文件引用
"instruction": "file://./prompts/custom_agent.md"

// 3. 内联内容
"instruction": "inline://你是一个专业的代码审查助手..."
```

### 可用的内置指令

| 指令名称 | 用途 |
|----------|------|
| `idea_agent` | 创意捕获和结构化 |
| `prd_actor` | PRD 文档生成 |
| `prd_critic` | PRD 文档审核 |
| `design_actor` | 技术设计生成 |
| `design_critic` | 技术设计审核 |
| `plan_actor` | 任务计划生成 |
| `plan_critic` | 任务计划审核 |
| `coding_actor` | 代码编写 |
| `coding_critic` | 代码审核 |
| `check_agent` | 质量检查 |
| `delivery_agent` | 交付部署 |
| `summary_agent` | 总结报告 |
| `knowledge_gen_agent` | 知识提取 |
| `project_manager` | 项目管理 |

### 可用工具列表

| 工具 ID | 用途 |
|---------|------|
| **数据工具** | |
| `save_idea` | 保存创意文档 |
| `load_idea` | 加载创意文档 |
| `create_requirement` | 创建需求 |
| `update_requirement` | 更新需求 |
| `delete_requirement` | 删除需求 |
| `get_requirements` | 获取需求列表 |
| `add_feature` | 添加功能特性 |
| `update_feature` | 更新功能特性 |
| `create_task` | 创建任务 |
| `update_task_status` | 更新任务状态 |
| **文件工具** | |
| `read_file` | 读取文件 |
| `write_file` | 写入文件 |
| `list_files` | 列出文件 |
| `run_command` | 执行命令 |
| **文档工具** | |
| `load_prd_doc` | 加载 PRD 文档 |
| `save_prd_doc` | 保存 PRD 文档 |
| `load_design_doc` | 加载设计文档 |
| `save_design_doc` | 保存设计文档 |
| `load_plan_doc` | 加载计划文档 |
| `save_plan_doc` | 保存计划文档 |
| `save_delivery_report` | 保存交付报告 |
| **记忆工具** | |
| `query_memory` | 查询记忆库 |
| `save_insight` | 保存洞察 |
| `save_issue` | 保存问题 |
| `save_learning` | 保存学习记录 |
| **验证工具** | |
| `check_feature_coverage` | 检查功能覆盖 |
| `check_task_dependencies` | 检查任务依赖 |
| `check_tests` | 检查测试 |
| `check_lint` | 检查代码风格 |
| **流程控制** | |
| `goto_stage` | 跳转到指定阶段 |

---

## 2. Stage 配置

Stage（阶段）是开发流程中的一个步骤，可以包含单个 Agent 或 Actor-Critic 循环。

### 配置结构

```json
{
  "id": "prd",
  "name": "PRD Stage",
  "description": "产品需求文档生成阶段",
  "stage_type": "actor_critic",
  "actor_critic": {
    "actor": "prd_actor",
    "critic": "prd_critic",
    "max_iterations": 3
  },
  "needs_confirmation": true,
  "artifacts": [
    {
      "path": "artifacts/prd.md",
      "required": true,
      "description": "PRD 文档"
    }
  ],
  "hooks": [],
  "timeout_secs": 600,
  "tags": ["built-in", "requirements"]
}
```

### 字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | string | ✅ | 唯一标识符 |
| `name` | string | ✅ | 显示名称 |
| `stage_type` | enum | ❌ | `simple` 或 `actor_critic` |
| `agent` | string | 条件必填 | Simple 类型时引用的 Agent ID |
| `actor_critic` | object | 条件必填 | ActorCritic 类型时的配置 |
| `needs_confirmation` | bool | ❌ | 是否需要人工确认（默认 false） |
| `artifacts` | array | ❌ | 输出产物配置 |
| `hooks` | array | ❌ | 集成钩子配置 |
| `timeout_secs` | number | ❌ | 超时时间（秒） |

### Stage 类型

#### Simple 类型

简单阶段由单个 Agent 执行一次：

```json
{
  "id": "idea",
  "name": "Idea Stage",
  "stage_type": "simple",
  "agent": "idea_agent",
  "needs_confirmation": true
}
```

#### ActorCritic 类型

Actor-Critic 循环阶段，Actor 生成内容，Critic 审核并反馈：

```json
{
  "id": "coding",
  "name": "Coding Stage",
  "stage_type": "actor_critic",
  "actor_critic": {
    "actor": "coding_actor",
    "critic": "coding_critic",
    "max_iterations": 5
  }
}
```

### Hook 配置

Stage 可以配置集成钩子，在特定时机触发外部系统：

```json
{
  "hooks": [
    {
      "integration_id": "deployment-system",
      "point": "post_execute",
      "blocking": true,
      "timeout_secs": 60,
      "continue_on_failure": false
    }
  ]
}
```

Hook 触发点：

| Hook Point | 触发时机 |
|------------|----------|
| `pre_execute` | 阶段执行前 |
| `post_execute` | 阶段执行后 |
| `pre_confirmation` | 人工确认前 |
| `post_confirmation` | 人工确认后 |
| `on_failure` | 阶段失败时 |

---

## 3. Flow 配置

Flow（流程）定义了一组有序的阶段序列，构成完整的开发流程。

### 配置结构

```json
{
  "id": "default",
  "name": "Default Development Flow",
  "description": "标准的七阶段开发流程",
  "stages": [
    { "stage_id": "idea" },
    { "stage_id": "prd" },
    { "stage_id": "design" },
    { "stage_id": "plan" },
    { "stage_id": "coding" },
    { "stage_id": "check" },
    { "stage_id": "delivery" }
  ],
  "start_stage": "idea",
  "config": {
    "stop_on_failure": true,
    "save_state_on_interrupt": true,
    "memory_scope": "merged"
  }
}
```

### 字段说明

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | string | ✅ | 唯一标识符 |
| `name` | string | ✅ | 显示名称 |
| `stages` | array | ✅ | 阶段引用列表 |
| `start_stage` | string | ❌ | 起始阶段 ID |
| `config` | object | ❌ | 流程配置 |

### Stage 引用配置

```json
{
  "stage_id": "prd",
  "alias": "prd-v2",
  "condition": "has_requirements",
  "on_success": "design",
  "on_failure": "idea",
  "overrides": {
    "needs_confirmation": false,
    "timeout_secs": 300
  }
}
```

### 流程配置选项

```json
{
  "config": {
    "stop_on_failure": true,
    "max_total_time_secs": 3600,
    "save_state_on_interrupt": true,
    "memory_scope": "merged",
    "inheritance": {
      "default_mode": "partial",
      "stage_mapping": {
        "none": "idea",
        "partial": "plan",
        "full": "idea"
      }
    }
  }
}
```

### 继承模式（Inheritance Mode）

用于演进式迭代开发：

| 模式 | 说明 |
|------|------|
| `none` | 不继承，从头开始 |
| `partial` | 继承代码，从计划阶段开始 |
| `full` | 继承代码和产物，从创意阶段开始 |

---

## 4. Skill 配置

Skill（技能）是领域特定的能力扩展，可以为 Agent 提供额外的工具、提示和上下文。

### 配置结构

```json
{
  "id": "web-frontend",
  "name": "Web Frontend Development",
  "description": "Web 前端开发技能包",
  "version": "1.0.0",
  "category": "web_frontend",
  "author": "Cowork Team",
  "dependencies": [
    { "skill_id": "typescript", "optional": true }
  ],
  "tools": [
    {
      "tool_id": "generate_react_component",
      "implementation": "builtin",
      "description": "生成 React 组件",
      "enabled": true
    }
  ],
  "prompts": [
    {
      "prompt_type": "inline",
      "content": "你是 React 和 TypeScript 专家...",
      "injection_point": "prepend",
      "priority": 100
    }
  ],
  "tags": ["frontend", "react", "typescript"]
}
```

### 技能类别

| 类别 | 说明 |
|------|------|
| `general` | 通用技能 |
| `web_frontend` | Web 前端 |
| `web_backend` | Web 后端 |
| `mobile` | 移动开发 |
| `desktop` | 桌面应用 |
| `devops` | DevOps |
| `testing` | 测试 |
| `documentation` | 文档 |
| `data_processing` | 数据处理 |
| `machine_learning` | 机器学习 |
| `security` | 安全 |

### 工具实现类型

```json
// 内置工具
{ "implementation": "builtin" }

// 脚本工具
{
  "implementation": {
    "script": {
      "interpreter": "python",
      "script": "./scripts/generate.py"
    }
  }
}

// HTTP 工具
{
  "implementation": {
    "http": {
      "method": "POST",
      "url_template": "https://api.example.com/generate",
      "headers": { "Content-Type": "application/json" }
    }
  }
}
```

---

## 5. Integration 配置

Integration（集成）用于连接外部系统，如部署平台、需求管理系统等。

### 配置结构

```json
{
  "id": "deployment-system",
  "name": "Deployment System",
  "description": "连接部署平台",
  "integration_type": "rest_api",
  "connection": {
    "base_url": "https://deploy.example.com/api/v1",
    "endpoints": {
      "deploy": {
        "path": "/deployments",
        "method": "POST",
        "body_template": "{\"iteration_id\": \"{{iteration_id}}\"}"
      }
    }
  },
  "auth": {
    "auth_type": "bearer_token",
    "credentials": {
      "env_var": { "name": "DEPLOY_API_TOKEN" }
    }
  },
  "retry": {
    "max_attempts": 3,
    "initial_delay_ms": 1000
  },
  "enabled": true
}
```

### 认证类型

| 类型 | 配置示例 |
|------|----------|
| 无认证 | `"auth_type": "none"` |
| API Key | `"auth_type": "api_key"` |
| Bearer Token | `"auth_type": "bearer_token"` |
| Basic Auth | `"auth_type": "basic_auth"` |
| OAuth2 | `"auth_type": "oauth2"` |

### 凭据来源

```json
// 环境变量（推荐）
{ "credentials": { "env_var": { "name": "API_TOKEN" } } }

// 文件
{ "credentials": { "file": { "path": "/etc/secrets/token" } } }

// 密钥管理器
{ "credentials": { "secret_manager": { "key": "aws/secrets/api-token" } } }
```

---

## 配置文件位置

### 默认配置

内置配置位于：
```
crates/cowork-core/src/config_definition/default_configs/
├── agents/built-in/    # Agent 定义
├── stages/             # Stage 定义
└── flows/              # Flow 定义
```

### 用户配置

用户自定义配置位于应用数据目录：
```
Windows: %APPDATA%/com.cowork-forge.app/config/
macOS: ~/Library/Application Support/com.cowork-forge.app/config/
Linux: ~/.config/com.cowork-forge.app/config/
```

配置目录结构：
```
config/
├── agents/
│   ├── custom_agent.json
│   └── ...
├── stages/
│   └── ...
├── flows/
│   └── custom_flow.json
├── skills/
│   └── my-skill/
│       ├── manifest.json
│       └── prompts/
└── integrations/
    └── deployment.json
```

## GUI 配置界面使用

在应用中通过 **配置** 菜单进入配置管理界面，可以：

### 工具栏按钮

- **Refresh** - 刷新配置列表
- **Reset to Defaults** - 重置所有配置到内置默认值（会清除自定义配置）

### 1. Agent 管理

- 查看所有已注册的 Agent
- 新建自定义 Agent
- 编辑 Agent 配置
- 删除自定义 Agent

**创建新 Agent 步骤：**
1. 点击「新建 Agent」
2. 填写基本信息（ID、名称、描述）
3. 选择指令来源（内置/文件/内联）
4. 添加所需工具
5. 配置模型参数
6. 保存配置

### 2. Flow 管理

- 查看可用流程（预设流程和自定义流程分组显示）
- 创建自定义流程
- 调整阶段顺序
- 配置阶段覆盖项
- 设置默认流程

**流程分组：**
- **预设流程 (Preset Flows)**：系统内置流程，只读不可编辑
- **自定义流程 (Custom Flows)**：用户创建的流程，可编辑和删除

**创建自定义 Flow 步骤：**
1. 点击「新建 Flow」
2. 填写基本信息
3. 添加阶段（从已注册阶段中选择）
4. 配置起始阶段和流程选项
5. 保存配置

**设置默认流程：**
默认流程决定了新迭代使用的工作流程。设置方法：
1. 在流程列表中找到目标流程
2. 点击「Set Default」按钮（星形图标）
3. 当前默认流程会显示金色标签标识

默认流程设置会持久化保存，重启应用后保持不变。

### 3. Skill 管理

- 浏览可用技能
- 查看技能详情
- 安装/卸载技能

### 4. Integration 管理

- 配置外部系统集成
- 设置认证凭据
- 测试连接状态

---

## 示例：创建自定义开发流程

### 场景：简化版原型开发流程

假设我们需要一个快速的「创意 → 原型」流程：

**1. 创建自定义 Flow 配置：**

```json
{
  "id": "quick-prototype",
  "name": "Quick Prototype Flow",
  "description": "快速原型开发流程，跳过设计阶段",
  "stages": [
    { "stage_id": "idea" },
    { 
      "stage_id": "prd",
      "overrides": { "needs_confirmation": false }
    },
    { 
      "stage_id": "coding",
      "overrides": { 
        "timeout_secs": 300,
        "needs_confirmation": false 
      }
    },
    { "stage_id": "delivery" }
  ],
  "start_stage": "idea",
  "config": {
    "stop_on_failure": false,
    "save_state_on_interrupt": true
  }
}
```

**2. 将配置保存到：**
```
%APPDATA%/com.cowork-forge.app/config/flows/quick-prototype.json
```

**3. 重启应用或刷新配置**

**4. 在项目中使用新流程**

---

## 最佳实践

### 1. Agent 命名规范

- 使用小写和下划线：`my_custom_agent`
- 名称体现职责：`code_reviewer`
- 添加版本标签：`my_agent_v2`

### 2. 工具选择原则

- 只授予必要的工具
- 避免工具冗余
- 优先使用内置工具

### 3. Stage 设计建议

- 单一职责：每个阶段只做一件事
- 合理设置超时
- 重要阶段启用人工确认
- 关键产物标记为 required

### 4. Flow 设计建议

- 保持流程简洁
- 合理使用阶段覆盖
- 设置合理的继承模式

### 5. 安全建议

- 使用环境变量存储敏感信息
- 不要在配置中硬编码凭据
- 定期更新认证令牌

---

## 故障排除

### 配置加载失败

1. 检查 JSON 格式是否正确
2. 验证引用的 ID 是否存在
3. 查看应用日志获取详细错误信息

### Agent 执行异常

1. 检查工具 ID 是否有效
2. 确认指令引用路径正确
3. 验证模型配置是否合理

### 集成连接失败

1. 检查网络连接
2. 验证认证凭据
3. 检查 API 端点配置
