#![cfg_attr(coverage_nightly, coverage(off))]
//! Command execution logic - dispatch, scaffold, maintain, debug, forward

mod debug_exec;
mod dispatch;
mod dispatch_ext;
mod dispatch_ext_advanced;
mod dispatch_ext_scoring;
mod dispatch_ext_tools;
mod forward;
mod maintain;
mod scaffold;
mod tests;
mod tests_integration;
mod tests_property;
