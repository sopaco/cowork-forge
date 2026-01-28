# CLI接口域技术文档

## 模块概述

CLI接口域是Cowork Forge系统的用户交互入口模块，负责命令行界面的解析、会话管理、管道协调和项目状态跟踪。作为AI驱动软件开发编排器的主要用户界面，该模块采用异步架构设计，支持完整的项目生命周期管理。

## 技术架构

### 核心依赖
- **clap**: 命令行参数解析库
- **tokio**: 异步运行时
- **adk_runner**: AI代理执行引擎
- **cowork_core**: 核心业务逻辑模块
- **tracing**: 结构化日志系统

### 模块结构
```rust
crates/cowork-cli/
└── src/
    └── main.rs           # CLI主入口模块
```

## 核心功能实现

### 1. 命令行接口设计

#### 命令解析框架
```rust
[derive(Parser)]
#[command(name = "cowork")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    // 全局参数配置
}

#[derive(Subcommand)]
enum Commands {
    New { idea: String },        // 新建项目
    Resume { base: Option<String> }, // 恢复项目
    Revert { from: String },     // 回滚项目
    Modify { idea: String, base: Option<String> }, // 修改项目
    Status { sessions: bool },   // 状态查询
    Init,                       // 初始化配置
}
```

#### 异步执行入口
```rust
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    // 日志配置和初始化
    setup_logging(&cli);
    
    match cli.command {
        Commands::New { idea } => cmd_new(idea, &config, enable_stream).await?,
        // 其他命令处理...
    }
    Ok(())
}
```

### 2. 项目生命周期管理

#### 新建项目流程 (`cmd_new`)
```rust
async fn cmd_new(idea: String, config: &ModelConfig, enable_stream: bool) -> Result<()> {
    // 1. 项目初始化检查
    if is_project_initialized() {
        return Err(anyhow!("Project already initialized"));
    }
    
    // 2. 创建项目索引和会话记录
    let project_name = extract_project_name(&idea);
    let mut index = init_project_index(project_name)?;
    let session_id = generate_session_id();
    
    // 3. 保存会话元数据
    let session_record = SessionRecord::new(session_id.clone(), SessionType::New, idea.clone());
    index.add_session(session_record);
    save_project_index(&index)?;
    
    // 4. 创建并执行新建管道
    let pipeline = create_cowork_pipeline(config, &session_id)?;
    execute_pipeline(pipeline, &idea, enable_stream).await?;
    
    // 5. 标记会话完成状态
    mark_session_completed(&session_id)?;
    Ok(())
}
```

#### 恢复项目流程 (`cmd_resume`)
```rust
async fn cmd_resume(base: Option<String>, config: &ModelConfig, enable_stream: bool) -> Result<()> {
    // 1. 确定基础会话ID
    let base_session_id = determine_base_session(base)?;
    
    // 2. 创建新会话记录
    let session_id = generate_session_id();
    let session_record = SessionRecord::resume(session_id.clone(), base_session_id.clone());
    
    // 3. 从基础会话初始化状态
    init_session_from_base(&session_id, &base_session_id)?;
    
    // 4. 创建恢复管道并执行
    let pipeline = create_resume_pipeline(config, &session_id, &base_session_id)?;
    execute_pipeline(pipeline, "Resume from last checkpoint", enable_stream).await?;
    
    Ok(())
}
```

#### 回滚项目流程 (`cmd_revert`)
```rust
async fn cmd_revert(from_stage: &str, config: &ModelConfig, enable_stream: bool) -> Result<()> {
    // 支持自动阶段检测
    let resolved_stage = if from_stage == "auto" {
        detect_restart_stage()?
    } else {
        from_stage
    };
    
    // 创建部分管道（从指定阶段开始）
    let pipeline = create_partial_pipeline(config, &session_id, &base_session_id, resolved_stage)?;
    execute_pipeline(pipeline, &format!("Revert from {} stage", resolved_stage), enable_stream).await?;
}
```

#### 增量修改流程 (`cmd_modify`)
```rust
async fn cmd_modify(idea: &str, base: Option<String>, config: &ModelConfig, enable_stream: bool) -> Result<()> {
    // 1. 创建变更请求
    let change_request = ChangeRequest::new(session_id.clone(), idea.to_string(), base_session_id.clone());
    
    // 2. 文件变更跟踪
    let before_files = collect_project_file_fingerprints()?;
    
    // 3. 执行修改管道
    let pipeline = create_modify_pipeline(config, &session_id, &base_session_id)?;
    execute_pipeline(pipeline, idea, enable_stream).await?;
    
    // 4. 生成补丁元数据
    let after_files = collect_project_file_fingerprints()?;
    let patch = generate_patch_metadata(&before_files, &after_files);
    save_patch_metadata(&session_id, &patch)?;
}
```

### 3. 管道执行引擎

#### 异步管道执行器
```rust
async fn execute_pipeline(pipeline: Arc<dyn adk_core::Agent>, input: &str, enable_stream: bool) -> Result<()> {
    // 1. 创建会话服务
    let session_service = Arc::new(InMemorySessionService::new());
    
    // 2. 配置运行器
    let runner = Runner::new(RunnerConfig {
        app_name: "cowork-forge".to_string(),
        agent: pipeline,
        session_service,
        // 其他配置...
    })?;
    
    // 3. 执行AI代理流程
    let mut event_stream = runner.run(user_id, session_id, content).await?;
    
    // 4. 处理流式输出
    while let Some(event_result) = event_stream.next().await {
        match event_result {
            Ok(event) => {
                if enable_stream {
                    // 实时显示LLM思考过程
                    display_streaming_output(&event);
                }
            }
            Err(e) => handle_execution_error(e)?,
        }
    }
}
```

### 4. 项目状态管理

#### 状态查询功能
```rust
async fn cmd_status(show_sessions: bool) -> Result<()> {
    let index = load_project_index()?;
    
    if show_sessions {
        // 显示所有会话详情
        display_session_details(&index);
    } else {
        // 显示项目摘要
        display_project_summary(&index);
    }
}
```

#### 文件指纹收集
```rust
fn collect_project_file_fingerprints() -> Result<HashMap<String, (u64, u64)>> {
    let mut fingerprints = HashMap::new();
    
    for entry in WalkDir::new(".") {
        if should_ignore_path(entry.path()) { continue; }
        
        let metadata = entry.metadata()?;
        fingerprints.insert(
            entry.path().to_string_lossy().into_owned(),
            (metadata.len(), extract_mtime(&metadata))
        );
    }
    Ok(fingerprints)
}
```

## 技术特性

### 1. 异步架构设计
- 基于tokio异步运行时，支持高并发操作
- 非阻塞I/O操作，提高系统响应性
- 流式事件处理，实时显示执行进度

### 2. 错误恢复机制
- 多层错误处理和回滚策略
- 会话状态持久化，支持中断恢复
- 优雅的错误信息展示

### 3. 用户交互优化
- 实时进度指示器
- 流式LLM输出显示
- 结构化日志输出到stderr

### 4. 配置管理
```rust
fn load_config(path: &str) -> Result<ModelConfig> {
    if Path::new(path).exists() {
        ModelConfig::from_file(path)  // 文件配置优先
    } else {
        ModelConfig::from_env()       // 环境变量备用
    }
}
```

## 会话生命周期管理

### 会话状态流转
```
新建会话 → 执行中 → 完成/失败
    ↑         |
    └── 恢复/回滚 ──┘
```

### 会话元数据结构
```rust
struct SessionRecord {
    session_id: String,
    session_type: SessionType,    // New/Resume/Revert/Modify
    status: SessionStatus,        // InProgress/Completed/Failed
    base_session_id: Option<String>, // 基础会话引用
    created_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}
```

## 性能优化特性

### 1. 文件操作优化
- 增量文件变更检测，避免全量扫描
- 路径过滤机制，忽略构建产物和系统文件
- 并行文件指纹计算

### 2. 内存管理
- 按需加载会话数据
- 智能缓存策略
- 资源及时释放

### 3. 网络通信优化
- LLM API调用速率限制
- 请求重试机制
- 连接池管理

## 扩展性设计

### 命令扩展接口
通过clap的Subcommand特性，支持轻松添加新命令：
```rust
#[derive(Subcommand)]
enum Commands {
    // 现有命令...
    #[command(name = "export")] 
    Export { format: String }, // 新增导出命令
}
```

### 配置扩展支持
配置文件采用TOML格式，支持灵活的配置项扩展：
```toml
[llm]
api_base_url = "http://localhost:8000/v1"
api_key = "your-api-key"

[storage]
engine = "file"  # 未来支持数据库存储
```

## 安全考虑

### 1. 路径安全验证
```rust
fn should_ignore_project_path(path: &str) -> bool {
    let unsafe_patterns = ["../", "/etc/", "/root/"];
    !unsafe_patterns.iter().any(|p| path.contains(p))
}
```

### 2. 会话隔离
- 每个会话独立的工作目录
- 基于会话ID的文件命名空间
- 跨会话访问控制

### 3. 输入验证
- 命令行参数类型检查
- 文件路径规范化
- 敏感信息过滤

CLI接口域作为系统的用户交互层，成功地将复杂的AI驱动开发流程封装为简单易用的命令行工具，为软件开发团队提供了高效的项目管理界面。