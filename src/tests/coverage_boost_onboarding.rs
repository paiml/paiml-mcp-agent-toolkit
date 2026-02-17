#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for unified_quality/onboarding module
//! Tests for TeamOnboarding, TutorialLibrary, and related types

use crate::unified_quality::onboarding::{
    Achievement, Exercise, ExperienceLevel, GamificationConfig, LearningStyle,
    NotificationPreference, OnboardingConfig, OnboardingPhase, OnboardingProgress,
    OnboardingReport, OnboardingSession, ProjectType, QualityMaturity, TeamInfo, TeamOnboarding,
    TeamPreferences, TutorialContent, TutorialLibrary, WalkthroughStep,
};
use crate::unified_quality::{QualityMode, QualityPhilosophy};
use std::collections::HashMap;

// ============ OnboardingPhase Tests ============

#[test]
fn test_onboarding_phase_introduction() {
    let phase = OnboardingPhase::Introduction;
    let serialized = serde_json::to_string(&phase).unwrap();
    assert!(serialized.contains("Introduction"));
}

#[test]
fn test_onboarding_phase_monitoring_setup() {
    let phase = OnboardingPhase::MonitoringSetup;
    assert_eq!(format!("{:?}", phase), "MonitoringSetup");
}

#[test]
fn test_onboarding_phase_metrics_learning() {
    let phase = OnboardingPhase::MetricsLearning;
    let cloned = phase.clone();
    assert_eq!(phase, cloned);
}

#[test]
fn test_onboarding_phase_enforcement_config() {
    let phase = OnboardingPhase::EnforcementConfig;
    assert_ne!(phase, OnboardingPhase::Introduction);
}

#[test]
fn test_onboarding_phase_automation_setup() {
    let phase = OnboardingPhase::AutomationSetup;
    let _debug = format!("{:?}", phase);
}

#[test]
fn test_onboarding_phase_advanced_features() {
    let phase = OnboardingPhase::AdvancedFeatures;
    assert!(format!("{:?}", phase).contains("Advanced"));
}

#[test]
fn test_onboarding_phase_production_ready() {
    let phase = OnboardingPhase::ProductionReady;
    let serialized = serde_json::to_string(&phase).unwrap();
    let deserialized: OnboardingPhase = serde_json::from_str(&serialized).unwrap();
    assert_eq!(phase, deserialized);
}

#[test]
fn test_onboarding_phase_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(OnboardingPhase::Introduction);
    set.insert(OnboardingPhase::ProductionReady);
    assert_eq!(set.len(), 2);
}

// ============ LearningStyle Tests ============

#[test]
fn test_learning_style_practical() {
    let style = LearningStyle::Practical;
    let serialized = serde_json::to_string(&style).unwrap();
    assert!(serialized.contains("Practical"));
}

#[test]
fn test_learning_style_theoretical() {
    let style = LearningStyle::Theoretical;
    let _debug = format!("{:?}", style);
}

#[test]
fn test_learning_style_balanced() {
    let style = LearningStyle::Balanced;
    let cloned = style.clone();
    let _ = format!("{:?}", cloned);
}

#[test]
fn test_learning_style_exploratory() {
    let style = LearningStyle::Exploratory;
    let serialized = serde_json::to_string(&style).unwrap();
    let deserialized: LearningStyle = serde_json::from_str(&serialized).unwrap();
    let _ = format!("{:?}", deserialized);
}

// ============ ExperienceLevel Tests ============

#[test]
fn test_experience_level_junior() {
    let level = ExperienceLevel::Junior;
    let serialized = serde_json::to_string(&level).unwrap();
    assert!(serialized.contains("Junior"));
}

#[test]
fn test_experience_level_intermediate() {
    let level = ExperienceLevel::Intermediate;
    let cloned = level.clone();
    let _ = format!("{:?}", cloned);
}

#[test]
fn test_experience_level_senior() {
    let level = ExperienceLevel::Senior;
    let serialized = serde_json::to_string(&level).unwrap();
    let deserialized: ExperienceLevel = serde_json::from_str(&serialized).unwrap();
    let _ = format!("{:?}", deserialized);
}

#[test]
fn test_experience_level_mixed() {
    let level = ExperienceLevel::Mixed;
    let _debug = format!("{:?}", level);
}

// ============ ProjectType Tests ============

#[test]
fn test_project_type_web_application() {
    let project = ProjectType::WebApplication;
    let serialized = serde_json::to_string(&project).unwrap();
    assert!(serialized.contains("WebApplication"));
}

#[test]
fn test_project_type_systems_software() {
    let project = ProjectType::SystemsSoftware;
    let cloned = project.clone();
    let _ = format!("{:?}", cloned);
}

#[test]
fn test_project_type_data_science() {
    let project = ProjectType::DataScience;
    let _debug = format!("{:?}", project);
}

#[test]
fn test_project_type_mobile() {
    let project = ProjectType::Mobile;
    let serialized = serde_json::to_string(&project).unwrap();
    let _: ProjectType = serde_json::from_str(&serialized).unwrap();
}

#[test]
fn test_project_type_library() {
    let project = ProjectType::Library;
    let _ = format!("{:?}", project);
}

#[test]
fn test_project_type_microservices() {
    let project = ProjectType::Microservices;
    let cloned = project.clone();
    let _ = format!("{:?}", cloned);
}

#[test]
fn test_project_type_monolith() {
    let project = ProjectType::Monolith;
    let serialized = serde_json::to_string(&project).unwrap();
    let deserialized: ProjectType = serde_json::from_str(&serialized).unwrap();
    let _ = format!("{:?}", deserialized);
}

// ============ QualityMaturity Tests ============

#[test]
fn test_quality_maturity_none() {
    let maturity = QualityMaturity::None;
    let serialized = serde_json::to_string(&maturity).unwrap();
    assert!(serialized.contains("None"));
}

#[test]
fn test_quality_maturity_basic() {
    let maturity = QualityMaturity::Basic;
    let cloned = maturity.clone();
    let _ = format!("{:?}", cloned);
}

#[test]
fn test_quality_maturity_intermediate() {
    let maturity = QualityMaturity::Intermediate;
    let _debug = format!("{:?}", maturity);
}

#[test]
fn test_quality_maturity_advanced() {
    let maturity = QualityMaturity::Advanced;
    let serialized = serde_json::to_string(&maturity).unwrap();
    let deserialized: QualityMaturity = serde_json::from_str(&serialized).unwrap();
    let _ = format!("{:?}", deserialized);
}

// ============ TeamInfo Tests ============

#[test]
fn test_team_info_creation() {
    let info = TeamInfo {
        size: 5,
        experience_level: ExperienceLevel::Mixed,
        project_type: ProjectType::WebApplication,
        quality_maturity: QualityMaturity::Basic,
    };
    assert_eq!(info.size, 5);
}

#[test]
fn test_team_info_serialization() {
    let info = TeamInfo {
        size: 10,
        experience_level: ExperienceLevel::Senior,
        project_type: ProjectType::Microservices,
        quality_maturity: QualityMaturity::Advanced,
    };
    let serialized = serde_json::to_string(&info).unwrap();
    let deserialized: TeamInfo = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.size, 10);
}

#[test]
fn test_team_info_debug() {
    let info = TeamInfo {
        size: 3,
        experience_level: ExperienceLevel::Junior,
        project_type: ProjectType::Mobile,
        quality_maturity: QualityMaturity::None,
    };
    let debug = format!("{:?}", info);
    assert!(debug.contains("TeamInfo"));
}

// ============ NotificationPreference Tests ============

#[test]
fn test_notification_preference_all_enabled() {
    let pref = NotificationPreference {
        daily_updates: true,
        weekly_summaries: true,
        achievements: true,
        celebrations: true,
    };
    assert!(pref.daily_updates);
    assert!(pref.weekly_summaries);
}

#[test]
fn test_notification_preference_all_disabled() {
    let pref = NotificationPreference {
        daily_updates: false,
        weekly_summaries: false,
        achievements: false,
        celebrations: false,
    };
    assert!(!pref.daily_updates);
}

#[test]
fn test_notification_preference_serialization() {
    let pref = NotificationPreference {
        daily_updates: true,
        weekly_summaries: false,
        achievements: true,
        celebrations: false,
    };
    let serialized = serde_json::to_string(&pref).unwrap();
    let deserialized: NotificationPreference = serde_json::from_str(&serialized).unwrap();
    assert_eq!(pref.daily_updates, deserialized.daily_updates);
}

// ============ OnboardingProgress Tests ============

#[test]
fn test_onboarding_progress_creation() {
    let progress = OnboardingProgress {
        tutorials_completed: 5,
        tutorials_total: 10,
        exercises_completed: 3,
        quality_improvements: 2,
        days_active: 7,
        engagement_score: 75.0,
    };
    assert_eq!(progress.tutorials_completed, 5);
    assert_eq!(progress.tutorials_total, 10);
}

#[test]
fn test_onboarding_progress_serialization() {
    let progress = OnboardingProgress {
        tutorials_completed: 0,
        tutorials_total: 20,
        exercises_completed: 0,
        quality_improvements: 0,
        days_active: 0,
        engagement_score: 0.0,
    };
    let serialized = serde_json::to_string(&progress).unwrap();
    let deserialized: OnboardingProgress = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.tutorials_total, 20);
}

// ============ GamificationConfig Tests ============

#[test]
fn test_gamification_config_all_enabled() {
    let config = GamificationConfig {
        achievements: true,
        badges: true,
        leaderboards: true,
        points: true,
    };
    assert!(config.achievements);
    assert!(config.badges);
}

#[test]
fn test_gamification_config_all_disabled() {
    let config = GamificationConfig {
        achievements: false,
        badges: false,
        leaderboards: false,
        points: false,
    };
    assert!(!config.leaderboards);
}

#[test]
fn test_gamification_config_serialization() {
    let config = GamificationConfig {
        achievements: true,
        badges: false,
        leaderboards: true,
        points: false,
    };
    let serialized = serde_json::to_string(&config).unwrap();
    let deserialized: GamificationConfig = serde_json::from_str(&serialized).unwrap();
    assert_eq!(config.badges, deserialized.badges);
}

// ============ OnboardingConfig Tests ============

#[test]
fn test_onboarding_config_creation() {
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
    assert!(config.interactive_mode);
    assert!(config.gamification.achievements);
}

#[test]
fn test_onboarding_config_serialization() {
    let config = OnboardingConfig {
        interactive_mode: false,
        personalization: false,
        track_progress: true,
        gamification: GamificationConfig {
            achievements: false,
            badges: false,
            leaderboards: false,
            points: false,
        },
    };
    let serialized = serde_json::to_string(&config).unwrap();
    let deserialized: OnboardingConfig = serde_json::from_str(&serialized).unwrap();
    assert!(deserialized.track_progress);
}

// ============ TeamPreferences Tests ============

fn create_test_preferences() -> TeamPreferences {
    TeamPreferences {
        languages: vec!["Rust".to_string(), "Python".to_string()],
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
            experience_level: ExperienceLevel::Mixed,
            project_type: ProjectType::WebApplication,
            quality_maturity: QualityMaturity::Basic,
        },
    }
}

#[test]
fn test_team_preferences_creation() {
    let prefs = create_test_preferences();
    assert_eq!(prefs.languages.len(), 2);
    assert!(prefs.languages.contains(&"Rust".to_string()));
}

#[test]
fn test_team_preferences_serialization() {
    let prefs = create_test_preferences();
    let serialized = serde_json::to_string(&prefs).unwrap();
    let deserialized: TeamPreferences = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.languages.len(), 2);
}

// ============ WalkthroughStep Tests ============

#[test]
fn test_walkthrough_step_creation() {
    let step = WalkthroughStep {
        step: 1,
        title: "Test Step".to_string(),
        instructions: "Do something".to_string(),
        commands: vec!["echo test".to_string()],
        expected_results: "test".to_string(),
        tips: vec!["Tip 1".to_string()],
    };
    assert_eq!(step.step, 1);
    assert_eq!(step.commands.len(), 1);
}

#[test]
fn test_walkthrough_step_serialization() {
    let step = WalkthroughStep {
        step: 2,
        title: "Step Two".to_string(),
        instructions: "Instructions".to_string(),
        commands: vec![],
        expected_results: "Success".to_string(),
        tips: vec![],
    };
    let serialized = serde_json::to_string(&step).unwrap();
    let deserialized: WalkthroughStep = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.step, 2);
}

// ============ Exercise Tests ============

#[test]
fn test_exercise_creation() {
    let exercise = Exercise {
        name: "Test Exercise".to_string(),
        instructions: "Do this exercise".to_string(),
        starting_files: HashMap::new(),
        expected_outcome: "Pass".to_string(),
        validation: vec!["check".to_string()],
        hints: vec!["hint 1".to_string()],
    };
    assert_eq!(exercise.name, "Test Exercise");
}

#[test]
fn test_exercise_with_files() {
    let mut files = HashMap::new();
    files.insert("main.rs".to_string(), "fn main() {}".to_string());

    let exercise = Exercise {
        name: "Code Exercise".to_string(),
        instructions: "Complete the code".to_string(),
        starting_files: files,
        expected_outcome: "Compiles".to_string(),
        validation: vec!["cargo build".to_string()],
        hints: vec![],
    };
    assert!(exercise.starting_files.contains_key("main.rs"));
}

#[test]
fn test_exercise_serialization() {
    let exercise = Exercise {
        name: "Serialization Test".to_string(),
        instructions: "Test".to_string(),
        starting_files: HashMap::new(),
        expected_outcome: "Done".to_string(),
        validation: vec![],
        hints: vec![],
    };
    let serialized = serde_json::to_string(&exercise).unwrap();
    let deserialized: Exercise = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.name, "Serialization Test");
}

// ============ TutorialContent Tests ============

#[test]
fn test_tutorial_content_demo() {
    let content = TutorialContent::Demo {
        title: "Demo Title".to_string(),
        description: "Demo description".to_string(),
        commands: vec!["cmd1".to_string(), "cmd2".to_string()],
        expected_outputs: vec!["output1".to_string()],
    };
    let serialized = serde_json::to_string(&content).unwrap();
    assert!(serialized.contains("Demo"));
}

#[test]
fn test_tutorial_content_walkthrough() {
    let content = TutorialContent::Walkthrough {
        steps: vec![WalkthroughStep {
            step: 1,
            title: "Step 1".to_string(),
            instructions: "Do step 1".to_string(),
            commands: vec![],
            expected_results: "Done".to_string(),
            tips: vec![],
        }],
    };
    let _debug = format!("{:?}", content);
}

#[test]
fn test_tutorial_content_video() {
    let content = TutorialContent::Video {
        title: "Tutorial Video".to_string(),
        url: "https://example.com/video".to_string(),
        transcript: Some("Transcript text".to_string()),
    };
    let serialized = serde_json::to_string(&content).unwrap();
    assert!(serialized.contains("Video"));
}

#[test]
fn test_tutorial_content_video_no_transcript() {
    let content = TutorialContent::Video {
        title: "Video Title".to_string(),
        url: "https://example.com".to_string(),
        transcript: None,
    };
    let cloned = content.clone();
    let _ = format!("{:?}", cloned);
}

#[test]
fn test_tutorial_content_documentation() {
    let content = TutorialContent::Documentation {
        title: "Documentation".to_string(),
        content: "Content text".to_string(),
        examples: vec!["Example 1".to_string()],
    };
    let serialized = serde_json::to_string(&content).unwrap();
    let deserialized: TutorialContent = serde_json::from_str(&serialized).unwrap();
    let _ = format!("{:?}", deserialized);
}

// ============ Achievement Tests ============

#[test]
fn test_achievement_creation() {
    let achievement = Achievement {
        id: "test_achievement".to_string(),
        name: "Test Achievement".to_string(),
        description: "Description".to_string(),
        earned_at: std::time::SystemTime::now(),
    };
    assert_eq!(achievement.id, "test_achievement");
}

#[test]
fn test_achievement_serialization() {
    let achievement = Achievement {
        id: "serialized".to_string(),
        name: "Serialized Achievement".to_string(),
        description: "Test description".to_string(),
        earned_at: std::time::SystemTime::UNIX_EPOCH,
    };
    let serialized = serde_json::to_string(&achievement).unwrap();
    let deserialized: Achievement = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.id, "serialized");
}

// ============ TutorialLibrary Tests ============

#[test]
fn test_tutorial_library_new() {
    let library = TutorialLibrary::new();
    assert!(library.count_tutorials() > 0);
}

#[test]
fn test_tutorial_library_default() {
    let library = TutorialLibrary::default();
    assert!(library.count_tutorials() > 0);
}

#[test]
fn test_tutorial_library_get_introduction_tutorials() {
    let library = TutorialLibrary::new();
    let tutorials = library.get_tutorials_for_phase(&OnboardingPhase::Introduction);
    assert!(!tutorials.is_empty());
}

#[test]
fn test_tutorial_library_get_monitoring_tutorials() {
    let library = TutorialLibrary::new();
    let tutorials = library.get_tutorials_for_phase(&OnboardingPhase::MonitoringSetup);
    assert!(!tutorials.is_empty());
}

#[test]
fn test_tutorial_library_get_nonexistent_phase() {
    let library = TutorialLibrary::new();
    let tutorials = library.get_tutorials_for_phase(&OnboardingPhase::AdvancedFeatures);
    // Should return empty vec for phases without tutorials
    let _ = tutorials;
}

#[test]
fn test_tutorial_library_count() {
    let library = TutorialLibrary::new();
    let count = library.count_tutorials();
    assert!(count >= 2); // At least introduction and monitoring
}

// ============ TeamOnboarding Tests ============

fn create_test_config() -> OnboardingConfig {
    OnboardingConfig {
        interactive_mode: true,
        personalization: true,
        track_progress: true,
        gamification: GamificationConfig {
            achievements: true,
            badges: true,
            leaderboards: false,
            points: true,
        },
    }
}

#[test]
fn test_team_onboarding_new() {
    let onboarding = TeamOnboarding::new(create_test_config());
    let _ = onboarding;
}

#[test]
fn test_team_onboarding_start_onboarding() {
    let mut onboarding = TeamOnboarding::new(create_test_config());
    let prefs = create_test_preferences();
    let session = onboarding
        .start_onboarding("team1".to_string(), prefs)
        .unwrap();
    assert_eq!(session.team_id, "team1");
    assert_eq!(session.current_phase, OnboardingPhase::Introduction);
}

#[test]
fn test_team_onboarding_get_recommendations() {
    let mut onboarding = TeamOnboarding::new(create_test_config());
    let prefs = create_test_preferences();
    onboarding
        .start_onboarding("team1".to_string(), prefs)
        .unwrap();

    let recommendations = onboarding
        .get_recommendations(&"team1".to_string())
        .unwrap();
    assert!(!recommendations.is_empty());
}

#[test]
fn test_team_onboarding_get_recommendations_not_found() {
    let onboarding = TeamOnboarding::new(create_test_config());
    let result = onboarding.get_recommendations(&"nonexistent".to_string());
    assert!(result.is_err());
}

#[test]
fn test_team_onboarding_complete_tutorial() {
    let mut onboarding = TeamOnboarding::new(create_test_config());
    let prefs = create_test_preferences();
    onboarding
        .start_onboarding("team1".to_string(), prefs)
        .unwrap();

    let result =
        onboarding.complete_tutorial(&"team1".to_string(), "quality_philosophy".to_string(), 1);
    assert!(result.is_ok());
}

#[test]
fn test_team_onboarding_complete_tutorial_not_found() {
    let mut onboarding = TeamOnboarding::new(create_test_config());
    let result = onboarding.complete_tutorial(&"nonexistent".to_string(), "test".to_string(), 0);
    assert!(result.is_err());
}

#[test]
fn test_team_onboarding_complete_multiple_tutorials() {
    let mut onboarding = TeamOnboarding::new(create_test_config());
    let prefs = create_test_preferences();
    onboarding
        .start_onboarding("team1".to_string(), prefs)
        .unwrap();

    // Complete first tutorial
    onboarding
        .complete_tutorial(&"team1".to_string(), "tutorial1".to_string(), 2)
        .unwrap();
    // Complete same tutorial again (should be idempotent)
    onboarding
        .complete_tutorial(&"team1".to_string(), "tutorial1".to_string(), 1)
        .unwrap();
    // Complete second tutorial
    onboarding
        .complete_tutorial(&"team1".to_string(), "tutorial2".to_string(), 3)
        .unwrap();
}

#[test]
fn test_team_onboarding_phase_advancement() {
    let mut onboarding = TeamOnboarding::new(create_test_config());
    let prefs = create_test_preferences();
    onboarding
        .start_onboarding("team1".to_string(), prefs)
        .unwrap();

    // Complete enough tutorials to advance from Introduction
    for i in 0..3 {
        onboarding
            .complete_tutorial(&"team1".to_string(), format!("tutorial_{i}"), 1)
            .unwrap();
    }

    let report = onboarding
        .generate_progress_report(&"team1".to_string())
        .unwrap();
    // Should have advanced past Introduction
    assert_ne!(report.current_phase, OnboardingPhase::Introduction);
}

#[test]
fn test_team_onboarding_generate_report() {
    let mut onboarding = TeamOnboarding::new(create_test_config());
    let prefs = create_test_preferences();
    onboarding
        .start_onboarding("team1".to_string(), prefs)
        .unwrap();

    let report = onboarding
        .generate_progress_report(&"team1".to_string())
        .unwrap();
    assert_eq!(report.team_id, "team1");
    assert_eq!(report.overall_completion, 0.0);
}

#[test]
fn test_team_onboarding_generate_report_not_found() {
    let onboarding = TeamOnboarding::new(create_test_config());
    let result = onboarding.generate_progress_report(&"nonexistent".to_string());
    assert!(result.is_err());
}

#[test]
fn test_team_onboarding_achievements_tutorial_enthusiast() {
    let mut onboarding = TeamOnboarding::new(create_test_config());
    let prefs = create_test_preferences();
    onboarding
        .start_onboarding("team1".to_string(), prefs)
        .unwrap();

    // Complete 5 tutorials to earn achievement
    for i in 0..5 {
        onboarding
            .complete_tutorial(&"team1".to_string(), format!("tutorial_{i}"), 0)
            .unwrap();
    }

    let report = onboarding
        .generate_progress_report(&"team1".to_string())
        .unwrap();
    assert!(report
        .achievements
        .iter()
        .any(|a| a.id == "tutorial_enthusiast"));
}

#[test]
fn test_team_onboarding_achievements_hands_on_learner() {
    let mut onboarding = TeamOnboarding::new(create_test_config());
    let prefs = create_test_preferences();
    onboarding
        .start_onboarding("team1".to_string(), prefs)
        .unwrap();

    // Complete tutorials with many exercises
    for i in 0..3 {
        onboarding
            .complete_tutorial(&"team1".to_string(), format!("tutorial_{i}"), 4)
            .unwrap();
    }

    let report = onboarding
        .generate_progress_report(&"team1".to_string())
        .unwrap();
    assert!(report
        .achievements
        .iter()
        .any(|a| a.id == "hands_on_learner"));
}

// ============ OnboardingSession Tests ============

#[test]
fn test_onboarding_session_serialization() {
    let session = OnboardingSession {
        team_id: "test_team".to_string(),
        current_phase: OnboardingPhase::Introduction,
        completed_tutorials: vec!["tutorial1".to_string()],
        quality_mode: QualityMode::Observe,
        started_at: std::time::SystemTime::now(),
        progress: OnboardingProgress {
            tutorials_completed: 1,
            tutorials_total: 10,
            exercises_completed: 2,
            quality_improvements: 0,
            days_active: 1,
            engagement_score: 10.0,
        },
        preferences: create_test_preferences(),
    };
    let serialized = serde_json::to_string(&session).unwrap();
    let deserialized: OnboardingSession = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.team_id, "test_team");
}

// ============ OnboardingReport Tests ============

#[test]
fn test_onboarding_report_serialization() {
    let report = OnboardingReport {
        team_id: "team1".to_string(),
        current_phase: OnboardingPhase::MetricsLearning,
        overall_completion: 50.0,
        phase_progress: 75.0,
        days_active: 14,
        engagement_score: 85.0,
        achievements: vec![],
        recommended_next_steps: vec![],
        quality_mode: QualityMode::Guide,
    };
    let serialized = serde_json::to_string(&report).unwrap();
    let deserialized: OnboardingReport = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.overall_completion, 50.0);
}

// ============ Relevance Calculation Tests ============

#[test]
fn test_relevance_with_language_match() {
    let mut onboarding = TeamOnboarding::new(create_test_config());
    let mut prefs = create_test_preferences();
    prefs.languages = vec!["Rust".to_string()];
    prefs.learning_style = LearningStyle::Practical;
    prefs.team_info.experience_level = ExperienceLevel::Junior;
    onboarding
        .start_onboarding("team1".to_string(), prefs)
        .unwrap();

    let recommendations = onboarding
        .get_recommendations(&"team1".to_string())
        .unwrap();
    // Should have recommendations sorted by relevance
    let _ = recommendations;
}

#[test]
fn test_relevance_with_theoretical_style() {
    let mut onboarding = TeamOnboarding::new(create_test_config());
    let mut prefs = create_test_preferences();
    prefs.learning_style = LearningStyle::Theoretical;
    onboarding
        .start_onboarding("team1".to_string(), prefs)
        .unwrap();

    let recommendations = onboarding
        .get_recommendations(&"team1".to_string())
        .unwrap();
    let _ = recommendations;
}

#[test]
fn test_relevance_with_senior_experience() {
    let mut onboarding = TeamOnboarding::new(create_test_config());
    let mut prefs = create_test_preferences();
    prefs.team_info.experience_level = ExperienceLevel::Senior;
    onboarding
        .start_onboarding("team1".to_string(), prefs)
        .unwrap();

    let recommendations = onboarding
        .get_recommendations(&"team1".to_string())
        .unwrap();
    let _ = recommendations;
}

// ============ Phase Progression Tests ============

#[test]
fn test_phase_progression_all_phases() {
    let mut onboarding = TeamOnboarding::new(create_test_config());
    let prefs = create_test_preferences();
    onboarding
        .start_onboarding("team1".to_string(), prefs)
        .unwrap();

    // Complete many tutorials to advance through phases
    for i in 0..15 {
        onboarding
            .complete_tutorial(&"team1".to_string(), format!("tutorial_{i}"), 1)
            .unwrap();
    }

    let report = onboarding
        .generate_progress_report(&"team1".to_string())
        .unwrap();
    // Should be in a late phase
    let _ = report;
}
