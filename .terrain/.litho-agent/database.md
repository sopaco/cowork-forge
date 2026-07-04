# 数据库概览报告

## 数据库状况

**本项目未使用关系型数据库。**

Cowork Forge 采用 JSON 文件持久化方案，而非传统的关系型数据库。所有数据存储在项目根目录的 `.cowork-v2/` 目录中：

```
.cowork-v2/
├── project.json              # 项目信息
├── iterations/
│   └── {iteration_id}.json   # 每次迭代的快照数据
├── memory/
│   ├── project/
│   │   └── project_memory.json  # 项目级记忆决策/模式
│   └── iterations/
│       └── {iteration_id}.json  # 迭代级知识快照
└── workspace/
    └── {iteration_id}/       # 迭代的工作区代码文件
```

这种设计决策的原因：
1. **无需数据库运维**——桌面工具追求开箱即用，不依赖数据库服务
2. **数据结构动态变化**——JSON 文件天然支持 schema 演化，适合迭代式开发
3. **文件即备份**——整个项目状态就是一组文件，可以直接用 Git 管理
4. **低并发场景**——单用户桌面工具不需要数据库的并发控制能力
