#![cfg_attr(coverage_nightly, coverage(off))]
//! Data types and structures for the duplicate code detection system.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use xxhash_rust::xxh64::xxh64;

/// Language supported by the duplicate detection engine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    C,
    Cpp,
    Kotlin,
}

/// Types of code clones detected
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CloneType {
    /// Exact clones (modulo whitespace)
    Type1 { similarity: f64 },
    /// Parametric clones (identifiers/literals differ)
    Type2 { similarity: f64, normalized: bool },
    /// Structural clones (statements added/removed)
    Type3 { similarity: f64, ast_distance: f64 },
}

/// Token types for normalization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenKind {
    Identifier(String),
    Literal(String),
    Keyword(String),
    Operator(String),
    Delimiter(String),
    Comment,
    Whitespace,
}

/// Normalized token
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
}

impl Token {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(kind: TokenKind) -> Self {
        let text = match &kind {
            TokenKind::Identifier(s) => s.clone(),
            TokenKind::Literal(s) => s.clone(),
            TokenKind::Keyword(s) => s.clone(),
            TokenKind::Operator(s) => s.clone(),
            TokenKind::Delimiter(s) => s.clone(),
            TokenKind::Comment => "//".to_string(),
            TokenKind::Whitespace => " ".to_string(),
        };
        Self { kind, text }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    #[must_use]
    /// Hash.
    pub fn hash(&self) -> u64 {
        xxh64(self.text.as_bytes(), 0)
    }
}

/// `MinHash` signature for similarity estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinHashSignature {
    pub values: Vec<u64>,
}

impl MinHashSignature {
    /// Compute Jaccard similarity between two MinHash signatures
    ///
    /// # Performance
    ///
    /// - **SIMD-accelerated** (when `simd` feature enabled): Uses trueno vectorized comparison
    /// - **Scalar fallback**: Standard iterator-based comparison
    /// - **Typical speedup**: 4-8x on AVX2/AVX-512 CPUs (compares 8+ hashes in parallel)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let sig1 = MinHashSignature { values: vec![1, 2, 3] };
    /// let sig2 = MinHashSignature { values: vec![1, 5, 3] };
    /// let similarity = sig1.jaccard_similarity(&sig2); // 0.666... (2 matches out of 3)
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn jaccard_similarity(&self, other: &MinHashSignature) -> f64 {
        #[cfg(feature = "simd")]
        {
            self.jaccard_similarity_simd(other)
        }
        #[cfg(not(feature = "simd"))]
        {
            self.jaccard_similarity_scalar(other)
        }
    }

    /// SIMD-accelerated Jaccard similarity using trueno
    ///
    /// Uses vectorized comparison to count matching hash values in parallel
    #[cfg(feature = "simd")]
    #[must_use]
    fn jaccard_similarity_simd(&self, other: &MinHashSignature) -> f64 {
        use trueno::Vector;

        // Convert u64 hash values to f32 for SIMD operations
        // Comparison via subtraction: if (a - b) == 0.0, then a == b
        let self_f32: Vec<f32> = self.values.iter().map(|&v| v as f32).collect();
        let other_f32: Vec<f32> = other.values.iter().map(|&v| v as f32).collect();

        let v1 = Vector::from_slice(&self_f32);
        let v2 = Vector::from_slice(&other_f32);

        // SIMD-accelerated subtraction: diff[i] = v1[i] - v2[i]
        let diff = match v1.sub(&v2) {
            Ok(d) => d,
            Err(_) => {
                // Fallback to scalar if SIMD fails (shouldn't happen with equal-length vectors)
                return self.jaccard_similarity_scalar(other);
            }
        };

        // Count matches: diff[i] == 0.0 means hash values match
        // This is the only scalar operation after SIMD subtraction
        let matches = diff
            .as_slice()
            .iter()
            .filter(|&&val| val.abs() < 1e-6) // Use epsilon for floating-point comparison
            .count();

        matches as f64 / self.values.len() as f64
    }

    /// Scalar fallback for Jaccard similarity (used when simd feature disabled)
    #[must_use]
    fn jaccard_similarity_scalar(&self, other: &MinHashSignature) -> f64 {
        let matches = self
            .values
            .iter()
            .zip(&other.values)
            .filter(|(a, b)| a == b)
            .count();
        matches as f64 / self.values.len() as f64
    }
}

pub type FragmentId = u64;

/// Code fragment for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeFragment {
    pub id: FragmentId,
    pub file_path: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub raw_content: String,
    pub tokens: Vec<Token>,
    pub normalized_tokens: Vec<Token>,
    pub signature: MinHashSignature,
    pub hash: u64,
    pub language: Language,
}

/// Clone instance in a clone group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneInstance {
    pub file: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub similarity_to_representative: f64,
    pub normalized_hash: u64,
}

/// Group of similar code fragments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneGroup {
    pub id: u64,
    pub clone_type: CloneType,
    pub fragments: Vec<CloneInstance>,
    pub total_lines: usize,
    pub total_tokens: usize,
    pub average_similarity: f64,
    pub representative: FragmentId,
}

/// Summary of duplication analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneSummary {
    pub total_files: usize,
    pub total_fragments: usize,
    pub duplicate_lines: usize,
    pub total_lines: usize,
    pub duplication_ratio: f64,
    pub clone_groups: usize,
    pub largest_group_size: usize,
}

/// Duplication hotspot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicationHotspot {
    pub file: PathBuf,
    pub duplicate_lines: usize,
    pub clone_groups: usize,
    pub severity: f64,
}

/// Counted evidence of what the clone SEARCH did, so a blow-up is a number
/// rather than a hang.
///
/// #1059: `analyze duplicates` timed out on a 391-file corpus of transpiler
/// output while finishing a corpus 65x larger in 138s, because MinHash + LSH
/// prunes by BUCKETING and near-identical documents all land in the same
/// bucket. Wall-clock cannot be asserted on a shared runner, but these four
/// numbers can: on a saturated corpus `comparisons` used to be exactly
/// `fragments * (fragments - 1) / 2`, and a regression here shows up as a
/// count long before it shows up as a timeout.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloneSearchStats {
    /// Fragments handed to the search.
    pub fragments: usize,
    /// Fragments actually entered into the LSH search: one per class of
    /// byte-identical `MinHash` signature, plus every fragment the collapse
    /// could not apply to. Fragments sharing a signature are exact clones of
    /// one another and score identically against every third fragment, so one
    /// of each class stands for all of them.
    ///
    /// NOT "the number of distinct signatures": under a threshold above 1.0
    /// nothing may be collapsed, and this then equals `fragments` however few
    /// distinct signatures the corpus really has. It is the size of the set
    /// the quadratic stage runs over, which is the thing worth counting.
    pub searched_fragments: usize,
    /// Members of the largest single LSH band bucket. Equal to
    /// `searched_fragments` when banding has stopped discriminating at all,
    /// which is the #1059 corpus.
    pub max_bucket_occupancy: usize,
    /// Fragment pairs whose Jaccard similarity was actually computed. This is
    /// the quantity that used to grow with the square of the corpus.
    pub comparisons: u64,
}

/// Complete clone detection report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneReport {
    pub summary: CloneSummary,
    pub groups: Vec<CloneGroup>,
    pub hotspots: Vec<DuplicationHotspot>,
}

/// Configuration for duplicate detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateDetectionConfig {
    pub min_tokens: usize,
    pub similarity_threshold: f64,
    pub shingle_size: usize,
    pub num_hash_functions: usize,
    pub num_bands: usize,
    pub rows_per_band: usize,
    pub normalize_identifiers: bool,
    pub normalize_literals: bool,
    pub ignore_comments: bool,
    pub min_group_size: usize,
}

impl Default for DuplicateDetectionConfig {
    fn default() -> Self {
        Self {
            min_tokens: 50,
            similarity_threshold: 0.70,
            shingle_size: 5,
            num_hash_functions: 200,
            num_bands: 20,
            rows_per_band: 10,
            normalize_identifiers: true,
            normalize_literals: true,
            ignore_comments: true,
            min_group_size: 2,
        }
    }
}
