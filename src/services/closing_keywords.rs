//! GitHub closing keywords — find them, and neutralise the ones pmat did not mean (PMAT-900001).
//!
//! GitHub closes an issue when a merged PR body, or a commit that reaches the
//! default branch, puts `close`/`fix`/`resolve` (any of `-s`/`-d`, any case,
//! optionally followed by a colon) directly before `#N` or `owner/repo#N`. It
//! reads no word boundary a human would: `no-close: #3091` closed aprender#3091,
//! and a PR body closed #1339 here (1cdffdcca). pmat interpolated free text
//! (roadmap titles, analysis output) into commit subjects and issue bodies, so
//! pmat could close an issue nobody asked it to.
//!
//! One predicate, two renderings: [`PATTERN`] here, and the same ERE in
//! `closing_keywords_lint.sh` ([`LINT_SH`]), which every commit-msg hook pmat
//! writes and CI's PR-body lint source. The differential test at the bottom
//! runs both over one fixture table.
//!
//! The one sanctioned form is a line that STARTS with `Closes #N` — this
//! repository's merge flow writes it (44 of the last 3000 commit bodies). The
//! rest of that line is still judged.

use regex::Regex;
use std::sync::LazyLock;

/// The closing-reference ERE, shared verbatim with [`LINT_SH`] (`grep -iE`
/// under `LC_ALL=C`). ASCII classes only, so both engines agree on bytes.
pub const PATTERN: &str = r"(^|[^[:alnum:]_])(close[sd]?|fix(e[sd])?|resolve[sd]?):?[[:space:]]*([[:alnum:]_.-]+/[[:alnum:]_.-]+)?#[0-9]+";

/// The shell rendering: `pmat_closing_keyword_hits` and
/// `pmat_closing_keyword_commit_msg_lint`. Spliced into the commit-msg hooks.
pub const LINT_SH: &str = include_str!("closing_keywords_lint.sh");

/// The placeholder a hook template carries where [`LINT_SH`] goes.
pub const LINT_PLACEHOLDER: &str = "__PMAT_CLOSING_KEYWORD_LINT__";

static CLOSING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!("(?i){PATTERN}")).expect("PATTERN is a valid regex"));

static SANCTIONED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^Closes #[0-9]+").expect("valid regex"));

/// One line that would close an issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    /// 1-based line number.
    pub line: usize,
    /// The whole line, as written.
    pub text: String,
}

/// Every line of `text` holding a closing reference, after the sanctioned
/// `Closes #N` line prefix is set aside.
pub fn find(text: &str) -> Vec<Hit> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| CLOSING.is_match(&SANCTIONED.replace(line, "")))
        .map(|(i, line)| Hit {
            line: i + 1,
            text: line.to_string(),
        })
        .collect()
}

/// `text` with every closing reference broken by inserting `issue ` before
/// the reference: `fixes #5` becomes `fixes issue #5`. The author's meaning
/// survives and GitHub no longer reads a close (grill D4). A sanctioned
/// `Closes #N` line prefix is left alone; pmat-written text never starts
/// with one, so neutralising interpolated text cannot mint a close.
pub fn neutralise(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        let keep = SANCTIONED.find(line).map_or(0, |m| m.end());
        out.push_str(&line[..keep]);
        out.push_str(&neutralise_fragment(&line[keep..]));
    }
    out
}

fn neutralise_fragment(fragment: &str) -> String {
    let mut out = String::with_capacity(fragment.len() + 8);
    let mut rest = fragment;
    // Each search resumes just after a reference's `#`, where a digit stands,
    // so the pattern's `^` alternative cannot match anywhere but the start.
    while let Some(caps) = CLOSING.captures(rest) {
        let whole = caps.get(0).expect("group 0");
        let hash = whole.start() + whole.as_str().rfind('#').expect("a match holds '#'");
        // The reference starts at the optional owner/repo, else at `#`.
        let reference = caps.get(4).map_or(hash, |m| m.start());
        out.push_str(&rest[..reference]);
        out.push_str("issue ");
        out.push_str(&rest[reference..=hash]);
        rest = &rest[hash + 1..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fixture table both renderings are judged over. `true` = closes.
    const FIXTURES: &[(&str, bool)] = &[
        ("no-close: #3091", true),
        ("No-Close #12", true),
        ("this fixes #5", true),
        ("re-fixes: #7", true),
        ("fixes owner/repo#9", true),
        ("Resolved #44", true),
        ("feat: parse the header (closes #8)", true),
        ("fixed:#3", true),
        ("Closes #1", false),
        ("Closes #1 and fixes #2", true),
        ("keeps-open #3091", false),
        ("prefix #12", false),
        ("fixture #3", false),
        ("fixes issue #5", false),
        ("Refs PMAT-900001", false),
        (
            "Merge pull request #1390 from paiml/PMAT-1336-lifecycle-4",
            false,
        ),
        ("fix: vendor the whole release protocol (#1234)", false),
        ("closes the gap noted in #4", false),
        ("unclosed #6", false),
        ("closes_#6", false),
    ];

    #[test]
    fn every_fixture_is_judged_as_the_table_says() {
        for (text, closes) in FIXTURES {
            assert_eq!(!find(text).is_empty(), *closes, "{text:?}");
        }
    }

    #[test]
    fn neutralised_text_never_closes_and_keeps_its_words() {
        for (text, _) in FIXTURES {
            let n = neutralise(text);
            assert!(find(&n).is_empty(), "{text:?} -> {n:?}");
            assert_eq!(
                n.replace("issue ", ""),
                text.replace("issue ", ""),
                "{text:?}"
            );
        }
        assert_eq!(neutralise("this fixes #5"), "this fixes issue #5");
        assert_eq!(neutralise("fixes owner/repo#9"), "fixes issue owner/repo#9");
        assert_eq!(
            neutralise("fix #1, close #2"),
            "fix issue #1, close issue #2"
        );
        assert_eq!(neutralise("a\nno-close: #3091"), "a\nno-close: issue #3091");
        assert_eq!(
            neutralise("Closes #1 and fixes #2"),
            "Closes #1 and fixes issue #2"
        );
        assert!(find(&neutralise("Closes #1 and fixes #2")).is_empty());
    }

    #[test]
    fn find_reports_one_based_lines() {
        let hits = find("subject\n\nbody fixes #5\nCloses #9");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].line, 3);
    }

    #[test]
    fn the_shell_rendering_carries_the_rust_pattern_byte_for_byte() {
        assert!(
            LINT_SH.contains(&format!("'{PATTERN}'")),
            "closing_keywords_lint.sh drifted from PATTERN"
        );
    }

    /// Differential: the shell function and [`find`] agree on every fixture,
    /// as one multi-line message (line numbers compared).
    #[test]
    #[cfg(unix)]
    fn the_shell_rendering_and_the_rust_predicate_agree() {
        let message: String = FIXTURES.iter().map(|(t, _)| format!("{t}\n")).collect();
        let script = format!("{LINT_SH}\npmat_closing_keyword_hits");
        let mut child = std::process::Command::new("bash")
            .args(["-c", &script])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("bash");
        use std::io::Write as _;
        child
            .stdin
            .take()
            .expect("stdin")
            .write_all(message.as_bytes())
            .expect("write");
        let out = child.wait_with_output().expect("wait");
        let shell: Vec<usize> = String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(|l| {
                l.split(':')
                    .next()
                    .expect("N:")
                    .parse()
                    .expect("line number")
            })
            .collect();
        let rust: Vec<usize> = find(&message).iter().map(|h| h.line).collect();
        assert!(!rust.is_empty(), "vacuous: no fixture closes");
        assert_eq!(shell, rust);
    }

    /// The commit-msg entry point refuses a closing body, passes a clean one,
    /// and ignores git's comment lines and a `commit -v` diff.
    #[test]
    #[cfg(unix)]
    fn the_commit_msg_lint_judges_a_message_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let judge = |body: &str| {
            let file = dir.path().join("MSG");
            std::fs::write(&file, body).expect("write");
            let script = format!("{LINT_SH}\npmat_closing_keyword_commit_msg_lint \"$1\"");
            std::process::Command::new("bash")
                .args(["-c", &script, "lint"])
                .arg(&file)
                .output()
                .expect("bash")
                .status
                .code()
        };
        assert_eq!(judge("feat: x\n\nthis fixes #5\n"), Some(1));
        assert_eq!(judge("feat: x\n\nCloses #5\n"), Some(0));
        assert_eq!(judge("feat: x\n# fixes #5 in a comment\n"), Some(0));
        let scissors = format!("feat: x\n# {0} >8 {0}\ndiff: fixes #5\n", "-".repeat(24));
        assert_eq!(judge(&scissors), Some(0));
    }
}
