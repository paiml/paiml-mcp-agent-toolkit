//! Defect prediction analysis: the library entry point for the command.
//!
//! #948/#954 shape: this module used to hold a second, private implementation
//! of `analyze defect-prediction` — its own summary/detailed/JSON/CSV/SARIF
//! renderers over a model fed by a comment-density number called `churn_score`
//! and two hardcoded `0.0`s. The CLI never routed here. The duplicate is
//! deleted; see [`handler`] for what it computed and why syncing it was the
//! wrong repair.

mod handler;

// Re-export the main handler for external use
pub use handler::handle_analyze_defect_prediction;
