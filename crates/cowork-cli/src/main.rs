use anyhow::Result;
use clap::{Parser, Subcommand};
use cowork_core::{ArtifactStore, Orchestrator, ModelConfig};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "cowork")]
#[command(about = "AI-powered multi-agent software development system", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to model configuration file (TOML)
    #[arg(long, default_value = "项目材料/大模型配置说明.md")]
    config: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Resume a session
    Resume {
        session_id: String,
    },
    /// Inspect a session's artifacts
    Inspect {
        session_id: String,
    },
    /// Export final deliverables
    Export {
        session_id: String,
    },
    /// Modify requirements or design and trigger re-execution
    Modify {
        session_id: String,
        /// Modification description (if not provided, will prompt interactively)
        #[arg(short, long)]
        change: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    let cli = Cli::parse();

    // Load model configuration
    let model_config = ModelConfig::from_file(&cli.config)
        .or_else(|e| {
            tracing::warn!("Failed to load config from file: {}, trying environment variables", e);
            ModelConfig::from_env()
        })?;

    tracing::info!("Model configuration loaded:");
    tracing::info!("  LLM: {} at {}", model_config.llm.model_name, model_config.llm.api_base_url);

    // Initialize ArtifactStore
    let store = ArtifactStore::new(".cowork");
    let orchestrator = Orchestrator::new(store);

    match cli.command {
        None => {
            // Default: interactive mode - create new session
            interactive_mode(orchestrator, model_config).await?;
        }
        Some(Commands::Resume { session_id }) => {
            resume_session(orchestrator, &session_id, model_config).await?;
        }
        Some(Commands::Inspect { session_id }) => {
            inspect_session(orchestrator, &session_id)?;
        }
        Some(Commands::Export { session_id }) => {
            export_session(&session_id)?;
        }
        Some(Commands::Modify { session_id, change }) => {
            modify_session(orchestrator, &session_id, change, model_config).await?;
        }
    }

    Ok(())
}

async fn interactive_mode(orchestrator: Orchestrator, model_config: ModelConfig) -> Result<()> {
    use console::style;

    println!("{}", style("Welcome to Cowork!").bold().cyan());
    println!("AI-powered multi-agent software development system\n");

    // Create new session
    let session_id = orchestrator.create_session()?;
    println!("Session created: {}\n", style(&session_id).green());

    // Run workflow
    println!("Starting workflow...\n");
    orchestrator.run_full_workflow(&session_id, &model_config).await?;

    println!("\n{}", style("Session completed!").bold().green());
    println!("Session ID: {}", session_id);
    println!("Artifacts saved to: .cowork/{}/artifacts/", session_id);

    Ok(())
}

async fn resume_session(orchestrator: Orchestrator, session_id: &str, model_config: ModelConfig) -> Result<()> {
    use console::style;

    println!("{}", style(format!("🔄 恢复会话: {}", session_id)).bold().cyan());

    // 调用 orchestrator 的 resume_session 方法
    orchestrator.resume_session(session_id, &model_config).await?;

    println!("\n{}", style("✅ 会话恢复完成！").bold().green());

    Ok(())
}

fn inspect_session(orchestrator: Orchestrator, session_id: &str) -> Result<()> {
    use console::style;
    use cowork_core::StageStatus;

    println!("{}", style(format!("🔍 检查会话: {}", session_id)).bold().cyan());

    // 加载 session meta
    let meta = orchestrator.load_session_meta(session_id)?;
    println!("\n📊 会话信息:");
    println!("  创建时间: {}", meta.created_at);
    println!("  当前阶段: {:?}", meta.current_stage);
    
    // 显示已完成的阶段
    let completed_stages: Vec<_> = meta.stage_status.iter()
        .filter(|(_, status)| matches!(status, StageStatus::Completed { .. }))
        .map(|(stage, _)| stage)
        .collect();
    println!("  已完成阶段: {:?}", completed_stages);

    let artifacts = orchestrator.list_artifacts(session_id)?;

    if artifacts.is_empty() {
        println!("{}", style("\n⚠️  没有找到 artifacts").yellow());
        return Ok(());
    }

    println!("\n📦 Artifacts ({} 个):", artifacts.len());
    for artifact in artifacts {
        println!("  ┌─ {} ({:?})", artifact.artifact_id, artifact.stage);
        println!("  │  JSON: {}", artifact.path_json.display());
        println!("  └─ MD:   {}", artifact.path_md.display());
    }

    // 显示下一步建议
    let all_stages = cowork_core::Stage::all();
    let next_stage = all_stages
        .iter()
        .find(|s| !matches!(meta.stage_status.get(s), Some(StageStatus::Completed { .. })))
        .cloned();

    if let Some(stage) = next_stage {
        println!("\n💡 提示:");
        println!("  下一阶段: {:?}", stage);
        println!("  恢复命令: cowork resume {}", session_id);
    } else {
        println!("\n✅ 所有阶段已完成！");
    }

    Ok(())
}

fn export_session(session_id: &str) -> Result<()> {
    use console::style;
    use std::fs;
    use std::path::PathBuf;

    println!("{}", style(format!("📤 导出会话: {}", session_id)).bold().cyan());

    let session_dir = PathBuf::from(".cowork").join(session_id);
    if !session_dir.exists() {
        return Err(anyhow::anyhow!("Session {} not found", session_id));
    }

    // 创建导出目录
    let export_dir = PathBuf::from("exports").join(session_id);
    fs::create_dir_all(&export_dir)?;

    // 复制所有 markdown 文件
    let artifacts_dir = session_dir.join("artifacts");
    let mut exported_count = 0;

    if artifacts_dir.exists() {
        for entry in fs::read_dir(&artifacts_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                let file_name = path.file_name().unwrap();
                let dest = export_dir.join(file_name);
                fs::copy(&path, &dest)?;
                println!("  ✓ {}", file_name.to_string_lossy());
                exported_count += 1;
            }
        }
    }

    // 复制 meta.json
    let meta_src = session_dir.join("meta.json");
    if meta_src.exists() {
        fs::copy(&meta_src, export_dir.join("meta.json"))?;
        println!("  ✓ meta.json");
        exported_count += 1;
    }

    println!("\n✅ 导出完成！");
    println!("  导出文件数: {}", exported_count);
    println!("  导出目录: {}", export_dir.display());

    Ok(())
}

async fn modify_session(
    orchestrator: Orchestrator,
    session_id: &str,
    change: Option<String>,
    model_config: ModelConfig,
) -> Result<()> {
    use console::style;
    use cowork_core::{HitlController, StageStatus};

    println!("{}", style(format!("🔧 修改会话: {}", session_id)).bold().cyan());

    // 检查 session 是否存在
    let meta = orchestrator.load_session_meta(session_id)?;
    
    // 显示已完成的阶段
    let completed_stages: Vec<_> = meta.stage_status.iter()
        .filter(|(_, status)| matches!(status, StageStatus::Completed { .. }))
        .map(|(stage, _)| stage)
        .collect();
    
    println!("\n📊 当前会话状态:");
    println!("  创建时间: {}", meta.created_at);
    println!("  已完成阶段: {:?}", completed_stages);
    println!("  Feedback 迭代次数: {}/{}", meta.feedback_iterations, meta.max_feedback_iterations);

    // 获取修改内容
    let hitl = HitlController::new();
    let modification = if let Some(c) = change {
        c
    } else {
        println!("\n请描述您的修改需求（可以是需求变更、技术调整等）:");
        hitl.input("修改内容")?
    };

    if modification.trim().is_empty() {
        return Err(anyhow::anyhow!("修改内容不能为空"));
    }

    println!("\n🔄 正在处理修改请求...");
    println!("修改内容: {}", modification);

    // 调用 orchestrator 的 modify_and_rerun 方法
    orchestrator.modify_and_rerun(session_id, &modification, &model_config).await?;

    println!("\n{}", style("✅ 修改完成！").bold().green());

    Ok(())
}
