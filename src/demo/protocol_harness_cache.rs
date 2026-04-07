// Protocol harness cache — ContextCache and TraceStore impl methods
// Included by protocol_harness.rs — no `use` imports or `#!` attributes allowed

impl ContextCache {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(config: DemoConfig) -> Self {
        Self {
            entries: HashMap::new(),
            config,
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get(&self, key: &str) -> Option<&AnalysisResult> {
        self.entries.get(key).map(|entry| &entry.result)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn insert(&mut self, key: String, result: AnalysisResult) {
        let entry = CacheEntry {
            key: key.clone(),
            result,
            created_at: std::time::Instant::now(),
            access_count: 1,
            last_accessed: std::time::Instant::now(),
        };
        self.entries.insert(key, entry);
        self.evict_if_needed();
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn evict_if_needed(&mut self) {
        if self.entries.len() > self.config.max_cache_entries {
            // Simple LRU eviction
            let oldest_key = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.last_accessed)
                .map(|(key, _)| key.clone());

            if let Some(key) = oldest_key {
                self.entries.remove(&key);
            }
        }
    }
}

impl TraceStore {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(max_traces: usize) -> Self {
        Self {
            traces: RwLock::new(HashMap::new()),
            max_traces,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn add_trace(&self, trace: ApiTrace) {
        let mut traces = self.traces.write().await;
        traces.insert(trace.id, trace);

        // Simple cleanup if we exceed max traces
        if traces.len() > self.max_traces {
            // Remove oldest traces
            let mut trace_list: Vec<_> = traces
                .iter()
                .map(|(id, trace)| (*id, trace.created_at))
                .collect();
            trace_list.sort_by_key(|(_, created_at)| *created_at);

            let to_remove: Vec<_> = trace_list
                .iter()
                .take(traces.len() - self.max_traces)
                .map(|(id, _)| *id)
                .collect();

            for id in to_remove {
                traces.remove(&id);
            }
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_trace(&self, trace_id: Uuid) -> Option<ApiTrace> {
        self.traces.read().await.get(&trace_id).cloned()
    }

    pub async fn get_all_traces(&self) -> Vec<ApiTrace> {
        let traces = self.traces.read().await;
        let mut trace_list: Vec<_> = traces.values().cloned().collect();
        trace_list.sort_by_key(|t| t.created_at);
        trace_list
    }
}
