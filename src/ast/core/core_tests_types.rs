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

    // ==================== BytePos Tests ====================

    #[test]
    fn test_byte_pos_to_usize() {
        let pos = BytePos(42);
        assert_eq!(pos.to_usize(), 42);
    }

    #[test]
    fn test_byte_pos_from_usize() {
        let pos = BytePos::from_usize(1000);
        assert_eq!(pos.0, 1000);
        assert_eq!(pos.to_usize(), 1000);
    }

    #[test]
    fn test_byte_pos_from_usize_max() {
        let pos = BytePos::from_usize(u32::MAX as usize);
        assert_eq!(pos.0, u32::MAX);
    }

    #[test]
    fn test_byte_pos_ordering() {
        let pos1 = BytePos(10);
        let pos2 = BytePos(20);
        let pos3 = BytePos(10);

        assert!(pos1 < pos2);
        assert!(pos2 > pos1);
        assert!(pos1 <= pos3);
        assert!(pos1 >= pos3);
        assert_eq!(pos1, pos3);
    }

    #[test]
    fn test_byte_pos_hash() {
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        BytePos(42).hash(&mut hasher1);
        BytePos(42).hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    // ==================== Span Tests ====================

    #[test]
    fn test_span_new() {
        let span = Span::new(10, 50);
        assert_eq!(span.start.0, 10);
        assert_eq!(span.end.0, 50);
    }

    #[test]
    fn test_span_len() {
        let span = Span::new(10, 50);
        assert_eq!(span.len(), 40);
    }

    #[test]
    fn test_span_len_zero() {
        let span = Span::new(10, 10);
        assert_eq!(span.len(), 0);
    }

    #[test]
    fn test_span_is_empty() {
        let empty = Span::new(10, 10);
        let non_empty = Span::new(10, 20);
        let invalid = Span::new(20, 10); // end < start

        assert!(empty.is_empty());
        assert!(!non_empty.is_empty());
        assert!(invalid.is_empty());
    }

    #[test]
    fn test_span_contains() {
        let span = Span::new(10, 20);

        assert!(span.contains(BytePos(10))); // start is inclusive
        assert!(span.contains(BytePos(15)));
        assert!(!span.contains(BytePos(20))); // end is exclusive
        assert!(!span.contains(BytePos(9)));
        assert!(!span.contains(BytePos(21)));
    }

    #[test]
    fn test_span_hash() {
        let span1 = Span::new(10, 20);
        let span2 = Span::new(10, 20);

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        span1.hash(&mut hasher1);
        span2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    // ==================== Location Tests ====================

    #[test]
    fn test_location_new() {
        let loc = Location::new(PathBuf::from("test.rs"), 100, 200);
        assert_eq!(loc.file_path, PathBuf::from("test.rs"));
        assert_eq!(loc.span.start.0, 100);
        assert_eq!(loc.span.end.0, 200);
        assert_eq!(loc.span.len(), 100);
    }

    #[test]
    fn test_location_contains() {
        let file = PathBuf::from("test.rs");
        let outer = Location::new(file.clone(), 0, 100);
        let inner = Location::new(file.clone(), 10, 50);
        let exact = Location::new(file.clone(), 0, 100);
        let partial_overlap = Location::new(file.clone(), 50, 150);
        let different_file = Location::new(PathBuf::from("other.rs"), 10, 50);

        assert!(outer.contains(&inner));
        assert!(outer.contains(&exact));
        assert!(!inner.contains(&outer));
        assert!(!outer.contains(&partial_overlap));
        assert!(!outer.contains(&different_file));
    }

    #[test]
    fn test_location_overlaps() {
        let file = PathBuf::from("test.rs");
        let loc1 = Location::new(file.clone(), 0, 50);
        let loc2 = Location::new(file.clone(), 25, 75);
        let loc3 = Location::new(file.clone(), 50, 100); // touches but doesn't overlap
        let loc4 = Location::new(file.clone(), 100, 150);
        let different_file = Location::new(PathBuf::from("other.rs"), 25, 75);

        assert!(loc1.overlaps(&loc2));
        assert!(loc2.overlaps(&loc1));
        assert!(!loc1.overlaps(&loc3)); // end of loc1 == start of loc3, but not overlapping
        assert!(!loc1.overlaps(&loc4));
        assert!(!loc1.overlaps(&different_file));
    }

    #[test]
    fn test_location_self_overlap_and_contain() {
        let loc = Location::new(PathBuf::from("test.rs"), 10, 50);
        assert!(loc.overlaps(&loc));
        assert!(loc.contains(&loc));
    }

    #[test]
    fn test_location_hash() {
        let loc1 = Location::new(PathBuf::from("test.rs"), 10, 50);
        let loc2 = Location::new(PathBuf::from("test.rs"), 10, 50);

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        loc1.hash(&mut hasher1);
        loc2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_location_hash_prefix_matching() {
        // End position is omitted for prefix matching scenarios
        let loc1 = Location::new(PathBuf::from("test.rs"), 10, 50);
        let loc2 = Location::new(PathBuf::from("test.rs"), 10, 100);

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        loc1.hash(&mut hasher1);
        loc2.hash(&mut hasher2);

        // Should have same hash due to prefix matching design
        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    // ==================== QualifiedName Tests ====================

    #[test]
    fn test_qualified_name_new() {
        let qname = QualifiedName::new(
            vec!["std".to_string(), "io".to_string()],
            "Read".to_string(),
        );
        assert_eq!(qname.module_path, vec!["std", "io"]);
        assert_eq!(qname.name, "Read");
        assert!(qname.disambiguator.is_none());
    }

    #[test]
    fn test_qualified_name_new_empty_path() {
        let qname = QualifiedName::new(vec![], "main".to_string());
        assert!(qname.module_path.is_empty());
        assert_eq!(qname.name, "main");
    }

    #[test]
    fn test_qualified_name_with_disambiguator() {
        let qname =
            QualifiedName::new(vec!["crate".to_string()], "func".to_string()).with_disambiguator(1);
        assert_eq!(qname.disambiguator, Some(1));
    }

    #[test]
    fn test_qualified_name_from_string() {
        let qname = QualifiedName::from_string("std::collections::HashMap").expect("parse failed");
        assert_eq!(qname.module_path, vec!["std", "collections"]);
        assert_eq!(qname.name, "HashMap");
    }

    #[test]
    fn test_qualified_name_from_string_simple() {
        let qname = QualifiedName::from_string("main").expect("parse failed");
        assert!(qname.module_path.is_empty());
        assert_eq!(qname.name, "main");
    }

    #[test]
    fn test_qualified_name_from_string_empty() {
        let result = QualifiedName::from_string("");
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Empty qualified name");
    }

    #[test]
    fn test_qualified_name_from_string_trailing_separator() {
        let result = QualifiedName::from_string("std::io::");
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Empty qualified name");
    }

    #[test]
    fn test_qualified_name_to_qualified_string() {
        let qname = QualifiedName::new(
            vec!["crate".to_string(), "module".to_string()],
            "function".to_string(),
        );
        assert_eq!(qname.to_qualified_string(), "crate::module::function");
    }

    #[test]
    fn test_qualified_name_to_qualified_string_with_disambiguator() {
        let qname =
            QualifiedName::new(vec!["crate".to_string()], "func".to_string()).with_disambiguator(2);
        assert_eq!(qname.to_qualified_string(), "crate::func#2");
    }

    #[test]
    fn test_qualified_name_to_qualified_string_simple() {
        let qname = QualifiedName::new(vec![], "main".to_string());
        assert_eq!(qname.to_qualified_string(), "main");
    }

    #[test]
    fn test_qualified_name_display() {
        let qname = QualifiedName::new(
            vec!["std".to_string(), "io".to_string()],
            "Read".to_string(),
        );
        assert_eq!(format!("{}", qname), "std::io::Read");
    }

    #[test]
    fn test_qualified_name_from_str() {
        let qname: QualifiedName = "std::io::Read".parse().expect("parse failed");
        assert_eq!(qname.name, "Read");
        assert_eq!(qname.module_path, vec!["std", "io"]);
    }

    #[test]
    fn test_qualified_name_hash() {
        let qname1 = QualifiedName::new(vec!["std".to_string()], "io".to_string());
        let qname2 = QualifiedName::new(vec!["std".to_string()], "io".to_string());

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        qname1.hash(&mut hasher1);
        qname2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

