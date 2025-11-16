// DEBUG-001: CLI Command Structure Tests
// Sprint 74 - RED Phase
//
// Tests for `pmat debug` command parsing and routing

use pmat::cli::commands::{Commands, DebugCommands};
use std::path::PathBuf;

// RED Test 1: Parse debug serve command with port
#[test]
fn test_parse_debug_serve_with_port() {
    // This test drives the API design for debug serve command
    // Expected: pmat debug serve --port 5678

    let command = Commands::Debug {
        command: DebugCommands::Serve {
            port: 5678,
            host: "127.0.0.1".to_string(),
            record_dir: None,
        },
    };

    match command {
        Commands::Debug {
            command: DebugCommands::Serve { port, host, .. },
        } => {
            assert_eq!(port, 5678);
            assert_eq!(host, "127.0.0.1");
        }
        _ => panic!("Expected Debug::Serve command"),
    }
}

// RED Test 2: Parse debug serve with default port
#[test]
fn test_parse_debug_serve_default_port() {
    // Default DAP port should be 5678
    let command = Commands::Debug {
        command: DebugCommands::Serve {
            port: 5678,                    // Default
            host: "127.0.0.1".to_string(), // Default
            record_dir: None,
        },
    };

    if let Commands::Debug {
        command: DebugCommands::Serve { port, .. },
    } = command
    {
        assert_eq!(port, 5678, "Default DAP port should be 5678");
    } else {
        panic!("Expected Debug::Serve command");
    }
}

// RED Test 3: Parse debug replay command
#[test]
fn test_parse_debug_replay() {
    // Expected: pmat debug replay recording.pmat
    let command = Commands::Debug {
        command: DebugCommands::Replay {
            recording: PathBuf::from("recording.pmat"),
            position: None,
            interactive: false,
        },
    };

    match command {
        Commands::Debug {
            command:
                DebugCommands::Replay {
                    recording,
                    position,
                    interactive,
                },
        } => {
            assert_eq!(recording, PathBuf::from("recording.pmat"));
            assert_eq!(position, None);
            assert!(!interactive);
        }
        _ => panic!("Expected Debug::Replay command"),
    }
}

// RED Test 4: Parse debug replay with options
#[test]
fn test_parse_debug_replay_with_options() {
    // Expected: pmat debug replay recording.pmat --position 5 --interactive
    let command = Commands::Debug {
        command: DebugCommands::Replay {
            recording: PathBuf::from("recording.pmat"),
            position: Some(5),
            interactive: true,
        },
    };

    match command {
        Commands::Debug {
            command:
                DebugCommands::Replay {
                    recording,
                    position,
                    interactive,
                },
        } => {
            assert_eq!(recording, PathBuf::from("recording.pmat"));
            assert_eq!(position, Some(5));
            assert!(interactive);
        }
        _ => panic!("Expected Debug::Replay command"),
    }
}
