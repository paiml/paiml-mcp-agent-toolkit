impl TdgAnalyzerAst {
    /// Get scheduler statistics for diagnostics
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_scheduler_stats(&self) -> Option<crate::tdg::SchedulingStatistics> {
        if let Some(scheduler) = &self.scheduler {
            Some(scheduler.get_statistics().await)
        } else {
            None
        }
    }

    // GREEN Phase: Public methods for TDG dogfooding - accessing stored scores

    /// Get a reference to the storage system for querying stored scores
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_storage(&self) -> Option<&TieredStore> {
        self.storage.as_ref()
    }

    /// Get stored score for a specific file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn get_stored_score(&self, path: &Path) -> Result<Option<TdgScore>> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        if let Some(storage) = &self.storage {
            // Calculate content hash for the file
            let source = fs::read_to_string(path)?;
            let content_hash = blake3::hash(source.as_bytes());

            // Check hot cache first
            if let Some(hot_entry) = storage.get_hot(&content_hash) {
                let language = Language::from_extension(path);
                let score = TdgScore {
                    total: hot_entry.total_score,
                    grade: Grade::from_score(hot_entry.total_score),
                    language,
                    confidence: language.confidence(),
                    file_path: Some(path.to_path_buf()),
                    ..Default::default()
                };
                return Ok(Some(score));
            }

            // Check warm/cold storage
            if let Some(record) = storage.retrieve_full(&content_hash).await? {
                return Ok(Some(record.score));
            }
        }
        Ok(None)
    }

    /// Get storage statistics for monitoring
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_storage_stats(&self) -> Option<crate::tdg::StorageStatistics> {
        self.storage
            .as_ref()
            .map(super::storage::TieredStore::get_statistics)
    }

    /// Get adaptive threshold statistics for diagnostics
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_adaptive_stats(&self) -> Option<crate::tdg::PerformanceStatistics> {
        if let Some(adaptive) = &self.adaptive_manager {
            Some(adaptive.get_performance_stats().await)
        } else {
            None
        }
    }

    /// Get current adaptive thresholds
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_current_thresholds(&self) -> Option<crate::tdg::CurrentThresholds> {
        if let Some(adaptive) = &self.adaptive_manager {
            Some(adaptive.get_current_thresholds().await)
        } else {
            None
        }
    }

    /// Reset adaptive thresholds to defaults
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn reset_adaptive_thresholds(&self) -> Result<()> {
        if let Some(adaptive) = &self.adaptive_manager {
            adaptive.reset_to_defaults().await?;
        }
        Ok(())
    }

    /// Get resource controller statistics for diagnostics
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_resource_stats(&self) -> Option<crate::tdg::ResourceEnforcementStats> {
        if let Some(controller) = &self.resource_controller {
            Some(controller.get_enforcement_stats().await)
        } else {
            None
        }
    }

    /// Get current resource usage
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_resource_usage(&self) -> Option<crate::tdg::ResourceUsage> {
        if let Some(controller) = &self.resource_controller {
            Some(controller.get_current_usage().await)
        } else {
            None
        }
    }

    /// Estimate memory required for file analysis
    fn estimate_analysis_memory(&self, path: &Path) -> Result<f64> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let metadata = fs::metadata(path)?;
        let file_size_mb = metadata.len() as f64 / (1024.0 * 1024.0);

        // Estimate memory as 3-5x file size (for AST parsing, analysis structures)
        let base_memory = file_size_mb * 4.0;

        // Add language-specific memory overhead
        let language = Language::from_extension(path);
        let language_overhead = match language {
            Language::Rust => 20.0,              // Rust AST is memory intensive
            Language::Cpp | Language::C => 15.0, // C++ templates can be large
            Language::Java => 12.0,              // Java reflection analysis
            Language::TypeScript => 10.0,        // TS type checking
            Language::Python => 8.0,             // Python AST is relatively compact
            Language::JavaScript => 6.0,         // JS AST is simple
            _ => 5.0,                            // Default overhead for other languages
        };

        Ok((base_memory + language_overhead).max(5.0)) // Minimum 5MB estimate
    }
}
