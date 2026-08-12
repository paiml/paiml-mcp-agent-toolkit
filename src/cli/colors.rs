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
//! The raw sequences below used to be `const &'static str`, interpolated
//! directly at ~690 call sites, and every one of those sites emitted its escape
//! unconditionally. Migrating them a printer at a time is what produced three
//! separate half-fixes and still left twelve commands — `query`, `analyze
//! churn`, `analyze graph-metrics`, `infra-score`, `repo-score`, `diagnose`,
//! `comply check`, `deps-audit`, `popper-score`, `validate-docs`,
//! `show-metrics` and `enforce` — writing ANSI into a redirected file under an
//! explicit `--color never`.
//!
//! They are now [`Sgr`] values whose `Display` consults [`colors_enabled`]. A
//! call site that interpolates one gets the rule for free and cannot drift from
//! it, because there is no longer a spelling of "emit this escape
//! unconditionally" to reach for.

// ── ANSI escape sequences ───────────────────────────────────────────────────

/// An ANSI SGR sequence that renders itself **only when colour is enabled**.
///
/// Interpolating one (`format!("{}{text}{}", c::CYAN, c::RESET)`, or
/// `format!("{CYAN}…{RESET}")` where the constants are in scope) is the
/// migration-free way to honour `--color never` / `--color always` / a
/// redirected stdout: with colour off the value formats to the empty string.
///
/// Use [`Sgr::raw`] only where the bytes themselves are the subject — tests
/// asserting *which* colour was selected, or a buffer whose gating is done by
/// the caller.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Sgr(&'static str);

impl Sgr {
    /// Wrap a literal escape sequence.
    #[must_use]
    pub const fn new(sequence: &'static str) -> Self {
        Self(sequence)
    }

    /// The escape bytes, **ungated**. Pure: independent of `colors_enabled`.
    #[must_use]
    pub const fn raw(self) -> &'static str {
        self.0
    }
}

impl std::fmt::Display for Sgr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if colors_enabled() {
            f.write_str(self.0)
        } else {
            Ok(())
        }
    }
}

impl std::fmt::Debug for Sgr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sgr({:?})", self.0)
    }
}

pub const RESET: Sgr = Sgr::new("\x1b[0m");
pub const BOLD: Sgr = Sgr::new("\x1b[1m");
pub const DIM: Sgr = Sgr::new("\x1b[2m");
pub const ITALIC: Sgr = Sgr::new("\x1b[3m");
pub const UNDERLINE: Sgr = Sgr::new("\x1b[4m");

// Foreground colors
pub const RED: Sgr = Sgr::new("\x1b[31m");
pub const GREEN: Sgr = Sgr::new("\x1b[32m");
pub const YELLOW: Sgr = Sgr::new("\x1b[33m");
pub const BLUE: Sgr = Sgr::new("\x1b[34m");
pub const MAGENTA: Sgr = Sgr::new("\x1b[35m");
pub const CYAN: Sgr = Sgr::new("\x1b[36m");
pub const WHITE: Sgr = Sgr::new("\x1b[37m");

// Bright / bold foreground
pub const BOLD_RED: Sgr = Sgr::new("\x1b[1;31m");
pub const BOLD_GREEN: Sgr = Sgr::new("\x1b[1;32m");
pub const BOLD_YELLOW: Sgr = Sgr::new("\x1b[1;33m");
pub const BOLD_BLUE: Sgr = Sgr::new("\x1b[1;34m");
pub const BOLD_CYAN: Sgr = Sgr::new("\x1b[1;36m");
pub const BOLD_WHITE: Sgr = Sgr::new("\x1b[1;37m");

// Dim foreground
pub const DIM_WHITE: Sgr = Sgr::new("\x1b[2;37m");
pub const DIM_CYAN: Sgr = Sgr::new("\x1b[2;36m");

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

    // A test may force the answer for its own thread — see [`ForcedColor`].
    // Without this seam a unit test can only ever observe the *off* half of the
    // rule (`cargo test` captures stdout, and the decision below is a
    // process-wide `OnceLock`), which is precisely how five printers shipped
    // that emitted nothing under `--color always`: "no escapes here" passed as
    // green.
    #[cfg(test)]
    if let Some(forced) = forced_color::get() {
        return forced;
    }

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

#[cfg(test)]
mod forced_color {
    use std::cell::Cell;

    thread_local! {
        static FORCED: Cell<Option<bool>> = const { Cell::new(None) };
    }

    pub(super) fn get() -> Option<bool> {
        FORCED.with(Cell::get)
    }

    pub(super) fn set(value: Option<bool>) {
        FORCED.with(|slot| slot.set(value));
    }
}

/// Force [`colors_enabled`] for the current thread, restoring the previous
/// answer on drop. Test-only.
///
/// Thread-local rather than a global, so tests that force colour on cannot make
/// a concurrently running test see escapes it does not expect — the trap that
/// env-var-mutating colour tests kept falling into.
///
/// This exists so a test can assert **both halves** of the `--color` contract on
/// the same renderer: escapes present when colour is on, absent when it is off.
/// A one-sided "is plain" assertion is satisfied by a printer that has no colour
/// at all, which is exactly the defect shape this module keeps having to fix.
#[cfg(test)]
pub(crate) struct ForcedColor(Option<bool>);

#[cfg(test)]
impl ForcedColor {
    /// Force colour on (`true`) or off (`false`) until the guard drops.
    pub(crate) fn new(enabled: bool) -> Self {
        let previous = forced_color::get();
        forced_color::set(Some(enabled));
        Self(previous)
    }

    /// Force colour on — the state `--color always` produces.
    pub(crate) fn on() -> Self {
        Self::new(true)
    }

    /// Force colour off — the state `--color never` (or a redirected stdout)
    /// produces.
    pub(crate) fn off() -> Self {
        Self::new(false)
    }
}

#[cfg(test)]
impl Drop for ForcedColor {
    fn drop(&mut self) {
        forced_color::set(self.0);
    }
}

/// The escape byte no renderer may emit while colour is off.
#[cfg(test)]
pub(crate) const ESC: char = '\u{1b}';

/// Assert that `render` honours `--color` in **both** directions.
///
/// `render` is called twice: once with colour forced on, where its output must
/// carry at least one ANSI escape, and once with colour forced off, where it
/// must carry none. `what` names the surface in the failure message.
///
/// The "on" half is the one that catches an inert `--color`: a printer that
/// simply never colours anything passes every plain-output assertion.
#[cfg(test)]
pub(crate) fn assert_honours_color(what: &str, mut render: impl FnMut() -> String) {
    let colored = {
        let _guard = ForcedColor::on();
        render()
    };
    assert!(
        colored.contains(ESC),
        "{what}: `--color always` produced no ANSI escape, so the flag changes \
         nothing observable. Output was: {colored:?}"
    );

    let plain = {
        let _guard = ForcedColor::off();
        render()
    };
    let leaking: Vec<&str> = plain.lines().filter(|l| l.contains(ESC)).collect();
    assert!(
        leaking.is_empty(),
        "{what}: {} line(s) carry ANSI with colour off; first: {:?}",
        leaking.len(),
        leaking.first()
    );
}

/// Identity on an [`Sgr`], kept because ~50 call sites spell the gating
/// explicitly as `c::seq(c::BOLD)`.
///
/// The gating now lives in `Sgr`'s `Display`, so `c::seq(c::BOLD)` and a bare
/// `c::BOLD` are the same thing. Prefer the bare constant in new code.
#[must_use]
#[inline]
pub fn seq(sequence: Sgr) -> Sgr {
    sequence
}

/// Wrap `text` in `color` … `RESET`, or return it unchanged when colour is off.
#[inline]
fn paint(color: Sgr, text: &str) -> String {
    if colors_enabled() {
        format!("{}{text}{}", color.raw(), RESET.raw())
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
        format!("{}{}{text}{}", BOLD.raw(), UNDERLINE.raw(), RESET.raw())
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
pub fn colored(color: Sgr, text: &str) -> String {
    paint(color, text)
}

/// Colour a grade letter is rendered in. Pure: independent of whether colour
/// is enabled, so it stays assertable when output is plain.
#[must_use]
#[inline]
pub fn grade_color(g: &str) -> Sgr {
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
pub fn threshold_color(value: f64, good_threshold: f64, warn_threshold: f64) -> Sgr {
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
pub fn threshold_color_inverse(value: f64, good_threshold: f64, warn_threshold: f64) -> Sgr {
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
pub fn delta_color(value: f64) -> Sgr {
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

    // ── the seam that makes an inert `--color` fail ─────────────────────────

    /// `ForcedColor` must move `colors_enabled` in both directions and restore
    /// the previous answer on drop; everything asserted through
    /// [`assert_honours_color`] rests on this.
    #[test]
    fn forced_color_moves_the_decision_and_restores_it() {
        let baseline = colors_enabled();
        {
            let _on = ForcedColor::on();
            assert!(colors_enabled());
            assert!(format!("{BOLD}x{RESET}").contains(ESC));
            {
                let _off = ForcedColor::off();
                assert!(!colors_enabled());
                assert_eq!(format!("{BOLD}x{RESET}"), "x");
            }
            assert!(colors_enabled(), "inner guard must restore the outer one");
        }
        assert_eq!(
            colors_enabled(),
            baseline,
            "guard must restore the process answer"
        );
    }

    /// The point of the helper: a printer that emits no colour at all FAILS it.
    ///
    /// Every previous `--color` test in this crate asserted only "output is
    /// plain when colour is off", which such a printer passes — which is how
    /// five commands shipped with a `--color` flag that changed nothing.
    #[test]
    fn assert_honours_color_rejects_a_printer_that_never_colours() {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let outcome = std::panic::catch_unwind(|| {
            assert_honours_color("a printer with no colour", || {
                "Average Score: 90.9/100 (A)".to_string()
            });
        });
        std::panic::set_hook(previous);
        assert!(
            outcome.is_err(),
            "assert_honours_color must reject a renderer that emits no ANSI under --color always"
        );
    }

    /// …and accepts one that does.
    #[test]
    fn assert_honours_color_accepts_a_printer_that_does() {
        assert_honours_color("a printer that colours", || {
            format!("Average Score: {}/100 ({})", number("90.9"), grade("A"))
        });
    }

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
            assert_eq!(
                seq(raw).to_string(),
                "",
                "seq must not emit {raw:?} with colour off"
            );
            assert!(is_plain(&seq(raw).to_string()));
        }
    }

    /// The defect this type exists to close: **interpolating a raw constant**
    /// used to emit its escape unconditionally, so twelve commands wrote ANSI
    /// into a redirected file under an explicit `--color never`. A bare
    /// `{CYAN}` must now be exactly as gated as `c::seq(c::CYAN)`.
    #[test]
    fn raw_constants_emit_nothing_when_interpolated_with_colour_off() {
        assert!(
            !colors_enabled(),
            "cargo test captures stdout, so colour must resolve to off here"
        );
        for sgr in [
            RESET,
            BOLD,
            DIM,
            ITALIC,
            UNDERLINE,
            RED,
            GREEN,
            YELLOW,
            BLUE,
            MAGENTA,
            CYAN,
            WHITE,
            BOLD_RED,
            BOLD_GREEN,
            BOLD_YELLOW,
            BOLD_BLUE,
            BOLD_CYAN,
            BOLD_WHITE,
            DIM_WHITE,
            DIM_CYAN,
        ] {
            let line = format!("{sgr}src/lib.rs:12{RESET}");
            assert_eq!(
                line, "src/lib.rs:12",
                "interpolating {sgr:?} leaked an escape with colour off"
            );
            assert!(is_plain(&line));
        }
    }

    /// …while the sequence itself is still reachable, so tests can assert
    /// *which* colour a value selects and `paint` can build the wrapper.
    #[test]
    fn raw_exposes_the_ungated_sequence() {
        assert_eq!(RESET.raw(), "\x1b[0m");
        assert_eq!(BOLD_RED.raw(), "\x1b[1;31m");
        assert_eq!(grade_color("F").raw(), BOLD_RED.raw());
    }

    #[test]
    fn paint_wraps_only_when_enabled() {
        // `paint` is what every helper funnels through; assert both branches
        // explicitly rather than relying on the ambient environment.
        let wrapped = format!("{}ok{}", GREEN.raw(), RESET.raw());
        assert_eq!(
            if colors_enabled() {
                wrapped.clone()
            } else {
                "ok".to_string()
            },
            paint(GREEN, "ok")
        );
        assert!(wrapped.starts_with(GREEN.raw()) && wrapped.ends_with(RESET.raw()));
    }
}
