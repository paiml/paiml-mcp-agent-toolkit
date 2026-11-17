//! RUSTSEC-2026-0543: chrono is unmaintained (all versions affected, no fixed
//! release, no vulnerability). pmat uses chrono directly and reaches it through
//! several dependencies, so it cannot be dropped in one change. `deny.toml` and
//! `.cargo/audit.toml` carry ONE dated ignore for that id (#1814).
//!
//! The ignore is only honest while it is bounded; these tests pin it:
//!
//! * PRESENT exactly while chrono is locked: the row must go when chrono leaves
//!   `Cargo.lock`, and must exist while it is there.
//! * EXPIRING: the row carries `added YYYY-MM-DD` and `review by YYYY-MM-DD`;
//!   the review date must lie within 45 days of the added date (so a far-future
//!   date cannot pass) and must not be in the past.
//! * IN SYNC: both files carry the id with the identical reason text, and every
//!   chrono version in `Cargo.lock` is named in the reason.
//!
//! Reads files only: no network, no advisory database, no cargo subprocess.
//! Registered from `lib.rs` so the `cargo test --lib` job runs it.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const ID: &str = "RUSTSEC-2026-0543";
const CRATE: &str = "chrono";
const REVIEW_MARK: &str = "review by ";
const ADDED_MARK: &str = "added ";
const MAX_WINDOW_DAYS: i64 = 45;

fn read(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))
        .expect("the files this check pins must be readable")
}

/// Every version of `CRATE` locked in the given `Cargo.lock` text.
fn locked_versions(lock: &str) -> Vec<String> {
    let mut out = Vec::new();
    for block in lock.split("[[package]]").skip(1) {
        let mut name = None;
        let mut version = None;
        for line in block.lines().map(str::trim) {
            if let Some(v) = line.strip_prefix("name = ") {
                name = Some(v.trim_matches('"').to_string());
            } else if let Some(v) = line.strip_prefix("version = ") {
                version = Some(v.trim_matches('"').to_string());
            }
        }
        if name.as_deref() == Some(CRATE) {
            out.extend(version);
        }
    }
    out
}

/// The reason text of the deny.toml row for `ID`, if the row exists.
fn deny_reason(deny: &str) -> Option<String> {
    let section = deny.split("\n[advisories]").nth(1)?.split("\n[").next()?;
    let line = section
        .lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('#'))
        .find(|l| l.contains(ID))?;
    let reason = line.split("reason = \"").nth(1)?;
    Some(reason.split("\" }").next()?.to_string())
}

/// The trailing comment of the audit.toml row for `ID`, if the row exists.
fn audit_reason(audit: &str) -> Option<String> {
    audit
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with(&format!("\"{ID}\"")))
        .find_map(|l| l.split("# ").nth(1).map(str::to_string))
}

/// Days since 1970-01-01 for a civil date (Howard Hinnant's algorithm).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The `YYYY-MM-DD` right after `mark` in a reason, as days since the epoch.
fn date_after(reason: &str, mark: &str) -> Option<i64> {
    let date = reason.split(mark).nth(1)?.get(..10)?;
    let mut parts = date.split('-').map(|p| p.parse::<i64>());
    let (y, m, d) = (
        parts.next()?.ok()?,
        parts.next()?.ok()?,
        parts.next()?.ok()?,
    );
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    Some(days_from_civil(y, m, d))
}

fn today_days() -> i64 {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before 1970")
        .as_secs();
    (secs / 86_400) as i64
}

/// Pure check so it can be exercised against doctored inputs and a forced date.
/// `Ok(())` means the ignore is still bounded; `Err` names what is wrong.
fn bounded(lock: &str, deny: &str, audit: &str, today: i64) -> Result<(), String> {
    let versions = locked_versions(lock);
    let Some(reason) = deny_reason(deny) else {
        return if versions.is_empty() {
            Ok(())
        } else {
            Err(format!("{CRATE} is locked but deny.toml has no {ID} row"))
        };
    };
    if versions.is_empty() {
        return Err(format!(
            "{CRATE} is not in Cargo.lock: delete the {ID} rows (deny.toml and .cargo/audit.toml)"
        ));
    }
    if audit_reason(audit).as_deref() != Some(reason.as_str()) {
        return Err(format!(
            "{ID} in .cargo/audit.toml is missing or differs from deny.toml"
        ));
    }
    for v in &versions {
        if !reason.contains(&format!("{CRATE} {v}")) {
            return Err(format!("reason does not name the locked {CRATE} {v}"));
        }
    }
    let due = date_after(&reason, REVIEW_MARK)
        .ok_or_else(|| format!("reason carries no valid `{REVIEW_MARK}YYYY-MM-DD` date"))?;
    let added = date_after(&reason, ADDED_MARK)
        .ok_or_else(|| format!("reason carries no valid `{ADDED_MARK}YYYY-MM-DD` date"))?;
    if due > added + MAX_WINDOW_DAYS {
        return Err(format!(
            "review-by is more than {MAX_WINDOW_DAYS} days after the row was added"
        ));
    }
    if due < today {
        return Err(format!(
            "the {ID} row passed its review-by date: re-review it, or fix the root cause and delete it"
        ));
    }
    Ok(())
}

fn files() -> (String, String, String) {
    (
        read("Cargo.lock"),
        read("deny.toml"),
        read(".cargo/audit.toml"),
    )
}

#[test]
fn the_ignore_row_is_bounded_today() {
    let (lock, deny, audit) = files();
    assert_eq!(bounded(&lock, &deny, &audit, today_days()), Ok(()));
}

#[test]
fn the_row_is_present_while_chrono_is_locked() {
    let (lock, deny, audit) = files();
    assert!(
        !locked_versions(&lock).is_empty(),
        "{CRATE} left Cargo.lock: delete the row and this module"
    );
    assert!(deny_reason(&deny).is_some(), "deny.toml lost the {ID} row");
    assert!(
        audit_reason(&audit).is_some(),
        ".cargo/audit.toml lost the {ID} row"
    );
}

#[test]
fn the_check_is_red_once_the_review_by_date_has_passed() {
    let (lock, deny, audit) = files();
    let reason = deny_reason(&deny).expect("row present");
    let due = date_after(&reason, REVIEW_MARK).expect("row carries a review-by date");
    assert!(
        bounded(&lock, &deny, &audit, due).is_ok(),
        "due day itself is still in bounds"
    );
    assert!(
        bounded(&lock, &deny, &audit, due + 1).is_err(),
        "a day after the due date must be red"
    );
}

#[test]
fn the_check_is_red_when_the_row_is_missing_or_the_files_drift() {
    let (lock, deny, audit) = files();
    let today = today_days();
    let no_row = deny.replace(ID, "RUSTSEC-0000-0000");
    assert!(
        bounded(&lock, &no_row, &audit, today).is_err(),
        "locked chrono without a row"
    );
    let drift = audit.replace("unmaintained notice", "something else");
    assert!(
        bounded(&lock, &deny, &drift, today).is_err(),
        "audit.toml text differs"
    );
    let no_date = deny.replace(REVIEW_MARK, "reviewed ");
    let no_date_audit = audit.replace(REVIEW_MARK, "reviewed ");
    assert!(
        bounded(&lock, &no_date, &no_date_audit, today).is_err(),
        "no review-by date"
    );
}

#[test]
fn a_far_future_review_by_date_is_red() {
    let (lock, deny, audit) = files();
    let far = |s: &str| s.replace("review by 2026-12-17", "review by 2099-12-17");
    assert!(
        bounded(&lock, &far(&deny), &far(&audit), today_days()).is_err(),
        "a 2099 review-by date must not pass"
    );
}

#[test]
fn a_row_that_outlives_chrono_is_red() {
    let (lock, deny, audit) = files();
    let gone = lock.replace("name = \"chrono\"", "name = \"not-chrono\"");
    assert!(
        bounded(&gone, &deny, &audit, today_days()).is_err(),
        "stale row after chrono left the lock"
    );
}

#[test]
fn every_locked_chrono_version_must_be_named_in_the_reason() {
    let (lock, deny, audit) = files();
    let version = locked_versions(&lock).remove(0);
    let bumped = lock.replacen(
        &format!("version = \"{version}\""),
        "version = \"0.4.99\"",
        1,
    );
    assert!(
        bounded(&bumped, &deny, &audit, today_days()).is_err(),
        "stale version in the reason"
    );
    let second = format!("{lock}\n[[package]]\nname = \"chrono\"\nversion = \"0.3.0\"\n");
    assert_eq!(
        locked_versions(&second).len(),
        locked_versions(&lock).len() + 1
    );
    assert!(
        bounded(&second, &deny, &audit, today_days()).is_err(),
        "a second chrono must be named too"
    );
}
