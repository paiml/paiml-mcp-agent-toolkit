//! Pure comparator kernel for the ratchet (CB-1421) and threshold-coherence
//! (CB-1420) gates.
//!
//! Contracts: `contracts/comply-ratchet-v1.yaml`,
//! `contracts/comply-threshold-coherence-v1.yaml` (both `kind: kernel`).
//!
//! Everything here is total, integer-only and side-effect free so that Kani can
//! discharge the obligations exhaustively. Measurement, I/O and configuration
//! live in [`super::config`] and the comply check that drives them; this file
//! never reads a file or a clock.
//!
//! # Why integers
//!
//! Every ratcheted metric is carried as an `i64` in a unit the config declares
//! (`count`, `basis_points`, …). Floats were rejected: `85.0 > 85.0` is a
//! coin-flip under accumulated rounding, and a gate whose verdict depends on
//! the last bit of a division is a gate that flakes. `basis_points` (1/100 of a
//! percent) gives four significant digits for every percentage this repo
//! records, which is two more than any of them are measured to.

use serde::{Deserialize, Serialize};

/// Outcome of a ratchet comparison (CB-1421).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RatchetVerdict {
    /// Observed is at or below the captured baseline.
    Pass,
    /// Observed exceeded the captured baseline — the metric regressed.
    Fail,
}

/// Which side of a threshold is the bad side.
///
/// `Max` bounds a metric from above (`max_unwrap_calls`): larger is worse.
/// `Min` bounds it from below (`min_coverage_pct`): smaller is worse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    /// The threshold is an upper bound; `measured > limit` breaches it.
    Max,
    /// The threshold is a lower bound; `measured < limit` breaches it.
    Min,
}

/// What a configured threshold actually does against the measurement (CB-1420).
///
/// The three variants partition the space — see [`classify`] and
/// `INV-1403-3`. There is no fourth "unknown" variant on purpose: a threshold
/// whose metric could not be measured is not classified at all, it fails the
/// gate (`FALSIFY-1403-3`, unmeasurable != compliant). Making "unmeasurable" a
/// classification is precisely how an unenforceable number gets to look like a
/// gate in a report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Classification {
    /// The measurement sits inside the band below the limit: the threshold is
    /// live — a plausible regression trips it.
    Firing,
    /// The measurement already breaches the limit. The config asserts a bound
    /// the tree does not satisfy, and the build is green anyway.
    Violated,
    /// The limit is further from the measurement than the band: no regression
    /// the ratchet would tolerate can ever reach it. The number is decoration.
    Vacuous,
}

impl Classification {
    /// Stable lowercase-free wire name, for report rendering.
    pub fn as_str(self) -> &'static str {
        match self {
            Classification::Firing => "FIRING",
            Classification::Violated => "VIOLATED",
            Classification::Vacuous => "VACUOUS",
        }
    }
}

/// `INV-1404-1`: `verdict(b, o) = Fail  <->  o > b`.
///
/// Nothing else is consulted. In particular a metric that *improves* is a
/// `Pass` here and only becomes the new baseline through [`next_baseline`],
/// which the nightly job — never the PR gate — is allowed to write.
pub const fn ratchet_verdict(baseline: i64, observed: i64) -> RatchetVerdict {
    if observed > baseline {
        RatchetVerdict::Fail
    } else {
        RatchetVerdict::Pass
    }
}

/// `INV-1404-2`: `next(b, o) = o` when `o <= b`, else `b`.
///
/// Equivalently `min(b, o)`, which makes the sequence of baselines monotone
/// non-increasing and the operation idempotent (`INV-1404-3`). Written as the
/// explicit two-case form rather than `min` because the two-case form is the
/// contract's formula and a reader should not have to re-derive it.
pub const fn next_baseline(baseline: i64, observed: i64) -> i64 {
    if observed <= baseline {
        observed
    } else {
        baseline
    }
}

/// Slack a limit leaves beyond the measurement, in the metric's unit.
///
/// Negative means the limit is already breached. Computed in `i128` so that
/// `i64::MIN`/`i64::MAX` operands cannot overflow — Kani checks this, and an
/// overflow here would silently invert a verdict.
pub const fn slack(limit: i64, measured: i64, dir: Direction) -> i128 {
    match dir {
        Direction::Max => (limit as i128) - (measured as i128),
        Direction::Min => (measured as i128) - (limit as i128),
    }
}

/// `INV-1403-1` / `INV-1403-2` / `INV-1403-3`: classify one configured
/// threshold against its measurement.
///
/// ```text
/// slack = limit - measured        (Max)
///       = measured - limit        (Min)
///
/// classify = Violated  <->  slack < 0
///          = Vacuous   <->  slack > band
///          = Firing    <->  0 <= slack <= band
/// ```
///
/// The three arms are exhaustive and mutually exclusive by construction, which
/// is what `INV-1403-3` (totality) asks for. `band` is the ratchet's tolerance:
/// the largest movement the ratchet would still let through. A limit further
/// away than that cannot be reached by any change the ratchet permits, so it
/// can never fire.
pub const fn classify(limit: i64, measured: i64, band: u64, dir: Direction) -> Classification {
    let s = slack(limit, measured, dir);
    if s < 0 {
        Classification::Violated
    } else if s > band as i128 {
        Classification::Vacuous
    } else {
        Classification::Firing
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::{
        classify, next_baseline, ratchet_verdict, Classification, Direction, RatchetVerdict,
    };

    /// `KANI-1403-1`: `classify` is total, and its verdict agrees with the
    /// breach predicate (`INV-1403-1`, `INV-1403-2`) and partitions the space
    /// (`INV-1403-3`). Unbounded over every `i64` limit/measurement pair and
    /// every `u64` band — no assumptions.
    #[kani::proof]
    fn verify_classify_total_and_sound() {
        let limit: i64 = kani::any();
        let measured: i64 = kani::any();
        let band: u64 = kani::any();
        let dir = if kani::any::<bool>() {
            Direction::Max
        } else {
            Direction::Min
        };

        let c = classify(limit, measured, band, dir);

        // INV-1403-2: Violated iff the bound is actually breached.
        let breached = match dir {
            Direction::Max => measured > limit,
            Direction::Min => measured < limit,
        };
        assert!((c == Classification::Violated) == breached);

        // INV-1403-3: exactly one of the three holds (totality + disjointness).
        let n = (c == Classification::Firing) as u8
            + (c == Classification::Violated) as u8
            + (c == Classification::Vacuous) as u8;
        assert!(n == 1);
    }

    /// `KANI-1403-2`: `classify` is monotone in `limit` — loosening a `Firing`
    /// threshold never yields `Violated`. This is the anti-gaming property: you
    /// cannot make a live gate report a *worse* class by relaxing it, so
    /// `Violated` can only ever be escaped by fixing the tree or by moving the
    /// limit past the band, which lands on `Vacuous` and demands a
    /// justification.
    #[kani::proof]
    fn verify_classify_monotone_in_limit() {
        let limit: i64 = kani::any();
        let looser: i64 = kani::any();
        let measured: i64 = kani::any();
        let band: u64 = kani::any();

        // Max: looser means larger. Min: looser means smaller.
        kani::assume(looser >= limit);
        if classify(limit, measured, band, Direction::Max) == Classification::Firing {
            assert!(classify(looser, measured, band, Direction::Max) != Classification::Violated);
        }
        let tighter_dir_looser = limit.saturating_sub(looser.saturating_sub(limit));
        if classify(limit, measured, band, Direction::Min) == Classification::Firing {
            assert!(
                classify(tighter_dir_looser, measured, band, Direction::Min)
                    != Classification::Violated
            );
        }
    }

    /// `KANI-1404-1`: `next_baseline` is monotone non-increasing
    /// (`INV-1404-2`), idempotent (`INV-1404-3`), and agrees with the verdict
    /// (`INV-1404-1`) — a baseline is rewritten exactly when the metric did not
    /// regress.
    #[kani::proof]
    fn verify_next_baseline_monotone_idempotent() {
        let baseline: i64 = kani::any();
        let observed: i64 = kani::any();

        let next = next_baseline(baseline, observed);

        // INV-1404-2: never rises.
        assert!(next <= baseline);
        // INV-1404-3: idempotent.
        assert!(next_baseline(next, observed) == next);
        // INV-1404-1: Fail iff observed exceeds the baseline, and a Fail leaves
        // the baseline untouched.
        let v = ratchet_verdict(baseline, observed);
        assert!((v == RatchetVerdict::Fail) == (observed > baseline));
        if v == RatchetVerdict::Fail {
            assert!(next == baseline);
        } else {
            assert!(next == observed);
        }
    }
}
