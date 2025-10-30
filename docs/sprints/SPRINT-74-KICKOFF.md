# Sprint 74 Kickoff: Debug Command CLI Exposure
## Making Tracing Infrastructure User-Accessible

**Sprint**: 74
**Status**: Ready to Begin
**Date**: October 30, 2025
**Prerequisites**: Sprints 71, 72, 73 complete (infrastructure ready)
**Related Spec**: `docs/specifications/tracing-bug-discovery-tdg-git-expansion-spec.md`

---

## Executive Summary

Sprint 74 exposes the time-travel debugging infrastructure (from Sprints 71-73) through user-facing CLI commands. While Sprints 71-73 built the engine, Sprint 74 builds the **steering wheel** - enabling developers to use DAP debugging and execution replay from the command line.

**Goal**: Transform internal library code into production-ready CLI commands for debugging and tracing.

---

## Sprint Goals

1. **DEBUG-001**: CLI Command Structure
   - Add `pmat debug` command with subcommands
   - Argument parsing for `--serve`, `--replay`, `--port`
   - Command routing to handlers

2. **DEBUG-002**: DAP Server CLI Handler
   - `pmat debug serve` implementation
   - Port configuration and startup
   - Integration with existing DapServer

3. **DEBUG-003**: Replay CLI Handler
   - `pmat replay <recording.pmat>` implementation
   - Terminal UI for timeline navigation
   - Variable diff viewing

---

## Context from Sprints 71-73

### What We Built (Infrastructure)
- ✅ **Sprint 71** (30/30 tests): DAP Protocol Server, Breakpoint Management, Variable Inspection
- ✅ **Sprint 72** (30/30 tests): Execution Recording, Snapshot Compression, Replay Engine
- ✅ **Sprint 73** (20/30 tests): Timeline UI, Variable Diff (VSCode extension deferred)

### What's Missing (CLI Exposure)
- ❌ No `pmat debug` command exists
- ❌ No CLI handler for DAP server
- ❌ No way for users to start debugging session
- ❌ No way to replay recorded executions

---

## Ticket Breakdown

### DEBUG-001: CLI Command Structure

**Goal**: Add `pmat debug` command to CLI with proper argument parsing

**Estimated Time**: 2-3 hours

**Phase**: RED → GREEN → REFACTOR → COMMIT

**Implementation Files**:
- `server/src/cli/commands.rs` - Add Debug variant to Commands enum
- `server/src/cli/command_dispatcher.rs` - Add debug routing
- `server/src/cli/handlers/mod.rs` - Export debug handler

**Command Structure**:
```bash
# Serve DAP server
pmat debug serve [OPTIONS]
  --port <PORT>        # DAP server port (default: 5678)
  --host <HOST>        # Bind address (default: 127.0.0.1)
  --log-level <LEVEL>  # Logging level (default: info)

# Replay execution recording
pmat debug replay <RECORDING>
  --position <POS>     # Start at position (default: 0)
  --interactive        # Interactive timeline navigation
  --details            # Show detailed snapshot info
```

**Test Requirements** (`server/tests/debug_command_tests.rs`):

```rust
// RED Test 1: Parse debug serve command
#[test]
fn test_parse_debug_serve() {
    let args = vec!["pmat", "debug", "serve", "--port", "5678"];
    let cli = parse_args(args);

    match cli.command {
        Commands::Debug(DebugCommand::Serve { port, .. }) => {
            assert_eq!(port, 5678);
        }
        _ => panic!("Expected Debug::Serve command"),
    }
}

// RED Test 2: Parse debug replay command
#[test]
fn test_parse_debug_replay() {
    let args = vec!["pmat", "debug", "replay", "recording.pmat"];
    let cli = parse_args(args);

    match cli.command {
        Commands::Debug(DebugCommand::Replay { recording, .. }) => {
            assert_eq!(recording, PathBuf::from("recording.pmat"));
        }
        _ => panic!("Expected Debug::Replay command"),
    }
}

// RED Test 3: Default port for serve
#[test]
fn test_debug_serve_default_port() {
    let args = vec!["pmat", "debug", "serve"];
    let cli = parse_args(args);

    match cli.command {
        Commands::Debug(DebugCommand::Serve { port, .. }) => {
            assert_eq!(port, 5678); // Default DAP port
        }
        _ => panic!("Expected Debug::Serve command"),
    }
}

// RED Test 4: Command routing to handler
#[test]
async fn test_debug_command_routes_to_handler() {
    let cli = Cli {
        command: Commands::Debug(DebugCommand::Serve {
            port: 5678,
            host: "127.0.0.1".to_string(),
        }),
        ..Default::default()
    };

    let server = create_test_server();
    let result = CommandDispatcher::execute_command(cli.command, server).await;

    assert!(result.is_ok(), "Debug command should route successfully");
}
```

---

### DEBUG-002: DAP Server CLI Handler

**Goal**: Implement `pmat debug serve` to start DAP server

**Estimated Time**: 3-4 hours

**Phase**: RED → GREEN → REFACTOR → COMMIT

**Implementation Files**:
- `server/src/cli/handlers/debug_handlers.rs` (new) - Main debug handler
- `server/src/services/dap/server.rs` - Add async run() method

**Key Features**:
- Start DAP server on specified port
- Bind to specified host
- Log server startup information
- Handle graceful shutdown (Ctrl+C)
- Provide connection instructions to user

**Test Requirements** (`server/tests/debug_serve_tests.rs`):

```rust
// RED Test 1: Start DAP server successfully
#[tokio::test]
async fn test_start_dap_server() {
    let server = DapServer::new();
    let port = 5678;

    let handle = tokio::spawn(async move {
        server.run(port).await
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify server is listening
    let result = TcpStream::connect(("127.0.0.1", port)).await;
    assert!(result.is_ok(), "Server should be listening on port");

    handle.abort();
}

// RED Test 2: Handle port already in use
#[tokio::test]
async fn test_port_already_in_use() {
    // Bind port manually
    let _listener = TcpListener::bind(("127.0.0.1", 5678)).await.unwrap();

    // Try to start server on same port
    let server = DapServer::new();
    let result = server.run(5678).await;

    assert!(result.is_err(), "Should fail when port is in use");
    assert!(result.unwrap_err().to_string().contains("address already in use"));
}

// RED Test 3: Graceful shutdown on signal
#[tokio::test]
async fn test_graceful_shutdown() {
    let server = DapServer::new();
    let port = 5679;

    let handle = tokio::spawn(async move {
        server.run(port).await
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Send shutdown signal
    handle.abort();

    // Verify port is released
    tokio::time::sleep(Duration::from_millis(50)).await;
    let listener = TcpListener::bind(("127.0.0.1", port)).await;
    assert!(listener.is_ok(), "Port should be released after shutdown");
}

// RED Test 4: CLI integration test
#[tokio::test]
async fn test_debug_serve_cli_integration() {
    let output = Command::new("pmat")
        .args(&["debug", "serve", "--port", "5680"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify server is running
    let result = TcpStream::connect(("127.0.0.1", 5680)).await;
    assert!(result.is_ok(), "Server should be accessible via CLI");

    // Kill process
    output.kill().await.ok();
}
```

---

### DEBUG-003: Replay CLI Handler

**Goal**: Implement `pmat debug replay` to view execution recordings

**Estimated Time**: 2-3 hours

**Phase**: RED → GREEN → REFACTOR → COMMIT

**Implementation Files**:
- `server/src/cli/handlers/debug_handlers.rs` - Add replay handler
- Integration with Timeline UI and Variable Diff from Sprint 73

**Key Features**:
- Load execution recording from file
- Display timeline with current position
- Show snapshot details (variables, call stack)
- Interactive navigation (arrow keys)
- Variable diff between snapshots

**Test Requirements** (`server/tests/debug_replay_tests.rs`):

```rust
// RED Test 1: Load and display recording
#[tokio::test]
async fn test_load_recording() {
    let recording_path = create_test_recording("test_recording.pmat", 10);

    let result = handle_debug_replay(recording_path, None, false).await;

    assert!(result.is_ok(), "Should load recording successfully");
}

// RED Test 2: Display timeline
#[tokio::test]
async fn test_display_timeline() {
    let recording = create_test_recording_in_memory(10);
    let ui = TimelineUI::new(recording);

    let output = ui.render();

    assert!(output.contains("0"), "Timeline should show positions");
    assert!(output.contains("9"), "Timeline should show end position");
}

// RED Test 3: Show snapshot details
#[tokio::test]
async fn test_show_snapshot_details() {
    let recording = create_test_recording_with_vars(5);
    let ui = TimelineUI::new(recording);

    let details = ui.render_details();

    assert!(details.contains("Variables:"), "Should show variables section");
    assert!(details.contains("Call Stack:"), "Should show call stack");
    assert!(details.contains("Location:"), "Should show location");
}

// RED Test 4: CLI integration test
#[tokio::test]
async fn test_debug_replay_cli_integration() {
    let recording_path = create_test_recording("cli_test.pmat", 5);

    let output = Command::new("pmat")
        .args(&["debug", "replay", recording_path.to_str().unwrap()])
        .output()
        .await
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Snapshot"), "Should display snapshot info");
}
```

---

## Sprint 74 Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     CLI Layer (NEW)                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Commands   │  │  Dispatcher  │  │   Debug      │      │
│  │   Enum       │──▶  Routing     │──▶  Handlers    │      │
│  └──────────────┘  └──────────────┘  └──────┬───────┘      │
└────────────────────────────────────────────────┼────────────┘
                                                 │
                                    ┌────────────▼────────────┐
                                    │                         │
                              ┌─────▼─────┐          ┌──────▼──────┐
                              │ Debug     │          │   Debug     │
                              │ Serve     │          │   Replay    │
                              └─────┬─────┘          └──────┬──────┘
                                    │                       │
┌───────────────────────────────────┼───────────────────────┼────┐
│         Sprint 71-73 Infrastructure (Complete)            │    │
│                                   │                       │    │
│                            ┌──────▼─────────┐    ┌───────▼────┐│
│                            │   DAP Server   │    │ Timeline   ││
│                            │   (Sprint 71)  │    │ UI         ││
│                            └────────────────┘    │ (Sprint 73)││
│                                                  └────────────┘│
│                                                                │
│                    ┌──────────────┐    ┌──────────────┐      │
│                    │   Replay     │    │  Variable    │      │
│                    │   Engine     │    │  Diff        │      │
│                    │  (Sprint 72) │    │  (Sprint 73) │      │
│                    └──────────────┘    └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

---

## Files Structure

```
server/
├── src/cli/
│   ├── commands.rs                    # Add Debug variant (DEBUG-001)
│   ├── command_dispatcher.rs          # Add debug routing (DEBUG-001)
│   └── handlers/
│       ├── mod.rs                     # Export debug_handlers (DEBUG-001)
│       └── debug_handlers.rs          # NEW: Debug command handlers (DEBUG-002, DEBUG-003)
└── tests/
    ├── debug_command_tests.rs         # 4 tests for DEBUG-001
    ├── debug_serve_tests.rs           # 4 tests for DEBUG-002
    └── debug_replay_tests.rs          # 4 tests for DEBUG-003

docs/
└── sprints/
    └── SPRINT-74-KICKOFF.md           # This document
```

---

## Success Criteria

### Functional Requirements
- [ ] `pmat debug serve` starts DAP server on specified port
- [ ] `pmat debug replay <file>` displays execution timeline
- [ ] Commands parse correctly with all options
- [ ] All 12 tests passing (4 per ticket)

### Quality Requirements
- [ ] EXTREME TDD methodology followed (RED → GREEN → REFACTOR → COMMIT)
- [ ] Zero clippy warnings
- [ ] Commands documented in `pmat --help`
- [ ] Error messages are user-friendly

### Performance Requirements
- [ ] DAP server startup: <500ms
- [ ] Replay command: <100ms to display timeline

---

## Dependencies

### Internal Dependencies
- Sprint 71 infrastructure (DapServer, BreakpointManager, VariableInspector)
- Sprint 72 infrastructure (ExecutionRecorder, ReplayEngine)
- Sprint 73 infrastructure (TimelineUI, VariableDiff)
- CLI command infrastructure (Commands enum, CommandDispatcher)

### External Dependencies
- **tokio** - Async runtime for DAP server
- **clap** - CLI argument parsing
- **anyhow** - Error handling

---

## EXTREME TDD Methodology

### RED Phase (Write Failing Tests)
1. Write 12 tests total (4 per ticket)
2. Verify tests fail with meaningful errors
3. Tests drive API design

### GREEN Phase (Implement Minimal Code)
1. Implement just enough to pass tests
2. No premature optimization
3. Focus on correctness first

### REFACTOR Phase (Optimize & Clean)
1. Extract common patterns
2. Improve error messages
3. Optimize if needed

### COMMIT Phase (Document & Commit)
1. Comprehensive commit messages
2. Reference ticket numbers
3. Document test results

---

## Timeline

**Total Estimated Time**: 7-10 hours

| Ticket | Estimated Time | Tests |
|--------|---------------|-------|
| DEBUG-001 | 2-3 hours | 4 |
| DEBUG-002 | 3-4 hours | 4 |
| DEBUG-003 | 2-3 hours | 4 |
| **Total** | **7-10 hours** | **12** |

---

## Risk Assessment

### Low Risk
- CLI command structure (well-established pattern in codebase)
- Replay handler (builds on existing Timeline UI)

### Medium Risk
- DAP server async integration (tokio runtime)
- Port binding errors (need good error handling)

### Mitigation Strategies
- Test DAP server separately before CLI integration
- Comprehensive error messages for port conflicts
- Have fallback: Replay-only command if DAP server blocks

---

## Next Steps

1. **Create RED tests** for DEBUG-001 (CLI command structure)
2. **Verify tests fail** with meaningful errors
3. **GREEN phase**: Implement command parsing
4. **Repeat** for DEBUG-002 and DEBUG-003

---

**Ready to begin Sprint 74!** 🚀

Let's make time-travel debugging accessible to every PMAT user.
