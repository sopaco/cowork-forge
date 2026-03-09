use std::process::Command;

/// Initialize system locale at application startup
/// This should be called during Tauri setup to ensure locale is available
/// before any pipeline execution
pub fn init_system_locale() {
    let locale = detect_system_locale();
    cowork_core::set_system_locale(locale.clone());
    println!("[GUI] System locale initialized: {}", locale);
}

/// Detect system locale without setting it
fn detect_system_locale() -> String {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
            .args(["-Command", "(Get-Culture).Name"])
            .output();

        if let Ok(output) = output {
            let locale = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !locale.is_empty() {
                return locale;
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("sh")
            .args([
                "-c",
                "defaults read -g AppleLocale 2>/dev/null || echo 'en-US'",
            ])
            .output();

        if let Ok(output) = output {
            let locale = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !locale.is_empty() && locale != "en-US" {
                return locale;
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(locale) = std::env::var("LANG") {
            // Parse locale like "zh_CN.UTF-8" to "zh-CN"
            let locale = locale.split('.').next().unwrap_or("en-US");
            let locale = locale.replace('_', "-");
            if !locale.is_empty() {
                return locale;
            }
        }
    }

    "en-US".to_string()
}

#[tauri::command]
pub fn get_system_locale() -> String {
    // Return cached locale if already set
    let cached = cowork_core::get_system_locale();
    if cached != "en-US" {
        return cached;
    }

    // Otherwise detect and set
    let locale = detect_system_locale();
    cowork_core::set_system_locale(locale.clone());
    locale
}
