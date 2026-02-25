    // ==================== ConfidenceLevel Tests ====================

    #[test]
    fn test_confidence_level_ordering() {
        assert!(ConfidenceLevel::Low < ConfidenceLevel::Medium);
        assert!(ConfidenceLevel::Medium < ConfidenceLevel::High);
        assert!(ConfidenceLevel::Low < ConfidenceLevel::High);
    }

    #[test]
    fn test_confidence_level_values() {
        assert_eq!(ConfidenceLevel::Low as u8, 1);
        assert_eq!(ConfidenceLevel::Medium as u8, 2);
        assert_eq!(ConfidenceLevel::High as u8, 3);
    }

    #[test]
    fn test_confidence_level_serialization() {
        for level in [
            ConfidenceLevel::Low,
            ConfidenceLevel::Medium,
            ConfidenceLevel::High,
        ] {
            let json = serde_json::to_string(&level).expect("serialization failed");
            let deserialized: ConfidenceLevel =
                serde_json::from_str(&json).expect("deserialization failed");
            assert_eq!(level, deserialized);
        }
    }

    // ==================== PropertyType Tests ====================

    #[test]
    fn test_property_type_all_variants() {
        let variants = [
            PropertyType::MemorySafety,
            PropertyType::ThreadSafety,
            PropertyType::DataRaceFreeze,
            PropertyType::Termination,
            PropertyType::FunctionalCorrectness("test_spec".to_string()),
            PropertyType::ResourceBounds {
                cpu: Some(100),
                memory: Some(1024),
            },
        ];
        assert_eq!(variants.len(), 6);
    }

    #[test]
    fn test_property_type_resource_bounds_optional() {
        let cpu_only = PropertyType::ResourceBounds {
            cpu: Some(100),
            memory: None,
        };
        let memory_only = PropertyType::ResourceBounds {
            cpu: None,
            memory: Some(1024),
        };
        let both = PropertyType::ResourceBounds {
            cpu: Some(100),
            memory: Some(1024),
        };
        let neither = PropertyType::ResourceBounds {
            cpu: None,
            memory: None,
        };

        // All should serialize successfully
        for prop in [cpu_only, memory_only, both, neither] {
            let json = serde_json::to_string(&prop).expect("serialization failed");
            let _: PropertyType = serde_json::from_str(&json).expect("deserialization failed");
        }
    }

    #[test]
    fn test_property_type_hash() {
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        PropertyType::MemorySafety.hash(&mut hasher1);
        PropertyType::MemorySafety.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    // ==================== VerificationMethod Tests ====================

    #[test]
    fn test_verification_method_all_variants() {
        let variants = [
            VerificationMethod::BorrowChecker,
            VerificationMethod::FormalProof {
                prover: "Coq".to_string(),
            },
            VerificationMethod::StaticAnalysis {
                tool: "Miri".to_string(),
            },
            VerificationMethod::ModelChecking { bounded: true },
            VerificationMethod::ModelChecking { bounded: false },
            VerificationMethod::AbstractInterpretation,
        ];
        assert_eq!(variants.len(), 6);
    }

    #[test]
    fn test_verification_method_serialization() {
        let methods = [
            VerificationMethod::BorrowChecker,
            VerificationMethod::FormalProof {
                prover: "Lean".to_string(),
            },
            VerificationMethod::StaticAnalysis {
                tool: "Clippy".to_string(),
            },
            VerificationMethod::ModelChecking { bounded: true },
            VerificationMethod::AbstractInterpretation,
        ];

        for method in methods {
            let json = serde_json::to_string(&method).expect("serialization failed");
            let deserialized: VerificationMethod =
                serde_json::from_str(&json).expect("deserialization failed");
            assert_eq!(method, deserialized);
        }
    }

    // ==================== EvidenceType Tests ====================

    #[test]
    fn test_evidence_type_all_variants() {
        let variants = [
            EvidenceType::ImplicitTypeSystemGuarantee,
            EvidenceType::ProofScriptReference {
                uri: "file://proof.v".to_string(),
            },
            EvidenceType::TheoremName {
                theorem: "memory_safety_theorem".to_string(),
                theory: Some("MemorySafety".to_string()),
            },
            EvidenceType::TheoremName {
                theorem: "theorem".to_string(),
                theory: None,
            },
            EvidenceType::StaticAnalysisReport {
                report_id: "report_001".to_string(),
            },
            EvidenceType::CertificateHash {
                hash: "abc123".to_string(),
                algorithm: "sha256".to_string(),
            },
        ];
        assert_eq!(variants.len(), 6);
    }

    #[test]
    fn test_evidence_type_serialization() {
        let evidence = EvidenceType::CertificateHash {
            hash: "deadbeef".to_string(),
            algorithm: "sha512".to_string(),
        };
        let json = serde_json::to_string(&evidence).expect("serialization failed");
        let deserialized: EvidenceType =
            serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(evidence, deserialized);
    }

