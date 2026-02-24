pub mod file;
pub mod preview;
pub mod runner;
pub mod memory;
pub mod template;
pub mod pm;
pub mod system;

use lazy_static::lazy_static;
use std::sync::Mutex;

use crate::project_runner::ProjectRunner;

lazy_static! {
    pub static ref PROJECT_RUNNER: ProjectRunner = ProjectRunner::new();
    pub static ref RUNTIME_ANALYZER: Mutex<Option<cowork_core::RuntimeAnalyzer>> = Mutex::new(None);
}

pub fn init_app_handle(handle: tauri::AppHandle) {
    PROJECT_RUNNER.set_app_handle(handle);
}
