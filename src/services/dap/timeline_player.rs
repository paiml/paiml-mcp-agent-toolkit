#![cfg_attr(coverage_nightly, coverage(off))]
//! TIMELINE-001: TimelinePlayer State Management
//! Sprint 77 - GREEN Phase
//!
//! TimelinePlayer manages recording playback state and provides navigation controls
//! for the Timeline UI. It wraps a Recording and tracks the current frame position,
//! playback state, and speed.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ TimelinePlayer                                               │
//! │                                                               │
//! │  ┌─────────────┐         ┌──────────────┐                   │
//! │  │  Recording  │────────▶│  Snapshots   │                   │
//! │  │             │         │  [0..N]      │                   │
//! │  └─────────────┘         └──────────────┘                   │
//! │                                  ▲                           │
//! │                                  │                           │
//! │                          ┌───────┴────────┐                 │
//! │                          │ current_frame  │                 │
//! │                          │   (position)   │                 │
//! │                          └────────────────┘                 │
//! │                                                               │
//! │  Navigation Methods:                                         │
//! │  • next_frame() ──────▶ Advance forward                     │
//! │  • prev_frame() ──────▶ Move backward                       │
//! │  • jump_to(N)   ──────▶ Random access                       │
//! │                                                               │
//! │  Playback Control:                                           │
//! │  • play()       ──────▶ Enable auto-advance                 │
//! │  • pause()      ──────▶ Disable auto-advance                │
//! │  • set_speed()  ──────▶ Adjust playback rate                │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use pmat::services::dap::{Recording, TimelinePlayer};
//! use std::path::PathBuf;
//!
//! // Load recording from file
//! let recording = Recording::load_from_file(&PathBuf::from("session.pmat"))?;
//!
//! // Create player
//! let mut player = TimelinePlayer::new(recording);
//!
//! // Navigate through frames
//! player.next_frame();        // Advance to frame 1
//! player.prev_frame();        // Back to frame 0
//! player.jump_to(50)?;        // Jump to frame 50
//!
//! // Playback control
//! player.play();              // Start auto-advance mode
//! player.set_speed(2.0);      // 2x speed
//! player.pause();             // Stop auto-advance
//!
//! // Access current state
//! let snapshot = player.current_snapshot();
//! let position = player.current_frame();
//! let total = player.total_frames();
//! # Ok::<(), anyhow::Error>(())
//! ```

use super::recording::{Recording, Snapshot};
use anyhow::Result;

/// TimelinePlayer manages recording playback state and navigation
///
/// Wraps a Recording and provides:
/// - Frame position tracking (current_frame)
/// - Navigation controls (next, prev, jump)
/// - Playback state (play, pause, speed)
/// - Current snapshot access for UI rendering
#[derive(Debug)]
pub struct TimelinePlayer {
    /// The recording being played back
    recording: Recording,

    /// Current frame position (0-indexed)
    current_frame: usize,

    /// Total number of frames in recording
    total_frames: usize,

    /// Playback speed multiplier (1.0 = normal speed)
    playback_speed: f64,

    /// Whether playback is currently active
    is_playing: bool,
}

// Navigation and constructor methods (new, current_frame, total_frames, next_frame, prev_frame, jump_to, current_snapshot)
include!("timeline_player_navigation.rs");

// Playback control methods (play, pause, is_playing, set_speed, playback_speed, recording)
include!("timeline_player_playback.rs");

// Unit tests
include!("timeline_player_tests.rs");
