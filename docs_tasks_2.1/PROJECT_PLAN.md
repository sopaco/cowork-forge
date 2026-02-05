# Cowork Forge 迭代化重构计划

## 项目概述

将 Cowork Forge 从「New/Modify/Revert 多模式」架构改造为「统一迭代(Iteration)」架构。

### 核心变更
- **删除**: Session, ChangeRequest, Resume 等概念
- **新增**: Iteration 作为唯一开发单元
- **简化**: 项目级 Memory + 迭代级 Memory 两层架构

---

## 目录结构规划

```
crates/cowork-core/src/
├── lib.rs                    # 模块导出
├── domain/                   # 领域模型（新增）
│   ├── mod.rs
│   ├── project.rs            # 项目实体
│   ├── iteration.rs          # 迭代实体
│   └── memory.rs             # 记忆实体
├── persistence/              # 存储层（替代 storage）
│   ├── mod.rs
│   ├── project_store.rs      # 项目存储
│   ├── iteration_store.rs    # 迭代存储
│   └── memory_store.rs       # 记忆存储
├── pipeline/
│   └── mod.rs                # 重写：统一迭代执行器
├── agents/                   # 简化调用
│   └── mod.rs
├── interaction/              # 保持不变
├── tools/                    # 简化部分工具
└── llm/                      # 保持不变
```

---

## 实施阶段

### Phase 1: 基础模型 (Day 1-2)
- [ ] 创建 domain/ 模块
- [ ] 实现 Project, Iteration, Memory 模型
- [ ] 创建 persistence/ 存储层
- [ ] 单元测试

### Phase 2: Pipeline 重写 (Day 3-4)
- [ ] 删除旧 Pipeline
- [ ] 实现 IterationExecutor
- [ ] 重写 Stage 执行逻辑
- [ ] 集成测试

### Phase 3: CLI 重写 (Day 5)
- [ ] 删除旧命令
- [ ] 实现新命令: iter, list, show, continue
- [ ] 集成测试

### Phase 4: GUI 重写 (Day 6-8)
- [ ] 删除旧组件
- [ ] 实现迭代中心界面
- [ ] 实现新建迭代对话框
- [ ] 集成测试

### Phase 5: 清理收尾 (Day 9-10)
- [ ] 删除所有旧代码
- [ ] 完整 E2E 测试
- [ ] 文档更新

---

## 关键决策

1. **数据兼容性**: 不保留，完全新格式
2. **向后兼容**: 不保留，CLI/GUI 全新接口
3. **错误处理**: 使用 anyhow + thiserror
4. **异步**: 全异步，使用 tokio
5. **序列化**: 使用 serde + serde_json

---

## 风险与缓解

| 风险 | 缓解措施 |
|------|----------|
| 重构引入 Bug | 每个 Phase 后完整测试 |
| 功能遗漏 | 对照旧功能清单逐项验证 |
| 性能问题 | 延迟优化，先保证功能正确 |

---

## 成功标准

- [ ] 可创建 Genesis 迭代
- [ ] 可基于迭代创建 Evolution 迭代
- [ ] 可从指定阶段 Redo
- [ ] Memory 系统正常工作
- [ ] CLI 所有命令可用
- [ ] GUI 界面正常显示和操作
