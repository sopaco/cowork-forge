Cowork Forge 技术架构文档

## 1. 核心功能

Cowork Forge 是一个完整的 AI 驱动软件开发团队系统，通过模拟真实开发团队的角色分工，实现从创意到交付的全链路智能化开发。

### 主要功能
- 7 阶段开发工作流：Idea → PRD → Design → Plan → Coding → Check → Delivery
- 专业化 AI Agents：8 个专业智能体（Idea、PRD Loop、Design Loop、Plan Loop、Coding Loop、Check、Delivery）
- 迭代驱动架构：支持 Genesis（起源）和 Evolution（演化）两种迭代类型
- 人机协作验证：关键决策点保留人工确认机制（HITL，在 Pipeline 层处理）
- 增量代码更新：智能增量分析，只更新受影响文件
- 文件安全机制：所有文件操作限制在 iteration workspace 内
- 项目知识管理：自动生成迭代摘要和项目知识，支持 Evolution 迭代的知识注入

### Iteration 机制的作用
- 将软件开发抽象为可管理、可继承、可演进的迭代单元
- 每个迭代包含完整的开发生命周期和独立制品
- 通过继承机制实现功能增量开发
- 提供版本化的制品管理和上下文传递

---

## 2. 架构流程

### V2 架构核心设计理念
基于"迭代驱动"的核心理念，构建 AI 与人类协作的软件开发系统。系统通过模拟真实开发团队的角色分工和协作流程，将大型语言模型的能力结构化、流程化。

### 架构概览
```
用户接口层 (CLI/GUI)
    ↓
交互抽象层 (InteractiveBackend)
    ↓
核心服务层
├── 迭代执行器
├── 开发管道
├── 阶段引擎
└── Agent 运行时
    ↓
领域模型层
├── 项目管理
├── 迭代控制
├── 记忆管理
└── 制品管理
    ↓
基础设施层
├── LLM 服务
├── 文件存储
└── 配置管理
```

### Iteration 生命周期
Draft (草稿) → Running (运行中) → Paused (已暂停) → Completed (已完成) / Failed (失败)

### Pipeline 执行流程
1. 加载迭代和项目数据
2. 准备工作空间（根据继承模式复制或初始化）
3. 创建执行上下文
4. 确定起始阶段（智能分析变更描述）
5. Evolution 迭代：注入 base iteration 的项目知识到迭代记忆
6. 按顺序执行阶段序列
7. 每个阶段完成后更新迭代状态
8. 验证 Artifacts 是否生成（自动检查，未生成则重试）
9. 如果是关键阶段（Idea、PRD、Design、Plan），触发 HITL 用户审查
10. 生成并保存制品
11. 迭代完成后生成文档摘要（Summary Agent）
12. 生成迭代项目知识（Knowledge Generation Agent）
13. 保存项目知识到 Project Memory
14. 最终完成迭代

### Agent 协作机制
- **顺序协作**：Agent 按预定义顺序执行
- **Actor-Critic 循环协作**：复杂阶段（PRD、Design、Plan、Coding）采用内部循环提升质量
- **记忆共享协作**：所有 Agent 通过共享的记忆系统获取上下文
- **外层 HITL**：用户审查在 Pipeline 层进行，避免与 Critic 判断冲突
- **Artifacts 验证**：每个阶段完成后检查文件是否生成，未生成则自动重试（最多 3 次）

### Prompt 传递机制
- **SimpleInvocationContext**：封装用户消息并传递给 LLM
- **SimpleSession.conversation_history()**：返回存储的消息历史，确保 LLM 接收到完整的上下文
- **stage-specific prompts**：每个阶段都有明确的任务指导和工具使用说明

### 文件安全机制
- **Workspace 隔离**：所有文件操作限制在 `.cowork-v2/iterations/{id}/workspace` 目录内
- **相对路径验证**：只接受相对路径，拒绝绝对路径和 `..` 访问
- **路径安全检查**：验证路径是否在 workspace 目录内（相对于 workspace 验证）
- **Delivery 同步**：只在 Delivery 阶段将代码复制到项目根目录

---

## 3. 模块设计

### Crate 职责划分
```
cowork-forge/
├── cowork-cli/           # CLI 命令行接口
├── cowork-core/          # 核心业务逻辑
│   ├── domain/           # 领域模型（Project、Iteration、Memory）
│   ├── pipeline/         # 工作流编排
│   ├── agents/           # Agent 构建器（基于 adk-rust）
│   ├── instructions/     # Agent 指令模板
│   ├── tools/            # 工具实现（文件、命令、数据、验证、知识）
│   ├── llm/              # LLM 集成（配置、速率限制）
│   ├── data/             # 数据模型（Requirements、Features、Tasks）
│   ├── persistence/      # 持久化
│   ├── interaction/      # 交互抽象（CLI、GUI Backend）
│   └── storage/          # 文件存储
└── cowork-gui/           # GUI 桌面应用（基于 Tauri）
```

### 关键模块实现

#### Domain 模块
- **Project**：项目根实体，包含迭代列表和元数据
- **Iteration**：迭代实体，支持继承机制和生命周期管理
- **Memory**：记忆系统，支持项目级和迭代级记忆
  - **ProjectMemory**：项目级记忆，存储决策、模式、上下文
    - `iteration_knowledge: HashMap<String, IterationKnowledge>`：存储每个迭代的项目知识
  - **IterationMemory**：迭代级记忆，存储洞察、反馈历史
  - **MemoryStore**：持久化接口，支持保存和加载记忆

#### Pipeline 模块
- **IterationExecutor**：迭代执行器，管理完整生命周期
- 支持重试机制（最多 3 次）、反馈循环（最多 5 轮）
- 实现自愈机制（Check 失败时返回 Coding 阶段修复）
- 支持暂停/继续和失败重试
- Artifacts 验证：每个阶段完成后检查文件是否生成，未生成则自动重试
- 外层 HITL：在 Pipeline 层触发用户审查，避免与 Critic 判断冲突
- 知识生成：迭代完成后自动生成文档摘要和项目知识
- 知识注入：Evolution 迭代启动时注入 base iteration 的项目知识

#### Agent 系统
基于 adk-rust 框架构建：
- 使用 `LlmAgentBuilder` 创建单一 Agent
- 使用 `LoopAgent` 实现 Actor-Critic 循环（max_iterations=1）
- 不使用 `SequentialAgent`，避免终止问题
- HITL 工具已从 Actor 中移除，改在 Pipeline 层处理
- 所有 Actor 都配置了必需的 Save 工具
- 新增 **Summary Agent**：生成文档摘要
- 新增 **Knowledge Generation Agent**：生成项目知识

#### 数据模型
- **Requirements**：需求列表，包含优先级和验收标准
- **FeatureList**：功能列表，关联需求和任务
- **DesignSpec**：设计规范，包含架构和组件
- **ImplementationPlan**：实施计划，包含里程碑和任务

#### 持久化机制
基于文件系统的简单持久化：
- 使用 `.cowork-v2` 目录组织数据
- 结构化子目录：`iterations/`、`memory/`、`workspace/`

---

## 4. 技术实现

### adk-rust 使用方式
```rust
// 创建单一 Agent
let agent = LlmAgentBuilder::new("idea_agent")
    .instruction(IDEA_AGENT_INSTRUCTION)
    .model(model)
    .tool(Arc::new(SaveIdeaTool))
    .include_contents(IncludeContents::None)
    .build()?;

// 创建 LoopAgent（Actor-Critic）
let loop_agent = LoopAgent::new(
    "prd_loop",
    vec![Arc::new(prd_actor), Arc::new(prd_critic)]
).with_max_iterations(1);
```

### Prompt 传递实现
```rust
// 创建用户消息
let initial_content = Content::new("user").with_text(prompt);

// 创建 InvocationContext
let invocation_ctx = Arc::new(SimpleInvocationContext::new(
    ctx, 
    &initial_content, 
    agent.clone()
));

// Session 存储消息历史
struct SimpleSession {
    messages: Vec<Content>,  // 存储用户消息
    // ...
}

impl adk_core::Session for SimpleSession {
    fn conversation_history(&self) -> Vec<Content> {
        self.messages.clone()  // 返回存储的消息
    }
    // ...
}
```

### 工具系统设计

#### 核心原则：权限最小化和专用工具

#### 工具分类：
1. **Artifact 专用工具**（非编码阶段）：
   - Save 工具：SaveIdeaTool、SavePrdDocTool、SaveDesignDocTool、SavePlanDocTool、SaveDeliveryReportTool
   - Load 工具：LoadIdeaTool、LoadPrdDocTool、LoadDesignDocTool、LoadPlanDocTool

2. **文件操作工具**（编码阶段专用）：
   - ReadFileTool、WriteFileTool、ListFilesTool
   - ReadFileTruncatedTool：支持智能截断的文件读取（用于知识生成）
   - ReadFileWithLimitTool：带行数限制的文件读取（用于知识生成）
   - 安全特性：所有操作限制在 iteration workspace 内

3. **命令执行工具**（编码/检查阶段）：
   - RunCommandTool（带超时和安全检查）

4. **部署工具**（Delivery 阶段）：
   - CopyWorkspaceToProjectTool：将 workspace 代码复制到项目根目录

5. **数据管理工具**（所有阶段）：
   - CreateRequirementTool、CreateTaskTool、GetRequirementsTool、GetDesignTool、GetPlanTool 等

6. **验证工具**（检查阶段）：
   - CheckTestsTool、CheckLintTool、CheckFeatureCoverageTool、CheckTaskDependenciesTool 等

7. **知识管理工具**（迭代完成后）：
   - LoadDocumentSummaryTool：加载文档摘要
   - SaveSummaryTool：保存文档摘要（Summary Agent 使用）
   - ListWorkspaceFilesTool：列出工作空间文件（智能截断）
   - LoadProjectKnowledgeTool：加载项目级知识
   - SaveProjectKnowledgeTool：保存项目级知识（Knowledge Generation Agent 使用）

### 文件工具安全实现
所有文件操作工具都遵循以下安全规则：
```rust
// 1. 获取 iteration workspace 路径
let iteration_id = get_iteration_id()?;
let workspace_dir = iteration_store.workspace_path(&iteration_id)?;

// 2. 验证路径安全性（相对于 workspace）
let safe_path = validate_path_security_within_workspace(path, &workspace_dir)?;

// 3. 构造完整路径
let full_path = workspace_dir.join(&safe_path);

// 4. 执行文件操作
fs::write(&full_path, content)?;
```

**安全规则**：
- ✅ 接受相对路径：`src/index.html`
- ❌ 拒绝绝对路径：`/tmp/file.txt`
- ❌ 拒绝父目录访问：`../config.toml`
- ✅ 自动拼接到 workspace

### Artifacts 验证机制
在 executor.rs 中实现了 `check_artifact_exists` 方法，用于验证各阶段的 Artifacts 是否生成：

```rust
async fn check_artifact_exists(&self, stage_name: &str, workspace: &Path) -> bool {
    let iteration_dir = workspace.parent().unwrap_or(workspace);
    let artifacts_dir = iteration_dir.join("artifacts");

    match stage_name {
        "idea" => artifacts_dir.join("idea.md").exists(),
        "prd" => artifacts_dir.join("prd.md").exists() ||
                 iteration_dir.join("data/requirements.json").exists(),
        "design" => artifacts_dir.join("design.md").exists() ||
                   iteration_dir.join("data/design_spec.json").exists(),
        "plan" => artifacts_dir.join("plan.md").exists() ||
                 iteration_dir.join("data/implementation_plan.json").exists(),
        "coding" => {
            // 检查 workspace 中是否有代码文件
            let code_extensions = ["rs", "js", "jsx", "ts", "tsx", "py", "java", "go", "cpp", "c", "h"];
            // 遍历 workspace 检查代码文件
        }
        "delivery" => artifacts_dir.join("delivery_report.md").exists(),
        _ => true,
    }
}
```

---

## 5. 目录结构

### V2 架构目录组织
```
.cowork-v2/
├── iterations/              # 迭代数据
│   └── {iteration_id}/      # 每个迭代一个目录
│       ├── iteration.json   # 迭代元数据
│       ├── artifacts/       # 迭代制品
│       │   ├── idea.md
│       │   ├── prd.md
│       │   ├── design.md
│       │   ├── plan.md
│       │   └── delivery.md
│       ├── data/            # 结构化数据
│       │   ├── requirements.json
│       │   ├── feature_list.json
│       │   ├── design_spec.json
│       │   ├── implementation_plan.json
│       │   └── code_metadata.json
│       ├── session/         # 会话数据
│       │   ├── meta.json
│       │   └── feedback.json
│       ├── workspace/       # 代码工作空间
│       │   ├── src/
│       │   ├── components/
│       │   └── ...
│       └── logs/            # 日志文件
├── memory/                  # 记忆系统
│   ├── project/             # 项目级记忆
│   │   └── memory.json      # 包含 iteration_knowledge HashMap
│   └── iterations/          # 迭代级记忆
│       └── {iteration_id}/
│           └── memory.json
└── workspace/               # 共享工作空间
```

### InheritanceMode 实现
```rust
pub enum InheritanceMode {
    None,     // Genesis only - fresh start
    Full,     // Copy all workspace files from base
    Partial,  // Copy only artifacts and config, regenerate code
}
```

---

## 6. 数据流说明

### 开发流程数据流
```
用户输入创意
    ↓
Idea Agent 生成 idea.md（在 artifacts 目录）
    ↓
PRD Loop Agent 生成 requirements.json（在 data 目录）和 prd.md（在 artifacts 目录）
    ↓
Design Loop Agent 生成 design_spec.json（在 data 目录）和 design.md（在 artifacts 目录）
    ↓
Plan Loop Agent 生成 implementation_plan.json（在 data 目录）和 plan.md（在 artifacts 目录）
    ↓
Coding Loop Agent 生成代码文件（在 workspace 目录）
    ↓
Check Agent 验证质量和完整性（检查 workspace 和 data）
    ↓
Delivery Agent 生成 delivery.md（在 artifacts 目录）并复制代码到项目根目录
    ↓
Summary Agent 生成文档摘要
    ↓
Knowledge Generation Agent 生成项目知识并保存到 Project Memory
```

### 迭代间数据传递
```
迭代 N-1 制品
    ↓
演化迭代创建时继承
    ↓
根据 InheritanceMode 决定复制内容
    ↓
迭代 N 从指定阶段开始
    ↓
注入 base iteration 的项目知识到迭代记忆
    ↓
生成新的制品覆盖或扩展原有制品
    ↓
生成新的项目知识并保存
```

### 知识生成流程
```
迭代完成（Delivery 阶段结束）
    ↓
Summary Agent 生成文档摘要
    ├─ 读取 idea.md、prd.md、design.md、plan.md
    ├─ 生成 300-500 字摘要
    └─ 保存到迭代目录
    ↓
Knowledge Generation Agent 生成项目知识
    ├─ 加载文档摘要
    ├─ 列出工作空间文件（智能截断）
    ├─ 加载 base iteration 知识（Evolution 场景）
    ├─ 分析技术栈、关键决策、架构模式
    └─ 生成项目知识
    ↓
保存到 Project Memory.iteration_knowledge
    ├─ Key: iteration_id
    └─ Value: IterationKnowledge 结构体
```

### 知识注入流程
```
Evolution 迭代启动
    ↓
读取 base iteration 的项目知识
    ↓
注入到当前迭代的 IterationMemory
    ↓
Plan Agent 可以访问完整的上下文
    ↓
开始执行 Plan 阶段
```

---

## 7. V2 架构的关键特性

### 项目知识管理系统

#### 知识生成流程（迭代完成后自动执行）
1. **生成文档摘要**（Summary Agent）：
   - 读取 idea.md、prd.md、design.md、plan.md
   - 生成 300-500 字的文档摘要
   - 保存摘要到迭代目录

2. **生成迭代知识**（Knowledge Generation Agent）：
   - 加载文档摘要
   - 列出工作空间文件（智能截断）
   - 加载 base iteration 的项目知识（Evolution 场景）
   - 分析技术栈、关键决策、架构模式
   - 生成迭代项目知识
   - 保存到 Project Memory.iteration_knowledge

3. **知识注入流程**（Evolution 迭代启动时）：
   - 读取 base iteration 的项目知识
   - 注入到当前迭代的 IterationMemory 中
   - Plan Agent 可以访问完整的上下文

#### 知识恢复机制
- CLI 命令：`cowork regenerate-knowledge <iteration_id>`
- GUI 命令：`gui_regenerate_knowledge(iteration_id)`
- 支持失败后手动重新生成
- 幂等操作：多次执行不会产生重复数据

#### 知识清理机制
- `ProjectMemory.cleanup_old_knowledge(keep_count: usize)`
- 保留最近 N 个迭代的知识
- 自动清理旧知识，避免数据膨胀

### 相比 V1 的改进
1. 统一的迭代架构：所有开发活动都在迭代单元内进行
2. 灵活的继承机制：支持三种继承模式，适应不同场景
3. 智能起始阶段判断：自动分析变更描述确定起始阶段
4. 完善的错误处理：重试机制、反馈循环、自愈机制
5. 结构化的存储：清晰的目录组织和制品管理
6. 人机协作优化：HITL 改在 Pipeline 层处理，避免与 Critic 冲突
7. Agent 系统稳定性：修复 LoopAgent 终止问题
8. 文件安全机制：所有文件操作限制在 workspace 内，路径验证相对于 workspace
9. Artifacts 验证：自动检查文件是否生成，未生成则自动重试
10. Delivery 同步：只在 Delivery 阶段将代码复制到项目根目录
11. HITL 工具从 Actor 移除：改在 Pipeline 层处理，避免与 Critic 判断冲突
12. 所有 Actor 配置必需的 Save 工具：确保能够保存文档
13. **项目知识管理**：自动生成迭代摘要和项目知识，支持 Evolution 迭代的知识注入
14. **智能上下文传递**：Evolution 迭代能够访问 base iteration 的完整上下文
15. **知识恢复机制**：支持失败后手动重新生成，确保系统鲁棒性
16. **并行迭代支持**：基于 iteration_id 的知识隔离，支持多个 Evolution 迭代并行进行
17. **跨平台兼容性**：处理 Windows UNC 路径前缀，支持长路径；兼容路径分隔符（/ 和 \）

### 技术栈约束
系统强制执行技术栈选择规则：
- Web：HTML/JS/CSS（Vanilla 优先）或 React
- Tool：Node.js
- Backend：Rust
- Desktop：Rust (Tauri)
- Mobile：原生 Android/iOS
- Runtime：bun 优先，npm 作为备选

### 项目知识管理技术实现
- **Summary Agent**：基于 adk-rust 的 LlmAgentBuilder
  - 使用 LoadDocumentSummaryTool 和 SaveSummaryTool
  - 生成 300-500 字的文档摘要

- **Knowledge Generation Agent**：基于 adk-rust 的 LlmAgentBuilder
  - 使用 LoadDocumentSummaryTool、ListWorkspaceFilesTool、LoadProjectKnowledgeTool、SaveProjectKnowledgeTool
  - 分析技术栈、关键决策、架构模式
  - 支持 Genesis 和 Evolution 场景

- **知识存储**：ProjectMemory.iteration_knowledge (HashMap)
  - Key: iteration_id
  - Value: IterationKnowledge 结构体
  - 支持并行迭代的知识隔离

- **知识注入**：inject_project_knowledge 方法
  - 从 base iteration 加载知识
  - 注入到 Evolution 迭代的记忆中
  - 确保 Plan Agent 有完整的上下文

---

## 8. 跨平台兼容性

### Windows 路径处理
系统在 Windows 环境下特殊处理 UNC 路径前缀（`\\?\`），确保路径比较和操作的准确性：

#### UNC 路径前缀问题
Windows 使用 `\\?\` 前缀支持超过 260 字符的路径，但这会导致 `strip_prefix()` 失败：
- 系统路径：`\\?\D:\Workspace\toys\project\index.html`
- 用户输入：`D:\Workspace\toys\project\index.html`
- 前缀不一致，导致路径比较失败

#### 解决方案
在文件工具和部署工具中实现了 `strip_unc_prefix()` 函数：

```rust
fn strip_unc_prefix(path: &Path) -> PathBuf {
    let path_str = path.display().to_string();
    if path_str.starts_with(r"\\?\") {
        PathBuf::from(&path_str[4..])
    } else {
        path.to_path_buf()
    }
}
```

#### 应用的工具
1. **file_tools.rs**：`list_files` 工具，确保返回正确的相对路径
2. **deployment_tools.rs**：`copy_workspace_to_project` 工具，在删除和复制文件前规范化路径
3. **knowledge_tools.rs**：知识生成工具的路径处理

#### 路径分隔符兼容
Windows 使用反斜杠（`\`）而 Unix 使用正斜杠（`/`），系统在路径检查时同时支持两种分隔符：

```rust
// Deployment 工具中的安全检查
if path_str.starts_with(".cowork-v2/") || path_str.starts_with(".cowork-v2\\") {
    protected_skipped.push(path_str.clone());
    continue;
}
```

#### 影响范围
- ✅ 文件列表操作：正确返回相对路径
- ✅ 文件删除操作：不会误删 `.cowork-v2` 目录中的文件
- ✅ 文件复制操作：正确映射 workspace 到项目根目录
- ✅ Artifacts 验证：准确识别生成的文件

### 其他平台考虑
- **Linux/macOS**：原生支持路径操作，无需特殊处理
- **权限管理**：Windows 需要处理文件锁定问题（如正在运行的 `cowork.exe` 无法删除）
- **路径长度限制**：通过 UNC 前缀自动处理长路径问题

---

## 总结

Cowork Forge 是一个设计精良的 AI 驱动开发系统，核心优势在于：

1. 专业化的 Agent 分工：每个阶段都有专门的角色处理
2. 质量保证循环：Actor-Critic 模式确保产出质量
3. 灵活的迭代机制：支持增量开发和渐进式演进
4. 完善的人机协作：关键决策点保留人工参与（HITL 在 Pipeline 层）
5. 结构化的数据管理：版本化的制品和记忆系统
6. 文件安全机制：所有文件操作限制在 workspace 内，路径验证相对于 workspace
7. Artifacts 验证：自动检查文件生成，未生成则自动重试
8. Delivery 同步：代码最终才部署到项目根目录
9. HITL 外层处理：用户审查在 Pipeline 层进行，避免与 Critic 冲突
10. 完整的工具配置：所有 Agent 都配置了必需的 Save 工具
11. 项目知识管理：自动生成迭代摘要和项目知识，支持 Evolution 迭代的知识注入
12. 智能上下文传递：Evolution 迭代能够访问 base iteration 的完整上下文
13. 知识恢复机制：支持失败后手动重新生成，确保系统鲁棒性
14. 并行迭代支持：基于 iteration_id 的知识隔离，支持多个 Evolution 迭代并行进行
15. Prompt 传递机制：通过 SimpleInvocationContext 和 SimpleSession 确保 LLM 接收到完整的上下文
16. 阶段特定提示：每个阶段都有明确的任务指导和工具使用说明
17. 跨平台兼容性：处理 Windows UNC 路径，支持长路径，兼容不同路径分隔符