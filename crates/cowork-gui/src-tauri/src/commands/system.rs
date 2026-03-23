/// Initialize system locale at application startup
/// This should be called during Tauri setup to ensure locale is available
/// before any pipeline execution
pub fn init_system_locale() {
    let locale = detect_system_locale();
    cowork_core::set_system_locale(locale.clone());
    println!("[GUI] System locale initialized: {}", locale);
}

/// Detect system locale using native platform APIs
/// Uses sys-locale crate which:
/// - Windows: WinAPI GetUserDefaultLocaleName
/// - macOS: CFLocale (CoreFoundation)
/// - Linux: environment variables
fn detect_system_locale() -> String {
    // sys-locale uses native platform APIs, no external process needed
    sys_locale::get_locale()
        .unwrap_or_else(|| "en-US".to_string())
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