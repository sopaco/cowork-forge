// LLM module - Using adk-rust's built-in OpenAI client with custom base URL
pub mod config;
pub mod rate_limiter;
pub mod timeout;

pub use config::*;
pub use rate_limiter::*;
pub use timeout::*;

use std::sync::Arc;
use adk_core::Llm;
use std::sync::{Mutex, PoisonError};

static CURRENT_EXECUTION_LLM: Mutex<Option<Arc<dyn Llm>>> = Mutex::new(None);

/// Poison 容错的锁获取：跨线程/任务边界保持一致行为，panic 不致后续永久失败。
fn lock_execution_llm() -> std::sync::MutexGuard<'static, Option<Arc<dyn Llm>>> {
    CURRENT_EXECUTION_LLM.lock().unwrap_or_else(PoisonError::into_inner)
}

pub fn set_execution_llm(client: Arc<dyn Llm>) {
    let mut guard = lock_execution_llm();
    *guard = Some(client);
}

pub fn get_execution_llm() -> Option<Arc<dyn Llm>> {
    let guard = lock_execution_llm();
    guard.clone()
}

pub fn clear_execution_llm() {
    let mut guard = lock_execution_llm();
    *guard = None;
}
