// Public API functions for WASM analysis

/// Main entry point for WASM analysis
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn analyze_wasm_module(binary: &[u8]) -> Result<Analysis> {
    debug_assert!(!binary.is_empty(), "binary must not be empty");
    let analyzer = WasmAnalyzer::new()?;
    analyzer.analyze_streaming(binary)
}

/// Verify WASM module safety properties
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn verify_wasm_safety(binary: &[u8]) -> Result<VerificationResult> {
    debug_assert!(!binary.is_empty(), "binary must not be empty");
    let verifier = IncrementalVerifier::new()?;
    verifier.verify_module(binary)
}

/// Profile WASM module performance
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn profile_wasm_module(binary: &[u8]) -> Result<ProfilingReport> {
    debug_assert!(!binary.is_empty(), "binary must not be empty");
    let profiler = AsyncProfiler::new();
    profiler.profile_module(binary).await
}
