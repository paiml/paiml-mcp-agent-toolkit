/// Non-blocking profiler using shadow stack instrumentation
pub struct AsyncProfiler {
    shadow_stacks: Arc<RwLock<Vec<ShadowStack>>>,
    #[allow(dead_code)]
    sample_interval: Duration,
}

impl Default for AsyncProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncProfiler {
    #[must_use]
    pub fn new() -> Self {
        Self {
            shadow_stacks: Arc::new(RwLock::new(Vec::new())),
            sample_interval: Duration::from_millis(10), // 10ms sampling
        }
    }

    /// Profile a WASM module
    pub async fn profile_module(&self, binary: &[u8]) -> Result<ProfilingReport> {
        debug_assert!(!binary.is_empty(), "binary must not be empty");
        // Parse and analyze the module
        let instruction_mix = self.analyze_instruction_mix(binary)?;
        let hot_functions = self.identify_hot_functions(binary)?;
        let memory_usage = self.analyze_memory_usage(binary)?;

        Ok(ProfilingReport {
            instruction_mix,
            hot_functions,
            memory_usage,
        })
    }

    /// Analyze instruction mix in the module
    fn analyze_instruction_mix(&self, binary: &[u8]) -> Result<InstructionMix> {
        debug_assert!(!binary.is_empty(), "binary must not be empty");
        let mut total = 0;
        let mut control_flow = 0;
        let mut memory_ops = 0;
        let mut arithmetic = 0;
        let mut calls = 0;

        for payload in wasmparser::Parser::new(0).parse_all(binary) {
            let payload = payload?;

            if let Payload::CodeSectionEntry(body) = payload {
                let reader = body.get_operators_reader()?;

                for op in reader {
                    let operator = op?;
                    total += 1;

                    match categorize_for_profiling(&operator) {
                        InstructionCategory::ControlFlow => control_flow += 1,
                        InstructionCategory::Memory => memory_ops += 1,
                        InstructionCategory::Arithmetic => arithmetic += 1,
                        InstructionCategory::Call => calls += 1,
                        InstructionCategory::Other => {}
                    }
                }
            }
        }

        Ok(InstructionMix {
            total_instructions: total,
            control_flow,
            memory_ops,
            arithmetic,
            calls,
        })
    }

    /// Identify hot functions through static analysis
    fn identify_hot_functions(&self, binary: &[u8]) -> Result<Vec<HotFunction>> {
        debug_assert!(!binary.is_empty(), "binary must not be empty");
        let mut functions = Vec::new();
        let mut function_sizes = Vec::new();
        let mut total_size = 0;

        for payload in wasmparser::Parser::new(0).parse_all(binary) {
            let payload = payload?;

            if let Payload::CodeSectionEntry(body) = payload {
                let size = body.range().len();
                function_sizes.push(size);
                total_size += size;
            }
        }

        // Identify functions by relative size (heuristic for hot functions)
        for (idx, &size) in function_sizes.iter().enumerate() {
            if size > 0 {
                let percentage = (size as f64 / total_size as f64) * 100.0;

                // Consider functions > 5% of code as potentially hot
                if percentage > 5.0 {
                    functions.push(HotFunction {
                        name: format!("func_{idx}"),
                        samples: size, // Using size as proxy for samples
                        percentage,
                    });
                }
            }
        }

        // Sort by percentage descending
        functions.sort_by(|a, b| {
            b.percentage
                .partial_cmp(&a.percentage)
                .expect("internal error")
        });

        Ok(functions)
    }

    /// Analyze memory usage patterns
    fn analyze_memory_usage(&self, binary: &[u8]) -> Result<MemoryProfile> {
        debug_assert!(!binary.is_empty(), "binary must not be empty");
        let mut initial_pages = 1;
        let mut max_pages = None;
        let mut has_memory = false;

        for payload in wasmparser::Parser::new(0).parse_all(binary) {
            let payload = payload?;

            if let Payload::MemorySection(reader) = payload {
                if let Some(memory) = reader.into_iter().next() {
                    let memory = memory?;
                    has_memory = true;
                    initial_pages = memory.initial as u32;
                    max_pages = memory.maximum.map(|m| m as u32);
                }
            }
        }

        // If no memory section, assume default
        if !has_memory {
            initial_pages = 1;
            max_pages = Some(256); // Default max
        }

        Ok(MemoryProfile {
            initial_pages,
            max_pages,
            growth_events: Vec::new(), // Would be populated during runtime
        })
    }

    /// Start asynchronous sampling of a running instance
    #[must_use]
    pub fn start_sampling(&self, sample_interval: Duration) -> JoinHandle<()> {
        let shadow_stacks = self.shadow_stacks.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(sample_interval);
            let mut sample_count = 0;

            loop {
                interval.tick().await;

                // Simulate shadow stack sampling
                let stack = ShadowStack::sample();

                if let Ok(mut stacks) = shadow_stacks.write() {
                    stacks.push(stack);
                    sample_count += 1;

                    if sample_count >= 10000 {
                        break; // Sample limit reached
                    }
                }
            }
        })
    }
}
