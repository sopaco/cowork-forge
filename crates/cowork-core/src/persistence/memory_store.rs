use std::path::PathBuf;

use crate::domain::{IterationMemory, MemoryQuery, MemoryQueryResult, ProjectMemory};

use super::get_cowork_dir;

/// Memory store for persistence
pub struct MemoryStore;

impl MemoryStore {
    pub fn new() -> Self {
        Self
    }

    // Project Memory

    /// Load project memory
    pub fn load_project_memory(&self) -> anyhow::Result<ProjectMemory> {
        let path = self.project_memory_path()?;
        if !path.exists() {
            return Ok(ProjectMemory::new());
        }
        let content = std::fs::read_to_string(&path)?;
        let memory: ProjectMemory = serde_json::from_str(&content)?;
        Ok(memory)
    }

    /// Save project memory
    pub fn save_project_memory(&self, memory: &ProjectMemory) -> anyhow::Result<()> {
        let path = self.project_memory_path()?;
        let content = serde_json::to_string_pretty(memory)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Add decision to project memory
    pub fn add_decision(&self, decision: crate::domain::Decision) -> anyhow::Result<()> {
        let mut memory = self.load_project_memory()?;
        memory.add_decision(decision);
        self.save_project_memory(&memory)
    }

    /// Add pattern to project memory
    pub fn add_pattern(&self, pattern: crate::domain::Pattern) -> anyhow::Result<()> {
        let mut memory = self.load_project_memory()?;
        memory.add_pattern(pattern);
        self.save_project_memory(&memory)
    }

    // Iteration Memory

    /// Load iteration memory
    pub fn load_iteration_memory(&self, iteration_id: &str) -> anyhow::Result<IterationMemory> {
        let path = self.iteration_memory_path(iteration_id)?;
        if !path.exists() {
            return Ok(IterationMemory::new(iteration_id));
        }
        let content = std::fs::read_to_string(&path)?;
        let memory: IterationMemory = serde_json::from_str(&content)?;
        Ok(memory)
    }

    /// Save iteration memory
    pub fn save_iteration_memory(&self, memory: &IterationMemory) -> anyhow::Result<()> {
        let path = self.iteration_memory_path(&memory.iteration_id)?;
        let content = serde_json::to_string_pretty(memory)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Ensure iteration memory exists
    pub fn ensure_iteration_memory(&self, iteration_id: &str) -> anyhow::Result<IterationMemory> {
        let memory = self.load_iteration_memory(iteration_id)?;
        self.save_iteration_memory(&memory)?;
        Ok(memory)
    }

    /// Promote insights to decisions (P1 - memory elevation)
    pub fn promote_insights_to_decisions(&self, iteration_id: &str) -> anyhow::Result<usize> {
        let iteration_memory = self.load_iteration_memory(iteration_id)?;
        let mut project_memory = self.load_project_memory()?;
        
        let mut promoted_count = 0;
        
        // Promote critical insights to decisions
        for insight in &iteration_memory.insights {
            if insight.importance == crate::domain::Importance::Critical {
                let decision = crate::domain::Decision::new(
                    format!("Decision from iteration stage: {}", insight.stage),
                    insight.content.clone(),
                    format!("Critical insight: {}", insight.content),
                    iteration_id
                );
                
                project_memory.add_decision(decision);
                promoted_count += 1;
            }
        }
        
        self.save_project_memory(&project_memory)?;
        Ok(promoted_count)
    }

    // Query

    /// Query memory based on scope and type
    pub fn query(&self, query: &MemoryQuery, current_iteration_id: Option<&str>) -> anyhow::Result<MemoryQueryResult> {
        let mut result = MemoryQueryResult {
            decisions: Vec::new(),
            patterns: Vec::new(),
            insights: Vec::new(),
        };

        match query.scope {
            crate::domain::MemoryScope::Project => {
                let project_memory = self.load_project_memory()?;
                self.apply_query_to_project(&project_memory, query, &mut result);
            }
            crate::domain::MemoryScope::Iteration => {
                if let Some(iter_id) = current_iteration_id {
                    let iter_memory = self.load_iteration_memory(iter_id)?;
                    self.apply_query_to_iteration(&iter_memory, query, &mut result);
                }
            }
            crate::domain::MemoryScope::Smart => {
                // Load both and merge
                let project_memory = self.load_project_memory()?;
                self.apply_query_to_project(&project_memory, query, &mut result);

                if let Some(iter_id) = current_iteration_id {
                    let iter_memory = self.load_iteration_memory(iter_id)?;
                    self.apply_query_to_iteration(&iter_memory, query, &mut result);
                }
            }
        }

        // Apply limit
        if let Some(limit) = query.limit {
            result.decisions.truncate(limit);
            result.patterns.truncate(limit);
            result.insights.truncate(limit);
        }

        Ok(result)
    }

    fn apply_query_to_project(&self, memory: &ProjectMemory, query: &MemoryQuery, result: &mut MemoryQueryResult) {
        use crate::domain::MemoryQueryType;

        match query.query_type {
            MemoryQueryType::Decisions | MemoryQueryType::All => {
                if query.keywords.is_empty() {
                    result.decisions.extend(memory.decisions.clone());
                } else {
                    for keyword in &query.keywords {
                        result.decisions.extend(
                            memory.query_decisions(keyword).into_iter().cloned()
                        );
                    }
                }
            }
            _ => {}
        }

        match query.query_type {
            MemoryQueryType::Patterns | MemoryQueryType::All => {
                if query.keywords.is_empty() {
                    result.patterns.extend(memory.patterns.clone());
                } else {
                    for keyword in &query.keywords {
                        result.patterns.extend(
                            memory.query_patterns(keyword).into_iter().cloned()
                        );
                    }
                }
            }
            _ => {}
        }
    }

    fn apply_query_to_iteration(&self, memory: &IterationMemory, query: &MemoryQuery, result: &mut MemoryQueryResult) {
        use crate::domain::MemoryQueryType;

        match query.query_type {
            MemoryQueryType::Insights | MemoryQueryType::All => {
                result.insights.extend(memory.insights.clone());
            }
            _ => {}
        }
    }

    fn project_memory_path(&self) -> anyhow::Result<PathBuf> {
        Ok(get_cowork_dir()?.join("memory/project/memory.json"))
    }

    fn iteration_memory_path(&self, iteration_id: &str) -> anyhow::Result<PathBuf> {
        Ok(get_cowork_dir()?.join("memory/iterations").join(format!("{}.json", iteration_id)))
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}
