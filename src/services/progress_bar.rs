// Included from progress.rs — SimpleProgressBar impl + Clone impl
// NO use imports or #! inner attributes allowed (shares parent module scope)

impl Clone for SimpleProgressBar {
    fn clone(&self) -> Self {
        Self {
            last_emit: Arc::clone(&self.last_emit),
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(len: u64) -> Self {
        Self {
            last_emit: Arc::new(Mutex::new(None)),
            message: Arc::new(Mutex::new(String::new())),
            position: Arc::new(AtomicU64::new(0)),
            length: Arc::new(AtomicU64::new(len)),
            hidden: false,
        }
    }

    /// Create a spinner (indeterminate progress)
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new_spinner() -> Self {
        Self::new(0)
    }

    /// Create a hidden progress bar (no output)
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn hidden() -> Self {
        Self {
            last_emit: Arc::new(Mutex::new(None)),
            message: Arc::new(Mutex::new(String::new())),
            position: Arc::new(AtomicU64::new(0)),
            length: Arc::new(AtomicU64::new(0)),
            hidden: true,
        }
    }

    /// Check if this is a hidden progress bar
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    /// Set the current message
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn set_message(&self, msg: impl Into<String>) {
        if let Ok(mut m) = self.message.lock() {
            *m = msg.into();
        }
        self.emit_progress();
    }

    /// Get the current message
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn message(&self) -> String {
        self.message.lock().map(|m| m.clone()).unwrap_or_default()
    }

    /// Set the position
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn set_position(&self, pos: u64) {
        self.position.store(pos, Ordering::Relaxed);
    }

    /// Get the current position
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn position(&self) -> u64 {
        self.position.load(Ordering::Relaxed)
    }

    /// Increment the position by delta
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn inc(&self, delta: u64) {
        self.position.fetch_add(delta, Ordering::Relaxed);
        self.emit_progress();
    }

    /// Set the total length
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn set_length(&self, len: u64) {
        self.length.store(len, Ordering::Relaxed);
    }

    /// Get the length
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn length(&self) -> Option<u64> {
        let len = self.length.load(Ordering::Relaxed);
        if len > 0 {
            Some(len)
        } else {
            None
        }
    }

    /// Finish the progress bar with a message
    ///
    /// Progress text is status chatter, so it goes through
    /// [`status_eprintln!`](crate::status_eprintln) and is silent under
    /// `--quiet`. `pmat context` used to print "Analyzing project…" /
    /// "Analyses complete" / "Analysis complete!" through this bar no matter
    /// what, which is why `context --quiet` was byte-identical to `context`.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn finish_with_message(&self, msg: impl Into<String>) {
        self.clear_emitted();
        if !self.hidden {
            crate::status_eprintln!("{}", msg.into());
        }
    }

    /// Finish the progress bar
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn finish(&self) {
        self.clear_emitted();
        // No-op for simple implementation
    }

    /// Finish and clear the progress bar
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn finish_and_clear(&self) {
        self.clear_emitted();
        // No-op for simple implementation
    }

    /// Abandon the progress bar
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn abandon(&self) {
        self.clear_emitted();
        // No-op for simple implementation
    }

    /// Abandon the progress bar with a message
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn abandon_with_message(&self, msg: impl Into<String>) {
        self.clear_emitted();
        if !self.hidden {
            crate::status_eprintln!("{}", msg.into());
        }
    }

    /// Enable steady tick (no background thread; updates ride on
    /// `set_message` / `tick` / `inc`)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn enable_steady_tick(&self, _duration: std::time::Duration) {
        // No-op - we don't animate without indicatif
    }

    /// Tick the progress bar (emits a throttled stderr line on a TTY)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn tick(&self) {
        self.emit_progress();
    }

    /// Set style (no-op without indicatif)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn set_style(&self, _style: SimpleProgressStyle) {
        // No-op - styles not supported without indicatif
    }

    /// Print a message above the progress bar
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn println(&self, msg: impl AsRef<str>) {
        self.clear_emitted();
        if !self.hidden {
            crate::status_eprintln!("{}", msg.as_ref());
        }
    }

    /// Suspend the progress bar for closure execution
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn suspend<F: FnOnce() -> R, R>(&self, f: F) -> R {
        f()
    }

    /// Minimum gap between two stderr updates.
    const EMIT_INTERVAL: std::time::Duration = std::time::Duration::from_millis(250);

    /// The one-line text for the current state.
    #[must_use]
    pub fn render_line(&self) -> String {
        let msg = self.message();
        match self.length() {
            Some(len) => format!("{msg} [{}/{len}]", self.position()),
            None => msg,
        }
    }

    /// Whether an update is due, given the last emit time.
    #[must_use]
    pub fn emit_due(last: Option<std::time::Instant>, now: std::time::Instant) -> bool {
        last.map_or(true, |t| now.duration_since(t) >= Self::EMIT_INTERVAL)
    }

    /// Write one `\r`-rewritten line to `w` if shown and due. Returns whether
    /// anything was written.
    pub fn emit_to<W: std::io::Write>(
        &self,
        w: &mut W,
        stderr_is_tty: bool,
        quiet: bool,
        now: std::time::Instant,
    ) -> bool {
        if self.hidden || quiet || !stderr_is_tty {
            return false;
        }
        let Ok(mut last) = self.last_emit.lock() else {
            return false;
        };
        if !Self::emit_due(*last, now) {
            return false;
        }
        let line = self.render_line();
        if line.is_empty() {
            return false;
        }
        *last = Some(now);
        write!(w, "\r\x1b[2K{line}").is_ok() && w.flush().is_ok()
    }

    fn emit_progress(&self) {
        use std::io::IsTerminal;
        let stderr = std::io::stderr();
        let tty = stderr.is_terminal();
        let _ = self.emit_to(
            &mut stderr.lock(),
            tty,
            crate::cli::progress::quiet_mode_enabled(),
            std::time::Instant::now(),
        );
    }

    /// Erase a line this bar drew, so the next stderr text starts clean.
    fn clear_emitted(&self) {
        use std::io::{IsTerminal, Write};
        let drew = self
            .last_emit
            .lock()
            .map(|mut l| l.take().is_some())
            .unwrap_or(false);
        let stderr = std::io::stderr();
        if drew && stderr.is_terminal() {
            let _ = write!(stderr.lock(), "\r\x1b[2K");
        }
    }
}
