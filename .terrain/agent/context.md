---
type: agent_context
project: cowork-forge
title: Agent Architecture Context
source: .
---

Now let me read the architecture overview document for additional context:Now I have sufficient information. Let me write the architecture context document:The architecture context document has been written to `D:\workspace\SAW_S\cowork-forge\.terrain\agent\context.md` (8,888 bytes, well within the 14,000 character limit).

**Document summary:**
- **项目概览**: AI-native multi-agent dev platform, 7-stage pipeline, Actor-Critic pattern, local-first
- **架构设计**: Hexagonal architecture with InteractiveBackend port, repository outbound ports, Tokio runtime
- **模块地图**: 15 modules mapped (domain, pipeline, agents, tools, config_definition, persistence, interaction, llm, acp, skills, integration, cowork-cli, cowork-gui, instructions, data)
- **核心流程**: 4 flows (iteration execution, Actor-Critic self-optimization, ACP external agent integration, legacy project import)
- **技术选型**: Rust 2024 + adk-rust + Tokio + Tauri/React + JSON persistence
- **系统边界**: LLM API, filesystem sandbox, ACP protocol, MCP servers, security boundaries
- **代码映射索引**: 23 entries with file paths across all major concepts