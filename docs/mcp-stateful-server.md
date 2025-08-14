# Stateful MCP Server for Refactor Auto

## Overview

The stateful MCP (Model Context Protocol) server extends pmat's capabilities by providing persistent refactoring sessions. Unlike the stateless CLI, the MCP server maintains state across multiple interactions, enabling complex multi-step refactoring workflows.

## Architecture

### Components

1. **MCP Server** (`src/mcp_server/server.rs`)
   - Handles JSON-RPC communication over stdin/stdout
   - Routes requests to appropriate handlers
   - Manages server lifecycle

2. **State Manager** (`src/mcp_server/state_manager.rs`)
   - Controls RefactorStateMachine lifecycle
   - Manages session start/stop
   - Coordinates state persistence

3. **Snapshot Manager** (`src/mcp_server/snapshots.rs`)
   - Persists state to disk atomically
   - Loads state from snapshots
   - Uses JSON serialization (Cap'n Proto ready)

4. **Handlers** (`src/mcp_server/handlers.rs`)
   - Implements individual MCP methods
   - Validates parameters
   - Transforms state for responses

## MCP API Methods

### `refactor.start`

Initializes a new refactoring session.

**Parameters:**
```json
{
  "targets": ["path/to/file1.rs", "path/to/file2.rs"],
  "config": {
    "target_complexity": 15,
    "remove_satd": true,
    "max_function_lines": 50,
    "parallel_workers": 4,
    "memory_limit_mb": 512,
    "batch_size": 10
  }
}
```

**Response:**
```json
{
  "session_id": "refactor-session-001",
  "state": { /* current RefactorStateMachine state */ }
}
```

### `refactor.nextIteration`

Advances the refactoring state machine to the next iteration.

**Parameters:** None

**Response:**
```json
{
  /* current RefactorStateMachine state after advancement */
}
```

### `refactor.getState`

Retrieves the current refactoring state without modifying it.

**Parameters:** None

**Response:**
```json
{
  /* current RefactorStateMachine state */
}
```

### `refactor.stop`

Stops the current refactoring session and cleans up state.

**Parameters:** None

**Response:**
```json
{
  "message": "Refactoring session stopped successfully"
}
```

## Running the Stateful MCP Server

### Environment Setup

Set the `PMAT_REFACTOR_MCP` environment variable to enable the stateful refactor server:

```bash
export PMAT_REFACTOR_MCP=1
pmat
```

### Direct Invocation

```bash
PMAT_REFACTOR_MCP=1 pmat
```

### Integration with MCP Clients

The server communicates via JSON-RPC over stdin/stdout, making it compatible with any MCP client:

```bash
# Example using echo and pipe
echo '{"jsonrpc":"2.0","method":"refactor.start","params":{"targets":["src/main.rs"]},"id":1}' | PMAT_REFACTOR_MCP=1 pmat
```

## State Persistence

### Snapshot Location

State snapshots are stored in `.pmat-cache/refactor-state.bin` by default.

### Snapshot Format

Currently uses JSON serialization for reliability. The system is designed to support Cap'n Proto binary serialization when available.

### Atomic Operations

Snapshots are saved atomically:
1. Write to temp file (`.pmat-cache/refactor-state.tmp`)
2. Rename to final location (`.pmat-cache/refactor-state.bin`)

This ensures state consistency even if the process is interrupted.

## Error Handling

### Session Management Errors

- **"Session already active"**: Attempt to start a session when one exists
- **"No active session"**: Attempt to advance/stop without a session
- **"No snapshot file found"**: Load attempted without existing state

### Serialization Errors

- **"Failed to create snapshot directory"**: Filesystem permission issues
- **"Failed to write snapshot"**: Disk space or permission issues
- **"Failed to rename snapshot"**: Atomic operation failure

## Development and Testing

### Running Integration Tests

```bash
cargo test --package pmat --test mcp_server_integration
```

### Test Coverage

The MCP server includes comprehensive tests for:
- State lifecycle management
- Session handling (single and multiple)
- Error conditions and recovery
- Snapshot persistence
- Concurrent access patterns
- Complete workflow scenarios

### Creating Custom State Managers

For testing, use `StateManager::with_temp_dir()` to isolate state:

```rust
use tempfile::tempdir;
let temp_dir = tempdir().unwrap();
let manager = StateManager::with_temp_dir(temp_dir.path());
```

## Cap'n Proto Integration

### Schema Location

The Cap'n Proto schema is defined in `src/schema/refactor_state.capnp`.

### Build Configuration

Cap'n Proto compilation is handled by the build script when available:

```bash
PMAT_BUILD_MCP=1 cargo build
```

### Enabling Binary Serialization

When Cap'n Proto compiler is available, the system will automatically use binary serialization for improved performance and smaller snapshot sizes.

## Best Practices

1. **Session Lifecycle**: Always stop sessions when complete to free resources
2. **Error Handling**: Check responses for error states before proceeding
3. **State Validation**: Use `getState` to verify state before critical operations
4. **Concurrent Access**: The server handles concurrent requests safely via mutex protection

## Future Enhancements

1. **Session Recovery**: Automatic session restoration on server restart
2. **Multiple Sessions**: Support for parallel refactoring sessions
3. **Progress Streaming**: Real-time progress updates during long operations
4. **Remote Protocol**: Support for TCP/Unix socket communication
5. **State History**: Undo/redo capabilities with state snapshots

## Troubleshooting

### Server Won't Start

Check that `PMAT_REFACTOR_MCP` is set:
```bash
echo $PMAT_REFACTOR_MCP
```

### State Not Persisting

Verify cache directory permissions:
```bash
ls -la .pmat-cache/
```

### JSON-RPC Errors

Enable debug logging:
```bash
RUST_LOG=debug PMAT_REFACTOR_MCP=1 pmat
```

## Example Workflow

```bash
# Start server
PMAT_REFACTOR_MCP=1 pmat &
SERVER_PID=$!

# Start refactoring session
echo '{"jsonrpc":"2.0","method":"refactor.start","params":{"targets":["src/lib.rs"],"config":{"target_complexity":10}},"id":1}'

# Get current state
echo '{"jsonrpc":"2.0","method":"refactor.getState","id":2}'

# Advance to next iteration
echo '{"jsonrpc":"2.0","method":"refactor.nextIteration","id":3}'

# Stop session
echo '{"jsonrpc":"2.0","method":"refactor.stop","id":4}'

# Kill server
kill $SERVER_PID
```

## Integration with Claude Code and Other Tools

The stateful MCP server is designed to integrate seamlessly with AI coding assistants like Claude Code, enabling them to maintain context across multiple refactoring operations and provide more intelligent suggestions based on the full refactoring history.