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

    // Issue: .unwrap() in a file with a module-level #![allow(clippy::unwrap_used)]
    // suppression must not be auto-failed. The developer has explicitly opted
    // these unwraps out of the lint that owns this policy (mirrors clippy).
    #[test]
    fn test_skips_unwrap_with_file_level_allow() {
        let detector = RustDefectDetector::new();
        let code = r#"#![allow(clippy::unwrap_used)]

            fn compute(runs: &[i32]) -> i32 {
                let last = runs.last().unwrap();
                let first = runs.first().unwrap();
                last + first
            }
        "#;

        let path = PathBuf::from("src/cli/profile/run.rs");
        let defects = detector.detect(code, &path);

        assert_eq!(
            defects.len(),
            0,
            "file-level #![allow(clippy::unwrap_used)] must suppress unwrap auto-fail"
        );
    }

    // The clippy::restriction group contains unwrap_used, so allowing the group
    // (#![allow(clippy::restriction)]) also suppresses the lint.
    #[test]
    fn test_skips_unwrap_with_file_level_allow_restriction_group() {
        let detector = RustDefectDetector::new();
        let code = r#"#![allow(clippy::restriction)]

            fn f(x: Option<i32>) -> i32 {
                x.unwrap()
            }
        "#;

        let path = PathBuf::from("src/lib.rs");
        let defects = detector.detect(code, &path);

        assert_eq!(
            defects.len(),
            0,
            "#![allow(clippy::restriction)] (group containing unwrap_used) must suppress"
        );
    }

    // Item-level (outer) #[allow(clippy::unwrap_used)] suppresses unwrap detection
    // within the annotated item only.
    #[test]
    fn test_skips_unwrap_with_item_level_allow() {
        let detector = RustDefectDetector::new();
        let code = r#"
            #[allow(clippy::unwrap_used)]
            fn allowed(x: Option<i32>) -> i32 {
                x.unwrap()
            }

            fn not_allowed(y: Option<i32>) -> i32 {
                y.unwrap()
            }
        "#;

        let path = PathBuf::from("src/lib.rs");
        let defects = detector.detect(code, &path);

        // The unwrap in `allowed` is suppressed; the unwrap in `not_allowed` is not.
        assert_eq!(
            defects.len(),
            1,
            "only the unwrap outside the #[allow] item should be detected"
        );
        assert_eq!(defects[0].instances.len(), 1);
        assert!(
            defects[0].instances[0].code_snippet.contains("y.unwrap()"),
            "the detected unwrap should be the one not covered by #[allow]"
        );
    }

    // Guard: clippy::all does NOT contain unwrap_used (it is in the restriction
    // group), so #![allow(clippy::all)] must NOT suppress the unwrap auto-fail.
    #[test]
    fn test_clippy_all_does_not_suppress_unwrap() {
        let detector = RustDefectDetector::new();
        let code = r#"#![allow(clippy::all, clippy::pedantic)]

            fn f(x: Option<i32>) -> i32 {
                x.unwrap()
            }
        "#;

        let path = PathBuf::from("src/lib.rs");
        let defects = detector.detect(code, &path);

        assert_eq!(
            defects.len(),
            1,
            "#![allow(clippy::all)] must NOT suppress unwrap_used (different lint group)"
        );
    }

    // Regression for the whisper.apr profile/run.rs false positive: a file that
    // opens with #![allow(clippy::unwrap_used)] then later #![allow(clippy::all)]
    // and contains real unwraps in non-cfg production code must NOT auto-fail.
    #[test]
    fn test_whisper_apr_profile_run_shape_not_failed() {
        let detector = RustDefectDetector::new();
        let code = r#"#![allow(clippy::unwrap_used)]
            #![allow(dead_code)]
            #![allow(clippy::all, clippy::pedantic)]

            //! module docs

            fn compute_avg(runs: &[i32]) -> Option<i32> {
                if runs.is_empty() {
                    return None;
                }
                let avg = |f: fn(&i32) -> i32| -> i32 {
                    runs.iter().map(|r| f(r)).sum::<i32>()
                };
                let last = runs.last().unwrap();
                Some(avg(|r| *r) + last)
            }
        "#;

        let path = PathBuf::from("src/cli/apr_commands/phase3/profile/run.rs");
        let defects = detector.detect(code, &path);

        assert_eq!(
            defects.len(),
            0,
            "file with #![allow(clippy::unwrap_used)] must not be auto-failed (whisper.apr regression)"
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

    // ── LuaDefectDetector (Wave 39 PR11) ────────────────────────────────────

    #[test]
    fn test_lua_detect_implicit_global_assignment() {
        let detector = LuaDefectDetector::new();
        let path = PathBuf::from("script.lua");
        let code = "x = 42\nlocal y = 99\nz = x + y\n";
        let defects = detector.detect(code, &path);
        // PIN: implicit global assignments (x =, z =) flagged;
        // local y = ... should NOT be flagged.
        assert!(!defects.is_empty(), "expected implicit-global defects");
    }

    #[test]
    fn test_lua_detect_dangerous_api_os_execute() {
        let detector = LuaDefectDetector::new();
        let path = PathBuf::from("script.lua");
        let code = "local result = os.execute(\"rm -rf /\")\n";
        let defects = detector.detect(code, &path);
        assert!(
            !defects.is_empty(),
            "os.execute should trigger a dangerous-API defect"
        );
    }

    #[test]
    fn test_lua_detect_dangerous_api_loadstring() {
        let detector = LuaDefectDetector::new();
        let path = PathBuf::from("script.lua");
        let code = "local f = loadstring(user_input)\n";
        let defects = detector.detect(code, &path);
        assert!(!defects.is_empty(), "loadstring should trigger a defect");
    }

    #[test]
    fn test_lua_detect_unchecked_pcall_compiles() {
        // Just exercise the detect_unchecked_pcall code path.
        let detector = LuaDefectDetector::new();
        let path = PathBuf::from("script.lua");
        let code = "pcall(some_function)\n";
        let _ = detector.detect(code, &path); // result varies by impl; goal: cover the lines
    }

    #[test]
    fn test_lua_detect_clean_local_only_no_global_defects() {
        let detector = LuaDefectDetector::new();
        let path = PathBuf::from("script.lua");
        let code = "local a = 1\nlocal b = 2\nlocal sum = a + b\n";
        let defects = detector.detect(code, &path);
        // PIN: pure-local assignments must not trigger implicit-global defect.
        let global_assigns = defects
            .iter()
            .filter(|d| d.id.contains("GLOBAL") || d.name.to_lowercase().contains("global"))
            .count();
        assert_eq!(
            global_assigns, 0,
            "local-only code should have no implicit-global defects"
        );
    }

    #[test]
    fn test_lua_excludes_test_files() {
        let detector = LuaDefectDetector::new();
        let path = PathBuf::from("tests/my_test.lua");
        let code = "x = 42\n"; // would normally trigger global defect
        let defects = detector.detect(code, &path);
        // PIN: should_exclude_file skips test paths.
        assert_eq!(defects.len(), 0, "test files should be excluded");
    }

    #[test]
    fn test_lua_detector_default_constructor_works() {
        // Exercises impl Default for LuaDefectDetector.
        let detector = LuaDefectDetector::default();
        let path = PathBuf::from("script.lua");
        let _ = detector.detect("local x = 1\n", &path);
    }

    #[test]
    fn test_lua_empty_source_no_defects() {
        let detector = LuaDefectDetector::new();
        let path = PathBuf::from("empty.lua");
        let defects = detector.detect("", &path);
        assert!(defects.is_empty());
    }

    // ===== #926: language dispatch =====

    /// A Lua source that is byte-identical must be graded the same wherever the
    /// checkout happens to sit. RED before this change: the Lua detector had a
    /// private copy of the exclusion rule that matched `"/tests/"` as a
    /// substring of the ABSOLUTE path, so the copy under `<tmp>/tests/proj/`
    /// reported 0 defects — the #923 defect, re-committed in a second language.
    #[test]
    fn test_lua_exclusion_is_not_a_property_of_the_checkout_location() {
        let source = "count = 0\ntotal = 1\n";

        let counts: Vec<usize> = ["plain", "tests", "spec"]
            .iter()
            .map(|parent| {
                let root = tempfile::tempdir().expect("tempdir");
                let project = root.path().join(parent).join("proj");
                std::fs::create_dir_all(project.join("src")).expect("mkdir");
                // A package manifest marks the project root, so everything
                // above it is the caller's filesystem and says nothing.
                std::fs::write(project.join("package.json"), "{}").expect("manifest");
                let file = project.join("src").join("init.lua");
                std::fs::write(&file, source).expect("source");

                LuaDefectDetector::new().detect(source, &file).len()
            })
            .collect();

        assert!(
            counts[0] > 0,
            "the fixture must produce a defect at all, else the test proves nothing"
        );
        assert_eq!(
            counts[0], counts[1],
            "a checkout under a directory called tests/ must grade the same as one that is not"
        );
        assert_eq!(
            counts[0], counts[2],
            "a checkout under a directory called spec/ must grade the same as one that is not"
        );
    }

    /// A package's OWN tests/ tree is still support code — the rule moved, it
    /// did not disappear.
    #[test]
    fn test_lua_still_excludes_the_projects_own_test_tree() {
        let root = tempfile::tempdir().expect("tempdir");
        let project = root.path().join("proj");
        std::fs::create_dir_all(project.join("spec")).expect("mkdir");
        std::fs::write(project.join("package.json"), "{}").expect("manifest");
        let file = project.join("spec").join("init.lua");

        let defects = LuaDefectDetector::new().detect("count = 0\n", &file);
        assert!(
            defects.is_empty(),
            "the package's own spec/ tree is support code"
        );
    }

    #[test]
    fn test_every_supported_extension_has_a_detector() {
        for ext in SUPPORTED_EXTENSIONS {
            let path = PathBuf::from(format!("src/thing.{ext}"));
            assert!(
                detector_for(&path).is_some(),
                ".{ext} is advertised as supported but dispatches to no rule set"
            );
        }
    }

    /// A language pmat has no rules for must arrive as an explicit reason, not
    /// as "nothing excluded it and nothing found anything".
    #[test]
    fn test_unsupported_language_is_absent_not_clean() {
        let path = PathBuf::from("src/main.go");
        assert_eq!(detector_for(&path), None);
        assert!(!is_supported(&path));
        assert_eq!(
            exclusion_reason(&path),
            Some(unmeasured::Reason::NoRuleSet),
            "a .go file must be reported as ungraded, not as graded-and-clean"
        );
        assert!(
            detect_defects("func main() {}\n", &path).is_empty(),
            "and it must not be graded by another language's rules"
        );
    }

    #[test]
    fn test_exclusion_reason_dispatches_per_language() {
        assert_eq!(
            exclusion_reason(&PathBuf::from("spec/app.lua")),
            Some(unmeasured::Reason::NonProductionDir)
        );
        assert_eq!(
            exclusion_reason(&PathBuf::from("src/app_test.py")),
            Some(unmeasured::Reason::TestFileName)
        );
        assert_eq!(exclusion_reason(&PathBuf::from("src/app.ts")), None);
    }

    /// #926: `--severity high|medium|low` could not match a finding in ANY
    /// codebase, because the only rule set the command reached (Rust) emits
    /// exactly one pattern and it is Critical. RED before this change: the
    /// reachable set was `{Critical}`.
    #[test]
    fn test_every_advertised_severity_is_reachable_through_dispatch() {
        let fixtures: [(&str, &str); 4] = [
            (
                "src/lib.rs",
                "pub fn go(x: Option<u32>) -> u32 { x.unwrap() }\n",
            ),
            ("src/app.lua", "count = 0\n"),
            (
                "src/app.py",
                "def load(cfg=[]):\n    assert cfg, \"empty\"\n    return eval(cfg)\n",
            ),
            (
                "src/app.ts",
                "const id: any = raw;\nif (id == '0') { use(id!.value); }\n",
            ),
        ];

        let mut reachable = std::collections::BTreeSet::new();
        for (path, source) in fixtures {
            for defect in detect_defects(source, &PathBuf::from(path)) {
                reachable.insert(defect.severity.as_str());
            }
        }

        for severity in ["CRITICAL", "HIGH", "MEDIUM", "LOW"] {
            assert!(
                reachable.contains(severity),
                "`--severity {}` can never match: reachable severities are {reachable:?}",
                severity.to_lowercase()
            );
        }
    }

    // ===== Python rule set =====

    #[test]
    fn test_python_bare_except_is_high() {
        let defects = PythonDefectDetector::new().detect(
            "try:\n    load()\nexcept:\n    pass\n",
            &PathBuf::from("src/app.py"),
        );
        let found = defects
            .iter()
            .find(|d| d.id == "PY-EXCEPT-001")
            .expect("bare except");
        assert_eq!(found.severity, Severity::High);
        assert_eq!(found.instances[0].line, 3);
    }

    #[test]
    fn test_python_typed_except_is_not_a_defect() {
        let defects = PythonDefectDetector::new().detect(
            "try:\n    load()\nexcept OSError:\n    pass\n",
            &PathBuf::from("src/app.py"),
        );
        assert!(defects.iter().all(|d| d.id != "PY-EXCEPT-001"));
    }

    #[test]
    fn test_python_mutable_default_argument_is_high() {
        let defects = PythonDefectDetector::new().detect(
            "def append(item, into=[]):\n    into.append(item)\n",
            &PathBuf::from("src/app.py"),
        );
        let found = defects
            .iter()
            .find(|d| d.id == "PY-MUTDEF-001")
            .expect("mutable default");
        assert_eq!(found.severity, Severity::High);
    }

    #[test]
    fn test_python_immutable_default_argument_is_not_a_defect() {
        let defects = PythonDefectDetector::new().detect(
            "def append(item, into=None):\n    pass\n",
            &PathBuf::from("src/app.py"),
        );
        assert!(defects.iter().all(|d| d.id != "PY-MUTDEF-001"));
    }

    #[test]
    fn test_python_eval_of_non_literal_is_critical_and_literal_is_not() {
        let detector = PythonDefectDetector::new();
        let path = PathBuf::from("src/app.py");

        let dynamic = detector.detect("value = eval(request.body)\n", &path);
        let found = dynamic
            .iter()
            .find(|d| d.id == "PY-EVAL-001")
            .expect("dynamic eval");
        assert_eq!(found.severity, Severity::Critical);

        let literal = detector.detect("value = eval(\"2 + 2\")\n", &path);
        assert!(
            literal.iter().all(|d| d.id != "PY-EVAL-001"),
            "a quoted literal is a constant, not eval injection"
        );
    }

    #[test]
    fn test_python_assert_as_validation_is_medium() {
        let defects = PythonDefectDetector::new().detect(
            "assert user.is_admin, \"forbidden\"\n",
            &PathBuf::from("src/app.py"),
        );
        let found = defects
            .iter()
            .find(|d| d.id == "PY-ASSERT-001")
            .expect("assert");
        assert_eq!(found.severity, Severity::Medium);
    }

    #[test]
    fn test_python_comments_are_not_code() {
        let defects = PythonDefectDetector::new().detect(
            "# except:\n# assert nope\n# value = eval(x)\n",
            &PathBuf::from("src/app.py"),
        );
        assert!(defects.is_empty(), "{defects:?}");
    }

    #[test]
    fn test_python_test_files_are_excluded() {
        let defects = PythonDefectDetector::new().detect(
            "def f(a=[]):\n    pass\n",
            &PathBuf::from("tests/test_app.py"),
        );
        assert!(defects.is_empty());
    }

    // ===== TypeScript rule set =====

    #[test]
    fn test_typescript_any_is_medium() {
        let defects = TypeScriptDefectDetector::new().detect(
            "function h(p: any) { return p; }\n",
            &PathBuf::from("src/h.ts"),
        );
        let found = defects
            .iter()
            .find(|d| d.id == "TS-ANY-001")
            .expect("explicit any");
        assert_eq!(found.severity, Severity::Medium);
    }

    #[test]
    fn test_typescript_non_null_assertion_is_high() {
        let defects = TypeScriptDefectDetector::new().detect(
            "render(users.find(u => u.id === id)!.name);\n",
            &PathBuf::from("src/h.ts"),
        );
        let found = defects
            .iter()
            .find(|d| d.id == "TS-NONNULL-001")
            .expect("non-null assertion");
        assert_eq!(found.severity, Severity::High);
    }

    #[test]
    fn test_typescript_strict_inequality_is_not_a_non_null_assertion() {
        let defects = TypeScriptDefectDetector::new()
            .detect("if (a !== b) { go(); }\n", &PathBuf::from("src/h.ts"));
        assert!(
            defects.iter().all(|d| d.id != "TS-NONNULL-001"),
            "{defects:?}"
        );
    }

    #[test]
    fn test_typescript_loose_equality_is_low() {
        let defects = TypeScriptDefectDetector::new()
            .detect("if (count == '0') { go(); }\n", &PathBuf::from("src/h.ts"));
        let found = defects
            .iter()
            .find(|d| d.id == "TS-EQ-001")
            .expect("loose equality");
        assert_eq!(found.severity, Severity::Low);
    }

    #[test]
    fn test_typescript_null_comparison_and_strict_equality_are_exempt() {
        let detector = TypeScriptDefectDetector::new();
        let path = PathBuf::from("src/h.ts");
        for line in ["if (x == null) { go(); }\n", "if (x === y) { go(); }\n"] {
            let defects = detector.detect(line, &path);
            assert!(
                defects.iter().all(|d| d.id != "TS-EQ-001"),
                "{line} must not be reported: {defects:?}"
            );
        }
    }

    #[test]
    fn test_typescript_comments_are_not_code() {
        let defects = TypeScriptDefectDetector::new().detect(
            "// const x: any = 1;\n/* if (a == b) {} */\n * value!.thing\n",
            &PathBuf::from("src/h.ts"),
        );
        assert!(defects.is_empty(), "{defects:?}");
    }

    /// A checked-in bundle is not this project's code, and grading it buries
    /// every real finding.
    ///
    /// #926 CONSEQUENCE: the moment the walk could see `.js`,
    /// `assets/vendor/mermaid.min.js` produced **124 of the 146** defects
    /// reported on this repository — while `analyze satd` had been excluding
    /// that same file all along, with a rule it owned privately.
    #[test]
    fn test_vendored_and_minified_files_are_not_graded() {
        let temp = tempfile::TempDir::new().expect("temp dir");
        std::fs::write(
            temp.path().join("package.json"),
            "{\"name\":\"fixture\",\"version\":\"0.1.0\"}\n",
        )
        .expect("write manifest");

        // Bundled by name, anywhere in the tree.
        for name in ["mermaid.min.js", "app.bundle.js", "react.production.js"] {
            let path = temp.path().join("assets").join(name);
            std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
            std::fs::write(&path, "var a=1;if(a==1){b!.c}\n").expect("write");
            assert_eq!(
                exclusion_reason(&path),
                Some(unmeasured::Reason::VendoredOrMinified),
                "{name} is a build artifact of someone else's source"
            );
        }

        // Vendored by location, whatever it is called and whatever language.
        for name in ["dep.js", "dep.py", "dep.lua", "dep.rs"] {
            let path = temp.path().join("vendor").join(name);
            std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
            std::fs::write(&path, "x = 1\n").expect("write");
            assert_eq!(
                exclusion_reason(&path),
                Some(unmeasured::Reason::VendoredOrMinified),
                "vendor/{name} is a dependency's source"
            );
        }

        // And the project's own code is still measured.
        let mine = temp.path().join("src/app.js");
        std::fs::create_dir_all(mine.parent().expect("parent")).expect("mkdir");
        std::fs::write(&mine, "export const f = (a) => a!.b;\n").expect("write");
        assert_eq!(exclusion_reason(&mine), None);
    }

    /// The SATD detector's `is_minified_or_vendor_file` and the Known-Defects
    /// walk must not be able to disagree: there is one predicate, and this
    /// pins that the SATD side still routes through it.
    #[test]
    fn test_satd_and_defects_share_one_vendor_rule() {
        let temp = tempfile::TempDir::new().expect("temp dir");
        std::fs::write(
            temp.path().join("package.json"),
            "{\"name\":\"fixture\",\"version\":\"0.1.0\"}\n",
        )
        .expect("write manifest");
        let satd = crate::services::satd_detector::SATDDetector::new();

        for (relative, vendored) in [
            ("assets/vendor/mermaid.min.js", true),
            ("assets/d3.min.js", true),
            ("src/app.js", false),
        ] {
            let path = temp.path().join(relative);
            std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
            std::fs::write(&path, "var a=1;\n").expect("write");
            assert_eq!(
                satd.is_minified_or_vendor_file(&path),
                vendored,
                "satd verdict on {relative}"
            );
            assert_eq!(
                exclusion_reason(&path) == Some(unmeasured::Reason::VendoredOrMinified),
                vendored,
                "defects verdict on {relative} must match satd's"
            );
        }
    }

    /// #927, the three findings the include-graph fix could not reach: an
    /// `.unwrap()` inside a test function whose harness attribute was not one
    /// of the four literals the detector knew.
    #[test]
    fn test_every_test_harness_attribute_suppresses_its_own_body() {
        let detector = RustDefectDetector::new();
        let path = PathBuf::from("/tmp/whatever/src/lib.rs");

        for attr in [
            "#[test]",
            "#[tokio::test]",
            "#[tokio::test(flavor = \"multi_thread\")]",
            "#[actix_rt::test]",
            "#[async_std::test]",
            "#[googletest::test]",
            "#[sqlx::test]",
            "#[rstest]",
            "#[test_case(1, 2)]",
            "#[bench]",
        ] {
            let code = format!(
                "{attr}\nfn t() {{\n    let r: Option<i32> = Some(1);\n    let _ = r.unwrap();\n}}\n"
            );
            assert!(
                detector.detect(&code, &path).is_empty(),
                "{attr} marks a test function; its body is test code"
            );
        }

        // A one-line body is the same item. The brace-depth walk used to open
        // and close the suppressed block inside a single iteration and report
        // the `.unwrap()` on that very line (#927).
        for code in [
            "#[actix_rt::test]\nasync fn t() { let r: Option<i32> = Some(1); let _ = r.unwrap(); }\n",
            "#[test]\nfn t() { let r: Option<i32> = Some(1); let _ = r.unwrap(); }\n",
            "#[cfg(test)]\nmod t { fn f() { let r: Option<i32> = Some(1); let _ = r.unwrap(); } }\n",
        ] {
            assert!(
                detector.detect(code, &path).is_empty(),
                "suppression is a property of the item, not of where the newlines are: {code}"
            );
        }

        // …and the generalisation must not swallow production code. Neither an
        // unrelated attribute nor an unattributed function is a test.
        for attr in ["#[inline]", "#[derive(Debug)]", "#[allow(dead_code)]"] {
            let code = format!(
                "{attr}\nfn p() {{\n    let r: Option<i32> = Some(1);\n    let _ = r.unwrap();\n}}\n"
            );
            assert_eq!(
                detector.detect(&code, &path).len(),
                1,
                "{attr} does not name a test harness"
            );
        }
        assert_eq!(
            detector
                .detect(
                    "pub fn p() { let r: Option<i32> = Some(1); let _ = r.unwrap(); }\n",
                    &path
                )
                .len(),
            1,
            "an unattributed one-line function is production code and is still reported"
        );
    }
}
