// ProofCache implementation methods
// Included from proof_annotator.rs - do NOT add `use` imports or `#!` attributes here.

impl ProofCache {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
            file_times: std::collections::HashMap::new(),
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get(&self, key: &str) -> Option<&Vec<ProofAnnotation>> {
        debug_assert!(!key.is_empty(), "key must not be empty");
        self.cache.get(key)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn insert(&mut self, key: String, annotations: Vec<ProofAnnotation>) {
        self.cache.insert(key, annotations);
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn is_file_cached(&self, path: &Path) -> bool {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                if let Some(cached_time) = self.file_times.get(path) {
                    return modified <= *cached_time;
                }
            }
        }
        false
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn update_file_time(&mut self, path: std::path::PathBuf) {
        if let Ok(metadata) = std::fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                self.file_times.insert(path, modified);
            }
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn clear(&mut self) {
        self.cache.clear();
        self.file_times.clear();
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn size(&self) -> usize {
        self.cache.len()
    }
}
