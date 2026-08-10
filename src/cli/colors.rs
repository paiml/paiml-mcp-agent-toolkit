#![cfg_attr(coverage_nightly, coverage(off))]
//! Shared ANSI color constants and formatting helpers for CLI output.
//!
//! All CLI handlers should use the **helper functions** below rather than the
//! raw constants: the helpers consult [`colors_enabled`] and emit plain text
//! when colour is off, so `--color never` and a redirected stdout produce
//! diffable output.
//!
//! This module used to claim "the `--color` flag is handled at the top-level
//! dispatcher by stripping ANSI sequences". Nothing stripped anything: `--color
//! never` only set `NO_COLOR=1`, which nothing read, so `tdg --format table
//! --color never > out.txt` still wrote `Overall Score: ^[[1;37m97.2^[[0m/100`
//! (GH #684).
//!
//! The raw `pub const` sequences below cannot be made conditional — they are
//! `const` and interpolated directly at ~490 call sites. Those sites still emit
//! colour unconditionally; migrating them to the helpers is the remaining work.
//!
//! Where no semantic helper fits (a call site that opens a sequence in one
//! `format!` argument and closes it in another), [`seq`] is the mechanical
//! migration: `{c::BOLD}` becomes `{}` fed by `c::seq(c::BOLD)`, which is `""`
//! when colour is off. `analyze provability` and `analyze duplicates` are
//! migrated; they used to write 17 and 68 escape-bearing lines into a
//! redirected file under `--color never`, indistinguishable from `--color
//! auto`.

// ── ANSI escape sequences ───────────────────────────────────────────────────

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const ITALIC: &str = "\x1b[3m";
pub const UNDERLINE: &str = "\x1b[4m";

// Foreground colors
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";
pub const WHITE: &str = "\x1b[37m";

// Bright / bold foreground
pub const BOLD_RED: &str = "\x1b[1;31m";
pub const BOLD_GREEN: &str = "\x1b[1;32m";
pub const BOLD_YELLOW: &str = "\x1b[1;33m";
pub const BOLD_BLUE: &str = "\x1b[1;34m";
pub const BOLD_CYAN: &str = "\x1b[1;36m";
pub const BOLD_WHITE: &str = "\x1b[1;37m";

// Dim foreground
pub const DIM_WHITE: &str = "\x1b[2;37m";
pub const DIM_CYAN: &str = "\x1b[2;36m";

// ── Colour enablement ───────────────────────────────────────────────────────

/// Decide colour from the three inputs that govern it, in precedence order.
///
/// Split out from [`colors_enabled`] so it is testable without mutating process
/// environment (env mutation in tests races across threads).
///
/// * `NO_COLOR` set (to anything non-empty) — off. Set by `--color never`.
/// * `CLICOLOR_FORCE` set — on. Set by `--color always`.
/// * otherwise — on only when stdout is a terminal (`--color auto`, the
///   documented default).
#[must_use]
pub fn colors_enabled_from(no_color: bool, clicolor_force: bool, stdout_is_tty: bool) -> bool {
    if no_color {
        return false;
    }
    if clicolor_force {
        return true;
    }
    stdout_is_tty
}

/// Whether ANSI sequences should be emitted at all.
///
/// Resolved once per process, after `apply_ux_settings` has translated
/// `--color` into `NO_COLOR` / `CLICOLOR_FORCE`.
#[must_use]
pub fn colors_enabled() -> bool {
    use std::io::IsTerminal;
    use std::sync::OnceLock;

    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        let is_set = |k: &str| std::env::var_os(k).is_some_and(|v| !v.is_empty());
        colors_enabled_from(
            is_set("NO_COLOR"),
            is_set("CLICOLOR_FORCE"),
            std::io::stdout().is_terminal(),
        )
    })
}

/// A raw escape sequence, gated on [`colors_enabled`].
///
/// Returns `sequence` when colour is on and `""` when it is off. This is the
/// escape hatch for call sites that cannot use a semantic helper because the
/// opening and closing sequences land in different `format!` arguments —
/// `c::seq(c::BOLD)` is a drop-in for a bare `c::BOLD` interpolation and, unlike
/// the `const`, honours `--color never`, `NO_COLOR` and a redirected stdout.
#[must_use]
#[inline]
pub fn seq(sequence: &'static str) -> &'static str {
    if colors_enabled() {
        sequence
    } else {
        ""
    }
}

/// Wrap `text` in `color` … `RESET`, or return it unchanged when colour is off.
#[inline]
fn paint(color: &str, text: &str) -> String {
    if colors_enabled() {
        format!("{color}{text}{RESET}")
    } else {
        text.to_string()
    }
}

// ── Semantic formatting helpers ─────────────────────────────────────────────

/// Format a section header (bold + underline)
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn header(text: &str) -> String {
    if colors_enabled() {
        format!("{BOLD}{UNDERLINE}{text}{RESET}")
    } else {
        text.to_string()
    }
}

/// Format a subheader (bold)
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn subheader(text: &str) -> String {
    paint(BOLD, text)
}

/// Format a success/pass item
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn pass(text: &str) -> String {
    format!("{} {text}", paint(GREEN, "✓"))
}

/// Format a warning item
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn warn(text: &str) -> String {
    format!("{} {text}", paint(YELLOW, "⚠"))
}

/// Format a failure/error item
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn fail(text: &str) -> String {
    format!("{} {text}", paint(RED, "✗"))
}

/// Format a skipped item
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn skip(text: &str) -> String {
    format!("{} {}", paint(DIM, "⏭"), paint(DIM, text))
}

/// Format a dimmed/secondary text
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn dim(text: &str) -> String {
    paint(DIM, text)
}

/// Format a file path (cyan, like rg/fd)
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn path(text: &str) -> String {
    paint(CYAN, text)
}

/// Format a number/score (bold white)
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn number(text: &str) -> String {
    paint(BOLD_WHITE, text)
}

/// Format a label (bold)
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn label(text: &str) -> String {
    paint(BOLD, text)
}

/// Wrap `text` in an explicit ANSI sequence, or return it unchanged when colour
/// is off.
///
/// For call sites where no semantic helper fits — a colour picked by
/// [`grade_color`], or a one-off highlight. Prefer the semantic helpers when one
/// applies; the point of both is that they consult [`colors_enabled`], which the
/// raw `pub const`s cannot.
#[must_use]
#[inline]
pub fn colored(color: &str, text: &str) -> String {
    paint(color, text)
}

/// Colour a grade letter is rendered in. Pure: independent of whether colour
/// is enabled, so it stays assertable when output is plain.
#[must_use]
#[inline]
pub fn grade_color(g: &str) -> &'static str {
    match g.chars().next() {
        Some('A') => GREEN,
        Some('B' | 'C') => YELLOW,
        Some('D') => RED,
        Some('F') => BOLD_RED,
        _ => WHITE,
    }
}

/// Colour for a higher-is-better value against two thresholds. Pure.
#[must_use]
#[inline]
pub fn threshold_color(value: f64, good_threshold: f64, warn_threshold: f64) -> &'static str {
    if value >= good_threshold {
        GREEN
    } else if value >= warn_threshold {
        YELLOW
    } else {
        RED
    }
}

/// Colour for a lower-is-better value against two thresholds. Pure.
#[must_use]
#[inline]
pub fn threshold_color_inverse(
    value: f64,
    good_threshold: f64,
    warn_threshold: f64,
) -> &'static str {
    if value <= good_threshold {
        GREEN
    } else if value <= warn_threshold {
        YELLOW
    } else {
        RED
    }
}

/// Colour for a signed delta. Pure.
#[must_use]
#[inline]
pub fn delta_color(value: f64) -> &'static str {
    if value > 0.0 {
        GREEN
    } else if value < 0.0 {
        RED
    } else {
        DIM
    }
}

/// Format a grade with appropriate color
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn grade(g: &str) -> String {
    paint(grade_color(g), g)
}

/// Format a percentage with threshold coloring (higher is better)
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn pct(value: f64, good_threshold: f64, warn_threshold: f64) -> String {
    paint(
        threshold_color(value, good_threshold, warn_threshold),
        &format!("{value:.1}%"),
    )
}

/// Format a percentage with inverted threshold coloring (lower is better, e.g. pressure)
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn pct_inverse(value: f64, good_threshold: f64, warn_threshold: f64) -> String {
    paint(
        threshold_color_inverse(value, good_threshold, warn_threshold),
        &format!("{value:.1}%"),
    )
}

/// Format a delta value (positive = green/improvement, negative = red/regression)
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn delta(value: f64) -> String {
    paint(delta_color(value), &format!("{value:+.1}"))
}

/// Percentage a score fraction represents, 0.0 when `max` is not positive.
#[must_use]
#[inline]
pub fn score_percentage(earned: f64, max: f64) -> f64 {
    if max > 0.0 {
        earned / max * 100.0
    } else {
        0.0
    }
}

/// Format a score fraction (e.g., "14.5/15.0") with threshold coloring
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
pub fn score(earned: f64, max: f64, good_pct: f64, warn_pct: f64) -> String {
    let color = threshold_color(score_percentage(earned, max), good_pct, warn_pct);
    format!(
        "{}/{}",
        paint(color, &format!("{earned:.1}")),
        paint(DIM, &format!("{max:.1}"))
    )
}

/// Format a horizontal rule
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn rule() -> String {
    paint(DIM, "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
}

/// Format a section separator (thin)
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn separator() -> String {
    paint(DIM, "───────────────────────────────────────────────────")
}

#[cfg(test)]
mod tests {
    use super::*;

    // GH #684: these tests used to assert that every helper emitted ANSI
    // unconditionally — i.e. they pinned the defect. `--color never` and a
    // redirected stdout must both produce plain text, so colour SELECTION
    // (which colour a value maps to) is now asserted separately from colour
    // EMISSION (whether any escape is written at all).

    /// Escape byte that must not appear in plain output.
    const ESC: char = '\x1b';

    fn is_plain(s: &str) -> bool {
        !s.contains(ESC)
    }

    // ── enablement policy ───────────────────────────────────────────────────

    #[test]
    fn no_color_beats_everything() {
        // `--color never` sets NO_COLOR; it wins even over a terminal and even
        // over CLICOLOR_FORCE.
        assert!(!colors_enabled_from(true, false, true));
        assert!(!colors_enabled_from(true, true, true));
        assert!(!colors_enabled_from(true, true, false));
    }

    #[test]
    fn clicolor_force_enables_without_a_tty() {
        // `--color always` must colour a pipe.
        assert!(colors_enabled_from(false, true, false));
    }

    #[test]
    fn auto_follows_the_tty() {
        // The documented `auto` behaviour: on for a terminal, off for a pipe.
        assert!(colors_enabled_from(false, false, true));
        assert!(!colors_enabled_from(false, false, false));
    }

    // ── emission: the test binary's stdout is not a terminal ────────────────

    #[test]
    fn helpers_emit_no_escapes_when_colour_is_disabled() {
        assert!(
            !colors_enabled(),
            "cargo test captures stdout, so colour must resolve to off here"
        );

        let rendered = vec![
            header("Title"),
            subheader("Sub"),
            pass("ok"),
            warn("hmm"),
            fail("bad"),
            skip("later"),
            dim("note"),
            path("src/lib.rs"),
            number("42"),
            label("Name:"),
            grade("A+"),
            grade("F"),
            pct(95.0, 90.0, 70.0),
            pct_inverse(5.0, 10.0, 30.0),
            delta(2.5),
            score(14.0, 15.0, 80.0, 60.0),
            rule(),
            separator(),
        ];

        for s in &rendered {
            assert!(is_plain(s), "expected plain text, got {s:?}");
        }
    }

    // ── content survives with colour off ────────────────────────────────────

    #[test]
    fn helpers_keep_their_payload_text() {
        assert_eq!(header("Title"), "Title");
        assert_eq!(subheader("Sub"), "Sub");
        assert_eq!(pass("ok"), "✓ ok");
        assert_eq!(warn("hmm"), "⚠ hmm");
        assert_eq!(fail("bad"), "✗ bad");
        assert_eq!(skip("later"), "⏭ later");
        assert_eq!(dim("note"), "note");
        assert_eq!(path("src/lib.rs"), "src/lib.rs");
        assert_eq!(number("42"), "42");
        assert_eq!(label("Name:"), "Name:");
        assert_eq!(grade("A+"), "A+");
        assert_eq!(pct(95.0, 90.0, 70.0), "95.0%");
        assert_eq!(pct_inverse(5.0, 10.0, 30.0), "5.0%");
        assert_eq!(delta(2.5), "+2.5");
        assert_eq!(delta(0.0), "+0.0");
        assert_eq!(score(14.0, 15.0, 80.0, 60.0), "14.0/15.0");
        assert!(rule().contains('━'));
        assert!(separator().contains('─'));
    }

    // ── selection: which colour a value maps to ─────────────────────────────

    #[test]
    fn grade_colors_by_letter() {
        assert_eq!(grade_color("A+"), GREEN);
        assert_eq!(grade_color("B"), YELLOW);
        assert_eq!(grade_color("C-"), YELLOW);
        assert_eq!(grade_color("D+"), RED);
        assert_eq!(grade_color("F"), BOLD_RED);
        assert_eq!(grade_color("?"), WHITE);
        assert_eq!(grade_color(""), WHITE);
    }

    #[test]
    fn threshold_color_is_higher_is_better() {
        assert_eq!(threshold_color(95.0, 90.0, 70.0), GREEN);
        assert_eq!(threshold_color(90.0, 90.0, 70.0), GREEN); // inclusive
        assert_eq!(threshold_color(80.0, 90.0, 70.0), YELLOW);
        assert_eq!(threshold_color(70.0, 90.0, 70.0), YELLOW); // inclusive
        assert_eq!(threshold_color(50.0, 90.0, 70.0), RED);
    }

    #[test]
    fn threshold_color_inverse_is_lower_is_better() {
        assert_eq!(threshold_color_inverse(5.0, 10.0, 30.0), GREEN);
        assert_eq!(threshold_color_inverse(10.0, 10.0, 30.0), GREEN); // inclusive
        assert_eq!(threshold_color_inverse(20.0, 10.0, 30.0), YELLOW);
        assert_eq!(threshold_color_inverse(50.0, 10.0, 30.0), RED);
    }

    #[test]
    fn delta_color_follows_the_sign() {
        assert_eq!(delta_color(2.5), GREEN);
        assert_eq!(delta_color(-3.0), RED);
        assert_eq!(delta_color(0.0), DIM);
    }

    #[test]
    fn score_percentage_guards_against_a_zero_max() {
        assert!((score_percentage(14.0, 15.0) - 93.333_333).abs() < 0.001);
        assert!((score_percentage(0.0, 0.0) - 0.0).abs() < f64::EPSILON);
        // 0% is below any positive threshold, so a zero max renders red.
        assert_eq!(threshold_color(score_percentage(0.0, 0.0), 80.0, 60.0), RED);
    }

    // ── painting, exercised in both states ──────────────────────────────────

    /// `--color never` was indistinguishable from `--color auto` on the
    /// commands that interpolate the raw consts. `seq` is what lets those call
    /// sites be migrated without restructuring their `format!`s.
    #[test]
    fn seq_is_empty_when_colour_is_disabled() {
        assert!(
            !colors_enabled(),
            "cargo test captures stdout, so colour must resolve to off here"
        );
        for raw in [RESET, BOLD, DIM, GREEN, YELLOW, RED, CYAN, BOLD_WHITE] {
            assert_eq!(seq(raw), "", "seq must not emit {raw:?} with colour off");
            assert!(is_plain(seq(raw)));
        }
    }

    #[test]
    fn paint_wraps_only_when_enabled() {
        // `paint` is what every helper funnels through; assert both branches
        // explicitly rather than relying on the ambient environment.
        let wrapped = format!("{GREEN}ok{RESET}");
        assert_eq!(
            if colors_enabled() {
                wrapped.clone()
            } else {
                "ok".to_string()
            },
            paint(GREEN, "ok")
        );
        assert!(wrapped.starts_with(GREEN) && wrapped.ends_with(RESET));
    }
}
