#![cfg_attr(coverage_nightly, coverage(off))]
//! `MinHash` generator for computing similarity signatures from shingles.

use blake3::Hasher;
use xxhash_rust::xxh64::xxh64;

use super::types::{MinHashSignature, Token};

/// `MinHash` generator for similarity estimation
pub struct MinHashGenerator {
    pub(super) num_hashes: usize,
    pub(super) seeds: Vec<u64>,
}

impl MinHashGenerator {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(num_hashes: usize) -> Self {
        let seeds = (0..num_hashes).map(|i| i as u64).collect();

        Self { num_hashes, seeds }
    }

    /// Compute `MinHash` signature from shingles
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub fn compute_signature(&self, shingles: &[u64]) -> MinHashSignature {
        // Parallel over SEEDS, not over shingles: each signature slot is an
        // independent min-reduction, so the result is identical to the serial
        // loop for any thread count. This is the hot loop of the whole clone
        // engine (|shingles| x |seeds| hashes per fragment); serially it took
        // ~50s of the 76s an `analyze duplicates` run over a million lines
        // spends.
        use rayon::prelude::*;
        let values: Vec<u64> = self
            .seeds
            .par_iter()
            .map(|&seed| {
                shingles
                    .iter()
                    .map(|&shingle| xxh64(&shingle.to_le_bytes(), seed))
                    .min()
                    .unwrap_or(u64::MAX)
            })
            .collect();

        MinHashSignature { values }
    }

    /// Generate k-shingles from tokens.
    ///
    /// HOW MANY TIMES a shingle occurs is part of the shingle: the n-th
    /// repetition of a window hashes differently from the first. `MinHash`
    /// estimates the Jaccard similarity of SETS, so without this a function
    /// built from six copies of a block and one built from seven copies had
    /// exactly the same shingle set and measured 1.00 similar — the corpus's
    /// 40 branch-heavy files, which differ only in how many times the same
    /// block repeats, were all "identical" and no near-miss clone existed
    /// anywhere in a project designed to be full of them. Counting the
    /// repetition makes the same pair measure 6/7.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn generate_shingles(&self, tokens: &[Token], k: usize) -> Vec<u64> {
        if tokens.len() < k {
            return vec![];
        }

        let mut shingles = Vec::new();
        let mut hasher = Hasher::new();
        let mut seen: std::collections::HashMap<u64, u32> = std::collections::HashMap::new();

        for window in tokens.windows(k) {
            hasher.reset();
            for token in window {
                hasher.update(token.text.as_bytes());
            }
            let hash = u64::from_le_bytes(
                hasher.finalize().as_bytes()[0..8]
                    .try_into()
                    .expect("internal error"),
            );

            let occurrence = seen.entry(hash).or_insert(0);
            // The first occurrence keeps the plain hash, so a fragment with no
            // repeats shingles exactly as it did before.
            let shingle = if *occurrence == 0 {
                hash
            } else {
                xxh64(&hash.to_le_bytes(), u64::from(*occurrence))
            };
            *occurrence += 1;

            shingles.push(shingle);
        }

        shingles
    }
}
