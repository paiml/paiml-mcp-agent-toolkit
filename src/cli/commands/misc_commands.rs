#![cfg_attr(coverage_nightly, coverage(off))]
// Misc command types - extracted for file health (CB-040)

use super::config_hooks::{ConfigCommands, ConfigFormat};
use crate::cli::{OutputFormat, TdgOutputFormat};
use clap::Subcommand;
use std::path::PathBuf;

// CUDA-SIMD TDG and Oracle command types
include!("misc_commands_cuda_oracle.rs");

// Comply command types and output formats
include!("misc_commands_comply.rs");

// Spec, Debug, QualityGates, and Maintain command types
include!("misc_commands_spec_debug.rs");

// TDG, Storage, Diagnostic, and Baseline command types
include!("misc_commands_tdg_storage.rs");
