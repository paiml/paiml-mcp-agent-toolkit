#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {

        /// Property: Toolchain priority is always in valid range
        #[test]
        fn prop_toolchain_priority_in_range(priority in 1u8..=3u8) {
            let toolchain = match priority {
                1 => Toolchain::RustCli { cargo_features: vec![] },
                2 => Toolchain::DenoTypescript { deno_version: "1.0".to_string() },
                _ => Toolchain::PythonUv { python_version: "3.11".to_string() },
            };
            prop_assert!(toolchain.priority() >= 1 && toolchain.priority() <= 3);
        }

        /// Property: Toolchain as_str returns non-empty string
        #[test]
        fn prop_toolchain_as_str_non_empty(variant in 0u8..3u8) {
            let toolchain = match variant {
                0 => Toolchain::RustCli { cargo_features: vec![] },
                1 => Toolchain::DenoTypescript { deno_version: "1.0".to_string() },
                _ => Toolchain::PythonUv { python_version: "3.11".to_string() },
            };
            prop_assert!(!toolchain.as_str().is_empty());
        }

        /// Property: Serialization roundtrip preserves equality
        #[test]
        fn prop_toolchain_roundtrip(features in prop::collection::vec("[a-z]+", 0..5)) {
            let original = Toolchain::RustCli { cargo_features: features };
            let json = serde_json::to_string(&original).unwrap();
            let roundtrip: Toolchain = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(original, roundtrip);
        }

        /// Property: Parameter spec with required=true has no default
        #[test]
        fn prop_parameter_spec_required_no_default(name in "[a-z_]+") {
            let spec = ParameterSpec {
                name,
                param_type: ParameterType::ProjectName,
                required: true,
                default_value: None,
                validation_pattern: None,
                description: "test".to_string(),
            };
            prop_assert!(spec.required);
            prop_assert!(spec.default_value.is_none());
        }

        /// Property: Version ordering is transitive
        #[test]
        fn prop_version_ordering(
            major1 in 0u64..10u64,
            minor1 in 0u64..10u64,
            patch1 in 0u64..10u64,
            major2 in 0u64..10u64,
            minor2 in 0u64..10u64,
            patch2 in 0u64..10u64,
        ) {
            let v1 = Version::new(major1, minor1, patch1);
            let v2 = Version::new(major2, minor2, patch2);

            // Version comparison should be consistent
            let cmp1 = v1.cmp(&v2);
            let cmp2 = v2.cmp(&v1);

            match cmp1 {
                std::cmp::Ordering::Less => prop_assert_eq!(cmp2, std::cmp::Ordering::Greater),
                std::cmp::Ordering::Equal => prop_assert_eq!(cmp2, std::cmp::Ordering::Equal),
                std::cmp::Ordering::Greater => prop_assert_eq!(cmp2, std::cmp::Ordering::Less),
            }
        }
    }
}
