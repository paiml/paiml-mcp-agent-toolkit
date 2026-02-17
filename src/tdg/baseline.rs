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
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Complete TDG baseline for a project
///
/// Captures quality state of all files at a specific point in time.
/// Uses content hashing for efficient deduplication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TdgBaseline {
    /// PMAT version that created this baseline
    pub version: String,

    /// When this baseline was created
    pub created_at: DateTime<Utc>,

    /// Git context (commit, branch, author) when baseline was created
    pub git_context: Option<GitContext>,

    /// Per-file TDG scores indexed by file path
    pub files: HashMap<PathBuf, BaselineEntry>,

    /// Aggregate statistics across all files
    pub summary: BaselineSummary,
}

/// Single file entry in baseline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineEntry {
    /// Blake3 content hash for deduplication
    pub content_hash: Blake3Hash,

    /// TDG score for this file
    pub score: TdgScore,

    /// Component breakdown (complexity, duplication, etc.)
    pub components: ComponentScores,

    /// Git context when this file was analyzed
    pub git_context: Option<GitContext>,
}

/// Aggregate statistics for baseline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineSummary {
    /// Total number of files analyzed
    pub total_files: usize,

    /// Average TDG score across all files
    pub avg_score: f32,

    /// Distribution of grades (A+, A, B+, etc.)
    pub grade_distribution: HashMap<Grade, usize>,

    /// Count by programming language
    pub languages: HashMap<String, usize>,
}

/// Comparison result between two baselines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineComparison {
    /// Files with improved scores
    pub improved: Vec<FileComparison>,

    /// Files with regressed scores
    pub regressed: Vec<FileComparison>,

    /// Files with unchanged scores
    pub unchanged: Vec<PathBuf>,

    /// Files added since baseline
    pub added: Vec<PathBuf>,

    /// Files removed since baseline
    pub removed: Vec<PathBuf>,
}

/// Detailed comparison for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileComparison {
    /// File path
    pub path: PathBuf,

    /// Score in old baseline
    pub old_score: TdgScore,

    /// Score in new baseline
    pub new_score: TdgScore,

    /// Score delta (positive = improvement, negative = regression)
    pub delta: f32,

    /// Grade change (old, new)
    pub grade_change: (Grade, Grade),
}

impl TdgBaseline {
    /// Create empty baseline
    pub fn new(git_context: Option<GitContext>) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: Utc::now(),
            git_context,
            files: HashMap::new(),
            summary: BaselineSummary {
                total_files: 0,
                avg_score: 0.0,
                grade_distribution: HashMap::new(),
                languages: HashMap::new(),
            },
        }
    }

    /// Add file entry to baseline
    pub fn add_entry(&mut self, path: PathBuf, entry: BaselineEntry) {
        self.files.insert(path, entry);
        self.recompute_summary();
    }

    /// Recompute summary statistics
    fn recompute_summary(&mut self) {
        self.summary.total_files = self.files.len();

        if self.files.is_empty() {
            self.summary.avg_score = 0.0;
            return;
        }

        // Calculate average score
        let total: f32 = self.files.values().map(|e| e.score.total).sum();
        self.summary.avg_score = total / self.files.len() as f32;

        // Calculate grade distribution
        self.summary.grade_distribution.clear();
        for entry in self.files.values() {
            *self
                .summary
                .grade_distribution
                .entry(entry.score.grade)
                .or_insert(0) += 1;
        }

        // Calculate language distribution
        self.summary.languages.clear();
        for entry in self.files.values() {
            let lang = format!("{:?}", entry.score.language);
            *self.summary.languages.entry(lang).or_insert(0) += 1;
        }
    }

    /// Compare this baseline with another
    pub fn compare(&self, other: &TdgBaseline) -> BaselineComparison {
        let mut improved = Vec::new();
        let mut regressed = Vec::new();
        let mut unchanged = Vec::new();
        let mut added = Vec::new();
        let mut removed = Vec::new();

        // Check files in current baseline
        for (path, new_entry) in &other.files {
            if let Some(old_entry) = self.files.get(path) {
                let delta = new_entry.score.total - old_entry.score.total;

                if delta.abs() < 0.01 {
                    // Unchanged (within floating point tolerance)
                    unchanged.push(path.clone());
                } else if delta > 0.0 {
                    // Improved
                    improved.push(FileComparison {
                        path: path.clone(),
                        old_score: old_entry.score.clone(),
                        new_score: new_entry.score.clone(),
                        delta,
                        grade_change: (old_entry.score.grade, new_entry.score.grade),
                    });
                } else {
                    // Regressed
                    regressed.push(FileComparison {
                        path: path.clone(),
                        old_score: old_entry.score.clone(),
                        new_score: new_entry.score.clone(),
                        delta,
                        grade_change: (old_entry.score.grade, new_entry.score.grade),
                    });
                }
            } else {
                // File added
                added.push(path.clone());
            }
        }

        // Check for removed files
        for path in self.files.keys() {
            if !other.files.contains_key(path) {
                removed.push(path.clone());
            }
        }

        // Sort by delta magnitude
        improved.sort_by(|a, b| b.delta.total_cmp(&a.delta));
        regressed.sort_by(|a, b| a.delta.total_cmp(&b.delta));

        BaselineComparison {
            improved,
            regressed,
            unchanged,
            added,
            removed,
        }
    }

    /// Save baseline to JSON file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load baseline from JSON file
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let baseline = serde_json::from_str(&json)?;
        Ok(baseline)
    }
}

impl BaselineComparison {
    /// Check if there are any regressions
    pub fn has_regressions(&self) -> bool {
        !self.regressed.is_empty()
    }

    /// Get total number of changes
    pub fn total_changes(&self) -> usize {
        self.improved.len() + self.regressed.len() + self.added.len() + self.removed.len()
    }

    /// Format comparison as human-readable text
    pub fn format_text(&self) -> String {
        let mut output = String::new();

        if !self.improved.is_empty() {
            output.push_str(&format!("✅ Improved: {} files\n", self.improved.len()));
            for cmp in &self.improved {
                output.push_str(&format!(
                    "   - {}: {:?} ({:.1}) → {:?} ({:.1}) [+{:.1}]\n",
                    cmp.path.display(),
                    cmp.grade_change.0,
                    cmp.old_score.total,
                    cmp.grade_change.1,
                    cmp.new_score.total,
                    cmp.delta
                ));
            }
        }

        if !self.regressed.is_empty() {
            output.push_str(&format!("⚠️  Regressed: {} files\n", self.regressed.len()));
            for cmp in &self.regressed {
                output.push_str(&format!(
                    "   - {}: {:?} ({:.1}) → {:?} ({:.1}) [{:.1}]\n",
                    cmp.path.display(),
                    cmp.grade_change.0,
                    cmp.old_score.total,
                    cmp.grade_change.1,
                    cmp.new_score.total,
                    cmp.delta
                ));
            }
        }

        if !self.unchanged.is_empty() {
            output.push_str(&format!("➡️  Unchanged: {} files\n", self.unchanged.len()));
        }

        if !self.added.is_empty() {
            output.push_str(&format!("➕ Added: {} files\n", self.added.len()));
            for path in &self.added {
                output.push_str(&format!("   - {}\n", path.display()));
            }
        }

        if !self.removed.is_empty() {
            output.push_str(&format!("➖ Removed: {} files\n", self.removed.len()));
            for path in &self.removed {
                output.push_str(&format!("   - {}\n", path.display()));
            }
        }

        output
    }
}

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

    // ========== TdgBaseline::new() Tests ==========

    #[test]
    fn test_create_baseline_empty() {
        let baseline = TdgBaseline::new(None);
        assert_eq!(baseline.files.len(), 0);
        assert_eq!(baseline.summary.total_files, 0);
        assert_eq!(baseline.summary.avg_score, 0.0);
    }

    #[test]
    fn test_create_baseline_with_git_context() {
        let git_context = Some(GitContext {
            commit_sha: "abc123def456789012345678901234567890abcd".to_string(),
            commit_sha_short: "abc123d".to_string(),
            branch: "master".to_string(),
            author_name: "Test Author".to_string(),
            author_email: "test@example.com".to_string(),
            commit_timestamp: Utc::now(),
            commit_message: "Test commit".to_string(),
            tags: vec![],
            parent_commits: vec![],
            remote_url: None,
            is_clean: true,
            uncommitted_files: 0,
        });

        let baseline = TdgBaseline::new(git_context.clone());
        assert!(baseline.git_context.is_some());
        let ctx = baseline.git_context.unwrap();
        assert_eq!(ctx.commit_sha, "abc123def456789012345678901234567890abcd");
        assert_eq!(ctx.branch, "master");
        assert_eq!(ctx.author_name, "Test Author");
    }

    #[test]
    fn test_baseline_version_is_set() {
        let baseline = TdgBaseline::new(None);
        assert!(!baseline.version.is_empty());
        // Version should be a semantic version from Cargo.toml
        assert!(baseline.version.contains('.'));
    }

    #[test]
    fn test_baseline_created_at_is_recent() {
        let before = Utc::now();
        let baseline = TdgBaseline::new(None);
        let after = Utc::now();

        assert!(baseline.created_at >= before);
        assert!(baseline.created_at <= after);
    }

    // ========== TdgBaseline::add_entry() Tests ==========

    #[test]
    fn test_add_entry_updates_summary() {
        let mut baseline = TdgBaseline::new(None);
        let entry = create_test_entry(95.0, Grade::APLus);

        baseline.add_entry(PathBuf::from("test.rs"), entry);

        assert_eq!(baseline.summary.total_files, 1);
        assert!((baseline.summary.avg_score - 95.0).abs() < 0.01);
    }

    #[test]
    fn test_add_multiple_entries_updates_average() {
        let mut baseline = TdgBaseline::new(None);

        baseline.add_entry(PathBuf::from("a.rs"), create_test_entry(90.0, Grade::A));
        baseline.add_entry(PathBuf::from("b.rs"), create_test_entry(80.0, Grade::BPlus));
        baseline.add_entry(
            PathBuf::from("c.rs"),
            create_test_entry(70.0, Grade::BMinus),
        );

        assert_eq!(baseline.summary.total_files, 3);
        // Average: (90 + 80 + 70) / 3 = 80.0
        assert!((baseline.summary.avg_score - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_add_entry_overwrites_existing_path() {
        let mut baseline = TdgBaseline::new(None);

        baseline.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(80.0, Grade::BPlus),
        );
        baseline.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(95.0, Grade::APLus),
        );

        assert_eq!(baseline.summary.total_files, 1);
        assert!((baseline.summary.avg_score - 95.0).abs() < 0.01);
    }

    #[test]
    fn test_add_entry_updates_grade_distribution() {
        let mut baseline = TdgBaseline::new(None);

        baseline.add_entry(PathBuf::from("a.rs"), create_test_entry(95.0, Grade::APLus));
        baseline.add_entry(PathBuf::from("b.rs"), create_test_entry(90.0, Grade::A));
        baseline.add_entry(
            PathBuf::from("c.rs"),
            create_test_entry(88.0, Grade::AMinus),
        );
        baseline.add_entry(PathBuf::from("d.rs"), create_test_entry(75.0, Grade::B));

        assert_eq!(
            *baseline
                .summary
                .grade_distribution
                .get(&Grade::APLus)
                .unwrap(),
            1
        );
        assert_eq!(
            *baseline.summary.grade_distribution.get(&Grade::A).unwrap(),
            1
        );
        assert_eq!(
            *baseline
                .summary
                .grade_distribution
                .get(&Grade::AMinus)
                .unwrap(),
            1
        );
        assert_eq!(
            *baseline.summary.grade_distribution.get(&Grade::B).unwrap(),
            1
        );
    }

    #[test]
    fn test_add_entry_updates_language_distribution() {
        let mut baseline = TdgBaseline::new(None);

        baseline.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry_with_lang(90.0, Grade::A, Language::Rust),
        );
        baseline.add_entry(
            PathBuf::from("test2.rs"),
            create_test_entry_with_lang(85.0, Grade::AMinus, Language::Rust),
        );
        baseline.add_entry(
            PathBuf::from("test.py"),
            create_test_entry_with_lang(80.0, Grade::BPlus, Language::Python),
        );

        assert_eq!(
            *baseline
                .summary
                .languages
                .get(&format!("{:?}", Language::Rust))
                .unwrap(),
            2
        );
        assert_eq!(
            *baseline
                .summary
                .languages
                .get(&format!("{:?}", Language::Python))
                .unwrap(),
            1
        );
    }

    // ========== TdgBaseline::compare() Tests ==========

    #[test]
    fn test_compare_detects_improvements() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        old.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(80.0, Grade::BPlus),
        );
        new.add_entry(PathBuf::from("test.rs"), create_test_entry(90.0, Grade::A));

        let comparison = old.compare(&new);
        assert_eq!(comparison.improved.len(), 1);
        assert_eq!(comparison.regressed.len(), 0);
        assert!((comparison.improved[0].delta - 10.0).abs() < 0.01);
        assert_eq!(
            comparison.improved[0].grade_change,
            (Grade::BPlus, Grade::A)
        );
    }

    #[test]
    fn test_compare_detects_regressions() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        old.add_entry(PathBuf::from("test.rs"), create_test_entry(90.0, Grade::A));
        new.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(80.0, Grade::BPlus),
        );

        let comparison = old.compare(&new);
        assert_eq!(comparison.improved.len(), 0);
        assert_eq!(comparison.regressed.len(), 1);
        assert!((comparison.regressed[0].delta + 10.0).abs() < 0.01);
        assert_eq!(
            comparison.regressed[0].grade_change,
            (Grade::A, Grade::BPlus)
        );
    }

    #[test]
    fn test_compare_detects_unchanged_files() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        old.add_entry(PathBuf::from("test.rs"), create_test_entry(90.0, Grade::A));
        new.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(90.005, Grade::A),
        ); // Within tolerance

        let comparison = old.compare(&new);
        assert_eq!(comparison.unchanged.len(), 1);
        assert_eq!(comparison.improved.len(), 0);
        assert_eq!(comparison.regressed.len(), 0);
    }

    #[test]
    fn test_compare_detects_added_files() {
        let old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        new.add_entry(
            PathBuf::from("new_file.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );

        let comparison = old.compare(&new);
        assert_eq!(comparison.added.len(), 1);
        assert_eq!(comparison.added[0], PathBuf::from("new_file.rs"));
    }

    #[test]
    fn test_compare_detects_removed_files() {
        let mut old = TdgBaseline::new(None);
        let new = TdgBaseline::new(None);

        old.add_entry(
            PathBuf::from("removed_file.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );

        let comparison = old.compare(&new);
        assert_eq!(comparison.removed.len(), 1);
        assert_eq!(comparison.removed[0], PathBuf::from("removed_file.rs"));
    }

    #[test]
    fn test_compare_sorts_improvements_by_delta_descending() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        // Add files with different improvement magnitudes
        old.add_entry(
            PathBuf::from("small.rs"),
            create_test_entry(80.0, Grade::BPlus),
        );
        old.add_entry(
            PathBuf::from("large.rs"),
            create_test_entry(70.0, Grade::BMinus),
        );
        old.add_entry(
            PathBuf::from("medium.rs"),
            create_test_entry(75.0, Grade::B),
        );

        new.add_entry(
            PathBuf::from("small.rs"),
            create_test_entry(81.0, Grade::BPlus),
        ); // +1.0
        new.add_entry(
            PathBuf::from("large.rs"),
            create_test_entry(85.0, Grade::AMinus),
        ); // +15.0
        new.add_entry(
            PathBuf::from("medium.rs"),
            create_test_entry(80.0, Grade::BPlus),
        ); // +5.0

        let comparison = old.compare(&new);

        assert_eq!(comparison.improved.len(), 3);
        assert_eq!(comparison.improved[0].path, PathBuf::from("large.rs"));
        assert_eq!(comparison.improved[1].path, PathBuf::from("medium.rs"));
        assert_eq!(comparison.improved[2].path, PathBuf::from("small.rs"));
    }

    #[test]
    fn test_compare_sorts_regressions_by_delta_ascending() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        old.add_entry(
            PathBuf::from("small.rs"),
            create_test_entry(80.0, Grade::BPlus),
        );
        old.add_entry(PathBuf::from("large.rs"), create_test_entry(90.0, Grade::A));
        old.add_entry(
            PathBuf::from("medium.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );

        new.add_entry(PathBuf::from("small.rs"), create_test_entry(79.0, Grade::B)); // -1.0
        new.add_entry(PathBuf::from("large.rs"), create_test_entry(75.0, Grade::B)); // -15.0
        new.add_entry(
            PathBuf::from("medium.rs"),
            create_test_entry(80.0, Grade::BPlus),
        ); // -5.0

        let comparison = old.compare(&new);

        assert_eq!(comparison.regressed.len(), 3);
        assert_eq!(comparison.regressed[0].path, PathBuf::from("large.rs"));
        assert_eq!(comparison.regressed[1].path, PathBuf::from("medium.rs"));
        assert_eq!(comparison.regressed[2].path, PathBuf::from("small.rs"));
    }

    #[test]
    fn test_compare_mixed_changes() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        // File that improves
        old.add_entry(
            PathBuf::from("improved.rs"),
            create_test_entry(70.0, Grade::BMinus),
        );
        new.add_entry(
            PathBuf::from("improved.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );

        // File that regresses
        old.add_entry(
            PathBuf::from("regressed.rs"),
            create_test_entry(90.0, Grade::A),
        );
        new.add_entry(
            PathBuf::from("regressed.rs"),
            create_test_entry(75.0, Grade::B),
        );

        // File that stays the same
        old.add_entry(
            PathBuf::from("unchanged.rs"),
            create_test_entry(80.0, Grade::BPlus),
        );
        new.add_entry(
            PathBuf::from("unchanged.rs"),
            create_test_entry(80.0, Grade::BPlus),
        );

        // File that was removed
        old.add_entry(
            PathBuf::from("removed.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );

        // File that was added
        new.add_entry(
            PathBuf::from("added.rs"),
            create_test_entry(88.0, Grade::AMinus),
        );

        let comparison = old.compare(&new);

        assert_eq!(comparison.improved.len(), 1);
        assert_eq!(comparison.regressed.len(), 1);
        assert_eq!(comparison.unchanged.len(), 1);
        assert_eq!(comparison.added.len(), 1);
        assert_eq!(comparison.removed.len(), 1);
    }

    // ========== TdgBaseline::save() and load() Tests ==========

    #[test]
    fn test_baseline_serialization_roundtrip() {
        let mut baseline = TdgBaseline::new(None);
        baseline.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(95.0, Grade::APLus),
        );
        baseline.add_entry(
            PathBuf::from("test2.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );

        let temp_file = std::env::temp_dir().join(format!(
            "test_baseline_roundtrip_{}.json",
            std::process::id()
        ));
        baseline.save(&temp_file).expect("Failed to save baseline");

        let loaded = TdgBaseline::load(&temp_file).expect("Failed to load baseline");

        assert_eq!(loaded.files.len(), baseline.files.len());
        assert!((loaded.summary.avg_score - baseline.summary.avg_score).abs() < 0.01);
        assert_eq!(loaded.summary.total_files, baseline.summary.total_files);

        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_baseline_load_invalid_path() {
        let result = TdgBaseline::load("/nonexistent/path/baseline.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_baseline_save_to_nested_path() {
        let temp_dir = std::env::temp_dir().join(format!("baseline_test_{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        let baseline = TdgBaseline::new(None);
        let temp_file = temp_dir.join("nested/deep/baseline.json");

        // This should fail because parent directories don't exist
        let result = baseline.save(&temp_file);
        assert!(result.is_err());

        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_baseline_serialization_with_git_context() {
        let git_context = Some(GitContext {
            commit_sha: "1234567890abcdef1234567890abcdef12345678".to_string(),
            commit_sha_short: "1234567".to_string(),
            branch: "feature/test".to_string(),
            author_name: "Test Developer".to_string(),
            author_email: "dev@test.com".to_string(),
            commit_timestamp: Utc::now(),
            commit_message: "Test serialization".to_string(),
            tags: vec!["v1.0.0".to_string()],
            parent_commits: vec!["abcdef1234567890abcdef1234567890abcdef12".to_string()],
            remote_url: Some("https://github.com/test/repo.git".to_string()),
            is_clean: false,
            uncommitted_files: 3,
        });

        let mut baseline = TdgBaseline::new(git_context);
        baseline.add_entry(PathBuf::from("test.rs"), create_test_entry(90.0, Grade::A));

        let temp_file =
            std::env::temp_dir().join(format!("test_baseline_git_{}.json", std::process::id()));
        baseline.save(&temp_file).expect("Failed to save baseline");

        let loaded = TdgBaseline::load(&temp_file).expect("Failed to load baseline");

        assert!(loaded.git_context.is_some());
        let ctx = loaded.git_context.unwrap();
        assert_eq!(ctx.branch, "feature/test");
        assert_eq!(ctx.tags, vec!["v1.0.0"]);
        assert!(!ctx.is_clean);
        assert_eq!(ctx.uncommitted_files, 3);

        std::fs::remove_file(temp_file).ok();
    }

    // ========== BaselineEntry Tests ==========

    #[test]
    fn test_baseline_entry_content_hash_deduplication() {
        let mut baseline = TdgBaseline::new(None);
        let content_hash = blake3::hash(b"identical content");

        let entry1 = BaselineEntry {
            content_hash,
            score: TdgScore {
                total: 90.0,
                grade: Grade::A,
                ..Default::default()
            },
            components: ComponentScores::default(),
            git_context: None,
        };

        let entry2 = BaselineEntry {
            content_hash,
            score: TdgScore {
                total: 90.0,
                grade: Grade::A,
                ..Default::default()
            },
            components: ComponentScores::default(),
            git_context: None,
        };

        baseline.add_entry(PathBuf::from("file1.rs"), entry1);
        baseline.add_entry(PathBuf::from("file2.rs"), entry2);

        assert_eq!(baseline.files.len(), 2);
        assert_eq!(
            baseline
                .files
                .get(&PathBuf::from("file1.rs"))
                .unwrap()
                .content_hash,
            baseline
                .files
                .get(&PathBuf::from("file2.rs"))
                .unwrap()
                .content_hash
        );
    }

    #[test]
    fn test_baseline_entry_with_components() {
        let mut components = ComponentScores::default();
        components
            .complexity_breakdown
            .insert("cyclomatic".to_string(), 5.0);
        components
            .duplication_sources
            .push("line 10-20".to_string());
        components
            .coupling_dependencies
            .push("module_a".to_string());
        components
            .doc_missing_items
            .push("fn process()".to_string());
        components
            .consistency_violations
            .push("naming_style".to_string());

        let entry = BaselineEntry {
            content_hash: blake3::hash(b"test"),
            score: TdgScore {
                total: 85.0,
                grade: Grade::AMinus,
                ..Default::default()
            },
            components,
            git_context: None,
        };

        let mut baseline = TdgBaseline::new(None);
        baseline.add_entry(PathBuf::from("test.rs"), entry);

        let retrieved = baseline.files.get(&PathBuf::from("test.rs")).unwrap();
        assert_eq!(retrieved.components.complexity_breakdown.len(), 1);
        assert_eq!(retrieved.components.duplication_sources.len(), 1);
        assert_eq!(retrieved.components.coupling_dependencies.len(), 1);
        assert_eq!(retrieved.components.doc_missing_items.len(), 1);
        assert_eq!(retrieved.components.consistency_violations.len(), 1);
    }

    // ========== BaselineComparison Tests ==========

    #[test]
    fn test_comparison_has_regressions() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        old.add_entry(PathBuf::from("test.rs"), create_test_entry(90.0, Grade::A));
        new.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(80.0, Grade::BPlus),
        );

        let comparison = old.compare(&new);
        assert!(comparison.has_regressions());
    }

    #[test]
    fn test_comparison_no_regressions() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        old.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(80.0, Grade::BPlus),
        );
        new.add_entry(PathBuf::from("test.rs"), create_test_entry(90.0, Grade::A));

        let comparison = old.compare(&new);
        assert!(!comparison.has_regressions());
    }

    #[test]
    fn test_comparison_total_changes() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        old.add_entry(
            PathBuf::from("improved.rs"),
            create_test_entry(70.0, Grade::BMinus),
        );
        old.add_entry(
            PathBuf::from("regressed.rs"),
            create_test_entry(90.0, Grade::A),
        );
        old.add_entry(
            PathBuf::from("removed.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );

        new.add_entry(
            PathBuf::from("improved.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );
        new.add_entry(
            PathBuf::from("regressed.rs"),
            create_test_entry(75.0, Grade::B),
        );
        new.add_entry(
            PathBuf::from("added.rs"),
            create_test_entry(88.0, Grade::AMinus),
        );

        let comparison = old.compare(&new);
        // 1 improved + 1 regressed + 1 added + 1 removed = 4 changes
        assert_eq!(comparison.total_changes(), 4);
    }

    #[test]
    fn test_comparison_format_text_improvements() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        old.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(70.0, Grade::BMinus),
        );
        new.add_entry(PathBuf::from("test.rs"), create_test_entry(90.0, Grade::A));

        let comparison = old.compare(&new);
        let text = comparison.format_text();

        assert!(text.contains("Improved: 1 files"));
        assert!(text.contains("test.rs"));
    }

    #[test]
    fn test_comparison_format_text_regressions() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        old.add_entry(PathBuf::from("test.rs"), create_test_entry(90.0, Grade::A));
        new.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(70.0, Grade::BMinus),
        );

        let comparison = old.compare(&new);
        let text = comparison.format_text();

        assert!(text.contains("Regressed: 1 files"));
        assert!(text.contains("test.rs"));
    }

    #[test]
    fn test_comparison_format_text_added_removed() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        old.add_entry(
            PathBuf::from("removed.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );
        new.add_entry(
            PathBuf::from("added.rs"),
            create_test_entry(88.0, Grade::AMinus),
        );

        let comparison = old.compare(&new);
        let text = comparison.format_text();

        assert!(text.contains("Added: 1 files"));
        assert!(text.contains("added.rs"));
        assert!(text.contains("Removed: 1 files"));
        assert!(text.contains("removed.rs"));
    }

    #[test]
    fn test_comparison_format_text_unchanged() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        old.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );
        new.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );

        let comparison = old.compare(&new);
        let text = comparison.format_text();

        assert!(text.contains("Unchanged: 1 files"));
    }

    // ========== Edge Cases and Boundary Tests ==========

    #[test]
    fn test_baseline_empty_project() {
        let baseline = TdgBaseline::new(None);

        assert_eq!(baseline.summary.total_files, 0);
        assert_eq!(baseline.summary.avg_score, 0.0);
        assert!(baseline.summary.grade_distribution.is_empty());
        assert!(baseline.summary.languages.is_empty());
    }

    #[test]
    fn test_baseline_single_file() {
        let mut baseline = TdgBaseline::new(None);
        baseline.add_entry(
            PathBuf::from("single.rs"),
            create_test_entry(92.5, Grade::A),
        );

        assert_eq!(baseline.summary.total_files, 1);
        assert!((baseline.summary.avg_score - 92.5).abs() < 0.01);
        assert_eq!(
            *baseline.summary.grade_distribution.get(&Grade::A).unwrap(),
            1
        );
    }

    #[test]
    fn test_baseline_large_project() {
        let mut baseline = TdgBaseline::new(None);

        for i in 0..500 {
            let score = 70.0 + (i % 30) as f32;
            let grade = Grade::from_score(score);
            baseline.add_entry(
                PathBuf::from(format!("src/module{}/file.rs", i)),
                BaselineEntry {
                    content_hash: blake3::hash(format!("content{}", i).as_bytes()),
                    score: TdgScore {
                        total: score,
                        grade,
                        ..Default::default()
                    },
                    components: ComponentScores::default(),
                    git_context: None,
                },
            );
        }

        assert_eq!(baseline.summary.total_files, 500);
        // Average should be around 84.5 ((70+99)/2 roughly, accounting for modulo distribution)
        assert!(baseline.summary.avg_score > 80.0 && baseline.summary.avg_score < 90.0);
    }

    #[test]
    fn test_compare_empty_baselines() {
        let old = TdgBaseline::new(None);
        let new = TdgBaseline::new(None);

        let comparison = old.compare(&new);

        assert!(comparison.improved.is_empty());
        assert!(comparison.regressed.is_empty());
        assert!(comparison.unchanged.is_empty());
        assert!(comparison.added.is_empty());
        assert!(comparison.removed.is_empty());
        assert_eq!(comparison.total_changes(), 0);
        assert!(!comparison.has_regressions());
    }

    #[test]
    fn test_compare_with_floating_point_tolerance() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        // Delta of 0.005 should be considered unchanged (within 0.01 tolerance)
        old.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );
        new.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(85.005, Grade::AMinus),
        );

        let comparison = old.compare(&new);
        assert_eq!(comparison.unchanged.len(), 1);
        assert!(comparison.improved.is_empty());
        assert!(comparison.regressed.is_empty());
    }

    #[test]
    fn test_compare_at_tolerance_boundary() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        // Delta of 0.02 should be detected as improvement (above 0.01 tolerance)
        old.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(85.0, Grade::AMinus),
        );
        new.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(85.02, Grade::AMinus),
        );

        let comparison = old.compare(&new);
        assert_eq!(comparison.improved.len(), 1);
        assert!(comparison.unchanged.is_empty());
    }

    #[test]
    fn test_file_comparison_delta_calculation() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        old.add_entry(PathBuf::from("test.rs"), create_test_entry(75.5, Grade::B));
        new.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry(88.3, Grade::AMinus),
        );

        let comparison = old.compare(&new);
        assert_eq!(comparison.improved.len(), 1);

        let file_cmp = &comparison.improved[0];
        assert_eq!(file_cmp.path, PathBuf::from("test.rs"));
        assert!((file_cmp.delta - 12.8).abs() < 0.01);
        assert!((file_cmp.old_score.total - 75.5).abs() < 0.01);
        assert!((file_cmp.new_score.total - 88.3).abs() < 0.01);
    }

    // ========== Summary Recomputation Tests ==========

    #[test]
    fn test_summary_recomputes_on_each_add() {
        let mut baseline = TdgBaseline::new(None);

        baseline.add_entry(PathBuf::from("a.rs"), create_test_entry(80.0, Grade::BPlus));
        assert_eq!(baseline.summary.total_files, 1);
        assert!((baseline.summary.avg_score - 80.0).abs() < 0.01);

        baseline.add_entry(PathBuf::from("b.rs"), create_test_entry(90.0, Grade::A));
        assert_eq!(baseline.summary.total_files, 2);
        assert!((baseline.summary.avg_score - 85.0).abs() < 0.01);

        baseline.add_entry(
            PathBuf::from("c.rs"),
            create_test_entry(70.0, Grade::BMinus),
        );
        assert_eq!(baseline.summary.total_files, 3);
        assert!((baseline.summary.avg_score - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_summary_grade_distribution_clears_on_recompute() {
        let mut baseline = TdgBaseline::new(None);

        baseline.add_entry(PathBuf::from("a.rs"), create_test_entry(95.0, Grade::APLus));
        assert_eq!(
            *baseline
                .summary
                .grade_distribution
                .get(&Grade::APLus)
                .unwrap(),
            1
        );

        // Overwrite with different grade
        baseline.add_entry(PathBuf::from("a.rs"), create_test_entry(75.0, Grade::B));
        assert!(baseline
            .summary
            .grade_distribution
            .get(&Grade::APLus)
            .is_none());
        assert_eq!(
            *baseline.summary.grade_distribution.get(&Grade::B).unwrap(),
            1
        );
    }

    #[test]
    fn test_summary_language_distribution_clears_on_recompute() {
        let mut baseline = TdgBaseline::new(None);

        baseline.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry_with_lang(90.0, Grade::A, Language::Rust),
        );
        assert_eq!(
            *baseline
                .summary
                .languages
                .get(&format!("{:?}", Language::Rust))
                .unwrap(),
            1
        );

        // Overwrite with different language
        baseline.add_entry(
            PathBuf::from("test.rs"),
            create_test_entry_with_lang(90.0, Grade::A, Language::Python),
        );
        assert!(baseline
            .summary
            .languages
            .get(&format!("{:?}", Language::Rust))
            .is_none());
        assert_eq!(
            *baseline
                .summary
                .languages
                .get(&format!("{:?}", Language::Python))
                .unwrap(),
            1
        );
    }

    // ========== recompute_summary empty files branch ==========

    #[test]
    fn test_recompute_summary_empty_files() {
        let mut baseline = TdgBaseline::new(None);
        // Add an entry so summary has non-zero values
        baseline.add_entry(PathBuf::from("test.rs"), create_test_entry(90.0, Grade::A));
        assert_eq!(baseline.summary.total_files, 1);
        assert!((baseline.summary.avg_score - 90.0).abs() < 0.01);

        // Clear files directly (pub field) and call recompute
        baseline.files.clear();
        baseline.recompute_summary();

        assert_eq!(baseline.summary.total_files, 0);
        assert_eq!(baseline.summary.avg_score, 0.0);
    }

    // ========== All Grade Distribution Tests ==========

    #[test]
    fn test_all_grades_in_distribution() {
        let mut baseline = TdgBaseline::new(None);

        let grades_and_scores = [
            ("a.rs", Grade::APLus, 96.0),
            ("b.rs", Grade::A, 92.0),
            ("c.rs", Grade::AMinus, 87.0),
            ("d.rs", Grade::BPlus, 82.0),
            ("e.rs", Grade::B, 77.0),
            ("f.rs", Grade::BMinus, 72.0),
            ("g.rs", Grade::CPlus, 67.0),
            ("h.rs", Grade::C, 62.0),
            ("i.rs", Grade::CMinus, 57.0),
            ("j.rs", Grade::D, 52.0),
            ("k.rs", Grade::F, 40.0),
        ];

        for (path, grade, score) in grades_and_scores {
            baseline.add_entry(PathBuf::from(path), create_test_entry(score, grade));
        }

        assert_eq!(baseline.summary.total_files, 11);
        for (_, grade, _) in grades_and_scores {
            assert_eq!(
                *baseline
                    .summary
                    .grade_distribution
                    .get(&grade)
                    .unwrap_or(&0),
                1,
                "Grade {:?} should have count 1",
                grade
            );
        }
    }
}
