# AI上下文维护实践

## 会话启动检测

在AGENTS.md中添加启动检测指令：

```markdown
## 每次开始编程任务前

1. 运行: git diff --name-only HEAD~10
2. 如果变更涉及关键目录，提醒用户更新文档
```

### 关键目录映射

| 目录变更 | 提醒更新 |
|---------|---------|
| `tools/*.rs` | tools.md |
| `pipeline/stages/*.rs` | pipeline.md |
| `domain/*.rs` | domain-logic.md |
| 新增安全规则 | constraints.md |

---

## 更新工作流

### 用户主动更新

```
用户: "Update .ai-context because I added a new tool"

Agent执行：
1. 读取 manifest.yaml 了解更新规则
2. 读取对应文件（如 tools.md）
3. 添加条目到合适位置
4. 保持格式一致
```

### Agent检测更新

```
Agent在会话启动时：
1. git diff --name-only HEAD~10
2. 检查是否涉及关键目录
3. 如有，提醒："检测到[目录]变更，是否更新.ai-context？"
```

---

## 文档一致性检查

### 手动检查清单

- [ ] 新增工具是否在tools.md中有条目？
- [ ] 新增Stage是否在pipeline.md中有记录？
- [ ] 架构决策是否记录在architecture-decisions.md？
- [ ] 文档中的路径是否仍然有效？

### 自动化检查（可选）

```bash
# 检查tools.md中提到的文件是否存在
grep -o "tools/[^.]*\.rs" .ai-context/domains/tools.md | while read f; do
  [ -f "crates/cowork-core/src/$f" ] || echo "Missing: $f"
done
```

---

## 常见更新场景

### 场景1：新增工具

```
用户: "我刚加了一个ValidateSchemaTool"

Agent:
1. 读取 tools.md
2. 判断工具属于哪个分类（Validation）
3. 在对应表格添加：| ValidateSchemaTool | Validate JSON schema |
```

### 场景2：架构决策

```
用户: "我决定用Redis替代内存缓存"

Agent:
1. 读取 architecture-decisions.md
2. 添加ADR：
   ## ADR-XXX: Redis for Caching
   Decision: 使用Redis替代进程内缓存
   Reason: 需要跨进程共享、持久化
   Impact: memory模块
```

### 场景3：新增模块

```
用户: "我加了一个新的reporting模块"

Agent:
1. 更新 modules.map 模块列表
2. 更新 project.snapshot 结构说明
3. 如有新的Store，更新 storage structure
```

---

## 不需要更新的场景

| 场景 | 原因 |
|------|------|
| 重命名变量 | AI读代码即可 |
| 提取函数 | 不影响结构 |
| struct增减字段 | 文档不记录字段 |
| 函数签名变化 | AI直接看代码 |
| 移动文件位置 | 如不影响模块关系，可不更新 |

---

## 文档过时了怎么办？

### 轻度过时（10-20%）

不影响使用，AI能从代码推断正确信息。

### 中度过时（20-40%）

AI可能定位错误，需要用户手动更新。

### 重度过时（>40%）

建议重新生成文档或手动全面更新。

---

## CI集成（可选）

```yaml
# .github/workflows/ai-context-check.yml
name: AI Context Check
on: [pull_request]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Check for significant changes
        run: |
          CHANGED=$(git diff --name-only origin/main)
          if echo "$CHANGED" | grep -q "tools/\|pipeline/stages/\|domain/"; then
            echo "::warning::Significant changes detected. Consider updating .ai-context/"
          fi
```

这只是提醒，不阻塞CI。
