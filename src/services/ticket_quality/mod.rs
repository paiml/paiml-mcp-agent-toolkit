//! Ticket quality (FLOW-OWNER-001 §3 and §4): a fixed 0-100 rubric over a ticket's
//! text, and the findings over a repository's open tickets. Pure checks; pmat reads
//! GitHub and never writes it here.

pub mod findings;
pub mod rubric;
pub mod run;
pub mod source;

#[cfg(test)]
mod findings_tests;
#[cfg(test)]
mod rubric_tests;
#[cfg(test)]
mod run_tests;
#[cfg(test)]
mod source_tests;
#[cfg(test)]
mod test_support;
