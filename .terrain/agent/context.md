---
type: agent_context
project: cowork-forge
title: Agent Architecture Context
source: .
---

Now I have enough information. Let me write the context document.Generated `agent/context.md` (9,014 bytes) with all 7 required sections. Key highlights:

- **Hexagonal architecture** with `cowork-core` as pure domain core, `InteractiveBackend` as the inbound port, JSON stores as outbound ports
- **7-stage pipeline** (Idea→PRD→Design→Plan→Coding→Check→Delivery) with Actor-Critic self-refinement per stage
- **20 rows** in the 代码映射索引 covering domain aggregates, pipeline, tools, agents, config, persistence, security, ACP, and GUI layers
- **4 core flows**: pipeline execution, Actor-Critic refinement, iteration inheritance, external agent integration
- All paths reference `crates/cowork-core/src/` and `crates/cowork-gui/src/` — no code bodies, no grep output pasted