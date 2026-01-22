//! Comprehensive tests for demo server
//!
//! Tests all public functions and code paths without actual network operations.
//! Split into multiple parts for file health compliance (CB-040).

use super::*;
use std::collections::HashMap;

#[path = "server_tests_part1.rs"]
mod server_tests_part1;

#[path = "server_tests_part2.rs"]
mod server_tests_part2;

#[path = "server_tests_part3.rs"]
mod server_tests_part3;

#[path = "server_tests_part4.rs"]
mod server_tests_part4;

// Re-export test helpers for use across parts
pub use server_tests_part1::{create_test_dag, create_test_demo_content, create_test_hotspots};
