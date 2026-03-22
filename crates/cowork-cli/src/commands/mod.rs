//! CLI command modules
//! 
//! Each module implements a single command with clear responsibilities.

pub mod iter;
pub mod list;
pub mod show;
pub mod continue_cmd;
pub mod init;
pub mod status;
pub mod delete;
pub mod knowledge;
pub mod import;
pub mod config;

// Re-export command execution functions with clean names
pub use iter::execute as iter;
pub use list::execute as list;
pub use show::execute as show;
pub use continue_cmd::execute as continue_iteration;
pub use init::execute as init;
pub use status::execute as status;
pub use delete::execute as delete;
pub use knowledge::execute as regenerate_knowledge;
pub use import::execute as import;
pub use config::execute as config;
