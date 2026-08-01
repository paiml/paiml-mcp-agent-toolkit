// Proof annotation factory methods for RustBorrowChecker
// Included by rust_borrow_checker.rs - no `use` imports or `#!` attributes

impl RustBorrowChecker {
    /// Canonical seed for an annotation's id: the site plus the claim.
    ///
    /// DETERMINISM: see `crate::models::unified_ast::derive_annotation_id`.
    /// Every factory below used `Uuid::new_v4()`, so the same annotation about
    /// the same unchanged line got a brand new `annotationId` on every run.
    fn annotation_seed(&self, location: &Location, kind: &str) -> String {
        format!(
            "{}|{}-{}|{}|pmat-{}|{}",
            location.file_path.display(),
            location.span.start.0,
            location.span.end.0,
            kind,
            self.rustc_channel,
            self.rustc_version,
        )
    }

    /// Create memory safety annotation
    fn memory_safety_annotation(&self, location: &Location) -> ProofAnnotation {
        ProofAnnotation {
            annotation_id: crate::models::unified_ast::derive_annotation_id(
                &self.annotation_seed(location, "MemorySafety/BorrowChecker"),
            ),
            property_proven: PropertyType::MemorySafety,
            specification_id: None,
            method: VerificationMethod::BorrowChecker,
            // "pmat-syn-static-analysis", not a compiler that never ran.
            tool_name: format!("pmat-{}", self.rustc_channel),
            tool_version: self.rustc_version.clone(),
            confidence_level: ConfidenceLevel::High,
            assumptions: vec![
                "Safe Rust subset".to_string(),
                "No compiler bugs".to_string(),
            ],
            evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
            evidence_location: None,
            date_verified: chrono::Utc::now(),
        }
    }

    /// Create thread safety annotation
    fn create_thread_safety_annotation(&self, location: &Location) -> ProofAnnotation {
        ProofAnnotation {
            annotation_id: crate::models::unified_ast::derive_annotation_id(
                &self.annotation_seed(location, "ThreadSafety/BorrowChecker"),
            ),
            property_proven: PropertyType::ThreadSafety,
            specification_id: None,
            method: VerificationMethod::BorrowChecker,
            // "pmat-syn-static-analysis", not a compiler that never ran.
            tool_name: format!("pmat-{}", self.rustc_channel),
            tool_version: self.rustc_version.clone(),
            confidence_level: ConfidenceLevel::High,
            assumptions: vec![
                "Send + Sync bounds satisfied".to_string(),
                "No interior mutability without synchronization".to_string(),
            ],
            evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
            evidence_location: None,
            date_verified: chrono::Utc::now(),
        }
    }

    /// Create const fn termination annotation
    fn const_fn_termination(&self, location: &Location) -> ProofAnnotation {
        ProofAnnotation {
            annotation_id: crate::models::unified_ast::derive_annotation_id(
                &self.annotation_seed(location, "Termination/ConstFn"),
            ),
            property_proven: PropertyType::Termination,
            specification_id: None,
            method: VerificationMethod::BorrowChecker,
            // "pmat-syn-static-analysis", not a compiler that never ran.
            tool_name: format!("pmat-{}", self.rustc_channel),
            tool_version: self.rustc_version.clone(),
            confidence_level: ConfidenceLevel::High,
            assumptions: vec!["const fn restrictions guarantee termination".to_string()],
            evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
            evidence_location: None,
            date_verified: chrono::Utc::now(),
        }
    }

    /// Create auto trait annotation
    #[cfg(all(feature = "rust-ast", feature = "quote"))]
    fn auto_trait_annotation(&self, location: &Location, trait_path: &syn::Path) -> ProofAnnotation {
        let trait_name = quote::quote!(#trait_path).to_string();
        let property = match trait_name.as_str() {
            "Send" | "Sync" => PropertyType::ThreadSafety,
            _ => PropertyType::MemorySafety,
        };

        ProofAnnotation {
            annotation_id: crate::models::unified_ast::derive_annotation_id(
                &self.annotation_seed(location, &format!("AutoTrait/{trait_name}")),
            ),
            property_proven: property,
            specification_id: Some(format!("auto_trait_{trait_name}")),
            method: VerificationMethod::BorrowChecker,
            // "pmat-syn-static-analysis", not a compiler that never ran.
            tool_name: format!("pmat-{}", self.rustc_channel),
            tool_version: self.rustc_version.clone(),
            confidence_level: ConfidenceLevel::High,
            assumptions: vec![format!("{} auto trait implementation", trait_name)],
            evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
            evidence_location: None,
            date_verified: chrono::Utc::now(),
        }
    }

    /// Create auto trait annotation without quote
    #[cfg(all(feature = "rust-ast", not(feature = "quote")))]
    fn auto_trait_annotation(&self, location: &Location, trait_path: &syn::Path) -> ProofAnnotation {
        let trait_name = if let Some(segment) = trait_path.segments.last() {
            segment.ident.to_string()
        } else {
            "Unknown".to_string()
        };

        let property = match trait_name.as_str() {
            "Send" | "Sync" => PropertyType::ThreadSafety,
            _ => PropertyType::MemorySafety,
        };

        ProofAnnotation {
            annotation_id: crate::models::unified_ast::derive_annotation_id(
                &self.annotation_seed(location, &format!("AutoTrait/{trait_name}")),
            ),
            property_proven: property,
            specification_id: Some(format!("auto_trait_{}", trait_name)),
            method: VerificationMethod::BorrowChecker,
            // "pmat-syn-static-analysis", not a compiler that never ran.
            tool_name: format!("pmat-{}", self.rustc_channel),
            tool_version: self.rustc_version.clone(),
            confidence_level: ConfidenceLevel::High,
            assumptions: vec![format!("{} auto trait implementation", trait_name)],
            evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
            evidence_location: None,
            date_verified: chrono::Utc::now(),
        }
    }
}
