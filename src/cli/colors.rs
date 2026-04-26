#![cfg_attr(coverage_nightly, coverage(off))]
//! Shared ANSI color constants and formatting helpers for CLI output.
//!
//! All CLI handlers should use these constants for consistent colorized output.
//! Colors are unconditionally emitted — the `--color` flag is handled at the
//! top-level dispatcher by stripping ANSI sequences when `--color never`.

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

// ── Semantic formatting helpers ─────────────────────────────────────────────

/// Format a section header (bold + underline)
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn header(text: &str) -> String {
    format!("{BOLD}{UNDERLINE}{text}{RESET}")
}

/// Format a subheader (bold)
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn subheader(text: &str) -> String {
    format!("{BOLD}{text}{RESET}")
}

/// Format a success/pass item
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn pass(text: &str) -> String {
    format!("{GREEN}✓{RESET} {text}")
}

/// Format a warning item
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn warn(text: &str) -> String {
    format!("{YELLOW}⚠{RESET} {text}")
}

/// Format a failure/error item
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn fail(text: &str) -> String {
    format!("{RED}✗{RESET} {text}")
}

/// Format a skipped item
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn skip(text: &str) -> String {
    format!("{DIM}⏭{RESET} {DIM}{text}{RESET}")
}

/// Format a dimmed/secondary text
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn dim(text: &str) -> String {
    format!("{DIM}{text}{RESET}")
}

/// Format a file path (cyan, like rg/fd)
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn path(text: &str) -> String {
    format!("{CYAN}{text}{RESET}")
}

/// Format a number/score (bold white)
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn number(text: &str) -> String {
    format!("{BOLD_WHITE}{text}{RESET}")
}

/// Format a label (bold)
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn label(text: &str) -> String {
    format!("{BOLD}{text}{RESET}")
}

/// Format a grade with appropriate color
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn grade(g: &str) -> String {
    let color = match g.chars().next() {
        Some('A') => GREEN,
        Some('B') => YELLOW,
        Some('C') => YELLOW,
        Some('D') => RED,
        Some('F') => BOLD_RED,
        _ => WHITE,
    };
    format!("{color}{g}{RESET}")
}

/// Format a percentage with threshold coloring (higher is better)
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn pct(value: f64, good_threshold: f64, warn_threshold: f64) -> String {
    let color = if value >= good_threshold {
        GREEN
    } else if value >= warn_threshold {
        YELLOW
    } else {
        RED
    };
    format!("{color}{value:.1}%{RESET}")
}

/// Format a percentage with inverted threshold coloring (lower is better, e.g. pressure)
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn pct_inverse(value: f64, good_threshold: f64, warn_threshold: f64) -> String {
    let color = if value <= good_threshold {
        GREEN
    } else if value <= warn_threshold {
        YELLOW
    } else {
        RED
    };
    format!("{color}{value:.1}%{RESET}")
}

/// Format a delta value (positive = green/improvement, negative = red/regression)
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn delta(value: f64) -> String {
    let color = if value > 0.0 {
        GREEN
    } else if value < 0.0 {
        RED
    } else {
        DIM
    };
    format!("{color}{value:+.1}{RESET}")
}

/// Format a score fraction (e.g., "14.5/15.0") with threshold coloring
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
pub fn score(earned: f64, max: f64, good_pct: f64, warn_pct: f64) -> String {
    let percentage = if max > 0.0 { earned / max * 100.0 } else { 0.0 };
    let color = if percentage >= good_pct {
        GREEN
    } else if percentage >= warn_pct {
        YELLOW
    } else {
        RED
    };
    format!("{color}{earned:.1}{RESET}/{DIM}{max:.1}{RESET}")
}

/// Format a horizontal rule
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn rule() -> String {
    format!("{DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{RESET}")
}

/// Format a section separator (thin)
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn separator() -> String {
    format!("{DIM}───────────────────────────────────────────────────{RESET}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_wraps_with_bold_underline_and_reset() {
        let s = header("Title");
        assert!(s.starts_with(BOLD));
        assert!(s.contains(UNDERLINE));
        assert!(s.contains("Title"));
        assert!(s.ends_with(RESET));
    }

    #[test]
    fn test_subheader_wraps_with_bold_and_reset() {
        let s = subheader("Sub");
        assert!(s.starts_with(BOLD));
        assert!(s.contains("Sub"));
        assert!(s.ends_with(RESET));
    }

    #[test]
    fn test_pass_starts_with_green_check() {
        let s = pass("ok");
        assert!(s.starts_with(GREEN));
        assert!(s.contains('✓'));
        assert!(s.contains("ok"));
    }

    #[test]
    fn test_warn_starts_with_yellow_warning() {
        let s = warn("hmm");
        assert!(s.starts_with(YELLOW));
        assert!(s.contains('⚠'));
        assert!(s.contains("hmm"));
    }

    #[test]
    fn test_fail_starts_with_red_cross() {
        let s = fail("bad");
        assert!(s.starts_with(RED));
        assert!(s.contains('✗'));
        assert!(s.contains("bad"));
    }

    #[test]
    fn test_skip_uses_dim_double_wrap() {
        let s = skip("later");
        assert!(s.starts_with(DIM));
        assert!(s.contains('⏭'));
        assert!(s.contains("later"));
    }

    #[test]
    fn test_dim_wraps_with_dim_and_reset() {
        let s = dim("note");
        assert_eq!(s, format!("{DIM}note{RESET}"));
    }

    #[test]
    fn test_path_uses_cyan() {
        let s = path("src/lib.rs");
        assert!(s.starts_with(CYAN));
        assert!(s.contains("src/lib.rs"));
        assert!(s.ends_with(RESET));
    }

    #[test]
    fn test_number_uses_bold_white() {
        let s = number("42");
        assert!(s.starts_with(BOLD_WHITE));
        assert!(s.contains("42"));
    }

    #[test]
    fn test_label_uses_bold() {
        let s = label("Name:");
        assert!(s.starts_with(BOLD));
        assert!(s.contains("Name:"));
    }

    // ── grade arms ──────────────────────────────────────────────────────────

    #[test]
    fn test_grade_a_is_green() {
        let s = grade("A+");
        assert!(s.starts_with(GREEN));
        assert!(s.contains("A+"));
    }

    #[test]
    fn test_grade_b_is_yellow() {
        let s = grade("B");
        assert!(s.starts_with(YELLOW));
        assert!(s.contains('B'));
    }

    #[test]
    fn test_grade_c_is_yellow() {
        let s = grade("C-");
        assert!(s.starts_with(YELLOW));
        assert!(s.contains("C-"));
    }

    #[test]
    fn test_grade_d_is_red() {
        let s = grade("D+");
        assert!(s.starts_with(RED));
        assert!(s.contains("D+"));
    }

    #[test]
    fn test_grade_f_is_bold_red() {
        let s = grade("F");
        assert!(s.starts_with(BOLD_RED));
        assert!(s.contains('F'));
    }

    #[test]
    fn test_grade_other_is_white() {
        // Non-letter prefix and empty fall through to WHITE.
        let s_q = grade("?");
        assert!(s_q.starts_with(WHITE));
        let s_empty = grade("");
        assert!(s_empty.starts_with(WHITE));
    }

    // ── pct (higher is better) ──────────────────────────────────────────────

    #[test]
    fn test_pct_above_good_is_green() {
        let s = pct(95.0, 90.0, 70.0);
        assert!(s.starts_with(GREEN));
        assert!(s.contains("95.0%"));
    }

    #[test]
    fn test_pct_at_good_threshold_is_green() {
        let s = pct(90.0, 90.0, 70.0);
        assert!(s.starts_with(GREEN));
    }

    #[test]
    fn test_pct_between_thresholds_is_yellow() {
        let s = pct(80.0, 90.0, 70.0);
        assert!(s.starts_with(YELLOW));
        assert!(s.contains("80.0%"));
    }

    #[test]
    fn test_pct_below_warn_is_red() {
        let s = pct(50.0, 90.0, 70.0);
        assert!(s.starts_with(RED));
        assert!(s.contains("50.0%"));
    }

    // ── pct_inverse (lower is better) ───────────────────────────────────────

    #[test]
    fn test_pct_inverse_below_good_is_green() {
        let s = pct_inverse(5.0, 10.0, 30.0);
        assert!(s.starts_with(GREEN));
        assert!(s.contains("5.0%"));
    }

    #[test]
    fn test_pct_inverse_between_thresholds_is_yellow() {
        let s = pct_inverse(20.0, 10.0, 30.0);
        assert!(s.starts_with(YELLOW));
    }

    #[test]
    fn test_pct_inverse_above_warn_is_red() {
        let s = pct_inverse(50.0, 10.0, 30.0);
        assert!(s.starts_with(RED));
        assert!(s.contains("50.0%"));
    }

    // ── delta (signed) ──────────────────────────────────────────────────────

    #[test]
    fn test_delta_positive_is_green() {
        let s = delta(2.5);
        assert!(s.starts_with(GREEN));
        assert!(s.contains("+2.5"));
    }

    #[test]
    fn test_delta_negative_is_red() {
        let s = delta(-3.0);
        assert!(s.starts_with(RED));
        assert!(s.contains("-3.0"));
    }

    #[test]
    fn test_delta_zero_is_dim() {
        let s = delta(0.0);
        assert!(s.starts_with(DIM));
        // sign prefix on zero is "+" per `:+.1`
        assert!(s.contains("+0.0"));
    }

    // ── score ───────────────────────────────────────────────────────────────

    #[test]
    fn test_score_above_good_pct_is_green() {
        let s = score(14.0, 15.0, 80.0, 60.0); // 93.3%
        assert!(s.starts_with(GREEN));
        assert!(s.contains("14.0"));
        assert!(s.contains("15.0"));
    }

    #[test]
    fn test_score_between_pct_thresholds_is_yellow() {
        let s = score(10.5, 15.0, 80.0, 60.0); // 70%
        assert!(s.starts_with(YELLOW));
    }

    #[test]
    fn test_score_below_warn_pct_is_red() {
        let s = score(6.0, 15.0, 80.0, 60.0); // 40%
        assert!(s.starts_with(RED));
    }

    #[test]
    fn test_score_max_zero_treats_as_red_zero_pct() {
        // max == 0 short-circuits to 0% which is below any positive threshold → RED.
        let s = score(0.0, 0.0, 80.0, 60.0);
        assert!(s.starts_with(RED));
        assert!(s.contains("0.0"));
    }

    // ── rule / separator ────────────────────────────────────────────────────

    #[test]
    fn test_rule_returns_dim_horizontal_line() {
        let s = rule();
        assert!(s.starts_with(DIM));
        assert!(s.contains('━'));
        assert!(s.ends_with(RESET));
    }

    #[test]
    fn test_separator_returns_dim_thin_line() {
        let s = separator();
        assert!(s.starts_with(DIM));
        assert!(s.contains('─'));
        assert!(s.ends_with(RESET));
    }
}
