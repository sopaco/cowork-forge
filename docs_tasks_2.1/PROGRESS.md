# Cowork Forge 迭代化重构进度报告

## 当前状态

**已完成**: Phase 3 & Phase 4 (CLI/GUI 重写)

## 已完成的修改

### Phase 1: 领域模型 (✅ 完成)

#### 新增文件
```
crates/cowork-core/src/domain/
├── mod.rs              # 模块导出
├── project.rs          # Project 实体
├── iteration.rs        # Iteration 实体
└── memory.rs           # Memory 实体
```

### Phase 2: 存储层 (✅ 完成)

#### 新增文件
```
crates/cowork-core/src/persistence/
├── mod.rs              # 模块导出
├── project_store.rs    # Project 存储
├── iteration_store.rs  # Iteration 存储
└── memory_store.rs     # Memory 存储
```

### Phase 2: Pipeline 重写 (✅ 完成)

#### 新增/修改文件
```
crates/cowork-core/src/pipeline/
├── mod.rs                          # 重写
├── executor.rs                     # 新增: IterationExecutor
└── stages/
    ├── mod.rs                      # 新增
    ├── idea.rs                     # 新增
    ├── prd.rs                      # 新增
    ├── design.rs                   # 新增
    ├── plan.rs                     # 新增
    ├── coding.rs                   # 新增
    ├── check.rs                    # 新增
    └── delivery.rs                 # 新增
```

### Phase 3: CLI 重写 (✅ 完成)

#### 修改文件
```
crates/cowork-cli/src/main.rs       # 完全重写
```

**新命令**:
- `cowork init [name]` - 初始化新项目
- `cowork iter <title>` - 创建并执行迭代
  - `--description` - 详细描述
  - `--base <id>` - 基于现有迭代
  - `--inherit <mode>` - 继承模式 (none/full/partial)
- `cowork list [--all]` - 列出迭代
- `cowork show [id]` - 显示迭代详情
- `cowork continue [id]` - 继续暂停的迭代
- `cowork status` - 显示项目状态
- `cowork delete <id>` - 删除迭代

### Phase 4: GUI 重写 (✅ 完成)

#### 新增文件
```
crates/cowork-gui/src-tauri/src/
├── iteration_commands.rs           # 新迭代命令

crates/cowork-gui/src/components/
├── IterationsPanel.jsx             # 迭代中心界面
```

#### 修改文件
```
crates/cowork-gui/src-tauri/src/lib.rs              # 添加新命令
crates/cowork-gui/src/App.jsx                       # 重写主组件
crates/cowork-gui/src/components/CommandPalette.jsx # 添加新命令
```

**新界面**:
- IterationsPanel - 迭代中心，显示所有迭代
  - 迭代列表卡片视图
  - 进度条显示
  - 阶段时间线
  - 创建新迭代对话框
  - 迭代详情查看
- 更新 App 组件支持迭代选择
- 统一命令面板

### API 使用示例

#### CLI 使用

```bash
# 初始化项目
cowork init "MyProject"

# 创建首个迭代 (Genesis)
cowork iter "初始化项目" --description "构建一个Web应用"

# 基于已完成迭代创建新迭代
cowork iter "添加用户系统" --base iter-1-xxx

# 查看所有迭代
cowork list --all

# 继续暂停的迭代
cowork continue
```

#### 编程 API

```rust
use cowork_core::domain::{Project, Iteration};
use cowork_core::persistence::{ProjectStore, IterationStore};
use cowork_core::pipeline::IterationExecutor;

// 创建项目
let project_store = ProjectStore::new();
let project = project_store.create("MyProject")?;

// 创建迭代执行器
let executor = IterationExecutor::new(interaction);

// 创建并执行 Genesis 迭代
let iteration = executor.create_genesis_iteration(
    &mut project,
    "初始化项目",
    "构建一个Web应用"
)?;

// 执行迭代
executor.execute(&mut project, &iteration.id).await?;
```

## 待完成工作

### Phase 5: 清理收尾 (进行中)
- [ ] 标记旧代码为 deprecated
- [ ] 添加向后兼容层 (如需要)
- [ ] 更新文档
- [ ] 完整 E2E 测试

## 架构对比

### 旧架构
```
Project
├── Session (New/Modify/Revert)
│   ├── ChangeRequest
│   ├── Resume
│   └── Complex branching
└── .cowork/sessions/
```

### 新架构
```
Project
├── Iteration (统一概念)
│   ├── Genesis (首个迭代)
│   ├── Evolution (基于迭代演进)
│   └── Simple linear flow
└── .cowork-v2/iterations/
```

## 关键设计决策

1. **无向后兼容** - 全新数据结构 (.cowork-v2/)
2. **统一术语** - 只用 Iteration
3. **两层记忆** - Project 级 + Iteration 级
4. **智能阶段选择** - 根据变更范围自动确定起始阶段
5. **阶段确认机制** - 关键阶段后人工确认

## 下一步

Phase 5 清理收尾 - 等待测试验证后删除旧代码
