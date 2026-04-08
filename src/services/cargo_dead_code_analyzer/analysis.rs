// Core analysis methods for CargoDeadCodeAnalyzer
// Included from cargo_dead_code_analyzer.rs - shares parent module scope

impl CargoDeadCodeAnalyzer {
    /// Perform accurate dead code analysis using cargo with O(1) caching
    ///
    /// Uses a four-layer detection strategy:
    /// 1. SUPPRESSION_SCAN: Detect  attributes (explicit admissions)
    /// 2. COMPILER_LINT: Run cargo check with -W dead_code
    /// 3. REFERENCE_GRAPH: (future) Build call graph for unreachable code
    /// 4. HEURISTICS: (future) Pattern-based detection
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn analyze(&self) -> Result<AccurateDeadCodeReport> {
        use tokio::time::{timeout, Duration};

        // Try cache first for O(1) performance
        if let Some(cached) = self.try_load_cache() {
            return Ok(cached);
        }

        // Cache miss - run full analysis
        let analysis_future = async {
            // Layer 1: Scan for suppression attributes (fast, catches explicit admissions)
            let mut all_dead_items = self.scan_for_suppression_attributes()?;

            // Layer 2: Run cargo check for compiler-detected dead code
            let cargo_output = self.run_cargo_check()?;
            let compiler_dead_items = self.parse_cargo_warnings(&cargo_output)?;
            all_dead_items.extend(compiler_dead_items);

            let files_with_dead_code = self.group_by_file(all_dead_items);
            let report = self.calculate_metrics(files_with_dead_code).await?;

            // Save to cache for next time
            self.save_cache(&report);

            Ok(report)
        };

        // Apply 90 second timeout to the entire analysis
        timeout(Duration::from_secs(90), analysis_future)
            .await
            .map_err(|_| anyhow::anyhow!("Dead code analysis timed out after 90 seconds"))?
    }

    /// Layer 1: Scan for  attributes
    ///
    /// These attributes are explicit admissions that code is unused.
    /// Detecting them is fast (~10ms for large projects) and catches
    /// code that developers knowingly left as dead.
    fn scan_for_suppression_attributes(&self) -> Result<Vec<(PathBuf, DeadItem)>> {
        use regex::Regex;
        use std::fs;

        let mut suppressed_items = Vec::new();

        // Patterns for dead_code suppression
        // Matches: #[allow(dead_code)], #[allow(unused)], #![allow(dead_code)]
        let suppression_re =
            Regex::new(r#"#!?\[allow\((dead_code|unused)\)\]"#).expect("Invalid regex");

        // Pattern to extract the item name on the following line
        let item_re = Regex::new(
            r#"^\s*(?:pub\s+)?(?:async\s+)?(?:const\s+)?(?:static\s+)?(?:unsafe\s+)?(fn|struct|enum|type|trait|mod|const|static)\s+(\w+)"#
        ).expect("Invalid regex");

        // Walk through all Rust files
        for entry in walkdir::WalkDir::new(&self.project_path)
            .max_depth(self.max_depth)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();

            // Skip target directory and non-Rust files
            if path.starts_with(self.project_path.join("target")) {
                continue;
            }

            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }

            // Read file content
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let lines: Vec<&str> = content.lines().collect();

            // Scan for suppression attributes
            self.scan_file_for_suppressions(
                path,
                &lines,
                &suppression_re,
                &item_re,
                &mut suppressed_items,
            );
        }

        tracing::debug!(
            "Layer 1 (suppression scan): found {} items with ",
            suppressed_items.len()
        );

        Ok(suppressed_items)
    }

    /// Scan a single file's lines for suppression attributes
    fn scan_file_for_suppressions(
        &self,
        path: &Path,
        lines: &[&str],
        suppression_re: &regex::Regex,
        item_re: &regex::Regex,
        suppressed_items: &mut Vec<(PathBuf, DeadItem)>,
    ) {
        for (i, line) in lines.iter().enumerate() {
            if suppression_re.is_match(line) {
                // Try to find the item on the next non-attribute line
                let mut item_line = i + 1;
                while item_line < lines.len() {
                    let next_line = lines[item_line];
                    // Skip additional attributes and empty lines
                    if next_line.trim().starts_with("#[")
                        || next_line.trim().starts_with("#![")
                        || next_line.trim().is_empty()
                    {
                        item_line += 1;
                        continue;
                    }

                    // Try to extract the item
                    if let Some(caps) = item_re.captures(next_line) {
                        let kind_str = caps.get(1).map(|m| m.as_str()).unwrap_or("unknown");
                        let name = caps.get(2).map(|m| m.as_str()).unwrap_or("unknown");

                        let relative_path = path
                            .strip_prefix(&self.project_path)
                            .unwrap_or(path)
                            .to_path_buf();

                        suppressed_items.push((
                            relative_path,
                            DeadItem {
                                name: name.to_string(),
                                kind: DeadCodeKind::Suppressed,
                                line: item_line + 1, // 1-indexed
                                column: 1,
                                message: format!(
                                    "{} `{}` has  suppression (explicit dead code admission)",
                                    kind_str, name
                                ),
                            },
                        ));
                    }
                    break;
                }
            }
        }
    }

    /// Run cargo check and capture JSON output with timeout
    fn run_cargo_check(&self) -> Result<String> {
        // PMAT_DEAD_CODE_SKIP=1 can be used to skip in specific test scenarios
        // Removed CI bypass per CB-128 spec - dead code detection must work everywhere
        if std::env::var("PMAT_DEAD_CODE_SKIP").is_ok() {
            return Ok(r#"{"reason":"build-finished","success":true}"#.to_string());
        }

        let mut cmd = Command::new("cargo");
        cmd.current_dir(&self.project_path)
            .arg("check")
            .arg("--message-format=json");

        // Don't modify RUSTFLAGS — changing flags forces full recompilation
        // of all deps (including cc, which fails with extra warnings).
        // Dead code detection uses heuristic source scanning instead.
        // The compiler-level dead_code lint is only useful for catching
        // items the heuristic misses, but the cost (full recompile) is too high.

        // Use targeted checks instead of --all-targets for faster execution
        if self.exclude_tests {
            cmd.arg("--lib").arg("--bins");
        } else {
            // Check only the lib by default for faster execution
            cmd.arg("--lib");
        }

        let output = cmd.output().context("Failed to run cargo check")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Cargo check failed: {}", stderr));
        }

        // Cargo outputs JSON messages to stdout
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }
}
