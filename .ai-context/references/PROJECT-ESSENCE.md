# Project Essence — Cowork Forge

> **Stability: HIGH** | Update: Quarterly or major version changes
>
> Last reviewed: 2026-04-17

---

## What Is This Project?

Cowork Forge is an AI-native multi-agent software development platform that simulates a complete virtual development team — Product Manager, Architect, Project Manager, and Engineer — working collaboratively through a 7-stage pipeline to transform ideas into production-ready software.

---

## Why Does It Exist?

**Problem:** Traditional AI coding assistants only generate code snippets. Real software development requires requirements analysis, architecture design, task planning, quality verification, and delivery — a multi-role collaborative process that single-model tools cannot cover.

**Solution:**
- Orchestrate specialized AI agents through a structured 7-stage pipeline (Idea→PRD→Design→Plan→Coding→Check→Delivery)
- Apply Actor-Critic pattern for iterative self-refinement at critical stages
- Insert Human-in-the-Loop validation gates at key decision points
- Preserve institutional knowledge across iterations via a Memory system
- Support evolution iterations with intelligent change scope analysis

---

## Who Is This For?

| User | Use Case |
|------|----------|
| Individual Developers | Rapid prototyping via CLI automation |
| Development Teams | Standardized workflows with cross-iteration memory |
| AI-Augmented Developers | Interactive GUI with real-time streaming and HITL |
| Existing Project Owners | Import and reverse-engineer documentation |

---

## Key Constraints

1. **Local-First**: All computation and storage is local; no cloud dependency beyond LLM API calls
2. **LLM Rate Limiting**: 30 req/min with concurrency=1 via global semaphore
3. **Workspace Containment**: File operations are validated against project boundaries to prevent path traversal
4. **Security-First**: Command sanitization, build tool whitelisting, and watchdog monitoring
5. **anyhow::Result**: All error handling uses `anyhow::Result`; no `unwrap()` in production code
6. **Rust Edition 2024**: Workspace uses Rust edition 2024 with `adk-rust` 0.5.0 framework

---

*This file captures the stable essence of the project. For architecture details, see [ARCHITECTURE.md](ARCHITECTURE.md).*
