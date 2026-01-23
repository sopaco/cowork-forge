# Project Analysis Summary Report (Full Version)

Generation Time: 2026-01-21 10:14:33 UTC

## Execution Timing Statistics

- **Total Execution Time**: 1810.99 seconds
- **Preprocessing Phase**: 475.71 seconds (26.3%)
- **Research Phase**: 483.71 seconds (26.7%)
- **Document Generation Phase**: 851.57 seconds (47.0%)
- **Output Phase**: 0.00 seconds (0.0%)
- **Summary Generation Time**: 0.001 seconds

## Cache Performance Statistics and Savings

### Performance Metrics
- **Cache Hit Rate**: 54.9%
- **Total Operations**: 82
- **Cache Hits**: 45 times
- **Cache Misses**: 37 times
- **Cache Writes**: 38 times

### Savings
- **Inference Time Saved**: 222.7 seconds
- **Tokens Saved**: 72232 input + 26758 output = 98990 total
- **Estimated Cost Savings**: $0.0460
- **Performance Improvement**: 54.9%
- **Efficiency Improvement Ratio**: 0.1x (saved time / actual execution time)

## Core Research Data Summary

Complete content of four types of research materials according to Prompt template data integration rules:

### System Context Research Report
Provides core objectives, user roles, and system boundary information for the project.

```json
{
  "business_value": "通过AI驱动的自动化开发流程，显著提升软件开发效率和质量，减少人工编码工作量，支持迭代式开发和需求变更管理，为开发团队提供智能化的开发助手工具。",
  "confidence_score": 0.95,
  "external_systems": [
    {
      "description": "提供大语言模型能力，用于智能体决策、代码生成和文档生成",
      "interaction_type": "API调用",
      "name": "OpenAI兼容的LLM服务"
    },
    {
      "description": "本地文件存储系统，用于项目文件管理、代码存储和会话持久化",
      "interaction_type": "读写操作",
      "name": "文件系统"
    },
    {
      "description": "支持Rust、Python、JavaScript等语言的编译和语法检查工具",
      "interaction_type": "命令行执行",
      "name": "编译器/解释器"
    }
  ],
  "project_description": "这是一个基于人工智能的多智能体协同软件开发系统，通过命令行界面提供完整的软件开发生命周期管理。系统实现了从创意输入到最终交付的8阶段工作流程，整合了多个专业智能体（PRD生成、设计规划、代码执行、检查验证等）和人工介入环节，支持多语言代码生成和验证。",
  "project_name": "Cowork AI 多智能体软件开发系统",
  "project_type": "CLITool",
  "system_boundary": {
    "excluded_components": [
      "图形用户界面(GUI)",
      "版本控制系统集成",
      "持续集成/持续部署流水线",
      "云端部署和运维管理",
      "团队协作功能",
      "第三方API集成(除LLM服务外)"
    ],
    "included_components": [
      "命令行界面(CLI)和用户交互",
      "工作流程编排器(Orchestrator)",
      "多智能体系统(PRD、设计、规划、代码执行、检查等)",
      "人工介入(HITL)控制器",
      "工件存储和会话管理",
      "文件操作工具集",
      "配置管理和模型服务集成"
    ],
    "scope": "AI驱动的多智能体软件开发命令行工具"
  },
  "target_users": [
    {
      "description": "需要快速原型开发或自动化代码生成的程序员，熟悉命令行工具和软件开发流程",
      "name": "软件开发工程师",
      "needs": [
        "快速将创意转化为可执行代码",
        "自动化处理重复性编码任务",
        "代码质量验证和规范检查",
        "迭代式开发和需求变更管理"
      ]
    },
    {
      "description": "负责项目需求管理和开发流程协调的技术管理人员",
      "name": "技术项目经理",
      "needs": [
        "结构化需求文档生成",
        "开发进度跟踪和验证",
        "技术方案设计和评审",
        "交付物管理和验收"
      ]
    },
    {
      "description": "专注于AI技术应用的开发人员，需要智能代码生成工具",
      "name": "AI应用开发者",
      "needs": [
        "多智能体协同工作流程",
        "LLM集成和提示工程",
        "文件系统操作和代码管理",
        "会话状态持久化和恢复"
      ]
    }
  ]
}
```

### Domain Modules Research Report
Provides high-level domain division, module relationships, and core business process information.

```json
{
  "architecture_summary": "Cowork AI 多智能体软件开发系统采用分层架构设计，包含核心业务域、基础设施域和工具支持域。系统以工作流编排器为核心，协调多个智能体完成从创意输入到代码交付的完整开发流程。架构采用模块化设计，各域之间通过清晰的接口进行交互，支持会话状态的持久化和恢复机制。",
  "business_flows": [
    {
      "description": "完整的AI驱动软件开发流程，从用户创意输入开始，经过需求分析、技术设计、实现规划、代码生成、质量检查、反馈处理，最终生成交付报告。该流程支持多轮迭代和人工介入。",
      "entry_point": "用户通过CLI输入创意或恢复现有会话",
      "importance": 10.0,
      "involved_domains_count": 5,
      "name": "AI驱动软件开发主流程",
      "steps": [
        {
          "code_entry_point": "crates/cowork-core/src/agents/idea_intake.rs",
          "domain_module": "智能体执行域",
          "operation": "创意输入处理，将用户非结构化输入转化为结构化IdeaSpec",
          "step": 1,
          "sub_module": "创意输入智能体"
        },
        {
          "code_entry_point": "crates/cowork-core/src/agents/prd_agent.rs",
          "domain_module": "智能体执行域",
          "operation": "产品需求文档生成，基于创意规范生成结构化PRD",
          "step": 2,
          "sub_module": "PRD生成智能体"
        },
        {
          "code_entry_point": "crates/cowork-core/src/agents/design_agent.rs",
          "domain_module": "智能体执行域",
          "operation": "技术设计生成，基于PRD生成系统架构和技术方案",
          "step": 3,
          "sub_module": "设计智能体"
        },
        {
          "code_entry_point": "crates/cowork-core/src/agents/plan_agent.rs",
          "domain_module": "智能体执行域",
          "operation": "实施计划制定，将设计转化为具体的代码实现计划",
          "step": 4,
          "sub_module": "计划智能体"
        },
        {
          "code_entry_point": "crates/cowork-core/src/agents/code_executor.rs",
          "domain_module": "智能体执行域",
          "operation": "代码生成和执行，基于计划生成和修改代码文件",
          "step": 5,
          "sub_module": "代码执行智能体"
        },
        {
          "code_entry_point": "crates/cowork-core/src/agents/check_agent.rs",
          "domain_module": "智能体执行域",
          "operation": "代码质量检查，验证代码符合要求和质量标准",
          "step": 6,
          "sub_module": "检查智能体"
        },
        {
          "code_entry_point": "crates/cowork-core/src/agents/feedback_agent.rs",
          "domain_module": "智能体执行域",
          "operation": "反馈处理，分析用户反馈并确定需要重做的阶段",
          "step": 7,
          "sub_module": "反馈智能体"
        },
        {
          "code_entry_point": "crates/cowork-core/src/agents/delivery_agent.rs",
          "domain_module": "智能体执行域",
          "operation": "交付报告生成，汇总所有阶段成果生成最终交付物",
          "step": 8,
          "sub_module": "交付智能体"
        }
      ]
    },
    {
      "description": "会话管理和状态恢复流程，支持用户中断后从任意阶段恢复开发工作，确保开发过程的连续性和可恢复性。",
      "entry_point": "用户选择恢复现有会话",
      "importance": 8.0,
      "involved_domains_count": 3,
      "name": "会话恢复流程",
      "steps": [
        {
          "code_entry_point": "crates/cowork-core/src/orchestrator/mod.rs",
          "domain_module": "工作流编排域",
          "operation": "会话状态加载，从持久化存储中恢复会话上下文",
          "step": 1,
          "sub_module": "编排器"
        },
        {
          "code_entry_point": "crates/cowork-core/src/memory/mod.rs",
          "domain_module": "数据持久化域",
          "operation": "工件数据读取，加载特定会话的所有阶段工件",
          "step": 2,
          "sub_module": "工件存储"
        },
        {
          "code_entry_point": "crates/cowork-core/src/orchestrator/mod.rs",
          "domain_module": "工作流编排域",
          "operation": "流程重定向，从指定阶段继续执行开发流程",
          "step": 3,
          "sub_module": "编排器"
        }
      ]
    },
    {
      "description": "代码变更和更新管理流程，支持需求变更时的增量代码更新，保护用户自定义修改。",
      "entry_point": "检测到PRD版本变更",
      "importance": 7.0,
      "involved_domains_count": 3,
      "name": "增量代码更新流程",
      "steps": [
        {
          "code_entry_point": "crates/cowork-core/src/agents/code_updater.rs",
          "domain_module": "智能体执行域",
          "operation": "需求差异分析，比较新旧PRD识别变更需求",
          "step": 1,
          "sub_module": "代码更新智能体"
        },
        {
          "code_entry_point": "crates/cowork-core/src/agents/code_updater.rs",
          "domain_module": "智能体执行域",
          "operation": "影响文件分析，基于需求映射识别需要修改的文件",
          "step": 2,
          "sub_module": "代码更新智能体"
        },
        {
          "code_entry_point": "crates/cowork-core/src/agents/code_updater.rs",
          "domain_module": "智能体执行域",
          "operation": "更新计划生成，制定保护用户修改的合并策略",
          "step": 3,
          "sub_module": "代码更新智能体"
        }
      ]
    }
  ],
  "confidence_score": 9.2,
  "domain_modules": [
    {
      "code_paths": [
        "crates/cowork-core/src/orchestrator/mod.rs",
        "crates/cowork-core/src/workflow.rs"
      ],
      "complexity": 9.0,
      "description": "负责协调整个软件开发流程的核心业务逻辑，管理8阶段工作流的执行顺序、状态转换和异常处理。",
      "domain_type": "核心业务域",
      "importance": 10.0,
      "name": "工作流编排域",
      "sub_modules": [
        {
          "code_paths": [
            "crates/cowork-core/src/orchestrator/mod.rs"
          ],
          "description": "主要工作流编排引擎，负责阶段调度、会话管理和流程控制",
          "importance": 10.0,
          "key_functions": [
            "流程阶段管理",
            "会话状态控制",
            "智能体调用协调",
            "错误处理和重试"
          ],
          "name": "编排器核心"
        },
        {
          "code_paths": [
            "crates/cowork-core/src/workflow.rs"
          ],
          "description": "工作流业务逻辑实现（待完善），处理核心业务流程",
          "importance": 6.0,
          "key_functions": [
            "业务流程执行",
            "状态转换处理"
          ],
          "name": "业务流程模块"
        }
      ]
    },
    {
      "code_paths": [
        "crates/cowork-core/src/agents/mod.rs",
        "crates/cowork-core/src/agents/idea_intake.rs",
        "crates/cowork-core/src/agents/prd_agent.rs",
        "crates/cowork-core/src/agents/design_agent.rs",
        "crates/cowork-core/src/agents/plan_agent.rs",
        "crates/cowork-core/src/agents/code_executor.rs",
        "crates/cowork-core/src/agents/check_agent.rs",
        "crates/cowork-core/src/agents/feedback_agent.rs",
        "crates/cowork-core/src/agents/delivery_agent.rs",
        "crates/cowork-core/src/agents/code_updater.rs",
        "crates/cowork-core/src/agents/code_planner.rs",
        "crates/cowork-core/src/agents/todo_manager.rs",
        "crates/cowork-core/src/agents/watchdog.rs",
        "crates/cowork-core/src/agents/error_analyzer.rs",
        "crates/cowork-core/src/agents/code_plan_normalizer.rs",
        "crates/cowork-core/src/agents/batch_context.rs"
      ],
      "complexity": 9.5,
      "description": "包含所有专业智能体的执行逻辑，每个智能体负责特定的开发阶段任务，通过LLM实现智能化决策和代码生成。",
      "domain_type": "核心业务域",
      "importance": 9.5,
      "name": "智能体执行域",
      "sub_modules": [
        {
          "code_paths": [
            "crates/cowork-core/src/agents/mod.rs"
          ],
          "description": "智能体统一接口和管理模块，提供标准的智能体执行框架",
          "importance": 8.0,
          "key_functions": [
            "智能体注册",
            "执行上下文管理",
            "结果格式标准化"
          ],
          "name": "智能体框架"
        },
        {
          "code_paths": [
            "crates/cowork-core/src/agents/idea_intake.rs",
            "crates/cowork-core/src/agents/prd_agent.rs",
            "crates/cowork-core/src/agents/design_agent.rs",
            "crates/cowork-core/src/agents/plan_agent.rs"
          ],
          "description": "需求分析和设计阶段智能体，处理创意到技术方案的转化",
          "importance": 9.0,
          "key_functions": [
            "需求结构化",
            "技术设计生成",
            "实施计划制定"
          ],
          "name": "需求设计智能体组"
        },
        {
          "code_paths": [
            "crates/cowork-core/src/agents/code_executor.rs",
            "crates/cowork-core/src/agents/code_planner.rs",
            "crates/cowork-core/src/agents/code_updater.rs"
          ],
          "description": "代码生成和执行智能体，负责实际的代码修改和文件操作",
          "importance": 9.0,
          "key_functions": [
            "代码生成",
            "文件修改",
            "批量处理",
            "增量更新"
          ],
          "name": "代码执行智能体组"
        },
        {
          "code_paths": [
            "crates/cowork-core/src/agents/check_agent.rs",
            "crates/cowork-core/src/agents/error_analyzer.rs"
          ],
          "description": "质量检查和验证智能体，确保代码质量和符合要求",
          "importance": 8.0,
          "key_functions": [
            "代码验证",
            "错误分析",
            "质量评估"
          ],
          "name": "质量检查智能体组"
        },
        {
          "code_paths": [
            "crates/cowork-core/src/agents/feedback_agent.rs",
            "crates/cowork-core/src/agents/delivery_agent.rs"
          ],
          "description": "反馈处理和交付阶段智能体，管理迭代和改进",
          "importance": 7.0,
          "key_functions": [
            "反馈分析",
            "迭代规划",
            "交付报告生成"
          ],
          "name": "反馈交付智能体组"
        },
        {
          "code_paths": [
            "crates/cowork-core/src/agents/todo_manager.rs",
            "crates/cowork-core/src/agents/watchdog.rs",
            "crates/cowork-core/src/agents/batch_context.rs",
            "crates/cowork-core/src/agents/code_plan_normalizer.rs"
          ],
          "description": "辅助工具智能体，提供任务管理、监控和数据处理支持",
          "importance": 6.0,
          "key_functions": [
            "任务状态跟踪",
            "执行监控",
            "数据标准化",
            "上下文管理"
          ],
          "name": "辅助工具智能体组"
        }
      ]
    },
    {
      "code_paths": [
        "crates/cowork-core/src/artifacts/mod.rs",
        "crates/cowork-core/src/memory/mod.rs",
        "crates/cowork-core/src/data.rs"
      ],
      "complexity": 7.0,
      "description": "管理软件开发过程中产生的所有结构化数据，包括工件定义、存储和检索机制。",
      "domain_type": "核心业务域",
      "importance": 8.0,
      "name": "数据模型域",
      "sub_modules": [
        {
          "code_paths": [
            "crates/cowork-core/src/artifacts/mod.rs"
          ],
          "description": "完整的软件开发工件数据模型，定义8个开发阶段的数据结构",
          "importance": 9.0,
          "key_functions": [
            "工件类型定义",
            "元数据管理",
            "版本控制",
            "关系追踪"
          ],
          "name": "工件数据模型"
        },
        {
          "code_paths": [
            "crates/cowork-core/src/memory/mod.rs"
          ],
          "description": "工件持久化存储系统，支持会话级别的数据管理和检索",
          "importance": 8.0,
          "key_functions": [
            "会话管理",
            "文件存储",
            "JSON序列化",
            "Markdown文档生成"
          ],
          "name": "工件存储系统"
        },
        {
          "code_paths": [
            "crates/cowork-core/src/data.rs"
          ],
          "description": "嵌入式数据层（待实现），计划支持样本数据管理",
          "importance": 3.0,
          "key_functions": [
            "数据访问",
            "样本管理"
          ],
          "name": "数据访问层"
        }
      ]
    },
    {
      "code_paths": [
        "crates/cowork-core/src/hitl/mod.rs"
      ],
      "complexity": 6.0,
      "description": "提供人工介入功能，在关键决策点允许用户审核、确认和提供反馈，确保AI决策符合用户期望。",
      "domain_type": "核心业务域",
      "importance": 7.0,
      "name": "人工介入域",
      "sub_modules": [
        {
          "code_paths": [
            "crates/cowork-core/src/hitl/mod.rs"
          ],
          "description": "人工交互控制器，提供多种交互模式支持用户参与",
          "importance": 7.0,
          "key_functions": [
            "用户输入收集",
            "内容审核确认",
            "JSON编辑验证",
            "菜单选择"
          ],
          "name": "HITL控制器"
        }
      ]
    },
    {
      "code_paths": [
        "crates/cowork-cli/src/main.rs",
        "crates/cowork-cli/src/server.rs"
      ],
      "complexity": 5.0,
      "description": "命令行界面实现，提供用户交互入口和命令执行框架。",
      "domain_type": "基础设施域",
      "importance": 6.0,
      "name": "用户界面域",
      "sub_modules": [
        {
          "code_paths": [
            "crates/cowork-cli/src/main.rs"
          ],
          "description": "主CLI入口点，定义命令结构和执行路由",
          "importance": 6.0,
          "key_functions": [
            "命令解析",
            "日志初始化",
            "配置加载",
            "执行路由"
          ],
          "name": "CLI主程序"
        },
        {
          "code_paths": [
            "crates/cowork-cli/src/server.rs"
          ],
          "description": "服务器功能（待实现），计划扩展服务端能力",
          "importance": 2.0,
          "key_functions": [
            "服务端功能"
          ],
          "name": "服务器模块"
        }
      ]
    },
    {
      "code_paths": [
        "crates/cowork-core/src/config.rs"
      ],
      "complexity": 4.0,
      "description": "系统配置管理，集中处理LLM服务配置和运行时参数。",
      "domain_type": "基础设施域",
      "importance": 7.0,
      "name": "配置管理域",
      "sub_modules": [
        {
          "code_paths": [
            "crates/cowork-core/src/config.rs"
          ],
          "description": "配置数据结构和加载逻辑，支持文件和环境变量配置",
          "importance": 7.0,
          "key_functions": [
            "配置模型定义",
            "TOML文件解析",
            "环境变量读取",
            "类型安全访问"
          ],
          "name": "配置管理器"
        }
      ]
    },
    {
      "code_paths": [
        "crates/cowork-core/src/tools/file_tools.rs",
        "crates/cowork-core/src/tools/mod.rs"
      ],
      "complexity": 6.0,
      "description": "提供文件系统操作工具集，支持基本的文件读写和高级的增量编辑功能。",
      "domain_type": "工具支持域",
      "importance": 7.0,
      "name": "文件工具域",
      "sub_modules": [
        {
          "code_paths": [
            "crates/cowork-core/src/tools/file_tools.rs"
          ],
          "description": "完整的文件操作工具实现，支持异步文件处理",
          "importance": 7.0,
          "key_functions": [
            "文件读写",
            "目录操作",
            "行级编辑",
            "参数验证"
          ],
          "name": "文件操作工具"
        },
        {
          "code_paths": [
            "crates/cowork-core/src/tools/mod.rs"
          ],
          "description": "工具模块入口，提供统一的工具访问接口",
          "importance": 5.0,
          "key_functions": [
            "模块导出",
            "接口统一"
          ],
          "name": "工具模块管理"
        }
      ]
    },
    {
      "code_paths": [
        "crates/cowork-core/src/utils/prd_utils.rs",
        "crates/cowork-core/src/utils/mod.rs"
      ],
      "complexity": 3.0,
      "description": "提供通用的工具函数和辅助功能，支持其他模块的业务逻辑。",
      "domain_type": "工具支持域",
      "importance": 5.0,
      "name": "工具函数域",
      "sub_modules": [
        {
          "code_paths": [
            "crates/cowork-core/src/utils/prd_utils.rs"
          ],
          "description": "PRD摘要提取工具，为监控系统提供结构化摘要",
          "importance": 5.0,
          "key_functions": [
            "PRD摘要生成",
            "文本格式化"
          ],
          "name": "PRD工具函数"
        },
        {
          "code_paths": [
            "crates/cowork-core/src/utils/mod.rs"
          ],
          "description": "工具函数模块管理，提供统一的工具访问点",
          "importance": 4.0,
          "key_functions": [
            "函数导出",
            "模块组织"
          ],
          "name": "工具函数管理"
        }
      ]
    },
    {
      "code_paths": [
        "crates/cowork-core/src/lib.rs"
      ],
      "complexity": 4.0,
      "description": "核心库的公共API定义和模块组织，提供统一的对外接口。",
      "domain_type": "基础设施域",
      "importance": 6.0,
      "name": "库接口域",
      "sub_modules": [
        {
          "code_paths": [
            "crates/cowork-core/src/lib.rs"
          ],
          "description": "主要的库接口定义，整合所有子模块的公共API",
          "importance": 6.0,
          "key_functions": [
            "模块重导出",
            "API统一",
            "依赖管理"
          ],
          "name": "核心库接口"
        }
      ]
    }
  ],
  "domain_relations": [
    {
      "description": "编排器依赖智能体执行具体的开发阶段任务，通过标准接口调用各智能体",
      "from_domain": "工作流编排域",
      "relation_type": "服务调用",
      "strength": 9.0,
      "to_domain": "智能体执行域"
    },
    {
      "description": "智能体在执行过程中需要读写工件数据，依赖数据模型域提供结构化数据支持",
      "from_domain": "智能体执行域",
      "relation_type": "数据依赖",
      "strength": 8.0,
      "to_domain": "数据模型域"
    },
    {
      "description": "编排器通过人工介入域在关键决策点获取用户确认和反馈",
      "from_domain": "工作流编排域",
      "relation_type": "服务调用",
      "strength": 7.0,
      "to_domain": "人工介入域"
    },
    {
      "description": "智能体执行代码生成和文件操作时依赖文件工具域提供底层文件系统支持",
      "from_domain": "智能体执行域",
      "relation_type": "工具支持",
      "strength": 7.0,
      "to_domain": "文件工具域"
    },
    {
      "description": "所有核心业务域依赖配置管理域获取LLM服务配置和运行时参数",
      "from_domain": "智能体执行域",
      "relation_type": "配置依赖",
      "strength": 8.0,
      "to_domain": "配置管理域"
    },
    {
      "description": "用户界面域作为系统入口，依赖工作流编排域执行具体的业务逻辑",
      "from_domain": "用户界面域",
      "relation_type": "服务调用",
      "strength": 8.0,
      "to_domain": "工作流编排域"
    },
    {
      "description": "工具函数域为其他域提供通用的辅助功能支持",
      "from_domain": "智能体执行域",
      "relation_type": "工具支持",
      "strength": 5.0,
      "to_domain": "工具函数域"
    },
    {
      "description": "库接口域作为系统对外的统一接口，封装所有内部域的公共API",
      "from_domain": "用户界面域",
      "relation_type": "服务调用",
      "strength": 7.0,
      "to_domain": "库接口域"
    },
    {
      "description": "数据模型域为工作流编排提供会话状态和工件数据的持久化支持",
      "from_domain": "工作流编排域",
      "relation_type": "数据依赖",
      "strength": 8.0,
      "to_domain": "数据模型域"
    }
  ]
}
```

### Workflow Research Report
Contains static analysis results of the codebase and business process analysis.

```json
{
  "main_workflow": {
    "description": "这是Cowork AI系统的核心工作流程，实现了从创意输入到最终交付的完整8阶段软件开发生命周期。流程从用户通过CLI输入创意开始，经过需求分析、技术设计、实现规划、代码生成、质量检查、反馈处理，最终生成交付报告。该流程支持多轮迭代和人工介入，通过工作流编排器协调多个智能体协同工作。",
    "flowchart_mermaid": "flowchart TD\n    A[用户创意输入] --> B[创意输入智能体]\n    B --> C[PRD生成智能体]\n    C --> D[设计智能体]\n    D --> E[计划智能体]\n    E --> F[代码执行智能体]\n    F --> G[检查智能体]\n    G --> H{质量检查通过?}\n    H -->|否| I[反馈智能体]\n    I --> F\n    H -->|是| J[交付智能体]\n    J --> K[最终交付物]\n    \n    L[人工介入点] --> C\n    L --> D\n    L --> E\n    L --> F\n    L --> G\n    \n    M[会话恢复] --> N[加载会话状态]\n    N --> O[识别当前阶段]\n    O --> P[从指定阶段继续]\n    P --> Q[继续后续流程]\n    \n    style A fill:#e1f5fe\n    style K fill:#c8e6c9\n    style L fill:#fff3e0\n    style M fill:#f3e5f5",
    "name": "AI驱动的多智能体软件开发主流程"
  },
  "other_important_workflows": [
    {
      "description": "支持用户中断后从任意阶段恢复开发工作，确保开发过程的连续性和可恢复性。流程包括会话状态加载、工件数据读取和流程重定向三个主要步骤。",
      "flowchart_mermaid": "flowchart TD\n    A[用户选择恢复会话] --> B[编排器加载会话状态]\n    B --> C[工件存储读取阶段数据]\n    C --> D[识别当前完成阶段]\n    D --> E[确定继续执行起点]\n    E --> F[从指定阶段继续工作流]\n    F --> G[继续后续开发流程]\n    \n    style A fill:#f3e5f5\n    style G fill:#e1f5fe",
      "name": "会话恢复和管理流程"
    },
    {
      "description": "支持需求变更时的增量代码更新，保护用户自定义修改。通过需求差异分析、影响文件识别和更新计划生成三个步骤实现智能化的代码更新管理。",
      "flowchart_mermaid": "flowchart TD\n    A[检测PRD版本变更] --> B[代码更新智能体]\n    B --> C[需求差异分析]\n    C --> D[识别受影响文件]\n    D --> E[生成更新计划]\n    E --> F[制定合并策略]\n    F --> G[执行增量更新]\n    G --> H[更新完成]\n    \n    style A fill:#fff3e0\n    style H fill:#c8e6c9",
      "name": "增量代码更新流程"
    },
    {
      "description": "在关键决策点允许用户审核、确认和提供反馈，确保AI决策符合用户期望。通过HITL控制器提供多种交互模式支持用户参与。",
      "flowchart_mermaid": "flowchart TD\n    A[智能体生成结果] --> B{需要人工确认?}\n    B -->|是| C[HITL控制器介入]\n    C --> D[用户审核内容]\n    D --> E{用户确认?}\n    E -->|是| F[继续后续流程]\n    E -->|否| G[用户提供反馈]\n    G --> H[反馈智能体处理]\n    H --> I[重新生成或修改]\n    I --> A\n    B -->|否| F\n    \n    style C fill:#fff3e0\n    style G fill:#ffcdd2",
      "name": "人工介入控制流程"
    },
    {
      "description": "多层次的代码验证流程，包括需求覆盖检查、文件存在性验证、内容质量检查和语言特定编译检查，确保生成的代码符合质量标准和功能要求。",
      "flowchart_mermaid": "flowchart TD\n    A[代码变更完成] --> B[检查智能体启动]\n    B --> C[加载PRD和计划工件]\n    C --> D[需求覆盖验证]\n    D --> E[文件存在性检查]\n    E --> F[内容质量检查]\n    F --> G[语言编译检查]\n    G --> H[生成检查报告]\n    H --> I[更新TodoList状态]\n    I --> J[检查完成]\n    \n    style J fill:#c8e6c9\n    style H fill:#e1f5fe",
      "name": "代码质量检查流程"
    }
  ]
}
```

### Code Insights Data
Code analysis results from preprocessing phase, including definitions of functions, classes, and modules.

```json
[
  {
    "code_dossier": {
      "code_purpose": "entry",
      "description": null,
      "file_path": "crates/cowork-cli/src/main.rs",
      "functions": [
        "main",
        "interactive_mode",
        "resume_session",
        "inspect_session",
        "export_session"
      ],
      "importance_score": 1.0,
      "interfaces": [
        "Cli",
        "Commands"
      ],
      "name": "main.rs",
      "source_summary": "use anyhow::Result;\nuse clap::{Parser, Subcommand};\nuse cowork_core::{ArtifactStore, Orchestrator, ModelConfig};\nuse tracing_subscriber::EnvFilter;\n\n#[derive(Parser)]\n#[command(name = \"cowork\")]\n#[command(about = \"AI-powered multi-agent software development system\", long_about = None)]\nstruct Cli {\n    #[command(subcommand)]\n    command: Option<Commands>,\n\n    /// Path to model configuration file (TOML)\n    #[arg(long, default_value = \"项目材料/大模型配置说明.md\")]\n    config: String,\n}\n\n#[derive(Subcommand)]\nenum Commands {\n    /// Resume a session\n    Resume {\n        session_id: String,\n    },\n    /// Inspect a session's artifacts\n    Inspect {\n        session_id: String,\n    },\n    /// Export final deliverables\n    Export {\n        session_id: String,\n    },\n}\n\n#[tokio::main]\nasync fn main() -> Result<()> {\n    // Load environment variables\n    dotenv::dotenv().ok();\n\n    // Initialize logging\n    tracing_subscriber::fmt()\n        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))\n        .init();\n\n    let cli = Cli::parse();\n\n    // Load model configuration\n    let model_config = ModelConfig::from_file(&cli.config)\n        .or_else(|e| {\n            tracing::warn!(\"Failed to load config from file: {}, trying environment variables\", e);\n            ModelConfig::from_env()\n        })?;\n\n    tracing::info!(\"Model configuration loaded:\");\n    tracing::info!(\"  LLM: {} at {}\", model_config.llm.model_name, model_config.llm.api_base_url);\n\n    // Initialize ArtifactStore\n    let store = ArtifactStore::new(\".cowork\");\n    let orchestrator = Orchestrator::new(store);\n\n    match cli.command {\n        None => {\n            // Default: interactive mode - create new session\n            interactive_mode(orchestrator, model_config).await?;\n        }\n        Some(Commands::Resume { session_id }) => {\n            resume_session(orchestrator, &session_id, model_config).await?;\n        }\n        Some(Commands::Inspect { session_id }) => {\n            inspect_session(orchestrator, &session_id)?;\n        }\n        Some(Commands::Export { session_id }) => {\n            export_session(&session_id)?;\n        }\n    }\n\n    Ok(())\n}\n\nasync fn interactive_mode(orchestrator: Orchestrator, model_config: ModelConfig) -> Result<()> {\n    use console::style;\n\n    println!(\"{}\", style(\"Welcome to Cowork!\").bold().cyan());\n    println!(\"AI-powered multi-agent software development system\\n\");\n\n    // Create new session\n    let session_id = orchestrator.create_session()?;\n    println!(\"Session created: {}\\n\", style(&session_id).green());\n\n    // Run workflow\n    println!(\"Starting workflow...\\n\");\n    orchestrator.run_full_workflow(&session_id, &model_config).await?;\n\n    println!(\"\\n{}\", style(\"Session completed!\").bold().green());\n    println!(\"Session ID: {}\", session_id);\n    println!(\"Artifacts saved to: .cowork/{}/artifacts/\", session_id);\n\n    Ok(())\n}\n\nasync fn resume_session(orchestrator: Orchestrator, session_id: &str, model_config: ModelConfig) -> Result<()> {\n    use console::style;\n\n    println!(\"{}\", style(format!(\"🔄 恢复会话: {}\", session_id)).bold().cyan());\n\n    // 调用 orchestrator 的 resume_session 方法\n    orchestrator.resume_session(session_id, &model_config).await?;\n\n    println!(\"\\n{}\", style(\"✅ 会话恢复完成！\").bold().green());\n\n    Ok(())\n}\n\nfn inspect_session(orchestrator: Orchestrator, session_id: &str) -> Result<()> {\n    use console::style;\n\n    println!(\"{}\", style(format!(\"🔍 检查会话: {}\", session_id)).bold().cyan());\n\n    // 加载 session meta\n    let meta = orchestrator.load_session_meta(session_id)?;\n    println!(\"\\n📊 会话信息:\");\n    println!(\"  创建时间: {}\", meta.created_at);\n    println!(\"  当前阶段: {:?}\", meta.current_stage);\n    println!(\"  已完成阶段: {:?}\", meta.completed_stages);\n\n    let artifacts = orchestrator.list_artifacts(session_id)?;\n\n    if artifacts.is_empty() {\n        println!(\"{}\", style(\"\\n⚠️  没有找到 artifacts\").yellow());\n        return Ok(());\n    }\n\n    println!(\"\\n📦 Artifacts ({} 个):\", artifacts.len());\n    for artifact in artifacts {\n        println!(\"  ┌─ {} ({:?})\", artifact.artifact_id, artifact.stage);\n        println!(\"  │  JSON: {}\", artifact.path_json.display());\n        println!(\"  └─ MD:   {}\", artifact.path_md.display());\n    }\n\n    // 显示下一步建议\n    let all_stages = cowork_core::Stage::all();\n    let next_stage = all_stages\n        .iter()\n        .find(|s| !meta.completed_stages.contains(s))\n        .cloned();\n\n    if let Some(stage) = next_stage {\n        println!(\"\\n💡 提示:\");\n        println!(\"  下一阶段: {:?}\", stage);\n        println!(\"  恢复命令: cowork resume {}\", session_id);\n    } else {\n        println!(\"\\n✅ 所有阶段已完成！\");\n    }\n\n    Ok(())\n}\n\nfn export_session(session_id: &str) -> Result<()> {\n    use console::style;\n    use std::fs;\n    use std::path::PathBuf;\n\n    println!(\"{}\", style(format!(\"📤 导出会话: {}\", session_id)).bold().cyan());\n\n    let session_dir = PathBuf::from(\".cowork\").join(session_id);\n    if !session_dir.exists() {\n        return Err(anyhow::anyhow!(\"Session {} not found\", session_id));\n    }\n\n    // 创建导出目录\n    let export_dir = PathBuf::from(\"exports\").join(session_id);\n    fs::create_dir_all(&export_dir)?;\n\n    // 复制所有 markdown 文件\n    let artifacts_dir = session_dir.join(\"artifacts\");\n    let mut exported_count = 0;\n\n    if artifacts_dir.exists() {\n        for entry in fs::read_dir(&artifacts_dir)? {\n            let entry = entry?;\n            let path = entry.path();\n            \n            if path.extension().and_then(|s| s.to_str()) == Some(\"md\") {\n                let file_name = path.file_name().unwrap();\n                let dest = export_dir.join(file_name);\n                fs::copy(&path, &dest)?;\n                println!(\"  ✓ {}\", file_name.to_string_lossy());\n                exported_count += 1;\n            }\n        }\n    }\n\n    // 复制 meta.json\n    let meta_src = session_dir.join(\"meta.json\");\n    if meta_src.exists() {\n        fs::copy(&meta_src, export_dir.join(\"meta.json\"))?;\n        println!(\"  ✓ meta.json\");\n        exported_count += 1;\n    }\n\n    println!(\"\\n✅ 导出完成！\");\n    println!(\"  导出文件数: {}\", exported_count);\n    println!(\"  导出目录: {}\", export_dir.display());\n\n    Ok(())\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 10.0,
      "lines_of_code": 205,
      "number_of_classes": 2,
      "number_of_functions": 5
    },
    "dependencies": [
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "anyhow",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "clap",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": false,
        "line_number": null,
        "name": "cowork_core",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "tracing_subscriber",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "dotenv",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "tokio",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "console",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "std",
        "is_external": false,
        "line_number": null,
        "name": "std::fs",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "std",
        "is_external": false,
        "line_number": null,
        "name": "std::path",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": null,
        "name": "cowork_core::Stage",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "This is the entry point of the Cowork CLI application, a command-line interface for an AI-powered multi-agent software development system. It defines the CLI structure with subcommands (Resume, Inspect, Export) and a default interactive mode. The main function initializes logging, loads model configuration from a file or environment variables, creates an orchestrator with an artifact store, and routes execution based on user commands. It supports creating new sessions, resuming existing ones, inspecting session artifacts, and exporting final deliverables. The code includes both English and Chinese UI messages, indicating multilingual support. All business logic is delegated to the cowork_core module, making this component a thin CLI wrapper.",
    "interfaces": [
      {
        "description": "CLI configuration structure with subcommand and config file path",
        "interface_type": "struct",
        "name": "Cli",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "command",
            "param_type": "Option<Commands>"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "config",
            "param_type": "String"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Enumeration of supported CLI subcommands with session_id parameter",
        "interface_type": "enum",
        "name": "Commands",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "Resume",
            "param_type": "Resume"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "Inspect",
            "param_type": "Inspect"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "Export",
            "param_type": "Export"
          }
        ],
        "return_type": null,
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Parse and validate CLI commands",
      "Initialize system components (logging, config, artifact store)",
      "Route execution to appropriate session handlers",
      "Manage session lifecycle via orchestrator",
      "Provide user-facing feedback and output formatting"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "config",
      "description": null,
      "file_path": "crates/cowork-core/src/config.rs",
      "functions": [
        "ModelConfig::from_file",
        "ModelConfig::from_env"
      ],
      "importance_score": 0.9,
      "interfaces": [
        "LlmConfig",
        "EmbeddingConfig",
        "ModelConfig"
      ],
      "name": "config.rs",
      "source_summary": "use anyhow::Result;\nuse serde::{Deserialize, Serialize};\n\n/// 大模型配置（从文件加载）\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct LlmConfig {\n    pub api_base_url: String,\n    pub api_key: String,\n    pub model_name: String,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct EmbeddingConfig {\n    pub api_base_url: String,\n    pub api_key: String,\n    pub model_name: String,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct ModelConfig {\n    pub llm: LlmConfig,\n    pub embedding: EmbeddingConfig,\n}\n\nimpl ModelConfig {\n    /// 从 TOML 文件加载配置\n    pub fn from_file(path: &str) -> Result<Self> {\n        let content = std::fs::read_to_string(path)?;\n        Ok(toml::from_str(&content)?)\n    }\n\n    /// 从环境变量加载配置（备用）\n    pub fn from_env() -> Result<Self> {\n        Ok(Self {\n            llm: LlmConfig {\n                api_base_url: std::env::var(\"LLM_API_BASE_URL\")?,\n                api_key: std::env::var(\"LLM_API_KEY\")?,\n                model_name: std::env::var(\"LLM_MODEL_NAME\")?,\n            },\n            embedding: EmbeddingConfig {\n                api_base_url: std::env::var(\"EMBEDDING_API_BASE_URL\")?,\n                api_key: std::env::var(\"EMBEDDING_API_KEY\")?,\n                model_name: std::env::var(\"EMBEDDING_MODEL_NAME\")?,\n            },\n        })\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 1.0,
      "lines_of_code": 47,
      "number_of_classes": 3,
      "number_of_functions": 2
    },
    "dependencies": [
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "anyhow",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "serde",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "toml",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "This configuration component defines structured data models for large language model (LLM) and embedding service configurations, along with static methods to load these configurations either from a TOML file or from environment variables. It serves as the central configuration hub for AI-related services in the system, ensuring consistent and type-safe access to critical API endpoints and credentials.",
    "interfaces": [
      {
        "description": "Configuration for large language model API access",
        "interface_type": "struct",
        "name": "LlmConfig",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "api_base_url",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "api_key",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "model_name",
            "param_type": "String"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Configuration for embedding model API access",
        "interface_type": "struct",
        "name": "EmbeddingConfig",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "api_base_url",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "api_key",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "model_name",
            "param_type": "String"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Composite configuration containing both LLM and embedding settings",
        "interface_type": "struct",
        "name": "ModelConfig",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "llm",
            "param_type": "LlmConfig"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "embedding",
            "param_type": "EmbeddingConfig"
          }
        ],
        "return_type": null,
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Define structured configuration models for LLM and embedding services",
      "Provide methods to load configuration from TOML files",
      "Provide fallback methods to load configuration from environment variables",
      "Ensure type safety and serialization/deserialization compatibility",
      "Centralize configuration access logic for AI services"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "model",
      "description": null,
      "file_path": "crates/cowork-core/src/artifacts/mod.rs",
      "functions": [
        "new",
        "with_summary",
        "with_prev",
        "as_str",
        "all"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "ArtifactEnvelope",
        "ArtifactMeta",
        "ArtifactLinks",
        "Stage",
        "IdeaSpec",
        "PRD",
        "Scope",
        "Requirement",
        "Priority",
        "RequirementType",
        "Constraint",
        "HitlQuestion",
        "DesignDoc",
        "CliDesign",
        "Workflow",
        "Architecture",
        "IoConfig",
        "Plan",
        "C4Design",
        "Task",
        "Milestone",
        "TodoList",
        "TodoItem",
        "TodoStatus",
        "CodeChange",
        "RequirementMapping",
        "TargetProject",
        "ProjectStructure",
        "Layout",
        "Module",
        "ModuleType",
        "Tooling",
        "Change",
        "Command",
        "Phase",
        "CheckReport",
        "CheckResult",
        "AcceptanceResult",
        "Issue",
        "TodoCompletion",
        "RequirementCoverage",
        "RequirementChecklist",
        "ChecklistItem",
        "VerificationStatus",
        "Feedback",
        "Delta",
        "Rerun",
        "DeliveryReport",
        "IdeaSpecArtifact",
        "PRDArtifact",
        "DesignDocArtifact",
        "PlanArtifact",
        "CodeChangeArtifact",
        "CheckReportArtifact",
        "FeedbackArtifact",
        "DeliveryReportArtifact"
      ],
      "name": "mod.rs",
      "source_summary": "use chrono::{DateTime, Utc};\nuse serde::{Deserialize, Serialize};\nuse uuid::Uuid;\n\n#[cfg(test)]\nmod tests;\n\n/// Artifact metadata envelope (所有 json 共享)\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct ArtifactEnvelope<T> {\n    pub meta: ArtifactMeta,\n    pub summary: Vec<String>,\n    pub links: ArtifactLinks,\n    pub data: T,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct ArtifactMeta {\n    pub session_id: String,\n    pub artifact_id: String,\n    pub stage: Stage,\n    pub v: u32,\n    #[serde(with = \"chrono::serde::ts_seconds\")]\n    pub ts: DateTime<Utc>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct ArtifactLinks {\n    pub prev: Vec<String>,\n}\n\n/// Stage 枚举\n#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]\n#[serde(rename_all = \"snake_case\")]\npub enum Stage {\n    IdeaIntake,\n    Requirements,\n    Design,\n    Plan,\n    Coding,\n    Check,\n    Feedback,\n    Delivery,\n}\n\nimpl Stage {\n    pub fn as_str(&self) -> &'static str {\n        match self {\n            Stage::IdeaIntake => \"idea_intake\",\n            Stage::Requirements => \"requirements\",\n            Stage::Design => \"design\",\n            Stage::Plan => \"plan\",\n            Stage::Coding => \"coding\",\n            Stage::Check => \"check\",\n            Stage::Feedback => \"feedback\",\n            Stage::Delivery => \"delivery\",\n        }\n    }\n\n    pub fn all() -> &'static [Stage] {\n        &[\n            Stage::IdeaIntake,\n            Stage::Requirements,\n            Stage::Design,\n            Stage::Plan,\n            Stage::Coding,\n            Stage::Check,\n            Stage::Feedback,\n            Stage::Delivery,\n        ]\n    }\n}\n\n/// IDEA Intake → IdeaSpec\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct IdeaSpec {\n    pub bg: String,\n    pub g: Vec<String>,\n    pub ng: Vec<String>,\n    pub c: Vec<String>,\n    pub sc: Vec<String>,\n    pub r: Vec<String>,\n    pub q: Vec<String>,\n}\n\n/// Requirements → PRD\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct PRD {\n    pub scope: Scope,\n    pub reqs: Vec<Requirement>,\n    pub cons: Vec<Constraint>,\n    pub hitl: Vec<HitlQuestion>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Scope {\n    pub g: Vec<String>,\n    pub ng: Vec<String>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Requirement {\n    pub id: String,\n    pub pri: Priority,\n    #[serde(rename = \"type\")]\n    pub req_type: RequirementType,\n    pub desc: String,\n    pub deps: Vec<String>,\n    pub ac: Vec<String>,\n}\n\n#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]\n#[serde(rename_all = \"lowercase\")]\npub enum Priority {\n    P0,\n    P1,\n    P2,\n}\n\n#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]\n#[serde(rename_all = \"lowercase\")]\npub enum RequirementType {\n    Func,\n    Nfr,\n    Constraint,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Constraint {\n    pub id: String,\n    pub desc: String,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct HitlQuestion {\n    pub id: String,\n    pub q: String,\n    pub opts: Vec<String>,\n    pub def: String,\n}\n\n/// Design → DesignDoc\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct DesignDoc {\n    pub cli: CliDesign,\n    pub wf: Workflow,\n    pub arch: Architecture,\n    pub io: IoConfig,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct CliDesign {\n    pub modes: Vec<String>,\n    pub hitl_flow: Vec<String>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Workflow {\n    pub stages: Vec<String>,\n    pub transitions: Vec<String>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Architecture {\n    pub layers: Vec<String>,\n    pub comps: Vec<String>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct IoConfig {\n    pub artifact_dir: String,\n    pub formats: Vec<String>,\n}\n\n/// Plan → Plan\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Plan {\n    pub c4: C4Design,\n    pub tasks: Vec<Task>,\n    pub milestones: Vec<Milestone>,\n    #[serde(default, skip_serializing_if = \"Option::is_none\")]\n    pub todo_list: Option<TodoList>,  // 新增：任务分解列表\n}\n\n/// TodoList（任务分解）\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct TodoList {\n    pub items: Vec<TodoItem>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct TodoItem {\n    pub id: String,                      // \"TASK-001\"\n    pub description: String,             // \"实现用户登录功能\"\n    pub status: TodoStatus,\n    pub related_requirements: Vec<String>,  // [\"REQ-001\", \"REQ-002\"]\n    pub related_files: Vec<String>,         // [\"src/auth/login.rs\"]\n    pub verification_method: String,        // \"unit_test\" | \"manual_test\" | \"code_review\"\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\n#[serde(rename_all = \"snake_case\")]\npub enum TodoStatus {\n    Pending,\n    InProgress,\n    Completed,\n    Blocked { reason: String },\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct C4Design {\n    pub context: Vec<String>,\n    pub containers: Vec<String>,\n    pub components: Vec<String>,\n    pub code: Vec<String>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Task {\n    pub id: String,\n    pub pri: Priority,\n    pub desc: String,\n    pub deps: Vec<String>,\n    pub out: Vec<String>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Milestone {\n    pub id: String,\n    pub desc: String,\n    pub done_when: Vec<String>,\n}\n\n/// Coding → CodeChange\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct CodeChange {\n    pub target: TargetProject,\n    pub project: ProjectStructure,\n    pub changes: Vec<Change>,\n    pub cmds: Vec<Command>,\n    #[serde(default, skip_serializing_if = \"Vec::is_empty\")]\n    pub requirement_mapping: Vec<RequirementMapping>,  // 新增：需求映射\n}\n\n/// 需求到文件的映射关系\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct RequirementMapping {\n    pub req_id: String,\n    pub files: Vec<String>,\n    pub note: String,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct TargetProject {\n    pub lang: String,\n    pub stack: Vec<String>,\n    pub build: Vec<String>,\n    pub test: Vec<String>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct ProjectStructure {\n    pub root: String,\n    pub layout: Layout,\n    pub modules: Vec<Module>,\n    pub tooling: Tooling,\n}\n\n#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]\n#[serde(rename_all = \"lowercase\")]\npub enum Layout {\n    Mono,\n    Single,\n    Unknown,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Module {\n    pub name: String,\n    pub path: String,\n    #[serde(rename = \"type\")]\n    pub module_type: ModuleType,\n}\n\n#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]\n#[serde(rename_all = \"lowercase\")]\npub enum ModuleType {\n    Service,\n    Lib,\n    App,\n    Pkg,\n    Unknown,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Tooling {\n    pub pkg: String,\n    pub build: Vec<String>,\n    pub test: Vec<String>,\n    pub lint: Vec<String>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Change {\n    pub path: String,\n    pub kind: String,\n    pub note: String,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Command {\n    pub cmd: String,\n    pub expect: String,\n    pub phase: Phase,\n}\n\n#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]\n#[serde(rename_all = \"lowercase\")]\npub enum Phase {\n    Check,\n    Build,\n    Test,\n    Lint,\n    Run,\n}\n\n/// Check → CheckReport\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct CheckReport {\n    pub checks: Vec<CheckResult>,\n    pub ac_results: Vec<AcceptanceResult>,\n    pub issues: Vec<Issue>,\n    #[serde(default, skip_serializing_if = \"Option::is_none\")]\n    pub todo_completion: Option<TodoCompletion>,        // 新增：TodoList 完成度\n    #[serde(default, skip_serializing_if = \"Option::is_none\")]\n    pub requirement_coverage: Option<RequirementCoverage>,  // 新增：需求覆盖度\n}\n\n/// TodoList 完成度统计\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct TodoCompletion {\n    pub total: usize,\n    pub completed: usize,\n    pub pending: usize,\n    pub blocked: usize,\n}\n\n/// 需求覆盖度统计\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct RequirementCoverage {\n    pub total_requirements: usize,\n    pub verified: usize,\n    pub partially_verified: usize,\n    pub not_verified: usize,\n    pub failed: usize,\n    pub coverage_percentage: f64,\n}\n\n/// 需求检查清单\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct RequirementChecklist {\n    pub items: Vec<ChecklistItem>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct ChecklistItem {\n    pub req_id: String,                  // \"REQ-001\"\n    pub description: String,             // \"支持诗歌语义化展示\"\n    pub implemented_in: Vec<String>,     // [\"poem.html\"]\n    pub verification_status: VerificationStatus,\n    pub evidence: Vec<String>,           // [\"Found <article> tags\", \"Semantic HTML structure\"]\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\n#[serde(rename_all = \"snake_case\")]\npub enum VerificationStatus {\n    NotVerified,\n    Verified,\n    PartiallyVerified,\n    Failed { reason: String },\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct CheckResult {\n    pub id: String,\n    pub cmd: String,\n    pub status: String,\n    pub out_ref: String,\n    pub notes: Vec<String>,\n    pub phase: Phase,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct AcceptanceResult {\n    pub req_id: String,\n    pub ac: String,\n    pub status: String,\n    pub notes: Vec<String>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Issue {\n    pub id: String,\n    pub sev: String,\n    pub desc: String,\n    pub fix_hint: String,\n}\n\n/// Feedback → Feedback\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Feedback {\n    pub delta: Vec<Delta>,\n    pub rerun: Vec<Rerun>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Delta {\n    pub target_stage: Stage,\n    pub change: String,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Rerun {\n    pub stage: Stage,\n    pub reason: String,\n}\n\n/// Delivery → DeliveryReport\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct DeliveryReport {\n    pub cap: Vec<String>,\n    pub howto: Vec<String>,\n    pub limits: Vec<String>,\n    pub acceptance: Vec<String>,\n}\n\n/// Type aliases for convenience\npub type IdeaSpecArtifact = ArtifactEnvelope<IdeaSpec>;\npub type PRDArtifact = ArtifactEnvelope<PRD>;\npub type DesignDocArtifact = ArtifactEnvelope<DesignDoc>;\npub type PlanArtifact = ArtifactEnvelope<Plan>;\npub type CodeChangeArtifact = ArtifactEnvelope<CodeChange>;\npub type CheckReportArtifact = ArtifactEnvelope<CheckReport>;\npub type FeedbackArtifact = ArtifactEnvelope<Feedback>;\npub type DeliveryReportArtifact = ArtifactEnvelope<DeliveryReport>;\n\nimpl<T> ArtifactEnvelope<T>\nwhere\n    T: Serialize,\n{\n    pub fn new(session_id: String, stage: Stage, data: T) -> Self {\n        Self {\n            meta: ArtifactMeta {\n                session_id: session_id.clone(),\n                artifact_id: Uuid::new_v4().to_string(),\n                stage,\n                v: 1,\n                ts: Utc::now(),\n            },\n            summary: Vec::new(),\n            links: ArtifactLinks { prev: Vec::new() },\n            data,\n        }\n    }\n\n    pub fn with_summary(mut self, summary: Vec<String>) -> Self {\n        self.summary = summary;\n        self\n    }\n\n    pub fn with_prev(mut self, prev: Vec<String>) -> Self {\n        self.links.prev = prev;\n        self\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 7.0,
      "lines_of_code": 475,
      "number_of_classes": 55,
      "number_of_functions": 5
    },
    "dependencies": [
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 1,
        "name": "chrono",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 2,
        "name": "serde",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 3,
        "name": "uuid",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": 7,
        "name": "tests",
        "path": "./crates/cowork-core/src/artifacts/tests.rs",
        "version": null
      }
    ],
    "detailed_description": "The mod.rs component is a comprehensive data model module that defines the complete artifact structure for a collaborative software development workflow system. It implements a sophisticated type system that captures the entire software development lifecycle from idea intake to delivery. The component provides strongly-typed data structures for each development stage (IdeaIntake, Requirements, Design, Plan, Coding, Check, Feedback, Delivery) with detailed metadata tracking, relationships, and validation capabilities. The architecture follows a generic envelope pattern (ArtifactEnvelope<T>) that wraps stage-specific data with consistent metadata, session tracking, and versioning.",
    "interfaces": [
      {
        "description": "Generic envelope that wraps stage-specific artifact data with metadata",
        "interface_type": "struct",
        "name": "ArtifactEnvelope",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "meta",
            "param_type": "ArtifactMeta"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "summary",
            "param_type": "Vec<String>"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "links",
            "param_type": "ArtifactLinks"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "data",
            "param_type": "T"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Create new artifact envelope with generated metadata",
        "interface_type": "method",
        "name": "new",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "session_id",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "stage",
            "param_type": "Stage"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "data",
            "param_type": "T"
          }
        ],
        "return_type": "Self",
        "visibility": "public"
      },
      {
        "description": "Builder method to add summary lines",
        "interface_type": "method",
        "name": "with_summary",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "self",
            "param_type": "Self"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "summary",
            "param_type": "Vec<String>"
          }
        ],
        "return_type": "Self",
        "visibility": "public"
      },
      {
        "description": "Builder method to add previous artifact links",
        "interface_type": "method",
        "name": "with_prev",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "self",
            "param_type": "Self"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "prev",
            "param_type": "Vec<String>"
          }
        ],
        "return_type": "Self",
        "visibility": "public"
      },
      {
        "description": "Convert Stage enum to string representation",
        "interface_type": "method",
        "name": "as_str",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "self",
            "param_type": "Stage"
          }
        ],
        "return_type": "&'static str",
        "visibility": "public"
      },
      {
        "description": "Get all available stages as slice",
        "interface_type": "method",
        "name": "all",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "self",
            "param_type": "Stage"
          }
        ],
        "return_type": "&'static [Stage]",
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Define and maintain the complete data schema for software development artifacts across all workflow stages",
      "Provide type-safe data structures for serialization/deserialization between system components",
      "Implement workflow stage tracking and artifact relationship management through metadata and links",
      "Support extensible artifact evolution through versioning and generic type parameters",
      "Enable comprehensive development progress tracking through nested structures and validation fields"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "tool",
      "description": "A comprehensive file operations toolkit providing file reading, writing, directory listing, and incremental file editing capabilities",
      "file_path": "crates/cowork-core/src/tools/file_tools.rs",
      "functions": [
        "read_file",
        "write_file",
        "list_directory",
        "file_exists",
        "create_directory",
        "read_file_range",
        "replace_line_range",
        "insert_lines",
        "delete_line_range",
        "append_to_file"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "ReadFileParams",
        "WriteFileParams",
        "ListDirParams",
        "FileExistsParams",
        "CreateDirParams",
        "ReadFileRangeParams",
        "ReplaceLineRangeParams",
        "InsertLinesParams",
        "DeleteLineRangeParams",
        "AppendToFileParams",
        "FunctionTool",
        "FileToolsBundle"
      ],
      "name": "file_tools.rs",
      "source_summary": "use adk_rust::prelude::*;\nuse adk_rust::AdkError;\nuse schemars::JsonSchema;\nuse serde::{Deserialize, Serialize};\nuse serde_json::json;\nuse std::sync::Arc;\nuse std::path::Path;\n\n/// 文件读取参数\n#[derive(JsonSchema, Serialize, Deserialize)]\npub struct ReadFileParams {\n    /// 文件路径（相对或绝对路径）\n    pub path: String,\n}\n\n/// 文件写入参数\n#[derive(JsonSchema, Serialize, Deserialize)]\npub struct WriteFileParams {\n    /// 文件路径\n    pub path: String,\n    /// 文件内容\n    pub content: String,\n}\n\n/// 目录列表参数\n#[derive(JsonSchema, Serialize, Deserialize)]\npub struct ListDirParams {\n    /// 目录路径\n    pub path: String,\n    /// 是否递归列出子目录\n    #[serde(default)]\n    pub recursive: bool,\n    /// 是否包含隐藏文件（默认不包含）\n    #[serde(default)]\n    pub include_hidden: bool,\n}\n\n/// 文件存在检查参数\n#[derive(JsonSchema, Serialize, Deserialize)]\npub struct FileExistsParams {\n    /// 文件路径\n    pub path: String,\n}\n\n/// 创建目录参数\n#[derive(JsonSchema, Serialize, Deserialize)]\npub struct CreateDirParams {\n    /// 目录路径\n    pub path: String,\n    /// 是否创建父目录\n    #[serde(default)]\n    pub recursive: bool,\n}\n\n/// 读取文件范围参数\n#[derive(JsonSchema, Serialize, Deserialize)]\npub struct ReadFileRangeParams {\n    /// 文件路径\n    pub path: String,\n    /// 起始行号（1-based，包含）\n    pub start_line: usize,\n    /// 结束行号（1-based，包含）。如果省略，读到文件末尾\n    #[serde(default)]\n    pub end_line: Option<usize>,\n}\n\n/// 替换文件行范围参数\n#[derive(JsonSchema, Serialize, Deserialize)]\npub struct ReplaceLineRangeParams {\n    /// 文件路径\n    pub path: String,\n    /// 起始行号（1-based，包含）\n    pub start_line: usize,\n    /// 结束行号（1-based，包含）\n    pub end_line: usize,\n    /// 新内容（多行文本）\n    pub new_content: String,\n}\n\n/// 插入行参数\n#[derive(JsonSchema, Serialize, Deserialize)]\npub struct InsertLinesParams {\n    /// 文件路径\n    pub path: String,\n    /// 在此行号之后插入（1-based）。0 表示在文件开头插入\n    pub after_line: usize,\n    /// 要插入的内容\n    pub content: String,\n}\n\n/// 删除行范围参数\n#[derive(JsonSchema, Serialize, Deserialize)]\npub struct DeleteLineRangeParams {\n    /// 文件路径\n    pub path: String,\n    /// 起始行号（1-based，包含）\n    pub start_line: usize,\n    /// 结束行号（1-based，包含）\n    pub end_line: usize,\n}\n\n/// 追加到文件参数\n#[derive(JsonSchema, Serialize, Deserialize)]\npub struct AppendToFileParams {\n    /// 文件路径\n    pub path: String,\n    /// 要追加的内容\n    pub content: String,\n}\n\n/// 检查文件名是否为隐藏文件\n#[cfg(test)]\npub(crate) fn is_hidden_file(path: &Path) -> bool {\n    path.file_name()\n        .and_then(|s| s.to_str())\n        .map(|s| s.starts_with('.'))\n        .unwrap_or(false)\n}\n\n/// 构建 gitignore walker\npub(crate) fn build_gitignore_walker(root: &str, recursive: bool, include_hidden: bool) -> ignore::Walk {\n    let mut builder = ignore::WalkBuilder::new(root);\n    \n    // 设置最大深度\n    if !recursive {\n        builder.max_depth(Some(1));\n    }\n    \n    // 控制是否包含隐藏文件\n    if !include_hidden {\n        builder.hidden(false); // 排除隐藏文件\n    } else {\n        builder.hidden(true); // 包含隐藏文件\n    }\n    \n    // 始终遵循 .gitignore 规则\n    builder.git_ignore(true);\n    builder.git_global(true);\n    builder.git_exclude(true);\n    \n    // 不遵循符号链接（避免循环）\n    builder.follow_links(false);\n    \n    builder.build()\n}\n\n/// 文件工具集合\npub struct FileToolsBundle {\n    pub read_file: Arc<FunctionTool>,\n    pub write_file: Arc<FunctionTool>,\n    pub list_dir: Arc<FunctionTool>,\n    pub file_exists: Arc<FunctionTool>,\n    pub create_dir: Arc<FunctionTool>,\n    // 增量编辑工具\n    pub read_file_range: Arc<FunctionTool>,\n    pub replace_line_range: Arc<FunctionTool>,\n    pub insert_lines: Arc<FunctionTool>,\n    pub delete_line_range: Arc<FunctionTool>,\n    pub append_to_file: Arc<FunctionTool>,\n}\n\n/// 创建文件操作工具集\npub fn create_file_tools() -> FileToolsBundle {\n    // 1. 读取文件工具\n    let read_file = Arc::new(\n        FunctionTool::new(\n            \"read_file\",\n            \"Read the contents of a file. Returns the file content as a string.\",\n            |_ctx, args| async move {\n                let params: ReadFileParams = serde_json::from_value(args)\n                    .map_err(|e| AdkError::Tool(format!(\"Invalid parameters: {}\", e)))?;\n\n                match std::fs::read_to_string(&params.path) {\n                    Ok(content) => Ok(json!({\n                        \"success\": true,\n                        \"path\": params.path,\n                        \"content\": content,\n                        \"size\": content.len()\n                    })),\n                    Err(e) => Err(AdkError::Tool(format!(\n                        \"Failed to read file '{}': {}\",\n                        params.path, e\n                    ))),\n                }\n            },\n        )\n        .with_parameters_schema::<ReadFileParams>(),\n    );\n\n    // 2. 写入文件工具\n    let write_file = Arc::new(\n        FunctionTool::new(\n            \"write_file\",\n            \"Write content to a file. Creates the file if it doesn't exist, overwrites if it does.\",\n            |_ctx, args| async move {\n                let params: WriteFileParams = serde_json::from_value(args)\n                    .map_err(|e| AdkError::Tool(format!(\"Invalid parameters: {}\", e)))?;\n\n                // 确保父目录存在\n                if let Some(parent) = Path::new(&params.path).parent() {\n                    if !parent.exists() {\n                        std::fs::create_dir_all(parent).map_err(|e| {\n                            AdkError::Tool(format!(\n                                \"Failed to create parent directories: {}\",\n                                e\n                            ))\n                        })?;\n                    }\n                }\n\n                match std::fs::write(&params.path, &params.content) {\n                    Ok(_) => Ok(json!({\n                        \"success\": true,\n                        \"path\": params.path,\n                        \"bytes_written\": params.content.len()\n                    })),\n                    Err(e) => Err(AdkError::Tool(format!(\n                        \"Failed to write file '{}': {}\",\n                        params.path, e\n                    ))),\n                }\n            },\n        )\n        .with_parameters_schema::<WriteFileParams>(),\n    );\n\n    // 3. 列出目录工具（使用 ignore crate 处理 .gitignore）\n    let list_dir = Arc::new(\n        FunctionTool::new(\n            \"list_directory\",\n            \"List files and directories in a directory. Automatically respects .gitignore rules and excludes hidden files by default. Use include_hidden=true to show hidden files.\",\n            |_ctx, args| async move {\n                let params: ListDirParams = serde_json::from_value(args)\n                    .map_err(|e| AdkError::Tool(format!(\"Invalid parameters: {}\", e)))?;\n\n                let mut entries = Vec::new();\n                \n                // 使用 ignore crate 构建 walker（自动处理 .gitignore）\n                let walker = build_gitignore_walker(&params.path, params.recursive, params.include_hidden);\n\n                for result in walker {\n                    match result {\n                        Ok(entry) => {\n                            let path = entry.path();\n                            \n                            // 跳过根目录自身\n                            if path == Path::new(&params.path) {\n                                continue;\n                            }\n                            \n                            let path_str = path.to_string_lossy().to_string();\n                            let is_dir = path.is_dir();\n                            let is_file = path.is_file();\n                            \n                            let size = if is_file {\n                                std::fs::metadata(path).ok().map(|m| m.len()).unwrap_or(0)\n                            } else {\n                                0\n                            };\n\n                            entries.push(json!({\n                                \"path\": path_str,\n                                \"is_dir\": is_dir,\n                                \"is_file\": is_file,\n                                \"size\": size\n                            }));\n                        }\n                        Err(e) => {\n                            // 记录错误但继续处理其他文件\n                            tracing::warn!(\"Error walking directory: {}\", e);\n                        }\n                    }\n                }\n\n                Ok(json!({\n                    \"success\": true,\n                    \"path\": params.path,\n                    \"count\": entries.len(),\n                    \"entries\": entries,\n                    \"note\": \"Hidden files and .gitignore patterns are excluded by default\"\n                }))\n            },\n        )\n        .with_parameters_schema::<ListDirParams>(),\n    );\n\n    // 4. 检查文件是否存在工具\n    let file_exists = Arc::new(\n        FunctionTool::new(\n            \"file_exists\",\n            \"Check if a file or directory exists.\",\n            |_ctx, args| async move {\n                let params: FileExistsParams = serde_json::from_value(args)\n                    .map_err(|e| AdkError::Tool(format!(\"Invalid parameters: {}\", e)))?;\n\n                let path = Path::new(&params.path);\n                let exists = path.exists();\n                let is_dir = path.is_dir();\n                let is_file = path.is_file();\n\n                Ok(json!({\n                    \"path\": params.path,\n                    \"exists\": exists,\n                    \"is_dir\": is_dir,\n                    \"is_file\": is_file\n                }))\n            },\n        )\n        .with_parameters_schema::<FileExistsParams>(),\n    );\n\n    // 5. 创建目录工具\n    let create_dir = Arc::new(\n        FunctionTool::new(\n            \"create_directory\",\n            \"Create a directory. Can create parent directories if recursive is true.\",\n            |_ctx, args| async move {\n                let params: CreateDirParams = serde_json::from_value(args)\n                    .map_err(|e| AdkError::Tool(format!(\"Invalid parameters: {}\", e)))?;\n\n                let result = if params.recursive {\n                    std::fs::create_dir_all(&params.path)\n                } else {\n                    std::fs::create_dir(&params.path)\n                };\n\n                match result {\n                    Ok(_) => Ok(json!({\n                        \"success\": true,\n                        \"path\": params.path\n                    })),\n                    Err(e) => Err(AdkError::Tool(format!(\n                        \"Failed to create directory '{}': {}\",\n                        params.path, e\n                    ))),\n                }\n            },\n        )\n        .with_parameters_schema::<CreateDirParams>(),\n    );\n\n    // 6. 读取文件范围工具（用于大文件）\n    let read_file_range = Arc::new(\n        FunctionTool::new(\n            \"read_file_range\",\n            \"Read a specific range of lines from a file. Useful for large files to avoid context overflow. Line numbers are 1-based.\",\n            |_ctx, args| async move {\n                let params: ReadFileRangeParams = serde_json::from_value(args)\n                    .map_err(|e| AdkError::Tool(format!(\"Invalid parameters: {}\", e)))?;\n\n                let content = std::fs::read_to_string(&params.path)\n                    .map_err(|e| AdkError::Tool(format!(\"Failed to read file '{}': {}\", params.path, e)))?;\n\n                let lines: Vec<&str> = content.lines().collect();\n                let total_lines = lines.len();\n\n                if params.start_line < 1 || params.start_line > total_lines {\n                    return Err(AdkError::Tool(format!(\n                        \"Invalid start_line: {} (file has {} lines)\",\n                        params.start_line, total_lines\n                    )));\n                }\n\n                let start_idx = params.start_line - 1;\n                let end_idx = match params.end_line {\n                    Some(end) if end > 0 => end.min(total_lines),\n                    _ => total_lines,\n                };\n\n                let selected_lines = &lines[start_idx..end_idx];\n                let selected_content = selected_lines.join(\"\\n\");\n\n                Ok(json!({\n                    \"success\": true,\n                    \"path\": params.path,\n                    \"start_line\": params.start_line,\n                    \"end_line\": end_idx,\n                    \"total_lines\": total_lines,\n                    \"content\": selected_content,\n                    \"lines_read\": selected_lines.len()\n                }))\n            },\n        )\n        .with_parameters_schema::<ReadFileRangeParams>(),\n    );\n\n    // 7. 替换行范围工具\n    let replace_line_range = Arc::new(\n        FunctionTool::new(\n            \"replace_line_range\",\n            \"Replace a range of lines in a file with new content. Useful for modifying specific sections without rewriting the entire file. Line numbers are 1-based.\",\n            |_ctx, args| async move {\n                let params: ReplaceLineRangeParams = serde_json::from_value(args)\n                    .map_err(|e| AdkError::Tool(format!(\"Invalid parameters: {}\", e)))?;\n\n                let content = std::fs::read_to_string(&params.path)\n                    .map_err(|e| AdkError::Tool(format!(\"Failed to read file '{}': {}\", params.path, e)))?;\n\n                let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();\n                let total_lines = lines.len();\n\n                if params.start_line < 1 || params.start_line > total_lines {\n                    return Err(AdkError::Tool(format!(\"Invalid start_line: {}\", params.start_line)));\n                }\n                if params.end_line < params.start_line || params.end_line > total_lines {\n                    return Err(AdkError::Tool(format!(\"Invalid end_line: {}\", params.end_line)));\n                }\n\n                // 替换指定范围\n                let start_idx = params.start_line - 1;\n                let end_idx = params.end_line;\n                \n                let new_lines: Vec<String> = params.new_content.lines().map(|s| s.to_string()).collect();\n                lines.splice(start_idx..end_idx, new_lines.clone());\n\n                let new_content = lines.join(\"\\n\");\n                std::fs::write(&params.path, new_content)\n                    .map_err(|e| AdkError::Tool(format!(\"Failed to write file: {}\", e)))?;\n\n                Ok(json!({\n                    \"success\": true,\n                    \"path\": params.path,\n                    \"replaced_lines\": format!(\"{}-{}\", params.start_line, params.end_line),\n                    \"new_line_count\": new_lines.len(),\n                    \"total_lines_after\": lines.len()\n                }))\n            },\n        )\n        .with_parameters_schema::<ReplaceLineRangeParams>(),\n    );\n\n    // 8. 插入行工具\n    let insert_lines = Arc::new(\n        FunctionTool::new(\n            \"insert_lines\",\n            \"Insert new lines after a specific line number. Line numbers are 1-based. Use after_line=0 to insert at the beginning.\",\n            |_ctx, args| async move {\n                let params: InsertLinesParams = serde_json::from_value(args)\n                    .map_err(|e| AdkError::Tool(format!(\"Invalid parameters: {}\", e)))?;\n\n                let content = std::fs::read_to_string(&params.path)\n                    .map_err(|e| AdkError::Tool(format!(\"Failed to read file '{}': {}\", params.path, e)))?;\n\n                let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();\n                let total_lines = lines.len();\n\n                if params.after_line > total_lines {\n                    return Err(AdkError::Tool(format!(\n                        \"Invalid after_line: {} (file has {} lines)\",\n                        params.after_line, total_lines\n                    )));\n                }\n\n                let new_lines: Vec<String> = params.content.lines().map(|s| s.to_string()).collect();\n                let insert_idx = params.after_line;\n                \n                for (i, line) in new_lines.iter().enumerate() {\n                    lines.insert(insert_idx + i, line.clone());\n                }\n\n                let new_content = lines.join(\"\\n\");\n                std::fs::write(&params.path, new_content)\n                    .map_err(|e| AdkError::Tool(format!(\"Failed to write file: {}\", e)))?;\n\n                Ok(json!({\n                    \"success\": true,\n                    \"path\": params.path,\n                    \"inserted_after_line\": params.after_line,\n                    \"lines_inserted\": new_lines.len(),\n                    \"total_lines_after\": lines.len()\n                }))\n            },\n        )\n        .with_parameters_schema::<InsertLinesParams>(),\n    );\n\n    // 9. 删除行范围工具\n    let delete_line_range = Arc::new(\n        FunctionTool::new(\n            \"delete_line_range\",\n            \"Delete a range of lines from a file. Line numbers are 1-based.\",\n            |_ctx, args| async move {\n                let params: DeleteLineRangeParams = serde_json::from_value(args)\n                    .map_err(|e| AdkError::Tool(format!(\"Invalid parameters: {}\", e)))?;\n\n                let content = std::fs::read_to_string(&params.path)\n                    .map_err(|e| AdkError::Tool(format!(\"Failed to read file '{}': {}\", params.path, e)))?;\n\n                let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();\n                let total_lines = lines.len();\n\n                if params.start_line < 1 || params.start_line > total_lines {\n                    return Err(AdkError::Tool(format!(\"Invalid start_line: {}\", params.start_line)));\n                }\n                if params.end_line < params.start_line || params.end_line > total_lines {\n                    return Err(AdkError::Tool(format!(\"Invalid end_line: {}\", params.end_line)));\n                }\n\n                let start_idx = params.start_line - 1;\n                let end_idx = params.end_line;\n                let deleted_count = end_idx - start_idx;\n                \n                lines.drain(start_idx..end_idx);\n\n                let new_content = lines.join(\"\\n\");\n                std::fs::write(&params.path, new_content)\n                    .map_err(|e| AdkError::Tool(format!(\"Failed to write file: {}\", e)))?;\n\n                Ok(json!({\n                    \"success\": true,\n                    \"path\": params.path,\n                    \"deleted_lines\": format!(\"{}-{}\", params.start_line, params.end_line),\n                    \"lines_deleted\": deleted_count,\n                    \"total_lines_after\": lines.len()\n                }))\n            },\n        )\n        .with_parameters_schema::<DeleteLineRangeParams>(),\n    );\n\n    // 10. 追加到文件工具\n    let append_to_file = Arc::new(\n        FunctionTool::new(\n            \"append_to_file\",\n            \"Append content to the end of a file. Adds a newline before the content if the file doesn't end with one.\",\n            |_ctx, args| async move {\n                let params: AppendToFileParams = serde_json::from_value(args)\n                    .map_err(|e| AdkError::Tool(format!(\"Invalid parameters: {}\", e)))?;\n\n                let mut file = std::fs::OpenOptions::new()\n                    .create(true)\n                    .append(true)\n                    .open(&params.path)\n                    .map_err(|e| AdkError::Tool(format!(\"Failed to open file '{}': {}\", params.path, e)))?;\n\n                use std::io::Write;\n                \n                // 如果文件不为空且不以换行结尾，先加个换行\n                let metadata = file.metadata()\n                    .map_err(|e| AdkError::Tool(format!(\"Failed to get metadata: {}\", e)))?;\n                \n                if metadata.len() > 0 {\n                    write!(file, \"\\n\")\n                        .map_err(|e| AdkError::Tool(format!(\"Failed to write newline: {}\", e)))?;\n                }\n\n                write!(file, \"{}\", params.content)\n                    .map_err(|e| AdkError::Tool(format!(\"Failed to append content: {}\", e)))?;\n\n                Ok(json!({\n                    \"success\": true,\n                    \"path\": params.path,\n                    \"bytes_appended\": params.content.len()\n                }))\n            },\n        )\n        .with_parameters_schema::<AppendToFileParams>(),\n    );\n\n    FileToolsBundle {\n        read_file,\n        write_file,\n        list_dir,\n        file_exists,\n        create_dir,\n        read_file_range,\n        replace_line_range,\n        insert_lines,\n        delete_line_range,\n        append_to_file,\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 30.0,
      "lines_of_code": 572,
      "number_of_classes": 11,
      "number_of_functions": 10
    },
    "dependencies": [
      {
        "dependency_type": "external",
        "is_external": true,
        "line_number": null,
        "name": "adk_rust",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "external",
        "is_external": true,
        "line_number": null,
        "name": "schemars",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "external",
        "is_external": true,
        "line_number": null,
        "name": "serde",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "external",
        "is_external": true,
        "line_number": null,
        "name": "serde_json",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "external",
        "is_external": true,
        "line_number": null,
        "name": "ignore",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "std::sync::Arc",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "std::path::Path",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "std::fs",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "This component provides a comprehensive set of file system operations as functional tools. It offers basic file operations (read/write), directory operations (list/create), and advanced incremental file editing capabilities (line-based insert/replace/delete). The implementation uses Rust's async/await pattern and provides structured parameter validation through JSON schemas. The toolkit is designed to be used as building blocks for file manipulation workflows in a larger system.",
    "interfaces": [
      {
        "description": "Reads the entire content of a file",
        "interface_type": "function",
        "name": "read_file",
        "parameters": [
          {
            "description": "File path to read",
            "is_optional": false,
            "name": "path",
            "param_type": "string"
          }
        ],
        "return_type": "json",
        "visibility": "public"
      },
      {
        "description": "Writes content to a file, creating parent directories if needed",
        "interface_type": "function",
        "name": "write_file",
        "parameters": [
          {
            "description": "File path to write",
            "is_optional": false,
            "name": "path",
            "param_type": "string"
          },
          {
            "description": "Content to write",
            "is_optional": false,
            "name": "content",
            "param_type": "string"
          }
        ],
        "return_type": "json",
        "visibility": "public"
      },
      {
        "description": "Lists directory contents with .gitignore support",
        "interface_type": "function",
        "name": "list_directory",
        "parameters": [
          {
            "description": "Directory path",
            "is_optional": false,
            "name": "path",
            "param_type": "string"
          },
          {
            "description": "Whether to list recursively",
            "is_optional": true,
            "name": "recursive",
            "param_type": "boolean"
          },
          {
            "description": "Whether to include hidden files",
            "is_optional": true,
            "name": "include_hidden",
            "param_type": "boolean"
          }
        ],
        "return_type": "json",
        "visibility": "public"
      },
      {
        "description": "Checks if a file or directory exists",
        "interface_type": "function",
        "name": "file_exists",
        "parameters": [
          {
            "description": "Path to check",
            "is_optional": false,
            "name": "path",
            "param_type": "string"
          }
        ],
        "return_type": "json",
        "visibility": "public"
      },
      {
        "description": "Creates a directory with optional parent creation",
        "interface_type": "function",
        "name": "create_directory",
        "parameters": [
          {
            "description": "Directory path to create",
            "is_optional": false,
            "name": "path",
            "param_type": "string"
          },
          {
            "description": "Whether to create parent directories",
            "is_optional": true,
            "name": "recursive",
            "param_type": "boolean"
          }
        ],
        "return_type": "json",
        "visibility": "public"
      },
      {
        "description": "Reads specific line range from a file",
        "interface_type": "function",
        "name": "read_file_range",
        "parameters": [
          {
            "description": "File path",
            "is_optional": false,
            "name": "path",
            "param_type": "string"
          },
          {
            "description": "Starting line number (1-based)",
            "is_optional": false,
            "name": "start_line",
            "param_type": "integer"
          },
          {
            "description": "Ending line number (optional)",
            "is_optional": true,
            "name": "end_line",
            "param_type": "integer"
          }
        ],
        "return_type": "json",
        "visibility": "public"
      },
      {
        "description": "Replaces a range of lines in a file",
        "interface_type": "function",
        "name": "replace_line_range",
        "parameters": [
          {
            "description": "File path",
            "is_optional": false,
            "name": "path",
            "param_type": "string"
          },
          {
            "description": "Starting line number",
            "is_optional": false,
            "name": "start_line",
            "param_type": "integer"
          },
          {
            "description": "Ending line number",
            "is_optional": false,
            "name": "end_line",
            "param_type": "integer"
          },
          {
            "description": "New content to replace with",
            "is_optional": false,
            "name": "new_content",
            "param_type": "string"
          }
        ],
        "return_type": "json",
        "visibility": "public"
      },
      {
        "description": "Inserts lines after a specific line number",
        "interface_type": "function",
        "name": "insert_lines",
        "parameters": [
          {
            "description": "File path",
            "is_optional": false,
            "name": "path",
            "param_type": "string"
          },
          {
            "description": "Line number to insert after",
            "is_optional": false,
            "name": "after_line",
            "param_type": "integer"
          },
          {
            "description": "Content to insert",
            "is_optional": false,
            "name": "content",
            "param_type": "string"
          }
        ],
        "return_type": "json",
        "visibility": "public"
      },
      {
        "description": "Deletes a range of lines from a file",
        "interface_type": "function",
        "name": "delete_line_range",
        "parameters": [
          {
            "description": "File path",
            "is_optional": false,
            "name": "path",
            "param_type": "string"
          },
          {
            "description": "Starting line number",
            "is_optional": false,
            "name": "start_line",
            "param_type": "integer"
          },
          {
            "description": "Ending line number",
            "is_optional": false,
            "name": "end_line",
            "param_type": "integer"
          }
        ],
        "return_type": "json",
        "visibility": "public"
      },
      {
        "description": "Appends content to the end of a file",
        "interface_type": "function",
        "name": "append_to_file",
        "parameters": [
          {
            "description": "File path",
            "is_optional": false,
            "name": "path",
            "param_type": "string"
          },
          {
            "description": "Content to append",
            "is_optional": false,
            "name": "content",
            "param_type": "string"
          }
        ],
        "return_type": "json",
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Provide atomic file system operations with proper error handling",
      "Implement incremental file editing capabilities for large files",
      "Handle directory operations with .gitignore support and hidden file filtering",
      "Ensure data integrity through proper file locking and atomic operations",
      "Provide comprehensive parameter validation and schema documentation"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "database",
      "description": "Memory module component providing artifact storage and management functionality",
      "file_path": "crates/cowork-core/src/memory/mod.rs",
      "functions": [
        "new",
        "put",
        "get",
        "list",
        "session_exists",
        "session_dir",
        "artifacts_dir",
        "artifact_path",
        "parse_stage",
        "generate_markdown"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "ArtifactStore",
        "FileArtifactStore",
        "ArtifactMeta"
      ],
      "name": "mod.rs",
      "source_summary": "use anyhow::Result;\nuse serde::{de::DeserializeOwned, Serialize};\nuse std::path::{Path, PathBuf};\n\nuse crate::artifacts::Stage;\n\n#[cfg(test)]\nmod tests;\n\n/// Artifact 存储接口（简化为直接使用 FileArtifactStore）\npub struct ArtifactStore {\n    store: FileArtifactStore,\n}\n\nimpl ArtifactStore {\n    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {\n        Self {\n            store: FileArtifactStore::new(base_dir),\n        }\n    }\n\n    /// 写入 artifact（json + md）\n    pub fn put<T: Serialize>(&self, session_id: &str, stage: Stage, artifact: &T) -> Result<String> {\n        self.store.put(session_id, stage, artifact)\n    }\n\n    /// 读取 artifact（json）\n    pub fn get<T: DeserializeOwned>(&self, session_id: &str, artifact_id: &str) -> Result<T> {\n        self.store.get(session_id, artifact_id)\n    }\n\n    /// 列出 session 的所有 artifacts\n    pub fn list(&self, session_id: &str) -> Result<Vec<ArtifactMeta>> {\n        self.store.list(session_id)\n    }\n\n    /// 检查 session 是否存在\n    pub fn session_exists(&self, session_id: &str) -> bool {\n        self.store.session_exists(session_id)\n    }\n}\n\n#[derive(Debug, Clone)]\npub struct ArtifactMeta {\n    pub artifact_id: String,\n    pub stage: Stage,\n    pub path_json: PathBuf,\n    pub path_md: PathBuf,\n}\n\n/// 默认的文件存储实现\nstruct FileArtifactStore {\n    base_dir: PathBuf,\n}\n\nimpl FileArtifactStore {\n    fn new<P: AsRef<Path>>(base_dir: P) -> Self {\n        Self {\n            base_dir: base_dir.as_ref().to_path_buf(),\n        }\n    }\n\n    fn session_dir(&self, session_id: &str) -> PathBuf {\n        self.base_dir.join(session_id)\n    }\n\n    fn artifacts_dir(&self, session_id: &str) -> PathBuf {\n        self.session_dir(session_id).join(\"artifacts\")\n    }\n\n    fn artifact_path(&self, session_id: &str, stage: Stage, artifact_id: &str, ext: &str) -> PathBuf {\n        self.artifacts_dir(session_id)\n            .join(format!(\"{}.{}.{}\", stage.as_str(), artifact_id, ext))\n    }\n\n    fn put<T: Serialize>(&self, session_id: &str, stage: Stage, artifact: &T) -> Result<String> {\n        use std::fs;\n\n        let artifacts_dir = self.artifacts_dir(session_id);\n        fs::create_dir_all(&artifacts_dir)?;\n\n        // Extract artifact_id from the artifact (assuming it has a meta field)\n        let json_str = serde_json::to_string_pretty(artifact)?;\n        let json_value: serde_json::Value = serde_json::from_str(&json_str)?;\n        let artifact_id = json_value[\"meta\"][\"artifact_id\"]\n            .as_str()\n            .ok_or_else(|| anyhow::anyhow!(\"Missing artifact_id in meta\"))?\n            .to_string();\n\n        // Write JSON\n        let json_path = self.artifact_path(session_id, stage, &artifact_id, \"json\");\n        fs::write(&json_path, json_str)?;\n\n        // Write MD (minimal template)\n        let md_content = self.generate_markdown(&json_value)?;\n        let md_path = self.artifact_path(session_id, stage, &artifact_id, \"md\");\n        fs::write(&md_path, md_content)?;\n\n        tracing::info!(\"Artifact saved: {}\", artifact_id);\n        Ok(artifact_id)\n    }\n\n    fn get<T: DeserializeOwned>(&self, session_id: &str, artifact_id: &str) -> Result<T> {\n        use std::fs;\n\n        // Find the artifact by scanning the artifacts directory\n        let artifacts_dir = self.artifacts_dir(session_id);\n        for entry in fs::read_dir(&artifacts_dir)? {\n            let entry = entry?;\n            let path = entry.path();\n            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {\n                if name.contains(artifact_id) && name.ends_with(\".json\") {\n                    let content = fs::read_to_string(&path)?;\n                    return Ok(serde_json::from_str(&content)?);\n                }\n            }\n        }\n\n        anyhow::bail!(\"Artifact not found: {}\", artifact_id)\n    }\n\n    fn list(&self, session_id: &str) -> Result<Vec<ArtifactMeta>> {\n        use std::fs;\n\n        let artifacts_dir = self.artifacts_dir(session_id);\n        if !artifacts_dir.exists() {\n            return Ok(Vec::new());\n        }\n\n        let mut artifacts = Vec::new();\n        for entry in fs::read_dir(&artifacts_dir)? {\n            let entry = entry?;\n            let path = entry.path();\n            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {\n                if name.ends_with(\".json\") {\n                    // Parse: <stage>.<artifact_id>.json\n                    let parts: Vec<&str> = name.rsplitn(3, '.').collect();\n                    if parts.len() == 3 {\n                        let artifact_id = parts[1].to_string();\n                        let stage_str = parts[2];\n                        if let Some(stage) = self.parse_stage(stage_str) {\n                            let path_json = path.clone();\n                            let path_md = path.with_extension(\"md\");\n                            artifacts.push(ArtifactMeta {\n                                artifact_id,\n                                stage,\n                                path_json,\n                                path_md,\n                            });\n                        }\n                    }\n                }\n            }\n        }\n\n        Ok(artifacts)\n    }\n\n    fn session_exists(&self, session_id: &str) -> bool {\n        self.session_dir(session_id).exists()\n    }\n\n    fn parse_stage(&self, s: &str) -> Option<Stage> {\n        match s {\n            \"idea_intake\" => Some(Stage::IdeaIntake),\n            \"requirements\" => Some(Stage::Requirements),\n            \"design\" => Some(Stage::Design),\n            \"plan\" => Some(Stage::Plan),\n            \"coding\" => Some(Stage::Coding),\n            \"check\" => Some(Stage::Check),\n            \"feedback\" => Some(Stage::Feedback),\n            \"delivery\" => Some(Stage::Delivery),\n            _ => None,\n        }\n    }\n\n    fn generate_markdown(&self, json: &serde_json::Value) -> Result<String> {\n        let mut md = String::new();\n\n        // Meta\n        if let Some(meta) = json.get(\"meta\") {\n            md.push_str(\"# Artifact\\n\\n\");\n            md.push_str(&format!(\"- **Session ID**: {}\\n\", meta[\"session_id\"].as_str().unwrap_or(\"\")));\n            md.push_str(&format!(\"- **Artifact ID**: {}\\n\", meta[\"artifact_id\"].as_str().unwrap_or(\"\")));\n            md.push_str(&format!(\"- **Stage**: {}\\n\", meta[\"stage\"].as_str().unwrap_or(\"\")));\n            md.push_str(&format!(\"- **Version**: {}\\n\", meta[\"v\"].as_u64().unwrap_or(0)));\n            md.push_str(&format!(\"- **Timestamp**: {}\\n\", meta[\"ts\"].as_i64().unwrap_or(0)));\n            md.push_str(\"\\n\");\n        }\n\n        // Summary\n        if let Some(summary) = json.get(\"summary\").and_then(|s| s.as_array()) {\n            md.push_str(\"## Summary\\n\\n\");\n            for item in summary {\n                if let Some(s) = item.as_str() {\n                    md.push_str(&format!(\"- {}\\n\", s));\n                }\n            }\n            md.push_str(\"\\n\");\n        }\n\n        // Data (simplified representation)\n        if let Some(data) = json.get(\"data\") {\n            md.push_str(\"## Data\\n\\n\");\n            md.push_str(\"```json\\n\");\n            md.push_str(&serde_json::to_string_pretty(data)?);\n            md.push_str(\"\\n```\\n\");\n        }\n\n        Ok(md)\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 16.0,
      "lines_of_code": 212,
      "number_of_classes": 3,
      "number_of_functions": 10
    },
    "dependencies": [
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 1,
        "name": "anyhow",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 2,
        "name": "serde",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "standard_library",
        "is_external": false,
        "line_number": 3,
        "name": "std::path",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 5,
        "name": "crate::artifacts::Stage",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "test_module",
        "is_external": false,
        "line_number": 8,
        "name": "tests",
        "path": "./crates/cowork-core/src/artifacts/tests.rs",
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 66,
        "name": "serde_json",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "standard_library",
        "is_external": false,
        "line_number": 64,
        "name": "std::fs",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 76,
        "name": "tracing",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "This component implements an artifact storage system for a coworking application. It provides a public ArtifactStore interface that wraps a private FileArtifactStore implementation. The system manages artifacts organized by session ID and development stage (idea_intake, requirements, design, plan, coding, check, feedback, delivery). Each artifact is stored as both JSON data and generated Markdown documentation. The component handles file system operations including directory creation, file writing/reading, and artifact metadata management.",
    "interfaces": [
      {
        "description": "Public interface for artifact storage operations",
        "interface_type": "struct",
        "name": "ArtifactStore",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Creates a new ArtifactStore instance",
        "interface_type": "constructor",
        "name": "ArtifactStore::new",
        "parameters": [
          {
            "description": "Base directory for artifact storage",
            "is_optional": false,
            "name": "base_dir",
            "param_type": "P: AsRef<Path>"
          }
        ],
        "return_type": "ArtifactStore",
        "visibility": "public"
      },
      {
        "description": "Stores an artifact with JSON and Markdown formats",
        "interface_type": "method",
        "name": "ArtifactStore::put",
        "parameters": [
          {
            "description": "Session identifier",
            "is_optional": false,
            "name": "session_id",
            "param_type": "&str"
          },
          {
            "description": "Development stage",
            "is_optional": false,
            "name": "stage",
            "param_type": "Stage"
          },
          {
            "description": "Artifact data to store",
            "is_optional": false,
            "name": "artifact",
            "param_type": "&T: Serialize"
          }
        ],
        "return_type": "Result<String>",
        "visibility": "public"
      },
      {
        "description": "Retrieves an artifact by ID",
        "interface_type": "method",
        "name": "ArtifactStore::get",
        "parameters": [
          {
            "description": "Session identifier",
            "is_optional": false,
            "name": "session_id",
            "param_type": "&str"
          },
          {
            "description": "Artifact identifier",
            "is_optional": false,
            "name": "artifact_id",
            "param_type": "&str"
          }
        ],
        "return_type": "Result<T>",
        "visibility": "public"
      },
      {
        "description": "Lists all artifacts in a session",
        "interface_type": "method",
        "name": "ArtifactStore::list",
        "parameters": [
          {
            "description": "Session identifier",
            "is_optional": false,
            "name": "session_id",
            "param_type": "&str"
          }
        ],
        "return_type": "Result<Vec<ArtifactMeta>>",
        "visibility": "public"
      },
      {
        "description": "Checks if a session exists",
        "interface_type": "method",
        "name": "ArtifactStore::session_exists",
        "parameters": [
          {
            "description": "Session identifier",
            "is_optional": false,
            "name": "session_id",
            "param_type": "&str"
          }
        ],
        "return_type": "bool",
        "visibility": "public"
      },
      {
        "description": "Metadata for stored artifacts",
        "interface_type": "struct",
        "name": "ArtifactMeta",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Internal file-based storage implementation",
        "interface_type": "struct",
        "name": "FileArtifactStore",
        "parameters": [],
        "return_type": null,
        "visibility": "private"
      },
      {
        "description": "Creates a new FileArtifactStore instance",
        "interface_type": "constructor",
        "name": "FileArtifactStore::new",
        "parameters": [
          {
            "description": "Base directory for storage",
            "is_optional": false,
            "name": "base_dir",
            "param_type": "P: AsRef<Path>"
          }
        ],
        "return_type": "FileArtifactStore",
        "visibility": "private"
      },
      {
        "description": "Internal implementation of artifact storage",
        "interface_type": "method",
        "name": "FileArtifactStore::put",
        "parameters": [
          {
            "description": "Session identifier",
            "is_optional": false,
            "name": "session_id",
            "param_type": "&str"
          },
          {
            "description": "Development stage",
            "is_optional": false,
            "name": "stage",
            "param_type": "Stage"
          },
          {
            "description": "Artifact data",
            "is_optional": false,
            "name": "artifact",
            "param_type": "&T: Serialize"
          }
        ],
        "return_type": "Result<String>",
        "visibility": "private"
      },
      {
        "description": "Internal implementation of artifact retrieval",
        "interface_type": "method",
        "name": "FileArtifactStore::get",
        "parameters": [
          {
            "description": "Session identifier",
            "is_optional": false,
            "name": "session_id",
            "param_type": "&str"
          },
          {
            "description": "Artifact identifier",
            "is_optional": false,
            "name": "artifact_id",
            "param_type": "&str"
          }
        ],
        "return_type": "Result<T>",
        "visibility": "private"
      },
      {
        "description": "Internal implementation of artifact listing",
        "interface_type": "method",
        "name": "FileArtifactStore::list",
        "parameters": [
          {
            "description": "Session identifier",
            "is_optional": false,
            "name": "session_id",
            "param_type": "&str"
          }
        ],
        "return_type": "Result<Vec<ArtifactMeta>>",
        "visibility": "private"
      },
      {
        "description": "Internal implementation of session existence check",
        "interface_type": "method",
        "name": "FileArtifactStore::session_exists",
        "parameters": [
          {
            "description": "Session identifier",
            "is_optional": false,
            "name": "session_id",
            "param_type": "&str"
          }
        ],
        "return_type": "bool",
        "visibility": "private"
      }
    ],
    "responsibilities": [
      "Artifact storage management with session-based organization",
      "File system operations for JSON and Markdown artifact files",
      "Artifact metadata extraction and management",
      "Session lifecycle management (creation, existence checking)",
      "Artifact discovery and listing functionality"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "controller",
      "description": null,
      "file_path": "crates/cowork-core/src/hitl/mod.rs",
      "functions": [
        "new",
        "input",
        "confirm",
        "review_and_edit_json",
        "review",
        "collect_feedback",
        "collect_feedback_with_default",
        "select"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "HitlController::new()",
        "HitlController::input()",
        "HitlController::confirm()",
        "HitlController::review_and_edit_json()",
        "HitlController::review()",
        "HitlController::collect_feedback()",
        "HitlController::collect_feedback_with_default()",
        "HitlController::select()",
        "Default::default()"
      ],
      "name": "mod.rs",
      "source_summary": "use anyhow::Result;\nuse dialoguer::{Confirm, Input, Editor};\nuse serde::Serialize;\nuse std::fmt::Display;\n\n/// Human-in-the-Loop 控制器\npub struct HitlController;\n\nimpl HitlController {\n    pub fn new() -> Self {\n        Self\n    }\n\n    /// 获取用户输入\n    pub fn input(&self, prompt: &str) -> Result<String> {\n        let input: String = Input::new()\n            .with_prompt(prompt)\n            .interact_text()?;\n        Ok(input)\n    }\n\n    /// 确认（是/否）\n    pub fn confirm(&self, prompt: &str) -> Result<bool> {\n        let confirmed = Confirm::new()\n            .with_prompt(prompt)\n            .default(true)\n            .interact()?;\n        Ok(confirmed)\n    }\n\n    /// 让用户在编辑器中审查和修改 JSON 内容\n    /// \n    /// 返回值：\n    /// - Ok(Some(modified_json)) - 用户修改了内容\n    /// - Ok(None) - 用户接受原内容\n    /// - Err(_) - 发生错误\n    pub fn review_and_edit_json<T>(&self, title: &str, data: &T) -> Result<Option<String>>\n    where\n        T: Serialize,\n    {\n        println!(\"\\n📝 请审查 {} 的内容\", title);\n        \n        // 转换为格式化的 JSON\n        let json_str = serde_json::to_string_pretty(data)?;\n        \n        // 显示摘要\n        let line_count = json_str.lines().count();\n        println!(\"  内容预览（共 {} 行）：\", line_count);\n        println!(\"  ────────────────────────────────────────\");\n        for (i, line) in json_str.lines().take(10).enumerate() {\n            println!(\"  {}: {}\", i + 1, line);\n        }\n        if line_count > 10 {\n            println!(\"  ... ({} 行省略)\", line_count - 10);\n        }\n        println!(\"  ────────────────────────────────────────\\n\");\n\n        // 询问用户是否要编辑\n        let should_edit = Confirm::new()\n            .with_prompt(\"是否需要修改此内容？\")\n            .default(false)\n            .interact()?;\n\n        if !should_edit {\n            return Ok(None);\n        }\n\n        // 打开编辑器\n        println!(\"📝 打开编辑器...（保存并关闭编辑器以提交修改）\");\n        let edited = Editor::new()\n            .require_save(true)\n            .edit(&json_str)?;\n\n        match edited {\n            Some(text) if text.trim() != json_str.trim() => {\n                // 验证 JSON 格式\n                match serde_json::from_str::<serde_json::Value>(&text) {\n                    Ok(_) => {\n                        println!(\"✅ JSON 格式验证通过\");\n                        Ok(Some(text))\n                    }\n                    Err(e) => {\n                        println!(\"❌ JSON 格式错误: {}\", e);\n                        let retry = Confirm::new()\n                            .with_prompt(\"是否重新编辑？\")\n                            .default(true)\n                            .interact()?;\n                        \n                        if retry {\n                            self.review_and_edit_json(title, data)\n                        } else {\n                            println!(\"⚠️  放弃修改，使用原始内容\");\n                            Ok(None)\n                        }\n                    }\n                }\n            }\n            _ => {\n                println!(\"ℹ️  内容未修改\");\n                Ok(None)\n            }\n        }\n    }\n\n    /// 简化版：让用户确认内容并选择是否修改\n    pub fn review<T>(&self, title: &str, data: &T) -> Result<bool>\n    where\n        T: Serialize + Display,\n    {\n        println!(\"\\n┌─────────────────────────────────────────┐\");\n        println!(\"│ 审查: {}                            \", title);\n        println!(\"└─────────────────────────────────────────┘\");\n        println!(\"{}\", data);\n        println!();\n\n        let approved = Confirm::new()\n            .with_prompt(\"是否接受此结果？\")\n            .default(true)\n            .interact()?;\n\n        Ok(approved)\n    }\n\n    /// 让用户提供反馈意见\n    pub fn collect_feedback(&self, prompt: &str) -> Result<String> {\n        println!(\"\\n💬 {}\", prompt);\n        \n        let feedback = Editor::new()\n            .require_save(false)\n            .edit(\"\")?\n            .unwrap_or_default();\n\n        Ok(feedback.trim().to_string())\n    }\n\n    /// 让用户提供反馈意见（带默认值）\n    pub fn collect_feedback_with_default(&self, prompt: &str, default: &str) -> Result<String> {\n        println!(\"\\n💬 {}\", prompt);\n        println!(\"(当前内容已预填充，可直接保存或修改)\");\n        \n        let feedback = Editor::new()\n            .require_save(false)\n            .edit(default)?\n            .unwrap_or_else(|| default.to_string());\n\n        Ok(feedback.trim().to_string())\n    }\n\n    /// 显示选项菜单并让用户选择\n    pub fn select(&self, prompt: &str, options: &[&str]) -> Result<usize> {\n        use dialoguer::Select;\n        \n        let selection = Select::new()\n            .with_prompt(prompt)\n            .items(options)\n            .default(0)\n            .interact()?;\n\n        Ok(selection)\n    }\n}\n\nimpl Default for HitlController {\n    fn default() -> Self {\n        Self::new()\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 9.0,
      "lines_of_code": 167,
      "number_of_classes": 1,
      "number_of_functions": 8
    },
    "dependencies": [
      {
        "dependency_type": "error_handling",
        "is_external": true,
        "line_number": 1,
        "name": "anyhow",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "user_interface",
        "is_external": true,
        "line_number": 2,
        "name": "dialoguer",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "serialization",
        "is_external": true,
        "line_number": 3,
        "name": "serde",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "standard_library",
        "is_external": false,
        "line_number": 4,
        "name": "std",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "json_processing",
        "is_external": true,
        "line_number": 54,
        "name": "serde_json",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "This is a Human-in-the-Loop (HITL) controller component that provides interactive user interface capabilities for collecting user input, confirmation, content review, and feedback collection. The component implements various interaction patterns including text input, yes/no confirmation, JSON content editing with validation, simplified content review, feedback collection with optional defaults, and menu selection. It serves as a bridge between automated processes and human oversight, enabling users to review, modify, and provide input on system outputs.",
    "interfaces": [
      {
        "description": "Creates a new instance of HitlController",
        "interface_type": "constructor",
        "name": "new",
        "parameters": [],
        "return_type": "HitlController",
        "visibility": "public"
      },
      {
        "description": "Collects text input from the user with a custom prompt",
        "interface_type": "method",
        "name": "input",
        "parameters": [
          {
            "description": "The prompt message displayed to the user",
            "is_optional": false,
            "name": "prompt",
            "param_type": "&str"
          }
        ],
        "return_type": "Result<String>",
        "visibility": "public"
      },
      {
        "description": "Presents a yes/no confirmation dialog with default set to true",
        "interface_type": "method",
        "name": "confirm",
        "parameters": [
          {
            "description": "The confirmation prompt message",
            "is_optional": false,
            "name": "prompt",
            "param_type": "&str"
          }
        ],
        "return_type": "Result<bool>",
        "visibility": "public"
      },
      {
        "description": "Allows users to review and edit JSON content with validation",
        "interface_type": "method",
        "name": "review_and_edit_json",
        "parameters": [
          {
            "description": "Title for the review session",
            "is_optional": false,
            "name": "title",
            "param_type": "&str"
          },
          {
            "description": "Serializable data to review and edit",
            "is_optional": false,
            "name": "data",
            "param_type": "&T"
          }
        ],
        "return_type": "Result<Option<String>>",
        "visibility": "public"
      },
      {
        "description": "Simplified review interface for content approval",
        "interface_type": "method",
        "name": "review",
        "parameters": [
          {
            "description": "Title for the review",
            "is_optional": false,
            "name": "title",
            "param_type": "&str"
          },
          {
            "description": "Displayable data to review",
            "is_optional": false,
            "name": "data",
            "param_type": "&T"
          }
        ],
        "return_type": "Result<bool>",
        "visibility": "public"
      },
      {
        "description": "Collects user feedback through an editor interface",
        "interface_type": "method",
        "name": "collect_feedback",
        "parameters": [
          {
            "description": "Feedback prompt message",
            "is_optional": false,
            "name": "prompt",
            "param_type": "&str"
          }
        ],
        "return_type": "Result<String>",
        "visibility": "public"
      },
      {
        "description": "Collects feedback with pre-populated default content",
        "interface_type": "method",
        "name": "collect_feedback_with_default",
        "parameters": [
          {
            "description": "Feedback prompt message",
            "is_optional": false,
            "name": "prompt",
            "param_type": "&str"
          },
          {
            "description": "Default feedback content",
            "is_optional": false,
            "name": "default",
            "param_type": "&str"
          }
        ],
        "return_type": "Result<String>",
        "visibility": "public"
      },
      {
        "description": "Presents a menu for user selection from multiple options",
        "interface_type": "method",
        "name": "select",
        "parameters": [
          {
            "description": "Selection prompt",
            "is_optional": false,
            "name": "prompt",
            "param_type": "&str"
          },
          {
            "description": "Array of option strings",
            "is_optional": false,
            "name": "options",
            "param_type": "&[&str]"
          }
        ],
        "return_type": "Result<usize>",
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Interactive User Input Collection",
      "Content Review and Approval",
      "JSON Data Validation and Editing",
      "User Feedback Collection",
      "Menu-based Selection Interface"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": "Intelligent agent that generates code change plans based on project requirements, design documents, and project analysis using a two-phase approach",
      "file_path": "crates/cowork-core/src/agents/code_planner.rs",
      "functions": [
        "new",
        "execute",
        "analyze_project_structure",
        "generate_code_plan"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "CodePlanner::new",
        "CodePlanner::execute",
        "CodePlanner::analyze_project_structure",
        "CodePlanner::generate_code_plan"
      ],
      "name": "code_planner.rs",
      "source_summary": "use anyhow::Result;\nuse adk_rust::prelude::*;\nuse adk_rust::model::{OpenAIClient, OpenAIConfig};\nuse adk_rust::runner::{Runner, RunnerConfig};\nuse adk_rust::session::{InMemorySessionService, CreateRequest, SessionService, GetRequest};\nuse futures::StreamExt;\nuse std::collections::HashMap;\nuse std::sync::Arc;\n\nuse crate::artifacts::*;\nuse crate::memory::ArtifactStore;\nuse crate::config::LlmConfig;\nuse crate::tools::create_file_tools;\nuse crate::agents::code_plan_normalizer::CodePlanNormalizer;\n\n/// Code Planner - 基于 Plan 生成代码变更计划\n/// 采用分阶段策略避免 max iteration 问题\n/// 注意：这是规划阶段，不执行实际的文件操作\npub struct CodePlanner {\n    model: Arc<OpenAIClient>,\n    store: Arc<ArtifactStore>,\n}\n\nimpl CodePlanner {\n    pub fn new(llm_config: &LlmConfig, store: Arc<ArtifactStore>) -> Result<Self> {\n        let config = OpenAIConfig::compatible(\n            llm_config.api_key.clone(),\n            llm_config.api_base_url.clone(),\n            llm_config.model_name.clone(),\n        );\n        \n        tracing::info!(\"Creating Code Planner with OpenAI-compatible client\");\n        \n        let model = OpenAIClient::new(config)?;\n\n        Ok(Self {\n            model: Arc::new(model),\n            store,\n        })\n    }\n\n    pub async fn execute(\n        &self, \n        session_id: &str,\n        prd_artifact: &PRDArtifact,\n        design_artifact: &DesignDocArtifact,\n        plan_artifact: &PlanArtifact\n    ) -> Result<CodeChangeArtifact> {\n        tracing::info!(\"CodePlanner: generating code change plan for session {}\", session_id);\n\n        // 分阶段执行策略：\n        // 1. 先分析项目结构（使用工具）\n        // 2. 再生成代码变更计划（基于 PRD + Design + Plan，不使用工具）\n        \n        // Phase 1: 项目结构分析\n        tracing::info!(\"Phase 1: Analyzing project structure...\");\n        let project_context = self.analyze_project_structure(session_id).await?;\n        \n        // Phase 2: 生成代码变更计划（基于分析结果和需求）\n        tracing::info!(\"Phase 2: Generating code change plan...\");\n        let code_change = self.generate_code_plan(\n            session_id,\n            prd_artifact,\n            design_artifact, \n            plan_artifact, \n            &project_context\n        ).await?;\n\n        // 保存 artifact\n        let summary = vec![\n            format!(\"Language: {}\", code_change.target.lang),\n            format!(\"Modules: {}\", code_change.project.modules.len()),\n            format!(\"Changes: {}\", code_change.changes.len()),\n            format!(\"Commands: {}\", code_change.cmds.len()),\n        ];\n\n        let artifact = ArtifactEnvelope::new(session_id.to_string(), Stage::Coding, code_change)\n            .with_summary(summary)\n            .with_prev(vec![plan_artifact.meta.artifact_id.clone()]);\n\n        self.store.put(session_id, Stage::Coding, &artifact)?;\n\n        tracing::info!(\"Code change artifact saved successfully\");\n\n        Ok(artifact)\n    }\n\n    /// Phase 1: 分析项目结构（限制工具调用次数）\n    async fn analyze_project_structure(&self, session_id: &str) -> Result<String> {\n        let file_tools = create_file_tools();\n\n        // 使用简化的 agent，只做项目结构分析\n        let agent = Arc::new(\n            LlmAgentBuilder::new(\"project_analyzer\")\n                .description(\"Analyze project structure efficiently\")\n                .instruction(\n                    r#\"You are a project structure analyzer. Your task is to understand the current project layout.\n\n**IMPORTANT RULES TO AVOID MAX ITERATIONS:**\n1. Call list_directory ONLY ONCE on the root directory (recursive=true)\n2. Based on the file list, identify key directories (src/, tests/, etc.)\n3. Read at most 2-3 key files (README.md, Cargo.toml, package.json, etc.)\n4. After gathering information, output your findings in JSON format\n5. DO NOT explore every file - just get the overview\n\n**Output JSON Format:**\n{\n  \"project_type\": \"rust|javascript|python|unknown\",\n  \"layout\": \"mono|single\",\n  \"key_dirs\": [\"src\", \"tests\", \"docs\"],\n  \"package_manager\": \"cargo|npm|pip|unknown\",\n  \"existing_files\": [\"list of important files\"],\n  \"notes\": \"brief observations\"\n}\n\nRemember: Maximum 5 tool calls total. Focus on efficiency.\"#,\n                )\n                .model(self.model.clone())\n                .output_key(\"project_analysis\")\n                .tool(file_tools.list_dir.clone())\n                .tool(file_tools.read_file.clone())\n                .tool(file_tools.file_exists.clone())\n                .build()?,\n        );\n\n        let session_service = Arc::new(InMemorySessionService::new());\n        let app_name = \"cowork\".to_string();\n        let user_id = format!(\"{}_analysis\", session_id);\n\n        let _session = session_service\n            .create(CreateRequest {\n                app_name: app_name.clone(),\n                user_id: user_id.clone(),\n                session_id: Some(format!(\"{}_phase1\", session_id)),\n                state: HashMap::new(),\n            })\n            .await?;\n\n        let runner = Runner::new(RunnerConfig {\n            app_name: app_name.clone(),\n            agent: agent.clone(),\n            session_service: session_service.clone(),\n            artifact_service: None,\n            memory_service: None,\n            run_config: None,\n        })?;\n\n        let input_content = Content::new(\"user\").with_text(\n            \"Analyze the current project structure in the current directory (.)\"\n        );\n\n        tracing::info!(\"Analyzing project structure...\");\n\n        let mut event_stream = runner\n            .run(user_id.clone(), format!(\"{}_phase1\", session_id), input_content)\n            .await?;\n\n        while let Some(event_result) = event_stream.next().await {\n            match event_result {\n                Ok(_event) => {},\n                Err(e) => {\n                    tracing::error!(\"Error during project analysis: {}\", e);\n                    return Err(anyhow::anyhow!(\"Project analysis failed: {}\", e));\n                }\n            }\n        }\n\n        let updated_session = session_service\n            .get(GetRequest {\n                user_id: user_id.clone(),\n                session_id: format!(\"{}_phase1\", session_id),\n                app_name: app_name.clone(),\n                after: None,\n                num_recent_events: None,\n            })\n            .await?;\n\n        let state = updated_session.state();\n        let analysis = state\n            .get(\"project_analysis\")\n            .ok_or_else(|| anyhow::anyhow!(\"No analysis output\"))?;\n\n        let analysis_str = match analysis {\n            serde_json::Value::String(s) => s.clone(),\n            v => serde_json::to_string_pretty(&v)?,\n        };\n\n        tracing::info!(\"Project analysis complete\");\n        Ok(analysis_str)\n    }\n\n    /// Phase 2: 生成代码变更计划（基于需求、设计和项目分析，不使用工具）\n    async fn generate_code_plan(\n        &self,\n        session_id: &str,\n        prd_artifact: &PRDArtifact,\n        design_artifact: &DesignDocArtifact,\n        plan_artifact: &PlanArtifact,\n        project_context: &str,\n    ) -> Result<CodeChange> {\n        let output_schema = serde_json::json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"target\": {\n                    \"type\": \"object\",\n                    \"properties\": {\n                        \"lang\": {\"type\": \"string\"},\n                        \"stack\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}},\n                        \"build\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}},\n                        \"test\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}\n                    },\n                    \"required\": [\"lang\", \"stack\", \"build\", \"test\"]\n                },\n                \"project\": {\n                    \"type\": \"object\",\n                    \"properties\": {\n                        \"root\": {\"type\": \"string\"},\n                        \"layout\": {\"type\": \"string\", \"enum\": [\"mono\", \"single\", \"unknown\"]},\n                        \"modules\": {\n                            \"type\": \"array\",\n                            \"items\": {\n                                \"type\": \"object\",\n                                \"properties\": {\n                                    \"name\": {\"type\": \"string\"},\n                                    \"path\": {\"type\": \"string\"},\n                                    \"type\": {\"type\": \"string\", \"enum\": [\"service\", \"lib\", \"app\", \"pkg\", \"unknown\"]}\n                                },\n                                \"required\": [\"name\", \"path\", \"type\"]\n                            }\n                        },\n                        \"tooling\": {\n                            \"type\": \"object\",\n                            \"properties\": {\n                                \"pkg\": {\"type\": \"string\"},\n                                \"build\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}},\n                                \"test\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}},\n                                \"lint\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}\n                            },\n                            \"required\": [\"pkg\", \"build\", \"test\", \"lint\"]\n                        }\n                    },\n                    \"required\": [\"root\", \"layout\", \"modules\", \"tooling\"]\n                },\n                \"changes\": {\n                    \"type\": \"array\",\n                    \"items\": {\n                        \"type\": \"object\",\n                        \"properties\": {\n                            \"path\": {\"type\": \"string\"},\n                            \"kind\": {\"type\": \"string\"},\n                            \"note\": {\"type\": \"string\"}\n                        },\n                        \"required\": [\"path\", \"kind\", \"note\"]\n                    }\n                },\n                \"cmds\": {\n                    \"type\": \"array\",\n                    \"items\": {\n                        \"type\": \"object\",\n                        \"properties\": {\n                            \"cmd\": {\"type\": \"string\"},\n                            \"expect\": {\"type\": \"string\"},\n                            \"phase\": {\"type\": \"string\", \"enum\": [\"check\", \"build\", \"test\", \"lint\", \"run\"]}\n                        },\n                        \"required\": [\"cmd\", \"expect\", \"phase\"]\n                    }\n                },\n                \"requirement_mapping\": {\n                    \"type\": \"array\",\n                    \"items\": {\n                        \"type\": \"object\",\n                        \"properties\": {\n                            \"req_id\": {\"type\": \"string\"},\n                            \"files\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}},\n                            \"note\": {\"type\": \"string\"}\n                        },\n                        \"required\": [\"req_id\", \"files\", \"note\"]\n                    }\n                }\n            },\n            \"required\": [\"target\", \"project\", \"changes\", \"cmds\", \"requirement_mapping\"]\n        });\n\n        // 提取关键信息 - 从实际的 artifact 结构中提取\n        // PRD: target (从 IdeaSpec), features (从 reqs)\n        let target = format!(\"{}; Goals: {}\", \n            &prd_artifact.data.scope.g.join(\", \"),\n            &prd_artifact.data.scope.ng.join(\", \")\n        );\n        \n        let features: Vec<String> = prd_artifact.data.reqs.iter()\n            .take(5)\n            .map(|r| format!(\"{}: {}\", r.id, r.desc))\n            .collect();\n        \n        let tech_requirements: Vec<String> = prd_artifact.data.cons.iter()\n            .map(|c| format!(\"{}: {}\", c.id, c.desc))\n            .collect();\n\n        // DesignDoc: cli, wf, arch, io\n        let architecture_layers = design_artifact.data.arch.layers.join(\", \");\n        let components = design_artifact.data.arch.comps.join(\", \");\n        let workflow_stages = design_artifact.data.wf.stages.join(\", \");\n\n        // 压缩任务信息，只保留关键内容\n        let task_summary: Vec<String> = plan_artifact.data.tasks.iter()\n            .take(5)  // 只取前5个任务\n            .map(|t| format!(\"{}: {}\", t.id, t.desc))\n            .collect();\n        \n        // ✅ 提取并强调 TodoList\n        let todo_context = if let Some(ref todo_list) = plan_artifact.data.todo_list {\n            let mut lines = vec![\"**TodoList (IMPORTANT - ensure all related files are generated):**\".to_string()];\n            for item in &todo_list.items {\n                lines.push(format!(\"- {}: {}\", item.id, item.description));\n                if !item.related_files.is_empty() {\n                    lines.push(format!(\"  Files to generate: {}\", item.related_files.join(\", \")));\n                }\n                if !item.related_requirements.is_empty() {\n                    lines.push(format!(\"  Requirements: {}\", item.related_requirements.join(\", \")));\n                }\n            }\n            lines.push(\"\".to_string());\n            lines.push(\"CRITICAL: Every file mentioned in TodoList must be included in the 'changes' array.\".to_string());\n            lines.push(\"\".to_string());\n            lines.join(\"\\n\")\n        } else {\n            String::new()\n        };\n\n        let context = format!(\n            r#\"Based on the user requirements, design decisions, and implementation plan, generate a code change plan.\n\n{}\n\n**User Requirements (from PRD):**\n- Target Scope: {}\n- Key Features:\n{}\n- Technical Constraints:\n{}\n\n**Design Decisions (from DesignDoc):**\n- CLI Modes: {}\n- Workflow Stages: {}\n- Architecture Layers: {}\n- Key Components: {}\n- Artifact Formats: {}\n\n**Project Analysis (current state):**\n{}\n\n**Implementation Plan Summary:**\n- C4 Context: {}\n- C4 Containers: {}\n- C4 Components: {}\n- Top 5 Tasks:\n{}\n\n**Milestones:**\n{}\n\n**CRITICAL RULES FOR LANGUAGE/TECH STACK DETECTION:**\n1. Analyze the requirements and design to infer the target technology\n2. If requirements mention \"web\", \"HTML\", \"browser\", \"frontend\" → generate .html, .css, .js files\n3. If requirements mention \"Python\", \"Flask\", \"Django\" → generate .py files\n4. If requirements mention \"Rust\", \"cargo\", or current project is Rust → generate .rs files\n5. If requirements mention \"Node\", \"JavaScript\", \"npm\" → generate .js/.ts and package.json\n6. DO NOT blindly copy the current project structure!\n7. Match the file types to what the user actually wants to build\n\nGenerate a comprehensive but concise code change plan.\"#,\n            todo_context,\n            target,\n            features.join(\"\\n  \"),\n            tech_requirements.join(\"\\n  \"),\n            design_artifact.data.cli.modes.join(\", \"),\n            workflow_stages,\n            architecture_layers,\n            components,\n            design_artifact.data.io.formats.join(\", \"),\n            project_context,\n            plan_artifact.data.c4.context.join(\", \"),\n            plan_artifact.data.c4.containers.join(\", \"),\n            plan_artifact.data.c4.components.join(\", \"),\n            task_summary.join(\"\\n  \"),\n            plan_artifact.data.milestones.iter()\n                .take(3)  // 只取前3个里程碑\n                .map(|m| format!(\"{}: {}\", m.id, m.desc))\n                .collect::<Vec<_>>()\n                .join(\"\\n  \"),\n        );\n\n        // 创建无工具的 agent（避免工具调用循环）\n        let agent = Arc::new(\n            LlmAgentBuilder::new(\"code_planner\")\n                .description(\"Generate code change plan based on requirements, design and analysis\")\n                .instruction(\n                    r#\"You are a code planning specialist. Based on the project analysis, user requirements, design decisions, and implementation plan, create a detailed code change plan WITH requirement mapping.\n\n**CRITICAL: Respect the target language in the Design document!**\n\nLanguage-specific file generation rules:\n- If Design says \"html\", \"web\", or \"frontend\" → generate .html, .css, .js files (NOT .rs files)\n- If Design says \"python\" → generate .py files (NOT .rs files)\n- If Design says \"rust\" → generate .rs files and Cargo.toml\n- If Design says \"javascript\" or \"node\" → generate .js files and package.json\n- If Design says \"typescript\" → generate .ts files and tsconfig.json\n\n**Output Requirements:**\n1. Respond with ONLY valid JSON (no markdown, no explanations, just the JSON object)\n2. File paths MUST match the target language specified in Design\n3. The \"lang\" field in output MUST match the Design language\n4. tooling.pkg MUST match: \"none\" for html, \"npm\" for js/ts, \"pip\" for python, \"cargo\" for rust\n5. Be specific about file paths based on language conventions\n6. **MUST include requirement_mapping** - map each requirement ID to implementing files\n\n**Requirement Mapping Guidelines:**\n1. For each requirement ID (REQ-001, REQ-002, etc.), list which files implement it\n2. Provide a brief note explaining how the files address the requirement\n3. One requirement can map to multiple files\n4. One file can implement multiple requirements\n5. Ensure ALL requirements from PRD are mapped\n\n**Example for HTML/Web project:**\n{\n  \"target\": {\n    \"lang\": \"html\",\n    \"stack\": [\"vanilla-js\", \"css3\"],\n    \"build\": [],\n    \"test\": []\n  },\n  \"project\": {\n    \"root\": \"./\",\n    \"layout\": \"single\",\n    \"modules\": [],\n    \"tooling\": {\n      \"pkg\": \"none\",\n      \"build\": [],\n      \"test\": [],\n      \"lint\": []\n    }\n  },\n  \"changes\": [\n    {\"path\": \"index.html\", \"kind\": \"create\", \"note\": \"Main HTML structure\"},\n    {\"path\": \"styles.css\", \"kind\": \"create\", \"note\": \"Styling\"},\n    {\"path\": \"script.js\", \"kind\": \"create\", \"note\": \"Interactivity\"}\n  ],\n  \"cmds\": [],\n  \"requirement_mapping\": [\n    {\n      \"req_id\": \"REQ-001\",\n      \"files\": [\"index.html\", \"styles.css\"],\n      \"note\": \"Semantic HTML structure and responsive design implement this requirement\"\n    },\n    {\n      \"req_id\": \"REQ-002\",\n      \"files\": [\"script.js\"],\n      \"note\": \"JavaScript handles interactivity for this requirement\"\n    }\n  ]\n}\n\nFollow the exact JSON schema provided in the context.\"#,\n                )\n                .model(self.model.clone())\n                .output_schema(output_schema)\n                .output_key(\"code_plan\")\n                .build()?,\n        );\n\n        let session_service = Arc::new(InMemorySessionService::new());\n        let app_name = \"cowork\".to_string();\n        let user_id = format!(\"{}_planning\", session_id);\n\n        let _session = session_service\n            .create(CreateRequest {\n                app_name: app_name.clone(),\n                user_id: user_id.clone(),\n                session_id: Some(format!(\"{}_phase2\", session_id)),\n                state: HashMap::new(),\n            })\n            .await?;\n\n        let runner = Runner::new(RunnerConfig {\n            app_name: app_name.clone(),\n            agent: agent.clone(),\n            session_service: session_service.clone(),\n            artifact_service: None,\n            memory_service: None,\n            run_config: None,\n        })?;\n\n        let input_content = Content::new(\"user\").with_text(&context);\n\n        tracing::info!(\"Generating code plan...\");\n\n        let mut event_stream = runner\n            .run(user_id.clone(), format!(\"{}_phase2\", session_id), input_content)\n            .await?;\n\n        while let Some(event_result) = event_stream.next().await {\n            match event_result {\n                Ok(_event) => {},\n                Err(e) => {\n                    tracing::error!(\"Error during code planning: {}\", e);\n                    return Err(anyhow::anyhow!(\"Code planning failed: {}\", e));\n                }\n            }\n        }\n\n        let updated_session = session_service\n            .get(GetRequest {\n                user_id: user_id.clone(),\n                session_id: format!(\"{}_phase2\", session_id),\n                app_name: app_name.clone(),\n                after: None,\n                num_recent_events: None,\n            })\n            .await?;\n\n        let state = updated_session.state();\n        let raw_output = state\n            .get(\"code_plan\")\n            .ok_or_else(|| anyhow::anyhow!(\"No output from Code planner\"))?;\n\n        // 增强的 JSON 解析：先做 schema 容错归一化，再反序列化\n        let code_change: CodeChange = {\n            // 1) 获取原始 JSON Value\n            let raw_value: serde_json::Value = match raw_output {\n                serde_json::Value::String(json_str) => {\n                    tracing::debug!(\"Parsing JSON string output\");\n                    serde_json::from_str(json_str.as_str()).map_err(|e| {\n                        tracing::error!(\"JSON parse error: {}\", e);\n                        tracing::error!(\n                            \"Raw JSON string (first 500 chars): {}\",\n                            &json_str.chars().take(500).collect::<String>()\n                        );\n                        anyhow::anyhow!(\n                            \"Failed to parse code plan JSON string: {}\\n\\\n                            This usually means the LLM didn't return valid JSON.\",\n                            e\n                        )\n                    })?\n                }\n                value => value.clone(),\n            };\n\n            // 2) 容错归一化（layout/modules 等常见偏差）\n            let normalized_value = CodePlanNormalizer::normalize(raw_value.clone());\n\n            // 3) 反序列化\n            serde_json::from_value(normalized_value.clone()).map_err(|e| {\n                tracing::error!(\"JSON parse error after normalization: {}\", e);\n                tracing::error!(\n                    \"Raw JSON value: {}\",\n                    serde_json::to_string_pretty(&raw_value)\n                        .unwrap_or_else(|_| \"unparseable\".to_string())\n                );\n                tracing::error!(\n                    \"Normalized JSON value: {}\",\n                    serde_json::to_string_pretty(&normalized_value)\n                        .unwrap_or_else(|_| \"unparseable\".to_string())\n                );\n                anyhow::anyhow!(\n                    \"Failed to parse code plan JSON: {}\\n\\\n                    This usually means the LLM didn't follow the schema correctly.\\n\\\n                    Common issues:\\n\\\n                    - project.layout must be one of: mono|single|unknown\\n\\\n                    - project.modules must be array of objects with name/path/type fields\\n\\\n                    Please check the logs for the raw/normalized JSON output.\",\n                    e\n                )\n            })?\n        };\n\n        tracing::info!(\"Successfully parsed CodeChange\");\n\n        Ok(code_change)\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 21.0,
      "lines_of_code": 581,
      "number_of_classes": 1,
      "number_of_functions": 4
    },
    "dependencies": [
      {
        "dependency_type": "error_handling",
        "is_external": true,
        "line_number": 1,
        "name": "anyhow",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "framework",
        "is_external": false,
        "line_number": 2,
        "name": "adk_rust::prelude",
        "path": "adk_rust",
        "version": null
      },
      {
        "dependency_type": "model",
        "is_external": false,
        "line_number": 3,
        "name": "adk_rust::model",
        "path": "adk_rust",
        "version": null
      },
      {
        "dependency_type": "execution",
        "is_external": false,
        "line_number": 4,
        "name": "adk_rust::runner",
        "path": "adk_rust",
        "version": null
      },
      {
        "dependency_type": "session_management",
        "is_external": false,
        "line_number": 5,
        "name": "adk_rust::session",
        "path": "adk_rust",
        "version": null
      },
      {
        "dependency_type": "async",
        "is_external": true,
        "line_number": 6,
        "name": "futures",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "data_structure",
        "is_external": false,
        "line_number": 7,
        "name": "std::collections::HashMap",
        "path": "std",
        "version": null
      },
      {
        "dependency_type": "concurrency",
        "is_external": false,
        "line_number": 8,
        "name": "std::sync::Arc",
        "path": "std",
        "version": null
      },
      {
        "dependency_type": "artifact_types",
        "is_external": false,
        "line_number": 10,
        "name": "crate::artifacts",
        "path": "crate",
        "version": null
      },
      {
        "dependency_type": "storage",
        "is_external": false,
        "line_number": 11,
        "name": "crate::memory::ArtifactStore",
        "path": "crate",
        "version": null
      },
      {
        "dependency_type": "configuration",
        "is_external": false,
        "line_number": 12,
        "name": "crate::config::LlmConfig",
        "path": "crate",
        "version": null
      },
      {
        "dependency_type": "tooling",
        "is_external": false,
        "line_number": 13,
        "name": "crate::tools::create_file_tools",
        "path": "crate",
        "version": null
      },
      {
        "dependency_type": "utility",
        "is_external": false,
        "line_number": 14,
        "name": "crate::agents::code_plan_normalizer::CodePlanNormalizer",
        "path": "crate",
        "version": null
      }
    ],
    "detailed_description": "CodePlanner is an intelligent agent component responsible for generating detailed code change plans. It operates in two distinct phases: Phase 1 analyzes the current project structure using file system tools to understand the existing codebase layout, and Phase 2 generates a comprehensive code change plan based on PRD (Product Requirements Document), design artifacts, and the project analysis results. The component uses LLM (Large Language Model) agents to process requirements and create structured code change artifacts with requirement mapping, ensuring that all implementation requirements are properly tracked to specific files.",
    "interfaces": [
      {
        "description": "Creates a new CodePlanner instance with LLM configuration and artifact store",
        "interface_type": "constructor",
        "name": "new",
        "parameters": [
          {
            "description": "Configuration for LLM client",
            "is_optional": false,
            "name": "llm_config",
            "param_type": "&LlmConfig"
          },
          {
            "description": "Artifact storage service",
            "is_optional": false,
            "name": "store",
            "param_type": "Arc<ArtifactStore>"
          }
        ],
        "return_type": "Result<Self>",
        "visibility": "public"
      },
      {
        "description": "Main execution method that orchestrates the two-phase planning process",
        "interface_type": "method",
        "name": "execute",
        "parameters": [
          {
            "description": "Session identifier",
            "is_optional": false,
            "name": "session_id",
            "param_type": "&str"
          },
          {
            "description": "Product requirements document artifact",
            "is_optional": false,
            "name": "prd_artifact",
            "param_type": "&PRDArtifact"
          },
          {
            "description": "Design document artifact",
            "is_optional": false,
            "name": "design_artifact",
            "param_type": "&DesignDocArtifact"
          },
          {
            "description": "Implementation plan artifact",
            "is_optional": false,
            "name": "plan_artifact",
            "param_type": "&PlanArtifact"
          }
        ],
        "return_type": "Result<CodeChangeArtifact>",
        "visibility": "public"
      },
      {
        "description": "Phase 1: Analyzes project structure using file system tools and LLM agent",
        "interface_type": "method",
        "name": "analyze_project_structure",
        "parameters": [
          {
            "description": "Session identifier",
            "is_optional": false,
            "name": "session_id",
            "param_type": "&str"
          }
        ],
        "return_type": "Result<String>",
        "visibility": "private"
      },
      {
        "description": "Phase 2: Generates code change plan using LLM agent with structured output schema",
        "interface_type": "method",
        "name": "generate_code_plan",
        "parameters": [
          {
            "description": "Session identifier",
            "is_optional": false,
            "name": "session_id",
            "param_type": "&str"
          },
          {
            "description": "Product requirements artifact",
            "is_optional": false,
            "name": "prd_artifact",
            "param_type": "&PRDArtifact"
          },
          {
            "description": "Design document artifact",
            "is_optional": false,
            "name": "design_artifact",
            "param_type": "&DesignDocArtifact"
          },
          {
            "description": "Implementation plan artifact",
            "is_optional": false,
            "name": "plan_artifact",
            "param_type": "&PlanArtifact"
          },
          {
            "description": "Project analysis results from phase 1",
            "is_optional": false,
            "name": "project_context",
            "param_type": "&str"
          }
        ],
        "return_type": "Result<CodeChange>",
        "visibility": "private"
      }
    ],
    "responsibilities": [
      "Project structure analysis using file system tools",
      "Code change plan generation based on requirements and design",
      "Requirement-to-file mapping for traceability",
      "Integration with LLM agents for intelligent planning",
      "Artifact management and storage"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": null,
      "file_path": "crates/cowork-core/src/agents/prd_agent.rs",
      "functions": [
        "new",
        "execute"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "PrdAgent",
        "LlmAgentBuilder",
        "OpenAIClient",
        "InMemorySessionService"
      ],
      "name": "prd_agent.rs",
      "source_summary": "use anyhow::Result;\nuse adk_rust::prelude::*;\nuse adk_rust::model::{OpenAIClient, OpenAIConfig};\nuse adk_rust::runner::{Runner, RunnerConfig};\nuse adk_rust::session::{InMemorySessionService, CreateRequest, SessionService, GetRequest};\nuse futures::StreamExt;\nuse std::collections::HashMap;\nuse std::sync::Arc;\n\nuse crate::artifacts::*;\nuse crate::memory::ArtifactStore;\nuse crate::config::LlmConfig;\n\n/// PRD Agent - 基于 IdeaSpec 生成产品需求文档\npub struct PrdAgent {\n    model: Arc<OpenAIClient>,\n    store: Arc<ArtifactStore>,\n}\n\nimpl PrdAgent {\n    pub fn new(llm_config: &LlmConfig, store: Arc<ArtifactStore>) -> Result<Self> {\n        let config = OpenAIConfig::compatible(\n            llm_config.api_key.clone(),\n            llm_config.api_base_url.clone(),\n            llm_config.model_name.clone(),\n        );\n        \n        tracing::info!(\"Creating PRD Agent with OpenAI-compatible client )\");\n        \n        let model = OpenAIClient::new(config)?;\n\n        Ok(Self {\n            model: Arc::new(model),\n            store,\n        })\n    }\n\n    pub async fn execute(&self, session_id: &str, idea_artifact: &IdeaSpecArtifact) -> Result<PRDArtifact> {\n        tracing::info!(\"PrdAgent: generating PRD for session {}\", session_id);\n\n        // Define output schema for PRD\n        let output_schema = serde_json::json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"scope\": {\n                    \"type\": \"object\",\n                    \"properties\": {\n                        \"g\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}},\n                        \"ng\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}\n                    },\n                    \"required\": [\"g\", \"ng\"]\n                },\n                \"reqs\": {\n                    \"type\": \"array\",\n                    \"items\": {\n                        \"type\": \"object\",\n                        \"properties\": {\n                            \"id\": {\"type\": \"string\"},\n                            \"pri\": {\"type\": \"string\", \"enum\": [\"p0\", \"p1\", \"p2\"]},\n                            \"type\": {\"type\": \"string\", \"enum\": [\"func\", \"nfr\", \"constraint\"]},\n                            \"desc\": {\"type\": \"string\"},\n                            \"deps\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}},\n                            \"ac\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}\n                        },\n                        \"required\": [\"id\", \"pri\", \"type\", \"desc\", \"deps\", \"ac\"]\n                    }\n                },\n                \"cons\": {\n                    \"type\": \"array\",\n                    \"items\": {\n                        \"type\": \"object\",\n                        \"properties\": {\n                            \"id\": {\"type\": \"string\"},\n                            \"desc\": {\"type\": \"string\"}\n                        },\n                        \"required\": [\"id\", \"desc\"]\n                    }\n                },\n                \"hitl\": {\n                    \"type\": \"array\",\n                    \"items\": {\n                        \"type\": \"object\",\n                        \"properties\": {\n                            \"id\": {\"type\": \"string\"},\n                            \"q\": {\"type\": \"string\"},\n                            \"opts\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}},\n                            \"def\": {\"type\": \"string\"}\n                        },\n                        \"required\": [\"id\", \"q\", \"opts\", \"def\"]\n                    }\n                }\n            },\n            \"required\": [\"scope\", \"reqs\", \"cons\", \"hitl\"]\n        });\n\n        // Build context from IdeaSpec\n        let context = format!(\n            r#\"Based on the following IDEA specification, create a detailed Product Requirements Document (PRD).\n\n**IDEA Background:**\n{}\n\n**Goals:**\n{}\n\n**Non-Goals:**\n{}\n\n**Constraints:**\n{}\n\n**Success Criteria:**\n{}\n\n**Risks:**\n{}\n\n**Questions:**\n{}\"#,\n            idea_artifact.data.bg,\n            idea_artifact.data.g.join(\"\\n- \"),\n            idea_artifact.data.ng.join(\"\\n- \"),\n            idea_artifact.data.c.join(\"\\n- \"),\n            idea_artifact.data.sc.join(\"\\n- \"),\n            idea_artifact.data.r.join(\"\\n- \"),\n            idea_artifact.data.q.join(\"\\n- \"),\n        );\n\n        let agent = Arc::new(\n            LlmAgentBuilder::new(\"prd_generator\")\n                .description(\"Generate Product Requirements Document from IdeaSpec\")\n                .instruction(\n                    r#\"You are a product manager. Create a structured PRD (Product Requirements Document) from the IDEA specification.\n\n**Required JSON Structure:**\n{\n  \"scope\": {\n    \"g\": [\"array of in-scope goals\"],\n    \"ng\": [\"array of out-of-scope items\"]\n  },\n  \"reqs\": [\n    {\n      \"id\": \"REQ-001\",\n      \"pri\": \"p0|p1|p2\",\n      \"type\": \"func|nfr|constraint\",\n      \"desc\": \"requirement description\",\n      \"deps\": [\"array of req IDs this depends on\"],\n      \"ac\": [\"array of acceptance criteria\"]\n    }\n  ],\n  \"cons\": [\n    {\n      \"id\": \"CON-001\",\n      \"desc\": \"constraint description\"\n    }\n  ],\n  \"hitl\": [\n    {\n      \"id\": \"HITL-001\",\n      \"q\": \"question needing human input\",\n      \"opts\": [\"option1\", \"option2\"],\n      \"def\": \"default option\"\n    }\n  ]\n}\n\n**Output Requirements:**\n1. Respond with ONLY valid JSON (no markdown, no code blocks)\n2. All arrays can be empty but must be present\n3. Use clear, actionable language\n4. Each requirement must have specific, testable acceptance criteria\n5. Priority p0 = critical, p1 = important, p2 = nice-to-have\n6. Include HITL questions for unclear decisions\n\nGenerate the PRD now based on the IDEA provided.\"#,\n                )\n                .model(self.model.clone())\n                .output_schema(output_schema)\n                .output_key(\"prd_raw\")\n                .build()?,\n        );\n\n        let session_service = Arc::new(InMemorySessionService::new());\n        let app_name = \"cowork\".to_string();\n        let user_id = session_id.to_string();\n\n        let _session = session_service\n            .create(CreateRequest {\n                app_name: app_name.clone(),\n                user_id: user_id.clone(),\n                session_id: Some(session_id.to_string()),\n                state: HashMap::new(),\n            })\n            .await?;\n\n        let runner = Runner::new(RunnerConfig {\n            app_name: app_name.clone(),\n            agent: agent.clone(),\n            session_service: session_service.clone(),\n            artifact_service: None,\n            memory_service: None,\n            run_config: None,\n        })?;\n\n        let input_content = Content::new(\"user\").with_text(&context);\n\n        tracing::info!(\"Invoking PRD generation agent...\");\n\n        let mut event_stream = runner\n            .run(user_id.clone(), session_id.to_string(), input_content)\n            .await?;\n\n        while let Some(event_result) = event_stream.next().await {\n            match event_result {\n                Ok(event) => {\n                    tracing::debug!(\"Event received: {:?}\", event);\n                }\n                Err(e) => {\n                    tracing::error!(\"Error during PRD generation: {}\", e);\n                    return Err(anyhow::anyhow!(\"PRD generation failed: {}\", e));\n                }\n            }\n        }\n\n        tracing::info!(\"PRD generation complete\");\n\n        let updated_session = session_service\n            .get(GetRequest {\n                user_id: user_id.clone(),\n                session_id: session_id.to_string(),\n                app_name: app_name.clone(),\n                after: None,\n                num_recent_events: None,\n            })\n            .await?;\n\n        let state = updated_session.state();\n        let raw_output = state\n            .get(\"prd_raw\")\n            .ok_or_else(|| anyhow::anyhow!(\"No output from PRD agent\"))?;\n\n        tracing::debug!(\"Raw PRD output: {}\", raw_output);\n\n        let prd: PRD = match raw_output {\n            serde_json::Value::String(json_str) => {\n                tracing::debug!(\"Output is a JSON string, parsing...\");\n                serde_json::from_str(json_str.as_str())\n                    .map_err(|e| anyhow::anyhow!(\"Failed to parse PRD JSON: {}\", e))?\n            }\n            value => {\n                tracing::debug!(\"Output is a structured JSON value\");\n                serde_json::from_value(value.clone())\n                    .map_err(|e| anyhow::anyhow!(\"Failed to deserialize PRD: {}\", e))?\n            }\n        };\n\n        tracing::info!(\"Successfully parsed PRD\");\n\n        let summary = vec![\n            format!(\"Scope: {} goals, {} non-goals\", prd.scope.g.len(), prd.scope.ng.len()),\n            format!(\"Requirements: {} total\", prd.reqs.len()),\n            format!(\"Constraints: {}\", prd.cons.len()),\n            format!(\"HITL Questions: {}\", prd.hitl.len()),\n        ];\n\n        let artifact = ArtifactEnvelope::new(session_id.to_string(), Stage::Requirements, prd)\n            .with_summary(summary)\n            .with_prev(vec![idea_artifact.meta.artifact_id.clone()]);\n\n        self.store.put(session_id, Stage::Requirements, &artifact)?;\n\n        tracing::info!(\"PRD artifact saved successfully\");\n\n        Ok(artifact)\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 7.0,
      "lines_of_code": 276,
      "number_of_classes": 1,
      "number_of_functions": 2
    },
    "dependencies": [
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "anyhow",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "adk_rust",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "futures",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "serde_json",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "tracing",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::artifacts",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::memory::ArtifactStore",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::config::LlmConfig",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "IdeaSpecArtifact",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "PRDArtifact",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "PRD",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "The PrdAgent is an intelligent agent that generates a structured Product Requirements Document (PRD) from an IdeaSpec artifact. It uses an OpenAI-compatible LLM to transform user-provided idea data into a formal PRD with defined scope, requirements, constraints, and human-in-the-loop (HITL) questions. The agent constructs a detailed context prompt from the IdeaSpec, invokes an LLM agent with a strict JSON output schema, captures the generated output via an in-memory session service, validates and parses the JSON response, and finally stores the resulting PRD artifact in the artifact store with metadata and lineage tracking. This component bridges high-level product ideas to structured, actionable requirements for downstream development teams.",
    "interfaces": [
      {
        "description": null,
        "interface_type": "struct",
        "name": "PrdAgent",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "LlmAgentBuilder",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "new",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "description",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "instruction",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "model",
            "param_type": "Arc<OpenAIClient>"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "output_schema",
            "param_type": "serde_json::Value"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "output_key",
            "param_type": "String"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "OpenAIClient",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "config",
            "param_type": "OpenAIConfig"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "InMemorySessionService",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Generate structured Product Requirements Document (PRD) from IdeaSpec input",
      "Enforce strict JSON output schema via LLM instruction and schema validation",
      "Manage LLM session lifecycle and capture output through session service",
      "Transform raw LLM output into typed PRD model and validate structure",
      "Persist generated PRD artifact with metadata and dependency lineage"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": null,
      "file_path": "crates/cowork-core/src/agents/design_agent.rs",
      "functions": [
        "new",
        "execute"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "DesignAgent",
        "PRDArtifact",
        "DesignDocArtifact",
        "DesignDoc"
      ],
      "name": "design_agent.rs",
      "source_summary": "use anyhow::Result;\nuse adk_rust::prelude::*;\nuse adk_rust::model::{OpenAIClient, OpenAIConfig};\nuse adk_rust::runner::{Runner, RunnerConfig};\nuse adk_rust::session::{InMemorySessionService, CreateRequest, SessionService, GetRequest};\nuse futures::StreamExt;\nuse std::collections::HashMap;\nuse std::sync::Arc;\n\nuse crate::artifacts::*;\nuse crate::memory::ArtifactStore;\nuse crate::config::LlmConfig;\n\n/// Design Agent - 基于 PRD 生成技术设计文档\npub struct DesignAgent {\n    model: Arc<OpenAIClient>,\n    store: Arc<ArtifactStore>,\n}\n\nimpl DesignAgent {\n    pub fn new(llm_config: &LlmConfig, store: Arc<ArtifactStore>) -> Result<Self> {\n        let config = OpenAIConfig::compatible(\n            llm_config.api_key.clone(),\n            llm_config.api_base_url.clone(),\n            llm_config.model_name.clone(),\n        );\n        \n        tracing::info!(\"Creating Design Agent with OpenAI-compatible client )\");\n        \n        let model = OpenAIClient::new(config)?;\n\n        Ok(Self {\n            model: Arc::new(model),\n            store,\n        })\n    }\n\n    pub async fn execute(&self, session_id: &str, prd_artifact: &PRDArtifact) -> Result<DesignDocArtifact> {\n        tracing::info!(\"DesignAgent: generating design document for session {}\", session_id);\n\n        let output_schema = serde_json::json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"cli\": {\n                    \"type\": \"object\",\n                    \"properties\": {\n                        \"modes\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}},\n                        \"hitl_flow\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}\n                    },\n                    \"required\": [\"modes\", \"hitl_flow\"]\n                },\n                \"wf\": {\n                    \"type\": \"object\",\n                    \"properties\": {\n                        \"stages\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}},\n                        \"transitions\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}\n                    },\n                    \"required\": [\"stages\", \"transitions\"]\n                },\n                \"arch\": {\n                    \"type\": \"object\",\n                    \"properties\": {\n                        \"layers\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}},\n                        \"comps\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}\n                    },\n                    \"required\": [\"layers\", \"comps\"]\n                },\n                \"io\": {\n                    \"type\": \"object\",\n                    \"properties\": {\n                        \"artifact_dir\": {\"type\": \"string\"},\n                        \"formats\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}\n                    },\n                    \"required\": [\"artifact_dir\", \"formats\"]\n                }\n            },\n            \"required\": [\"cli\", \"wf\", \"arch\", \"io\"]\n        });\n\n        // Build context from PRD\n        let req_summary: Vec<String> = prd_artifact.data.reqs.iter()\n            .map(|r| format!(\"{} [{}]: {}\", r.id, r.pri as u8, r.desc))\n            .collect();\n\n        let context = format!(\n            r#\"Based on the following PRD, create a technical design document.\n\n**Scope:**\nIn-scope goals: {}\nOut-of-scope: {}\n\n**Requirements:**\n{}\n\n**Constraints:**\n{}\n\nCreate a design that addresses all functional and non-functional requirements.\"#,\n            prd_artifact.data.scope.g.join(\", \"),\n            prd_artifact.data.scope.ng.join(\", \"),\n            req_summary.join(\"\\n\"),\n            prd_artifact.data.cons.iter().map(|c| c.desc.as_str()).collect::<Vec<_>>().join(\"\\n\"),\n        );\n\n        let agent = Arc::new(\n            LlmAgentBuilder::new(\"design_generator\")\n                .description(\"Generate technical design document from PRD\")\n                .instruction(\n                    r#\"You are a technical architect. Create a structured design document.\n\n**Required JSON Structure:**\n{\n  \"cli\": {\n    \"modes\": [\"interactive\", \"batch\", \"server\"],\n    \"hitl_flow\": [\"description of human-in-the-loop interaction points\"]\n  },\n  \"wf\": {\n    \"stages\": [\"stage1\", \"stage2\", ...],\n    \"transitions\": [\"stage1 -> stage2: condition\", ...]\n  },\n  \"arch\": {\n    \"layers\": [\"presentation\", \"business\", \"data\", ...],\n    \"comps\": [\"component descriptions\"]\n  },\n  \"io\": {\n    \"artifact_dir\": \"./.output\",\n    \"formats\": [\"json\", \"markdown\", ...]\n  }\n}\n\n**Output Requirements:**\n1. Respond with ONLY valid JSON\n2. All arrays must be present (can be empty)\n3. Design should be practical and implementable\n4. Consider scalability and maintainability\"#,\n                )\n                .model(self.model.clone())\n                .output_schema(output_schema)\n                .output_key(\"design_raw\")\n                .build()?,\n        );\n\n        let session_service = Arc::new(InMemorySessionService::new());\n        let app_name = \"cowork\".to_string();\n        let user_id = session_id.to_string();\n\n        let _session = session_service\n            .create(CreateRequest {\n                app_name: app_name.clone(),\n                user_id: user_id.clone(),\n                session_id: Some(session_id.to_string()),\n                state: HashMap::new(),\n            })\n            .await?;\n\n        let runner = Runner::new(RunnerConfig {\n            app_name: app_name.clone(),\n            agent: agent.clone(),\n            session_service: session_service.clone(),\n            artifact_service: None,\n            memory_service: None,\n            run_config: None,\n        })?;\n\n        let input_content = Content::new(\"user\").with_text(&context);\n\n        tracing::info!(\"Invoking Design generation agent...\");\n\n        let mut event_stream = runner\n            .run(user_id.clone(), session_id.to_string(), input_content)\n            .await?;\n\n        while let Some(event_result) = event_stream.next().await {\n            match event_result {\n                Ok(_event) => {},\n                Err(e) => {\n                    tracing::error!(\"Error during design generation: {}\", e);\n                    return Err(anyhow::anyhow!(\"Design generation failed: {}\", e));\n                }\n            }\n        }\n\n        tracing::info!(\"Design generation complete\");\n\n        let updated_session = session_service\n            .get(GetRequest {\n                user_id: user_id.clone(),\n                session_id: session_id.to_string(),\n                app_name: app_name.clone(),\n                after: None,\n                num_recent_events: None,\n            })\n            .await?;\n\n        let state = updated_session.state();\n        let raw_output = state\n            .get(\"design_raw\")\n            .ok_or_else(|| anyhow::anyhow!(\"No output from Design agent\"))?;\n\n        let design: DesignDoc = match raw_output {\n            serde_json::Value::String(json_str) => {\n                serde_json::from_str(json_str.as_str())?\n            }\n            value => {\n                serde_json::from_value(value.clone())?\n            }\n        };\n\n        tracing::info!(\"Successfully parsed DesignDoc\");\n\n        let summary = vec![\n            format!(\"CLI modes: {}\", design.cli.modes.len()),\n            format!(\"Workflow stages: {}\", design.wf.stages.len()),\n            format!(\"Architecture components: {}\", design.arch.comps.len()),\n            format!(\"Output formats: {}\", design.io.formats.join(\", \")),\n        ];\n\n        let artifact = ArtifactEnvelope::new(session_id.to_string(), Stage::Design, design)\n            .with_summary(summary)\n            .with_prev(vec![prd_artifact.meta.artifact_id.clone()]);\n\n        self.store.put(session_id, Stage::Design, &artifact)?;\n\n        tracing::info!(\"Design artifact saved successfully\");\n\n        Ok(artifact)\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 5.0,
      "lines_of_code": 228,
      "number_of_classes": 1,
      "number_of_functions": 2
    },
    "dependencies": [
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "anyhow",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "adk_rust",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "futures",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "serde_json",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "tracing",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::artifacts",
        "path": "crates/cowork-core/src/artifacts/mod.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::memory::ArtifactStore",
        "path": "crates/cowork-core/src/memory/artifact_store.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::config::LlmConfig",
        "path": "crates/cowork-core/src/config/llm_config.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "Content",
        "path": "crates/cowork-core/src/model/content.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "Stage",
        "path": "crates/cowork-core/src/model/stage.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "ArtifactEnvelope",
        "path": "crates/cowork-core/src/model/artifact_envelope.rs",
        "version": null
      }
    ],
    "detailed_description": "The DesignAgent is an intelligent agent responsible for generating technical design documents from Product Requirement Documents (PRDs). It leverages an OpenAI LLM client to process PRD inputs and produce a structured JSON output covering CLI modes, workflow stages, architecture layers/components, and I/O specifications. The agent constructs a context string from PRD data, invokes an LLM agent with strict output schema constraints, manages a session via InMemorySessionService, and persists the generated design as an ArtifactEnvelope in the ArtifactStore. The execution flow includes context building, LLM invocation, event streaming, result extraction, validation, and artifact storage.",
    "interfaces": [
      {
        "description": null,
        "interface_type": "struct",
        "name": "DesignAgent",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "model",
            "param_type": "Arc<OpenAIClient>"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "store",
            "param_type": "Arc<ArtifactStore>"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "PRDArtifact",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "data",
            "param_type": "PRDData"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "meta",
            "param_type": "ArtifactMeta"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "DesignDocArtifact",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "data",
            "param_type": "DesignDoc"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "meta",
            "param_type": "ArtifactMeta"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "DesignDoc",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "cli",
            "param_type": "CLIDesign"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "wf",
            "param_type": "WorkflowDesign"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "arch",
            "param_type": "ArchitectureDesign"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "io",
            "param_type": "IODesign"
          }
        ],
        "return_type": null,
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Generate structured technical design documents from PRD inputs",
      "Manage LLM agent lifecycle and prompt engineering with strict JSON schema enforcement",
      "Coordinate session state management and artifact persistence",
      "Validate and transform raw LLM output into strongly-typed DesignDoc structure",
      "Integrate with system-wide artifact storage and versioning system"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": null,
      "file_path": "crates/cowork-core/src/agents/delivery_agent.rs",
      "functions": [
        "DeliveryAgent::new",
        "DeliveryAgent::execute"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "ArtifactStore",
        "CheckReportArtifact",
        "IdeaSpecArtifact",
        "DeliveryReportArtifact",
        "ArtifactEnvelope",
        "Stage",
        "DeliveryReport"
      ],
      "name": "delivery_agent.rs",
      "source_summary": "use anyhow::Result;\nuse std::sync::Arc;\n\nuse crate::artifacts::*;\nuse crate::memory::ArtifactStore;\nuse crate::config::LlmConfig;\n\n/// Delivery Agent - 生成最终交付报告\npub struct DeliveryAgent {\n    store: Arc<ArtifactStore>,\n}\n\nimpl DeliveryAgent {\n    pub fn new(_llm_config: &LlmConfig, store: Arc<ArtifactStore>) -> Result<Self> {\n        tracing::info!(\"Creating Delivery Agent\");\n        \n        Ok(Self {\n            store,\n        })\n    }\n\n    pub async fn execute(\n        &self,\n        session_id: &str,\n        check_artifact: &CheckReportArtifact,\n        _idea_artifact: &IdeaSpecArtifact,\n    ) -> Result<DeliveryReportArtifact> {\n        tracing::info!(\"DeliveryAgent: generating delivery report for session {}\", session_id);\n\n        // TODO: Implement comprehensive delivery report generation\n        // For now, create a placeholder report\n        \n        let delivery_report = DeliveryReport {\n            cap: vec![\n                \"Core functionality implemented\".to_string(),\n                \"Basic error handling in place\".to_string(),\n            ],\n            howto: vec![\n                \"Run: cargo run\".to_string(),\n                \"Build: cargo build --release\".to_string(),\n            ],\n            limits: vec![\n                \"Full workflow not yet complete\".to_string(),\n                \"Limited test coverage\".to_string(),\n            ],\n            acceptance: vec![\n                format!(\"Checks run: {}\", check_artifact.data.checks.len()),\n                format!(\"Issues found: {}\", check_artifact.data.issues.len()),\n            ],\n        };\n\n        let summary = vec![\n            format!(\"Capabilities: {}\", delivery_report.cap.len()),\n            format!(\"Usage steps: {}\", delivery_report.howto.len()),\n            format!(\"Known limits: {}\", delivery_report.limits.len()),\n        ];\n\n        let artifact = ArtifactEnvelope::new(session_id.to_string(), Stage::Delivery, delivery_report)\n            .with_summary(summary)\n            .with_prev(vec![check_artifact.meta.artifact_id.clone()]);\n\n        self.store.put(session_id, Stage::Delivery, &artifact)?;\n\n        tracing::info!(\"Delivery report saved successfully\");\n\n        Ok(artifact)\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 2.0,
      "lines_of_code": 68,
      "number_of_classes": 1,
      "number_of_functions": 2
    },
    "dependencies": [
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "anyhow",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "std",
        "is_external": true,
        "line_number": null,
        "name": "std",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::artifacts",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::memory::ArtifactStore",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::config::LlmConfig",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "The DeliveryAgent is an intelligent agent responsible for generating and persisting a final delivery report based on analysis artifacts from previous stages. It receives a session ID and two artifacts (CheckReportArtifact and IdeaSpecArtifact), constructs a structured delivery report containing capabilities, usage instructions, known limitations, and acceptance criteria, then saves this report to the ArtifactStore. The agent currently implements a placeholder report generation logic with hardcoded values and basic data aggregation from the CheckReportArtifact, but the core functionality is not yet fully implemented (marked by TODO). The agent operates asynchronously and uses tracing for logging.",
    "interfaces": [
      {
        "description": null,
        "interface_type": "struct",
        "name": "DeliveryAgent",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "_llm_config",
            "param_type": "&LlmConfig"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "store",
            "param_type": "Arc<ArtifactStore>"
          }
        ],
        "return_type": "Result<Self>",
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "function",
        "name": "execute",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "session_id",
            "param_type": "&str"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "check_artifact",
            "param_type": "&CheckReportArtifact"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "_idea_artifact",
            "param_type": "&IdeaSpecArtifact"
          }
        ],
        "return_type": "Result<DeliveryReportArtifact>",
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Generate a structured delivery report summarizing system capabilities, usage instructions, limitations, and acceptance criteria",
      "Aggregate data from CheckReportArtifact to populate acceptance criteria in the delivery report",
      "Persist the generated delivery report to the ArtifactStore using the session context",
      "Log execution lifecycle events using tracing for observability",
      "Provide a clean, injectable interface for downstream components to request delivery report generation"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": "Intelligent agent responsible for managing TodoList state and tracking progress",
      "file_path": "crates/cowork-core/src/agents/todo_manager.rs",
      "functions": [
        "update_from_execution",
        "verify_from_check",
        "generate_status_report",
        "print_status"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "TodoListManager",
        "TodoStatusReport"
      ],
      "name": "todo_manager.rs",
      "source_summary": "use crate::artifacts::*;\n\n/// TodoList 管理器 - 负责更新和追踪 TodoList 状态\npub struct TodoListManager;\n\nimpl TodoListManager {\n    /// 根据执行结果更新 TodoList 状态\n    pub fn update_from_execution(\n        todo_list: &mut TodoList,\n        _changes: &[Change],\n        successful_files: &[String],\n        failed_files: &[String],\n    ) {\n        for todo_item in &mut todo_list.items {\n            // 检查这个 Todo 相关的文件是否都已成功生成\n            let all_files_successful = todo_item.related_files.iter()\n                .all(|file| successful_files.contains(file));\n            \n            let some_files_failed = todo_item.related_files.iter()\n                .any(|file| failed_files.contains(file));\n            \n            // 根据文件生成情况更新状态\n            if some_files_failed {\n                todo_item.status = TodoStatus::Blocked {\n                    reason: format!(\"Some related files failed to generate: {:?}\", \n                        todo_item.related_files.iter()\n                            .filter(|f| failed_files.contains(f))\n                            .collect::<Vec<_>>())\n                };\n            } else if all_files_successful && !todo_item.related_files.is_empty() {\n                // 所有相关文件都成功生成\n                match &todo_item.status {\n                    TodoStatus::Pending | TodoStatus::InProgress => {\n                        todo_item.status = TodoStatus::Completed;\n                    }\n                    _ => {}  // 保持现有状态\n                }\n            } else if todo_item.related_files.iter().any(|file| successful_files.contains(file)) {\n                // 部分文件生成成功\n                match &todo_item.status {\n                    TodoStatus::Pending => {\n                        todo_item.status = TodoStatus::InProgress;\n                    }\n                    _ => {}\n                }\n            }\n        }\n    }\n    \n    /// 从 CheckReport 验证 TodoList 完成度\n    pub fn verify_from_check(\n        todo_list: &mut TodoList,\n        check_report: &CheckReport,\n    ) {\n        // 构建失败文件列表\n        let failed_files: Vec<String> = check_report.issues.iter()\n            .filter(|issue| issue.sev == \"error\")\n            .filter_map(|issue| {\n                // 从 issue.id 提取文件路径\n                if issue.id.starts_with(\"ISSUE-FILE-\") {\n                    Some(issue.id.strip_prefix(\"ISSUE-FILE-\").unwrap_or(\"\").to_string())\n                } else if issue.id.starts_with(\"ISSUE-EMPTY-\") {\n                    Some(issue.id.strip_prefix(\"ISSUE-EMPTY-\").unwrap_or(\"\").to_string())\n                } else {\n                    None\n                }\n            })\n            .collect();\n        \n        for todo_item in &mut todo_list.items {\n            // 如果相关文件有验证失败，标记为 Blocked\n            let has_failed_files = todo_item.related_files.iter()\n                .any(|file| failed_files.contains(file));\n            \n            if has_failed_files {\n                todo_item.status = TodoStatus::Blocked {\n                    reason: format!(\"Verification failed for: {:?}\",\n                        todo_item.related_files.iter()\n                            .filter(|f| failed_files.contains(f))\n                            .collect::<Vec<_>>())\n                };\n            }\n        }\n    }\n    \n    /// 生成 TodoList 状态报告\n    pub fn generate_status_report(todo_list: &TodoList) -> TodoStatusReport {\n        let mut total = 0;\n        let mut pending = 0;\n        let mut in_progress = 0;\n        let mut completed = 0;\n        let mut blocked = 0;\n        \n        for item in &todo_list.items {\n            total += 1;\n            match &item.status {\n                TodoStatus::Pending => pending += 1,\n                TodoStatus::InProgress => in_progress += 1,\n                TodoStatus::Completed => completed += 1,\n                TodoStatus::Blocked { .. } => blocked += 1,\n            }\n        }\n        \n        let completion_percentage = if total > 0 {\n            (completed as f64 / total as f64) * 100.0\n        } else {\n            0.0\n        };\n        \n        TodoStatusReport {\n            total,\n            pending,\n            in_progress,\n            completed,\n            blocked,\n            completion_percentage,\n        }\n    }\n    \n    /// 打印 TodoList 状态\n    pub fn print_status(todo_list: &TodoList) {\n        let report = Self::generate_status_report(todo_list);\n        \n        println!(\"\\n╔═══════════════════════════════════════╗\");\n        println!(\"║   TodoList 状态                       ║\");\n        println!(\"╚═══════════════════════════════════════╝\");\n        println!(\"总任务数: {}\", report.total);\n        println!(\"✅ 已完成: {}\", report.completed);\n        println!(\"🔄 进行中: {}\", report.in_progress);\n        println!(\"⏳ 待开始: {}\", report.pending);\n        println!(\"🚫 阻塞: {}\", report.blocked);\n        println!(\"完成度: {:.1}%\", report.completion_percentage);\n        println!();\n        \n        // 显示阻塞的任务\n        if report.blocked > 0 {\n            println!(\"⚠️  阻塞的任务:\");\n            for item in &todo_list.items {\n                if let TodoStatus::Blocked { reason } = &item.status {\n                    println!(\"  - {}: {}\", item.id, item.description);\n                    println!(\"    原因: {}\", reason);\n                }\n            }\n            println!();\n        }\n        \n        // 显示已完成的任务\n        if report.completed > 0 {\n            println!(\"✅ 已完成的任务:\");\n            for item in &todo_list.items {\n                if matches!(item.status, TodoStatus::Completed) {\n                    println!(\"  - {}: {}\", item.id, item.description);\n                }\n            }\n            println!();\n        }\n    }\n}\n\n/// TodoList 状态报告\n#[derive(Debug, Clone)]\npub struct TodoStatusReport {\n    pub total: usize,\n    pub pending: usize,\n    pub in_progress: usize,\n    pub completed: usize,\n    pub blocked: usize,\n    pub completion_percentage: f64,\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn test_update_from_execution() {\n        let mut todo_list = TodoList {\n            items: vec![\n                TodoItem {\n                    id: \"TODO-001\".to_string(),\n                    description: \"Implement login\".to_string(),\n                    status: TodoStatus::Pending,\n                    related_requirements: vec![\"REQ-001\".to_string()],\n                    related_files: vec![\"login.rs\".to_string(), \"session.rs\".to_string()],\n                    verification_method: \"unit_test\".to_string(),\n                },\n            ],\n        };\n        \n        let successful_files = vec![\"login.rs\".to_string(), \"session.rs\".to_string()];\n        let failed_files = vec![];\n        \n        TodoListManager::update_from_execution(\n            &mut todo_list,\n            &[],\n            &successful_files,\n            &failed_files,\n        );\n        \n        assert!(matches!(todo_list.items[0].status, TodoStatus::Completed));\n    }\n    \n    #[test]\n    fn test_status_report() {\n        let todo_list = TodoList {\n            items: vec![\n                TodoItem {\n                    id: \"TODO-001\".to_string(),\n                    description: \"Task 1\".to_string(),\n                    status: TodoStatus::Completed,\n                    related_requirements: vec![],\n                    related_files: vec![],\n                    verification_method: \"test\".to_string(),\n                },\n                TodoItem {\n                    id: \"TODO-002\".to_string(),\n                    description: \"Task 2\".to_string(),\n                    status: TodoStatus::InProgress,\n                    related_requirements: vec![],\n                    related_files: vec![],\n                    verification_method: \"test\".to_string(),\n                },\n                TodoItem {\n                    id: \"TODO-003\".to_string(),\n                    description: \"Task 3\".to_string(),\n                    status: TodoStatus::Pending,\n                    related_requirements: vec![],\n                    related_files: vec![],\n                    verification_method: \"test\".to_string(),\n                },\n            ],\n        };\n        \n        let report = TodoListManager::generate_status_report(&todo_list);\n        \n        assert_eq!(report.total, 3);\n        assert_eq!(report.completed, 1);\n        assert_eq!(report.in_progress, 1);\n        assert_eq!(report.pending, 1);\n        assert!((report.completion_percentage - 33.33333333333333).abs() < 1e-9);\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 20.0,
      "lines_of_code": 242,
      "number_of_classes": 2,
      "number_of_functions": 4
    },
    "dependencies": [
      {
        "dependency_type": "struct",
        "is_external": false,
        "line_number": null,
        "name": "TodoList",
        "path": "crate::artifacts",
        "version": null
      },
      {
        "dependency_type": "struct",
        "is_external": false,
        "line_number": null,
        "name": "TodoItem",
        "path": "crate::artifacts",
        "version": null
      },
      {
        "dependency_type": "enum",
        "is_external": false,
        "line_number": null,
        "name": "TodoStatus",
        "path": "crate::artifacts",
        "version": null
      },
      {
        "dependency_type": "struct",
        "is_external": false,
        "line_number": null,
        "name": "Change",
        "path": "crate::artifacts",
        "version": null
      },
      {
        "dependency_type": "struct",
        "is_external": false,
        "line_number": null,
        "name": "CheckReport",
        "path": "crate::artifacts",
        "version": null
      }
    ],
    "detailed_description": "The TodoListManager is an intelligent agent component that manages the lifecycle and state transitions of TodoList items. It handles state updates based on file execution results, verification outcomes, and provides comprehensive status reporting capabilities. The component tracks todo item progress through different states (Pending, InProgress, Completed, Blocked) and calculates completion metrics.",
    "interfaces": [
      {
        "description": "Updates TodoList status based on file execution results",
        "interface_type": "function",
        "name": "update_from_execution",
        "parameters": [
          {
            "description": "Mutable reference to TodoList to update",
            "is_optional": false,
            "name": "todo_list",
            "param_type": "&mut TodoList"
          },
          {
            "description": "Array of changes (currently unused)",
            "is_optional": false,
            "name": "_changes",
            "param_type": "&[Change]"
          },
          {
            "description": "Array of successfully generated file paths",
            "is_optional": false,
            "name": "successful_files",
            "param_type": "&[String]"
          },
          {
            "description": "Array of failed file paths",
            "is_optional": false,
            "name": "failed_files",
            "param_type": "&[String]"
          }
        ],
        "return_type": "()",
        "visibility": "public"
      },
      {
        "description": "Validates TodoList completion based on check report issues",
        "interface_type": "function",
        "name": "verify_from_check",
        "parameters": [
          {
            "description": "Mutable reference to TodoList to update",
            "is_optional": false,
            "name": "todo_list",
            "param_type": "&mut TodoList"
          },
          {
            "description": "Reference to check report containing validation issues",
            "is_optional": false,
            "name": "check_report",
            "param_type": "&CheckReport"
          }
        ],
        "return_type": "()",
        "visibility": "public"
      },
      {
        "description": "Generates statistical report of TodoList status distribution",
        "interface_type": "function",
        "name": "generate_status_report",
        "parameters": [
          {
            "description": "Reference to TodoList to analyze",
            "is_optional": false,
            "name": "todo_list",
            "param_type": "&TodoList"
          }
        ],
        "return_type": "TodoStatusReport",
        "visibility": "public"
      },
      {
        "description": "Prints formatted TodoList status to console",
        "interface_type": "function",
        "name": "print_status",
        "parameters": [
          {
            "description": "Reference to TodoList to display",
            "is_optional": false,
            "name": "todo_list",
            "param_type": "&TodoList"
          }
        ],
        "return_type": "()",
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Manage TodoList state transitions based on file execution results",
      "Validate TodoList completion through check report analysis",
      "Generate comprehensive status reports with completion metrics",
      "Provide formatted console output for TodoList status visualization",
      "Handle blocked state detection and reporting with detailed failure reasons"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "context",
      "description": null,
      "file_path": "crates/cowork-core/src/agents/batch_context.rs",
      "functions": [
        "FileContext",
        "BatchContext::new",
        "BatchContext::add_file",
        "BatchContext::generate_summary",
        "FileSummaryGenerator::generate",
        "FileSummaryGenerator::generate_rust_context",
        "FileSummaryGenerator::generate_python_context",
        "FileSummaryGenerator::generate_js_context",
        "FileSummaryGenerator::generate_html_context",
        "FileSummaryGenerator::generate_generic_context"
      ],
      "importance_score": 0.8,
      "interfaces": [],
      "name": "batch_context.rs",
      "source_summary": "\n/// 文件上下文摘要 - 用于批次间传递\n#[derive(Debug, Clone)]\npub struct FileContext {\n    /// 文件路径\n    pub path: String,\n    \n    /// 文件摘要描述\n    pub summary: String,\n    \n    /// 导出的符号/函数/类型\n    pub exports: Vec<String>,\n    \n    /// 导入的依赖\n    pub imports: Vec<String>,\n    \n    /// 关键类型定义\n    pub key_types: Vec<String>,\n}\n\n/// 批次上下文 - 包含已完成文件的详细信息\n#[derive(Debug, Clone)]\npub struct BatchContext {\n    /// 已完成的文件上下文\n    pub completed_files: Vec<FileContext>,\n}\n\nimpl BatchContext {\n    pub fn new() -> Self {\n        Self {\n            completed_files: Vec::new(),\n        }\n    }\n    \n    /// 添加文件上下文\n    pub fn add_file(&mut self, context: FileContext) {\n        self.completed_files.push(context);\n    }\n    \n    /// 生成简洁的上下文摘要（用于嵌入 instruction）\n    pub fn generate_summary(&self) -> String {\n        if self.completed_files.is_empty() {\n            return String::new();\n        }\n        \n        let mut lines = vec![\n            \"**📚 Previously Generated Files:**\".to_string(),\n            \"\".to_string(),\n        ];\n        \n        for file in &self.completed_files {\n            lines.push(format!(\"### {}\", file.path));\n            lines.push(format!(\"- Summary: {}\", file.summary));\n            \n            if !file.exports.is_empty() {\n                lines.push(format!(\"- Exports: {}\", file.exports.join(\", \")));\n            }\n            \n            if !file.key_types.is_empty() {\n                lines.push(format!(\"- Key Types: {}\", file.key_types.join(\", \")));\n            }\n            \n            lines.push(\"\".to_string());\n        }\n        \n        lines.push(\"**IMPORTANT**: Ensure consistency with these files (naming, types, imports).\".to_string());\n        lines.push(\"\".to_string());\n        \n        lines.join(\"\\n\")\n    }\n}\n\n/// 文件摘要生成器\npub struct FileSummaryGenerator;\n\nimpl FileSummaryGenerator {\n    /// 从文件内容生成上下文摘要\n    pub fn generate(path: &str, content: &str, lang: &str) -> FileContext {\n        match lang {\n            \"rust\" => Self::generate_rust_context(path, content),\n            \"python\" => Self::generate_python_context(path, content),\n            \"javascript\" | \"typescript\" => Self::generate_js_context(path, content),\n            \"html\" => Self::generate_html_context(path, content),\n            _ => Self::generate_generic_context(path, content),\n        }\n    }\n    \n    /// Rust 文件摘要\n    fn generate_rust_context(path: &str, content: &str) -> FileContext {\n        let mut exports = Vec::new();\n        let mut imports = Vec::new();\n        let mut key_types = Vec::new();\n        \n        // 提取 pub struct/enum/fn\n        for line in content.lines() {\n            let trimmed = line.trim();\n            \n            // pub struct Xxx\n            if trimmed.starts_with(\"pub struct \") {\n                if let Some(name) = trimmed.strip_prefix(\"pub struct \").and_then(|s| s.split_whitespace().next()) {\n                    exports.push(name.trim_end_matches('{').trim().to_string());\n                    key_types.push(format!(\"struct {}\", name.trim_end_matches('{').trim()));\n                }\n            }\n            \n            // pub enum Xxx\n            if trimmed.starts_with(\"pub enum \") {\n                if let Some(name) = trimmed.strip_prefix(\"pub enum \").and_then(|s| s.split_whitespace().next()) {\n                    exports.push(name.trim_end_matches('{').trim().to_string());\n                    key_types.push(format!(\"enum {}\", name.trim_end_matches('{').trim()));\n                }\n            }\n            \n            // pub fn xxx\n            if trimmed.starts_with(\"pub fn \") {\n                if let Some(name) = trimmed.strip_prefix(\"pub fn \").and_then(|s| s.split('(').next()) {\n                    exports.push(format!(\"{}()\", name.trim()));\n                }\n            }\n            \n            // use xxx;\n            if trimmed.starts_with(\"use \") && trimmed.ends_with(';') {\n                if let Some(import) = trimmed.strip_prefix(\"use \").and_then(|s| s.strip_suffix(';')) {\n                    imports.push(import.trim().to_string());\n                }\n            }\n        }\n        \n        let summary = if !exports.is_empty() {\n            format!(\"Rust module with {} public items\", exports.len())\n        } else {\n            \"Rust source file\".to_string()\n        };\n        \n        FileContext {\n            path: path.to_string(),\n            summary,\n            exports,\n            imports,\n            key_types,\n        }\n    }\n    \n    /// Python 文件摘要\n    fn generate_python_context(path: &str, content: &str) -> FileContext {\n        let mut exports = Vec::new();\n        let mut imports = Vec::new();\n        let mut key_types = Vec::new();\n        \n        for line in content.lines() {\n            let trimmed = line.trim();\n            \n            // class Xxx:\n            if trimmed.starts_with(\"class \") {\n                if let Some(name) = trimmed.strip_prefix(\"class \").and_then(|s| s.split(':').next()) {\n                    let class_name = name.split('(').next().unwrap_or(name).trim().to_string();\n                    exports.push(class_name.clone());\n                    key_types.push(format!(\"class {}\", class_name));\n                }\n            }\n            \n            // def xxx():\n            if trimmed.starts_with(\"def \") && !trimmed.starts_with(\"def _\") {\n                if let Some(name) = trimmed.strip_prefix(\"def \").and_then(|s| s.split('(').next()) {\n                    exports.push(format!(\"{}()\", name.trim()));\n                }\n            }\n            \n            // import/from xxx import\n            if trimmed.starts_with(\"import \") || trimmed.starts_with(\"from \") {\n                imports.push(trimmed.to_string());\n            }\n        }\n        \n        let summary = format!(\"Python module with {} exports\", exports.len());\n        \n        FileContext {\n            path: path.to_string(),\n            summary,\n            exports,\n            imports,\n            key_types,\n        }\n    }\n    \n    /// JavaScript/TypeScript 文件摘要\n    fn generate_js_context(path: &str, content: &str) -> FileContext {\n        let mut exports = Vec::new();\n        let mut imports = Vec::new();\n        \n        for line in content.lines() {\n            let trimmed = line.trim();\n            \n            // export function xxx\n            if trimmed.starts_with(\"export function \") {\n                if let Some(name) = trimmed.strip_prefix(\"export function \").and_then(|s| s.split('(').next()) {\n                    exports.push(format!(\"{}()\", name.trim()));\n                }\n            }\n            \n            // export class Xxx\n            if trimmed.starts_with(\"export class \") {\n                if let Some(name) = trimmed.strip_prefix(\"export class \").and_then(|s| s.split_whitespace().next()) {\n                    exports.push(name.trim().to_string());\n                }\n            }\n            \n            // export const xxx\n            if trimmed.starts_with(\"export const \") {\n                if let Some(name) = trimmed.strip_prefix(\"export const \").and_then(|s| s.split('=').next()) {\n                    exports.push(name.trim().to_string());\n                }\n            }\n            \n            // import xxx from\n            if trimmed.starts_with(\"import \") {\n                imports.push(trimmed.to_string());\n            }\n        }\n        \n        let summary = format!(\"JavaScript module with {} exports\", exports.len());\n        \n        FileContext {\n            path: path.to_string(),\n            summary,\n            exports,\n            imports,\n            key_types: Vec::new(),\n        }\n    }\n    \n    /// HTML 文件摘要\n    fn generate_html_context(path: &str, content: &str) -> FileContext {\n        let mut key_types = Vec::new();\n        \n        // 提取 id 和 class\n        let mut ids = Vec::new();\n        let mut classes = Vec::new();\n        \n        for line in content.lines() {\n            // id=\"xxx\"\n            if let Some(start) = line.find(\"id=\\\"\") {\n                if let Some(end) = line[start + 4..].find('\"') {\n                    ids.push(line[start + 4..start + 4 + end].to_string());\n                }\n            }\n            \n            // class=\"xxx\"\n            if let Some(start) = line.find(\"class=\\\"\") {\n                if let Some(end) = line[start + 7..].find('\"') {\n                    let class_str = &line[start + 7..start + 7 + end];\n                    for cls in class_str.split_whitespace() {\n                        if !classes.contains(&cls.to_string()) {\n                            classes.push(cls.to_string());\n                        }\n                    }\n                }\n            }\n        }\n        \n        if !ids.is_empty() {\n            key_types.push(format!(\"IDs: {}\", ids.join(\", \")));\n        }\n        \n        if !classes.is_empty() {\n            key_types.push(format!(\"Classes: {}\", classes.iter().take(10).cloned().collect::<Vec<_>>().join(\", \")));\n        }\n        \n        FileContext {\n            path: path.to_string(),\n            summary: \"HTML document\".to_string(),\n            exports: Vec::new(),\n            imports: Vec::new(),\n            key_types,\n        }\n    }\n    \n    /// 通用文件摘要\n    fn generate_generic_context(path: &str, content: &str) -> FileContext {\n        let lines = content.lines().count();\n        \n        FileContext {\n            path: path.to_string(),\n            summary: format!(\"File with {} lines\", lines),\n            exports: Vec::new(),\n            imports: Vec::new(),\n            key_types: Vec::new(),\n        }\n    }\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn test_rust_context_generation() {\n        let rust_code = r#\"\nuse serde::{Serialize, Deserialize};\n\npub struct TodoItem {\n    pub id: String,\n    pub title: String,\n}\n\npub enum Status {\n    Active,\n    Done,\n}\n\npub fn create_todo(title: String) -> TodoItem {\n    TodoItem { id: uuid::new_v4(), title }\n}\n\"#;\n        \n        let context = FileSummaryGenerator::generate(\"todo.rs\", rust_code, \"rust\");\n        \n        assert_eq!(context.exports.len(), 3);  // TodoItem, Status, create_todo\n        assert!(context.exports.contains(&\"TodoItem\".to_string()));\n        assert!(context.exports.contains(&\"Status\".to_string()));\n        assert!(context.exports.contains(&\"create_todo()\".to_string()));\n        assert!(context.imports.len() > 0);\n    }\n    \n    #[test]\n    fn test_batch_context_summary() {\n        let mut batch_ctx = BatchContext::new();\n        \n        batch_ctx.add_file(FileContext {\n            path: \"todo.rs\".to_string(),\n            summary: \"Todo data model\".to_string(),\n            exports: vec![\"TodoItem\".to_string(), \"create_todo()\".to_string()],\n            imports: vec![\"serde::Serialize\".to_string()],\n            key_types: vec![\"struct TodoItem\".to_string()],\n        });\n        \n        let summary = batch_ctx.generate_summary();\n        assert!(summary.contains(\"todo.rs\"));\n        assert!(summary.contains(\"TodoItem\"));\n        assert!(summary.contains(\"consistency\"));\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 39.0,
      "lines_of_code": 342,
      "number_of_classes": 3,
      "number_of_functions": 11
    },
    "dependencies": [
      {
        "dependency_type": "runtime",
        "is_external": true,
        "line_number": null,
        "name": "serde",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "runtime",
        "is_external": true,
        "line_number": null,
        "name": "uuid",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "This component provides context management for batch processing of source code files. It defines two main structures: FileContext to capture metadata about individual files (path, summary, exports, imports, key types), and BatchContext to aggregate multiple FileContext instances. The FileSummaryGenerator utility class analyzes source code content in multiple languages (Rust, Python, JavaScript/TypeScript, HTML, and generic) to extract semantic information such as public declarations and imports, generating structured summaries. The BatchContext.generate_summary method produces a human-readable markdown-formatted summary for use in AI agent instructions, ensuring consistency across generated files. This component acts as a bridge between code analysis and instruction generation in a code-generation workflow.",
    "interfaces": [],
    "responsibilities": [
      "Extract and structure metadata from source code files across multiple programming languages",
      "Aggregate file-level context into a unified batch context for cross-file consistency",
      "Generate human-readable markdown summaries for AI agent instruction prompts",
      "Provide language-specific parsing logic for public declarations and imports",
      "Support consistency enforcement in multi-file code generation workflows"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": null,
      "file_path": "crates/cowork-core/src/agents/watchdog.rs",
      "functions": [
        "WatchDogAgent::new",
        "WatchDogAgent::should_remind",
        "WatchDogAgent::generate_reminder",
        "WatchDogAgent::update_objective",
        "WatchDogAgent::reset_check_count",
        "WatchDogAgent::stats",
        "WatchDogStats"
      ],
      "importance_score": 0.8,
      "interfaces": [],
      "name": "watchdog.rs",
      "source_summary": "/// WatchDog Agent - 监控执行 Agent 的行为，防止跑偏\n/// \n/// 核心功能：\n/// 1. 每隔 N 次工具调用，提醒 Agent 当前目标\n/// 2. 记录检查次数和历史\n/// 3. 生成目标提醒消息\npub struct WatchDogAgent {\n    /// 用户原始需求\n    original_requirements: String,\n    \n    /// 当前阶段目标\n    current_objective: String,\n    \n    /// 检查间隔（每 N 次工具调用检查一次）\n    check_interval: usize,\n    \n    /// 已检查次数\n    check_count: usize,\n}\n\nimpl WatchDogAgent {\n    /// 创建新的 WatchDog Agent\n    /// \n    /// # 参数\n    /// - `original_requirements`: 用户的原始需求描述\n    /// - `current_objective`: 当前阶段的具体目标\n    /// - `check_interval`: 每隔多少次工具调用进行一次检查\n    pub fn new(original_requirements: String, current_objective: String, check_interval: usize) -> Self {\n        tracing::info!(\n            \"WatchDog initialized: interval={}, objective={}\",\n            check_interval,\n            &current_objective\n        );\n        \n        Self {\n            original_requirements,\n            current_objective,\n            check_interval,\n            check_count: 0,\n        }\n    }\n    \n    /// 检查是否需要注入提醒\n    /// \n    /// # 参数\n    /// - `tool_call_count`: 当前工具调用总次数\n    /// \n    /// # 返回\n    /// 如果需要提醒返回 true\n    pub fn should_remind(&self, tool_call_count: usize) -> bool {\n        tool_call_count > 0 && tool_call_count % self.check_interval == 0\n    }\n    \n    /// 生成提醒消息\n    /// \n    /// # 返回\n    /// 格式化的提醒消息，包含原始需求和当前目标\n    pub fn generate_reminder(&mut self) -> String {\n        self.check_count += 1;\n        \n        let reminder = format!(\n            r#\"⚠️  **WatchDog 提醒 #{count}**\n\n📋 **原始需求**:\n{requirements}\n\n🎯 **当前目标**:\n{objective}\n\n❓ **自检问题**:\n1. 你当前的行为是否偏离了原始需求？\n2. 你是否在做不必要的工作？\n3. 你是否遗漏了关键需求？\n\n✅ **继续执行**，但请保持专注于目标。\"#,\n            count = self.check_count,\n            requirements = self.original_requirements,\n            objective = self.current_objective\n        );\n        \n        tracing::info!(\"WatchDog reminder generated (#{}))\", self.check_count);\n        \n        reminder\n    }\n    \n    /// 更新当前目标\n    /// \n    /// 用于在执行过程中切换不同的子任务目标\n    /// \n    /// # 参数\n    /// - `new_objective`: 新的目标描述\n    pub fn update_objective(&mut self, new_objective: String) {\n        tracing::info!(\n            \"WatchDog objective updated: {} -> {}\",\n            &self.current_objective,\n            &new_objective\n        );\n        self.current_objective = new_objective;\n    }\n    \n    /// 重置检查计数器\n    /// \n    /// 用于在开始新的阶段时重置统计\n    pub fn reset_check_count(&mut self) {\n        tracing::info!(\"WatchDog check count reset (was: {})\", self.check_count);\n        self.check_count = 0;\n    }\n    \n    /// 获取统计信息\n    pub fn stats(&self) -> WatchDogStats {\n        WatchDogStats {\n            check_count: self.check_count,\n            check_interval: self.check_interval,\n        }\n    }\n}\n\n/// WatchDog 统计信息\n#[derive(Debug, Clone)]\npub struct WatchDogStats {\n    pub check_count: usize,\n    pub check_interval: usize,\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n    \n    #[test]\n    fn test_watchdog_should_remind() {\n        let watchdog = WatchDogAgent::new(\n            \"Build a web app\".to_string(),\n            \"Generate HTML files\".to_string(),\n            3\n        );\n        \n        assert!(!watchdog.should_remind(0));\n        assert!(!watchdog.should_remind(1));\n        assert!(!watchdog.should_remind(2));\n        assert!(watchdog.should_remind(3));\n        assert!(!watchdog.should_remind(4));\n        assert!(!watchdog.should_remind(5));\n        assert!(watchdog.should_remind(6));\n    }\n    \n    #[test]\n    fn test_watchdog_generate_reminder() {\n        let mut watchdog = WatchDogAgent::new(\n            \"Build a web app\".to_string(),\n            \"Generate HTML files\".to_string(),\n            3\n        );\n        \n        let reminder1 = watchdog.generate_reminder();\n        assert!(reminder1.contains(\"WatchDog 提醒 #1\"));\n        assert!(reminder1.contains(\"Build a web app\"));\n        assert!(reminder1.contains(\"Generate HTML files\"));\n        \n        let reminder2 = watchdog.generate_reminder();\n        assert!(reminder2.contains(\"WatchDog 提醒 #2\"));\n    }\n    \n    #[test]\n    fn test_watchdog_update_objective() {\n        let mut watchdog = WatchDogAgent::new(\n            \"Build a web app\".to_string(),\n            \"Generate HTML files\".to_string(),\n            3\n        );\n        \n        watchdog.update_objective(\"Generate CSS files\".to_string());\n        \n        let reminder = watchdog.generate_reminder();\n        assert!(reminder.contains(\"Generate CSS files\"));\n        assert!(!reminder.contains(\"Generate HTML files\"));\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 1.0,
      "lines_of_code": 177,
      "number_of_classes": 2,
      "number_of_functions": 7
    },
    "dependencies": [],
    "detailed_description": "The WatchDogAgent is an intelligent monitoring agent designed to prevent AI agents from deviating from their intended objectives during execution. It operates by periodically reminding the agent of its original requirements and current objective after a configurable number of tool calls. The agent tracks check counts, generates structured reminder messages with self-check questions, and allows dynamic updates to the current objective. It includes a companion struct WatchDogStats for exposing internal metrics. The component is heavily commented in Chinese and includes comprehensive unit tests covering all core functionalities.",
    "interfaces": [],
    "responsibilities": [
      "Monitor agent behavior by tracking tool call frequency",
      "Generate structured reminders to realign agent focus with original requirements",
      "Allow dynamic updating of current objectives during multi-stage execution",
      "Maintain and expose internal state statistics for observability",
      "Provide testable, deterministic logic for behavioral auditing"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": null,
      "file_path": "crates/cowork-core/src/agents/feedback_agent.rs",
      "functions": [
        "FeedbackAgent::new",
        "FeedbackAgent::execute"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "FeedbackAgent"
      ],
      "name": "feedback_agent.rs",
      "source_summary": "use anyhow::Result;\nuse adk_rust::prelude::*;\nuse adk_rust::model::{OpenAIClient, OpenAIConfig};\nuse adk_rust::runner::{Runner, RunnerConfig};\nuse adk_rust::session::{InMemorySessionService, CreateRequest, SessionService, GetRequest};\nuse futures::StreamExt;\nuse std::collections::HashMap;\nuse std::sync::Arc;\n\nuse crate::artifacts::*;\nuse crate::memory::ArtifactStore;\nuse crate::config::LlmConfig;\n\n/// Feedback Agent - 收集反馈并决定是否需要迭代\npub struct FeedbackAgent {\n    model: Arc<OpenAIClient>,\n    store: Arc<ArtifactStore>,\n}\n\nimpl FeedbackAgent {\n    pub fn new(llm_config: &LlmConfig, store: Arc<ArtifactStore>) -> Result<Self> {\n        let config = OpenAIConfig::compatible(\n            llm_config.api_key.clone(),\n            llm_config.api_base_url.clone(),\n            llm_config.model_name.clone(),\n        );\n        \n        tracing::info!(\"Creating Feedback Agent with OpenAI-compatible client )\");\n        \n        let model = OpenAIClient::new(config)?;\n\n        Ok(Self {\n            model: Arc::new(model),\n            store,\n        })\n    }\n\n    pub async fn execute(\n        &self,\n        session_id: &str,\n        check_artifact: &CheckReportArtifact,\n        user_feedback: &str,\n    ) -> Result<FeedbackArtifact> {\n        tracing::info!(\"FeedbackAgent: processing feedback for session {}\", session_id);\n\n        let output_schema = serde_json::json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"delta\": {\n                    \"type\": \"array\",\n                    \"items\": {\n                        \"type\": \"object\",\n                        \"properties\": {\n                            \"target_stage\": {\n                                \"type\": \"string\",\n                                \"enum\": [\"idea_intake\", \"requirements\", \"design\", \"plan\", \"coding\", \"check\", \"feedback\", \"delivery\"]\n                            },\n                            \"change\": {\"type\": \"string\"}\n                        },\n                        \"required\": [\"target_stage\", \"change\"]\n                    }\n                },\n                \"rerun\": {\n                    \"type\": \"array\",\n                    \"items\": {\n                        \"type\": \"object\",\n                        \"properties\": {\n                            \"stage\": {\n                                \"type\": \"string\",\n                                \"enum\": [\"idea_intake\", \"requirements\", \"design\", \"plan\", \"coding\", \"check\", \"feedback\", \"delivery\"]\n                            },\n                            \"reason\": {\"type\": \"string\"}\n                        },\n                        \"required\": [\"stage\", \"reason\"]\n                    }\n                }\n            },\n            \"required\": [\"delta\", \"rerun\"]\n        });\n\n        let context = format!(\n            r#\"Based on the check report and user feedback, analyze what needs to be changed.\n\n**Check Report Summary:**\nTotal checks: {}\nIssues found: {}\n\n**Issues:**\n{}\n\n**User Feedback:**\n{}\n\nDetermine what changes are needed and which stages should be re-run.\"#,\n            check_artifact.data.checks.len(),\n            check_artifact.data.issues.len(),\n            check_artifact.data.issues.iter()\n                .map(|i| format!(\"[{}] {}: {}\", i.sev, i.id, i.desc))\n                .collect::<Vec<_>>()\n                .join(\"\\n\"),\n            user_feedback,\n        );\n\n        let agent = Arc::new(\n            LlmAgentBuilder::new(\"feedback_analyzer\")\n                .description(\"Analyze feedback and determine necessary changes\")\n                .instruction(\n                    r#\"You are a project coordinator. Analyze feedback and determine next steps.\n\n**Required JSON Structure:**\n{\n  \"delta\": [\n    {\n      \"target_stage\": \"stage_name\",\n      \"change\": \"description of what needs to change\"\n    }\n  ],\n  \"rerun\": [\n    {\n      \"stage\": \"stage_to_rerun\",\n      \"reason\": \"why it needs to be re-run\"\n    }\n  ]\n}\n\n**Stage Names:**\n- idea_intake, requirements, design, plan, coding, check, feedback, delivery\n\n**Output Requirements:**\n1. Respond with ONLY valid JSON\n2. Arrays can be empty if no changes/reruns needed\n3. Be specific about what needs to change\n4. Provide clear reasons for re-runs\n5. Consider dependency order (e.g., if design changes, plan/code may need rerun)\"#,\n                )\n                .model(self.model.clone())\n                .output_schema(output_schema)\n                .output_key(\"feedback_raw\")\n                .build()?,\n        );\n\n        let session_service = Arc::new(InMemorySessionService::new());\n        let app_name = \"cowork\".to_string();\n        let user_id = session_id.to_string();\n\n        let _session = session_service\n            .create(CreateRequest {\n                app_name: app_name.clone(),\n                user_id: user_id.clone(),\n                session_id: Some(session_id.to_string()),\n                state: HashMap::new(),\n            })\n            .await?;\n\n        let runner = Runner::new(RunnerConfig {\n            app_name: app_name.clone(),\n            agent: agent.clone(),\n            session_service: session_service.clone(),\n            artifact_service: None,\n            memory_service: None,\n            run_config: None,\n        })?;\n\n        let input_content = Content::new(\"user\").with_text(&context);\n\n        tracing::info!(\"Invoking Feedback analysis agent...\");\n\n        let mut event_stream = runner\n            .run(user_id.clone(), session_id.to_string(), input_content)\n            .await?;\n\n        while let Some(event_result) = event_stream.next().await {\n            match event_result {\n                Ok(_event) => {},\n                Err(e) => {\n                    tracing::error!(\"Error during feedback analysis: {}\", e);\n                    return Err(anyhow::anyhow!(\"Feedback analysis failed: {}\", e));\n                }\n            }\n        }\n\n        tracing::info!(\"Feedback analysis complete\");\n\n        let updated_session = session_service\n            .get(GetRequest {\n                user_id: user_id.clone(),\n                session_id: session_id.to_string(),\n                app_name: app_name.clone(),\n                after: None,\n                num_recent_events: None,\n            })\n            .await?;\n\n        let state = updated_session.state();\n        let raw_output = state\n            .get(\"feedback_raw\")\n            .ok_or_else(|| anyhow::anyhow!(\"No output from Feedback agent\"))?;\n\n        let feedback: Feedback = match raw_output {\n            serde_json::Value::String(json_str) => {\n                serde_json::from_str(json_str.as_str())?\n            }\n            value => {\n                serde_json::from_value(value.clone())?\n            }\n        };\n\n        tracing::info!(\"Successfully parsed Feedback\");\n\n        let summary = vec![\n            format!(\"Changes needed: {}\", feedback.delta.len()),\n            format!(\"Stages to rerun: {}\", feedback.rerun.len()),\n        ];\n\n        let artifact = ArtifactEnvelope::new(session_id.to_string(), Stage::Feedback, feedback)\n            .with_summary(summary)\n            .with_prev(vec![check_artifact.meta.artifact_id.clone()]);\n\n        self.store.put(session_id, Stage::Feedback, &artifact)?;\n\n        tracing::info!(\"Feedback artifact saved successfully\");\n\n        Ok(artifact)\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 8.0,
      "lines_of_code": 225,
      "number_of_classes": 1,
      "number_of_functions": 2
    },
    "dependencies": [
      {
        "dependency_type": "runtime",
        "is_external": true,
        "line_number": null,
        "name": "anyhow",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "runtime",
        "is_external": true,
        "line_number": null,
        "name": "adk_rust",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "runtime",
        "is_external": true,
        "line_number": null,
        "name": "futures",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "runtime",
        "is_external": true,
        "line_number": null,
        "name": "serde_json",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "runtime",
        "is_external": true,
        "line_number": null,
        "name": "tracing",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::artifacts",
        "path": "crates/cowork-core/src/artifacts/mod.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::memory::ArtifactStore",
        "path": "crates/cowork-core/src/memory/artifact_store.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::config::LlmConfig",
        "path": "crates/cowork-core/src/config/llm_config.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "Content",
        "path": "crates/cowork-core/src/model/content.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "ArtifactEnvelope",
        "path": "crates/cowork-core/src/artifacts/artifact_envelope.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "Stage",
        "path": "crates/cowork-core/src/artifacts/mod.rs",
        "version": null
      }
    ],
    "detailed_description": "The FeedbackAgent is an intelligent agent responsible for analyzing user feedback and check reports to determine necessary changes and re-runs in a software development workflow. It leverages an OpenAI LLM to process structured input containing check report summaries and user feedback, then outputs a JSON-formatted analysis specifying required changes (delta) and stages that need to be re-executed (rerun). The agent operates within a session-based execution framework, creating a temporary in-memory session, invoking the LLM via a runner, capturing the output, and persisting the result as a FeedbackArtifact in the artifact store. This enables iterative refinement of software artifacts based on quality assessments and user input.",
    "interfaces": [
      {
        "description": "Main agent struct holding LLM client and artifact store references",
        "interface_type": "struct",
        "name": "FeedbackAgent",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "model",
            "param_type": "Arc<OpenAIClient>"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "store",
            "param_type": "Arc<ArtifactStore>"
          }
        ],
        "return_type": null,
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Analyze check reports and user feedback to identify required changes",
      "Determine which development stages need to be re-executed",
      "Generate structured JSON output conforming to a predefined schema",
      "Manage LLM interaction via session and runner infrastructure",
      "Persist analysis results as artifacts in the system's memory store"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": null,
      "file_path": "crates/cowork-core/src/agents/code_executor.rs",
      "functions": [
        "CodeExecutor::new",
        "CodeExecutor::execute",
        "CodeExecutor::execute_with_todo",
        "CodeExecutor::execute_with_batches",
        "CodeExecutor::execute_batch",
        "CodeExecutor::execute_single_agent",
        "CodeExecutor::build_requirements_summary",
        "CodeExecutor::build_batch_instruction",
        "CodeExecutor::build_instruction",
        "CodeExecutor::build_task_description"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "ExecutionReport",
        "ChangeResult",
        "ChangeStatus",
        "Change",
        "BatchExecutionReport"
      ],
      "name": "code_executor.rs",
      "source_summary": "use anyhow::Result;\nuse std::sync::Arc;\nuse std::collections::HashMap;\n\nuse crate::artifacts::*;\nuse crate::hitl::HitlController;\nuse crate::config::LlmConfig;\nuse crate::tools::create_file_tools;\nuse adk_rust::model::{OpenAIClient, OpenAIConfig};\nuse adk_rust::prelude::*;\nuse adk_rust::runner::{Runner, RunnerConfig};\nuse adk_rust::session::{InMemorySessionService, CreateRequest, SessionService};\nuse futures::StreamExt;\n\n/// Code Executor - 使用 LLM Agent + file tools 自动实现代码\n/// \n/// 核心思想：\n/// 1. 创建一个 LlmAgent，挂载文件操作工具\n/// 2. 给 Agent 提供变更计划和需求描述\n/// 3. Agent 自己决定如何调用工具来实现代码\npub struct CodeExecutor {\n    model: Arc<OpenAIClient>,\n}\n\nimpl CodeExecutor {\n    pub fn new(llm_config: &LlmConfig) -> Result<Self> {\n        let config = OpenAIConfig::compatible(\n            llm_config.api_key.clone(),\n            llm_config.api_base_url.clone(),\n            llm_config.model_name.clone(),\n        );\n        \n        let client = OpenAIClient::new(config)?;\n        \n        Ok(Self {\n            model: Arc::new(client),\n        })\n    }\n\n    /// 执行代码变更计划（便捷方法）\n    pub async fn execute(\n        &self,\n        code_artifact: &CodeChangeArtifact,\n        hitl: &HitlController\n    ) -> Result<ExecutionReport> {\n        // 便捷方法：不追踪 TodoList\n        self.execute_with_todo(code_artifact, hitl, None, None).await\n    }\n    \n    /// 执行代码变更计划（完整版本，支持 TodoList 追踪和 WatchDog）\n    pub async fn execute_with_todo(\n        &self,\n        code_artifact: &CodeChangeArtifact,\n        hitl: &HitlController,\n        prd_summary: Option<&str>,\n        todo_list: Option<&mut TodoList>,\n    ) -> Result<ExecutionReport> {\n        tracing::info!(\"Starting AI-powered code execution with batch sub-agents...\");\n        \n        println!(\"\\n╔═══════════════════════════════════════╗\");\n        println!(\"║   AI 代码生成与执行                   ║\");\n        println!(\"╚═══════════════════════════════════════╝\\n\");\n\n        println!(\"📋 计划执行 {} 个代码变更：\", code_artifact.data.changes.len());\n        for (i, change) in code_artifact.data.changes.iter().enumerate() {\n            println!(\"  {}. [{}] {} - {}\", \n                i + 1, \n                change.kind, \n                change.path, \n                change.note\n            );\n        }\n        println!();\n\n        if !hitl.confirm(\"是否让 AI Agent 自动实现代码并写入文件？\")? {\n            tracing::info!(\"Code execution cancelled by user\");\n            return Ok(ExecutionReport {\n                total_changes: code_artifact.data.changes.len(),\n                successful: 0,\n                failed: 0,\n                skipped: code_artifact.data.changes.len(),\n                details: Vec::new(),\n            });\n        }        // 决策：根据文件数量选择策略\n        let changes = &code_artifact.data.changes;\n        if changes.len() <= 3 {\n            // 少于等于 3 个文件：单个 Agent 处理\n            println!(\"📝 使用单个 Agent 模式（文件数 <= 3）\\n\");\n            self.execute_single_agent(code_artifact, hitl, prd_summary, todo_list).await\n        } else {\n            // 3 个以上文件：分批处理\n            println!(\"📦 使用分批 Sub-Agent 模式（文件数 > 3）\\n\");\n            self.execute_with_batches(code_artifact, hitl, prd_summary, todo_list).await\n        }\n    }\n\n    /// 分批处理模式（带上下文传递和 WatchDog）\n    async fn execute_with_batches(\n        &self,\n        code_artifact: &CodeChangeArtifact,\n        _hitl: &HitlController,\n        prd_summary: Option<&str>,\n        todo_list: Option<&mut TodoList>,\n    ) -> Result<ExecutionReport> {\n        const BATCH_SIZE: usize = 3;  // 每批处理 3 个文件\n        \n        let changes = &code_artifact.data.changes;\n        let batches: Vec<&[crate::artifacts::Change]> = changes.chunks(BATCH_SIZE).collect();\n        \n        println!(\"📦 将 {} 个文件分成 {} 批处理（每批最多 {} 个文件）\",\n            changes.len(),\n            batches.len(),\n            BATCH_SIZE\n        );\n        println!();\n        \n        let mut all_details = Vec::new();\n        let mut successful_count = 0;\n        let mut failed_count = 0;\n        \n        // 构建原始需求描述（用于 WatchDog）\n        let original_requirements = prd_summary\n            .map(|s| s.to_string())\n            .unwrap_or_else(|| self.build_requirements_summary(code_artifact));\n        \n        // 批次上下文（包含文件摘要）\n        let mut batch_context = crate::agents::BatchContext::new();\n        \n        // 逐批处理\n        for (batch_idx, batch) in batches.iter().enumerate() {\n            println!(\"╔═══════════════════════════════════════╗\");\n            println!(\"║   批次 {}/{}                         \", batch_idx + 1, batches.len());\n            println!(\"╚═══════════════════════════════════════╝\\n\");\n            \n            println!(\"📝 批次 {} 包含 {} 个文件：\", batch_idx + 1, batch.len());\n            for (i, change) in batch.iter().enumerate() {\n                println!(\"  {}. [{}] {}\", i + 1, change.kind, change.path);\n            }\n            println!();\n            \n            // 显示批次上下文\n            if !batch_context.completed_files.is_empty() {\n                println!(\"📚 已完成的文件 ({} 个):\", batch_context.completed_files.len());\n                for file_ctx in &batch_context.completed_files {\n                    println!(\"  - {} ({})\", file_ctx.path, file_ctx.summary);\n                    if !file_ctx.exports.is_empty() {\n                        println!(\"    Exports: {}\", file_ctx.exports.iter().take(3).cloned().collect::<Vec<_>>().join(\", \"));\n                    }\n                }\n                println!();\n            }\n            \n            // 为这一批创建独立的 Sub-Agent，传入 WatchDog 需求和上下文摘要\n            let batch_result = self.execute_batch(\n                batch_idx,\n                batch,\n                &code_artifact.data.target,\n                Some(&original_requirements),  // 启用 WatchDog\n                &batch_context,  // 批次间上下文摘要\n            ).await?;\n            \n            // 生成文件上下文并添加到批次上下文\n            for detail in &batch_result.details {\n                if detail.status == ChangeStatus::Success {\n                    // 读取文件内容并生成摘要\n                    if let Ok(content) = std::fs::read_to_string(&detail.change.path) {\n                        let file_ctx = crate::agents::FileSummaryGenerator::generate(\n                            &detail.change.path,\n                            &content,\n                            &code_artifact.data.target.lang\n                        );\n                        batch_context.add_file(file_ctx);\n                    }\n                }\n            }\n            \n            successful_count += batch_result.successful;\n            failed_count += batch_result.failed;\n            all_details.extend(batch_result.details);\n            \n            println!(\"✅ 批次 {} 完成: {} 成功, {} 失败\\n\",\n                batch_idx + 1,\n                batch_result.successful,\n                batch_result.failed\n            );\n        }\n        \n        println!(\"╔═══════════════════════════════════════╗\");\n        println!(\"║   总执行摘要                          ║\");\n        println!(\"╚═══════════════════════════════════════╝\");\n        println!(\"总批次: {}\", batches.len());\n        println!(\"计划变更: {}\", changes.len());\n        println!(\"✅ 成功: {}\", successful_count);\n        println!(\"❌ 失败: {}\", failed_count);\n        \n        // 更新 TodoList（如果提供了）\n        if let Some(todo_list) = todo_list {\n            let successful_files: Vec<String> = all_details.iter()\n                .filter(|d| d.status == ChangeStatus::Success)\n                .map(|d| d.change.path.clone())\n                .collect();\n            \n            let failed_files: Vec<String> = all_details.iter()\n                .filter(|d| d.status == ChangeStatus::Failed)\n                .map(|d| d.change.path.clone())\n                .collect();\n            \n            crate::agents::TodoListManager::update_from_execution(\n                todo_list,\n                &code_artifact.data.changes,\n                &successful_files,\n                &failed_files,\n            );\n            \n            // 打印 TodoList 状态\n            crate::agents::TodoListManager::print_status(todo_list);\n        }\n        \n        Ok(ExecutionReport {\n            total_changes: changes.len(),\n            successful: successful_count,\n            failed: failed_count,\n            skipped: 0,\n            details: all_details,\n        })\n    }\n\n    /// 执行单个批次（集成 WatchDog 和上下文传递）\n    async fn execute_batch(\n        &self,\n        batch_idx: usize,\n        batch: &[crate::artifacts::Change],\n        target: &TargetProject,\n        original_requirements: Option<&str>,\n        batch_context: &crate::agents::BatchContext,  // 批次上下文摘要\n    ) -> Result<BatchExecutionReport> {\n        // 创建文件操作工具\n        let file_tools = create_file_tools();\n        \n        // 构建批次任务描述\n        let task_description = format!(\n            \"Please implement the following {} code changes:\\n\\n{}\",\n            batch.len(),\n            batch.iter()\n                .enumerate()\n                .map(|(i, change)| format!(\n                    \"{}. [{}] {} - {}\",\n                    i + 1,\n                    change.kind,\n                    change.path,\n                    change.note\n                ))\n                .collect::<Vec<_>>()\n                .join(\"\\n\")\n        );\n        \n        // 为每个批次创建独立的 Agent（上下文隔离）+ WatchDog 提醒 + 上下文传递\n        let agent = Arc::new(\n            LlmAgentBuilder::new(format!(\"batch_{}_executor\", batch_idx))\n                .description(\"Batch code executor\")\n                .instruction(&self.build_batch_instruction(\n                    target, \n                    batch.len(), \n                    original_requirements,\n                    batch_context\n                ))\n                .model(self.model.clone())\n                // 挂载所有文件工具（10 个）\n                .tool(file_tools.write_file.clone())\n                .tool(file_tools.read_file.clone())\n                .tool(file_tools.list_dir.clone())\n                .tool(file_tools.file_exists.clone())\n                .tool(file_tools.create_dir.clone())\n                .tool(file_tools.read_file_range.clone())\n                .tool(file_tools.replace_line_range.clone())\n                .tool(file_tools.insert_lines.clone())\n                .tool(file_tools.delete_line_range.clone())\n                .tool(file_tools.append_to_file.clone())\n                .build()?\n        );\n        \n        // 创建独立的 Session\n        let session_service = Arc::new(InMemorySessionService::new());\n        let app_name = \"cowork_batch_executor\".to_string();\n        let session_id = format!(\"batch_{}_{}\", batch_idx, uuid::Uuid::new_v4());\n        let user_id = \"batch_executor\".to_string();\n        \n        session_service.create(CreateRequest {\n            app_name: app_name.clone(),\n            user_id: user_id.clone(),\n            session_id: Some(session_id.clone()),\n            state: HashMap::new(),\n        }).await?;\n        \n        let runner = Runner::new(RunnerConfig {\n            app_name,\n            agent,\n            session_service,\n            artifact_service: None,\n            memory_service: None,\n            run_config: None,\n        })?;\n        \n        println!(\"🤖 Sub-Agent 开始执行批次 {}...\\n\", batch_idx + 1);\n        \n        // 执行\n        let mut event_stream = runner.run(\n            user_id,\n            session_id,\n            Content::new(\"user\").with_text(&task_description)\n        ).await?;\n        \n        while let Some(event_result) = event_stream.next().await {\n            if let Err(e) = event_result {\n                tracing::error!(\"Error in batch {}: {}\", batch_idx, e);\n                return Ok(BatchExecutionReport {\n                    successful: 0,\n                    failed: batch.len(),\n                    details: vec![ChangeResult {\n                        change: Change {\n                            path: format!(\"batch_{}\", batch_idx),\n                            kind: \"batch\".to_string(),\n                        },\n                        status: ChangeStatus::Failed,\n                        message: format!(\"Batch {} failed: {}\", batch_idx, e),\n                    }],\n                });\n            }\n        }\n        \n        println!(\"✅ Sub-Agent 批次 {} 执行完成\\n\", batch_idx + 1);\n        \n        // 验证文件是否存在\n        let mut successful = 0;\n        let mut failed = 0;\n        let mut details = Vec::new();\n        \n        for change in batch {\n            let file_exists = std::path::Path::new(&change.path).exists();\n            if file_exists {\n                successful += 1;\n                details.push(ChangeResult {\n                    change: Change {\n                        path: change.path.clone(),\n                        kind: change.kind.clone(),\n                    },\n                    status: ChangeStatus::Success,\n                    message: format!(\"File created: {}\", change.path),\n                });\n            } else {\n                failed += 1;\n                details.push(ChangeResult {\n                    change: Change {\n                        path: change.path.clone(),\n                        kind: change.kind.clone(),\n                    },\n                    status: ChangeStatus::Failed,\n                    message: format!(\"File not found after execution: {}\", change.path),\n                });\n            }\n        }\n        \n        Ok(BatchExecutionReport {\n            successful,\n            failed,\n            details,\n        })\n    }\n    \n    /// 单个 Agent 处理（原有逻辑，用于少量文件）\n    async fn execute_single_agent(\n        &self,\n        code_artifact: &CodeChangeArtifact,\n        _hitl: &HitlController,\n        _prd_summary: Option<&str>,\n        todo_list: Option<&mut TodoList>,\n    ) -> Result<ExecutionReport> {\n        // 创建文件操作工具\n        let file_tools = create_file_tools();\n\n        // 构建任务描述\n        let task_description = self.build_task_description(code_artifact);\n\n        // 创建执行 Agent（带文件工具）\n        let agent = Arc::new(\n            LlmAgentBuilder::new(\"code_executor_agent\")\n                .description(\"AI agent that implements code changes by calling file tools\")\n                .instruction(&self.build_instruction(&code_artifact.data))\n                .model(self.model.clone())\n                .tool(file_tools.write_file.clone())\n                .tool(file_tools.read_file.clone())\n                .tool(file_tools.list_dir.clone())\n                .tool(file_tools.file_exists.clone())\n                .tool(file_tools.create_dir.clone())\n                // 增量编辑工具（用于大文件）\n                .tool(file_tools.read_file_range.clone())\n                .tool(file_tools.replace_line_range.clone())\n                .tool(file_tools.insert_lines.clone())\n                .tool(file_tools.delete_line_range.clone())\n                .tool(file_tools.append_to_file.clone())\n                .build()?\n        );\n\n        let session_service = Arc::new(InMemorySessionService::new());\n        let app_name = \"cowork_executor\".to_string();\n        let session_id = format!(\"exec_{}\", uuid::Uuid::new_v4().to_string());\n        let user_id = \"code_executor\".to_string();\n\n        session_service.create(CreateRequest {\n            app_name: app_name.clone(),\n            user_id: user_id.clone(),\n            session_id: Some(session_id.clone()),\n            state: HashMap::new(),\n        }).await?;\n\n        let runner = Runner::new(RunnerConfig {\n            app_name: app_name.clone(),\n            agent,\n            session_service: session_service.clone(),\n            artifact_service: None,\n            memory_service: None,\n            run_config: None,\n        })?;\n\n        let input_content = Content::new(\"user\").with_text(&task_description);\n\n        println!(\"🤖 AI Agent 开始执行任务...\\n\");\n        \n        // 执行\n        let mut event_stream = runner.run(user_id, session_id, input_content).await?;\n        \n        while let Some(event_result) = event_stream.next().await {\n            if let Err(e) = event_result {\n                tracing::error!(\"Error during execution: {}\", e);\n                return Err(anyhow::anyhow!(\"Code execution failed: {}\", e));\n            }\n        }\n        \n        println!(\"✅ AI Agent 执行完成\\n\");\n\n        // 验证文件是否存在\n        let mut successful = 0;\n        let mut failed = 0;\n        let mut details = Vec::new();\n\n        for change in &code_artifact.data.changes {\n            let file_exists = std::path::Path::new(&change.path).exists();\n            if file_exists {\n                successful += 1;\n                details.push(ChangeResult {\n                    change: Change {\n                        path: change.path.clone(),\n                        kind: change.kind.clone(),\n                    },\n                    status: ChangeStatus::Success,\n                    message: format!(\"File created: {}\", change.path),\n                });\n            } else {\n                failed += 1;\n                details.push(ChangeResult {\n                    change: Change {\n                        path: change.path.clone(),\n                        kind: change.kind.clone(),\n                    },\n                    status: ChangeStatus::Failed,\n                    message: format!(\"File not found after execution: {}\", change.path),\n                });\n            }\n        }\n        \n        // 更新 TodoList（如果提供了）\n        if let Some(todo_list) = todo_list {\n            let successful_files: Vec<String> = details.iter()\n                .filter(|d| d.status == ChangeStatus::Success)\n                .map(|d| d.change.path.clone())\n                .collect();\n            \n            let failed_files: Vec<String> = details.iter()\n                .filter(|d| d.status == ChangeStatus::Failed)\n                .map(|d| d.change.path.clone())\n                .collect();\n            \n            crate::agents::TodoListManager::update_from_execution(\n                todo_list,\n                &code_artifact.data.changes,\n                &successful_files,\n                &failed_files,\n            );\n            \n            // 打印 TodoList 状态\n            crate::agents::TodoListManager::print_status(todo_list);\n        }\n\n        Ok(ExecutionReport {\n            total_changes: code_artifact.data.changes.len(),\n            successful,\n            failed,\n            skipped: 0,\n            details,\n        })\n    }\n    \n    /// 构建原始需求摘要（用于 WatchDog）\n    fn build_requirements_summary(&self, code_artifact: &CodeChangeArtifact) -> String {\n        let lang = &code_artifact.data.target.lang;\n        let stack = code_artifact.data.target.stack.join(\", \");\n        \n        format!(\n            \"Target Language: {}\\nTech Stack: {}\\nTotal Files: {}\",\n            lang,\n            stack,\n            code_artifact.data.changes.len()\n        )\n    }\n    \n    /// 构建批次指令（集成 WatchDog 提醒和上下文传递）\n    fn build_batch_instruction(\n        &self, \n        target: &TargetProject, \n        file_count: usize, \n        original_requirements: Option<&str>,\n        batch_context: &crate::agents::BatchContext\n    ) -> String {\n        // WatchDog 提醒\n        let watchdog_reminder = if let Some(reqs) = original_requirements {\n            format!(\n                r#\"\n\n**⚠️  WATCHDOG REMINDER: Original User Requirements**\n{}\n\n**Self-Check Questions (review every 3 tool calls):**\n1. Am I still aligned with the user's original requirements?\n2. Am I generating files in the correct language ({})?\n3. Am I creating production-ready code (no TODOs, no placeholders)?\n\"#,\n                reqs,\n                target.lang\n            )\n        } else {\n            String::new()\n        };\n        \n        // 上下文传递：使用详细的文件摘要\n        let context_info = batch_context.generate_summary();\n        \n        format!(\n            r#\"You are a professional software developer.\n\n**Your Task**: Implement {} code file(s) for a {} project.\n\n**Technology Context**:\n- Language: {}\n- Tech Stack: {}\n{}{}\n\n**Instructions**:\n1. For each file change:\n   - Generate COMPLETE, PRODUCTION-READY code (no TODO, no placeholders)\n   - Call write_file to save the code\n   \n2. File Size Strategy:\n   - For small files (< 500 lines): use write_file with complete content\n   - For large files (> 500 lines): use incremental tools (read_file_range, replace_line_range)\n   \n3. Code Quality:\n   - Include all necessary imports and dependencies\n   - Follow best practices for {}\n   - Add clear comments\n   - Code should be ready to run/compile\n   \n4. Consistency:\n   - If referencing previously generated files, read them first to understand their structure\n   - Maintain consistent naming, types, and patterns\n   \n5. Work systematically through each file in the list.\n\nIMPORTANT: This is a batch of {} files. Focus only on these files, complete them, then stop.\"#,\n            file_count,\n            target.lang,\n            target.lang,\n            target.stack.join(\", \"),\n            watchdog_reminder,\n            context_info,\n            target.lang,\n            file_count\n        )\n    }\n\n    /// 构建 Agent 指令\n    fn build_instruction(&self, code_plan: &CodeChange) -> String {\n        let lang = &code_plan.target.lang;\n        let tech_stack = code_plan.target.stack.join(\", \");\n\n        format!(\n            r#\"You are an expert software developer with access to file system tools.\n\n**Your Task:** Implement the code changes described by the user.\n\n**Technology Context:**\n- Language: {}\n- Tech Stack: {}\n\n**Available Tools:**\n1. write_file(path, content) - Write complete code to a file\n2. read_file(path) - Read entire file content\n3. list_directory(path, recursive) - List files in a directory\n4. file_exists(path) - Check if a file exists\n5. create_dir(path, recursive) - Create directories\n\n**For Large Files (to avoid context overflow):**\n6. read_file_range(path, start_line, end_line) - Read specific lines\n7. replace_line_range(path, start_line, end_line, new_content) - Replace specific lines\n8. insert_lines(path, after_line, content) - Insert lines after a specific position\n9. delete_line_range(path, start_line, end_line) - Delete specific lines\n10. append_to_file(path, content) - Append to end of file\n\n**Instructions:**\n1. For each file change requested by the user:\n   - If file is small (<500 lines): use write_file with complete code\n   - If file is large (>500 lines): use incremental editing tools (read_file_range, replace_line_range, etc.)\n   - Generate COMPLETE, PRODUCTION-READY code (no TODO comments, no placeholders)\n   \n2. Code Quality Requirements:\n   - Write complete, working code\n   - Include all necessary imports and dependencies\n   - Follow best practices for {}\n   - Add clear comments for complex logic\n   - The code should be ready to run/compile immediately\n\n3. For HTML files:\n   - Include complete HTML5 structure\n   - Embed CSS in <style> tags\n   - Add responsive design with meta viewport\n   - Include basic JavaScript if needed\n\n4. For configuration files:\n   - Use appropriate format (JSON, TOML, etc.)\n   - Include all necessary fields\n\n5. Work systematically:\n   - Process one file at a time\n   - Confirm each file is written before moving to the next\n   - If you encounter errors, explain what went wrong\n\n**IMPORTANT:**\n- Generate REAL, WORKING code - not templates, not TODOs\n- Use the write_file tool to save every file\n- Be thorough and complete in your implementations\"#,\n            lang,\n            tech_stack,\n            lang\n        )\n    }\n\n    /// 构建任务描述\n    fn build_task_description(&self, code_artifact: &CodeChangeArtifact) -> String {\n        let changes_list = code_artifact.data.changes.iter()\n            .map(|change| {\n                format!(\"- [{}] {}: {}\", change.kind, change.path, change.note)\n            })\n            .collect::<Vec<_>>()\n            .join(\"\\n\");\n\n        format!(\n            r#\"Please implement the following code changes:\n\n{}\n\nFor each file:\n1. Generate complete, production-ready code based on the description\n2. Use write_file tool to save the code to the specified path\n3. Ensure all code is complete and ready to run\n\nStart implementing now. Work through each file systematically.\"#,\n            changes_list\n        )\n    }\n}\n\n/// 执行报告\n#[derive(Debug, Clone)]\npub struct ExecutionReport {\n    pub total_changes: usize,\n    pub successful: usize,\n    pub failed: usize,\n    pub skipped: usize,\n    pub details: Vec<ChangeResult>,\n}\n\n/// 单个变更的执行结果\n#[derive(Debug, Clone)]\npub struct ChangeResult {\n    pub change: Change,\n    pub status: ChangeStatus,\n    pub message: String,\n}\n\n/// 变更状态\n#[derive(Debug, Clone, PartialEq)]\npub enum ChangeStatus {\n    Success,\n    Failed,\n    Skipped,\n}\n\n/// 简化的变更信息（用于报告）\n#[derive(Debug, Clone)]\npub struct Change {\n    pub path: String,\n    pub kind: String,\n}\n\n/// 批次执行报告（内部使用）\n#[derive(Debug)]\nstruct BatchExecutionReport {\n    successful: usize,\n    failed: usize,\n    details: Vec<ChangeResult>,\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 29.0,
      "lines_of_code": 720,
      "number_of_classes": 1,
      "number_of_functions": 10
    },
    "dependencies": [
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "anyhow",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "rust_std",
        "is_external": true,
        "line_number": null,
        "name": "std",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "adk_rust",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "futures",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "uuid",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 10,
        "name": "crate::artifacts",
        "path": "crates/cowork-core/src/artifacts.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 11,
        "name": "crate::hitl",
        "path": "crates/cowork-core/src/hitl.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 12,
        "name": "crate::config",
        "path": "crates/cowork-core/src/config.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 13,
        "name": "crate::tools",
        "path": "crates/cowork-core/src/tools.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 14,
        "name": "adk_rust::model::OpenAIClient",
        "path": "adk_rust/model.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 15,
        "name": "adk_rust::prelude::*",
        "path": "adk_rust/prelude.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 16,
        "name": "adk_rust::runner::Runner",
        "path": "adk_rust/runner.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 17,
        "name": "adk_rust::session::InMemorySessionService",
        "path": "adk_rust/session.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 41,
        "name": "crate::agents::BatchContext",
        "path": "crates/cowork-core/src/agents/batch_context.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 157,
        "name": "crate::agents::FileSummaryGenerator",
        "path": "crates/cowork-core/src/agents/file_summary_generator.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 275,
        "name": "crate::agents::TodoListManager",
        "path": "crates/cowork-core/src/agents/todo_list_manager.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 139,
        "name": "crate::agents::LlmAgentBuilder",
        "path": "crates/cowork-core/src/agents/llm_agent_builder.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 42,
        "name": "crate::artifacts::CodeChangeArtifact",
        "path": "crates/cowork-core/src/artifacts.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 182,
        "name": "crate::artifacts::TargetProject",
        "path": "crates/cowork-core/src/artifacts.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 42,
        "name": "crate::artifacts::Change",
        "path": "crates/cowork-core/src/artifacts.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 42,
        "name": "crate::config::LlmConfig",
        "path": "crates/cowork-core/src/config.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 42,
        "name": "crate::hitl::HitlController",
        "path": "crates/cowork-core/src/hitl.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 115,
        "name": "crate::tools::create_file_tools",
        "path": "crates/cowork-core/src/tools.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 42,
        "name": "adk_rust::model::OpenAIConfig",
        "path": "adk_rust/model.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 147,
        "name": "adk_rust::runner::RunnerConfig",
        "path": "adk_rust/runner.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 148,
        "name": "adk_rust::session::CreateRequest",
        "path": "adk_rust/session.rs",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 174,
        "name": "adk_rust::Content",
        "path": "adk_rust/content.rs",
        "version": null
      },
      {
        "dependency_type": "rust_std",
        "is_external": false,
        "line_number": 7,
        "name": "std::sync::Arc",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "rust_std",
        "is_external": false,
        "line_number": 8,
        "name": "std::collections::HashMap",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "rust_std",
        "is_external": false,
        "line_number": 18,
        "name": "futures::StreamExt",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": false,
        "line_number": 148,
        "name": "uuid::Uuid",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "rust_std",
        "is_external": false,
        "line_number": 157,
        "name": "std::fs::read_to_string",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "rust_std",
        "is_external": false,
        "line_number": 204,
        "name": "std::path::Path::exists",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": false,
        "line_number": 104,
        "name": "tracing",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "rust_std",
        "is_external": false,
        "line_number": 105,
        "name": "println",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": false,
        "line_number": 1,
        "name": "anyhow::Result",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "The CodeExecutor is an intelligent agent that automates code generation and file modification using LLM (Large Language Model) agents. It accepts a CodeChangeArtifact containing a list of code changes to implement and uses AI agents to generate and write code files based on instructions and available file system tools. It supports two execution modes: single-agent mode for small changes (≤3 files) and batched sub-agent mode for larger changes (>3 files). Each sub-agent operates in isolation with its own session and context, and the system implements a WatchDog mechanism to ensure alignment with original requirements. It integrates with a human-in-the-loop (HitlController) for user confirmation and updates a TodoList to track execution outcomes. The agent uses a rich set of file manipulation tools (read, write, append, delete, etc.) and dynamically constructs prompts based on project context, file count, and batch context.",
    "interfaces": [
      {
        "description": "Overall report of code execution outcomes",
        "interface_type": "struct",
        "name": "ExecutionReport",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "total_changes",
            "param_type": "usize"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "successful",
            "param_type": "usize"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "failed",
            "param_type": "usize"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "skipped",
            "param_type": "usize"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "details",
            "param_type": "Vec<ChangeResult>"
          }
        ],
        "return_type": null,
        "visibility": "pub"
      },
      {
        "description": "Result of a single file change operation",
        "interface_type": "struct",
        "name": "ChangeResult",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "change",
            "param_type": "Change"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "status",
            "param_type": "ChangeStatus"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "message",
            "param_type": "String"
          }
        ],
        "return_type": null,
        "visibility": "pub"
      },
      {
        "description": "Enumeration of possible execution statuses: Success, Failed, Skipped",
        "interface_type": "enum",
        "name": "ChangeStatus",
        "parameters": [],
        "return_type": null,
        "visibility": "pub"
      },
      {
        "description": "Simplified representation of a code change target",
        "interface_type": "struct",
        "name": "Change",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "path",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "kind",
            "param_type": "String"
          }
        ],
        "return_type": null,
        "visibility": "pub"
      },
      {
        "description": "Internal report for batch execution results (not exposed externally)",
        "interface_type": "struct",
        "name": "BatchExecutionReport",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "successful",
            "param_type": "usize"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "failed",
            "param_type": "usize"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "details",
            "param_type": "Vec<ChangeResult>"
          }
        ],
        "return_type": null,
        "visibility": "private"
      }
    ],
    "responsibilities": [
      "Orchestrating AI-powered code generation based on change plans",
      "Managing execution strategy (single-agent vs. batched sub-agents)",
      "Maintaining context and consistency across batched file operations using WatchDog and file summaries",
      "Integrating with human-in-the-loop (Hitl) for user approval and feedback",
      "Updating and managing TodoList based on execution success/failure"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": "Intelligent agent responsible for analyzing requirement changes and generating incremental code update plans",
      "file_path": "crates/cowork-core/src/agents/code_updater.rs",
      "functions": [
        "new",
        "analyze_changes",
        "diff_requirements",
        "find_affected_files"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "CodeUpdater",
        "RequirementChanges",
        "IncrementalUpdatePlan",
        "AffectedFile",
        "FileImpact",
        "MergeStrategy"
      ],
      "name": "code_updater.rs",
      "source_summary": "use anyhow::Result;\nuse std::sync::Arc;\n\nuse crate::artifacts::*;\nuse crate::memory::ArtifactStore;\nuse crate::config::LlmConfig;\nuse adk_rust::model::{OpenAIClient, OpenAIConfig};\n\n/// Code Updater Agent - 增量修改现有代码\n/// \n/// 核心功能：\n/// 1. 分析需求变更，识别受影响的文件\n/// 2. 生成增量修改计划（而非全量重新生成）\n/// 3. 保护用户手动修改的代码\n/// 4. 支持合并策略（覆盖/合并/保留）\n#[allow(dead_code)]\npub struct CodeUpdater {\n    model: Arc<OpenAIClient>,\n    store: Arc<ArtifactStore>,\n}\n\nimpl CodeUpdater {\n    pub fn new(llm_config: &LlmConfig, store: Arc<ArtifactStore>) -> Result<Self> {\n        let config = OpenAIConfig::compatible(\n            llm_config.api_key.clone(),\n            llm_config.api_base_url.clone(),\n            llm_config.model_name.clone(),\n        );\n        \n        let client = OpenAIClient::new(config)?;\n        \n        Ok(Self {\n            model: Arc::new(client),\n            store,\n        })\n    }\n\n    /// 分析需求变更，生成增量更新计划\n    /// \n    /// # 参数\n    /// - session_id: 会话 ID\n    /// - old_prd: 旧版 PRD\n    /// - new_prd: 新版 PRD\n    /// - existing_code: 现有代码变更记录\n    /// \n    /// # 返回\n    /// - IncrementalUpdatePlan: 增量更新计划\n    pub async fn analyze_changes(\n        &self,\n        session_id: &str,\n        old_prd: &PRD,\n        new_prd: &PRD,\n        existing_code: &CodeChange,\n    ) -> Result<IncrementalUpdatePlan> {\n        tracing::info!(\"Analyzing requirement changes for session {}\", session_id);\n        \n        // 1. 识别新增、修改、删除的需求\n        let req_changes = self.diff_requirements(old_prd, new_prd);\n        \n        // 2. 基于 RequirementMapping 找到受影响的文件\n        let affected_files = self.find_affected_files(&req_changes, existing_code);\n        \n        // 3. 生成修改策略\n        let update_plan = IncrementalUpdatePlan {\n            added_requirements: req_changes.added.clone(),\n            modified_requirements: req_changes.modified.clone(),\n            removed_requirements: req_changes.removed.clone(),\n            affected_files,\n            merge_strategy: MergeStrategy::Smart,  // 默认智能合并\n        };\n        \n        tracing::info!(\n            \"Update plan: {} added, {} modified, {} removed requirements, {} affected files\",\n            update_plan.added_requirements.len(),\n            update_plan.modified_requirements.len(),\n            update_plan.removed_requirements.len(),\n            update_plan.affected_files.len()\n        );\n        \n        Ok(update_plan)\n    }\n    \n    /// Diff 两个 PRD，识别变化\n    fn diff_requirements(&self, old_prd: &PRD, new_prd: &PRD) -> RequirementChanges {\n        let mut added = Vec::new();\n        let mut modified = Vec::new();\n        let mut removed = Vec::new();\n        \n        // 识别新增和修改\n        for new_req in &new_prd.reqs {\n            match old_prd.reqs.iter().find(|r| r.id == new_req.id) {\n                Some(old_req) => {\n                    // 检查是否有修改\n                    if old_req.desc != new_req.desc || old_req.pri != new_req.pri {\n                        modified.push(new_req.clone());\n                    }\n                }\n                None => {\n                    // 新增需求\n                    added.push(new_req.clone());\n                }\n            }\n        }\n        \n        // 识别删除\n        for old_req in &old_prd.reqs {\n            if !new_prd.reqs.iter().any(|r| r.id == old_req.id) {\n                removed.push(old_req.id.clone());\n            }\n        }\n        \n        RequirementChanges {\n            added,\n            modified,\n            removed,\n        }\n    }\n    \n    /// 查找受影响的文件\n    fn find_affected_files(\n        &self,\n        req_changes: &RequirementChanges,\n        existing_code: &CodeChange,\n    ) -> Vec<AffectedFile> {\n        let mut affected = Vec::new();\n        \n        // 遍历所有需求映射\n        for mapping in &existing_code.requirement_mapping {\n            let mut impact = FileImpact::None;\n            \n            // 检查是否被删除\n            if req_changes.removed.contains(&mapping.req_id) {\n                impact = FileImpact::RequirementRemoved;\n            }\n            // 检查是否被修改\n            else if req_changes.modified.iter().any(|r| r.id == mapping.req_id) {\n                impact = FileImpact::RequirementModified;\n            }\n            \n            if impact != FileImpact::None {\n                for file_path in &mapping.files {\n                    affected.push(AffectedFile {\n                        path: file_path.clone(),\n                        impact,\n                        related_requirement: mapping.req_id.clone(),\n                    });\n                }\n            }\n        }\n        \n        // 新增需求需要创建新文件（暂时标记为 None，后续由 CodePlanner 决定）\n        \n        affected\n    }\n}\n\n/// 需求变更记录\n#[derive(Debug, Clone)]\npub struct RequirementChanges {\n    pub added: Vec<Requirement>,\n    pub modified: Vec<Requirement>,\n    pub removed: Vec<String>,  // 需求 ID\n}\n\n/// 增量更新计划\n#[derive(Debug, Clone)]\npub struct IncrementalUpdatePlan {\n    pub added_requirements: Vec<Requirement>,\n    pub modified_requirements: Vec<Requirement>,\n    pub removed_requirements: Vec<String>,\n    pub affected_files: Vec<AffectedFile>,\n    pub merge_strategy: MergeStrategy,\n}\n\n/// 受影响的文件\n#[derive(Debug, Clone)]\npub struct AffectedFile {\n    pub path: String,\n    pub impact: FileImpact,\n    pub related_requirement: String,\n}\n\n/// 文件影响类型\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum FileImpact {\n    None,\n    RequirementModified,  // 需求修改\n    RequirementRemoved,   // 需求删除\n}\n\n/// 合并策略\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum MergeStrategy {\n    /// 覆盖（危险：丢失用户修改）\n    Overwrite,\n    /// 智能合并（保留用户修改，添加新功能）\n    Smart,\n    /// 保留原文件，生成 .new 文件\n    KeepOriginal,\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn test_diff_requirements() {\n        let old_prd = PRD {\n            scope: Scope {\n                g: vec![\"Test\".to_string()],\n                ng: vec![],\n            },\n            reqs: vec![\n                Requirement {\n                    id: \"REQ-001\".to_string(),\n                    desc: \"Old description\".to_string(),\n                    pri: Priority::P0,\n                    req_type: RequirementType::Func,\n                    deps: vec![],\n                    ac: vec![],\n                },\n            ],\n            cons: vec![],\n            hitl: vec![],\n        };\n        \n        let new_prd = PRD {\n            scope: Scope {\n                g: vec![\"Test\".to_string()],\n                ng: vec![],\n            },\n            reqs: vec![\n                Requirement {\n                    id: \"REQ-001\".to_string(),\n                    desc: \"New description\".to_string(),  // 修改\n                    pri: Priority::P0,\n                    req_type: RequirementType::Func,\n                    deps: vec![],\n                    ac: vec![],\n                },\n                Requirement {\n                    id: \"REQ-002\".to_string(),  // 新增\n                    desc: \"New requirement\".to_string(),\n                    pri: Priority::P1,\n                    req_type: RequirementType::Func,\n                    deps: vec![],\n                    ac: vec![],\n                },\n            ],\n            cons: vec![],\n            hitl: vec![],\n        };\n        \n        // 创建临时存储和配置\n        let store = Arc::new(ArtifactStore::new(\".cowork_test\"));\n        let llm_config = LlmConfig {\n            api_key: \"test\".to_string(),\n            api_base_url: \"http://test\".to_string(),\n            model_name: \"test\".to_string(),\n        };\n        \n        let updater = CodeUpdater::new(&llm_config, store).unwrap();\n        let changes = updater.diff_requirements(&old_prd, &new_prd);\n        \n        assert_eq!(changes.added.len(), 1);\n        assert_eq!(changes.added[0].id, \"REQ-002\");\n        \n        assert_eq!(changes.modified.len(), 1);\n        assert_eq!(changes.modified[0].id, \"REQ-001\");\n        \n        assert_eq!(changes.removed.len(), 0);\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 12.0,
      "lines_of_code": 273,
      "number_of_classes": 6,
      "number_of_functions": 4
    },
    "dependencies": [
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": null,
        "name": "anyhow",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "standard_library",
        "is_external": false,
        "line_number": null,
        "name": "std::sync::Arc",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::artifacts",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::memory::ArtifactStore",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::config::LlmConfig",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": null,
        "name": "adk_rust::model::OpenAIClient",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": null,
        "name": "adk_rust::model::OpenAIConfig",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "The CodeUpdater agent is an intelligent component that analyzes requirement changes between different versions of Product Requirement Documents (PRDs) and generates incremental update plans for code modifications. It performs requirement diff analysis, identifies affected files based on requirement mapping, and generates strategic update plans with merge strategies to protect user modifications while implementing necessary changes.",
    "interfaces": [
      {
        "description": "Creates a new CodeUpdater instance with LLM configuration and artifact store",
        "interface_type": "constructor",
        "name": "new",
        "parameters": [
          {
            "description": "Configuration for the language model client",
            "is_optional": false,
            "name": "llm_config",
            "param_type": "&LlmConfig"
          },
          {
            "description": "Shared artifact storage",
            "is_optional": false,
            "name": "store",
            "param_type": "Arc<ArtifactStore>"
          }
        ],
        "return_type": "Result<Self>",
        "visibility": "public"
      },
      {
        "description": "Main entry point for analyzing requirement changes and generating update plans",
        "interface_type": "method",
        "name": "analyze_changes",
        "parameters": [
          {
            "description": "Session identifier for tracking",
            "is_optional": false,
            "name": "session_id",
            "param_type": "&str"
          },
          {
            "description": "Previous version of product requirements",
            "is_optional": false,
            "name": "old_prd",
            "param_type": "&PRD"
          },
          {
            "description": "Updated version of product requirements",
            "is_optional": false,
            "name": "new_prd",
            "param_type": "&PRD"
          },
          {
            "description": "Current code change records",
            "is_optional": false,
            "name": "existing_code",
            "param_type": "&CodeChange"
          }
        ],
        "return_type": "Result<IncrementalUpdatePlan>",
        "visibility": "public"
      },
      {
        "description": "Compares two PRD versions to identify requirement changes",
        "interface_type": "method",
        "name": "diff_requirements",
        "parameters": [
          {
            "description": "Previous requirement document",
            "is_optional": false,
            "name": "old_prd",
            "param_type": "&PRD"
          },
          {
            "description": "New requirement document",
            "is_optional": false,
            "name": "new_prd",
            "param_type": "&PRD"
          }
        ],
        "return_type": "RequirementChanges",
        "visibility": "private"
      },
      {
        "description": "Maps requirement changes to affected code files",
        "interface_type": "method",
        "name": "find_affected_files",
        "parameters": [
          {
            "description": "Detected requirement changes",
            "is_optional": false,
            "name": "req_changes",
            "param_type": "&RequirementChanges"
          },
          {
            "description": "Current code mapping information",
            "is_optional": false,
            "name": "existing_code",
            "param_type": "&CodeChange"
          }
        ],
        "return_type": "Vec<AffectedFile>",
        "visibility": "private"
      }
    ],
    "responsibilities": [
      "Analyze requirement changes between PRD versions and identify added, modified, and removed requirements",
      "Generate incremental update plans that minimize code changes and preserve user modifications",
      "Map requirement changes to affected code files using requirement mapping data",
      "Provide intelligent merge strategies to handle code modifications safely",
      "Coordinate with LLM models for intelligent code analysis and planning"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": null,
      "file_path": "crates/cowork-core/src/agents/idea_intake.rs",
      "functions": [
        "IdeaIntakeAgent::new",
        "IdeaIntakeAgent::execute"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "IdeaIntakeAgent"
      ],
      "name": "idea_intake.rs",
      "source_summary": "use anyhow::Result;\nuse adk_rust::prelude::*;\nuse adk_rust::model::{OpenAIClient, OpenAIConfig};\nuse adk_rust::runner::{Runner, RunnerConfig};\nuse adk_rust::session::{InMemorySessionService, CreateRequest, SessionService, GetRequest};\nuse futures::StreamExt;\nuse std::collections::HashMap;\nuse std::sync::Arc;\n\nuse crate::artifacts::*;\nuse crate::memory::ArtifactStore;\nuse crate::config::LlmConfig;\n\n/// IDEA Intake Agent - 将用户输入转换为结构化的 IdeaSpec\npub struct IdeaIntakeAgent {\n    model: Arc<OpenAIClient>,\n    store: Arc<ArtifactStore>,\n}\n\nimpl IdeaIntakeAgent {\n    pub fn new(llm_config: &LlmConfig, store: Arc<ArtifactStore>) -> Result<Self> {\n        // Create OpenAI-compatible client using the compatible() constructor\n        // This sets the custom base_url for private deployment\n        let config = OpenAIConfig::compatible(\n            llm_config.api_key.clone(),\n            llm_config.api_base_url.clone(),\n            llm_config.model_name.clone(),\n        );\n        \n        tracing::info!(\"Creating OpenAI-compatible client\");\n        tracing::info!(\"  API Base: {}\", llm_config.api_base_url);\n        tracing::info!(\"  Model: {}\", llm_config.model_name);\n        tracing::info!(\"  API Key: {}...\", &llm_config.api_key[..10]);\n        \n        let model = OpenAIClient::new(config)?;\n\n        Ok(Self {\n            model: Arc::new(model),\n            store,\n        })\n    }\n\n    pub async fn execute(&self, session_id: &str, user_input: &str) -> Result<IdeaSpecArtifact> {\n        tracing::info!(\"IdeaIntakeAgent: processing user input for session {}\", session_id);\n\n        // Define the output schema for IdeaSpec\n        // Note: For OpenAI-compatible APIs that don't support response_format,\n        // this schema is primarily used for documentation and potential guardrail validation.\n        // The actual structure is enforced through the instruction prompt.\n        let output_schema = serde_json::json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"bg\": {\n                    \"type\": \"string\",\n                    \"description\": \"Background (1-2 sentences describing the context)\"\n                },\n                \"g\": {\n                    \"type\": \"array\",\n                    \"items\": { \"type\": \"string\" },\n                    \"description\": \"Goals (list of project objectives)\"\n                },\n                \"ng\": {\n                    \"type\": \"array\",\n                    \"items\": { \"type\": \"string\" },\n                    \"description\": \"Non-goals (what's explicitly out of scope)\"\n                },\n                \"c\": {\n                    \"type\": \"array\",\n                    \"items\": { \"type\": \"string\" },\n                    \"description\": \"Constraints (technical/business limitations)\"\n                },\n                \"sc\": {\n                    \"type\": \"array\",\n                    \"items\": { \"type\": \"string\" },\n                    \"description\": \"Success criteria (measurable outcomes)\"\n                },\n                \"r\": {\n                    \"type\": \"array\",\n                    \"items\": { \"type\": \"string\" },\n                    \"description\": \"Risks (potential issues)\"\n                },\n                \"q\": {\n                    \"type\": \"array\",\n                    \"items\": { \"type\": \"string\" },\n                    \"description\": \"Questions (unresolved points needing clarification)\"\n                }\n            },\n            \"required\": [\"bg\", \"g\", \"ng\", \"c\", \"sc\", \"r\", \"q\"]\n        });\n\n        // Build agent with output_schema and detailed instruction\n        // Since the OpenAI-compatible API may not support response_format,\n        // we provide explicit JSON structure in the instruction.\n        let agent = Arc::new(\n            LlmAgentBuilder::new(\"idea_intake\")\n                .description(\"Convert user IDEA into structured IdeaSpec\")\n                .instruction(\n                    r#\"You are an IDEA analyzer. Extract and structure the user's idea into a JSON object.\n\n**Required JSON Structure:**\n{\n  \"bg\": \"string - Background context in 1-2 sentences\",\n  \"g\": [\"array of strings - Project goals/objectives\"],\n  \"ng\": [\"array of strings - Non-goals (out of scope items)\"],\n  \"c\": [\"array of strings - Constraints (technical/business limitations)\"],\n  \"sc\": [\"array of strings - Success criteria (measurable outcomes)\"],\n  \"r\": [\"array of strings - Risks (potential issues)\"],\n  \"q\": [\"array of strings - Questions (unresolved points)\"]\n}\n\n**Output Requirements:**\n1. Respond with ONLY valid JSON (no markdown, no code blocks, no additional text)\n2. All fields are required (use empty arrays if no items)\n3. Be concise - use short phrases\n4. Ensure all array items are non-empty strings\n\n**Example:**\n{\n  \"bg\": \"Build a landing page to showcase product features\",\n  \"g\": [\"Attract potential customers\", \"Explain core value proposition\"],\n  \"ng\": [\"E-commerce functionality\", \"User authentication\"],\n  \"c\": [\"Static HTML only\", \"Load time < 3s\"],\n  \"sc\": [\"Mobile responsive\", \"90+ Lighthouse score\"],\n  \"r\": [\"Content may become outdated\"],\n  \"q\": [\"What color scheme?\", \"Need multilingual support?\"]\n}\"#,\n                )\n                .model(self.model.clone())\n                .output_schema(output_schema)  // For documentation and future guardrail validation\n                .output_key(\"idea_spec_raw\")\n                .build()?,\n        );\n\n        // Initialize session service and create a session\n        let session_service = Arc::new(InMemorySessionService::new());\n        let app_name = \"cowork\".to_string();\n        let user_id = session_id.to_string();\n\n        let session = session_service\n            .create(CreateRequest {\n                app_name: app_name.clone(),\n                user_id: user_id.clone(),\n                session_id: Some(session_id.to_string()),\n                state: HashMap::new(),\n            })\n            .await?;\n\n        tracing::debug!(\"Session created: {}\", session.id());\n\n        // Create the Runner with agent in config\n        let runner = Runner::new(RunnerConfig {\n            app_name: app_name.clone(),\n            agent: agent.clone(),\n            session_service: session_service.clone(),\n            artifact_service: None,\n            memory_service: None,\n            run_config: None,\n        })?;\n\n        // Define the input content\n        let input_content = Content::new(\"user\").with_text(user_input);\n\n        tracing::info!(\"Invoking LLM agent...\");\n\n        // Run the agent and consume event stream\n        let mut event_stream = runner\n            .run(user_id.clone(), session_id.to_string(), input_content)\n            .await?;\n\n        // Consume the event stream to ensure agent execution completes\n        while let Some(event_result) = event_stream.next().await {\n            match event_result {\n                Ok(event) => {\n                    tracing::debug!(\"Event received: {:?}\", event);\n                    // Optionally process LLM responses\n                    if let Some(llm_response_content) = event.llm_response.content {\n                        for part in llm_response_content.parts {\n                            if let Some(text) = part.text() {\n                                tracing::debug!(\"LLM output: {}\", text);\n                            }\n                        }\n                    }\n                }\n                Err(e) => {\n                    tracing::error!(\"Error during agent execution: {}\", e);\n                    return Err(anyhow::anyhow!(\"Agent execution failed: {}\", e));\n                }\n            }\n        }\n\n        tracing::info!(\"Agent execution complete\");\n\n        // Retrieve the session state and extract the structured data\n        let updated_session = session_service\n            .get(GetRequest {\n                user_id: user_id.clone(),\n                session_id: session_id.to_string(),\n                app_name: app_name.clone(),\n                after: None,\n                num_recent_events: None,\n            })\n            .await?;\n\n        let state = updated_session.state();\n\n        // Extract the output from session state\n        let raw_output = state\n            .get(\"idea_spec_raw\")\n            .ok_or_else(|| anyhow::anyhow!(\"No output from agent (key 'idea_spec_raw' not found)\"))?;\n\n        tracing::debug!(\"Raw output from session state: {}\", raw_output);\n\n        // Parse the JSON output into IdeaSpec\n        // The LLM might return a JSON string or a JSON object\n        let idea_spec: IdeaSpec = match raw_output {\n            serde_json::Value::String(json_str) => {\n                // If it's a string, parse it first\n                tracing::debug!(\"Output is a JSON string, parsing...\");\n                serde_json::from_str(json_str.as_str())\n                    .map_err(|e| anyhow::anyhow!(\"Failed to parse JSON string: {}\", e))?\n            }\n            value => {\n                // If it's already a structured value, deserialize directly\n                tracing::debug!(\"Output is a structured JSON value\");\n                serde_json::from_value(value.clone())\n                    .map_err(|e| anyhow::anyhow!(\"Failed to deserialize JSON value: {}\", e))?\n            }\n        };\n\n        tracing::info!(\"Successfully parsed IdeaSpec\");\n\n        // Create artifact\n        let summary = vec![\n            format!(\"Background: {}\", idea_spec.bg),\n            format!(\"Goals: {}\", idea_spec.g.len()),\n            format!(\"Non-Goals: {}\", idea_spec.ng.len()),\n            format!(\"Constraints: {}\", idea_spec.c.len()),\n        ];\n\n        let artifact = ArtifactEnvelope::new(session_id.to_string(), Stage::IdeaIntake, idea_spec)\n            .with_summary(summary);\n\n        // Save to store\n        self.store.put(session_id, Stage::IdeaIntake, &artifact)?;\n\n        tracing::info!(\"IdeaSpec artifact saved successfully\");\n\n        Ok(artifact)\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 13.0,
      "lines_of_code": 250,
      "number_of_classes": 1,
      "number_of_functions": 2
    },
    "dependencies": [
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "anyhow",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "adk_rust",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "futures",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "standard_library",
        "is_external": false,
        "line_number": null,
        "name": "std",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "serde_json",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::artifacts",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::memory::ArtifactStore",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::config::LlmConfig",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "LlmAgentBuilder",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "Content",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "ArtifactEnvelope",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "The IdeaIntakeAgent is an intelligent agent responsible for transforming unstructured user input into a structured IdeaSpec JSON object. It leverages an OpenAI-compatible LLM via the adk_rust framework to parse natural language descriptions of ideas into predefined fields: background, goals, non-goals, constraints, success criteria, risks, and questions. The agent operates within a session context, using an in-memory session service to persist intermediate state. After the LLM processes the input, the agent extracts the raw JSON output from the session state, parses it into a typed IdeaSpec structure, validates its integrity, generates a summary, and saves the result as an ArtifactEnvelope to an ArtifactStore for downstream use in the cowork system. This component acts as the first structured interpretation layer between user ideation and system-driven planning.",
    "interfaces": [
      {
        "description": "Main agent struct holding LLM client and artifact storage reference",
        "interface_type": "struct",
        "name": "IdeaIntakeAgent",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "model",
            "param_type": "Arc<OpenAIClient>"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "store",
            "param_type": "Arc<ArtifactStore>"
          }
        ],
        "return_type": null,
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Transform unstructured user input into structured IdeaSpec JSON",
      "Manage LLM agent lifecycle and session context",
      "Parse and validate LLM output into strongly-typed domain model",
      "Persist structured output as artifacts in the system",
      "Handle error conditions and provide tracing for observability"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": "Module file defining the core agent framework and exposing all agent implementations",
      "file_path": "crates/cowork-core/src/agents/mod.rs",
      "functions": [
        "Agent::name()",
        "Agent::execute()"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "Agent",
        "AgentContext",
        "AgentOutput"
      ],
      "name": "mod.rs",
      "source_summary": "use anyhow::Result;\nuse async_trait::async_trait;\n\npub mod idea_intake;\nmod prd_agent;\nmod design_agent;\nmod plan_agent;\nmod code_planner;\nmod code_executor;\nmod check_agent;\nmod feedback_agent;\nmod delivery_agent;\npub mod watchdog;\npub mod code_updater;\npub mod error_analyzer;\npub mod batch_context;\npub mod todo_manager;\npub mod code_plan_normalizer;\n\npub use idea_intake::IdeaIntakeAgent;\npub use prd_agent::PrdAgent;\npub use design_agent::DesignAgent;\npub use plan_agent::PlanAgent;\npub use code_planner::CodePlanner;\npub use code_executor::{CodeExecutor, ExecutionReport, ChangeResult, ChangeStatus};\npub use check_agent::CheckAgent;\npub use feedback_agent::FeedbackAgent;\npub use delivery_agent::DeliveryAgent;\npub use watchdog::WatchDogAgent;\npub use code_updater::CodeUpdater;\npub use error_analyzer::{ErrorAnalyzer, ErrorAnalysis};\npub use batch_context::{BatchContext, FileContext, FileSummaryGenerator};\npub use todo_manager::{TodoListManager, TodoStatusReport};\n\n/// Agent trait（通用接口）\n#[async_trait]\npub trait Agent: Send + Sync {\n    /// Agent 名称\n    fn name(&self) -> &str;\n\n    /// 执行 Agent 逻辑\n    async fn execute(&self, context: &AgentContext) -> Result<AgentOutput>;\n}\n\n/// Agent 执行上下文\n#[derive(Debug, Clone)]\npub struct AgentContext {\n    pub session_id: String,\n    pub input: String,\n    pub prev_artifacts: Vec<String>,\n}\n\n/// Agent 输出\n#[derive(Debug, Clone)]\npub struct AgentOutput {\n    pub content: String,\n    pub artifact_id: Option<String>,\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 1.0,
      "lines_of_code": 58,
      "number_of_classes": 2,
      "number_of_functions": 2
    },
    "dependencies": [
      {
        "dependency_type": "external_crate",
        "is_external": true,
        "line_number": 1,
        "name": "anyhow",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "external_crate",
        "is_external": true,
        "line_number": 2,
        "name": "async_trait",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal_module",
        "is_external": false,
        "line_number": 4,
        "name": "idea_intake",
        "path": "./idea_intake",
        "version": null
      },
      {
        "dependency_type": "internal_module",
        "is_external": false,
        "line_number": 5,
        "name": "prd_agent",
        "path": "./prd_agent",
        "version": null
      },
      {
        "dependency_type": "internal_module",
        "is_external": false,
        "line_number": 6,
        "name": "design_agent",
        "path": "./design_agent",
        "version": null
      },
      {
        "dependency_type": "internal_module",
        "is_external": false,
        "line_number": 7,
        "name": "plan_agent",
        "path": "./plan_agent",
        "version": null
      },
      {
        "dependency_type": "internal_module",
        "is_external": false,
        "line_number": 8,
        "name": "code_planner",
        "path": "./code_planner",
        "version": null
      },
      {
        "dependency_type": "internal_module",
        "is_external": false,
        "line_number": 9,
        "name": "code_executor",
        "path": "./code_executor",
        "version": null
      },
      {
        "dependency_type": "internal_module",
        "is_external": false,
        "line_number": 10,
        "name": "check_agent",
        "path": "./check_agent",
        "version": null
      },
      {
        "dependency_type": "internal_module",
        "is_external": false,
        "line_number": 11,
        "name": "feedback_agent",
        "path": "./feedback_agent",
        "version": null
      },
      {
        "dependency_type": "internal_module",
        "is_external": false,
        "line_number": 12,
        "name": "delivery_agent",
        "path": "./delivery_agent",
        "version": null
      },
      {
        "dependency_type": "internal_module",
        "is_external": false,
        "line_number": 13,
        "name": "watchdog",
        "path": "./watchdog",
        "version": null
      },
      {
        "dependency_type": "internal_module",
        "is_external": false,
        "line_number": 14,
        "name": "code_updater",
        "path": "./code_updater",
        "version": null
      },
      {
        "dependency_type": "internal_module",
        "is_external": false,
        "line_number": 15,
        "name": "error_analyzer",
        "path": "./error_analyzer",
        "version": null
      },
      {
        "dependency_type": "internal_module",
        "is_external": false,
        "line_number": 16,
        "name": "batch_context",
        "path": "./batch_context",
        "version": null
      },
      {
        "dependency_type": "internal_module",
        "is_external": false,
        "line_number": 17,
        "name": "todo_manager",
        "path": "./todo_manager",
        "version": null
      },
      {
        "dependency_type": "internal_module",
        "is_external": false,
        "line_number": 18,
        "name": "code_plan_normalizer",
        "path": "./code_plan_normalizer",
        "version": null
      }
    ],
    "detailed_description": "This module serves as the central coordination point for all intelligent agents in the cowork-core system. It defines the foundational Agent trait that all specialized agents must implement, providing a standardized interface for agent execution. The module acts as a facade that exposes all agent implementations while maintaining clean separation of concerns through proper module organization. It establishes the execution context and output format that enables consistent agent behavior across the system.",
    "interfaces": [
      {
        "description": "Core trait defining the interface that all agents must implement",
        "interface_type": "trait",
        "name": "Agent",
        "parameters": [
          {
            "description": "Reference to the agent instance",
            "is_optional": false,
            "name": "self",
            "param_type": "&Self"
          },
          {
            "description": "Execution context containing session and input data",
            "is_optional": false,
            "name": "context",
            "param_type": "&AgentContext"
          }
        ],
        "return_type": "Result<AgentOutput>",
        "visibility": "public"
      },
      {
        "description": "Container for execution context data",
        "interface_type": "struct",
        "name": "AgentContext",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Standardized output format for agent execution results",
        "interface_type": "struct",
        "name": "AgentOutput",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Defining the core Agent trait interface for uniform agent execution",
      "Providing execution context and output structures for agent operations",
      "Exposing all specialized agent implementations through public module declarations",
      "Centralizing agent imports and dependencies management",
      "Maintaining agent interoperability through standardized interfaces"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": null,
      "file_path": "crates/cowork-core/src/agents/error_analyzer.rs",
      "functions": [
        "analyze",
        "extract_file_path",
        "extract_files_from_compilation_errors"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "ErrorAnalysis"
      ],
      "name": "error_analyzer.rs",
      "source_summary": "use std::collections::HashMap;\n\nuse crate::artifacts::*;\n\n/// 错误分析器 - 从 CheckReport 中提取关键信息\npub struct ErrorAnalyzer;\n\nimpl ErrorAnalyzer {\n    /// 分析检查报告，提取受影响的文件和错误摘要\n    pub fn analyze(check_report: &CheckReport) -> ErrorAnalysis {\n        let mut affected_files: HashMap<String, Vec<String>> = HashMap::new();\n        let mut error_count_by_severity: HashMap<String, usize> = HashMap::new();\n        \n        // 遍历所有 issues\n        for issue in &check_report.issues {\n            // 统计严重程度\n            *error_count_by_severity.entry(issue.sev.clone()).or_insert(0) += 1;\n            \n            // 从 issue.id 中提取文件路径\n            // 例如: \"ISSUE-FILE-app.rs\" -> \"app.rs\"\n            //      \"ISSUE-COMPILE-RUST\" -> 影响所有文件\n            //      \"ISSUE-SYNTAX-PY-main.py\" -> \"main.py\"\n            let file_path = Self::extract_file_path(&issue.id);\n            \n            if !file_path.is_empty() {\n                affected_files.entry(file_path.clone())\n                    .or_insert_with(Vec::new)\n                    .push(format!(\"[{}] {}\", issue.sev, issue.desc));\n            }\n        }\n        \n        // 生成摘要\n        let total_errors = check_report.issues.len();\n        let critical_errors = error_count_by_severity.get(\"error\").copied().unwrap_or(0);\n        let warnings = error_count_by_severity.get(\"warning\").copied().unwrap_or(0);\n        \n        let summary = if total_errors == 0 {\n            \"All checks passed\".to_string()\n        } else {\n            format!(\n                \"{} total issues ({} errors, {} warnings)\",\n                total_errors, critical_errors, warnings\n            )\n        };\n        \n        // 提取详细错误信息（用于传递给重试）\n        let detailed_errors = check_report.issues.iter()\n            .filter(|issue| issue.sev == \"error\")\n            .map(|issue| format!(\"- {}: {}\\n  Fix hint: {}\", issue.id, issue.desc, issue.fix_hint))\n            .collect::<Vec<_>>()\n            .join(\"\\n\\n\");\n        \n        ErrorAnalysis {\n            affected_files: affected_files.keys().cloned().collect(),\n            error_details_by_file: affected_files,\n            summary,\n            detailed_errors,\n            has_critical_errors: critical_errors > 0,\n        }\n    }\n    \n    /// 从 issue ID 中提取文件路径\n    fn extract_file_path(issue_id: &str) -> String {\n        // ISSUE-FILE-app.rs -> app.rs\n        if issue_id.starts_with(\"ISSUE-FILE-\") {\n            return issue_id.strip_prefix(\"ISSUE-FILE-\").unwrap_or(\"\").to_string();\n        }\n        \n        // ISSUE-EMPTY-src/main.rs -> src/main.rs\n        if issue_id.starts_with(\"ISSUE-EMPTY-\") {\n            return issue_id.strip_prefix(\"ISSUE-EMPTY-\").unwrap_or(\"\").to_string();\n        }\n        \n        // ISSUE-TODO-app.js -> app.js\n        if issue_id.starts_with(\"ISSUE-TODO-\") {\n            return issue_id.strip_prefix(\"ISSUE-TODO-\").unwrap_or(\"\").to_string();\n        }\n        \n        // ISSUE-SYNTAX-PY-main.py -> main.py\n        if issue_id.starts_with(\"ISSUE-SYNTAX-PY-\") {\n            return issue_id.strip_prefix(\"ISSUE-SYNTAX-PY-\").unwrap_or(\"\").to_string();\n        }\n        \n        // ISSUE-COMPILE-RUST -> 空（影响多个文件）\n        String::new()\n    }\n    \n    /// 从编译错误中智能提取文件路径\n    pub fn extract_files_from_compilation_errors(stderr: &str) -> Vec<String> {\n        let mut files = Vec::new();\n        \n        // Rust: error[E0XXX]: ... --> src/main.rs:42:5\n        for line in stderr.lines() {\n            if line.contains(\" --> \") {\n                if let Some(pos) = line.find(\" --> \") {\n                    let path_part = &line[pos + 5..];\n                    if let Some(colon_pos) = path_part.find(':') {\n                        let file_path = path_part[..colon_pos].trim().to_string();\n                        if !files.contains(&file_path) {\n                            files.push(file_path);\n                        }\n                    }\n                }\n            }\n        }\n        \n        // Python: File \"main.py\", line 10\n        for line in stderr.lines() {\n            if line.contains(\"File \\\"\") {\n                if let Some(start) = line.find(\"File \\\"\") {\n                    let rest = &line[start + 6..];\n                    if let Some(end) = rest.find('\"') {\n                        let file_path = rest[..end].to_string();\n                        if !files.contains(&file_path) {\n                            files.push(file_path);\n                        }\n                    }\n                }\n            }\n        }\n        \n        files\n    }\n}\n\n/// 错误分析结果\n#[derive(Debug, Clone)]\npub struct ErrorAnalysis {\n    /// 受影响的文件列表\n    pub affected_files: Vec<String>,\n    \n    /// 每个文件的详细错误\n    pub error_details_by_file: HashMap<String, Vec<String>>,\n    \n    /// 错误摘要\n    pub summary: String,\n    \n    /// 详细错误信息（用于传递给 Agent）\n    pub detailed_errors: String,\n    \n    /// 是否有严重错误\n    pub has_critical_errors: bool,\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn test_extract_file_path() {\n        assert_eq!(ErrorAnalyzer::extract_file_path(\"ISSUE-FILE-app.rs\"), \"app.rs\");\n        assert_eq!(ErrorAnalyzer::extract_file_path(\"ISSUE-EMPTY-src/main.rs\"), \"src/main.rs\");\n        assert_eq!(ErrorAnalyzer::extract_file_path(\"ISSUE-TODO-index.html\"), \"index.html\");\n        assert_eq!(ErrorAnalyzer::extract_file_path(\"ISSUE-COMPILE-RUST\"), \"\");\n    }\n    \n    #[test]\n    fn test_extract_files_from_compilation_errors() {\n        let rust_error = r#\"\nerror[E0425]: cannot find value `x` in this scope\n --> src/main.rs:42:5\n  |\n42 |     x + 1\n  |     ^ not found in this scope\n\nerror[E0308]: mismatched types\n --> src/lib.rs:10:20\n  |\n10 |     let y: i32 = \"hello\";\n   |                  ^^^^^^^ expected `i32`, found `&str`\n\"#;\n        \n        let files = ErrorAnalyzer::extract_files_from_compilation_errors(rust_error);\n        assert_eq!(files, vec![\"src/main.rs\", \"src/lib.rs\"]);\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 18.0,
      "lines_of_code": 176,
      "number_of_classes": 2,
      "number_of_functions": 3
    },
    "dependencies": [
      {
        "dependency_type": "struct",
        "is_external": false,
        "line_number": 6,
        "name": "CheckReport",
        "path": "crate::artifacts::*",
        "version": null
      },
      {
        "dependency_type": "struct",
        "is_external": false,
        "line_number": 15,
        "name": "ErrorAnalysis",
        "path": "crate::artifacts::*",
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 1,
        "name": "std::collections::HashMap",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "The ErrorAnalyzer is an intelligent agent component responsible for parsing and interpreting error reports generated by code analysis tools. It processes CheckReport data to extract meaningful insights about affected files, error severity distribution, and detailed error messages. The component provides two main analytical functions: one that processes structured CheckReport data to generate a comprehensive ErrorAnalysis summary, and another that parses raw compiler stderr output to extract file paths from compilation errors in Rust and Python. The component is designed to support automated code review and repair workflows by converting raw diagnostic data into structured, actionable information.",
    "interfaces": [
      {
        "description": "Data structure representing the outcome of error analysis, containing file-level and summary error information",
        "interface_type": "struct",
        "name": "ErrorAnalysis",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "affected_files",
            "param_type": "Vec<String>"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "error_details_by_file",
            "param_type": "HashMap<String, Vec<String>>"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "summary",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "detailed_errors",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "has_critical_errors",
            "param_type": "bool"
          }
        ],
        "return_type": null,
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Parse CheckReport to identify affected files and categorize errors by severity",
      "Generate human-readable summaries and detailed error reports for agent consumption",
      "Extract file paths from raw compiler stderr output for Rust and Python languages",
      "Support error-driven automation by providing structured error context for retry or repair logic",
      "Maintain clean separation between error parsing logic and reporting structure"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": null,
      "file_path": "crates/cowork-core/src/agents/plan_agent.rs",
      "functions": [
        "PlanAgent::new",
        "PlanAgent::execute"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "OpenAIClient",
        "ArtifactStore",
        "LlmConfig",
        "InMemorySessionService",
        "Runner",
        "Content",
        "ArtifactEnvelope",
        "Plan",
        "DesignDocArtifact",
        "PlanArtifact",
        "Stage"
      ],
      "name": "plan_agent.rs",
      "source_summary": "use anyhow::Result;\nuse adk_rust::prelude::*;\nuse adk_rust::model::{OpenAIClient, OpenAIConfig};\nuse adk_rust::runner::{Runner, RunnerConfig};\nuse adk_rust::session::{InMemorySessionService, CreateRequest, SessionService, GetRequest};\nuse futures::StreamExt;\nuse std::collections::HashMap;\nuse std::sync::Arc;\n\nuse crate::artifacts::*;\nuse crate::memory::ArtifactStore;\nuse crate::config::LlmConfig;\n\n/// Plan Agent - 基于 Design 生成实施计划\npub struct PlanAgent {\n    model: Arc<OpenAIClient>,\n    store: Arc<ArtifactStore>,\n}\n\nimpl PlanAgent {\n    pub fn new(llm_config: &LlmConfig, store: Arc<ArtifactStore>) -> Result<Self> {\n        let config = OpenAIConfig::compatible(\n            llm_config.api_key.clone(),\n            llm_config.api_base_url.clone(),\n            llm_config.model_name.clone(),\n        );\n        \n        tracing::info!(\"Creating Plan Agent with OpenAI-compatible client )\");\n        \n        let model = OpenAIClient::new(config)?;\n\n        Ok(Self {\n            model: Arc::new(model),\n            store,\n        })\n    }\n\n    pub async fn execute(&self, session_id: &str, design_artifact: &DesignDocArtifact) -> Result<PlanArtifact> {\n        tracing::info!(\"PlanAgent: generating implementation plan for session {}\", session_id);\n\n        let output_schema = serde_json::json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"c4\": {\n                    \"type\": \"object\",\n                    \"properties\": {\n                        \"context\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}},\n                        \"containers\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}},\n                        \"components\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}},\n                        \"code\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}\n                    },\n                    \"required\": [\"context\", \"containers\", \"components\", \"code\"]\n                },\n                \"tasks\": {\n                    \"type\": \"array\",\n                    \"items\": {\n                        \"type\": \"object\",\n                        \"properties\": {\n                            \"id\": {\"type\": \"string\"},\n                            \"pri\": {\"type\": \"string\", \"enum\": [\"p0\", \"p1\", \"p2\"]},\n                            \"desc\": {\"type\": \"string\"},\n                            \"deps\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}},\n                            \"out\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}\n                        },\n                        \"required\": [\"id\", \"pri\", \"desc\", \"deps\", \"out\"]\n                    }\n                },\n                \"milestones\": {\n                    \"type\": \"array\",\n                    \"items\": {\n                        \"type\": \"object\",\n                        \"properties\": {\n                            \"id\": {\"type\": \"string\"},\n                            \"desc\": {\"type\": \"string\"},\n                            \"done_when\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}\n                        },\n                        \"required\": [\"id\", \"desc\", \"done_when\"]\n                    }\n                },\n                \"todo_list\": {\n                    \"type\": \"object\",\n                    \"properties\": {\n                        \"items\": {\n                            \"type\": \"array\",\n                            \"items\": {\n                                \"type\": \"object\",\n                                \"properties\": {\n                                    \"id\": {\"type\": \"string\"},\n                                    \"description\": {\"type\": \"string\"},\n                                    \"status\": {\"type\": \"string\", \"enum\": [\"pending\", \"in_progress\", \"completed\", \"blocked\"]},\n                                    \"related_requirements\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}},\n                                    \"related_files\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}},\n                                    \"verification_method\": {\"type\": \"string\"}\n                                },\n                                \"required\": [\"id\", \"description\", \"status\", \"related_requirements\", \"related_files\", \"verification_method\"]\n                            }\n                        }\n                    },\n                    \"required\": [\"items\"]\n                }\n            },\n            \"required\": [\"c4\", \"tasks\", \"milestones\", \"todo_list\"]\n        });\n\n        let context = format!(\n            r#\"Based on the following Design Document, create an implementation plan.\n\n**CLI Modes:**\n{}\n\n**Workflow Stages:**\n{}\n\n**Architecture Layers:**\n{}\n\n**Architecture Components:**\n{}\n\nCreate a detailed C4 model and task breakdown.\"#,\n            design_artifact.data.cli.modes.join(\", \"),\n            design_artifact.data.wf.stages.join(\" → \"),\n            design_artifact.data.arch.layers.join(\", \"),\n            design_artifact.data.arch.comps.join(\"\\n\"),\n        );\n\n        let agent = Arc::new(\n            LlmAgentBuilder::new(\"plan_generator\")\n                .description(\"Generate implementation plan from design document\")\n                .instruction(\n                    r#\"You are a technical planner. Create a structured implementation plan using C4 model and TodoList for task tracking.\n\n**Required JSON Structure:**\n{\n  \"c4\": {\n    \"context\": [\"system context descriptions\"],\n    \"containers\": [\"container (app/service/db) descriptions\"],\n    \"components\": [\"component descriptions\"],\n    \"code\": [\"key code structure descriptions\"]\n  },\n  \"tasks\": [\n    {\n      \"id\": \"TASK-001\",\n      \"pri\": \"p0|p1|p2\",\n      \"desc\": \"task description\",\n      \"deps\": [\"TASK-XXX dependencies\"],\n      \"out\": [\"expected outputs/deliverables\"]\n    }\n  ],\n  \"milestones\": [\n    {\n      \"id\": \"M1\",\n      \"desc\": \"milestone description\",\n      \"done_when\": [\"completion criteria\"]\n    }\n  ],\n  \"todo_list\": {\n    \"items\": [\n      {\n        \"id\": \"TODO-001\",\n        \"description\": \"Specific actionable task\",\n        \"status\": \"pending\",\n        \"related_requirements\": [\"REQ-001\"],\n        \"related_files\": [\"path/to/file.ext\"],\n        \"verification_method\": \"unit_test|manual_test|code_review\"\n      }\n    ]\n  }\n}\n\n**TodoList Generation Guidelines:**\n1. Break down tasks into specific, actionable TodoItems\n2. Each TodoItem should map to specific requirements (from PRD)\n3. List expected files to be created/modified\n4. Specify clear verification methods\n5. All todos should start with status \"pending\"\n6. Ensure todos are ordered by dependencies\n\n**Output Requirements:**\n1. Respond with ONLY valid JSON\n2. All arrays must be present (including todo_list)\n3. Tasks and todos should be ordered by dependencies\n4. Each milestone should have clear, testable criteria\n5. C4 model should be comprehensive yet concise\n6. TodoList should cover ALL major implementation work\"#,\n                )\n                .model(self.model.clone())\n                .output_schema(output_schema)\n                .output_key(\"plan_raw\")\n                .build()?,\n        );\n\n        let session_service = Arc::new(InMemorySessionService::new());\n        let app_name = \"cowork\".to_string();\n        let user_id = session_id.to_string();\n\n        let _session = session_service\n            .create(CreateRequest {\n                app_name: app_name.clone(),\n                user_id: user_id.clone(),\n                session_id: Some(session_id.to_string()),\n                state: HashMap::new(),\n            })\n            .await?;\n\n        let runner = Runner::new(RunnerConfig {\n            app_name: app_name.clone(),\n            agent: agent.clone(),\n            session_service: session_service.clone(),\n            artifact_service: None,\n            memory_service: None,\n            run_config: None,\n        })?;\n\n        let input_content = Content::new(\"user\").with_text(&context);\n\n        tracing::info!(\"Invoking Plan generation agent...\");\n\n        let mut event_stream = runner\n            .run(user_id.clone(), session_id.to_string(), input_content)\n            .await?;\n\n        while let Some(event_result) = event_stream.next().await {\n            match event_result {\n                Ok(_event) => {},\n                Err(e) => {\n                    tracing::error!(\"Error during plan generation: {}\", e);\n                    return Err(anyhow::anyhow!(\"Plan generation failed: {}\", e));\n                }\n            }\n        }\n\n        tracing::info!(\"Plan generation complete\");\n\n        let updated_session = session_service\n            .get(GetRequest {\n                user_id: user_id.clone(),\n                session_id: session_id.to_string(),\n                app_name: app_name.clone(),\n                after: None,\n                num_recent_events: None,\n            })\n            .await?;\n\n        let state = updated_session.state();\n        let raw_output = state\n            .get(\"plan_raw\")\n            .ok_or_else(|| anyhow::anyhow!(\"No output from Plan agent\"))?;\n\n        let plan: Plan = match raw_output {\n            serde_json::Value::String(json_str) => {\n                serde_json::from_str(json_str.as_str())?\n            }\n            value => {\n                serde_json::from_value(value.clone())?\n            }\n        };\n\n        tracing::info!(\"Successfully parsed Plan\");\n\n        let summary = vec![\n            format!(\"C4 Context: {} items\", plan.c4.context.len()),\n            format!(\"Tasks: {} total\", plan.tasks.len()),\n            format!(\"Milestones: {}\", plan.milestones.len()),\n        ];\n\n        let artifact = ArtifactEnvelope::new(session_id.to_string(), Stage::Plan, plan)\n            .with_summary(summary)\n            .with_prev(vec![design_artifact.meta.artifact_id.clone()]);\n\n        self.store.put(session_id, Stage::Plan, &artifact)?;\n\n        tracing::info!(\"Plan artifact saved successfully\");\n\n        Ok(artifact)\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 6.0,
      "lines_of_code": 277,
      "number_of_classes": 1,
      "number_of_functions": 2
    },
    "dependencies": [
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "anyhow",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "adk_rust",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "futures",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "serde_json",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "tracing",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": null,
        "name": "crate::artifacts",
        "path": "crates/cowork-core/src/artifacts/mod.rs",
        "version": null
      },
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": null,
        "name": "crate::memory::ArtifactStore",
        "path": "crates/cowork-core/src/memory/artifact_store.rs",
        "version": null
      },
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": null,
        "name": "crate::config::LlmConfig",
        "path": "crates/cowork-core/src/config/llm_config.rs",
        "version": null
      },
      {
        "dependency_type": "struct",
        "is_external": false,
        "line_number": null,
        "name": "LlmAgentBuilder",
        "path": "crates/adk-rust/src/agent/builder.rs",
        "version": null
      },
      {
        "dependency_type": "struct",
        "is_external": false,
        "line_number": null,
        "name": "Content",
        "path": "crates/adk-rust/src/model/content.rs",
        "version": null
      },
      {
        "dependency_type": "struct",
        "is_external": false,
        "line_number": null,
        "name": "ArtifactEnvelope",
        "path": "crates/cowork-core/src/artifacts/mod.rs",
        "version": null
      }
    ],
    "detailed_description": "The PlanAgent is an intelligent agent responsible for transforming a Design Document into a structured implementation plan. It leverages an OpenAI LLM to generate a comprehensive plan in JSON format containing C4 architecture modeling (context, containers, components, code), task breakdowns with priorities and dependencies, milestones with completion criteria, and a detailed todo list with verification methods. The agent constructs a context string from the design document's CLI modes, workflow stages, architecture layers, and components, then invokes an LLM agent with strict output schema constraints to ensure structured, machine-readable output. It manages session state via InMemorySessionService, runs the LLM agent through a Runner, extracts the generated plan from session state, validates and parses it, and finally persists the result as a PlanArtifact in the ArtifactStore.",
    "interfaces": [
      {
        "description": null,
        "interface_type": "struct",
        "name": "OpenAIClient",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "config",
            "param_type": "OpenAIConfig"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "trait",
        "name": "ArtifactStore",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "session_id",
            "param_type": "&str"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "stage",
            "param_type": "Stage"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "artifact",
            "param_type": "&ArtifactEnvelope"
          }
        ],
        "return_type": "Result<()>",
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "LlmConfig",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "api_key",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "api_base_url",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "model_name",
            "param_type": "String"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "InMemorySessionService",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "Runner",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "config",
            "param_type": "RunnerConfig"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "Content",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "role",
            "param_type": "String"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "ArtifactEnvelope",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "session_id",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "stage",
            "param_type": "Stage"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "data",
            "param_type": "Plan"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "Plan",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "c4",
            "param_type": "C4Model"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "tasks",
            "param_type": "Vec<Task>"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "milestones",
            "param_type": "Vec<Milestone>"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "todo_list",
            "param_type": "TodoList"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "DesignDocArtifact",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "data",
            "param_type": "DesignData"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "meta",
            "param_type": "ArtifactMeta"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "type_alias",
        "name": "PlanArtifact",
        "parameters": [],
        "return_type": "ArtifactEnvelope<Plan>",
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "enum",
        "name": "Stage",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Generate structured implementation plan from Design Document using LLM",
      "Enforce strict JSON output schema for reliable parsing",
      "Manage LLM session lifecycle and state persistence",
      "Transform raw LLM output into typed PlanArtifact",
      "Persist generated plan to ArtifactStore with metadata and lineage"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": "Intelligent agent component for normalizing code plan output from code planners",
      "file_path": "crates/cowork-core/src/agents/code_plan_normalizer.rs",
      "functions": [
        "guess_phase",
        "normalize",
        "guess_module_type",
        "guess_module_path",
        "module_type_as_str",
        "_to_module"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "guess_phase",
        "normalize",
        "guess_module_type",
        "guess_module_path",
        "module_type_as_str",
        "_to_module"
      ],
      "name": "code_plan_normalizer.rs",
      "source_summary": "use serde_json::Value;\n\nuse crate::artifacts::{Module, ModuleType};\n\nfn guess_phase(cmd: &str) -> &'static str {\n    let c = cmd.to_lowercase();\n\n    if c.contains(\" test\") || c.contains(\"jest\") || c.contains(\"testing\") {\n        return \"test\";\n    }\n    if c.contains(\"lint\") || c.contains(\"eslint\") || c.contains(\"prettier\") {\n        return \"lint\";\n    }\n    if c.contains(\"build\") || c.contains(\"webpack\") || c.contains(\"tsc\") {\n        return \"build\";\n    }\n    if c.contains(\"check\") {\n        return \"check\";\n    }\n\n    \"run\"\n}\n\n\npub struct CodePlanNormalizer;\n\nimpl CodePlanNormalizer {\n    /// 对 code_planner 的 JSON 输出做容错归一化，修正常见 schema 偏差。\n    pub fn normalize(mut v: Value) -> Value {\n        // project.layout: monorepo/workspace -> mono\n        if let Some(layout) = v.pointer_mut(\"/project/layout\") {\n            if let Some(s) = layout.as_str() {\n                let normalized = match s {\n                    \"mono\" | \"single\" | \"unknown\" => s.to_string(),\n                    \"monorepo\" | \"mono_repo\" | \"workspace\" | \"multi\" | \"multi_repo\" => \"mono\".to_string(),\n                    _ => \"unknown\".to_string(),\n                };\n                *layout = Value::String(normalized);\n            }\n        }\n\n        // project.modules: [\"frontend\", \"backend\"] -> [{name,path,type}, ...]\n        if let Some(modules) = v.pointer_mut(\"/project/modules\") {\n            if let Some(arr) = modules.as_array() {\n                let all_strings = arr.iter().all(|x| x.is_string());\n                if all_strings {\n                    let new_modules: Vec<Value> = arr\n                        .iter()\n                        .filter_map(|x| x.as_str())\n                        .map(|name| {\n                            let module_type = guess_module_type(name);\n                            let module_path = guess_module_path(name, module_type);\n                            serde_json::json!({\n                                \"name\": name,\n                                \"path\": module_path,\n                                \"type\": module_type_as_str(module_type),\n                            })\n                        })\n                        .collect();\n                    *modules = Value::Array(new_modules);\n                }\n            }\n        }\n\n        // cmds: [\"npm init -y\", ...] -> [{cmd, expect, phase}, ...]\n        if let Some(cmds) = v.pointer_mut(\"/cmds\") {\n            if let Some(arr) = cmds.as_array() {\n                let all_strings = arr.iter().all(|x| x.is_string());\n                if all_strings {\n                    let new_cmds: Vec<Value> = arr\n                        .iter()\n                        .filter_map(|x| x.as_str())\n                        .map(|cmd| {\n                            let phase = guess_phase(cmd);\n                            serde_json::json!({\n                                \"cmd\": cmd,\n                                \"expect\": \"exit_code_0\",\n                                \"phase\": phase,\n                            })\n                        })\n                        .collect();\n                    *cmds = Value::Array(new_cmds);\n                } else {\n                    // 如果是对象数组但缺字段，做兜底补全\n                    let new_cmds: Vec<Value> = arr\n                        .iter()\n                        .map(|item| {\n                            if let Some(obj) = item.as_object() {\n                                let mut o = obj.clone();\n                                if !o.contains_key(\"expect\") {\n                                    o.insert(\"expect\".to_string(), Value::String(\"exit_code_0\".to_string()));\n                                }\n                                if !o.contains_key(\"phase\") {\n                                    let cmd = o.get(\"cmd\").and_then(|v| v.as_str()).unwrap_or(\"\");\n                                    o.insert(\"phase\".to_string(), Value::String(guess_phase(cmd).to_string()));\n                                }\n                                Value::Object(o)\n                            } else {\n                                item.clone()\n                            }\n                        })\n                        .collect();\n                    *cmds = Value::Array(new_cmds);\n                }\n            }\n        }\n\n        // project.layout 的 enum 是 mono|single|unknown；如果 LLM 输出了 schema 外的值，兜底。\n        if let Some(layout) = v.pointer_mut(\"/project/layout\") {\n            if let Some(s) = layout.as_str() {\n                if s != \"mono\" && s != \"single\" && s != \"unknown\" {\n                    *layout = Value::String(\"unknown\".to_string());\n                }\n            }\n        }\n\n        v\n    }\n}\n\nfn guess_module_type(name: &str) -> ModuleType {\n    let n = name.to_lowercase();\n\n    if n.contains(\"front\") || n.contains(\"web\") || n.contains(\"ui\") || n.contains(\"app\") {\n        return ModuleType::App;\n    }\n    if n.contains(\"backend\") || n.contains(\"api\") || n.contains(\"service\") {\n        return ModuleType::Service;\n    }\n    if n.contains(\"cli\") {\n        return ModuleType::Pkg;\n    }\n    if n.contains(\"shared\") || n.contains(\"common\") || n.contains(\"lib\") {\n        return ModuleType::Lib;\n    }\n\n    ModuleType::Unknown\n}\n\nfn guess_module_path(name: &str, module_type: ModuleType) -> String {\n    if name.contains('/') {\n        return name.to_string();\n    }\n\n    match module_type {\n        ModuleType::App => format!(\"apps/{}\", name),\n        ModuleType::Service => format!(\"services/{}\", name),\n        ModuleType::Lib => format!(\"crates/{}\", name),\n        ModuleType::Pkg => format!(\"packages/{}\", name),\n        ModuleType::Unknown => name.to_string(),\n    }\n}\n\nfn module_type_as_str(t: ModuleType) -> &'static str {\n    match t {\n        ModuleType::Service => \"service\",\n        ModuleType::Lib => \"lib\",\n        ModuleType::App => \"app\",\n        ModuleType::Pkg => \"pkg\",\n        ModuleType::Unknown => \"unknown\",\n    }\n}\n\n#[allow(dead_code)]\nfn _to_module(v: &Value) -> Option<Module> {\n    Some(Module {\n        name: v.get(\"name\")?.as_str()?.to_string(),\n        path: v.get(\"path\")?.as_str()?.to_string(),\n        module_type: serde_json::from_value(v.get(\"type\")?.clone()).ok()?,\n    })\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 27.0,
      "lines_of_code": 171,
      "number_of_classes": 1,
      "number_of_functions": 6
    },
    "dependencies": [
      {
        "dependency_type": "data_processing",
        "is_external": true,
        "line_number": 1,
        "name": "serde_json::Value",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "data_structure",
        "is_external": false,
        "line_number": 3,
        "name": "crate::artifacts::{Module, ModuleType}",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "This component serves as an intelligent agent that performs fault-tolerant normalization on code planner JSON outputs. It corrects common schema deviations and standardizes the structure of code planning data. The main functionality includes normalizing project layouts (monorepo/workspace to mono format), transforming module arrays from string format to structured objects with inferred types and paths, and normalizing command arrays by adding missing fields like phase and expectations. The component uses heuristic-based guessing functions to infer module types, paths, and command phases based on naming patterns and content analysis.",
    "interfaces": [
      {
        "description": "Main normalization function that processes and standardizes code planner JSON output",
        "interface_type": "function",
        "name": "normalize",
        "parameters": [
          {
            "description": "JSON value to normalize",
            "is_optional": false,
            "name": "v",
            "param_type": "Value"
          }
        ],
        "return_type": "Value",
        "visibility": "public"
      },
      {
        "description": "Guesses the execution phase based on command content keywords",
        "interface_type": "function",
        "name": "guess_phase",
        "parameters": [
          {
            "description": "Command string to analyze",
            "is_optional": false,
            "name": "cmd",
            "param_type": "&str"
          }
        ],
        "return_type": "&'static str",
        "visibility": "private"
      },
      {
        "description": "Infers module type based on naming patterns and keywords",
        "interface_type": "function",
        "name": "guess_module_type",
        "parameters": [
          {
            "description": "Module name to analyze",
            "is_optional": false,
            "name": "name",
            "param_type": "&str"
          }
        ],
        "return_type": "ModuleType",
        "visibility": "private"
      },
      {
        "description": "Generates appropriate module path based on type and name",
        "interface_type": "function",
        "name": "guess_module_path",
        "parameters": [
          {
            "description": "Module name",
            "is_optional": false,
            "name": "name",
            "param_type": "&str"
          },
          {
            "description": "Inferred module type",
            "is_optional": false,
            "name": "module_type",
            "param_type": "ModuleType"
          }
        ],
        "return_type": "String",
        "visibility": "private"
      },
      {
        "description": "Converts ModuleType enum to string representation",
        "interface_type": "function",
        "name": "module_type_as_str",
        "parameters": [
          {
            "description": "Module type enum",
            "is_optional": false,
            "name": "t",
            "param_type": "ModuleType"
          }
        ],
        "return_type": "&'static str",
        "visibility": "private"
      },
      {
        "description": "Helper function to convert JSON to Module struct (currently unused)",
        "interface_type": "function",
        "name": "_to_module",
        "parameters": [
          {
            "description": "JSON value containing module data",
            "is_optional": false,
            "name": "v",
            "param_type": "&Value"
          }
        ],
        "return_type": "Option<Module>",
        "visibility": "private"
      }
    ],
    "responsibilities": [
      "Schema normalization and standardization of code planner JSON output",
      "Heuristic-based inference of missing data fields (module types, paths, command phases)",
      "Data validation and fallback handling for schema deviations",
      "Project structure analysis and module classification",
      "Command phase categorization and expectation standardization"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": null,
      "file_path": "crates/cowork-core/src/agents/check_agent.rs",
      "functions": [
        "CheckAgent::new",
        "CheckAgent::execute",
        "CheckAgent::load_plan_artifact",
        "CheckAgent::load_prd_artifact",
        "CheckAgent::verify_requirement_coverage",
        "CheckAgent::check_file_existence",
        "CheckAgent::check_file_content_quality",
        "CheckAgent::check_compilation",
        "CheckAgent::check_rust_compilation",
        "CheckAgent::check_python_syntax",
        "CheckAgent::check_js_syntax"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "ArtifactStore",
        "CodeChangeArtifact",
        "CheckReportArtifact",
        "PRDArtifact",
        "PlanArtifact",
        "PRD",
        "CodeChange",
        "CheckResult",
        "Issue",
        "TodoCompletion",
        "RequirementCoverage",
        "Stage",
        "ArtifactEnvelope"
      ],
      "name": "check_agent.rs",
      "source_summary": "use anyhow::Result;\nuse std::sync::Arc;\n\nuse crate::artifacts::*;\nuse crate::memory::ArtifactStore;\nuse crate::config::LlmConfig;\n\n/// Check Agent - 检查代码质量和完整性\npub struct CheckAgent {\n    store: Arc<ArtifactStore>,\n}\n\nimpl CheckAgent {\n    pub fn new(_llm_config: &LlmConfig, store: Arc<ArtifactStore>) -> Result<Self> {\n        tracing::info!(\"Creating Check Agent\");\n        \n        Ok(Self {\n            store,\n        })\n    }\n\n    pub async fn execute(&self, session_id: &str, code_artifact: &CodeChangeArtifact) -> Result<CheckReportArtifact> {\n        tracing::info!(\"CheckAgent: checking code for session {}\", session_id);\n\n        // 尝试加载 PRD artifact（包含 requirements）\n        let prd_artifact_result = self.load_prd_artifact(session_id);\n        \n        // 验证需求覆盖度\n        let requirement_coverage = if let Ok(prd_artifact) = prd_artifact_result {\n            self.verify_requirement_coverage(&prd_artifact.data, &code_artifact.data).await\n        } else {\n            tracing::warn!(\"PRD artifact not found, skipping requirement coverage verification\");\n            None\n        };\n        \n        // 基础检查\n        let mut checks = Vec::new();\n        let mut issues = Vec::new();\n        \n        // 1. 文件存在性检查\n        self.check_file_existence(&code_artifact.data, &mut checks, &mut issues);\n        \n        // 2. 文件内容质量检查\n        self.check_file_content_quality(&code_artifact.data, &mut checks, &mut issues);\n        \n        // 3. 编译/语法检查（根据语言类型）\n        self.check_compilation(&code_artifact.data, &mut checks, &mut issues).await;\n        \n        // 创建初步的 CheckReport\n        let mut check_report = CheckReport {\n            checks,\n            ac_results: vec![],\n            issues,\n            todo_completion: None,\n            requirement_coverage,\n        };\n        \n        // 验证 TodoList 完成度并更新状态\n        let todo_completion = if let Ok(mut plan_artifact) = self.load_plan_artifact(session_id) {\n            if let Some(ref mut todo_list) = plan_artifact.data.todo_list {\n                // 根据验证结果更新 TodoList 状态\n                crate::agents::TodoListManager::verify_from_check(todo_list, &check_report);\n                \n                // 生成状态报告（在保存前）\n                let report = crate::agents::TodoListManager::generate_status_report(todo_list);\n                \n                // 保存更新后的 TodoList（移动到后面，避免借用冲突）\n                self.store.put(session_id, Stage::Plan, &plan_artifact)?;\n                \n                Some(TodoCompletion {\n                    total: report.total,\n                    completed: report.completed,\n                    pending: report.pending,\n                    blocked: report.blocked,\n                })\n            } else {\n                None\n            }\n        } else {\n            tracing::warn!(\"Plan artifact not found, skipping TodoList verification\");\n            None\n        };\n        \n        // 更新 check_report 的 todo_completion\n        check_report.todo_completion = todo_completion;\n\n        let summary = vec![\n            format!(\"Checks: {}\", check_report.checks.len()),\n            format!(\"Issues: {}\", check_report.issues.len()),\n            if let Some(ref tc) = check_report.todo_completion {\n                format!(\"Todo: {}/{} completed\", tc.completed, tc.total)\n            } else {\n                \"Todo: N/A\".to_string()\n            },\n            if let Some(ref rc) = check_report.requirement_coverage {\n                format!(\"Coverage: {:.1}%\", rc.coverage_percentage)\n            } else {\n                \"Coverage: N/A\".to_string()\n            },\n        ];\n\n        let artifact = ArtifactEnvelope::new(session_id.to_string(), Stage::Check, check_report)\n            .with_summary(summary)\n            .with_prev(vec![code_artifact.meta.artifact_id.clone()]);\n\n        self.store.put(session_id, Stage::Check, &artifact)?;\n\n        tracing::info!(\"Check report saved successfully\");\n\n        Ok(artifact)\n    }\n    \n    /// 加载 Plan artifact\n    fn load_plan_artifact(&self, session_id: &str) -> Result<PlanArtifact> {\n        // 列出所有 artifacts，找到 plan stage 的\n        let artifacts = self.store.list(session_id)?;\n        \n        for meta in artifacts {\n            if meta.stage == Stage::Plan {\n                return self.store.get(session_id, &meta.artifact_id);\n            }\n        }\n        \n        Err(anyhow::anyhow!(\"Plan artifact not found\"))\n    }\n    \n    /// 加载 PRD artifact\n    fn load_prd_artifact(&self, session_id: &str) -> Result<PRDArtifact> {\n        let artifacts = self.store.list(session_id)?;\n        \n        for meta in artifacts {\n            if meta.stage == Stage::Requirements {\n                return self.store.get(session_id, &meta.artifact_id);\n            }\n        }\n        \n        Err(anyhow::anyhow!(\"PRD artifact not found\"))\n    }\n    \n    /// 验证需求覆盖度\n    async fn verify_requirement_coverage(&self, prd: &PRD, code_change: &CodeChange) -> Option<RequirementCoverage> {\n        let mut verified = 0;\n        let mut not_verified = 0;\n        \n        for req in &prd.reqs {\n            // 查找对应的文件映射\n            if let Some(mapping) = code_change.requirement_mapping.iter()\n                .find(|m| m.req_id == req.id) \n            {\n                // 检查映射的文件是否都存在\n                let all_files_exist = mapping.files.iter()\n                    .all(|file| std::path::Path::new(file).exists());\n                \n                if all_files_exist {\n                    verified += 1;\n                } else {\n                    not_verified += 1;\n                }\n            } else {\n                not_verified += 1;\n            }\n        }\n        \n        let total = prd.reqs.len();\n        let coverage_percentage = if total > 0 {\n            (verified as f64 / total as f64) * 100.0\n        } else {\n            0.0\n        };\n        \n        Some(RequirementCoverage {\n            total_requirements: total,\n            verified,\n            partially_verified: 0,\n            not_verified,\n            failed: 0,\n            coverage_percentage,\n        })\n    }\n    \n    /// 检查文件存在性\n    fn check_file_existence(&self, code_change: &CodeChange, checks: &mut Vec<CheckResult>, issues: &mut Vec<Issue>) {\n        for change in &code_change.changes {\n            let file_exists = std::path::Path::new(&change.path).exists();\n            \n            if file_exists {\n                checks.push(CheckResult {\n                    id: format!(\"FILE-EXIST-{}\", change.path),\n                    cmd: format!(\"check file exists: {}\", change.path),\n                    status: \"passed\".to_string(),\n                    out_ref: \"\".to_string(),\n                    notes: vec![format!(\"File {} exists\", change.path)],\n                    phase: Phase::Check,\n                });\n            } else {\n                checks.push(CheckResult {\n                    id: format!(\"FILE-EXIST-{}\", change.path),\n                    cmd: format!(\"check file exists: {}\", change.path),\n                    status: \"failed\".to_string(),\n                    out_ref: \"\".to_string(),\n                    notes: vec![format!(\"File {} does not exist\", change.path)],\n                    phase: Phase::Check,\n                });\n                \n                issues.push(Issue {\n                    id: format!(\"ISSUE-FILE-{}\", change.path),\n                    sev: \"error\".to_string(),\n                    desc: format!(\"File not found: {}\", change.path),\n                    fix_hint: format!(\"Create file: {}\", change.path),\n                });\n            }\n        }\n    }\n    \n    /// 检查文件内容质量（检测空文件、TODO、placeholder等）\n    fn check_file_content_quality(&self, code_change: &CodeChange, checks: &mut Vec<CheckResult>, issues: &mut Vec<Issue>) {\n        use std::fs;\n        \n        for change in &code_change.changes {\n            let path = std::path::Path::new(&change.path);\n            \n            if !path.exists() {\n                continue;  // 已在上一步检查\n            }\n            \n            // 读取文件内容\n            let content = match fs::read_to_string(path) {\n                Ok(c) => c,\n                Err(e) => {\n                    issues.push(Issue {\n                        id: format!(\"ISSUE-READ-{}\", change.path),\n                        sev: \"warning\".to_string(),\n                        desc: format!(\"Cannot read file {}: {}\", change.path, e),\n                        fix_hint: \"Check file permissions\".to_string(),\n                    });\n                    continue;\n                }\n            };\n            \n            let lines: Vec<&str> = content.lines().collect();\n            let non_empty_lines: Vec<&str> = lines.iter()\n                .filter(|line| !line.trim().is_empty())\n                .copied()\n                .collect();\n            \n            // 检查 1: 空文件\n            if non_empty_lines.is_empty() {\n                checks.push(CheckResult {\n                    id: format!(\"CONTENT-QUALITY-{}\", change.path),\n                    cmd: format!(\"check file content: {}\", change.path),\n                    status: \"failed\".to_string(),\n                    out_ref: \"\".to_string(),\n                    notes: vec![\"File is empty\".to_string()],\n                    phase: Phase::Check,\n                });\n                \n                issues.push(Issue {\n                    id: format!(\"ISSUE-EMPTY-{}\", change.path),\n                    sev: \"error\".to_string(),\n                    desc: format!(\"File {} is empty\", change.path),\n                    fix_hint: \"Generate actual code content\".to_string(),\n                });\n                continue;\n            }\n            \n            // 检查 2: TODO/FIXME/placeholder\n            let todo_count = content.matches(\"TODO\").count() + \n                            content.matches(\"FIXME\").count() +\n                            content.matches(\"placeholder\").count();\n            \n            if todo_count > 0 {\n                checks.push(CheckResult {\n                    id: format!(\"CONTENT-QUALITY-{}\", change.path),\n                    cmd: format!(\"check for TODOs: {}\", change.path),\n                    status: \"warning\".to_string(),\n                    out_ref: \"\".to_string(),\n                    notes: vec![format!(\"Found {} TODO/FIXME/placeholder markers\", todo_count)],\n                    phase: Phase::Check,\n                });\n                \n                issues.push(Issue {\n                    id: format!(\"ISSUE-TODO-{}\", change.path),\n                    sev: \"warning\".to_string(),\n                    desc: format!(\"File {} contains {} incomplete markers (TODO/FIXME/placeholder)\", change.path, todo_count),\n                    fix_hint: \"Complete the implementation\".to_string(),\n                });\n            } else {\n                // 内容质量通过\n                checks.push(CheckResult {\n                    id: format!(\"CONTENT-QUALITY-{}\", change.path),\n                    cmd: format!(\"check file content: {}\", change.path),\n                    status: \"passed\".to_string(),\n                    out_ref: \"\".to_string(),\n                    notes: vec![format!(\"File has {} lines of content\", non_empty_lines.len())],\n                    phase: Phase::Check,\n                });\n            }\n        }\n    }\n    \n    /// 编译/语法检查\n    async fn check_compilation(&self, code_change: &CodeChange, checks: &mut Vec<CheckResult>, issues: &mut Vec<Issue>) {\n        let lang = &code_change.target.lang;\n        \n        match lang.as_str() {\n            \"rust\" => self.check_rust_compilation(checks, issues).await,\n            \"python\" => self.check_python_syntax(code_change, checks, issues).await,\n            \"javascript\" | \"typescript\" => self.check_js_syntax(code_change, checks, issues).await,\n            \"html\" | \"web\" => {\n                // HTML 不需要编译，但可以检查基本结构\n                tracing::info!(\"HTML project - skipping compilation check\");\n            }\n            _ => {\n                tracing::warn!(\"Unknown language {}, skipping compilation check\", lang);\n            }\n        }\n    }\n    \n    /// Rust 编译检查\n    async fn check_rust_compilation(&self, checks: &mut Vec<CheckResult>, issues: &mut Vec<Issue>) {\n        use std::process::Command;\n        \n        tracing::info!(\"Running cargo check...\");\n        \n        let output = Command::new(\"cargo\")\n            .arg(\"check\")\n            .arg(\"--message-format=short\")\n            .output();\n        \n        match output {\n            Ok(result) => {\n                let _stdout = String::from_utf8_lossy(&result.stdout);\n                let stderr = String::from_utf8_lossy(&result.stderr);\n                \n                if result.status.success() {\n                    checks.push(CheckResult {\n                        id: \"COMPILE-RUST\".to_string(),\n                        cmd: \"cargo check\".to_string(),\n                        status: \"passed\".to_string(),\n                        out_ref: \"\".to_string(),\n                        notes: vec![\"Compilation successful\".to_string()],\n                        phase: Phase::Check,\n                    });\n                } else {\n                    checks.push(CheckResult {\n                        id: \"COMPILE-RUST\".to_string(),\n                        cmd: \"cargo check\".to_string(),\n                        status: \"failed\".to_string(),\n                        out_ref: \"\".to_string(),\n                        notes: vec![format!(\"Compilation failed:\\n{}\", stderr)],\n                        phase: Phase::Check,\n                    });\n                    \n                    issues.push(Issue {\n                        id: \"ISSUE-COMPILE-RUST\".to_string(),\n                        sev: \"error\".to_string(),\n                        desc: \"Rust compilation failed\".to_string(),\n                        fix_hint: format!(\"Fix compilation errors:\\n{}\", stderr.lines().take(10).collect::<Vec<_>>().join(\"\\n\")),\n                    });\n                }\n            }\n            Err(e) => {\n                tracing::warn!(\"Failed to run cargo check: {}\", e);\n                checks.push(CheckResult {\n                    id: \"COMPILE-RUST\".to_string(),\n                    cmd: \"cargo check\".to_string(),\n                    status: \"skipped\".to_string(),\n                    out_ref: \"\".to_string(),\n                    notes: vec![format!(\"Cannot run cargo: {}\", e)],\n                    phase: Phase::Check,\n                });\n            }\n        }\n    }\n    \n    /// Python 语法检查\n    async fn check_python_syntax(&self, code_change: &CodeChange, checks: &mut Vec<CheckResult>, issues: &mut Vec<Issue>) {\n        use std::process::Command;\n        \n        for change in &code_change.changes {\n            if !change.path.ends_with(\".py\") {\n                continue;\n            }\n            \n            let output = Command::new(\"python3\")\n                .arg(\"-m\")\n                .arg(\"py_compile\")\n                .arg(&change.path)\n                .output();\n            \n            match output {\n                Ok(result) => {\n                    if result.status.success() {\n                        checks.push(CheckResult {\n                            id: format!(\"SYNTAX-PY-{}\", change.path),\n                            cmd: format!(\"python3 -m py_compile {}\", change.path),\n                            status: \"passed\".to_string(),\n                            out_ref: \"\".to_string(),\n                            notes: vec![\"Syntax check passed\".to_string()],\n                            phase: Phase::Check,\n                        });\n                    } else {\n                        let stderr = String::from_utf8_lossy(&result.stderr);\n                        checks.push(CheckResult {\n                            id: format!(\"SYNTAX-PY-{}\", change.path),\n                            cmd: format!(\"python3 -m py_compile {}\", change.path),\n                            status: \"failed\".to_string(),\n                            out_ref: \"\".to_string(),\n                            notes: vec![format!(\"Syntax error:\\n{}\", stderr)],\n                            phase: Phase::Check,\n                        });\n                        \n                        issues.push(Issue {\n                            id: format!(\"ISSUE-SYNTAX-PY-{}\", change.path),\n                            sev: \"error\".to_string(),\n                            desc: format!(\"Python syntax error in {}\", change.path),\n                            fix_hint: stderr.to_string(),\n                        });\n                    }\n                }\n                Err(e) => {\n                    tracing::warn!(\"Failed to check Python syntax for {}: {}\", change.path, e);\n                }\n            }\n        }\n    }\n    \n    /// JavaScript/TypeScript 语法检查\n    async fn check_js_syntax(&self, _code_change: &CodeChange, _checks: &mut Vec<CheckResult>, _issues: &mut Vec<Issue>) {\n        // 简化版：检查是否有 package.json，如果有则运行 npm run build/check\n        let has_package_json = std::path::Path::new(\"package.json\").exists();\n        \n        if !has_package_json {\n            tracing::info!(\"No package.json found, skipping JS build check\");\n            return;\n        }\n        \n        // 这里可以扩展为实际的 npm build 检查\n        tracing::info!(\"JavaScript project detected, consider adding npm build check\");\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 32.0,
      "lines_of_code": 441,
      "number_of_classes": 1,
      "number_of_functions": 11
    },
    "dependencies": [
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "anyhow",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "rust_std",
        "is_external": true,
        "line_number": null,
        "name": "std",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::artifacts",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::memory",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::config",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "tracing",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::agents::TodoListManager",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "rust_std",
        "is_external": true,
        "line_number": null,
        "name": "std::process::Command",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "The CheckAgent is a core intelligence agent in the Cowork system responsible for validating code changes against requirements, quality standards, and compilation integrity. It processes CodeChangeArtifacts by performing a multi-layered verification: first loading related PRD (Product Requirements Document) and plan artifacts, then validating requirement coverage by checking if modified files are mapped to requirements, followed by file existence and content quality checks (empty files, TODO markers), and finally executing language-specific compilation/syntax checks (Rust, Python, JavaScript). The agent generates a comprehensive CheckReportArtifact containing pass/fail checks, issues, todo completion status, and coverage metrics, which is persisted to the artifact store. It also updates the TodoList via TodoListManager to reflect verification outcomes. The agent is designed to be extensible for new languages and integrates tightly with the system's artifact lifecycle management.",
    "interfaces": [
      {
        "description": null,
        "interface_type": "struct",
        "name": "ArtifactStore",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "store",
            "param_type": "Arc<ArtifactStore>"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "CodeChangeArtifact",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "data",
            "param_type": "CodeChange"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "CheckReportArtifact",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "data",
            "param_type": "CheckReport"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "PRDArtifact",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "data",
            "param_type": "PRD"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "PlanArtifact",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "data",
            "param_type": "Plan"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "PRD",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "reqs",
            "param_type": "Vec<Requirement>"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "CodeChange",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "changes",
            "param_type": "Vec<CodeChangeItem>"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "requirement_mapping",
            "param_type": "Vec<RequirementMapping>"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "target",
            "param_type": "Target"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "CheckResult",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "id",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "cmd",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "status",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "out_ref",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "notes",
            "param_type": "Vec<String>"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "phase",
            "param_type": "Phase"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "Issue",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "id",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "sev",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "desc",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "fix_hint",
            "param_type": "String"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "TodoCompletion",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "total",
            "param_type": "usize"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "completed",
            "param_type": "usize"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "pending",
            "param_type": "usize"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "blocked",
            "param_type": "usize"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "RequirementCoverage",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "total_requirements",
            "param_type": "usize"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "verified",
            "param_type": "usize"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "partially_verified",
            "param_type": "usize"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "not_verified",
            "param_type": "usize"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "failed",
            "param_type": "usize"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "coverage_percentage",
            "param_type": "f64"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "enum",
        "name": "Stage",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "ArtifactEnvelope",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "session_id",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "stage",
            "param_type": "Stage"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "data",
            "param_type": "CheckReport"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "LlmConfig",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "model",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "temperature",
            "param_type": "f64"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "TodoListManager",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Validate code changes against documented requirements",
      "Perform file existence and content quality checks",
      "Execute language-specific compilation/syntax validation",
      "Update and synchronize TodoList status based on verification results",
      "Generate and persist structured CheckReportArtifact for downstream agents"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "util",
      "description": null,
      "file_path": "crates/cowork-core/src/utils/prd_utils.rs",
      "functions": [
        "extract_prd_summary"
      ],
      "importance_score": 0.8,
      "interfaces": [],
      "name": "prd_utils.rs",
      "source_summary": "use crate::artifacts::*;\n\n/// 从 PRD Artifact 中提取摘要（用于 WatchDog）\npub fn extract_prd_summary(prd_artifact: &PRDArtifact) -> String {\n    let prd = &prd_artifact.data;\n    \n    let mut summary_parts = vec![];\n    \n    // 项目范围\n    if !prd.scope.g.is_empty() {\n        summary_parts.push(\"**Goals**:\".to_string());\n        for goal in prd.scope.g.iter().take(3) {\n            summary_parts.push(format!(\"- {}\", goal));\n        }\n    }\n    \n    // 需求摘要（取前 5 个）\n    if !prd.reqs.is_empty() {\n        summary_parts.push(\"\\n**Requirements**:\".to_string());\n        for req in prd.reqs.iter().take(5) {\n            summary_parts.push(format!(\"- {}: {}\", req.id, req.desc));\n        }\n        \n        if prd.reqs.len() > 5 {\n            summary_parts.push(format!(\"... and {} more requirements\", prd.reqs.len() - 5));\n        }\n    }\n    \n    summary_parts.join(\"\\n\")\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn test_extract_prd_summary() {\n        let prd_artifact = ArtifactEnvelope {\n            meta: ArtifactMeta {\n                session_id: \"session_001\".to_string(),\n                artifact_id: \"prd_001\".to_string(),\n                stage: Stage::Requirements,\n                v: 1,\n                ts: chrono::Utc::now(),\n            },\n            summary: vec![],\n            links: ArtifactLinks { prev: vec![] },\n            data: PRD {\n                scope: Scope {\n                    g: vec![\"Create a todo app\".to_string()],\n                    ng: vec![],\n                },\n                reqs: vec![\n                    Requirement {\n                        id: \"REQ-001\".to_string(),\n                        desc: \"User can create todos\".to_string(),\n                        pri: Priority::P0,\n                        req_type: RequirementType::Func,\n                        deps: vec![],\n                        ac: vec![],\n                    },\n                    Requirement {\n                        id: \"REQ-002\".to_string(),\n                        desc: \"User can delete todos\".to_string(),\n                        pri: Priority::P0,\n                        req_type: RequirementType::Func,\n                        deps: vec![],\n                        ac: vec![],\n                    },\n                ],\n                cons: vec![],\n                hitl: vec![],\n            },\n        };\n\n        let summary = extract_prd_summary(&prd_artifact);\n        \n        assert!(summary.contains(\"Goals\"));\n        assert!(summary.contains(\"REQ-001\"));\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 6.0,
      "lines_of_code": 81,
      "number_of_classes": 0,
      "number_of_functions": 1
    },
    "dependencies": [
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::artifacts::*",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "This utility function extracts a human-readable summary from a PRD (Product Requirements Document) artifact for use by a WatchDog system. It processes the PRD data structure to generate a formatted text summary including up to three project goals and up to five requirements, with a note if more requirements exist. The function is designed to provide concise, structured overviews for monitoring or notification purposes.",
    "interfaces": [],
    "responsibilities": [
      "Extract and format project goals from PRD artifact",
      "Extract and format requirement summaries with ID and description",
      "Handle truncation of requirements list with overflow notification",
      "Generate clean, readable text output for monitoring systems",
      "Support testability through well-defined input/output contracts"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "module",
      "description": null,
      "file_path": "crates/cowork-core/src/orchestrator/mod.rs",
      "functions": [
        "StageStatus",
        "SessionMeta",
        "Orchestrator::new",
        "Orchestrator::create_session",
        "Orchestrator::load_session_meta",
        "Orchestrator::save_session_meta",
        "Orchestrator::run_full_workflow",
        "Orchestrator::mark_stage_in_progress",
        "Orchestrator::mark_stage_completed",
        "Orchestrator::mark_stage_failed",
        "Orchestrator::is_stage_completed_and_verified",
        "Orchestrator::run_workflow_from_stage",
        "Orchestrator::load_artifact",
        "Orchestrator::resume_session",
        "Orchestrator::run_full_workflow_legacy",
        "Orchestrator::list_artifacts",
        "Orchestrator::print_idea_summary",
        "Orchestrator::print_prd_summary",
        "Orchestrator::print_design_summary",
        "Orchestrator::print_plan_summary",
        "Orchestrator::print_code_summary",
        "Orchestrator::print_check_summary",
        "Orchestrator::print_feedback_summary",
        "Orchestrator::print_delivery_summary"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "Stage",
        "StageStatus",
        "SessionMeta",
        "ArtifactStore",
        "HitlController",
        "ModelConfig",
        "IdeaIntakeAgent",
        "PrdAgent",
        "DesignAgent",
        "PlanAgent",
        "CodePlanner",
        "CheckAgent",
        "FeedbackAgent",
        "DeliveryAgent",
        "ErrorAnalyzer",
        "CodeExecutor",
        "ArtifactEnvelope",
        "IdeaSpec",
        "PRD",
        "DesignDoc",
        "Plan",
        "CodeChangeArtifact",
        "CheckReportArtifact",
        "FeedbackArtifact",
        "DeliveryReportArtifact"
      ],
      "name": "mod.rs",
      "source_summary": "use anyhow::Result;\nuse serde::{Deserialize, Serialize};\nuse std::sync::Arc;\nuse std::collections::HashMap;\n\nuse crate::artifacts::Stage;\nuse crate::memory::ArtifactStore;\nuse crate::agents::{\n    IdeaIntakeAgent, PrdAgent, DesignAgent, PlanAgent, \n    CodePlanner, CheckAgent, FeedbackAgent, DeliveryAgent\n};\nuse crate::hitl::HitlController;\nuse crate::config::ModelConfig;\n\n#[cfg(test)]\nmod tests;\n\n/// Stage 执行状态\n#[derive(Debug, Clone, Serialize, Deserialize)]\n#[serde(tag = \"status\", rename_all = \"snake_case\")]\npub enum StageStatus {\n    /// 未开始\n    NotStarted,\n    \n    /// 执行中\n    InProgress {\n        started_at: chrono::DateTime<chrono::Utc>,\n    },\n    \n    /// 完成（可能有或没有验证）\n    Completed {\n        artifact_id: String,\n        completed_at: chrono::DateTime<chrono::Utc>,\n        verified: bool,  // 是否经过验证\n    },\n    \n    /// 失败\n    Failed {\n        error: String,\n        failed_at: chrono::DateTime<chrono::Utc>,\n        can_retry: bool,\n    },\n}\n\n/// Session 元信息\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct SessionMeta {\n    pub session_id: String,\n    pub created_at: chrono::DateTime<chrono::Utc>,\n    pub current_stage: Option<Stage>,\n    \n    #[serde(default)]\n    pub stage_status: HashMap<Stage, StageStatus>,  // 详细状态\n    \n    // 保留旧字段用于向后兼容\n    #[serde(default, skip_serializing_if = \"Vec::is_empty\")]\n    pub completed_stages: Vec<Stage>,\n}\n\n/// Orchestrator 负责驱动多阶段流程\npub struct Orchestrator {\n    store: Arc<ArtifactStore>,\n}\n\nimpl Orchestrator {\n    pub fn new(store: ArtifactStore) -> Self {\n        Self {\n            store: Arc::new(store),\n        }\n    }\n\n    /// 创建新 session\n    pub fn create_session(&self) -> Result<String> {\n        let session_id = uuid::Uuid::new_v4().to_string();\n        let meta = SessionMeta {\n            session_id: session_id.clone(),\n            created_at: chrono::Utc::now(),\n            current_stage: None,\n            stage_status: HashMap::new(),\n            completed_stages: Vec::new(),\n        };\n\n        self.save_session_meta(&meta)?;\n\n        tracing::info!(\"Session created: {}\", session_id);\n        Ok(session_id)\n    }\n\n    /// 加载 session meta\n    pub fn load_session_meta(&self, session_id: &str) -> Result<SessionMeta> {\n        use std::fs;\n        use std::path::PathBuf;\n\n        let meta_path = PathBuf::from(\".cowork\")\n            .join(session_id)\n            .join(\"meta.json\");\n\n        let content = fs::read_to_string(&meta_path)?;\n        Ok(serde_json::from_str(&content)?)\n    }\n\n    /// 保存 session meta\n    pub fn save_session_meta(&self, meta: &SessionMeta) -> Result<()> {\n        use std::fs;\n        use std::path::PathBuf;\n\n        let session_dir = PathBuf::from(\".cowork\").join(&meta.session_id);\n        fs::create_dir_all(&session_dir)?;\n\n        let meta_path = session_dir.join(\"meta.json\");\n        let content = serde_json::to_string_pretty(meta)?;\n        fs::write(&meta_path, content)?;\n\n        Ok(())\n    }\n\n    /// 运行完整的 8 阶段工作流\n    pub async fn run_full_workflow(&self, session_id: &str, model_config: &ModelConfig) -> Result<()> {\n        self.run_workflow_from_stage(session_id, model_config, None).await\n    }\n    \n    /// 标记阶段为进行中\n    fn mark_stage_in_progress(&self, meta: &mut SessionMeta, stage: Stage) -> Result<()> {\n        meta.stage_status.insert(\n            stage,\n            StageStatus::InProgress {\n                started_at: chrono::Utc::now(),\n            }\n        );\n        meta.current_stage = Some(stage);\n        self.save_session_meta(meta)?;\n        Ok(())\n    }\n    \n    /// 标记阶段为完成\n    fn mark_stage_completed(\n        &self,\n        meta: &mut SessionMeta,\n        stage: Stage,\n        artifact_id: String,\n        verified: bool\n    ) -> Result<()> {\n        meta.stage_status.insert(\n            stage,\n            StageStatus::Completed {\n                artifact_id,\n                completed_at: chrono::Utc::now(),\n                verified,\n            }\n        );\n        // 向后兼容\n        if !meta.completed_stages.contains(&stage) {\n            meta.completed_stages.push(stage);\n        }\n        self.save_session_meta(meta)?;\n        Ok(())\n    }\n    \n    /// 标记阶段为失败\n    fn mark_stage_failed(\n        &self,\n        meta: &mut SessionMeta,\n        stage: Stage,\n        error: String,\n        can_retry: bool\n    ) -> Result<()> {\n        meta.stage_status.insert(\n            stage,\n            StageStatus::Failed {\n                error,\n                failed_at: chrono::Utc::now(),\n                can_retry,\n            }\n        );\n        self.save_session_meta(meta)?;\n        Ok(())\n    }\n    \n    /// 检查阶段是否已成功完成并验证\n    fn is_stage_completed_and_verified(&self, meta: &SessionMeta, stage: Stage) -> bool {\n        matches!(\n            meta.stage_status.get(&stage),\n            Some(StageStatus::Completed { verified: true, .. })\n        )\n    }\n\n    /// 从指定阶段开始运行工作流（用于恢复）\n    /// \n    /// # 参数\n    /// - `session_id`: 会话 ID\n    /// - `model_config`: 模型配置\n    /// - `resume_from`: 从哪个阶段开始（None = 从头开始）\n    pub async fn run_workflow_from_stage(\n        &self,\n        session_id: &str,\n        model_config: &ModelConfig,\n        resume_from: Option<Stage>,\n    ) -> Result<()> {\n        tracing::info!(\"Running workflow for session: {}, resume_from: {:?}\", session_id, resume_from);\n\n        let hitl = HitlController::new();\n        let mut meta = self.load_session_meta(session_id)?;\n\n        // 确定起始阶段\n        let start_stage = resume_from.unwrap_or(Stage::IdeaIntake);\n        \n        // 如果是恢复模式，显示已完成的阶段并验证状态\n        if resume_from.is_some() {\n            println!(\"\\n╔═══════════════════════════════════════╗\");\n            println!(\"║   🔄 恢复会话: {}  \", &session_id[..8]);\n            println!(\"╚═══════════════════════════════════════╝\");\n            \n            // 验证前置阶段\n            for stage in Stage::all() {\n                if *stage == start_stage { break; }\n                \n                match meta.stage_status.get(stage) {\n                    Some(StageStatus::Completed { verified: true, artifact_id, .. }) => {\n                        println!(\"✅ {} - 已完成并验证 (artifact: {})\", stage.as_str(), &artifact_id[..8]);\n                    }\n                    Some(StageStatus::Completed { verified: false, artifact_id, .. }) => {\n                        println!(\"⚠️  {} - 已完成但未验证 (artifact: {})\", stage.as_str(), &artifact_id[..8]);\n                        println!(\"   建议：重新验证或从此阶段重新运行\");\n                    }\n                    Some(StageStatus::Failed { error, can_retry, .. }) => {\n                        println!(\"❌ {} - 失败: {}\", stage.as_str(), error);\n                        if *can_retry {\n                            println!(\"   提示：可以重试此阶段\");\n                        }\n                        return Err(anyhow::anyhow!(\"前置阶段 {} 失败，无法继续\", stage.as_str()));\n                    }\n                    Some(StageStatus::InProgress { .. }) => {\n                        println!(\"🔄 {} - 未完成（进行中）\", stage.as_str());\n                        return Err(anyhow::anyhow!(\"前置阶段 {} 未完成\", stage.as_str()));\n                    }\n                    _ => {\n                        // 兼容旧格式：检查 completed_stages\n                        if meta.completed_stages.contains(stage) {\n                            println!(\"✅ {} - 已完成（旧格式，状态未知）\", stage.as_str());\n                        } else {\n                            println!(\"❓ {} - 未开始\", stage.as_str());\n                            return Err(anyhow::anyhow!(\"前置阶段 {} 未完成\", stage.as_str()));\n                        }\n                    }\n                }\n            }\n            \n            println!(\"从阶段继续: {:?}\", start_stage);\n            println!();\n        }\n\n        // Stage 1: IDEA Intake\n        let idea_artifact = if self.is_stage_completed_and_verified(&meta, Stage::IdeaIntake) {\n            println!(\"✓ 跳过 Stage 1: IDEA Intake (已完成)\");\n            self.load_artifact::<crate::artifacts::IdeaSpecArtifact>(session_id, Stage::IdeaIntake)?\n        } else {\n            println!(\"\\n╔═══════════════════════════════════════╗\");\n            println!(\"║   Stage 1: IDEA Intake               ║\");\n            println!(\"╚═══════════════════════════════════════╝\\n\");\n            \n            self.mark_stage_in_progress(&mut meta, Stage::IdeaIntake)?;\n            \n            let user_idea = hitl.input(\"请描述你的 IDEA：\")?;\n            \n            let idea_agent = IdeaIntakeAgent::new(&model_config.llm, self.store.clone())?;\n            let mut idea_artifact = idea_agent.execute(session_id, &user_idea).await?;\n            \n            // HITL 审查和修改\n            if let Some(modified_json) = hitl.review_and_edit_json(\"IdeaSpec\", &idea_artifact.data)? {\n                let modified_data: crate::artifacts::IdeaSpec = serde_json::from_str(&modified_json)?;\n                idea_artifact.data = modified_data;\n                self.store.put(session_id, Stage::IdeaIntake, &idea_artifact)?;\n                println!(\"✅ IdeaSpec 已更新\");\n            }\n            \n            self.mark_stage_completed(&mut meta, Stage::IdeaIntake, idea_artifact.meta.artifact_id.clone(), true)?;\n\n            self.print_idea_summary(&idea_artifact);\n\n            if !hitl.confirm(\"继续生成 PRD？\")? {\n                return Ok(());\n            }\n            \n            idea_artifact\n        };\n\n        // Stage 2: PRD Generation\n        let prd_artifact = if meta.completed_stages.contains(&Stage::Requirements) {\n            println!(\"✓ 跳过 Stage 2: Requirements (已完成)\");\n            self.load_artifact::<crate::artifacts::PRDArtifact>(session_id, Stage::Requirements)?\n        } else {\n            println!(\"\\n╔═══════════════════════════════════════╗\");\n            println!(\"║   Stage 2: Requirements (PRD)        ║\");\n            println!(\"╚═══════════════════════════════════════╝\\n\");\n            \n            let prd_agent = PrdAgent::new(&model_config.llm, self.store.clone())?;\n            let mut prd_artifact = prd_agent.execute(session_id, &idea_artifact).await?;\n            \n            // HITL 审查和修改\n            if let Some(modified_json) = hitl.review_and_edit_json(\"PRD\", &prd_artifact.data)? {\n                let modified_data: crate::artifacts::PRD = serde_json::from_str(&modified_json)?;\n                prd_artifact.data = modified_data;\n                self.store.put(session_id, Stage::Requirements, &prd_artifact)?;\n                println!(\"✅ PRD 已更新\");\n            }\n            \n            meta.current_stage = Some(Stage::Requirements);\n            meta.completed_stages.push(Stage::Requirements);\n            self.save_session_meta(&meta)?;\n\n            self.print_prd_summary(&prd_artifact);\n\n            if !hitl.confirm(\"继续生成设计文档？\")? {\n                return Ok(());\n            }\n            \n            prd_artifact\n        };\n\n        // Stage 3: Design\n        let design_artifact = if meta.completed_stages.contains(&Stage::Design) {\n            println!(\"✓ 跳过 Stage 3: Design (已完成)\");\n            self.load_artifact::<crate::artifacts::DesignDocArtifact>(session_id, Stage::Design)?\n        } else {\n            println!(\"\\n╔═══════════════════════════════════════╗\");\n            println!(\"║   Stage 3: Design Document            ║\");\n            println!(\"╚═══════════════════════════════════════╝\\n\");\n            \n            let design_agent = DesignAgent::new(&model_config.llm, self.store.clone())?;\n            let mut design_artifact = design_agent.execute(session_id, &prd_artifact).await?;\n            \n            // HITL 审查和修改\n            if let Some(modified_json) = hitl.review_and_edit_json(\"DesignDoc\", &design_artifact.data)? {\n                let modified_data: crate::artifacts::DesignDoc = serde_json::from_str(&modified_json)?;\n                design_artifact.data = modified_data;\n                self.store.put(session_id, Stage::Design, &design_artifact)?;\n                println!(\"✅ DesignDoc 已更新\");\n            }\n            \n            meta.current_stage = Some(Stage::Design);\n            meta.completed_stages.push(Stage::Design);\n            self.save_session_meta(&meta)?;\n\n            self.print_design_summary(&design_artifact);\n\n            if !hitl.confirm(\"继续生成实施计划？\")? {\n                return Ok(());\n            }\n            \n            design_artifact\n        };\n\n        // Stage 4: Plan\n        let mut plan_artifact = if meta.completed_stages.contains(&Stage::Plan) {\n            println!(\"✓ 跳过 Stage 4: Plan (已完成)\");\n            self.load_artifact::<crate::artifacts::PlanArtifact>(session_id, Stage::Plan)?\n        } else {\n            println!(\"\\n╔═══════════════════════════════════════╗\");\n            println!(\"║   Stage 4: Implementation Plan        ║\");\n            println!(\"╚═══════════════════════════════════════╝\\n\");\n            \n            let plan_agent = PlanAgent::new(&model_config.llm, self.store.clone())?;\n            let mut plan_artifact = plan_agent.execute(session_id, &design_artifact).await?;\n            \n            // HITL 审查和修改\n            if let Some(modified_json) = hitl.review_and_edit_json(\"Plan\", &plan_artifact.data)? {\n                let modified_data: crate::artifacts::Plan = serde_json::from_str(&modified_json)?;\n                plan_artifact.data = modified_data;\n                self.store.put(session_id, Stage::Plan, &plan_artifact)?;\n                println!(\"✅ Plan 已更新\");\n            }\n            \n            meta.current_stage = Some(Stage::Plan);\n            meta.completed_stages.push(Stage::Plan);\n            self.save_session_meta(&meta)?;\n\n            self.print_plan_summary(&plan_artifact);\n\n            if !hitl.confirm(\"继续生成代码？\")? {\n                return Ok(());\n            }\n            \n            plan_artifact\n        };\n\n        // Stage 5: Coding\n        let code_artifact = if self.is_stage_completed_and_verified(&meta, Stage::Coding) {\n            println!(\"✓ 跳过 Stage 5: Coding (已完成并验证)\");\n            self.load_artifact::<crate::artifacts::CodeChangeArtifact>(session_id, Stage::Coding)?\n        } else {\n            println!(\"\\n╔═══════════════════════════════════════╗\");\n            println!(\"║   Stage 5: Code Planning              ║\");\n            println!(\"╚═══════════════════════════════════════╝\\n\");\n            \n            // 标记为进行中\n            self.mark_stage_in_progress(&mut meta, Stage::Coding)?;\n            \n            let code_planner = CodePlanner::new(&model_config.llm, self.store.clone())?;\n            let code_artifact = code_planner.execute(\n                session_id,\n                &prd_artifact,\n                &design_artifact,\n                &plan_artifact\n            ).await?;\n\n            self.print_code_summary(&code_artifact);\n\n            // 询问是否执行代码变更\n            let mut execution_verified = false;\n            if hitl.confirm(\"是否执行代码变更（AI 自动生成并写入文件）？\")? {\n                println!(\"\\n╔═══════════════════════════════════════╗\");\n                println!(\"║   Stage 5.5: AI Code Generation       ║\");\n                println!(\"╚═══════════════════════════════════════╝\\n\");\n                \n                // 使用支持 AI 代码生成的 executor\n                let executor = crate::agents::CodeExecutor::new(&model_config.llm)?;\n                \n                // 提取 PRD 摘要（用于 WatchDog）\n                let prd_summary = crate::utils::extract_prd_summary(&prd_artifact);\n                \n                // 获取 TodoList（如果存在）\n                let mut todo_list = plan_artifact.data.todo_list.clone();\n                \n                match executor.execute_with_todo(\n                    &code_artifact,\n                    &hitl,\n                    Some(&prd_summary),\n                    todo_list.as_mut(),\n                ).await {\n                    Ok(report) => {\n                        println!(\"\\n代码生成完成:\");\n                        println!(\"  ✅ 成功: {}\", report.successful);\n                        println!(\"  ❌ 失败: {}\", report.failed);\n                        println!(\"  ⏭️  跳过: {}\", report.skipped);\n                        \n                        // 如果全部成功，标记为已验证\n                        execution_verified = report.failed == 0 && report.successful > 0;\n                        \n                        if !execution_verified {\n                            println!(\"⚠️  部分文件生成失败，Coding 阶段将标记为未验证\");\n                        }\n                        \n                        // 保存更新后的 TodoList\n                        if let Some(updated_todo_list) = todo_list {\n                            plan_artifact.data.todo_list = Some(updated_todo_list);\n                            // 更新 plan artifact\n                            self.store.put(session_id, Stage::Plan, &plan_artifact)?;\n                        }\n                    }\n                    Err(e) => {\n                        tracing::error!(\"Code execution failed: {}\", e);\n                        self.mark_stage_failed(&mut meta, Stage::Coding, e.to_string(), true)?;\n                        return Err(e);\n                    }\n                }\n            } else {\n                println!(\"⏭️  跳过代码生成，仅保留计划（未验证）\");\n            }\n            \n            // 标记为完成\n            self.mark_stage_completed(&mut meta, Stage::Coding, code_artifact.meta.artifact_id.clone(), execution_verified)?;\n\n            if !hitl.confirm(\"继续代码检查？\")? {\n                return Ok(());\n            }\n            \n            code_artifact\n        };\n\n        // Stage 6: Check（支持智能重试）\n        const MAX_RETRY: usize = 3;\n        let mut retry_count = 0;\n        let check_artifact = loop {\n            if meta.completed_stages.contains(&Stage::Check) && retry_count == 0 {\n                println!(\"✓ 跳过 Stage 6: Check (已完成)\");\n                break self.load_artifact::<crate::artifacts::CheckReportArtifact>(session_id, Stage::Check)?;\n            }\n            \n            if retry_count > 0 {\n                println!(\"\\n🔄 智能重试 Check 阶段 (第 {} 次)\", retry_count);\n            } else {\n                println!(\"\\n╔═══════════════════════════════════════╗\");\n                println!(\"║   Stage 6: Quality Check              ║\");\n                println!(\"╚═══════════════════════════════════════╝\\n\");\n            }\n            \n            let check_agent = CheckAgent::new(&model_config.llm, self.store.clone())?;\n            let check_artifact = check_agent.execute(session_id, &code_artifact).await?;\n            \n            meta.current_stage = Some(Stage::Check);\n            if !meta.completed_stages.contains(&Stage::Check) {\n                meta.completed_stages.push(Stage::Check);\n            }\n            self.save_session_meta(&meta)?;\n\n            self.print_check_summary(&check_artifact);\n            \n            // 使用 ErrorAnalyzer 分析错误\n            let error_analysis = crate::agents::ErrorAnalyzer::analyze(&check_artifact.data);\n            \n            if error_analysis.has_critical_errors && retry_count < MAX_RETRY {\n                println!(\"\\n⚠️  发现 {} 个严重问题:\", \n                    check_artifact.data.issues.iter().filter(|i| i.sev == \"error\").count());\n                println!(\"{}\", error_analysis.summary);\n                println!(\"\\n受影响的文件 ({} 个):\", error_analysis.affected_files.len());\n                for file in &error_analysis.affected_files {\n                    println!(\"  - {}\", file);\n                    if let Some(errors) = error_analysis.error_details_by_file.get(file) {\n                        for error in errors.iter().take(2) {  // 只显示前 2 个\n                            println!(\"    {}\", error);\n                        }\n                    }\n                }\n                \n                if hitl.confirm(&format!(\"是否针对性修复这些文件？ ({}/{} 次重试)\", retry_count + 1, MAX_RETRY))? {\n                    println!(\"\\n🔧 执行针对性修复（只重新生成受影响的文件）...\\n\");\n                    \n                    // 创建只包含受影响文件的修复计划\n                    let fix_changes: Vec<crate::artifacts::Change> = code_artifact.data.changes.iter()\n                        .filter(|c| error_analysis.affected_files.contains(&c.path))\n                        .cloned()\n                        .collect();\n                    \n                    if fix_changes.is_empty() {\n                        println!(\"⚠️  无法识别受影响的文件，跳过重试\");\n                        break check_artifact;\n                    }\n                    \n                    println!(\"📝 修复计划: 重新生成 {} 个文件\", fix_changes.len());\n                    for change in &fix_changes {\n                        println!(\"  - {}\", change.path);\n                    }\n                    println!();\n                    \n                    // 创建临时的 CodeChangeArtifact（只包含需要修复的文件）\n                    let mut fix_artifact = code_artifact.clone();\n                    fix_artifact.data.changes = fix_changes;\n                    \n                    // 构建修复指令（包含错误信息）\n                    let fix_context = format!(\n                        \"Previous generation (attempt {}) had the following errors:\\n\\n{}\\n\\n\\\n                        IMPORTANT:\\n\\\n                        - Focus on fixing the specific errors mentioned above\\n\\\n                        - Only modify the files that have errors\\n\\\n                        - Ensure the code compiles and runs correctly\",\n                        retry_count,\n                        error_analysis.detailed_errors\n                    );\n                    \n                    // 执行针对性修复（使用完整 API）\n                    let executor = crate::agents::CodeExecutor::new(&model_config.llm)?;\n                    \n                    // 提取 PRD 摘要（WatchDog）\n                    let prd_summary = crate::utils::extract_prd_summary(&prd_artifact);\n                    \n                    // 获取 TodoList（可变引用）\n                    let mut todo_list = plan_artifact.data.todo_list.clone();\n                    \n                    println!(\"💡 修复提示:\\n{}\\n\", fix_context);\n                    \n                    match executor.execute_with_todo(\n                        &fix_artifact,\n                        &hitl,\n                        Some(&prd_summary),      // WatchDog 提醒\n                        todo_list.as_mut(),       // TodoList 更新\n                    ).await {\n                        Ok(report) => {\n                            println!(\"\\n针对性修复完成:\");\n                            println!(\"  ✅ 成功: {}\", report.successful);\n                            println!(\"  ❌ 失败: {}\", report.failed);\n                            \n                            // 保存更新后的 TodoList\n                            if let Some(updated_todo_list) = todo_list {\n                                plan_artifact.data.todo_list = Some(updated_todo_list);\n                                self.store.put(session_id, Stage::Plan, &plan_artifact)?;\n                            }\n                            \n                            if report.failed == 0 && report.successful > 0 {\n                                // 更新 Coding 阶段为已验证\n                                self.mark_stage_completed(&mut meta, Stage::Coding, code_artifact.meta.artifact_id.clone(), true)?;\n                                retry_count += 1;\n                                continue;  // 重新运行 Check\n                            } else {\n                                println!(\"⚠️  部分文件修复失败\");\n                                if retry_count + 1 < MAX_RETRY {\n                                    println!(\"提示：还有 {} 次重试机会\", MAX_RETRY - retry_count - 1);\n                                }\n                            }\n                        }\n                        Err(e) => {\n                            tracing::error!(\"Targeted fix failed: {}\", e);\n                            println!(\"❌ 针对性修复失败: {}\", e);\n                        }\n                    }\n                } else {\n                    println!(\"用户选择不重试，继续下一步\");\n                }\n            }\n            \n            break check_artifact;\n        };\n\n        // Stage 7: Feedback (Optional)\n        let user_feedback = hitl.input(\"有反馈吗？（直接回车跳过）\")?;\n        \n        if !user_feedback.trim().is_empty() {\n            println!(\"\\n╔═══════════════════════════════════════╗\");\n            println!(\"║   Stage 7: Feedback Analysis          ║\");\n            println!(\"╚═══════════════════════════════════════╝\\n\");\n            \n            let feedback_agent = FeedbackAgent::new(&model_config.llm, self.store.clone())?;\n            let feedback_artifact = feedback_agent.execute(session_id, &check_artifact, &user_feedback).await?;\n            \n            meta.current_stage = Some(Stage::Feedback);\n            meta.completed_stages.push(Stage::Feedback);\n            self.save_session_meta(&meta)?;\n\n            self.print_feedback_summary(&feedback_artifact);\n\n            if !feedback_artifact.data.rerun.is_empty() {\n                println!(\"\\n⚠️  需要重新执行以下阶段：\");\n                for rerun in &feedback_artifact.data.rerun {\n                    println!(\"  - {:?}: {}\", rerun.stage, rerun.reason);\n                }\n                println!(\"\\n提示：使用 'cowork resume {}' 继续迭代\", session_id);\n                return Ok(());\n            }\n        }\n\n        // Stage 8: Delivery\n        if !meta.completed_stages.contains(&Stage::Delivery) {\n            println!(\"\\n╔═══════════════════════════════════════╗\");\n            println!(\"║   Stage 8: Delivery Report            ║\");\n            println!(\"╚═══════════════════════════════════════╝\\n\");\n            \n            let delivery_agent = DeliveryAgent::new(&model_config.llm, self.store.clone())?;\n            let delivery_artifact = delivery_agent.execute(session_id, &check_artifact, &idea_artifact).await?;\n            \n            meta.current_stage = Some(Stage::Delivery);\n            meta.completed_stages.push(Stage::Delivery);\n            self.save_session_meta(&meta)?;\n\n            self.print_delivery_summary(&delivery_artifact);\n        } else {\n            println!(\"✓ 跳过 Stage 8: Delivery (已完成)\");\n        }\n\n        println!(\"\\n╔═══════════════════════════════════════╗\");\n        println!(\"║   🎉 工作流完成！                     ║\");\n        println!(\"╚═══════════════════════════════════════╝\\n\");\n        println!(\"Session ID: {}\", session_id);\n        println!(\"Artifacts: .cowork/{}/artifacts/\", session_id);\n        println!(\"\\n完成的阶段: {:?}\", meta.completed_stages);\n\n        Ok(())\n    }\n\n    /// 从文件系统加载指定阶段的 artifact\n    fn load_artifact<T>(&self, session_id: &str, stage: Stage) -> Result<T>\n    where\n        T: serde::de::DeserializeOwned,\n    {\n        use std::fs;\n\n        let artifacts = self.store.list(session_id)?;\n        \n        // 找到该阶段的最新 artifact\n        let artifact_meta = artifacts\n            .iter()\n            .filter(|a| a.stage == stage)\n            .max_by_key(|a| &a.path_json)\n            .ok_or_else(|| anyhow::anyhow!(\"No artifact found for stage {:?}\", stage))?;\n\n        let content = fs::read_to_string(&artifact_meta.path_json)?;\n        let artifact: T = serde_json::from_str(&content)?;\n        \n        tracing::info!(\"Loaded artifact for stage {:?} from {}\", stage, artifact_meta.path_json.display());\n        \n        Ok(artifact)\n    }\n\n    /// 恢复会话（从中断点继续）\n    pub async fn resume_session(&self, session_id: &str, model_config: &ModelConfig) -> Result<()> {\n        // 检查 session 是否存在\n        if !self.store.session_exists(session_id) {\n            return Err(anyhow::anyhow!(\"Session {} not found\", session_id));\n        }\n\n        // 加载 session meta\n        let meta = self.load_session_meta(session_id)?;\n        \n        // 确定下一个要执行的阶段\n        let all_stages = Stage::all();\n        let next_stage = all_stages\n            .iter()\n            .find(|s| !meta.completed_stages.contains(s))\n            .cloned();\n\n        if let Some(stage) = next_stage {\n            println!(\"\\n📋 恢复会话: {}\", session_id);\n            println!(\"已完成: {:?}\", meta.completed_stages);\n            println!(\"下一阶段: {:?}\", stage);\n            println!();\n            \n            self.run_workflow_from_stage(session_id, model_config, Some(stage)).await\n        } else {\n            println!(\"\\n✅ 会话 {} 已全部完成\", session_id);\n            println!(\"完成的阶段: {:?}\", meta.completed_stages);\n            Ok(())\n        }\n    }\n\n    /// 运行完整的 8 阶段工作流（旧版本，保持兼容）\n    #[allow(dead_code)]\n    async fn run_full_workflow_legacy(&self, session_id: &str, model_config: &ModelConfig) -> Result<()> {\n        tracing::info!(\"Running full workflow for session: {}\", session_id);\n\n        let hitl = HitlController::new();\n        let mut meta = self.load_session_meta(session_id)?;\n\n        // ===== Stage 1: IDEA Intake =====\n        println!(\"\\n╔═══════════════════════════════════════╗\");\n        println!(\"║   Stage 1: IDEA Intake               ║\");\n        println!(\"╚═══════════════════════════════════════╝\\n\");\n        \n        let user_idea = hitl.input(\"请描述你的 IDEA：\")?;\n        \n        let idea_agent = IdeaIntakeAgent::new(&model_config.llm, self.store.clone())?;\n        let mut idea_artifact = idea_agent.execute(session_id, &user_idea).await?;\n        \n        // HITL 审查和修改\n        if let Some(modified_json) = hitl.review_and_edit_json(\"IdeaSpec\", &idea_artifact.data)? {\n            let modified_data: crate::artifacts::IdeaSpec = serde_json::from_str(&modified_json)?;\n            idea_artifact.data = modified_data;\n            // 重新保存修改后的artifact\n            self.store.put(session_id, Stage::IdeaIntake, &idea_artifact)?;\n            println!(\"✅ IdeaSpec 已更新\");\n        }\n        \n        meta.current_stage = Some(Stage::IdeaIntake);\n        meta.completed_stages.push(Stage::IdeaIntake);\n        self.save_session_meta(&meta)?;\n\n        self.print_idea_summary(&idea_artifact);\n\n        if !hitl.confirm(\"继续生成 PRD？\")? {\n            return Ok(());\n        }\n\n        // ===== Stage 2: PRD Generation =====\n        println!(\"\\n╔═══════════════════════════════════════╗\");\n        println!(\"║   Stage 2: Requirements (PRD)        ║\");\n        println!(\"╚═══════════════════════════════════════╝\\n\");\n        \n        let prd_agent = PrdAgent::new(&model_config.llm, self.store.clone())?;\n        let mut prd_artifact = prd_agent.execute(session_id, &idea_artifact).await?;\n        \n        // HITL 审查和修改\n        if let Some(modified_json) = hitl.review_and_edit_json(\"PRD\", &prd_artifact.data)? {\n            let modified_data: crate::artifacts::PRD = serde_json::from_str(&modified_json)?;\n            prd_artifact.data = modified_data;\n            self.store.put(session_id, Stage::Requirements, &prd_artifact)?;\n            println!(\"✅ PRD 已更新\");\n        }\n        \n        meta.current_stage = Some(Stage::Requirements);\n        meta.completed_stages.push(Stage::Requirements);\n        self.save_session_meta(&meta)?;\n\n        self.print_prd_summary(&prd_artifact);\n\n        if !hitl.confirm(\"继续生成设计文档？\")? {\n            return Ok(());\n        }\n\n        // ===== Stage 3: Design =====\n        println!(\"\\n╔═══════════════════════════════════════╗\");\n        println!(\"║   Stage 3: Design Document            ║\");\n        println!(\"╚═══════════════════════════════════════╝\\n\");\n        \n        let design_agent = DesignAgent::new(&model_config.llm, self.store.clone())?;\n        let mut design_artifact = design_agent.execute(session_id, &prd_artifact).await?;\n        \n        // HITL 审查和修改\n        if let Some(modified_json) = hitl.review_and_edit_json(\"DesignDoc\", &design_artifact.data)? {\n            let modified_data: crate::artifacts::DesignDoc = serde_json::from_str(&modified_json)?;\n            design_artifact.data = modified_data;\n            self.store.put(session_id, Stage::Design, &design_artifact)?;\n            println!(\"✅ DesignDoc 已更新\");\n        }\n        \n        meta.current_stage = Some(Stage::Design);\n        meta.completed_stages.push(Stage::Design);\n        self.save_session_meta(&meta)?;\n\n        self.print_design_summary(&design_artifact);\n\n        if !hitl.confirm(\"继续生成实施计划？\")? {\n            return Ok(());\n        }\n\n        // ===== Stage 4: Plan =====\n        println!(\"\\n╔═══════════════════════════════════════╗\");\n        println!(\"║   Stage 4: Implementation Plan        ║\");\n        println!(\"╚═══════════════════════════════════════╝\\n\");\n        \n        let plan_agent = PlanAgent::new(&model_config.llm, self.store.clone())?;\n        let mut plan_artifact = plan_agent.execute(session_id, &design_artifact).await?;\n        \n        // HITL 审查和修改\n        if let Some(modified_json) = hitl.review_and_edit_json(\"Plan\", &plan_artifact.data)? {\n            let modified_data: crate::artifacts::Plan = serde_json::from_str(&modified_json)?;\n            plan_artifact.data = modified_data;\n            self.store.put(session_id, Stage::Plan, &plan_artifact)?;\n            println!(\"✅ Plan 已更新\");\n        }\n        \n        meta.current_stage = Some(Stage::Plan);\n        meta.completed_stages.push(Stage::Plan);\n        self.save_session_meta(&meta)?;\n\n        self.print_plan_summary(&plan_artifact);\n\n        if !hitl.confirm(\"继续生成代码？\")? {\n            return Ok(());\n        }\n\n        // ===== Stage 5: Coding =====\n        println!(\"\\n╔═══════════════════════════════════════╗\");\n        println!(\"║   Stage 5: Code Planning              ║\");\n        println!(\"╚═══════════════════════════════════════╝\\n\");\n        \n        let code_planner = CodePlanner::new(&model_config.llm, self.store.clone())?;\n        let code_artifact = code_planner.execute(\n            session_id,\n            &prd_artifact,\n            &design_artifact,\n            &plan_artifact\n        ).await?;\n        \n        meta.current_stage = Some(Stage::Coding);\n        meta.completed_stages.push(Stage::Coding);\n        self.save_session_meta(&meta)?;\n\n        self.print_code_summary(&code_artifact);\n\n        if !hitl.confirm(\"继续代码检查？\")? {\n            return Ok(());\n        }\n\n        // ===== Stage 6: Check =====\n        println!(\"\\n╔═══════════════════════════════════════╗\");\n        println!(\"║   Stage 6: Quality Check              ║\");\n        println!(\"╚═══════════════════════════════════════╝\\n\");\n        \n        let check_agent = CheckAgent::new(&model_config.llm, self.store.clone())?;\n        let check_artifact = check_agent.execute(session_id, &code_artifact).await?;\n        \n        meta.current_stage = Some(Stage::Check);\n        meta.completed_stages.push(Stage::Check);\n        self.save_session_meta(&meta)?;\n\n        self.print_check_summary(&check_artifact);\n\n        // ===== Stage 7: Feedback (Optional) =====\n        let user_feedback = hitl.input(\"有反馈吗？（直接回车跳过）\")?;\n        \n        if !user_feedback.trim().is_empty() {\n            println!(\"\\n╔═══════════════════════════════════════╗\");\n            println!(\"║   Stage 7: Feedback Analysis          ║\");\n            println!(\"╚═══════════════════════════════════════╝\\n\");\n            \n            let feedback_agent = FeedbackAgent::new(&model_config.llm, self.store.clone())?;\n            let feedback_artifact = feedback_agent.execute(session_id, &check_artifact, &user_feedback).await?;\n            \n            meta.current_stage = Some(Stage::Feedback);\n            meta.completed_stages.push(Stage::Feedback);\n            self.save_session_meta(&meta)?;\n\n            self.print_feedback_summary(&feedback_artifact);\n\n            if !feedback_artifact.data.rerun.is_empty() {\n                println!(\"\\n⚠️  需要重新执行以下阶段：\");\n                for rerun in &feedback_artifact.data.rerun {\n                    println!(\"  - {:?}: {}\", rerun.stage, rerun.reason);\n                }\n                println!(\"\\n提示：使用 'cowork resume {}' 继续迭代\", session_id);\n                return Ok(());\n            }\n        }\n\n        // ===== Stage 8: Delivery =====\n        println!(\"\\n╔═══════════════════════════════════════╗\");\n        println!(\"║   Stage 8: Delivery Report            ║\");\n        println!(\"╚═══════════════════════════════════════╝\\n\");\n        \n        let delivery_agent = DeliveryAgent::new(&model_config.llm, self.store.clone())?;\n        let delivery_artifact = delivery_agent.execute(session_id, &check_artifact, &idea_artifact).await?;\n        \n        meta.current_stage = Some(Stage::Delivery);\n        meta.completed_stages.push(Stage::Delivery);\n        self.save_session_meta(&meta)?;\n\n        self.print_delivery_summary(&delivery_artifact);\n\n        println!(\"\\n╔═══════════════════════════════════════╗\");\n        println!(\"║   🎉 工作流完成！                     ║\");\n        println!(\"╚═══════════════════════════════════════╝\\n\");\n        println!(\"Session ID: {}\", session_id);\n        println!(\"Artifacts: .cowork/{}/artifacts/\", session_id);\n        println!(\"\\n完成的阶段: {:?}\", meta.completed_stages);\n\n        Ok(())\n    }\n\n    /// 列出 session 的所有 artifacts\n    pub fn list_artifacts(&self, session_id: &str) -> Result<Vec<crate::memory::ArtifactMeta>> {\n        self.store.list(session_id)\n    }\n\n    // Helper methods for printing summaries\n    fn print_idea_summary(&self, artifact: &crate::artifacts::IdeaSpecArtifact) {\n        println!(\"✓ IdeaSpec 生成成功！\");\n        println!(\"  背景: {}\", artifact.data.bg);\n        println!(\"  目标: {} 项\", artifact.data.g.len());\n        println!(\"  非目标: {} 项\", artifact.data.ng.len());\n        println!(\"  约束: {} 项\", artifact.data.c.len());\n    }\n\n    fn print_prd_summary(&self, artifact: &crate::artifacts::PRDArtifact) {\n        println!(\"✓ PRD 生成成功！\");\n        println!(\"  需求总数: {}\", artifact.data.reqs.len());\n        println!(\"    - P0: {}\", artifact.data.reqs.iter().filter(|r| matches!(r.pri, crate::artifacts::Priority::P0)).count());\n        println!(\"    - P1: {}\", artifact.data.reqs.iter().filter(|r| matches!(r.pri, crate::artifacts::Priority::P1)).count());\n        println!(\"    - P2: {}\", artifact.data.reqs.iter().filter(|r| matches!(r.pri, crate::artifacts::Priority::P2)).count());\n        println!(\"  约束: {}\", artifact.data.cons.len());\n        println!(\"  待确认问题: {}\", artifact.data.hitl.len());\n    }\n\n    fn print_design_summary(&self, artifact: &crate::artifacts::DesignDocArtifact) {\n        println!(\"✓ 设计文档生成成功！\");\n        println!(\"  CLI 模式: {:?}\", artifact.data.cli.modes);\n        println!(\"  工作流阶段: {}\", artifact.data.wf.stages.len());\n        println!(\"  架构层次: {:?}\", artifact.data.arch.layers);\n    }\n\n    fn print_plan_summary(&self, artifact: &crate::artifacts::PlanArtifact) {\n        println!(\"✓ 实施计划生成成功！\");\n        println!(\"  C4 上下文: {}\", artifact.data.c4.context.len());\n        println!(\"  任务总数: {}\", artifact.data.tasks.len());\n        println!(\"  里程碑: {}\", artifact.data.milestones.len());\n    }\n\n    fn print_code_summary(&self, artifact: &crate::artifacts::CodeChangeArtifact) {\n        println!(\"✓ 代码结构生成成功！\");\n        println!(\"  语言: {}\", artifact.data.target.lang);\n        println!(\"  模块: {}\", artifact.data.project.modules.len());\n        println!(\"  文件变更: {}\", artifact.data.changes.len());\n        println!(\"  命令: {}\", artifact.data.cmds.len());\n    }\n\n    fn print_check_summary(&self, artifact: &crate::artifacts::CheckReportArtifact) {\n        println!(\"✓ 检查报告生成完成！\");\n        println!(\"  检查项: {}\", artifact.data.checks.len());\n        println!(\"  发现问题: {}\", artifact.data.issues.len());\n    }\n\n    fn print_feedback_summary(&self, artifact: &crate::artifacts::FeedbackArtifact) {\n        println!(\"✓ 反馈分析完成！\");\n        println!(\"  需要修改: {} 处\", artifact.data.delta.len());\n        println!(\"  需要重跑: {} 个阶段\", artifact.data.rerun.len());\n    }\n\n    fn print_delivery_summary(&self, artifact: &crate::artifacts::DeliveryReportArtifact) {\n        println!(\"✓ 交付报告生成完成！\");\n        println!(\"  功能: {} 项\", artifact.data.cap.len());\n        println!(\"  使用说明: {} 条\", artifact.data.howto.len());\n        println!(\"  已知限制: {} 项\", artifact.data.limits.len());\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 64.0,
      "lines_of_code": 980,
      "number_of_classes": 1,
      "number_of_functions": 24
    },
    "dependencies": [
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "anyhow",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "serde",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "rust_std",
        "is_external": false,
        "line_number": null,
        "name": "std",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "uuid",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "chrono",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "tracing",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": null,
        "name": "serde_json",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::artifacts::Stage",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::memory::ArtifactStore",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::agents::IdeaIntakeAgent",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::agents::PrdAgent",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::agents::DesignAgent",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::agents::PlanAgent",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::agents::CodePlanner",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::agents::CheckAgent",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::agents::FeedbackAgent",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::agents::DeliveryAgent",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::hitl::HitlController",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::config::ModelConfig",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::utils::extract_prd_summary",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::agents::ErrorAnalyzer",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::agents::CodeExecutor",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::memory::ArtifactMeta",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::artifacts::IdeaSpecArtifact",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::artifacts::PRDArtifact",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::artifacts::DesignDocArtifact",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::artifacts::PlanArtifact",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::artifacts::CodeChangeArtifact",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::artifacts::CheckReportArtifact",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::artifacts::FeedbackArtifact",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": null,
        "name": "crate::artifacts::DeliveryReportArtifact",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "The orchestrator module is the central workflow engine of the Cowork AI-assisted development system. It manages an 8-stage iterative development lifecycle from idea intake to delivery, coordinating AI agents, human-in-the-loop (HITL) interactions, and artifact persistence. The Orchestrator struct coordinates the execution flow, handles session state management via file-based storage, implements intelligent resume capabilities, and supports retry logic with targeted fixes. It integrates with various AI agents (IdeaIntake, PRD, Design, etc.) and uses a structured artifact system to pass data between stages. The orchestrator enables both full workflow execution and partial resumption from any stage, making it resilient to interruptions. It provides rich CLI feedback, summary printing, and validation mechanisms to guide users through complex AI-driven development processes.",
    "interfaces": [
      {
        "description": null,
        "interface_type": "enum",
        "name": "Stage",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "enum",
        "name": "StageStatus",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "SessionMeta",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "session_id",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "created_at",
            "param_type": "DateTime<Utc>"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "current_stage",
            "param_type": "Option<Stage>"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "stage_status",
            "param_type": "HashMap<Stage, StageStatus>"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "completed_stages",
            "param_type": "Vec<Stage>"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "trait",
        "name": "ArtifactStore",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "HitlController",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "ModelConfig",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "IdeaIntakeAgent",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "PrdAgent",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "DesignAgent",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "PlanAgent",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "CodePlanner",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "CheckAgent",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "FeedbackAgent",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "DeliveryAgent",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "ErrorAnalyzer",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "CodeExecutor",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "ArtifactEnvelope",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "IdeaSpec",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "PRD",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "DesignDoc",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "Plan",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "CodeChangeArtifact",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "CheckReportArtifact",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "FeedbackArtifact",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": null,
        "interface_type": "struct",
        "name": "DeliveryReportArtifact",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Orchestrating the 8-stage AI-assisted development workflow",
      "Managing session state and artifact persistence across stages",
      "Implementing intelligent resume and retry logic with targeted fixes",
      "Coordinating human-in-the-loop (HITL) interactions and validation",
      "Providing structured artifact flow and versioning between AI agents"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "command",
      "description": null,
      "file_path": "crates/cowork-cli/src/server.rs",
      "functions": [],
      "importance_score": 0.6,
      "interfaces": [],
      "name": "server.rs",
      "source_summary": ""
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 1.0,
      "lines_of_code": 0,
      "number_of_classes": 0,
      "number_of_functions": 0
    },
    "dependencies": [],
    "detailed_description": "The server.rs component appears to be an empty file with no actual code implementation. Based on the component type 'Command' and file path, this component is likely intended to be a command-line interface (CLI) command for server-related operations in the cowork-cli application. However, the complete absence of code suggests this component is either in early development phase, placeholder for future implementation, or potentially abandoned.",
    "interfaces": [],
    "responsibilities": [
      "Placeholder for server command functionality",
      "Future implementation of server management operations",
      "CLI command structure definition"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "tool",
      "description": "Module file that re-exports file_tools functionality and organizes test modules",
      "file_path": "crates/cowork-core/src/tools/mod.rs",
      "functions": [
        "create_file_tools"
      ],
      "importance_score": 0.6,
      "interfaces": [
        "FileToolsBundle"
      ],
      "name": "mod.rs",
      "source_summary": "mod file_tools;\n\n#[cfg(test)]\nmod file_tools_tests;\n\npub use file_tools::{create_file_tools, FileToolsBundle};\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 1.0,
      "lines_of_code": 6,
      "number_of_classes": 0,
      "number_of_functions": 0
    },
    "dependencies": [
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": 1,
        "name": "file_tools",
        "path": "./crates/cowork-core/src/tools/file_tools.rs",
        "version": null
      },
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": 4,
        "name": "file_tools_tests",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "This is a Rust module file that serves as an entry point for the tools module. It declares the file_tools submodule and its corresponding test module (conditionally compiled for test builds). The primary functionality is re-exporting the file_tools module's public API, specifically the create_file_tools function and FileToolsBundle struct, making them accessible to external consumers of the tools module. This is a typical Rust module organization pattern where the mod.rs file acts as a facade that groups related functionality.",
    "interfaces": [
      {
        "description": "Factory function that creates and returns a FileToolsBundle instance containing all file operation tools",
        "interface_type": "function",
        "name": "create_file_tools",
        "parameters": [],
        "return_type": "FileToolsBundle",
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Module organization and namespace management",
      "Public API exposure and re-exporting",
      "Test module organization (conditional compilation)",
      "Dependency declaration and module structure definition"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "module",
      "description": null,
      "file_path": "crates/cowork-core/src/lib.rs",
      "functions": [],
      "importance_score": 0.6,
      "interfaces": [],
      "name": "lib.rs",
      "source_summary": "pub mod artifacts;\npub mod memory;\npub mod config;\npub mod tools;\npub mod agents;\npub mod orchestrator;\npub mod hitl;\npub mod utils;\n\npub use artifacts::{Stage, ArtifactEnvelope};\npub use memory::ArtifactStore;\npub use config::ModelConfig;\npub use orchestrator::Orchestrator;\npub use hitl::HitlController;\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 1.0,
      "lines_of_code": 14,
      "number_of_classes": 0,
      "number_of_functions": 0
    },
    "dependencies": [],
    "detailed_description": "This component serves as the public entry point and module orchestrator for the cowork-core crate. It organizes and re-exports key modules and types from submodules such as artifacts, memory, config, tools, agents, orchestrator, hitl, and utils. By using pub use statements, it provides a consolidated public API surface that allows consumers of the crate to access essential types and structures without needing to import from deep module paths. This facilitates a clean, hierarchical API design where the core crate exposes only the necessary interfaces while maintaining internal modularity.",
    "interfaces": [],
    "responsibilities": [
      "Orchestrating submodule organization",
      "Providing a consolidated public API for the cowork-core crate",
      "Re-exporting critical types and structures for external consumption",
      "Maintaining module boundaries and encapsulation",
      "Enabling modular development through logical grouping of functionality"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "util",
      "description": null,
      "file_path": "crates/cowork-core/src/utils/mod.rs",
      "functions": [
        "extract_prd_summary"
      ],
      "importance_score": 0.6,
      "interfaces": [],
      "name": "mod.rs",
      "source_summary": "pub mod prd_utils;\n\npub use prd_utils::extract_prd_summary;\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 1.0,
      "lines_of_code": 3,
      "number_of_classes": 0,
      "number_of_functions": 1
    },
    "dependencies": [],
    "detailed_description": "This module serves as a re-export layer for utility functions related to product (PRD) summary extraction. It imports the 'prd_utils' submodule and publicly re-exports the 'extract_prd_summary' function, enabling other parts of the codebase to access this utility without directly referencing its internal location. This pattern enhances modularity and reduces coupling by providing a centralized entry point for utility functions under the 'utils' namespace.",
    "interfaces": [],
    "responsibilities": [
      "Re-exporting the extract_prd_summary function from prd_utils",
      "Providing a clean, centralized interface for utility functions in the utils module",
      "Maintaining abstraction by hiding the internal structure of the prd_utils submodule"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "dao",
      "description": "Placeholder component for embedded data layer implementation",
      "file_path": "crates/cowork-core/src/data.rs",
      "functions": [],
      "importance_score": 0.6,
      "interfaces": [],
      "name": "data.rs",
      "source_summary": "// TODO: Implement - Implement embedded data layer with sample classical poems\n// File: crates/cowork-core/src/data.rs\n\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 1.0,
      "lines_of_code": 3,
      "number_of_classes": 0,
      "number_of_functions": 0
    },
    "dependencies": [],
    "detailed_description": "This is a placeholder component with only a TODO comment indicating it needs to be implemented as an embedded data layer with sample classical poems. The file currently contains no functional code, only a comment describing the intended functionality. Based on the file path and naming convention, this appears to be intended as a data access layer component for the cowork-core crate.",
    "interfaces": [],
    "responsibilities": [
      "Provide embedded data storage functionality",
      "Manage sample classical poems data",
      "Implement data persistence layer",
      "Handle data access operations",
      "Provide data layer abstraction for the core system"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "service",
      "description": "Placeholder component for core business logic implementation",
      "file_path": "crates/cowork-core/src/workflow.rs",
      "functions": [],
      "importance_score": 0.6,
      "interfaces": [],
      "name": "workflow.rs",
      "source_summary": "// TODO: Implement - Develop core business logic for poem selection and rendering\n// File: crates/cowork-core/src/workflow.rs\n\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 2.0,
      "lines_of_code": 3,
      "number_of_classes": 0,
      "number_of_functions": 0
    },
    "dependencies": [],
    "detailed_description": "This is an incomplete/placeholder component that currently contains only a TODO comment indicating the need to implement core business logic for poem selection and rendering. The file is essentially empty with no actual implementation, serving as a stub for future development.",
    "interfaces": [],
    "responsibilities": [
      "Core business logic implementation (pending)",
      "Poem selection algorithm development (pending)",
      "Rendering logic implementation (pending)",
      "Workflow orchestration (pending)",
      "Business rule enforcement (pending)"
    ]
  }
]
```

## Memory Storage Statistics

**Total Storage Size**: 627548 bytes

- **timing**: 39 bytes (0.0%)
- **studies_research**: 85543 bytes (13.6%)
- **documentation**: 132908 bytes (21.2%)
- **preprocess**: 409058 bytes (65.2%)

## Generated Documents Statistics

Number of Generated Documents: 13

- Key Modules and Components Research Report_人工介入域
- Boundary Interfaces
- Core Workflows
- Project Overview
- Architecture Description
- Key Modules and Components Research Report_智能体执行域
- Key Modules and Components Research Report_数据模型域
- Key Modules and Components Research Report_工具函数域
- Key Modules and Components Research Report_配置管理域
- Key Modules and Components Research Report_文件工具域
- Key Modules and Components Research Report_用户界面域
- Key Modules and Components Research Report_库接口域
- Key Modules and Components Research Report_工作流编排域
