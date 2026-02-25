// Tests for semantic chunker
// Extracted for file health compliance (CB-040)
// Split into include files for file health compliance (CB-040)

use super::*;

mod tests {
    use super::*;

    #[test]
    fn test_checksum_computation() {
        let content = "fn test() {}";
        let checksum1 = compute_checksum(content);
        let checksum2 = compute_checksum(content);

        assert_eq!(checksum1, checksum2);
        assert_eq!(checksum1.len(), 64); // SHA256 hex length
    }

    #[test]
    fn test_language_enum() {
        assert_eq!(Language::Rust, Language::Rust);
        assert_ne!(Language::Rust, Language::Python);
    }

    #[test]
    fn test_chunk_type_enum() {
        assert_eq!(ChunkType::Function, ChunkType::Function);
        assert_ne!(ChunkType::Function, ChunkType::Class);
    }
}

/// Comprehensive coverage tests for the semantic chunker module
mod coverage_tests {
    use super::*;

    // --- Empty/edge case tests + Rust language tests ---
    include!("chunker_tests_empty_and_rust.rs");

    // --- TypeScript language tests ---
    include!("chunker_tests_typescript.rs");

    // --- Feature-gated language tests (Python, C, C++, Go) ---
    include!("chunker_tests_featuregated.rs");

    // --- Type/field/enum/struct tests + complex code ---
    include!("chunker_tests_types_fields.rs");

    // --- Edge cases, checksums, errors, doc comments, nesting, perf, parser ---
    include!("chunker_tests_edge_cases.rs");

    // --- TRUENO-RAG recursive chunker integration tests ---
    include!("chunker_tests_rag.rs");
}

/// Tests for extract_file_details and related helpers
mod extract_tests {
    use super::*;

    // --- File extract detail tests ---
    include!("chunker_tests_extract.rs");
}
