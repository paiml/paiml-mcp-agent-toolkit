//! #1059: `analyze duplicates` must not degrade superlinearly on a corpus of
//! near-identical files.
//!
//! MinHash + LSH prunes by BUCKETING, and a corpus of generated code — a
//! transpiler's `examples/`, a codegen crate's fixtures, a vendored SDK — puts
//! every fragment in the same bucket in every band. The pruning stops pruning
//! and the candidate set becomes every pair. Measured on the corpus in the
//! issue (391 files, 43,099 fragments): 23.5M candidate pairs materialised into
//! a `HashSet`, then 15.6M surviving pairs union-found with a recursive,
//! non-compressing `find`. The run took 162s where a corpus 65x larger took
//! 138s.
//!
//! These tests pin COUNTS, not seconds. Wall-clock on a shared runner is a coin
//! toss; `comparisons` is not. The blow-up had an exact arithmetic signature —
//! `fragments * (fragments - 1) / 2` — so a regression is visible as a number
//! long before anyone waits out a timeout.
//!
//! The speed half of the fix is worthless without the correctness half, so the
//! oracle test below is the load-bearing one: the fast path must return the
//! same clone groups as exhaustively comparing every bucket-mate pair. "Return
//! early and report nothing" is a 1000x speed-up and a total regression, and
//! [`fast_path_matches_the_exhaustive_answer`] is what tells the two apart.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use crate::services::duplicate_detector::{
        CodeFragment, DuplicateDetectionConfig, DuplicateDetectionEngine, FragmentId, Language,
    };
    use std::collections::{BTreeSet, HashMap};
    use std::path::PathBuf;

    /// A corpus as `detect_duplicates` takes it: path, source, language.
    type Corpus = Vec<(PathBuf, String, Language)>;

    /// Small signatures keep the tests quick in an unoptimised test build
    /// without changing the property under test: identical fragments still
    /// share every band, which is the whole point.
    fn test_config(threshold: f64) -> DuplicateDetectionConfig {
        DuplicateDetectionConfig {
            min_tokens: 20,
            similarity_threshold: threshold,
            shingle_size: 5,
            num_hash_functions: 40,
            num_bands: 4,
            rows_per_band: 10,
            normalize_identifiers: true,
            normalize_literals: true,
            ignore_comments: true,
            min_group_size: 2,
        }
    }

    /// One function whose body repeats an inner block `reps` times.
    ///
    /// Identifiers and literals are normalised away by the tokenizer, so two
    /// functions with the same `reps` are indistinguishable to the engine and
    /// two with different `reps` differ by exactly the statements that were
    /// added — a Type-3 near-miss, the case the similarity threshold exists to
    /// grade.
    fn function(file: usize, index: usize, reps: usize, family: usize) -> String {
        let mut body = String::new();
        for step in 0..reps {
            match family {
                0 => body.push_str(&format!(
                    "        acc = acc.wrapping_add(*value * {});\n        if acc > bound {{\n            acc = acc.wrapping_sub(*value / {});\n            seen += 1;\n        }}\n",
                    step + 2,
                    step + 2
                )),
                _ => body.push_str(&format!(
                    "        buffer.push_str(&segment[{}..]);\n        buffer.push('|');\n        if buffer.ends_with(\"||\") {{\n            buffer.pop();\n        }}\n",
                    step + 2
                )),
            }
        }
        match family {
            0 => format!(
                "pub fn alpha_{file}_{index}(input: &[i64], bound: i64) -> i64 {{\n    let mut acc: i64 = 0;\n    let mut seen: usize = 0;\n    for value in input.iter() {{\n{body}        if seen > 100 {{\n            break;\n        }}\n    }}\n    acc\n}}\n"
            ),
            _ => format!(
                "pub fn gamma_{file}_{index}(segment: &str, buffer: &mut String) {{\n    for _pass in 0..2 {{\n{body}    }}\n    buffer.shrink_to_fit();\n}}\n"
            ),
        }
    }

    /// `files` files of `per_file` functions that are all THE SAME function
    /// once identifiers and literals are normalised — the shape of transpiler
    /// output, and the shape that saturates every LSH band.
    fn near_identical_corpus(files: usize, per_file: usize) -> Corpus {
        (0..files)
            .map(|file| {
                let source: String = (0..per_file)
                    .map(|index| function(file, index, 6, 0))
                    .collect();
                (
                    PathBuf::from(format!("generated_{file:04}.rs")),
                    source,
                    Language::Rust,
                )
            })
            .collect()
    }

    /// Two clone families, each varying structurally within itself. The engine
    /// has to keep the families apart AND keep each family together, so this
    /// corpus fails both an over-merging bug and an under-merging one.
    fn two_family_corpus(files: usize, per_file: usize) -> Corpus {
        (0..files)
            .map(|file| {
                let source: String = (0..per_file)
                    .map(|index| {
                        let family = index % 2;
                        let reps = 6 + ((file + index / 2) % 4);
                        function(file, index, reps, family)
                    })
                    .collect();
                (
                    PathBuf::from(format!("mixed_{file:04}.rs")),
                    source,
                    Language::Rust,
                )
            })
            .collect()
    }

    fn extract_all(
        engine: &DuplicateDetectionEngine,
        files: &[(PathBuf, String, Language)],
    ) -> Vec<CodeFragment> {
        let mut fragments = Vec::new();
        for (path, content, lang) in files {
            fragments.extend(
                engine
                    .extract_fragments(path, content, *lang)
                    .expect("fragment extraction must succeed"),
            );
        }
        fragments
    }

    fn root(parent: &mut Vec<usize>, index: usize) -> usize {
        let mut node = index;
        while parent[node] != node {
            parent[node] = parent[parent[node]];
            node = parent[node];
        }
        node
    }

    fn merge(parent: &mut Vec<usize>, a: usize, b: usize) {
        let (ra, rb) = (root(parent, a), root(parent, b));
        if ra != rb {
            parent[rb] = ra;
        }
    }

    fn components(parent: &mut Vec<usize>, ids: &[FragmentId]) -> BTreeSet<BTreeSet<FragmentId>> {
        let mut by_root: HashMap<usize, BTreeSet<FragmentId>> = HashMap::new();
        for index in 0..ids.len() {
            let r = root(parent, index);
            by_root.entry(r).or_default().insert(ids[index]);
        }
        by_root
            .into_values()
            .filter(|group| group.len() >= 2)
            .collect()
    }

    /// The exhaustive answer: EVERY pair of LSH bucket-mates compared, and the
    /// clone groups read off as the connected components of the pairs that
    /// clear the threshold. This is precisely what the engine did before
    /// #1059, and it is quadratic on purpose — it is the oracle, not the
    /// implementation.
    fn exhaustive_groups(
        engine: &DuplicateDetectionEngine,
        fragments: &[CodeFragment],
        threshold: f64,
    ) -> BTreeSet<BTreeSet<FragmentId>> {
        let every = (0..fragments.len()).collect::<Vec<_>>();
        let buckets = engine.build_lsh_buckets(fragments, &every);
        let mut parent: Vec<usize> = (0..fragments.len()).collect();

        for band in &buckets {
            for bucket in band.values().filter(|b| b.len() >= 2) {
                for i in 0..bucket.len() {
                    for j in (i + 1)..bucket.len() {
                        let (a, b) = (bucket[i], bucket[j]);
                        let similarity = fragments[a]
                            .signature
                            .jaccard_similarity(&fragments[b].signature);
                        if similarity >= threshold {
                            merge(&mut parent, a, b);
                        }
                    }
                }
            }
        }

        let ids: Vec<FragmentId> = fragments.iter().map(|f| f.id).collect();
        components(&mut parent, &ids)
    }

    /// The groups the shipping fast path actually produces.
    fn fast_path_groups(
        engine: &DuplicateDetectionEngine,
        fragments: &[CodeFragment],
    ) -> BTreeSet<BTreeSet<FragmentId>> {
        let (pairs, _) = engine.find_clone_pairs_with_stats(fragments);
        let position: HashMap<FragmentId, usize> = fragments
            .iter()
            .enumerate()
            .map(|(index, fragment)| (fragment.id, index))
            .collect();
        let mut parent: Vec<usize> = (0..fragments.len()).collect();
        for (a, b, _) in &pairs {
            if let (Some(&ia), Some(&ib)) = (position.get(a), position.get(b)) {
                merge(&mut parent, ia, ib);
            }
        }
        let ids: Vec<FragmentId> = fragments.iter().map(|f| f.id).collect();
        components(&mut parent, &ids)
    }

    /// THE COUNTER-TEST. Speed that costs answers is not a fix.
    ///
    /// The fast path skips a comparison whenever the two fragments are already
    /// in one clone group. That is sound only because the clone groups are the
    /// CONNECTED COMPONENTS of the similarity graph, and an edge inside a
    /// component cannot change any component. This test is what holds that
    /// argument to account: over corpora that saturate LSH, corpora with two
    /// families, and thresholds that make the components split, the fast path
    /// must return the exact same set of groups — same count, same members —
    /// as comparing every bucket-mate pair.
    ///
    /// Without it, `find_clone_pairs_with_stats` returning an empty vector
    /// would pass every timing assertion in this file.
    #[test]
    fn fast_path_matches_the_exhaustive_answer() {
        let corpora: Vec<(&str, Corpus)> = vec![
            ("near-identical", near_identical_corpus(24, 4)),
            ("two-family", two_family_corpus(24, 4)),
            ("single-file", two_family_corpus(1, 8)),
        ];

        for (label, files) in &corpora {
            for threshold in [0.5_f64, 0.7, 0.8, 0.9, 0.95, 1.0] {
                let engine = DuplicateDetectionEngine::new(test_config(threshold));
                let fragments = extract_all(&engine, files);
                assert!(
                    !fragments.is_empty(),
                    "{label}: corpus produced no fragments, the test would be vacuous"
                );

                let expected = exhaustive_groups(&engine, &fragments, threshold);
                let actual = fast_path_groups(&engine, &fragments);

                assert_eq!(
                    expected, actual,
                    "{label} at threshold {threshold}: the fast path and the exhaustive \
                     all-pairs answer disagree about the clone groups"
                );
            }
        }
    }

    /// The exhaustive oracle must actually be exercised — a corpus that yields
    /// no groups at all would make [`fast_path_matches_the_exhaustive_answer`]
    /// agree about nothing. Absence rendered as success is the defect family
    /// this repo keeps closing, and a green oracle over an empty answer is an
    /// instance of it.
    #[test]
    fn the_oracle_finds_a_non_trivial_partition_to_compare() {
        let files = two_family_corpus(24, 4);
        let engine = DuplicateDetectionEngine::new(test_config(0.8));
        let fragments = extract_all(&engine, &files);
        let expected = exhaustive_groups(&engine, &fragments, 0.8);

        assert!(
            expected.len() >= 2,
            "the two-family corpus must produce at least two distinct clone groups \
             for the oracle comparison to mean anything, got {}",
            expected.len()
        );
        let members: usize = expected.iter().map(BTreeSet::len).sum();
        assert!(
            members >= fragments.len() / 2,
            "the oracle grouped only {members} of {} fragments",
            fragments.len()
        );
    }

    /// THE COMPLEXITY PIN.
    ///
    /// On a corpus where every fragment is a clone of every other, the number
    /// of similarity comparisons used to be exactly `n * (n - 1) / 2`: at 480
    /// fragments, 114,960 of them. It should be `n - 1` — a spanning tree of
    /// the one clone group — because every later pair joins two fragments
    /// already known to be in it.
    ///
    /// The bound asserted is `4n`, four times the spanning-tree ideal, so
    /// ordinary implementation drift does not trip it while a return to
    /// pairwise comparison misses it by three orders of magnitude.
    #[test]
    fn a_saturated_corpus_costs_a_linear_number_of_comparisons() {
        for files in [30_usize, 60, 120] {
            let engine = DuplicateDetectionEngine::new(test_config(0.7));
            let corpus = near_identical_corpus(files, 4);
            let fragments = extract_all(&engine, &corpus);
            let (_, stats) = engine.find_clone_pairs_with_stats(&fragments);

            let n = stats.fragments as u64;
            assert_eq!(
                n,
                (files * 4) as u64,
                "expected one fragment per generated function"
            );

            // The corpus really is the pathological one: LSH has stopped
            // discriminating. If this ever fails the test has drifted into
            // measuring an easy input and proves nothing.
            assert_eq!(
                stats.searched_fragments, 1,
                "every fragment should collapse onto one signature class; got {} — \
                 this is no longer the saturated corpus the test is about",
                stats.searched_fragments
            );

            let quadratic = n * (n - 1) / 2;
            assert!(
                stats.comparisons <= 4 * n,
                "{n} near-identical fragments took {} comparisons; the spanning-tree \
                 answer is {}, and pairwise would be {quadratic}",
                stats.comparisons,
                n - 1
            );
        }
    }

    /// The shape of the growth, not just its value at one point.
    ///
    /// Doubling a saturated corpus must roughly double the comparisons. Under
    /// the old candidate enumeration it quadrupled them, and that ratio is what
    /// a timeout at 391 files and a clean run at 40 actually was.
    #[test]
    fn doubling_a_saturated_corpus_does_not_square_the_work() {
        let mut measured = Vec::new();
        for files in [40_usize, 80, 160] {
            let engine = DuplicateDetectionEngine::new(test_config(0.7));
            let corpus = near_identical_corpus(files, 4);
            let fragments = extract_all(&engine, &corpus);
            let (_, stats) = engine.find_clone_pairs_with_stats(&fragments);
            measured.push((stats.fragments as u64, stats.comparisons));
        }

        for window in measured.windows(2) {
            let (small_n, small_c) = window[0];
            let (large_n, large_c) = window[1];
            assert_eq!(large_n, small_n * 2, "corpus sizes must actually double");
            // Quadratic would be 4x. Anything at or above 3x is the blow-up
            // coming back; linear is 2x.
            assert!(
                large_c < small_c.max(1) * 3,
                "{small_n} fragments cost {small_c} comparisons and {large_n} cost {large_c}: \
                 doubling the corpus more than tripled the work"
            );
        }
    }

    /// THE OTHER COUNTER-TEST: the detector must still report the duplication.
    ///
    /// This runs the whole engine, not just the pair search, so it also covers
    /// the grouping and the summary. A corpus that is 100% duplicated must come
    /// back as ONE group holding every fragment. An implementation that got
    /// fast by finding fewer clones fails here even though every count in this
    /// file would be smaller and every timing better.
    #[test]
    fn every_member_of_a_near_identical_corpus_is_still_reported() {
        let files = near_identical_corpus(60, 4);
        let engine = DuplicateDetectionEngine::new(test_config(0.7));
        let report = engine
            .detect_duplicates(&files)
            .expect("detection must succeed");

        assert_eq!(
            report.summary.total_fragments, 240,
            "the corpus should yield one fragment per generated function"
        );
        assert_eq!(
            report.groups.len(),
            1,
            "every fragment is a clone of every other, so there is exactly one group"
        );
        assert_eq!(
            report.summary.largest_group_size, 240,
            "the single clone group must hold every fragment, not a prefix of them"
        );
        assert!(
            report.summary.duplication_ratio > 0.99,
            "a corpus of identical functions must report near-total duplication, got {}",
            report.summary.duplication_ratio
        );

        // Every file must appear, so the answer cannot have been produced by
        // examining a truncated corpus.
        let seen: BTreeSet<PathBuf> = report
            .groups
            .iter()
            .flat_map(|group| group.fragments.iter().map(|f| f.file.clone()))
            .collect();
        assert_eq!(
            seen.len(),
            60,
            "all 60 files must be represented in the clone group"
        );
    }

    /// THE OVER-CORRECTION COUNTER-TEST: collapsing must not fuse what is
    /// merely bucket-adjacent.
    ///
    /// Fragments with byte-identical signatures are collapsed into one class
    /// because their similarity is 1.0 with each other and identical against
    /// every third fragment. If that reduction were applied too eagerly — say
    /// keyed on the LSH band rather than the whole signature — two unrelated
    /// clone families would merge into one group and the report would claim
    /// duplication that is not there.
    #[test]
    fn distinct_clone_families_are_not_merged() {
        let files = two_family_corpus(30, 4);
        let engine = DuplicateDetectionEngine::new(test_config(0.8));
        let report = engine
            .detect_duplicates(&files)
            .expect("detection must succeed");

        assert!(
            report.groups.len() >= 2,
            "two structurally different families must not collapse into one group, got {}",
            report.groups.len()
        );
        assert!(
            report.summary.largest_group_size < report.summary.total_fragments,
            "no single group may swallow the whole corpus: {} of {} fragments",
            report.summary.largest_group_size,
            report.summary.total_fragments
        );
    }

    /// A threshold above 1.0 is unreachable even for identical fragments, so
    /// nothing may be collapsed under one. This guards the one place the
    /// exact-signature reduction could have introduced clones that the
    /// threshold forbids.
    #[test]
    fn an_unreachable_threshold_reports_no_clones() {
        let files = near_identical_corpus(20, 4);
        let engine = DuplicateDetectionEngine::new(test_config(1.5));
        let fragments = extract_all(&engine, &files);
        let (pairs, stats) = engine.find_clone_pairs_with_stats(&fragments);

        assert!(
            pairs.is_empty(),
            "a similarity threshold of 1.5 cannot be met by any pair, got {} pairs",
            pairs.len()
        );
        assert_eq!(
            stats.searched_fragments, stats.fragments,
            "no fragment may be collapsed when the threshold is unreachable"
        );
    }

    /// The stats are the evidence this whole file rests on, so they have to
    /// describe the run rather than be plausible-looking constants.
    #[test]
    fn the_reported_stats_describe_the_corpus_they_measured() {
        let engine = DuplicateDetectionEngine::new(test_config(0.7));
        let files = near_identical_corpus(25, 4);
        let fragments = extract_all(&engine, &files);
        let (_, stats) = engine.find_clone_pairs_with_stats(&fragments);

        assert_eq!(stats.fragments, fragments.len());
        assert_eq!(
            stats.searched_fragments, 1,
            "the saturated corpus collapses onto exactly one signature class"
        );
        // One signature class means one leader, and a bucket needs two members
        // to be scanned at all — so LSH sees nothing to compare and every
        // union came from the collapse.
        assert_eq!(
            stats.max_bucket_occupancy, 0,
            "with a single leader no LSH bucket can hold two fragments"
        );
        assert_eq!(
            stats.comparisons, 0,
            "a corpus of one signature class needs no similarity comparison at all"
        );
    }
}
