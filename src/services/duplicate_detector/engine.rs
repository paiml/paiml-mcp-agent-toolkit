#![cfg_attr(coverage_nightly, coverage(off))]
//! Main duplicate detection engine orchestrating fragment extraction, LSH, and grouping.

use anyhow::Result;
use blake3::Hasher;
use dashmap::DashMap;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::minhash::MinHashGenerator;
use super::tokenizer::UniversalFeatureExtractor;
use super::types::{
    CloneGroup, CloneInstance, CloneReport, CloneSummary, CloneType, CodeFragment,
    DuplicateDetectionConfig, DuplicationHotspot, FragmentId, Language,
};

/// Main duplicate detection engine
pub struct DuplicateDetectionEngine {
    pub(super) feature_extractor: UniversalFeatureExtractor,
    pub(super) minhash_generator: MinHashGenerator,
    pub(super) config: DuplicateDetectionConfig,
    pub(super) fragments: DashMap<FragmentId, CodeFragment>,
    next_fragment_id: std::sync::atomic::AtomicU64,
}

impl DuplicateDetectionEngine {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(config: DuplicateDetectionConfig) -> Self {
        let minhash_generator = MinHashGenerator::new(config.num_hash_functions);
        let feature_extractor = UniversalFeatureExtractor::new(config.clone());

        Self {
            feature_extractor,
            minhash_generator,
            config,
            fragments: DashMap::new(),
            next_fragment_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Detect duplicates in a set of files
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn detect_duplicates(&self, files: &[(PathBuf, String, Language)]) -> Result<CloneReport> {
        // Phase 1: Extract fragments from all files
        let mut all_fragments = Vec::new();
        for (path, content, lang) in files {
            let fragments = self.extract_fragments(path, content, *lang)?;
            all_fragments.extend(fragments);
        }

        // Phase 2: Find similar fragments using MinHash
        let clone_pairs = self.find_clone_pairs(&all_fragments)?;

        // Phase 3: Group clones into clone groups
        let clone_groups = self.group_clones(clone_pairs)?;

        // Phase 4: Generate summary and hotspots
        let summary = self.compute_summary(&all_fragments, &clone_groups, files.len());
        let hotspots = self.compute_hotspots(&clone_groups);

        Ok(CloneReport {
            summary,
            groups: clone_groups,
            hotspots,
        })
    }

    /// Extract code fragments from a single file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn extract_fragments(
        &self,
        path: &Path,
        content: &str,
        lang: Language,
    ) -> Result<Vec<CodeFragment>> {
        let tokens = self.feature_extractor.extract_features(content, lang);
        let lines: Vec<&str> = content.lines().collect();
        let mut fragments = self.extract_function_fragments(path, &lines, lang)?;

        // If no functions found, treat entire file as one fragment
        if fragments.is_empty() && tokens.len() >= self.config.min_tokens {
            let fragment = self.create_fragment(path, content, tokens, 1, lines.len(), lang)?;
            fragments.push(fragment);
        }

        Ok(fragments)
    }

    /// Extract function-level fragments by detecting function boundaries
    fn extract_function_fragments(
        &self,
        path: &Path,
        lines: &[&str],
        lang: Language,
    ) -> Result<Vec<CodeFragment>> {
        let mut fragments = Vec::new();
        let mut current_function_start = None;

        for (line_idx, line) in lines.iter().enumerate() {
            let line = line.trim();
            if self.is_function_start(line, lang) {
                current_function_start = Some(line_idx);
            }
            if let Some(start_line) = current_function_start {
                if self.is_function_end(line, lang) && line_idx > start_line {
                    self.try_add_fragment(path, lines, start_line, line_idx, lang, &mut fragments)?;
                    current_function_start = None;
                }
            }
        }
        Ok(fragments)
    }

    /// Try to create and add a fragment if it meets the minimum token threshold
    fn try_add_fragment(
        &self,
        path: &Path,
        lines: &[&str],
        start_line: usize,
        end_line: usize,
        lang: Language,
        fragments: &mut Vec<CodeFragment>,
    ) -> Result<()> {
        let fragment_content = lines[start_line..=end_line].join("\n");
        let fragment_tokens = self
            .feature_extractor
            .extract_features(&fragment_content, lang);
        if fragment_tokens.len() >= self.config.min_tokens {
            let fragment = self.create_fragment(
                path,
                &fragment_content,
                fragment_tokens,
                start_line + 1,
                end_line + 1,
                lang,
            )?;
            fragments.push(fragment);
        }
        Ok(())
    }

    /// Create a code fragment
    fn create_fragment(
        &self,
        path: &Path,
        content: &str,
        tokens: Vec<super::types::Token>,
        start_line: usize,
        end_line: usize,
        lang: Language,
    ) -> Result<CodeFragment> {
        let id = self
            .next_fragment_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        // Generate shingles and signature
        let shingles = self
            .minhash_generator
            .generate_shingles(&tokens, self.config.shingle_size);
        let signature = self.minhash_generator.compute_signature(&shingles);

        // Compute normalized hash
        let mut hasher = Hasher::new();
        for token in &tokens {
            hasher.update(token.text.as_bytes());
        }
        let hash = u64::from_le_bytes(
            hasher.finalize().as_bytes()[0..8]
                .try_into()
                .expect("internal error"),
        );

        let fragment = CodeFragment {
            id,
            file_path: path.to_path_buf(),
            start_line,
            end_line,
            start_column: 1,
            end_column: 1,
            raw_content: content.to_string(),
            tokens: Vec::new(), // Save memory by not storing raw tokens
            normalized_tokens: tokens,
            signature,
            hash,
            language: lang,
        };

        self.fragments.insert(id, fragment.clone());
        Ok(fragment)
    }

    /// Check if line starts a function
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn is_function_start(&self, line: &str, lang: Language) -> bool {
        match lang {
            Language::Rust => line.contains("fn ") && line.contains('('),
            Language::TypeScript | Language::JavaScript => {
                line.contains("function ")
                    || line.contains("=> {")
                    || (line.contains('(') && line.contains(") {"))
            }
            Language::Python => line.starts_with("def ") && line.contains('('),
            Language::C | Language::Cpp => {
                // C/C++ function detection (simplified)
                line.contains('(') && (line.contains(") {") || line.ends_with('{'))
            }
            Language::Kotlin => line.contains("fun ") && line.contains('('),
        }
    }

    /// Check if line ends a function
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn is_function_end(&self, line: &str, lang: Language) -> bool {
        match lang {
            Language::Rust
            | Language::TypeScript
            | Language::JavaScript
            | Language::C
            | Language::Cpp
            | Language::Kotlin => line == "}",
            Language::Python => {
                // Python function ends when we reach another def or class at the same level
                line.starts_with("def ")
                    || line.starts_with("class ")
                    || (!line.starts_with(' ')
                        && !line.starts_with('\t')
                        && !line.trim().is_empty())
            }
        }
    }

    /// Find clone pairs using LSH for efficient similarity search
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn find_clone_pairs(
        &self,
        fragments: &[CodeFragment],
    ) -> Result<Vec<(FragmentId, FragmentId, f64)>> {
        let lsh_buckets = self.build_lsh_buckets(fragments);
        let candidate_pairs = Self::collect_candidate_pairs(&lsh_buckets);

        // Verify candidate pairs with exact similarity calculation
        let threshold = self.config.similarity_threshold;
        let clone_pairs: Vec<(FragmentId, FragmentId, f64)> = candidate_pairs
            .into_par_iter()
            .filter_map(|(i, j)| {
                let similarity = fragments[i]
                    .signature
                    .jaccard_similarity(&fragments[j].signature);
                (similarity >= threshold).then(|| (fragments[i].id, fragments[j].id, similarity))
            })
            .collect();

        Ok(clone_pairs)
    }

    /// Build LSH buckets by hashing each fragment's signature bands
    fn build_lsh_buckets(&self, fragments: &[CodeFragment]) -> Vec<HashMap<u64, Vec<usize>>> {
        let bands = self.config.num_bands;
        let rows_per_band = self.config.rows_per_band;
        let mut lsh_buckets: Vec<HashMap<u64, Vec<usize>>> = vec![HashMap::new(); bands];

        for (idx, fragment) in fragments.iter().enumerate() {
            for (band, bucket) in lsh_buckets.iter_mut().enumerate().take(bands) {
                let start = band * rows_per_band;
                let end = start + rows_per_band;
                let mut hasher = xxhash_rust::xxh64::Xxh64::new(band as u64);
                for i in start..end.min(fragment.signature.values.len()) {
                    hasher.update(&fragment.signature.values[i].to_le_bytes());
                }
                bucket.entry(hasher.digest()).or_default().push(idx);
            }
        }
        lsh_buckets
    }

    /// Collect candidate pairs from LSH buckets
    fn collect_candidate_pairs(
        lsh_buckets: &[HashMap<u64, Vec<usize>>],
    ) -> HashSet<(usize, usize)> {
        let mut candidate_pairs = HashSet::new();
        for band_buckets in lsh_buckets {
            for bucket in band_buckets.values().filter(|b| b.len() >= 2) {
                for i in 0..bucket.len() {
                    for j in (i + 1)..bucket.len() {
                        let pair = if bucket[i] < bucket[j] {
                            (bucket[i], bucket[j])
                        } else {
                            (bucket[j], bucket[i])
                        };
                        candidate_pairs.insert(pair);
                    }
                }
            }
        }
        candidate_pairs
    }

    /// Group similar fragments into clone groups
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn group_clones(
        &self,
        clone_pairs: Vec<(FragmentId, FragmentId, f64)>,
    ) -> Result<Vec<CloneGroup>> {
        // Use Union-Find for grouping
        let mut groups: HashMap<FragmentId, Vec<FragmentId>> = HashMap::new();
        let mut representative: HashMap<FragmentId, FragmentId> = HashMap::new();

        // Initialize each fragment as its own group
        for fragment in &self.fragments {
            let id = *fragment.key();
            representative.insert(id, id);
            groups.insert(id, vec![id]);
        }

        // Union fragments in clone pairs
        for (id1, id2, _similarity) in clone_pairs {
            let rep1 = Self::find_representative(&representative, id1);
            let rep2 = Self::find_representative(&representative, id2);

            if rep1 != rep2 {
                // Merge groups
                if let (Some(group1), Some(group2)) = (groups.remove(&rep1), groups.remove(&rep2)) {
                    let mut merged = group1;
                    merged.extend(group2);
                    groups.insert(rep1, merged);
                    representative.insert(rep2, rep1);
                }
            }
        }

        // Convert to CloneGroup format
        let mut clone_groups = Vec::new();
        let mut group_id = 1;

        for (rep_id, fragment_ids) in groups {
            if fragment_ids.len() >= self.config.min_group_size {
                let instances: Vec<CloneInstance> = fragment_ids
                    .iter()
                    .filter_map(|&id| self.fragments.get(&id))
                    .map(|frag| CloneInstance {
                        file: frag.file_path.clone(),
                        start_line: frag.start_line,
                        end_line: frag.end_line,
                        start_column: frag.start_column,
                        end_column: frag.end_column,
                        similarity_to_representative: 1.0, // Simplified
                        normalized_hash: frag.hash,
                    })
                    .collect();

                if !instances.is_empty() {
                    let total_lines = instances
                        .iter()
                        .map(|i| i.end_line - i.start_line + 1)
                        .sum();

                    let total_tokens = fragment_ids
                        .iter()
                        .filter_map(|&id| self.fragments.get(&id))
                        .map(|f| f.normalized_tokens.len())
                        .sum();

                    clone_groups.push(CloneGroup {
                        id: group_id,
                        clone_type: CloneType::Type2 {
                            similarity: self.config.similarity_threshold,
                            normalized: true,
                        },
                        fragments: instances,
                        total_lines,
                        total_tokens,
                        average_similarity: self.config.similarity_threshold,
                        representative: rep_id,
                    });

                    group_id += 1;
                }
            }
        }

        Ok(clone_groups)
    }

    /// Find representative in Union-Find structure
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn find_representative(
        representative: &HashMap<FragmentId, FragmentId>,
        id: FragmentId,
    ) -> FragmentId {
        if let Some(&rep) = representative.get(&id) {
            if rep == id {
                id
            } else {
                Self::find_representative(representative, rep)
            }
        } else {
            id
        }
    }

    /// Compute summary statistics
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub(crate) fn compute_summary(
        &self,
        fragments: &[CodeFragment],
        groups: &[CloneGroup],
        file_count: usize,
    ) -> CloneSummary {
        let duplicate_lines = groups.iter().map(|g| g.total_lines).sum();

        let total_lines = fragments
            .iter()
            .map(|f| f.end_line - f.start_line + 1)
            .sum();

        let duplication_ratio = if total_lines > 0 {
            duplicate_lines as f64 / total_lines as f64
        } else {
            0.0
        };

        let largest_group_size = groups.iter().map(|g| g.fragments.len()).max().unwrap_or(0);

        CloneSummary {
            total_files: file_count,
            total_fragments: fragments.len(),
            duplicate_lines,
            total_lines,
            duplication_ratio,
            clone_groups: groups.len(),
            largest_group_size,
        }
    }

    /// Compute duplication hotspots
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub(crate) fn compute_hotspots(&self, groups: &[CloneGroup]) -> Vec<DuplicationHotspot> {
        let mut file_stats: HashMap<PathBuf, (usize, HashSet<usize>)> = HashMap::new();

        for group in groups {
            for instance in &group.fragments {
                let (lines, group_ids) = file_stats
                    .entry(instance.file.clone())
                    .or_insert((0, HashSet::new()));
                *lines += instance.end_line - instance.start_line + 1;
                group_ids.insert(group.id as usize);
            }
        }

        let mut hotspots: Vec<DuplicationHotspot> = file_stats
            .into_iter()
            .map(|(file, (duplicate_lines, group_ids))| {
                let clone_groups = group_ids.len();
                let severity =
                    (duplicate_lines as f64).ln().max(1.0) * (clone_groups as f64).sqrt();
                DuplicationHotspot {
                    file,
                    duplicate_lines,
                    clone_groups,
                    severity,
                }
            })
            .collect();

        hotspots.sort_by(|a, b| b.severity.total_cmp(&a.severity));
        hotspots.truncate(10); // Top 10 hotspots
        hotspots
    }
}

impl Default for DuplicateDetectionEngine {
    fn default() -> Self {
        Self::new(DuplicateDetectionConfig::default())
    }
}
