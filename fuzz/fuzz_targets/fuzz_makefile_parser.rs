#![no_main]
use libfuzzer_sys::fuzz_target;
use paiml_mcp_agent_toolkit::services::makefile_linter::{MakefileParser, RuleRegistry};

fuzz_target!(|data: &[u8]| {
    // Convert bytes to string, skip if not valid UTF-8
    if let Ok(input) = std::str::from_utf8(data) {
        // Test parser - should never panic
        let mut parser = MakefileParser::new(input);
        if let Ok(ast) = parser.parse() {
            // Test linting - should never panic
            let registry = RuleRegistry::new();
            let _ = registry.check_all(&ast);
        }
    }
});