//! Find where a job actually invokes a command — directly in a `run:` block, or
//! one hop away through `make <target>` or a shell script.
//!
//! Indirection is where gates go to die: a workflow step reads
//! `make quality-gate`, the Makefile recipe reads `pmat quality-gate || exit 1`,
//! and nobody notices that the tool exited 0 while printing FAILED. Resolving
//! the hop is the difference between "a job mentions the gate" and "the gate
//! can fail this job".

use super::effect;
use super::workflow::Job;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// One resolved invocation site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    pub workflow: PathBuf,
    pub job_id: String,
    /// Step name, or `step #N` when the step is unnamed.
    pub step: String,
    /// How the command was reached: `run`, `make <target>`, `scripts/x.sh`.
    pub via: String,
    /// Reasons the exit code cannot reach the required check. Empty ⇒ enforcing.
    pub suppressions: Vec<String>,
    /// `None` ⇒ a full-roster run: this invocation covers every rule. `Some(ids)`
    /// when the invoking line carries a rule-subsetting flag: `--checks` yields
    /// the ids it names (upper-cased); every other `RULE_SUBSET_FLAGS` spelling
    /// yields `Some(vec![])` — known to restrict, attribution unknown.
    pub selected: Option<Vec<String>>,
}

impl Invocation {
    pub fn is_enforcing(&self) -> bool {
        self.suppressions.is_empty()
    }

    /// Does this invocation even name `id`? `selected` is `None` for a
    /// full-roster run — every rule is covered — or `Some(ids)` for a run
    /// restricted with `--checks`/`--only`/…, where only the named ids are.
    pub fn covers_rule(&self, id: &str) -> bool {
        match &self.selected {
            None => true,
            Some(ids) => ids.iter().any(|s| s.eq_ignore_ascii_case(id)),
        }
    }

    /// True when every suppression on this invocation is the "restricts the
    /// roster" sentence — nothing else stands between its exit code and the
    /// required check. `is_enforcing` requires *zero* suppressions (evidence
    /// for the whole roster); this is the weaker fact the graph's leaf edges
    /// and per-rule attribution need: a subset run can still be a live carrier
    /// for the rules it names.
    pub fn carries_selected_rules(&self) -> bool {
        self.suppressions.iter().all(|s| is_roster_restriction(s))
    }

    /// Does this invocation's exit code, if `id` fails, actually reach the
    /// required check? Covering the rule is necessary but not sufficient — a
    /// `continue-on-error` step that also carries `--checks CB-2100` covers
    /// CB-2100 and enforces nothing.
    pub fn enforces_rule(&self, id: &str) -> bool {
        self.covers_rule(id) && self.carries_selected_rules()
    }
}

/// Every invocation of `needle` reachable from this job's own steps.
///
/// `job_suppressions` are folded into every invocation found — a job-level
/// `continue-on-error` neuters everything inside it, however carefully the
/// individual step handles its own exit code.
pub fn find_in_job(
    project_path: &Path,
    job: &Job,
    needles: &[&str],
    job_suppressions: &[String],
) -> Vec<Invocation> {
    let mut out = Vec::new();
    for (i, step) in job.steps.iter().enumerate() {
        let Some(script) = step.run.as_deref() else {
            continue;
        };
        let label = step
            .name
            .clone()
            .unwrap_or_else(|| format!("step #{}", i + 1));
        let mut suppressions = job_suppressions.to_vec();
        if step.continue_on_error.suppresses() {
            suppressions.push(format!(
                "step `{label}` carries continue-on-error, so its failure never fails the job"
            ));
        }
        collect_from_script(
            project_path,
            job,
            &label,
            script,
            needles,
            &suppressions,
            &mut out,
            0,
        );
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn collect_from_script(
    project_path: &Path,
    job: &Job,
    label: &str,
    script: &str,
    needles: &[&str],
    inherited: &[String],
    out: &mut Vec<Invocation>,
    depth: usize,
) {
    if depth > 3 {
        return;
    }
    let via = if depth == 0 { "run" } else { "indirect" };
    for needle in needles {
        let Some(idx) = effect::find_line(script, needle) else {
            continue;
        };
        let mut suppressions = inherited.to_vec();
        suppressions.extend(effect::assess(script, idx));
        let line = script.lines().nth(idx).unwrap_or("");
        if let Some(elsewhere) = foreign_tree(line) {
            suppressions.push(format!(
                "runs comply against another tree (--path {elsewhere}), so its verdict is not \
                 evidence for this repository"
            ));
        }
        if let Some(file) = fixture_snapshot(line) {
            suppressions.push(format!(
                "judges GitHub-facing rules from a snapshot file (--github-snapshot {file}), so its \
                 verdict is not evidence for this repository"
            ));
        }
        let selected = parse_selected(line);
        if selected.is_some() {
            suppressions.push(roster_restriction_reason(&selected));
        }
        out.push(Invocation {
            workflow: job.workflow.clone(),
            job_id: job.id.clone(),
            step: label.to_string(),
            via: via.to_string(),
            suppressions,
            selected,
        });
    }
    for hop in indirect_targets(project_path, script) {
        // INV-2100-2 is a property of EVERY EDGE on the path, not of the
        // terminal node. A hop is an edge: `make comply || true` neuters the
        // gate without touching the recipe it calls, so the line that makes the
        // hop is judged exactly as the line the needle lands on is. Passing
        // `inherited` through unchanged here is what let a suppressed hop be
        // credited as enforcement.
        let mut carried = inherited.to_vec();
        carried.extend(effect::assess(script, hop.line));
        carried.extend(hop.extra);
        collect_from_script(
            project_path,
            job,
            &format!("{label} -> {}", hop.label),
            &hop.body,
            needles,
            &carried,
            out,
            depth + 1,
        );
    }
}

/// Flags that would make one `pmat comply check` run a subset of the rules.
/// None exist today; the list is here so that adding one cannot silently turn a
/// partial run into evidence for the whole roster.
const RULE_SUBSET_FLAGS: &[&str] = &["--checks", "--only", "--rules", "--skip", "--check-id"];

fn selects_a_rule_subset(line: &str) -> bool {
    RULE_SUBSET_FLAGS.iter().any(|f| line.contains(f))
}

/// Which rule ids (if any) a line's rule-subsetting flag names.
///
/// `None` when the line carries no `RULE_SUBSET_FLAGS` spelling at all — a
/// full-roster run. `Some(ids)` when it does: `--checks` is parsed for the ids
/// it names (`--checks a,b`, `--checks=a,b`, repeated `--checks` flags, and
/// whitespace-separated values up to the next token starting with `-`), each
/// trimmed and upper-cased. Every other `RULE_SUBSET_FLAGS` spelling yields
/// `Some(vec![])`: known to restrict the roster, but attribution to specific
/// ids is unknown, so it enforces no rule.
fn parse_selected(line: &str) -> Option<Vec<String>> {
    let code = line.split('#').next().unwrap_or(line);
    if !selects_a_rule_subset(code) {
        return None;
    }
    Some(parse_checks_flag(code).unwrap_or_default())
}

/// `--checks` specifically, parsed for the ids it names. `None` when the line
/// has no `--checks` flag (but may have another `RULE_SUBSET_FLAGS` spelling).
fn parse_checks_flag(code: &str) -> Option<Vec<String>> {
    let toks: Vec<&str> = code.split_whitespace().collect();
    let mut ids = Vec::new();
    let mut found = false;
    let mut i = 0;
    while i < toks.len() {
        let t = toks[i];
        if let Some(val) = t.strip_prefix("--checks=") {
            found = true;
            ids.extend(split_ids(val));
            i += 1;
            continue;
        }
        if t == "--checks" {
            found = true;
            i += 1;
            while i < toks.len() && !toks[i].starts_with('-') {
                ids.extend(split_ids(toks[i]));
                i += 1;
            }
            continue;
        }
        i += 1;
    }
    found.then_some(ids)
}

fn split_ids(s: &str) -> Vec<String> {
    s.split(',')
        .map(str::trim)
        .filter(|x| !x.is_empty())
        .map(str::to_uppercase)
        .collect()
}

/// The sentence recorded when an invocation's line restricts the roster, named
/// with the ids it selects when they are known.
fn roster_restriction_reason(selected: &Option<Vec<String>>) -> String {
    match selected {
        Some(ids) if !ids.is_empty() => format!(
            "the invocation restricts which rules run (--checks {}), so it cannot stand in for \
             the whole error-severity roster",
            ids.join(",")
        ),
        _ => "the invocation restricts which rules run, so it cannot stand in for the whole \
              error-severity roster"
            .to_string(),
    }
}

/// The `--path`/`-p` argument on an invoking line, when it points somewhere
/// other than this tree.
///
/// PMAT-719's control step runs the very same `pmat comply check --checks
/// CB-2113` line as the gate, against a throwaway fixture (`--path "$repo"`).
/// Read as an invocation of the rule it is one; read as evidence that the rule
/// is enforced ON THIS REPOSITORY it is nothing of the kind, and a ledger that
/// credited it would have kept CB-2113 "ENFORCED" after the real step was
/// deleted. `.`, `./`, `$PWD` and `$GITHUB_WORKSPACE` are this tree; anything
/// else — a variable, a temp dir, a sibling checkout — is another one.
/// The `--github-snapshot` argument on an invoking line, when present.
///
/// PMAT-722: CB-2115 can be judged from a snapshot file for its control; a
/// CI line that does so on the real tree judges a fixture, not GitHub, and
/// must not be credited as enforcing the rule.
fn fixture_snapshot(line: &str) -> Option<String> {
    let code = line.split('#').next().unwrap_or(line);
    let toks: Vec<&str> = code.split_whitespace().collect();
    let mut i = 0;
    while i < toks.len() {
        let t = toks[i];
        if let Some(v) = t.strip_prefix("--github-snapshot=") {
            return Some(v.to_string());
        }
        if t == "--github-snapshot" {
            return Some(
                toks.get(i + 1)
                    .map_or_else(|| "<missing>".to_string(), |v| (*v).to_string()),
            );
        }
        i += 1;
    }
    None
}

fn foreign_tree(line: &str) -> Option<String> {
    let code = line.split('#').next().unwrap_or(line);
    let toks: Vec<&str> = code.split_whitespace().collect();
    let mut i = 0;
    while i < toks.len() {
        let t = toks[i];
        let value = if let Some(v) = t.strip_prefix("--path=") {
            Some(v.to_string())
        } else if t == "--path" || t == "-p" {
            toks.get(i + 1).map(|v| (*v).to_string())
        } else {
            None
        };
        if let Some(v) = value {
            let bare = v.trim_matches(|c| c == '"' || c == '\'');
            let here = matches!(
                bare,
                "." | "./" | "$PWD" | "${PWD}" | "$GITHUB_WORKSPACE" | "${GITHUB_WORKSPACE}"
            );
            if !here {
                return Some(v);
            }
        }
        i += 1;
    }
    None
}

/// Does `reason` read as [`roster_restriction_reason`]? Used to tell "nothing
/// else suppresses this invocation" from "something genuinely does" —
/// [`Invocation::carries_selected_rules`] needs exactly that distinction.
pub fn is_roster_restriction(reason: &str) -> bool {
    reason.starts_with("the invocation restricts which rules run")
}

/// One resolvable hop out of a script: where it goes, and which line goes there.
///
/// The line index is the whole point of this type. Without it the recursion has
/// no way to judge the hop, and a suppression on the invoking line is invisible.
struct Hop {
    /// Index, in the **invoking** script, of the line that makes the hop.
    line: usize,
    /// `make comply`, `scripts/gate.sh` — how the body was reached.
    label: String,
    body: String,
    /// Reasons the hop cannot carry a failure that [`effect::assess`] cannot
    /// see, because they belong to the invoking *command* rather than to the
    /// shell line it sits on.
    extra: Vec<String>,
}

/// Bodies reachable one hop from `script`: Makefile recipes and shell scripts.
fn indirect_targets(project_path: &Path, script: &str) -> Vec<Hop> {
    let mut out = Vec::new();
    let makefile = load_makefile(project_path);
    for (n, line) in script.lines().enumerate() {
        let code = line.split('#').next().unwrap_or("");
        for call in make_calls(code) {
            if let Some(recipe) = makefile.get(&call.target) {
                out.push(Hop {
                    line: n,
                    label: format!("make {}", call.target),
                    body: recipe.clone(),
                    extra: call.extra,
                });
            }
        }
        if let Some(path) = script_path(code) {
            if let Ok(body) = std::fs::read_to_string(project_path.join(&path)) {
                out.push(Hop {
                    line: n,
                    label: path,
                    body,
                    extra: Vec::new(),
                });
            }
        }
    }
    out
}

/// A `make` invocation on one line: which target, and whether the invocation
/// itself declines to run it.
struct MakeCall {
    target: String,
    extra: Vec<String>,
}

/// Make's own "print, do not execute" spellings that [`effect::assess`] does
/// not already carry (`--dry-run` is in its `COMPILE_ONLY_FLAGS`). Matched only
/// against the tokens of the make invocation that owns them, so an `echo -n`
/// elsewhere on the line cannot trip it: this is INV-2100-6 one hop up, and a
/// false positive here would call a working gate broken.
const MAKE_DRY_RUN_FLAGS: &[&str] = &["-n", "--just-print", "--recon"];

fn make_calls(code: &str) -> Vec<MakeCall> {
    // Normalise before tokenising: `$(MAKE)` must survive the split on `(`.
    let normalized = code.replace("$(MAKE)", "make").replace("${MAKE}", "make");
    let toks = words(&normalized);
    let mut out = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        if !is_make(t) {
            continue;
        }
        let mut extra = Vec::new();
        for cand in toks.iter().skip(i + 1) {
            if MAKE_DRY_RUN_FLAGS.contains(cand) {
                extra.push(format!(
                    "`make {cand}` prints the recipe instead of running it, so the hop is not \
                     evidence that the rule ran"
                ));
                continue;
            }
            if cand.starts_with('-') || cand.contains('=') {
                continue;
            }
            out.push(MakeCall {
                target: (*cand).to_string(),
                extra,
            });
            break;
        }
    }
    out
}

/// A leading `-` is a Makefile recipe prefix, not part of the command name.
/// `-$(MAKE) comply` is still a call to make, and refusing to recognise it
/// means the hop is never followed — so CB-2100 reports "no invocation" where
/// the truth is "the hop ignores its exit code". Both fail; only one is honest.
fn is_make(tok: &str) -> bool {
    tok.trim_start_matches('-') == "make"
}

/// The command words of a shell line.
///
/// Splitting on whitespace alone is not enough. `if make comply; then` yields
/// `comply;` and `OUT=$(make comply)` yields `$(make`, so in both shapes the
/// hop is silently not followed and the suppression on it is never named. A
/// verdict that is right for the wrong reason goes green the day the reason
/// changes.
fn words(code: &str) -> Vec<&str> {
    code.split(|c: char| c.is_whitespace() || matches!(c, ';' | '&' | '|' | '(' | ')' | '`'))
        .filter(|w| !w.is_empty())
        .collect()
}

fn script_path(code: &str) -> Option<String> {
    words(code)
        .into_iter()
        .map(|t| t.trim_start_matches("./"))
        .find(|t| t.ends_with(".sh") && !t.starts_with('-'))
        .map(str::to_string)
}

/// Very small Makefile reader: target name → recipe body. Enough to follow a
/// workflow step into a recipe; not a Make implementation.
fn load_makefile(project_path: &Path) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Ok(text) = std::fs::read_to_string(project_path.join("Makefile")) else {
        return map;
    };
    let mut current: Option<String> = None;
    let mut body = String::new();
    for line in text.lines() {
        if line.starts_with('\t') {
            if current.is_some() {
                body.push_str(line.trim_start_matches('\t').trim_start_matches('@'));
                body.push('\n');
            }
            continue;
        }
        if let Some(name) = current.take() {
            map.insert(name, std::mem::take(&mut body));
        }
        current = target_name(line);
    }
    if let Some(name) = current {
        map.insert(name, body);
    }
    map
}

fn target_name(line: &str) -> Option<String> {
    let colon = line.find(':')?;
    if line.starts_with(' ') || line.starts_with('#') {
        return None;
    }
    let name = line[..colon].trim();
    let after = line[colon..].chars().nth(1);
    if after == Some('=') || name.is_empty() || name.contains(' ') {
        return None;
    }
    Some(name.to_string())
}
