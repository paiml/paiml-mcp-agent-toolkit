// DynamicDispatchAnalyzer and DeadCodeProver implementation methods
// Included from dead_code_prover.rs - do NOT add `use` imports or `#!` attributes here.

impl DynamicDispatchAnalyzer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            trait_impls: HashMap::new(),
            function_pointers: HashSet::new(),
            trait_objects: HashMap::new(),
        }
    }

    /// Find trait object usage for a symbol
    #[must_use]
    pub fn find_trait_object_usage(&self, symbol: &SymbolId) -> Option<Usage> {
        // Check if symbol implements trait used in dyn Trait
        for (trait_name, impls) in &self.trait_impls {
            if impls.contains(symbol) && self.trait_objects.contains_key(trait_name) {
                return Some(Usage::TraitObject(trait_name.clone()));
            }
        }

        // Check if symbol address taken for fn pointer
        if self.function_pointers.contains(symbol) {
            return Some(Usage::FunctionPointer);
        }

        None
    }
}

impl DeadCodeProver {
    #[must_use]
    pub fn new() -> Self {
        Self {
            reachability: ReachabilityAnalyzer::new(),
            ffi_tracker: FFIReferenceTracker::new(),
            dynamic_analyzer: DynamicDispatchAnalyzer::new(),
        }
    }

    /// Get access to FFI tracker for testing
    #[must_use]
    pub fn ffi_tracker(&self) -> &FFIReferenceTracker {
        &self.ffi_tracker
    }

    /// Analyze file for dead code with FFI awareness
    pub fn analyze_file(&mut self, file_path: &Path, content: &str) -> Vec<DeadCodeProof> {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        let file_path_str = file_path.to_string_lossy().to_string();
        self.ffi_tracker.scan_for_ffi_exports(content, &file_path_str);

        let mut proofs = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("fn ") || trimmed.contains("fn ") {
                if let Some(func_name) = self.ffi_tracker.extract_function_name_from_line(line) {
                    let symbol = SymbolId {
                        file_path: file_path_str.clone(),
                        function_name: func_name.clone(),
                        line_number: line_num + 1,
                    };
                    proofs.push(self.classify_symbol(symbol, &file_path_str, line_num));
                }
            }
        }
        proofs
    }

    fn classify_symbol(&self, symbol: SymbolId, file_path_str: &str, line_num: usize) -> DeadCodeProof {
        debug_assert!(!file_path_str.is_empty(), "file_path_str must not be empty");
        if self.is_ffi_visible(&symbol, file_path_str, line_num) {
            return DeadCodeProof {
                item: symbol,
                proof_type: DeadCodeProofType::ProvenLive,
                confidence: 0.95,
                evidence: vec![Evidence {
                    evidence_type: EvidenceType::FFIExport,
                    description: "Function is exported via FFI".to_string(),
                    confidence: 0.95,
                }],
            };
        }
        if let Some(usage) = self.dynamic_analyzer.find_trait_object_usage(&symbol) {
            return DeadCodeProof {
                item: symbol,
                proof_type: DeadCodeProofType::ProvenLive,
                confidence: 0.8,
                evidence: vec![Evidence {
                    evidence_type: EvidenceType::DynamicDispatch,
                    description: format!("Function used via dynamic dispatch: {usage:?}"),
                    confidence: 0.8,
                }],
            };
        }
        DeadCodeProof {
            item: symbol,
            proof_type: DeadCodeProofType::UnknownLiveness,
            confidence: 0.6,
            evidence: vec![Evidence {
                evidence_type: EvidenceType::NoReferences,
                description: "No obvious references found".to_string(),
                confidence: 0.6,
            }],
        }
    }

    fn is_ffi_visible(&self, symbol: &SymbolId, file_path_str: &str, line_num: usize) -> bool {
        debug_assert!(!file_path_str.is_empty(), "file_path_str must not be empty");
        if self.ffi_tracker.is_externally_visible(symbol) {
            return true;
        }
        (0..=3).any(|offset| {
            let alt = SymbolId {
                file_path: file_path_str.to_string(),
                function_name: symbol.function_name.clone(),
                line_number: line_num + 1 + offset,
            };
            self.ffi_tracker.is_externally_visible(&alt)
        })
    }

    /// Generate comprehensive dead code report
    #[must_use]
    pub fn generate_report(&self, proofs: &[DeadCodeProof]) -> DeadCodeReport {
        let mut dead_functions = Vec::new();

        for proof in proofs {
            if matches!(proof.proof_type, DeadCodeProofType::ProvenDead) {
                dead_functions.push(DeadCodeItem {
                    node_key: 0, // Would be proper node key in real implementation
                    name: proof.item.function_name.clone(),
                    file_path: proof.item.file_path.clone(),
                    line_number: proof.item.line_number as u32,
                    dead_type: DeadCodeType::UnusedFunction,
                    confidence: proof.confidence as f32,
                    reason: proof
                        .evidence
                        .first()
                        .map_or_else(|| "Unknown".to_string(), |e| e.description.clone()),
                });
            }
        }

        DeadCodeReport {
            dead_functions,
            dead_classes: Vec::new(),
            dead_variables: Vec::new(),
            unreachable_code: Vec::new(),
            summary: crate::services::dead_code_analyzer::DeadCodeSummary {
                total_dead_code_lines: 0,
                percentage_dead: 0.0,
                dead_by_type: HashMap::new(),
                confidence_level: 0.8,
            },
        }
    }
}
