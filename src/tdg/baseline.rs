/// Sprint 66 Phase 1: TDG Baseline System
///
/// Provides project-wide TDG quality tracking with content-hash based deduplication.
/// Enables regression detection and quality trend analysis.
use super::storage::ComponentScores;
use super::{Grade, TdgScore};
use crate::models::git_context::GitContext;
use anyhow::Result;
use blake3::Hash as Blake3Hash;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

include!("baseline_types.rs");
include!("baseline_impl.rs");
include!("baseline_comparison.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tdg::language_simple::Language;

    // Helper function to create a test baseline entry
    fn create_test_entry(score: f32, grade: Grade) -> BaselineEntry {
        BaselineEntry {
            content_hash: blake3::hash(format!("test_content_{}", score).as_bytes()),
            score: TdgScore {
                total: score,
                grade,
                ..Default::default()
            },
            components: ComponentScores::default(),
            git_context: None,
        }
    }

    // Helper function to create a test entry with specific language
    fn create_test_entry_with_lang(score: f32, grade: Grade, lang: Language) -> BaselineEntry {
        BaselineEntry {
            content_hash: blake3::hash(format!("test_content_{}_{:?}", score, lang).as_bytes()),
            score: TdgScore {
                total: score,
                grade,
                language: lang,
                ..Default::default()
            },
            components: ComponentScores::default(),
            git_context: None,
        }
    }

    include!("baseline_tests_core.rs");
    include!("baseline_tests_compare.rs");
    include!("baseline_tests_serialization.rs");
    include!("baseline_tests_comparison_api.rs");
    include!("baseline_tests_edge_cases.rs");
    include!("baseline_tests_grade_spelling.rs");
}
