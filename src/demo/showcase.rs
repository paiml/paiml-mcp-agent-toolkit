//! Repository Showcase Gallery
//!
//! This module provides a curated collection of example repositories that demonstrate
//! the capabilities of PMAT across different languages, frameworks, and architectural patterns.
//! The showcase serves as both a demo and a reference for users exploring the tool.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Showcase repository.
pub struct ShowcaseRepository {
    pub name: String,
    pub url: String,
    pub description: String,
    pub primary_language: String,
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub category: RepositoryCategory,
    pub complexity_tier: ComplexityTier,
    pub estimated_analysis_time_seconds: u32,
    pub highlights: Vec<String>,
    pub analysis_preview: Option<AnalysisPreview>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
/// Category classification for repository.
pub enum RepositoryCategory {
    WebFramework,
    SystemsProgramming,
    DataScience,
    CloudNative,
    DeveloperTools,
    GameDevelopment,
    MachineLearning,
    Blockchain,
    Mobile,
    Embedded,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
/// Complexity tier.
pub enum ComplexityTier {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Analysis preview.
pub struct AnalysisPreview {
    pub estimated_files: u32,
    pub estimated_functions: u32,
    pub estimated_complexity: f64,
    pub key_insights: Vec<String>,
    pub recommended_focus_areas: Vec<String>,
}

/// Showcase gallery.
pub struct ShowcaseGallery {
    repositories: HashMap<String, ShowcaseRepository>,
    categories: HashMap<RepositoryCategory, Vec<String>>,
}

#[derive(Debug, Serialize)]
/// Summary of showcase analysis.
pub struct ShowcaseSummary {
    pub total_repositories: usize,
    pub total_languages: usize,
    pub total_categories: usize,
    pub complexity_distribution: HashMap<ComplexityTier, usize>,
    pub featured_count: usize,
    pub quick_start_count: usize,
}

impl Default for ShowcaseGallery {
    fn default() -> Self {
        Self::new()
    }
}

// Repository data initialization (Tokio, Django, React, Kubernetes, etc.)
include!("showcase_repositories.rs");

// Gallery constructor and query/filter methods
include!("showcase_queries.rs");

// Unit tests and property tests
include!("showcase_tests.rs");
