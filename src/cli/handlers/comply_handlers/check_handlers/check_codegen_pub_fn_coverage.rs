// CB-1637: every L2+ ticket's modified `pub fn`-bearing file carries a matching
// `#[pmat_work_contract(id = ...)]` attribute.
// Included from check_codegen.rs — do NOT add `use` imports or `#!` attributes here.

/// CB-1637 (L2+): every modified `.rs` file under `src/` that declares a
/// `pub fn` must carry at least one `#[pmat_work_contract(id = "<ticket-id>")]`
/// attribute in that same file. This is the coverage gate for Component 30's
/// `pmat work codegen` output: once codegen wraps target functions, every
/// public surface touched by an L2+ ticket should be contracted.
///
/// The check is file-level rather than function-level: as long as the file
/// contains *one* matching attribute, every `pub fn` in it is presumed
/// covered. A later check (when codegen lands proper AST awareness) can
/// tighten this to per-function matching.
///
/// # Skip semantics (tiered)
///
/// * `.pmat-work/` absent                                   → Skip
/// * no `.pmat-work/<ID>/contract.json` tickets present     → Skip
/// * no ticket targets L2 or higher                         → Skip
/// * no L2+ ticket has `modified-files.json`                → Skip
/// * no modified file under `src/**/*.rs` declares `pub fn` → Skip
///
/// # Pass
///
/// Every in-scope `pub fn`-bearing modified file has a matching attribute.
///
/// # Fail
///
/// Any in-scope file declares `pub fn` but lacks a matching
/// `#[pmat_work_contract(id = "<ticket-id>")]` attribute.
pub(crate) fn check_l2_public_fn_coverage(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1637: L2+ Public Function Coverage";
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/` directory present".into(),
            severity: Severity::Info,
        };
    }
    let Ok(entries) = std::fs::read_dir(&work_dir) else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "Unable to read `.pmat-work/`".into(),
            severity: Severity::Info,
        };
    };

    let mut saw_any_ticket = false;
    let mut saw_any_l2_plus = false;
    let mut saw_any_modified = false;
    let mut evaluated_files = 0usize;
    let mut violations: Vec<String> = Vec::new();

    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let Some(ticket_id) = entry.file_name().to_str().map(String::from) else {
            continue;
        };
        if ticket_id.starts_with('.') || ticket_id == "ledger" || ticket_id == "codegen" {
            continue;
        }
        let Some(contract) = load_contract_json(project_path, &ticket_id) else {
            continue;
        };
        saw_any_ticket = true;
        if !contract_level_at_least(&contract, 2) {
            continue;
        }
        saw_any_l2_plus = true;
        let Some(modified) = load_modified_files(project_path, &ticket_id) else {
            continue;
        };
        saw_any_modified = true;

        for rel in &modified {
            if !rel.starts_with("src/") || !rel.ends_with(".rs") {
                continue;
            }
            let abs = project_path.join(rel);
            let Ok(text) = std::fs::read_to_string(&abs) else {
                continue;
            };
            if !file_has_pub_fn(&text) {
                continue;
            }
            evaluated_files += 1;
            if !file_has_attribute_for_ticket(&text, &ticket_id) {
                violations.push(format!("  {} → {}", ticket_id, rel));
            }
        }
    }

    if !saw_any_ticket {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/<ID>/contract.json` tickets present".into(),
            severity: Severity::Info,
        };
    }
    if !saw_any_l2_plus {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No ticket targets L2 or higher".into(),
            severity: Severity::Info,
        };
    }
    if !saw_any_modified {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message:
                "No L2+ ticket has `modified-files.json` — work CLI has not emitted diff receipts yet"
                    .into(),
            severity: Severity::Info,
        };
    }
    if evaluated_files == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No L2+ ticket modified a `src/**/*.rs` file declaring `pub fn`".into(),
            severity: Severity::Info,
        };
    }

    if !violations.is_empty() {
        let mut msg = format!(
            "{} file(s) declare `pub fn` without matching `#[pmat_work_contract(id = ...)]`:\n",
            violations.len()
        );
        let preview: Vec<&String> = violations.iter().take(5).collect();
        for line in preview {
            msg.push_str(line);
            msg.push('\n');
        }
        if violations.len() > 5 {
            msg.push_str(&format!("  …and {} more\n", violations.len() - 5));
        }
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: msg,
            severity: Severity::Error,
        };
    }

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        message: format!(
            "{} file(s): every `pub fn`-bearing modified file has a matching contract attribute",
            evaluated_files
        ),
        severity: Severity::Info,
    }
}

/// Line-anchored scan for a public function signature. Matches `pub fn`,
/// `pub(crate) fn`, `pub(super) fn`, and modifier combinations such as
/// `pub async fn`, `pub unsafe fn`, `pub const fn`, `pub extern "C" fn`.
/// The character class `[ \t]` (not `\s`) keeps the match on a single line
/// so signatures split across lines do not false-positive off of arbitrary
/// tokens on adjacent lines.
///
/// The pattern is a compile-time literal, so its compilation cannot depend on
/// runtime input. `LazyLock` compiles it once for the process (the previous
/// form rebuilt it on every call) and `pub_fn_regex_compiles` forces the cell
/// so a malformed edit to the literal fails the test suite rather than the
/// production run. Returning `false` on a compile error was not an option: it
/// would report "this file declares no public function", turning a broken
/// checker into a gate that passes everything.
fn file_has_pub_fn(text: &str) -> bool {
    static PUB_FN_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        Regex::new(
            r#"(?m)^[ \t]*pub(\([^)]*\))?(?:[ \t]+(?:async|unsafe|const|safe|extern(?:[ \t]+"[^"]*")?))*[ \t]+fn[ \t]+\w+"#,
        )
        .expect("CB-1637 pub-fn regex is a literal; compilation is proven by pub_fn_regex_compiles")
    });
    PUB_FN_RE.is_match(text)
}

/// Return true iff `text` contains at least one
/// `#[pmat_work_contract(id = "<ticket_id>", …)]` attribute. Uses the same
/// line-anchored regex as `collect_attribute_usages` so raw-string fixtures
/// are ignored.
fn file_has_attribute_for_ticket(text: &str, ticket_id: &str) -> bool {
    let (attr_rx, id_rx, _, _) = attribute_parser();
    for cap in attr_rx.captures_iter(text) {
        let Some(body) = cap.get(1) else { continue };
        if let Some(idcap) = id_rx.captures(body.as_str()) {
            if idcap.get(1).map(|m| m.as_str()) == Some(ticket_id) {
                return true;
            }
        }
    }
    false
}
