# Debug Context Template

## Purpose
Pre-generated context for AI agents debugging issues in this project.

---

## System Context

You are debugging **Cowork Forge**, an AI-native iterative software development platform.

### Debug Logging
- **Framework**: `tracing` crate
- **Level Control**: `RUST_LOG` environment variable
- **Common Levels**: `debug`, `info`, `warn`, `error`

### Enable Debug Logging
```bash
RUST_LOG=cowork_core=debug cargo run
RUST_LOG=debug cargo run --package cowork-cli
```

---

## Common Issues & Solutions

### Issue: Iteration stuck at stage
**Symptoms**: Pipeline hangs, no progress
**Debug Steps**:
1. Check `iteration.current_stage` in `.cowork-v2/iterations/*/iteration.json`
2. Look for HITL pending: GUI modal may be waiting
3. Check LLM API connectivity
4. Review logs for tool errors

**Key Files**:
- `pipeline/executor/mod.rs:execute_stages_from()`
- `interaction/tauri.rs` (for GUI HITL)

### Issue: Tool execution failure
**Symptoms**: Tool returns error, stage fails
**Debug Steps**:
1. Check tool name and parameters in logs
2. Validate path is within workspace
3. Check command is in allowed list (for RunCommandTool)
4. Verify file exists (for read operations)

**Key Files**:
- `tools/mod.rs` (notification system)
- `runtime_security.rs` (validation)

### Issue: Memory not persisting
**Symptoms**: Decisions/patterns disappear
**Debug Steps**:
1. Check `.cowork-v2/memory.json` exists and is valid JSON
2. Verify `promote_insights_to_decisions()` called
3. Check iteration memory at `.cowork-v2/iterations/*/memory.json`

**Key Files**:
- `persistence/memory_store.rs`
- `pipeline/executor/knowledge.rs`

### Issue: Agent not using tools
**Symptoms**: Agent responds but doesn't call tools
**Debug Steps**:
1. Check tool is registered in agent builder
2. Verify instruction mentions tool usage
3. Check tool schema is correct
4. Review LLM response for tool calls

**Key Files**:
- `agents/mod.rs` (agent builders)
- `instructions/*.rs` (prompts)

### Issue: GUI not receiving events
**Symptoms**: UI doesn't update, streaming not working
**Debug Steps**:
1. Check Tauri event emission in backend
2. Verify event name matches frontend listener
3. Check for JavaScript errors in dev console
4. Ensure `send_streaming` is called

**Key Files**:
- `interaction/tauri.rs`
- GUI frontend event listeners

---

## Debug Commands

### Check Iteration State
```bash
# View iteration JSON
cat .cowork-v2/iterations/iter-1-*/iteration.json | jq

# View memory
cat .cowork-v2/memory.json | jq
```

### Test LLM Connectivity
```bash
# Test with curl
curl -X POST https://api.openai.com/v1/chat/completions \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model":"gpt-4","messages":[{"role":"user","content":"test"}]}'
```

### Validate Storage
```bash
# Check all JSON files are valid
find .cowork-v2 -name "*.json" -exec jq . {} \;
```

---

## Key Debugging Locations

| Issue Type | Primary File | Secondary |
|------------|--------------|-----------|
| Pipeline hang | `pipeline/executor/mod.rs` | `interaction/mod.rs` |
| Tool failure | `tools/mod.rs` | `runtime_security.rs` |
| Memory issues | `persistence/memory_store.rs` | `domain/memory.rs` |
| Agent behavior | `agents/mod.rs` | `instructions/*.rs` |
| GUI issues | `interaction/tauri.rs` | Frontend components |
| LLM issues | `llm/config.rs` | Agent builders |

---

## Error Code Reference

| Error | Meaning | Resolution |
|-------|---------|------------|
| `IterationNotPaused` | Continue called on running iteration | Check status first |
| `PathValidationFailed` | Path outside workspace | Use relative paths |
| `CommandBlocked` | Command not in whitelist | Add to allowed list |
| `RateLimitExceeded` | Too many LLM calls | Wait and retry |
| `ArtifactNotFound` | Expected file missing | Check stage completed |

---

## Injection Points

- `{{ERROR_MESSAGE}}`: The error being debugged
- `{{STACK_TRACE}}`: Relevant stack trace
- `{{RECENT_LOGS}}`: Recent log output
- `{{ITERATION_STATE}}`: Current iteration JSON
