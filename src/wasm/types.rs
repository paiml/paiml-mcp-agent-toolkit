// Types for WebAssembly profiling reports

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingReport {
    pub instruction_mix: InstructionMix,
    pub hot_functions: Vec<HotFunction>,
    pub memory_usage: MemoryProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionMix {
    pub total_instructions: usize,
    pub control_flow: usize,
    pub memory_ops: usize,
    pub arithmetic: usize,
    pub calls: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotFunction {
    pub name: String,
    pub samples: usize,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryProfile {
    pub initial_pages: u32,
    pub max_pages: Option<u32>,
    pub growth_events: Vec<GrowthEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthEvent {
    pub timestamp: u64,
    pub pages_before: u32,
    pub pages_after: u32,
}
