// Adoption playbook types and defaults
// Included from config.rs - no `use` imports or `#!` inner attributes

/// Team adoption playbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdoptionPlaybook {
    /// Current week in adoption process
    pub current_week: u32,

    /// Adoption phases
    pub phases: Vec<AdoptionPhase>,
}

/// An adoption phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdoptionPhase {
    /// Week range (e.g., 1-2)
    pub weeks: (u32, u32),

    /// Phase name
    pub name: String,

    /// Activities in this phase
    pub activities: Vec<String>,

    /// Success criteria
    pub success_criteria: Vec<String>,
}

impl Default for AdoptionPlaybook {
    fn default() -> Self {
        Self {
            current_week: 1,
            phases: vec![
                AdoptionPhase {
                    weeks: (1, 2),
                    name: "Baseline Establishment".to_string(),
                    activities: vec![
                        "Install PMAT in observe mode".to_string(),
                        "Gather baseline metrics".to_string(),
                        "No enforcement, just visibility".to_string(),
                    ],
                    success_criteria: vec![
                        "PMAT installed and running".to_string(),
                        "Baseline metrics collected".to_string(),
                    ],
                },
                AdoptionPhase {
                    weeks: (3, 4),
                    name: "Team Education".to_string(),
                    activities: vec![
                        "Review complexity hotspots together".to_string(),
                        "Discuss SATD findings".to_string(),
                        "Set initial quality goals".to_string(),
                    ],
                    success_criteria: vec![
                        "Team understands metrics".to_string(),
                        "Quality goals agreed upon".to_string(),
                    ],
                },
                AdoptionPhase {
                    weeks: (5, 8),
                    name: "Assisted Improvement".to_string(),
                    activities: vec![
                        "Enable suggestions".to_string(),
                        "Track suggestion success rate".to_string(),
                        "Celebrate improvements".to_string(),
                    ],
                    success_criteria: vec![
                        "60% suggestion acceptance rate".to_string(),
                        "Measurable quality improvement".to_string(),
                    ],
                },
                AdoptionPhase {
                    weeks: (9, 12),
                    name: "Gradual Enforcement".to_string(),
                    activities: vec![
                        "Enable warnings on PR".to_string(),
                        "Set generous error budgets".to_string(),
                        "Allow overrides with justification".to_string(),
                    ],
                    success_criteria: vec![
                        "80% PRs pass quality gates".to_string(),
                        "Error budget not exceeded".to_string(),
                    ],
                },
            ],
        }
    }
}
