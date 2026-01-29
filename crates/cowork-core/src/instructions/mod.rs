// Agent instructions - Prompt templates for each agent

pub mod idea;
pub mod prd;
pub mod design;
pub mod plan;
pub mod coding;
pub mod check;
pub mod delivery;
pub mod modify;
pub mod code_patch;
pub mod modify_delivery;

pub use idea::*;
pub use prd::*;
pub use design::*;
pub use plan::*;
pub use coding::*;
pub use check::*;
pub use delivery::*;
pub use modify::*;
pub use code_patch::*;
pub use modify_delivery::*;
