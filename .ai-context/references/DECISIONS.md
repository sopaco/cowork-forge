# Design Decisions — Cowork Forge

> Key architectural and design decisions. Update when decisions are made or revisited.
>
> Last reviewed: 2026-04-17

---

## Decision Index

| ID | Decision | Status | Date |
|----|----------|--------|------|
| ADR-001 | Multi-Crate Workspace Structure | Active | 2025 |
| ADR-002 | Trait-Based Backend Abstraction | Active | 2025 |
| ADR-003 | JSON-First Persistence | Active | 2025 |
| ADR-004 | Rate Limiting at Infrastructure Layer | Active | 2025 |
| ADR-005 | Event-Driven GUI with Asymmetric Communication | Active | 2025 |
| ADR-006 | Post-Delivery PM Agent | Active | 2025 |
| ADR-007 | agentskills.io Standard for Skills | Active | 2025 |
| ADR-008 | MCP Integration for External Tools | Active | 2025 |

---

## ADR-001: Multi-Crate Workspace Structure

**Context**: Core domain logic, CLI, and GUI have different deployment and dependency profiles. GUI requires Tauri/WebView; CLI should remain lightweight.

**Decision**: Separate CLI, GUI, and Core into distinct crates within a Cargo workspace.

**Rationale**: Enables independent deployment while sharing domain logic. Prevents GUI dependencies from bloating CLI binary.

**Trade-offs**:
- (+) Clean dependency boundaries, independent versioning
- (-) More complex workspace management, cross-crate refactoring

---

## ADR-002: Trait-Based Backend Abstraction

**Context**: Pipeline needs to interact with users through both CLI and GUI, which have fundamentally different interaction models (blocking vs event-driven).

**Decision**: `InteractiveBackend` trait to unify CLI and GUI interactions.

**Rationale**: Single pipeline code path supports both automation and interactive modes without conditional logic throughout the domain layer.

**Trade-offs**:
- (+) Clean hexagonal architecture, testable pipeline in isolation
- (-) Trait must accommodate both synchronous (CLI) and asynchronous (GUI) patterns

---

## ADR-003: JSON-First Persistence

**Context**: Need portable, inspectable, version-control-friendly project storage for a local-first desktop application.

**Decision**: File-based JSON storage instead of a database.

**Rationale**: Portability, version control compatibility, and local-first architecture. Users can inspect and modify project state with standard tools.

**Trade-offs**:
- (+) Zero dependencies, human-readable, git-friendly
- (-) No query capabilities, potential consistency issues with concurrent access

---

## ADR-004: Rate Limiting at Infrastructure Layer

**Context**: LLM API calls are expensive and quota-limited (typically 30 req/min). Uncontrolled parallelism could exhaust API quotas.

**Decision**: Decorator pattern with global semaphore (concurrency=1) and 2-second delay for LLM rate limiting.

**Rationale**: API quota protection and cost control. Global rate limiter ensures compliance regardless of pipeline stage parallelism.

**Trade-offs**:
- (+) Simple, reliable quota compliance
- (-) Serial LLM calls reduce throughput potential

---

## ADR-005: Event-Driven GUI with Asymmetric Communication

**Context**: GUI needs both request-response operations and real-time streaming of LLM tokens, tool calls, and process logs.

**Decision**: Tauri commands for requests, events for streaming responses.

**Rationale**: Commands provide request-response semantics; events enable server-push for streaming without polling overhead.

**Trade-offs**:
- (+) Efficient streaming, responsive UI
- (-) Two communication patterns to maintain, oneshot channels for HITL add complexity

---

## ADR-006: Post-Delivery PM Agent

**Context**: After an iteration completes, users need to continue interacting with the project — fix bugs, add features, ask questions — without understanding pipeline internals.

**Decision**: Dedicated PM Agent with intent recognition for post-delivery interaction.

**Rationale**: Natural language bridge from completed pipeline to ongoing project maintenance. Supports bug fixes (goto_stage), new features (create_iteration), and consultation (respond).

**Trade-offs**:
- (+) User-friendly post-delivery experience, no pipeline knowledge required
- (-) Keyword-based intent recognition can be imprecise; may misclassify ambiguous requests

---

## ADR-007: agentskills.io Standard for Skills

**Context**: Need a mechanism to inject domain-specific tools, prompts, and context into agents without code modifications.

**Decision**: Implement agentskills.io standard with SKILL.md markdown format.

**Rationale**: Industry standard ensures compatibility with external skill packages. Simple markdown format lowers barrier for skill authoring. Auto-discovery from `.skills/` directory.

**Trade-offs**:
- (+) Community-compatible, zero-code extensibility
- (-) Semantic skill matching quality depends on skill metadata

---

## ADR-008: MCP Integration for External Tools

**Context**: Agents need access to external capabilities (web search, code documentation) without complex local API integration.

**Decision**: Model Context Protocol integration for external tool server connectivity.

**Rationale**: Standardized protocol enables seamless third-party AI service integration (Tavily, DeepWiki). Config-driven auto-initialization at startup with automatic injection into all agents.

**Trade-offs**:
- (+) Extensible without code changes, standardized protocol
- (-) External service dependency, HTTP transport latency

---

## Template for New Decisions

```markdown
## ADR-XXX: [Short Title]

**Context**: [What is the issue?]

**Decision**: [What did we decide?]

**Rationale**: [Why this choice?]

**Trade-offs**:
- (+) [Benefit]
- (-) [Cost]
```

---

*This file captures decisions that aren't obvious from code.*
