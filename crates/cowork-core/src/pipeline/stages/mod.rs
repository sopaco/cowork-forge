// Pipeline stages

use super::{PipelineContext, Stage, StageResult};

pub mod idea;
pub mod prd;
pub mod design;
pub mod plan;
pub mod coding;
pub mod check;
pub mod delivery;

pub use coding::CodingStage;
pub use check::CheckStage;
pub use delivery::DeliveryStage;
pub use design::DesignStage;
pub use idea::IdeaStage;
pub use plan::PlanStage;
pub use prd::PrdStage;
