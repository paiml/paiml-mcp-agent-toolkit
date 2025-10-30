// DEBUG-002: Debug Command Handlers
// Sprint 74 - GREEN Phase
// Sprint 75 - REPLAY-003: Recording deserialization integration
// Sprint 76 - CAPTURE-003: CLI Recording Workflow
//
// Handlers for `pmat debug` subcommands

use crate::services::dap::{DapServer, Recording};
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
    let loaded = Recording::load_from_file(&recording)
        .context("Failed to load recording file")?;

    // Display recording metadata
    println!("📋 Recording Metadata:");
    let metadata = loaded.metadata();
    println!("   Program: {}", metadata.program);

    if !metadata.args.is_empty() {
        println!("   Arguments: {}", metadata.args.join(" "));
    }

    // Format timestamp
    use std::time::{Duration, UNIX_EPOCH};
    if let Some(datetime) = UNIX_EPOCH.checked_add(Duration::from_millis(metadata.timestamp)) {
        if let Ok(system_time) = std::time::SystemTime::try_from(datetime) {
            println!("   Recorded: {:?}", system_time);
        }
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
        println!("   Instruction Pointer: 0x{:x}", snapshot.instruction_pointer);

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
                println!("      ... and {} more frames", snapshot.stack_frames.len() - 3);
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
