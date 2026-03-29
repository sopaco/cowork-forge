# Architecture Decisions

> Records counter-intuitive design decisions that AI agents cannot infer from code.
> Only add entries for decisions that would surprise someone reading the code.

---

## ADR-001: LoopAgent max_iterations=1

**Decision**: All Actor-Critic loops use `max_iterations=1`

**Reason**: SequentialAgent has a bug where `exit_loop()` terminates the entire chain, not just the loop. Using `max_iterations=1` lets LoopAgent complete naturally.

**Impact**: All critical stages (PRD, Design, Plan, Coding)

**Do not**: Change this parameter or migrate to SequentialAgent without testing

---

## ADR-002: JSON Storage over SQLite

**Decision**: Use JSON files for persistence instead of SQLite

**Reason**: Simpler debugging, easy to inspect/edit, no external dependencies. Data volume is small and there's no concurrent access requirement.

**Impact**: All persistence layer, no migrations needed

**Limitation**: Not suitable for large-scale concurrent scenarios

---

## ADR-003: Coding Stage has 5 Iterations

**Decision**: Coding Actor-Critic loop allows `max_iterations=5` while others only 1

**Reason**: Code often needs iterative refinement based on test results. Other stages typically complete in one pass.

**Impact**: CodingStage, coding_loop agent

---

## ADR-004: Two-Step Knowledge Promotion

**Decision**: Insights → Decisions requires two separate tools (SaveInsightTool + PromoteToDecisionTool)

**Reason**: Not all insights should become decisions. Human review at promotion step ensures quality.

**Impact**: Memory tools, knowledge workflow

---

## ADR-005: GotoStage Only Goes Backward

**Decision**: GotoStageTool can only jump to earlier stages, never forward

**Reason**: Skipping stages would miss required artifacts. Re-executing earlier stages is safe.

**Impact**: Pipeline control flow, PM Agent navigation

---

## ADR-006: HITL Timeout Default Pass

**Decision**: If HITL confirmation times out, default action is "Pass" (continue)

**Reason**: Better to proceed than to block indefinitely. User can always re-run or use PM Agent to navigate back.

**Impact**: Stage execution, GUI timeout handling

---

## Adding New ADRs

When making a design decision that:
- Goes against common patterns
- Has non-obvious rationale
- Would confuse someone reading the code later

Add an entry with:
- Clear decision statement
- The "why" (most important)
- Impact scope
- Any "do not" warnings
