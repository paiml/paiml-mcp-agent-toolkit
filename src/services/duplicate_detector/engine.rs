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
        let lines: Vec<&str> = content.lines().collect();
        let mut fragments = self.extract_function_fragments(path, &lines, lang)?;

        // If no functions found, treat entire file as one fragment.
        //
        // The whole-file tokenization this needs used to run UNCONDITIONALLY,
        // one line above, while being read only inside this branch — so every
        // file that does contain functions was tokenized twice: once here and
        // discarded, once more per function by `try_add_fragment`. Tokenization
        // is 98.7% of `detect_duplicates` (measured: 7.33s of 7.43s over 2,647
        // files), so the discarded pass was most of the run. Deferring it into
        // the branch that reads it changes no fragment, no signature and no
        // ratio; it only stops paying for an answer nobody asked for.
        if fragments.is_empty() {
            let tokens = self.feature_extractor.extract_features(content, lang);
            if tokens.len() >= self.config.min_tokens {
                let fragment = self.create_fragment(path, content, tokens, 1, lines.len(), lang)?;
                fragments.push(fragment);
            }
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
        if matches!(lang, Language::Python) {
            return self.extract_indented_fragments(path, lines, lang);
        }
        self.extract_braced_fragments(path, lines, lang)
    }

    /// Fragments for brace languages, delimited by BRACE DEPTH.
    ///
    /// This used to end a function at the first line whose trimmed text was
    /// `"}"` — which is every nested block close, at any indentation. A function
    /// containing an `if` therefore produced a "fragment" that stopped at the
    /// end of that `if`, so the engine compared prefixes of functions rather
    /// than functions: two 20-line functions differing only in their last ten
    /// lines were byte-identical to the detector.
    fn extract_braced_fragments(
        &self,
        path: &Path,
        lines: &[&str],
        lang: Language,
    ) -> Result<Vec<CodeFragment>> {
        let mut fragments = Vec::new();
        let mut start: Option<usize> = None;
        let mut depth: i32 = 0;
        let mut opened = false;

        for (line_idx, line) in lines.iter().enumerate() {
            if start.is_none() {
                if !self.is_function_start(line.trim(), lang) {
                    continue;
                }
                start = Some(line_idx);
                depth = 0;
                opened = false;
            }

            for ch in line.chars() {
                match ch {
                    '{' => {
                        depth += 1;
                        opened = true;
                    }
                    '}' => depth -= 1,
                    _ => {}
                }
            }

            if let Some(start_line) = start {
                if opened && depth <= 0 && line_idx > start_line {
                    self.try_add_fragment(path, lines, start_line, line_idx, lang, &mut fragments)?;
                    start = None;
                }
            }
        }
        Ok(fragments)
    }

    /// Fragments for indentation-delimited languages (Python), where the end of
    /// a function is the next top-level statement.
    fn extract_indented_fragments(
        &self,
        path: &Path,
        lines: &[&str],
        lang: Language,
    ) -> Result<Vec<CodeFragment>> {
        let mut fragments = Vec::new();
        let mut current_function_start = None;

        for (line_idx, line) in lines.iter().enumerate() {
            if self.is_function_start(line.trim(), lang) {
                current_function_start = Some(line_idx);
            }
            if let Some(start_line) = current_function_start {
                // The RAW line, not the trimmed one: `is_function_end` for
                // Python asks whether the line is indented, and every line looks
                // unindented once it has been trimmed — so a Python function
                // ended on its own first body line.
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

    /// Measured similarity between two fragments, in `0.0..=1.0`.
    ///
    /// The MinHash signatures ARE the measurement: `jaccard_similarity` over two
    /// 200-value signatures is the same number `find_clone_pairs` thresholds on.
    /// `pair_similarity` is a fallback for callers that hand `group_clones`
    /// synthetic pairs whose fragments carry no signature (unit tests); an empty
    /// signature would otherwise divide by zero and produce NaN.
    fn measured_similarity(
        &self,
        a: FragmentId,
        b: FragmentId,
        pair_similarity: &HashMap<(FragmentId, FragmentId), f64>,
    ) -> f64 {
        if a == b {
            return 1.0;
        }

        if let (Some(fa), Some(fb)) = (self.fragments.get(&a), self.fragments.get(&b)) {
            if !fa.signature.values.is_empty() && !fb.signature.values.is_empty() {
                return fa.signature.jaccard_similarity(&fb.signature);
            }
        }

        let key = if a < b { (a, b) } else { (b, a) };
        pair_similarity.get(&key).copied().unwrap_or(0.0)
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

        // The similarity each pair was accepted with, kept so that the reported
        // numbers are the ones detection actually computed.
        let mut pair_similarity: HashMap<(FragmentId, FragmentId), f64> = HashMap::new();
        for &(id1, id2, similarity) in &clone_pairs {
            let key = if id1 < id2 { (id1, id2) } else { (id2, id1) };
            pair_similarity.insert(key, similarity);
        }

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

        // Convert to CloneGroup format.
        //
        // DETERMINISM: `groups` is a `HashMap` and the union order comes from a
        // `HashSet` of candidate pairs, so both the iteration order AND which
        // member ended up as the union-find root varied per process. The root is
        // the fragment every `similarity_to_representative` is measured against,
        // so an unstable root moved the reported numbers, not just their order.
        // The lowest fragment id in the group is a property of the input
        // (fragments are numbered in file order), so it is used instead.
        let mut ordered_groups: Vec<Vec<FragmentId>> = groups
            .into_values()
            .filter(|ids| ids.len() >= self.config.min_group_size)
            .map(|mut ids| {
                ids.sort_unstable();
                ids
            })
            .collect();
        ordered_groups.sort();

        let mut clone_groups = Vec::new();
        for fragment_ids in ordered_groups {
            let group_id = clone_groups.len() as u64 + 1;
            if let Some(group) = self.build_clone_group(&fragment_ids, group_id, &pair_similarity) {
                clone_groups.push(group);
            }
        }

        Ok(clone_groups)
    }

    /// Build one `CloneGroup` from the fragment ids the union-find put together.
    ///
    /// `fragment_ids` is sorted, so its first element is the representative and
    /// the instance order is fixed.
    fn build_clone_group(
        &self,
        fragment_ids: &[FragmentId],
        group_id: u64,
        pair_similarity: &HashMap<(FragmentId, FragmentId), f64>,
    ) -> Option<CloneGroup> {
        let rep_id = *fragment_ids.first()?;

        // MEASURED, not "Simplified": this was the literal 1.0 for every
        // instance of every group, so a Type-3 near-miss clone and a
        // byte-identical copy reported the same perfect similarity, and
        // `DuplicationDefectAnalyzer` printed "100% similar" for both.
        let instances: Vec<CloneInstance> = fragment_ids
            .iter()
            .filter_map(|&id| self.fragments.get(&id))
            .map(|frag| CloneInstance {
                file: frag.file_path.clone(),
                start_line: frag.start_line,
                end_line: frag.end_line,
                start_column: frag.start_column,
                end_column: frag.end_column,
                similarity_to_representative: self.measured_similarity(
                    frag.id,
                    rep_id,
                    pair_similarity,
                ),
                normalized_hash: frag.hash,
            })
            .collect();

        if instances.is_empty() {
            return None;
        }

        let total_lines = instances
            .iter()
            .map(|i| i.end_line - i.start_line + 1)
            .sum();

        let total_tokens = fragment_ids
            .iter()
            .filter_map(|&id| self.fragments.get(&id))
            .map(|f| f.normalized_tokens.len())
            .sum();

        // `average_similarity` and `clone_type`'s similarity used to be
        // `self.config.similarity_threshold` — the CUT-OFF the search used,
        // echoed back as if it were the answer. Every group in every report
        // therefore carried the same similarity (0.70 by default) whatever the
        // code looked like, and raising the threshold RAISED the reported
        // similarity of the surviving groups.
        // The representative's own 1.0 is excluded: it is a comparison of a
        // fragment with itself, and including it makes every pair look closer
        // than it is (a 0.90 pair would report 0.95).
        let copies: Vec<f64> = fragment_ids
            .iter()
            .filter(|&&id| id != rep_id)
            .map(|&id| self.measured_similarity(id, rep_id, pair_similarity))
            .collect();
        #[allow(clippy::cast_precision_loss)]
        let average_similarity = if copies.is_empty() {
            1.0
        } else {
            copies.iter().sum::<f64>() / copies.len() as f64
        };

        // An exact group (every member identical to the representative) is not a
        // near-miss; anything below 1.0 has statements added or removed, which
        // is a Type-3 clone.
        let clone_type = if average_similarity >= 1.0 {
            CloneType::Type2 {
                similarity: average_similarity,
                normalized: true,
            }
        } else {
            CloneType::Type3 {
                similarity: average_similarity,
                ast_distance: 1.0 - average_similarity,
            }
        };

        Some(CloneGroup {
            id: group_id,
            clone_type,
            fragments: instances,
            total_lines,
            total_tokens,
            average_similarity,
            representative: rep_id,
        })
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
