# Dynamics — Active Issues & Constraints

> **Last updated:** 2026-04-17
> **Stability:** Dynamic — Update as issues arise/resolve

---

## ⚡ Quick Scan

| Status | Issue | Impact | Workaround |
|--------|-------|--------|------------|
| 🟡 | LLM serial execution | Throughput limited by concurrency=1 | Intentional; rate limit compliance |
| 🟡 | Keyword-based intent recognition (PM Agent) | Ambiguous requests may be misclassified | PM Agent asks for clarification |
| 🟡 | JSON persistence lacks query capabilities | No efficient search across projects | Use file system tools for search |

---

## 🟡 Known Constraints

### LLM Serial Execution
- Global semaphore (concurrency=1) means all LLM calls are serialized
- This is intentional for rate limit compliance (30 req/min)
- Pipeline stages cannot parallelize LLM-dependent work

### PM Agent Intent Recognition
- Uses keyword-based NLP (not ML-based)
- Ambiguous user input triggers clarification requests
- Intent types: bug_fix, requirement_change, new_feature, consultation, ambiguous

### JSON Storage Limitations
- No built-in indexing or query engine
- Concurrent access not protected (single-user assumption)
- Large projects may experience slower load times

### Storage Location
- Project data: `.cowork-v2/` in project root
- User config: platform-specific app data directory
- See `crates/cowork-core/src/config.rs` for path resolution

---

## 🟢 Recently Resolved

| Issue | Resolution | Date |
|-------|------------|------|
| N/A | N/A | — |

---

## 📋 Under Consideration

- SQLite backend option for team/large-project scenarios
- Local model support (Llama, Mistral) for offline operation
- Multi-user collaboration features (not currently in scope)

---

*Remember: This file changes frequently. Verify against current code state.*
