// Tests for DAP server
// Extracted for file health compliance (CB-040)
// Split into include files for file health compliance (CB-040)

use super::*;

mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = DapServer::new();
        assert!(!server.is_initialized());
        assert!(!server.is_running());
        assert!(!server.has_program_loaded());
    }

    #[test]
    fn test_server_default() {
        let server = DapServer::default();
        assert!(!server.is_initialized());
    }

    #[test]
    fn test_next_seq_increments() {
        let server = DapServer::new();
        let seq1 = server.next_seq();
        let seq2 = server.next_seq();
        assert_eq!(seq2, seq1 + 1);
    }
}

mod coverage_tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::{tempdir, NamedTempFile};

    // Part 1: Server creation and state tests
    include!("server_tests_creation_state.rs");

    // Part 2: Request handling (invalid requests, initialize, launch, config, disconnect, terminate)
    include!("server_tests_request_handling.rs");

    // Part 3: Breakpoints, threads, stack trace, scopes, variables inspection
    include!("server_tests_breakpoints_inspection.rs");

    // Part 4: Execution control (continue, step) and language detection
    include!("server_tests_execution_language.rs");

    // Part 5: AST caching, stop simulation, variable inspection at line, recording
    include!("server_tests_ast_recording.rs");

    // Part 6: Default capabilities, full session lifecycle, edge cases
    include!("server_tests_lifecycle_edge.rs");
}
