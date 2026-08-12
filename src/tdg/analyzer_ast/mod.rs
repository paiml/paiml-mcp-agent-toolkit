#![cfg_attr(coverage_nightly, coverage(off))]
use anyhow::Result;
use blake3;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::tdg::{
    config::TdgConfig, AdaptiveThresholdFactory, AdaptiveThresholdManager, AnalysisMetadata,
    ComponentScores, FileIdentity, FullTdgRecord, Grade, Language, MetricCategory,
    OperationPriority, PenaltyTracker, PlatformResourceController, ProjectScore,
    ResourceControllerFactory, SchedulerFactory, SemanticSignature, SimpleFairScheduler, TdgScore,
    TieredStorageFactory, TieredStore,
};

/// AST-based TDG analyzer - proper implementation per specification
pub struct TdgAnalyzerAst {
    pub(crate) config: TdgConfig,
    storage: Option<TieredStore>,
    scheduler: Option<SimpleFairScheduler>,
    adaptive_manager: Option<AdaptiveThresholdManager>,
    resource_controller: Option<PlatformResourceController>,
    /// Sprint 65: Optional git context for commit correlation
    git_context: Option<crate::models::git_context::GitContext>,
}

include!("analyzer_impl1.rs");
include!("analyzer_impl2.rs");
include!("visitors.rs");

#[cfg(test)]
mod cwd_independence_tests;
