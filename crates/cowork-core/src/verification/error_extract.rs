use regex::Regex;

/// Try to extract affected file paths from stderr/stdout.
///
/// We keep it simple and robust:
/// - TypeScript/JS: "path/to/file.ts:line:col"
/// - Rust: "--> src/main.rs:42:5"
/// - Python: "File \"main.py\", line 10"
pub fn extract_paths(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();

    // TS/JS and generic: foo/bar.ext:12:34
    let re_generic = Regex::new(r"(?m)([A-Za-z0-9_./\\-]+\.(?:ts|tsx|js|jsx|mjs|cjs|rs|py|go|java|kt|cpp|h|hpp)):(\d+):(\d+)").ok();
    if let Some(re) = re_generic {
        for cap in re.captures_iter(text) {
            let p = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            push_unique(&mut out, normalize_path(p));
        }
    }

    // Rust: --> src/main.rs:42:5
    let re_rust = Regex::new(r"(?m)-->\s+([A-Za-z0-9_./\\-]+\.rs):\d+:\d+").ok();
    if let Some(re) = re_rust {
        for cap in re.captures_iter(text) {
            let p = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            push_unique(&mut out, normalize_path(p));
        }
    }

    // Python: File "main.py"
    let re_py = Regex::new(r#"(?m)File\s+\"([^\"]+\.py)\""#).ok();
    if let Some(re) = re_py {
        for cap in re.captures_iter(text) {
            let p = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            push_unique(&mut out, normalize_path(p));
        }
    }

    out
}

fn push_unique(out: &mut Vec<String>, p: String) {
    if p.is_empty() {
        return;
    }
    if !out.contains(&p) {
        out.push(p);
    }
}

fn normalize_path(p: &str) -> String {
    p.replace('\\', "/")
}
