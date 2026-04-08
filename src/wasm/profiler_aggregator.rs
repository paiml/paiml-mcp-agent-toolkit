/// Profile aggregator for multiple runs
pub struct ProfileAggregator {
    profiles: Vec<ProfilingReport>,
}

impl Default for ProfileAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileAggregator {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Add profile.
    pub fn add_profile(&mut self, profile: ProfilingReport) {
        self.profiles.push(profile);
    }

    /// Get average instruction mix across profiles
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn average_instruction_mix(&self) -> InstructionMix {
        if self.profiles.is_empty() {
            return InstructionMix {
                total_instructions: 0,
                control_flow: 0,
                memory_ops: 0,
                arithmetic: 0,
                calls: 0,
            };
        }

        let count = self.profiles.len();
        let total: usize = self
            .profiles
            .iter()
            .map(|p| p.instruction_mix.total_instructions)
            .sum();
        let control: usize = self
            .profiles
            .iter()
            .map(|p| p.instruction_mix.control_flow)
            .sum();
        let memory: usize = self
            .profiles
            .iter()
            .map(|p| p.instruction_mix.memory_ops)
            .sum();
        let arith: usize = self
            .profiles
            .iter()
            .map(|p| p.instruction_mix.arithmetic)
            .sum();
        let calls: usize = self.profiles.iter().map(|p| p.instruction_mix.calls).sum();

        InstructionMix {
            total_instructions: total / count,
            control_flow: control / count,
            memory_ops: memory / count,
            arithmetic: arith / count,
            calls: calls / count,
        }
    }
}
