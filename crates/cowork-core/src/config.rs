// Global configuration for Cowork Forge

use std::sync::Mutex;

/// Global system locale (set by GUI on startup)
static SYSTEM_LOCALE: Mutex<Option<String>> = Mutex::new(None);

/// Get the current system locale
pub fn get_system_locale() -> String {
    let guard = SYSTEM_LOCALE.lock().unwrap();
    guard.clone().unwrap_or_else(|| "en-US".to_string())
}

/// Set the system locale (called by GUI on startup)
pub fn set_system_locale(locale: String) {
    let mut guard = SYSTEM_LOCALE.lock().unwrap();
    *guard = Some(locale);
}

/// Build language instruction based on system locale
pub fn get_language_instruction() -> String {
    let locale = get_system_locale();
    
    if locale.starts_with("zh") {
        "请使用简体中文回复。所有生成的内容（文档、代码注释、说明等）都应使用简体中文。".to_string()
    } else if locale.starts_with("ja") {
        "日本語で返信してください。生成されるコンテンツ（ドキュメント、コードコメント、説明など）は日本語を使用してください。".to_string()
    } else if locale.starts_with("ko") {
        "한국어로 답변해 주세요.".to_string()
    } else if locale.starts_with("es") {
        "Por favor, responda en español.".to_string()
    } else if locale.starts_with("fr") {
        "Veuillez répondre en français.".to_string()
    } else if locale.starts_with("de") {
        "Bitte antworten Sie auf Deutsch.".to_string()
    } else {
        "Please respond in English.".to_string()
    }
}
