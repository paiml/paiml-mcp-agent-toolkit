// Included from progress.rs — SimpleProgressBar impl + Clone impl
// NO use imports or #! inner attributes allowed (shares parent module scope)

impl Clone for SimpleProgressBar {
    fn clone(&self) -> Self {
        Self {
            message: Arc::clone(&self.message),
            position: Arc::clone(&self.position),
            length: Arc::clone(&self.length),
            hidden: self.hidden,
        }
    }
}

impl SimpleProgressBar {
    /// Create a new progress bar
    #[must_use]
    pub fn new(len: u64) -> Self {
        Self {
            message: Arc::new(Mutex::new(String::new())),
            position: Arc::new(AtomicU64::new(0)),
            length: Arc::new(AtomicU64::new(len)),
            hidden: false,
        }
    }

    /// Create a spinner (indeterminate progress)
    #[must_use]
    pub fn new_spinner() -> Self {
        Self::new(0)
    }

    /// Create a hidden progress bar (no output)
    #[must_use]
    pub fn hidden() -> Self {
        Self {
            message: Arc::new(Mutex::new(String::new())),
            position: Arc::new(AtomicU64::new(0)),
            length: Arc::new(AtomicU64::new(0)),
            hidden: true,
        }
    }

    /// Check if this is a hidden progress bar
    #[must_use]
    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    /// Set the current message
    pub fn set_message(&self, msg: impl Into<String>) {
        if let Ok(mut m) = self.message.lock() {
            *m = msg.into();
        }
    }

    /// Get the current message
    #[must_use]
    pub fn message(&self) -> String {
        self.message.lock().map(|m| m.clone()).unwrap_or_default()
    }

    /// Set the position
    pub fn set_position(&self, pos: u64) {
        self.position.store(pos, Ordering::Relaxed);
    }

    /// Get the current position
    #[must_use]
    pub fn position(&self) -> u64 {
        self.position.load(Ordering::Relaxed)
    }

    /// Increment the position by delta
    pub fn inc(&self, delta: u64) {
        self.position.fetch_add(delta, Ordering::Relaxed);
    }

    /// Set the total length
    pub fn set_length(&self, len: u64) {
        self.length.store(len, Ordering::Relaxed);
    }

    /// Get the length
    #[must_use]
    pub fn length(&self) -> Option<u64> {
        let len = self.length.load(Ordering::Relaxed);
        if len > 0 {
            Some(len)
        } else {
            None
        }
    }

    /// Finish the progress bar with a message
    pub fn finish_with_message(&self, msg: impl Into<String>) {
        if !self.hidden {
            eprintln!("{}", msg.into());
        }
    }

    /// Finish the progress bar
    pub fn finish(&self) {
        // No-op for simple implementation
    }

    /// Finish and clear the progress bar
    pub fn finish_and_clear(&self) {
        // No-op for simple implementation
    }

    /// Abandon the progress bar
    pub fn abandon(&self) {
        // No-op for simple implementation
    }

    /// Abandon the progress bar with a message
    pub fn abandon_with_message(&self, msg: impl Into<String>) {
        if !self.hidden {
            eprintln!("{}", msg.into());
        }
    }

    /// Enable steady tick (no-op without indicatif)
    pub fn enable_steady_tick(&self, _duration: std::time::Duration) {
        // No-op - we don't animate without indicatif
    }

    /// Tick the progress bar (no-op)
    pub fn tick(&self) {
        // No-op
    }

    /// Set style (no-op without indicatif)
    pub fn set_style(&self, _style: SimpleProgressStyle) {
        // No-op - styles not supported without indicatif
    }

    /// Print a message above the progress bar
    pub fn println(&self, msg: impl AsRef<str>) {
        if !self.hidden {
            eprintln!("{}", msg.as_ref());
        }
    }

    /// Suspend the progress bar for closure execution
    pub fn suspend<F: FnOnce() -> R, R>(&self, f: F) -> R {
        f()
    }
}
