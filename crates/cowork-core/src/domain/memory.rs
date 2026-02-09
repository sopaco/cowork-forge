use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Project-level memory (across iterations)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectMemory {
    pub decisions: Vec<Decision>,
    pub patterns: Vec<Pattern>,
    pub context: ProjectContext,
    /// Iteration knowledge cache - stores knowledge snapshots for each iteration
    pub iteration_knowledge: std::collections::HashMap<String, IterationKnowledge>,
}

impl ProjectMemory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_decision(&mut self, decision: Decision) {
        self.decisions.push(decision);
    }

    pub fn add_pattern(&mut self, pattern: Pattern) {
        self.patterns.push(pattern);
    }

    /// Query decisions by keyword
    pub fn query_decisions(&self, keyword: &str) -> Vec<&Decision> {
        let keyword_lower = keyword.to_lowercase();
        self.decisions
            .iter()
            .filter(|d| {
                d.title.to_lowercase().contains(&keyword_lower)
                    || d.context.to_lowercase().contains(&keyword_lower)
            })
            .collect()
    }

    /// Query patterns by tag
    pub fn query_patterns(&self, tag: &str) -> Vec<&Pattern> {
        let tag_lower = tag.to_lowercase();
        self.patterns
            .iter()
            .filter(|p| p.tags.iter().any(|t| t.to_lowercase() == tag_lower))
            .collect()
    }

    /// Save iteration knowledge to project memory
    pub fn save_iteration_knowledge(&mut self, knowledge: IterationKnowledge) {
        self.iteration_knowledge.insert(knowledge.iteration_id.clone(), knowledge);
    }

    /// Get iteration knowledge by iteration ID
    pub fn get_iteration_knowledge(&self, iteration_id: &str) -> Option<&IterationKnowledge> {
        self.iteration_knowledge.get(iteration_id)
    }

    /// Remove iteration knowledge
    pub fn remove_iteration_knowledge(&mut self, iteration_id: &str) {
        self.iteration_knowledge.remove(iteration_id);
    }

    /// Cleanup old knowledge, keeping only the most recent N iterations
    pub fn cleanup_old_knowledge(&mut self, keep_count: usize) {
        let mut knowledge_vec: Vec<_> = self.iteration_knowledge.values().collect();
        
        // Sort by iteration number
        knowledge_vec.sort_by_key(|k| k.iteration_number);
        
        // Keep only the most recent N
        if knowledge_vec.len() > keep_count {
            let to_remove = knowledge_vec.len() - keep_count;
            let iteration_ids: Vec<String> = knowledge_vec[..to_remove].iter().map(|k| k.iteration_id.clone()).collect();
            for id in iteration_ids {
                self.iteration_knowledge.remove(&id);
            }
        }
    }
}

/// Iteration knowledge snapshot - represents the knowledge learned from an iteration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationKnowledge {
    /// Iteration identifier
    pub iteration_id: String,
    /// Iteration number
    pub iteration_number: u32,
    
    /// Summarized content (not full documents to avoid excessive size)
    pub idea_summary: String,
    pub prd_summary: String,
    pub design_summary: String,
    pub plan_summary: String,
    
    /// Tech stack information
    pub tech_stack: Vec<String>,
    
    /// Key decisions extracted from this iteration
    pub key_decisions: Vec<String>,
    
    /// Key patterns identified in this iteration
    pub key_patterns: Vec<String>,
    
    /// Code structure summary (important files and their purposes)
    pub code_structure: String,
    
    /// Known issues or limitations
    pub known_issues: Vec<String>,
    
    /// Timestamp when knowledge was generated
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl IterationKnowledge {
    pub fn new(iteration_id: String, iteration_number: u32) -> Self {
        let now = chrono::Utc::now();
        Self {
            iteration_id,
            iteration_number,
            idea_summary: String::new(),
            prd_summary: String::new(),
            design_summary: String::new(),
            plan_summary: String::new(),
            tech_stack: Vec::new(),
            key_decisions: Vec::new(),
            key_patterns: Vec::new(),
            code_structure: String::new(),
            known_issues: Vec::new(),
            created_at: now,
        }
    }
}

/// Decision - Key project decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: String,
    pub title: String,
    pub context: String,
    pub decision: String,
    pub consequences: Vec<String>,
    pub iteration_id: String,
    pub created_at: DateTime<Utc>,
}

impl Decision {
    pub fn new(
        title: impl Into<String>,
        context: impl Into<String>,
        decision: impl Into<String>,
        iteration_id: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        let iteration_id = iteration_id.into();
        Self {
            id: format!("dec-{}-{}", iteration_id, now.timestamp()),
            title: title.into(),
            context: context.into(),
            decision: decision.into(),
            consequences: Vec::new(),
            iteration_id,
            created_at: now,
        }
    }
}

/// Pattern - Reusable pattern or best practice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: String,
    pub name: String,
    pub description: String,
    pub usage: Vec<String>,
    pub tags: Vec<String>,
    pub code_example: Option<String>,
    pub iteration_id: String,
    pub created_at: DateTime<Utc>,
}

impl Pattern {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        iteration_id: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        let iteration_id = iteration_id.into();
        Self {
            id: format!("pat-{}-{}", iteration_id, now.timestamp()),
            name: name.into(),
            description: description.into(),
            usage: Vec::new(),
            tags: Vec::new(),
            code_example: None,
            iteration_id,
            created_at: now,
        }
    }
}

/// Project context - Technical context
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectContext {
    pub tech_stack: Vec<String>,
    pub architecture_style: Option<String>,
    pub key_dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub purpose: String,
}

/// Iteration-level memory (current iteration insights)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IterationMemory {
    pub iteration_id: String,
    pub insights: Vec<Insight>,
    pub issues: Vec<Issue>,
    pub learnings: Vec<Learning>,
}

impl IterationMemory {
    pub fn new(iteration_id: impl Into<String>) -> Self {
        Self {
            iteration_id: iteration_id.into(),
            insights: Vec::new(),
            issues: Vec::new(),
            learnings: Vec::new(),
        }
    }

    pub fn add_insight(&mut self, stage: impl Into<String>, content: impl Into<String>) {
        self.insights.push(Insight {
            stage: stage.into(),
            content: content.into(),
            importance: Importance::Important,
            created_at: Utc::now(),
        });
    }

    pub fn add_issue(&mut self, stage: impl Into<String>, content: impl Into<String>) {
        self.issues.push(Issue {
            stage: stage.into(),
            content: content.into(),
            resolved: false,
            created_at: Utc::now(),
            resolved_at: None,
        });
    }

    pub fn add_learning(&mut self, content: impl Into<String>) {
        self.learnings.push(Learning {
            content: content.into(),
            created_at: Utc::now(),
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    pub stage: String,
    pub content: String,
    pub importance: Importance,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub stage: String,
    pub content: String,
    pub resolved: bool,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Learning {
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Importance {
    Critical,
    Important,
    Normal,
}

/// Memory query request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQuery {
    pub scope: MemoryScope,
    pub query_type: MemoryQueryType,
    pub keywords: Vec<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryScope {
    Project,      // Only project-level
    Iteration,    // Only current iteration
    Smart,        // Smart merge (project + current iteration)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryQueryType {
    Decisions,
    Patterns,
    Insights,
    All,
}

/// Memory query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQueryResult {
    pub decisions: Vec<Decision>,
    pub patterns: Vec<Pattern>,
    pub insights: Vec<Insight>,
}

impl MemoryQueryResult {
    pub fn is_empty(&self) -> bool {
        self.decisions.is_empty() && self.patterns.is_empty() && self.insights.is_empty()
    }

    pub fn merge(&mut self, other: MemoryQueryResult) {
        self.decisions.extend(other.decisions);
        self.patterns.extend(other.patterns);
        self.insights.extend(other.insights);
    }

    pub fn to_context_string(&self) -> String {
        let mut parts = Vec::new();

        if !self.decisions.is_empty() {
            parts.push("## Key Decisions\n".to_string());
            for d in &self.decisions {
                parts.push(format!("- {}: {}", d.title, d.decision));
            }
        }

        if !self.patterns.is_empty() {
            parts.push("\n## Patterns\n".to_string());
            for p in &self.patterns {
                parts.push(format!("- {}: {}", p.name, p.description));
            }
        }

        if !self.insights.is_empty() {
            parts.push("\n## Insights\n".to_string());
            for i in &self.insights {
                parts.push(format!("- [{}] {}", i.stage, i.content));
            }
        }

        parts.join("\n")
    }
}

