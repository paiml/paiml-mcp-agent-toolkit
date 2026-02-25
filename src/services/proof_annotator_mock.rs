// MockProofSource implementation
// Included from proof_annotator.rs - do NOT add `use` imports or `#!` attributes here.

impl MockProofSource {
    #[must_use]
    pub fn new(name: String, delay_ms: u64, annotation_count: usize) -> Self {
        Self {
            name,
            delay_ms,
            annotation_count,
        }
    }
}

impl ProofSource for MockProofSource {
    fn clone_box(&self) -> Box<dyn ProofSource> {
        Box::new(self.clone())
    }

    fn collect(
        &self,
        _project_root: &Path,
        _cache: &Arc<RwLock<ProofCache>>,
        _symbol_table: &Arc<SymbolTable>,
    ) -> Pin<
        Box<dyn Future<Output = Result<ProofCollectionResult, ProofCollectionError>> + Send + '_>,
    > {
        let delay_ms = self.delay_ms;
        let annotation_count = self.annotation_count;
        let tool_name = self.name.clone();

        Box::pin(async move {
            // Simulate some work
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;

            let mut annotations = Vec::new();

            // Generate mock annotations with unique file names per source
            for i in 0..annotation_count {
                let location = Location::new(
                    std::path::PathBuf::from(format!("{tool_name}_{i}.rs")),
                    i as u32 * 10,
                    (i as u32 + 1) * 10,
                );

                let annotation = ProofAnnotation {
                    annotation_id: uuid::Uuid::new_v4(),
                    property_proven: PropertyType::MemorySafety,
                    specification_id: None,
                    method: VerificationMethod::BorrowChecker,
                    tool_name: tool_name.clone(),
                    tool_version: "1.0.0".to_string(),
                    confidence_level: ConfidenceLevel::Medium,
                    assumptions: vec![],
                    evidence_type:
                        crate::models::unified_ast::EvidenceType::ImplicitTypeSystemGuarantee,
                    evidence_location: None,
                    date_verified: chrono::Utc::now(),
                };

                annotations.push((location, annotation));
            }

            Ok(ProofCollectionResult {
                annotations,
                errors: Vec::new(),
                metrics: CollectionMetrics {
                    files_processed: annotation_count,
                    annotations_found: annotation_count,
                    cache_hits: 0,
                    duration_ms: delay_ms,
                },
            })
        })
    }
}
