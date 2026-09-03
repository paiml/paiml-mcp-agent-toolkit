// Parsing and metrics calculation for CargoDeadCodeAnalyzer
// Included from cargo_dead_code_analyzer.rs - shares parent module scope

impl CargoDeadCodeAnalyzer {
    /// Parse cargo's JSON output for dead code warnings
    fn parse_cargo_warnings(&self, output: &str) -> Result<Vec<(PathBuf, DeadItem)>> {
        let mut dead_items = Vec::new();

        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let json: Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue, // Skip non-JSON lines
            };

            // Check if this is a compiler message
            if json["reason"] != "compiler-message" {
                continue;
            }

            let message = &json["message"];

            // Check if this is a dead code warning
            if let Some(code) = message["code"]["code"].as_str() {
                if code == "dead_code" {
                    if let Some(item) = self.in_scope_finding(self.extract_dead_item(message)) {
                        dead_items.push(item);
                    }
                } else if code == "unreachable_code" {
                    // rustc's OTHER dead-code lint. It was discarded here, so
                    // nothing on the CLI path could ever produce an unreachable
                    // block and `--include-unreachable` was inert on every
                    // input — a fixture with four statements after a `return`
                    // still printed "Unreachable blocks: 0". Tagged with
                    // `DeadCodeKind::UnreachableCode` so `group_by_file` can
                    // keep it out of `dead_items` and out of every count a
                    // default run prints.
                    if let Some(item) =
                        self.in_scope_finding(self.extract_unreachable_item(message))
                    {
                        dead_items.push(item);
                    }
                }
            }
        }

        Ok(dead_items)
    }

    /// A compiler finding, re-expressed against the path the caller asked
    /// about, or `None` when it does not belong to this report.
    ///
    /// Two filters, and both of them are about SCOPE:
    ///
    /// * outside the requested tree — cargo compiles the whole crate because
    ///   rustc cannot type-check less than one, so a request for
    ///   `<crate>/src/inner` gets warnings for `<crate>/src/other.rs` too. That
    ///   used to be reported: `analyze dead-code --path <crate>/src/inner`
    ///   listed `src/other.rs` under a summary claiming one file analysed.
    /// * excluded source — one scope predicate for both layers. The suppression
    ///   walk already consults `is_excluded_source`; the compiler layer did
    ///   not, so any target cargo happened to build (a lib's implicit
    ///   `cfg(test)` bench target, say) could put files into the report that
    ///   the walk that produced the denominator never opened.
    fn in_scope_finding(
        &self,
        extracted: Option<(PathBuf, DeadItem)>,
    ) -> Option<(PathBuf, DeadItem)> {
        let (reported, item) = extracted?;
        let scoped = self.scoped_report_path(&reported)?;
        (!self.is_excluded_source(&scoped)).then_some((scoped, item))
    }

    /// Where a cargo diagnostic's file sits INSIDE the requested tree, or
    /// `None` when it sits outside it.
    ///
    /// This is the mapping every row depends on. rustc names a file relative to
    /// the WORKSPACE root — not the package root, and not the directory this
    /// command was pointed at — so the reported name has to be resolved to a
    /// real file before it can be compared with anything. Two consequences the
    /// raw name could not deliver: a workspace member's rows were unreadable
    /// (`total_lines: null`, because `<package>/crates/member/src/x.rs` is not
    /// a path), and a subdirectory request could not tell its own files from
    /// the rest of the crate's.
    fn scoped_report_path(&self, reported: &Path) -> Option<PathBuf> {
        let absolute = self.resolve_reported_path(reported);
        // Scope is decided against the requested path, naming against the
        // directory that holds it — the two differ only when the request IS a
        // file, where naming a row relative to itself would leave it nameless.
        if !absolute.starts_with(&self.report_root) {
            return None;
        }
        absolute
            .strip_prefix(&self.report_base)
            .ok()
            .map(Path::to_path_buf)
    }

    /// The file a cargo diagnostic names, as an absolute path.
    ///
    /// Cargo emits the path relative to the workspace root, which is the crate
    /// root for a standalone package and an ancestor of it for a workspace
    /// member. Rather than shell out to `cargo metadata` for the distinction,
    /// the bases are tried outwards from the crate root and the first one that
    /// yields an existing file wins. A name that resolves nowhere keeps the
    /// crate root as its base, so it is still comparable rather than dropped on
    /// a technicality.
    fn resolve_reported_path(&self, reported: &Path) -> PathBuf {
        if reported.is_absolute() {
            return reported.to_path_buf();
        }
        self.cargo_root
            .ancestors()
            .map(|base| base.join(reported))
            .find(|candidate| candidate.exists())
            .unwrap_or_else(|| self.cargo_root.join(reported))
    }

    /// Extract an `unreachable_code` finding from a compiler message.
    ///
    /// `parse_message` cannot be reused: its patterns all key off "` is never
    /// used`", and rustc words this lint "unreachable statement" / "unreachable
    /// expression" with no item name. The name is the source line, so the
    /// report can point at something.
    fn extract_unreachable_item(&self, message: &Value) -> Option<(PathBuf, DeadItem)> {
        let spans = message["spans"].as_array()?;
        let primary_span = spans
            .iter()
            .find(|s| s["is_primary"].as_bool() == Some(true))?;

        let file_path = PathBuf::from(primary_span["file_name"].as_str()?);
        let line = primary_span["line_start"].as_u64()? as usize;
        let column = primary_span["column_start"].as_u64()? as usize;
        let message_text = message["message"].as_str()?;
        let name = primary_span["text"]
            .as_array()
            .and_then(|texts| texts.first())
            .and_then(|t| t["text"].as_str())
            .map_or_else(|| format!("line {line}"), |t| t.trim().to_string());

        Some((
            file_path,
            DeadItem {
                name,
                kind: DeadCodeKind::UnreachableCode,
                line,
                column,
                message: message_text.to_string(),
            },
        ))
    }

    /// Extract dead code item from compiler message
    fn extract_dead_item(&self, message: &Value) -> Option<(PathBuf, DeadItem)> {
        let spans = message["spans"].as_array()?;
        let primary_span = spans
            .iter()
            .find(|s| s["is_primary"].as_bool() == Some(true))?;

        let file_path = PathBuf::from(primary_span["file_name"].as_str()?);
        let line = primary_span["line_start"].as_u64()? as usize;
        let column = primary_span["column_start"].as_u64()? as usize;

        let message_text = message["message"].as_str()?;
        let (name, kind) = self.parse_message(message_text)?;

        Some((
            file_path,
            DeadItem {
                name,
                kind,
                line,
                column,
                message: message_text.to_string(),
            },
        ))
    }

    /// Parse the warning message to extract name and kind
    fn parse_message(&self, message: &str) -> Option<(String, DeadCodeKind)> {
        // Common patterns in dead code messages
        let patterns = [
            ("function `", "` is never used", DeadCodeKind::Function),
            ("method `", "` is never used", DeadCodeKind::Method),
            ("struct `", "` is never constructed", DeadCodeKind::Struct),
            ("enum `", "` is never used", DeadCodeKind::Enum),
            ("variant `", "` is never constructed", DeadCodeKind::Variant),
            ("field `", "` is never read", DeadCodeKind::Field),
            ("constant `", "` is never used", DeadCodeKind::Constant),
            ("static `", "` is never used", DeadCodeKind::Static),
            ("module `", "` is never used", DeadCodeKind::Module),
            ("trait `", "` is never used", DeadCodeKind::Trait),
            ("type alias `", "` is never used", DeadCodeKind::TypeAlias),
        ];

        for (prefix, suffix, kind) in &patterns {
            if let Some(start) = message.find(prefix) {
                let name_start = start + prefix.len();
                if let Some(end) = message[name_start..].find(suffix) {
                    let name = message[name_start..name_start + end].to_string();
                    return Some((name, kind.clone()));
                }
            }
        }

        // Fallback for unknown patterns
        if message.contains("is never") || message.contains("never used") {
            // Try to extract name between backticks
            if let Some(start) = message.find('`') {
                if let Some(end) = message[start + 1..].find('`') {
                    let name = message[start + 1..start + 1 + end].to_string();
                    return Some((name, DeadCodeKind::Other("unknown".to_string())));
                }
            }
        }

        None
    }

    /// Group dead items by file
    fn group_by_file(&self, items: Vec<(PathBuf, DeadItem)>) -> Vec<FileDeadCode> {
        let mut file_map: HashMap<PathBuf, Vec<DeadItem>> = HashMap::new();

        for (path, item) in items {
            file_map.entry(path).or_default().push(item);
        }

        file_map
            .into_iter()
            .map(|(file_path, items)| {
                // SPLIT, DO NOT MERGE: unreachable findings must not reach any
                // counter a default run prints, so they leave `dead_items`
                // here — before the percentage, the estimated line counts and
                // `dead_by_type` are computed from it.
                let (unreachable_items, dead_items): (Vec<DeadItem>, Vec<DeadItem>) = items
                    .into_iter()
                    .partition(|item| item.kind == DeadCodeKind::UnreachableCode);

                // Measure the file once and carry BOTH numbers. The percentage
                // used to be derived from a line count that was then thrown
                // away, and the renderer substituted the constant 100.
                let total_lines = self.count_file_lines(&file_path);
                let file_dead_percentage = file_percentage(total_lines, &dead_items);

                FileDeadCode {
                    file_path,
                    dead_items,
                    unreachable_items,
                    file_dead_percentage,
                    total_lines,
                }
            })
            .collect()
    }

    /// Physical line count for a file, or `None` when it cannot be read.
    ///
    /// `None` means "not measured" and must never be rendered as a number —
    /// see `contracts/pmat-no-fabrication-v1.yaml`, `measured_or_absent`.
    fn count_file_lines(&self, file_path: &Path) -> Option<usize> {
        let full_path = if file_path.is_absolute() {
            file_path.to_path_buf()
        } else {
            // The ABSOLUTE directory the rows are relative to. `project_path`
            // may itself be relative (`--path src/foo`), which would make this
            // join depend on the process's working directory.
            self.report_base.join(file_path)
        };

        std::fs::read_to_string(&full_path)
            .ok()
            .map(|content| content.lines().count())
    }

    /// Calculate overall metrics
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    async fn calculate_metrics(&self, files: Vec<FileDeadCode>) -> Result<AccurateDeadCodeReport> {
        let mut total_lines = 0;
        let mut total_files = 0;
        let mut project_files = 0;
        let mut dead_lines = 0;
        let mut dead_by_type = HashMap::new();
        let total_dead_items = files.iter().map(|f| f.dead_items.len()).sum();

        // Count lines in all Rust files. Use ignore::WalkBuilder so the walk
        // respects .gitignore and skips hidden dirs (e.g. `.claude/worktrees/`
        // git-worktree copies) — a raw walkdir here counted ~26M lines across
        // worktree duplicates, so the `total_files_analyzed` estimate
        // (total_lines / 100) ballooned to ~263k instead of ~4.2k.
        for entry in ignore::WalkBuilder::new(&self.project_path)
            .max_depth(Some(self.max_depth)) // limit traversal depth
            .hidden(true)
            .git_ignore(true)
            .git_global(true)
            .build()
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();

            // Belt-and-suspenders: also skip target/ explicitly.
            if path.starts_with(self.project_path.join("target")) {
                continue;
            }

            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                project_files += 1;
                // Layer 1 stopped scanning the excluded trees (#915), but this
                // walk kept counting them, so the totals described a wider set
                // than the dead items did: a default run printed "4273 files
                // analyzed, 0 with dead code" over 1236 test files it never
                // opened, and divided the dead lines by their lines too.
                if self.is_excluded_source(path) {
                    continue;
                }
                total_files += 1;
                if let Ok(content) = std::fs::read_to_string(path) {
                    total_lines += content.lines().count();
                }
            }
        }

        // Count dead lines and categorize by type. The line estimate uses the
        // SAME per-kind weights as the per-file figure (`estimated_dead_lines`)
        // — the two used to disagree (project total 94 from 5/3/2 weights vs
        // 76 from `items * 4` summed over the listed files), so the summary
        // contradicted the list underneath it.
        for file in &files {
            for item in &file.dead_items {
                let kind_str = dead_code_kind_to_str(&item.kind);
                *dead_by_type.entry(kind_str.to_string()).or_insert(0) += 1;
            }
            // Bounded by the file's own length. `estimated_dead_lines` charges
            // 5 lines per dead function, which is an estimate from an item
            // count, not a measured span — four dead one-line functions in a
            // five-line file estimated 20 dead lines. Unbounded, that summed
            // into a project figure of 400%, which `--fail-on-violation`
            // compared against its threshold and printed as a percentage. A
            // file cannot contain more dead lines than lines, so the bound goes
            // here, at the accumulation, rather than as a clamp on the ratio:
            // clamping the output would still leave `dead_lines` itself larger
            // than the code it describes.
            dead_lines += estimated_dead_lines_bounded(&file.dead_items, file.total_lines);
        }

        let dead_code_percentage = if total_lines > 0 {
            #[allow(clippy::cast_precision_loss)]
            let pct = (dead_lines as f64 / total_lines as f64) * 100.0;
            // The same ceiling `file_percentage` already applies. It was missing
            // here, so the project figure was the one surface that could report
            // an impossible percentage.
            pct.min(100.0)
        } else {
            0.0
        };

        Ok(AccurateDeadCodeReport {
            files_with_dead_code: files,
            total_dead_items,
            dead_code_percentage,
            total_lines,
            total_files,
            project_files,
            dead_lines,
            dead_by_type,
            // Filled in by `analyze`, which is the only caller that knows
            // whether Layer 2 ran. `None` is not "it ran".
            compiler_scan: None,
            cache: None,
        })
    }
}

/// Estimated dead lines for a set of dead items.
///
/// This is the single estimator for the whole command: a function or method is
/// charged 5 lines, a struct or enum 3, anything else 2. It is an estimate of
/// lines from a measured item count, not a measured line span — the summary and
/// the per-file rows must at least agree with each other.
pub(crate) fn estimated_dead_lines(items: &[DeadItem]) -> usize {
    items
        .iter()
        .map(|item| match item.kind {
            DeadCodeKind::Function | DeadCodeKind::Method => 5,
            DeadCodeKind::Struct | DeadCodeKind::Enum => 3,
            _ => 2,
        })
        .sum()
}

/// Estimated dead lines for one file, bounded by that file's own length.
///
/// The bound lives here rather than at the call sites because it is part of what
/// the estimate MEANS: `estimated_dead_lines` charges 5 lines per dead function
/// from an item count, not a measured span, so four dead one-line functions in a
/// five-line file estimate 20. Bounding at one call site and not the other is
/// how the summary came to print "Total dead lines: 20" for a 5-line file while
/// the project percentage had already been capped. `None` means the length is
/// unknown, and the raw estimate is the best available answer.
pub(crate) fn estimated_dead_lines_bounded(items: &[DeadItem], total_lines: Option<usize>) -> usize {
    let estimate = estimated_dead_lines(items);
    total_lines.map_or(estimate, |lines| estimate.min(lines))
}

/// Dead-code percentage for one file: estimated dead lines over the file's
/// measured line count. `0.0` when the line count is unavailable — the caller
/// reports `total_lines: null` alongside, so the zero is not read as a
/// measurement of "no dead code".
fn file_percentage(total_lines: Option<usize>, items: &[DeadItem]) -> f64 {
    match total_lines {
        Some(lines) if lines > 0 => {
            #[allow(clippy::cast_precision_loss)]
            let pct = (estimated_dead_lines(items) as f64 / lines as f64) * 100.0;
            pct.min(100.0)
        }
        _ => 0.0,
    }
}

/// Convert DeadCodeKind to string representation
fn dead_code_kind_to_str(kind: &DeadCodeKind) -> &str {
    match kind {
        DeadCodeKind::Function => "function",
        DeadCodeKind::Method => "method",
        DeadCodeKind::Struct => "struct",
        DeadCodeKind::Enum => "enum",
        DeadCodeKind::Variant => "variant",
        DeadCodeKind::Field => "field",
        DeadCodeKind::Constant => "constant",
        DeadCodeKind::Static => "static",
        DeadCodeKind::Module => "module",
        DeadCodeKind::Trait => "trait",
        DeadCodeKind::TypeAlias => "type_alias",
        DeadCodeKind::UnreachableCode => "unreachable",
        DeadCodeKind::Other(s) => s,
    }
}

#[cfg(test)]
mod unreachable_lint_tests {
    //! rustc reports UNUSED items and UNREACHABLE statements as two different
    //! lints. `parse_cargo_warnings` matched only `dead_code`, so nothing on the
    //! CLI path could ever produce an unreachable block and
    //! `analyze dead-code --include-unreachable` was inert on every input.
    use super::*;

    fn diagnostic(code: &str, message: &str, file: &str) -> String {
        serde_json::json!({
            "reason": "compiler-message",
            "message": {
                "code": { "code": code },
                "message": message,
                "spans": [{
                    "is_primary": true,
                    "file_name": file,
                    "line_start": 3,
                    "column_start": 5,
                    "text": [{ "text": "    let y = x * 2;" }],
                }],
            }
        })
        .to_string()
    }

    #[test]
    fn unreachable_code_warnings_are_collected_and_tagged() {
        let analyzer = CargoDeadCodeAnalyzer::new(std::path::Path::new("."));
        let output = format!(
            "{}\n{}\n",
            diagnostic("unreachable_code", "unreachable statement", "src/lib.rs"),
            diagnostic("dead_code", "function `helper` is never used", "src/lib.rs"),
        );

        let items = analyzer
            .parse_cargo_warnings(&output)
            .expect("parse cargo output");

        let unreachable: Vec<_> = items
            .iter()
            .filter(|(_, i)| i.kind == DeadCodeKind::UnreachableCode)
            .collect();
        assert_eq!(
            unreachable.len(),
            1,
            "the unreachable_code warning was dropped: {items:?}"
        );
        assert_eq!(unreachable[0].1.line, 3);
        assert_eq!(
            unreachable[0].1.name, "let y = x * 2;",
            "the source line is the only name rustc gives this lint"
        );
    }

    /// The split is what keeps a default run byte-identical: an unreachable
    /// finding must never reach `dead_items`, which every count and every
    /// estimated line total is computed from.
    #[test]
    fn grouping_keeps_unreachable_out_of_dead_items() {
        let analyzer = CargoDeadCodeAnalyzer::new(std::path::Path::new("."));
        let path = PathBuf::from("src/lib.rs");
        let items = vec![
            (
                path.clone(),
                DeadItem {
                    name: "helper".to_string(),
                    kind: DeadCodeKind::Function,
                    line: 10,
                    column: 1,
                    message: "`helper` is never used".to_string(),
                },
            ),
            (
                path.clone(),
                DeadItem {
                    name: "let y = 1;".to_string(),
                    kind: DeadCodeKind::UnreachableCode,
                    line: 3,
                    column: 5,
                    message: "unreachable statement".to_string(),
                },
            ),
        ];

        let grouped = analyzer.group_by_file(items);
        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped[0].dead_items.len(), 1);
        assert_eq!(grouped[0].unreachable_items.len(), 1);
        assert_eq!(
            estimated_dead_lines(&grouped[0].dead_items),
            5,
            "an unreachable statement must not be charged as dead lines"
        );
    }
}
