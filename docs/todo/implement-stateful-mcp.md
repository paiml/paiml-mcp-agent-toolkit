# Spec: Stateful MCP Server for `pmat refactor auto`

**Status:** Proposed
**Champion:** Gemini
**Date:** 2025-07-02

## 1. Overview

This document specifies the architecture for evolving `pmat` from a stateless command-line tool into a hybrid system with a persistent, stateful MCP server. The primary goal is to enhance the `pmat refactor auto` feature, transforming it from a single-shot, file-based process into a highly efficient, interactive, and long-running service.

This new architecture will be based on an **in-memory state machine with Cap'n Proto snapshots**, which was determined to be the most plausible and least disruptive evolution of the current system.

## 2. Core Problem & Motivation

The current `pmat refactor auto` command is stateless. On every invocation, it must:
1.  Load the entire state from a JSON file on disk.
2.  Perform one iteration of analysis.
3.  Serialize the entire state back to a JSON file.

This approach, while simple, has significant drawbacks for interactive or server-based use cases:
-   **High Latency:** File I/O and JSON deserialization add significant overhead to each iteration.
-   **Inefficient State Transfer:** JSON is a verbose, text-based format, ill-suited for complex, nested state.
-   **Lack of Interactivity:** The tool cannot be easily controlled or queried by other processes (like an IDE extension) during a refactoring session.

This specification addresses these issues by introducing a true stateful server.

## 3. Technology Choices

To ensure robustness and performance, we will use the following mature and well-tested frameworks:

-   **`tokio`:** As the core asynchronous runtime for the MCP server. It is the de-facto standard for asynchronous programming in Rust, offering exceptional performance and a rich ecosystem.
-   **`capnp`:** The Rust implementation of Cap'n Proto. It will be used for:
    -   **Serialization:** Defining a schema for our state and serializing it to a compact binary format.
    -   **Zero-Copy Reads:** Allowing for extremely fast reads of the state snapshots without the overhead of parsing and allocation that plagues JSON.
-   **`serde`:** Will continue to be used for the JSON-RPC communication over MCP, as the protocol itself is JSON-based. The change is in how the *internal state* is managed, not the public-facing API format.

## 4. Architecture

The proposed architecture introduces a dual-mode system.

### 4.1. High-Level Diagram

```mermaid
graph TD
    subgraph "Stateless Mode (Existing)"
        A[CLI: `pmat refactor auto`] --> B[Load state.json];
        B --> C[Run One Iteration];
        C --> D[Write state.json];
    end

    subgraph "Stateful Mode (New)"
        E[Client (IDE, etc.)] -- JSON-RPC --> F{Persistent `pmat` MCP Server};
        F -- Manages --> G[In-Memory RefactorState];
        G -- Snapshots to --> H[state.bin (Cap'n Proto)];
        F -- Loads from --> H;
    end

    I[User] --> A;
    I --> E;
```

### 4.2. Key Components

1.  **Persistent MCP Server:** A long-running `pmat` process that listens for MCP requests on `stdin`. This process will own and manage the in-memory state.
2.  **In-Memory `RefactorState`:** The `RefactorState` struct will be held in memory within the server, likely wrapped in an `Arc<Mutex<>>` to allow for safe, concurrent access from multiple MCP requests.
3.  **Cap'n Proto Snapshots:** For durability and crash recovery, the in-memory state will be periodically snapshotted to a binary file (`.pmat-cache/refactor-state.bin`) using Cap'n Proto's efficient serialization. The server will load from this file on startup.

### 4.3. Dual-Mode Operation

The system will support both the existing stateless workflow and the new stateful one:

-   **Stateless (CLI):** When a user runs `pmat refactor auto` directly in a terminal, it will function exactly as it does now. This preserves the simplicity of the command for single-shot tasks and CI/CD pipelines.
-   **Stateful (MCP Server):** When `pmat` is launched as an MCP server (e.g., by an IDE), it will initialize the in-memory state and expose new MCP methods to manage the refactoring session.

## 5. State Management

### 5.1. Schema Definition

A new schema file, `server/src/schema/refactor_state.capnp`, will be created. This file will define the structure of the `RefactorState`, mirroring the existing Rust struct but in Cap'n Proto's Interface Definition Language.

### 5.2. Persistence

-   **On Change:** After any action that modifies the state (e.g., completing a refactoring iteration), the server will serialize the in-memory `RefactorState` to the snapshot file.
-   **Atomic Writes:** The write will be atomic to prevent corruption. The server will write to a temporary file and then rename it to the final destination.
-   **On Startup:** When the MCP server starts, it will check for the existence of `refactor-state.bin`. If found, it will load it into the in-memory `RefactorState` object to resume the previous session.

## 6. New MCP API

To manage the stateful session, the following new JSON-RPC methods will be added under the `refactor` namespace:

-   **`refactor.start(params)`:**
    -   Description: Starts a new refactoring session, clearing any existing state.
    -   Params: An object containing configuration options (e.g., `max_iterations`, `exclude_patterns`).
    -   Returns: A session ID and the initial state.

-   **`refactor.nextIteration()`:**
    -   Description: Executes a single iteration of the refactoring loop.
    -   Params: None.
    -   Returns: The updated `RefactorState`.

-   **`refactor.getState()`:**
    -   Description: Retrieves the current state of the refactoring session without advancing it.
    -   Params: None.
    -   Returns: The current `RefactorState`.

-   **`refactor.stop()`:**
    -   Description: Ends the current refactoring session and clears the in-memory state.
    -   Params: None.
    -   Returns: A confirmation message.

## 7. Example Workflow

1.  **IDE Starts `pmat`:** An IDE extension launches the `pmat` process in MCP mode.
2.  **Server Initializes:** `pmat` starts, loads any existing state from `refactor-state.bin`, and waits for requests.
3.  **User Starts Refactoring:** The user clicks a "Start Refactoring" button in the IDE.
4.  **IDE Sends Request:** The IDE sends a `refactor.start` request to the server.
5.  **Server Responds:** The server initializes a new `RefactorState` in memory and sends the initial state back to the IDE.
6.  **IDE UI Updates:** The IDE uses the state to display progress. It then sends a `refactor.nextIteration` request.
7.  **Server Iterates:** The server runs one refactoring loop, updates the in-memory state, snapshots it to disk, and returns the new state.
8.  **Loop:** Steps 6 and 7 repeat, allowing the IDE to show real-time progress and give the user control over the process.
