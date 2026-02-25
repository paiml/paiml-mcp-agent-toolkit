// Supporting type definitions: Tutorial, TutorialContent, WalkthroughStep, Exercise,
// OnboardingConfig, GamificationConfig, OnboardingReport, Achievement

/// Individual tutorial
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tutorial {
    pub id: String,
    pub name: String,
    pub description: String,
    pub duration_minutes: u32,
    pub prerequisites: Vec<String>,
    pub objectives: Vec<String>,
    pub content: TutorialContent,
    pub exercises: Vec<Exercise>,
    pub success_criteria: Vec<String>,
}

/// Tutorial content types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TutorialContent {
    Demo { title: String, description: String, commands: Vec<String>, expected_outputs: Vec<String> },
    Walkthrough { steps: Vec<WalkthroughStep> },
    Video { title: String, url: String, transcript: Option<String> },
    Documentation { title: String, content: String, examples: Vec<String> },
}

/// Walkthrough step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkthroughStep {
    pub step: u32,
    pub title: String,
    pub instructions: String,
    pub commands: Vec<String>,
    pub expected_results: String,
    pub tips: Vec<String>,
}

/// Hands-on exercise
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exercise {
    pub name: String,
    pub instructions: String,
    pub starting_files: HashMap<String, String>,
    pub expected_outcome: String,
    pub validation: Vec<String>,
    pub hints: Vec<String>,
}

/// Onboarding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingConfig {
    pub interactive_mode: bool,
    pub personalization: bool,
    pub track_progress: bool,
    pub gamification: GamificationConfig,
}

/// Gamification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamificationConfig {
    pub achievements: bool,
    pub badges: bool,
    pub leaderboards: bool,
    pub points: bool,
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
