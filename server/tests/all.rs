//! Unified test binary - all integration tests in one binary
//!
//! This consolidates 192 separate test files into a single binary,
//! reducing llvm-cov report generation time from 15+ min to <5 min.
//!
//! The key insight: each .rs file in tests/ compiles to a separate binary.
//! By using `autotests = false` and consolidating into one [[test]], we get:
//! - 1 binary instead of 192
//! - Faster compilation (shared codegen)
//! - Much faster coverage reports

mod modules;
