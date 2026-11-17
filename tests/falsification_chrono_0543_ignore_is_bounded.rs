//! RUSTSEC-2026-0543: chrono is unmaintained (all versions affected, no fixed
//! release, no vulnerability). pmat uses chrono directly and reaches it through
//! several dependencies, so it cannot be dropped in one change. `deny.toml` and
//! `.cargo/audit.toml` carry ONE dated ignore for that id (#1814).
//!
//! The ignore is only honest while it is bounded; this test pins it:
//!
//! * PRESENT exactly while chrono is locked: the row must go when chrono leaves
//!   `Cargo.lock`, and must exist while it is there.
//! * EXPIRING: the row carries `review by YYYY-MM-DD`; once that date has
//!   passed the test is red until a person re-reviews and moves the date.
//! * IN SYNC: both files carry the id with the identical reason text, and the
//!   chrono version named in the reason equals `Cargo.lock`.
//!
//! Reads files only: no network, no advisory database, no cargo subprocess.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const ID: &str = "RUSTSEC-2026-0543";
const CRATE: &str = "chrono";
const REVIEW_MARK: &str = "review by ";

fn read(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// Version of `CRATE` locked in the given `Cargo.lock` text, if any.
fn locked_version(lock: &str) -> Option<String> {
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
            return version;
        }
    }
    None
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

/// The `YYYY-MM-DD` after `review by ` in a reason, as days since the epoch.
fn review_by_days(reason: &str) -> Option<i64> {
    let date = reason.split(REVIEW_MARK).nth(1)?.get(..10)?;
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
    let locked = locked_version(lock);
    let reason = deny_reason(deny);
    match (&locked, &reason) {
        (None, None) => return Ok(()),
        (None, Some(_)) => {
            return Err(format!(
            "{CRATE} is not in Cargo.lock: delete the {ID} rows (deny.toml and .cargo/audit.toml)"
        ))
        }
        (Some(_), None) => return Err(format!("{CRATE} is locked but deny.toml has no {ID} row")),
        _ => {}
    }
    let (version, reason) = (locked.unwrap(), reason.unwrap());
    if audit_reason(audit).as_deref() != Some(reason.as_str()) {
        return Err(format!(
            "{ID} in .cargo/audit.toml is missing or differs from deny.toml"
        ));
    }
    if !reason.contains(&format!("{CRATE} {version}:")) {
        return Err(format!("reason does not name the locked {CRATE} {version}"));
    }
    match review_by_days(&reason) {
        None => Err(format!("reason carries no valid `{REVIEW_MARK}YYYY-MM-DD` date")),
        Some(due) if due < today => Err(format!(
            "the {ID} row passed its review-by date: re-review it, or fix the root cause and delete it"
        )),
        Some(_) => Ok(()),
    }
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
    if let Err(why) = bounded(&lock, &deny, &audit, today_days()) {
        panic!("{why}");
    }
}

#[test]
fn the_row_is_present_while_chrono_is_locked() {
    let (lock, deny, audit) = files();
    assert!(
        locked_version(&lock).is_some(),
        "{CRATE} left Cargo.lock: delete the row and this test"
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
    let due = review_by_days(&reason).expect("row carries a review-by date");
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
fn a_row_that_outlives_chrono_is_red() {
    let (lock, deny, audit) = files();
    let gone = lock.replace("name = \"chrono\"", "name = \"not-chrono\"");
    assert!(
        bounded(&gone, &deny, &audit, today_days()).is_err(),
        "stale row after chrono left the lock"
    );
}

#[test]
fn the_reason_version_must_match_cargo_lock() {
    let (lock, deny, audit) = files();
    let version = locked_version(&lock).expect("chrono locked");
    let bumped = lock.replacen(
        &format!("version = \"{version}\""),
        "version = \"0.4.99\"",
        1,
    );
    assert_eq!(
        locked_version(&bumped).as_deref(),
        Some("0.4.99"),
        "doctoring hit chrono"
    );
    assert!(
        bounded(&bumped, &deny, &audit, today_days()).is_err(),
        "stale version in the reason"
    );
}
