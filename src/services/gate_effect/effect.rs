//! INV-2100-2: "a reachable invocation whose failure cannot propagate is NOT
//! reachable".
//!
//! Running a gate is not the same as being gated by it. This module decides,
//! for one shell line that invokes the rule, whether a non-zero exit from that
//! line can still fail the job. The ways it cannot:
//!
//! 1. `continue-on-error` at job or step level (handled by the caller, which
//!    owns the YAML);
//! 2. the exit code is **suppressed** on the spot — `|| true`, `|| :`,
//!    `|| echo …`, a leading `-` in a Makefile recipe;
//! 3. the exit code is **captured but never compared** — assigned to a
//!    variable, consumed by `if`, swallowed by a pipeline without `pipefail`,
//!    or disarmed by `set +e`, with nothing downstream that exits non-zero;
//! 4. the wrapper **prints a failure verdict and then exits 0** (INV-2100-4);
//! 5. the line is **dead**: an earlier line in the same script fails
//!    unconditionally under errexit, so the invocation never runs (INV-2100-5);
//! 6. the invocation **compiles without executing** — `--no-run` and friends
//!    build the thing whose execution was supposed to be the evidence
//!    (INV-2100-6).
//!
//! GitHub runs `run:` blocks under `bash -e` by default — errexit on, but
//! **pipefail off** — so a piped invocation loses its status unless the script
//! turns pipefail on itself.

/// Why an invocation's failure cannot reach the required check.
/// Empty vector ⇒ the failure propagates.
pub type Suppressions = Vec<String>;

/// Assess the line at `idx` of `script`, which is known to contain the rule
/// invocation. Returns every reason the exit code cannot propagate.
#[provable_contracts_macros::contract(
    "comply-gate-effect-v1.yaml",
    equation = "failure_propagation"
)]
pub fn assess(script: &str, idx: usize) -> Suppressions {
    let lines: Vec<&str> = script.lines().collect();
    let Some(line) = lines.get(idx) else {
        return vec!["invocation line index out of range".into()];
    };
    let mut reasons = Suppressions::new();
    if let Some(r) = suppressed_on_the_spot(line) {
        reasons.push(r);
    }
    if let Some(r) = captured_never_compared(&lines, idx) {
        reasons.push(r);
    }
    if let Some(r) = compiles_without_executing(line) {
        reasons.push(r);
    }
    if let Some(r) = dead_after_unconditional_failure(&lines, idx) {
        reasons.push(r);
    }
    if let Some(r) = prints_failure_but_exits_zero(&lines, idx) {
        reasons.push(r);
    }
    reasons
}

/// INV-2100-6: flags that build the thing without running it. A job that
/// *compiles* a test suite establishes that it compiles, and nothing else — it
/// is not evidence that any assertion in it ever executed.
const COMPILE_ONLY_FLAGS: &[&str] = &["--no-run", "--dry-run", "--list"];

fn compiles_without_executing(line: &str) -> Option<String> {
    let code = line.split('#').next().unwrap_or("");
    let flag = COMPILE_ONLY_FLAGS.iter().find(|f| {
        code.split_whitespace()
            .any(|t| t.trim_end_matches(',') == **f)
    })?;
    Some(format!(
        "`{flag}` builds the invocation without executing it, so it cannot be evidence that the \
         rule ran"
    ))
}

/// INV-2100-5: an invocation that can never be reached can never fail anything.
/// Under `bash -e` an unconditional `false` or a non-zero `exit` ends the
/// script, so everything after it is decoration.
fn dead_after_unconditional_failure(lines: &[&str], idx: usize) -> Option<String> {
    if errexit_disabled_before(lines, idx) {
        return None; // handled, and named differently, by `captured_never_compared`
    }
    let (n, killer) = first_unconditional_failure_in(&lines[..idx])?;
    Some(format!(
        "line {} (`{killer}`) always fails and errexit is on, so the invocation on line {} never \
         runs",
        n + 1,
        idx + 1
    ))
}

/// The first line of `script` that fails every time it is reached, at the
/// script's **top level**.
///
/// Depth matters, and getting it wrong is the failure mode that would discredit
/// the whole rule: `if …; then exit 1; fi` is a working gate, not a job that
/// can never succeed. Only a failure nothing guards counts.
pub fn first_unconditional_failure(script: &str) -> Option<(usize, String)> {
    let lines: Vec<&str> = script.lines().collect();
    first_unconditional_failure_in(&lines)
}

fn first_unconditional_failure_in(lines: &[&str]) -> Option<(usize, String)> {
    let mut depth: usize = 0;
    for (n, raw) in lines.iter().enumerate() {
        let t = raw.split('#').next().unwrap_or("").trim();
        if t.is_empty() {
            continue;
        }
        // A line that opens a block is itself conditional, even when it also
        // closes it: `if …; then exit 1; fi` is a gate, not a dead job.
        let opens = openers(t);
        let guarded = depth > 0 || opens > 0;
        depth = (depth + opens).saturating_sub(closers(t));
        if guarded || t.contains("||") || t.contains("&&") {
            continue;
        }
        if t == "false" || exit_nonzero(t) {
            return Some((n, t.to_string()));
        }
    }
    None
}

fn openers(t: &str) -> usize {
    t.split_whitespace()
        .filter(|w| matches!(*w, "if" | "for" | "while" | "until" | "case"))
        .count()
}

fn closers(t: &str) -> usize {
    t.split_whitespace()
        .filter(|w| matches!(*w, "fi" | "done" | "esac"))
        .count()
}

/// INV-2100-4: the wrapper announces a failure and then exits 0.
///
/// This is the shape the `pmat quality-gate --perf` defect had: the verdict was
/// printed, the exit code was 0, and the job went green. Only an *explicit*
/// `exit 0` counts — a script that merely ends after printing may still be
/// carrying a non-zero status from the invocation itself.
fn prints_failure_but_exits_zero(lines: &[&str], idx: usize) -> Option<String> {
    let tail = &lines[idx..];
    if downstream_exits_nonzero(lines, idx) {
        return None;
    }
    let verdict = tail.iter().find(|l| prints_failure_verdict(l))?;
    if !tail.iter().any(|l| l.trim().starts_with("exit 0")) {
        return None;
    }
    Some(format!(
        "the script prints a failure verdict (`{}`) and then exits 0, so a failed run reports \
         success",
        verdict.trim()
    ))
}

/// Tokens a command uses to say it failed. Deliberately narrow: a false
/// positive here would call a working gate broken, and this rule's credibility
/// rests on never doing that.
const FAILURE_VERDICT_MARKERS: &[&str] =
    &["FAILED", "❌", "::error::", "NOT COMPLIANT", "VIOLATION"];

fn prints_failure_verdict(line: &str) -> bool {
    let code = line.split('#').next().unwrap_or("");
    let printing = ["echo", "printf", "cat"]
        .iter()
        .any(|c| code.trim_start().starts_with(c));
    printing && FAILURE_VERDICT_MARKERS.iter().any(|m| code.contains(m))
}

/// `|| true`, `|| :`, `|| echo …`, `|| exit 0`, or a Makefile recipe's leading `-`.
fn suppressed_on_the_spot(line: &str) -> Option<String> {
    let t = line.trim();
    if t.starts_with('-') && !t.starts_with("--") {
        return Some(
            "Makefile recipe line is prefixed with `-`, which ignores its exit code".into(),
        );
    }
    let after = t.split("||").nth(1)?.trim();
    let head = after.split_whitespace().next().unwrap_or("");
    let benign = matches!(head, "true" | ":" | "echo" | "printf" | "warn" | "cat");
    if benign {
        return Some(format!("exit code suppressed by `|| {head}`"));
    }
    if after.starts_with("exit 0") {
        return Some("exit code suppressed by `|| exit 0`".into());
    }
    None
}

/// The exit code is taken somewhere it stops being fatal, and nothing
/// downstream turns it back into a non-zero exit.
fn captured_never_compared(lines: &[&str], idx: usize) -> Option<String> {
    let line = lines[idx].trim();
    let kind = capture_kind(line, lines, idx)?;
    if downstream_exits_nonzero(lines, idx) {
        return None;
    }
    Some(format!("{kind}, and no later line exits non-zero on it"))
}

fn capture_kind(line: &str, lines: &[&str], idx: usize) -> Option<&'static str> {
    if is_assignment_capture(line) {
        return Some("exit code captured into a variable");
    }
    if line.starts_with("if ") || line.starts_with("if(") {
        return Some("exit code consumed by `if`");
    }
    if piped_without_pipefail(line, lines) {
        return Some("exit code lost to a pipeline (bash -e does not set pipefail)");
    }
    if errexit_disabled_before(lines, idx) {
        return Some("errexit disabled by an earlier `set +e`");
    }
    None
}

/// `VAR=$(cmd)` / `VAR=`cmd`` / `export VAR=$(cmd)`.
fn is_assignment_capture(line: &str) -> bool {
    let t = line
        .trim_start_matches("export ")
        .trim_start_matches("local ");
    let Some(eq) = t.find('=') else { return false };
    let name = &t[..eq];
    !name.is_empty()
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && (t[eq..].contains("$(") || t[eq..].contains('`'))
}

fn piped_without_pipefail(line: &str, lines: &[&str]) -> bool {
    let piped = line.split("||").next().unwrap_or(line).split('|').count() > 1;
    piped && !lines.iter().any(|l| l.contains("pipefail"))
}

fn errexit_disabled_before(lines: &[&str], idx: usize) -> bool {
    lines[..idx].iter().any(|l| {
        let t = l.trim();
        t == "set +e" || t.starts_with("set +e ") || t.contains("set +eo") || t == "set +ex"
    })
}

/// Something after the invocation can still end the job non-zero: an explicit
/// `exit <non-zero>`, a `$?` test, or a `::error::` annotation paired with an
/// exit. Conservative on purpose — a false "it propagates" is the failure mode
/// this whole rule exists to catch, so only literal non-zero exits count.
fn downstream_exits_nonzero(lines: &[&str], idx: usize) -> bool {
    lines[idx..].iter().skip(1).any(|l| {
        let t = l.trim();
        exit_nonzero(t) || t.starts_with("false") || t.contains("&& exit 1")
    })
}

fn exit_nonzero(t: &str) -> bool {
    let Some(rest) = t.strip_prefix("exit") else {
        return false;
    };
    let arg = rest.trim().trim_end_matches(';').trim();
    !arg.is_empty() && arg != "0"
}

/// Index of the first line of `script` containing `needle`, if any.
pub fn find_line(script: &str, needle: &str) -> Option<usize> {
    script.lines().position(|l| line_invokes(l, needle))
}

/// A line invokes `needle` only when it is not commented out.
fn line_invokes(line: &str, needle: &str) -> bool {
    let code = line.split('#').next().unwrap_or("");
    code.contains(needle)
}
