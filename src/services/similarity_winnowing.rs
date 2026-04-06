// Winnowing algorithm implementation for code fingerprinting.
// Provides k-gram extraction, fingerprint selection via sliding window,
// Jaccard similarity between fingerprint sets, and substring match detection.

impl Winnowing {
    #[must_use]
    pub fn new(window_size: usize, k_gram_size: usize) -> Self {
        debug_assert!(window_size > 0, "window_size must be positive");
        Self {
            window_size,
            k_gram_size,
        }
    }

    #[must_use]
    pub fn fingerprint(&self, text: &str) -> Vec<u64> {
        debug_assert!(!text.is_empty(), "text must not be empty");
        let k_grams = self.extract_k_grams(text);
        self.select_fingerprints(&k_grams)
    }

    #[must_use]
    pub fn similarity(&self, fp1: &[u64], fp2: &[u64]) -> f64 {
        let set1: HashSet<_> = fp1.iter().collect();
        let set2: HashSet<_> = fp2.iter().collect();

        let intersection = set1.intersection(&set2).count() as f64;
        let union = set1.union(&set2).count() as f64;

        if union > 0.0 {
            intersection / union
        } else {
            0.0
        }
    }

    #[must_use]
    pub fn find_matches(&self, text_fp: &[u64], sub_fp: &[u64]) -> Vec<usize> {
        let mut matches = Vec::new();
        let sub_set: HashSet<_> = sub_fp.iter().collect();

        for (i, fp) in text_fp.iter().enumerate() {
            if sub_set.contains(fp) {
                matches.push(i);
            }
        }

        matches
    }

    fn extract_k_grams(&self, text: &str) -> Vec<u64> {
        let chars: Vec<char> = text.chars().collect();
        let mut k_grams = Vec::new();

        for i in 0..chars.len().saturating_sub(self.k_gram_size - 1) {
            let gram: String = chars[i..i + self.k_gram_size].iter().collect();
            k_grams.push(self.hash_k_gram(&gram));
        }

        k_grams
    }

    fn select_fingerprints(&self, k_grams: &[u64]) -> Vec<u64> {
        let mut fingerprints = Vec::new();

        for window in k_grams.windows(self.window_size) {
            if let Some(min) = window.iter().min() {
                if !fingerprints.contains(min) {
                    fingerprints.push(*min);
                }
            }
        }

        fingerprints
    }

    fn hash_k_gram(&self, gram: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        gram.hash(&mut hasher);
        hasher.finish()
    }
}
