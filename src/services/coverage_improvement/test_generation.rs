// Property test generation from AST analysis
// Included into mod.rs via include!() -- no `use` imports or `#!` attributes allowed

/// What a generated `tests/` file may legally reference in the target project.
pub(crate) struct GeneratedTestTargets {
    /// The rust identifier of the crate under test (`-` replaced by `_`).
    pub(crate) crate_ident: String,
    /// Whether the project can compile `use proptest::prelude::*;` at all.
    pub(crate) has_proptest: bool,
}

/// Read the target project's manifest for the facts a generated test depends on.
///
/// Generated files used to open with a fixed `use proptest::prelude::*;` and
/// `use crate::<file_stem>::*;`. Both are wrong for a `tests/` integration
/// target: `crate::` there is the *test* crate, not the crate under test, and
/// proptest was never added as a dev-dependency. Writing that file turned a
/// green `cargo test --no-run` in the user's project into E0433/E0432 — and
/// pmat still counted the file as tests_generated. The manifest now decides
/// what may be emitted, and a project without proptest gets nothing rather
/// than a broken build.
pub(crate) fn read_generated_test_targets(manifest: &str) -> Option<GeneratedTestTargets> {
    let manifest: toml::Value = toml::from_str(manifest).ok()?;
    let package_name = manifest.get("package")?.get("name")?.as_str()?;
    let crate_ident = manifest
        .get("lib")
        .and_then(|lib| lib.get("name"))
        .and_then(toml::Value::as_str)
        .unwrap_or(package_name)
        .replace('-', "_");
    let has_proptest = manifest
        .get("dev-dependencies")
        .and_then(|deps| deps.get("proptest"))
        .is_some();

    Some(GeneratedTestTargets {
        crate_ident,
        has_proptest,
    })
}

impl CoverageImprovementService {
    /// Generate property-based tests for target files
    ///
    /// Parses Rust files using syn, extracts function signatures,
    /// and generates proptest templates for common types.
    ///
    /// Supports: i32, i64, u32, u64, String, Vec<T>, Option<T>
    async fn generate_property_tests(&self, targets: &[PathBuf]) -> Result<usize> {
        crate::status_eprintln!("🧪 Generating property-based tests...");

        let manifest_path = self.config.project_path.join("Cargo.toml");
        let manifest = tokio::fs::read_to_string(&manifest_path).await.ok();
        let Some(test_targets) = manifest.as_deref().and_then(read_generated_test_targets) else {
            eprintln!(
                "⚠️  Not generating property tests: no readable [package] in {}",
                manifest_path.display()
            );
            return Ok(0);
        };

        if !test_targets.has_proptest {
            eprintln!(
                "⚠️  Not generating property tests: {} has no `proptest` dev-dependency.",
                manifest_path.display()
            );
            eprintln!(
                "   Add `proptest = \"1\"` under [dev-dependencies] and re-run — emitting the\n\
                 \x20  tests now would leave the project unable to compile its test targets."
            );
            return Ok(0);
        }

        let mut tests_generated = 0;
        let mut written: Vec<PathBuf> = Vec::new();

        for target in targets {
            // Only process .rs files
            if target.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }

            // Read the file
            let full_path = if target.is_absolute() {
                target.clone()
            } else {
                self.config.project_path.join(target)
            };

            let content = match tokio::fs::read_to_string(&full_path).await {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("⚠️  Could not read {}: {}", target.display(), e);
                    continue;
                }
            };

            // Parse with syn
            let syntax_tree = match syn::parse_file(&content) {
                Ok(tree) => tree,
                Err(e) => {
                    eprintln!("⚠️  Could not parse {}: {}", target.display(), e);
                    continue;
                }
            };

            // Extract public functions
            let functions = self.extract_public_functions(&syntax_tree);

            if functions.is_empty() {
                eprintln!("  ℹ️  No public functions found in {}", target.display());
                continue;
            }

            // Generate proptest for each function
            let test_content =
                self.generate_proptest_module(&test_targets.crate_ident, target, &functions)?;

            // Write to tests directory
            let test_filename = format!(
                "proptest_{}.rs",
                target.file_stem().unwrap_or_default().to_string_lossy()
            );
            let test_path = self.config.project_path.join("tests").join(&test_filename);

            // Create tests directory if it doesn't exist
            if let Some(parent) = test_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            tokio::fs::write(&test_path, test_content).await?;
            written.push(test_path);

            tests_generated += functions.len();
            crate::status_eprintln!(
                "  ✅ Generated {} tests for {} -> {}",
                functions.len(),
                target.display(),
                test_filename
            );
        }

        // Nothing previously compiled the files we just dropped into the user's
        // tests/ directory, so a generated file that does not typecheck stayed
        // there and broke `cargo test` for everything else — while still being
        // reported as tests_generated. A file we cannot verify is rolled back.
        if !written.is_empty() && !self.generated_tests_compile().await {
            for path in &written {
                let _ = tokio::fs::remove_file(path).await;
            }
            eprintln!(
                "↩️  Reverted {} generated test file(s): they do not compile.",
                written.len()
            );
            return Ok(0);
        }

        crate::status_eprintln!("✅ Generated {} property tests total", tests_generated);

        Ok(tests_generated)
    }

    /// Whether the target project's test targets still build.
    ///
    /// Returns false when cargo cannot be run at all: an unverifiable
    /// generated file is treated exactly like a broken one.
    async fn generated_tests_compile(&self) -> bool {
        let output = Command::new("cargo")
            .args(["test", "--no-run", "--tests"])
            .current_dir(&self.config.project_path)
            .output()
            .await;

        match output {
            Ok(out) if out.status.success() => true,
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                eprintln!("⚠️  Generated tests do not compile:");
                for line in stderr.lines().take(20) {
                    eprintln!("   {line}");
                }
                false
            }
            Err(e) => {
                eprintln!("⚠️  Could not verify generated tests compile: {e}");
                false
            }
        }
    }

    /// Extract public functions from a syn::File
    pub(crate) fn extract_public_functions(&self, syntax_tree: &syn::File) -> Vec<syn::ItemFn> {
        let mut functions = Vec::new();

        for item in &syntax_tree.items {
            if let syn::Item::Fn(func) = item {
                // Check if function is public
                if matches!(func.vis, syn::Visibility::Public(_)) {
                    functions.push(func.clone());
                }
            }
        }

        functions
    }

    /// Generate a proptest module for the given functions
    ///
    /// `crate_ident` is the crate under test as an integration test must name
    /// it. This used to emit `use crate::<file_stem>::*;`, which in a `tests/`
    /// target resolves against the test crate itself and never compiles.
    pub(crate) fn generate_proptest_module(
        &self,
        crate_ident: &str,
        _target: &PathBuf,
        functions: &[syn::ItemFn],
    ) -> Result<String> {
        let mut module = String::from(
            r#"//! Auto-generated property tests
//! Generated by pmat coverage improve

use proptest::prelude::*;

"#,
        );

        module.push_str(&format!("use {}::*;\n\n", crate_ident));

        for func in functions {
            let func_name = &func.sig.ident;
            let test_name = format!("proptest_{}", func_name);

            // Extract parameters and generate strategies
            let mut param_strategies = Vec::new();
            let mut param_names = Vec::new();

            for input in &func.sig.inputs {
                if let syn::FnArg::Typed(pat_type) = input {
                    if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                        let param_name = pat_ident.ident.to_string();
                        let strategy = self.generate_strategy_for_type(&pat_type.ty);

                        param_names.push(param_name.clone());
                        param_strategies.push(format!("{} in {}", param_name, strategy));
                    }
                }
            }

            // Generate the proptest
            if param_strategies.is_empty() {
                // No parameters - simple test
                module.push_str(&format!(
                    r#"#[test]
fn {}() {{
    // Function has no parameters
    let _result = {}();
    // Add assertions based on expected behavior
}}

"#,
                    test_name, func_name
                ));
            } else {
                // Has parameters - property test
                let params_str = param_strategies.join(",\n        ");
                let call_params = param_names.join(", ");

                module.push_str(&format!(
                    r#"proptest! {{
    #[test]
    fn {}(
        {}
    ) {{
        // Property test for {}
        let _result = {}({});
        // Basic invariant: function should not panic
        prop_assert!(true);
    }}
}}

"#,
                    test_name, params_str, func_name, func_name, call_params
                ));
            }
        }

        Ok(module)
    }

    /// Generate a proptest strategy for a given type
    pub(crate) fn generate_strategy_for_type(&self, ty: &syn::Type) -> String {
        match ty {
            syn::Type::Path(type_path) => {
                let type_str = type_path
                    .path
                    .segments
                    .last()
                    .map(|seg| seg.ident.to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                match type_str.as_str() {
                    "i8" => "any::<i8>()".to_string(),
                    "i16" => "any::<i16>()".to_string(),
                    "i32" => "any::<i32>()".to_string(),
                    "i64" => "any::<i64>()".to_string(),
                    "u8" => "any::<u8>()".to_string(),
                    "u16" => "any::<u16>()".to_string(),
                    "u32" => "any::<u32>()".to_string(),
                    "u64" => "any::<u64>()".to_string(),
                    "usize" => "any::<usize>()".to_string(),
                    "isize" => "any::<isize>()".to_string(),
                    "f32" => "any::<f32>()".to_string(),
                    "f64" => "any::<f64>()".to_string(),
                    "bool" => "any::<bool>()".to_string(),
                    "char" => "any::<char>()".to_string(),
                    "String" => r#"".*""#.to_string(),
                    "str" => r#"".*""#.to_string(),
                    "Vec" => "prop::collection::vec(any::<i32>(), 0..100)".to_string(),
                    "Option" => "prop::option::of(any::<i32>())".to_string(),
                    "Result" => "any::<i32>()".to_string(), // Simplified
                    "PathBuf" => r#""[a-z0-9/]+""#.to_string(),
                    "Path" => r#""[a-z0-9/]+""#.to_string(),
                    _ => "any::<i32>()".to_string(), // Default fallback
                }
            }
            syn::Type::Reference(type_ref) => {
                // For references, generate strategy for the inner type
                self.generate_strategy_for_type(&type_ref.elem)
            }
            _ => "any::<i32>()".to_string(), // Default fallback
        }
    }
}
