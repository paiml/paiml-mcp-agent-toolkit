#![allow(unused)]
//! Advanced Analysis Tool Handlers
//! Split for file health compliance (CB-040)
#![cfg_attr(coverage_nightly, coverage(off))]

// R22-2 / D102: Glob-aware `project_path` resolution lives in the shared
// `crate::services::path_glob` module so both MCP dispatcher trees call the
// same implementation.
use crate::services::path_glob::{resolve_project_path_with_globs, ResolvedProjectPath};

include!("tools_advanced_part1.rs");
include!("tools_advanced_part2.rs");
include!("tools_advanced_part3.rs");
include!("tools_advanced_part4.rs");
