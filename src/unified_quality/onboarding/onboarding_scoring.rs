// TeamOnboarding private helper methods: relevance scoring, engagement scoring,
// phase progress calculation, achievement calculation, phase progression

impl TeamOnboarding {
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
    
    fn next_phase(&self, current: &OnboardingPhase) -> OnboardingPhase {
        match current {
            OnboardingPhase::Introduction => OnboardingPhase::MonitoringSetup,
            OnboardingPhase::MonitoringSetup => OnboardingPhase::MetricsLearning,
            OnboardingPhase::MetricsLearning => OnboardingPhase::EnforcementConfig,
            OnboardingPhase::EnforcementConfig => OnboardingPhase::AutomationSetup,
            OnboardingPhase::AutomationSetup => OnboardingPhase::AdvancedFeatures,
            OnboardingPhase::AdvancedFeatures => OnboardingPhase::ProductionReady,
            OnboardingPhase::ProductionReady => OnboardingPhase::ProductionReady,
        }
    }

    /// Get recommended quality mode for phase
    
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
