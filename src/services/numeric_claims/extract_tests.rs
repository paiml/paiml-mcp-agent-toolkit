//! Unit tests for R2's extractor.
//!
//! Each of these pins a decision the rules cannot work without, and each was
//! demonstrated failing against the stub extractor that returned no mentions.

use super::extract::{
    dim_of_unit, extract_file, key_dim, key_polarity, norm_key, parse_value_expr, to_canon, Dim,
    Polarity,
};

#[test]
fn reachability_probe_extract() {}

/// Values are **evaluated**, not read.
///
/// Read literally, `50 * 1024 * 1024` is the numeral `50`, and the whole
/// `binary_size.rs` contradiction disappears — `50` against `50,000,000` looks
/// like a unit difference the ambiguity set would forgive. Every form below
/// appears in the audited corpus.
#[test]
fn values_are_evaluated_not_read() {
    let cases = [
        ("50_000_000", 50_000_000.0),
        ("1,234", 1_234.0),
        ("0xFFFF", 65_535.0),
        ("1e6", 1_000_000.0),
        ("50 * 1024 * 1024", 52_428_800.0),
        ("1 << 20", 1_048_576.0),
        ("37 * 1024 * 1024", 38_797_312.0),
        ("512", 512.0),
    ];
    for (raw, want) in cases {
        let got = parse_value_expr(raw);
        assert_eq!(
            got,
            Some(want),
            "{raw:?} must evaluate to {want}, got {got:?}"
        );
    }
    // Counter-control: the evaluator is narrow on purpose. It is not a general
    // expression engine and it never runs anything from the corpus.
    for raw in ["\"A-\"", "true", "foo + 1", "1 + 2", "std::u64::MAX"] {
        assert_eq!(parse_value_expr(raw), None, "{raw:?} must not evaluate");
    }
}

/// Units normalise to an **ambiguity set**, not a choice.
///
/// `MB` is 10^6 to a disk vendor and 2^20 to a linker, and one repository
/// writes both. A rule may fire only when every reading disagrees.
#[test]
fn unit_ambiguity_is_a_set_not_a_choice() {
    assert_eq!(
        to_canon(50.0, "mb", Dim::Bytes),
        vec![50_000_000.0, 52_428_800.0],
        "MB is ambiguous and both readings must survive"
    );
    assert_eq!(
        to_canon(50.0, "mib", Dim::Bytes),
        vec![52_428_800.0],
        "MiB is not ambiguous — IEC says what it means"
    );
    assert_eq!(to_canon(1.0, "h", Dim::Time), vec![3_600_000.0]);
    assert_eq!(to_canon(30.0, "s", Dim::Time), vec![30_000.0]);
    assert_eq!(
        to_canon(570.0, "", Dim::Count),
        vec![570.0],
        "a count carries no unit and must not be scaled"
    );
    assert_eq!(dim_of_unit("gib"), Some(Dim::Bytes));
    assert_eq!(dim_of_unit("mins"), Some(Dim::Time));
    assert_eq!(dim_of_unit("%"), Some(Dim::Pct));
    assert_eq!(
        dim_of_unit("workers"),
        None,
        "an English noun is not a unit — treating it as one invents a dimension"
    );
}

/// TOML keys must be **section-qualified**.
///
/// Unqualified, `.pmat-ratchet.toml`'s `[metric.*].baseline` keys collapse into
/// one cluster called `baseline` that contradicts itself by construction: three
/// unrelated quantities, three different values, one name.
#[test]
fn toml_keys_are_section_qualified() {
    let toml = "\
[metric.unwrap_calls_src_total]
baseline = 20387

[metric.panic_macro_src_total]
baseline = 788

[metric.allow_attributes_src_total]
baseline = 497
";
    let ms = extract_file(".pmat-ratchet.toml", toml);
    let keys: Vec<&str> = ms.iter().map(|m| m.key.as_str()).collect();
    assert_eq!(
        keys,
        vec![
            "metric.unwrap_calls_src_total.baseline",
            "metric.panic_macro_src_total.baseline",
            "metric.allow_attributes_src_total.baseline",
        ]
    );
    // Counter-control: the leaf names really are identical, so without the
    // section prefix these three would be one quantity holding three values.
    let leaves: std::collections::BTreeSet<&str> = keys
        .iter()
        .map(|k| k.rsplit('.').next().unwrap_or(k))
        .collect();
    assert_eq!(leaves.len(), 1, "the bogus cluster this guard prevents");
}

/// YAML keys are qualified by indent path, which is what makes the sibling
/// `codecov.yml` comparison a comparison of one quantity.
#[test]
fn yaml_keys_are_indent_qualified() {
    let yaml = "\
coverage:
  status:
    project:
      default:
        target: 95%
        threshold: 2%
";
    let ms = extract_file("codecov.yml", yaml);
    let keys: Vec<&str> = ms.iter().map(|m| m.key.as_str()).collect();
    assert!(
        keys.contains(&"coverage.status.project.default.threshold"),
        "expected an indent-qualified threshold, got {keys:?}"
    );
    assert!(keys.contains(&"coverage.status.project.default.target"));
}

/// Rust contributes `const` and `static` declarations only.
///
/// A local binding is an operand, not a claim the repository makes about
/// itself, and pulling locals in is how the corpus goes from 3,883 mentions to
/// tens of thousands of numbers nobody wrote down as a fact.
#[test]
fn rust_contributes_const_and_static_only() {
    let rs = "\
const MAX_SIZE_BYTES: u64 = 50 * 1024 * 1024; // 50MB (aligned with .pmat-metrics.toml binary_max_bytes)
static RETRY_LIMIT: usize = 3;
fn f() { let max_size_bytes = 99; }
";
    let ms = extract_file("src/tests/binary_size.rs", rs);
    let keys: Vec<&str> = ms.iter().map(|m| m.key.as_str()).collect();
    assert_eq!(keys, vec!["MAX_SIZE_BYTES", "RETRY_LIMIT"]);
    assert_eq!(ms[0].value, 52_428_800.0);
    assert_eq!(ms[0].line, 1);
    assert!(ms[0].annot.contains("aligned with"), "{:?}", ms[0].annot);
    assert_eq!(
        ms[0].dim,
        Dim::Bytes,
        "the _bytes suffix names the dimension"
    );
}

/// Markdown inside a fence is code, not prose. The fence state has to be
/// tracked, because the same `**Port**: 8080` line means different things
/// inside and outside one.
#[test]
fn markdown_fences_are_not_prose() {
    let outside = "**Timeout**: 30 s\n";
    let inside = "```\n**Timeout**: 30 s\n```\n";
    assert_eq!(extract_file("README.md", outside).len(), 1);
    assert_eq!(extract_file("README.md", inside).len(), 0);
}

/// A trailing comment is the annotation; a preceding block is context.
///
/// This is the whole of R-3: at HEAD the explanation of `max_unwrap_calls = 0`
/// lives on the lines above it, and reading it as an assertion would re-flag
/// work already finished.
#[test]
fn trailing_comment_and_preceding_block_are_kept_apart() {
    let toml = "\
[quality_gates]
# The tree measures 20390 by the predicate .pmat-ratchet.toml pins.
# Test code may unwrap freely — a panic in a test IS the assertion.
max_unwrap_calls = 0
other_limit = 5   # Current: 9
";
    let ms = extract_file(".pmat-metrics.toml", toml);
    assert_eq!(ms.len(), 2);
    assert_eq!(ms[0].annot, "", "no trailing comment on the declaration");
    assert!(
        ms[0].block.contains("20390"),
        "the preceding block is kept, as context: {:?}",
        ms[0].block
    );
    assert_eq!(
        ms[1].annot, "Current: 9",
        "a same-line comment is the annotation"
    );
}

/// Key suffixes name dimensions, and `_min` is a floor rather than minutes.
///
/// The prototype encoded that with a negative lookbehind the Rust engine cannot
/// express, so it is a separate pattern here — and worth pinning, because
/// reading `provability_min = 0.60` as 36 seconds puts a score in the wrong
/// dimension and lets it be compared with durations.
#[test]
fn key_suffixes_name_dimensions_and_polarity() {
    assert_eq!(key_dim("lint_max_ms"), (Dim::Time, "ms"));
    assert_eq!(key_dim("binary_max_bytes"), (Dim::Bytes, "b"));
    assert_eq!(key_dim("max_memory_usage_mb"), (Dim::Bytes, "mb"));
    assert_eq!(key_dim("min_coverage_pct"), (Dim::Pct, "%"));
    assert_eq!(key_dim("timeout_mins"), (Dim::Time, "min"));
    assert_eq!(
        key_dim("provability_min"),
        (Dim::Count, "n"),
        "a bare _min suffix is a floor, not minutes"
    );
    assert_eq!(key_polarity("provability_min"), Some(Polarity::Min));
    assert_eq!(key_polarity("binary_max_bytes"), Some(Polarity::Max));
    assert_eq!(
        key_polarity("regression_threshold_pct"),
        Some(Polarity::Max)
    );
    assert_eq!(key_polarity("retention_days"), None);
}

/// A cross-reference naming a single generic token identifies no quantity.
#[test]
fn norm_key_drops_generic_tokens() {
    assert_eq!(
        norm_key("thresholds.binary_max_bytes"),
        vec!["binary", "bytes", "thresholds"]
    );
    assert_eq!(norm_key("maxCountOfTheThing"), vec!["thing"]);
    assert_eq!(
        norm_key("threshold").len(),
        1,
        "one surviving token: too generic to resolve"
    );
}
