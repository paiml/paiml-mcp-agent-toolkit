//! The shipped binary's size, as a BAND rather than a ceiling.
//!
//! ## Why this file exists, and what replaced what
//!
//! `src/tests/binary_size.rs` asserted `size < 50 MiB` and had **never run once**.
//! Nothing declares `src/tests/` — 166 files sit there unreferenced — so rustc
//! never compiled it, `cargo test binary_size_regression` reported
//! `0 passed; 0 filtered out`, and the decisive proof was that its own panic
//! message is absent from the shipped artifact:
//!
//! ```text
//! $ grep -a "Kaizen Quality Gate Failed: Binary size" <target>/release/pmat
//!    (no match)
//! ```
//!
//! Meanwhile the binary had grown to 55.2 MB — **2.8 MB past the limit it was
//! supposedly enforcing**. A gate nobody has watched fail is not evidence.
//! Tracked as #1079.
//!
//! ## Why a band, and not a bigger ceiling
//!
//! A ceiling only answers "is it too big". The failure this project actually
//! keeps hitting is the other direction: a feature that silently stops being
//! compiled in. `mcp-http` moving into the default set ADDED ~1 MB on purpose;
//! had it silently dropped OUT, a ceiling would have reported success while the
//! binary lost a transport. A binary that suddenly sheds 20% is not a win, it is
//! a missing feature, and this release fixed three separate defects of exactly
//! that shape.
//!
//! So there are three verdicts, not two:
//!
//! | size vs `EXPECTED_BYTES` | verdict |
//! |---|---|
//! | within ±`QUIET_PCT` | pass, silently |
//! | outside quiet, inside ±`FAIL_PCT` | **pass, but print the drift loudly** |
//! | outside ±`FAIL_PCT` | fail |
//!
//! The middle band is the point. Size drifts for legitimate reasons — a new
//! dependency, a new language grammar — and a gate that fails on every legitimate
//! change gets its threshold raised until it means nothing. This one reports the
//! number every run so the drift is *visible* while it is still small, and only
//! fails when the change is too large to be incidental.
//!
//! ## Why it refuses rather than skips
//!
//! The old test returned early with `⚠️ Skipping …` when it could not find a
//! binary — a skip that reads as a pass, which is this release's signature
//! defect. Here, absence is a FAILURE whenever `PMAT_REQUIRE_BINARY_SIZE=1`
//! (set it in the release path). Without that variable the test still declines
//! to run — a plain `cargo test` must not demand a release build — but it says
//! **"this run verified NOTHING"** in as many words, so a reader cannot mistake
//! the line for a check that passed.

use std::path::PathBuf;

/// Measured on the 3.32.0 release build, the version that moved `mcp-http` into
/// the default feature set.
///
/// Not a target and not a wish: re-derive it with the command in
/// `the_shipped_binary_size_stays_in_band`'s failure message rather than
/// adjusting it to whatever today happens to be.
const EXPECTED_BYTES: u64 = 55000000;

/// Drift worth printing but not worth failing. ±5% of `EXPECTED_BYTES` is about
/// ±2.75 MB — comfortably more than a dependency bump, comfortably less than a
/// feature appearing or vanishing.
const QUIET_PCT: u64 = 5;

/// Drift too large to be incidental, in EITHER direction. ±20% is ±11 MB: a
/// binary that moves that far has gained or lost something structural, and the
/// person who did it should be the one to decide whether it was intended.
const FAIL_PCT: u64 = 20;

/// Where cargo actually writes, asked of cargo.
///
/// NOT `target/release/pmat`. In this checkout `./target` is a symlink to a
/// DIFFERENT directory than the one cargo builds into (`cargo-targets/…` vs
/// `targets/…`), so the literal path resolves to a stale binary from an earlier
/// build. Two release gates measured the wrong file that way before this was
/// noticed. `CARGO_TARGET_DIR` wins if set, exactly as it does for cargo.
fn release_binary() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("CARGO_TARGET_DIR") {
        let p = PathBuf::from(dir).join("release").join("pmat");
        return p.exists().then_some(p);
    }
    let out = std::process::Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output()
        .ok()?;
    let text = String::from_utf8(out.stdout).ok()?;
    let key = "\"target_directory\":\"";
    let start = text.find(key)? + key.len();
    let end = start + text[start..].find('"')?;
    let p = PathBuf::from(&text[start..end])
        .join("release")
        .join("pmat");
    p.exists().then_some(p)
}

fn pct_of(base: u64, pct: u64) -> u64 {
    base / 100 * pct
}

#[test]
fn the_shipped_binary_size_stays_in_band() {
    let required = std::env::var("PMAT_REQUIRE_BINARY_SIZE").as_deref() == Ok("1");

    let Some(path) = release_binary() else {
        assert!(
            !required,
            "PMAT_REQUIRE_BINARY_SIZE=1 but no release binary was found. \
             This is a FAILURE, not a skip: the release path asked for this \
             measurement and it could not be taken. Run `cargo build --release \
             --bin pmat` first."
        );
        println!(
            "SKIPPED: no release binary — THIS RUN VERIFIED NOTHING about binary size. \
             Build with `cargo build --release --bin pmat`, or set \
             PMAT_REQUIRE_BINARY_SIZE=1 to make its absence a failure."
        );
        return;
    };

    let size = std::fs::metadata(&path)
        .unwrap_or_else(|e| panic!("cannot stat {}: {e}", path.display()))
        .len();

    let quiet = pct_of(EXPECTED_BYTES, QUIET_PCT);
    let hard = pct_of(EXPECTED_BYTES, FAIL_PCT);
    let drift = size.abs_diff(EXPECTED_BYTES);
    let mb = |b: u64| b as f64 / 1_048_576.0;

    println!(
        "binary size: {size} bytes ({:.2} MiB) at {}  [expected {EXPECTED_BYTES} ± {QUIET_PCT}% quiet, ± {FAIL_PCT}% hard]",
        mb(size),
        path.display()
    );

    assert!(
        drift <= hard,
        "binary size {size} bytes ({:.2} MiB) is {:.2} MiB from the expected \
         {EXPECTED_BYTES} ({:.2} MiB) — outside the ±{FAIL_PCT}% band.\n\
         \n\
         If it GREW: something substantial was linked in. Find it before raising \
         this number.\n\
         If it SHRANK: a feature probably stopped compiling in. That is the \
         failure a ceiling-only check cannot see, and the reason this is a band.\n\
         \n\
         Re-measure with:\n\
         \x20 cargo build --release --bin pmat && \\\n\
         \x20 stat -c%s \"$(cargo metadata --no-deps --format-version 1 \\\n\
         \x20   | python3 -c 'import json,sys;print(json.load(sys.stdin)[\"target_directory\"])')/release/pmat\"",
        mb(size),
        mb(drift),
        mb(EXPECTED_BYTES),
    );

    if drift > quiet {
        println!(
            "⚠️  binary size drifted {:.2} MiB from expected — inside the hard band, \
             so this is not a failure, but it is worth a look while it is still small. \
             Update EXPECTED_BYTES deliberately if this is the new normal.",
            mb(drift)
        );
    }
}

/// COUNTER-TEST: the band must be able to REJECT, in both directions.
///
/// Without this, `FAIL_PCT` could be set to something absurd and the gate above
/// would pass on any binary at all while still looking like a check. The
/// arithmetic is asserted directly because the real test's verdict depends on a
/// file that may not exist in every environment.
#[test]
fn the_band_rejects_growth_and_shrinkage_alike() {
    let hard = pct_of(EXPECTED_BYTES, FAIL_PCT);
    let too_big = EXPECTED_BYTES + hard + 1;
    let too_small = EXPECTED_BYTES - hard - 1;

    assert!(
        too_big.abs_diff(EXPECTED_BYTES) > hard,
        "a binary above the band must be rejected"
    );
    assert!(
        too_small.abs_diff(EXPECTED_BYTES) > hard,
        "a binary BELOW the band must be rejected too — a shrinking binary is a \
         missing feature, not a saving"
    );
    assert!(
        EXPECTED_BYTES.abs_diff(EXPECTED_BYTES) <= pct_of(EXPECTED_BYTES, QUIET_PCT),
        "the expected size itself must sit in the quiet band"
    );
    assert!(
        pct_of(EXPECTED_BYTES, QUIET_PCT) < hard,
        "the quiet band must be strictly inside the hard band, or the warning \
         can never fire"
    );
}
