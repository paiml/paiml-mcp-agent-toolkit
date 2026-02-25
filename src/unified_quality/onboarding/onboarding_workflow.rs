// TeamOnboarding public workflow methods: start_onboarding, get_recommendations,
// complete_tutorial, generate_progress_report

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
                    OnboardingPhase::ProductionReady => OnboardingPhase::ProductionReady,
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
}
