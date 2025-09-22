//! Team onboarding materials and tutorials for unified quality system
//!
//! Provides interactive tutorials, onboarding guides, and training materials

use crate::unified_quality::{QualityMode, QualityPhilosophy};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Team onboarding system for progressive quality adoption
#[allow(dead_code)]
pub struct TeamOnboarding {
    /// Current onboarding sessions
    sessions: HashMap<TeamId, OnboardingSession>,

    /// Tutorial materials
    tutorials: TutorialLibrary,

    /// Configuration
    config: OnboardingConfig,
}

/// Team identifier
pub type TeamId = String;

/// Onboarding session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingSession {
    /// Team information
    pub team_id: TeamId,

    /// Current phase in onboarding
    pub current_phase: OnboardingPhase,

    /// Completed tutorials
    pub completed_tutorials: Vec<String>,

    /// Quality mode progression
    pub quality_mode: QualityMode,

    /// Start date
    pub started_at: std::time::SystemTime,

    /// Progress tracking
    pub progress: OnboardingProgress,

    /// Team preferences
    pub preferences: TeamPreferences,
}

/// Onboarding phases (Progressive adoption)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OnboardingPhase {
    /// Introduction to concepts
    Introduction,

    /// Setting up monitoring
    MonitoringSetup,

    /// Understanding metrics
    MetricsLearning,

    /// Enforcement setup
    EnforcementConfig,

    /// Automation configuration
    AutomationSetup,

    /// Advanced features
    AdvancedFeatures,

    /// Graduation to production
    ProductionReady,
}

/// Progress tracking for onboarding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingProgress {
    /// Tutorials completed
    pub tutorials_completed: u32,

    /// Total tutorials available
    pub tutorials_total: u32,

    /// Hands-on exercises completed
    pub exercises_completed: u32,

    /// Quality improvements demonstrated
    pub quality_improvements: u32,

    /// Days since start
    pub days_active: u32,

    /// Engagement score
    pub engagement_score: f64,
}

/// Team preferences and settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamPreferences {
    /// Preferred programming languages
    pub languages: Vec<String>,

    /// Learning style preference
    pub learning_style: LearningStyle,

    /// Notification preferences
    pub notifications: NotificationPreference,

    /// Quality philosophy alignment
    pub philosophy: QualityPhilosophy,

    /// Team size and composition
    pub team_info: TeamInfo,
}

/// Learning style preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningStyle {
    /// Hands-on, practical approach
    Practical,

    /// Theoretical understanding first
    Theoretical,

    /// Mixed approach
    Balanced,

    /// Self-paced exploration
    Exploratory,
}

/// Notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreference {
    /// Daily progress notifications
    pub daily_updates: bool,

    /// Weekly summary reports
    pub weekly_summaries: bool,

    /// Achievement notifications
    pub achievements: bool,

    /// Quality improvement celebrations
    pub celebrations: bool,
}

/// Team information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamInfo {
    /// Team size
    pub size: u32,

    /// Experience level
    pub experience_level: ExperienceLevel,

    /// Primary project type
    pub project_type: ProjectType,

    /// Quality maturity level
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
    /// No formal quality processes
    None,

    /// Basic testing in place
    Basic,

    /// CI/CD with some quality gates
    Intermediate,

    /// Comprehensive quality culture
    Advanced,
}

/// Tutorial library
#[derive(Debug, Clone)]
pub struct TutorialLibrary {
    /// Available tutorials by phase
    tutorials: HashMap<OnboardingPhase, Vec<Tutorial>>,
}

/// Individual tutorial
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tutorial {
    /// Unique identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Description
    pub description: String,

    /// Estimated duration (minutes)
    pub duration_minutes: u32,

    /// Prerequisites
    pub prerequisites: Vec<String>,

    /// Learning objectives
    pub objectives: Vec<String>,

    /// Interactive content
    pub content: TutorialContent,

    /// Hands-on exercises
    pub exercises: Vec<Exercise>,

    /// Success criteria
    pub success_criteria: Vec<String>,
}

/// Tutorial content types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TutorialContent {
    /// Interactive demo
    Demo {
        title: String,
        description: String,
        commands: Vec<String>,
        expected_outputs: Vec<String>,
    },

    /// Guided walkthrough
    Walkthrough { steps: Vec<WalkthroughStep> },

    /// Video content
    Video {
        title: String,
        url: String,
        transcript: Option<String>,
    },

    /// Documentation
    Documentation {
        title: String,
        content: String,
        examples: Vec<String>,
    },
}

/// Walkthrough step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkthroughStep {
    /// Step number
    pub step: u32,

    /// Title
    pub title: String,

    /// Instructions
    pub instructions: String,

    /// Commands to run
    pub commands: Vec<String>,

    /// Expected results
    pub expected_results: String,

    /// Tips and troubleshooting
    pub tips: Vec<String>,
}

/// Hands-on exercise
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exercise {
    /// Exercise name
    pub name: String,

    /// Instructions
    pub instructions: String,

    /// Starting code/files
    pub starting_files: HashMap<String, String>,

    /// Expected outcome
    pub expected_outcome: String,

    /// Validation criteria
    pub validation: Vec<String>,

    /// Hints for completion
    pub hints: Vec<String>,
}

/// Onboarding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingConfig {
    /// Enable interactive tutorials
    pub interactive_mode: bool,

    /// Personalization enabled
    pub personalization: bool,

    /// Progress tracking enabled
    pub track_progress: bool,

    /// Gamification features
    pub gamification: GamificationConfig,
}

/// Gamification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamificationConfig {
    /// Enable achievements
    pub achievements: bool,

    /// Progress badges
    pub badges: bool,

    /// Leaderboards
    pub leaderboards: bool,

    /// Point system
    pub points: bool,
}

impl TeamOnboarding {
    /// Create new onboarding system
    #[must_use]
    pub fn new(config: OnboardingConfig) -> Self {
        Self {
            sessions: HashMap::new(),
            tutorials: TutorialLibrary::new(),
            config,
        }
    }

    /// Start onboarding for a new team
    pub fn start_onboarding(
        &mut self,
        team_id: TeamId,
        preferences: TeamPreferences,
    ) -> Result<OnboardingSession> {
        let session = OnboardingSession {
            team_id: team_id.clone(),
            current_phase: OnboardingPhase::Introduction,
            completed_tutorials: Vec::new(),
            quality_mode: QualityMode::Observe,
            started_at: std::time::SystemTime::now(),
            progress: OnboardingProgress {
                tutorials_completed: 0,
                tutorials_total: self.tutorials.count_tutorials(),
                exercises_completed: 0,
                quality_improvements: 0,
                days_active: 0,
                engagement_score: 0.0,
            },
            preferences,
        };

        self.sessions.insert(team_id, session.clone());
        Ok(session)
    }

    /// Get personalized tutorial recommendations
    pub fn get_recommendations(&self, team_id: &TeamId) -> Result<Vec<Tutorial>> {
        let session = self
            .sessions
            .get(team_id)
            .ok_or_else(|| anyhow::anyhow!("Team not found: {team_id}"))?;

        let phase_tutorials = self
            .tutorials
            .get_tutorials_for_phase(&session.current_phase);
        let mut recommendations = Vec::new();

        for tutorial in phase_tutorials {
            // Filter based on completed tutorials
            if !session.completed_tutorials.contains(&tutorial.id) {
                // Check prerequisites
                let prerequisites_met = tutorial
                    .prerequisites
                    .iter()
                    .all(|prereq| session.completed_tutorials.contains(prereq));

                if prerequisites_met {
                    recommendations.push(tutorial);
                }
            }
        }

        // Sort by relevance to team preferences
        recommendations.sort_by(|a, b| {
            let relevance_a = self.calculate_relevance(a, &session.preferences);
            let relevance_b = self.calculate_relevance(b, &session.preferences);
            relevance_b
                .partial_cmp(&relevance_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(recommendations)
    }

    /// Complete a tutorial and update progress
    pub fn complete_tutorial(
        &mut self,
        team_id: &TeamId,
        tutorial_id: String,
        exercises_completed: u32,
    ) -> Result<()> {
        let needs_update = {
            let session = self
                .sessions
                .get(team_id)
                .ok_or_else(|| anyhow::anyhow!("Team not found: {team_id}"))?;
            !session.completed_tutorials.contains(&tutorial_id)
        };

        if needs_update {
            // Calculate engagement score first to avoid borrow conflicts
            let engagement_score = {
                let session = self
                    .sessions
                    .get(team_id)
                    .ok_or_else(|| anyhow::anyhow!("Team not found: {team_id}"))?;
                // Create temporary session with updated values for score calculation
                let mut temp_session = session.clone();
                temp_session.progress.tutorials_completed += 1;
                temp_session.progress.exercises_completed += exercises_completed;
                self.calculate_engagement_score(&temp_session)
            };

            let session = self
                .sessions
                .get_mut(team_id)
                .ok_or_else(|| anyhow::anyhow!("Team not found: {team_id}"))?;

            session.completed_tutorials.push(tutorial_id);
            session.progress.tutorials_completed += 1;
            session.progress.exercises_completed += exercises_completed;
            session.progress.engagement_score = engagement_score;

            // Check for phase advancement - clone needed data first
            let current_phase = session.current_phase.clone();
            let tutorials_completed = session.progress.tutorials_completed;

            // Check if advancement is needed
            let should_advance = match current_phase {
                OnboardingPhase::Introduction if tutorials_completed >= 2 => true,
                OnboardingPhase::MonitoringSetup if tutorials_completed >= 4 => true,
                OnboardingPhase::MetricsLearning if tutorials_completed >= 6 => true,
                OnboardingPhase::EnforcementConfig if tutorials_completed >= 8 => true,
                OnboardingPhase::AutomationSetup if tutorials_completed >= 10 => true,
                _ => false,
            };

            if should_advance {
                let next_phase = match current_phase {
                    OnboardingPhase::Introduction => OnboardingPhase::MonitoringSetup,
                    OnboardingPhase::MonitoringSetup => OnboardingPhase::MetricsLearning,
                    OnboardingPhase::MetricsLearning => OnboardingPhase::EnforcementConfig,
                    OnboardingPhase::EnforcementConfig => OnboardingPhase::AutomationSetup,
                    OnboardingPhase::AutomationSetup => OnboardingPhase::AdvancedFeatures,
                    OnboardingPhase::AdvancedFeatures => OnboardingPhase::ProductionReady,
                    OnboardingPhase::ProductionReady => OnboardingPhase::ProductionReady, // Stay at production ready
                };

                session.current_phase = next_phase;
            }
        }

        Ok(())
    }

    /// Generate onboarding report
    pub fn generate_progress_report(&self, team_id: &TeamId) -> Result<OnboardingReport> {
        let session = self
            .sessions
            .get(team_id)
            .ok_or_else(|| anyhow::anyhow!("Team not found: {team_id}"))?;

        let completion_percentage = (f64::from(session.progress.tutorials_completed)
            / f64::from(session.progress.tutorials_total))
            * 100.0;

        let current_phase_progress = self.calculate_phase_progress(session);
        let recommended_next_steps = self.get_recommendations(team_id)?;

        let achievements = self.calculate_achievements(session);

        Ok(OnboardingReport {
            team_id: team_id.clone(),
            current_phase: session.current_phase.clone(),
            overall_completion: completion_percentage,
            phase_progress: current_phase_progress,
            days_active: session.progress.days_active,
            engagement_score: session.progress.engagement_score,
            achievements,
            recommended_next_steps: recommended_next_steps.into_iter().take(3).collect(),
            quality_mode: session.quality_mode,
        })
    }

    /// Calculate tutorial relevance to team preferences
    fn calculate_relevance(&self, tutorial: &Tutorial, preferences: &TeamPreferences) -> f64 {
        let mut relevance: f64 = 0.0;

        // Language relevance
        if preferences.languages.iter().any(|lang| {
            tutorial
                .description
                .to_lowercase()
                .contains(&lang.to_lowercase())
        }) {
            relevance += 0.3;
        }

        // Learning style alignment
        match preferences.learning_style {
            LearningStyle::Practical => {
                if tutorial.exercises.len() > 2 {
                    relevance += 0.2;
                }
            }
            LearningStyle::Theoretical => {
                if matches!(tutorial.content, TutorialContent::Documentation { .. }) {
                    relevance += 0.2;
                }
            }
            _ => relevance += 0.1,
        }

        // Experience level adjustment
        match preferences.team_info.experience_level {
            ExperienceLevel::Junior => {
                if tutorial.duration_minutes <= 30 {
                    relevance += 0.2;
                }
            }
            ExperienceLevel::Senior => {
                if tutorial.exercises.len() > 1 {
                    relevance += 0.2;
                }
            }
            _ => relevance += 0.1,
        }

        relevance.min(1.0_f64)
    }

    /// Calculate engagement score
    fn calculate_engagement_score(&self, session: &OnboardingSession) -> f64 {
        let tutorial_ratio = f64::from(session.progress.tutorials_completed)
            / f64::from(session.progress.tutorials_total);

        let exercise_bonus = (f64::from(session.progress.exercises_completed) * 0.1).min(0.3);
        let improvement_bonus = (f64::from(session.progress.quality_improvements) * 0.05).min(0.2);

        ((tutorial_ratio + exercise_bonus + improvement_bonus) * 100.0).min(100.0)
    }

    /// Calculate current phase progress
    fn calculate_phase_progress(&self, session: &OnboardingSession) -> f64 {
        let phase_tutorials = self
            .tutorials
            .get_tutorials_for_phase(&session.current_phase);
        let completed_in_phase = phase_tutorials
            .iter()
            .filter(|t| session.completed_tutorials.contains(&t.id))
            .count();

        (completed_in_phase as f64 / phase_tutorials.len() as f64) * 100.0
    }

    /// Calculate achievements earned
    fn calculate_achievements(&self, session: &OnboardingSession) -> Vec<Achievement> {
        let mut achievements = Vec::new();

        if session.progress.tutorials_completed >= 5 {
            achievements.push(Achievement {
                id: "tutorial_enthusiast".to_string(),
                name: "Tutorial Enthusiast".to_string(),
                description: "Completed 5 tutorials".to_string(),
                earned_at: session.started_at,
            });
        }

        if session.progress.exercises_completed >= 10 {
            achievements.push(Achievement {
                id: "hands_on_learner".to_string(),
                name: "Hands-on Learner".to_string(),
                description: "Completed 10 exercises".to_string(),
                earned_at: session.started_at,
            });
        }

        if session.progress.engagement_score >= 90.0 {
            achievements.push(Achievement {
                id: "quality_champion".to_string(),
                name: "Quality Champion".to_string(),
                description: "Achieved 90% engagement score".to_string(),
                earned_at: session.started_at,
            });
        }

        achievements
    }

    /// Get next phase in progression
    #[allow(dead_code)]
    fn next_phase(&self, current: &OnboardingPhase) -> OnboardingPhase {
        match current {
            OnboardingPhase::Introduction => OnboardingPhase::MonitoringSetup,
            OnboardingPhase::MonitoringSetup => OnboardingPhase::MetricsLearning,
            OnboardingPhase::MetricsLearning => OnboardingPhase::EnforcementConfig,
            OnboardingPhase::EnforcementConfig => OnboardingPhase::AutomationSetup,
            OnboardingPhase::AutomationSetup => OnboardingPhase::AdvancedFeatures,
            OnboardingPhase::AdvancedFeatures => OnboardingPhase::ProductionReady,
            OnboardingPhase::ProductionReady => OnboardingPhase::ProductionReady, // Stay at final phase
        }
    }

    /// Get recommended quality mode for phase
    #[allow(dead_code)]
    fn recommended_quality_mode(&self, phase: &OnboardingPhase) -> QualityMode {
        match phase {
            OnboardingPhase::Introduction => QualityMode::Observe,
            OnboardingPhase::MonitoringSetup => QualityMode::Observe,
            OnboardingPhase::MetricsLearning => QualityMode::Advise,
            OnboardingPhase::EnforcementConfig => QualityMode::Guide,
            OnboardingPhase::AutomationSetup => QualityMode::Enforce,
            OnboardingPhase::AdvancedFeatures => QualityMode::Enforce,
            OnboardingPhase::ProductionReady => QualityMode::Enforce,
        }
    }
}

/// Onboarding progress report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingReport {
    pub team_id: TeamId,
    pub current_phase: OnboardingPhase,
    pub overall_completion: f64,
    pub phase_progress: f64,
    pub days_active: u32,
    pub engagement_score: f64,
    pub achievements: Vec<Achievement>,
    pub recommended_next_steps: Vec<Tutorial>,
    pub quality_mode: QualityMode,
}

/// Achievement earned by team
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub earned_at: std::time::SystemTime,
}

impl Default for TutorialLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl TutorialLibrary {
    /// Create new tutorial library with built-in content
    #[must_use]
    pub fn new() -> Self {
        let mut tutorials = HashMap::new();

        // Introduction phase tutorials
        tutorials.insert(OnboardingPhase::Introduction, vec![
            Tutorial {
                id: "quality_philosophy".to_string(),
                name: "Quality Philosophy and Benefits".to_string(),
                description: "Learn about the unified quality approach and its benefits".to_string(),
                duration_minutes: 15,
                prerequisites: vec![],
                objectives: vec![
                    "Understand quality-driven development".to_string(),
                    "Learn about error budgets".to_string(),
                    "See ROI of quality investment".to_string(),
                ],
                content: TutorialContent::Documentation {
                    title: "Quality Philosophy".to_string(),
                    content: "Quality is not just about catching bugs - it's about building sustainable, maintainable software that delivers business value.".to_string(),
                    examples: vec!["Case studies from high-performing teams".to_string()],
                },
                exercises: vec![],
                success_criteria: vec!["Complete reading".to_string(), "Pass knowledge check".to_string()],
            },
        ]);

        // Monitoring setup tutorials
        tutorials.insert(
            OnboardingPhase::MonitoringSetup,
            vec![Tutorial {
                id: "setup_monitoring".to_string(),
                name: "Setting Up Quality Monitoring".to_string(),
                description: "Configure real-time quality monitoring for your project".to_string(),
                duration_minutes: 30,
                prerequisites: vec!["quality_philosophy".to_string()],
                objectives: vec![
                    "Install and configure PMAT".to_string(),
                    "Set up file watching".to_string(),
                    "Understand monitoring outputs".to_string(),
                ],
                content: TutorialContent::Walkthrough {
                    steps: vec![WalkthroughStep {
                        step: 1,
                        title: "Install PMAT".to_string(),
                        instructions: "Install PMAT using cargo".to_string(),
                        commands: vec!["cargo install pmat".to_string()],
                        expected_results: "PMAT installed successfully".to_string(),
                        tips: vec!["Use --force to update existing installation".to_string()],
                    }],
                },
                exercises: vec![Exercise {
                    name: "Monitor Your Project".to_string(),
                    instructions: "Set up monitoring for your current project".to_string(),
                    starting_files: HashMap::new(),
                    expected_outcome: "Quality metrics displayed".to_string(),
                    validation: vec!["pmat analyze complexity".to_string()],
                    hints: vec!["Start with a small test file".to_string()],
                }],
                success_criteria: vec![
                    "Successfully run pmat commands".to_string(),
                    "See quality metrics".to_string(),
                ],
            }],
        );

        Self { tutorials }
    }

    /// Get tutorials for a specific phase
    #[must_use]
    pub fn get_tutorials_for_phase(&self, phase: &OnboardingPhase) -> Vec<Tutorial> {
        self.tutorials.get(phase).cloned().unwrap_or_default()
    }

    /// Count total tutorials
    #[must_use]
    pub fn count_tutorials(&self) -> u32 {
        self.tutorials.values().map(|v| v.len() as u32).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_onboarding_session_creation() {
        let preferences = TeamPreferences {
            languages: vec!["rust".to_string()],
            learning_style: LearningStyle::Practical,
            notifications: NotificationPreference {
                daily_updates: true,
                weekly_summaries: true,
                achievements: true,
                celebrations: true,
            },
            philosophy: QualityPhilosophy::default(),
            team_info: TeamInfo {
                size: 5,
                experience_level: ExperienceLevel::Intermediate,
                project_type: ProjectType::WebApplication,
                quality_maturity: QualityMaturity::Basic,
            },
        };

        let config = OnboardingConfig {
            interactive_mode: true,
            personalization: true,
            track_progress: true,
            gamification: GamificationConfig {
                achievements: true,
                badges: true,
                leaderboards: false,
                points: true,
            },
        };

        let mut onboarding = TeamOnboarding::new(config);
        let session = onboarding
            .start_onboarding("team-1".to_string(), preferences)
            .unwrap();

        assert_eq!(session.team_id, "team-1");
        assert_eq!(session.current_phase, OnboardingPhase::Introduction);
        assert_eq!(session.quality_mode, QualityMode::Observe);
        assert!(session.completed_tutorials.is_empty());
    }

    #[test]
    fn test_tutorial_library() {
        let library = TutorialLibrary::new();
        let intro_tutorials = library.get_tutorials_for_phase(&OnboardingPhase::Introduction);
        assert!(!intro_tutorials.is_empty());

        let total_count = library.count_tutorials();
        assert!(total_count > 0);
    }

    #[test]
    fn test_phase_progression() {
        let config = OnboardingConfig {
            interactive_mode: true,
            personalization: true,
            track_progress: true,
            gamification: GamificationConfig {
                achievements: true,
                badges: true,
                leaderboards: false,
                points: true,
            },
        };

        let onboarding = TeamOnboarding::new(config);

        assert_eq!(
            onboarding.next_phase(&OnboardingPhase::Introduction),
            OnboardingPhase::MonitoringSetup
        );

        assert_eq!(
            onboarding.next_phase(&OnboardingPhase::ProductionReady),
            OnboardingPhase::ProductionReady
        );
    }

    #[test]
    fn test_quality_mode_progression() {
        let config = OnboardingConfig {
            interactive_mode: true,
            personalization: true,
            track_progress: true,
            gamification: GamificationConfig {
                achievements: true,
                badges: true,
                leaderboards: false,
                points: true,
            },
        };

        let onboarding = TeamOnboarding::new(config);

        assert_eq!(
            onboarding.recommended_quality_mode(&OnboardingPhase::Introduction),
            QualityMode::Observe
        );

        assert_eq!(
            onboarding.recommended_quality_mode(&OnboardingPhase::EnforcementConfig),
            QualityMode::Guide
        );

        assert_eq!(
            onboarding.recommended_quality_mode(&OnboardingPhase::ProductionReady),
            QualityMode::Enforce
        );
    }

    #[test]
    fn test_learning_styles() {
        let styles = vec![
            LearningStyle::Practical,
            LearningStyle::Theoretical,
            LearningStyle::Balanced,
            LearningStyle::Exploratory,
        ];

        assert_eq!(styles.len(), 4);

        // Test serialization works
        let serialized = serde_json::to_string(&styles[0]).unwrap();
        assert!(serialized.contains("Practical"));
    }

    #[test]
    fn test_gamification_config() {
        let config = GamificationConfig {
            achievements: true,
            badges: false,
            leaderboards: true,
            points: false,
        };

        assert!(config.achievements);
        assert!(!config.badges);
        assert!(config.leaderboards);
        assert!(!config.points);
    }
}
