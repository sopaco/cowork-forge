# AI Context Maintenance Guide

> How to keep this knowledge base accurate. Last updated: 2026-04-17.

---

## 🎯 Purpose

This guide tells you (the coding agent) how to maintain `.ai-context`.

---

## 📋 Maintenance Triggers

| Observation | Action | File |
|-------------|--------|------|
| Code contradicts docs | Fix docs | Relevant file |
| New crate/module | Add entry | ARCHITECTURE.md |
| Major decision | Document | DECISIONS.md |
| Blocking issue | Add entry | DYNAMICS.md |
| Issue resolved | Remove | DYNAMICS.md |
| New stage added | Update pipeline section | ARCHITECTURE.md |
| New tool category | Update tools section | ARCHITECTURE.md |
| New agent type | Update component list | ARCHITECTURE.md |

---

## ✍️ Writing Guidelines

### PROJECT-ESSENCE.md
- Under 100 lines
- "What" and "why", not "how"
- No code snippets
- Update: Quarterly

### ARCHITECTURE.md
- Diagrams over paragraphs
- Component-level, not file-level
- Show data flow
- Update: Monthly

### DECISIONS.md
- Non-obvious choices only
- Include rationale
- Update: As decisions made

### DYNAMICS.md
- Current issues only
- Remove when resolved
- Update: As needed

---

## 🔄 Update Workflow

1. Identify file needing update
2. Read current content
3. Make minimal changes
4. Update "Last updated" date
5. Continue your task

---

## ❌ Anti-Patterns

- Don't copy code snippets — link to files instead
- Don't document every file/function
- Don't keep resolved issues in DYNAMICS.md
- Don't duplicate content across files

---

## ✅ Quality Checklist

- [ ] PROJECT-ESSENCE.md readable in 2 min
- [ ] ARCHITECTURE.md shows big picture
- [ ] DECISIONS.md has rationale
- [ ] DYNAMICS.md only current issues
- [ ] All files dated

---

## 📎 Project-Specific Notes

### Key Files for Context Updates

| What Changed | Where to Look | Which .ai-context File |
|-------------|---------------|----------------------|
| Pipeline stages | `crates/cowork-core/src/pipeline/stages/*.rs` | ARCHITECTURE.md |
| Domain entities | `crates/cowork-core/src/domain/*.rs` | ARCHITECTURE.md |
| Tools | `crates/cowork-core/src/tools/*.rs` | ARCHITECTURE.md |
| Agent configs | `crates/cowork-core/src/config_definition/default_configs/*.json` | ARCHITECTURE.md |
| Instructions | `crates/cowork-core/src/instructions/*.rs` | DECISIONS.md (if pattern changes) |
| Security | `crates/cowork-core/src/runtime_security.rs` | DECISIONS.md |

---

*Update this guide when you discover better maintenance patterns.*
