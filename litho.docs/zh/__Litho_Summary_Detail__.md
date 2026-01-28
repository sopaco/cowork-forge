# Project Analysis Summary Report (Full Version)

Generation Time: 2026-01-28 11:22:56 UTC

## Execution Timing Statistics

- **Total Execution Time**: 1530.49 seconds
- **Preprocessing Phase**: 1.04 seconds (0.1%)
- **Research Phase**: 465.73 seconds (30.4%)
- **Document Generation Phase**: 1063.72 seconds (69.5%)
- **Output Phase**: 0.00 seconds (0.0%)
- **Summary Generation Time**: 0.001 seconds

## Cache Performance Statistics and Savings

### Performance Metrics
- **Cache Hit Rate**: 68.1%
- **Total Operations**: 94
- **Cache Hits**: 64 times
- **Cache Misses**: 30 times
- **Cache Writes**: 31 times

### Savings
- **Inference Time Saved**: 306.5 seconds
- **Tokens Saved**: 85275 input + 34443 output = 119718 total
- **Estimated Cost Savings**: $0.0580
- **Performance Improvement**: 68.1%
- **Efficiency Improvement Ratio**: 0.2x (saved time / actual execution time)

## Core Research Data Summary

Complete content of four types of research materials according to Prompt template data integration rules:

### System Context Research Report
Provides core objectives, user roles, and system boundary information for the project.

```json
{
  "business_value": "通过AI代理协作和人类监督相结合的方式，显著提升软件开发效率和质量。系统能够自动化处理软件开发流程中的重复性任务，同时通过人类专家参与确保关键决策的质量，为软件开发团队提供高效、可靠的开发编排工具。",
  "confidence_score": 0.95,
  "external_systems": [
    {
      "description": "提供大语言模型服务用于智能代理的决策和代码生成",
      "interaction_type": "API调用",
      "name": "OpenAI API"
    },
    {
      "description": "Git等版本控制系统用于代码版本管理和变更跟踪",
      "interaction_type": "文件系统操作",
      "name": "版本控制系统"
    },
    {
      "description": "用户默认文本编辑器用于HITL环节的内容编辑",
      "interaction_type": "进程调用",
      "name": "文本编辑器"
    }
  ],
  "project_description": "一个AI驱动的软件开发编排系统，通过智能代理和人类参与循环(HITL)实现从创意到交付的完整软件开发流程自动化。系统采用命令行界面作为主要交互方式，包含需求分析、架构设计、实现规划、编码实施、质量检查和交付报告等完整开发阶段。",
  "project_name": "Cowork Forge",
  "project_type": "CLITool",
  "system_boundary": {
    "excluded_components": [
      "具体的业务逻辑实现",
      "用户界面开发",
      "部署和运维功能",
      "第三方服务集成开发"
    ],
    "included_components": [
      "命令行界面(CLI)",
      "智能代理系统(Agent)",
      "工作流管道(Pipeline)",
      "数据存储层(Storage)",
      "工具集(Tools)",
      "LLM集成模块"
    ],
    "scope": "软件开发流程编排和自动化"
  },
  "target_users": [
    {
      "description": "需要高效开发软件项目的技术团队，包括项目经理、架构师和开发人员",
      "name": "软件开发团队",
      "needs": [
        "自动化开发流程",
        "标准化开发规范",
        "团队协作支持",
        "质量保证机制"
      ]
    },
    {
      "description": "个人开发者或小型团队，需要系统化的开发工具来管理项目生命周期",
      "name": "独立开发者",
      "needs": [
        "简化开发流程",
        "减少重复工作",
        "确保代码质量",
        "项目文档自动化"
      ]
    },
    {
      "description": "负责项目管理和技术决策的管理人员",
      "name": "技术项目经理",
      "needs": [
        "项目进度可视化",
        "质量检查自动化",
        "变更管理支持",
        "交付文档生成"
      ]
    }
  ]
}
```

### Domain Modules Research Report
Provides high-level domain division, module relationships, and core business process information.

```json
{
  "architecture_summary": "Cowork Forge采用分层架构设计，核心是基于AI代理的软件开发编排系统。架构分为应用层（CLI）、编排层（Pipeline）、代理层（Agents）、指令层（Instructions）、工具层（Tools）、数据层（Data/Storage）和基础设施层（LLM）。系统采用Actor-Critic模式结合Human-in-the-Loop机制，实现从创意到交付的完整软件开发流程自动化。技术栈主要基于Rust语言，采用模块化设计支持可扩展性。",
  "business_flows": [
    {
      "description": "新项目创建流程，从用户创意输入开始，经过需求分析、架构设计、实现规划、编码实施、质量检查到最终交付的完整开发周期",
      "entry_point": "main.rs命令行接口",
      "importance": 10.0,
      "involved_domains_count": 6,
      "name": "新项目开发流程",
      "steps": [
        {
          "code_entry_point": "crates/cowork-core/src/instructions/idea.rs",
          "domain_module": "需求管理域",
          "operation": "创意采集和结构化处理",
          "step": 1,
          "sub_module": "创意代理"
        },
        {
          "code_entry_point": "crates/cowork-core/src/instructions/prd.rs",
          "domain_module": "需求管理域",
          "operation": "产品需求文档生成和验证",
          "step": 2,
          "sub_module": "PRD代理"
        },
        {
          "code_entry_point": "crates/cowork-core/src/instructions/design.rs",
          "domain_module": "设计管理域",
          "operation": "系统架构设计和评审",
          "step": 3,
          "sub_module": "设计代理"
        },
        {
          "code_entry_point": "crates/cowork-core/src/instructions/plan.rs",
          "domain_module": "规划管理域",
          "operation": "实现任务规划和依赖分析",
          "step": 4,
          "sub_module": "规划代理"
        },
        {
          "code_entry_point": "crates/cowork-core/src/instructions/coding.rs",
          "domain_module": "编码实施域",
          "operation": "代码实现和质量检查",
          "step": 5,
          "sub_module": "编码代理"
        },
        {
          "code_entry_point": "crates/cowork-core/src/instructions/delivery.rs",
          "domain_module": "交付管理域",
          "operation": "项目交付验证和报告生成",
          "step": 6,
          "sub_module": "交付代理"
        }
      ]
    },
    {
      "description": "增量修改流程，处理现有项目的变更请求，包括变更分析、影响评估、代码修改和变更报告生成",
      "entry_point": "main.rs修改命令",
      "importance": 9.0,
      "involved_domains_count": 5,
      "name": "项目修改流程",
      "steps": [
        {
          "code_entry_point": "crates/cowork-core/src/instructions/modify.rs",
          "domain_module": "变更管理域",
          "operation": "变更请求分析和范围确定",
          "step": 1,
          "sub_module": "变更分析代理"
        },
        {
          "code_entry_point": "crates/cowork-core/src/instructions/code_patch.rs",
          "domain_module": "编码实施域",
          "operation": "增量代码修改实施",
          "step": 2,
          "sub_module": "代码补丁代理"
        },
        {
          "code_entry_point": "crates/cowork-core/src/instructions/modify_delivery.rs",
          "domain_module": "交付管理域",
          "operation": "变更报告生成和交付",
          "step": 3,
          "sub_module": "修改交付代理"
        }
      ]
    },
    {
      "description": "工作流恢复和阶段跳转流程，支持从特定阶段重新开始工作流，用于错误恢复和流程调整",
      "entry_point": "goto_stage_tool工具",
      "importance": 7.0,
      "involved_domains_count": 3,
      "name": "流程恢复流程",
      "steps": [
        {
          "code_entry_point": "crates/cowork-core/src/tools/goto_stage_tool.rs",
          "domain_module": "编排管理域",
          "operation": "阶段验证和会话元数据加载",
          "step": 1,
          "sub_module": "阶段跳转工具"
        },
        {
          "code_entry_point": "crates/cowork-core/src/pipeline/mod.rs",
          "domain_module": "编排管理域",
          "operation": "工作流管道重新组装",
          "step": 2,
          "sub_module": "管道编排器"
        },
        {
          "code_entry_point": "crates/cowork-core/src/agents/mod.rs",
          "domain_module": "代理管理域",
          "operation": "代理实例重新创建",
          "step": 3,
          "sub_module": "代理工厂"
        }
      ]
    }
  ],
  "confidence_score": 9.2,
  "domain_modules": [
    {
      "code_paths": [
        "crates/cowork-core/src/instructions/idea.rs",
        "crates/cowork-core/src/instructions/prd.rs",
        "crates/cowork-core/src/data/tools/data_tools.rs"
      ],
      "complexity": 8.0,
      "description": "负责项目需求的采集、分析、结构化和管理，包括创意处理、PRD生成、需求验证和变更管理",
      "domain_type": "核心业务域",
      "importance": 9.0,
      "name": "需求管理域",
      "sub_modules": [
        {
          "code_paths": [
            "crates/cowork-core/src/instructions/idea.rs",
            "crates/cowork-core/src/tools/idea_tools.rs"
          ],
          "description": "创意采集和结构化处理模块",
          "importance": 8.0,
          "key_functions": [
            "创意理解",
            "结构化保存",
            "用户评审"
          ],
          "name": "创意代理模块"
        },
        {
          "code_paths": [
            "crates/cowork-core/src/instructions/prd.rs",
            "crates/cowork-core/src/tools/data_tools.rs"
          ],
          "description": "产品需求文档生成和验证模块",
          "importance": 9.0,
          "key_functions": [
            "需求分析",
            "PRD生成",
            "用户验证",
            "文档保存"
          ],
          "name": "PRD代理模块"
        },
        {
          "code_paths": [
            "crates/cowork-core/src/instructions/modify.rs",
            "crates/cowork-core/src/tools/modify_tools.rs"
          ],
          "description": "变更请求分析和范围确定模块",
          "importance": 8.0,
          "key_functions": [
            "变更分析",
            "影响评估",
            "风险分析",
            "接受标准提取"
          ],
          "name": "变更分析模块"
        }
      ]
    },
    {
      "code_paths": [
        "crates/cowork-core/src/instructions/design.rs",
        "crates/cowork-core/src/tools/validation_tools.rs"
      ],
      "complexity": 9.0,
      "description": "负责系统架构设计、技术方案制定、设计验证和架构文档管理",
      "domain_type": "核心业务域",
      "importance": 9.0,
      "name": "设计管理域",
      "sub_modules": [
        {
          "code_paths": [
            "crates/cowork-core/src/instructions/design.rs",
            "crates/cowork-core/src/tools/hitl_content_tools.rs"
          ],
          "description": "系统架构设计和评审模块",
          "importance": 9.0,
          "key_functions": [
            "架构设计",
            "设计评审",
            "用户反馈整合",
            "设计文档保存"
          ],
          "name": "设计代理模块"
        },
        {
          "code_paths": [
            "crates/cowork-core/src/tools/validation_tools.rs"
          ],
          "description": "设计验证和覆盖率检查模块",
          "importance": 7.0,
          "key_functions": [
            "设计验证",
            "特征覆盖率检查",
            "架构完整性评估"
          ],
          "name": "设计验证模块"
        }
      ]
    },
    {
      "code_paths": [
        "crates/cowork-core/src/instructions/plan.rs",
        "crates/cowork-core/src/tools/validation_tools.rs"
      ],
      "complexity": 7.0,
      "description": "负责实现任务规划、依赖分析、任务分配和计划验证",
      "domain_type": "核心业务域",
      "importance": 8.0,
      "name": "规划管理域",
      "sub_modules": [
        {
          "code_paths": [
            "crates/cowork-core/src/instructions/plan.rs",
            "crates/cowork-core/src/tools/data_tools.rs"
          ],
          "description": "实现任务规划和验证模块",
          "importance": 8.0,
          "key_functions": [
            "任务创建",
            "依赖分析",
            "用户评审",
            "计划验证"
          ],
          "name": "规划代理模块"
        },
        {
          "code_paths": [
            "crates/cowork-core/src/tools/validation_tools.rs"
          ],
          "description": "任务依赖和循环检测模块",
          "importance": 6.0,
          "key_functions": [
            "依赖分析",
            "循环检测",
            "任务完整性检查"
          ],
          "name": "依赖分析模块"
        }
      ]
    },
    {
      "code_paths": [
        "crates/cowork-core/src/instructions/coding.rs",
        "crates/cowork-core/src/instructions/code_patch.rs",
        "crates/cowork-core/src/instructions/check.rs"
      ],
      "complexity": 9.0,
      "description": "负责代码实现、质量检查、代码修改和基础验证",
      "domain_type": "核心业务域",
      "importance": 9.0,
      "name": "编码实施域",
      "sub_modules": [
        {
          "code_paths": [
            "crates/cowork-core/src/instructions/coding.rs",
            "crates/cowork-core/src/tools/file_tools.rs"
          ],
          "description": "代码实现和质量检查模块",
          "importance": 9.0,
          "key_functions": [
            "代码生成",
            "质量验证",
            "简单代码原则",
            "任务完成检查"
          ],
          "name": "编码代理模块"
        },
        {
          "code_paths": [
            "crates/cowork-core/src/instructions/code_patch.rs",
            "crates/cowork-core/src/tools/modify_tools.rs"
          ],
          "description": "增量代码修改模块",
          "importance": 8.0,
          "key_functions": [
            "代码修改",
            "变更实施",
            "最佳实践指导"
          ],
          "name": "代码补丁模块"
        },
        {
          "code_paths": [
            "crates/cowork-core/src/instructions/check.rs",
            "crates/cowork-core/src/tools/validation_tools.rs"
          ],
          "description": "最小质量验证模块",
          "importance": 7.0,
          "key_functions": [
            "结构完整性检查",
            "基础验证",
            "问题检测"
          ],
          "name": "质量检查模块"
        }
      ]
    },
    {
      "code_paths": [
        "crates/cowork-core/src/instructions/delivery.rs",
        "crates/cowork-core/src/instructions/modify_delivery.rs",
        "crates/cowork-core/src/tools/artifact_tools.rs"
      ],
      "complexity": 6.0,
      "description": "负责项目交付验证、报告生成、文档管理和变更报告",
      "domain_type": "核心业务域",
      "importance": 8.0,
      "name": "交付管理域",
      "sub_modules": [
        {
          "code_paths": [
            "crates/cowork-core/src/instructions/delivery.rs",
            "crates/cowork-core/src/tools/artifact_tools.rs"
          ],
          "description": "项目交付验证和报告生成模块",
          "importance": 8.0,
          "key_functions": [
            "交付验证",
            "报告生成",
            "文档保存",
            "完整性检查"
          ],
          "name": "交付代理模块"
        },
        {
          "code_paths": [
            "crates/cowork-core/src/instructions/modify_delivery.rs",
            "crates/cowork-core/src/tools/artifact_tools.rs"
          ],
          "description": "变更报告生成模块",
          "importance": 7.0,
          "key_functions": [
            "变更分析",
            "报告生成",
            "修改文档化",
            "会话跟踪"
          ],
          "name": "修改交付模块"
        }
      ]
    },
    {
      "code_paths": [
        "crates/cowork-core/src/pipeline/mod.rs",
        "crates/cowork-core/src/tools/goto_stage_tool.rs"
      ],
      "complexity": 8.0,
      "description": "负责工作流编排、管道管理、阶段控制和流程恢复",
      "domain_type": "编排协调域",
      "importance": 8.0,
      "name": "编排管理域",
      "sub_modules": [
        {
          "code_paths": [
            "crates/cowork-core/src/pipeline/mod.rs"
          ],
          "description": "工作流管道编排模块",
          "importance": 9.0,
          "key_functions": [
            "管道创建",
            "阶段检测",
            "条件工作流组装",
            "流程管理"
          ],
          "name": "管道编排器"
        },
        {
          "code_paths": [
            "crates/cowork-core/src/tools/goto_stage_tool.rs"
          ],
          "description": "阶段跳转和流程恢复模块",
          "importance": 7.0,
          "key_functions": [
            "阶段验证",
            "会话管理",
            "重启信息更新",
            "元数据操作"
          ],
          "name": "阶段控制模块"
        }
      ]
    },
    {
      "code_paths": [
        "crates/cowork-core/src/agents/mod.rs",
        "crates/cowork-core/src/agents/hitl.rs",
        "crates/cowork-core/src/instructions/mod.rs"
      ],
      "complexity": 7.0,
      "description": "负责智能代理的创建、管理、错误恢复和指令模板管理",
      "domain_type": "代理管理域",
      "importance": 8.0,
      "name": "代理管理域",
      "sub_modules": [
        {
          "code_paths": [
            "crates/cowork-core/src/agents/mod.rs"
          ],
          "description": "代理工厂和生命周期管理模块",
          "importance": 8.0,
          "key_functions": [
            "代理创建",
            "流程编排",
            "生命周期管理",
            "错误处理"
          ],
          "name": "代理工厂模块"
        },
        {
          "code_paths": [
            "crates/cowork-core/src/agents/hitl.rs"
          ],
          "description": "人类参与循环错误恢复模块",
          "importance": 7.0,
          "key_functions": [
            "错误拦截",
            "用户干预",
            "恢复选项",
            "流处理包装"
          ],
          "name": "HITL恢复模块"
        },
        {
          "code_paths": [
            "crates/cowork-core/src/instructions/mod.rs"
          ],
          "description": "指令模板聚合和管理模块",
          "importance": 6.0,
          "key_functions": [
            "指令聚合",
            "统一访问",
            "模块组织"
          ],
          "name": "指令管理模块"
        }
      ]
    },
    {
      "code_paths": [
        "crates/cowork-core/src/tools/mod.rs",
        "crates/cowork-core/src/tools/file_tools.rs",
        "crates/cowork-core/src/tools/control_tools.rs"
      ],
      "complexity": 6.0,
      "description": "提供各类功能工具，包括文件操作、用户交互、数据验证和内容管理",
      "domain_type": "工具支持域",
      "importance": 7.0,
      "name": "工具支持域",
      "sub_modules": [
        {
          "code_paths": [
            "crates/cowork-core/src/tools/file_tools.rs"
          ],
          "description": "安全文件操作工具模块",
          "importance": 8.0,
          "key_functions": [
            "文件列表",
            "文件读写",
            "命令执行",
            "路径安全验证"
          ],
          "name": "文件工具模块"
        },
        {
          "code_paths": [
            "crates/cowork-core/src/tools/control_tools.rs",
            "crates/cowork-core/src/tools/hitl_tools.rs"
          ],
          "description": "用户交互和控制工具模块",
          "importance": 7.0,
          "key_functions": [
            "反馈收集",
            "用户查询",
            "内容评审",
            "交互处理"
          ],
          "name": "控制工具模块"
        },
        {
          "code_paths": [
            "crates/cowork-core/src/tools/validation_tools.rs",
            "crates/cowork-core/src/tools/data_tools.rs"
          ],
          "description": "数据验证和管理工具模块",
          "importance": 7.0,
          "key_functions": [
            "数据验证",
            "格式检查",
            "覆盖率分析",
            "依赖检测"
          ],
          "name": "验证工具模块"
        }
      ]
    },
    {
      "code_paths": [
        "crates/cowork-core/src/storage/mod.rs",
        "crates/cowork-core/src/data/models.rs",
        "crates/cowork-core/src/data/mod.rs"
      ],
      "complexity": 7.0,
      "description": "负责数据持久化、会话管理、模型定义和存储架构",
      "domain_type": "数据存储域",
      "importance": 7.0,
      "name": "数据存储域",
      "sub_modules": [
        {
          "code_paths": [
            "crates/cowork-core/src/storage/mod.rs"
          ],
          "description": "会话范围文件存储模块",
          "importance": 8.0,
          "key_functions": [
            "CRUD操作",
            "会话管理",
            "文件存储",
            "事务完整性"
          ],
          "name": "存储引擎模块"
        },
        {
          "code_paths": [
            "crates/cowork-core/src/data/models.rs",
            "crates/cowork-core/src/data/mod.rs"
          ],
          "description": "数据模型定义和管理模块",
          "importance": 7.0,
          "key_functions": [
            "模型定义",
            "序列化",
            "时间戳跟踪",
            "架构支持"
          ],
          "name": "数据模型模块"
        }
      ]
    },
    {
      "code_paths": [
        "crates/cowork-core/src/llm/mod.rs",
        "crates/cowork-core/src/llm/config.rs",
        "crates/cowork-core/src/llm/rate_limiter.rs"
      ],
      "complexity": 5.0,
      "description": "负责大语言模型集成、配置管理和API调用优化",
      "domain_type": "基础设施域",
      "importance": 6.0,
      "name": "LLM集成域",
      "sub_modules": [
        {
          "code_paths": [
            "crates/cowork-core/src/llm/config.rs"
          ],
          "description": "LLM配置管理模块",
          "importance": 6.0,
          "key_functions": [
            "配置加载",
            "环境变量管理",
            "客户端创建",
            "错误处理"
          ],
          "name": "配置管理模块"
        },
        {
          "code_paths": [
            "crates/cowork-core/src/llm/rate_limiter.rs"
          ],
          "description": "API速率限制中间件模块",
          "importance": 5.0,
          "key_functions": [
            "速率限制",
            "延迟控制",
            "API调用包装",
            "流量管理"
          ],
          "name": "速率限制模块"
        }
      ]
    },
    {
      "code_paths": [
        "crates/cowork-cli/src/main.rs"
      ],
      "complexity": 4.0,
      "description": "提供命令行界面，作为系统的主要用户交互入口",
      "domain_type": "应用接口域",
      "importance": 6.0,
      "name": "CLI接口域",
      "sub_modules": [
        {
          "code_paths": [
            "crates/cowork-cli/src/main.rs"
          ],
          "description": "命令行界面主入口模块",
          "importance": 6.0,
          "key_functions": [
            "命令解析",
            "会话管理",
            "管道协调",
            "状态跟踪"
          ],
          "name": "CLI主模块"
        }
      ]
    }
  ],
  "domain_relations": [
    {
      "description": "CLI接口域通过调用编排管理域来启动和管理整个软件开发工作流",
      "from_domain": "CLI接口域",
      "relation_type": "服务调用",
      "strength": 9.0,
      "to_domain": "编排管理域"
    },
    {
      "description": "编排管理域依赖代理管理域来创建和管理各个开发阶段的智能代理实例",
      "from_domain": "编排管理域",
      "relation_type": "服务调用",
      "strength": 9.0,
      "to_domain": "代理管理域"
    },
    {
      "description": "代理管理域通过指令模板调用各个业务域的功能模块执行具体开发任务",
      "from_domain": "代理管理域",
      "relation_type": "服务调用",
      "strength": 8.0,
      "to_domain": "需求管理域"
    },
    {
      "description": "需求管理域为设计管理域提供结构化的产品需求作为设计输入",
      "from_domain": "需求管理域",
      "relation_type": "数据依赖",
      "strength": 9.0,
      "to_domain": "设计管理域"
    },
    {
      "description": "设计管理域为规划管理域提供系统架构设计作为任务规划的基础",
      "from_domain": "设计管理域",
      "relation_type": "数据依赖",
      "strength": 9.0,
      "to_domain": "规划管理域"
    },
    {
      "description": "规划管理域为编码实施域提供详细的实现任务和依赖关系",
      "from_domain": "规划管理域",
      "relation_type": "数据依赖",
      "strength": 9.0,
      "to_domain": "编码实施域"
    },
    {
      "description": "编码实施域为交付管理域提供实现的代码文件和相关元数据",
      "from_domain": "编码实施域",
      "relation_type": "数据依赖",
      "strength": 9.0,
      "to_domain": "交付管理域"
    },
    {
      "description": "所有业务域都依赖工具支持域提供各种功能工具来执行具体操作",
      "from_domain": "需求管理域",
      "relation_type": "工具依赖",
      "strength": 8.0,
      "to_domain": "工具支持域"
    },
    {
      "description": "工具支持域依赖数据存储域进行数据持久化和会话状态管理",
      "from_domain": "工具支持域",
      "relation_type": "数据依赖",
      "strength": 7.0,
      "to_domain": "数据存储域"
    },
    {
      "description": "代理管理域依赖LLM集成域提供智能决策和代码生成能力",
      "from_domain": "代理管理域",
      "relation_type": "服务调用",
      "strength": 7.0,
      "to_domain": "LLM集成域"
    },
    {
      "description": "变更管理流程涉及需求、设计、规划和编码多个域的协同工作",
      "from_domain": "变更管理域",
      "relation_type": "流程协作",
      "strength": 8.0,
      "to_domain": "需求管理域"
    },
    {
      "description": "流程恢复功能需要编排管理域与代理管理域的紧密协作",
      "from_domain": "编排管理域",
      "relation_type": "流程协作",
      "strength": 8.0,
      "to_domain": "代理管理域"
    }
  ]
}
```

### Workflow Research Report
Contains static analysis results of the codebase and business process analysis.

```json
{
  "main_workflow": {
    "description": "完整的软件开发生命周期流程，从创意采集到项目交付的全自动化处理。该流程采用Actor-Critic模式结合Human-in-the-Loop机制，确保每个阶段的质量控制和用户参与。",
    "flowchart_mermaid": "graph TD\n    A[用户创意输入] --> B[创意代理处理]\n    B --> C[PRD生成和验证]\n    C --> D[系统架构设计]\n    D --> E[实现任务规划]\n    E --> F[代码实现和质量检查]\n    F --> G[项目交付验证]\n    G --> H[交付报告生成]\n    \n    C --> C1[用户评审]\n    D --> D1[架构评审]\n    E --> E1[计划评审]\n    \n    C1 --> C\n    D1 --> D\n    E1 --> E\n    \n    style A fill:#e1f5fe\n    style H fill:#c8e6c9",
    "name": "新项目开发流程"
  },
  "other_important_workflows": [
    {
      "description": "处理现有项目的增量修改请求，包括变更分析、影响评估、代码修改和变更报告生成。支持精确的变更范围控制和风险评估。",
      "flowchart_mermaid": "graph TD\n    A[变更请求输入] --> B[变更分析代理]\n    B --> C[范围确定和风险评估]\n    C --> D[代码补丁实施]\n    D --> E[修改验证]\n    E --> F[变更报告生成]\n    \n    C --> C1[影响分析]\n    C1 --> C2[接受标准提取]\n    C2 --> C\n    \n    style A fill:#fff3e0\n    style F fill:#ffecb3",
      "name": "项目修改流程"
    },
    {
      "description": "工作流恢复和阶段跳转机制，支持从特定阶段重新开始工作流，用于错误恢复和流程调整。提供灵活的流程控制能力。",
      "flowchart_mermaid": "graph TD\n    A[流程中断或错误] --> B[阶段跳转工具]\n    B --> C[阶段验证]\n    C --> D[会话元数据加载]\n    D --> E[工作流管道重新组装]\n    E --> F[代理实例重新创建]\n    F --> G[从指定阶段继续]\n    \n    C --> C1[阶段有效性检查]\n    D --> D1[状态恢复]\n    \n    style A fill:#fce4ec\n    style G fill:#e8f5e8",
      "name": "流程恢复流程"
    },
    {
      "description": "关键决策点的用户参与流程，确保重要决策的质量和准确性。提供编辑、反馈和审批等多种交互方式。",
      "flowchart_mermaid": "graph TD\n    A[生成内容] --> B[用户审核触发]\n    B --> C{审核选项}\n    C --> D[编辑模式]\n    C --> E[通过模式]\n    C --> F[反馈模式]\n    \n    D --> D1[文本编辑器打开]\n    D1 --> D2[用户编辑]\n    D2 --> G[内容更新]\n    \n    E --> G\n    \n    F --> F1[反馈输入]\n    F1 --> G\n    \n    G --> H[继续流程]\n    \n    style B fill:#fff9c4\n    style H fill:#e8f5e8",
      "name": "Human-in-the-Loop审核流程"
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
      "description": "Cowork Forge CLI entry point - AI-powered software development system orchestrator",
      "file_path": "crates/cowork-cli/src/main.rs",
      "functions": [
        "main",
        "load_config",
        "cmd_new",
        "cmd_resume",
        "cmd_revert",
        "cmd_modify",
        "collect_project_file_fingerprints",
        "diff_project_files",
        "should_ignore_project_path",
        "execute_pipeline",
        "cmd_status",
        "cmd_init"
      ],
      "importance_score": 1.0,
      "interfaces": [
        "Cli",
        "Commands",
        "execute_pipeline"
      ],
      "name": "main.rs",
      "source_summary": "// Cowork Forge - CLI Entry Point\n\nuse anyhow::Result;\nuse clap::{Parser, Subcommand};\nuse cowork_core::llm::ModelConfig;\nuse cowork_core::pipeline::{create_cowork_pipeline, create_partial_pipeline, create_resume_pipeline, create_modify_pipeline};\nuse cowork_core::storage::is_project_initialized;\nuse std::path::Path;\nuse std::sync::Arc;\nuse std::collections::{HashMap, HashSet};\nuse walkdir::WalkDir;\nuse tracing::{info, error};\nuse adk_runner::{Runner, RunnerConfig};\nuse adk_session::InMemorySessionService;\nuse adk_core::Content;\nuse futures::StreamExt;\n\n#[derive(Parser)]\n#[command(name = \"cowork\")]\n#[command(about = \"AI-powered software development system\", long_about = None)]\nstruct Cli {\n    #[command(subcommand)]\n    command: Commands,\n\n    /// Path to config file (default: config.toml)\n    #[arg(short, long, global = true)]\n    config: Option<String>,\n\n    /// Enable verbose logging\n    #[arg(short, long, global = true)]\n    verbose: bool,\n\n    /// Enable LLM streaming output (shows AI thinking process in real-time)\n    #[arg(short, long, global = true)]\n    stream: bool,\n}\n\n#[derive(Subcommand)]\nenum Commands {\n    /// Start a new project\n    New {\n        /// Project idea/description\n        idea: String,\n    },\n\n    /// Resume an existing project\n    Resume {\n        /// Resume from a specific session ID (optional).\n        /// If omitted, defaults to latest successful session; if none, tries latest in-progress session.\n        #[arg(short, long)]\n        base: Option<String>,\n    },\n\n    /// Revert project and restart from a specific stage\n    Revert {\n        /// Stage to restart from (prd, design, plan, coding, check, delivery)\n        #[arg(short, long)]\n        from: String,\n    },\n\n    /// Modify existing project with incremental changes\n    Modify {\n        /// Change idea/description\n        idea: String,\n        /// Base session ID (defaults to latest successful session)\n        #[arg(short, long)]\n        base: Option<String>,\n    },\n\n    /// Show project status\n    Status {\n        /// Show all sessions\n        #[arg(short, long)]\n        sessions: bool,\n    },\n\n    /// Initialize config file\n    Init,\n}\n\n#[tokio::main]\nasync fn main() -> Result<()> {\n    let cli = Cli::parse();\n\n    // Setup logging - output to stderr, not stdout\n    let log_filter = if cli.verbose {\n        // Verbose mode: show all logs including adk internals\n        \"debug\".to_string()\n    } else {\n        // Normal mode: filter out adk verbose logs to avoid clutter\n        \"info,adk_agent=warn,adk_core=warn,adk_runner=warn\".to_string()\n    };\n    \n    tracing_subscriber::fmt()\n        .with_writer(std::io::stderr) // Force logs to stderr\n        .with_env_filter(log_filter)\n        .init();\n\n    // Load configuration\n    let config_path = cli.config.unwrap_or_else(|| \"config.toml\".to_string());\n    let config = load_config(&config_path)?;\n\n    // Execute command\n    let enable_stream = cli.stream;\n    match cli.command {\n        Commands::New { idea } => cmd_new(idea, &config, enable_stream).await?,\n        Commands::Resume { base } => cmd_resume(base, &config, enable_stream).await?,\n        Commands::Revert { from } => cmd_revert(&from, &config, enable_stream).await?,\n        Commands::Modify { idea, base } => cmd_modify(&idea, base, &config, enable_stream).await?,\n        Commands::Status { sessions } => cmd_status(sessions).await?,\n        Commands::Init => cmd_init()?,\n    }\n\n    Ok(())\n}\n\n/// Load configuration from file or environment\nfn load_config(path: &str) -> Result<ModelConfig> {\n    if Path::new(path).exists() {\n        info!(\"Loading configuration from {}\", path);\n        ModelConfig::from_file(path)\n    } else {\n        info!(\"Config file not found, attempting to load from environment variables\");\n        ModelConfig::from_env()\n    }\n}\n\n/// Start a new project\nasync fn cmd_new(idea: String, config: &ModelConfig, enable_stream: bool) -> Result<()> {\n    use cowork_core::storage::*;\n    use cowork_core::data::*;\n    \n    info!(\"Starting new project with idea: {}\", idea);\n\n    if is_project_initialized() {\n        error!(\".cowork directory already initialized. Use 'resume' or 'modify' instead.\");\n        anyhow::bail!(\"Project already initialized\");\n    }\n\n    // Initialize project index\n    let project_name = idea.split_whitespace().take(3).collect::<Vec<_>>().join(\"_\");\n    let mut index = init_project_index(project_name)?;\n    \n    // Generate session ID\n    let session_id = format!(\"session-{}\", chrono::Utc::now().timestamp());\n    \n    // Create session record\n    let session_record = SessionRecord {\n        session_id: session_id.clone(),\n        session_type: SessionType::New,\n        created_at: chrono::Utc::now(),\n        completed_at: None,\n        status: SessionStatus::InProgress,\n        base_session_id: None,\n        input_description: idea.clone(),\n        change_request_id: None,\n    };\n    \n    index.add_session(session_record);\n    save_project_index(&index)?;\n    \n    // Save session input\n    let session_input = SessionInput {\n        session_id: session_id.clone(),\n        session_type: SessionType::New,\n        description: idea.clone(),\n        base_session_id: None,\n        created_at: chrono::Utc::now(),\n    };\n    save_session_input(&session_id, &session_input)?;\n\n    // Create pipeline\n    let pipeline = create_cowork_pipeline(config, &session_id)?;\n\n    // Execute pipeline with idea as input\n    println!(\"✨ Creating new project...\");\n    println!(\"Session ID: {}\", session_id);\n    println!(\"Idea: {}\", idea);\n    println!();\n\n    let result = execute_pipeline(pipeline, &idea, enable_stream).await;\n\n    // Mark session status based on result\n    match result {\n        Ok(_) => {\n            mark_session_completed(&session_id)?;\n            println!(\"\\n✅ Project creation complete!\");\n            println!(\"Session ID: {}\", session_id);\n            println!(\"Check .cowork/sessions/{}/artifacts/ for outputs\", session_id);\n        }\n        Err(e) => {\n            mark_session_failed(&session_id)?;\n            return Err(e);\n        }\n    }\n\n    Ok(())\n}\n\n/// Resume an existing project\nasync fn cmd_resume(base: Option<String>, config: &ModelConfig, enable_stream: bool) -> Result<()> {\n    use cowork_core::storage::*;\n    \n    info!(\"Resuming project\");\n\n    if !is_project_initialized() {\n        error!(\".cowork directory not found. Use 'new' to create a project.\");\n        anyhow::bail!(\"No project found\");\n    }\n\n    // Determine base session\n    let base_session_id = if let Some(base_id) = base {\n        base_id\n    } else if let Some(latest_ok) = get_latest_successful_session()? {\n        latest_ok\n    } else {\n        // Fallback: try latest in-progress session (useful when previous run was interrupted)\n        let index = load_project_index()?;\n        let last_in_progress = index\n            .sessions\n            .iter()\n            .rev()\n            .find(|s| s.status == cowork_core::data::SessionStatus::InProgress)\n            .map(|s| s.session_id.clone());\n\n        if let Some(sid) = last_in_progress {\n            sid\n        } else {\n            error!(\"No successful session found. Cannot resume.\");\n            anyhow::bail!(\"No session to resume from\");\n        }\n    };\n\n    info!(\"Resuming from session: {}\", base_session_id);\n\n    // Create new session for resume\n    let session_id = format!(\"session-{}\", chrono::Utc::now().timestamp());\n    \n    let mut index = load_project_index()?;\n    let session_record = cowork_core::data::SessionRecord {\n        session_id: session_id.clone(),\n        session_type: cowork_core::data::SessionType::New, // Resume is treated as continuation\n        created_at: chrono::Utc::now(),\n        completed_at: None,\n        status: cowork_core::data::SessionStatus::InProgress,\n        base_session_id: Some(base_session_id.clone()),\n        input_description: \"Resume from last checkpoint\".to_string(),\n        change_request_id: None,\n    };\n    index.add_session(session_record);\n    save_project_index(&index)?;\n    \n    // Save session input\n    let session_input = SessionInput {\n        session_id: session_id.clone(),\n        session_type: cowork_core::data::SessionType::New, // Resume is treated as continuation\n        description: \"Resume from last checkpoint\".to_string(),\n        base_session_id: Some(base_session_id.clone()),\n        created_at: chrono::Utc::now(),\n    };\n    save_session_input(&session_id, &session_input)?;\n\n    // Bootstrap session state from base session\n    init_session_from_base(&session_id, &base_session_id)?;\n\n    // Create resume pipeline\n    let pipeline = create_resume_pipeline(config, &session_id, &base_session_id)?;\n\n    // Execute pipeline\n    println!(\"🔄 Resuming project...\");\n    println!(\"Base session: {}\", base_session_id);\n    println!(\"New session: {}\", session_id);\n    println!();\n\n    let result = execute_pipeline(pipeline, \"Resume from last checkpoint\", enable_stream).await;\n\n    match result {\n        Ok(_) => {\n            mark_session_completed(&session_id)?;\n            println!(\"\\n✅ Project resume complete!\");\n        }\n        Err(e) => {\n            mark_session_failed(&session_id)?;\n            return Err(e);\n        }\n    }\n\n    Ok(())\n}\n\n/// Revert project and restart from a specific stage\nasync fn cmd_revert(from_stage: &str, config: &ModelConfig, enable_stream: bool) -> Result<()> {\n    use cowork_core::storage::*;    info!(\"Reverting project from stage: {}\", from_stage);\n\n    if !is_project_initialized() {\n        error!(\".cowork directory not found. Use 'new' to create a project.\");\n        anyhow::bail!(\"No project found\");\n    }\n\n    let latest_session = get_latest_successful_session()?;\n    if latest_session.is_none() {\n        error!(\"No successful session found. Cannot revert.\");\n        anyhow::bail!(\"No session to revert from\");\n    }\n    \n    let base_session_id = latest_session.unwrap();\n\n    // Support `--from auto`: use the latest session meta's requested restart stage (if any)\n    let resolved_stage = if from_stage == \"auto\" {\n        let index = load_project_index()?;\n        let last_session_id = index\n            .sessions\n            .last()\n            .map(|s| s.session_id.clone())\n            .ok_or_else(|| anyhow::anyhow!(\"No session records found\"))?;\n\n        if let Some(meta) = load_session_meta(&last_session_id)? {\n            if let Some(stage) = meta.current_stage {\n                match stage {\n                    cowork_core::data::Stage::Prd => \"prd\",\n                    cowork_core::data::Stage::Design => \"design\",\n                    cowork_core::data::Stage::Plan => \"plan\",\n                    cowork_core::data::Stage::Coding => \"coding\",\n                    cowork_core::data::Stage::Check => \"check\",\n                    cowork_core::data::Stage::Delivery => \"delivery\",\n                    cowork_core::data::Stage::Idea => \"prd\",\n                }\n            } else {\n                \"prd\"\n            }\n        } else {\n            \"prd\"\n        }\n    } else {\n        from_stage\n    };\n\n    // Create new session for revert\n    let session_id = format!(\"session-{}\", chrono::Utc::now().timestamp());\n    \n    let mut index = load_project_index()?;\n    let session_record = cowork_core::data::SessionRecord {\n        session_id: session_id.clone(),\n        session_type: cowork_core::data::SessionType::Revert,\n        created_at: chrono::Utc::now(),\n        completed_at: None,\n        status: cowork_core::data::SessionStatus::InProgress,\n        base_session_id: Some(base_session_id.clone()),\n        input_description: format!(\"Revert from {} stage\", resolved_stage),\n        change_request_id: None,\n    };\n    index.add_session(session_record);\n    save_project_index(&index)?;\n    \n    // Save session input\n    let session_input = SessionInput {\n        session_id: session_id.clone(),\n        session_type: cowork_core::data::SessionType::Revert,\n        description: format!(\"Revert from {} stage\", resolved_stage),\n        base_session_id: Some(base_session_id.clone()),\n        created_at: chrono::Utc::now(),\n    };\n    save_session_input(&session_id, &session_input)?;\n\n    // Bootstrap session state from base session\n    init_session_from_base(&session_id, &base_session_id)?;\n\n    // Create partial pipeline\n    let pipeline = create_partial_pipeline(config, &session_id, &base_session_id, resolved_stage)?;\n\n    // Execute pipeline\n    println!(\"🔧 Reverting project from {} stage...\", resolved_stage);\n    println!(\"Base session: {}\", base_session_id);\n    println!(\"New session: {}\", session_id);\n    println!();\n\n    let result = execute_pipeline(pipeline, &format!(\"Revert from {} stage\", resolved_stage), enable_stream).await;\n\n    match result {\n        Ok(_) => {\n            mark_session_completed(&session_id)?;\n            println!(\"\\n✅ Revert complete!\");\n        }\n        Err(e) => {\n            mark_session_failed(&session_id)?;\n            return Err(e);\n        }\n    }\n\n    Ok(())\n}\n\nfn should_ignore_project_path(path: &str) -> bool {\n    // Ignore cowork internal state and common build artifacts\n    let ignore_patterns = [\n        \"./.cowork/\",\n        \"./target/\",\n        \"./node_modules/\",\n        \"./.git/\",\n        \"./dist/\",\n        \"./build/\",\n        \"./.vscode/\",\n        \"./.idea/\",\n    ];\n    ignore_patterns.iter().any(|p| path.contains(p))\n}\n\nfn collect_project_file_fingerprints() -> Result<HashMap<String, (u64, u64)>> {\n    // path -> (len, mtime_secs)\n    let mut map = HashMap::new();\n\n    for entry in WalkDir::new(\".\").follow_links(false) {\n        let entry = entry?;\n        if !entry.file_type().is_file() {\n            continue;\n        }\n        let p = entry.path();\n        let rel = p.strip_prefix(\".\").unwrap_or(p).to_string_lossy();\n        let rel = format!(\"./{}\", rel.trim_start_matches(\"/\"));\n\n        if should_ignore_project_path(&rel) {\n            continue;\n        }\n\n        let md = entry.metadata()?;\n        let len = md.len();\n        let mtime = md\n            .modified()\n            .ok()\n            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())\n            .map(|d| d.as_secs())\n            .unwrap_or(0);\n\n        map.insert(rel, (len, mtime));\n    }\n\n    Ok(map)\n}\n\nfn diff_project_files(\n    before: &HashMap<String, (u64, u64)>,\n    after: &HashMap<String, (u64, u64)>,\n) -> (Vec<String>, Vec<String>, Vec<String>) {\n    let before_keys: HashSet<&String> = before.keys().collect();\n    let after_keys: HashSet<&String> = after.keys().collect();\n\n    let mut added = Vec::new();\n    let mut deleted = Vec::new();\n    let mut modified = Vec::new();\n\n    for k in after_keys.difference(&before_keys) {\n        added.push((**k).clone());\n    }\n\n    for k in before_keys.difference(&after_keys) {\n        deleted.push((**k).clone());\n    }\n\n    for k in before_keys.intersection(&after_keys) {\n        let b = before.get(*k);\n        let a = after.get(*k);\n        if b != a {\n            modified.push((**k).clone());\n        }\n    }\n\n    added.sort();\n    deleted.sort();\n    modified.sort();\n\n    (added, modified, deleted)\n}\n\n/// Modify existing project with incremental changes\nasync fn cmd_modify(idea: &str, base: Option<String>, config: &ModelConfig, enable_stream: bool) -> Result<()> {\n    use cowork_core::storage::*;\n    use cowork_core::data::*;\n    \n    info!(\"Modifying project with idea: {}\", idea);\n\n    if !is_project_initialized() {\n        error!(\".cowork directory not found. Use 'new' to create a project.\");\n        anyhow::bail!(\"No project found\");\n    }\n\n    // Determine base session\n    let base_session_id = if let Some(base_id) = base {\n        base_id\n    } else {\n        get_latest_successful_session()?\n            .ok_or_else(|| anyhow::anyhow!(\"No successful session found. Cannot modify without a base.\"))?\n    };\n    \n    info!(\"Using base session: {}\", base_session_id);\n\n    // Create new session for modify\n    let session_id = format!(\"session-{}\", chrono::Utc::now().timestamp());\n    \n    // Create change request\n    let change_request = ChangeRequest::new(\n        session_id.clone(),\n        idea.to_string(),\n        base_session_id.clone(),\n    );\n    let change_request_id = change_request.id.clone();\n    save_change_request(&session_id, &change_request)?;\n    \n    // Create session record\n    let mut index = load_project_index()?;\n    let session_record = SessionRecord {\n        session_id: session_id.clone(),\n        session_type: SessionType::Modify,\n        created_at: chrono::Utc::now(),\n        completed_at: None,\n        status: SessionStatus::InProgress,\n        base_session_id: Some(base_session_id.clone()),\n        input_description: idea.to_string(),\n        change_request_id: Some(change_request_id.clone()),\n    };\n    index.add_session(session_record);\n    save_project_index(&index)?;\n    \n    // Save session input\n    let session_input = SessionInput {\n        session_id: session_id.clone(),\n        session_type: SessionType::Modify,\n        description: idea.to_string(),\n        base_session_id: Some(base_session_id.clone()),\n        created_at: chrono::Utc::now(),\n    };\n    save_session_input(&session_id, &session_input)?;\n\n    // Bootstrap session state from base session\n    init_session_from_base(&session_id, &base_session_id)?;\n\n    // Create modify pipeline (incremental change pipeline)\n    let pipeline = create_modify_pipeline(config, &session_id, &base_session_id)?;\n\n    // Snapshot project files before modification (for patch metadata)\n    let before_files = collect_project_file_fingerprints()?;\n\n    // Execute pipeline\n    println!(\"🔄 Applying incremental changes...\");\n    println!(\"Change: {}\", idea);\n    println!(\"Base session: {}\", base_session_id);\n    println!(\"New session: {}\", session_id);\n    println!();\n\n    let result = execute_pipeline(pipeline, idea, enable_stream).await;\n\n    match result {\n        Ok(_) => {\n            // Snapshot after modification and persist patch metadata\n            let after_files = collect_project_file_fingerprints()?;\n            let (added_files, modified_files, deleted_files) = diff_project_files(&before_files, &after_files);\n\n            let mut patch = PatchMetadata::new(session_id.clone(), base_session_id.clone());\n            patch.added_files = added_files;\n            patch.modified_files = modified_files;\n            patch.deleted_files = deleted_files;\n            save_patch_metadata(&session_id, &patch)?;\n\n            mark_session_completed(&session_id)?;\n            println!(\"\\n✅ Modification complete!\");\n            println!(\"Session ID: {}\", session_id);\n            println!(\"Patch metadata: .cowork/sessions/{}/patch/metadata.json\", session_id);\n        }\n        Err(e) => {\n            mark_session_failed(&session_id)?;\n            return Err(e);\n        }\n    }\n\n    Ok(())\n}\n\n/// Execute a pipeline with given input\nasync fn execute_pipeline(pipeline: Arc<dyn adk_core::Agent>, input: &str, enable_stream: bool) -> Result<()> {\n    use adk_core::RunConfig;\n    use adk_session::{CreateRequest, SessionService};\n    use std::collections::HashMap;\n\n    // Create session service\n    let session_service = Arc::new(InMemorySessionService::new());\n\n    // Create session FIRST\n    let user_id = \"cowork-user\".to_string();\n    let app_name = \"cowork-forge\".to_string();\n    \n    let session = session_service\n        .create(CreateRequest {\n            app_name: app_name.clone(),\n            user_id: user_id.clone(),\n            session_id: None, // Auto-generate session ID\n            state: HashMap::new(),\n        })\n        .await\n        .map_err(|e| anyhow::anyhow!(\"Failed to create session: {}\", e))?;\n    \n    let session_id = session.id().to_string();\n\n    // Create runner with run config\n    let runner = Runner::new(RunnerConfig {\n        app_name,\n        agent: pipeline,\n        session_service,\n        artifact_service: None,\n        memory_service: None,\n        run_config: Some(RunConfig::default()),\n    })?;\n\n    // Execute\n    let content = Content::new(\"user\").with_text(input);\n\n    let mut event_stream = runner.run(user_id, session_id, content).await?;\n\n    // Simple phase indicator - show when we start processing\n    println!(\"🚀 Starting execution...\\n\");\n    \n    // Optional: Show streaming mode status\n    if enable_stream {\n        println!(\"💬 Streaming mode enabled - showing LLM output in real-time\\n\");\n    }\n    \n    while let Some(event_result) = event_stream.next().await {\n        match event_result {\n            Ok(event) => {\n                // If streaming is enabled, show LLM output\n                if enable_stream {\n                    if let Some(llm_content) = &event.llm_response.content {\n                        use std::io::Write;\n                        let mut stdout = std::io::stdout();\n                        \n                        for part in &llm_content.parts {\n                            if let Some(text) = part.text() {\n                                // Filter out standalone newlines to reduce erratic line breaks\n                                if text != \"\\n\" {\n                                    print!(\"{}\", text);\n                                    stdout.flush().ok();\n                                }\n                            }\n                        }\n                    }\n                }\n                // Tools will always print their own progress (e.g., \"📝 Writing file: ...\")\n            }\n            Err(e) => {\n                error!(\"Error during pipeline execution: {}\", e);\n                anyhow::bail!(\"Pipeline execution failed: {}\", e);\n            }\n        }\n    }\n\n    println!(\"\\n✅ Pipeline complete!\");\n\n    Ok(())\n}\n\n/// Show project status\nasync fn cmd_status(show_sessions: bool) -> Result<()> {\n    use cowork_core::storage::*;\n    use cowork_core::data::*;\n\n    if !is_project_initialized() {\n        println!(\"❌ No project found in current directory\");\n        return Ok(());\n    }\n\n    let index = load_project_index()?;\n    \n    println!(\"📊 Project Status\\n\");\n    println!(\"Project: {}\", index.project_name);\n    println!(\"Created: {}\", index.created_at.format(\"%Y-%m-%d %H:%M:%S\"));\n    println!();\n\n    if show_sessions {\n        // Show all sessions\n        println!(\"Sessions ({} total):\", index.sessions.len());\n        println!(\"{:<20} {:<10} {:<15} {:<25}\", \"Session ID\", \"Type\", \"Status\", \"Created At\");\n        println!(\"{:-<70}\", \"\");\n        \n        for session in &index.sessions {\n            let session_type = match session.session_type {\n                SessionType::New => \"New\",\n                SessionType::Modify => \"Modify\",\n                SessionType::Revert => \"Revert\",\n            };\n            let status = match session.status {\n                SessionStatus::InProgress => \"In Progress\",\n                SessionStatus::Completed => \"Completed\",\n                SessionStatus::Failed => \"Failed\",\n            };\n            println!(\n                \"{:<20} {:<10} {:<15} {}\",\n                session.session_id,\n                session_type,\n                status,\n                session.created_at.format(\"%Y-%m-%d %H:%M:%S\")\n            );\n        }\n        println!();\n        \n        if let Some(latest) = &index.latest_successful_session {\n            println!(\"Latest successful: {}\", latest);\n        }\n    } else {\n        // Show summary of latest session\n        if let Some(latest_id) = &index.latest_successful_session {\n            println!(\"Latest successful session: {}\", latest_id);\n            \n            // Try to load artifacts from latest session\n            match load_requirements(latest_id) {\n                Ok(reqs) => {\n                    println!(\"Requirements: {} total\", reqs.requirements.len());\n                }\n                Err(_) => println!(\"Requirements: Not yet created\"),\n            }\n\n            match load_feature_list(latest_id) {\n                Ok(features) => {\n                    let completed = features.features.iter().filter(|f| matches!(f.status, FeatureStatus::Completed)).count();\n                    println!(\"Features: {}/{} completed\", completed, features.features.len());\n                }\n                Err(_) => println!(\"Features: Not yet created\"),\n            }\n\n            match load_design_spec(latest_id) {\n                Ok(design) => {\n                    println!(\"Components: {} defined\", design.architecture.components.len());\n                }\n                Err(_) => println!(\"Design: Not yet created\"),\n            }\n\n            match load_implementation_plan(latest_id) {\n                Ok(plan) => {\n                    let completed = plan.tasks.iter().filter(|t| matches!(t.status, TaskStatus::Completed)).count();\n                    println!(\"Tasks: {}/{} completed\", completed, plan.tasks.len());\n                }\n                Err(_) => println!(\"Implementation Plan: Not yet created\"),\n            }\n        } else {\n            println!(\"No successful sessions yet\");\n        }\n        \n        println!();\n        println!(\"Tip: Use 'cowork status --sessions' to see all sessions\");\n    }\n\n    Ok(())\n}\n\n/// Initialize configuration file\nfn cmd_init() -> Result<()> {\n    let config_path = \"config.toml\";\n\n    if Path::new(config_path).exists() {\n        error!(\"config.toml already exists\");\n        anyhow::bail!(\"Configuration file already exists\");\n    }\n\n    let default_config = r#\"[llm]\napi_base_url = \"http://localhost:8000/v1\"\napi_key = \"your-api-key-here\"\nmodel_name = \"gpt-4\"\n\"#;\n\n    std::fs::write(config_path, default_config)?;\n    println!(\"✅ Created config.toml\");\n    println!(\"\\nPlease edit config.toml and set your API credentials:\");\n    println!(\"  - api_base_url: Your OpenAI-compatible API endpoint\");\n    println!(\"  - api_key: Your API key\");\n    println!(\"  - model_name: Model to use (e.g., gpt-4, gpt-3.5-turbo)\");\n\n    Ok(())\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 55.0,
      "lines_of_code": 775,
      "number_of_classes": 2,
      "number_of_functions": 12
    },
    "dependencies": [
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 3,
        "name": "anyhow",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 4,
        "name": "clap",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 5,
        "name": "cowork_core",
        "path": "crates/cowork-core",
        "version": null
      },
      {
        "dependency_type": "standard",
        "is_external": false,
        "line_number": 7,
        "name": "std::path",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "standard",
        "is_external": false,
        "line_number": 8,
        "name": "std::sync",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "standard",
        "is_external": false,
        "line_number": 9,
        "name": "std::collections",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 10,
        "name": "walkdir",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 11,
        "name": "tracing",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 12,
        "name": "adk_runner",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 13,
        "name": "adk_session",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 14,
        "name": "adk_core",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 15,
        "name": "futures",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "This is the main entry point for the Cowork Forge CLI application, which serves as an AI-powered software development orchestrator. The component handles command-line interface parsing, session management, and coordinates the entire AI-driven development pipeline. It manages project lifecycle operations including creating new projects, resuming existing ones, reverting to specific stages, and applying incremental modifications. The system maintains session state, tracks project changes through file fingerprinting, and provides project status monitoring capabilities.",
    "interfaces": [
      {
        "description": "Main CLI argument parser structure",
        "interface_type": "struct",
        "name": "Cli",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Available CLI commands enumeration",
        "interface_type": "enum",
        "name": "Commands",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Executes the AI pipeline with streaming capability",
        "interface_type": "function",
        "name": "execute_pipeline",
        "parameters": [
          {
            "description": "The AI pipeline to execute",
            "is_optional": false,
            "name": "pipeline",
            "param_type": "Arc<dyn adk_core::Agent>"
          },
          {
            "description": "Input text for the pipeline",
            "is_optional": false,
            "name": "input",
            "param_type": "&str"
          },
          {
            "description": "Whether to enable LLM streaming output",
            "is_optional": false,
            "name": "enable_stream",
            "param_type": "bool"
          }
        ],
        "return_type": "Result<()>",
        "visibility": "private"
      }
    ],
    "responsibilities": [
      "Command-line interface parsing and command dispatching",
      "Project lifecycle management (new, resume, revert, modify)",
      "Session creation, tracking, and state management",
      "Pipeline execution orchestration with streaming support",
      "Project file change detection and patch management"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "config",
      "description": "LLM configuration management component for cowork-core system",
      "file_path": "crates/cowork-core/src/llm/config.rs",
      "functions": [
        "from_file",
        "from_env",
        "create_llm_client"
      ],
      "importance_score": 0.9,
      "interfaces": [
        "LlmConfig",
        "ModelConfig",
        "create_llm_client"
      ],
      "name": "config.rs",
      "source_summary": "// LLM configuration using adk-rust's OpenAI client\nuse anyhow::{Context, Result};\nuse serde::{Deserialize, Serialize};\nuse std::sync::Arc;\nuse adk_model::openai::{OpenAIClient, OpenAIConfig};\nuse adk_core::Llm;\n\n/// Configuration for LLM from config.toml\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct LlmConfig {\n    pub api_base_url: String,\n    pub api_key: String,\n    pub model_name: String,\n}\n\n/// Configuration for the entire model setup\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct ModelConfig {\n    pub llm: LlmConfig,\n}\n\nimpl ModelConfig {\n    /// Load from TOML file\n    pub fn from_file(path: &str) -> Result<Self> {\n        let content = std::fs::read_to_string(path)\n            .with_context(|| format!(\"Failed to read config file: {}\", path))?;\n        let config: Self = toml::from_str(&content)\n            .with_context(|| \"Failed to parse config.toml\")?;\n        Ok(config)\n    }\n\n    /// Load from environment variables (fallback)\n    pub fn from_env() -> Result<Self> {\n        Ok(Self {\n            llm: LlmConfig {\n                api_base_url: std::env::var(\"LLM_API_BASE_URL\")\n                    .with_context(|| \"LLM_API_BASE_URL not set\")?,\n                api_key: std::env::var(\"LLM_API_KEY\")\n                    .with_context(|| \"LLM_API_KEY not set\")?,\n                model_name: std::env::var(\"LLM_MODEL_NAME\")\n                    .with_context(|| \"LLM_MODEL_NAME not set\")?,\n            },\n        })\n    }\n}\n\n/// Create an LLM client using adk-rust's OpenAI client with custom base URL\n/// \n/// This uses the built-in OpenAIClient from adk-model and configures it\n/// to point to a custom OpenAI-compatible endpoint.\n/// \n/// **Rate Limiting**: Automatically wraps the client with a 2-second delay\n/// to comply with rate limits (<30 calls per minute).\npub fn create_llm_client(config: &LlmConfig) -> Result<Arc<dyn Llm>> {\n    use crate::llm::rate_limiter::RateLimitedLlm;\n\n    // Create OpenAI config with custom base URL using OpenAIConfig::compatible\n    let openai_config = OpenAIConfig::compatible(\n        &config.api_key,\n        &config.api_base_url,\n        &config.model_name,\n    );\n\n    // Create the OpenAI client\n    let client = OpenAIClient::new(openai_config)\n        .with_context(|| \"Failed to create OpenAI client\")?;\n\n    // Wrap with rate limiter (2-second delay for <30 calls/min)\n    let rate_limited_client = RateLimitedLlm::with_default_delay(Arc::new(client));\n\n    Ok(Arc::new(rate_limited_client))\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn test_config_parse() {\n        let toml_content = r#\"\n[llm]\napi_base_url = \"http://localhost:8000/v1\"\napi_key = \"test-key\"\nmodel_name = \"gpt-4\"\n        \"#;\n\n        let config: ModelConfig = toml::from_str(toml_content).unwrap();\n        assert_eq!(config.llm.api_base_url, \"http://localhost:8000/v1\");\n        assert_eq!(config.llm.api_key, \"test-key\");\n        assert_eq!(config.llm.model_name, \"gpt-4\");\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 4.0,
      "lines_of_code": 92,
      "number_of_classes": 2,
      "number_of_functions": 3
    },
    "dependencies": [
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 2,
        "name": "anyhow",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
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
        "dependency_type": "internal",
        "is_external": true,
        "line_number": 5,
        "name": "adk_model",
        "path": "adk_model::openai",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": true,
        "line_number": 6,
        "name": "adk_core",
        "path": "adk_core::Llm",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 34,
        "name": "crate::llm::rate_limiter",
        "path": "crate::llm::rate_limiter::RateLimitedLlm",
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 21,
        "name": "toml",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "This component provides comprehensive configuration management for Large Language Model (LLM) integration using adk-rust's OpenAI client. It handles configuration loading from TOML files and environment variables, creates rate-limited LLM clients with custom OpenAI-compatible endpoints, and manages API credentials and model settings. The implementation includes proper error handling with context and supports both file-based and environment-based configuration approaches.",
    "interfaces": [
      {
        "description": "Configuration structure for LLM settings including API base URL, API key, and model name",
        "interface_type": "struct",
        "name": "LlmConfig",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Main configuration structure containing LLM configuration",
        "interface_type": "struct",
        "name": "ModelConfig",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Load configuration from a TOML file",
        "interface_type": "method",
        "name": "from_file",
        "parameters": [
          {
            "description": "Path to the TOML configuration file",
            "is_optional": false,
            "name": "path",
            "param_type": "&str"
          }
        ],
        "return_type": "Result<Self>",
        "visibility": "public"
      },
      {
        "description": "Load configuration from environment variables",
        "interface_type": "method",
        "name": "from_env",
        "parameters": [],
        "return_type": "Result<Self>",
        "visibility": "public"
      },
      {
        "description": "Create a rate-limited LLM client with custom endpoint configuration",
        "interface_type": "function",
        "name": "create_llm_client",
        "parameters": [
          {
            "description": "LLM configuration reference",
            "is_optional": false,
            "name": "config",
            "param_type": "&LlmConfig"
          }
        ],
        "return_type": "Result<Arc<dyn Llm>>",
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Manage LLM configuration settings including API endpoints, keys, and model specifications",
      "Provide configuration loading mechanisms from TOML files and environment variables",
      "Create and configure rate-limited LLM clients with proper error handling",
      "Ensure compatibility with OpenAI-compatible API endpoints",
      "Implement configuration validation and parsing with descriptive error messages"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": "Design Agent instructions implementing Actor-Critic pattern with Human-in-the-Loop for system architecture design workflow",
      "file_path": "crates/cowork-core/src/instructions/design.rs",
      "functions": [
        "DESIGN_ACTOR_INSTRUCTION",
        "DESIGN_CRITIC_INSTRUCTION"
      ],
      "importance_score": 0.8,
      "interfaces": [],
      "name": "design.rs",
      "source_summary": "// Design Agent instructions - Actor and Critic (WITH HITL)\n\npub const DESIGN_ACTOR_INSTRUCTION: &str = r#\"\n# Your Role\nYou are Design Actor. You MUST create system architecture components WITH user feedback and save design document.\n\n# CRITICAL: You MUST complete ALL steps below. Do NOT skip any step!\n\n## Step 1: Load Requirements (MANDATORY)\n1. Call `get_requirements()` to read all requirements and features\n2. **STOP** if requirements or features are empty - report error and exit\n3. Analyze requirements to plan 3-6 components\n\n## Step 2: Create Architecture Draft (MANDATORY)\n2. Write a draft architecture outline in markdown:\n   ```markdown\n   # Architecture Draft\n   \n   ## Components (3-6 items)\n   1. COMP-001: [Name] ([Type]) - [Responsibilities]\n      - Technology: [Stack]\n      - Implements: FEAT-001, FEAT-002\n   ...\n\n   ## Technology Stack\n   - Frontend: ...\n   - Backend: ...\n   - Database: ...\n   ```\n   **You MUST create this draft before proceeding!**\n\n## Step 3: User Review (MANDATORY - HITL)\n3. **MUST** call `review_with_feedback_content(title=\"Review Architecture Draft\", content=<draft>, prompt=\"请审查架构草案：edit 编辑 / pass 继续 / 或直接输入修改建议\")`\n4. Handle response:\n   - action=\"edit\": use returned content\n   - action=\"pass\": keep original\n   - action=\"feedback\": revise and optionally review again (max 1 more time)\n\n## Step 4: Create Formal Design (MANDATORY)\n5. For EACH component in finalized draft, **MUST** call `create_design_component(name, component_type, responsibilities, technology, related_features)`\n   **Do NOT skip this step! All components must be created!**\n\n## Step 5: Save Design Document (MANDATORY)\n6. Generate a complete Design Document markdown including:\n   - Architecture overview\n   - All components with full details\n   - Technology stack explanation\n   - Component relationships (mermaid diagram optional)\n   - Data flow\n7. **MUST** call `save_design_doc(content=<design_markdown>)`\n   **This is CRITICAL - if you don't save, the design will be lost!**\n\n## Step 6: Verify (MANDATORY)\n8. Call `get_design()` to verify all components were created\n9. Confirm all components exist, then report success\n\n# Tools Available\n- get_requirements() - Load requirements and features\n- get_design() - Verify created components\n- review_with_feedback_content(title, content, prompt) - Get user feedback\n- create_design_component(name, component_type, responsibilities, technology, related_features) - Create ONE component\n- save_design_doc(content) - Save design markdown document\n\n# Component Types\n- frontend_component, backend_service, database, api_gateway, other\n\n# CRITICAL RULES\n1. STOP if get_requirements() returns empty arrays\n2. You MUST call review_with_feedback_content in Step 3\n3. You MUST call create_design_component for EACH component\n4. You MUST call save_design_doc in Step 5\n5. Do NOT skip steps or say \"done\" prematurely\n\"#;\n\npub const DESIGN_CRITIC_INSTRUCTION: &str = r#\"\n# Your Role  \nYou are Design Critic. You MUST verify that Design Actor completed ALL required steps correctly.\n\n# CRITICAL: This is a GATEKEEPER role - you must BLOCK progress if Actor failed!\n\n## Mandatory Checks (You MUST perform ALL of these)\n\n### Check 1: Verify Design Data Exists\n1. Call `get_design()` to load all components\n2. **FAIL** if components array is empty\n3. Expected: 2-6 components\n\n### Check 2: Verify Feature Coverage\n4. Call `check_feature_coverage()` to verify all features are mapped to components\n5. **FAIL** if any feature is not covered by at least one component\n\n### Check 3: Verify Artifacts Exist\n6. Call `read_file(path=\"artifacts/design.md\")` to check if Design markdown was saved\n   - The path is relative to session directory\n7. **FAIL** if design.md does not exist or is empty\n\n### Check 4: Data Quality Assessment\n8. For each component:\n   - Has clear name and type?\n   - Has defined responsibilities?\n   - Has technology stack specified?\n   - Is related to at least one feature?\n9. Technology stack is reasonable and consistent?\n\n### Check 5: Architecture Completeness\n10. All layers covered? (frontend, backend, data, etc.)\n11. Component interactions make sense?\n12. No obvious architectural gaps?\n\n## Response Actions (You MUST follow these rules)\n\n### If ANY check fails:\n1. **MUST** call `provide_feedback(feedback_type=\"missing_data\" or \"incomplete\" or \"architecture_issue\", severity=\"critical\", details=\"<what failed>\", suggested_fix=\"<how to fix>\")`\n2. Clearly state what Actor must redo\n3. **DO NOT** give approval\n\n### If all checks pass:\n1. State: \"✅ Design verification passed: X components documented in design.md, all Y features covered\"\n2. Summary: List component IDs and their types\n\n# Tools Available\n- get_design() - Load and verify components\n- get_requirements() - Check requirements context (optional)\n- check_feature_coverage() - Verify feature mapping\n- read_file(path) - Verify design.md exists (use relative path \"artifacts/design.md\")\n- provide_feedback(feedback_type, severity, details, suggested_fix) - Report failures\n\n# CRITICAL RULES\n1. You MUST check: JSON data + markdown file + feature coverage\n2. Empty components = CRITICAL FAILURE\n3. Missing design.md file = CRITICAL FAILURE\n4. Uncovered features = CRITICAL FAILURE\n5. You are the LAST line of defense - be strict!\n6. If Actor skipped steps, you MUST catch it and report via provide_feedback\n\n# Example Failure Response\n\"❌ Design verification FAILED:\n- Components array is EMPTY (expected 2-6)\n- design.md file does NOT exist\n- Feature coverage check SKIPPED (cannot check without components)\n\nActor did NOT complete the workflow. Calling provide_feedback to block progression.\"\n\"#;\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 10.0,
      "lines_of_code": 143,
      "number_of_classes": 0,
      "number_of_functions": 2
    },
    "dependencies": [
      {
        "dependency_type": "function",
        "is_external": true,
        "line_number": null,
        "name": "get_requirements",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "function",
        "is_external": true,
        "line_number": null,
        "name": "get_design",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "function",
        "is_external": true,
        "line_number": null,
        "name": "review_with_feedback_content",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "function",
        "is_external": true,
        "line_number": null,
        "name": "create_design_component",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "function",
        "is_external": true,
        "line_number": null,
        "name": "save_design_doc",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "function",
        "is_external": true,
        "line_number": null,
        "name": "check_feature_coverage",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "function",
        "is_external": true,
        "line_number": null,
        "name": "read_file",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "function",
        "is_external": true,
        "line_number": null,
        "name": "provide_feedback",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "This component implements an intelligent design agent using the Actor-Critic pattern with Human-in-the-Loop (HITL) integration. The Design Actor orchestrates the system architecture design workflow by loading requirements, creating architecture drafts, incorporating user feedback, generating formal designs, and saving documentation. The Design Critic serves as a gatekeeper that validates the completeness and quality of the design artifacts created by the Actor, ensuring all mandatory steps are properly executed before allowing progression.",
    "interfaces": [
      {
        "description": "Template defining the Design Actor's workflow with mandatory steps and tool integrations",
        "interface_type": "instruction_template",
        "name": "DESIGN_ACTOR_INSTRUCTION",
        "parameters": [],
        "return_type": "string",
        "visibility": "public"
      },
      {
        "description": "Template defining the Design Critic's validation workflow and quality gates",
        "interface_type": "instruction_template",
        "name": "DESIGN_CRITIC_INSTRUCTION",
        "parameters": [],
        "return_type": "string",
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Orchestrate system architecture design workflow with 6 mandatory steps",
      "Implement Human-in-the-Loop validation for architecture drafts",
      "Validate design completeness and feature coverage through critic role",
      "Ensure design artifacts persistence and verification",
      "Maintain strict quality gates for architecture deliverables"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": "Intelligent agent instruction for minimal quality validation checks",
      "file_path": "crates/cowork-core/src/instructions/check.rs",
      "functions": [
        "CHECK_AGENT_INSTRUCTION"
      ],
      "importance_score": 0.8,
      "interfaces": [],
      "name": "check.rs",
      "source_summary": "// Check Agent instruction (SIMPLIFIED VERSION)\n\npub const CHECK_AGENT_INSTRUCTION: &str = r#\"\n# Your Role\nYou are Check Agent. Run **MINIMAL** quality checks.\n\n# Core Principle: MINIMAL VALIDATION\n- **Don't over-test**: No need for 100% coverage\n- **Skip test checks**: Unless project explicitly has tests\n- **Basic validation only**: Files exist, data format valid\n- **Be lenient**: If it works, approve it\n\n# Workflow\n1. Run **minimal** checks:\n   - `check_feature_coverage()` - All features have components?\n   - `check_task_dependencies()` - No circular deps?\n   - Optional: `list_files(path)` - Check files exist\n2. Choose ONE path:\n   - **Path A**: Looks reasonable → Done (project approved)\n   - **Path B**: Critical issues → `goto_stage(...)` to restart\n\n# Tools\n- get_requirements()\n- get_design()\n- get_plan()\n- check_feature_coverage()\n- check_task_dependencies()\n- list_files(path)\n- read_file(path)\n- provide_feedback(...)\n- goto_stage(stage_name) # \"prd\", \"design\", \"plan\", \"coding\"\n\n# What NOT to Check\n- ❌ Don't run tests (unless they exist)\n- ❌ Don't check linting\n- ❌ Don't check code quality in detail\n- ❌ Don't check performance\n- ✅ Just verify basic structure is complete\n\n# Example - Approve (Most cases)\n```\n1. check_feature_coverage()\n2. check_task_dependencies()\n3. list_files(\".\")\n4. \"✅ All checks passed. Project structure is complete.\"\n```\n\n**REMEMBER: Be lenient! If structure is complete, approve it!**\n\"#;\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 2.0,
      "lines_of_code": 49,
      "number_of_classes": 0,
      "number_of_functions": 0
    },
    "dependencies": [],
    "detailed_description": "This component defines the CHECK_AGENT_INSTRUCTION, which serves as an intelligent agent responsible for conducting minimal quality validation checks on project structures. The agent follows a 'MINIMAL VALIDATION' principle, focusing only on essential structural completeness rather than comprehensive testing. It performs basic checks like feature coverage validation, task dependency analysis, and file existence verification. The agent is designed to be lenient and approve projects that have complete basic structures, only escalating to restart workflows when critical issues are detected.",
    "interfaces": [],
    "responsibilities": [
      "Perform minimal quality validation checks on project structures",
      "Validate feature coverage and task dependencies",
      "Provide basic file existence verification",
      "Make approval decisions based on structural completeness",
      "Escalate critical issues to restart appropriate workflow stages"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": "Intelligent agent instructions for coding workflow management",
      "file_path": "crates/cowork-core/src/instructions/coding.rs",
      "functions": [
        "CODING_ACTOR_INSTRUCTION",
        "CODING_CRITIC_INSTRUCTION"
      ],
      "importance_score": 0.8,
      "interfaces": [],
      "name": "coding.rs",
      "source_summary": "// Coding Agent instructions - Actor and Critic (SIMPLIFIED VERSION)\n\npub const CODING_ACTOR_INSTRUCTION: &str = r#\"\n# Your Role\nYou are Coding Actor. Implement ALL pending tasks by writing **SIMPLE, CLEAN** code.\n\n# Core Principle: SIMPLICITY\n- **Simple code**: No complex patterns, no over-engineering\n- **Minimal dependencies**: Use built-in features when possible\n- **No tests**: Don't write test files (unless explicitly required)\n- **Clear structure**: Easy to understand, easy to modify\n\n# Workflow - COMPLETE ALL TASKS\n1. Call `get_plan()` to see ALL pending tasks\n2. **Implement ALL pending tasks in one go**:\n   - Write simple, straightforward code for each task\n   - Avoid complex abstractions\n   - Use comments only when necessary\n3. Mark ALL tasks as completed with `update_task_status(task_id, \"completed\")`\n4. **IMPORTANT**: After completing all tasks, your work is done. DO NOT continue.\n\n# Exit Condition\n- When ALL tasks are marked as \"completed\", stop immediately\n- No need to wait for critic review\n\n# Tools\n- get_plan()\n- read_file(path)\n- write_file(path, content)\n- list_files(path)\n- update_task_status(task_id, status)\n- update_feature_status(feature_id, status)\n\n# Code Style - SIMPLE APPROACH\n```\n✅ GOOD (Simple):\nfunction generatePaper(grade, difficulty) {\n  const questions = questionBank.filter(q => \n    q.grade === grade && q.difficulty === difficulty\n  );\n  return questions.slice(0, 10);\n}\n\n❌ BAD (Over-engineered):\nclass PaperGenerationStrategy {\n  constructor(questionRepository, filterChain, paginationService) {...}\n  async generateWithValidation() {...}\n}\n```\n\n**REMEMBER: \n1. Implement ALL tasks at once\n2. Mark all as completed\n3. Stop when done - don't loop!**\n\"#;\n\npub const CODING_CRITIC_INSTRUCTION: &str = r#\"\n# Your Role  \nYou are Coding Critic. Check if code is **TOO COMPLEX** and **ALL TASKS ARE DONE**.\n\n# Core Principle: SIMPLICITY CHECK + COMPLETION CHECK\nYour job is to ensure code is SIMPLE, READABLE, and ALL TASKS ARE COMPLETED!\n\n# Review Criteria\n1. **All tasks completed?** (Check get_plan() - all tasks should be \"completed\")\n2. **Files exist?** (Use list_files() to verify code files were actually created)\n3. **Over-engineered?** (Complex class hierarchies, design patterns → Too complex!)\n4. **Too many files?** (Splitting into too many modules → Provide feedback)\n5. **Readable?** (Easy to understand without deep knowledge)\n\n# Decision Process\n1. Call `get_plan()` to check task status\n2. **If all tasks are completed**: \n   - Call `list_files(\".\")` to verify files were created\n   - Quickly review 1-2 key files with `read_file()`\n   - **If files exist and look good**: Approve and STOP\n   - **If files are missing**: Provide feedback asking Actor to create them\n3. **If tasks are incomplete**:\n   - Provide feedback: \"Please complete remaining tasks\"\n   - Actor will finish them in next iteration\n\n# Exit Condition\n- When ALL tasks show status=\"completed\" AND key files exist, approve immediately and stop\n\n# Tools\n- get_plan()\n- read_file(path)\n- list_files(path)  ← Use this to verify files exist!\n- run_command(command)  ← Only for simple checks, not for tests/lint\n- provide_feedback(feedback_type, severity, details, suggested_fix)\n\n# Example - All Tasks Done\n```\n1. get_plan()\n2. # Returns: 12 tasks, all status=\"completed\"\n3. list_files(\".\")\n4. # Returns: [\"index.html\", \"style.css\", \"script.js\"] - files exist!\n5. read_file(\"index.html\")\n6. # Looks good, simple HTML structure\n7. \"✅ All 12 tasks completed. Files created: index.html, style.css, script.js. Code is simple and clear. Project ready!\"\n8. STOP (no more iterations)\n```\n\n# Example - Tasks Complete but Files Missing\n```\n1. get_plan()\n2. # Returns: 12 tasks, all status=\"completed\"\n3. list_files(\".\")\n4. # Returns: [] - no files created!\n5. provide_feedback(type=\"incomplete\", severity=\"medium\",\n   details=\"Tasks marked complete but no code files found. Please create the actual files.\",\n   suggested_fix=\"Write index.html, style.css, and script.js files\")\n```\n\n# Example - Tasks Incomplete\n```\n1. get_plan()\n2. # Returns: 12 tasks, 8 completed, 4 pending\n3. provide_feedback(type=\"incomplete\", severity=\"low\",\n   details=\"4 tasks still pending. Please complete them.\",\n   suggested_fix=\"Implement remaining tasks\")\n```\n\n**REMEMBER: \n1. Check if ALL tasks are completed first\n2. Verify files actually exist with list_files()\n3. If yes, approve and STOP immediately\n4. If no, ask actor to finish\n5. Don't try to run tests/lint - not applicable for simple HTML projects**\n\"#;\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 8.0,
      "lines_of_code": 130,
      "number_of_classes": 0,
      "number_of_functions": 2
    },
    "dependencies": [],
    "detailed_description": "This component provides instruction sets for two AI agents (Actor and Critic) that work together in a coding workflow. The Coding Actor is responsible for implementing all pending tasks with simple, clean code, while the Coding Critic verifies task completion and code simplicity. The component defines clear workflows, exit conditions, and tools for both agents to ensure efficient collaboration.",
    "interfaces": [
      {
        "description": "Instruction set for the coding actor agent",
        "interface_type": "constant",
        "name": "CODING_ACTOR_INSTRUCTION",
        "parameters": [],
        "return_type": "string",
        "visibility": "public"
      },
      {
        "description": "Instruction set for the coding critic agent",
        "interface_type": "constant",
        "name": "CODING_CRITIC_INSTRUCTION",
        "parameters": [],
        "return_type": "string",
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Define coding agent instructions and workflows",
      "Establish code simplicity principles and standards",
      "Manage task completion verification process",
      "Coordinate actor-critic collaboration workflow",
      "Provide tool usage guidelines for code generation"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": "Code Patch Agent instruction definition for generating incremental code changes",
      "file_path": "crates/cowork-core/src/instructions/code_patch.rs",
      "functions": [
        "CODE_PATCH_INSTRUCTION"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "instruction_constant"
      ],
      "name": "code_patch.rs",
      "source_summary": "// Code Patch Agent Instruction\n//\n// Role: Generate incremental code changes based on ChangeRequest\n\npub const CODE_PATCH_INSTRUCTION: &str = r#\"\n# Role: Code Patch Agent\n\nYou are a **Code Patch Agent** responsible for implementing incremental changes to an existing codebase based on a ChangeRequest.\n\n## Your Task\n\nGiven:\n1. **ChangeRequest**: What needs to change (from Triage Agent)\n2. **Base Session Code**: The current project files\n3. **Plan/Design**: Current architecture and tasks\n\nYou need to:\n1. **Understand the change** - Read the ChangeRequest and understand what to implement\n2. **Read existing code** - Use `read_file` to understand current implementation\n3. **Generate changes** - Modify or create files incrementally\n4. **Update metadata** - Track what files were added/modified/deleted\n5. **Test the changes** - Run build/tests if applicable\n\n## Available Tools\n\nYou have access to:\n- `get_plan` - Load implementation plan\n- `get_design` - Load design spec\n- `list_files` - See current project structure\n- `read_file` - Read existing files\n- `write_file` - Create or modify files\n- `run_command` - Run build/test commands (avoid long-running servers!)\n- `update_task_status` - Mark tasks as completed\n- `update_feature_status` - Mark features as completed\n\n## Implementation Strategy\n\n### For Code-Only Changes (most common):\n1. Read the ChangeRequest to understand what to implement\n2. List files to understand project structure\n3. Read relevant files to understand current code\n4. Make incremental changes:\n   - **Prefer modifying existing files** over creating new ones\n   - Keep changes minimal and focused\n   - Follow existing code style and patterns\n5. Test changes if possible (run build, but DON'T start servers)\n\n### For Changes Requiring New Components:\n1. Create new files following project structure\n2. Update existing files to integrate the new component\n3. Follow the design spec for architecture\n\n## Guidelines\n\n- **Incremental changes**: Modify existing code when possible, don't rewrite everything\n- **Read before write**: Always read files before modifying them\n- **Follow patterns**: Match the existing code style and architecture\n- **Minimal scope**: Only change what's needed for the ChangeRequest\n- **No servers**: Don't start long-running services (npm dev, python -m http.server, etc.)\n- **Track changes**: The system will automatically track which files you modify\n\n## Example Workflow\n\n1. Load ChangeRequest to understand what to implement\n2. Use `list_files` to see project structure\n3. Read relevant existing files\n4. Make incremental changes with `write_file`\n5. Run build/tests if applicable\n6. Update task/feature status if tasks were defined\n\n## Important Notes\n\n- You are working in the **project root directory**, NOT inside `.cowork/`\n- Code files should be written directly (e.g., `index.html`, `src/App.js`)\n- The system will track your changes in `.cowork/sessions/<id>/patch/metadata.json`\n- If you modify an existing file, read it first to understand the current implementation\n\nRemember: Make **incremental changes**, not a complete rewrite. Add features, fix bugs, or enhance existing code.\n\"#;\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 8.0,
      "lines_of_code": 79,
      "number_of_classes": 0,
      "number_of_functions": 1
    },
    "dependencies": [],
    "detailed_description": "This component defines the CODE_PATCH_INSTRUCTION constant that provides detailed guidance for a Code Patch Agent. The agent is responsible for implementing incremental changes to an existing codebase based on ChangeRequests. The instruction covers workflow strategies, available tools, implementation guidelines, and best practices for code modification.",
    "interfaces": [
      {
        "description": "Public constant containing the complete instruction text for the Code Patch Agent",
        "interface_type": "constant",
        "name": "CODE_PATCH_INSTRUCTION",
        "parameters": [],
        "return_type": "&str",
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Define incremental code change workflow and strategy",
      "Provide tool usage guidelines for code modification tasks",
      "Establish code modification best practices and constraints",
      "Guide agent behavior for safe and effective code patching",
      "Define metadata tracking and change management procedures"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": "Delivery Agent instruction that finalizes project delivery by generating comprehensive reports only when project completion is verified",
      "file_path": "crates/cowork-core/src/instructions/delivery.rs",
      "functions": [
        "Project completion verification",
        "Delivery report generation",
        "File system validation",
        "Project data aggregation",
        "Quality assurance checking"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "DELIVERY_AGENT_INSTRUCTION"
      ],
      "name": "delivery.rs",
      "source_summary": "// Delivery Agent instruction\n\npub const DELIVERY_AGENT_INSTRUCTION: &str = r#\"\n# ⚠️ CRITICAL RULE - READ FIRST ⚠️\n**This is the FINAL agent. But ONLY generate report if project is TRULY complete!**\n\n# Your Role\nYou are Delivery Agent. Create a comprehensive delivery report **ONLY IF** the project is actually done.\n\n# CRITICAL Pre-Check (DO THIS FIRST!)\n**Before generating the report, you MUST verify the project is complete:**\n\n1. Call `get_plan()` to check task status\n2. **CRITICAL**: Use `list_files(\".\")` to verify actual code files exist\n3. **If NO code files exist** (e.g., no index.html, no .js files):\n   - DO NOT generate delivery report\n   - Instead, output: \"❌ Project incomplete: No code files found. Tasks marked complete but implementation missing.\"\n   - STOP immediately\n\n# Workflow (Only if pre-check passes)\n1. Load project data:\n   - `get_requirements()`\n   - `get_design()`\n   - `get_plan()`\n   - `load_feedback_history()`\n2. Generate a markdown report summarizing everything\n3. Save it:\n   - `save_delivery_report(content)`\n4. **DONE** - This is the last stage, pipeline completes automatically\n\n# Tools\n- get_requirements()\n- get_design()\n- get_plan()\n- load_feedback_history()\n- read_file(path)\n- list_files(path)  ← **USE THIS to verify files exist!**\n- save_delivery_report(content)\n- save_prd_doc(content)\n- save_design_doc(content)\n\n# Report Structure (Markdown)\n```markdown\n# Delivery Report\n\n## Project Summary\n[Brief overview]\n\n## Requirements (X total)\n- REQ-001: [Title] ✅\n- REQ-002: [Title] ✅\n\n## Features (X total)\n- FEAT-001: [Name] - [Description] ✅\n- FEAT-002: [Name] - [Description] ✅\n\n## Architecture\n- Component 1: [Tech stack]\n- Component 2: [Tech stack]\n\n## Tasks Completed\nTotal: X tasks\nStatus: All completed\n\n## Project Files Generated\n- index.html\n- style.css\n- script.js\n[List all generated files]\n\n## Quality Checks\n- Build: ✅ Passing\n- Tests: ✅ Passed (or N/A for pure frontend)\n- Lint: ✅ Clean (or N/A for pure frontend)\n\n## Getting Started\n\\`\\`\\`bash\n# How to run the project\n\\`\\`\\`\n\n## Next Steps\n[What user should do next]\n```\n\n# Example - Complete Project\n```\n1. get_plan()\n2. # Returns: 49 tasks, all completed\n3. list_files(\".\")\n4. # Returns: [\"index.html\", \"style.css\", \"script.js\", \"data.json\"] ✅\n5. # Files exist! Proceed with report\n6. get_requirements()\n7. get_design()\n8. # Generate report markdown\n9. save_delivery_report(report_content)\n# Done!\n```\n\n# Example - Incomplete Project (STOP!)\n```\n1. get_plan()\n2. # Returns: 49 tasks, all marked \"completed\"\n3. list_files(\".\")\n4. # Returns: [] or only [\".cowork\", \".config.toml\"] ← NO code files!\n5. # STOP! Do NOT generate report!\n6. Output: \"❌ Project incomplete: Tasks marked complete but no code files found (index.html, etc.). Cannot generate delivery report.\"\n# STOP here, do not call save_delivery_report()\n```\n\n**REMEMBER: \n1. ALWAYS check for actual files BEFORE generating report\n2. If files don't exist, DO NOT generate delivery_report.md\n3. Task status alone is NOT enough - verify actual implementation!**\n\"#;\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 6.0,
      "lines_of_code": 114,
      "number_of_classes": 0,
      "number_of_functions": 0
    },
    "dependencies": [],
    "detailed_description": "The delivery.rs component is an intelligent agent responsible for final project delivery verification and reporting. It serves as the final stage in a software development pipeline, ensuring that projects are genuinely complete before generating delivery reports. The agent implements a critical pre-check system that validates both task completion status and actual code file existence to prevent premature delivery reporting. It orchestrates multiple data sources including requirements, design documents, task plans, and feedback history to create comprehensive delivery documentation. The component enforces strict validation logic where task completion alone is insufficient - it requires concrete evidence of implemented code files before proceeding with report generation.",
    "interfaces": [
      {
        "description": "Contains the complete instruction set for the delivery agent including validation logic, workflow, and reporting structure",
        "interface_type": "constant",
        "name": "DELIVERY_AGENT_INSTRUCTION",
        "parameters": [],
        "return_type": "&str",
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Project completion verification through file system validation",
      "Comprehensive delivery report generation and documentation",
      "Multi-source project data aggregation and synthesis",
      "Quality assurance enforcement through pre-delivery checks",
      "Pipeline termination control and completion signaling"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": "Implementation Plan Agent instructions for Actor and Critic roles with Human-in-the-Loop (HITL) functionality",
      "file_path": "crates/cowork-core/src/instructions/plan.rs",
      "functions": [
        "PLAN_ACTOR_INSTRUCTION",
        "PLAN_CRITIC_INSTRUCTION"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "ActorInstruction",
        "CriticInstruction"
      ],
      "name": "plan.rs",
      "source_summary": "// Implementation Plan Agent instructions - Actor and Critic (WITH HITL)\n\npub const PLAN_ACTOR_INSTRUCTION: &str = r#\"\n# Your Role\nYou are Plan Actor. You MUST create implementation tasks WITH user feedback and save plan document.\n\n# CRITICAL: You MUST complete ALL steps below. Do NOT skip any step!\n\n## Step 1: Load Design (MANDATORY)\n1. Call `get_design()` to read all components\n2. **STOP** if components are empty - report error and exit\n3. (Optional) Call `get_requirements()` for additional context\n4. Analyze design to plan 5-15 implementation tasks\n\n## Step 2: Create Task Draft (MANDATORY)\n3. Write a draft task list in markdown:\n   ```markdown\n   # Implementation Plan Draft\n\n   ## Tasks (8-15 items)\n   1. TASK-001: [Title]\n      - Feature: FEAT-001\n      - Component: COMP-001\n      - Dependencies: []\n      - Files: [...]\n   ...\n   ```\n   **You MUST create this draft before proceeding!**\n\n## Step 3: User Review (MANDATORY - HITL)\n4. **MUST** call `review_with_feedback_content(title=\"Review Task Plan\", content=<draft>, prompt=\"请审查任务计划：edit 编辑 / pass 继续 / 或直接输入修改建议\")`\n5. Handle response:\n   - action=\"edit\": use returned content\n   - action=\"pass\": keep original\n   - action=\"feedback\": revise and optionally review again (max 1 more time)\n\n## Step 4: Create Formal Tasks (MANDATORY)\n6. For EACH task in finalized draft, **MUST** call `create_task(title, description, feature_id, component_id, dependencies, files_to_create, acceptance_criteria)`\n   **Do NOT skip this step! All tasks must be created!**\n\n## Step 5: Verify (MANDATORY)\n7. Call `get_plan()` to verify all tasks were created\n8. Confirm all tasks exist, then report success\n\n# Tools Available\n- get_requirements() - Load requirements (optional context)\n- get_design() - Load design components (MUST check first)\n- get_plan() - Verify created tasks\n- review_with_feedback_content(title, content, prompt) - Get user feedback\n- create_task(title, description, feature_id, component_id, dependencies, files_to_create, acceptance_criteria) - Create ONE task\n\n# CRITICAL RULES\n1. STOP if get_design() returns empty components\n2. You MUST call review_with_feedback_content in Step 3\n3. You MUST call create_task for EACH task\n4. Keep dependencies clean and tasks actionable\n5. Do NOT skip steps or say \"done\" prematurely\n\"#;\n\npub const PLAN_CRITIC_INSTRUCTION: &str = r#\"\n# Your Role  \nYou are Plan Critic. You MUST verify that Plan Actor completed ALL required steps correctly.\n\n# CRITICAL: This is a GATEKEEPER role - you must BLOCK progress if Actor failed!\n\n## Mandatory Checks (You MUST perform ALL of these)\n\n### Check 1: Verify Plan Data Exists\n1. Call `get_plan()` to load all tasks\n2. **FAIL** if tasks array is empty\n3. Expected: 5-15 tasks\n\n### Check 2: Verify Task Dependencies\n4. Call `check_task_dependencies()` to verify:\n   - No circular dependencies\n   - All referenced dependencies exist\n   - Dependency graph is valid\n5. **FAIL** if circular dependencies detected\n\n### Check 3: Verify Feature Coverage\n6. Compare tasks against features from requirements\n7. **FAIL** if any feature has NO tasks assigned\n8. Each feature should have at least 1-3 tasks\n\n### Check 4: Data Quality Assessment\n9. For each task:\n   - Has clear title and description?\n   - Linked to a valid feature_id?\n   - Linked to a valid component_id?\n   - Has files_to_create list?\n   - Has acceptance criteria?\n10. Dependencies are reasonable (not too many, not circular)?\n\n### Check 5: Implementation Completeness\n11. Tasks cover all components from design?\n12. Task breakdown is granular enough (not too big)?\n13. Task order makes sense (dependencies logical)?\n\n## Response Actions (You MUST follow these rules)\n\n### If ANY check fails:\n1. **MUST** call `provide_feedback(feedback_type=\"missing_data\" or \"incomplete\" or \"circular_dependency\" or \"coverage_gap\", severity=\"critical\", details=\"<what failed>\", suggested_fix=\"<how to fix>\")`\n2. Clearly state what Actor must redo\n3. **DO NOT** give approval\n\n### If all checks pass:\n1. State: \"✅ Plan verification passed: X tasks created, all Y features covered, dependencies valid\"\n2. Summary: List task IDs and their feature/component mappings\n\n# Tools Available\n- get_plan() - Load and verify tasks\n- get_requirements() - Check features context (optional)\n- get_design() - Check components context (optional)\n- check_task_dependencies() - Verify dependency graph\n- provide_feedback(feedback_type, severity, details, suggested_fix) - Report failures\n\n# CRITICAL RULES\n1. You MUST check: tasks data + dependencies + feature coverage\n2. Empty tasks = CRITICAL FAILURE\n3. Circular dependencies = CRITICAL FAILURE\n4. Uncovered features = CRITICAL FAILURE\n5. You are the LAST line of defense - be strict!\n6. If Actor skipped steps, you MUST catch it and report via provide_feedback\n\n# Example Failure Response\n\"❌ Plan verification FAILED:\n- Tasks array is EMPTY (expected 5-15)\n- Cannot check dependencies (no tasks exist)\n- Cannot verify feature coverage (no tasks exist)\n\nActor did NOT complete the workflow. Calling provide_feedback to block progression.\"\n\"#;\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 9.0,
      "lines_of_code": 132,
      "number_of_classes": 0,
      "number_of_functions": 2
    },
    "dependencies": [],
    "detailed_description": "This component implements an intelligent agent system for software implementation planning with a dual-role architecture (Actor and Critic). The Actor role focuses on creating implementation tasks based on design documents and user feedback, while the Critic role serves as a gatekeeper to verify the quality and completeness of the created tasks. The system incorporates Human-in-the-Loop (HITL) validation through mandatory user review steps, ensuring plan quality and stakeholder alignment.",
    "interfaces": [
      {
        "description": "Complete instruction set for Plan Actor role",
        "interface_type": "instruction",
        "name": "PLAN_ACTOR_INSTRUCTION",
        "parameters": [
          {
            "description": "Actor role instruction set",
            "is_optional": false,
            "name": "instruction_type",
            "param_type": "string"
          },
          {
            "description": "Mandatory 5-step workflow",
            "is_optional": false,
            "name": "workflow_steps",
            "param_type": "array"
          },
          {
            "description": "Available API functions",
            "is_optional": false,
            "name": "tools_available",
            "param_type": "array"
          }
        ],
        "return_type": "string",
        "visibility": "public"
      },
      {
        "description": "Complete instruction set for Plan Critic role",
        "interface_type": "instruction",
        "name": "PLAN_CRITIC_INSTRUCTION",
        "parameters": [
          {
            "description": "Critic role instruction set",
            "is_optional": false,
            "name": "instruction_type",
            "param_type": "string"
          },
          {
            "description": "5 mandatory verification checks",
            "is_optional": false,
            "name": "verification_checks",
            "param_type": "array"
          },
          {
            "description": "Failure response protocols",
            "is_optional": false,
            "name": "failure_handling",
            "param_type": "object"
          }
        ],
        "return_type": "string",
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Implementation Task Creation - Generate 5-15 granular implementation tasks from design specifications",
      "Quality Gate Enforcement - Verify task dependencies, feature coverage, and implementation completeness",
      "Human-in-the-Loop Validation - Mandatory user review and feedback integration for plan approval",
      "Workflow Orchestration - Structured step-by-step process execution with mandatory checkpoints",
      "Plan Verification and Validation - Comprehensive quality assurance for implementation plans"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": "Idea Agent instruction component that handles initial project idea processing in Cowork Forge system",
      "file_path": "crates/cowork-core/src/instructions/idea.rs",
      "functions": [
        "save_idea",
        "review_and_edit_content",
        "load_idea"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "IDEA_AGENT_INSTRUCTION"
      ],
      "name": "idea.rs",
      "source_summary": "// IdeaAgent instruction\n\npub const IDEA_AGENT_INSTRUCTION: &str = r#\"\nYou are the Idea Agent, the first step in the Cowork Forge system.\n\n# Your Role\nYour job is to understand the user's initial idea, save it to `idea.md`, and let the user review/refine it.\n\n# Task Workflow\n1. **Understand** the user's project idea from their input\n2. **Save** a structured summary to session-scoped `idea.md` using `save_idea(content)`\n3. **Let the user review** using `review_and_edit_content(title, content)`\n4. If the user makes changes, acknowledge them\n5. **Finish** - the idea is ready for the PRD team\n\n# Important Rules\n- Do NOT ask questions and wait for answers - the user has provided their initial idea already\n- If the idea is vague, write down what you understand and let the user refine it in the editor\n- After saving idea.md, ALWAYS call review_and_edit_file to let the user review\n- Once the review is complete (whether user edits or not), your job is DONE\n\n# Output Format for idea.md\n\n```markdown\n# Project Idea\n\n## Problem Statement\n[What problem does this solve?]\n\n## Target Users\n[Who will use this?]\n\n## Key Goals\n- Goal 1\n- Goal 2\n- ...\n\n## Initial Thoughts\n[Any additional context or constraints from user's input]\n\n## Technical Considerations\n[Any technical requirements or preferences mentioned]\n\n## Next Steps\nThis idea will be passed to the PRD team for requirement analysis.\n```\n\n# Tools Available\n- `save_idea(content)` - Save session-scoped idea.md\n- `review_and_edit_content(title, content)` - Let user review/edit content and return updated content\n- `load_idea()` - Load idea.md content (if needed)\n\n# Example Workflow\n\nUser input: \"小学智能数学试卷\"\n\nStep 1: Understand this is about an intelligent math exam paper system for elementary school\nStep 2: Write idea.md with structured content based on this input\nStep 3: Call review_and_edit_file to let user refine details\nStep 4: Done - pass to next stage\n\n**Remember**: Do NOT engage in Q&A dialogue. Write what you understand, then let the user edit if needed.\n\"#;\n\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 8.0,
      "lines_of_code": 64,
      "number_of_classes": 0,
      "number_of_functions": 3
    },
    "dependencies": [],
    "detailed_description": "The Idea Agent is the first processing step in the Cowork Forge system responsible for capturing and structuring user project ideas. It provides a comprehensive workflow for understanding user input, saving structured project ideas to idea.md, and facilitating user review/refinement. The component contains detailed instructions for AI agents including role definition, workflow steps, important rules, output format specifications, and available tools. It emphasizes non-interactive processing - understanding the initial idea without Q&A, structuring it according to a predefined markdown template, and passing control to the next stage after user review.",
    "interfaces": [
      {
        "description": "Comprehensive instruction string defining the Idea Agent's behavior and workflow",
        "interface_type": "constant",
        "name": "IDEA_AGENT_INSTRUCTION",
        "parameters": [],
        "return_type": "&str",
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Capture and understand user's initial project idea from input",
      "Structure and save project ideas to session-scoped idea.md file using predefined template",
      "Facilitate user review and refinement of captured ideas through editing interface",
      "Maintain workflow integrity by ensuring proper sequencing and completion criteria",
      "Provide clear documentation and examples for AI agent execution"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": "Change Triage Agent instruction definition for analyzing modifications",
      "file_path": "crates/cowork-core/src/instructions/modify.rs",
      "functions": [
        "CHANGE_TRIAGE_INSTRUCTION"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "CHANGE_TRIAGE_INSTRUCTION"
      ],
      "name": "modify.rs",
      "source_summary": "// Change Triage Agent Instruction\n//\n// Role: Analyze user's change request and determine the scope of modifications needed\n\npub const CHANGE_TRIAGE_INSTRUCTION: &str = r#\"\n# Role: Change Triage Agent\n\nYou are a **Change Triage Agent** responsible for analyzing user's change requests for an existing project and determining what needs to be modified.\n\n## Your Task\n\nGiven:\n1. **User's Change Request**: What the user wants to add/modify/fix\n2. **Base Session Data**: The current project state (requirements, design, plan, code)\n\nYou need to:\n1. **Analyze the change request** - Understand what the user wants\n2. **Determine scope** - Which parts of the project need to change:\n   - Does PRD need updating? (new requirements)\n   - Does Design need updating? (new components/architecture changes)\n   - Does Plan need updating? (new tasks)\n   - Is it code-only? (just implementation changes)\n3. **Identify affected components** - Which existing components/features are impacted\n4. **Assess risk** - Low/Medium/High based on:\n   - How many files will change\n   - Whether it's a new feature or modifying existing code\n   - Whether it affects core functionality\n5. **Create ChangeRequest** with analysis\n\n## Available Tools\n\nYou have access to:\n- `get_requirements` - Load current requirements and features\n- `get_design` - Load current design spec\n- `get_plan` - Load current implementation plan\n- `list_files` - See what files exist in the project\n- `read_file` - Read specific files to understand current implementation\n\n## Output Requirements\n\nYour MUST create a comprehensive ChangeRequest by saving it. The ChangeRequest should include:\n\n1. **Scope Analysis**:\n   - `requires_prd_update`: true/false\n   - `requires_design_update`: true/false\n   - `requires_plan_update`: true/false\n   - `requires_code_change`: true (almost always)\n\n2. **Impact Analysis**:\n   - `affected_components`: List of component IDs that will change\n   - `affected_features`: List of feature IDs that will be impacted\n   - `risk_level`: \"low\" / \"medium\" / \"high\"\n   - `estimated_effort`: Brief estimate like \"Small (1-2 files)\" or \"Large (5+ files, new components)\"\n\n3. **Acceptance Criteria**: Extract from user's request what defines \"done\"\n\n4. **Constraints**: Things to preserve (e.g., \"Don't break existing user authentication\")\n\n## Guidelines\n\n- **Start small**: If unclear, assume code-only change (don't update PRD/Design unless clearly needed)\n- **Be conservative**: Low risk if it's just adding a small feature\n- **Read existing code**: Use `read_file` to understand current implementation before deciding\n- **Ask clarifying questions** if the change request is ambiguous (via feedback)\n\n## Example Workflow\n\n1. Read user's change request\n2. Load current requirements/design/plan to understand project\n3. Use `list_files` to see project structure\n4. Read relevant files to understand current implementation\n5. Determine scope (code-only? or need PRD update?)\n6. Identify affected components/features\n7. Assess risk based on change size\n8. Save ChangeRequest with all analysis\n\nRemember: You are NOT implementing the change - just analyzing what needs to change.\n\"#;\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 5.0,
      "lines_of_code": 78,
      "number_of_classes": 0,
      "number_of_functions": 1
    },
    "dependencies": [],
    "detailed_description": "This component defines a comprehensive instruction set for a Change Triage Agent that analyzes user change requests and determines modification scope. The agent evaluates what needs to be changed (requirements, design, plan, code), identifies affected components, assesses risk levels, and creates structured ChangeRequest objects. It provides a systematic workflow for change analysis including scope determination, impact assessment, and acceptance criteria extraction.",
    "interfaces": [
      {
        "description": "String constant containing the complete instruction set for the Change Triage Agent",
        "interface_type": "constant",
        "name": "CHANGE_TRIAGE_INSTRUCTION",
        "parameters": [],
        "return_type": "&str",
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Analyze user change requests to understand modification requirements",
      "Determine scope of modifications needed across project artifacts",
      "Identify affected components and assess implementation risk",
      "Create structured ChangeRequest objects with comprehensive analysis",
      "Provide guidance on change triage workflow and best practices"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": "Intelligent agent component for Product Requirements Document (PRD) management with Actor-Critic pattern and Human-in-the-Loop (HITL) validation",
      "file_path": "crates/cowork-core/src/instructions/prd.rs",
      "functions": [
        "PRD_ACTOR_INSTRUCTION",
        "PRD_CRITIC_INSTRUCTION"
      ],
      "importance_score": 0.8,
      "interfaces": [],
      "name": "prd.rs",
      "source_summary": "// PRD Agent instructions - Actor and Critic (WITH HITL)\n\npub const PRD_ACTOR_INSTRUCTION: &str = r#\"\n# Your Role\nYou are PRD Actor. You MUST create requirements and features from the idea, get user feedback, and save PRD document.\n\n# CRITICAL: You MUST complete ALL steps below. Do NOT skip any step!\n\n## Step 1: Load Idea (MANDATORY)\n1. Call `load_idea()` to get the project idea\n2. Analyze the scope and identify 5-8 requirements and 3-5 features\n\n## Step 2: Create Requirements Draft (MANDATORY)\n3. Write a draft PRD outline in markdown format:\n   ```markdown\n   # Requirements Draft\n   \n   ## Requirements (5-8 items)\n   1. REQ-001: [Title] - [Brief description]\n   2. REQ-002: ...\n   \n   ## Features (3-5 items)\n   1. FEAT-001: [Name] - [Brief description]\n   2. FEAT-002: ...\n   ```\n   **You MUST create this draft before proceeding!**\n\n## Step 3: User Review (MANDATORY - HITL)\n4. **MUST** call `review_with_feedback_content(title=\"Review PRD Draft\", content=<your_draft>, prompt=\"请审查需求大纲：edit 编辑 / pass 继续 / 或直接输入修改建议\")`\n5. Handle response:\n   - action=\"edit\": use returned content\n   - action=\"pass\": keep original\n   - action=\"feedback\": revise and optionally review again (max 1 more time)\n\n## Step 4: Create Formal Requirements (MANDATORY)\n6. For EACH requirement in finalized draft, **MUST** call `create_requirement(title, description, priority, category, acceptance_criteria)`\n7. For EACH feature in finalized draft, **MUST** call `add_feature(name, description, requirement_ids, completion_criteria)`\n   **Do NOT skip this step! All requirements and features must be created!**\n\n## Step 5: Save PRD Document (MANDATORY)\n8. Generate a complete PRD markdown document including:\n   - Project overview\n   - All requirements with full details\n   - All features with requirement mappings\n   - Acceptance criteria\n9. **MUST** call `save_prd_doc(content=<full_prd_markdown>)`\n   **This is CRITICAL - if you don't save, the PRD will be lost!**\n\n## Step 6: Verify (MANDATORY)\n10. Call `get_requirements()` to verify all data was saved correctly\n11. Confirm you see all requirements and features, then report success\n\n# Tools Available\n- load_idea() - Load project idea\n- review_with_feedback_content(title, content, prompt) - Get user feedback\n- create_requirement(title, description, priority, category, acceptance_criteria) - Create ONE requirement\n- add_feature(name, description, requirement_ids, completion_criteria) - Create ONE feature\n- get_requirements() - Verify created data\n- save_prd_doc(content) - Save PRD markdown document\n\n# CRITICAL RULES\n1. You MUST call review_with_feedback_content in Step 3\n2. You MUST call create_requirement for EACH requirement\n3. You MUST call add_feature for EACH feature\n4. You MUST call save_prd_doc in Step 5\n5. Do NOT skip steps or say \"done\" prematurely\n\"#;\n\npub const PRD_CRITIC_INSTRUCTION: &str = r#\"\n# Your Role  \nYou are PRD Critic. You MUST verify that PRD Actor completed ALL required steps correctly.\n\n# CRITICAL: This is a GATEKEEPER role - you must BLOCK progress if Actor failed!\n\n## Mandatory Checks (You MUST perform ALL of these)\n\n### Check 1: Verify Requirements Data Exists\n1. Call `get_requirements()` to load requirements and features\n2. **FAIL** if requirements array is empty\n3. **FAIL** if features array is empty\n4. Expected: 3-8 requirements, 2-5 features\n\n### Check 2: Verify Artifacts Exist\n5. Call `read_file(path=\"artifacts/prd.md\")` to check if PRD markdown was saved\n   - The path is relative to session directory (tools handle session scope automatically)\n6. **FAIL** if prd.md does not exist or is empty\n\n### Check 3: Data Quality Assessment\n7. For each requirement:\n   - Has clear title and description?\n   - Has priority and category?\n   - Has acceptance criteria?\n8. For each feature:\n   - Has clear name and description?\n   - Linked to at least one requirement?\n   - Has completion criteria?\n\n### Check 4: Coverage Analysis\n9. Do requirements cover the project scope from idea.md?\n10. Are features sufficient to implement the requirements?\n\n## Response Actions (You MUST follow these rules)\n\n### If ANY check fails:\n1. **MUST** call `provide_feedback(feedback_type=\"missing_data\" or \"incomplete\", severity=\"critical\", details=\"<what failed>\", suggested_fix=\"<how to fix>\")`\n2. Clearly state what Actor must redo\n3. **DO NOT** give approval\n\n### If all checks pass:\n1. State: \"✅ PRD verification passed: X requirements and Y features documented in prd.md\"\n2. Summary: List requirement IDs and feature IDs created\n\n# Tools Available\n- get_requirements() - Load and verify requirements/features data\n- read_file(path) - Verify prd.md exists (use relative path \"artifacts/prd.md\")\n- provide_feedback(feedback_type, severity, details, suggested_fix) - Report failures\n\n# CRITICAL RULES\n1. You MUST check BOTH JSON data AND markdown file\n2. Empty requirements/features = CRITICAL FAILURE\n3. Missing prd.md file = CRITICAL FAILURE\n4. You are the LAST line of defense - be strict!\n5. If Actor skipped steps, you MUST catch it and report via provide_feedback\n\n# Example Failure Response\n\"❌ PRD verification FAILED:\n- Requirements array is EMPTY (expected 3-8)\n- Features array is EMPTY (expected 2-5)\n- prd.md file does NOT exist\n\nActor did NOT complete the workflow. Calling provide_feedback to block progression.\"\n\"#;\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 9.0,
      "lines_of_code": 132,
      "number_of_classes": 0,
      "number_of_functions": 2
    },
    "dependencies": [],
    "detailed_description": "This component implements a sophisticated PRD management system using an Actor-Critic pattern with Human-in-the-Loop validation. The PRD Actor is responsible for creating requirements and features from project ideas, incorporating user feedback through a mandatory review step, and persisting the final PRD document. The PRD Critic serves as a gatekeeper that rigorously verifies the Actor's work, ensuring all steps were completed correctly and data integrity is maintained. The system enforces a strict workflow with mandatory steps and validation checks.",
    "interfaces": [],
    "responsibilities": [
      "Orchestrate PRD creation workflow with mandatory Human-in-the-Loop validation",
      "Validate PRD data integrity and completeness through rigorous verification",
      "Provide structured requirements and features management with clear acceptance criteria",
      "Ensure data persistence through both structured storage and markdown documentation",
      "Enforce workflow compliance through strict gatekeeping mechanisms"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": "Intelligent agent responsible for generating comprehensive change reports for incremental modifications in a software project",
      "file_path": "crates/cowork-core/src/instructions/modify_delivery.rs",
      "functions": [
        "generate_change_report",
        "summarize_changes",
        "save_delivery_report"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "MODIFY_DELIVERY_INSTRUCTION"
      ],
      "name": "modify_delivery.rs",
      "source_summary": "// Modify Delivery Agent Instruction\n//\n// Role: Generate change report for incremental modifications\n\npub const MODIFY_DELIVERY_INSTRUCTION: &str = r#\"\n# Role: Modify Delivery Agent\n\nYou are a **Modify Delivery Agent** responsible for generating a comprehensive change report after incremental modifications.\n\n## Your Task\n\nGiven:\n1. **ChangeRequest**: What was requested\n2. **Patch Metadata**: What files were actually changed\n3. **Base Session**: Original project state\n4. **Current Session**: Updated project state\n\nYou need to:\n1. **Summarize changes** - What was added/modified/deleted\n2. **Generate change report** - Document the modifications\n3. **Save the report** as delivery_report.md\n\n## Available Tools\n\nYou have access to:\n- `get_requirements` - Load requirements (if updated)\n- `get_design` - Load design spec (if updated)\n- `get_plan` - Load plan (if updated)\n- `list_files` - See current files\n- `read_file` - Read modified files\n- `load_feedback_history` - Load any feedback during implementation\n- `save_delivery_report` - Save the final change report\n\n## Change Report Format\n\nYour change report should be structured like a **Pull Request description**:\n\n```markdown\n# Change Report: [Brief Title]\n\n## Summary\nBrief description of what changed.\n\n## Change Details\n\n### User Request\n[Original user's change request]\n\n### Implementation\n- **Files Added**: List of new files\n- **Files Modified**: List of modified files  \n- **Files Deleted**: List of deleted files\n\n### Scope Analysis\n- PRD Updated: Yes/No\n- Design Updated: Yes/No\n- Plan Updated: Yes/No\n- Code Changed: Yes/No\n\n## Changes Made\n\n### [Component/Feature Name]\n- What was added\n- What was modified\n- Why it was changed\n\n## Testing\n- Build status: ✅ / ❌\n- Tests run: Yes/No\n- Manual testing needed: [Instructions if any]\n\n## Notes\n- Any important considerations\n- Breaking changes (if any)\n- Next steps (if any)\n\n## Session Info\n- Base Session: session-xxx\n- Current Session: session-yyy\n- Timestamp: [date]\n```\n\n## Guidelines\n\n- **Clear and concise**: Focus on what actually changed\n- **Developer-friendly**: Write for someone reviewing the changes\n- **Include context**: Explain why changes were made\n- **Highlight risks**: Mention any breaking changes or concerns\n- **Actionable**: Include testing instructions if needed\n\n## Example Workflow\n\n1. Load ChangeRequest to see what was requested\n2. Read Patch Metadata to see what files changed\n3. Read modified files to understand the actual changes\n4. Load feedback history to see if there were issues\n5. Generate comprehensive change report\n6. Save using `save_delivery_report`\n\nRemember: This is a **change report**, not a full project delivery report. Focus on the incremental modifications.\n\"#;\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 12.0,
      "lines_of_code": 101,
      "number_of_classes": 0,
      "number_of_functions": 3
    },
    "dependencies": [],
    "detailed_description": "This component is an intelligent agent that serves as a Modify Delivery Agent instruction. Its primary function is to generate comprehensive change reports after incremental modifications to a codebase. The agent takes inputs including change requests, patch metadata, base session state, and current session state, then produces structured change reports in a pull request-like format. It follows a systematic workflow: loading requirements and design specs, analyzing file changes, reading modified files, incorporating feedback history, and generating detailed documentation of modifications including added/modified/deleted files, scope analysis, implementation details, testing status, and session information.",
    "interfaces": [
      {
        "description": "The main instruction string defining the agent's behavior and workflow",
        "interface_type": "constant",
        "name": "MODIFY_DELIVERY_INSTRUCTION",
        "parameters": [],
        "return_type": "string",
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Generate comprehensive change reports for incremental code modifications",
      "Analyze and document file-level changes (added/modified/deleted)",
      "Provide scope analysis covering PRD, design, plan, and code changes",
      "Structure change reports in developer-friendly pull request format",
      "Ensure change documentation includes testing instructions and risk assessment"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "entry",
      "description": "Main pipeline orchestrator for Cowork Forge workflow system",
      "file_path": "crates/cowork-core/src/pipeline/mod.rs",
      "functions": [
        "create_cowork_pipeline",
        "create_resume_pipeline",
        "create_partial_pipeline",
        "create_modify_pipeline",
        "create_change_triage_agent",
        "create_code_patch_agent",
        "create_modify_delivery_agent"
      ],
      "importance_score": 0.8,
      "interfaces": [],
      "name": "mod.rs",
      "source_summary": "// Main pipeline - Cowork Forge workflow\n\nuse crate::agents::*;\nuse crate::llm::*;\nuse adk_agent::SequentialAgent;\nuse adk_core::Agent;\nuse anyhow::Result;\nuse std::sync::Arc;\n\n/// Create the main Cowork Forge pipeline for new projects\n/// \n/// This assembles all agents into a sequential workflow:\n/// 1. IdeaAgent - Capture user's idea\n/// 2. PRD Loop - Requirements + Features (Actor-Critic)\n/// 3. Design Loop - Architecture (Actor-Critic)\n/// 4. Plan Loop - Implementation plan (Actor-Critic)\n/// 5. Coding Loop - Code implementation (Actor-Critic)\n/// 6. Check Agent - Quality assurance\n/// 7. Delivery Agent - Final report\npub fn create_cowork_pipeline(config: &ModelConfig, session_id: &str) -> Result<Arc<dyn Agent>> {\n    // Create LLM client\n    let llm = create_llm_client(&config.llm)?;\n\n    // Create all agents with session context\n    let idea_agent = create_idea_agent(llm.clone(), session_id)?;\n    let prd_loop = create_prd_loop(llm.clone(), session_id)?;\n    let design_loop = create_design_loop(llm.clone(), session_id)?;\n    let plan_loop = create_plan_loop(llm.clone(), session_id)?;\n    let coding_loop = create_coding_loop(llm.clone(), session_id)?;\n    let check_agent = create_check_agent(llm.clone(), session_id)?;\n    let delivery_agent = create_delivery_agent(llm, session_id)?;\n\n    // Assemble into SequentialAgent\n    let pipeline = SequentialAgent::new(\n        \"cowork_forge_pipeline\",\n        vec![\n            idea_agent,\n            prd_loop as Arc<dyn Agent>,\n            design_loop as Arc<dyn Agent>,\n            plan_loop as Arc<dyn Agent>,\n            coding_loop as Arc<dyn Agent>,\n            check_agent,\n            delivery_agent,\n        ],\n    );\n\n    Ok(Arc::new(pipeline))\n}\n\n/// Create a resume pipeline (skip Idea stage and completed stages)\n/// \n/// This function determines which stage to resume from by checking\n/// what data files exist in the base session\npub fn create_resume_pipeline(\n    config: &ModelConfig,\n    session_id: &str,\n    base_session_id: &str,\n) -> Result<Arc<dyn Agent>> {\n    use crate::storage::*;\n    \n    let _llm = create_llm_client(&config.llm)?;\n\n    // Determine which stage to start from based on existing data files in base session\n    // NOTE: load_* returns default empty structs when files don't exist, so we must check file existence.\n    let start_stage = if has_implementation_plan(base_session_id)?\n        && has_design_spec(base_session_id)?\n        && has_requirements(base_session_id)?\n    {\n        // PRD, Design, Plan exist → Resume from Coding\n        \"coding\"\n    } else if has_design_spec(base_session_id)? && has_requirements(base_session_id)? {\n        // PRD, Design exist → Resume from Plan\n        \"plan\"\n    } else if has_requirements(base_session_id)? {\n        // PRD exists → Resume from Design\n        \"design\"\n    } else {\n        // Nothing exists or only idea.md → Start from PRD\n        \"prd\"\n    };\n\n    println!(\"📍 Resuming from: {} stage\", start_stage);\n\n    // Use create_partial_pipeline to start from the determined stage\n    create_partial_pipeline(config, session_id, base_session_id, start_stage)\n}\n\n/// Create a partial pipeline starting from a specific stage (for revert)\n/// \n/// Useful for:\n/// - Modifying requirements (start from PRD)\n/// - Redesigning architecture (start from Design)\n/// - Replanning (start from Plan)\n/// - Recoding (start from Coding)\npub fn create_partial_pipeline(\n    config: &ModelConfig,\n    session_id: &str,\n    base_session_id: &str,\n    start_stage: &str,\n) -> Result<Arc<dyn Agent>> {\n    let llm = create_llm_client(&config.llm)?;\n\n    let agents: Vec<Arc<dyn Agent>> = match start_stage {\n        \"prd\" => {\n            vec![\n                create_prd_loop(llm.clone(), session_id)? as Arc<dyn Agent>,\n                create_design_loop(llm.clone(), session_id)? as Arc<dyn Agent>,\n                create_plan_loop(llm.clone(), session_id)? as Arc<dyn Agent>,\n                create_coding_loop(llm.clone(), session_id)? as Arc<dyn Agent>,\n                create_check_agent(llm.clone(), session_id)?,\n                create_delivery_agent(llm, session_id)?,\n            ]\n        }\n        \"design\" => {\n            vec![\n                create_design_loop(llm.clone(), session_id)? as Arc<dyn Agent>,\n                create_plan_loop(llm.clone(), session_id)? as Arc<dyn Agent>,\n                create_coding_loop(llm.clone(), session_id)? as Arc<dyn Agent>,\n                create_check_agent(llm.clone(), session_id)?,\n                create_delivery_agent(llm, session_id)?,\n            ]\n        }\n        \"plan\" => {\n            vec![\n                create_plan_loop(llm.clone(), session_id)? as Arc<dyn Agent>,\n                create_coding_loop(llm.clone(), session_id)? as Arc<dyn Agent>,\n                create_check_agent(llm.clone(), session_id)?,\n                create_delivery_agent(llm, session_id)?,\n            ]\n        }\n        \"coding\" => {\n            vec![\n                create_coding_loop(llm.clone(), session_id)? as Arc<dyn Agent>,\n                create_check_agent(llm.clone(), session_id)?,\n                create_delivery_agent(llm, session_id)?,\n            ]\n        }\n        \"check\" => {\n            vec![\n                create_check_agent(llm.clone(), session_id)?,\n                create_delivery_agent(llm, session_id)?,\n            ]\n        }\n        \"delivery\" => {\n            vec![create_delivery_agent(llm, session_id)?]\n        }\n        _ => {\n            anyhow::bail!(\"Unknown stage: {}. Valid stages: prd, design, plan, coding, check, delivery\", start_stage)\n        }\n    };\n\n    let pipeline = SequentialAgent::new(\n        format!(\"cowork_partial_pipeline_{}\", start_stage),\n        agents,\n    );\n\n    Ok(Arc::new(pipeline))\n}\n\n/// Create a modify pipeline for incremental changes\n/// \n/// This is a new pipeline designed for incremental updates:\n/// 1. Change Triage - Analyze the change and determine scope\n/// 2. Artifact Patch - Update affected artifacts (PRD/Design/Plan as needed)\n/// 3. Code Patch - Generate code changes (patches, not full rewrite)\n/// 4. Check - Verify changes\n/// 5. Delivery - Generate change report\npub fn create_modify_pipeline(\n    config: &ModelConfig,\n    session_id: &str,\n    base_session_id: &str,\n) -> Result<Arc<dyn Agent>> {\n    let llm = create_llm_client(&config.llm)?;\n\n    // For now, create a simplified modify pipeline\n    // TODO: Implement specialized change triage and patch agents\n    let agents: Vec<Arc<dyn Agent>> = vec![\n        create_change_triage_agent(llm.clone(), session_id, base_session_id)?,\n        create_code_patch_agent(llm.clone(), session_id, base_session_id)?,\n        create_check_agent(llm.clone(), session_id)?,\n        create_modify_delivery_agent(llm, session_id, base_session_id)?,\n    ];\n\n    let pipeline = SequentialAgent::new(\n        format!(\"cowork_modify_pipeline_{}\", session_id),\n        agents,\n    );\n\n    Ok(Arc::new(pipeline))\n}\n\n// Placeholder for new modify-specific agents\n// These are now implemented below\nfn create_change_triage_agent(\n    llm: Arc<dyn adk_core::Llm>,\n    session_id: &str,\n    base_session_id: &str,\n) -> Result<Arc<dyn Agent>> {\n    use crate::instructions::CHANGE_TRIAGE_INSTRUCTION;\n    use crate::tools::*;\n    use adk_agent::LlmAgentBuilder;\n    use adk_core::IncludeContents;\n    \n    let session = session_id.to_string();\n    \n    let agent = LlmAgentBuilder::new(\"change_triage_agent\")\n        .instruction(CHANGE_TRIAGE_INSTRUCTION)\n        .model(llm)\n        .tool(Arc::new(GetRequirementsTool::new(base_session_id.to_string())))\n        .tool(Arc::new(GetDesignTool::new(base_session_id.to_string())))\n        .tool(Arc::new(GetPlanTool::new(base_session_id.to_string())))\n        .tool(Arc::new(ListFilesTool))\n        .tool(Arc::new(ReadFileTool))\n        .tool(Arc::new(LoadChangeRequestTool::new(session.clone())))\n        .tool(Arc::new(SaveChangeRequestTool::new(session.clone())))\n        .tool(Arc::new(ProvideFeedbackTool::new(session.clone())))\n        .include_contents(IncludeContents::None)\n        .build()?;\n    \n    Ok(Arc::new(agent))\n}\n\nfn create_code_patch_agent(\n    llm: Arc<dyn adk_core::Llm>,\n    session_id: &str,\n    _base_session_id: &str,\n) -> Result<Arc<dyn Agent>> {\n    use crate::instructions::CODE_PATCH_INSTRUCTION;\n    use crate::tools::*;\n    use adk_agent::LlmAgentBuilder;\n    use adk_core::IncludeContents;\n    \n    let session = session_id.to_string();\n    \n    let agent = LlmAgentBuilder::new(\"code_patch_agent\")\n        .instruction(CODE_PATCH_INSTRUCTION)\n        .model(llm)\n        .tool(Arc::new(LoadChangeRequestTool::new(session.clone())))\n        .tool(Arc::new(GetPlanTool::new(session.clone())))\n        .tool(Arc::new(GetDesignTool::new(session.clone())))\n        .tool(Arc::new(ListFilesTool))\n        .tool(Arc::new(ReadFileTool))\n        .tool(Arc::new(WriteFileTool))\n        .tool(Arc::new(RunCommandTool))\n        .tool(Arc::new(UpdateTaskStatusTool::new(session.clone())))\n        .tool(Arc::new(UpdateFeatureStatusTool::new(session.clone())))\n        .include_contents(IncludeContents::None)\n        .build()?;\n    \n    Ok(Arc::new(agent))\n}\n\nfn create_modify_delivery_agent(\n    llm: Arc<dyn adk_core::Llm>,\n    session_id: &str,\n    _base_session_id: &str,\n) -> Result<Arc<dyn Agent>> {\n    use crate::instructions::MODIFY_DELIVERY_INSTRUCTION;\n    use crate::tools::*;\n    use adk_agent::LlmAgentBuilder;\n    use adk_core::IncludeContents;\n    \n    let session = session_id.to_string();\n    \n    let agent = LlmAgentBuilder::new(\"modify_delivery_agent\")\n        .instruction(MODIFY_DELIVERY_INSTRUCTION)\n        .model(llm)\n        .tool(Arc::new(LoadChangeRequestTool::new(session.clone())))\n        .tool(Arc::new(GetRequirementsTool::new(session.clone())))\n        .tool(Arc::new(GetDesignTool::new(session.clone())))\n        .tool(Arc::new(GetPlanTool::new(session.clone())))\n        .tool(Arc::new(ListFilesTool))\n        .tool(Arc::new(ReadFileTool))\n        .tool(Arc::new(LoadFeedbackHistoryTool::new(session.clone())))\n        .tool(Arc::new(SaveDeliveryReportTool::new(session.clone())))\n        .include_contents(IncludeContents::None)\n        .build()?;\n    \n    Ok(Arc::new(agent))\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn test_config_loading() {\n        // Test that we can create a config\n        let config = ModelConfig {\n            llm: LlmConfig {\n                api_base_url: \"http://localhost:8000/v1\".to_string(),\n                api_key: \"test-key\".to_string(),\n                model_name: \"gpt-4\".to_string(),\n            },\n        };\n\n        assert_eq!(config.llm.model_name, \"gpt-4\");\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 10.0,
      "lines_of_code": 299,
      "number_of_classes": 0,
      "number_of_functions": 7
    },
    "dependencies": [
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": 3,
        "name": "crate::agents::*",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": 4,
        "name": "crate::llm::*",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "struct",
        "is_external": true,
        "line_number": 5,
        "name": "adk_agent::SequentialAgent",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "trait",
        "is_external": true,
        "line_number": 6,
        "name": "adk_core::Agent",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "type",
        "is_external": true,
        "line_number": 7,
        "name": "anyhow::Result",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "type",
        "is_external": true,
        "line_number": 8,
        "name": "std::sync::Arc",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": 49,
        "name": "crate::storage::*",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "constant",
        "is_external": false,
        "line_number": 138,
        "name": "crate::instructions::CHANGE_TRIAGE_INSTRUCTION",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": 139,
        "name": "crate::tools::*",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "struct",
        "is_external": true,
        "line_number": 140,
        "name": "adk_agent::LlmAgentBuilder",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "enum",
        "is_external": true,
        "line_number": 141,
        "name": "adk_core::IncludeContents",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "trait",
        "is_external": true,
        "line_number": 133,
        "name": "adk_core::Llm",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "struct",
        "is_external": false,
        "line_number": 146,
        "name": "GetRequirementsTool",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "struct",
        "is_external": false,
        "line_number": 147,
        "name": "GetDesignTool",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "struct",
        "is_external": false,
        "line_number": 148,
        "name": "GetPlanTool",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "struct",
        "is_external": false,
        "line_number": 149,
        "name": "ListFilesTool",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "struct",
        "is_external": false,
        "line_number": 150,
        "name": "ReadFileTool",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "struct",
        "is_external": false,
        "line_number": 151,
        "name": "LoadChangeRequestTool",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "struct",
        "is_external": false,
        "line_number": 152,
        "name": "SaveChangeRequestTool",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "struct",
        "is_external": false,
        "line_number": 153,
        "name": "ProvideFeedbackTool",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "This component serves as the central orchestrator for the Cowork Forge system, responsible for creating and managing different types of workflow pipelines. It implements a comprehensive workflow management system that can handle new project creation, resuming interrupted workflows, partial workflows for specific stages, and modification pipelines for incremental changes. The component demonstrates sophisticated pipeline composition logic with intelligent stage detection and conditional workflow assembly.",
    "interfaces": [
      {
        "description": "Creates the main workflow pipeline with all stages",
        "interface_type": "function",
        "name": "create_cowork_pipeline",
        "parameters": [
          {
            "description": "Configuration for LLM models",
            "is_optional": false,
            "name": "config",
            "param_type": "&ModelConfig"
          },
          {
            "description": "Unique session identifier",
            "is_optional": false,
            "name": "session_id",
            "param_type": "&str"
          }
        ],
        "return_type": "Result<Arc<dyn Agent>>",
        "visibility": "public"
      },
      {
        "description": "Creates a pipeline that resumes from appropriate stage based on existing artifacts",
        "interface_type": "function",
        "name": "create_resume_pipeline",
        "parameters": [
          {
            "description": "Configuration for LLM models",
            "is_optional": false,
            "name": "config",
            "param_type": "&ModelConfig"
          },
          {
            "description": "Current session identifier",
            "is_optional": false,
            "name": "session_id",
            "param_type": "&str"
          },
          {
            "description": "Base session identifier for resume",
            "is_optional": false,
            "name": "base_session_id",
            "param_type": "&str"
          }
        ],
        "return_type": "Result<Arc<dyn Agent>>",
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Pipeline orchestration and workflow management",
      "Agent composition and sequential assembly",
      "Stage detection and resume logic",
      "Pipeline variant creation for different use cases",
      "Error handling and configuration management"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "tool",
      "description": "File operation tools with SECURITY constraints",
      "file_path": "crates/cowork-core/src/tools/file_tools.rs",
      "functions": [
        "validate_path_security",
        "should_ignore",
        "is_blocking_service_command"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "ListFilesTool",
        "ReadFileTool",
        "WriteFileTool",
        "RunCommandTool"
      ],
      "name": "file_tools.rs",
      "source_summary": "// File operation tools with SECURITY constraints\nuse adk_core::{Tool, ToolContext};\nuse async_trait::async_trait;\nuse serde_json::{json, Value};\nuse std::sync::Arc;\nuse std::fs;\nuse std::path::{Path, PathBuf};\nuse walkdir::WalkDir;\n\n// ============================================================================\n// Security Helper - Path Validation\n// ============================================================================\n\n/// Validate that a path is safe to access\n/// Rules:\n/// 1. Must be relative path (no absolute paths like /tmp, C:\\)\n/// 2. Must not escape current directory (no ..)\n/// 3. Must be within current working directory or .cowork\nfn validate_path_security(path: &str) -> Result<PathBuf, String> {\n    let path_obj = Path::new(path);\n    \n    // Rule 1: Reject absolute paths\n    if path_obj.is_absolute() {\n        return Err(format!(\n            \"Security: Absolute paths are not allowed. Path '{}' must be relative to current directory.\",\n            path\n        ));\n    }\n    \n    // Rule 2: Reject parent directory access (..)\n    if path.contains(\"..\") {\n        return Err(format!(\n            \"Security: Parent directory access (..) is not allowed. Path: '{}'\",\n            path\n        ));\n    }\n    \n    // Rule 3: Canonicalize and verify it's within current directory\n    let current_dir = std::env::current_dir()\n        .map_err(|e| format!(\"Failed to get current directory: {}\", e))?;\n    \n    let full_path = current_dir.join(path);\n    \n    // Canonicalize if path exists, otherwise just check the constructed path\n    let canonical_path = if full_path.exists() {\n        full_path.canonicalize()\n            .map_err(|e| format!(\"Failed to resolve path: {}\", e))?\n    } else {\n        // For non-existent paths (e.g., files to be created), just verify parent\n        full_path\n    };\n    \n    // Verify the path is within current directory\n    if !canonical_path.starts_with(&current_dir) {\n        return Err(format!(\n            \"Security: Path escapes current directory. Path '{}' resolves to '{}'\",\n            path,\n            canonical_path.display()\n        ));\n    }\n    \n    Ok(canonical_path)\n}\n\n// ============================================================================\n// ListFilesTool\n// ============================================================================\n\npub struct ListFilesTool;\n\n#[async_trait]\nimpl Tool for ListFilesTool {\n    fn name(&self) -> &str {\n        \"list_files\"\n    }\n\n    fn description(&self) -> &str {\n        \"List files in a directory (recursively or non-recursively). \\\n         SECURITY: Only works within current directory. \\\n         Useful for understanding project structure.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"path\": {\n                    \"type\": \"string\",\n                    \"description\": \"Directory path to list (default: current directory). Must be relative path.\"\n                },\n                \"recursive\": {\n                    \"type\": \"boolean\",\n                    \"description\": \"Whether to list files recursively (default: false)\"\n                },\n                \"max_depth\": {\n                    \"type\": \"integer\",\n                    \"description\": \"Maximum depth for recursive listing (default: 3)\"\n                }\n            }\n        }))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {\n        let path = args.get(\"path\")\n            .and_then(|v| v.as_str())\n            .unwrap_or(\".\");\n        \n        // Security check\n        let safe_path = match validate_path_security(path) {\n            Ok(p) => p,\n            Err(e) => {\n                return Ok(json!({\n                    \"status\": \"security_error\",\n                    \"message\": e\n                }));\n            }\n        };\n        \n        let recursive = args.get(\"recursive\")\n            .and_then(|v| v.as_bool())\n            .unwrap_or(false);\n        \n        let max_depth = args.get(\"max_depth\")\n            .and_then(|v| v.as_u64())\n            .unwrap_or(3) as usize;\n\n        if !safe_path.exists() {\n            return Ok(json!({\n                \"status\": \"error\",\n                \"message\": format!(\"Path not found: {}\", path)\n            }));\n        }\n\n        let mut files = Vec::new();\n        let mut directories = Vec::new();\n\n        if recursive {\n            // Recursive listing with max depth\n            let cwd = std::env::current_dir()\n                .map_err(|e| adk_core::AdkError::Tool(format!(\"Failed to get current dir: {}\", e)))?;\n\n            for entry in WalkDir::new(&safe_path)\n                .max_depth(max_depth)\n                .follow_links(false)\n                .into_iter()\n                .filter_entry(|e| {\n                    // Prune hidden directories early (except the root itself)\n                    if let Some(name) = e.file_name().to_str() {\n                        if name.starts_with('.') && name != \".\" {\n                            return false;\n                        }\n                    }\n                    true\n                })\n                .filter_map(|e| e.ok())\n            {\n                // Convert to relative path for stable ignore matching\n                let rel = entry.path().strip_prefix(&cwd).unwrap_or(entry.path());\n                let rel_str = rel.to_string_lossy();\n                let path_str = format!(\"./{}\", rel_str.trim_start_matches(\"./\"));\n\n                // Skip hidden files and common ignore patterns\n                if should_ignore(&path_str) {\n                    continue;\n                }\n\n                if entry.file_type().is_dir() {\n                    directories.push(path_str);\n                } else {\n                    files.push(path_str);\n                }\n            }\n        } else {\n            // Non-recursive listing\n            let cwd = std::env::current_dir()\n                .map_err(|e| adk_core::AdkError::Tool(format!(\"Failed to get current dir: {}\", e)))?;\n\n            let entries = fs::read_dir(&safe_path)\n                .map_err(|e| adk_core::AdkError::Tool(format!(\"Failed to read directory: {}\", e)))?;\n\n            for entry in entries {\n                let entry = entry.map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;\n\n                // Skip hidden at top-level\n                if let Some(name) = entry.file_name().to_str() {\n                    if name.starts_with('.') {\n                        continue;\n                    }\n                }\n\n                let full = entry.path().to_path_buf();\n                let rel = full.strip_prefix(&cwd).unwrap_or(&full);\n                let rel_str = rel.to_string_lossy();\n                let path_str = format!(\"./{}\", rel_str.trim_start_matches(\"./\"));\n\n                if should_ignore(&path_str) {\n                    continue;\n                }\n\n                if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {\n                    directories.push(path_str);\n                } else {\n                    files.push(path_str);\n                }\n            }\n        }\n\n        Ok(json!({\n            \"status\": \"success\",\n            \"path\": path,\n            \"files\": files,\n            \"directories\": directories,\n            \"total_files\": files.len(),\n            \"total_directories\": directories.len()\n        }))\n    }\n}\n\nfn should_ignore(path: &str) -> bool {\n    // Normalize: we mostly work with \"./...\" relative paths now\n\n    // 1) Hide dotfiles / dot-directories broadly\n    // (We still keep root path \".\" out of this function; callers handle it)\n    if let Some(name) = Path::new(path).file_name().and_then(|n| n.to_str()) {\n        if name.starts_with('.') {\n            return true;\n        }\n    }\n\n    // 2) Common ignore patterns\n    let ignore_patterns = [\n        \"./.git\", \"./target\", \"./node_modules\", \"./.cowork\", \"./.litho\",\n        \"./.idea\", \"./.vscode\", \"./dist\", \"./build\", \"./docs\", \"./tests\",\n        \"./.archived\",\n        \".DS_Store\", \"Thumbs.db\",\n    ];\n\n    ignore_patterns.iter().any(|pattern| path.contains(pattern))\n}\n\n// ============================================================================\n// ReadFileTool\n// ============================================================================\n\npub struct ReadFileTool;\n\n#[async_trait]\nimpl Tool for ReadFileTool {\n    fn name(&self) -> &str {\n        \"read_file\"\n    }\n\n    fn description(&self) -> &str {\n        \"Read the contents of a file. \\\n         SECURITY: Only works within current directory.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"path\": {\n                    \"type\": \"string\",\n                    \"description\": \"File path to read (must be relative path within current directory)\"\n                }\n            },\n            \"required\": [\"path\"]\n        }))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {\n        let path = args[\"path\"].as_str().unwrap();\n\n        // Security check\n        let safe_path = match validate_path_security(path) {\n            Ok(p) => p,\n            Err(e) => {\n                return Ok(json!({\n                    \"status\": \"security_error\",\n                    \"message\": e\n                }));\n            }\n        };\n\n        if !safe_path.exists() {\n            return Ok(json!({\n                \"status\": \"error\",\n                \"message\": format!(\"File not found: {}\", path)\n            }));\n        }\n        \n        match fs::read_to_string(&safe_path) {\n            Ok(content) => Ok(json!({\n                \"status\": \"success\",\n                \"path\": path,\n                \"content\": content\n            })),\n            Err(e) => Ok(json!({\n                \"status\": \"error\",\n                \"message\": format!(\"Failed to read file: {}\", e)\n            })),\n        }\n    }\n}\n\n// ============================================================================\n// WriteFileTool\n// ============================================================================\n\npub struct WriteFileTool;\n\n#[async_trait]\nimpl Tool for WriteFileTool {\n    fn name(&self) -> &str {\n        \"write_file\"\n    }\n\n    fn description(&self) -> &str {\n        \"Write content to a file. Creates parent directories if needed. \\\n         SECURITY: Only works within current directory. Absolute paths and .. are forbidden.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"path\": {\n                    \"type\": \"string\",\n                    \"description\": \"File path to write (must be relative path within current directory)\"\n                },\n                \"content\": {\n                    \"type\": \"string\",\n                    \"description\": \"Content to write\"\n                }\n            },\n            \"required\": [\"path\", \"content\"]\n        }))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {\n        let path = args[\"path\"].as_str().unwrap();\n        let content = args[\"content\"].as_str().unwrap();\n\n        // Security check\n        let safe_path = match validate_path_security(path) {\n            Ok(p) => p,\n            Err(e) => {\n                return Ok(json!({\n                    \"status\": \"security_error\",\n                    \"message\": e\n                }));\n            }\n        };\n\n        // Create parent directories if needed\n        if let Some(parent) = safe_path.parent() {\n            fs::create_dir_all(parent).map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;\n        }\n\n        match fs::write(&safe_path, content) {\n            Ok(_) => {\n                // Log file creation for user visibility\n                println!(\"📝 Writing file: {} ({} lines)\", path, content.lines().count());\n                Ok(json!({\n                    \"status\": \"success\",\n                    \"path\": path,\n                    \"lines_written\": content.lines().count()\n                }))\n            },\n            Err(e) => Ok(json!({\n                \"status\": \"error\",\n                \"message\": format!(\"Failed to write file: {}\", e)\n            })),\n        }\n    }\n}\n\n// ============================================================================\n// RunCommandTool with blocking detection\n// ============================================================================\n\npub struct RunCommandTool;\n\n/// Detect if a command is a long-running service that would block execution\nfn is_blocking_service_command(command: &str) -> bool {\n    let blocking_patterns = vec![\n        \"http.server\",      // python -m http.server\n        \"npm run dev\",      // npm dev server\n        \"npm start\",        // npm start\n        \"yarn dev\",\n        \"yarn start\",\n        \"pnpm dev\",\n        \"pnpm start\",\n        \"uvicorn\",          // Python ASGI server\n        \"gunicorn\",         // Python WSGI server\n        \"flask run\",\n        \"django runserver\",\n        \"rails server\",\n        \"cargo run\",        // Might be a server\n        \"serve\",            // serve package\n        \"webpack-dev-server\",\n        \"vite\",\n        \"next dev\",\n    ];\n\n    blocking_patterns.iter().any(|pattern| command.contains(pattern))\n}\n\n#[async_trait]\nimpl Tool for RunCommandTool {\n    fn name(&self) -> &str {\n        \"run_command\"\n    }\n\n    fn description(&self) -> &str {\n        \"Execute a shell command and return the output. \\\n         WARNING: This tool will REJECT commands that start long-running services \\\n         (like http.server, npm dev, etc.) as they would block execution. \\\n         Use this for: building, testing, linting - NOT for starting servers.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"command\": {\n                    \"type\": \"string\",\n                    \"description\": \"Shell command to execute (must not be a blocking service command)\"\n                }\n            },\n            \"required\": [\"command\"]\n        }))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {\n        let command = args[\"command\"].as_str().unwrap();\n\n        // Check if command would block\n        if is_blocking_service_command(command) {\n            return Ok(json!({\n                \"status\": \"rejected\",\n                \"message\": format!(\n                    \"BLOCKED: This command appears to start a long-running service: '{}'. \\\n                     Starting services would block the agent. \\\n                     If you need to verify the code works, just create the files - don't start servers.\",\n                    command\n                )\n            }));\n        }\n\n        // Execute command with timeout\n        let output = tokio::time::timeout(\n            std::time::Duration::from_secs(30),\n            tokio::process::Command::new(\"sh\")\n                .arg(\"-c\")\n                .arg(command)\n                .current_dir(std::env::current_dir().unwrap()) // Run in current dir\n                .output()\n        )\n        .await;\n\n        match output {\n            Ok(Ok(output)) => {\n                let stdout = String::from_utf8_lossy(&output.stdout).to_string();\n                let stderr = String::from_utf8_lossy(&output.stderr).to_string();\n\n                Ok(json!({\n                    \"status\": if output.status.success() { \"success\" } else { \"failed\" },\n                    \"exit_code\": output.status.code(),\n                    \"stdout\": stdout,\n                    \"stderr\": stderr\n                }))\n            }\n            Ok(Err(e)) => {\n                Ok(json!({\n                    \"status\": \"error\",\n                    \"message\": format!(\"Failed to execute command: {}\", e)\n                }))\n            }\n            Err(_) => {\n                Ok(json!({\n                    \"status\": \"timeout\",\n                    \"message\": \"Command execution timeout (30s limit)\"\n                }))\n            }\n        }\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 43.0,
      "lines_of_code": 488,
      "number_of_classes": 4,
      "number_of_functions": 7
    },
    "dependencies": [
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 2,
        "name": "adk_core",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 3,
        "name": "async_trait",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 4,
        "name": "serde_json",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "standard_library",
        "is_external": false,
        "line_number": 5,
        "name": "std",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 7,
        "name": "walkdir",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": null,
        "name": "tokio",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "This component provides secure file operation tools for the cowork-core system. It implements four main tools: ListFilesTool (recursive/non-recursive directory listing with security constraints), ReadFileTool (secure file reading within current directory), WriteFileTool (secure file writing with parent directory creation), and RunCommandTool (safe command execution with blocking service detection). The component emphasizes security through path validation that prevents absolute paths, parent directory traversal, and directory escaping. It includes intelligent features like hidden file filtering, common ignore patterns, and blocking command detection to prevent long-running service execution that would block the agent.",
    "interfaces": [
      {
        "description": "List files in a directory (recursively or non-recursively) with security constraints",
        "interface_type": "struct",
        "name": "ListFilesTool",
        "parameters": [
          {
            "description": "Directory path to list (default: current directory). Must be relative path.",
            "is_optional": true,
            "name": "path",
            "param_type": "string"
          },
          {
            "description": "Whether to list files recursively (default: false)",
            "is_optional": true,
            "name": "recursive",
            "param_type": "boolean"
          },
          {
            "description": "Maximum depth for recursive listing (default: 3)",
            "is_optional": true,
            "name": "max_depth",
            "param_type": "integer"
          }
        ],
        "return_type": "Value",
        "visibility": "public"
      },
      {
        "description": "Read the contents of a file with security constraints",
        "interface_type": "struct",
        "name": "ReadFileTool",
        "parameters": [
          {
            "description": "File path to read (must be relative path within current directory)",
            "is_optional": false,
            "name": "path",
            "param_type": "string"
          }
        ],
        "return_type": "Value",
        "visibility": "public"
      },
      {
        "description": "Write content to a file with parent directory creation and security constraints",
        "interface_type": "struct",
        "name": "WriteFileTool",
        "parameters": [
          {
            "description": "File path to write (must be relative path within current directory)",
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
        "return_type": "Value",
        "visibility": "public"
      },
      {
        "description": "Execute a shell command with timeout protection and blocking service detection",
        "interface_type": "struct",
        "name": "RunCommandTool",
        "parameters": [
          {
            "description": "Shell command to execute (must not be a blocking service command)",
            "is_optional": false,
            "name": "command",
            "param_type": "string"
          }
        ],
        "return_type": "Value",
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Provide secure file system operations with enforced security constraints",
      "Implement directory listing with recursive/non-recursive options and intelligent filtering",
      "Enable safe file reading and writing within constrained directory boundaries",
      "Execute shell commands safely with timeout protection and blocking service detection",
      "Maintain security boundaries by preventing path traversal and unauthorized access"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "tool",
      "description": "Validation tools for checking data quality (Session-scoped)",
      "file_path": "crates/cowork-core/src/tools/validation_tools.rs",
      "functions": [
        "check_data_format",
        "check_feature_coverage",
        "check_task_dependencies",
        "validate_requirements_schema",
        "validate_features_schema",
        "validate_design_schema",
        "validate_plan_schema",
        "detect_cycle"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "CheckDataFormatTool",
        "CheckFeatureCoverageTool",
        "CheckTaskDependenciesTool"
      ],
      "name": "validation_tools.rs",
      "source_summary": "// Validation tools for checking data quality (Session-scoped)\nuse crate::storage::*;\nuse adk_core::{Tool, ToolContext};\n\nuse async_trait::async_trait;\nuse serde_json::{json, Value};\nuse std::sync::Arc;\n\n// ============================================================================\n// CheckDataFormatTool\n// ============================================================================\n\npub struct CheckDataFormatTool {\n    session_id: String,\n}\n\nimpl CheckDataFormatTool {\n    pub fn new(session_id: String) -> Self {\n        Self { session_id }\n    }\n}\n\n#[async_trait]\nimpl Tool for CheckDataFormatTool {\n    fn name(&self) -> &str {\n        \"check_data_format\"\n    }\n\n    fn description(&self) -> &str {\n        \"Validate that a JSON data file conforms to its schema. Returns validation errors if any.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"data_type\": {\n                    \"type\": \"string\",\n                    \"enum\": [\"requirements\", \"features\", \"design\", \"plan\"],\n                    \"description\": \"Which data file to validate\"\n                }\n            },\n            \"required\": [\"data_type\"]\n        }))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {\n        let data_type = args[\"data_type\"].as_str().unwrap();\n\n        let errors = match data_type {\n            \"requirements\" => self.validate_requirements_schema(),\n            \"features\" => self.validate_features_schema(),\n            \"design\" => self.validate_design_schema(),\n            \"plan\" => self.validate_plan_schema(),\n            _ => return Ok(json!({\"status\": \"error\", \"message\": \"Unknown data type\"})),\n        };\n\n        if errors.is_empty() {\n            Ok(json!({\n                \"status\": \"valid\",\n                \"message\": format!(\"{} data is valid\", data_type)\n            }))\n        } else {\n            Ok(json!({\n                \"status\": \"invalid\",\n                \"errors\": errors\n            }))\n        }\n    }\n}\n\nimpl CheckDataFormatTool {\n    fn validate_requirements_schema(&self) -> Vec<String> {\n        let mut errors = vec![];\n        match load_requirements(&self.session_id) {\n            Ok(requirements) => {\n                for req in &requirements.requirements {\n                    if req.title.is_empty() {\n                        errors.push(format!(\"{}: title is empty\", req.id));\n                    }\n                    if req.acceptance_criteria.is_empty() {\n                        errors.push(format!(\"{}: missing acceptance criteria\", req.id));\n                    }\n                }\n            }\n            Err(e) => errors.push(format!(\"Failed to load requirements: {}\", e)),\n        }\n        errors\n    }\n\n    fn validate_features_schema(&self) -> Vec<String> {\n        let mut errors = vec![];\n        match load_feature_list(&self.session_id) {\n            Ok(features) => {\n                for feat in &features.features {\n                    if feat.name.is_empty() {\n                        errors.push(format!(\"{}: name is empty\", feat.id));\n                    }\n                    if feat.requirement_ids.is_empty() {\n                        errors.push(format!(\"{}: not linked to any requirement\", feat.id));\n                    }\n                }\n            }\n            Err(e) => errors.push(format!(\"Failed to load features: {}\", e)),\n        }\n        errors\n    }\n\n    fn validate_design_schema(&self) -> Vec<String> {\n        let mut errors = vec![];\n        match load_design_spec(&self.session_id) {\n            Ok(design) => {\n                if design.architecture.components.is_empty() {\n                    errors.push(\"No components defined\".to_string());\n                }\n            }\n            Err(e) => errors.push(format!(\"Failed to load design: {}\", e)),\n        }\n        errors\n    }\n\n    fn validate_plan_schema(&self) -> Vec<String> {\n        let mut errors = vec![];\n        match load_implementation_plan(&self.session_id) {\n            Ok(plan) => {\n                if plan.tasks.is_empty() {\n                    errors.push(\"No tasks defined\".to_string());\n                }\n            }\n            Err(e) => errors.push(format!(\"Failed to load plan: {}\", e)),\n        }\n        errors\n    }\n}\n\n// ============================================================================\n// CheckFeatureCoverageTool\n// ============================================================================\n\npub struct CheckFeatureCoverageTool {\n    session_id: String,\n}\n\nimpl CheckFeatureCoverageTool {\n    pub fn new(session_id: String) -> Self {\n        Self { session_id }\n    }\n}\n\n#[async_trait]\nimpl Tool for CheckFeatureCoverageTool {\n    fn name(&self) -> &str {\n        \"check_feature_coverage\"\n    }\n\n    fn description(&self) -> &str {\n        \"Check if all features are covered by design components.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\"type\": \"object\", \"properties\": {}}))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> adk_core::Result<Value> {\n        let features = load_feature_list(&self.session_id).map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;\n        let design = load_design_spec(&self.session_id).map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;\n\n        let uncovered: Vec<String> = features\n            .features\n            .iter()\n            .filter(|f| {\n                !design\n                    .architecture\n                    .components\n                    .iter()\n                    .any(|c| c.related_features.contains(&f.id))\n            })\n            .map(|f| f.id.clone())\n            .collect();\n\n        if uncovered.is_empty() {\n            Ok(json!({\n                \"status\": \"full_coverage\",\n                \"message\": \"All features are covered by components\"\n            }))\n        } else {\n            Ok(json!({\n                \"status\": \"incomplete_coverage\",\n                \"uncovered_features\": uncovered,\n                \"message\": format!(\"{} features are not covered\", uncovered.len())\n            }))\n        }\n    }\n}\n\n// ============================================================================\n// CheckTaskDependenciesTool\n// ============================================================================\n\npub struct CheckTaskDependenciesTool {\n    session_id: String,\n}\n\nimpl CheckTaskDependenciesTool {\n    pub fn new(session_id: String) -> Self {\n        Self { session_id }\n    }\n}\n\n#[async_trait]\nimpl Tool for CheckTaskDependenciesTool {\n    fn name(&self) -> &str {\n        \"check_task_dependencies\"\n    }\n\n    fn description(&self) -> &str {\n        \"Analyze task dependencies to detect circular dependencies.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\"type\": \"object\", \"properties\": {}}))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> adk_core::Result<Value> {\n        let plan = load_implementation_plan(&self.session_id).map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;\n\n        // Build dependency graph\n        let mut graph: std::collections::HashMap<String, Vec<String>> =\n            std::collections::HashMap::new();\n        for task in &plan.tasks {\n            graph.insert(task.id.clone(), task.dependencies.clone());\n        }\n\n        // Detect cycles using DFS\n        let has_cycles = detect_cycle(&graph);\n\n        if has_cycles {\n            Ok(json!({\n                \"status\": \"invalid\",\n                \"message\": \"Circular dependencies detected in task graph\"\n            }))\n        } else {\n            Ok(json!({\n                \"status\": \"valid\",\n                \"message\": \"No circular dependencies detected\"\n            }))\n        }\n    }\n}\n\n/// Detect cycles in dependency graph using DFS\nfn detect_cycle(graph: &std::collections::HashMap<String, Vec<String>>) -> bool {\n    use std::collections::HashSet;\n\n    let mut visited = HashSet::new();\n    let mut rec_stack = HashSet::new();\n\n    fn dfs(\n        node: &str,\n        graph: &std::collections::HashMap<String, Vec<String>>,\n        visited: &mut HashSet<String>,\n        rec_stack: &mut HashSet<String>,\n    ) -> bool {\n        visited.insert(node.to_string());\n        rec_stack.insert(node.to_string());\n\n        if let Some(neighbors) = graph.get(node) {\n            for neighbor in neighbors {\n                if !visited.contains(neighbor) {\n                    if dfs(neighbor, graph, visited, rec_stack) {\n                        return true;\n                    }\n                } else if rec_stack.contains(neighbor) {\n                    return true; // Cycle detected\n                }\n            }\n        }\n\n        rec_stack.remove(node);\n        false\n    }\n\n    for node in graph.keys() {\n        if !visited.contains(node) {\n            if dfs(node, graph, &mut visited, &mut rec_stack) {\n                return true;\n            }\n        }\n    }\n\n    false\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 32.0,
      "lines_of_code": 292,
      "number_of_classes": 3,
      "number_of_functions": 8
    },
    "dependencies": [
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": 2,
        "name": "crate::storage",
        "path": "crates/cowork-core/src/storage",
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 3,
        "name": "adk_core",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 5,
        "name": "async_trait",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 6,
        "name": "serde_json",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "standard_library",
        "is_external": false,
        "line_number": 7,
        "name": "std::sync::Arc",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "standard_library",
        "is_external": false,
        "line_number": 139,
        "name": "std::collections",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "This component provides a suite of validation tools for software development data quality assurance. It contains three main tools: 1) CheckDataFormatTool - validates JSON data files against schema requirements for requirements, features, design, and plan data types; 2) CheckFeatureCoverageTool - ensures all features are covered by design components; 3) CheckTaskDependenciesTool - analyzes task dependencies to detect circular dependencies using DFS algorithm. The tools operate within session scope and provide comprehensive validation for software development artifacts.",
    "interfaces": [
      {
        "description": "Tool for validating JSON data files against schema requirements",
        "interface_type": "struct",
        "name": "CheckDataFormatTool",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "session_id",
            "param_type": "String"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Tool for checking feature coverage by design components",
        "interface_type": "struct",
        "name": "CheckFeatureCoverageTool",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "session_id",
            "param_type": "String"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Tool for detecting circular dependencies in task graphs",
        "interface_type": "struct",
        "name": "CheckTaskDependenciesTool",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "session_id",
            "param_type": "String"
          }
        ],
        "return_type": null,
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Data format validation for software development artifacts",
      "Feature-to-component coverage analysis",
      "Task dependency graph cycle detection",
      "Session-scoped data quality assurance",
      "Error reporting and validation feedback"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "tool",
      "description": "HITL (Human-in-the-Loop) content review and editing tools for markdown content interaction",
      "file_path": "crates/cowork-core/src/tools/hitl_content_tools.rs",
      "functions": [
        "review_and_edit_content",
        "review_with_feedback_content"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "ReviewAndEditContentTool",
        "ReviewWithFeedbackContentTool"
      ],
      "name": "hitl_content_tools.rs",
      "source_summary": "// HITL tools (content-based) to avoid hardcoding artifact file paths\nuse adk_core::{Tool, ToolContext};\nuse async_trait::async_trait;\nuse dialoguer::{Editor, Input};\nuse serde_json::{json, Value};\nuse std::sync::Arc;\n\n/// review_and_edit_content\n/// - Takes content as input\n/// - Optionally lets user edit in editor\n/// - Returns edited content\npub struct ReviewAndEditContentTool;\n\n#[async_trait]\nimpl Tool for ReviewAndEditContentTool {\n    fn name(&self) -> &str {\n        \"review_and_edit_content\"\n    }\n\n    fn description(&self) -> &str {\n        \"Let the user review and optionally edit markdown content using their default editor. Returns edited content.\" \n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"title\": {\"type\": \"string\", \"description\": \"Title shown to user\"},\n                \"content\": {\"type\": \"string\", \"description\": \"Content to review/edit\"}\n            },\n            \"required\": [\"title\", \"content\"]\n        }))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {\n        let title = args[\"title\"].as_str().unwrap();\n        let content = args[\"content\"].as_str().unwrap();\n\n        println!(\"\\n📝 {}\", title);\n        println!(\"  ────────────────────────────────────────\");\n        for (i, line) in content.lines().take(12).enumerate() {\n            println!(\"  {}: {}\", i + 1, line);\n        }\n        let line_count = content.lines().count();\n        if line_count > 12 {\n            println!(\"  ... ({} more lines)\", line_count - 12);\n        }\n        println!(\"  ────────────────────────────────────────\\n\");\n\n        let input: String = Input::new()\n            .with_prompt(\"输入 'edit' 打开编辑器，或直接回车跳过\")\n            .allow_empty(true)\n            .interact_text()\n            .map_err(|e| adk_core::AdkError::Tool(format!(\"Interaction error: {}\", e)))?;\n\n        if input.trim().to_lowercase() != \"edit\" {\n            return Ok(json!({\n                \"action\": \"pass\",\n                \"content\": content,\n                \"message\": \"User skipped editing\"\n            }));\n        }\n\n        println!(\"📝 Opening editor... (Save and close to submit changes)\");\n        let edited = Editor::new()\n            .require_save(true)\n            .edit(content)\n            .map_err(|e| adk_core::AdkError::Tool(format!(\"Editor error: {}\", e)))?;\n\n        let new_content = edited.unwrap_or_else(|| content.to_string());\n\n        Ok(json!({\n            \"action\": \"edit\",\n            \"content\": new_content,\n            \"message\": \"Content edited\"\n        }))\n    }\n}\n\n/// review_with_feedback_content\n/// - Takes content as input\n/// - Allows edit/pass/feedback\n/// - Returns edited content OR feedback text\npub struct ReviewWithFeedbackContentTool;\n\n#[async_trait]\nimpl Tool for ReviewWithFeedbackContentTool {\n    fn name(&self) -> &str {\n        \"review_with_feedback_content\"\n    }\n\n    fn description(&self) -> &str {\n        \"Review content and allow user to: edit in editor, pass, or provide feedback text.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"title\": {\"type\": \"string\"},\n                \"content\": {\"type\": \"string\"},\n                \"prompt\": {\"type\": \"string\"}\n            },\n            \"required\": [\"title\", \"content\"]\n        }))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {\n        let title = args[\"title\"].as_str().unwrap();\n        let content = args[\"content\"].as_str().unwrap();\n        let default_prompt = \"输入 'edit' 编辑，'pass' 继续，或直接输入修改建议\";\n        let prompt = args.get(\"prompt\").and_then(|v| v.as_str()).unwrap_or(default_prompt);\n\n        println!(\"\\n📝 {}\", title);\n        println!(\"  ────────────────────────────────────────\");\n        for (i, line) in content.lines().take(15).enumerate() {\n            println!(\"  {}: {}\", i + 1, line);\n        }\n        let line_count = content.lines().count();\n        if line_count > 15 {\n            println!(\"  ... ({} more lines)\", line_count - 15);\n        }\n        println!(\"  ────────────────────────────────────────\\n\");\n\n        let user_input: String = Input::new()\n            .with_prompt(prompt)\n            .allow_empty(true)\n            .interact_text()\n            .map_err(|e| adk_core::AdkError::Tool(format!(\"Interaction error: {}\", e)))?;\n\n        let trimmed = user_input.trim();\n\n        match trimmed.to_lowercase().as_str() {\n            \"edit\" => {\n                println!(\"📝 Opening editor... (Save and close to submit changes)\");\n                let edited = Editor::new()\n                    .require_save(true)\n                    .edit(content)\n                    .map_err(|e| adk_core::AdkError::Tool(format!(\"Editor error: {}\", e)))?;\n\n                let new_content = edited.unwrap_or_else(|| content.to_string());\n                Ok(json!({\n                    \"action\": \"edit\",\n                    \"content\": new_content,\n                    \"message\": \"User edited content\"\n                }))\n            }\n            \"pass\" | \"\" => Ok(json!({\n                \"action\": \"pass\",\n                \"content\": content,\n                \"message\": \"User passed\"\n            })),\n            _ => Ok(json!({\n                \"action\": \"feedback\",\n                \"feedback\": trimmed,\n                \"content\": content,\n                \"message\": \"User provided feedback\"\n            })),\n        }\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 9.0,
      "lines_of_code": 161,
      "number_of_classes": 2,
      "number_of_functions": 2
    },
    "dependencies": [
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 2,
        "name": "adk_core",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 3,
        "name": "async_trait",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 4,
        "name": "dialoguer",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 5,
        "name": "serde_json",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "standard_library",
        "is_external": false,
        "line_number": 6,
        "name": "std",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "This component provides two HITL (Human-in-the-Loop) tools for content review and editing workflows. The ReviewAndEditContentTool allows users to review markdown content and optionally edit it using their default text editor. The ReviewWithFeedbackContentTool extends this functionality by providing three interaction modes: edit content in editor, pass without changes, or provide feedback text. Both tools implement the Tool trait interface and provide interactive command-line interfaces for user interaction.",
    "interfaces": [
      {
        "description": "Tool for reviewing and optionally editing content",
        "interface_type": "struct",
        "name": "ReviewAndEditContentTool",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Returns tool name",
        "interface_type": "method",
        "name": "name",
        "parameters": [
          {
            "description": "Tool instance",
            "is_optional": false,
            "name": "self",
            "param_type": "&Self"
          }
        ],
        "return_type": "&str",
        "visibility": "public"
      },
      {
        "description": "Returns tool description",
        "interface_type": "method",
        "name": "description",
        "parameters": [
          {
            "description": "Tool instance",
            "is_optional": false,
            "name": "self",
            "param_type": "&Self"
          }
        ],
        "return_type": "&str",
        "visibility": "public"
      },
      {
        "description": "Returns JSON schema for tool parameters",
        "interface_type": "method",
        "name": "parameters_schema",
        "parameters": [
          {
            "description": "Tool instance",
            "is_optional": false,
            "name": "self",
            "param_type": "&Self"
          }
        ],
        "return_type": "Option<Value>",
        "visibility": "public"
      },
      {
        "description": "Executes the tool with given arguments",
        "interface_type": "method",
        "name": "execute",
        "parameters": [
          {
            "description": "Tool instance",
            "is_optional": false,
            "name": "self",
            "param_type": "&Self"
          },
          {
            "description": "Tool execution context",
            "is_optional": false,
            "name": "_ctx",
            "param_type": "Arc<dyn ToolContext>"
          },
          {
            "description": "Tool arguments",
            "is_optional": false,
            "name": "args",
            "param_type": "Value"
          }
        ],
        "return_type": "adk_core::Result<Value>",
        "visibility": "public"
      },
      {
        "description": "Tool for reviewing content with edit/pass/feedback options",
        "interface_type": "struct",
        "name": "ReviewWithFeedbackContentTool",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Returns tool name",
        "interface_type": "method",
        "name": "name",
        "parameters": [
          {
            "description": "Tool instance",
            "is_optional": false,
            "name": "self",
            "param_type": "&Self"
          }
        ],
        "return_type": "&str",
        "visibility": "public"
      },
      {
        "description": "Returns tool description",
        "interface_type": "method",
        "name": "description",
        "parameters": [
          {
            "description": "Tool instance",
            "is_optional": false,
            "name": "self",
            "param_type": "&Self"
          }
        ],
        "return_type": "&str",
        "visibility": "public"
      },
      {
        "description": "Returns JSON schema for tool parameters",
        "interface_type": "method",
        "name": "parameters_schema",
        "parameters": [
          {
            "description": "Tool instance",
            "is_optional": false,
            "name": "self",
            "param_type": "&Self"
          }
        ],
        "return_type": "Option<Value>",
        "visibility": "public"
      },
      {
        "description": "Executes the tool with given arguments",
        "interface_type": "method",
        "name": "execute",
        "parameters": [
          {
            "description": "Tool instance",
            "is_optional": false,
            "name": "self",
            "param_type": "&Self"
          },
          {
            "description": "Tool execution context",
            "is_optional": false,
            "name": "_ctx",
            "param_type": "Arc<dyn ToolContext>"
          },
          {
            "description": "Tool arguments",
            "is_optional": false,
            "name": "args",
            "param_type": "Value"
          }
        ],
        "return_type": "adk_core::Result<Value>",
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Provide interactive content review interface for HITL workflows",
      "Enable content editing through external text editor integration",
      "Support multiple interaction modes (edit/pass/feedback) for content review",
      "Handle content display and user input processing",
      "Manage error handling and user interaction flows"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "tool",
      "description": "Session-scoped idea artifact tools for saving and loading idea.md files",
      "file_path": "crates/cowork-core/src/tools/idea_tools.rs",
      "functions": [
        "save_idea",
        "load_idea"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "SaveIdeaTool::new",
        "SaveIdeaTool::name",
        "SaveIdeaTool::description",
        "SaveIdeaTool::parameters_schema",
        "SaveIdeaTool::execute",
        "LoadIdeaTool::new",
        "LoadIdeaTool::name",
        "LoadIdeaTool::description",
        "LoadIdeaTool::parameters_schema",
        "LoadIdeaTool::execute"
      ],
      "name": "idea_tools.rs",
      "source_summary": "// Idea artifact tools (Session-scoped)\nuse crate::storage::*;\nuse adk_core::{Tool, ToolContext};\nuse async_trait::async_trait;\nuse serde_json::{json, Value};\nuse std::sync::Arc;\n\npub struct SaveIdeaTool {\n    session_id: String,\n}\n\nimpl SaveIdeaTool {\n    pub fn new(session_id: String) -> Self {\n        Self { session_id }\n    }\n}\n\n#[async_trait]\nimpl Tool for SaveIdeaTool {\n    fn name(&self) -> &str {\n        \"save_idea\"\n    }\n\n    fn description(&self) -> &str {\n        \"Save idea.md as a session-scoped artifact (.cowork/sessions/<id>/artifacts/idea.md).\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"content\": {\n                    \"type\": \"string\",\n                    \"description\": \"Markdown content of idea.md\"\n                }\n            },\n            \"required\": [\"content\"]\n        }))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {\n        let content = args[\"content\"].as_str().unwrap();\n        save_idea(&self.session_id, content)\n            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;\n\n        Ok(json!({\n            \"status\": \"success\",\n            \"message\": \"Idea saved successfully\"\n        }))\n    }\n}\n\npub struct LoadIdeaTool {\n    session_id: String,\n}\n\nimpl LoadIdeaTool {\n    pub fn new(session_id: String) -> Self {\n        Self { session_id }\n    }\n}\n\n#[async_trait]\nimpl Tool for LoadIdeaTool {\n    fn name(&self) -> &str {\n        \"load_idea\"\n    }\n\n    fn description(&self) -> &str {\n        \"Load idea.md from current session artifacts.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\"type\": \"object\", \"properties\": {}}))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> adk_core::Result<Value> {\n        let content = load_idea(&self.session_id)\n            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;\n\n        Ok(json!({\n            \"status\": \"success\",\n            \"content\": content\n        }))\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 3.0,
      "lines_of_code": 86,
      "number_of_classes": 2,
      "number_of_functions": 2
    },
    "dependencies": [
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": 2,
        "name": "crate::storage",
        "path": "crates/cowork-core/src/storage",
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 3,
        "name": "adk_core",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 4,
        "name": "async_trait",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 5,
        "name": "serde_json",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "standard_library",
        "is_external": false,
        "line_number": 6,
        "name": "std::sync::Arc",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "This component provides two session-scoped tools for managing idea artifacts: SaveIdeaTool and LoadIdeaTool. The SaveIdeaTool handles saving markdown content as idea.md files within session-specific artifact directories (.cowork/sessions/<id>/artifacts/idea.md). The LoadIdeaTool retrieves previously saved idea content from the same location. Both tools implement the async_trait pattern and follow the Tool interface contract, providing proper parameter schemas and error handling. The component demonstrates clean separation of concerns with dedicated structs for each operation.",
    "interfaces": [
      {
        "description": "Creates a new SaveIdeaTool instance with session context",
        "interface_type": "constructor",
        "name": "SaveIdeaTool::new",
        "parameters": [
          {
            "description": "Session identifier for artifact scoping",
            "is_optional": false,
            "name": "session_id",
            "param_type": "String"
          }
        ],
        "return_type": "SaveIdeaTool",
        "visibility": "public"
      },
      {
        "description": "Returns the tool name identifier",
        "interface_type": "method",
        "name": "SaveIdeaTool::name",
        "parameters": [],
        "return_type": "&str",
        "visibility": "public"
      },
      {
        "description": "Provides tool functionality description",
        "interface_type": "method",
        "name": "SaveIdeaTool::description",
        "parameters": [],
        "return_type": "&str",
        "visibility": "public"
      },
      {
        "description": "Defines JSON schema for tool parameters",
        "interface_type": "method",
        "name": "SaveIdeaTool::parameters_schema",
        "parameters": [],
        "return_type": "Option<Value>",
        "visibility": "public"
      },
      {
        "description": "Executes the idea saving operation",
        "interface_type": "async_method",
        "name": "SaveIdeaTool::execute",
        "parameters": [
          {
            "description": "Tool execution context",
            "is_optional": false,
            "name": "_ctx",
            "param_type": "Arc<dyn ToolContext>"
          },
          {
            "description": "JSON arguments containing content",
            "is_optional": false,
            "name": "args",
            "param_type": "Value"
          }
        ],
        "return_type": "adk_core::Result<Value>",
        "visibility": "public"
      },
      {
        "description": "Creates a new LoadIdeaTool instance with session context",
        "interface_type": "constructor",
        "name": "LoadIdeaTool::new",
        "parameters": [
          {
            "description": "Session identifier for artifact retrieval",
            "is_optional": false,
            "name": "session_id",
            "param_type": "String"
          }
        ],
        "return_type": "LoadIdeaTool",
        "visibility": "public"
      },
      {
        "description": "Returns the tool name identifier",
        "interface_type": "method",
        "name": "LoadIdeaTool::name",
        "parameters": [],
        "return_type": "&str",
        "visibility": "public"
      },
      {
        "description": "Provides tool functionality description",
        "interface_type": "method",
        "name": "LoadIdeaTool::description",
        "parameters": [],
        "return_type": "&str",
        "visibility": "public"
      },
      {
        "description": "Defines JSON schema for tool parameters",
        "interface_type": "method",
        "name": "LoadIdeaTool::parameters_schema",
        "parameters": [],
        "return_type": "Option<Value>",
        "visibility": "public"
      },
      {
        "description": "Executes the idea loading operation",
        "interface_type": "async_method",
        "name": "LoadIdeaTool::execute",
        "parameters": [
          {
            "description": "Tool execution context",
            "is_optional": false,
            "name": "_ctx",
            "param_type": "Arc<dyn ToolContext>"
          },
          {
            "description": "JSON arguments (empty for this tool)",
            "is_optional": false,
            "name": "_args",
            "param_type": "Value"
          }
        ],
        "return_type": "adk_core::Result<Value>",
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Session-scoped idea artifact persistence management",
      "Markdown content storage and retrieval operations",
      "Tool interface implementation for external integration",
      "Error handling and conversion between storage and tool errors",
      "Parameter validation and schema definition for tool operations"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "tool",
      "description": "Data operation tools for managing structured development artifacts (requirements, features, design components, tasks) within a session scope",
      "file_path": "crates/cowork-core/src/tools/data_tools.rs",
      "functions": [
        "create_requirement",
        "add_feature",
        "create_design_component",
        "create_task",
        "update_feature_status",
        "update_task_status",
        "get_requirements",
        "get_design",
        "get_plan"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "CreateRequirementTool",
        "AddFeatureTool",
        "CreateDesignComponentTool",
        "CreateTaskTool",
        "UpdateFeatureStatusTool",
        "UpdateTaskStatusTool",
        "GetRequirementsTool",
        "GetDesignTool",
        "GetPlanTool"
      ],
      "name": "data_tools.rs",
      "source_summary": "// Data operation tools - Create and modify structured data (Session-scoped)\nuse crate::data::*;\nuse crate::storage::*;\nuse adk_core::{Tool, ToolContext, AdkError};\nuse async_trait::async_trait;\nuse serde_json::{json, Value};\nuse std::sync::Arc;\n\n// ============================================================================\n// CreateRequirementTool\n// ============================================================================\n\npub struct CreateRequirementTool {\n    session_id: String,\n}\n\nimpl CreateRequirementTool {\n    pub fn new(session_id: String) -> Self {\n        Self { session_id }\n    }\n}\n\n#[async_trait]\nimpl Tool for CreateRequirementTool {\n    fn name(&self) -> &str {\n        \"create_requirement\"\n    }\n\n    fn description(&self) -> &str {\n        \"Create a new requirement in requirements.json. Requirements define what \\\n         the system must do. Each requirement should be SMART (Specific, Measurable, \\\n         Achievable, Relevant, Time-bound) with clear acceptance criteria.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"title\": {\n                    \"type\": \"string\",\n                    \"description\": \"Brief requirement title\"\n                },\n                \"description\": {\n                    \"type\": \"string\",\n                    \"description\": \"Detailed description of the requirement\"\n                },\n                \"priority\": {\n                    \"type\": \"string\",\n                    \"enum\": [\"high\", \"medium\", \"low\"],\n                    \"description\": \"Priority level\"\n                },\n                \"category\": {\n                    \"type\": \"string\",\n                    \"enum\": [\"functional\", \"non_functional\"],\n                    \"description\": \"Requirement category\"\n                },\n                \"acceptance_criteria\": {\n                    \"type\": \"array\",\n                    \"items\": {\"type\": \"string\"},\n                    \"description\": \"List of acceptance criteria\"\n                }\n            },\n            \"required\": [\"title\", \"description\", \"priority\", \"category\", \"acceptance_criteria\"]\n        }))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {\n        let mut reqs = load_requirements(&self.session_id).map_err(|e| AdkError::Tool(e.to_string()))?;\n\n        let req_id = generate_id(\"REQ\", reqs.requirements.len());\n\n        let priority = match args[\"priority\"].as_str().unwrap() {\n            \"high\" => Priority::High,\n            \"medium\" => Priority::Medium,\n            \"low\" => Priority::Low,\n            _ => Priority::Medium,\n        };\n\n        let category = match args[\"category\"].as_str().unwrap() {\n            \"functional\" => RequirementCategory::Functional,\n            \"non_functional\" => RequirementCategory::NonFunctional,\n            _ => RequirementCategory::Functional,\n        };\n\n        let requirement = Requirement {\n            id: req_id.clone(),\n            title: args[\"title\"].as_str().unwrap().to_string(),\n            description: args[\"description\"].as_str().unwrap().to_string(),\n            priority,\n            category,\n            acceptance_criteria: args[\"acceptance_criteria\"]\n                .as_array()\n                .unwrap()\n                .iter()\n                .map(|v| v.as_str().unwrap().to_string())\n                .collect(),\n            related_features: vec![],\n        };\n\n        reqs.requirements.push(requirement.clone());\n        reqs.updated_at = chrono::Utc::now();\n        save_requirements(&self.session_id, &reqs).map_err(|e| AdkError::Tool(e.to_string()))?;\n\n        println!(\"✅ Created: {} - {}\", req_id, requirement.title);\n\n        Ok(json!({\n            \"status\": \"success\",\n            \"requirement_id\": req_id,\n            \"message\": format!(\"Requirement {} created successfully\", req_id)\n        }))\n    }\n}\n\n// ============================================================================\n// AddFeatureTool\n// ============================================================================\n\npub struct AddFeatureTool {\n    session_id: String,\n}\n\nimpl AddFeatureTool {\n    pub fn new(session_id: String) -> Self {\n        Self { session_id }\n    }\n}\n\n#[async_trait]\nimpl Tool for AddFeatureTool {\n    fn name(&self) -> &str {\n        \"add_feature\"\n    }\n\n    fn description(&self) -> &str {\n        \"Add a new feature to feature_list.json. Features are concrete \\\n         functionalities that implement one or more requirements. Each \\\n         feature will later be broken down into implementation tasks.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"name\": {\n                    \"type\": \"string\",\n                    \"description\": \"Feature name\"\n                },\n                \"description\": {\n                    \"type\": \"string\",\n                    \"description\": \"Detailed description\"\n                },\n                \"requirement_ids\": {\n                    \"type\": \"array\",\n                    \"items\": {\"type\": \"string\"},\n                    \"description\": \"IDs of requirements this feature implements\"\n                },\n                \"completion_criteria\": {\n                    \"type\": \"array\",\n                    \"items\": {\"type\": \"string\"},\n                    \"description\": \"Criteria for feature completion\"\n                }\n            },\n            \"required\": [\"name\", \"description\", \"requirement_ids\", \"completion_criteria\"]\n        }))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {\n        let mut features = load_feature_list(&self.session_id).map_err(|e| AdkError::Tool(e.to_string()))?;\n\n        let feat_id = generate_id(\"FEAT\", features.features.len());\n\n        let feature = Feature {\n            id: feat_id.clone(),\n            name: args[\"name\"].as_str().unwrap().to_string(),\n            description: args[\"description\"].as_str().unwrap().to_string(),\n            requirement_ids: args[\"requirement_ids\"]\n                .as_array()\n                .unwrap()\n                .iter()\n                .map(|v| v.as_str().unwrap().to_string())\n                .collect(),\n            status: FeatureStatus::Pending,\n            assigned_to_tasks: vec![],\n            completion_criteria: args[\"completion_criteria\"]\n                .as_array()\n                .unwrap()\n                .iter()\n                .map(|v| v.as_str().unwrap().to_string())\n                .collect(),\n            created_at: chrono::Utc::now(),\n            completed_at: None,\n            metadata: FeatureMetadata::default(),\n        };\n\n        features.features.push(feature);\n        save_feature_list(&self.session_id, &features).map_err(|e| AdkError::Tool(e.to_string()))?;\n\n        Ok(json!({\n            \"status\": \"success\",\n            \"feature_id\": feat_id,\n            \"message\": format!(\"Feature {} created successfully\", feat_id)\n        }))\n    }\n}\n\n// ============================================================================\n// CreateDesignComponentTool\n// ============================================================================\n\npub struct CreateDesignComponentTool {\n    session_id: String,\n}\n\nimpl CreateDesignComponentTool {\n    pub fn new(session_id: String) -> Self {\n        Self { session_id }\n    }\n}\n\n#[async_trait]\nimpl Tool for CreateDesignComponentTool {\n    fn name(&self) -> &str {\n        \"create_design_component\"\n    }\n\n    fn description(&self) -> &str {\n        \"Create a new component in design_spec.json. Components are the \\\n         architectural building blocks (services, modules, UI components) \\\n         that implement features.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"name\": {\n                    \"type\": \"string\",\n                    \"description\": \"Component name\"\n                },\n                \"component_type\": {\n                    \"type\": \"string\",\n                    \"enum\": [\"backend_service\", \"frontend_component\", \"database\", \"api_gateway\"],\n                    \"description\": \"Type of component\"\n                },\n                \"responsibilities\": {\n                    \"type\": \"array\",\n                    \"items\": {\"type\": \"string\"},\n                    \"description\": \"List of responsibilities\"\n                },\n                \"technology\": {\n                    \"type\": \"string\",\n                    \"description\": \"Technology stack\"\n                },\n                \"related_features\": {\n                    \"type\": \"array\",\n                    \"items\": {\"type\": \"string\"},\n                    \"description\": \"Related feature IDs\"\n                }\n            },\n            \"required\": [\"name\", \"component_type\", \"responsibilities\", \"technology\"]\n        }))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {\n        let mut design = load_design_spec(&self.session_id).map_err(|e| AdkError::Tool(e.to_string()))?;\n\n        let comp_id = generate_id(\"COMP\", design.architecture.components.len());\n\n        let component_type = args.get(\"component_type\")\n            .and_then(|v| v.as_str())\n            .ok_or_else(|| AdkError::Tool(\"Missing or invalid 'component_type' parameter\".to_string()))?;\n        \n        let component_type = match component_type {\n            \"backend_service\" => ComponentType::BackendService,\n            \"frontend_component\" => ComponentType::FrontendComponent,\n            \"database\" => ComponentType::Database,\n            \"api_gateway\" => ComponentType::ApiGateway,\n            other => ComponentType::Other(other.to_string()),\n        };\n\n        let name = args.get(\"name\")\n            .and_then(|v| v.as_str())\n            .ok_or_else(|| AdkError::Tool(\"Missing or invalid 'name' parameter\".to_string()))?\n            .to_string();\n\n        let technology = args.get(\"technology\")\n            .and_then(|v| v.as_str())\n            .ok_or_else(|| AdkError::Tool(\"Missing or invalid 'technology' parameter\".to_string()))?\n            .to_string();\n\n        let responsibilities = args.get(\"responsibilities\")\n            .and_then(|v| v.as_array())\n            .ok_or_else(|| AdkError::Tool(\"Missing or invalid 'responsibilities' parameter (must be an array)\".to_string()))?\n            .iter()\n            .filter_map(|v| v.as_str().map(|s| s.to_string()))\n            .collect::<Vec<String>>();\n\n        if responsibilities.is_empty() {\n            return Err(AdkError::Tool(\"'responsibilities' array cannot be empty\".to_string()));\n        }\n\n        let related_features = args.get(\"related_features\")\n            .and_then(|v| v.as_array())\n            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())\n            .unwrap_or_default();\n\n        let component = DesignComponent {\n            id: comp_id.clone(),\n            name,\n            component_type,\n            responsibilities,\n            technology,\n            interfaces: vec![],\n            related_features,\n        };\n\n        design.architecture.components.push(component.clone());\n        save_design_spec(&self.session_id, &design).map_err(|e| AdkError::Tool(e.to_string()))?;\n\n        println!(\"🏗️  Created component: {} - {}\", comp_id, component.name);\n\n        Ok(json!({\n            \"status\": \"success\",\n            \"component_id\": comp_id,\n            \"message\": format!(\"Component {} created successfully\", comp_id)\n        }))\n    }\n}\n\n// ============================================================================\n// CreateTaskTool\n// ============================================================================\n\npub struct CreateTaskTool {\n    session_id: String,\n}\n\nimpl CreateTaskTool {\n    pub fn new(session_id: String) -> Self {\n        Self { session_id }\n    }\n}\n\n#[async_trait]\nimpl Tool for CreateTaskTool {\n    fn name(&self) -> &str {\n        \"create_task\"\n    }\n\n    fn description(&self) -> &str {\n        \"Create an implementation task in implementation_plan.json. Tasks \\\n         are concrete coding work items that implement features.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"title\": {\"type\": \"string\"},\n                \"description\": {\"type\": \"string\"},\n                \"feature_id\": {\"type\": \"string\"},\n                \"component_id\": {\"type\": \"string\"},\n                \"files_to_create\": {\n                    \"type\": \"array\",\n                    \"items\": {\"type\": \"string\"}\n                },\n                \"dependencies\": {\n                    \"type\": \"array\",\n                    \"items\": {\"type\": \"string\"},\n                    \"description\": \"Task IDs that must be completed first\"\n                },\n                \"acceptance_criteria\": {\n                    \"type\": \"array\",\n                    \"items\": {\"type\": \"string\"}\n                }\n            },\n            \"required\": [\"title\", \"description\", \"feature_id\", \"component_id\", \"acceptance_criteria\"]\n        }))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {\n        let mut plan = load_implementation_plan(&self.session_id).map_err(|e| AdkError::Tool(e.to_string()))?;\n\n        let task_id = generate_id(\"TASK\", plan.tasks.len());\n\n        let task = Task {\n            id: task_id.clone(),\n            title: args[\"title\"].as_str().unwrap().to_string(),\n            description: args[\"description\"].as_str().unwrap().to_string(),\n            feature_id: args[\"feature_id\"].as_str().unwrap().to_string(),\n            component_id: args[\"component_id\"].as_str().unwrap().to_string(),\n            status: TaskStatus::Pending,\n            dependencies: args.get(\"dependencies\")\n                .and_then(|v| v.as_array())\n                .map(|arr| arr.iter().map(|v| v.as_str().unwrap().to_string()).collect())\n                .unwrap_or_default(),\n            estimated_effort: None,\n            files_to_create: args.get(\"files_to_create\")\n                .and_then(|v| v.as_array())\n                .map(|arr| arr.iter().map(|v| v.as_str().unwrap().to_string()).collect())\n                .unwrap_or_default(),\n            acceptance_criteria: args[\"acceptance_criteria\"]\n                .as_array()\n                .unwrap()\n                .iter()\n                .map(|v| v.as_str().unwrap().to_string())\n                .collect(),\n            created_at: chrono::Utc::now(),\n            started_at: None,\n            completed_at: None,\n        };\n\n        plan.tasks.push(task);\n        save_implementation_plan(&self.session_id, &plan).map_err(|e| AdkError::Tool(e.to_string()))?;\n\n        Ok(json!({\n            \"status\": \"success\",\n            \"task_id\": task_id,\n            \"message\": format!(\"Task {} created successfully\", task_id)\n        }))\n    }\n}\n\n// ============================================================================\n// Update Status Tools\n// ============================================================================\n\npub struct UpdateFeatureStatusTool {\n    session_id: String,\n}\n\nimpl UpdateFeatureStatusTool {\n    pub fn new(session_id: String) -> Self {\n        Self { session_id }\n    }\n}\n\n#[async_trait]\nimpl Tool for UpdateFeatureStatusTool {\n    fn name(&self) -> &str {\n        \"update_feature_status\"\n    }\n\n    fn description(&self) -> &str {\n        \"Update the status of a feature. Valid transitions: \\\n         pending → in_progress → completed.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"feature_id\": {\"type\": \"string\"},\n                \"new_status\": {\n                    \"type\": \"string\",\n                    \"enum\": [\"pending\", \"in_progress\", \"completed\", \"blocked\"]\n                }\n            },\n            \"required\": [\"feature_id\", \"new_status\"]\n        }))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {\n        let mut features = load_feature_list(&self.session_id).map_err(|e| AdkError::Tool(e.to_string()))?;\n\n        let feature_id = args[\"feature_id\"].as_str().unwrap();\n        let new_status_str = args[\"new_status\"].as_str().unwrap();\n\n        let new_status = match new_status_str {\n            \"pending\" => FeatureStatus::Pending,\n            \"in_progress\" => FeatureStatus::InProgress,\n            \"completed\" => FeatureStatus::Completed,\n            \"blocked\" => FeatureStatus::Blocked,\n            _ => FeatureStatus::Pending,\n        };\n\n        if let Some(feature) = features.features.iter_mut().find(|f| f.id == feature_id) {\n            feature.status = new_status;\n            if new_status_str == \"completed\" {\n                feature.completed_at = Some(chrono::Utc::now());\n            }\n            save_feature_list(&self.session_id, &features).map_err(|e| AdkError::Tool(e.to_string()))?;\n\n            Ok(json!({\n                \"status\": \"success\",\n                \"feature_id\": feature_id,\n                \"new_status\": new_status_str,\n                \"message\": format!(\"Feature {} status updated to {}\", feature_id, new_status_str)\n            }))\n        } else {\n            Ok(json!({\n                \"status\": \"error\",\n                \"message\": format!(\"Feature {} not found\", feature_id)\n            }))\n        }\n    }\n}\n\npub struct UpdateTaskStatusTool {\n    session_id: String,\n}\n\nimpl UpdateTaskStatusTool {\n    pub fn new(session_id: String) -> Self {\n        Self { session_id }\n    }\n}\n\n#[async_trait]\nimpl Tool for UpdateTaskStatusTool {\n    fn name(&self) -> &str {\n        \"update_task_status\"\n    }\n\n    fn description(&self) -> &str {\n        \"Update task status. Call this as you start and complete tasks.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"task_id\": {\"type\": \"string\"},\n                \"new_status\": {\n                    \"type\": \"string\",\n                    \"enum\": [\"pending\", \"in_progress\", \"completed\", \"blocked\"]\n                }\n            },\n            \"required\": [\"task_id\", \"new_status\"]\n        }))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {\n        let mut plan = load_implementation_plan(&self.session_id).map_err(|e| AdkError::Tool(e.to_string()))?;\n\n        let task_id = args.get(\"task_id\")\n            .and_then(|v| v.as_str())\n            .ok_or_else(|| AdkError::Tool(\"Missing or invalid 'task_id' parameter\".to_string()))?;\n        \n        let new_status_str = args.get(\"new_status\")\n            .and_then(|v| v.as_str())\n            .ok_or_else(|| AdkError::Tool(\"Missing or invalid 'new_status' parameter\".to_string()))?;\n\n        let new_status = match new_status_str {\n            \"pending\" => TaskStatus::Pending,\n            \"in_progress\" => TaskStatus::InProgress,\n            \"completed\" => TaskStatus::Completed,\n            \"blocked\" => TaskStatus::Blocked,\n            _ => return Err(AdkError::Tool(format!(\"Invalid status: {}. Must be one of: pending, in_progress, completed, blocked\", new_status_str))),\n        };\n\n        if let Some(task) = plan.tasks.iter_mut().find(|t| t.id == task_id) {\n            task.status = new_status;\n            match new_status_str {\n                \"in_progress\" => task.started_at = Some(chrono::Utc::now()),\n                \"completed\" => task.completed_at = Some(chrono::Utc::now()),\n                _ => {}\n            }\n            save_implementation_plan(&self.session_id, &plan).map_err(|e| AdkError::Tool(e.to_string()))?;\n\n            println!(\"✓ Task {} → {}\", task_id, new_status_str);\n\n            Ok(json!({\n                \"status\": \"success\",\n                \"task_id\": task_id,\n                \"new_status\": new_status_str\n            }))\n        } else {\n            Ok(json!({\n                \"status\": \"error\",\n                \"message\": format!(\"Task {} not found\", task_id)\n            }))\n        }\n    }\n}\n\n// ============================================================================\n// Get/Read Tools\n// ============================================================================\n\npub struct GetRequirementsTool {\n    session_id: String,\n}\n\nimpl GetRequirementsTool {\n    pub fn new(session_id: String) -> Self {\n        Self { session_id }\n    }\n}\n\n#[async_trait]\nimpl Tool for GetRequirementsTool {\n    fn name(&self) -> &str {\n        \"get_requirements\"\n    }\n\n    fn description(&self) -> &str {\n        \"Retrieve all requirements and features.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\n            \"type\": \"object\",\n            \"properties\": {}\n        }))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> adk_core::Result<Value> {\n        let requirements = load_requirements(&self.session_id).map_err(|e| AdkError::Tool(e.to_string()))?;\n        let features = load_feature_list(&self.session_id).map_err(|e| AdkError::Tool(e.to_string()))?;\n\n        Ok(json!({\n            \"requirements\": requirements.requirements,\n            \"features\": features.features\n        }))\n    }\n}\n\npub struct GetDesignTool {\n    session_id: String,\n}\n\nimpl GetDesignTool {\n    pub fn new(session_id: String) -> Self {\n        Self { session_id }\n    }\n}\n\n#[async_trait]\nimpl Tool for GetDesignTool {\n    fn name(&self) -> &str {\n        \"get_design\"\n    }\n\n    fn description(&self) -> &str {\n        \"Retrieve the design specification.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\"type\": \"object\", \"properties\": {}}))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> adk_core::Result<Value> {\n        let design = load_design_spec(&self.session_id).map_err(|e| AdkError::Tool(e.to_string()))?;\n        Ok(serde_json::to_value(design).map_err(|e| AdkError::Tool(e.to_string()))?)\n    }\n}\n\npub struct GetPlanTool {\n    session_id: String,\n}\n\nimpl GetPlanTool {\n    pub fn new(session_id: String) -> Self {\n        Self { session_id }\n    }\n}\n\n#[async_trait]\nimpl Tool for GetPlanTool {\n    fn name(&self) -> &str {\n        \"get_plan\"\n    }\n\n    fn description(&self) -> &str {\n        \"Retrieve the implementation plan with all tasks.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\"type\": \"object\", \"properties\": {}}))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> adk_core::Result<Value> {\n        let plan = load_implementation_plan(&self.session_id).map_err(|e| AdkError::Tool(e.to_string()))?;\n        Ok(serde_json::to_value(plan).map_err(|e| AdkError::Tool(e.to_string()))?)\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 21.0,
      "lines_of_code": 677,
      "number_of_classes": 9,
      "number_of_functions": 9
    },
    "dependencies": [
      {
        "dependency_type": "internal_module",
        "is_external": false,
        "line_number": 2,
        "name": "crate::data",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "internal_module",
        "is_external": false,
        "line_number": 3,
        "name": "crate::storage",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "external_crate",
        "is_external": true,
        "line_number": 4,
        "name": "adk_core",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "external_crate",
        "is_external": true,
        "line_number": 5,
        "name": "async_trait",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "external_crate",
        "is_external": true,
        "line_number": 6,
        "name": "serde_json",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "standard_library",
        "is_external": false,
        "line_number": 7,
        "name": "std::sync::Arc",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "This component provides a comprehensive suite of tools for managing software development artifacts including requirements, features, design components, and implementation tasks. It implements the Tool trait from adk_core, enabling these operations to be executed as part of an automated development workflow. Each tool handles CRUD operations for specific artifact types with validation, error handling, and session-based data persistence. The tools follow a consistent pattern of loading data, performing operations, saving changes, and returning structured responses.",
    "interfaces": [
      {
        "description": "Creates new requirements with SMART criteria validation",
        "interface_type": "struct",
        "name": "CreateRequirementTool",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Adds features that implement requirements",
        "interface_type": "struct",
        "name": "AddFeatureTool",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Creates architectural components with technology specifications",
        "interface_type": "struct",
        "name": "CreateDesignComponentTool",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Creates implementation tasks with dependencies",
        "interface_type": "struct",
        "name": "CreateTaskTool",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Updates feature status with valid state transitions",
        "interface_type": "struct",
        "name": "UpdateFeatureStatusTool",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Updates task status with timestamp tracking",
        "interface_type": "struct",
        "name": "UpdateTaskStatusTool",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Retrieves requirements and features",
        "interface_type": "struct",
        "name": "GetRequirementsTool",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Retrieves design specifications",
        "interface_type": "struct",
        "name": "GetDesignTool",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Retrieves implementation plan with tasks",
        "interface_type": "struct",
        "name": "GetPlanTool",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Manage requirements creation and storage with SMART criteria validation",
      "Handle feature lifecycle management including status transitions and completion tracking",
      "Create and maintain design components with architectural specifications",
      "Manage implementation tasks with dependencies and acceptance criteria",
      "Provide read/write operations for all development artifacts with session isolation"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "tool",
      "description": "Human-in-the-Loop (HITL) tools providing interactive file review and editing capabilities with multiple user interaction modes",
      "file_path": "crates/cowork-core/src/tools/hitl_tools.rs",
      "functions": [
        "review_and_edit_file",
        "review_with_feedback"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "ReviewAndEditFileTool",
        "ReviewWithFeedbackTool"
      ],
      "name": "hitl_tools.rs",
      "source_summary": "// HITL (Human-in-the-Loop) tools\nuse adk_core::{Tool, ToolContext};\nuse async_trait::async_trait;\nuse dialoguer::{Confirm, Editor, Input};\nuse serde_json::{json, Value};\nuse std::fs;\nuse std::sync::Arc;\n\n/// ReviewAndEditFileTool - Original HITL tool (used in Idea stage)\npub struct ReviewAndEditFileTool;\n\n#[async_trait]\nimpl Tool for ReviewAndEditFileTool {\n    fn name(&self) -> &str {\n        \"review_and_edit_file\"\n    }\n\n    fn description(&self) -> &str {\n        \"Let the user review and optionally edit a file using their default editor. \\\n         User will be prompted: 'Do you want to edit this file? (y/n)'. \\\n         If 'y', opens the file in an editor. If 'n', continues without changes.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"file_path\": {\n                    \"type\": \"string\",\n                    \"description\": \"Path to the file to review and edit\"\n                },\n                \"title\": {\n                    \"type\": \"string\",\n                    \"description\": \"Title/description for the review prompt\"\n                }\n            },\n            \"required\": [\"file_path\", \"title\"]\n        }))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {\n        let file_path = args[\"file_path\"].as_str().unwrap();\n        let title = args[\"title\"].as_str().unwrap();\n\n        // Read current file content\n        let content = fs::read_to_string(file_path)\n            .map_err(|e| adk_core::AdkError::Tool(format!(\"Failed to read file {}: {}\", file_path, e)))?;\n\n        // Show preview\n        println!(\"\\n📝 {} - {}\", title, file_path);\n        println!(\"  ────────────────────────────────────────\");\n        let line_count = content.lines().count();\n        for (i, line) in content.lines().take(10).enumerate() {\n            println!(\"  {}: {}\", i + 1, line);\n        }\n        if line_count > 10 {\n            println!(\"  ... ({} more lines)\", line_count - 10);\n        }\n        println!(\"  ────────────────────────────────────────\\n\");\n\n        // Ask user if they want to edit\n        let should_edit = Confirm::new()\n            .with_prompt(\"Do you want to edit this file? (y/n)\")\n            .default(false)\n            .interact()\n            .map_err(|e| adk_core::AdkError::Tool(format!(\"Interaction error: {}\", e)))?;\n\n        if !should_edit {\n            return Ok(json!({\n                \"status\": \"no_changes\",\n                \"message\": \"User chose not to edit the file\"\n            }));\n        }\n\n        // Open editor\n        println!(\"📝 Opening editor... (Save and close to submit changes)\");\n        let edited = Editor::new()\n            .require_save(true)\n            .edit(&content)\n            .map_err(|e| adk_core::AdkError::Tool(format!(\"Editor error: {}\", e)))?;\n\n        match edited {\n            Some(new_content) if new_content.trim() != content.trim() => {\n                // Save changes\n                fs::write(file_path, &new_content)\n                    .map_err(|e| adk_core::AdkError::Tool(format!(\"Failed to write file: {}\", e)))?;\n\n                println!(\"✅ File updated successfully\");\n                Ok(json!({\n                    \"status\": \"edited\",\n                    \"message\": \"File was edited and saved\",\n                    \"changes_made\": true\n                }))\n            }\n            _ => {\n                println!(\"ℹ️  No changes made\");\n                Ok(json!({\n                    \"status\": \"no_changes\",\n                    \"message\": \"File was not modified\"\n                }))\n            }\n        }\n    }\n}\n\n/// ReviewWithFeedbackTool - Enhanced HITL tool with three modes:\n/// 1. User types \"edit\" → Opens editor\n/// 2. User types \"pass\" → Continues without changes\n/// 3. User types other text → Returns as feedback for agent to process\npub struct ReviewWithFeedbackTool;\n\n#[async_trait]\nimpl Tool for ReviewWithFeedbackTool {\n    fn name(&self) -> &str {\n        \"review_with_feedback\"\n    }\n\n    fn description(&self) -> &str {\n        \"Show user a file preview and ask for feedback. User can:\\n\\\n         - Type 'edit' to open the file in an editor\\n\\\n         - Type 'pass' to continue without changes\\n\\\n         - Type any other text to provide feedback/suggestions (agent will revise based on feedback)\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"path\": {\n                    \"type\": \"string\",\n                    \"description\": \"Path to the file to review\"\n                },\n                \"title\": {\n                    \"type\": \"string\",\n                    \"description\": \"Title/description for the review prompt\"\n                },\n                \"prompt\": {\n                    \"type\": \"string\",\n                    \"description\": \"Custom prompt to show the user (e.g., '请审查需求大纲')\"\n                }\n            },\n            \"required\": [\"path\", \"title\"]\n        }))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {\n        let file_path = args[\"path\"].as_str().unwrap();\n        let title = args[\"title\"].as_str().unwrap();\n        let default_prompt = \"输入 'edit' 编辑，'pass' 继续，或直接输入修改建议\";\n        let prompt = args[\"prompt\"].as_str().unwrap_or(default_prompt);\n\n        // Read current file content\n        let content = fs::read_to_string(file_path)\n            .map_err(|e| adk_core::AdkError::Tool(format!(\"Failed to read file {}: {}\", file_path, e)))?;\n\n        // Show preview\n        println!(\"\\n📝 {} - {}\", title, file_path);\n        println!(\"  ────────────────────────────────────────\");\n        let line_count = content.lines().count();\n        for (i, line) in content.lines().take(15).enumerate() {\n            println!(\"  {}: {}\", i + 1, line);\n        }\n        if line_count > 15 {\n            println!(\"  ... ({} more lines)\", line_count - 15);\n        }\n        println!(\"  ────────────────────────────────────────\\n\");\n\n        // Ask user for input\n        let user_input: String = Input::new()\n            .with_prompt(prompt)\n            .allow_empty(true)\n            .interact_text()\n            .map_err(|e| adk_core::AdkError::Tool(format!(\"Interaction error: {}\", e)))?;\n\n        let user_input = user_input.trim();\n\n        // Handle different input modes\n        match user_input.to_lowercase().as_str() {\n            \"edit\" => {\n                // Mode 1: Open editor\n                println!(\"📝 Opening editor... (Save and close to submit changes)\");\n                let edited = Editor::new()\n                    .require_save(true)\n                    .edit(&content)\n                    .map_err(|e| adk_core::AdkError::Tool(format!(\"Editor error: {}\", e)))?;\n\n                match edited {\n                    Some(new_content) if new_content.trim() != content.trim() => {\n                        fs::write(file_path, &new_content)\n                            .map_err(|e| adk_core::AdkError::Tool(format!(\"Failed to write file: {}\", e)))?;\n\n                        println!(\"✅ File updated successfully\");\n                        Ok(json!({\n                            \"action\": \"edit\",\n                            \"status\": \"edited\",\n                            \"message\": \"User edited the file in editor\",\n                            \"changes_made\": true\n                        }))\n                    }\n                    _ => {\n                        println!(\"ℹ️  No changes made in editor\");\n                        Ok(json!({\n                            \"action\": \"edit\",\n                            \"status\": \"no_changes\",\n                            \"message\": \"User opened editor but made no changes\"\n                        }))\n                    }\n                }\n            }\n            \"pass\" | \"\" => {\n                // Mode 2: Pass/Continue\n                println!(\"➡️  Continuing without changes...\");\n                Ok(json!({\n                    \"action\": \"pass\",\n                    \"status\": \"passed\",\n                    \"message\": \"User chose to continue without changes\"\n                }))\n            }\n            _ => {\n                // Mode 3: Feedback text\n                println!(\"💬 Feedback received: {}\", user_input);\n                println!(\"🔄 Agent will revise based on your feedback...\");\n                Ok(json!({\n                    \"action\": \"feedback\",\n                    \"status\": \"feedback_provided\",\n                    \"feedback\": user_input,\n                    \"message\": format!(\"User provided feedback: {}\", user_input)\n                }))\n            }\n        }\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 19.0,
      "lines_of_code": 232,
      "number_of_classes": 2,
      "number_of_functions": 2
    },
    "dependencies": [
      {
        "dependency_type": "framework",
        "is_external": true,
        "line_number": 2,
        "name": "adk_core",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "macro",
        "is_external": true,
        "line_number": 3,
        "name": "async_trait",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 4,
        "name": "dialoguer",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "serialization",
        "is_external": true,
        "line_number": 5,
        "name": "serde_json",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "standard_library",
        "is_external": false,
        "line_number": 6,
        "name": "std::fs",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "standard_library",
        "is_external": false,
        "line_number": 7,
        "name": "std::sync::Arc",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "This component implements two HITL (Human-in-the-Loop) tools that enable user interaction during file review processes. The ReviewAndEditFileTool provides basic file review with simple yes/no editing option, while ReviewWithFeedbackTool offers enhanced functionality with three interaction modes: edit mode (opens editor), pass mode (continues without changes), and feedback mode (provides text feedback for agent processing). Both tools implement the Tool trait from adk_core framework and provide file preview, user interaction, and file modification capabilities.",
    "interfaces": [
      {
        "description": "Basic HITL tool for file review with simple edit option",
        "interface_type": "struct",
        "name": "ReviewAndEditFileTool",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Enhanced HITL tool with multiple interaction modes",
        "interface_type": "struct",
        "name": "ReviewWithFeedbackTool",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Returns tool identifier name",
        "interface_type": "method",
        "name": "name",
        "parameters": [
          {
            "description": "Tool instance",
            "is_optional": false,
            "name": "self",
            "param_type": "&Self"
          }
        ],
        "return_type": "&str",
        "visibility": "public"
      },
      {
        "description": "Returns tool description",
        "interface_type": "method",
        "name": "description",
        "parameters": [
          {
            "description": "Tool instance",
            "is_optional": false,
            "name": "self",
            "param_type": "&Self"
          }
        ],
        "return_type": "&str",
        "visibility": "public"
      },
      {
        "description": "Returns JSON schema for tool parameters",
        "interface_type": "method",
        "name": "parameters_schema",
        "parameters": [
          {
            "description": "Tool instance",
            "is_optional": false,
            "name": "self",
            "param_type": "&Self"
          }
        ],
        "return_type": "Option<Value>",
        "visibility": "public"
      },
      {
        "description": "Executes the tool with provided arguments",
        "interface_type": "method",
        "name": "execute",
        "parameters": [
          {
            "description": "Tool instance",
            "is_optional": false,
            "name": "self",
            "param_type": "&Self"
          },
          {
            "description": "Tool execution context",
            "is_optional": false,
            "name": "_ctx",
            "param_type": "Arc<dyn ToolContext>"
          },
          {
            "description": "Tool arguments as JSON value",
            "is_optional": false,
            "name": "args",
            "param_type": "Value"
          }
        ],
        "return_type": "adk_core::Result<Value>",
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Provide interactive file review capabilities with user preview",
      "Handle file editing operations through external editors",
      "Manage multiple user interaction modes (edit/pass/feedback)",
      "Implement error handling for file operations and user interactions",
      "Return structured execution results for agent processing"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "tool",
      "description": "Goto Stage tool for Check Agent (Session-scoped) - allows restarting pipeline from specific stages",
      "file_path": "crates/cowork-core/src/tools/goto_stage_tool.rs",
      "functions": [
        "new",
        "name",
        "description",
        "parameters_schema",
        "execute"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "GotoStageTool::new",
        "GotoStageTool::name",
        "GotoStageTool::description",
        "GotoStageTool::parameters_schema",
        "GotoStageTool::execute"
      ],
      "name": "goto_stage_tool.rs",
      "source_summary": "// Goto Stage tool for Check Agent (Session-scoped)\nuse crate::data::*;\nuse crate::storage::*;\nuse adk_core::{Tool, ToolContext};\nuse async_trait::async_trait;\nuse serde_json::{json, Value};\nuse std::sync::Arc;\n\npub struct GotoStageTool {\n    session_id: String,\n}\n\nimpl GotoStageTool {\n    pub fn new(session_id: String) -> Self {\n        Self { session_id }\n    }\n}\n\n#[async_trait]\nimpl Tool for GotoStageTool {\n    fn name(&self) -> &str {\n        \"goto_stage\"\n    }\n\n    fn description(&self) -> &str {\n        \"Restart pipeline from a specific stage. Use this when critical issues \\\n         require going back to an earlier phase. Valid stages: prd, design, plan, coding.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"stage\": {\n                    \"type\": \"string\",\n                    \"enum\": [\"prd\", \"design\", \"plan\", \"coding\"],\n                    \"description\": \"Which stage to restart from\"\n                },\n                \"reason\": {\n                    \"type\": \"string\",\n                    \"description\": \"Why the restart is needed\"\n                }\n            },\n            \"required\": [\"stage\", \"reason\"]\n        }))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {\n        let stage_str = args[\"stage\"].as_str().unwrap();\n        let reason = args[\"reason\"].as_str().unwrap();\n\n        // Parse stage\n        let stage = match stage_str {\n            \"prd\" => Stage::Prd,\n            \"design\" => Stage::Design,\n            \"plan\" => Stage::Plan,\n            \"coding\" => Stage::Coding,\n            _ => {\n                return Ok(json!({\n                    \"status\": \"error\",\n                    \"message\": format!(\"Invalid stage: {}\", stage_str)\n                }));\n            }\n        };\n\n        // Load or create session meta\n        let mut meta = load_session_meta(&self.session_id)\n            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?\n            .unwrap_or_else(|| SessionMeta {\n                session_id: self.session_id.clone(),\n                created_at: chrono::Utc::now(),\n                current_stage: Some(Stage::Check),\n                restart_reason: None,\n            });\n\n        // Set restart information by updating current_stage and reason\n        meta.current_stage = Some(stage);\n        meta.restart_reason = Some(reason.to_string());\n\n        // Save session meta\n        save_session_meta(&self.session_id, &meta)\n            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;\n\n        Ok(json!({\n            \"status\": \"restart_scheduled\",\n            \"stage\": stage_str,\n            \"reason\": reason,\n            \"message\": format!(\"Pipeline will restart from {} stage. User should re-run with 'cowork revert --from {}' command.\", stage_str, stage_str)\n        }))\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 4.0,
      "lines_of_code": 91,
      "number_of_classes": 1,
      "number_of_functions": 5
    },
    "dependencies": [
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": 2,
        "name": "crate::data",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": 3,
        "name": "crate::storage",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 4,
        "name": "adk_core",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 5,
        "name": "async_trait",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 6,
        "name": "serde_json",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "standard_library",
        "is_external": false,
        "line_number": 7,
        "name": "std::sync::Arc",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "This component implements a session-scoped tool that enables restarting a pipeline workflow from specific stages (prd, design, plan, coding). It handles stage validation, session metadata loading/creation, updating restart information, and saving the updated metadata. The tool is designed for critical situations where workflow needs to revert to earlier phases.",
    "interfaces": [
      {
        "description": "Creates new GotoStageTool instance with session context",
        "interface_type": "constructor",
        "name": "new",
        "parameters": [
          {
            "description": "Session identifier for tool instantiation",
            "is_optional": false,
            "name": "session_id",
            "param_type": "String"
          }
        ],
        "return_type": "GotoStageTool",
        "visibility": "public"
      },
      {
        "description": "Returns tool identifier name",
        "interface_type": "method",
        "name": "name",
        "parameters": [],
        "return_type": "&str",
        "visibility": "public"
      },
      {
        "description": "Returns detailed tool functionality description",
        "interface_type": "method",
        "name": "description",
        "parameters": [],
        "return_type": "&str",
        "visibility": "public"
      },
      {
        "description": "Defines JSON schema for tool parameters",
        "interface_type": "method",
        "name": "parameters_schema",
        "parameters": [],
        "return_type": "Option<Value>",
        "visibility": "public"
      },
      {
        "description": "Main execution logic for stage restart functionality",
        "interface_type": "method",
        "name": "execute",
        "parameters": [
          {
            "description": "Tool execution context",
            "is_optional": false,
            "name": "_ctx",
            "param_type": "Arc<dyn ToolContext>"
          },
          {
            "description": "Input arguments as JSON value",
            "is_optional": false,
            "name": "args",
            "param_type": "Value"
          }
        ],
        "return_type": "adk_core::Result<Value>",
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Validate and parse stage input parameters",
      "Load or create session metadata",
      "Update session stage and restart reason",
      "Persist session metadata changes",
      "Provide clear restart status and user instructions"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "tool",
      "description": "Tools for modify workflow - Save/Load ChangeRequest and PatchMetadata",
      "file_path": "crates/cowork-core/src/tools/modify_tools.rs",
      "functions": [
        "SaveChangeRequestTool::new",
        "SaveChangeRequestTool::name",
        "SaveChangeRequestTool::description",
        "SaveChangeRequestTool::parameters_schema",
        "SaveChangeRequestTool::execute",
        "LoadChangeRequestTool::new",
        "LoadChangeRequestTool::name",
        "LoadChangeRequestTool::description",
        "LoadChangeRequestTool::parameters_schema",
        "LoadChangeRequestTool::execute"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "SaveChangeRequestTool",
        "LoadChangeRequestTool"
      ],
      "name": "modify_tools.rs",
      "source_summary": "// Tools for modify workflow - Save/Load ChangeRequest and PatchMetadata\nuse crate::data::*;\nuse crate::storage::*;\nuse adk_core::{Tool, ToolContext, AdkError};\nuse async_trait::async_trait;\nuse serde_json::{json, Value};\nuse std::sync::Arc;\n\n// ============================================================================\n// SaveChangeRequestTool\n// ============================================================================\n\npub struct SaveChangeRequestTool {\n    session_id: String,\n}\n\nimpl SaveChangeRequestTool {\n    pub fn new(session_id: String) -> Self {\n        Self { session_id }\n    }\n}\n\n#[async_trait]\nimpl Tool for SaveChangeRequestTool {\n    fn name(&self) -> &str {\n        \"save_change_request\"\n    }\n\n    fn description(&self) -> &str {\n        \"Save the analyzed ChangeRequest. This is the output of the Change Triage Agent.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"requires_prd_update\": {\n                    \"type\": \"boolean\",\n                    \"description\": \"Does PRD need updating?\"\n                },\n                \"requires_design_update\": {\n                    \"type\": \"boolean\",\n                    \"description\": \"Does design need updating?\"\n                },\n                \"requires_plan_update\": {\n                    \"type\": \"boolean\",\n                    \"description\": \"Does plan need updating?\"\n                },\n                \"requires_code_change\": {\n                    \"type\": \"boolean\",\n                    \"description\": \"Does code need changing?\"\n                },\n                \"affected_components\": {\n                    \"type\": \"array\",\n                    \"items\": {\"type\": \"string\"},\n                    \"description\": \"List of component IDs affected\"\n                },\n                \"affected_features\": {\n                    \"type\": \"array\",\n                    \"items\": {\"type\": \"string\"},\n                    \"description\": \"List of feature IDs affected\"\n                },\n                \"risk_level\": {\n                    \"type\": \"string\",\n                    \"enum\": [\"low\", \"medium\", \"high\"],\n                    \"description\": \"Risk assessment\"\n                },\n                \"estimated_effort\": {\n                    \"type\": \"string\",\n                    \"description\": \"Brief effort estimate\"\n                },\n                \"acceptance_criteria\": {\n                    \"type\": \"array\",\n                    \"items\": {\"type\": \"string\"},\n                    \"description\": \"What defines 'done'\"\n                },\n                \"constraints\": {\n                    \"type\": \"array\",\n                    \"items\": {\"type\": \"string\"},\n                    \"description\": \"Things to preserve\"\n                }\n            },\n            \"required\": [\"requires_prd_update\", \"requires_design_update\", \"requires_plan_update\", \"requires_code_change\", \"risk_level\"]\n        }))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {\n        // Load existing change request\n        let mut change_request = load_change_request(&self.session_id)\n            .map_err(|e| AdkError::Tool(e.to_string()))?;\n\n        // Update scope\n        change_request.scope.requires_prd_update = args[\"requires_prd_update\"].as_bool().unwrap_or(false);\n        change_request.scope.requires_design_update = args[\"requires_design_update\"].as_bool().unwrap_or(false);\n        change_request.scope.requires_plan_update = args[\"requires_plan_update\"].as_bool().unwrap_or(false);\n        change_request.scope.requires_code_change = args[\"requires_code_change\"].as_bool().unwrap_or(true);\n\n        // Update analysis\n        let risk_level = match args[\"risk_level\"].as_str().unwrap_or(\"medium\") {\n            \"low\" => RiskLevel::Low,\n            \"high\" => RiskLevel::High,\n            _ => RiskLevel::Medium,\n        };\n\n        let analysis = ChangeAnalysis {\n            affected_components: args.get(\"affected_components\")\n                .and_then(|v| v.as_array())\n                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())\n                .unwrap_or_default(),\n            affected_features: args.get(\"affected_features\")\n                .and_then(|v| v.as_array())\n                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())\n                .unwrap_or_default(),\n            risk_level,\n            estimated_effort: args.get(\"estimated_effort\")\n                .and_then(|v| v.as_str())\n                .unwrap_or(\"Unknown\")\n                .to_string(),\n        };\n\n        change_request.analysis = Some(analysis);\n\n        // Update acceptance criteria\n        if let Some(criteria) = args.get(\"acceptance_criteria\").and_then(|v| v.as_array()) {\n            change_request.acceptance_criteria = criteria.iter()\n                .filter_map(|v| v.as_str().map(|s| s.to_string()))\n                .collect();\n        }\n\n        // Update constraints\n        if let Some(constraints) = args.get(\"constraints\").and_then(|v| v.as_array()) {\n            change_request.constraints = constraints.iter()\n                .filter_map(|v| v.as_str().map(|s| s.to_string()))\n                .collect();\n        }\n\n        // Save\n        save_change_request(&self.session_id, &change_request)\n            .map_err(|e| AdkError::Tool(e.to_string()))?;\n\n        println!(\"✅ Change request analyzed and saved\");\n        println!(\"   Scope: PRD={}, Design={}, Plan={}, Code={}\",\n            change_request.scope.requires_prd_update,\n            change_request.scope.requires_design_update,\n            change_request.scope.requires_plan_update,\n            change_request.scope.requires_code_change);\n        println!(\"   Risk: {:?}\", change_request.analysis.as_ref().unwrap().risk_level);\n\n        Ok(json!({\n            \"status\": \"success\",\n            \"message\": \"Change request saved successfully\"\n        }))\n    }\n}\n\n// ============================================================================\n// LoadChangeRequestTool\n// ============================================================================\n\npub struct LoadChangeRequestTool {\n    session_id: String,\n}\n\nimpl LoadChangeRequestTool {\n    pub fn new(session_id: String) -> Self {\n        Self { session_id }\n    }\n}\n\n#[async_trait]\nimpl Tool for LoadChangeRequestTool {\n    fn name(&self) -> &str {\n        \"load_change_request\"\n    }\n\n    fn description(&self) -> &str {\n        \"Load the ChangeRequest for this session. Use this to understand what needs to be implemented.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\"type\": \"object\", \"properties\": {}}))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> adk_core::Result<Value> {\n        let change_request = load_change_request(&self.session_id)\n            .map_err(|e| AdkError::Tool(e.to_string()))?;\n\n        Ok(serde_json::to_value(change_request)\n            .map_err(|e| AdkError::Tool(e.to_string()))?)\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 8.0,
      "lines_of_code": 191,
      "number_of_classes": 2,
      "number_of_functions": 10
    },
    "dependencies": [
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 2,
        "name": "crate::data::*",
        "path": "crate::data",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 3,
        "name": "crate::storage::*",
        "path": "crate::storage",
        "version": null
      },
      {
        "dependency_type": "external",
        "is_external": true,
        "line_number": 4,
        "name": "adk_core",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "external",
        "is_external": true,
        "line_number": 5,
        "name": "async_trait",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "external",
        "is_external": true,
        "line_number": 6,
        "name": "serde_json",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "standard_library",
        "is_external": false,
        "line_number": 7,
        "name": "std::sync::Arc",
        "path": "std::sync",
        "version": null
      }
    ],
    "detailed_description": "This component provides specialized tools for managing change requests in a workflow system. It contains two main tools: SaveChangeRequestTool for persisting analyzed change requests with comprehensive scope analysis, and LoadChangeRequestTool for retrieving stored change requests. The SaveChangeRequestTool handles updating change request scope (PRD, design, plan, code changes), risk analysis, affected components/features, effort estimation, acceptance criteria, and constraints. Both tools operate within a session context and integrate with storage systems to persist and retrieve change request data.",
    "interfaces": [
      {
        "description": "Tool for saving analyzed change requests with comprehensive scope and risk analysis",
        "interface_type": "struct",
        "name": "SaveChangeRequestTool",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Constructor for SaveChangeRequestTool",
        "interface_type": "method",
        "name": "SaveChangeRequestTool::new",
        "parameters": [
          {
            "description": "Session identifier for change request context",
            "is_optional": false,
            "name": "session_id",
            "param_type": "String"
          }
        ],
        "return_type": "SaveChangeRequestTool",
        "visibility": "public"
      },
      {
        "description": "Returns tool name identifier",
        "interface_type": "method",
        "name": "SaveChangeRequestTool::name",
        "parameters": [],
        "return_type": "&str",
        "visibility": "public"
      },
      {
        "description": "Returns tool description",
        "interface_type": "method",
        "name": "SaveChangeRequestTool::description",
        "parameters": [],
        "return_type": "&str",
        "visibility": "public"
      },
      {
        "description": "Defines JSON schema for tool parameters",
        "interface_type": "method",
        "name": "SaveChangeRequestTool::parameters_schema",
        "parameters": [],
        "return_type": "Option<Value>",
        "visibility": "public"
      },
      {
        "description": "Executes the change request saving operation",
        "interface_type": "method",
        "name": "SaveChangeRequestTool::execute",
        "parameters": [
          {
            "description": "Tool execution context",
            "is_optional": false,
            "name": "_ctx",
            "param_type": "Arc<dyn ToolContext>"
          },
          {
            "description": "JSON arguments for tool execution",
            "is_optional": false,
            "name": "args",
            "param_type": "Value"
          }
        ],
        "return_type": "adk_core::Result<Value>",
        "visibility": "public"
      },
      {
        "description": "Tool for loading previously saved change requests",
        "interface_type": "struct",
        "name": "LoadChangeRequestTool",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Constructor for LoadChangeRequestTool",
        "interface_type": "method",
        "name": "LoadChangeRequestTool::new",
        "parameters": [
          {
            "description": "Session identifier for change request context",
            "is_optional": false,
            "name": "session_id",
            "param_type": "String"
          }
        ],
        "return_type": "LoadChangeRequestTool",
        "visibility": "public"
      },
      {
        "description": "Returns tool name identifier",
        "interface_type": "method",
        "name": "LoadChangeRequestTool::name",
        "parameters": [],
        "return_type": "&str",
        "visibility": "public"
      },
      {
        "description": "Returns tool description",
        "interface_type": "method",
        "name": "LoadChangeRequestTool::description",
        "parameters": [],
        "return_type": "&str",
        "visibility": "public"
      },
      {
        "description": "Defines JSON schema for tool parameters (empty)",
        "interface_type": "method",
        "name": "LoadChangeRequestTool::parameters_schema",
        "parameters": [],
        "return_type": "Option<Value>",
        "visibility": "public"
      },
      {
        "description": "Executes the change request loading operation",
        "interface_type": "method",
        "name": "LoadChangeRequestTool::execute",
        "parameters": [
          {
            "description": "Tool execution context",
            "is_optional": false,
            "name": "_ctx",
            "param_type": "Arc<dyn ToolContext>"
          },
          {
            "description": "JSON arguments for tool execution",
            "is_optional": false,
            "name": "_args",
            "param_type": "Value"
          }
        ],
        "return_type": "adk_core::Result<Value>",
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Change request persistence management",
      "Change request scope analysis and updating",
      "Risk assessment and analysis data handling",
      "Session-based change request lifecycle management",
      "Tool interface implementation for workflow integration"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "tool",
      "description": "Control tools for providing feedback and user interaction functionality",
      "file_path": "crates/cowork-core/src/tools/control_tools.rs",
      "functions": [
        "provide_feedback",
        "ask_user"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "ProvideFeedbackTool::name",
        "ProvideFeedbackTool::description",
        "ProvideFeedbackTool::parameters_schema",
        "ProvideFeedbackTool::execute",
        "AskUserTool::name",
        "AskUserTool::description",
        "AskUserTool::parameters_schema",
        "AskUserTool::execute"
      ],
      "name": "control_tools.rs",
      "source_summary": "// Control tools - provide_feedback, ask_user, etc. (Session-scoped)\nuse crate::data::*;\nuse crate::storage::*;\nuse adk_core::{Tool, ToolContext};\n\nuse async_trait::async_trait;\nuse dialoguer::{Confirm, Input};\nuse serde_json::{json, Value};\nuse std::sync::Arc;\n\n// ============================================================================\n// ProvideFeedbackTool\n// ============================================================================\n\npub struct ProvideFeedbackTool {\n    session_id: String,\n}\n\nimpl ProvideFeedbackTool {\n    pub fn new(session_id: String) -> Self {\n        Self { session_id }\n    }\n}\n\n#[async_trait]\nimpl Tool for ProvideFeedbackTool {\n    fn name(&self) -> &str {\n        \"provide_feedback\"\n    }\n\n    fn description(&self) -> &str {\n        \"Provide structured feedback to the Actor agent. \\\n         This feedback will be visible to the Actor in the next iteration.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"feedback_type\": {\n                    \"type\": \"string\",\n                    \"enum\": [\"build_error\", \"quality_issue\", \"missing_requirement\", \"suggestion\"],\n                },\n                \"severity\": {\n                    \"type\": \"string\",\n                    \"enum\": [\"critical\", \"major\", \"minor\"],\n                },\n                \"details\": {\"type\": \"string\"},\n                \"suggested_fix\": {\"type\": \"string\"}\n            },\n            \"required\": [\"feedback_type\", \"severity\", \"details\"]\n        }))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {\n        let feedback_type = match args[\"feedback_type\"].as_str().unwrap() {\n            \"build_error\" => FeedbackType::BuildError,\n            \"quality_issue\" => FeedbackType::QualityIssue,\n            \"missing_requirement\" => FeedbackType::MissingRequirement,\n            _ => FeedbackType::Suggestion,\n        };\n\n        let severity = match args[\"severity\"].as_str().unwrap() {\n            \"critical\" => Severity::Critical,\n            \"major\" => Severity::Major,\n            _ => Severity::Minor,\n        };\n\n        let feedback = Feedback {\n            feedback_type,\n            severity,\n            details: args[\"details\"].as_str().unwrap().to_string(),\n            suggested_fix: args\n                .get(\"suggested_fix\")\n                .and_then(|v| v.as_str())\n                .map(String::from),\n            timestamp: chrono::Utc::now(),\n        };\n\n        append_feedback(&self.session_id, &feedback).map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;\n\n        Ok(json!({\n            \"status\": \"feedback_recorded\",\n            \"message\": \"Feedback will be available to Actor in next iteration\"\n        }))\n    }\n}\n\n// ============================================================================\n// AskUserTool\n// ============================================================================\n\npub struct AskUserTool;\n\n#[async_trait]\nimpl Tool for AskUserTool {\n    fn name(&self) -> &str {\n        \"ask_user\"\n    }\n\n    fn description(&self) -> &str {\n        \"Ask the user for confirmation or input via CLI interface.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"question\": {\n                    \"type\": \"string\",\n                    \"description\": \"The question to ask the user\"\n                },\n                \"question_type\": {\n                    \"type\": \"string\",\n                    \"enum\": [\"yes_no\", \"text_input\"],\n                    \"description\": \"Type of question\"\n                }\n            },\n            \"required\": [\"question\", \"question_type\"]\n        }))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {\n        let question = args[\"question\"].as_str().unwrap();\n        let question_type = args[\"question_type\"].as_str().unwrap();\n\n        match question_type {\n            \"yes_no\" => {\n                let answer = Confirm::new()\n                    .with_prompt(question)\n                    .default(false)\n                    .interact()\n                    .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;\n\n                Ok(json!({\n                    \"answer\": answer,\n                    \"answer_type\": \"boolean\"\n                }))\n            }\n            \"text_input\" => {\n                let answer: String = Input::new()\n                    .with_prompt(question)\n                    .interact_text()\n                    .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;\n\n                Ok(json!({\n                    \"answer\": answer,\n                    \"answer_type\": \"text\"\n                }))\n            }\n            _ => Ok(json!({\"error\": \"Invalid question type\"})),\n        }\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 7.0,
      "lines_of_code": 154,
      "number_of_classes": 2,
      "number_of_functions": 2
    },
    "dependencies": [
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 2,
        "name": "crate::data",
        "path": "crate::data",
        "version": null
      },
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 3,
        "name": "crate::storage",
        "path": "crate::storage",
        "version": null
      },
      {
        "dependency_type": "external",
        "is_external": true,
        "line_number": 4,
        "name": "adk_core",
        "path": "adk_core",
        "version": null
      },
      {
        "dependency_type": "external",
        "is_external": true,
        "line_number": 6,
        "name": "async_trait",
        "path": "async_trait",
        "version": null
      },
      {
        "dependency_type": "external",
        "is_external": true,
        "line_number": 7,
        "name": "dialoguer",
        "path": "dialoguer",
        "version": null
      },
      {
        "dependency_type": "external",
        "is_external": true,
        "line_number": 8,
        "name": "serde_json",
        "path": "serde_json",
        "version": null
      },
      {
        "dependency_type": "standard_library",
        "is_external": false,
        "line_number": 9,
        "name": "std::sync::Arc",
        "path": "std::sync::Arc",
        "version": null
      }
    ],
    "detailed_description": "This component implements session-scoped control tools for agent systems. It contains two main tools: ProvideFeedbackTool for structured feedback collection and AskUserTool for interactive user queries. The feedback tool allows categorizing feedback by type (build_error, quality_issue, missing_requirement, suggestion) and severity (critical, major, minor), storing it for actor agents. The user interaction tool supports yes/no confirmations and text input collection via CLI interface. Both tools implement the Tool trait with proper parameter schemas and async execution.",
    "interfaces": [
      {
        "description": "Returns the tool name identifier",
        "interface_type": "method",
        "name": "name",
        "parameters": [],
        "return_type": "&str",
        "visibility": "public"
      },
      {
        "description": "Returns tool functionality description",
        "interface_type": "method",
        "name": "description",
        "parameters": [],
        "return_type": "&str",
        "visibility": "public"
      },
      {
        "description": "Defines JSON schema for tool parameters",
        "interface_type": "method",
        "name": "parameters_schema",
        "parameters": [],
        "return_type": "Option<Value>",
        "visibility": "public"
      },
      {
        "description": "Asynchronous tool execution method",
        "interface_type": "method",
        "name": "execute",
        "parameters": [
          {
            "description": "Tool execution context",
            "is_optional": false,
            "name": "ctx",
            "param_type": "Arc<dyn ToolContext>"
          },
          {
            "description": "Input parameters as JSON value",
            "is_optional": false,
            "name": "args",
            "param_type": "Value"
          }
        ],
        "return_type": "adk_core::Result<Value>",
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Structured feedback collection and storage for agent systems",
      "Interactive user input handling via CLI interface",
      "Session-scoped tool management and execution",
      "Parameter validation and schema definition for tool operations",
      "Error handling and result reporting for control operations"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "tool",
      "description": "Artifact operation tools for Delivery Agent (Session-scoped)",
      "file_path": "crates/cowork-core/src/tools/artifact_tools.rs",
      "functions": [
        "SaveDeliveryReportTool::new",
        "SaveDeliveryReportTool::execute",
        "SavePrdDocTool::new",
        "SavePrdDocTool::execute",
        "SaveDesignDocTool::new",
        "SaveDesignDocTool::execute",
        "LoadFeedbackHistoryTool::new",
        "LoadFeedbackHistoryTool::execute"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "SaveDeliveryReportTool",
        "SavePrdDocTool",
        "SaveDesignDocTool",
        "LoadFeedbackHistoryTool"
      ],
      "name": "artifact_tools.rs",
      "source_summary": "// Artifact operation tools for Delivery Agent (Session-scoped)\nuse crate::storage::*;\nuse adk_core::{Tool, ToolContext};\nuse async_trait::async_trait;\nuse serde_json::{json, Value};\nuse std::sync::Arc;\n\n// ============================================================================\n// SaveDeliveryReportTool\n// ============================================================================\n\npub struct SaveDeliveryReportTool {\n    session_id: String,\n}\n\nimpl SaveDeliveryReportTool {\n    pub fn new(session_id: String) -> Self {\n        Self { session_id }\n    }\n}\n\n#[async_trait]\nimpl Tool for SaveDeliveryReportTool {\n    fn name(&self) -> &str {\n        \"save_delivery_report\"\n    }\n\n    fn description(&self) -> &str {\n        \"Save the delivery report markdown document.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"content\": {\n                    \"type\": \"string\",\n                    \"description\": \"Markdown content of the delivery report\"\n                }\n            },\n            \"required\": [\"content\"]\n        }))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {\n        let content = args[\"content\"].as_str()\n            .or_else(|| args[\" content\"].as_str()) // Handle LLM adding space before key\n            .ok_or_else(|| adk_core::AdkError::Tool(\"Missing 'content' parameter\".to_string()))?;\n        \n        save_delivery_report(&self.session_id, content)\n            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;\n\n        Ok(json!({\n            \"status\": \"success\",\n            \"message\": \"Delivery report saved successfully\"\n        }))\n    }\n}\n\n// ============================================================================\n// SavePrdDocTool\n// ============================================================================\n\npub struct SavePrdDocTool {\n    session_id: String,\n}\n\nimpl SavePrdDocTool {\n    pub fn new(session_id: String) -> Self {\n        Self { session_id }\n    }\n}\n\n#[async_trait]\nimpl Tool for SavePrdDocTool {\n    fn name(&self) -> &str {\n        \"save_prd_doc\"\n    }\n\n    fn description(&self) -> &str {\n        \"Save the PRD (Product Requirements Document) markdown file.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"content\": {\n                    \"type\": \"string\",\n                    \"description\": \"Markdown content of the PRD document\"\n                }\n            },\n            \"required\": [\"content\"]\n        }))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {\n        let content = args[\"content\"].as_str()\n            .or_else(|| args[\" content\"].as_str()) // Handle LLM adding space before key\n            .ok_or_else(|| adk_core::AdkError::Tool(\"Missing 'content' parameter\".to_string()))?;\n        \n        save_prd_doc(&self.session_id, content)\n            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;\n\n        Ok(json!({\n            \"status\": \"success\",\n            \"message\": \"PRD document saved successfully\"\n        }))\n    }\n}\n\n// ============================================================================\n// SaveDesignDocTool\n// ============================================================================\n\npub struct SaveDesignDocTool {\n    session_id: String,\n}\n\nimpl SaveDesignDocTool {\n    pub fn new(session_id: String) -> Self {\n        Self { session_id }\n    }\n}\n\n#[async_trait]\nimpl Tool for SaveDesignDocTool {\n    fn name(&self) -> &str {\n        \"save_design_doc\"\n    }\n\n    fn description(&self) -> &str {\n        \"Save the Design Document markdown file.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\n            \"type\": \"object\",\n            \"properties\": {\n                \"content\": {\n                    \"type\": \"string\",\n                    \"description\": \"Markdown content of the design document\"\n                }\n            },\n            \"required\": [\"content\"]\n        }))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> adk_core::Result<Value> {\n        let content = args[\"content\"].as_str()\n            .or_else(|| args[\" content\"].as_str()) // Handle LLM adding space before key\n            .ok_or_else(|| adk_core::AdkError::Tool(\"Missing 'content' parameter\".to_string()))?;\n        \n        save_design_doc(&self.session_id, content)\n            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;\n\n        Ok(json!({\n            \"status\": \"success\",\n            \"message\": \"Design document saved successfully\"\n        }))\n    }\n}\n\n// ============================================================================\n// LoadFeedbackHistoryTool\n// ============================================================================\n\npub struct LoadFeedbackHistoryTool {\n    session_id: String,\n}\n\nimpl LoadFeedbackHistoryTool {\n    pub fn new(session_id: String) -> Self {\n        Self { session_id }\n    }\n}\n\n#[async_trait]\nimpl Tool for LoadFeedbackHistoryTool {\n    fn name(&self) -> &str {\n        \"load_feedback_history\"\n    }\n\n    fn description(&self) -> &str {\n        \"Load the feedback history from all development stages.\"\n    }\n\n    fn parameters_schema(&self) -> Option<Value> {\n        Some(json!({\n            \"type\": \"object\",\n            \"properties\": {}\n        }))\n    }\n\n    async fn execute(&self, _ctx: Arc<dyn ToolContext>, _args: Value) -> adk_core::Result<Value> {\n        let history = load_feedback_history(&self.session_id)\n            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?;\n\n        Ok(serde_json::to_value(history)\n            .map_err(|e| adk_core::AdkError::Tool(e.to_string()))?)\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 6.0,
      "lines_of_code": 202,
      "number_of_classes": 4,
      "number_of_functions": 8
    },
    "dependencies": [
      {
        "dependency_type": "internal_module",
        "is_external": false,
        "line_number": 2,
        "name": "crate::storage",
        "path": "crate::storage::*",
        "version": null
      },
      {
        "dependency_type": "external_library",
        "is_external": true,
        "line_number": 3,
        "name": "adk_core",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "external_library",
        "is_external": true,
        "line_number": 4,
        "name": "async_trait",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "external_library",
        "is_external": true,
        "line_number": 5,
        "name": "serde_json",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "standard_library",
        "is_external": false,
        "line_number": 6,
        "name": "std::sync::Arc",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "This component provides session-scoped artifact operation tools for a Delivery Agent system. It contains four main tools: SaveDeliveryReportTool for saving delivery reports, SavePrdDocTool for saving Product Requirements Documents, SaveDesignDocTool for saving design documents, and LoadFeedbackHistoryTool for loading feedback history. Each tool implements the Tool trait with async execution capabilities and follows a consistent pattern of accepting markdown content parameters for saving operations.",
    "interfaces": [
      {
        "description": "Tool for saving delivery report markdown documents",
        "interface_type": "struct",
        "name": "SaveDeliveryReportTool",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Tool for saving Product Requirements Document markdown files",
        "interface_type": "struct",
        "name": "SavePrdDocTool",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Tool for saving Design Document markdown files",
        "interface_type": "struct",
        "name": "SaveDesignDocTool",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Tool for loading feedback history from development stages",
        "interface_type": "struct",
        "name": "LoadFeedbackHistoryTool",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Provide artifact storage operations for delivery reports",
      "Handle PRD document persistence",
      "Manage design document storage operations",
      "Load and retrieve feedback history data",
      "Implement Tool interface for agent integration"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "middleware",
      "description": "Rate-limited LLM wrapper middleware that introduces delays before API calls",
      "file_path": "crates/cowork-core/src/llm/rate_limiter.rs",
      "functions": [
        "new",
        "with_default_delay",
        "name",
        "generate_content"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "RateLimitedLlm::new",
        "RateLimitedLlm::with_default_delay",
        "RateLimitedLlm::name",
        "RateLimitedLlm::generate_content",
        "Llm::name",
        "Llm::generate_content"
      ],
      "name": "rate_limiter.rs",
      "source_summary": "// Rate-limited LLM wrapper\nuse adk_core::{Llm, LlmRequest, LlmResponseStream};\nuse async_trait::async_trait;\nuse std::sync::Arc;\nuse tokio::time::{sleep, Duration};\n\n/// A wrapper around any Llm implementation that adds rate limiting\n/// by introducing a delay before each API call.\npub struct RateLimitedLlm {\n    inner: Arc<dyn Llm>,\n    delay_ms: u64,\n}\n\nimpl RateLimitedLlm {\n    /// Create a new rate-limited LLM wrapper\n    ///\n    /// # Arguments\n    /// * `inner` - The underlying LLM implementation\n    /// * `delay_ms` - Delay in milliseconds before each API call\n    pub fn new(inner: Arc<dyn Llm>, delay_ms: u64) -> Self {\n        Self { inner, delay_ms }\n    }\n\n    /// Create with 2-second delay (for <30 calls per minute limit)\n    pub fn with_default_delay(inner: Arc<dyn Llm>) -> Self {\n        Self::new(inner, 2000) // 2 seconds = 2000ms\n    }\n}\n\n#[async_trait]\nimpl Llm for RateLimitedLlm {\n    fn name(&self) -> &str {\n        self.inner.name()\n    }\n\n    async fn generate_content(\n        &self,\n        req: LlmRequest,\n        stream: bool,\n    ) -> adk_core::Result<LlmResponseStream> {\n        // Wait before making the API call\n        sleep(Duration::from_millis(self.delay_ms)).await;\n        \n        // Delegate to the inner LLM\n        self.inner.generate_content(req, stream).await\n    }\n}\n\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 3.0,
      "lines_of_code": 48,
      "number_of_classes": 1,
      "number_of_functions": 4
    },
    "dependencies": [
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": 2,
        "name": "adk_core",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": 3,
        "name": "async_trait",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "standard_library",
        "is_external": false,
        "line_number": 4,
        "name": "std::sync::Arc",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "crate",
        "is_external": true,
        "line_number": 5,
        "name": "tokio::time",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "This component implements a rate limiting middleware wrapper for LLM (Large Language Model) implementations. It wraps any LLM implementation and adds a configurable delay before each API call to enforce rate limiting. The component provides two constructors: one with custom delay configuration and another with a default 2-second delay suitable for <30 calls per minute rate limits. The middleware implements the Llm trait, delegating actual LLM operations to the wrapped implementation while enforcing rate limiting through pre-call delays.",
    "interfaces": [
      {
        "description": "Create a new rate-limited LLM wrapper with custom delay",
        "interface_type": "constructor",
        "name": "new",
        "parameters": [
          {
            "description": "The underlying LLM implementation",
            "is_optional": false,
            "name": "inner",
            "param_type": "Arc<dyn Llm>"
          },
          {
            "description": "Delay in milliseconds before each API call",
            "is_optional": false,
            "name": "delay_ms",
            "param_type": "u64"
          }
        ],
        "return_type": "Self",
        "visibility": "public"
      },
      {
        "description": "Create with 2-second delay (for <30 calls per minute limit)",
        "interface_type": "constructor",
        "name": "with_default_delay",
        "parameters": [
          {
            "description": "The underlying LLM implementation",
            "is_optional": false,
            "name": "inner",
            "param_type": "Arc<dyn Llm>"
          }
        ],
        "return_type": "Self",
        "visibility": "public"
      },
      {
        "description": "Get the name of the wrapped LLM implementation",
        "interface_type": "method",
        "name": "name",
        "parameters": [
          {
            "description": "Reference to self",
            "is_optional": false,
            "name": "self",
            "param_type": "&Self"
          }
        ],
        "return_type": "&str",
        "visibility": "public"
      },
      {
        "description": "Generate content with rate limiting delay",
        "interface_type": "method",
        "name": "generate_content",
        "parameters": [
          {
            "description": "Reference to self",
            "is_optional": false,
            "name": "self",
            "param_type": "&Self"
          },
          {
            "description": "LLM request object",
            "is_optional": false,
            "name": "req",
            "param_type": "LlmRequest"
          },
          {
            "description": "Whether to stream the response",
            "is_optional": false,
            "name": "stream",
            "param_type": "bool"
          }
        ],
        "return_type": "adk_core::Result<LlmResponseStream>",
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Enforce rate limiting on LLM API calls by introducing configurable delays",
      "Wrap and delegate to underlying LLM implementations transparently",
      "Provide rate limiting configuration flexibility through multiple constructors",
      "Maintain compatibility with Llm trait interface while adding rate limiting functionality",
      "Ensure thread-safe operation through Arc-based shared ownership"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": "Human-in-the-Loop (HITL) resilient agent wrapper that provides error recovery and user intervention capabilities",
      "file_path": "crates/cowork-core/src/agents/hitl.rs",
      "functions": [
        "new",
        "name",
        "description",
        "sub_agents",
        "run",
        "handle_error",
        "start_retry"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "ResilientAgent::new",
        "ResilientAgent::name",
        "ResilientAgent::description",
        "ResilientAgent::sub_agents",
        "ResilientAgent::run",
        "ResilientAgent::handle_error",
        "ResilientStream::new",
        "ResilientStream::start_retry",
        "ResilientStream::poll_next"
      ],
      "name": "hitl.rs",
      "source_summary": "use adk_core::{Agent, Event, AdkError, InvocationContext};\nuse async_trait::async_trait;\nuse std::sync::Arc;\nuse std::pin::Pin;\nuse std::task::{Context as TaskContext, Poll};\nuse futures::{Stream, Future};\nuse dialoguer::{Select, Input, theme::ColorfulTheme};\n\npub struct ResilientAgent {\n    inner: Arc<dyn Agent>,\n    subs: Vec<Arc<dyn Agent>>,\n}\n\nimpl ResilientAgent {\n    pub fn new(inner: Arc<dyn Agent>) -> Self {\n        Self {\n            inner: inner.clone(),\n            subs: vec![inner],\n        }\n    }\n}\n\ntype AgentOutput = Pin<Box<dyn Stream<Item = Result<Event, AdkError>> + Send>>;\n\n#[async_trait]\nimpl Agent for ResilientAgent {\n    fn name(&self) -> &str {\n        self.inner.name()\n    }\n\n    fn description(&self) -> &str {\n        self.inner.description()\n    }\n\n    fn sub_agents(&self) -> &[Arc<dyn Agent>] {\n        &self.subs\n    }\n\n    async fn run(&self, context: Arc<dyn InvocationContext>) -> Result<AgentOutput, AdkError> {\n        // Initial run\n        match self.inner.run(context.clone()).await {\n            Ok(stream) => {\n                // Wrap the stream to handle errors during iteration\n                Ok(Box::pin(ResilientStream::new(\n                    self.inner.clone(),\n                    context,\n                    stream\n                )))\n            },\n            Err(e) => {\n                // Handle immediate errors (same logic as before, but adapted for consistency)\n                // We can't use the Stream wrapper here easily without a stream.\n                // But we can just use the sync logic here since we are in async fn.\n                let err_msg = e.to_string();\n                if err_msg.contains(\"Max iterations\") {\n                     return self.handle_error(context, e).await;\n                }\n                Err(e)\n            }\n        }\n    }\n}\n\nimpl ResilientAgent {\n    // Helper for immediate errors (recursion in async fn)\n    async fn handle_error(&self, context: Arc<dyn InvocationContext>, e: AdkError) -> Result<AgentOutput, AdkError> {\n         println!(\"\\n⚠️  Agent '{}' encountered error: {}\", self.name(), e);\n         println!(\"The agent loop limit has been exceeded.\");\n         \n         let selections = &[\"Retry (reset counter)\", \"Provide Guidance & Retry\", \"Abort\"];\n         let selection = Select::with_theme(&ColorfulTheme::default())\n            .with_prompt(\"How would you like to proceed?\")\n            .default(0)\n            .items(&selections[..])\n            .interact()\n            .unwrap_or(2);\n\n         match selection {\n            0 => {\n                println!(\"🔄 Retrying agent execution...\");\n                return self.run(context).await;\n            },\n            1 => {\n                let input: String = Input::with_theme(&ColorfulTheme::default())\n                    .with_prompt(\"Please provide guidance for the agent\")\n                    .interact_text()\n                    .unwrap_or_default();\n                \n                if !input.is_empty() {\n                    println!(\"(Note: User guidance provided: '{}' - but context injection is not implemented. Retrying anyway.)\", input);\n                }\n                println!(\"🔄 Retrying with new guidance...\");\n                return self.run(context).await;\n            },\n            _ => return Err(e),\n         }\n    }\n}\n\n// ============================================================================\n// ResilientStream Implementation\n// ============================================================================\n\nenum StreamState {\n    Streaming(AgentOutput),\n    Retrying(Pin<Box<dyn Future<Output = Result<AgentOutput, AdkError>> + Send>>),\n}\n\nstruct ResilientStream {\n    inner_agent: Arc<dyn Agent>,\n    context: Arc<dyn InvocationContext>,\n    state: StreamState,\n    agent_name: String, // Cached for logging\n}\n\nimpl ResilientStream {\n    fn new(\n        inner_agent: Arc<dyn Agent>,\n        context: Arc<dyn InvocationContext>,\n        stream: AgentOutput,\n    ) -> Self {\n        let agent_name = inner_agent.name().to_string();\n        Self {\n            inner_agent,\n            context,\n            state: StreamState::Streaming(stream),\n            agent_name,\n        }\n    }\n\n    fn start_retry(&mut self) {\n        let agent = self.inner_agent.clone();\n        let ctx = self.context.clone();\n        // Create the future for running the agent again\n        let fut = Box::pin(async move {\n            agent.run(ctx).await\n        });\n        self.state = StreamState::Retrying(fut);\n    }\n}\n\nimpl Stream for ResilientStream {\n    type Item = Result<Event, AdkError>;\n\n    fn poll_next(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {\n        loop {\n            match &mut self.state {\n                StreamState::Streaming(stream) => {\n                    match stream.as_mut().poll_next(cx) {\n                        Poll::Ready(Some(Err(e))) => {\n                            // Intercept error\n                            let err_msg = e.to_string();\n                            if err_msg.contains(\"Max iterations\") {\n                                println!(\"\\n⚠️  Agent '{}' encountered error during stream: {}\", self.agent_name, err_msg);\n                                println!(\"The agent loop limit has been exceeded.\");\n                                \n                                // Blocking interaction\n                                let selections = &[\"Retry (reset counter)\", \"Provide Guidance & Retry\", \"Abort\"];\n                                let selection = Select::with_theme(&ColorfulTheme::default())\n                                    .with_prompt(\"How would you like to proceed?\")\n                                    .default(0)\n                                    .items(&selections[..])\n                                    .interact()\n                                    .unwrap_or(2);\n\n                                match selection {\n                                    0 => {\n                                        println!(\"🔄 Retrying agent execution...\");\n                                        self.start_retry();\n                                        continue; // Loop to poll the new state\n                                    },\n                                    1 => {\n                                        let input: String = Input::with_theme(&ColorfulTheme::default())\n                                            .with_prompt(\"Please provide guidance for the agent\")\n                                            .interact_text()\n                                            .unwrap_or_default();\n                                        if !input.is_empty() {\n                                            println!(\"(Note: User guidance provided: '{}' - but context injection is not implemented. Retrying anyway.)\", input);\n                                        }\n                                        println!(\"🔄 Retrying with new guidance...\");\n                                        self.start_retry();\n                                        continue;\n                                    },\n                                    _ => return Poll::Ready(Some(Err(e))),\n                                }\n                            }\n                            return Poll::Ready(Some(Err(e)));\n                        },\n                        Poll::Ready(other) => return Poll::Ready(other),\n                        Poll::Pending => return Poll::Pending,\n                    }\n                },\n                StreamState::Retrying(fut) => {\n                    match fut.as_mut().poll(cx) {\n                        Poll::Ready(Ok(new_stream)) => {\n                            // Retry successful, got new stream\n                            // Wrap it recursively? No, just replace current stream\n                            // But wait, the new stream might also fail later.\n                            // So we just go back to Streaming state with the new stream.\n                            self.state = StreamState::Streaming(new_stream);\n                            continue; // Loop to poll the new stream\n                        },\n                        Poll::Ready(Err(e)) => {\n                            // Retry failed immediately\n                            // We could offer HITL again here, but let's just error out for now to avoid infinite loops of immediate errors\n                            // Or better: recurse logic?\n                            // For simplicity, return the error.\n                            return Poll::Ready(Some(Err(e)));\n                        },\n                        Poll::Pending => return Poll::Pending,\n                    }\n                }\n            }\n        }\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 20.0,
      "lines_of_code": 216,
      "number_of_classes": 2,
      "number_of_functions": 7
    },
    "dependencies": [
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 1,
        "name": "adk_core",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 2,
        "name": "async_trait",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "standard_library",
        "is_external": false,
        "line_number": 3,
        "name": "std::sync::Arc",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "standard_library",
        "is_external": false,
        "line_number": 4,
        "name": "std::pin::Pin",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "standard_library",
        "is_external": false,
        "line_number": 5,
        "name": "std::task",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 6,
        "name": "futures",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 7,
        "name": "dialoguer",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "The hitl.rs component implements a resilient agent wrapper that provides Human-in-the-Loop (HITL) capabilities for error recovery. It wraps an existing agent and intercepts errors during execution, particularly 'Max iterations' errors, to allow human intervention. The component provides three recovery options: retry with reset counter, provide guidance and retry, or abort. It implements a custom stream wrapper (ResilientStream) that can handle errors during stream processing and restart execution with user guidance.",
    "interfaces": [
      {
        "description": "Creates a new ResilientAgent wrapper",
        "interface_type": "constructor",
        "name": "new",
        "parameters": [
          {
            "description": "The inner agent to wrap with HITL capabilities",
            "is_optional": false,
            "name": "inner",
            "param_type": "Arc<dyn Agent>"
          }
        ],
        "return_type": "Self",
        "visibility": "public"
      },
      {
        "description": "Returns the name of the wrapped agent",
        "interface_type": "method",
        "name": "name",
        "parameters": [],
        "return_type": "&str",
        "visibility": "public"
      },
      {
        "description": "Returns the description of the wrapped agent",
        "interface_type": "method",
        "name": "description",
        "parameters": [],
        "return_type": "&str",
        "visibility": "public"
      },
      {
        "description": "Returns the list of sub-agents",
        "interface_type": "method",
        "name": "sub_agents",
        "parameters": [],
        "return_type": "&[Arc<dyn Agent>]",
        "visibility": "public"
      },
      {
        "description": "Executes the agent with error recovery capabilities",
        "interface_type": "method",
        "name": "run",
        "parameters": [
          {
            "description": "The invocation context for agent execution",
            "is_optional": false,
            "name": "context",
            "param_type": "Arc<dyn InvocationContext>"
          }
        ],
        "return_type": "Result<AgentOutput, AdkError>",
        "visibility": "public"
      },
      {
        "description": "Handles immediate execution errors with user interaction",
        "interface_type": "method",
        "name": "handle_error",
        "parameters": [
          {
            "description": "The invocation context",
            "is_optional": false,
            "name": "context",
            "param_type": "Arc<dyn InvocationContext>"
          },
          {
            "description": "The error that occurred",
            "is_optional": false,
            "name": "e",
            "param_type": "AdkError"
          }
        ],
        "return_type": "Result<AgentOutput, AdkError>",
        "visibility": "private"
      }
    ],
    "responsibilities": [
      "Error interception and handling for agent execution failures",
      "Human-in-the-Loop interaction for error recovery decisions",
      "Stream resilience with retry mechanisms during asynchronous processing",
      "Agent lifecycle management with recovery capabilities",
      "User guidance integration for improved agent performance"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": "Agents module providing factory functions for creating intelligent agents using adk-rust framework",
      "file_path": "crates/cowork-core/src/agents/mod.rs",
      "functions": [
        "create_idea_agent",
        "create_prd_loop",
        "create_design_loop",
        "create_plan_loop",
        "create_coding_loop",
        "create_check_agent",
        "create_delivery_agent"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "create_idea_agent",
        "create_prd_loop",
        "create_design_loop",
        "create_plan_loop",
        "create_coding_loop",
        "create_check_agent",
        "create_delivery_agent"
      ],
      "name": "mod.rs",
      "source_summary": "// Agents module - Agent builders using adk-rust\n// \n// IMPORTANT: This file solves a CRITICAL bug where SequentialAgent stops after\n// the first LoopAgent completes. \n//\n// PROBLEM: When a sub-agent in LoopAgent calls exit_loop(), it terminates the\n// ENTIRE SequentialAgent, not just the LoopAgent. This is adk-rust's design.\n//\n// SOLUTION: Remove exit_loop tools and use max_iterations=1 to let LoopAgent\n// complete naturally, allowing SequentialAgent to continue to next agent.\n\nuse crate::instructions::*;\nuse crate::tools::*;\nuse adk_agent::{LlmAgentBuilder, LoopAgent};\nuse adk_core::{Llm, IncludeContents};\nuse anyhow::Result;\nuse std::sync::Arc;\n\nmod hitl;\nuse hitl::ResilientAgent;\n\n// ============================================================================\n// IdeaAgent - Simple agent to capture initial idea\n// ============================================================================\n\npub fn create_idea_agent(model: Arc<dyn Llm>, session_id: &str) -> Result<Arc<dyn adk_core::Agent>> {\n    let agent = LlmAgentBuilder::new(\"idea_agent\")\n        .instruction(IDEA_AGENT_INSTRUCTION)\n        .model(model)\n        .tool(Arc::new(SaveIdeaTool::new(session_id.to_string())))\n        .tool(Arc::new(LoadIdeaTool::new(session_id.to_string())))\n        .tool(Arc::new(ReviewAndEditContentTool))\n        .include_contents(IncludeContents::None)\n        .build()?;\n\n    Ok(Arc::new(agent))\n}\n\n// ============================================================================\n// PRD Loop - Actor + Critic with LoopAgent\n// ============================================================================\n\npub fn create_prd_loop(model: Arc<dyn Llm>, session_id: &str) -> Result<Arc<dyn adk_core::Agent>> {\n    let session = session_id.to_string();\n    \n    let prd_actor = LlmAgentBuilder::new(\"prd_actor\")\n        .instruction(PRD_ACTOR_INSTRUCTION)\n        .model(model.clone())\n        .tool(Arc::new(LoadIdeaTool::new(session.clone())))\n        .tool(Arc::new(ReviewWithFeedbackContentTool))\n        .tool(Arc::new(CreateRequirementTool::new(session.clone())))\n        .tool(Arc::new(AddFeatureTool::new(session.clone())))\n        .tool(Arc::new(GetRequirementsTool::new(session.clone())))\n        .tool(Arc::new(SavePrdDocTool::new(session.clone())))\n        .include_contents(IncludeContents::None)\n        .build()?;\n\n    let prd_critic = LlmAgentBuilder::new(\"prd_critic\")\n        .instruction(PRD_CRITIC_INSTRUCTION)\n        .model(model)\n        .tool(Arc::new(ReadFileTool))\n        .tool(Arc::new(GetRequirementsTool::new(session.clone())))\n        .tool(Arc::new(ProvideFeedbackTool::new(session.clone())))\n        .include_contents(IncludeContents::None)\n        .build()?;\n\n    let mut loop_agent = LoopAgent::new(\n        \"prd_loop\",\n        vec![Arc::new(prd_actor), Arc::new(prd_critic)],\n    );\n    loop_agent = loop_agent.with_max_iterations(3); // Allow up to 3 attempts for Actor to fix issues\n\n    Ok(Arc::new(ResilientAgent::new(Arc::new(loop_agent))))\n}\n\n// ============================================================================\n// Design Loop - Actor + Critic\n// ============================================================================\n\npub fn create_design_loop(model: Arc<dyn Llm>, session_id: &str) -> Result<Arc<dyn adk_core::Agent>> {\n    let session = session_id.to_string();\n    \n    let design_actor = LlmAgentBuilder::new(\"design_actor\")\n        .instruction(DESIGN_ACTOR_INSTRUCTION)\n        .model(model.clone())\n        .tool(Arc::new(GetRequirementsTool::new(session.clone())))\n        .tool(Arc::new(GetDesignTool::new(session.clone())))\n        .tool(Arc::new(ReviewWithFeedbackContentTool))\n        .tool(Arc::new(CreateDesignComponentTool::new(session.clone())))\n        .tool(Arc::new(SaveDesignDocTool::new(session.clone())))\n        .include_contents(IncludeContents::None)\n        .build()?;\n\n    let design_critic = LlmAgentBuilder::new(\"design_critic\")\n        .instruction(DESIGN_CRITIC_INSTRUCTION)\n        .model(model)\n        .tool(Arc::new(ReadFileTool))\n        .tool(Arc::new(GetRequirementsTool::new(session.clone())))\n        .tool(Arc::new(GetDesignTool::new(session.clone())))\n        .tool(Arc::new(CheckFeatureCoverageTool::new(session.clone())))\n        .tool(Arc::new(ProvideFeedbackTool::new(session.clone())))\n        .include_contents(IncludeContents::None)\n        .build()?;\n\n    let mut loop_agent = LoopAgent::new(\"design_loop\", vec![Arc::new(design_actor), Arc::new(design_critic)]);\n    loop_agent = loop_agent.with_max_iterations(3); // Allow up to 3 attempts\n\n    Ok(Arc::new(ResilientAgent::new(Arc::new(loop_agent))))\n}\n\n// ============================================================================\n// Plan Loop - Actor + Critic\n// ============================================================================\n\npub fn create_plan_loop(model: Arc<dyn Llm>, session_id: &str) -> Result<Arc<dyn adk_core::Agent>> {\n    let session = session_id.to_string();\n    \n    let plan_actor = LlmAgentBuilder::new(\"plan_actor\")\n        .instruction(PLAN_ACTOR_INSTRUCTION)\n        .model(model.clone())\n        .tool(Arc::new(GetRequirementsTool::new(session.clone())))\n        .tool(Arc::new(GetDesignTool::new(session.clone())))\n        .tool(Arc::new(GetPlanTool::new(session.clone())))\n        .tool(Arc::new(ReviewWithFeedbackContentTool))\n        .tool(Arc::new(CreateTaskTool::new(session.clone())))\n        .include_contents(IncludeContents::None)\n        .build()?;\n\n    let plan_critic = LlmAgentBuilder::new(\"plan_critic\")\n        .instruction(PLAN_CRITIC_INSTRUCTION)\n        .model(model)\n        .tool(Arc::new(GetPlanTool::new(session.clone())))\n        .tool(Arc::new(GetRequirementsTool::new(session.clone())))\n        .tool(Arc::new(GetDesignTool::new(session.clone())))\n        .tool(Arc::new(CheckTaskDependenciesTool::new(session.clone())))\n        .tool(Arc::new(ProvideFeedbackTool::new(session.clone())))\n        .include_contents(IncludeContents::None)\n        .build()?;\n\n    let mut loop_agent = LoopAgent::new(\"plan_loop\", vec![Arc::new(plan_actor), Arc::new(plan_critic)]);\n    loop_agent = loop_agent.with_max_iterations(3); // Allow up to 3 attempts\n\n    Ok(Arc::new(ResilientAgent::new(Arc::new(loop_agent))))\n}\n\n// ============================================================================\n// Coding Loop - Actor + Critic\n// ============================================================================\n\npub fn create_coding_loop(model: Arc<dyn Llm>, session_id: &str) -> Result<Arc<dyn adk_core::Agent>> {\n    let session = session_id.to_string();\n    \n    let coding_actor = LlmAgentBuilder::new(\"coding_actor\")\n        .instruction(CODING_ACTOR_INSTRUCTION)\n        .model(model.clone())\n        .tool(Arc::new(GetPlanTool::new(session.clone())))\n        .tool(Arc::new(UpdateTaskStatusTool::new(session.clone())))\n        .tool(Arc::new(UpdateFeatureStatusTool::new(session.clone())))\n        .tool(Arc::new(ReadFileTool))\n        .tool(Arc::new(WriteFileTool))\n        .tool(Arc::new(ListFilesTool))\n        .tool(Arc::new(RunCommandTool))\n        .tool(Arc::new(CheckTestsTool))\n        .include_contents(IncludeContents::None)\n        .build()?;\n\n    let coding_critic = LlmAgentBuilder::new(\"coding_critic\")\n        .instruction(CODING_CRITIC_INSTRUCTION)\n        .model(model)\n        .tool(Arc::new(GetPlanTool::new(session.clone())))\n        .tool(Arc::new(ReadFileTool))\n        .tool(Arc::new(ListFilesTool))\n        .tool(Arc::new(RunCommandTool))\n        .tool(Arc::new(ProvideFeedbackTool::new(session.clone())))\n        .include_contents(IncludeContents::None)\n        .build()?;\n\n    let mut loop_agent = LoopAgent::new(\"coding_loop\", vec![Arc::new(coding_actor), Arc::new(coding_critic)]);\n    loop_agent = loop_agent.with_max_iterations(5);\n\n    Ok(Arc::new(ResilientAgent::new(Arc::new(loop_agent))))\n}\n\n// ============================================================================\n// Check Agent - Quality assurance\n// ============================================================================\n\npub fn create_check_agent(model: Arc<dyn Llm>, session_id: &str) -> Result<Arc<dyn adk_core::Agent>> {\n    let session = session_id.to_string();\n    \n    let agent = LlmAgentBuilder::new(\"check_agent\")\n        .instruction(CHECK_AGENT_INSTRUCTION)\n        .model(model)\n        .tool(Arc::new(GetRequirementsTool::new(session.clone())))\n        .tool(Arc::new(GetDesignTool::new(session.clone())))\n        .tool(Arc::new(GetPlanTool::new(session.clone())))\n        .tool(Arc::new(CheckDataFormatTool::new(session.clone())))\n        .tool(Arc::new(CheckFeatureCoverageTool::new(session.clone())))\n        .tool(Arc::new(CheckTaskDependenciesTool::new(session.clone())))\n        .tool(Arc::new(RunCommandTool))\n        .tool(Arc::new(ReadFileTool))\n        .tool(Arc::new(ListFilesTool))\n        .tool(Arc::new(CheckTestsTool))\n        .tool(Arc::new(CheckLintTool))\n        .tool(Arc::new(ProvideFeedbackTool::new(session.clone())))\n        .tool(Arc::new(GotoStageTool::new(session.clone())))\n        .include_contents(IncludeContents::None)\n        .build()?;\n\n    Ok(Arc::new(agent))\n}\n\n// ============================================================================\n// Delivery Agent - Final report generation\n// ============================================================================\n\npub fn create_delivery_agent(model: Arc<dyn Llm>, session_id: &str) -> Result<Arc<dyn adk_core::Agent>> {\n    let session = session_id.to_string();\n    \n    let agent = LlmAgentBuilder::new(\"delivery_agent\")\n        .instruction(DELIVERY_AGENT_INSTRUCTION)\n        .model(model)\n        .tool(Arc::new(GetRequirementsTool::new(session.clone())))\n        .tool(Arc::new(GetDesignTool::new(session.clone())))\n        .tool(Arc::new(GetPlanTool::new(session.clone())))\n        .tool(Arc::new(LoadFeedbackHistoryTool::new(session.clone())))\n        .tool(Arc::new(ListFilesTool))\n        .tool(Arc::new(ReadFileTool))\n        .tool(Arc::new(SaveDeliveryReportTool::new(session.clone())))\n        .tool(Arc::new(SavePrdDocTool::new(session.clone())))\n        .tool(Arc::new(SaveDesignDocTool::new(session.clone())))\n        .include_contents(IncludeContents::None)\n        .build()?;\n\n    Ok(Arc::new(agent))\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 2.0,
      "lines_of_code": 236,
      "number_of_classes": 0,
      "number_of_functions": 7
    },
    "dependencies": [
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": 8,
        "name": "crate::instructions",
        "path": "crates/cowork-core/src/instructions",
        "version": null
      },
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": 9,
        "name": "crate::tools",
        "path": "crates/cowork-core/src/tools",
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 10,
        "name": "adk_agent",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 11,
        "name": "adk_core",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 12,
        "name": "anyhow",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "standard_library",
        "is_external": false,
        "line_number": 13,
        "name": "std::sync::Arc",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": 15,
        "name": "hitl",
        "path": "crates/cowork-core/src/agents/hitl.rs",
        "version": null
      },
      {
        "dependency_type": "type",
        "is_external": false,
        "line_number": 16,
        "name": "ResilientAgent",
        "path": "crates/cowork-core/src/agents/hitl.rs",
        "version": null
      }
    ],
    "detailed_description": "This component is a factory module for creating various intelligent agents that form a complete software development workflow. It implements a sophisticated agent orchestration system with actor-critic patterns and human-in-the-loop resilience. The module solves critical architectural issues with LoopAgent termination behavior by using max_iterations instead of exit_loop tools. Each agent serves a specific phase in the development lifecycle: idea generation, PRD creation, design, planning, coding, quality checking, and final delivery.",
    "interfaces": [
      {
        "description": "Creates an agent for capturing initial project ideas",
        "interface_type": "function",
        "name": "create_idea_agent",
        "parameters": [
          {
            "description": "Language model instance",
            "is_optional": false,
            "name": "model",
            "param_type": "Arc<dyn Llm>"
          },
          {
            "description": "Session identifier",
            "is_optional": false,
            "name": "session_id",
            "param_type": "&str"
          }
        ],
        "return_type": "Result<Arc<dyn Agent>>",
        "visibility": "public"
      },
      {
        "description": "Creates actor-critic loop for PRD (Product Requirements Document) creation",
        "interface_type": "function",
        "name": "create_prd_loop",
        "parameters": [
          {
            "description": "Language model instance",
            "is_optional": false,
            "name": "model",
            "param_type": "Arc<dyn Llm>"
          },
          {
            "description": "Session identifier",
            "is_optional": false,
            "name": "session_id",
            "param_type": "&str"
          }
        ],
        "return_type": "Result<Arc<dyn Agent>>",
        "visibility": "public"
      },
      {
        "description": "Creates actor-critic loop for design specification creation",
        "interface_type": "function",
        "name": "create_design_loop",
        "parameters": [
          {
            "description": "Language model instance",
            "is_optional": false,
            "name": "model",
            "param_type": "Arc<dyn Llm>"
          },
          {
            "description": "Session identifier",
            "is_optional": false,
            "name": "session_id",
            "param_type": "&str"
          }
        ],
        "return_type": "Result<Arc<dyn Agent>>",
        "visibility": "public"
      },
      {
        "description": "Creates actor-critic loop for task planning and scheduling",
        "interface_type": "function",
        "name": "create_plan_loop",
        "parameters": [
          {
            "description": "Language model instance",
            "is_optional": false,
            "name": "model",
            "param_type": "Arc<dyn Llm>"
          },
          {
            "description": "Session identifier",
            "is_optional": false,
            "name": "session_id",
            "param_type": "&str"
          }
        ],
        "return_type": "Result<Arc<dyn Agent>>",
        "visibility": "public"
      },
      {
        "description": "Creates actor-critic loop for code implementation",
        "interface_type": "function",
        "name": "create_coding_loop",
        "parameters": [
          {
            "description": "Language model instance",
            "is_optional": false,
            "name": "model",
            "param_type": "Arc<dyn Llm>"
          },
          {
            "description": "Session identifier",
            "is_optional": false,
            "name": "session_id",
            "param_type": "&str"
          }
        ],
        "return_type": "Result<Arc<dyn Agent>>",
        "visibility": "public"
      },
      {
        "description": "Creates quality assurance agent for validation",
        "interface_type": "function",
        "name": "create_check_agent",
        "parameters": [
          {
            "description": "Language model instance",
            "is_optional": false,
            "name": "model",
            "param_type": "Arc<dyn Llm>"
          },
          {
            "description": "Session identifier",
            "is_optional": false,
            "name": "session_id",
            "param_type": "&str"
          }
        ],
        "return_type": "Result<Arc<dyn Agent>>",
        "visibility": "public"
      },
      {
        "description": "Creates final delivery agent for report generation",
        "interface_type": "function",
        "name": "create_delivery_agent",
        "parameters": [
          {
            "description": "Language model instance",
            "is_optional": false,
            "name": "model",
            "param_type": "Arc<dyn Llm>"
          },
          {
            "description": "Session identifier",
            "is_optional": false,
            "name": "session_id",
            "param_type": "&str"
          }
        ],
        "return_type": "Result<Arc<dyn Agent>>",
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Agent Factory Creation - Provides factory functions for creating specialized agents for different development phases",
      "Workflow Orchestration - Implements sequential agent patterns with feedback loops for iterative refinement",
      "Error Resilience - Integrates human-in-the-loop capabilities through ResilientAgent wrapper",
      "Tool Configuration - Properly configures each agent with appropriate tools and instructions",
      "Performance Optimization - Sets appropriate iteration limits for different agent types"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "database",
      "description": "Storage layer for .cowork/ directory - Session-scoped architecture",
      "file_path": "crates/cowork-core/src/storage/mod.rs",
      "functions": [
        "get_cowork_dir",
        "get_session_dir",
        "get_project_root",
        "cowork_dir_exists",
        "is_project_initialized",
        "load_project_index",
        "save_project_index",
        "init_project_index",
        "save_session_input",
        "load_session_input",
        "save_change_request",
        "load_change_request",
        "save_idea",
        "load_idea",
        "save_prd_doc",
        "save_design_doc",
        "save_delivery_report",
        "state_file_exists",
        "has_requirements",
        "has_design_spec",
        "has_implementation_plan",
        "save_requirements",
        "load_requirements",
        "save_feature_list",
        "load_feature_list",
        "save_design_spec",
        "load_design_spec",
        "save_implementation_plan",
        "load_implementation_plan",
        "save_code_metadata",
        "load_code_metadata",
        "save_session_meta",
        "load_session_meta",
        "save_feedback_history",
        "load_feedback_history",
        "append_feedback",
        "save_patch_metadata",
        "load_patch_metadata",
        "init_session_from_base",
        "generate_id",
        "get_latest_successful_session",
        "mark_session_completed",
        "mark_session_failed"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "SessionInput"
      ],
      "name": "mod.rs",
      "source_summary": "// Storage layer for .cowork/ directory - Session-scoped architecture\nuse crate::data::*;\nuse anyhow::{Context, Result};\nuse std::fs;\nuse std::path::{Path, PathBuf};\n\n#[cfg(test)]\nmod storage_test;\n\nconst COWORK_DIR: &str = \".cowork\";\nconst INDEX_FILE: &str = \"index.json\";\nconst SESSIONS_DIR: &str = \"sessions\";\n\n// ============================================================================\n// Core Directory Structure\n// ============================================================================\n\n/// Get the .cowork directory path, create if not exists\npub fn get_cowork_dir() -> Result<PathBuf> {\n    let path = PathBuf::from(COWORK_DIR);\n    \n    // Create main directory and subdirectories\n    fs::create_dir_all(&path)\n        .with_context(|| format!(\"Failed to create .cowork directory at {:?}\", path))?;\n    fs::create_dir_all(path.join(SESSIONS_DIR))?;\n    \n    Ok(path)\n}\n\n/// Get path for a specific session directory\npub fn get_session_dir(session_id: &str) -> Result<PathBuf> {\n    let cowork_dir = get_cowork_dir()?;\n    let session_path = cowork_dir.join(SESSIONS_DIR).join(session_id);\n    \n    // Create session subdirectories\n    fs::create_dir_all(&session_path)?;\n    fs::create_dir_all(session_path.join(\"artifacts\"))?;\n    fs::create_dir_all(session_path.join(\"state\"))?;\n    fs::create_dir_all(session_path.join(\"patch\"))?;\n    fs::create_dir_all(session_path.join(\"logs\"))?;\n    \n    Ok(session_path)\n}\n\n/// Get the project root directory (where .cowork/ is located)\n/// This is the actual workspace where code files are written\npub fn get_project_root() -> Result<PathBuf> {\n    let current_dir = std::env::current_dir()\n        .with_context(|| \"Failed to get current directory\")?;\n    Ok(current_dir)\n}\n\n/// Check if .cowork directory exists\npub fn cowork_dir_exists() -> bool {\n    Path::new(COWORK_DIR).exists()\n}\n\n/// Check if project has been initialized (has index.json)\npub fn is_project_initialized() -> bool {\n    Path::new(COWORK_DIR).join(INDEX_FILE).exists()\n}\n\n// ============================================================================\n// Project Index (index.json at root of .cowork/)\n// ============================================================================\n\npub fn load_project_index() -> Result<ProjectIndex> {\n    let path = PathBuf::from(COWORK_DIR).join(INDEX_FILE);\n    if !path.exists() {\n        anyhow::bail!(\"Project not initialized. Run 'cowork new' first.\");\n    }\n    let content = fs::read_to_string(&path)\n        .with_context(|| format!(\"Failed to read {:?}\", path))?;\n    let index: ProjectIndex = serde_json::from_str(&content)\n        .with_context(|| \"Failed to parse index.json\")?;\n    Ok(index)\n}\n\npub fn save_project_index(index: &ProjectIndex) -> Result<()> {\n    let cowork_dir = get_cowork_dir()?;\n    let path = cowork_dir.join(INDEX_FILE);\n    let content = serde_json::to_string_pretty(index)?;\n    fs::write(&path, content)\n        .with_context(|| format!(\"Failed to write {:?}\", path))?;\n    Ok(())\n}\n\npub fn init_project_index(project_name: String) -> Result<ProjectIndex> {\n    if is_project_initialized() {\n        anyhow::bail!(\".cowork directory already initialized\");\n    }\n    let index = ProjectIndex::new(project_name);\n    save_project_index(&index)?;\n    Ok(index)\n}\n\n// ============================================================================\n// Session Input (sessions/<id>/input.json)\n// ============================================================================\n\n#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]\npub struct SessionInput {\n    pub session_id: String,\n    pub session_type: SessionType,\n    pub description: String,\n    pub base_session_id: Option<String>,\n    pub created_at: chrono::DateTime<chrono::Utc>,\n}\n\npub fn save_session_input(session_id: &str, input: &SessionInput) -> Result<()> {\n    let session_dir = get_session_dir(session_id)?;\n    let path = session_dir.join(\"input.json\");\n    let content = serde_json::to_string_pretty(input)?;\n    fs::write(&path, content)?;\n    Ok(())\n}\n\npub fn load_session_input(session_id: &str) -> Result<SessionInput> {\n    let session_dir = get_session_dir(session_id)?;\n    let path = session_dir.join(\"input.json\");\n    if !path.exists() {\n        anyhow::bail!(\"Session input not found for session {}\", session_id);\n    }\n    let content = fs::read_to_string(&path)?;\n    let input: SessionInput = serde_json::from_str(&content)?;\n    Ok(input)\n}\n\n// ============================================================================\n// Change Request (sessions/<id>/change_request.json - only for modify sessions)\n// ============================================================================\n\npub fn save_change_request(session_id: &str, change_request: &ChangeRequest) -> Result<()> {\n    let session_dir = get_session_dir(session_id)?;\n    let path = session_dir.join(\"change_request.json\");\n    let content = serde_json::to_string_pretty(change_request)?;\n    fs::write(&path, content)?;\n    Ok(())\n}\n\npub fn load_change_request(session_id: &str) -> Result<ChangeRequest> {\n    let session_dir = get_session_dir(session_id)?;\n    let path = session_dir.join(\"change_request.json\");\n    if !path.exists() {\n        anyhow::bail!(\"Change request not found for session {}\", session_id);\n    }\n    let content = fs::read_to_string(&path)?;\n    let cr: ChangeRequest = serde_json::from_str(&content)?;\n    Ok(cr)\n}\n\n// ============================================================================\n// Session-scoped Artifacts (sessions/<id>/artifacts/)\n// ============================================================================\n\nfn artifact_path(session_id: &str, filename: &str) -> Result<PathBuf> {\n    let session_dir = get_session_dir(session_id)?;\n    Ok(session_dir.join(\"artifacts\").join(filename))\n}\n\npub fn save_idea(session_id: &str, content: &str) -> Result<()> {\n    let path = artifact_path(session_id, \"idea.md\")?;\n    fs::write(&path, content)?;\n    Ok(())\n}\n\npub fn load_idea(session_id: &str) -> Result<String> {\n    let path = artifact_path(session_id, \"idea.md\")?;\n    if !path.exists() {\n        return Ok(String::new());\n    }\n    let content = fs::read_to_string(&path)?;\n    Ok(content)\n}\n\npub fn save_prd_doc(session_id: &str, content: &str) -> Result<()> {\n    let path = artifact_path(session_id, \"prd.md\")?;\n    fs::write(&path, content)?;\n    Ok(())\n}\n\npub fn save_design_doc(session_id: &str, content: &str) -> Result<()> {\n    let path = artifact_path(session_id, \"design.md\")?;\n    fs::write(&path, content)?;\n    Ok(())\n}\n\npub fn save_delivery_report(session_id: &str, content: &str) -> Result<()> {\n    let path = artifact_path(session_id, \"delivery_report.md\")?;\n    fs::write(&path, content)?;\n    Ok(())\n}\n\n// ============================================================================\n// Session-scoped State (sessions/<id>/state/)\n// ============================================================================\n\nfn state_path(session_id: &str, filename: &str) -> Result<PathBuf> {\n    let session_dir = get_session_dir(session_id)?;\n    Ok(session_dir.join(\"state\").join(filename))\n}\n\npub fn state_file_exists(session_id: &str, filename: &str) -> Result<bool> {\n    Ok(state_path(session_id, filename)?.exists())\n}\n\npub fn has_requirements(session_id: &str) -> Result<bool> {\n    state_file_exists(session_id, \"requirements.json\")\n}\n\npub fn has_design_spec(session_id: &str) -> Result<bool> {\n    state_file_exists(session_id, \"design_spec.json\")\n}\n\npub fn has_implementation_plan(session_id: &str) -> Result<bool> {\n    state_file_exists(session_id, \"implementation_plan.json\")\n}\n\npub fn save_requirements(session_id: &str, requirements: &Requirements) -> Result<()> {\n    let path = state_path(session_id, \"requirements.json\")?;\n    let content = serde_json::to_string_pretty(requirements)?;\n    fs::write(&path, content)?;\n    Ok(())\n}\n\npub fn load_requirements(session_id: &str) -> Result<Requirements> {\n    let path = state_path(session_id, \"requirements.json\")?;\n    if !path.exists() {\n        return Ok(Requirements::new());\n    }\n    let content = fs::read_to_string(&path)?;\n    let requirements: Requirements = serde_json::from_str(&content)?;\n    Ok(requirements)\n}\n\npub fn save_feature_list(session_id: &str, features: &FeatureList) -> Result<()> {\n    let path = state_path(session_id, \"feature_list.json\")?;\n    let content = serde_json::to_string_pretty(features)?;\n    fs::write(&path, content)?;\n    Ok(())\n}\n\npub fn load_feature_list(session_id: &str) -> Result<FeatureList> {\n    let path = state_path(session_id, \"feature_list.json\")?;\n    if !path.exists() {\n        return Ok(FeatureList::new());\n    }\n    let content = fs::read_to_string(&path)?;\n    let features: FeatureList = serde_json::from_str(&content)?;\n    Ok(features)\n}\n\npub fn save_design_spec(session_id: &str, design: &DesignSpec) -> Result<()> {\n    let path = state_path(session_id, \"design_spec.json\")?;\n    let content = serde_json::to_string_pretty(design)?;\n    fs::write(&path, content)?;\n    Ok(())\n}\n\npub fn load_design_spec(session_id: &str) -> Result<DesignSpec> {\n    let path = state_path(session_id, \"design_spec.json\")?;\n    if !path.exists() {\n        return Ok(DesignSpec::new());\n    }\n    let content = fs::read_to_string(&path)?;\n    let design: DesignSpec = serde_json::from_str(&content)?;\n    Ok(design)\n}\n\npub fn save_implementation_plan(session_id: &str, plan: &ImplementationPlan) -> Result<()> {\n    let path = state_path(session_id, \"implementation_plan.json\")?;\n    let content = serde_json::to_string_pretty(plan)?;\n    fs::write(&path, content)?;\n    Ok(())\n}\n\npub fn load_implementation_plan(session_id: &str) -> Result<ImplementationPlan> {\n    let path = state_path(session_id, \"implementation_plan.json\")?;\n    if !path.exists() {\n        return Ok(ImplementationPlan::new());\n    }\n    let content = fs::read_to_string(&path)?;\n    let plan: ImplementationPlan = serde_json::from_str(&content)?;\n    Ok(plan)\n}\n\npub fn save_code_metadata(session_id: &str, metadata: &CodeMetadata) -> Result<()> {\n    let path = state_path(session_id, \"code_metadata.json\")?;\n    let content = serde_json::to_string_pretty(metadata)?;\n    fs::write(&path, content)?;\n    Ok(())\n}\n\npub fn load_code_metadata(session_id: &str) -> Result<CodeMetadata> {\n    let path = state_path(session_id, \"code_metadata.json\")?;\n    if !path.exists() {\n        return Ok(CodeMetadata::new());\n    }\n    let content = fs::read_to_string(&path)?;\n    let metadata: CodeMetadata = serde_json::from_str(&content)?;\n    Ok(metadata)\n}\n\n// ============================================================================\n// Session Metadata (sessions/<id>/state/meta.json)\n// ============================================================================\n\npub fn save_session_meta(session_id: &str, meta: &SessionMeta) -> Result<()> {\n    let path = state_path(session_id, \"meta.json\")?;\n    let content = serde_json::to_string_pretty(meta)?;\n    fs::write(&path, content)?;\n    Ok(())\n}\n\npub fn load_session_meta(session_id: &str) -> Result<Option<SessionMeta>> {\n    let path = state_path(session_id, \"meta.json\")?;\n    if !path.exists() {\n        return Ok(None);\n    }\n    let content = fs::read_to_string(&path)?;\n    let meta: SessionMeta = serde_json::from_str(&content)?;\n    Ok(Some(meta))\n}\n\n// ============================================================================\n// Feedback History (sessions/<id>/state/feedback.json)\n// ============================================================================\n\npub fn save_feedback_history(session_id: &str, history: &FeedbackHistory) -> Result<()> {\n    let path = state_path(session_id, \"feedback.json\")?;\n    let content = serde_json::to_string_pretty(history)?;\n    fs::write(&path, content)?;\n    Ok(())\n}\n\npub fn load_feedback_history(session_id: &str) -> Result<FeedbackHistory> {\n    let path = state_path(session_id, \"feedback.json\")?;\n    if !path.exists() {\n        return Ok(FeedbackHistory::new());\n    }\n    let content = fs::read_to_string(&path)?;\n    let history: FeedbackHistory = serde_json::from_str(&content)?;\n    Ok(history)\n}\n\npub fn append_feedback(session_id: &str, feedback: &Feedback) -> Result<()> {\n    let mut history = load_feedback_history(session_id)?;\n    history.feedbacks.push(feedback.clone());\n    save_feedback_history(session_id, &history)?;\n    Ok(())\n}\n\n// ============================================================================\n// Patch Metadata (sessions/<id>/patch/metadata.json - for modify sessions)\n// ============================================================================\n\npub fn save_patch_metadata(session_id: &str, patch: &PatchMetadata) -> Result<()> {\n    let session_dir = get_session_dir(session_id)?;\n    let path = session_dir.join(\"patch\").join(\"metadata.json\");\n    let content = serde_json::to_string_pretty(patch)?;\n    fs::write(&path, content)?;\n    Ok(())\n}\n\npub fn load_patch_metadata(session_id: &str) -> Result<PatchMetadata> {\n    let session_dir = get_session_dir(session_id)?;\n    let path = session_dir.join(\"patch\").join(\"metadata.json\");\n    if !path.exists() {\n        anyhow::bail!(\"Patch metadata not found for session {}\", session_id);\n    }\n    let content = fs::read_to_string(&path)?;\n    let patch: PatchMetadata = serde_json::from_str(&content)?;\n    Ok(patch)\n}\n\n// ============================================================================\n// Session Inheritance / Bootstrap\n// ============================================================================\n\n/// Initialize a new session by copying state/artifacts from a base session.\n///\n/// This is critical for `modify` / `revert` / `resume` flows: a fresh session directory\n/// should not start with empty state, otherwise agents will see empty requirements/design/plan.\n///\n/// What we copy:\n/// - state/*.json (requirements, feature_list, design_spec, implementation_plan, code_metadata, feedback, meta)\n/// - artifacts/*.md (idea, prd, design, delivery_report) if present\n///\n/// Notes:\n/// - This function does NOT copy code files in the project root.\n/// - Missing files are skipped.\npub fn init_session_from_base(new_session_id: &str, base_session_id: &str) -> Result<()> {\n    let base_dir = get_session_dir(base_session_id)?;\n    let new_dir = get_session_dir(new_session_id)?;\n\n    // helper to copy a file if it exists\n    fn copy_if_exists(src: &Path, dst: &Path) -> Result<()> {\n        if !src.exists() {\n            return Ok(());\n        }\n        if let Some(parent) = dst.parent() {\n            fs::create_dir_all(parent)?;\n        }\n        fs::copy(src, dst).with_context(|| format!(\"Failed to copy {:?} -> {:?}\", src, dst))?;\n        Ok(())\n    }\n\n    // state files\n    let state_files = [\n        \"requirements.json\",\n        \"feature_list.json\",\n        \"design_spec.json\",\n        \"implementation_plan.json\",\n        \"code_metadata.json\",\n        \"feedback.json\",\n        \"meta.json\",\n    ];\n\n    for name in state_files {\n        let src = base_dir.join(\"state\").join(name);\n        let dst = new_dir.join(\"state\").join(name);\n        copy_if_exists(&src, &dst)?;\n    }\n\n    // artifact files\n    let artifact_files = [\"idea.md\", \"prd.md\", \"design.md\", \"delivery_report.md\"]; \n    for name in artifact_files {\n        let src = base_dir.join(\"artifacts\").join(name);\n        let dst = new_dir.join(\"artifacts\").join(name);\n        copy_if_exists(&src, &dst)?;\n    }\n\n    Ok(())\n}\n\n// ============================================================================\n// Helper utilities\n// ============================================================================\n\n/// Generate ID with prefix and counter\npub fn generate_id(prefix: &str, counter: usize) -> String {\n    format!(\"{}-{:03}\", prefix, counter + 1)\n}\n\n/// Get the latest successful session ID from index\npub fn get_latest_successful_session() -> Result<Option<String>> {\n    if !is_project_initialized() {\n        return Ok(None);\n    }\n    let index = load_project_index()?;\n    Ok(index.latest_successful_session)\n}\n\n/// Mark a session as completed successfully\npub fn mark_session_completed(session_id: &str) -> Result<()> {\n    let mut index = load_project_index()?;\n    \n    // Update session record\n    for session in &mut index.sessions {\n        if session.session_id == session_id {\n            session.status = SessionStatus::Completed;\n            session.completed_at = Some(chrono::Utc::now());\n            break;\n        }\n    }\n    \n    // Update latest successful session\n    index.update_latest_successful(session_id.to_string());\n    save_project_index(&index)?;\n    Ok(())\n}\n\n/// Mark a session as failed\npub fn mark_session_failed(session_id: &str) -> Result<()> {\n    let mut index = load_project_index()?;\n    \n    for session in &mut index.sessions {\n        if session.session_id == session_id {\n            session.status = SessionStatus::Failed;\n            session.completed_at = Some(chrono::Utc::now());\n            break;\n        }\n    }\n    \n    save_project_index(&index)?;\n    Ok(())\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 36.0,
      "lines_of_code": 487,
      "number_of_classes": 0,
      "number_of_functions": 47
    },
    "dependencies": [
      {
        "dependency_type": "internal",
        "is_external": false,
        "line_number": 2,
        "name": "crate::data",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "error_handling",
        "is_external": true,
        "line_number": 3,
        "name": "anyhow",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "standard_library",
        "is_external": false,
        "line_number": 4,
        "name": "std::fs",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "standard_library",
        "is_external": false,
        "line_number": 5,
        "name": "std::path",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "serialization",
        "is_external": true,
        "line_number": null,
        "name": "serde",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "datetime",
        "is_external": true,
        "line_number": null,
        "name": "chrono",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "This component serves as the persistent storage layer for the cowork-core system, implementing a session-scoped file-based storage architecture. It manages the .cowork/ directory structure containing project metadata, session-specific data, artifacts, and state information. The module provides comprehensive CRUD operations for various data types including project index, session inputs, requirements, design specifications, implementation plans, code metadata, feedback history, and patch metadata. It features session inheritance capabilities for modify/revert/resume workflows and maintains transactional integrity through file-based operations.",
    "interfaces": [
      {
        "description": "Input data structure for session creation",
        "interface_type": "struct",
        "name": "SessionInput",
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
            "name": "session_type",
            "param_type": "SessionType"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "description",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": true,
            "name": "base_session_id",
            "param_type": "Option<String>"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "created_at",
            "param_type": "chrono::DateTime<chrono::Utc>"
          }
        ],
        "return_type": null,
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Manage .cowork/ directory structure and session-scoped data organization",
      "Provide persistence operations for project metadata and session state",
      "Handle session inheritance and bootstrap operations for workflow continuity",
      "Maintain artifact storage for documentation and delivery reports",
      "Track session lifecycle and completion status"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "model",
      "description": "Core data models for Cowork Forge project management system",
      "file_path": "crates/cowork-core/src/data/models.rs",
      "functions": [
        "Requirements::new",
        "FeatureList::new",
        "DesignSpec::new",
        "ImplementationPlan::new",
        "CodeMetadata::new",
        "FeedbackHistory::new",
        "ProjectIndex::new",
        "ProjectIndex::add_session",
        "ProjectIndex::update_latest_successful",
        "ChangeRequest::new",
        "PatchMetadata::new"
      ],
      "importance_score": 0.8,
      "interfaces": [
        "Requirements",
        "Requirement",
        "Priority",
        "RequirementCategory",
        "FeatureList",
        "Feature",
        "FeatureStatus",
        "FeatureMetadata",
        "DesignSpec",
        "Architecture",
        "DesignComponent",
        "ComponentType",
        "ComponentInterface",
        "DataModel",
        "DataField",
        "TechnologyStack",
        "DeploymentInfo",
        "ImplementationPlan",
        "Milestone",
        "Task",
        "TaskStatus",
        "CodeMetadata",
        "FileMetadata",
        "BuildStatus",
        "TestStatus",
        "TestDetail",
        "SessionMeta",
        "Stage",
        "FeedbackHistory",
        "Feedback",
        "FeedbackType",
        "Severity",
        "ProjectIndex",
        "SessionRecord",
        "SessionType",
        "SessionStatus",
        "ChangeRequest",
        "ChangeScope",
        "ChangeAnalysis",
        "RiskLevel",
        "PatchMetadata",
        "ArtifactUpdate",
        "ArtifactType",
        "ChangeType"
      ],
      "name": "models.rs",
      "source_summary": "// Structured data models for Cowork Forge\nuse chrono::{DateTime, Utc};\nuse serde::{Deserialize, Serialize};\n\n// ============================================================================\n// Requirements (requirements.json)\n// ============================================================================\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Requirements {\n    pub schema_version: String,\n    pub created_at: DateTime<Utc>,\n    pub updated_at: DateTime<Utc>,\n    pub requirements: Vec<Requirement>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Requirement {\n    pub id: String,  // REQ-001, REQ-002, etc.\n    pub title: String,\n    pub description: String,\n    pub priority: Priority,\n    pub category: RequirementCategory,\n    pub acceptance_criteria: Vec<String>,\n    pub related_features: Vec<String>,  // Feature IDs\n}\n\n#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]\n#[serde(rename_all = \"lowercase\")]\npub enum Priority {\n    High,\n    Medium,\n    Low,\n}\n\n#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]\n#[serde(rename_all = \"snake_case\")]\npub enum RequirementCategory {\n    Functional,\n    NonFunctional,\n}\n\n// ============================================================================\n// Feature List (feature_list.json)\n// ============================================================================\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct FeatureList {\n    pub schema_version: String,\n    pub features: Vec<Feature>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Feature {\n    pub id: String,  // FEAT-001, FEAT-002, etc.\n    pub name: String,\n    pub description: String,\n    pub requirement_ids: Vec<String>,\n    pub status: FeatureStatus,\n    pub assigned_to_tasks: Vec<String>,  // Task IDs\n    pub completion_criteria: Vec<String>,\n    pub created_at: DateTime<Utc>,\n    pub completed_at: Option<DateTime<Utc>>,\n    #[serde(default)]\n    pub metadata: FeatureMetadata,\n}\n\n#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]\n#[serde(rename_all = \"snake_case\")]\npub enum FeatureStatus {\n    Pending,\n    InProgress,\n    Completed,\n    Blocked,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize, Default)]\npub struct FeatureMetadata {\n    #[serde(skip_serializing_if = \"Option::is_none\")]\n    pub estimated_effort: Option<String>,\n    #[serde(default)]\n    pub dependencies: Vec<String>,\n}\n\n// ============================================================================\n// Design Spec (design_spec.json)\n// ============================================================================\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct DesignSpec {\n    pub schema_version: String,\n    pub architecture: Architecture,\n    pub technology_stack: TechnologyStack,\n    pub deployment: DeploymentInfo,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Architecture {\n    pub style: String,  // \"microservices\", \"monolith\", etc.\n    pub components: Vec<DesignComponent>,\n    pub data_models: Vec<DataModel>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct DesignComponent {\n    pub id: String,  // COMP-001, COMP-002, etc.\n    pub name: String,\n    #[serde(rename = \"type\")]\n    pub component_type: ComponentType,\n    pub responsibilities: Vec<String>,\n    pub technology: String,\n    pub interfaces: Vec<ComponentInterface>,\n    pub related_features: Vec<String>,  // Feature IDs\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\n#[serde(rename_all = \"snake_case\")]\npub enum ComponentType {\n    BackendService,\n    FrontendComponent,\n    Database,\n    ApiGateway,\n    MessageQueue,\n    Other(String),\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct ComponentInterface {\n    pub name: String,\n    pub inputs: Vec<String>,\n    pub outputs: Vec<String>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct DataModel {\n    pub name: String,\n    pub fields: Vec<DataField>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct DataField {\n    pub name: String,\n    #[serde(rename = \"type\")]\n    pub field_type: String,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct TechnologyStack {\n    #[serde(skip_serializing_if = \"Option::is_none\")]\n    pub backend: Option<String>,\n    #[serde(skip_serializing_if = \"Option::is_none\")]\n    pub frontend: Option<String>,\n    #[serde(skip_serializing_if = \"Option::is_none\")]\n    pub database: Option<String>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct DeploymentInfo {\n    pub architecture: String,\n}\n\n// ============================================================================\n// Implementation Plan (implementation_plan.json)\n// ============================================================================\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct ImplementationPlan {\n    pub schema_version: String,\n    pub milestones: Vec<Milestone>,\n    pub tasks: Vec<Task>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Milestone {\n    pub id: String,  // M1, M2, etc.\n    pub name: String,\n    pub features: Vec<String>,  // Feature IDs\n    #[serde(skip_serializing_if = \"Option::is_none\")]\n    pub deadline: Option<String>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Task {\n    pub id: String,  // TASK-001, TASK-002, etc.\n    pub title: String,\n    pub description: String,\n    pub feature_id: String,\n    pub component_id: String,\n    pub status: TaskStatus,\n    pub dependencies: Vec<String>,  // Task IDs\n    #[serde(skip_serializing_if = \"Option::is_none\")]\n    pub estimated_effort: Option<String>,\n    pub files_to_create: Vec<String>,\n    pub acceptance_criteria: Vec<String>,\n    pub created_at: DateTime<Utc>,\n    pub started_at: Option<DateTime<Utc>>,\n    pub completed_at: Option<DateTime<Utc>>,\n}\n\n#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]\n#[serde(rename_all = \"snake_case\")]\npub enum TaskStatus {\n    Pending,\n    InProgress,\n    Completed,\n    Blocked,\n}\n\n// ============================================================================\n// Code Metadata (code_metadata.json)\n// ============================================================================\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct CodeMetadata {\n    pub schema_version: String,\n    pub files: Vec<FileMetadata>,\n    pub build_status: BuildStatus,\n    pub test_status: TestStatus,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct FileMetadata {\n    pub path: String,\n    pub task_id: String,\n    #[serde(skip_serializing_if = \"Option::is_none\")]\n    pub feature_id: Option<String>,\n    #[serde(skip_serializing_if = \"Option::is_none\")]\n    pub component_id: Option<String>,\n    pub created_at: DateTime<Utc>,\n    pub last_modified: DateTime<Utc>,\n    pub lines_of_code: usize,\n    pub test_coverage: f32,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct BuildStatus {\n    pub last_build: DateTime<Utc>,\n    pub success: bool,\n    pub errors: Vec<String>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct TestStatus {\n    pub last_run: DateTime<Utc>,\n    pub total: usize,\n    pub passed: usize,\n    pub failed: usize,\n    pub details: Vec<TestDetail>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct TestDetail {\n    pub test_name: String,\n    pub status: String,  // \"passed\" or \"failed\"\n    #[serde(skip_serializing_if = \"Option::is_none\")]\n    pub message: Option<String>,\n}\n\n// ============================================================================\n// Session Meta (session/meta.json)\n// ============================================================================\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct SessionMeta {\n    pub session_id: String,\n    pub created_at: DateTime<Utc>,\n    pub current_stage: Option<Stage>,\n    #[serde(skip_serializing_if = \"Option::is_none\")]\n    pub restart_reason: Option<String>,\n}\n\n#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]\n#[serde(rename_all = \"snake_case\")]\npub enum Stage {\n    Idea,\n    Prd,\n    Design,\n    Plan,\n    Coding,\n    Check,\n    Delivery,\n}\n\n// ============================================================================\n// Feedback (session/feedback.json)\n// ============================================================================\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct FeedbackHistory {\n    pub feedbacks: Vec<Feedback>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Feedback {\n    pub feedback_type: FeedbackType,\n    pub severity: Severity,\n    pub details: String,\n    #[serde(skip_serializing_if = \"Option::is_none\")]\n    pub suggested_fix: Option<String>,\n    pub timestamp: DateTime<Utc>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\n#[serde(rename_all = \"snake_case\")]\npub enum FeedbackType {\n    BuildError,\n    QualityIssue,\n    MissingRequirement,\n    Suggestion,\n}\n\n#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]\n#[serde(rename_all = \"lowercase\")]\npub enum Severity {\n    Critical,\n    Major,\n    Minor,\n}\n\n// ============================================================================\n// Session-scoped Models (for session isolation)\n// ============================================================================\n\n/// Project index - tracks all sessions and current state\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct ProjectIndex {\n    pub schema_version: String,\n    pub project_name: String,\n    pub created_at: DateTime<Utc>,\n    pub updated_at: DateTime<Utc>,\n    /// The latest successful session (for modify to use as base)\n    pub latest_successful_session: Option<String>,\n    /// All session records\n    pub sessions: Vec<SessionRecord>,\n}\n\n/// Record of a single session (new/modify/revert execution)\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct SessionRecord {\n    pub session_id: String,\n    pub session_type: SessionType,\n    pub created_at: DateTime<Utc>,\n    pub completed_at: Option<DateTime<Utc>>,\n    pub status: SessionStatus,\n    /// For modify sessions: which session is the base\n    pub base_session_id: Option<String>,\n    /// Input description (idea for new, change request for modify)\n    pub input_description: String,\n    /// Change request (only for modify sessions)\n    pub change_request_id: Option<String>,\n}\n\n#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]\n#[serde(rename_all = \"snake_case\")]\npub enum SessionType {\n    New,      // Full project creation (new command)\n    Modify,   // Incremental change (modify command)\n    Revert,   // Revert and rerun (revert command)\n}\n\n#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]\n#[serde(rename_all = \"snake_case\")]\npub enum SessionStatus {\n    InProgress,\n    Completed,\n    Failed,\n}\n\n/// Change request - describes an incremental modification\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct ChangeRequest {\n    pub id: String,\n    pub session_id: String,\n    pub created_at: DateTime<Utc>,\n    /// User's idea/description of the change\n    pub idea: String,\n    /// Which session to use as baseline\n    pub base_session_id: String,\n    /// Automatically determined scope (which artifacts need update)\n    pub scope: ChangeScope,\n    /// Acceptance criteria extracted from idea\n    pub acceptance_criteria: Vec<String>,\n    /// Constraints (e.g., don't break existing features)\n    pub constraints: Vec<String>,\n    /// Analysis result from triage agent\n    pub analysis: Option<ChangeAnalysis>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct ChangeScope {\n    pub requires_prd_update: bool,\n    pub requires_design_update: bool,\n    pub requires_plan_update: bool,\n    pub requires_code_change: bool,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct ChangeAnalysis {\n    /// Affected components\n    pub affected_components: Vec<String>,\n    /// Affected features\n    pub affected_features: Vec<String>,\n    /// Risk assessment\n    pub risk_level: RiskLevel,\n    /// Estimated effort\n    pub estimated_effort: String,\n}\n\n#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]\n#[serde(rename_all = \"lowercase\")]\npub enum RiskLevel {\n    Low,\n    Medium,\n    High,\n}\n\n/// Patch metadata - tracks what changed in a modify session\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct PatchMetadata {\n    pub session_id: String,\n    pub base_session_id: String,\n    pub created_at: DateTime<Utc>,\n    /// Files added\n    pub added_files: Vec<String>,\n    /// Files modified\n    pub modified_files: Vec<String>,\n    /// Files deleted\n    pub deleted_files: Vec<String>,\n    /// Artifact updates\n    pub artifact_updates: Vec<ArtifactUpdate>,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct ArtifactUpdate {\n    pub artifact_type: ArtifactType,\n    pub change_type: ChangeType,\n    pub summary: String,\n}\n\n#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]\n#[serde(rename_all = \"snake_case\")]\npub enum ArtifactType {\n    Requirements,\n    Features,\n    Design,\n    Plan,\n    Code,\n}\n\n#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]\n#[serde(rename_all = \"snake_case\")]\npub enum ChangeType {\n    Added,\n    Modified,\n    Deleted,\n}\n\n// ============================================================================\n// Helper implementations\n// ============================================================================\n\nimpl ProjectIndex {\n    pub fn new(project_name: String) -> Self {\n        Self {\n            schema_version: \"2.0\".to_string(),\n            project_name,\n            created_at: Utc::now(),\n            updated_at: Utc::now(),\n            latest_successful_session: None,\n            sessions: Vec::new(),\n        }\n    }\n\n    pub fn add_session(&mut self, record: SessionRecord) {\n        self.sessions.push(record);\n        self.updated_at = Utc::now();\n    }\n\n    pub fn update_latest_successful(&mut self, session_id: String) {\n        self.latest_successful_session = Some(session_id);\n        self.updated_at = Utc::now();\n    }\n}\n\nimpl ChangeRequest {\n    pub fn new(\n        session_id: String,\n        idea: String,\n        base_session_id: String,\n    ) -> Self {\n        Self {\n            id: format!(\"CR-{}\", Utc::now().timestamp()),\n            session_id,\n            created_at: Utc::now(),\n            idea,\n            base_session_id,\n            scope: ChangeScope::default(),\n            acceptance_criteria: Vec::new(),\n            constraints: Vec::new(),\n            analysis: None,\n        }\n    }\n}\n\nimpl Default for ChangeScope {\n    fn default() -> Self {\n        Self {\n            requires_prd_update: false,\n            requires_design_update: false,\n            requires_plan_update: false,\n            requires_code_change: true, // Default to code-only change\n        }\n    }\n}\n\nimpl PatchMetadata {\n    pub fn new(session_id: String, base_session_id: String) -> Self {\n        Self {\n            session_id,\n            base_session_id,\n            created_at: Utc::now(),\n            added_files: Vec::new(),\n            modified_files: Vec::new(),\n            deleted_files: Vec::new(),\n            artifact_updates: Vec::new(),\n        }\n    }\n}\n\nimpl Requirements {\n    pub fn new() -> Self {\n        Self {\n            schema_version: \"1.0\".to_string(),\n            created_at: Utc::now(),\n            updated_at: Utc::now(),\n            requirements: Vec::new(),\n        }\n    }\n}\n\nimpl FeatureList {\n    pub fn new() -> Self {\n        Self {\n            schema_version: \"1.0\".to_string(),\n            features: Vec::new(),\n        }\n    }\n}\n\nimpl DesignSpec {\n    pub fn new() -> Self {\n        Self {\n            schema_version: \"1.0\".to_string(),\n            architecture: Architecture {\n                style: String::new(),\n                components: Vec::new(),\n                data_models: Vec::new(),\n            },\n            technology_stack: TechnologyStack {\n                backend: None,\n                frontend: None,\n                database: None,\n            },\n            deployment: DeploymentInfo {\n                architecture: String::new(),\n            },\n        }\n    }\n}\n\nimpl ImplementationPlan {\n    pub fn new() -> Self {\n        Self {\n            schema_version: \"1.0\".to_string(),\n            milestones: Vec::new(),\n            tasks: Vec::new(),\n        }\n    }\n}\n\nimpl CodeMetadata {\n    pub fn new() -> Self {\n        Self {\n            schema_version: \"1.0\".to_string(),\n            files: Vec::new(),\n            build_status: BuildStatus {\n                last_build: Utc::now(),\n                success: false,\n                errors: Vec::new(),\n            },\n            test_status: TestStatus {\n                last_run: Utc::now(),\n                total: 0,\n                passed: 0,\n                failed: 0,\n                details: Vec::new(),\n            },\n        }\n    }\n}\n\nimpl FeedbackHistory {\n    pub fn new() -> Self {\n        Self {\n            feedbacks: Vec::new(),\n        }\n    }\n}\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 19.0,
      "lines_of_code": 608,
      "number_of_classes": 65,
      "number_of_functions": 11
    },
    "dependencies": [
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 2,
        "name": "chrono",
        "path": null,
        "version": null
      },
      {
        "dependency_type": "library",
        "is_external": true,
        "line_number": 3,
        "name": "serde",
        "path": null,
        "version": null
      }
    ],
    "detailed_description": "This component serves as the comprehensive data model foundation for Cowork Forge, a project management system that orchestrates software development lifecycle from requirements gathering to delivery. It defines structured data models for requirements management, feature tracking, design specifications, implementation planning, code metadata, session management, and feedback handling. The models support serialization/deserialization via Serde and include timestamp tracking with Chrono for audit purposes. The component implements a session-based architecture that supports project creation, modification, and reversion workflows with proper isolation and change tracking.",
    "interfaces": [
      {
        "description": "Container for project requirements with versioning and timestamps",
        "interface_type": "struct",
        "name": "Requirements",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "schema_version",
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
            "name": "updated_at",
            "param_type": "DateTime<Utc>"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "requirements",
            "param_type": "Vec<Requirement>"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Individual requirement with categorization and tracking",
        "interface_type": "struct",
        "name": "Requirement",
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
            "name": "title",
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
            "name": "priority",
            "param_type": "Priority"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "category",
            "param_type": "RequirementCategory"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "acceptance_criteria",
            "param_type": "Vec<String>"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "related_features",
            "param_type": "Vec<String>"
          }
        ],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Master index tracking all project sessions and current state",
        "interface_type": "struct",
        "name": "ProjectIndex",
        "parameters": [
          {
            "description": null,
            "is_optional": false,
            "name": "schema_version",
            "param_type": "String"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "project_name",
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
            "name": "updated_at",
            "param_type": "DateTime<Utc>"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "latest_successful_session",
            "param_type": "Option<String>"
          },
          {
            "description": null,
            "is_optional": false,
            "name": "sessions",
            "param_type": "Vec<SessionRecord>"
          }
        ],
        "return_type": null,
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Define and maintain structured data models for project artifacts (requirements, features, design, plan, code metadata)",
      "Provide session management capabilities for project lifecycle tracking and isolation",
      "Implement change tracking and version control mechanisms for incremental modifications",
      "Support serialization/deserialization for data persistence and interchange",
      "Establish relationships between different project artifacts through reference IDs"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "agent",
      "description": "Central module file for exporting all agent instruction modules",
      "file_path": "crates/cowork-core/src/instructions/mod.rs",
      "functions": [
        "module_reexport",
        "dependency_management"
      ],
      "importance_score": 0.6,
      "interfaces": [
        "public_module_interface"
      ],
      "name": "mod.rs",
      "source_summary": "// Agent instructions - Prompt templates for each agent\n\npub mod idea;\npub mod prd;\npub mod design;\npub mod plan;\npub mod coding;\npub mod check;\npub mod delivery;\npub mod modify;\npub mod code_patch;\npub mod modify_delivery;\n\npub use idea::*;\npub use prd::*;\npub use design::*;\npub use plan::*;\npub use coding::*;\npub use check::*;\npub use delivery::*;\npub use modify::*;\npub use code_patch::*;\npub use modify_delivery::*;\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 2.0,
      "lines_of_code": 23,
      "number_of_classes": 0,
      "number_of_functions": 0
    },
    "dependencies": [
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": 4,
        "name": "idea",
        "path": "crates/cowork-core/src/instructions/idea.rs",
        "version": null
      },
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": 5,
        "name": "prd",
        "path": "crates/cowork-core/src/instructions/prd.rs",
        "version": null
      },
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": 6,
        "name": "design",
        "path": "crates/cowork-core/src/instructions/design.rs",
        "version": null
      },
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": 7,
        "name": "plan",
        "path": "crates/cowork-core/src/instructions/plan.rs",
        "version": null
      },
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": 8,
        "name": "coding",
        "path": "crates/cowork-core/src/instructions/coding.rs",
        "version": null
      },
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": 9,
        "name": "check",
        "path": "crates/cowork-core/src/instructions/check.rs",
        "version": null
      },
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": 10,
        "name": "delivery",
        "path": "crates/cowork-core/src/instructions/delivery.rs",
        "version": null
      },
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": 11,
        "name": "modify",
        "path": "crates/cowork-core/src/instructions/modify.rs",
        "version": null
      },
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": 12,
        "name": "code_patch",
        "path": "crates/cowork-core/src/instructions/code_patch.rs",
        "version": null
      },
      {
        "dependency_type": "module",
        "is_external": false,
        "line_number": 13,
        "name": "modify_delivery",
        "path": "crates/cowork-core/src/instructions/modify_delivery.rs",
        "version": null
      }
    ],
    "detailed_description": "This mod.rs file serves as a central module aggregator and re-exporter for agent instruction prompt templates in the cowork-core crate. It doesn't contain any business logic itself but acts as a facade pattern to provide unified access to all instruction modules. The component re-exports 10 different instruction modules covering the complete agent workflow from idea generation to code modification and delivery.",
    "interfaces": [
      {
        "description": "Provides unified public access to all instruction modules",
        "interface_type": "module_export",
        "name": "public_module_interface",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      }
    ],
    "responsibilities": [
      "Module aggregation and centralized access point management",
      "Public API surface definition through re-export patterns",
      "Dependency organization and module structure maintenance",
      "Workflow orchestration interface provision"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "module",
      "description": "Tools module - adk-rust Tool implementations",
      "file_path": "crates/cowork-core/src/tools/mod.rs",
      "functions": [],
      "importance_score": 0.6,
      "interfaces": [],
      "name": "mod.rs",
      "source_summary": "// Tools module - adk-rust Tool implementations\npub mod data_tools;\npub mod validation_tools;\npub mod control_tools;\npub mod file_tools;\npub mod artifact_tools;\npub mod goto_stage_tool;\npub mod test_lint_tools;\npub mod hitl_tools;\npub mod hitl_content_tools;\npub mod modify_tools;\npub mod idea_tools;\n\npub use data_tools::*;\npub use validation_tools::*;\npub use control_tools::*;\npub use file_tools::*;\npub use artifact_tools::*;\npub use goto_stage_tool::*;\npub use test_lint_tools::*;\npub use hitl_tools::*;\npub use hitl_content_tools::*;\npub use modify_tools::*;\npub use idea_tools::*;\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 1.0,
      "lines_of_code": 24,
      "number_of_classes": 0,
      "number_of_functions": 0
    },
    "dependencies": [],
    "detailed_description": "This is a module declaration file that serves as a central hub for organizing and exporting various tool implementations in the cowork-core crate. The module aggregates 11 different tool submodules including data tools, validation tools, control tools, file tools, artifact tools, stage navigation tools, testing/linting tools, HITL (Human-in-the-Loop) tools, content tools, modification tools, and idea tools. It acts as a facade pattern providing unified access to all tool functionality through re-exports.",
    "interfaces": [],
    "responsibilities": [
      "Module organization and namespace management",
      "Tool functionality aggregation and export",
      "Dependency re-export for external consumers",
      "Module structure definition and visibility control"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "module",
      "description": "LLM module - Using adk-rust's built-in OpenAI client with custom base URL",
      "file_path": "crates/cowork-core/src/llm/mod.rs",
      "functions": [],
      "importance_score": 0.6,
      "interfaces": [],
      "name": "mod.rs",
      "source_summary": "// LLM module - Using adk-rust's built-in OpenAI client with custom base URL\npub mod config;\npub mod rate_limiter;\n\npub use config::*;\npub use rate_limiter::*;\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 1.0,
      "lines_of_code": 6,
      "number_of_classes": 0,
      "number_of_functions": 0
    },
    "dependencies": [],
    "detailed_description": "This is a Rust module file that serves as the entry point for the LLM (Large Language Model) module within the cowork-core crate. The module primarily organizes and re-exports functionality from two submodules: config and rate_limiter. It acts as a facade that provides centralized access to LLM-related configuration and rate limiting capabilities while leveraging adk-rust's built-in OpenAI client with custom base URL configuration. The component serves as a modularization boundary for LLM functionality within the system.",
    "interfaces": [],
    "responsibilities": [
      "Module organization and namespace management",
      "Public API exposure through re-exports",
      "Integration point for LLM-related submodules",
      "Dependency management for LLM functionality"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "lib",
      "description": "Cowork Forge core library providing foundational modules for AI agent collaboration system",
      "file_path": "crates/cowork-core/src/lib.rs",
      "functions": [],
      "importance_score": 0.6,
      "interfaces": [],
      "name": "lib.rs",
      "source_summary": "// Cowork Forge - Core Library\n// Built with adk-rust 0.2.1\n\npub mod data;\npub mod storage;\npub mod llm;\npub mod tools;\npub mod agents;\npub mod pipeline;\npub mod instructions;\n\n// Re-exports for convenience\npub use data::*;\npub use storage::*;\npub use llm::*;\n\n// Version info\npub const VERSION: &str = env!(\"CARGO_PKG_VERSION\");\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 2.0,
      "lines_of_code": 18,
      "number_of_classes": 0,
      "number_of_functions": 0
    },
    "dependencies": [
      {
        "dependency_type": "build_dependency",
        "is_external": true,
        "line_number": 2,
        "name": "adk-rust",
        "path": null,
        "version": "0.2.1"
      }
    ],
    "detailed_description": "This is the root library file for the Cowork Forge core library, serving as the main entry point and module organization structure. The component acts as a facade and module aggregator for the core functionality of an AI agent collaboration system. It organizes related modules including data handling, storage, LLM integration, tools, agents, pipelines, and instructions. The library provides convenient re-exports for downstream consumers and includes version information.",
    "interfaces": [],
    "responsibilities": [
      "Module organization and namespace management",
      "Public API exposure through re-exports",
      "Version information provision",
      "Library entry point definition",
      "Dependency aggregation for core functionality"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "model",
      "description": "Validation utilities for data schemas",
      "file_path": "crates/cowork-core/src/data/schemas/validation.rs",
      "functions": [],
      "importance_score": 0.6,
      "interfaces": [],
      "name": "validation.rs",
      "source_summary": "// Validation utilities for data schemas\n// This module provides validation logic for structured data\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 3.0,
      "lines_of_code": 2,
      "number_of_classes": 0,
      "number_of_functions": 0
    },
    "dependencies": [],
    "detailed_description": "This is a placeholder validation module with only a comment indicating it's intended to provide validation logic for structured data. The component is currently unimplemented with only 2 lines of code (a comment). Based on the file path and naming, it appears to be part of a schema validation system within a coworking/core data module, likely intended to validate data structures or schemas in a coworking application.",
    "interfaces": [],
    "responsibilities": [
      "Provide validation logic for structured data schemas",
      "Define validation rules and constraints for data models",
      "Handle schema validation errors and reporting",
      "Ensure data integrity through validation checks"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "model",
      "description": null,
      "file_path": "crates/cowork-core/src/data/mod.rs",
      "functions": [
        "module_re-export",
        "conditional_testing_module"
      ],
      "importance_score": 0.6,
      "interfaces": [
        "pub mod models",
        "pub mod schemas",
        "pub use models::*",
        "pub use schemas::*"
      ],
      "name": "mod.rs",
      "source_summary": "// Data models module\npub mod models;\npub mod schemas;\n\n#[cfg(test)]\nmod models_test;\n\npub use models::*;\npub use schemas::*;\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 1.0,
      "lines_of_code": 9,
      "number_of_classes": 0,
      "number_of_functions": 2
    },
    "dependencies": [
      {
        "dependency_type": "test_module",
        "is_external": false,
        "line_number": 4,
        "name": "models_test",
        "path": "./crates/cowork-core/src/data/models_test.rs",
        "version": null
      }
    ],
    "detailed_description": "This is a Rust module file that serves as the entry point for the data models module in the cowork-core crate. It acts as a module aggregator and re-exporter for data-related functionality. The module conditionally includes test modules only during test compilation, demonstrating proper separation of test and production code. It provides unified access to both models and schemas through re-export statements.",
    "interfaces": [
      {
        "description": "Public module exposing data models",
        "interface_type": "module",
        "name": "models",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Public module exposing data schemas",
        "interface_type": "module",
        "name": "schemas",
        "parameters": [],
        "return_type": null,
        "visibility": "public"
      },
      {
        "description": "Conditional test module for data models",
        "interface_type": "module",
        "name": "models_test",
        "parameters": [],
        "return_type": null,
        "visibility": "private"
      }
    ],
    "responsibilities": [
      "Module organization and aggregation for data-related components",
      "Public interface exposure through re-export patterns",
      "Conditional test module inclusion for testing environment",
      "Namespace management for data models and schemas",
      "Entry point configuration for data module hierarchy"
    ]
  },
  {
    "code_dossier": {
      "code_purpose": "model",
      "description": "Schema definitions module for validation purposes",
      "file_path": "crates/cowork-core/src/data/schemas.rs",
      "functions": [],
      "importance_score": 0.6,
      "interfaces": [],
      "name": "schemas.rs",
      "source_summary": "// JSON Schema definitions (for validation)\npub mod validation;\n"
    },
    "complexity_metrics": {
      "cyclomatic_complexity": 2.0,
      "lines_of_code": 2,
      "number_of_classes": 0,
      "number_of_functions": 0
    },
    "dependencies": [],
    "detailed_description": "This is a minimal Rust module that serves as an entry point for JSON Schema definitions used for validation. The component currently only contains a single module declaration for 'validation', indicating it's designed to organize validation-related schemas. Given its location in the data layer of a coworking core system, it likely defines data structures and validation rules for the domain model. The extremely minimal implementation suggests this is either a stub/placeholder for future development or a simple module organization structure.",
    "interfaces": [],
    "responsibilities": [
      "Module organization and namespace management for validation schemas",
      "Provide entry point to validation-related schema definitions",
      "Structure validation logic separation within the data layer",
      "Serve as foundation for future schema expansion",
      "Maintain architectural separation between data models and validation rules"
    ]
  }
]
```

## Memory Storage Statistics

**Total Storage Size**: 611089 bytes

- **preprocess**: 361793 bytes (59.2%)
- **timing**: 35 bytes (0.0%)
- **studies_research**: 98435 bytes (16.1%)
- **documentation**: 150826 bytes (24.7%)

## Generated Documents Statistics

Number of Generated Documents: 15

- Project Overview
- Boundary Interfaces
- Key Modules and Components Research Report_工具支持域
- Key Modules and Components Research Report_LLM集成域
- Core Workflows
- Key Modules and Components Research Report_数据存储域
- Key Modules and Components Research Report_编码实施域
- Key Modules and Components Research Report_CLI接口域
- Key Modules and Components Research Report_代理管理域
- Key Modules and Components Research Report_需求管理域
- Key Modules and Components Research Report_交付管理域
- Key Modules and Components Research Report_编排管理域
- Architecture Description
- Key Modules and Components Research Report_设计管理域
- Key Modules and Components Research Report_规划管理域
