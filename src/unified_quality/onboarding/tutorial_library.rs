// TutorialLibrary implementation: Default, new (built-in content),
// get_tutorials_for_phase, count_tutorials

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
