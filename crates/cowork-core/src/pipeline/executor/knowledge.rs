// Knowledge generation and injection logic for iteration executor

use futures::StreamExt;
use std::sync::Arc;

use crate::domain::{Importance, Iteration, IterationStatus};
use crate::persistence::IterationStore;
use crate::pipeline::{PipelineContext, stage_executor::{SimpleInvocationContext, extract_text_from_event}};
use adk_core::{Content, Llm};

/// Generate summaries for iteration documents using LLM
pub async fn generate_document_summaries(
    iteration_store: &IterationStore,
    iteration: &Iteration,
    model: Arc<dyn Llm>,
) -> anyhow::Result<()> {
    println!(
        "[Executor] Generating document summaries for iteration {}...",
        iteration.id
    );

    let iteration_dir = iteration_store.iteration_path(&iteration.id)?;
    let artifacts_dir = iteration_dir.join("artifacts");
    let summaries_dir = iteration_dir.join("summaries");

    std::fs::create_dir_all(&summaries_dir)?;

    let doc_types = vec!["idea", "prd", "design", "plan"];

    for (idx, doc_type) in doc_types.iter().enumerate() {
        let doc_path = artifacts_dir.join(format!("{}.md", doc_type));

        if !doc_path.exists() {
            println!("[Executor] Warning: {} not found, skipping", doc_type);
            continue;
        }

        let content = std::fs::read_to_string(&doc_path)?;

        let summary_agent = crate::agents::create_summary_agent(
            model.clone(),
            iteration.id.clone(),
            iteration.number,
        )?;

        let prompt = format!(
            "Document Type: {}\n\nDocument Content:\n\n{}\n\nPlease generate a summary following the format specified in your instructions.",
            doc_type, content
        );

        let ctx_content = Content::new("user").with_text(&prompt);
        let dummy_project = crate::domain::Project::new("temp");
        let invocation_ctx = Arc::new(SimpleInvocationContext::new(
            &PipelineContext::new(dummy_project, iteration.clone(), iteration_dir.clone()),
            &ctx_content,
            summary_agent.clone(),
        ));

        let stream = match summary_agent.run(invocation_ctx).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[Executor] Error creating stream for {}: {}", doc_type, e);
                continue;
            }
        };

        let mut generated_text = String::new();
        let mut stream = std::pin::pin!(stream);
        while let Some(result) = stream.next().await {
            if let Ok(event) = result {
                if let Some(text) = extract_text_from_event(&event) {
                    if !text.trim().is_empty() {
                        generated_text.push_str(&text);
                    }
                }
            }
        }

        if generated_text.is_empty() {
            eprintln!("[Executor] No output generated for {}", doc_type);
            continue;
        }

        let summary = extract_summary_from_response(&generated_text);
        let summary_path = summaries_dir.join(format!("{}.md", doc_type));
        std::fs::write(&summary_path, summary)?;

        println!("[Executor] Generated summary for {}", doc_type);

        // Add delay between document summaries to avoid rate limiting
        if idx < doc_types.len() - 1 {
            println!("[Executor] Waiting 2 seconds before processing next document...");
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }

    println!("[Executor] Document summaries generation completed");
    Ok(())
}

/// Generate iteration knowledge using LLM
pub async fn generate_iteration_knowledge(
    iteration_store: &IterationStore,
    iteration: &Iteration,
    model: Arc<dyn Llm>,
) -> anyhow::Result<()> {
    println!(
        "[Executor] Generating iteration knowledge for {}...",
        iteration.id
    );

    let memory_store = crate::persistence::MemoryStore::new();
    let project_memory = memory_store.load_project_memory()?;

    if project_memory
        .get_iteration_knowledge(&iteration.id)
        .is_some()
    {
        println!(
            "[Executor] Knowledge already exists for iteration {}, skipping",
            iteration.id
        );
        return Ok(());
    }

    println!("[Executor] Creating knowledge generation agent...");

    let knowledge_agent = crate::agents::create_knowledge_generation_agent(
        model.clone(),
        iteration.id.clone(),
        iteration.number,
        iteration.base_iteration_id.clone(),
    )?;

    println!("[Executor] Setting iteration ID for tool context...");
    crate::storage::set_iteration_id(iteration.id.clone());

    let prompt = "Please analyze this iteration and generate a comprehensive knowledge snapshot. Use the available tools to load document summaries, examine the codebase structure, and extract meaningful knowledge.";

    println!("[Executor] Creating invocation context...");

    let iteration_dir = iteration_store.iteration_path(&iteration.id)?;
    let ctx_content = Content::new("user").with_text(prompt);
    let dummy_project = crate::domain::Project::new("temp");
    let invocation_ctx = Arc::new(SimpleInvocationContext::new(
        &PipelineContext::new(dummy_project, iteration.clone(), iteration_dir.clone()),
        &ctx_content,
        knowledge_agent.clone(),
    ));

    println!("[Executor] Running knowledge generation agent...");

    let stream = match knowledge_agent.run(invocation_ctx).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[Executor] Failed to create stream: {}", e);
            return Err(anyhow::anyhow!("Failed to create stream: {}", e));
        }
    };

    println!("[Executor] Processing agent stream...");

    let mut stream = std::pin::pin!(stream);
    let mut step_count = 0;
    let mut last_error: Option<anyhow::Error> = None;
    while let Some(result) = stream.next().await {
        step_count += 1;
        if step_count % 10 == 0 {
            println!("[Executor] Stream processing step {}...", step_count);
        }
        if let Err(e) = result {
            eprintln!("[Executor] Stream error at step {}: {}", step_count, e);
            last_error = Some(anyhow::anyhow!("Stream error at step {}: {}", step_count, e));
        }
    }

    if let Some(e) = last_error {
        return Err(e);
    }

    println!(
        "[Executor] Stream processing completed after {} steps",
        step_count
    );

    let project_memory = memory_store.load_project_memory()?;
    if project_memory
        .get_iteration_knowledge(&iteration.id)
        .is_some()
    {
        println!("[Executor] Iteration knowledge generated and saved successfully");
    } else {
        eprintln!(
            "[Executor] Warning: Knowledge generation completed but knowledge not found in project memory"
        );
    }

    Ok(())
}

/// Inject project knowledge into iteration memory (for evolution iterations)
/// Falls back to loading artifacts directly if knowledge summary is not available
pub async fn inject_project_knowledge(
    iteration_store: &IterationStore,
    iteration: &Iteration,
) -> anyhow::Result<()> {
    let base_iteration_id = iteration
        .base_iteration_id
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Evolution iteration must have base_iteration_id"))?;

    println!(
        "[Executor] Injecting project knowledge from base iteration {}...",
        base_iteration_id
    );

    let memory_store = crate::persistence::MemoryStore::new();
    let project_memory = memory_store.load_project_memory()?;
    let base_knowledge = project_memory.get_iteration_knowledge(base_iteration_id);

    let mut iter_memory = memory_store.load_iteration_memory(&iteration.id)?;

    if let Some(knowledge) = base_knowledge {
        // Use knowledge summary (preferred)
        iter_memory.add_insight(
            "project_context",
            format!(
                "## Base Iteration Knowledge (#{})\n\n\
                **Iteration ID**: {}\n\n\
                **Tech Stack**: {}\n\n\
                **Project Vision**: {}\n\n\
                **Key Requirements**: {}\n\n\
                **System Design**: {}\n\n\
                **Implementation**: {}\n\n\
                **Key Decisions**: {}",
                knowledge.iteration_number,
                knowledge.iteration_id,
                knowledge.tech_stack.join(", "),
                knowledge.idea_summary,
                knowledge.prd_summary,
                knowledge.design_summary,
                knowledge.plan_summary,
                knowledge.key_decisions.join("; ")
            ),
        );
    } else {
        // Fallback: Load artifacts directly from base iteration
        println!(
            "[Executor] Knowledge summary not found, loading artifacts directly from base iteration..."
        );

        let base_workspace = iteration_store.workspace_path(base_iteration_id)?;
        let artifacts_dir = base_workspace.parent()
            .map(|p| p.join("artifacts"))
            .unwrap_or_else(|| base_workspace.join("artifacts"));

        let mut context_parts = vec![
            format!("## Base Iteration Context (#{})\n\n", base_iteration_id),
        ];

        for (artifact_name, label) in [
            ("idea.md", "Project Vision"),
            ("prd.md", "Key Requirements"),
            ("design.md", "System Design"),
            ("plan.md", "Implementation Summary"),
        ] {
            let artifact_path = artifacts_dir.join(artifact_name);
            if artifact_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&artifact_path) {
                    let truncated: String = content.chars().take(2000).collect();
                    context_parts.push(format!("### {}\n\n{}\n\n", label, truncated));
                }
            }
        }

        if context_parts.len() > 1 {
            iter_memory.add_insight("project_context", context_parts.join(""));
        } else {
            println!(
                "[Executor] Warning: No artifacts found in base iteration {}",
                base_iteration_id
            );
            iter_memory.add_insight(
                "project_context",
                format!(
                    "## Base Iteration Reference\n\nBase iteration '{}' ({}) is the foundation for this iteration.",
                    iteration.title, base_iteration_id
                ),
            );
        }
    }

    // Mark as critical
    if let Some(last_insight) = iter_memory.insights.last_mut() {
        last_insight.importance = Importance::Critical;
    }

    memory_store.save_iteration_memory(&iter_memory)?;

    println!(
        "[Executor] Project knowledge injected to iteration {}",
        iteration.id
    );
    Ok(())
}

/// Regenerate knowledge for a specific iteration (for recovery)
pub async fn regenerate_iteration_knowledge(
    iteration_store: &IterationStore,
    iteration_id: &str,
    model: Arc<dyn Llm>,
) -> anyhow::Result<()> {
    println!(
        "[Executor] Regenerating knowledge for iteration {}...",
        iteration_id
    );

    let iteration = iteration_store.load(iteration_id)?;

    if iteration.status != IterationStatus::Completed {
        return Err(anyhow::anyhow!(
            "Cannot regenerate knowledge for incomplete iteration"
        ));
    }

    // Remove existing knowledge if any
    let memory_store = crate::persistence::MemoryStore::new();
    let mut project_memory = memory_store.load_project_memory()?;
    project_memory.remove_iteration_knowledge(iteration_id);
    memory_store.save_project_memory(&project_memory)?;

    // Generate summaries first
    generate_document_summaries(iteration_store, &iteration, model.clone()).await?;

    // Then generate knowledge
    generate_iteration_knowledge(iteration_store, &iteration, model).await?;

    println!("[Executor] Knowledge regeneration completed");
    Ok(())
}

/// Extract summary from agent response
fn extract_summary_from_response(response: &str) -> String {
    response.trim().to_string()
}
