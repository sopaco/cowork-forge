---
name: ai-context-maintenance
description: Guidelines for maintaining and updating AI context files when code changes
version: 1.0.0
---

# AI Context Maintenance Skill

This skill guides AI agents to update `.ai-context/` files when code changes occur.

---

## When to Update

Trigger context update when:

1. **New module created** - Add to `modules.map` and relevant domain file
2. **New tool added** - Update `domains/tools.md`
3. **New stage added** - Update `domains/pipeline.md`
4. **Domain entity changed** - Update `domains/domain-logic.md`
5. **Trait interface changed** - Update `api/traits.md`
6. **New constraint/rule added** - Update `core/constraints.md`
7. **Major refactoring** - Regenerate `project.snapshot` and `modules.map`

---

## Update Commands

Ask the AI agent to:

```
"Update .ai-context based on recent changes to [specific files/modules]"
```

---

## File Update Guide

### 1. project.snapshot

**When**: Major structural changes, new crates, key concepts changed

**How**: 
- Update `CRATES STRUCTURE` section if new crate added
- Update `CORE CONCEPTS` if new patterns/concepts introduced
- Update `KEY DATA STRUCTURES` if entities changed
- Update `CRITICAL PATHS` if entry points moved

**Format**: Keep YAML-like structure for easy parsing

### 2. core/modules.map

**When**: New modules, changed dependencies, new call chains

**How**:
- Add new module to `MODULE FILES REFERENCE` section
- Update `CRITICAL CALL CHAINS` if flow changed
- Update `QUICK NAVIGATION BY TASK` if new task types added

**Format**: Keep ASCII diagrams and tables

### 3. core/constraints.md

**When**: New security rules, rate limits, validation logic

**How**:
- Add new constraint under appropriate section
- Update limits if changed
- Add new blocked commands if any

### 4. domains/pipeline.md

**When**: New stage, changed stage behavior, new stage result types

**How**:
- Add new stage to `Stage Implementations` table
- Update `Execution Flow` if sequence changed
- Update `Key Types` if new types introduced

### 5. domains/domain-logic.md

**When**: New entities, changed fields, new value objects

**How**:
- Add new entity/aggregate to appropriate section
- Update struct definitions with exact field names
- Add new methods to `Entity Behaviors` section

### 6. domains/tools.md

**When**: New tool added, tool behavior changed

**How**:
- Add tool to appropriate category table
- Update `Tool Categories` if new category needed
- Keep tool descriptions concise

### 7. domains/interaction.md

**When**: New InteractiveBackend methods, new message types

**How**:
- Update trait definition if methods changed
- Add new message types to `Supporting Types`
- Update event types if GUI events changed

### 8. domains/persistence.md

**When**: New store, changed storage format, new paths

**How**:
- Update `Storage Structure` if paths changed
- Add new store implementation
- Update store methods if interface changed

### 9. domains/agents.md

**When**: New agent, changed agent tools, new agent types

**How**:
- Add agent builder to `Agent Builders` section
- Update `Tool Configuration per Agent` tables
- Add new agent type to `Agent Types` section

### 10. api/traits.md

**When**: Trait methods changed, new traits added

**How**:
- Update trait definition with exact signatures
- Keep method signatures accurate
- Add new traits at the end

### 11. prompts/*.md

**When**: Rarely - only if project conventions fundamentally change

**How**:
- Update code examples
- Update file locations if moved
- Keep templates reusable

---

## Detection Heuristics

Use these patterns to detect what needs updating:

```rust
// New file in tools/ → update domains/tools.md
crates/cowork-core/src/tools/new_tool.rs

// New file in pipeline/stages/ → update domains/pipeline.md
crates/cowork-core/src/pipeline/stages/new_stage.rs

// Changed domain/*.rs → update domains/domain-logic.md
crates/cowork-core/src/domain/*.rs

// Changed interaction/mod.rs → update domains/interaction.md
crates/cowork-core/src/interaction/mod.rs

// New crate → update project.snapshot, modules.map
crates/new-crate/

// Changed Cargo.toml dependencies → check if affects constraints
Cargo.toml, crates/*/Cargo.toml
```

---

## Update Process

1. **Identify changed files** from git diff or conversation context
2. **Map to affected context files** using detection heuristics
3. **Read current context file** to understand existing content
4. **Update specific sections** without rewriting entire file
5. **Preserve format** - keep tables, code blocks, YAML structure
6. **Update manifest.yaml** version/timestamp if significant changes

---

## Validation Checklist

After updating, verify:

- [ ] File paths are correct and exist
- [ ] Struct/trait definitions match actual code
- [ ] Tables have consistent columns
- [ ] Code examples are valid Rust
- [ ] No duplicate entries added
- [ ] Cross-references between files are valid

---

## Example: Adding a New Tool

```
User: "I added a new tool called ValidateSchemaTool in tools/validation_tools.rs"

AI should:
1. Read .ai-context/domains/tools.md
2. Find the Validation Tools section
3. Add row to table: | ValidateSchemaTool | Validate JSON schema |
4. Read .ai-context/api/traits.md (if tool defines new trait)
5. Update if needed
```

---

## manifest.yaml Update

After significant updates, update the manifest:

```yaml
# Increment version for significant changes
manifest_version: "1.1"

# Update generation timestamp
documents:
  - path: domains/tools.md
    # ... existing fields ...
    last_updated: "2026-03-29"
```
