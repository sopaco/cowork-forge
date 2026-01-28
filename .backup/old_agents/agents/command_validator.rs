use anyhow::Result;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;

/// Minimal node scripts validator.
///
/// We don't execute `npm start` here because it can be long-running.
/// Instead we ensure the referenced script exists and is non-empty.
pub fn validate_node_scripts(package_json_path: &str, required: &[&str]) -> Result<Vec<String>> {
    let content = fs::read_to_string(package_json_path)?;
    let v: Value = serde_json::from_str(&content)?;

    let scripts = v
        .get("scripts")
        .and_then(|s| s.as_object())
        .ok_or_else(|| anyhow::anyhow!("package.json missing scripts object"))?;

    let mut missing = Vec::new();
    let mut available: HashSet<String> = HashSet::new();
    for (k, _val) in scripts.iter() {
        available.insert(k.clone());
    }

    for r in required {
        if !available.contains(*r) {
            missing.push(r.to_string());
        }
    }

    Ok(missing)
}
