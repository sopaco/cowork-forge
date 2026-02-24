use std::process::Command;

#[tauri::command]
pub fn get_system_locale() -> String {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
            .args(["-Command", "(Get-Culture).Name"])
            .output();

        if let Ok(output) = output {
            let locale = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !locale.is_empty() {
                cowork_core::set_system_locale(locale.clone());
                return locale;
            }
        }
    }

    let default_locale = "en-US".to_string();
    cowork_core::set_system_locale(default_locale.clone());
    default_locale
}
