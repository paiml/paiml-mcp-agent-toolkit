#![cfg_attr(coverage_nightly, coverage(off))]
//! Locality-Sensitive Hashing (LSH) index for approximate nearest-neighbor lookup.

use blake3::Hasher;
use std::collections::{HashMap, HashSet};

use super::types::{FragmentId, MinHashSignature};

// TRUENO-RAG-4-MINHASH: LSH Index for O(1) Lookup
// Locality-Sensitive Hashing for approximate nearest neighbor

/// Locality-Sensitive Hashing (LSH) index for MinHash signatures
///
/// This index enables O(1) approximate nearest neighbor lookup for code fragments,
/// replacing O(n) pairwise comparison with bucket-based candidate retrieval.
///
/// # Algorithm
///
/// 1. **Band partitioning**: Divide MinHash signature into `b` bands of `r` rows each
/// 2. **Band hashing**: Hash each band to a bucket
/// 3. **Candidate retrieval**: Signatures sharing any bucket are candidates
///
/// # Probability of Collision
///
/// For Jaccard similarity `s`, probability of becoming candidates:
/// `P = 1 - (1 - s^r)^b`
///
/// With `b=20` bands and `r=5` rows:
/// - s=0.9 → P≈1.0 (high similarity → almost always candidates)
/// - s=0.5 → P≈0.97 (medium similarity → usually candidates)
/// - s=0.2 → P≈0.04 (low similarity → rarely candidates)
#[derive(Debug, Clone)]
pub struct LshIndex {
    /// Number of bands to divide signature into
    bands: usize,
    /// Number of rows per band
    rows_per_band: usize,
    /// Buckets: band_index → (band_hash → fragment_ids)
    buckets: Vec<HashMap<u64, Vec<FragmentId>>>,
    /// Stored signatures for similarity computation
    signatures: HashMap<FragmentId, MinHashSignature>,
}

impl LshIndex {
    /// Create a new LSH index
    ///
    /// # Arguments
    /// * `num_bands` - Number of bands to partition signature into
    /// * `rows_per_band` - Number of rows (hash values) per band
    ///
    /// # Optimal Parameters (for 100-hash signatures)
    /// * `(20, 5)` - High recall for similarity > 0.5
    /// * `(10, 10)` - Higher precision for similarity > 0.8
    /// * `(25, 4)` - Maximum recall for similarity > 0.3
    #[must_use]
    pub fn new(num_bands: usize, rows_per_band: usize) -> Self {
        debug_assert!(num_bands > 0, "num_bands must be positive");
        let buckets = (0..num_bands).map(|_| HashMap::new()).collect();
        Self {
            bands: num_bands,
            rows_per_band,
            buckets,
            signatures: HashMap::new(),
        }
    }

    /// Insert a fragment's MinHash signature into the index
    ///
    /// # Arguments
    /// * `fragment_id` - Unique identifier for the code fragment
    /// * `signature` - MinHash signature of the fragment
    pub fn insert(&mut self, fragment_id: FragmentId, signature: MinHashSignature) {
        // Store the signature for later similarity computation
        self.signatures.insert(fragment_id, signature.clone());

        // Hash each band and add to corresponding bucket
        for (band_idx, band) in signature.values.chunks(self.rows_per_band).enumerate() {
            if band_idx >= self.bands {
                break;
            }

            let band_hash = self.hash_band(band);
            self.buckets[band_idx]
                .entry(band_hash)
                .or_default()
                .push(fragment_id);
        }
    }

    /// Find candidate similar fragments for a query signature
    ///
    /// # Arguments
    /// * `query` - MinHash signature to query
    ///
    /// # Returns
    /// Set of fragment IDs that are candidate matches (O(1) average)
    #[must_use]
    pub fn query(&self, query: &MinHashSignature) -> HashSet<FragmentId> {
        let mut candidates = HashSet::new();

        for (band_idx, band) in query.values.chunks(self.rows_per_band).enumerate() {
            if band_idx >= self.bands {
                break;
            }

            let band_hash = self.hash_band(band);
            if let Some(bucket) = self.buckets[band_idx].get(&band_hash) {
                candidates.extend(bucket.iter().copied());
            }
        }

        candidates
    }

    /// Find similar fragments with their computed Jaccard similarities
    ///
    /// # Arguments
    /// * `query` - MinHash signature to query
    /// * `threshold` - Minimum Jaccard similarity threshold
    ///
    /// # Returns
    /// Vector of (fragment_id, similarity) pairs above threshold
    #[must_use]
    pub fn find_similar(&self, query: &MinHashSignature, threshold: f64) -> Vec<(FragmentId, f64)> {
        let candidates = self.query(query);

        candidates
            .into_iter()
            .filter_map(|id| {
                self.signatures.get(&id).map(|sig| {
                    let similarity = query.jaccard_similarity(sig);
                    (id, similarity)
                })
            })
            .filter(|(_, sim)| *sim >= threshold)
            .collect()
    }

    /// Get the number of fragments in the index
    #[must_use]
    pub fn len(&self) -> usize {
        self.signatures.len()
    }

    /// Check if the index is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.signatures.is_empty()
    }

    /// Get a signature by fragment ID
    #[must_use]
    pub fn get_signature(&self, fragment_id: FragmentId) -> Option<&MinHashSignature> {
        self.signatures.get(&fragment_id)
    }

    /// Hash a band of MinHash values
    fn hash_band(&self, band: &[u64]) -> u64 {
        let mut hasher = Hasher::new();
        for &val in band {
            hasher.update(&val.to_le_bytes());
        }
        let hash_bytes = hasher.finalize();
        u64::from_le_bytes(
            hash_bytes.as_bytes()[0..8]
                .try_into()
                .expect("blake3 hash always has at least 8 bytes"),
        )
    }

    /// Calculate the probability of collision for a given Jaccard similarity
    ///
    /// P = 1 - (1 - s^r)^b where:
    /// - s = Jaccard similarity
    /// - r = rows per band
    /// - b = number of bands
    #[must_use]
    pub fn collision_probability(&self, jaccard_similarity: f64) -> f64 {
        let s = jaccard_similarity;
        let r = self.rows_per_band as f64;
        let b = self.bands as f64;

        1.0 - (1.0 - s.powf(r)).powf(b)
    }
}
