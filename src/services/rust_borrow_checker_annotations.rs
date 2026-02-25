// Proof annotation factory methods for RustBorrowChecker
// Included by rust_borrow_checker.rs - no `use` imports or `#!` attributes

impl RustBorrowChecker {
    /// Create memory safety annotation
    fn memory_safety_annotation(&self) -> ProofAnnotation {
        ProofAnnotation {
            annotation_id: uuid::Uuid::new_v4(),
            property_proven: PropertyType::MemorySafety,
            specification_id: None,
            method: VerificationMethod::BorrowChecker,
            tool_name: format!("rustc-{}", self.rustc_channel),
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
    fn create_thread_safety_annotation(&self) -> ProofAnnotation {
        ProofAnnotation {
            annotation_id: uuid::Uuid::new_v4(),
            property_proven: PropertyType::ThreadSafety,
            specification_id: None,
            method: VerificationMethod::BorrowChecker,
            tool_name: format!("rustc-{}", self.rustc_channel),
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
    fn const_fn_termination(&self) -> ProofAnnotation {
        ProofAnnotation {
            annotation_id: uuid::Uuid::new_v4(),
            property_proven: PropertyType::Termination,
            specification_id: None,
            method: VerificationMethod::BorrowChecker,
            tool_name: format!("rustc-{}", self.rustc_channel),
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
    fn auto_trait_annotation(&self, trait_path: &syn::Path) -> ProofAnnotation {
        let trait_name = quote::quote!(#trait_path).to_string();
        let property = match trait_name.as_str() {
            "Send" | "Sync" => PropertyType::ThreadSafety,
            _ => PropertyType::MemorySafety,
        };

        ProofAnnotation {
            annotation_id: uuid::Uuid::new_v4(),
            property_proven: property,
            specification_id: Some(format!("auto_trait_{trait_name}")),
            method: VerificationMethod::BorrowChecker,
            tool_name: format!("rustc-{}", self.rustc_channel),
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
    fn auto_trait_annotation(&self, trait_path: &syn::Path) -> ProofAnnotation {
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
            annotation_id: uuid::Uuid::new_v4(),
            property_proven: property,
            specification_id: Some(format!("auto_trait_{}", trait_name)),
            method: VerificationMethod::BorrowChecker,
            tool_name: format!("rustc-{}", self.rustc_channel),
            tool_version: self.rustc_version.clone(),
            confidence_level: ConfidenceLevel::High,
            assumptions: vec![format!("{} auto trait implementation", trait_name)],
            evidence_type: EvidenceType::ImplicitTypeSystemGuarantee,
            evidence_location: None,
            date_verified: chrono::Utc::now(),
        }
    }
}
