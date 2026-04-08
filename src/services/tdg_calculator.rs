#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]
//! Technical Debt Gradient (TDG) calculator for code quality assessment
//!
//! This module implements the TDG scoring system, a comprehensive metric that
//! combines multiple quality dimensions to identify code that needs attention.
//! TDG replaces traditional defect probability with a more nuanced approach
//! that considers complexity, churn, coupling, duplication, and domain risk.
//!
//! # TDG Components
//!
//! The TDG score is calculated from five key components:
//! - **Complexity Factor**: Cyclomatic and cognitive complexity metrics
//! - **Churn Factor**: Frequency and magnitude of code changes
//! - **Coupling Factor**: Dependencies and architectural entanglement
//! - **Duplication Factor**: Code clone detection and similarity analysis
//! - **Domain Risk Factor**: Business criticality and security considerations
//!
//! # Scoring System
//!
//! TDG scores range from 0.0 to 5.0:
//! - `0.0-1.0`: Excellent code quality (green)
//! - `1.0-2.0`: Good quality with minor issues (yellow)
//! - `2.0-3.0`: Moderate issues requiring attention (orange)
//! - `3.0-5.0`: High risk code needing immediate refactoring (red)
//!
//! # Example
//!
//! ```ignore
//! use pmat::services::tdg_calculator::TDGCalculator;
//! use pmat::models::tdg::TDGConfig;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let calculator = TDGCalculator::new();
//! let config = TDGConfig::default();
//!
//! // Analyze a single file
//! let file_path = PathBuf::from("src/main.rs");
//! let score = calculator.calculate_file(file_path, config).await?;
//!
//! println!("TDG Score: {:.2}", score.value);
//! println!("Severity: {:?}", score.severity);
//! println!("Components:");
//! println!("  Complexity: {:.2}", score.components.complexity);
//! println!("  Churn: {:.2}", score.components.churn);
//! println!("  Coupling: {:.2}", score.components.coupling);
//!
//! // Analyze entire project
//! let analysis = calculator.analyze_project(PathBuf::from("."), config).await?;
//! println!("Project average TDG: {:.2}", analysis.summary.average_score);
//! # Ok(())
//! # }
//! ```ignore

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use tokio::sync::{Mutex, Semaphore};

use crate::models::tdg::{
    RecommendationType, TDGAnalysis, TDGBucket, TDGComponents, TDGConfig, TDGDistribution,
    TDGHotspot, TDGRecommendation, TDGScore, TDGSeverity, TDGSummary,
};
use crate::models::unified_ast::{AstKind, UnifiedAstNode};
use crate::services::file_discovery::ProjectFileDiscovery;
use crate::services::git_analysis::GitAnalysisService;
use crate::services::lightweight_provability_analyzer::{
    FunctionId, LightweightProvabilityAnalyzer,
};
use crate::services::unified_ast_engine::UnifiedAstEngine;
use crate::services::verified_complexity::VerifiedComplexityAnalyzer;

// Core types, struct definitions, constructors, and scoring primitives
include!("tdg_calculator_core.rs");

// Factor calculation: complexity
include!("tdg_calculator_factors.rs");

// Factor calculation: churn (git history + fallback)
include!("tdg_calculator_factors_churn.rs");

// Factor calculation: coupling, duplication, domain risk, provability
include!("tdg_calculator_factors_coupling.rs");

// Analysis, explanation, recommendations, and distribution
include!("tdg_calculator_analysis.rs");

// Default implementation moved to models/tdg.rs via #[derive(Default)]
// CB-128: TDGComponents now has 6 fields including dead_code

// Tests extracted to tdg_calculator_tests.rs for file health compliance (CB-040)
#[cfg(test)]
#[path = "tdg_calculator_tests.rs"]
mod tests;
