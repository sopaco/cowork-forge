Cowork Forge 技术架构文档

  1. 核心功能

  Cowork Forge 是一个完整的 AI 驱动软件开发团队系统，通过模拟真实开发团队的角色分工，实现从创意到交付的全链路智能化开发。

  主要功能
   - 7 阶段开发工作流：Idea → PRD → Design → Plan → Coding → Check → Delivery
   - 专业化 AI Agents：8 个专业智能体（Idea、PRD Loop、Design Loop、Plan Loop、Coding Loop、Check、Delivery）
   - 迭代驱动架构：支持 Genesis（起源）和 Evolution（演化）两种迭代类型
   - 人机协作验证：关键决策点保留人工确认机制（HITL）
   - 增量代码更新：智能增量分析，只更新受影响文件

  Iteration 机制的作用
   - 将软件开发抽象为可管理、可继承、可演进的迭代单元
   - 每个迭代包含完整的开发生命周期和独立制品
   - 通过继承机制实现功能增量开发
   - 提供版本化的制品管理和上下文传递

  ---

  2. 架构流程

  V2 架构核心设计理念
  基于"迭代驱动"的核心理念，构建 AI 与人类协作的软件开发系统。系统通过模拟真实开发团队的角色分工和协作流程，将大型语言模型的能力结构化、流程化。

  架构概览
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

  Iteration 生命周期
   Draft (草稿) → Running (运行中) → Paused (已暂停) → Completed (已完成) / Failed (失败)

  Pipeline 执行流程
   1. 加载迭代和项目数据
   2. 准备工作空间（根据继承模式复制或初始化）
   3. 创建执行上下文
   4. 确定起始阶段（智能分析变更描述）
   5. 按顺序执行阶段序列
   6. 每个阶段完成后更新迭代状态
   7. 生成并保存制品
   8. 最终完成迭代

  Agent 协作机制
   - 顺序协作：Agent 按预定义顺序执行
   - Actor-Critic 循环协作：复杂阶段（PRD、Design、Plan、Coding）采用内部循环提升质量
   - 记忆共享协作：所有 Agent 通过共享的记忆系统获取上下文

  ---

  3. 模块设计

  Crate 职责划分
   cowork-forge/
   ├── cowork-cli/           # CLI 命令行接口
   ├── cowork-core/          # 核心业务逻辑
   │   ├── domain/           # 领域模型（Project、Iteration、Memory）
   │   ├── pipeline/         # 工作流编排
   │   ├── agents/           # Agent 构建器（基于 adk-rust）
   │   ├── instructions/     # Agent 指令模板
   │   ├── tools/            # 工具实现（文件、命令、数据、验证）
   │   ├── llm/              # LLM 集成（配置、速率限制）
   │   ├── data/             # 数据模型（Requirements、Features、Tasks）
   │   ├── persistence/      # 持久化
   │   ├── interaction/      # 交互抽象（CLI、GUI Backend）
   │   └── storage/          # 文件存储
   └── cowork-gui/           # GUI 桌面应用（基于 Tauri）

  关键模块实现

  Domain 模块
   - Project：项目根实体，包含迭代列表和元数据
   - Iteration：迭代实体，支持继承机制和生命周期管理
   - Memory：记忆系统，支持项目级和迭代级记忆

  Pipeline 模块
   - IterationExecutor：迭代执行器，管理完整生命周期
   - 支持重试机制（最多 3 次）、反馈循环（最多 5 轮）
   - 实现自愈机制（Check 失败时返回 Coding 阶段修复）
   - 支持暂停/继续和失败重试

  Agent 系统
  基于 adk-rust 框架构建：
   - 使用 LlmAgentBuilder 创建单一 Agent
   - 使用 LoopAgent 实现 Actor-Critic 循环
   - 关键修复：移除 exit_loop 工具，使用 max_iterations 避免 SequentialAgent 终止问题

  数据模型
   - Requirements：需求列表，包含优先级和验收标准
   - FeatureList：功能列表，关联需求和任务
   - DesignSpec：设计规范，包含架构和组件
   - ImplementationPlan：实施计划，包含里程碑和任务

  持久化机制
  基于文件系统的简单持久化：
   - 使用 .cowork-v2 目录组织数据
   - 结构化子目录：iterations/、memory/、workspace/

  ---

  4. 技术实现

  adk-rust 使用方式
   // 创建单一 Agent
   let agent = LlmAgentBuilder::new("idea_agent")
       .instruction(IDEA_AGENT_INSTRUCTION)
       .model(model)
       .tool(Arc::new(WriteFileTool))
       .include_contents(IncludeContents::None)
       .build()?;

   // 创建 LoopAgent（Actor-Critic）
   let loop_agent = LoopAgent::new(
       "prd_loop",
       vec![Arc::new(prd_actor), Arc::new(prd_critic)]
   ).with_max_iterations(1);

  工具系统设计

  核心原则：权限最小化和专用工具

  工具分类：
   1. Artifact 专用工具（非编码阶段）：
      - Save 工具：SaveIdeaTool、SavePrdDocTool、SaveDesignDocTool、SavePlanDocTool、SaveDeliveryReportTool
      - Load 工具：LoadIdeaTool、LoadPrdDocTool、LoadDesignDocTool、LoadPlanDocTool
   2. 文件操作工具（编码阶段专用）：
      - ReadFileTool、WriteFileTool、ListFilesTool
   3. 命令执行工具（编码/检查阶段）：
      - RunCommandTool（带超时和安全检查）
   4. 数据管理工具（所有阶段）：
      - CreateRequirementTool、CreateTaskTool、GetRequirementsTool、GetDesignTool、GetPlanTool 等
   5. 验证工具（检查阶段）：
      - CheckTestsTool、CheckLintTool、CheckFeatureCoverageTool、CheckTaskDependenciesTool 等
   6. 交互工具（人机协作）：
      - Content-based：ReviewWithFeedbackContentTool、ReviewAndEditContentTool（推荐）
      - Legacy：ReviewWithFeedbackTool、ReviewAndEditFileTool（废弃）

  工具实现最佳实践：

  ### 1. 安全参数提取
  所有工具使用安全的参数提取辅助函数，避免 unwrap() 导致的 panic：

  ```rust
  // ✅ 正确：使用安全参数提取
  pub struct SaveIdeaTool;

  #[async_trait]
  impl Tool for SaveIdeaTool {
      async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
          let content = get_required_string_param(&args, "content")?;  // 安全提取
          save_idea(content)?;
          Ok(json!({"status": "success"}))
      }
  }

  // ❌ 错误：使用 unwrap() 会导致 panic
  pub struct SaveIdeaToolBad;

  #[async_trait]
  impl Tool for SaveIdeaToolBad {
      async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {
          let content = args["content"].as_str().unwrap();  // 危险：可能 panic
          save_idea(content)?;
          Ok(json!({"status": "success"}))
      }
  }
  ```

  ### 2. 辅助函数
  提供三种参数提取函数：

  ```rust
  // 必需字符串参数 - 缺失时返回错误
  get_required_string_param(&args, "content")?
  get_required_array_param(&args, "options")?

  // 可选字符串参数 - 缺失时返回 None
  get_optional_string_param(&args, "description")

  // 可选数组参数 - 缺失时返回空数组
  get_optional_array_param(&args, "tags")
  ```

  ### 3. 错误处理
  参数缺失时返回清晰的错误信息：

  ```rust
  let title = args["title"].as_str()
      .ok_or_else(|| adk_core::AdkError::Tool("Missing required parameter: title".to_string()))?;

  // 如果参数缺失，agent 会收到：
  // Error: Missing required parameter: title
  // 而不是：
  // thread 'tokio-runtime-worker' panicked at called `Option::unwrap()` on a `None` value
  ```

  ### 4. 实际应用统计
  项目中共修复了 33 个危险的 unwrap() 调用：
  - hitl_content_tools.rs: 2个
  - validation_tools.rs: 1个
  - artifact_tools.rs: 5个
  - goto_stage_tool.rs: 2个
  - file_tools.rs: 3个
  - hitl_tools.rs: 4个
  - control_tools.rs: 5个
  - data_tools.rs: 11个

  结果：从 39 个潜在 panic 点减少到 6 个相对安全的点（在已验证的数组迭代中）。

  Agent 工作流程与工具体系

  Cowork Forge 采用 Actor-Critic 双智能体循环协作模式，每个阶段通过 Actor 生成内容、Critic 验证质量的循环确保输出质量。所有 Agent 遵循权限最小化原则，只获得完成任务所需的最小工具集。

  === 阶段 1: Idea（创意生成） ===

  Agent 类型: 单一 Agent（无 Critic）

  工作流程:
  1. 分析用户输入的创意描述和变更历史
  2. 生成结构化的 idea.md 文档，包含：
     - 项目背景和目标
     - 核心功能概述
     - 目标用户群体
     - 技术方向和约束
  3. 调用 review_and_edit_content() 让用户审查和编辑
  4. 根据用户反馈优化内容
  5. 调用 save_idea() 保存最终文档

  工具清单:
  - SaveIdeaTool: 保存 idea.md 到 artifacts 目录
  - ReviewAndEditContentTool: 人机协作工具（内容交互，不暴露文件路径）
  - (可选) QueryMemoryTool: 查询迭代记忆
  - (可选) SaveInsightTool: 保存洞见到记忆系统

  === 阶段 2: PRD（产品需求文档） ===

  Agent 类型: LoopAgent（Actor + Critic）

  工作流程:
  1. **PRD Actor**:
     - 调用 load_idea() 读取 idea.md 获取项目背景
     - 分析需求，创建结构化需求列表（Requirements）
     - 为每个需求添加功能特性（Features）
     - 生成 PRD 草稿（markdown 格式）
     - 调用 review_with_feedback_content() 让用户审查
     - 调用 save_prd_doc() 保存最终文档
  2. **PRD Critic**:
     - 调用 get_requirements() 读取结构化需求
     - 调用 load_idea() 验证与创意文档的一致性
     - 检查需求完整性、优先级合理性、验收标准明确性
     - 提供反馈或通过验证

  PRD Actor 工具清单:
  - LoadIdeaTool: 读取 idea.md 文档
  - ReviewWithFeedbackContentTool: 人机协作审查
  - CreateRequirementTool: 创建需求条目
  - AddFeatureTool: 添加功能特性
  - GetRequirementsTool: 读取需求列表
  - SavePrdDocTool: 保存 prd.md 文档

  PRD Critic 工具清单:
  - GetRequirementsTool: 读取结构化需求
  - LoadIdeaTool: 读取 idea.md 验证一致性
  - ProvideFeedbackTool: 提供反馈

  === 阶段 3: Design（系统设计） ===

  Agent 类型: LoopAgent（Actor + Critic）

  工作流程:
  1. **Design Actor**:
     - 调用 get_requirements() 读取需求列表
     - 调用 load_prd_doc() 读取 PRD 文档
     - 设计系统架构（技术栈选择、模块划分、接口设计）
     - 创建设计组件（DesignComponents）
     - 生成 Design 文档（markdown 格式）
     - 调用 review_with_feedback_content() 让用户审查
     - 调用 save_design_doc() 保存最终文档
  2. **Design Critic**:
     - 调用 get_requirements() 读取需求
     - 调用 get_design() 读取设计规范
     - 调用 load_design_doc() 验证 markdown 文档
     - 检查功能覆盖度、架构合理性、接口完整性
     - 提供反馈或通过验证

  Design Actor 工具清单:
  - GetRequirementsTool: 读取需求列表
  - GetDesignTool: 读取设计规范
  - LoadPrdDocTool: 读取 PRD 文档
  - ReviewWithFeedbackContentTool: 人机协作审查
  - CreateDesignComponentTool: 创建设计组件
  - SaveDesignDocTool: 保存 design.md 文档

  Design Critic 工具清单:
  - GetRequirementsTool: 读取需求列表
  - GetDesignTool: 读取设计规范
  - LoadDesignDocTool: 读取 design.md 验证
  - CheckFeatureCoverageTool: 检查功能覆盖度
  - ProvideFeedbackTool: 提供反馈

  === 阶段 4: Plan（实施计划） ===

  Agent 类型: LoopAgent（Actor + Critic）

  工作流程:
  1. **Plan Actor**:
     - 调用 get_requirements() 读取需求
     - 调用 get_design() 读取设计规范
     - 调用 load_prd_doc() 读取 PRD 文档
     - 调用 load_design_doc() 读取 Design 文档
     - 分解实施任务（Tasks），设置优先级和依赖关系
     - 制定里程碑（Milestones）
     - 生成 Plan 文档（markdown 格式）
     - 调用 review_with_feedback_content() 让用户审查
     - 调用 save_plan_doc() 保存最终文档
  2. **Plan Critic**:
     - 调用 get_plan() 读取实施计划
     - 调用 get_requirements() 读取需求
     - 调用 load_plan_doc() 验证 markdown 文档
     - 检查任务依赖关系、实施可行性、时间估算
     - 提供反馈或通过验证

  Plan Actor 工具清单:
  - GetRequirementsTool: 读取需求列表
  - GetDesignTool: 读取设计规范
  - GetPlanTool: 读取实施计划
  - LoadPrdDocTool: 读取 PRD 文档
  - LoadDesignDocTool: 读取 Design 文档
  - ReviewWithFeedbackContentTool: 人机协作审查
  - CreateTaskTool: 创建任务
  - SavePlanDocTool: 保存 plan.md 文档

  Plan Critic 工具清单:
  - GetPlanTool: 读取实施计划
  - GetRequirementsTool: 读取需求列表
  - LoadPlanDocTool: 读取 plan.md 验证
  - CheckTaskDependenciesTool: 检查任务依赖
  - ProvideFeedbackTool: 提供反馈

  === 阶段 5: Coding（代码实现） ===

  Agent 类型: LoopAgent（Actor + Critic，最多 5 轮迭代）

  工作流程:
  1. **Coding Actor**:
     - 调用 get_plan() 读取任务列表（结构化数据，非 LoadPlanDoc）
     - 选择下一个待执行任务
     - 读取现有代码文件（ReadFile）
     - 生成/更新代码文件（WriteFile）
     - 列出工作空间文件（ListFiles）
     - 运行测试验证（RunCommand + CheckTests）
     - 更新任务状态（UpdateTaskStatus）
     - 更新功能状态（UpdateFeatureStatus）
  2. **Coding Critic**:
     - 调用 get_plan() 读取任务列表
     - 读取代码文件验证实现
     - 运行测试检查功能正确性
     - 提供代码审查反馈

  Coding Actor 工具清单:
  - GetPlanTool: 读取任务列表（结构化数据）
  - UpdateTaskStatusTool: 更新任务状态
  - UpdateFeatureStatusTool: 更新功能状态
  - ReadFileTool: 读取代码文件
  - WriteFileTool: 写入代码文件
  - ListFilesTool: 列出工作空间文件
  - RunCommandTool: 运行测试和构建命令
  - CheckTestsTool: 检查测试通过率

  Coding Critic 工具清单:
  - GetPlanTool: 读取任务列表
  - ReadFileTool: 读取代码文件
  - ListFilesTool: 列出工作空间文件
  - RunCommandTool: 运行测试验证
  - ProvideFeedbackTool: 提供反馈

  === 阶段 6: Check（质量检查） ===

  Agent 类型: 单一 Agent

  工作流程:
  1. 调用 get_requirements() 读取需求列表
  2. 调用 get_design() 读取设计规范
  3. 调用 get_plan() 读取任务列表
  4. 检查数据格式完整性（CheckDataFormat）
  5. 检查功能覆盖度（CheckFeatureCoverage）
  6. 检查任务依赖关系（CheckTaskDependencies）
  7. 读取代码文件和列出文件
  8. 运行测试（CheckTests）
  9. 运行代码检查（CheckLint）
  10. 综合评估：
      - 如果全部通过，进入 Delivery 阶段
      - 如果发现问题，调用 goto_stage() 返回 Coding 阶段修复

  Check Agent 工具清单:
  - GetRequirementsTool: 读取需求列表
  - GetDesignTool: 读取设计规范
  - GetPlanTool: 读取任务列表
  - CheckDataFormatTool: 检查数据格式
  - CheckFeatureCoverageTool: 检查功能覆盖度
  - CheckTaskDependenciesTool: 检查任务依赖
  - RunCommandTool: 运行测试和构建
  - ReadFileTool: 读取代码文件
  - ListFilesTool: 列出工作空间文件
  - CheckTestsTool: 检查测试通过率
  - CheckLintTool: 运行代码检查
  - ProvideFeedbackTool: 提供反馈
  - GotoStageTool: 返回之前阶段

  === 阶段 7: Delivery（交付报告） ===

  Agent 类型: 单一 Agent

  工作流程:
  1. 调用 get_requirements() 读取需求列表
  2. 调用 get_design() 读取设计规范
  3. 调用 get_plan() 读取任务列表
  4. 调用 load_feedback_history() 读取反馈历史
  5. 调用 load_idea() 读取 idea.md
  6. 调用 load_prd_doc() 读取 prd.md
  7. 调用 load_design_doc() 读取 design.md
  8. 列出项目文件验证交付物
  9. 生成交付报告（delivery.md），包含：
     - 功能实现清单
     - 测试覆盖率
     - 已知问题和限制
     - 使用说明
  10. 调用 save_delivery_report() 保存报告

  Delivery Agent 工具清单:
  - GetRequirementsTool: 读取需求列表
  - GetDesignTool: 读取设计规范
  - GetPlanTool: 读取任务列表
  - LoadFeedbackHistoryTool: 读取反馈历史
  - ListFilesTool: 列出项目文件
  - LoadIdeaTool: 读取 idea.md
  - LoadPrdDocTool: 读取 prd.md
  - LoadDesignDocTool: 读取 design.md
  - SaveDeliveryReportTool: 保存 delivery.md

  工具权限矩阵：

  | 阶段 | ReadFile | WriteFile | LoadIdea | LoadPrd | LoadDesign | LoadPlan | SaveXxx | ListFiles | RunCommand | HITL | GetRequirements | GetDesign | GetPlan | CreateXxx | UpdateXxx | CheckXxx |
  |------|----------|-----------|----------|---------|------------|---------|---------|-----------|------------|------|-----------------|-------------|----------|-----------|-----------|----------|
  | Idea | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ save_idea | ❌ | ❌ | ✅ review_and_edit_content | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
  | PRD Actor | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ save_prd_doc | ❌ | ❌ | ✅ review_with_feedback_content | ✅ | ❌ | ❌ | ✅ create_requirement/add_feature | ❌ | ❌ |
  | PRD Critic | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
  | Design Actor | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ save_design_doc | ❌ | ❌ | ✅ review_with_feedback_content | ✅ | ✅ | ❌ | ✅ create_design_component | ❌ | ❌ |
  | Design Critic | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ check_feature_coverage |
  | Plan Actor | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ✅ save_plan_doc | ❌ | ❌ | ✅ review_with_feedback_content | ✅ | ✅ | ✅ | ✅ create_task | ❌ | ❌ |
  | Plan Critic | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ check_task_dependencies |
  | Coding Actor | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ update_task/update_feature | ✅ check_tests |
  | Coding Critic | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ |
  | Check Agent | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ check_data_format/check_feature_coverage/check_task_dependencies/check_tests/check_lint |
  | Delivery Agent | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | ✅ save_delivery_report | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |

  安全特性：
   - 权限最小化：非编码阶段不提供通用的文件读写权限
   - 路径限制：Artifact 工具只能操作 artifacts 目录下的文件
   - 路径验证：拒绝绝对路径和 .. 访问
   - 命令白名单：阻止危险命令（rm -rf、sudo 等）
   - 超时控制：命令执行 30 秒超时
   - 阻塞检测：自动拒绝长时间运行的命令
   - Content-based HITL：不暴露文件路径，使用内容交互
   - 参数验证：所有工具使用安全的参数提取函数，避免 panic
   - 错误处理：缺失参数时返回清晰的错误信息而非崩溃

  ---

  5. 目录结构

  V2 架构目录组织
   .cowork-v2/
   ├── iterations/              # 迭代数据
   │   └── {iteration_id}/      # 每个迭代一个目录
   │       ├── iteration.json   # 迭代元数据
   │       ├── artifacts/       # 迭代制品
   │       │   ├── idea.md
   │       │   ├── prd.md
   │       │   ├── design.md
   │       │   ├── plan.md
   │       │   ├── coding/      # 代码工作空间
   │       │   └── delivery.md
   │       └── workspace/       # 工作空间
   ├── memory/                  # 记忆系统
   │   ├── project/             # 项目级记忆
   │   └── iterations/          # 迭代级记忆
   └── workspace/               # 共享工作空间

  InheritanceMode 实现
   pub enum InheritanceMode {
       None,     // Genesis only - fresh start
       Full,     // Copy all workspace files from base
       Partial,  // Copy only artifacts and config, regenerate code
   }

  ---

  6. 数据流说明

  开发流程数据流
   用户输入创意
       ↓
   Idea Agent 生成 idea.md
       ↓
   PRD Loop Agent 生成 requirements.json 和 prd.md
       ↓
   Design Loop Agent 生成 design_spec.json 和 design.md
       ↓
   Plan Loop Agent 生成 implementation_plan.json 和 plan.md
       ↓
   Coding Loop Agent 生成代码文件
       ↓
   Check Agent 验证质量和完整性
       ↓
   Delivery Agent 生成 delivery.md

  迭代间数据传递
   迭代 N-1 制品
       ↓
   演化迭代创建时继承
       ↓
   根据 InheritanceMode 决定复制内容
       ↓
   迭代 N 从指定阶段开始
       ↓
   生成新的制品覆盖或扩展原有制品

  ---

  7. V2 架构的关键特性

  相比 V1 的改进
   1. 统一的迭代架构：所有开发活动都在迭代单元内进行
   2. 灵活的继承机制：支持三种继承模式，适应不同场景
   3. 智能起始阶段判断：自动分析变更描述确定起始阶段
   4. 完善的错误处理：重试机制、反馈循环、自愈机制
   5. 结构化的存储：清晰的目录组织和制品管理
   6. 人机协作优化：更好的 HITL 工具和交互体验
   7. Agent 系统稳定性：修复 LoopAgent 终止问题

  技术栈约束
  系统强制执行技术栈选择规则：
   - Web：HTML/JS/CSS（Vanilla 优先）或 React
   - Tool：Node.js
   - Backend：Rust
   - Desktop：Rust (Tauri)
   - Mobile：原生 Android/iOS
   - Runtime：bun 优先，npm 作为备选

  ---

  总结

  Cowork Forge 是一个设计精良的 AI 驱动开发系统，核心优势在于：

   1. 专业化的 Agent 分工：每个阶段都有专门的角色处理
   2. 质量保证循环：Actor-Critic 模式确保产出质量
   3. 灵活的迭代机制：支持增量开发和渐进式演进
   4. 完善的人机协作：关键决策点保留人工参与
   5. 结构化的数据管理：版本化的制品和记忆系统
