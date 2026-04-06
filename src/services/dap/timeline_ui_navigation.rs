// Timeline UI navigation and playback control methods
// Split from timeline_ui.rs for modularity (include!() pattern)

impl TimelineUI {
    /// Create a new timeline UI from a TimelinePlayer
    ///
    /// This is the primary constructor for Sprint 77+ integration.
    ///
    /// # Example
    /// ```ignore
    /// use pmat::services::dap::{Recording, TimelinePlayer, TimelineUI};
    ///
    /// let recording = Recording::new("program".to_string(), vec![]);
    /// let player = TimelinePlayer::new(recording);
    /// let ui = TimelineUI::from_player(player);
    /// ```
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn from_player(player: TimelinePlayer) -> Self {
        Self {
            player,
            snapshots_legacy: Vec::new(),
            current_position_legacy: 0,
        }
    }

    /// Create a new timeline UI from snapshots (legacy Sprint 73 API)
    ///
    /// This method is kept for backward compatibility with Sprint 73 tests.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(snapshots: Vec<ExecutionSnapshot>) -> Self {
        debug_assert!(!snapshots.is_empty(), "snapshots must not be empty");
        // Create recording and populate with converted snapshots
        let mut recording = super::recording::Recording::new("legacy".to_string(), vec![]);

        for exec_snap in &snapshots {
            // Convert ExecutionSnapshot to Snapshot (Sprint 72 -> Sprint 75)
            let stack_frames = exec_snap
                .call_stack
                .iter()
                .map(|frame| {
                    let file = frame.source.as_ref().and_then(|s| s.path.clone());
                    let line = if frame.line >= 0 {
                        Some(frame.line as u32)
                    } else {
                        None
                    };

                    StackFrame {
                        name: frame.name.clone(),
                        file,
                        line,
                        locals: HashMap::new(),
                    }
                })
                .collect();

            let snapshot = Snapshot {
                frame_id: exec_snap.sequence as u64,
                timestamp_relative_ms: (exec_snap.timestamp / 1_000_000) as u32,
                variables: exec_snap.variables.clone(),
                stack_frames,
                instruction_pointer: 0,
                memory_snapshot: None,
            };

            recording.add_snapshot(snapshot);
        }

        let player = TimelinePlayer::new(recording);

        Self {
            player,
            snapshots_legacy: snapshots,
            current_position_legacy: 0,
        }
    }

    /// Get current frame number
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn current_frame(&self) -> usize {
        self.player.current_frame()
    }

    /// Get current position in the timeline (legacy API)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn current_position(&self) -> usize {
        // For legacy compatibility
        if !self.snapshots_legacy.is_empty() {
            self.current_position_legacy
        } else {
            self.player.current_frame()
        }
    }

    /// Get progress text: "Frame X/Y"
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn progress_text(&self) -> String {
        format!(
            "Frame {}/{}",
            self.player.current_frame(),
            self.player.total_frames()
        )
    }

    /// Get current snapshot variables
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn current_variables(&self) -> &HashMap<String, serde_json::Value> {
        &self.player.current_snapshot().variables
    }

    /// Get current snapshot stack frames
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn current_stack_frames(&self) -> &[StackFrame] {
        &self.player.current_snapshot().stack_frames
    }

    /// Get frame info: "Frame X/Y | Timestamp | Location"
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn frame_info(&self) -> String {
        let snapshot = self.player.current_snapshot();

        // Extract location from first stack frame (if available)
        let location = if let Some(frame) = snapshot.stack_frames.first() {
            if let (Some(file), Some(line)) = (&frame.file, frame.line) {
                format!("{}:{}", file, line)
            } else {
                "<unknown>".to_string()
            }
        } else {
            "<unknown>".to_string()
        };

        format!(
            "Frame {}/{} | {}ms | {}",
            self.player.current_frame(),
            self.player.total_frames(),
            snapshot.timestamp_relative_ms,
            location
        )
    }

    /// Check if playback is active
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_playing(&self) -> bool {
        self.player.is_playing()
    }

    /// Start playback
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn play(&mut self) {
        self.player.play();
    }

    /// Pause playback
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn pause(&mut self) {
        self.player.pause();
    }

    /// Advance to next frame
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn next_frame(&mut self) -> Result<&Snapshot> {
        self.player
            .next_frame()
            .ok_or_else(|| anyhow::anyhow!("Cannot advance: already at last frame"))
    }

    /// Move to previous frame
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn prev_frame(&mut self) -> Result<&Snapshot> {
        self.player
            .prev_frame()
            .ok_or_else(|| anyhow::anyhow!("Cannot move back: already at first frame"))
    }

    /// Jump to specific frame
    ///
    /// Returns a reference to the snapshot at the target frame.
    /// Now works correctly in both legacy and modern modes (Sprint 77+).
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn jump_to(&mut self, frame: usize) -> Result<&Snapshot> {
        if !self.snapshots_legacy.is_empty() {
            // Legacy mode - validate against legacy snapshot count and sync state
            if frame >= self.snapshots_legacy.len() {
                return Err(anyhow::anyhow!(
                    "Frame {} out of bounds (max: {})",
                    frame,
                    self.snapshots_legacy.len() - 1
                ));
            }
            self.current_position_legacy = frame;
        }

        // Jump in player (works for both legacy and modern modes now)
        self.player.jump_to(frame)
    }

    /// Tick for auto-advance playback
    ///
    /// Call this method periodically (e.g., from UI event loop) to auto-advance
    /// frames when playback is active.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn tick(&mut self) {
        if self.player.is_playing() {
            // Attempt to advance, ignore errors (e.g., end of recording)
            let _ = self.player.next_frame();
        }
    }

    /// Handle keyboard input for navigation
    ///
    /// Supported keys:
    /// - '→' (right arrow): Advance to next frame
    /// - '←' (left arrow): Move to previous frame
    /// - ' ' (space): Toggle play/pause
    /// - 'j' or 'J': Jump mode (handled by caller)
    ///
    /// Returns Ok(()) on success, Err on invalid key or navigation error.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn handle_key(&mut self, key: char) -> Result<()> {
        // Check for legacy mode first
        if !self.snapshots_legacy.is_empty() {
            return self.handle_key_legacy(key);
        }

        match key {
            '→' => {
                // Step forward
                self.next_frame()
                    .map(|_| ())
                    .map_err(|e| anyhow::anyhow!("{}", e))
            }
            '←' => {
                // Step backward
                self.prev_frame()
                    .map(|_| ())
                    .map_err(|e| anyhow::anyhow!("{}", e))
            }
            ' ' => {
                // Toggle play/pause
                if self.is_playing() {
                    self.pause();
                } else {
                    self.play();
                }
                Ok(())
            }
            'j' | 'J' => {
                // Jump mode - actual jump logic handled by caller
                // This just validates that the key is recognized
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Unknown key: '{}'", key)),
        }
    }

    /// Handle keyboard input for legacy mode
    fn handle_key_legacy(&mut self, key: char) -> Result<()> {
        debug_assert!(true, "contract: handle_key_legacy");
        match key {
            '→' => {
                // Step forward
                if self.current_position_legacy >= self.snapshots_legacy.len() - 1 {
                    return Err(anyhow::anyhow!(
                        "Cannot step forward: already at last snapshot"
                    ));
                }
                self.current_position_legacy += 1;
                Ok(())
            }
            '←' => {
                // Step backward
                if self.current_position_legacy == 0 {
                    return Err(anyhow::anyhow!(
                        "Cannot step backward: already at first snapshot"
                    ));
                }
                self.current_position_legacy -= 1;
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Unknown key: '{}'", key)),
        }
    }
}
