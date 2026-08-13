/// Non-blocking profiler using shadow stack instrumentation
pub struct AsyncProfiler {
    shadow_stacks: Arc<RwLock<Vec<ShadowStack>>>,
    
    sample_interval: Duration,
}

impl Default for AsyncProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncProfiler {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            shadow_stacks: Arc::new(RwLock::new(Vec::new())),
            sample_interval: Duration::from_millis(10), // 10ms sampling
        }
    }

    /// Profile a WASM module
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn profile_module(&self, binary: &[u8]) -> Result<ProfilingReport> {
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

    /// Snapshot the shadow stacks collected so far by [`Self::start_sampling`].
    ///
    /// `start_sampling` pushed every sample into a private
    /// `Arc<RwLock<Vec<ShadowStack>>>` that had **no reader anywhere in the
    /// crate**: the sampler looked like it worked, and the profiling data it
    /// produced was unreachable by construction. This is the accessor that
    /// makes the collected data observable.
    ///
    /// Returns `None` when the lock is poisoned — that is "not measured", and
    /// it is deliberately distinct from `Some(vec![])`, which means "measured,
    /// nothing sampled yet". Collapsing the two would render absence as
    /// success.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn samples(&self) -> Option<Vec<ShadowStack>> {
        self.shadow_stacks.read().ok().map(|s| s.clone())
    }

    /// Start asynchronous sampling of a running instance
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

#[cfg(test)]
mod start_sampling_tests {
    //! #974. `start_sampling` had no test at all, and no caller: the samples it
    //! collected went into a private vector nothing could read.
    //!
    //! Every test here runs on a **paused** tokio clock (`start_paused = true`),
    //! so the sample counts are exact rather than timing-dependent — a test that
    //! only asserted "some samples appeared" would still pass if the
    //! `sample_interval` argument were ignored.
    use super::*;

    fn count(profiler: &AsyncProfiler) -> usize {
        profiler
            .samples()
            .expect("shadow stack lock must not be poisoned")
            .len()
    }

    #[tokio::test(start_paused = true)]
    async fn sampling_writes_shadow_stacks_the_caller_can_read_back() {
        let profiler = AsyncProfiler::new();
        assert_eq!(count(&profiler), 0, "nothing sampled before start_sampling");

        let handle = profiler.start_sampling(Duration::from_millis(100));
        // interval fires immediately, then every 100ms: t=0,100,200,300.
        tokio::time::sleep(Duration::from_millis(350)).await;
        handle.abort();

        let samples = profiler
            .samples()
            .expect("shadow stack lock must not be poisoned");
        assert_eq!(samples.len(), 4, "one sample per 100ms tick over 350ms");
        assert!(
            samples
                .iter()
                .all(|s| s.depth() == 2 && s.contains_function(1) && s.contains_function(5)),
            "each sample must be a real ShadowStack::sample(), not a placeholder"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn the_sample_interval_argument_decides_the_sampling_rate() {
        let fast = AsyncProfiler::new();
        let slow = AsyncProfiler::new();

        let fast_handle = fast.start_sampling(Duration::from_millis(10));
        let slow_handle = slow.start_sampling(Duration::from_millis(500));
        // 995ms, deliberately off every tick boundary so no sample races the
        // wake-up of this task.
        tokio::time::sleep(Duration::from_millis(995)).await;
        fast_handle.abort();
        slow_handle.abort();

        // If the argument were ignored (e.g. replaced by the hardcoded
        // `self.sample_interval` of 10ms) these two counts would be equal.
        assert_eq!(count(&fast), 100, "10ms sampler: ticks at 0, 10, .., 990");
        assert_eq!(count(&slow), 2, "500ms sampler: ticks at 0 and 500");
    }

    #[tokio::test(start_paused = true)]
    async fn aborting_the_returned_handle_stops_the_sampler() {
        let profiler = AsyncProfiler::new();
        let handle = profiler.start_sampling(Duration::from_millis(100));
        tokio::time::sleep(Duration::from_millis(250)).await;

        handle.abort();
        let at_abort = count(&profiler);
        assert_eq!(at_abort, 3, "ticks at 0, 100, 200");

        tokio::time::sleep(Duration::from_millis(5_000)).await;
        assert_eq!(
            count(&profiler),
            at_abort,
            "the returned handle must own the sampling task"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn sampling_stops_itself_at_the_ten_thousand_sample_limit() {
        let profiler = AsyncProfiler::new();
        let handle = profiler.start_sampling(Duration::from_millis(1));

        // The task must terminate on its own; nothing aborts it here.
        handle.await.expect("sampler task must finish at the limit");
        assert_eq!(count(&profiler), 10_000, "the sample cap is 10_000");
    }

    #[test]
    fn a_poisoned_lock_reports_not_measured_rather_than_empty() {
        let profiler = AsyncProfiler::new();
        let stacks = profiler.shadow_stacks.clone();
        let poisoner = std::thread::spawn(move || {
            let _guard = stacks.write().expect("first write must succeed");
            panic!("poison the shadow stack lock");
        });
        assert!(poisoner.join().is_err(), "the poisoning thread must panic");

        assert!(
            profiler.samples().is_none(),
            "an unreadable lock is `not measured` (None), never an empty sample set"
        );
    }
}
