//! The roster of CB rules, and where each one is defined.
//!
//! The population is **not** computed here. `pmat` already has one authority on
//! "which rules exist" — the clause ids the check builders register with
//! `filter_check_by_config`, enumerated by
//! [`crate::cli::handlers::comply_handlers::check_evidence_gates::enumerate_comply_rule_ids`] and
//! used by CB-1703 to hold the documentation to the registry. A second
//! implementation of "count the rules" is the one-rule-two-answers defect this
//! repository keeps finding in itself, so this module reuses that one and adds
//! only what the ledger needs on top: a title and a `file:line` citation per id.
//!
//! Deriving the population from `ComplyConfig` instead would have been easier
//! and wrong — the config declares the handful of ids `.pmat.yaml` overrides,
//! not the rules that run.
//!
//! A registered rule whose definition site cannot be found still gets a row,
//! with an empty citation. That is a finding — a rule nobody can point at — and
//! dropping it would hide exactly the population this rule exists to surface.
//!
//! Fails closed: an unenumerable registry is a failure, never a clean sheet.

use std::path::{Path, PathBuf};

/// Where the comply checks that name themselves live.
pub const HANDLER_DIR: &str = "src/cli/handlers/comply_handlers";

/// Where the severities are declared. Scanned too, so that a rule configured
/// but not yet implemented still gets a row rather than vanishing.
pub const CONFIG_FILE: &str = "src/models/comply_config_defaults.rs";

/// One rule, and where it is defined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    /// `CB-2100`, `CB-050-A`, …
    pub id: String,
    /// Everything after the colon, trimmed. Empty when the literal had none.
    pub title: String,
    /// Repo-relative path of the file the id was found in.
    pub file: PathBuf,
    /// 1-indexed line.
    pub line: usize,
}

impl Rule {
    /// `file:line`, or a marker when the rule is registered and undefined.
    pub fn citation(&self) -> String {
        if self.line == 0 {
            return "no definition site found".to_string();
        }
        format!("{}:{}", self.file.display(), self.line)
    }

    /// The config key a rule is filtered by: `CB-2100` → `cb-2100`.
    pub fn config_key(&self) -> String {
        self.id.to_lowercase()
    }

    /// Sort key that orders `CB-9` before `CB-10` and keeps suffixed variants
    /// next to their parent.
    fn sort_key(&self) -> (u32, String) {
        let digits: String = self
            .id
            .trim_start_matches("CB-")
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        (digits.parse().unwrap_or(u32::MAX), self.id.clone())
    }
}

/// Does this repository *define* CB rules at all?
///
/// The enforcement ledger is an obligation for the repository that ships the
/// rules, not for the repositories that run them. A fleet repo with no
/// `comply_handlers/` tree has no roster to account for — but a repo that has
/// the tree and yields no rules from it is a measurement failure, and
/// [`crate::services::gate_effect::ledger::rows`] fails closed on exactly that.
pub fn defines_rules(project_path: &Path) -> bool {
    project_path.join(HANDLER_DIR).is_dir()
}

/// Every registered rule, sorted by id, each carrying its definition site when
/// one can be found.
pub fn collect(project_path: &Path) -> Vec<Rule> {
    let Some(registered) =
        crate::cli::handlers::comply_handlers::check_evidence_gates::enumerate_comply_rule_ids(
            project_path,
        )
    else {
        return Vec::new(); // fail closed: the caller turns an empty roster into an error
    };
    let sites = definition_sites(project_path);
    let mut rules: Vec<Rule> = registered
        .iter()
        .map(|key| {
            sites
                .iter()
                .find(|r| r.config_key() == *key)
                .cloned()
                .unwrap_or_else(|| Rule {
                    id: key.to_uppercase(),
                    title: String::new(),
                    file: PathBuf::new(),
                    line: 0,
                })
        })
        .collect();
    rules.sort_by_key(Rule::sort_key);
    rules
}

/// Candidate definition sites, scanned out of the handler sources. Only used to
/// annotate ids the registry already vouched for.
fn definition_sites(project_path: &Path) -> Vec<Rule> {
    let dir = project_path.join(HANDLER_DIR);
    let mut files = Vec::new();
    walk(&dir, &mut files);
    files.sort();
    let config = project_path.join(CONFIG_FILE);
    if config.is_file() {
        files.push(config);
    }

    let mut rules: Vec<Rule> = Vec::new();
    for path in files {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let rel = path
            .strip_prefix(project_path)
            .unwrap_or(&path)
            .to_path_buf();
        for (n, line) in text.lines().enumerate() {
            // A rule id inside a test fixture is not a rule. `CB-001: test
            // issue` is an assertion's payload, and listing it would put rows
            // in the ledger that no check ever emits.
            if line.contains("#[cfg(test)]") {
                break;
            }
            rules.extend(rules_in_line(line, &rel, n + 1));
        }
    }
    // Ties prefer a declaration that carries a title (the check naming itself)
    // over a bare config key, so the citation points at the rule rather than at
    // its severity. Sorting is stable and files were sorted first, so the
    // result does not depend on directory iteration order.
    rules.sort_by(|a, b| {
        a.sort_key()
            .cmp(&b.sort_key())
            .then(a.title.is_empty().cmp(&b.title.is_empty()))
            .then(a.line.cmp(&b.line))
    });
    rules.dedup_by(|a, b| a.id == b.id);
    rules
}

/// A rule the registry knows about but no source line names.
impl Rule {
    pub fn has_citation(&self) -> bool {
        self.line > 0
    }
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") && !is_test_file(&p) {
            out.push(p);
        }
    }
}

/// Rule ids declared on one line.
///
/// Two declaration forms, both **inside a string literal** — a rule is a thing
/// the code says out loud, not a thing a comment mentions:
///
/// * `"CB-2100: Comply Gate Effect"` — the name a check reports itself under.
///   The colon is what separates a declaration from a cross-reference.
/// * `"cb-2100"` — the config key a severity is declared against.
///
/// This is a *lower bound* on the roster, and the ledger says so. A rule that
/// never names itself in either form cannot be attributed to a file:line at
/// all, and inventing a row for it would be the fabrication this rule exists to
/// prevent.
fn rules_in_line(line: &str, file: &Path, n: usize) -> Vec<Rule> {
    let mut out = Vec::new();
    for literal in string_literals(line) {
        if let Some(id) = config_key_id(&literal) {
            out.push(Rule {
                id,
                title: String::new(),
                file: file.to_path_buf(),
                line: n,
            });
            continue;
        }
        out.extend(named_rules(&literal, file, n));
    }
    out
}

/// The contents of every `"…"` on the line.
fn string_literals(line: &str) -> Vec<String> {
    line.split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect()
}

/// `"cb-2100"` → `CB-2100`. Whole-literal match only.
fn config_key_id(literal: &str) -> Option<String> {
    let rest = literal.strip_prefix("cb-")?;
    let (digits, suffix) = match rest.split_once('-') {
        Some((d, s)) => (d, Some(s)),
        None => (rest, None),
    };
    if digits.is_empty() || digits.len() > 4 || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    match suffix {
        None => Some(format!("CB-{digits}")),
        Some(s) if !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric()) => {
            Some(format!("CB-{digits}-{}", s.to_uppercase()))
        }
        Some(_) => None,
    }
}

fn named_rules(literal: &str, file: &Path, n: usize) -> Vec<Rule> {
    let mut out = Vec::new();
    let bytes = literal.as_bytes();
    let mut at = 0usize;
    while let Some(rel) = literal[at..].find("CB-") {
        let start = at + rel;
        at = start + 3;
        if start > 0 && is_ident(bytes[start - 1]) {
            continue;
        }
        let Some((id, after)) = parse_id(literal, start) else {
            continue;
        };
        at = after;
        if !literal[after..].starts_with(':') {
            continue;
        }
        out.push(Rule {
            id,
            title: title_after(&literal[after + 1..]),
            file: file.to_path_buf(),
            line: n,
        });
    }
    out
}

/// Files that exist to test the checks, not to declare them. This repository
/// keeps them beside the handlers (`check_binding_scope_tests_kani.rs`), and
/// their fixtures are full of ids like `CB-001: test issue`.
fn is_test_file(p: &Path) -> bool {
    p.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.contains("test"))
}

fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-'
}

/// `CB-` at `start`; returns the full id and the index just past it.
fn parse_id(line: &str, start: usize) -> Option<(String, usize)> {
    let rest = &line[start + 3..];
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() || digits.len() > 4 {
        return None;
    }
    let mut end = start + 3 + digits.len();
    let tail = &line[end..];
    if let Some(sfx) = tail.strip_prefix('-') {
        let suffix: String = sfx
            .chars()
            .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
            .collect();
        if !suffix.is_empty() {
            end += 1 + suffix.len();
            return Some((format!("CB-{digits}-{suffix}"), end));
        }
    }
    Some((format!("CB-{digits}"), end))
}

/// The human title, up to the end of the string literal it lives in.
fn title_after(rest: &str) -> String {
    rest.split(" ({")
        .next()
        .unwrap_or("")
        .trim()
        .trim_end_matches('\\')
        .trim()
        .to_string()
}
