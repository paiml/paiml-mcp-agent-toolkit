#![cfg_attr(coverage_nightly, coverage(off))]
//! Semantic file rename suggestions for `_part_` files (Issue #233).
//!
//! Analyzes FunctionEntry definitions within `_part_XX` files to suggest
//! meaningful names based on cascading signal priority.

use crate::services::agent_context::function_index::DefinitionType;
use crate::services::agent_context::{AgentContextIndex, FunctionEntry};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

// ── Types & constants ─────────────────────────────────────────────────────
include!("suggest_rename_types.rs");

// ── Public API ────────────────────────────────────────────────────────────
include!("suggest_rename_api.rs");

// ── Core analysis ─────────────────────────────────────────────────────────
include!("suggest_rename_analysis.rs");

// ── Signal analyzers ──────────────────────────────────────────────────────
include!("suggest_rename_signals.rs");

// ── Helpers ───────────────────────────────────────────────────────────────
include!("suggest_rename_helpers.rs");

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_context::function_index::DefinitionType;
    use crate::services::agent_context::{FunctionEntry, QualityMetrics};

    fn make_entry(name: &str, def_type: DefinitionType, doc: Option<&str>) -> FunctionEntry {
        FunctionEntry {
            file_path: "test/mod_part_01.rs".to_string(),
            function_name: name.to_string(),
            signature: format!("fn {name}()"),
            definition_type: def_type,
            doc_comment: doc.map(|d| d.to_string()),
            source: String::new(),
            start_line: 1,
            end_line: 10,
            language: "rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: String::new(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: vec![],
            linked_definition: None,
        }
    }

    #[test]
    fn test_is_part_file_positive() {
        // Underscore-separated: _part_NN
        assert!(is_part_file("src/llm/mod_part_02.rs"));
        assert!(is_part_file("src/llm/mod_part_02_part_04.rs"));
        assert!(is_part_file("foo_part_03_attn.rs"));
        assert!(is_part_file("deep/path/utils_part_01.rs"));
        // Digit-suffix: _partN (real-world pattern)
        assert!(is_part_file("quality_checks_part1.rs"));
        assert!(is_part_file("src/cli/quality_checks_part2.rs"));
        assert!(is_part_file("tests_part3.rs"));
    }

    #[test]
    fn test_is_part_file_negative() {
        assert!(!is_part_file("mod.rs"));
        assert!(!is_part_file("attention.rs"));
        assert!(!is_part_file("src/lib.rs"));
        assert!(!is_part_file("partial.rs"));
        assert!(!is_part_file("src/my_partition.rs"));
    }

    #[test]
    fn test_dominant_type_struct() {
        let entries = [
            make_entry("AttentionCache", DefinitionType::Struct, None),
            make_entry("new", DefinitionType::Function, None),
            make_entry("get", DefinitionType::Function, None),
        ];
        let refs: Vec<&FunctionEntry> = entries.iter().collect();
        let result = try_dominant_type(&refs);
        assert!(result.is_some());
        let (name, confidence, _) = result.unwrap();
        assert_eq!(name, "attention_cache");
        assert!((confidence - 0.95).abs() < 0.01);
    }

    #[test]
    fn test_dominant_type_enum() {
        let entries = [
            make_entry("TokenKind", DefinitionType::Enum, None),
            make_entry("from_str", DefinitionType::Function, None),
        ];
        let refs: Vec<&FunctionEntry> = entries.iter().collect();
        let result = try_dominant_type(&refs);
        assert!(result.is_some());
        let (name, confidence, _) = result.unwrap();
        assert_eq!(name, "token_kind");
        assert!((confidence - 0.95).abs() < 0.01);
    }

    #[test]
    fn test_function_theme_forward() {
        let entries = [
            make_entry("forward_pass", DefinitionType::Function, None),
            make_entry("forward_batch", DefinitionType::Function, None),
            make_entry("forward_single", DefinitionType::Function, None),
        ];
        let refs: Vec<&FunctionEntry> = entries.iter().collect();
        let result = try_function_theme(&refs);
        assert!(result.is_some());
        let (name, confidence, _) = result.unwrap();
        assert_eq!(name, "forward");
        assert!((confidence - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_common_prefix() {
        let entries = [
            make_entry("serialize_json", DefinitionType::Function, None),
            make_entry("serialize_yaml", DefinitionType::Function, None),
            make_entry("serialize_toml", DefinitionType::Function, None),
        ];
        let refs: Vec<&FunctionEntry> = entries.iter().collect();
        let result = try_common_prefix(&refs);
        assert!(result.is_some());
        let (name, confidence, _) = result.unwrap();
        assert_eq!(name, "serialize");
        assert!((confidence - 0.80).abs() < 0.01);
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("AttentionCache"), "attention_cache");
        assert_eq!(to_snake_case("RMSNorm"), "rms_norm");
        assert_eq!(to_snake_case("HTMLParser"), "html_parser");
        assert_eq!(to_snake_case("simple"), "simple");
        assert_eq!(to_snake_case("IOError"), "io_error");
    }

    #[test]
    fn test_collision_lowers_confidence() {
        // Build a minimal index with a file that would collide
        let index = AgentContextIndex {
            functions: vec![make_entry("attention", DefinitionType::Function, None)],
            name_index: HashMap::new(),
            file_index: {
                let mut m = HashMap::new();
                m.insert("src/attention.rs".to_string(), vec![0]);
                m.insert("src/mod_part_01.rs".to_string(), vec![]);
                m
            },
            corpus: vec![],
            corpus_lower: vec![],
            name_frequency: HashMap::new(),
            calls: HashMap::new(),
            called_by: HashMap::new(),
            graph_metrics: vec![],
            project_root: std::path::PathBuf::from("."),
            manifest: crate::services::agent_context::IndexManifest {
                version: "test".to_string(),
                built_at: String::new(),
                project_root: ".".to_string(),
                function_count: 1,
                file_count: 1,
                languages: vec![],
                avg_tdg_score: 0.0,
                tdg_scale: crate::services::agent_context::TDG_SCALE.to_string(),
                file_checksums: HashMap::new(),
                last_incremental_changes: 0,
            },
            db_path: None,
            coverage_off_files: std::collections::HashSet::new(),
        };

        assert!(check_collision("src/attention.rs", &index));
        assert!(!check_collision("src/cache.rs", &index));
    }

    #[test]
    fn test_generic_name_penalty() {
        let mut suggestions = vec![RenameSuggestion {
            current_path: "src/mod_part_01.rs".to_string(),
            suggested_name: "construction.rs".to_string(),
            suggested_path: "src/construction.rs".to_string(),
            confidence: 0.85,
            reasoning: "Function theme".to_string(),
            signal: RenameSignal::FunctionTheme,
            parent_file: None,
            inclusion_pattern: None,
            definition_count: 5,
        }];
        penalize_generic_names(&mut suggestions);
        assert!(
            suggestions[0].confidence < 0.60,
            "Generic name should be penalized: {}",
            suggestions[0].confidence
        );
        assert!(suggestions[0].reasoning.contains("[generic name penalty]"));
    }

    #[test]
    fn test_disambiguate_collisions() {
        let mut suggestions = vec![
            RenameSuggestion {
                current_path: "src/commands/run_part_03.rs".to_string(),
                suggested_name: "dispatch.rs".to_string(),
                suggested_path: "src/commands/dispatch.rs".to_string(),
                confidence: 0.85,
                reasoning: "Theme".to_string(),
                signal: RenameSignal::FunctionTheme,
                parent_file: None,
                inclusion_pattern: None,
                definition_count: 3,
            },
            RenameSuggestion {
                current_path: "src/commands/serve_part_02.rs".to_string(),
                suggested_name: "dispatch.rs".to_string(),
                suggested_path: "src/commands/dispatch.rs".to_string(),
                confidence: 0.85,
                reasoning: "Theme".to_string(),
                signal: RenameSignal::FunctionTheme,
                parent_file: None,
                inclusion_pattern: None,
                definition_count: 4,
            },
        ];
        disambiguate_collisions(&mut suggestions);

        // Both should now have unique names with numeric suffixes
        assert_ne!(suggestions[0].suggested_name, suggestions[1].suggested_name);
        assert!(
            suggestions[0].suggested_name.contains("dispatch_"),
            "got: {}",
            suggestions[0].suggested_name
        );
        // Both should be marked as disambiguated
        assert!(suggestions[0].reasoning.contains("[disambiguated]"));
        assert!(suggestions[1].reasoning.contains("[disambiguated]"));
        // Confidence should be reduced
        assert!(suggestions[0].confidence < 0.85);
    }

    #[test]
    fn test_strip_part_segments() {
        assert_eq!(strip_part_segments("mod_part_02"), "mod");
        assert_eq!(strip_part_segments("mod_part_02_part_04"), "mod");
        assert_eq!(strip_part_segments("utils_part_01_attn"), "utils_attn");
        assert_eq!(strip_part_segments("simple"), "simple");
    }

    #[test]
    fn test_collides_with_parent() {
        assert!(collides_with_parent(
            "src/mod.rs",
            &Some("src/mod.rs".to_string())
        ));
        assert!(!collides_with_parent(
            "src/attention.rs",
            &Some("src/mod.rs".to_string())
        ));
        assert!(!collides_with_parent("src/mod.rs", &None));
    }

    #[test]
    fn test_is_valid_module_name() {
        assert!(is_valid_module_name("attention_cache"));
        assert!(is_valid_module_name("forward"));
        assert!(is_valid_module_name("_private"));
        assert!(!is_valid_module_name(""));
        assert!(!is_valid_module_name("has-hyphen"));
        assert!(!is_valid_module_name("123numeric"));
        assert!(!is_valid_module_name("has space"));
    }

    #[test]
    fn test_matches_parent_dir() {
        assert!(matches_parent_dir("src/graph/mod_part_02.rs", "graph"));
        assert!(matches_parent_dir(
            "deep/path/cache/mod_part_01.rs",
            "cache"
        ));
        assert!(!matches_parent_dir("src/graph/mod_part_02.rs", "attention"));
        assert!(!matches_parent_dir("mod_part_02.rs", "graph"));
    }

    #[test]
    fn test_longest_common_prefix_unicode() {
        // Verify no panic on non-ASCII (CB-506 fix)
        assert_eq!(longest_common_prefix(&["café_a", "café_b"]), "café_");
        assert_eq!(longest_common_prefix(&["abc", "abd"]), "ab");
        assert_eq!(longest_common_prefix(&["xyz"]), "xyz");
        assert_eq!(longest_common_prefix(&[]), "");
    }

    #[test]
    fn test_original_base() {
        // Meaningful bases are preserved
        let result = try_original_base("src/cuda/activations_part_03.rs");
        assert!(result.is_some());
        let (name, confidence, _) = result.unwrap();
        assert_eq!(name, "activations");
        assert!((confidence - 0.82).abs() < 0.01);

        // Generic bases are rejected
        assert!(try_original_base("src/mod_part_02.rs").is_none());
        assert!(try_original_base("src/tests_part_03.rs").is_none());
        assert!(try_original_base("src/lib_part_02.rs").is_none());

        // Too-short bases rejected
        assert!(try_original_base("src/q4k_part_02.rs").is_none());

        // Accumulated _from_ artifacts rejected
        assert!(try_original_base("src/cache_from_cache_from_kv_part_02.rs").is_none());
        assert!(try_original_base("src/forward_from_model_part_02.rs").is_none());

        // mod_ prefix rejected
        assert!(try_original_base("src/mod_part_02_part_03_load.rs").is_none());

        // Too-long bases rejected
        assert!(
            try_original_base("src/very_long_name_that_exceeds_thirty_chars_part_02.rs").is_none()
        );
    }
}
