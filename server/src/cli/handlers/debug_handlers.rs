// DEBUG-002: Debug Command Handlers
// Sprint 74 - GREEN Phase
// Sprint 75 - REPLAY-003: Recording deserialization integration
// Sprint 76 - CAPTURE-003: CLI Recording Workflow
//
// Handlers for `pmat debug` subcommands

use crate::services::dap::{ComparisonView, DapServer, Recording, TimelinePlayer, TimelineUI};
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Handle `pmat debug serve` command
///
/// Starts a DAP (Debug Adapter Protocol) server on the specified port
/// allowing debuggers like VSCode to connect for time-travel debugging.
///
/// Sprint 76: Now supports optional recording to .pmat files via --record-dir
///
/// # Arguments
/// * `port` - Port number to bind the DAP server (default: 5678)
/// * `host` - Host address to bind (default: "127.0.0.1")
/// * `record_dir` - Optional directory to save recording files (Sprint 76)
///
/// # Returns
/// * `Ok(())` if server starts successfully
/// * `Err` if port is already in use or other startup errors
pub async fn handle_debug_serve(
    port: u16,
    host: String,
    record_dir: Option<PathBuf>,
) -> Result<()> {
    println!("🔍 Starting DAP server...");
    println!("   Host: {}", host);
    println!("   Port: {}", port);

    // Sprint 76: Display recording configuration
    if let Some(ref dir) = record_dir {
        println!("   Recording: enabled");
        println!("   Record directory: {}", dir.display());
    } else {
        println!("   Recording: disabled");
    }

    println!();
    println!("Connect your debugger to: {}:{}", host, port);
    println!("Press Ctrl+C to stop the server");
    println!();

    // Sprint 76: Create server with optional recording support
    let server = DapServer::with_recording(record_dir);
    server.run(port, host).await?;

    Ok(())
}

/// Handle `pmat debug replay` command
///
/// Replays a time-travel debugging recording with Timeline UI visualization.
/// Sprint 75: Fully integrated with Recording deserialization.
///
/// # Arguments
/// * `recording` - Path to the .pmat recording file
/// * `position` - Optional position to jump to (frame number)
/// * `interactive` - Enable interactive step-through mode
///
/// # Returns
/// * `Ok(())` if replay completes successfully
/// * `Err` if recording file not found or invalid format
pub async fn handle_debug_replay(
    recording: PathBuf,
    position: Option<usize>,
    interactive: bool,
) -> Result<()> {
    // Validate recording file exists
    if !recording.exists() {
        anyhow::bail!("Recording file not found: {}", recording.display());
    }

    println!("🎬 Replaying debug recording...");
    println!("   Recording: {}", recording.display());
    println!();

    // Load recording (Sprint 75 - REPLAY-003)
    let loaded = Recording::load_from_file(&recording).context("Failed to load recording file")?;

    // Display recording metadata
    println!("📋 Recording Metadata:");
    let metadata = loaded.metadata();
    println!("   Program: {}", metadata.program);

    if !metadata.args.is_empty() {
        println!("   Arguments: {}", metadata.args.join(" "));
    }

    // Format timestamp
    use std::time::{Duration, UNIX_EPOCH};
    if let Some(system_time) = UNIX_EPOCH.checked_add(Duration::from_millis(metadata.timestamp)) {
        println!("   Recorded: {:?}", system_time);
    }

    if !metadata.environment.is_empty() {
        println!("   Environment variables: {}", metadata.environment.len());
    }

    println!("   Snapshots: {}", loaded.snapshot_count());
    println!();

    // Handle position jump
    let target_position = position.unwrap_or(0);
    if target_position >= loaded.snapshot_count() {
        anyhow::bail!(
            "Position {} out of range (recording has {} snapshots)",
            target_position,
            loaded.snapshot_count()
        );
    }

    // Display snapshot information
    if loaded.snapshot_count() > 0 {
        println!("📊 Snapshot at position {}:", target_position);
        let snapshot = &loaded.snapshots()[target_position];

        println!("   Frame ID: {}", snapshot.frame_id);
        println!("   Timestamp: {}ms", snapshot.timestamp_relative_ms);
        println!(
            "   Instruction Pointer: 0x{:x}",
            snapshot.instruction_pointer
        );

        if !snapshot.variables.is_empty() {
            println!("   Variables: {}", snapshot.variables.len());
            for (name, value) in snapshot.variables.iter().take(5) {
                println!("      {} = {}", name, value);
            }
            if snapshot.variables.len() > 5 {
                println!("      ... and {} more", snapshot.variables.len() - 5);
            }
        }

        if !snapshot.stack_frames.is_empty() {
            println!("   Stack Frames: {}", snapshot.stack_frames.len());
            for (i, frame) in snapshot.stack_frames.iter().take(3).enumerate() {
                let location = match (&frame.file, &frame.line) {
                    (Some(file), Some(line)) => format!(" @ {}:{}", file, line),
                    _ => String::new(),
                };
                println!("      #{} {}{}", i, frame.name, location);
            }
            if snapshot.stack_frames.len() > 3 {
                println!(
                    "      ... and {} more frames",
                    snapshot.stack_frames.len() - 3
                );
            }
        }

        if snapshot.memory_snapshot.is_some() {
            println!("   Memory snapshot: present");
        }
        println!();
    }

    // Interactive mode
    if interactive {
        println!("🎮 Interactive Mode:");
        println!("   [Interactive step-through would appear here]");
        println!("   Sprint 72-73 Timeline UI integration pending");
        println!();
    }

    println!("✅ Replay complete");
    Ok(())
}

/// Handle `pmat debug timeline` command
///
/// Interactive timeline playback for a single recording with frame navigation.
/// Sprint 77 - TIMELINE-004: CLI Integration
///
/// # Arguments
/// * `recording` - Path to the .pmat recording file
///
/// # Returns
/// * `Ok(())` if timeline playback completes successfully
/// * `Err` if recording file not found or invalid format
pub async fn handle_debug_timeline(recording: PathBuf) -> Result<()> {
    // Validate recording file exists
    if !recording.exists() {
        anyhow::bail!("Recording file not found: {}", recording.display());
    }

    println!("⏱️  Timeline Playback...");
    println!("   Recording: {}", recording.display());
    println!();

    // Load recording
    let loaded = Recording::load_from_file(&recording).context("Failed to load recording file")?;

    // Display recording metadata
    println!("📋 Recording Metadata:");
    let metadata = loaded.metadata();
    println!("   Program: {}", metadata.program);
    println!("   Snapshots: {}", loaded.snapshot_count());
    println!();

    // Create TimelinePlayer and UI
    let player = TimelinePlayer::new(loaded);
    let ui = TimelineUI::from_player(player);

    println!("🎮 Timeline Player created");
    println!("   {}", ui.progress_text());
    println!();

    // Display frame info
    println!("📊 Frame Info:");
    println!("   {}", ui.progress_text());

    // Get current variables and stack frames
    let variables = ui.current_variables();
    let stack_frames = ui.current_stack_frames();

    // Display source location
    if !stack_frames.is_empty() {
        let frame = &stack_frames[0];
        if let (Some(file), Some(line)) = (&frame.file, &frame.line) {
            println!("   Location: {}:{}", file, line);
        }
    }

    // Display variables
    if !variables.is_empty() {
        println!("   Variables: {}", variables.len());
        for (name, value) in variables.iter().take(5) {
            println!("      {} = {}", name, value);
        }
        if variables.len() > 5 {
            println!("      ... and {} more", variables.len() - 5);
        }
    }
    println!();

    println!("✅ Timeline playback ready");
    println!("   [Interactive UI would appear here - Sprint 77 TIMELINE-002]");
    Ok(())
}

/// Handle `pmat debug compare` command
///
/// Side-by-side comparison of two recordings with diff highlighting.
/// Sprint 77 - TIMELINE-004: CLI Integration
///
/// # Arguments
/// * `recording_a` - Path to the first .pmat recording file
/// * `recording_b` - Path to the second .pmat recording file
///
/// # Returns
/// * `Ok(())` if comparison completes successfully
/// * `Err` if either recording file not found or invalid format
pub async fn handle_debug_compare(recording_a: PathBuf, recording_b: PathBuf) -> Result<()> {
    // Validate both recording files exist
    if !recording_a.exists() {
        anyhow::bail!("Recording A not found: {}", recording_a.display());
    }
    if !recording_b.exists() {
        anyhow::bail!("Recording B not found: {}", recording_b.display());
    }

    println!("🔀 Comparing Recordings...");
    println!("   Recording A: {}", recording_a.display());
    println!("   Recording B: {}", recording_b.display());
    println!();

    // Load both recordings
    let loaded_a = Recording::load_from_file(&recording_a).context("Failed to load recording A")?;
    let loaded_b = Recording::load_from_file(&recording_b).context("Failed to load recording B")?;

    println!("📋 Recording Metadata:");
    println!(
        "   Recording A: {} ({} snapshots)",
        loaded_a.metadata().program,
        loaded_a.snapshot_count()
    );
    println!(
        "   Recording B: {} ({} snapshots)",
        loaded_b.metadata().program,
        loaded_b.snapshot_count()
    );
    println!();

    // Create ComparisonView
    let comparison = ComparisonView::new(loaded_a, loaded_b);
    println!("🎮 ComparisonView created");
    println!();

    // Display split view
    println!("📊 Split View:");
    let split_output = comparison.render_split();
    println!("{}", split_output);

    // Display variable diff
    let diff = comparison.variable_diff();
    if !diff.is_empty() {
        println!("🔍 Variable Differences:");
        for (name, status) in diff.iter().take(10) {
            let status_icon = match status {
                crate::services::dap::DiffStatus::Same => "✓",
                crate::services::dap::DiffStatus::Modified => "~",
                crate::services::dap::DiffStatus::Added => "+",
                crate::services::dap::DiffStatus::Removed => "-",
            };
            println!("   {} {}", status_icon, name);
        }
        if diff.len() > 10 {
            println!("   ... and {} more variables", diff.len() - 10);
        }
        println!();
    }

    // Display divergence point if found
    if let Some(divergence_frame) = comparison.find_divergence_point() {
        println!("⚠️  Divergence detected at frame {}", divergence_frame);
    } else {
        println!("✅ Recordings are identical");
    }
    println!();

    println!("✅ Comparison complete");
    Ok(())
}
