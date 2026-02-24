#![cfg_attr(coverage_nightly, coverage(off))]
//! Interactive scaffolding interface for guided agent creation.

use super::context::{AgentContext, AgentContextBuilder};
use super::error::ScaffoldError;
use super::features::{AgentFeature, QualityLevel};
use super::hybrid::{CoreSpec, FallbackStrategy, ModelType, VerificationMethod, WrapperSpec};
use super::templates::AgentTemplate;
use anyhow::Result;
use console::Term;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use std::collections::HashSet;
use std::path::PathBuf;

// Core struct definition, impl blocks, and Default impl
include!("interactive_impl.rs");

// Unit tests
include!("interactive_tests.rs");

// Property-based tests and integration tests
include!("interactive_tests_advanced.rs");
