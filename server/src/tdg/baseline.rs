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
        improved.sort_by(|a, b| b.delta.partial_cmp(&a.delta).unwrap());
        regressed.sort_by(|a, b| a.delta.partial_cmp(&b.delta).unwrap());

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tdg::language_simple::Language;

    // Sprint 66 Phase 1 RED Tests
    // These tests define the expected behavior - implement to turn GREEN

    #[test]
    #[ignore] // RED: Turn GREEN by implementing TdgBaseline::new()
    fn test_create_baseline_empty() {
        let baseline = TdgBaseline::new(None);
        assert_eq!(baseline.files.len(), 0);
        assert_eq!(baseline.summary.total_files, 0);
        assert_eq!(baseline.summary.avg_score, 0.0);
    }

    #[test]
    #[ignore] // RED: Turn GREEN by implementing add_entry()
    fn test_add_entry_updates_summary() {
        let mut baseline = TdgBaseline::new(None);

        // Create mock entry
        let entry = BaselineEntry {
            content_hash: blake3::hash(b"test"),
            score: TdgScore {
                total: 95.0,
                grade: Grade::APLus,
                ..Default::default()
            },
            components: ComponentScores {
                complexity_breakdown: HashMap::new(),
                duplication_sources: Vec::new(),
                coupling_dependencies: Vec::new(),
                doc_missing_items: Vec::new(),
                consistency_violations: Vec::new(),
            },
            git_context: None,
        };

        baseline.add_entry(PathBuf::from("test.rs"), entry);

        assert_eq!(baseline.summary.total_files, 1);
        assert!((baseline.summary.avg_score - 95.0).abs() < 0.01);
    }

    #[test]
    #[ignore] // RED: Turn GREEN by implementing compare()
    fn test_compare_detects_improvements() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        // Old baseline: B grade
        old.add_entry(
            PathBuf::from("test.rs"),
            BaselineEntry {
                content_hash: blake3::hash(b"old"),
                score: TdgScore {
                    total: 80.0,
                    grade: Grade::B,
                    ..Default::default()
                },
                components: ComponentScores::default(),
                git_context: None,
            },
        );

        // New baseline: A grade (improved)
        new.add_entry(
            PathBuf::from("test.rs"),
            BaselineEntry {
                content_hash: blake3::hash(b"new"),
                score: TdgScore {
                    total: 90.0,
                    grade: Grade::A,
                    ..Default::default()
                },
                components: ComponentScores::default(),
                git_context: None,
            },
        );

        let comparison = old.compare(&new);
        assert_eq!(comparison.improved.len(), 1);
        assert_eq!(comparison.regressed.len(), 0);
        assert!(comparison.improved[0].delta > 0.0);
    }

    #[test]
    #[ignore] // RED: Turn GREEN by implementing compare()
    fn test_compare_detects_regressions() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        // Old baseline: A grade
        old.add_entry(
            PathBuf::from("test.rs"),
            BaselineEntry {
                content_hash: blake3::hash(b"old"),
                score: TdgScore {
                    total: 90.0,
                    grade: Grade::A,
                    ..Default::default()
                },
                components: ComponentScores::default(),
                git_context: None,
            },
        );

        // New baseline: B grade (regressed)
        new.add_entry(
            PathBuf::from("test.rs"),
            BaselineEntry {
                content_hash: blake3::hash(b"new"),
                score: TdgScore {
                    total: 80.0,
                    grade: Grade::B,
                    ..Default::default()
                },
                components: ComponentScores::default(),
                git_context: None,
            },
        );

        let comparison = old.compare(&new);
        assert_eq!(comparison.improved.len(), 0);
        assert_eq!(comparison.regressed.len(), 1);
        assert!(comparison.regressed[0].delta < 0.0);
    }

    #[test]
    #[ignore] // RED: Turn GREEN by implementing save/load
    fn test_baseline_serialization() {
        let mut baseline = TdgBaseline::new(None);
        baseline.add_entry(
            PathBuf::from("test.rs"),
            BaselineEntry {
                content_hash: blake3::hash(b"test"),
                score: TdgScore {
                    total: 95.0,
                    grade: Grade::APLus,
                    ..Default::default()
                },
                components: ComponentScores::default(),
                git_context: None,
            },
        );

        let temp_file = std::env::temp_dir().join("test_baseline.json");
        baseline.save(&temp_file).unwrap();

        let loaded = TdgBaseline::load(&temp_file).unwrap();
        assert_eq!(loaded.files.len(), baseline.files.len());
        assert!((loaded.summary.avg_score - baseline.summary.avg_score).abs() < 0.01);

        std::fs::remove_file(temp_file).unwrap();
    }

    #[test]
    #[ignore] // RED: Turn GREEN by implementing git context storage
    fn test_baseline_with_git_context() {
        let git_context = Some(GitContext {
            commit_sha: "abc123def456".to_string(),
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
        assert_eq!(
            baseline.git_context.unwrap().commit_sha,
            "abc123def456".to_string()
        );
    }

    #[test]
    #[ignore] // RED: Turn GREEN by implementing deduplication
    fn test_baseline_deduplication_by_hash() {
        let mut baseline = TdgBaseline::new(None);

        // Add same content with different paths
        let content_hash = blake3::hash(b"same content");

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

        // Both files tracked, but same hash means same content
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
    #[ignore] // RED: Turn GREEN by implementing grade distribution
    fn test_baseline_grade_distribution() {
        let mut baseline = TdgBaseline::new(None);

        // Add files with different grades
        for (path, grade, score) in [
            ("a.rs", Grade::APLus, 95.0),
            ("b.rs", Grade::A, 90.0),
            ("c.rs", Grade::A, 88.0),
            ("d.rs", Grade::B, 80.0),
        ] {
            baseline.add_entry(
                PathBuf::from(path),
                BaselineEntry {
                    content_hash: blake3::hash(path.as_bytes()),
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

        // Verify distribution
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
            2
        );
        assert_eq!(
            *baseline.summary.grade_distribution.get(&Grade::B).unwrap(),
            1
        );
    }

    #[test]
    #[ignore] // RED: Turn GREEN by implementing language distribution
    fn test_baseline_language_distribution() {
        let mut baseline = TdgBaseline::new(None);

        // Add files with different languages
        for (path, lang) in [
            ("test.rs", Language::Rust),
            ("test2.rs", Language::Rust),
            ("test.py", Language::Python),
        ] {
            baseline.add_entry(
                PathBuf::from(path),
                BaselineEntry {
                    content_hash: blake3::hash(path.as_bytes()),
                    score: TdgScore {
                        total: 90.0,
                        grade: Grade::A,
                        language: lang,
                        ..Default::default()
                    },
                    components: ComponentScores::default(),
                    git_context: None,
                },
            );
        }

        // Verify language counts
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

    #[test]
    #[ignore] // RED: Turn GREEN by implementing compare()
    fn test_compare_detects_added_files() {
        let old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        // New baseline has additional file
        new.add_entry(
            PathBuf::from("new_file.rs"),
            BaselineEntry {
                content_hash: blake3::hash(b"new content"),
                score: TdgScore {
                    total: 85.0,
                    grade: Grade::B,
                    ..Default::default()
                },
                components: ComponentScores::default(),
                git_context: None,
            },
        );

        let comparison = old.compare(&new);
        assert_eq!(comparison.added.len(), 1);
        assert_eq!(comparison.added[0], PathBuf::from("new_file.rs"));
    }

    #[test]
    #[ignore] // RED: Turn GREEN by implementing compare()
    fn test_compare_detects_removed_files() {
        let mut old = TdgBaseline::new(None);
        let new = TdgBaseline::new(None);

        // Old baseline has file that's no longer present
        old.add_entry(
            PathBuf::from("removed_file.rs"),
            BaselineEntry {
                content_hash: blake3::hash(b"old content"),
                score: TdgScore {
                    total: 85.0,
                    grade: Grade::B,
                    ..Default::default()
                },
                components: ComponentScores::default(),
                git_context: None,
            },
        );

        let comparison = old.compare(&new);
        assert_eq!(comparison.removed.len(), 1);
        assert_eq!(comparison.removed[0], PathBuf::from("removed_file.rs"));
    }

    #[test]
    #[ignore] // RED: Turn GREEN by implementing compare() sorting
    fn test_compare_sorts_by_delta() {
        let mut old = TdgBaseline::new(None);
        let mut new = TdgBaseline::new(None);

        // Add files with different improvement magnitudes
        for (path, old_score, new_score) in [
            ("small.rs", 80.0, 81.0),  // +1.0
            ("large.rs", 70.0, 85.0),  // +15.0
            ("medium.rs", 75.0, 80.0), // +5.0
        ] {
            old.add_entry(
                PathBuf::from(path),
                BaselineEntry {
                    content_hash: blake3::hash(path.as_bytes()),
                    score: TdgScore {
                        total: old_score,
                        grade: Grade::B,
                        ..Default::default()
                    },
                    components: ComponentScores::default(),
                    git_context: None,
                },
            );

            new.add_entry(
                PathBuf::from(path),
                BaselineEntry {
                    content_hash: blake3::hash(path.as_bytes()),
                    score: TdgScore {
                        total: new_score,
                        grade: Grade::B,
                        ..Default::default()
                    },
                    components: ComponentScores::default(),
                    git_context: None,
                },
            );
        }

        let comparison = old.compare(&new);

        // Should be sorted by delta magnitude (largest first)
        assert_eq!(comparison.improved.len(), 3);
        assert_eq!(comparison.improved[0].path, PathBuf::from("large.rs"));
        assert_eq!(comparison.improved[1].path, PathBuf::from("medium.rs"));
        assert_eq!(comparison.improved[2].path, PathBuf::from("small.rs"));
    }

    #[test]
    #[ignore] // RED: Turn GREEN by handling edge case
    fn test_baseline_empty_project() {
        let baseline = TdgBaseline::new(None);

        assert_eq!(baseline.summary.total_files, 0);
        assert_eq!(baseline.summary.avg_score, 0.0);
        assert!(baseline.summary.grade_distribution.is_empty());
        assert!(baseline.summary.languages.is_empty());
    }

    #[test]
    #[ignore] // RED: Turn GREEN by implementing efficient handling
    fn test_baseline_large_project() {
        let mut baseline = TdgBaseline::new(None);

        // Simulate large project with 1000+ files
        for i in 0..1500 {
            baseline.add_entry(
                PathBuf::from(format!("src/module{}/file.rs", i)),
                BaselineEntry {
                    content_hash: blake3::hash(format!("content{}", i).as_bytes()),
                    score: TdgScore {
                        total: 85.0 + (i % 10) as f32,
                        grade: Grade::B,
                        ..Default::default()
                    },
                    components: ComponentScores::default(),
                    git_context: None,
                },
            );
        }

        assert_eq!(baseline.summary.total_files, 1500);
        assert!(baseline.summary.avg_score > 80.0 && baseline.summary.avg_score < 95.0);
    }

    #[test]
    #[ignore] // RED: Turn GREEN by implementing error handling
    fn test_baseline_load_invalid_path() {
        let result = TdgBaseline::load("/nonexistent/path/baseline.json");
        assert!(result.is_err());
    }
}
