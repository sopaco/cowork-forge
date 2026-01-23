use crate::verification::ProjectKind;
use std::path::Path;

/// Detect project kind by file fingerprints.
///
/// This is intentionally shallow and deterministic.
pub fn detect_project_kind(root: &str) -> ProjectKind {
    let root_path = Path::new(root);

    // Rust
    if root_path.join("Cargo.toml").exists() {
        return ProjectKind::Rust;
    }

    // Node/JS/TS
    if root_path.join("package.json").exists() {
        return ProjectKind::Node;
    }

    // Python
    if has_any_py_file(root_path) {
        return ProjectKind::Python;
    }

    // HTML
    if has_any_ext(root_path, "html") {
        return ProjectKind::Html;
    }

    ProjectKind::Unknown
}

fn has_any_py_file(root: &Path) -> bool {
    has_any_ext(root, "py")
}

fn has_any_ext(root: &Path, ext: &str) -> bool {
    if !root.exists() {
        return false;
    }
    let walker = ignore::WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .follow_links(false)
        .build();

    for entry in walker.flatten() {
        let p = entry.path();
        if p.is_file() {
            if p.extension().and_then(|s| s.to_str()) == Some(ext) {
                return true;
            }
        }
    }
    false
}
