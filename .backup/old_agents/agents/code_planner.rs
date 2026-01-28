use anyhow::Result;
use adk_rust::prelude::*;
use adk_rust::model::{OpenAIClient, OpenAIConfig};
use adk_rust::runner::{Runner, RunnerConfig};
use adk_rust::session::{InMemorySessionService, CreateRequest, SessionService, GetRequest};
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;

use crate::artifacts::*;
use crate::memory::ArtifactStore;
use crate::config::LlmConfig;
use crate::tools::create_file_tools;

/// Code Planner - 基于 Plan 生成代码变更计划
/// 采用分阶段策略避免 max iteration 问题
/// 注意：这是规划阶段，不执行实际的文件操作
pub struct CodePlanner {
    model: Arc<OpenAIClient>,
    store: Arc<ArtifactStore>,
}

impl CodePlanner {
    pub fn new(llm_config: &LlmConfig, store: Arc<ArtifactStore>) -> Result<Self> {
        let config = OpenAIConfig::compatible(
            llm_config.api_key.clone(),
            llm_config.api_base_url.clone(),
            llm_config.model_name.clone(),
        );
        
        tracing::info!("Creating Code Planner with OpenAI-compatible client");
        
        let model = OpenAIClient::new(config)?;

        Ok(Self {
            model: Arc::new(model),
            store,
        })
    }

    pub async fn execute(
        &self, 
        session_id: &str,
        prd_artifact: &PRDArtifact,
        design_artifact: &DesignDocArtifact,
        plan_artifact: &PlanArtifact
    ) -> Result<CodeChangeArtifact> {
        tracing::info!("CodePlanner: generating code change plan for session {}", session_id);

        // 🆕 读取修改上下文（如果有）
        let modification_context = self.load_modification_context(session_id)?;
        if let Some(ref ctx) = modification_context {
            tracing::info!("Modification context found: {}", ctx);
            println!("📌 检测到修改请求: {}", ctx);
        }

        // 分阶段执行策略：
        // 1. 先分析项目结构（使用工具）
        // 2. 再生成代码变更计划（基于 PRD + Design + Plan，不使用工具）
        
        // Phase 1: 项目结构分析
        tracing::info!("Phase 1: Analyzing project structure...");
        let project_context = self.analyze_project_structure(session_id).await?;
        
        // Phase 2: 生成代码变更计划（基于分析结果和需求）
        tracing::info!("Phase 2: Generating code change plan...");
        let code_change = self.generate_code_plan(
            session_id,
            prd_artifact,
            design_artifact, 
            plan_artifact, 
            &project_context,
            modification_context.as_deref()  // 🆕 传递修改上下文
        ).await?;

        // 保存 artifact
        let summary = vec![
            format!("Language: {}", code_change.target.lang),
            format!("Modules: {}", code_change.project.modules.len()),
            format!("Changes: {}", code_change.changes.len()),
            format!("Commands: {}", code_change.cmds.len()),
        ];

        let artifact = ArtifactEnvelope::new(session_id.to_string(), Stage::Coding, code_change)
            .with_summary(summary)
            .with_prev(vec![plan_artifact.meta.artifact_id.clone()]);

        self.store.put(session_id, Stage::Coding, &artifact)?;

        tracing::info!("Code change artifact saved successfully");

        Ok(artifact)
    }

    /// 🆕 从 SessionMeta 读取修改上下文
    fn load_modification_context(&self, session_id: &str) -> Result<Option<String>> {
        use std::fs;
        use std::path::PathBuf;

        let meta_path = PathBuf::from(".cowork")
            .join(session_id)
            .join("meta.json");

        if !meta_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&meta_path)?;
        let meta: serde_json::Value = serde_json::from_str(&content)?;
        
        Ok(meta.get("modification_context")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()))
    }

    /// Phase 1: 分析项目结构（限制工具调用次数）
    async fn analyze_project_structure(&self, session_id: &str) -> Result<String> {
        let file_tools = create_file_tools();

        // 使用简化的 agent，只做项目结构分析
        let agent = Arc::new(
            LlmAgentBuilder::new("project_analyzer")
                .description("Analyze project structure efficiently")
                .instruction(
                    r#"You are a project structure analyzer. Your task is to understand the current project layout.

**IMPORTANT RULES TO AVOID MAX ITERATIONS:**
1. Call list_directory ONLY ONCE on the root directory (recursive=true)
2. Based on the file list, identify key directories (src/, tests/, etc.)
3. Read at most 2-3 key files (README.md, Cargo.toml, package.json, etc.)
4. After gathering information, output your findings in JSON format
5. DO NOT explore every file - just get the overview

**Output JSON Format:**
{
  "project_type": "rust|javascript|python|unknown",
  "layout": "mono|single",
  "key_dirs": ["src", "tests", "docs"],
  "package_manager": "cargo|npm|pip|unknown",
  "existing_files": ["list of important files"],
  "notes": "brief observations"
}

Remember: Maximum 5 tool calls total. Focus on efficiency."#,
                )
                .model(self.model.clone())
                .output_key("project_analysis")
                .tool(file_tools.list_dir.clone())
                .tool(file_tools.read_file.clone())
                .tool(file_tools.file_exists.clone())
                .build()?,
        );

        let session_service = Arc::new(InMemorySessionService::new());
        let app_name = "cowork".to_string();
        let user_id = format!("{}_analysis", session_id);

        let _session = session_service
            .create(CreateRequest {
                app_name: app_name.clone(),
                user_id: user_id.clone(),
                session_id: Some(format!("{}_phase1", session_id)),
                state: HashMap::new(),
            })
            .await?;

        let runner = Runner::new(RunnerConfig {
            app_name: app_name.clone(),
            agent: agent.clone(),
            session_service: session_service.clone(),
            artifact_service: None,
            memory_service: None,
            run_config: None,
        })?;

        let input_content = Content::new("user").with_text(
            "Analyze the current project structure in the current directory (.)"
        );

        tracing::info!("Analyzing project structure...");

        let mut event_stream = runner
            .run(user_id.clone(), format!("{}_phase1", session_id), input_content)
            .await?;

        while let Some(event_result) = event_stream.next().await {
            match event_result {
                Ok(_event) => {},
                Err(e) => {
                    tracing::error!("Error during project analysis: {}", e);
                    return Err(anyhow::anyhow!("Project analysis failed: {}", e));
                }
            }
        }

        let updated_session = session_service
            .get(GetRequest {
                user_id: user_id.clone(),
                session_id: format!("{}_phase1", session_id),
                app_name: app_name.clone(),
                after: None,
                num_recent_events: None,
            })
            .await?;

        let state = updated_session.state();
        let analysis = state
            .get("project_analysis")
            .ok_or_else(|| anyhow::anyhow!("No analysis output"))?;

        let analysis_str = match analysis {
            serde_json::Value::String(s) => s.clone(),
            v => serde_json::to_string_pretty(&v)?,
        };

        tracing::info!("Project analysis complete");
        Ok(analysis_str)
    }

    /// Phase 2: 生成代码变更计划（基于需求、设计和项目分析，不使用工具）
    async fn generate_code_plan(
        &self,
        session_id: &str,
        prd_artifact: &PRDArtifact,
        design_artifact: &DesignDocArtifact,
        plan_artifact: &PlanArtifact,
        project_context: &str,
        modification_context: Option<&str>,  // 🆕 新增参数
    ) -> Result<CodeChange> {
        let output_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "target": {
                    "type": "object",
                    "properties": {
                        "lang": {"type": "string"},
                        "stack": {"type": "array", "items": {"type": "string"}},
                        "build": {"type": "array", "items": {"type": "string"}},
                        "test": {"type": "array", "items": {"type": "string"}}
                    },
                    "required": ["lang", "stack", "build", "test"]
                },
                "project": {
                    "type": "object",
                    "properties": {
                        "root": {"type": "string"},
                        "layout": {"type": "string", "enum": ["mono", "single", "unknown"]},
                        "modules": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "name": {"type": "string"},
                                    "path": {"type": "string"},
                                    "type": {"type": "string", "enum": ["service", "lib", "app", "pkg", "unknown"]}
                                },
                                "required": ["name", "path", "type"]
                            }
                        },
                        "tooling": {
                            "type": "object",
                            "properties": {
                                "pkg": {"type": "string"},
                                "build": {"type": "array", "items": {"type": "string"}},
                                "test": {"type": "array", "items": {"type": "string"}},
                                "lint": {"type": "array", "items": {"type": "string"}}
                            },
                            "required": ["pkg", "build", "test", "lint"]
                        }
                    },
                    "required": ["root", "layout", "modules", "tooling"]
                },
                "changes": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "path": {"type": "string"},
                            "kind": {"type": "string"},
                            "note": {"type": "string"}
                        },
                        "required": ["path", "kind", "note"]
                    }
                },
                "cmds": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "cmd": {"type": "string"},
                            "expect": {"type": "string"},
                            "phase": {"type": "string", "enum": ["check", "build", "test", "lint", "run"]}
                        },
                        "required": ["cmd", "expect", "phase"]
                    }
                },
                "requirement_mapping": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "req_id": {"type": "string"},
                            "files": {"type": "array", "items": {"type": "string"}},
                            "note": {"type": "string"}
                        },
                        "required": ["req_id", "files", "note"]
                    }
                }
            },
            "required": ["target", "project", "changes", "cmds", "requirement_mapping"]
        });

        // 提取关键信息 - 从实际的 artifact 结构中提取
        // PRD: target (从 IdeaSpec), features (从 reqs)
        let target = format!("{}; Goals: {}", 
            &prd_artifact.data.scope.g.join(", "),
            &prd_artifact.data.scope.ng.join(", ")
        );
        
        let features: Vec<String> = prd_artifact.data.reqs.iter()
            .take(5)
            .map(|r| format!("{}: {}", r.id, r.desc))
            .collect();
        
        let tech_requirements: Vec<String> = prd_artifact.data.cons.iter()
            .map(|c| format!("{}: {}", c.id, c.desc))
            .collect();

        // DesignDoc: cli, wf, arch, io
        let architecture_layers = design_artifact.data.arch.layers.join(", ");
        let components = design_artifact.data.arch.comps.join(", ");
        let workflow_stages = design_artifact.data.wf.stages.join(", ");

        // 压缩任务信息，只保留关键内容
        let task_summary: Vec<String> = plan_artifact.data.tasks.iter()
            .take(5)  // 只取前5个任务
            .map(|t| format!("{}: {}", t.id, t.desc))
            .collect();
        
        // ✅ 提取并强调 TodoList
        let todo_context = if let Some(ref todo_list) = plan_artifact.data.todo_list {
            let mut lines = vec!["**TodoList (IMPORTANT - ensure all related files are generated):**".to_string()];
            for item in &todo_list.items {
                lines.push(format!("- {}: {}", item.id, item.description));
                if !item.related_files.is_empty() {
                    lines.push(format!("  Files to generate: {}", item.related_files.join(", ")));
                }
                if !item.related_requirements.is_empty() {
                    lines.push(format!("  Requirements: {}", item.related_requirements.join(", ")));
                }
            }
            lines.push("".to_string());
            lines.push("CRITICAL: Every file mentioned in TodoList must be included in the 'changes' array.".to_string());
            lines.push("".to_string());
            lines.join("\n")
        } else {
            String::new()
        };

        let context = format!(
            r#"Based on the user requirements, design decisions, and implementation plan, generate a code change plan.

{}

{}

**User Requirements (from PRD):**
- Target Scope: {}
- Key Features:
{}
- Technical Constraints:
{}

**Design Decisions (from DesignDoc):**
- CLI Modes: {}
- Workflow Stages: {}
- Architecture Layers: {}
- Key Components: {}
- Artifact Formats: {}

**Project Analysis (current state):**
{}

**Implementation Plan Summary:**
- C4 Context: {}
- C4 Containers: {}
- C4 Components: {}
- Top 5 Tasks:
{}

**Milestones:**
{}

**CRITICAL RULES FOR LANGUAGE/TECH STACK DETECTION:**
1. Analyze the requirements and design to infer the target technology
2. If requirements mention "web", "HTML", "browser", "frontend" → generate .html, .css, .js files
3. If requirements mention "Python", "Flask", "Django" → generate .py files
4. If requirements mention "Rust", "cargo", or current project is Rust → generate .rs files
5. If requirements mention "Node", "JavaScript", "npm" → generate .js/.ts and package.json
6. DO NOT blindly copy the current project structure!
7. Match the file types to what the user actually wants to build

Generate a comprehensive but concise code change plan."#,
            todo_context,
            // 🆕 添加修改上下文（如果有）
            if let Some(modification) = modification_context {
                format!(
                    r#"
🔧 **MODIFICATION MODE - CRITICAL INSTRUCTIONS:**
This is a MODIFICATION task, NOT creating from scratch!

**User's Modification Request:**
"{}"

**MANDATORY RULES:**
1. Check the "Project Analysis" section for "existing_files"
2. If a file already exists in the project → use "kind": "modify", NOT "create"
3. ONLY modify the parts related to the user's request
4. DO NOT regenerate the entire project
5. DO NOT change files that are not related to the modification
6. Preserve existing code structure and functionality
7. Focus on implementing ONLY what the user asked for

**Example:**
- User says "改为中文" (change to Chinese) → modify text content in HTML/JS files
- User says "改用 PostgreSQL" → modify database config and connection files
- User says "增加登录功能" → add new login-related files, modify relevant existing files

**WRONG behavior:**
❌ Regenerating all files with "create"
❌ Changing unrelated functionality
❌ Rewriting the entire project

**CORRECT behavior:**
✅ Using "modify" for existing files
✅ Only touching files related to the modification
✅ Adding new files ONLY if necessary
"#,
                    modification
                )
            } else {
                String::new()
            },
            target,
            features.join("\n  "),
            tech_requirements.join("\n  "),
            design_artifact.data.cli.modes.join(", "),
            workflow_stages,
            architecture_layers,
            components,
            design_artifact.data.io.formats.join(", "),
            project_context,
            plan_artifact.data.c4.context.join(", "),
            plan_artifact.data.c4.containers.join(", "),
            plan_artifact.data.c4.components.join(", "),
            task_summary.join("\n  "),
            plan_artifact.data.milestones.iter()
                .take(3)  // 只取前3个里程碑
                .map(|m| format!("{}: {}", m.id, m.desc))
                .collect::<Vec<_>>()
                .join("\n  "),
        );

        // 创建无工具的 agent（避免工具调用循环）
        let agent = Arc::new(
            LlmAgentBuilder::new("code_planner")
                .description("Generate code change plan based on requirements, design and analysis")
                .instruction(
                    r#"You are a code planning specialist. Based on the project analysis, user requirements, design decisions, and implementation plan, create a detailed code change plan WITH requirement mapping AND verification commands.

**CRITICAL: Respect the target language in the Design document!**

Language-specific file generation rules:
- If Design says "html", "web", or "frontend" → generate .html, .css, .js files (NOT .rs files)
- If Design says "python" → generate .py files (NOT .rs files)
- If Design says "rust" → generate .rs files and Cargo.toml
- If Design says "javascript" or "node" → generate .js files and package.json
- If Design says "typescript" → generate .ts files and tsconfig.json

**Output Requirements:**
1. Respond with ONLY valid JSON (no markdown, no explanations, just the JSON object)
2. File paths MUST match the target language specified in Design
3. The "lang" field in output MUST match the Design language
4. tooling.pkg MUST match: "none" for html, "npm" for js/ts, "pip" for python, "cargo" for rust
5. Be specific about file paths based on language conventions
6. **MUST include requirement_mapping** - map each requirement ID to implementing files
7. **MUST include cmds** - verification/build/test commands (cross-language)

**Requirement Mapping Guidelines:**
1. For each requirement ID (REQ-001, REQ-002, etc.), list which files implement it
2. Provide a brief note explaining how the files address the requirement
3. One requirement can map to multiple files
4. One file can implement multiple requirements
5. Ensure ALL requirements from PRD are mapped

**Commands Generation Guidelines (IMPORTANT - Keep It Simple):**
Generate a MINIMAL list of verification commands in the "cmds" array:

**SIMPLICITY PRINCIPLE:**
- Focus ONLY on basic syntax validation and running the application
- Do NOT add testing frameworks, linters, or coverage tools unless explicitly required
- Keep commands minimal and essential
- Prefer "no commands" over complex build pipelines for simple projects

**Command Priority (execute in this order, but ONLY if necessary):**
1. **check** - Basic syntax validation (optional for simple projects)
2. **build** - Compilation/bundling (only if needed)
3. **run** - Quick sanity check (avoid long-running servers)

**Language-Specific Command Examples (MINIMAL):**

**Rust projects:**
[
  {"cmd": "cargo check", "expect": "compiles without errors", "phase": "check"},
  {"cmd": "cargo build", "expect": "builds successfully", "phase": "build"}
]
// NOTE: Skip cargo test unless testing is explicitly required

**Node/JavaScript/TypeScript projects:**
[
  {"cmd": "npm install", "expect": "dependencies installed", "phase": "build"}
]
// NOTE: Skip npm run lint, npm run build, npm test unless explicitly required
// For simple projects, just npm install is enough

**Python projects:**
[
  {"cmd": "pip install -r requirements.txt", "expect": "dependencies installed", "phase": "build"}
]
// NOTE: Skip pytest, pylint unless testing is explicitly required

**Static HTML/CSS/JS projects:**
[]
// NOTE: No commands needed for static sites - they work directly in browser

**Command Rules:**
- **DEFAULT TO EMPTY ARRAY** for simple projects
- Only add commands that are ESSENTIAL to verify the code runs
- Do NOT add: test runners, linters, formatters, coverage tools
- Do NOT add: CI/CD commands, deployment scripts
- Keep it minimal - user can add more later if needed

**Example for HTML/Web project (no build tools):**
{
  "target": {
    "lang": "html",
    "stack": ["vanilla-js", "css3"],
    "build": [],
    "test": []
  },
  "project": {
    "root": "./",
    "layout": "single",
    "modules": [],
    "tooling": {
      "pkg": "none",
      "build": [],
      "test": [],
      "lint": []
    }
  },
  "changes": [
    {"path": "index.html", "kind": "create", "note": "Main HTML structure"},
    {"path": "styles.css", "kind": "create", "note": "Styling"},
    {"path": "script.js", "kind": "create", "note": "Interactivity"}
  ],
  "cmds": [],
  "requirement_mapping": [
    {
      "req_id": "REQ-001",
      "files": ["index.html", "styles.css"],
      "note": "Semantic HTML structure and responsive design implement this requirement"
    },
    {
      "req_id": "REQ-002",
      "files": ["script.js"],
      "note": "JavaScript handles interactivity for this requirement"
    }
  ]
}

**Example for Node/TypeScript project:**
{
  "target": {
    "lang": "typescript",
    "stack": ["node", "express"],
    "build": ["npm run build"],
    "test": ["npm test"]
  },
  "project": {
    "root": "./",
    "layout": "single",
    "modules": [{"name": "api", "path": "src/api", "type": "service"}],
    "tooling": {
      "pkg": "npm",
      "build": ["npm run build"],
      "test": ["npm test"],
      "lint": ["npm run lint"]
    }
  },
  "changes": [
    {"path": "package.json", "kind": "create", "note": "Project metadata and scripts"},
    {"path": "tsconfig.json", "kind": "create", "note": "TypeScript config"},
    {"path": "src/index.ts", "kind": "create", "note": "Entry point"}
  ],
  "cmds": [
    {"cmd": "npm install", "expect": "dependencies installed", "phase": "build"},
    {"cmd": "npm run build", "expect": "TypeScript compiles", "phase": "build"},
    {"cmd": "npm test", "expect": "tests pass", "phase": "test"}
  ],
  "requirement_mapping": [...]
}

Follow the exact JSON schema provided in the context."#,
                )
                .model(self.model.clone())
                .output_schema(output_schema)
                .output_key("code_plan")
                .build()?,
        );

        let session_service = Arc::new(InMemorySessionService::new());
        let app_name = "cowork".to_string();
        let user_id = format!("{}_planning", session_id);

        let _session = session_service
            .create(CreateRequest {
                app_name: app_name.clone(),
                user_id: user_id.clone(),
                session_id: Some(format!("{}_phase2", session_id)),
                state: HashMap::new(),
            })
            .await?;

        let runner = Runner::new(RunnerConfig {
            app_name: app_name.clone(),
            agent: agent.clone(),
            session_service: session_service.clone(),
            artifact_service: None,
            memory_service: None,
            run_config: None,
        })?;

        let input_content = Content::new("user").with_text(&context);

        tracing::info!("Generating code plan...");

        let mut event_stream = runner
            .run(user_id.clone(), format!("{}_phase2", session_id), input_content)
            .await?;

        while let Some(event_result) = event_stream.next().await {
            match event_result {
                Ok(_event) => {},
                Err(e) => {
                    tracing::error!("Error during code planning: {}", e);
                    return Err(anyhow::anyhow!("Code planning failed: {}", e));
                }
            }
        }

        let updated_session = session_service
            .get(GetRequest {
                user_id: user_id.clone(),
                session_id: format!("{}_phase2", session_id),
                app_name: app_name.clone(),
                after: None,
                num_recent_events: None,
            })
            .await?;

        let state = updated_session.state();
        let raw_output = state
            .get("code_plan")
            .ok_or_else(|| anyhow::anyhow!("No output from Code planner"))?;

        // 增强的 JSON 解析，带详细错误信息
        let code_change: CodeChange = match raw_output {
            serde_json::Value::String(json_str) => {
                tracing::debug!("Parsing JSON string output");
                serde_json::from_str(json_str.as_str()).map_err(|e| {
                    tracing::error!("JSON parse error: {}", e);
                    tracing::error!("Raw JSON string (first 500 chars): {}", 
                        &json_str.chars().take(500).collect::<String>());
                    anyhow::anyhow!(
                        "Failed to parse code plan JSON: {}\n\
                        This usually means the LLM didn't follow the schema correctly.\n\
                        Common issues:\n\
                        - modules must be array of objects, not strings\n\
                        - All required fields must be present\n\
                        Please check the logs for the raw JSON output.",
                        e
                    )
                })?
            }
            value => {
                tracing::debug!("Parsing JSON value output");
                serde_json::from_value(value.clone()).map_err(|e| {
                    tracing::error!("JSON parse error: {}", e);
                    tracing::error!("Raw JSON value: {}", 
                        serde_json::to_string_pretty(&value).unwrap_or_else(|_| "unparseable".to_string()));
                    anyhow::anyhow!(
                        "Failed to parse code plan JSON: {}\n\
                        This usually means the LLM didn't follow the schema correctly.\n\
                        Common issues:\n\
                        - modules must be array of objects with name/path/type fields\n\
                        - Each module must be {{\"name\": \"...\", \"path\": \"...\", \"type\": \"...\"}}\n\
                        - NOT just strings like [\"module1\", \"module2\"]\n\
                        Please check the logs for the raw JSON output.",
                        e
                    )
                })?
            }
        };

        tracing::info!("Successfully parsed CodeChange");

        Ok(code_change)
    }
}
