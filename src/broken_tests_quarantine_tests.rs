//! #1023 — the `broken-tests` quarantine is 47 declarations nothing checks.
//!
//! `broken-tests` is an empty feature that is deliberately in no bundle, so
//! every `#[cfg(all(test, pmat_broken_tests))]` item is compiled out of
//! every build anybody runs. That is the intent — they do not compile. The
//! consequence is that everything *about* those declarations is unchecked too,
//! including the one thing that is mechanically checkable: whether the
//! `#[path = "..."]` beside them names a file that exists.
//!
//! It did not, twice. `refactor_auto_handlers/output_handler.rs` pointed at a
//! sibling `refactor_auto_handlers_tests.rs` while the real 40 KB file sat one
//! directory above, under a comment that claimed the file was "missing";
//! `work_handlers/ticket_handlers.rs` had the identical defect under a comment
//! blaming a broken syntax split. Neither could ever have produced the error its
//! comment described, because neither path resolved. A `#[path]` under a
//! disabled `cfg` is never read by rustc, so nothing said so for two years.
//!
//! The first of those two has since been repaired and taken off the quarantine,
//! which is why the ceiling below reads 47 rather than the 48 #1023 counted. Its
//! own structural guards live beside it in
//! `cli/handlers/refactor_auto_handlers/output_handler.rs`.
//!
//! These tests do not judge whether a quarantine is justified — that requires
//! compiling each site, which by construction cannot be done here. They pin the
//! two properties that survive the `cfg`: the declarations point at real files,
//! and the quarantine may only shrink.

use std::path::{Path, PathBuf};

/// The quarantine size after the first #1023 repair.
///
/// #1023 counted 48; `refactor_auto_handlers_tests.rs` and its five
/// `refactor_auto_comprehensive_tests_*` include!s have since been repaired and
/// re-enabled, so the ceiling comes down to 47.
///
/// A ceiling, never a target. It may only go down. It is not an equality
/// because deleting a site is progress and must not fail the build; adding one
/// must, because 47 disabled test modules is already 47 places where tests
/// exist, are tracked, are counted by every file-based metric, and never run.
const QUARANTINE_CEILING: usize = 47;

/// The attribute that marks a quarantined item.
const MARKER: &str = "pmat_broken_tests";

fn src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// One quarantined declaration: the file it lives in, its line, and the
/// `#[path]` value that follows it, if any.
struct Site {
    file: PathBuf,
    line: usize,
    path_attr: Option<String>,
}

fn quarantine_sites() -> Vec<Site> {
    let mut files = Vec::new();
    rust_files(&src_root(), &mut files);
    files.sort();

    let mut sites = Vec::new();
    for file in files {
        let Ok(body) = std::fs::read_to_string(&file) else {
            continue;
        };
        let lines: Vec<&str> = body.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            // An ATTRIBUTE, not a mention. This file, and several comments in
            // the tree, name the feature in prose; counting those would inflate
            // the census and make the ceiling meaningless.
            if !line.contains(MARKER) || !line.trim_start().starts_with("#[") {
                continue;
            }
            // The `#[path]` may be on either side of the `cfg`.
            let mut path_attr = None;
            for probe in [i.checked_sub(1), Some(i + 1)].into_iter().flatten() {
                let Some(candidate) = lines.get(probe) else {
                    continue;
                };
                if let Some(rest) = candidate.trim().strip_prefix("#[path = \"") {
                    if let Some(end) = rest.find('"') {
                        path_attr = Some(rest[..end].to_string());
                    }
                }
            }
            sites.push(Site {
                file: file.clone(),
                line: i + 1,
                path_attr,
            });
        }
    }
    sites
}

/// The census must actually find the quarantine.
///
/// Without this the two tests below pass over an empty list the moment the
/// walker breaks — the failure mode this whole file exists to reject.
#[test]
fn the_census_finds_the_quarantine() {
    let sites = quarantine_sites();
    assert!(
        sites.len() >= 30,
        "only {} quarantined declarations found — the source walk is broken and \
         every assertion below is vacuous",
        sites.len()
    );
}

/// Every quarantined `#[path]` names a file that exists.
///
/// This is the check rustc cannot do: the item is `cfg`'d out, so the path is
/// never resolved. Two of the 48 sites were wrong when #1023 was filed, and both
/// sat under a comment that described an error the compiler could never have
/// reached, because it could not find the file to read.
#[test]
fn every_quarantined_path_attribute_resolves() {
    let mut dangling = Vec::new();
    for site in quarantine_sites() {
        let Some(rel) = &site.path_attr else {
            continue;
        };
        let dir = site.file.parent().unwrap_or_else(|| Path::new("."));
        let target = dir.join(rel);
        if !target.exists() {
            dangling.push(format!(
                "{}:{} -> #[path = {rel:?}] (resolves to {})",
                site.file.display(),
                site.line,
                target.display()
            ));
        }
    }
    assert!(
        dangling.is_empty(),
        "these quarantined modules name a file that does not exist:\n  {}\n\n\
         A `#[path]` under a disabled `cfg` is never checked by rustc, so a wrong \
         one is invisible — and it makes the comment above it describe an error \
         nothing could have produced. Fix the path or delete the declaration.",
        dangling.join("\n  ")
    );
}

/// The quarantine may only shrink.
#[test]
fn the_quarantine_only_shrinks() {
    let sites = quarantine_sites();
    assert!(
        sites.len() <= QUARANTINE_CEILING,
        "{} quarantined test declarations against a ceiling of {QUARANTINE_CEILING}. \
         Every one is a test that exists, is tracked, is counted by file-based \
         metrics and never runs. Repair the tests or delete them; raising the \
         ceiling records the debt without paying it.",
        sites.len()
    );
}

/// The marker check must reject a prose mention.
///
/// `quarantine_sites` requires the line to begin an attribute. Without that,
/// this file's own module documentation — and the doc comment in
/// `qa_work_handler/mod.rs` that quotes the cfg — would be counted as sites,
/// and the census would report a quarantine larger than the one that exists.
#[test]
fn a_prose_mention_is_not_a_site() {
    let prose = format!("//! gated behind `#[cfg(all(test, {MARKER}))]`");
    assert!(!prose.trim_start().starts_with("#["));
    let attribute = format!("#[cfg(all(test, {MARKER}))]");
    assert!(attribute.trim_start().starts_with("#["));
}
