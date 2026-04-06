impl RateLimiter {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(capacity: u32, refill_rate: u32) -> Self {
        Self {
            capacity,
            tokens: AtomicU32::new(capacity),
            refill_rate,
            last_refill: parking_lot::Mutex::new(Instant::now()),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn try_acquire(&self, tokens: u32) -> bool {
        self.refill();

        let mut current = self.tokens.load(Ordering::Relaxed);
        loop {
            if current < tokens {
                return false; // Would exceed rate limit
            }

            match self.tokens.compare_exchange_weak(
                current,
                current - tokens,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => return true,
                Err(actual) => current = actual,
            }
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn acquire(&self, tokens: u32) {
        while !self.try_acquire(tokens) {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    fn refill(&self) {
        let mut last_refill = self.last_refill.lock();
        let now = Instant::now();
        let elapsed = now.duration_since(*last_refill);

        if elapsed.as_secs_f64() > 0.0 {
            let tokens_to_add = (elapsed.as_secs_f64() * self.refill_rate as f64) as u32;

            if tokens_to_add > 0 {
                let current = self.tokens.load(Ordering::Relaxed);
                let new_tokens = current.saturating_add(tokens_to_add).min(self.capacity);
                self.tokens.store(new_tokens, Ordering::Relaxed);
                *last_refill = now;
            }
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn available_tokens(&self) -> u32 {
        self.refill();
        self.tokens.load(Ordering::Relaxed)
    }
}
