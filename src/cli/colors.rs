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
    debug_assert!(!text.is_empty(), "text must not be empty");
    format!("{BOLD}{UNDERLINE}{text}{RESET}")
}

/// Format a subheader (bold)
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn subheader(text: &str) -> String {
    debug_assert!(!text.is_empty(), "text must not be empty");
    format!("{BOLD}{text}{RESET}")
}

/// Format a success/pass item
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn pass(text: &str) -> String {
    debug_assert!(!text.is_empty(), "text must not be empty");
    format!("{GREEN}✓{RESET} {text}")
}

/// Format a warning item
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn warn(text: &str) -> String {
    debug_assert!(!text.is_empty(), "text must not be empty");
    format!("{YELLOW}⚠{RESET} {text}")
}

/// Format a failure/error item
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn fail(text: &str) -> String {
    debug_assert!(!text.is_empty(), "text must not be empty");
    format!("{RED}✗{RESET} {text}")
}

/// Format a skipped item
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn skip(text: &str) -> String {
    debug_assert!(!text.is_empty(), "text must not be empty");
    format!("{DIM}⏭{RESET} {DIM}{text}{RESET}")
}

/// Format a dimmed/secondary text
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn dim(text: &str) -> String {
    debug_assert!(!text.is_empty(), "text must not be empty");
    format!("{DIM}{text}{RESET}")
}

/// Format a file path (cyan, like rg/fd)
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn path(text: &str) -> String {
    debug_assert!(!text.is_empty(), "text must not be empty");
    format!("{CYAN}{text}{RESET}")
}

/// Format a number/score (bold white)
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn number(text: &str) -> String {
    debug_assert!(!text.is_empty(), "text must not be empty");
    format!("{BOLD_WHITE}{text}{RESET}")
}

/// Format a label (bold)
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn label(text: &str) -> String {
    debug_assert!(!text.is_empty(), "text must not be empty");
    format!("{BOLD}{text}{RESET}")
}

/// Format a grade with appropriate color
#[inline]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn grade(g: &str) -> String {
    debug_assert!(!g.is_empty(), "g must not be empty");
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
