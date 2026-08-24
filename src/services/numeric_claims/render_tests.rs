//! CB-2104 — what the check prints.
//!
//! The census is the proof the check ran, so it leads and it is unconditional.
//! A finding names `file:line` for every site it reports, because a developer
//! who has to re-run the tool to find out where to look has been told nothing.

use super::census::{self, SelfTest, Vacuity};
use super::cohort::CohortConfig;
use super::render;
use super::{Census, Finding, NumericClaimsReport, RuleId, Site, Status};

fn site(file: &str, line: usize, value: f64) -> Site {
    Site {
        file: file.to_string(),
        line,
        value,
        text: format!("{file} line {line}"),
    }
}

fn finding() -> Finding {
    Finding {
        rule: RuleId::C5,
        quantity: "thresholds.binary_max_bytes".to_string(),
        sites: vec![
            site("src/tests/binary_size.rs", 40, 52_428_800.0),
            site(".pmat-metrics.toml", 12, 50_000_000.0),
        ],
        wrong_floor: 1,
        anchored: false,
        detail: "they differ by 2428800".to_string(),
        evidence: "aligned with .pmat-metrics.toml binary_max_bytes".to_string(),
        fix: "make one equal the other, or delete the claim".to_string(),
    }
}

fn report(findings: Vec<Finding>, census: Census, status: Status) -> NumericClaimsReport {
    NumericClaimsReport {
        check: "CB-2104",
        severity: "warn",
        status,
        findings,
        census,
        warnings: Vec::new(),
        self_test: SelfTest {
            passed: true,
            planted: 4,
            recovered: 4,
            innocent_items: census::INNOCENT_ITEMS,
            ..SelfTest::default()
        },
    }
}

fn populated() -> Census {
    Census {
        files_scanned: 1_291,
        r1_files_scanned: 1_207,
        files_tracked: 3_252,
        r1_framed_numerals: 7_346,
        r1_cohorts_min2: 180,
        r1_cohorts_at_min_sites: 1,
        r2_mentions: 3_883,
        r2_assertive_annotations: 85,
        suppressed_generated: 35,
        suppressed_multi_slot: 0,
        suppressed_derivation: 0,
        suppressed_unit_ambiguity: 4,
        suppressed_unresolved_xref: 5,
        raw_numeric_literals: 246_926,
        excluded_machine_managed: 0,
        excluded_fixture_tree: 0,
        excluded_changelog: 0,
        unreadable: 0,
        elapsed_ms: 912,
    }
}

// ---------------------------------------------------------------------------

/// RED without the census-first layout.
#[test]
fn text_leads_with_the_census() {
    let out = render::text(&report(vec![finding()], populated(), Status::Ok));
    let census_at = out.find("CENSUS").expect("census block must be printed");
    let finding_at = out
        .find("binary_max_bytes")
        .expect("the finding must be printed");
    assert!(
        census_at < finding_at,
        "the census leads, so a silent pass still shows its working:\n{out}"
    );
    assert!(
        out.starts_with("CB-2104"),
        "the first line names the rule: {:?}",
        out.lines().next()
    );
    assert!(
        out.contains("WARN"),
        "the header must say the check never blocks:\n{out}"
    );
}

/// RED without an unconditional census.
///
/// The whole point: "analysed 12,693 numbers, found nothing" must not be
/// byte-identical to "`git ls-files` returned nothing".
#[test]
fn a_clean_run_still_shows_its_working() {
    let clean = render::text(&report(Vec::new(), populated(), Status::Ok));
    for needle in [
        "1,291", // files scanned
        "3,252", // tracked
        "7,346", // framed numerals
        "3,883", // mentions
        "246,926",
    ] {
        assert!(
            clean.contains(needle),
            "a clean run must print {needle}:\n{clean}"
        );
    }
    assert!(clean.contains("4/4"), "self-test result must appear");

    let nothing = render::text(&report(Vec::new(), Census::default(), Status::Ok));
    assert_ne!(
        clean, nothing,
        "a measured clean tree and an empty corpus must not print the same bytes"
    );
}

/// RED without per-site rendering.
#[test]
fn every_site_is_named_with_file_and_line() {
    let out = render::text(&report(vec![finding()], populated(), Status::Ok));
    for s in finding().sites {
        assert!(
            out.contains(&format!("{}:{}", s.file, s.line)),
            "site {}:{} was not named:\n{out}",
            s.file,
            s.line
        );
    }
    assert!(
        out.contains("C5") && out.contains("NAMED CROSS-REFERENCE"),
        "the rule id and title must be printed:\n{out}"
    );
    assert!(
        out.contains("aligned with"),
        "the evidence that fired the rule must be quoted:\n{out}"
    );
    assert!(
        out.contains("make one equal the other"),
        "the fix must be printed:\n{out}"
    );
    // The floor is named as a floor, and the count agrees with itself.
    assert!(
        out.contains("2 sites; disagreement floor 1; anchored: no"),
        "the site count and the disagreement floor must be stated plainly:\n{out}"
    );

    // Counter-control on the pluralisation: one site is a site.
    let mut single = finding();
    single.sites.truncate(1);
    let one = render::text(&report(vec![single], populated(), Status::Ok));
    assert!(
        one.contains("1 site; disagreement floor 1"),
        "a single site must not be printed as \"1 sites\":\n{one}"
    );
}

/// The R1 lane measured this and asked for it in writing: G1 runs before G2, so
/// a zero in the G2 column means "G1 got there first", never "G2 is idle".
#[test]
fn the_g2_zero_is_not_presented_as_evidence_g2_is_idle() {
    let out = render::text(&report(Vec::new(), populated(), Status::Ok));
    assert!(
        out.contains("G1 runs first"),
        "a 0 beside G2 while G1 suppressed 35 must carry the ordering note:\n{out}"
    );

    // Counter-control: with nothing suppressed by G1 there is no ordering to
    // explain, and the note must not appear.
    let no_g1 = Census {
        suppressed_generated: 0,
        ..populated()
    };
    assert!(
        !render::text(&report(Vec::new(), no_g1, Status::Ok)).contains("G1 runs first"),
        "the note is about an ordering that did not happen here"
    );
}

/// RED without the UNMEASURABLE path.
#[test]
fn unmeasurable_says_so_and_prints_no_findings() {
    let mut r = report(vec![finding()], Census::default(), Status::Unmeasurable);
    r.warnings.push(Vacuity::EmptyCorpus.reason());
    let out = render::text(&r);
    assert!(out.contains("UNMEASURABLE"), "{out}");
    assert!(
        out.contains("exit 2"),
        "the exit code must be stated, not inferred:\n{out}"
    );
    assert!(
        !out.contains("binary_max_bytes"),
        "an unmeasurable run must not print a result it did not earn:\n{out}"
    );
    assert!(
        out.contains("CENSUS"),
        "the census is unconditional, including here:\n{out}"
    );
}

/// The rule reports disagreement, never a verdict on which value is right.
#[test]
fn the_output_never_claims_to_know_which_value_is_wrong() {
    let mut census = populated();
    census.suppressed_generated = 0;
    let out = render::text(&report(vec![finding()], census, Status::Ok));
    for forbidden in ["is wrong", "correct value", "fabricat", "fraud"] {
        assert!(
            !out.to_lowercase().contains(forbidden),
            "the output must not say {forbidden:?}:\n{out}"
        );
    }
    assert!(
        out.contains("anchored"),
        "the report must say whether the quantity has a declared anchor:\n{out}"
    );
}

/// RED without the JSON renderer.
#[test]
fn json_carries_the_exit_code_the_status_and_the_self_test() {
    let out = render::json(&report(vec![finding()], populated(), Status::Ok)).expect("json");
    let v: serde_json::Value = serde_json::from_str(&out).expect("valid json");
    assert_eq!(v["check"], "CB-2104");
    assert_eq!(v["severity"], "warn");
    assert_eq!(v["status"], "OK");
    assert_eq!(v["exit"], 0, "findings must not change the exit code");
    assert_eq!(v["findings"][0]["rule"], "C5");
    assert_eq!(v["findings"][0]["sites"][0]["line"], 40);
    assert_eq!(v["findings"][0]["anchored"], false);
    assert_eq!(v["census"]["r2_mentions"], 3_883);
    assert_eq!(v["census"]["r1_files_scanned"], 1_207);
    assert_eq!(v["self_test"]["recovered"], 4);
    assert_eq!(v["self_test"]["passed"], true);

    let bad =
        render::json(&report(Vec::new(), Census::default(), Status::Unmeasurable)).expect("json");
    let v: serde_json::Value = serde_json::from_str(&bad).expect("valid json");
    assert_eq!(v["status"], "UNMEASURABLE");
    assert_eq!(v["exit"], 2, "unmeasurable is the only non-zero exit");
}

/// The renderer runs over what the check actually produces, not only over
/// hand-built literals — including an R1 finding with many sites.
#[test]
fn the_real_report_renders() {
    let (findings, c, warnings) =
        census::run_corpus(census::fixture_corpus(), &CohortConfig::default());
    assert!(!findings.is_empty(), "the fixture must produce findings");
    let mut r = report(findings, c, Status::Ok);
    r.warnings = warnings;
    let out = render::text(&r);
    for f in &r.findings {
        for s in &f.sites {
            assert!(
                out.contains(&format!("{}:{}", s.file, s.line)),
                "site {}:{} missing from a {} finding:\n{out}",
                s.file,
                s.line,
                f.rule.as_str()
            );
        }
    }
    assert!(render::json(&r).is_ok(), "the real report must serialise");
}

/// Every exclusion is a place where a wrong number survives, so the census has
/// to say how many were refused as well as how many were read.
#[test]
fn the_census_says_what_it_refused_to_look_at() {
    let census = Census {
        excluded_machine_managed: 3,
        excluded_fixture_tree: 61,
        excluded_changelog: 2,
        unreadable: 1,
        ..populated()
    };
    let out = render::text(&report(Vec::new(), census, Status::Ok));
    for needle in [
        "machine-managed 3",
        "fixture-tree 61",
        "changelog 2",
        "unreadable 1",
    ] {
        assert!(
            out.contains(needle),
            "the census must report {needle}:\n{out}"
        );
    }
}
