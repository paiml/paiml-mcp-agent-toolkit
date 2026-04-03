#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detect_unwrap() {
        let detector = RustDefectDetector::new();
        let code = r#"
            fn main() {
                let x = Some(42).unwrap();
            }
        "#;

        let path = PathBuf::from("src/main.rs");
        let defects = detector.detect(code, &path);

        assert_eq!(defects.len(), 1);
        assert_eq!(defects[0].id, "RUST-UNWRAP-001");
        assert_eq!(defects[0].severity, Severity::Critical);
        assert_eq!(defects[0].instances.len(), 1);
    }

    #[test]
    fn test_excludes_doc_comments() {
        let detector = RustDefectDetector::new();
        let code = r#"
            /// # Examples
            ///
            /// ```
            /// let result = something.unwrap();
            /// ```
            pub fn something() -> Option<i32> {
                Some(42)
            }

            //! Module doc with example
            //! let x = foo.unwrap();
        "#;

        let path = PathBuf::from("src/lib.rs");
        let defects = detector.detect(code, &path);

        assert_eq!(
            defects.len(),
            0,
            "Doc comments should be excluded (issue #131)"
        );
    }

    #[test]
    fn test_excludes_test_code() {
        let detector = RustDefectDetector::new();
        let code = r#"
            #[cfg_attr(coverage_nightly, coverage(off))]
            #[cfg(test)]
            mod tests {
                fn test_foo() {
                    let x = Some(42).unwrap();
                }
            }
        "#;

        let path = PathBuf::from("src/lib.rs");
        let defects = detector.detect(code, &path);

        assert_eq!(defects.len(), 0, "Test code should be excluded");
    }

    #[test]
    fn test_excludes_test_directory() {
        let detector = RustDefectDetector::new();
        let code = r#"
            fn test_helper() {
                let x = Some(42).expect("internal error");
            }
        "#;

        let path = PathBuf::from("tests/integration_test.rs");
        let defects = detector.detect(code, &path);

        assert_eq!(defects.len(), 0, "Tests directory should be excluded");
    }

    #[test]
    fn test_excludes_examples_directory() {
        let detector = RustDefectDetector::new();
        let code = r#"
            fn main() {
                let x = Some(42).expect("internal error");
            }
        "#;

        // Test various examples path patterns
        for path in &[
            "examples/demo.rs",
            "./examples/demo.rs",
            "server/examples/demo.rs",
        ] {
            let path = PathBuf::from(path);
            let defects = detector.detect(code, &path);
            assert_eq!(
                defects.len(),
                0,
                "Examples directory should be excluded: {}",
                path.display()
            );
        }
    }

    // Issue #279: .unwrap() inside #[cfg(feature)] blocks should not be detected
    #[test]
    fn test_skips_unwrap_in_cfg_feature_block() {
        let detector = RustDefectDetector::new();
        let code = r#"
            #[cfg(feature = "cuda")]
            impl GpuBackend {
                fn init() {
                    let device = adapter.request_device().unwrap();
                }
            }
        "#;

        let path = PathBuf::from("src/gpu/wgpu.rs");
        let defects = detector.detect(code, &path);

        assert_eq!(
            defects.len(),
            0,
            "unwrap() inside #[cfg(feature)] blocks should be skipped (issue #279)"
        );
    }

    #[test]
    fn test_skips_unwrap_in_cfg_target_block() {
        let detector = RustDefectDetector::new();
        let code = r#"
            #[cfg(target_os = "linux")]
            fn platform_init() {
                let fd = open_device().unwrap();
            }
        "#;

        let path = PathBuf::from("src/platform.rs");
        let defects = detector.detect(code, &path);

        assert_eq!(
            defects.len(),
            0,
            "unwrap() inside #[cfg(target_os)] should be skipped"
        );
    }

    #[test]
    fn test_detects_unwrap_outside_cfg_block() {
        let detector = RustDefectDetector::new();
        let code = r#"
            #[cfg(feature = "cuda")]
            impl GpuBackend {
                fn init() {
                    let device = adapter.request_device().unwrap();
                }
            }

            fn regular_code() {
                let x = Some(42).unwrap();
            }
        "#;

        let path = PathBuf::from("src/gpu/wgpu.rs");
        let defects = detector.detect(code, &path);

        assert_eq!(
            defects.len(),
            1,
            "unwrap() OUTSIDE #[cfg] block should still be detected"
        );
        assert_eq!(defects[0].instances.len(), 1);
    }

    #[test]
    fn test_skips_unwrap_in_nested_cfg_block() {
        let detector = RustDefectDetector::new();
        let code = r#"
            #[cfg(feature = "cuda")]
            mod gpu {
                fn inner() {
                    let x = something.unwrap();
                    if true {
                        let y = other.unwrap();
                    }
                }
            }
        "#;

        let path = PathBuf::from("src/gpu.rs");
        let defects = detector.detect(code, &path);

        assert_eq!(
            defects.len(),
            0,
            "unwrap() in nested scopes inside #[cfg] should be skipped"
        );
    }

    #[test]
    fn test_excludes_fuzz_directory() {
        let detector = RustDefectDetector::new();
        let code = r#"
            fn fuzz_target() {
                let x = Some(42).expect("internal error");
            }
        "#;

        // Test various fuzz path patterns
        for path in &[
            "fuzz/fuzz_targets/target.rs",
            "./fuzz/fuzz_targets/target.rs",
            "server/fuzz/target.rs",
        ] {
            let path = PathBuf::from(path);
            let defects = detector.detect(code, &path);
            assert_eq!(
                defects.len(),
                0,
                "Fuzz directory should be excluded: {}",
                path.display()
            );
        }
    }
}
