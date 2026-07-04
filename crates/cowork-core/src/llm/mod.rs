// LLM module - Using adk-rust's built-in OpenAI client with custom base URL
pub mod config;
pub mod rate_limiter;

pub use config::*;
pub use rate_limiter::*;

use std::sync::Arc;
use adk_core::Llm;
use std::sync::Mutex;

static CURRENT_EXECUTION_LLM: Mutex<Option<Arc<dyn Llm>>> = Mutex::new(None);

pub fn set_execution_llm(client: Arc<dyn Llm>) {
    let mut guard = CURRENT_EXECUTION_LLM.lock().unwrap();
    *guard = Some(client);
}

pub fn get_execution_llm() -> Option<Arc<dyn Llm>> {
    let guard = CURRENT_EXECUTION_LLM.lock().unwrap();
    guard.clone()
}

pub fn clear_execution_llm() {
    let mut guard = CURRENT_EXECUTION_LLM.lock().unwrap();
    *guard = None;
}
