//! `--color` must reach the query output modes.
//!
//! `tint` tested `stdout().is_terminal()` directly, a second and weaker copy of
//! the rule in `cli::colors`. That copy knows nothing of `--color`, which is
//! translated into `NO_COLOR` / `CLICOLOR_FORCE` before it gets here — so
//! `--color always` emitted no colour at all once output was piped, and
//! `--color never` on a TTY still emitted it. The fix that stopped raw escapes
//! leaking into redirected files had made the flag unreachable for
//! `--files-with-matches`, `--count` and `-A/-B/-C`.
//!
//! These assert the decision table itself, which is what the duplicate got
//! wrong; `colors_enabled_from` is the single place that owns it.

use crate::cli::colors::colors_enabled_from;

#[test]
fn color_always_wins_over_a_non_tty() {
    // The reported case: piping `--color always` produced plain text.
    assert!(
        colors_enabled_from(false, true, false),
        "--color always (CLICOLOR_FORCE) must colour even when piped"
    );
}

#[test]
fn color_never_wins_over_a_tty() {
    assert!(
        !colors_enabled_from(true, false, true),
        "--color never (NO_COLOR) must suppress colour even on a terminal"
    );
}

#[test]
fn never_beats_always_when_both_are_set() {
    assert!(
        !colors_enabled_from(true, true, true),
        "suppression must win, so a script setting NO_COLOR cannot be overridden"
    );
}

#[test]
fn auto_follows_the_terminal() {
    assert!(colors_enabled_from(false, false, true));
    assert!(!colors_enabled_from(false, false, false));
}
