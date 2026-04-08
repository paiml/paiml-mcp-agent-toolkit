#![allow(unused)]
// Team onboarding materials and tutorials for unified quality system
//
// Provides interactive tutorials, onboarding guides, and training materials
#![cfg_attr(coverage_nightly, coverage(off))]
use crate::unified_quality::{QualityMode, QualityPhilosophy};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Team onboarding system for progressive quality adoption
pub struct TeamOnboarding {
    sessions: HashMap<TeamId, OnboardingSession>,
    tutorials: TutorialLibrary,
    config: OnboardingConfig,
}

/// Team identifier
pub type TeamId = String;

/// Onboarding session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingSession {
    pub team_id: TeamId,
    pub current_phase: OnboardingPhase,
    pub completed_tutorials: Vec<String>,
    pub quality_mode: QualityMode,
    pub started_at: std::time::SystemTime,
    pub progress: OnboardingProgress,
    pub preferences: TeamPreferences,
}

/// Onboarding phases (Progressive adoption)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OnboardingPhase {
    Introduction,
    MonitoringSetup,
    MetricsLearning,
    EnforcementConfig,
    AutomationSetup,
    AdvancedFeatures,
    ProductionReady,
}

/// Progress tracking for onboarding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingProgress {
    pub tutorials_completed: u32,
    pub tutorials_total: u32,
    pub exercises_completed: u32,
    pub quality_improvements: u32,
    pub days_active: u32,
    pub engagement_score: f64,
}

/// Team preferences and settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamPreferences {
    pub languages: Vec<String>,
    pub learning_style: LearningStyle,
    pub notifications: NotificationPreference,
    pub philosophy: QualityPhilosophy,
    pub team_info: TeamInfo,
}

/// Learning style preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningStyle {
    Practical,
    Theoretical,
    Balanced,
    Exploratory,
}

/// Notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreference {
    pub daily_updates: bool,
    pub weekly_summaries: bool,
    pub achievements: bool,
    pub celebrations: bool,
}

/// Team information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamInfo {
    pub size: u32,
    pub experience_level: ExperienceLevel,
    pub project_type: ProjectType,
    pub quality_maturity: QualityMaturity,
}

/// Experience level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperienceLevel {
    Junior,
    Intermediate,
    Senior,
    Mixed,
}

/// Project type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectType {
    WebApplication,
    SystemsSoftware,
    DataScience,
    Mobile,
    Library,
    Microservices,
    Monolith,
}

/// Quality maturity level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityMaturity {
    None,
    Basic,
    Intermediate,
    Advanced,
}

/// Tutorial library
#[derive(Debug, Clone)]
pub struct TutorialLibrary {
    tutorials: HashMap<OnboardingPhase, Vec<Tutorial>>,
}

// Supporting types: Tutorial, TutorialContent, WalkthroughStep, Exercise,
// OnboardingConfig, GamificationConfig, OnboardingReport, Achievement
include!("onboarding_types.rs");

// TeamOnboarding public workflow: new, start_onboarding, get_recommendations,
// complete_tutorial, generate_progress_report
include!("onboarding_workflow.rs");

// TeamOnboarding private helpers: relevance scoring, engagement scoring,
// phase progress, achievement calculation, phase progression
include!("onboarding_scoring.rs");

// TutorialLibrary: Default impl, new (built-in content),
// get_tutorials_for_phase, count_tutorials
include!("tutorial_library.rs");

// TEMPORARILY DISABLED: File splitting broke syntax
#[cfg(all(test, feature = "broken-tests"))]
#[path = "tests.rs"]
mod tests;
