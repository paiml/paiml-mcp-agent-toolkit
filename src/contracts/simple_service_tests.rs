//! Tests for simple service
//! Extracted to separate file for file health compliance (CB-040)
//! Split into multiple parts for CB-040 compliance (<500 lines each)

use super::*;

#[path = "simple_service_tests_part1.rs"]
mod simple_service_tests_part1;

#[path = "simple_service_tests_part2.rs"]
mod simple_service_tests_part2;

#[path = "simple_service_tests_part3.rs"]
mod simple_service_tests_part3;

#[path = "simple_service_tests_part4.rs"]
mod simple_service_tests_part4;
