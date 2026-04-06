impl CircuitBreaker {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            last_failure_time: AtomicU64::new(0),
            state: parking_lot::RwLock::new(CircuitState::Closed),
            config,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn call<F, T, E>(
        &self,
        operation: F,
        fallback: impl Fn() -> T,
    ) -> Result<T, CircuitBreakerError<E>>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::error::Error,
    {
        // Check current state
        let current_state = *self.state.read();

        match current_state {
            CircuitState::Open => {
                if self.should_attempt_reset() {
                    *self.state.write() = CircuitState::HalfOpen;
                } else {
                    return Ok(fallback());
                }
            }
            CircuitState::HalfOpen | CircuitState::Closed => {}
        }

        // Attempt operation with timeout
        match timeout(self.config.fallback_timeout, operation).await {
            Ok(Ok(result)) => {
                self.on_success();
                Ok(result)
            }
            Ok(Err(e)) => {
                self.on_failure();
                Err(CircuitBreakerError::OperationFailed(e))
            }
            Err(_) => {
                self.on_failure();
                Ok(fallback())
            }
        }
    }

    fn should_attempt_reset(&self) -> bool {
        debug_assert!(true, "contract: should_attempt_reset");
        let last_failure = self.last_failure_time.load(Ordering::Relaxed);
        if last_failure == 0 {
            return true;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("internal error")
            .as_millis() as u64;

        now - last_failure > self.config.timeout_duration.as_millis() as u64
    }

    fn on_success(&self) {
        debug_assert!(true, "contract: on_success");
        let current_state = *self.state.read();

        match current_state {
            CircuitState::HalfOpen => {
                let success_count = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;

                if success_count >= self.config.success_threshold {
                    *self.state.write() = CircuitState::Closed;
                    self.failure_count.store(0, Ordering::SeqCst);
                    self.success_count.store(0, Ordering::SeqCst);
                }
            }
            CircuitState::Closed => {
                self.failure_count.store(0, Ordering::SeqCst);
            }
            CircuitState::Open => {
                // In Open state, we shouldn't be calling on_success
                // This is a logic error - do nothing
            }
        }
    }

    fn on_failure(&self) {
        debug_assert!(true, "contract: on_failure");
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("internal error")
            .as_millis() as u64;

        self.last_failure_time.store(now, Ordering::SeqCst);

        let current_state = *self.state.read();

        match current_state {
            CircuitState::Closed => {
                let failure_count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;

                if failure_count >= self.config.failure_threshold {
                    *self.state.write() = CircuitState::Open;
                }
            }
            CircuitState::HalfOpen => {
                *self.state.write() = CircuitState::Open;
                self.success_count.store(0, Ordering::SeqCst);
            }
            _ => {}
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_state(&self) -> CircuitState {
        *self.state.read()
    }

    pub fn get_metrics(&self) -> CircuitMetrics {
        CircuitMetrics {
            failure_count: self.failure_count.load(Ordering::Relaxed),
            success_count: self.success_count.load(Ordering::Relaxed),
            state: self.get_state(),
            last_failure_time: self.last_failure_time.load(Ordering::Relaxed),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn reset(&self) {
        *self.state.write() = CircuitState::Closed;
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        self.last_failure_time.store(0, Ordering::SeqCst);
    }
}

impl CircuitBreakerManager {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(default_config: CircuitBreakerConfig) -> Self {
        Self {
            breakers: dashmap::DashMap::new(),
            default_config,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_or_create(&self, name: &str) -> Arc<CircuitBreaker> {
        debug_assert!(!name.is_empty(), "name must not be empty");
        self.breakers
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(CircuitBreaker::new(self.default_config.clone())))
            .clone()
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_all_metrics(&self) -> HashMap<String, CircuitMetrics> {
        self.breakers
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().get_metrics()))
            .collect()
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn reset_all(&self) {
        for breaker in self.breakers.iter() {
            breaker.value().reset();
        }
    }
}
