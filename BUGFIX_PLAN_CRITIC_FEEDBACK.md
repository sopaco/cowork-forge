# 修复：Plan Critic反馈机制失效问题

## 问题描述

**用户报告**：
- 工作目录：`/Users/jiangmeng/workspace/SAW/cowork-space-tmp`
- Session：`/.cowork/sessions/session-1769667877`
- 现象：feedback.json中有Critic反馈说要移除PDF相关功能，但implementation_plan.json中仍然存在这些任务
- 结论：Critic反馈机制失效，Actor没有实际调整任务

## 根本原因分析

### 1. **发现的问题证据**

查看feedback.json（第48-50行）：
```json
{
  "feedback_type": "suggestion",
  "severity": "critical",
  "details": "Tasks include non-core items: TASK-005 (Create PDF Generation Functionality), TASK-006 (Implement Print Preview Feature), TASK-011 (Implement PDF generation functionality), TASK-012 (Add question display and preview functionality).",
  "suggested_fix": "Remove all PDF generation and print preview tasks. Keep ONLY core feature implementation tasks..."
}
```

但查看implementation_plan.json，这些任务仍然存在且状态为"completed"：
```json
{
  "id": "TASK-005",
  "title": "Create PDF Generation Functionality",
  "status": "completed"  // ← 仍然存在！
}
{
  "id": "TASK-011",
  "title": "Implement PDF generation functionality",
  "status": "completed"  // ← 也存在！
}
```

### 2. **问题根源**

**Actor-Critic Loop机制的问题**：

当前的Plan Loop使用了Actor-Critic模式，max_iterations=3：

```rust
// crates/cowork-core/src/agents/mod.rs
let mut loop_agent = LoopAgent::new("plan_loop", vec![
    Arc::new(plan_actor),  
    Arc::new(plan_critic)
]);
loop_agent = loop_agent.with_max_iterations(3);
```

**流程执行顺序**：
1. **Iteration 1**: 
   - Actor创建任务（包含TASK-005, TASK-006等PDF相关任务）
   - Critic检查，发现非核心任务，调用`provide_feedback()`
   
2. **Iteration 2**:
   - Actor **应该**读取feedback并删除被拒绝的任务
   - 但Actor**没有工具**来删除任务！❌
   - Actor也**没有在指令中被告知**如何处理Critic的反馈

3. **Iteration 3**:
   - 循环继续，但任务还是老样子
   - 最终max_iterations耗尽，loop结束

**核心问题**：
1. ❌ **Plan Actor没有delete_task工具**（只有create_task）
2. ❌ **Plan Actor指令中没有告知如何处理Critic反馈**
3. ❌ **Feedback只是记录到JSON，没有实际触发Actor的修正行为**

### 3. **设计缺陷**

当前的反馈机制：
```
Critic → provide_feedback() → 写入feedback.json
                              ↓
                          （结束了，没有下文）
```

**缺少的环节**：
- Critic提供反馈后，Actor需要读取并采取行动
- Actor需要工具来修正问题（delete_task, update_task）
- Actor需要指令告知如何响应反馈

---

## 解决方案

### 修复1：给Plan Actor添加任务管理工具

**修改文件**：`crates/cowork-core/src/agents/mod.rs`

```rust
pub fn create_plan_loop(model: Arc<dyn Llm>, session_id: &str) -> Result<Arc<dyn adk_core::Agent>> {
    let session = session_id.to_string();
    
    let plan_actor = LlmAgentBuilder::new("plan_actor")
        .instruction(PLAN_ACTOR_INSTRUCTION)
        .model(model.clone())
        .tool(Arc::new(GetRequirementsTool::new(session.clone())))
        .tool(Arc::new(GetDesignTool::new(session.clone())))
        .tool(Arc::new(GetPlanTool::new(session.clone())))
        .tool(Arc::new(ReviewWithFeedbackContentTool))
        .tool(Arc::new(CreateTaskTool::new(session.clone())))
        // ✅ 新增：添加任务管理工具
        .tool(Arc::new(UpdateTaskTool::new(session.clone())))
        .tool(Arc::new(DeleteTaskTool::new(session.clone())))
        .include_contents(IncludeContents::None)
        .build()?;
    
    // ... rest of the code
}
```

### 修复2：更新Plan Actor指令

**修改文件**：`crates/cowork-core/src/instructions/plan.rs`

在指令中新增"Step 6: Handle Critic Feedback"：

```markdown
## Step 6: Handle Critic Feedback (IF NEEDED)
**NEW - IMPORTANT**: If Critic calls `provide_feedback` saying you have non-core tasks:
1. Read the feedback carefully - it will list specific task IDs to remove
2. For EACH task ID mentioned in the feedback:
   - Call `delete_task(task_id="TASK-XXX", reason="Removing non-core task per Critic feedback: <copy feedback details>")`
3. After deleting all problematic tasks, call `get_plan()` to verify
4. Report: "✅ Removed X non-core tasks per Critic feedback. Remaining tasks focus on core implementation only."
5. **DO NOT** recreate deleted tasks - Critic rejected them for good reason

# Tools Available (Updated)
- create_task(...) - Create ONE task
- delete_task(task_id, reason) - Delete a task (use when Critic rejects it) ← NEW
- update_task(task_id, reason, ...) - Update task properties ← NEW (if needed)

# CRITICAL RULES (Updated)
8. If Critic provides feedback about non-core tasks, you MUST delete them (don't defend or recreate)
```

---

## 修复效果

### Before（修复前）
```
Iteration 1:
  Actor: 创建12个任务（包含TASK-005 PDF生成、TASK-011 PDF功能）
  Critic: ❌ 发现非核心任务，调用provide_feedback()
  
Iteration 2:
  Actor: （无法删除任务，因为没有工具）继续说"我完成了"
  Critic: ❌ 还是看到那些任务，再次provide_feedback()
  
Iteration 3:
  （重复，循环耗尽，结束）

结果：feedback.json有记录，但任务还在 ❌
```

### After（修复后）
```
Iteration 1:
  Actor: 创建12个任务（包含TASK-005 PDF生成、TASK-011 PDF功能）
  Critic: ❌ 发现非核心任务，调用provide_feedback()
  
Iteration 2:
  Actor: 读取feedback，发现被拒绝的任务
  Actor: 调用delete_task("TASK-005", "Removing per Critic feedback")
  Actor: 调用delete_task("TASK-006", "Removing per Critic feedback")
  Actor: 调用delete_task("TASK-011", "Removing per Critic feedback")
  Actor: 调用delete_task("TASK-012", "Removing per Critic feedback")
  Actor: 报告"✅ Removed 4 non-core tasks"
  
Iteration 3:
  Critic: ✅ 检查，任务已删除，只剩核心任务，批准通过

结果：非核心任务被删除，只保留核心功能 ✅
```

---

## 关键改进点

### 1. **工具层面**
- ✅ Plan Actor现在有`delete_task()`工具
- ✅ Plan Actor现在有`update_task()`工具
- ✅ 可以响应Critic的反馈

### 2. **指令层面**
- ✅ 新增"Step 6: Handle Critic Feedback"
- ✅ 明确告知Actor如何处理被拒绝的任务
- ✅ 强调不要重新创建被删除的任务

### 3. **流程层面**
```
Before: Critic反馈 → 记录到JSON → 无后续行动
After:  Critic反馈 → Actor读取 → Actor删除任务 → Critic再次检查 → 通过
```

---

## 适用性扩展

这个修复同样适用于其他Actor-Critic Loop：

### PRD Loop
- PRD Actor可能需要`delete_requirement()`, `delete_feature()`工具
- 当Critic拒绝复杂需求时，Actor可以删除

### Design Loop
- Design Actor可能需要`delete_component()`工具
- 当Critic拒绝过度复杂的架构时，Actor可以简化

### Coding Loop
- Coding Actor已经有任务管理工具（刚刚添加的）
- 可以动态调整实现计划

---

## 编译验证

```bash
✅ cargo check -p cowork-core --lib  # 通过
✅ cargo build --release              # 成功
```

---

## 总结

**根本问题**：
- Critic的反馈只是记录，Actor没有工具和指令来响应

**解决方案**：
1. 给Actor添加`delete_task()`和`update_task()`工具
2. 在指令中明确告知如何处理Critic反馈
3. 形成真正的反馈闭环

**效果**：
- Critic发现问题 → Actor修正 → Critic再次检查 → 通过
- 反馈机制真正生效，不再是"空喊"

---

## 测试建议

重新运行相同的项目：
1. Plan Actor创建任务时可能仍会包含PDF相关任务
2. Plan Critic会检测到并提供反馈
3. **新行为**：Plan Actor应该在下一次iteration中删除这些任务
4. Plan Critic再次检查时应该通过

如果仍然出现问题，可能需要检查：
- LLM是否理解了新增的指令
- feedback是否正确传递给Actor
- Actor是否正确调用delete_task工具
