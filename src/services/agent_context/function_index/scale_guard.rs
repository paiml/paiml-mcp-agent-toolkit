//! The ONE implementation of the "persisted TDG scale must match this build" rule.
//!
//! R30. Before v3.30.0 the index persisted a 0-10 lower-is-better *debt* number
//! in `functions.tdg_score` and a five-letter grade in `functions.tdg_grade`.
//! Today those columns hold a 0-100 higher-is-better *quality* score and an
//! eleven-band [`crate::tdg::Grade`]. Both scales fit the same `REAL`/`TEXT`
//! columns, so a stale index passes every structural check while every stored
//! `0.12` — the BEST possible legacy score — reads as 0.12/100, an F.
//!
//! The rule used to be implemented three times with three different answers:
//! `stored_scale_is_current` for the SQLite branch, an inline `manifest
//! .tdg_scale !=` comparison for the blob branch, and *nothing at all* on
//! `pmat sql`, which therefore printed legacy scores with exit 0. Every surface
//! now calls into this module: [`stale_scale_reason`] is the only place that
//! decides, and [`discard_stale_index`] is the only place that remediates.

use std::path::Path;

use crate::services::agent_context::TDG_SCALE;

/// How a missing marker is reported. Pre-v3.30.0 builds wrote no marker at all,
/// so "absent" is precisely the stale case — an unmeasured signal is a distinct
/// state from a clean one and must never pass as one.
const UNMARKED: &str = "unmarked (pre-v3.30.0, 0-10 lower-is-better)";

/// Decide whether a persisted scale marker may be read by this build.
///
/// `found` is the marker exactly as stored: `None` when the artifact carries no
/// marker (SQLite `metadata` row absent, or the table itself missing), and
/// `Some("")` when a manifest deserialised the missing key to its default.
/// Both are stale.
///
/// Returns `None` when the index is readable, `Some(reason)` when it is not.
pub(crate) fn stale_scale_reason(found: Option<&str>) -> Option<String> {
    let found = found.unwrap_or("");
    if found == TDG_SCALE {
        return None;
    }
    let described = if found.is_empty() { UNMARKED } else { found };
    Some(format!(
        "index was written under TDG scale {described}, this build reads {TDG_SCALE}; rebuild required"
    ))
}

/// Read the `tdg_scale` marker out of an already-open index database.
///
/// `None` covers every way the marker can be absent (no `metadata` table, no
/// row, unreadable value) because they all mean the same thing: unmeasured.
pub(crate) fn db_scale(conn: &rusqlite::Connection) -> Option<String> {
    conn.query_row(
        "SELECT value FROM metadata WHERE key = 'tdg_scale'",
        [],
        |r| r.get::<_, String>(0),
    )
    .ok()
}

/// Read the `tdg_scale` marker out of an index directory's `manifest.json`.
///
/// `Ok(None)` means "no manifest to judge" — the caller's own error path takes
/// over. `Ok(Some(scale))` is the stored marker (`""` for pre-v3.30.0).
fn manifest_scale(index_path: &Path) -> Option<String> {
    let raw = std::fs::read_to_string(index_path.join("manifest.json")).ok()?;
    let json: serde_json::Value = serde_json::from_str(&raw).ok()?;
    Some(
        json.get("tdg_scale")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    )
}

/// Verify a persisted index directory (SQLite `<index>.db` preferred, else the
/// blob `manifest.json`) against this build's scale.
///
/// Returns `Ok(())` when there is nothing to judge — an absent index is not a
/// stale one, and the caller's own "not found" error is the better message.
pub(crate) fn verify_index_scale(index_path: &Path) -> Result<(), String> {
    let db_candidate = index_path.with_extension("db");
    if db_candidate.exists() {
        if let Ok(conn) = open_readonly(&db_candidate) {
            return match stale_scale_reason(db_scale(&conn).as_deref()) {
                Some(reason) => Err(reason),
                None => Ok(()),
            };
        }
    }
    match manifest_scale(index_path) {
        Some(scale) => match stale_scale_reason(Some(&scale)) {
            Some(reason) => Err(reason),
            None => Ok(()),
        },
        None => Ok(()),
    }
}

/// Verify a single index database file against this build's scale.
///
/// This is the entry point for readers that hold a `.db` path rather than an
/// index directory — `pmat sql`, which lets a user run arbitrary SQL over
/// `functions.tdg_score` / `functions.tdg_grade` and so must refuse a database
/// written on a scale it cannot interpret.
pub fn verify_db_scale(db_path: &Path) -> Result<(), String> {
    let conn =
        open_readonly(db_path).map_err(|e| format!("Failed to open {}: {e}", db_path.display()))?;
    match stale_scale_reason(db_scale(&conn).as_deref()) {
        Some(reason) => Err(reason),
        None => Ok(()),
    }
}

fn open_readonly(db_path: &Path) -> Result<rusqlite::Connection, rusqlite::Error> {
    rusqlite::Connection::open_with_flags(
        db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
}

/// Discard an index this build cannot read, so the NEXT load rebuilds it.
///
/// Both persisted artifacts go: the SQLite `<index>.db` and the `<index>`
/// directory holding `manifest.json` (plus any legacy LZ4 blob). Deleting them
/// — rather than only reporting the mismatch — is what makes recovery uniform.
/// The SQLite branch used to delete only its own stale `.db` while the blob
/// branch merely returned `Err`, so the CLI self-healed and MCP's
/// `IndexManager`, which propagates that `Err`, answered
/// `-32603 … rebuild required` on every default call forever.
pub(crate) fn discard_stale_index(index_path: &Path) {
    let _ = std::fs::remove_file(index_path.with_extension("db"));
    let _ = std::fs::remove_dir_all(index_path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_marker_is_readable() {
        assert_eq!(stale_scale_reason(Some(TDG_SCALE)), None);
    }

    #[test]
    fn absent_marker_is_stale_not_clean() {
        // Unmeasured is its own state; it must FAIL, not pass as clean.
        for found in [None, Some("")] {
            let reason = stale_scale_reason(found)
                .unwrap_or_else(|| panic!("{found:?} must be rejected, not accepted as current"));
            assert!(reason.contains(UNMARKED), "got: {reason}");
        }
    }

    #[test]
    fn foreign_marker_is_named_verbatim() {
        let reason = stale_scale_reason(Some("tdg-0-10-lower-is-better"))
            .expect("a different marker must be rejected");
        assert!(reason.contains("tdg-0-10-lower-is-better"), "got: {reason}");
        assert!(reason.contains(TDG_SCALE), "got: {reason}");
    }

    #[test]
    fn verify_db_scale_rejects_db_without_metadata_table() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("legacy.db");
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute_batch("CREATE TABLE functions (id INTEGER PRIMARY KEY, tdg_score REAL);")
            .unwrap();
        drop(conn);

        let err =
            verify_db_scale(&db_path).expect_err("a database with no scale marker must be refused");
        assert!(err.contains("rebuild required"), "got: {err}");
    }

    #[test]
    fn discard_removes_both_artifacts() {
        let dir = tempfile::tempdir().unwrap();
        let index_path = dir.path().join("context.idx");
        std::fs::create_dir_all(&index_path).unwrap();
        std::fs::write(index_path.join("manifest.json"), "{}").unwrap();
        let db_path = index_path.with_extension("db");
        std::fs::write(&db_path, b"stale").unwrap();

        discard_stale_index(&index_path);

        assert!(!db_path.exists(), "stale .db must be removed");
        assert!(!index_path.exists(), "stale index dir must be removed");
    }
}
