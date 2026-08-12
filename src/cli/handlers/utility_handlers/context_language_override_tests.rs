//! Regression tests for `pmat context --language` / `--languages`.
//!
//! The flag is labelled "BUG-012: Single language override support" but used
//! to change one header line and nothing else: `--language python` over a
//! Go+Python+TypeScript tree relabelled the document `**Language**: python`
//! while the body still reported `Total Files: 3` including the Go and
//! TypeScript files. `--languages "python,go"` produced output identical to
//! `--language python`. A header that contradicts its own body is worse than
//! no override at all.

use super::canonical_language;

/// A language and each alias the CLI accepts for it must compare equal, or
/// `--language ts` would select nothing on a TypeScript tree.
#[test]
fn aliases_resolve_to_one_canonical_name() {
    assert_eq!(canonical_language("ts"), canonical_language("typescript"));
    assert_eq!(canonical_language("TSX"), canonical_language("typescript"));
    assert_eq!(canonical_language("js"), canonical_language("JavaScript"));
    assert_eq!(canonical_language("py"), canonical_language("Python"));
    assert_eq!(canonical_language("golang"), canonical_language("go"));
    assert_eq!(canonical_language("c++"), canonical_language("cpp"));
    assert_eq!(canonical_language("rs"), canonical_language("rust"));
}

/// Distinct languages must stay distinct — the whole point of the filter.
#[test]
fn distinct_languages_do_not_collide() {
    for (a, b) in [
        ("python", "go"),
        ("typescript", "javascript"),
        ("rust", "c"),
        ("cpp", "c"),
    ] {
        assert_ne!(
            canonical_language(a),
            canonical_language(b),
            "{a} and {b} must not select each other"
        );
    }
}

/// An unrecognised language is passed through lowercased rather than mapped to
/// some plausible-looking default, so `--language kotlin` selects Kotlin files
/// and nothing else.
#[test]
fn unknown_languages_are_passed_through_lowercased() {
    assert_eq!(canonical_language("Kotlin"), "kotlin");
    assert_eq!(canonical_language("SWIFT"), "swift");
}
