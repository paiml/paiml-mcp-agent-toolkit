//! CB-2104 R2 — the RED-first rule tests (spec §6.2).
//!
//! Every test here was demonstrated failing against a `run()` that returns an
//! empty [`R2Outcome`] before it was kept. The three negative tests (R-3, R-9,
//! R-10) would pass trivially against that stub, so each carries a *counter-
//! control* in the same test: a one-character mutation of the same input that
//! must fire. A negative test with no counter-control measures nothing.

use super::{run_r2, CorpusFile, Finding, RuleId};

// ---------------------------------------------------------------------------
// helpers

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_repo_file(rel: &str) -> String {
    std::fs::read_to_string(repo_root().join(rel))
        .expect("CB-2104 fixture file must be readable from the pmat work tree")
}

/// The live tree, in the exact shape the research prototype scanned.
///
/// This duplicates what `corpus.rs` will own. That is deliberate: R-1 is an
/// assertion about *this repository at HEAD*, and it must not be able to go
/// green because a sibling lane's file discovery quietly narrowed.
fn live_repo_corpus() -> Vec<CorpusFile> {
    let out = std::process::Command::new("git")
        .args([
            "ls-files", "-z", "*.toml", "*.yaml", "*.yml", "*.rs", "*.md", "*.json",
        ])
        .current_dir(repo_root())
        .output()
        .expect("git ls-files must run inside the pmat work tree");
    assert!(out.status.success(), "git ls-files failed");
    let listing = String::from_utf8_lossy(&out.stdout);
    // Exclusion goes through `path_exclusion`, the SAME function the binary
    // uses. This was a second regex — `(target|node_modules|vendor|fixtures?|
    // testdata|\.git)/` — and the two implementations diverged the moment the
    // real one learned to exclude `*_tests.rs`: the binary stopped reporting
    // `extract_tests.rs` and this test kept reporting it, so a fix that WORKED
    // read as a fix that had failed.
    //
    // Two implementations of one rule is the defect class this whole check
    // exists to report. A test that reimplements the thing it is testing is
    // measuring its own copy.
    let mut files = Vec::new();
    for rel in listing.split('\0').filter(|s| !s.is_empty()) {
        if super::corpus::path_exclusion(rel).is_some() {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(repo_root().join(rel)) else {
            continue;
        };
        if text.lines().count() > 60_000 {
            continue;
        }
        files.push(CorpusFile::new(rel, text));
    }
    assert!(
        files.len() > 200,
        "live corpus collapsed to {} files — the test would be vacuous",
        files.len()
    );
    files
}

fn of_rule(findings: &[Finding], rule: RuleId) -> Vec<&Finding> {
    findings.iter().filter(|f| f.rule == rule).collect()
}

fn one_file(path: &str, text: &str) -> Vec<CorpusFile> {
    vec![CorpusFile::new(path, text)]
}

// ---------------------------------------------------------------------------
// R-1  c5_binary_size_mismatch_at_head
// ---------------------------------------------------------------------------

/// The live contradiction the whole check was built around.
///
/// `src/tests/binary_size.rs:40` declares `50 * 1024 * 1024` = 52,428,800 and
/// says in the same breath that it is *aligned with* `.pmat-metrics.toml`'s
/// `binary_max_bytes`, which is 50,000,000. Two numbers, one claimed identity,
/// 2,428,800 apart.
///
/// RED without C5. Also RED without expression evaluation: read literally,
/// `50 * 1024 * 1024` is the numeral `50`, and `50` versus `50,000,000` is a
/// unit difference the ambiguity set would forgive.
#[test]
fn c5_binary_size_mismatch_at_head() {
    let corpus = live_repo_corpus();
    let out = run_r2(&corpus);
    let c5 = of_rule(&out.findings, RuleId::C5);
    assert_eq!(
        c5.len(),
        1,
        "expected exactly one C5 on the live tree, got {}: {:#?}",
        c5.len(),
        c5
    );
    let f = c5[0];
    assert_eq!(f.sites.len(), 2, "C5 names a source and a target");
    let src = &f.sites[0];
    let tgt = &f.sites[1];
    assert_eq!(src.file, "src/tests/binary_size.rs");
    assert_eq!(src.line, 40);
    assert!(
        (src.value - 52_428_800.0).abs() < 1.0,
        "50 * 1024 * 1024 must be evaluated, got {}",
        src.value
    );
    assert_eq!(tgt.file, ".pmat-metrics.toml");
    assert_eq!(tgt.line, 12);
    assert!(
        (tgt.value - 50_000_000.0).abs() < 1.0,
        "50_000_000 must parse past the separators, got {}",
        tgt.value
    );
    assert!(
        (src.value - tgt.value - 2_428_800.0).abs() < 1.0,
        "delta must be 2,428,800, got {}",
        src.value - tgt.value
    );
    assert!(
        f.evidence.contains("aligned with"),
        "the finding must quote the annotation that fired it: {:?}",
        f.evidence
    );
    assert_eq!(
        f.wrong_floor, 1,
        "two disagreeing sites: at least one wrong"
    );
}

// ---------------------------------------------------------------------------
// R-2  c1_fires_on_historical_metrics_file
// ---------------------------------------------------------------------------

/// `.pmat-metrics.toml` as it stood at `2f15cab92`, verbatim.
///
/// Line 41 is the brief's headline defect: a ceiling of 100 whose own trailing
/// comment reports 570. Kept as a literal rather than read from `git show` so
/// the test cannot rot when history is rewritten, and so the whole file — not
/// just the defective line — is held to "exactly one finding".
const HISTORICAL_METRICS_TOML: &str = r#"# O(1) Quality Gate Metric Thresholds
# Spec: docs/specifications/quick-test-build-O(1)-checking.md
# Pattern: Toyota Way - Jidoka (MEAN mode: warnings = errors)

[thresholds]
# Hard limits (block commits)
lint_max_ms = 30_000              # 30s (user requirement: "30 second pre-commit test/lint")
test_fast_max_ms = 300_000        # 5min (user requirement: "5<make test-fast")
coverage_max_ms = 600_000         # 10min (user requirement: "under 10 min coverage")
binary_max_bytes = 50_000_000     # 50 MB (current: 42 MB, 16% headroom)
deps_default_max = 3_000          # 3,000 dependencies (current: 2,754, 8% headroom)

# Soft limits (warnings only)
build_release_max_ms = 900_000    # 15min (current: 11m 57s)
deps_minimal_max = 2_500          # 2,500 dependencies (rust-only, current: 2,055)

[staleness]
# Metrics older than this trigger warnings
max_age_days = 7

[enforcement]
# Pre-commit behavior (MEAN mode = strict)
fail_on_stale_metrics = false     # Warn, don't block
fail_on_missing_metrics = false   # Allow commits if no cache
fail_on_threshold_violation = true # Block commits on violations (MEAN mode)

[trend_analysis]
# Track metrics for Kaizen (continuous improvement)
enabled = true
retention_days = 90
alert_on_regression = true
regression_threshold_pct = 10.0   # Alert if >10% slower

# PMAT-specific quality gates
[quality_gates]
# Target quality standards for PMAT itself (dogfooding)
min_coverage_pct = 85.0           # Target: ≥85% line coverage (NASA standard)
min_mutation_score_pct = 80.0     # Target: ≥80% mutation score
max_cyclomatic_complexity = 15    # Target: ≤15 per function
min_tdg_grade = "A-"              # Target: A- or better (≥88)
max_unwrap_calls = 100            # Current: 570 (CRITICAL - must reduce!)

# Performance budgets (PMAT operation targets)
[performance]
# PMAT command execution targets
min_tdg_analysis_throughput = 1000   # 1000+ lines/sec
max_memory_usage_mb = 512            # ≤512MB for typical projects
max_regression_pct = 5.0             # ≤5% performance regression allowed
"#;

/// RED without C1.
#[test]
fn c1_fires_on_historical_metrics_file() {
    let corpus = one_file(".pmat-metrics.toml", HISTORICAL_METRICS_TOML);
    let out = run_r2(&corpus);
    let c1 = of_rule(&out.findings, RuleId::C1);
    assert_eq!(
        c1.len(),
        1,
        "one self-breach in the historical file, got {:#?}",
        out.findings
    );
    let f = c1[0];
    assert!(
        f.quantity.ends_with("max_unwrap_calls"),
        "quantity must be section-qualified, got {:?}",
        f.quantity
    );
    assert_eq!(
        f.quantity, "quality_gates.max_unwrap_calls",
        "unqualified keys collapse distinct quantities into one bogus cluster"
    );
    assert_eq!(f.sites.len(), 1);
    assert_eq!(f.sites[0].line, 41);
    assert!((f.sites[0].value - 100.0).abs() < f64::EPSILON);
    assert!(
        f.evidence.contains("570"),
        "the finding must carry the observation that breached it: {:?}",
        f.evidence
    );

    // The rest of the file is not collateral: 22 other keys, several of them
    // limits with annotations, and none of them may fire.
    assert_eq!(
        out.findings.len(),
        1,
        "exactly one finding over the whole historical file, got {:#?}",
        out.findings
    );
    assert!(
        out.census.r2_mentions >= 15,
        "census must show the file was actually read, got {}",
        out.census.r2_mentions
    );
}

// ---------------------------------------------------------------------------
// R-3  c1_silent_on_head_metrics_file
// ---------------------------------------------------------------------------

/// The guard against re-flagging a state that has been fixed.
///
/// At HEAD the same key reads `max_unwrap_calls = 0` with the explanation on
/// *preceding* lines, and it is >72 characters. Both facts must classify it as
/// context rather than assertion. If either half of the annotation gate rots,
/// this file starts firing again and the check begins nagging about work
/// already done.
///
/// Counter-control: the same rules over the historical file must fire, so a
/// `run()` that reports nothing at all cannot pass this test.
#[test]
fn c1_silent_on_head_metrics_file() {
    let head = read_repo_file(".pmat-metrics.toml");
    assert!(
        head.contains("max_unwrap_calls"),
        "fixture drifted: HEAD's .pmat-metrics.toml no longer declares the key"
    );
    let out = run_r2(&one_file(".pmat-metrics.toml", &head));
    assert!(
        out.findings.is_empty(),
        "HEAD's .pmat-metrics.toml must report zero, got {:#?}",
        out.findings
    );
    assert!(
        out.census.r2_mentions >= 15,
        "silence must come from the rules, not from an empty read: {} mentions",
        out.census.r2_mentions
    );

    // Counter-control: these same rules, on the pre-fix file, do fire.
    let before = run_r2(&one_file(".pmat-metrics.toml", HISTORICAL_METRICS_TOML));
    assert_eq!(
        of_rule(&before.findings, RuleId::C1).len(),
        1,
        "counter-control: the historical file must still fire, or this test is vacuous"
    );
}

// ---------------------------------------------------------------------------
// R-9  correct_derivations_are_never_flagged
// ---------------------------------------------------------------------------

/// The ten hand-audited correct derivations from the attack corpus.
///
/// Each annotation *computes* the value beside it out of parts, so the parts
/// disagreeing with the whole is arithmetic, not contradiction.
const DERIVATIONS_RS: &str = r"
const A_BYTES: usize = 1088;    // 64 rows * 17 bytes
const B_BYTES: usize = 1104;    // 64 * 17 + 16 bytes header
const C_BYTES: usize = 320;     // 4 lanes * 8 regs * 10 bytes
const D_MS: u64 = 250;          // 1000 ms / 4 workers
const E_BYTES: usize = 3072;    // 3 * 1024 bytes
const F_MS: u64 = 90;           // 30 ms * 3 retries
const G_BYTES: usize = 132;     // 128 bytes payload + 4 bytes crc
const H_MS: u64 = 1500;         // 500 ms budget, 3 phases
const I_BYTES: usize = 2080;    // (64 + 1) * 32 bytes
const J_MS: u64 = 7200000;      // 2 hours
";

/// RED without the derivation guard.
#[test]
fn correct_derivations_are_never_flagged() {
    let out = run_r2(&one_file("src/layout.rs", DERIVATIONS_RS));
    assert!(
        out.findings.is_empty(),
        "correct derivations must never be flagged, got {:#?}",
        out.findings
    );
    assert_eq!(
        out.census.r2_mentions, 10,
        "all ten declarations must be extracted, or the silence is vacuous"
    );

    // Counter-control 1: break one derivation's arithmetic and C2 must fire.
    // `64 rows * 17 bytes` = 1088; the declaration now says 999.
    let broken = DERIVATIONS_RS.replace(
        "const A_BYTES: usize = 1088;",
        "const A_BYTES: usize = 999; ",
    );
    let out2 = run_r2(&one_file("src/layout.rs", &broken));
    assert_eq!(
        of_rule(&out2.findings, RuleId::C2).len(),
        1,
        "counter-control: 999 is not 64 * 17, C2 must fire — got {:#?}",
        out2.findings
    );

    // Counter-control 2: the guard must be counted, never silent. A suppression
    // that leaves no trace is indistinguishable from a rule that never ran.
    assert!(
        out.census.suppressed_derivation >= 1,
        "every derivation suppression must be counted in the census"
    );
}

// ---------------------------------------------------------------------------
// R-10  ambiguous_units_do_not_fire
// ---------------------------------------------------------------------------

/// `MB` is 10^6 to a disk vendor and 2^20 to a linker, and a repository writes
/// both. A rule may fire only when *every* reading disagrees, so
/// `50_000_000  # 50 MB` — true under the SI reading, false under the IEC one —
/// must stay silent.
///
/// Counter-control: `40 MB` disagrees under both readings and must fire.
#[test]
fn ambiguous_units_do_not_fire() {
    let agrees = "[thresholds]\nbinary_max_bytes = 50_000_000     # 50 MB\n";
    let out = run_r2(&one_file(".pmat-metrics.toml", agrees));
    assert!(
        out.findings.is_empty(),
        "50 MB is a true reading of 50,000,000 — must not fire: {:#?}",
        out.findings
    );
    assert_eq!(out.census.r2_mentions, 1, "the key must be extracted");
    assert_eq!(
        out.census.r2_assertive_annotations, 1,
        "the annotation must reach the rules — silence from a dropped \
         annotation would not test the ambiguity set"
    );

    let disagrees = "[thresholds]\nbinary_max_bytes = 50_000_000     # 40 MB\n";
    let out2 = run_r2(&one_file(".pmat-metrics.toml", disagrees));
    assert_eq!(
        of_rule(&out2.findings, RuleId::C2).len(),
        1,
        "counter-control: 40 MB is wrong under both readings — got {:#?}",
        out2.findings
    );

    // And the IEC reading alone is enough to acquit: 52,428,800 is 50 MiB, and
    // "50 MB" is a legitimate way to write it.
    let iec = "[thresholds]\nbinary_max_bytes = 52_428_800     # 50 MB\n";
    assert!(
        run_r2(&one_file(".pmat-metrics.toml", iec))
            .findings
            .is_empty(),
        "52,428,800 is exactly 50 MiB — one true reading acquits"
    );
}

// ---------------------------------------------------------------------------
// live-tree quietness guard
// ---------------------------------------------------------------------------

/// The check earns its precision by being quiet, and any change that makes it
/// louder should be assumed to have broken that until re-measured.
///
/// The research pass measured pmat at exactly one R2 finding — the C5 that
/// [`c5_binary_size_mismatch_at_head`] pins — over 3,883 mentions, and zero
/// findings across five healthy third-party repositories. This bounds the noise
/// without pinning the count: one genuinely new contradiction committed by
/// someone else is a true positive and must not fail their build, but an
/// extraction change that starts firing on ordinary config lines will.
#[test]
fn r2_on_the_live_tree_stays_quiet() {
    let out = run_r2(&live_repo_corpus());
    let mut by_rule: Vec<(&str, usize)> = Vec::new();
    for r in [RuleId::C1, RuleId::C2, RuleId::C3, RuleId::C4, RuleId::C5] {
        by_rule.push((r.as_str(), of_rule(&out.findings, r).len()));
    }
    println!("CB-2104 R2 on the live tree: {by_rule:?}");
    println!(
        "  mentions {} / assertive {} / raw literals {} / files {} / {} ms",
        out.census.r2_mentions,
        out.census.r2_assertive_annotations,
        out.census.raw_numeric_literals,
        out.census.files_scanned,
        out.census.elapsed_ms
    );
    println!(
        "  suppressed: derivation {} unit-ambiguity {} unresolved-xref {}",
        out.census.suppressed_derivation,
        out.census.suppressed_unit_ambiguity,
        out.census.suppressed_unresolved_xref
    );
    for f in &out.findings {
        println!("  [{}] {}", f.rule.as_str(), f.detail);
    }
    assert!(
        out.census.r2_mentions > 2_000,
        "extraction collapsed to {} mentions — the quietness would be vacuous",
        out.census.r2_mentions
    );
    // 85 at `583ea9ac2`, reproducing the research prototype's count exactly.
    // (The spec's illustrative census block prints 411 for this field; the
    // prototype it was derived from measures 85, and mentions match at 3,883,
    // so the 411 is an example figure and not a measurement.)
    assert!(
        out.census.r2_assertive_annotations > 50,
        "the annotation gate admitted only {} annotations — silence would be vacuous",
        out.census.r2_assertive_annotations
    );
    assert!(
        out.findings.len() <= 5,
        "R2 got loud on pmat ({} findings) — re-measure precision before shipping: {:#?}",
        out.findings.len(),
        out.findings
    );
}

// ---------------------------------------------------------------------------
// C3 and C4 — every rule must be shown able to fire
// ---------------------------------------------------------------------------

/// C3 ARITHMETIC.
///
/// `binary_max_bytes = 50_000_000  # 50 MB (current: 42 MB, 16% headroom)` is
/// sound: 8 MB of 50 MB is 16%. The failure mode is copy-forward — the
/// observation gets updated and the percentage does not — and the line then
/// misreports how close the budget is to breaking.
///
/// Both readings of "headroom" are accepted, `(L-C)/L` and `(L-C)/C`, because
/// repositories use both and the rule must not adjudicate English. So the
/// counter-control below has to fail under *both*.
#[test]
fn c3_fires_on_headroom_the_numbers_do_not_support() {
    let stale =
        "[thresholds]\nbinary_max_bytes = 50_000_000     # 50 MB (current: 42 MB, 40% headroom)\n";
    let out = run_r2(&one_file(".pmat-metrics.toml", stale));
    assert_eq!(
        of_rule(&out.findings, RuleId::C3).len(),
        1,
        "8 MB of 50 MB is 16%, not 40% — got {:#?}",
        out.findings
    );

    // Counter-control: the arithmetic the file actually ships is sound, under
    // the first reading, and must stay silent.
    let sound =
        "[thresholds]\nbinary_max_bytes = 50_000_000     # 50 MB (current: 42 MB, 16% headroom)\n";
    assert!(
        run_r2(&one_file(".pmat-metrics.toml", sound))
            .findings
            .is_empty(),
        "the sound line must not fire, or C3 is just noise"
    );

    // Counter-control 2: the other reading, (L-C)/C = 19%, is also accepted.
    let other_reading =
        "[thresholds]\nbinary_max_bytes = 50_000_000     # 50 MB (current: 42 MB, 19% headroom)\n";
    assert!(run_r2(&one_file(".pmat-metrics.toml", other_reading))
        .findings
        .is_empty());
}

/// C4 UNJUSTIFIED DIVERGENCE — the audit's live specimen, reduced.
///
/// A repository's root `codecov.yml` carried `threshold: 95%` where both
/// sibling files said `2%`, one line under a comment about requiring 95%
/// *coverage*: the `target` value copied into `threshold`. In codecov's schema
/// `threshold` is the allowed **drop**, so 95 there does not tighten the gate,
/// it disables it — the root coverage gate could never fail again.
///
/// The rule needs three sites and a real factor, not a margin, because a
/// per-crate override that differs by a margin is policy.
#[test]
fn c4_fires_on_sibling_schema_divergence() {
    fn codecov(threshold: &str) -> String {
        format!(
            "coverage:\n  status:\n    project:\n      default:\n        target: 95%\n        threshold: {threshold}\n"
        )
    }
    let corpus = |root: &str| {
        vec![
            CorpusFile::new("codecov.yml", codecov(root)),
            CorpusFile::new("crates/a/codecov.yml", codecov("2%")),
            CorpusFile::new("crates/b/codecov.yml", codecov("2%")),
        ]
    };

    let out = run_r2(&corpus("95%"));
    let c4 = of_rule(&out.findings, RuleId::C4);
    assert_eq!(c4.len(), 1, "expected one C4, got {:#?}", out.findings);
    assert_eq!(c4[0].quantity, "coverage.status.project.default.threshold");
    assert_eq!(c4[0].sites[0].file, "codecov.yml");
    assert_eq!(c4[0].sites[0].value, 95.0);
    assert!(
        c4[0].detail.contains("47.5x"),
        "the finding must state the factor: {:?}",
        c4[0].detail
    );
    assert!(
        !out.findings.iter().any(|f| f.quantity.ends_with("target")),
        "`target` agrees across all three files and must not be reported"
    );

    // Counter-control 1: a margin is policy, not a defect. 3% against 2% is a
    // 1.5x difference and must stay silent.
    assert!(
        run_r2(&corpus("3%")).findings.is_empty(),
        "a per-site margin must not fire — that is how C4 stays at 1/1"
    );

    // Counter-control 2: a divergence with a stated reason is policy too.
    let with_reason = vec![
        CorpusFile::new(
            "codecov.yml",
            codecov("95%   # wide drop allowance during the rewrite"),
        ),
        CorpusFile::new("crates/a/codecov.yml", codecov("2%")),
        CorpusFile::new("crates/b/codecov.yml", codecov("2%")),
    ];
    assert!(
        run_r2(&with_reason).findings.is_empty(),
        "a divergence that gives a reason is a decision, not a contradiction"
    );
}

/// Findings do not depend on the order files arrive in.
///
/// Driven from a corpus that fires three different rules at once, so a
/// reordering cannot be hidden by there being nothing to reorder.
#[test]
fn output_is_deterministic_regardless_of_input_order() {
    fn codecov(threshold: &str) -> String {
        format!(
            "coverage:\n  status:\n    project:\n      default:\n        target: 95%\n        threshold: {threshold}\n"
        )
    }
    let mut corpus = vec![
        CorpusFile::new(".pmat-metrics.toml", HISTORICAL_METRICS_TOML),
        CorpusFile::new(
            "src/tests/binary_size.rs",
            "const MAX_SIZE_BYTES: u64 = 50 * 1024 * 1024; // 50MB (aligned with .pmat-metrics.toml binary_max_bytes)\n",
        ),
        CorpusFile::new("codecov.yml", codecov("95%")),
        CorpusFile::new("crates/a/codecov.yml", codecov("2%")),
        CorpusFile::new("crates/b/codecov.yml", codecov("2%")),
    ];
    let details = |files: &[CorpusFile]| {
        let mut d: Vec<String> = run_r2(files)
            .findings
            .iter()
            .map(|f| f.detail.clone())
            .collect();
        d.sort();
        d
    };
    let forward = details(&corpus);
    assert_eq!(
        forward.len(),
        3,
        "the corpus must fire C1, C4 and C5 at once, or order-independence is vacuous: {forward:#?}"
    );
    corpus.reverse();
    assert_eq!(forward, details(&corpus));
    corpus.rotate_left(2);
    assert_eq!(forward, details(&corpus));
}

/// The check reads its own thresholds from its own source, never from the tree
/// it is scanning.
///
/// A corpus is untrusted input. A file that says
/// `**coherence_max_assertion_chars**: 120 chars` must not widen the annotation
/// gate — otherwise any repository could switch the check off by writing down a
/// number, which is precisely the failure mode this whole check exists to find.
#[test]
fn the_corpus_cannot_reconfigure_the_checker() {
    let hostile = vec![
        CorpusFile::new(
            "docs/coherence.md",
            "**coherence_max_assertion_chars**: 120 chars\n**DIVERGENCE_FACTOR**: 9999\n",
        ),
        CorpusFile::new(".pmat-metrics.toml", HISTORICAL_METRICS_TOML),
    ];
    assert_eq!(
        of_rule(&run_r2(&hostile).findings, RuleId::C1).len(),
        1,
        "the planted configuration must not silence C1"
    );
    assert_eq!(
        super::annotate::MAX_ASSERTION_CHARS,
        72,
        "the bound is a constant in this crate, not a value in the corpus"
    );
}

/// The two sites of the flagship finding must survive the corpus exclusions.
///
/// R2's whole live result is one C5 spanning `src/tests/binary_size.rs` and
/// `.pmat-metrics.toml`. Both sit under paths a fixture-tree or generated-file
/// rule could plausibly swallow — `src/tests/` looks like a test tree, and a
/// dotfile at the root looks like machine state — and if either is dropped the
/// finding disappears silently, with a census that still looks healthy. This
/// pins the interface between the two lanes rather than trusting it.
#[test]
fn the_flagship_sites_are_not_excluded_from_the_corpus() {
    use super::corpus::{classify_generated, path_exclusion};
    for path in ["src/tests/binary_size.rs", ".pmat-metrics.toml"] {
        assert_eq!(
            path_exclusion(path),
            None,
            "{path} carries half of R2's only live finding and must stay in the corpus"
        );
        let text = read_repo_file(path);
        assert_eq!(
            classify_generated(path, &text),
            None,
            "{path} is hand-written and must not be read as machine-generated"
        );
    }
    // Counter-control: the exclusions are not simply inert.
    assert!(path_exclusion("tests/fixtures/numeric_claims/planted.toml").is_some());
    assert!(path_exclusion("CHANGELOG.md").is_some());
}
