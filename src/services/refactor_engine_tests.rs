// Tests for refactor engine
// Extracted for file health compliance (CB-040)

use super::*;

mod tests {
    use super::*;
    use std::path::PathBuf;

    include!("refactor_engine_tests_basic.rs");
}

mod coverage_tests {
    use super::*;
    use crate::services::cache::unified::UnifiedCacheConfig;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn create_test_engine(mode: EngineMode, targets: Vec<PathBuf>) -> UnifiedEngine {
        let ast_engine = Arc::new(UnifiedAstEngine::new());
        let cache = Arc::new(
            UnifiedCacheManager::new(UnifiedCacheConfig::default()).expect("create cache"),
        );
        let config = RefactorConfig::default();

        UnifiedEngine::new(ast_engine, cache, mode, config, targets)
    }

    // Type and sync tests (RingBuffer, ExplainLevel, Command, EngineError,
    // InteractiveState, ComplexityInfo, UnifiedEngine)
    include!("refactor_engine_tests_types.rs");

    // Async engine state-machine tests
    include!("refactor_engine_tests_async.rs");

    // Checkpoint save/load and analyze_incremental tests
    include!("refactor_engine_tests_checkpoint.rs");

    // should_emit, create_payload, and run_batch tests
    include!("refactor_engine_tests_emit.rs");

    // State export edge cases, rollback, JS/JSX/TSX, TDG,
    // engine metrics mutation, and StateInfo struct tests
    include!("refactor_engine_tests_state.rs");
}
