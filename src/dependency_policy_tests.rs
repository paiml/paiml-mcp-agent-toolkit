//! Lockfile dependency policy — the assertions behind #1075.
//!
//! `thrift 0.17.0` (GHSA-2f9f-gq7v-9h6m) reached pmat through exactly one edge:
//! `aprender-db -> parquet ^57 -> thrift ^0.17`. `parquet 59` dropped the
//! dependency outright, so the fix was to move `aprender-db` to 0.64 (which
//! requires `arrow`/`parquet` ^59) and `arrow` to 59 in the same change.
//!
//! Registered from `lib.rs` rather than dropped under `tests/` because
//! `autotests = false` (`Cargo.toml:30`) means an unregistered test file is
//! silently never compiled — and a dependency gate that does not run is worse
//! than no gate, because it reads as one. `cargo test --lib -- dependency_policy`.

/// `Cargo.lock` as committed.
///
/// `include_str!` rather than a runtime `read_to_string` so the scan cannot
/// degrade into "file not found ⇒ nothing found", which is the exact failure
/// mode that would make every absence assertion below pass vacuously. Cargo
/// ships `Cargo.lock` inside the published `.crate` (verified against
/// `target/package/pmat-3.31.0/Cargo.lock`), so this also compiles during
/// `cargo package` verification.
const CARGO_LOCK: &str = include_str!("../Cargo.lock");

/// `Cargo.toml` as committed, for cross-checking the manifest pin against what
/// actually resolved.
const CARGO_TOML: &str = include_str!("../Cargo.toml");

/// The value of a `key = "value"` line, or `None` if this line is not that key.
///
/// Extracted so `locked_versions` stays flat: inlining the two `strip_*` steps
/// nests them three deep inside the loop and costs more cognitive complexity
/// than the whole rest of the module put together.
fn quoted_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    line.strip_prefix(key)?.strip_suffix('"')
}

/// Every version recorded for `name` in `Cargo.lock`.
///
/// Empty means the crate is not in the dependency graph at all; more than one
/// entry means two semver-incompatible majors are linked in simultaneously.
fn locked_versions(name: &str) -> Vec<&'static str> {
    let mut found = Vec::new();
    let mut is_target = false;
    for raw in CARGO_LOCK.lines() {
        let line = raw.trim();
        if line == "[[package]]" {
            is_target = false;
        } else if let Some(package) = quoted_value(line, "name = \"") {
            is_target = package == name;
        } else if is_target {
            found.extend(quoted_value(line, "version = \""));
        }
    }
    found
}

/// Leading numeric component of a semver string (`"59.2.0"` -> `59`).
///
/// Returns `None` rather than defaulting, so a malformed version fails the
/// assertion that reads it instead of silently comparing as zero.
fn major(version: &str) -> Option<u64> {
    version.split('.').next().and_then(|m| m.parse().ok())
}

/// The version string of a top-level `name = { version = "..." }` dependency
/// line in `Cargo.toml`.
fn manifest_pin(name: &str) -> Option<&'static str> {
    let prefix = format!("{name} = ");
    CARGO_TOML
        .lines()
        .find(|line| line.starts_with(&prefix))
        .and_then(|line| line.split_once("version = \""))
        .and_then(|(_, rest)| rest.split('"').next())
}

/// #1075. RED before the fix: `thrift 0.17.0` was in `Cargo.lock` via
/// `parquet 57.3.1`.
#[test]
fn dependency_policy_thrift_is_absent_from_the_lockfile() {
    let found = locked_versions("thrift");
    assert!(
        found.is_empty(),
        "GHSA-2f9f-gq7v-9h6m: thrift is back in Cargo.lock at {found:?}. \
         Its only route into pmat is parquet <59 (parquet 59 removed the \
         dependency), so this means arrow/parquet or aprender-db moved \
         backwards. Confirm with `cargo tree -i thrift`."
    );
}

/// Over-correction guard for the assertion above.
///
/// "thrift is absent" passes for the wrong reasons too: a broken parse, a
/// changed lockfile format, or a wrong include path all report *everything* as
/// absent. Anything that would make thrift look absent spuriously makes these
/// look absent as well, so this test fails first.
#[test]
fn dependency_policy_absence_scan_can_actually_find_crates() {
    for present in ["serde", "tokio", "arrow", "parquet", "aprender-db"] {
        assert!(
            !locked_versions(present).is_empty(),
            "the Cargo.lock scan found no `{present}`, so it is not reading the \
             lockfile — every absence assertion in this module is vacuous until \
             this is fixed"
        );
    }
}

/// #1075, second half. Over-correction guard: dropping the analytics stack
/// altogether would also remove thrift, and would pass the absence test above.
/// arrow and parquet must still be present, at one major each, and that major
/// must be >= 59 — the release in which parquet stopped depending on thrift.
#[test]
fn dependency_policy_arrow_and_parquet_are_lockstep_at_59_or_newer() {
    let arrow = locked_versions("arrow");
    let parquet = locked_versions("parquet");
    assert_eq!(
        arrow.len(),
        1,
        "arrow must be linked exactly once, found {arrow:?}"
    );
    assert_eq!(
        parquet.len(),
        1,
        "parquet must be linked exactly once, found {parquet:?}"
    );

    let arrow_major = major(arrow[0]);
    let parquet_major = major(parquet[0]);
    assert_eq!(
        arrow_major, parquet_major,
        "arrow {arrow:?} and parquet {parquet:?} are released in lockstep and \
         share types; different majors is a type-identity split"
    );
    assert!(
        parquet_major >= Some(59),
        "parquet {parquet:?} is older than 59, the release that dropped the \
         thrift dependency"
    );
    assert_eq!(
        manifest_pin("arrow").and_then(major),
        arrow_major,
        "Cargo.toml pins arrow {:?} but Cargo.lock resolved {arrow:?}; the pin \
         MUST match aprender-db's arrow because RecordBatch/arrays cross the \
         aprender-db (lib `trueno_db`) boundary",
        manifest_pin("arrow")
    );
}

/// Version splits that predate #1075 and sit outside the arrow/type-identity
/// boundary the test below protects.
///
/// Each entry must still be REAL: `versions_outside_accepted_splits` fails when
/// an allowance stops matching anything, so this list cannot decay into a
/// blanket exemption for whatever splits later.
const ACCEPTED_SPLITS: &[(&str, &str, &str)] = &[(
    "aprender",
    "0.25.9",
    "ruchy 4.2.1, behind pmat's optional `ruchy-ast` feature, pins aprender \
     0.25.9; it is not on the arrow/trueno_db path and the split predates #1075",
)];

/// `locked_versions` minus the entries `ACCEPTED_SPLITS` records, having first
/// checked that each allowance is still load-bearing.
fn versions_outside_accepted_splits(crate_name: &str) -> Vec<&'static str> {
    let mut versions = locked_versions(crate_name);
    for (name, allowed, why) in ACCEPTED_SPLITS.iter().filter(|s| s.0 == crate_name) {
        assert!(
            versions.contains(allowed),
            "ACCEPTED_SPLITS still exempts {name} {allowed} ({why}), but that \
             version is no longer in Cargo.lock. Delete the entry — a stale \
             allowance is an exemption nobody is watching."
        );
        versions.retain(|v| v != allowed);
    }
    versions
}

/// Over-correction guard for the *shape* of the #1075 fix.
///
/// Bumping `aprender-db` alone resolves to two majors of the sovereign stack
/// (aprender-db 0.64 requires aprender-compute ^0.64, while aprender-graph 0.61
/// requires ^0.61), and pmat passes types across those boundaries. That failure
/// surfaces as "expected `RecordBatch`, found `RecordBatch`", not as a version
/// conflict, so it is worth pinning here rather than discovering it in a build
/// log.
#[test]
fn dependency_policy_sovereign_stack_is_linked_at_one_version() {
    for crate_name in [
        "aprender",
        "aprender-core",
        "aprender-common",
        "aprender-compute",
        "aprender-db",
        "aprender-graph",
        "aprender-rag",
        "aprender-viz",
        "aprender-contracts-macros",
    ] {
        let versions = versions_outside_accepted_splits(crate_name);
        assert_eq!(
            versions.len(),
            1,
            "{crate_name} is linked at {versions:?}. Two majors of the aprender \
             stack in one binary means two incompatible copies of the same \
             types; bump the whole family together."
        );
    }
}
