//! ML Reproducibility - Seed Management
//!
//! Provides deterministic random number generation for all ML operations.
//! This module ensures embedding generation and clustering are reproducible.

use std::sync::atomic::{AtomicU64, Ordering};

/// Global seed for embedding operations
pub static EMBEDDING_SEED: AtomicU64 = AtomicU64::new(42);

/// Global seed for clustering operations
pub static CLUSTERING_SEED: AtomicU64 = AtomicU64::new(12345);

/// Global seed for mutation testing randomization
pub static MUTATION_SEED: AtomicU64 = AtomicU64::new(98765);

/// Set the embedding seed for reproducible embedding generation
///
/// # Example
/// ```
/// use pmat::services::ml_seed;
/// ml_seed::set_embedding_seed(42);
/// assert_eq!(ml_seed::get_embedding_seed(), 42);
/// ```
pub fn set_embedding_seed(seed: u64) {
    EMBEDDING_SEED.store(seed, Ordering::SeqCst);
}

/// Get the current embedding seed
pub fn get_embedding_seed() -> u64 {
    EMBEDDING_SEED.load(Ordering::SeqCst)
}

/// Set the clustering seed for reproducible clustering
pub fn set_clustering_seed(seed: u64) {
    CLUSTERING_SEED.store(seed, Ordering::SeqCst);
}

/// Get the current clustering seed
pub fn get_clustering_seed() -> u64 {
    CLUSTERING_SEED.load(Ordering::SeqCst)
}

/// Set the mutation seed for reproducible mutation testing
pub fn set_mutation_seed(seed: u64) {
    MUTATION_SEED.store(seed, Ordering::SeqCst);
}

/// Get the current mutation seed
pub fn get_mutation_seed() -> u64 {
    MUTATION_SEED.load(Ordering::SeqCst)
}

/// Initialize all seeds from environment variables or defaults
///
/// Environment variables:
/// - PMAT_EMBEDDING_SEED (default: 42)
/// - PMAT_CLUSTERING_SEED (default: 12345)
/// - PMAT_MUTATION_SEED (default: 98765)
pub fn init_seeds_from_env() {
    if let Ok(seed) = std::env::var("PMAT_EMBEDDING_SEED") {
        if let Ok(seed) = seed.parse::<u64>() {
            set_embedding_seed(seed);
        }
    }

    if let Ok(seed) = std::env::var("PMAT_CLUSTERING_SEED") {
        if let Ok(seed) = seed.parse::<u64>() {
            set_clustering_seed(seed);
        }
    }

    if let Ok(seed) = std::env::var("PMAT_MUTATION_SEED") {
        if let Ok(seed) = seed.parse::<u64>() {
            set_mutation_seed(seed);
        }
    }
}

/// Create a seeded RNG for embedding operations
pub fn create_embedding_rng() -> rand::rngs::StdRng {
    use rand::SeedableRng;
    rand::rngs::StdRng::seed_from_u64(get_embedding_seed())
}

/// Create a seeded RNG for clustering operations
pub fn create_clustering_rng() -> rand::rngs::StdRng {
    use rand::SeedableRng;
    rand::rngs::StdRng::seed_from_u64(get_clustering_seed())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_defaults() {
        // Reset to defaults
        set_embedding_seed(42);
        set_clustering_seed(12345);
        set_mutation_seed(98765);

        assert_eq!(get_embedding_seed(), 42);
        assert_eq!(get_clustering_seed(), 12345);
        assert_eq!(get_mutation_seed(), 98765);
    }

    #[test]
    fn test_seed_modification() {
        set_embedding_seed(100);
        assert_eq!(get_embedding_seed(), 100);

        // Reset
        set_embedding_seed(42);
    }

    #[test]
    fn test_rng_reproducibility() {
        use rand::Rng;

        set_embedding_seed(42);
        let mut rng1 = create_embedding_rng();
        let val1: u64 = rng1.gen();

        set_embedding_seed(42);
        let mut rng2 = create_embedding_rng();
        let val2: u64 = rng2.gen();

        assert_eq!(val1, val2, "Same seed must produce same sequence");
    }
}
