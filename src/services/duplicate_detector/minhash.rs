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
    pub fn new(num_hashes: usize) -> Self {
        debug_assert!(num_hashes > 0, "num_hashes must be positive");
        let seeds = (0..num_hashes).map(|i| i as u64).collect();

        Self { num_hashes, seeds }
    }

    /// Compute `MinHash` signature from shingles
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub fn compute_signature(&self, shingles: &[u64]) -> MinHashSignature {
        debug_assert!(!shingles.is_empty(), "shingles must not be empty");
        let mut signature = vec![u64::MAX; self.num_hashes];

        for &shingle in shingles {
            for (i, &seed) in self.seeds.iter().enumerate() {
                let hash = xxh64(&shingle.to_le_bytes(), seed);
                signature[i] = signature[i].min(hash);
            }
        }

        MinHashSignature { values: signature }
    }

    /// Generate k-shingles from tokens
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn generate_shingles(&self, tokens: &[Token], k: usize) -> Vec<u64> {
        debug_assert!(k > 0, "k must be positive");
        debug_assert!(!tokens.is_empty(), "tokens must not be empty");
        if tokens.len() < k {
            return vec![];
        }

        let mut shingles = Vec::new();
        let mut hasher = Hasher::new();

        for window in tokens.windows(k) {
            hasher.reset();
            for token in window {
                hasher.update(token.text.as_bytes());
            }
            let hash = hasher.finalize();
            shingles.push(u64::from_le_bytes(
                hash.as_bytes()[0..8].try_into().expect("internal error"),
            ));
        }

        shingles
    }
}
