//! `--format json` must put the document on stdout and nothing else.
//!
//! Issue #1061 reported that `pmat analyze complexity --format json` emitted
//! banners before its JSON, so a consumer failed at character 0. It does not
//! reproduce: on 3.32.0 (`583ea9ac2`)
//!
//! ```text
//! OUT=$(pmat analyze complexity --top-files 3 --format json 2>/dev/null)  # jq: OK,   exit 0
//! OUT=$(pmat analyze complexity --top-files 3 --format json 2>&1)         # jq: BAD-JSON, exit 0
//! ```
//!
//! Same binary, same commit — the only difference is the redirection. The
//! report came from a harness that captured `2>&1` (`run_audit.sh:26` does
//! exactly this, while the dogfood skill's own P1 check uses `2>/dev/null` and
//! passes). Merging stderr into stdout and then complaining that stdout is
//! impure measures the harness, not the binary: progress chatter on stderr is
//! the correct design, and it is what `status_eprintln!` exists to enforce.
//!
//! So there is no bug to fix — but there was also no test holding the line, and
//! the class is real: `analyze cluster` shipped its banners on stdout via
//! `println!` (see `entropy_semantic.rs`'s `stdout_purity_tests`), and v3.18.2
//! regressed the same way. This module pins the invariant for the complexity
//! handlers, which are what #1061 named.
//!
//! The invariant, stated precisely: in those handlers `println!` may only emit
//! a **whole pre-formatted payload** — a format string that is exactly one
//! `{…}` interpolation and no literal text. Decoration, counts and banners must
//! go through `status_eprintln!` (stderr, and silent under `--quiet`). A
//! literal like `println!("✅ Analyzed {} files", n)` is precisely the #1061
//! shape and fails here.
//!
//! Like the existing precedent this is a source-level check: a unit test cannot
//! observe the real process stdout, so the guard is made against the source
//! that writes it. It lives under `commands/` rather than beside the handlers
//! only to keep this change inside one module tree.

/// Handler sources whose stdout is a machine-readable surface.
const GUARDED: &[&str] = &[
    "src/cli/handlers/complexity_handlers/analysis.rs",
    "src/cli/handlers/complexity_handlers/churn.rs",
    "src/cli/handlers/complexity_handlers/mod.rs",
    "src/cli/handlers/complexity_handlers/output.rs",
    "src/cli/handlers/complexity_handlers/satd.rs",
    "src/cli/handlers/complexity_handlers/watch.rs",
];

fn read_source(rel: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path)
        .map_err(|e| format!("guarded source {} must be readable: {e}", path.display()))
        .expect("guarded source must be readable")
}

/// True when `line` begins a `println!` call (and not an `eprintln!`, which
/// ends in the same eight characters).
fn is_stdout_print(line: &str) -> bool {
    let t = line.trim_start();
    ["println!(", "crate::status_println!(", "status_println!("]
        .iter()
        .any(|p| t.starts_with(p))
}

/// The format-string literal of a single-line `println!`, if it has one.
fn format_literal(line: &str) -> Option<&str> {
    let open = line.find('(')?;
    let rest = &line[open + 1..];
    let rest = rest.trim_start();
    let body = rest.strip_prefix('"')?;
    let end = body.find('"')?;
    Some(&body[..end])
}

/// A payload passthrough is a format string that is exactly one `{…}` group.
fn is_payload_passthrough(fmt: &str) -> bool {
    let Some(inner) = fmt.strip_prefix('{').and_then(|s| s.strip_suffix('}')) else {
        return false;
    };
    !inner.contains('{') && !inner.contains('}')
}

/// Every `println!` in the guarded sources that decorates its output.
fn decorating_stdout_prints() -> Vec<String> {
    let mut problems = Vec::new();
    for rel in GUARDED {
        let src = read_source(rel);
        for (i, line) in src.lines().enumerate() {
            let t = line.trim_start();
            if t.starts_with("//") || t.starts_with("///") || !is_stdout_print(line) {
                continue;
            }
            match format_literal(line) {
                // A non-literal first argument (a variable) is a passthrough.
                None => {}
                Some(fmt) if is_payload_passthrough(fmt) => {}
                Some(fmt) => {
                    problems.push(format!("{rel}:{}: println!(\"{fmt}\" …)", i + 1));
                }
            }
        }
    }
    problems
}

#[test]
fn complexity_handlers_never_decorate_stdout() {
    let problems = decorating_stdout_prints();
    assert!(
        problems.is_empty(),
        "these write decorated text to STDOUT, which is where `--format json` \
         puts its document — a JSON consumer fails at character 0 (issue \
         #1061's reported shape). Route chatter through `status_eprintln!`:\n  {}",
        problems.join("\n  ")
    );
}

/// The guarded files must keep routing their chatter through the stderr macro.
/// If `status_eprintln!` disappeared from `analysis.rs`, the test above would
/// still pass while the banners had moved somewhere unchecked.
#[test]
fn complexity_progress_banners_still_go_through_the_stderr_macro() {
    let src = read_source("src/cli/handlers/complexity_handlers/analysis.rs");
    // The banner #1061's reporter saw first. It must be emitted, and emitted
    // through the stderr macro.
    let emitters: Vec<&str> = src
        .lines()
        .map(str::trim_start)
        .filter(|l| !l.starts_with("//"))
        .filter(|l| l.contains("Successfully analyzed"))
        .filter(|l| l.contains("println!"))
        .collect();
    assert!(
        !emitters.is_empty(),
        "the `Successfully analyzed` banner vanished from analysis.rs — this \
         guard would then be pinning nothing"
    );
    for line in emitters {
        assert!(
            line.contains("status_eprintln!"),
            "progress banner must go to stderr via status_eprintln!: {line}"
        );
    }
}

/// Counter-test: the guard has to fire on the #1061 shape, or the test above
/// is just asserting that a clean file is clean.
#[test]
fn the_guard_rejects_a_decorated_stdout_print() {
    assert!(
        !is_payload_passthrough("✅ Successfully analyzed {} file(s)"),
        "a banner with literal text is NOT a payload passthrough"
    );
    assert!(
        !is_payload_passthrough("{} of {} files"),
        "two interpolations with literal text between them is decoration"
    );
    assert!(
        format_literal(r#"    println!("🔍 Analyzing...");"#)
            .is_some_and(|f| !is_payload_passthrough(f)),
        "the exact #1061 shape must be flagged"
    );
}

/// Counter-test bounding the over-correction: real payload passthroughs, which
/// are how the report itself reaches stdout, must NOT be flagged.
#[test]
fn the_guard_accepts_payload_passthroughs() {
    for fmt in ["{formatted_output}", "{content}", "{mermaid_content}"] {
        assert!(
            is_payload_passthrough(fmt),
            "{fmt} emits a whole pre-formatted document and is correct"
        );
    }
    assert!(
        !is_stdout_print(r#"    eprintln!("chatter");"#),
        "eprintln! ends in the same characters as println! but writes to stderr"
    );
    assert!(
        !is_stdout_print(r#"    crate::status_eprintln!("chatter");"#),
        "status_eprintln! is the approved chatter macro"
    );
}
