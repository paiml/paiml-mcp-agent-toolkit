//! Gate smells, wave 2: GS-03, GS-04, GS-05, GS-12, GS-14 (paiml/paiml-mcp-agent-toolkit#1673).
//!
//! Static and deterministic: each detector reads a workflow file already on disk and
//! never touches the network. GS-12 needs one fact that lives in git, not in the file
//! (is this sha reachable from the shared repo's default branch?); that fact is injected
//! as a resolver, and a resolver that cannot answer yields `not_measured`, never a pass.
//!
//! | ID    | Smell                                                           |
//! |-------|-----------------------------------------------------------------|
//! | GS-03 | a PR job reads live external state, so the verdict is not f(commit) |
//! | GS-04 | a gate self-test runs on the product path                       |
//! | GS-05 | a roll-up job (only `needs` + a result check) on a heavy runner |
//! | GS-12 | shared workflow pinned to a sha not on the default branch       |
//! | GS-14 | a repo-global check on the per-PR path                          |

mod detectors;
mod model;
mod scan;

#[cfg(test)]
mod tests;

pub use detectors::{detect_gs03, detect_gs04, detect_gs05, detect_gs12, detect_gs14};
pub use model::{Finding, Report, Workflow};
pub use scan::scan_workflows;
