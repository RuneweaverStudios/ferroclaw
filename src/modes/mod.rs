//! Modes provide specialized workflows for different types of tasks.
//!
//! Each mode encapsulates a state machine and domain-specific logic:
//! - PlanMode: Structured multi-phase planning with waves and approval gates
//! - (Future): CodeReviewMode, RefactorMode, etc.

pub mod plan;

pub use plan::{ApprovalGate, PlanMode, PlanPhase, PlanStatus, PlanStep, PlanStepStatus, Wave};
