# 修复：HITL编辑内容未生效问题

## 问题描述

**用户报告**：
在HITL（Human-in-the-Loop）环节，用户通过`edit`选项编辑了内容，但Agent在后续流程中仍然按照它之前的方案执行，编辑的内容没有生效。

## 问题分析

### 1. **HITL工具机制**

查看 `crates/cowork-core/src/tools/hitl_content_tools.rs`，`ReviewWithFeedbackContentTool` 的返回格式：

```rust
// 当用户选择 "edit"
match trimmed.to_lowercase().as_str() {
    "edit" => {
        let edited = Editor::new()
            .require_save(true)
            .edit(content)
            .map_err(...)?;

        let new_content = edited.unwrap_or_else(|| content.to_string());
        Ok(json!({
            "action": "edit",
            "content": new_content,  // ← 编辑后的内容在这里
            "message": "User edited content"
        }))
    }
    "pass" | "" => Ok(json!({
        "action": "pass",
        "content": content,  // ← 原始内容
        "message": "User passed"
    })),
    _ => Ok(json!({
        "action": "feedback",
        "feedback": trimmed,
        "content": content,  // ← 原始内容 + 反馈文本
        "message": "User provided feedback"
    })),
}
```

**工具返回的数据结构**：
- `action`: "edit" | "pass" | "feedback"
- `content`: 内容（edit时是编辑后的，pass时是原始的）
- `feedback`: 反馈文本（仅在action="feedback"时）

### 2. **指令中的问题**

查看原始指令（PRD/Design/Plan），都是这样写的：

```markdown
## Step 3: User Review (MANDATORY - HITL)
4. **MUST** call `review_with_feedback_content(...)`
5. Handle response:
   - action="edit": use returned content
   - action="pass": keep original
   - action="feedback": revise and optionally review again

## Step 4: Create Formal Requirements (MANDATORY)
6. For EACH requirement in finalized draft, **MUST** call `create_requirement(...)`
```

**问题所在**：
1. ❌ **指令太模糊**："use returned content" - 但没有明确说怎么用
2. ❌ **没有强调必须使用**：LLM可能理解为"可以选择用"
3. ❌ **没有明确警告**：不要使用原始draft，要用finalized draft
4. ❌ **Step 4与Step 3脱节**：Step 4说"For EACH in finalized draft"，但没有明确finalized draft就是Step 3的返回内容

### 3. **LLM的理解偏差**

由于指令不够明确，LLM可能的理解：

**错误理解**：
```
Step 2: 创建draft（保存在变量draft中）
Step 3: 调用review_with_feedback_content(content=draft)
        返回：{"action": "edit", "content": "编辑后的内容"}
        理解：哦，用户编辑了，但我已经有draft了
Step 4: 使用draft（而不是返回的content）创建正式数据
```

**正确理解**：
```
Step 2: 创建draft
Step 3: 调用review_with_feedback_content(content=draft)
        返回：{"action": "edit", "content": "编辑后的内容"}
        关键：finalized_draft = 返回的content字段
Step 4: 使用finalized_draft创建正式数据
```

---

## 解决方案

### 修复策略

在三个Actor指令中（PRD、Design、Plan），明确强调：
1. **必须使用**编辑后的内容
2. **明确字段**：content字段就是finalized draft
3. **警告不要用**原始draft
4. **Step 4要解析**Step 3的返回内容

### 修复1：PRD Actor指令

**文件**：`crates/cowork-core/src/instructions/prd.rs`

```markdown
## Step 3: User Review (MANDATORY - HITL)
4. **MUST** call `review_with_feedback_content(...)`
5. **Handle response carefully**:
   - **If action="edit"**: The tool returns edited content in the "content" field. 
     **YOU MUST USE THIS EDITED CONTENT** as your finalized draft for Step 4.
   - **If action="pass"**: Use your original draft as the finalized draft.
   - **If action="feedback"**: Read the feedback text, revise your draft accordingly, 
     then optionally call review_with_feedback_content again (max 1 more time).
   
   **CRITICAL**: Whatever content you get from the final review call (either edited or original), 
   that becomes your "finalized draft" for the next step. Do NOT ignore the edited content!

## Step 4: Create Formal Requirements (MANDATORY)
6. **Parse the finalized draft** from Step 3 (the content field from review_with_feedback_content result)
7. For EACH requirement in the **finalized draft**, **MUST** call `create_requirement(...)`
8. For EACH feature in the **finalized draft**, **MUST** call `add_feature(...)`
   **Do NOT skip this step! All requirements and features must be created!**
   **Do NOT use your original draft - use the finalized one from Step 3!**
```

### 修复2：Design Actor指令

**文件**：`crates/cowork-core/src/instructions/design.rs`

同样的修改逻辑：

```markdown
## Step 3: User Review (MANDATORY - HITL)
3. **MUST** call `review_with_feedback_content(...)`
4. **Handle response carefully**:
   - **If action="edit"**: The tool returns edited content in the "content" field. 
     **YOU MUST USE THIS EDITED CONTENT** as your finalized draft for Step 4.
   ...

## Step 4: Create Formal Design (MANDATORY)
5. **Parse the finalized draft** from Step 3 (the content field from review_with_feedback_content result)
6. For EACH component in the **finalized draft**, **MUST** call `create_design_component(...)`
   **Do NOT use your original draft - use the finalized one from Step 3!**
```

### 修复3：Plan Actor指令

**文件**：`crates/cowork-core/src/instructions/plan.rs`

同样的修改：

```markdown
## Step 3: User Review (MANDATORY - HITL)
4. **MUST** call `review_with_feedback_content(...)`
5. **Handle response carefully**:
   - **If action="edit"**: The tool returns edited content in the "content" field. 
     **YOU MUST USE THIS EDITED CONTENT** as your finalized draft for Step 4.
   ...

## Step 4: Create Formal Tasks (MANDATORY)
6. **Parse the finalized draft** from Step 3 (the content field from review_with_feedback_content result)
7. For EACH task in the **finalized draft**, **MUST** call `create_task(...)`
   **Do NOT use your original draft - use the finalized one from Step 3!**
```

---

## 修复要点

### 1. **强调"MUST USE"**
- 从"use returned content"改为"**YOU MUST USE THIS EDITED CONTENT**"
- 使用粗体、大写强调

### 2. **明确字段名**
- 明确说明："the content field from review_with_feedback_content result"
- 不留歧义

### 3. **明确命名**
- "finalized draft" = Step 3的返回content
- 在Step 4中多次提到"finalized draft from Step 3"

### 4. **明确警告**
- "Do NOT ignore the edited content!"
- "Do NOT use your original draft - use the finalized one from Step 3!"

### 5. **Step连接**
- Step 4明确说"Parse the finalized draft from Step 3"
- 建立Step之间的依赖关系

---

## 效果对比

### Before（修复前）

**指令**：
```
Step 3: Handle response:
  - action="edit": use returned content
  - action="pass": keep original

Step 4: For EACH requirement in finalized draft, call create_requirement(...)
```

**LLM理解**：
- "finalized draft"是什么？可能是我Step 2创建的draft
- "use returned content"怎么用？记录一下？

**结果**：
- 编辑内容被忽略 ❌

### After（修复后）

**指令**：
```
Step 3: **Handle response carefully**:
  - **If action="edit"**: The tool returns edited content in the "content" field. 
    **YOU MUST USE THIS EDITED CONTENT** as your finalized draft for Step 4.
  
  **CRITICAL**: Whatever content you get from the final review call, 
  that becomes your "finalized draft" for the next step. Do NOT ignore the edited content!

Step 4: **Parse the finalized draft** from Step 3 (the content field from review_with_feedback_content result)
  For EACH requirement in the **finalized draft**, call create_requirement(...)
  **Do NOT use your original draft - use the finalized one from Step 3!**
```

**LLM理解**：
- 清晰：finalized draft = Step 3返回的content字段
- 强制：MUST USE，不是可选
- 警告：不要用原始draft

**结果**：
- 编辑内容被正确使用 ✅

---

## 编译验证

```bash
✅ cargo check -p cowork-core --lib  # 通过
✅ cargo build --release              # 成功
```

---

## 测试建议

### 测试步骤
1. 运行 `cowork new`
2. 在PRD阶段，当看到draft预览时
3. 输入 `edit` 打开编辑器
4. 修改一些内容（例如：删除一个requirement，添加一个新的）
5. 保存并关闭编辑器
6. 观察后续创建的requirements.json

**预期结果**：
- ✅ requirements.json应该反映编辑后的内容
- ✅ 不应该包含被删除的requirement
- ✅ 应该包含新添加的requirement

### 类似测试
- Design阶段：编辑components，验证design_spec.json
- Plan阶段：编辑tasks，验证implementation_plan.json

---

## 总结

**根本问题**：
- 指令太模糊，LLM不理解要使用编辑后的内容

**解决方案**：
1. 明确强调"MUST USE EDITED CONTENT"
2. 明确字段名"content field"
3. 明确命名"finalized draft from Step 3"
4. 明确警告"Do NOT use original draft"
5. 建立Step之间的明确依赖关系

**效果**：
- HITL编辑的内容会被正确使用
- Agent不再使用原始draft
- 用户的编辑真正生效
