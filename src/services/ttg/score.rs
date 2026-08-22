//! TTG — the score.
//!
//! This module turns the base measures of [`TokenMeasures`] into a number and
//! a letter. It is independent of the tokenizer: it reads `T` and `D` and
//! nothing else, so it can be reasoned about, and falsified, on its own.
//!
//! # Why this exists
//!
//! The incumbent scorer, `calculate_simple_tdg`, removes
//! `COMPLEXITY_PENALTY_PER_BRANCH = 1.5` points per branch. Its own comment
//! says why: "so the pre-commit CC<=30 gate lands at 56.5 (C-)". That is a
//! constant reverse-engineered from an unrelated gate's threshold — the grade
//! was fitted to the gate rather than the gate set from the grade, and nobody
//! could say what 1.5 meant.
//!
//! Every constant below carries its source in a doc comment, and there are
//! only two curves' worth of them. Both curves have **zero free parameters**:
//! the anchors are published thresholds or landmarks the incumbent already
//! declares, and every slope is derived from them. The test
//! `piecewise_slopes_are_derived_from_the_anchors` asserts that derivation
//! rather than restating it, so the anchors stay the single source of truth.
//!
//! # The model
//!
//! ```text
//!   CL      = T / TOKENS_PER_LINE                     a comparable-lines unit
//!   S_size  = PW(CL, SIZE_ANCHORS)
//!
//!   kind != Function:   score = S_size
//!   kind == Function:   S_cx  = PW(D, CX_ANCHORS)
//!                       score = 100 - min(100, ((100-S_cx)^4 + (100-S_size)^4)^(1/4))
//!
//!   score = clamp(score, 0, 100)
//!   grade = Grade::from_score(score)                  GRADE_BANDS UNCHANGED
//! ```
//!
//! Specification: §2.2 (the score), §2.3 (every constant), §2.4 (the bands),
//! §2.5 (`p = 4`, disclosed), §2.7 (non-`Function` kinds).
//!
//! # Example
//!
//! ```
//! use pmat::services::ttg::{measure, score::{ttg_score, DefKind}};
//! use pmat::tdg::Grade;
//!
//! let m = measure("fn f(a: bool, b: bool) -> u8 { if a { 1 } else if b { 2 } else { 0 } }");
//! let r = ttg_score(m, DefKind::Function);
//! assert_eq!(r.grade, Grade::APlus);
//! ```

use super::TokenMeasures;
use crate::tdg::Grade;

/// Tokens per comparable line — the unit conversion, and the model's only
/// empirical scale factor.
///
/// Measured 6.5545 aggregate over the 23,451 indexed definitions, flat to
/// within ±4% across every complexity band, then **rounded strict** (down, so
/// the same token count buys fewer lines of allowance). It converts `T` into a
/// unit the published size thresholds below are stated in; it is not itself a
/// threshold and nothing keys on it.
pub const TOKENS_PER_LINE: f64 = 6.5;

/// The size curve, in comparable lines.
///
/// ```text
///   CL =  0  ->  100    a definition with no tokens costs a reader nothing
///   CL = 30  ->   90    SIG low-risk ceiling      -> the `A` floor
///   CL = 44  ->   70    SIG moderate-risk ceiling -> the `B-` floor
///   CL = 74  ->   50    SIG high-risk ceiling     -> the `D` floor
/// ```
///
/// Source: Alves, Ypma & Visser, *Deriving Metric Thresholds from Benchmark
/// Data*, ICSM 2010 — the SIG unit-size risk ceilings. They were derived once,
/// from a benchmark of **other** systems, and are frozen here. Deriving them
/// from this repository's own distribution would make the grade a description
/// of what this repository already does, which is what a grade must not be.
///
/// The three interior values map to the three interior `GRADE_BANDS` floors
/// (§2.3): `A` is grade index 1, `B-` index 5, `D` index 9 — spacing 4 and 4,
/// with `A+` and `F` bracketing symmetrically. A four-band risk classification
/// needs exactly four regions of the eleven-grade ladder, and there is one
/// such partition.
pub const SIZE_ANCHORS: [(f64, f64); 4] = [(0.0, 100.0), (30.0, 90.0), (44.0, 70.0), (74.0, 50.0)];

/// The complexity curve, in decision points.
///
/// ```text
///   D =  0  ->  100    straight-line code
///   D =  6  ->   90    the incumbent's own A-line   -> the `A` floor
///   D = 13  ->   70    SEI/SATC moderate, scaled    -> the `B-` floor
///   D = 34  ->   50    the incumbent's own budget   -> the `D` floor
/// ```
///
/// **`D = 6` is the incumbent's A-line, retained decision for decision.** At
/// `COMPLEXITY_PENALTY_PER_BRANCH = 1.5`, `cc = 7` scores 91 (`A`) and
/// `cc = 8` scores 89.5 (`A-`); `cc <= 7` is exactly `D <= 6`. `D = 3` falls
/// out as a consequence — `PW(3) = 95.0` exactly, and the incumbent's A+ line
/// is `cc <= 4`, i.e. `D <= 3`. **Identical, decision for decision.**
///
/// **`D = 34` is the incumbent's own stated exhaustion point.** At 1.5 per
/// branch, `COMPLEXITY_PENALTY_CAP = 50` exhausts at `cc = 35`, i.e. `D = 34`,
/// as that constant's doc comment says.
///
/// `D = 13` is the SEI/SATC moderate-risk ceiling `v(G) = 20`, scaled by the
/// same `7/10` that separates the incumbent's retained A-line (`cc = 7`) from
/// McCabe's published 10.
///
/// # Do not "fix" this to 10
///
/// McCabe's published 10 and NIST SP 500-235's relaxation to 15 are both
/// **looser** than the `cc = 7` this repository already enforces. Adopting
/// either would be a relaxation of a shipped gate dressed up as a citation, so
/// both were **declined**. TTG is deliberately stricter than the literature on
/// the complexity axis, and the reason it is stricter is that the incumbent
/// already was.
pub const CX_ANCHORS: [(f64, f64); 4] = [(0.0, 100.0), (6.0, 90.0), (13.0, 70.0), (34.0, 50.0)];

/// The exponent of the norm that combines the two axes.
///
/// **The model's one non-derived constant, disclosed as such.** Its meaning:
/// *two risks at their published ceilings is worse than one risk at its
/// ceiling.* A definition at exactly `(D = 6, CL = 30)` — on both published
/// low-risk lines at once — scores 88.11 (`A-`), where `p = infinity` (a plain
/// `max`) would score exactly 90.00 (`A`). TTG is therefore stricter than the
/// conjunction of the two published standards, and the test
/// `two_risks_at_their_ceilings_is_worse_than_one` pins that.
///
/// It was chosen by measurement, not taste. Sweeping `p` over the whole index:
///
/// ```text
///   p       CB-200 failures   free tokens on the non-binding axis
///   max     3002              45,510  (1.95% of sum T)
///   8       3212                   0
///   6       3236                   0
///   4       3333                   0
///   3       3477                   0
///   2       3826               1,600
///   1       5173                   -
/// ```
///
/// `p = 4` is the least aggressive whole-number norm on the plateau where the
/// free slack on the non-binding axis is exactly zero. Under `max`, an attacker
/// can add 45,510 tokens repo-wide and change no score at all.
///
/// Disclosed residual: a `p`-norm attenuates the smaller term, so once a
/// definition is far into failure on one axis the other is nearly free (at
/// `CL = 90`, one more decision costs 0.0000 points). In the *passing* region,
/// where gaming would pay, a decision still costs 0.31–2.86 points. The
/// exposure is confined to definitions that already fail, where there is
/// nothing left to win.
pub const AGGREGATION_P: f64 = 4.0;

/// What kind of definition is being scored.
///
/// Only [`DefKind::Function`] has a control-flow graph, and therefore only
/// [`DefKind::Function`] is scored on the complexity axis (§2.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DefKind {
    /// A function, method, or closure — has a control-flow graph.
    Function,
    /// A `struct` declaration.
    Struct,
    /// An `enum` declaration.
    Enum,
    /// A `trait` declaration.
    Trait,
    /// A `type` alias.
    TypeAlias,
    /// A C/C++ forward declaration: a prototype with no body.
    Declaration,
}

impl DefKind {
    /// Whether this kind has a control-flow graph, and so a defined
    /// cyclomatic complexity.
    ///
    /// Cyclomatic complexity is defined over a control-flow graph. A `struct`
    /// has none. The incumbent did not merely tolerate that — it laundered it,
    /// forcing `effective_loc = 0` for `Enum | Struct | Trait | TypeAlias` so
    /// that 4,447 of 4,451 declarations scored the literal constant 100.0. A
    /// fifth of the graded population was a hard-coded number. TTG reports
    /// [`TtgRecord::s_cx`] as `None` — *not applicable*, never as a passing
    /// score — and charges these kinds for the one thing they do cost a
    /// reader: their size.
    #[must_use]
    pub fn has_control_flow(self) -> bool {
        matches!(self, DefKind::Function)
    }
}

/// Which axis is costing a definition its points.
///
/// Stored per definition because without a per-axis record a grade change is
/// unexplainable — the failure mode `contracts/tdg-grade-order-v1.yaml`
/// already documents.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Axis {
    /// Both sub-scores are 100.0: neither axis is charging anything.
    Neither,
    /// The size sub-score is the lower of the two.
    Size,
    /// The complexity sub-score is the lower of the two.
    Complexity,
    /// The two sub-scores are equal and below 100.0. Under `p = 4` both are
    /// charged for, so naming one of them would name an arbitrary winner.
    Both,
}

/// One definition's score, with the per-axis record that explains it.
///
/// Note what is *not* here. `truncated` (§2.8) is not derivable from
/// [`TokenMeasures`], and a field this function could only ever set to `false`
/// would read as "verified untruncated" — the exact defect §2.8 forbids, where
/// an unmeasured definition is indistinguishable from a clean one. The caller
/// that owns the chunk owns that flag. `satd_count` is likewise absent: it has
/// fired zero times across 23,451 rows against a free allowance of 2, so it is
/// stored and reported as an integer elsewhere and does not score (§2.6).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TtgRecord {
    /// The score, clamped to `[0, 100]`.
    ///
    /// `f32` to match [`Grade::from_score`] and the rest of `tdg`; the
    /// arithmetic that produces it is `f64` throughout, because `f32` rounding
    /// of a `^4` aggregation is large enough to move a definition across a
    /// band floor.
    pub score: f32,
    /// The letter, from the **unchanged** `GRADE_BANDS`.
    pub grade: Grade,
    /// `D` — decision points, as measured.
    pub d: u32,
    /// `T` — tokens, as measured.
    pub t: u32,
    /// `CL` — comparable lines, `T / TOKENS_PER_LINE`.
    pub cl: f64,
    /// `S_cx`, or `None` for a kind with no control-flow graph. `None` means
    /// *not applicable*; it must never be rendered as a passing number.
    pub s_cx: Option<f64>,
    /// `S_size`.
    pub s_size: f64,
    /// `N` — max control-flow nesting depth. **Carried, never scored.**
    ///
    /// Ablation (§2.6): 143 gate-scope rows have `N >= 5` and 133 of them
    /// already fail on the two axes, so a nesting gateway would change 10 of
    /// 22,724 outcomes — 0.04%. Shipping that as a component would repeat the
    /// dead-SATD defect in a new coat. It is reported because it explains
    /// *why* a unit is hard to read.
    pub n: u32,
    /// Which axis is binding.
    pub binding: Axis,
}

/// Clamp a score to the `[0, 100]` the bands are defined over.
fn clamp_score(v: f64) -> f64 {
    v.clamp(0.0, 100.0)
}

/// Piecewise-linear interpolation through `anchors`, **extrapolating the final
/// segment's slope** past the last anchor, clamped to `[0, 100]`.
///
/// Extrapolation rather than a floor at the last anchor's value is what keeps
/// the curve falling past the published ceilings. A floor would make every
/// definition beyond `CL = 74` score the same 50, so the worst definitions in
/// the repository would be indistinguishable from the merely bad ones and
/// growth past the ceiling would be free. Extrapolating introduces no new
/// constant: the slope is the one the last two anchors already imply.
///
/// Below the first anchor the first anchor's value is returned, so the curve
/// never rises above 100.
#[must_use]
pub fn piecewise(x: f64, anchors: &[(f64, f64)]) -> f64 {
    let Some(&(x_first, y_first)) = anchors.first() else {
        return 100.0;
    };
    if x <= x_first {
        return clamp_score(y_first);
    }
    for w in anchors.windows(2) {
        let ((x0, y0), (x1, y1)) = (w[0], w[1]);
        if x <= x1 {
            return clamp_score(y0 + (y1 - y0) * (x - x0) / (x1 - x0));
        }
    }
    match anchors.windows(2).next_back() {
        Some(w) => {
            let ((x0, y0), (x1, y1)) = (w[0], w[1]);
            clamp_score(y1 + (y1 - y0) / (x1 - x0) * (x - x1))
        }
        // Fewer than two anchors: there is no slope to extrapolate.
        None => clamp_score(y_first),
    }
}

/// `CL` — tokens expressed in comparable lines.
#[must_use]
pub fn comparable_lines(tokens: u32) -> f64 {
    f64::from(tokens) / TOKENS_PER_LINE
}

/// `S_size` — the size sub-score for a token count.
#[must_use]
pub fn size_score(tokens: u32) -> f64 {
    piecewise(comparable_lines(tokens), &SIZE_ANCHORS)
}

/// `S_cx` — the complexity sub-score for a decision count.
#[must_use]
pub fn complexity_score(decisions: u32) -> f64 {
    piecewise(f64::from(decisions), &CX_ANCHORS)
}

/// The `p`-norm of the two axes' penalties, `((100-S_cx)^p + (100-S_size)^p)^(1/p)`.
///
/// It is bounded below by `max` (the `p = infinity` case, which would let one
/// axis be stuffed for free) and above by the sum (the `p = 1` case, which
/// double-charges a definition that is merely average on both).
fn aggregate_penalty(s_cx: f64, s_size: f64) -> f64 {
    let cx = 100.0 - s_cx;
    let size = 100.0 - s_size;
    (cx.powf(AGGREGATION_P) + size.powf(AGGREGATION_P)).powf(1.0 / AGGREGATION_P)
}

/// Which of the two sub-scores is charging more.
fn binding_axis(s_cx: Option<f64>, s_size: f64) -> Axis {
    let Some(s_cx) = s_cx else {
        // No complexity axis: size is the only thing that can charge.
        return if s_size < 100.0 {
            Axis::Size
        } else {
            Axis::Neither
        };
    };
    if s_cx >= 100.0 && s_size >= 100.0 {
        Axis::Neither
    } else if s_cx < s_size {
        Axis::Complexity
    } else if s_size < s_cx {
        Axis::Size
    } else {
        Axis::Both
    }
}

/// Score one definition from its base measures.
///
/// A `Function` is scored on both axes, combined by the `p = 4` norm of
/// [`AGGREGATION_P`]. Every other kind is scored on the size axis alone and
/// reports `s_cx: None` (§2.7). The result is clamped to `[0, 100]` and graded
/// through the unchanged `GRADE_BANDS`.
///
/// # Example
///
/// ```
/// use pmat::services::ttg::TokenMeasures;
/// use pmat::services::ttg::score::{ttg_score, Axis, DefKind};
/// use pmat::tdg::Grade;
///
/// // 195 tokens is exactly CL = 30, the SIG low-risk ceiling: the A floor.
/// let m = TokenMeasures { tokens: 195, decisions: 0, max_nesting: 0 };
/// assert_eq!(ttg_score(m, DefKind::Function).grade, Grade::A);
///
/// // A declaration is never charged for a control-flow graph it does not have.
/// let m = TokenMeasures { tokens: 195, decisions: 99, max_nesting: 0 };
/// let r = ttg_score(m, DefKind::Struct);
/// assert_eq!(r.s_cx, None);
/// assert_eq!(r.binding, Axis::Size);
/// assert_eq!(r.grade, Grade::A);
/// ```
#[must_use]
pub fn ttg_score(measures: TokenMeasures, kind: DefKind) -> TtgRecord {
    let TokenMeasures {
        tokens,
        decisions,
        max_nesting,
    } = measures;

    let s_size = size_score(tokens);
    let s_cx = kind.has_control_flow().then(|| complexity_score(decisions));

    let penalty = match s_cx {
        Some(s_cx) => aggregate_penalty(s_cx, s_size),
        None => 100.0 - s_size,
    };
    let score = clamp_score(100.0 - penalty.min(100.0));

    TtgRecord {
        score: score as f32,
        grade: Grade::from_score(score as f32),
        d: decisions,
        t: tokens,
        cl: comparable_lines(tokens),
        s_cx,
        s_size,
        n: max_nesting,
        binding: binding_axis(s_cx, s_size),
    }
}

#[cfg(test)]
mod tests {
    use super::DefKind as K;
    use super::*;

    /// Score a `(T, D, kind)` triple, the way the parity grid states them.
    fn sc(tokens: u32, decisions: u32, kind: DefKind) -> TtgRecord {
        ttg_score(
            TokenMeasures {
                tokens,
                decisions,
                max_nesting: 0,
            },
            kind,
        )
    }

    /// The slopes each pair of consecutive anchors implies.
    fn slopes(anchors: &[(f64, f64)]) -> Vec<f64> {
        anchors
            .windows(2)
            .map(|w| (w[1].1 - w[0].1) / (w[1].0 - w[0].0))
            .collect()
    }

    fn close(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() <= eps
    }

    // ---------------------------------------------------------------------
    // The curves
    // ---------------------------------------------------------------------

    /// §2.2 states six slopes. They are **derived**, not chosen: this asserts
    /// the derivation instead of restating the numbers in the source, so the
    /// anchors remain the single place either curve can be changed.
    ///
    /// RED under: `SIZE_ANCHORS[2] = (44.0, 72.0)`.
    #[test]
    fn piecewise_slopes_are_derived_from_the_anchors() {
        let size = slopes(&SIZE_ANCHORS);
        assert_eq!(size.len(), 3);
        assert!(
            close(size[0], -1.0 / 3.0, 1e-12),
            "size 0..30: {:?}",
            size[0]
        );
        assert!(
            close(size[1], -10.0 / 7.0, 1e-12),
            "size 30..44: {:?}",
            size[1]
        );
        assert!(
            close(size[2], -2.0 / 3.0, 1e-12),
            "size 44..74: {:?}",
            size[2]
        );

        let cx = slopes(&CX_ANCHORS);
        assert_eq!(cx.len(), 3);
        assert!(close(cx[0], -5.0 / 3.0, 1e-12), "cx 0..6: {:?}", cx[0]);
        assert!(close(cx[1], -20.0 / 7.0, 1e-12), "cx 6..13: {:?}", cx[1]);
        assert!(close(cx[2], -20.0 / 21.0, 1e-12), "cx 13..34: {:?}", cx[2]);

        // The published decimal forms in §2.2, to the digits it prints.
        assert!(close(size[1], -1.428_571, 1e-6));
        assert!(close(cx[1], -2.857_143, 1e-6));
        assert!(close(cx[2], -0.952_381, 1e-6));
    }

    /// Every anchor is hit exactly, so the published thresholds are the
    /// thresholds and not approximations of them.
    #[test]
    fn every_anchor_is_hit_exactly() {
        for &(x, y) in &SIZE_ANCHORS {
            assert_eq!(piecewise(x, &SIZE_ANCHORS), y, "size anchor {x}");
        }
        for &(x, y) in &CX_ANCHORS {
            assert_eq!(piecewise(x, &CX_ANCHORS), y, "cx anchor {x}");
        }
    }

    /// **RED-first.** A curve that stopped at its last anchor would make every
    /// definition past `CL = 74` score the same 50: growth beyond the
    /// high-risk ceiling would be free, and the worst code in the repository
    /// would be indistinguishable from the merely bad.
    ///
    /// RED under: `piecewise` returning `y1` instead of extrapolating (i.e.
    /// replacing the final `clamp_score(y1 + slope * (x - x1))` with
    /// `clamp_score(y1)`).
    #[test]
    fn piecewise_keeps_falling_past_the_last_anchor() {
        let (last_x, last_y) = SIZE_ANCHORS[SIZE_ANCHORS.len() - 1];
        assert_eq!(piecewise(last_x, &SIZE_ANCHORS), last_y);

        let mut prev = last_y;
        for step in 1..=70 {
            let v = piecewise(last_x + f64::from(step), &SIZE_ANCHORS);
            assert!(
                v < prev,
                "size stalled at CL = {}",
                last_x + f64::from(step)
            );
            prev = v;
        }
        // The extrapolated slope is the final segment's, not a new constant.
        assert!(close(
            piecewise(84.0, &SIZE_ANCHORS),
            50.0 - 10.0 * 2.0 / 3.0,
            1e-12
        ));

        let (last_x, last_y) = CX_ANCHORS[CX_ANCHORS.len() - 1];
        let mut prev = last_y;
        for step in 1..=20 {
            let v = piecewise(last_x + f64::from(step), &CX_ANCHORS);
            assert!(v < prev, "cx stalled at D = {}", last_x + f64::from(step));
            prev = v;
        }
    }

    /// Counter-test bounding the over-correction above: extrapolation must not
    /// run off the bottom of the scale, and the curve must never exceed 100
    /// below the first anchor.
    #[test]
    fn extrapolation_stays_inside_the_band_range() {
        for anchors in [&SIZE_ANCHORS, &CX_ANCHORS] {
            for x in [-1e9, -1.0, 0.0, 1.0, 500.0, 1e6, 1e9] {
                let v = piecewise(x, anchors);
                assert!((0.0..=100.0).contains(&v), "piecewise({x}) = {v}");
            }
            assert_eq!(piecewise(-1.0, anchors), 100.0);
            assert_eq!(piecewise(1e9, anchors), 0.0);
        }
    }

    // ---------------------------------------------------------------------
    // The aggregation
    // ---------------------------------------------------------------------

    /// **RED-first.** `p = 4` is not a `max`. A definition bad on *both* axes
    /// must score strictly worse than one equally bad on either axis alone —
    /// otherwise the non-binding axis is free and can be stuffed (§2.5: 45,510
    /// free tokens repo-wide under `max`).
    ///
    /// RED under: `aggregate_penalty` returning `cx.max(size)`.
    #[test]
    fn both_axes_bad_scores_worse_than_either_alone() {
        for &(d, t) in &[(6u32, 195u32), (13, 286), (7, 218), (34, 481), (3, 98)] {
            let both = sc(t, d, K::Function).score;
            let cx_only = sc(0, d, K::Function).score;
            let size_only = sc(t, 0, K::Function).score;
            assert!(
                both < cx_only && both < size_only,
                "D={d} T={t}: both={both} cx_only={cx_only} size_only={size_only}"
            );
        }
    }

    /// Counter-test bounding that over-correction from the other side. When
    /// both axes are charging, the `p = 4` penalty must sit **strictly
    /// between** `max` (`p = infinity`, the free-axis hole the test above
    /// forbids) and the sum (`p = 1`, which double-charges a definition that
    /// is merely average on both and cost 5,173 rather than 3,333 failures in
    /// §2.5's sweep). When one axis is clean the two bounds coincide and the
    /// penalty must equal both.
    ///
    /// RED under: `aggregate_penalty` returning `cx.max(size)` **or**
    /// `cx + size`.
    #[test]
    fn the_p_norm_sits_strictly_between_max_and_sum() {
        let mut both_charging = 0;
        for d in [0u32, 1, 3, 6, 7, 13, 20, 34, 40] {
            for t in [0u32, 65, 195, 286, 400, 481, 700] {
                let (s_cx, s_size) = (complexity_score(d), size_score(t));
                let (cx, size) = (100.0 - s_cx, 100.0 - s_size);
                let pen = aggregate_penalty(s_cx, s_size);
                if cx > 0.0 && size > 0.0 {
                    both_charging += 1;
                    assert!(pen > cx.max(size) + 1e-9, "D={d} T={t}: {pen} <= max");
                    assert!(pen < cx + size - 1e-9, "D={d} T={t}: {pen} >= sum");
                } else {
                    assert!(close(pen, cx.max(size), 1e-9), "D={d} T={t}: {pen}");
                    assert!(close(pen, cx + size, 1e-9), "D={d} T={t}: {pen}");
                }
            }
        }
        assert!(
            both_charging >= 40,
            "the strict branch barely ran: {both_charging}"
        );
    }

    /// §2.5's disclosed meaning of `p`, as a number: a definition sitting on
    /// **both** published low-risk ceilings at once is `A-`, where a plain
    /// `max` would call it exactly `A`. TTG is stricter than the conjunction
    /// of the two standards, deliberately.
    #[test]
    fn two_risks_at_their_ceilings_is_worse_than_one() {
        // D = 6 and CL = 30 (T = 195) are the two A-floor anchors.
        assert_eq!(complexity_score(6), 90.0);
        assert_eq!(size_score(195), 90.0);

        let r = sc(195, 6, K::Function);
        // 1e-4 is the parity tolerance: `score` is `f32`, whose ulp near 88
        // is 7.6e-6, so a tighter bound would test the storage type, not the
        // model. `parity_with_the_python_reference_model` uses the same bound.
        assert!(
            close(f64::from(r.score), 88.107_928_85, 1e-4),
            "{}",
            r.score
        );
        assert_eq!(r.grade, Grade::AMinus);
        assert_eq!(r.binding, Axis::Both);

        // Either ceiling alone is exactly the A floor.
        assert_eq!(sc(195, 0, K::Function).grade, Grade::A);
        assert_eq!(sc(0, 6, K::Function).grade, Grade::A);
    }

    // ---------------------------------------------------------------------
    // Non-Function kinds
    // ---------------------------------------------------------------------

    /// **RED-first.** A `struct` has no control-flow graph, so it is scored on
    /// size alone and reports `NOT_APPLICABLE` on complexity — never a passing
    /// number, which is how the incumbent came to hard-code 100.0 for 4,447 of
    /// 4,451 declarations.
    ///
    /// RED under: `DefKind::has_control_flow` returning `true` for every kind.
    #[test]
    fn non_function_kinds_ignore_the_complexity_axis() {
        for kind in [K::Struct, K::Enum, K::Trait, K::TypeAlias, K::Declaration] {
            assert!(!kind.has_control_flow(), "{kind:?}");
            let base = sc(286, 0, kind);
            assert_eq!(base.s_cx, None, "{kind:?} reported a complexity score");
            for d in [0u32, 1, 7, 34, 1_000, u32::MAX] {
                let r = sc(286, d, kind);
                assert_eq!(r.score, base.score, "{kind:?} moved on D = {d}");
                assert_eq!(r.grade, base.grade, "{kind:?} moved on D = {d}");
                assert_eq!(r.binding, Axis::Size, "{kind:?} on D = {d}");
            }
        }
        assert!(K::Function.has_control_flow());
    }

    /// Counter-test bounding that over-correction: ignoring the complexity
    /// axis must not become *exempting the kind*. The incumbent's exemption is
    /// exactly what hid `AnalyzeCommands` (1,891 lines) at `A-`. A declaration
    /// is charged for its size on the identical curve a function is, so at
    /// `D = 0` the two agree to the bit.
    #[test]
    fn declarations_are_charged_for_size_on_the_same_curve() {
        for t in [0u32, 65, 195, 196, 286, 481, 700, 5_000] {
            let f = sc(t, 0, K::Function);
            let s = sc(t, 0, K::Struct);
            assert_eq!(s.score, f.score, "T = {t}");
            assert_eq!(s.s_size, f.s_size, "T = {t}");
        }
        // The two CLI enums the exemption was hiding: CL = 305.8 and 297.2.
        assert_eq!(sc(1_988, 0, K::Enum).grade, Grade::F);
        assert_eq!(sc(1_932, 0, K::Enum).grade, Grade::F);
        // A mean-sized struct is small, and small things are small.
        assert_eq!(sc(57, 0, K::Struct).grade, Grade::APlus);
    }

    // ---------------------------------------------------------------------
    // Parity with the Python reference
    // ---------------------------------------------------------------------

    /// Every row is `scratchpad/spec/model.py`'s own output for that
    /// `(T, D, kind)`, printed to ten decimal places. Scores must agree to
    /// within 1e-4 and grades must agree exactly.
    ///
    /// The grid is chosen to straddle every structural feature of the model:
    /// each anchor's exact token count (195 = CL 30, 286 = CL 44, 481 = CL 74)
    /// and the token immediately past two of them (196, 482), each complexity
    /// anchor (0, 3, 6, 13, 34) and the decision past the budget (35, 100),
    /// and the extrapolated tail through the point where `S_size` reaches zero
    /// (968 -> 0.05, 1000 -> 0.00).
    ///
    /// RED under: `TOKENS_PER_LINE = 6.0`, or any anchor moved.
    #[test]
    fn parity_with_the_python_reference_model() {
        #[rustfmt::skip]
        const GRID: &[(u32, u32, DefKind, f64, &str)] = &[
            (0, 0, K::Function, 100.0000000000, "A+"),
            (0, 3, K::Function, 95.0000000000, "A+"),
            (0, 6, K::Function, 90.0000000000, "A"),
            (0, 7, K::Function, 87.1428571429, "A-"),
            (0, 13, K::Function, 70.0000000000, "B-"),
            (0, 34, K::Function, 50.0000000000, "D"),
            (0, 35, K::Function, 49.0476190476, "F"),
            (0, 100, K::Function, 0.0000000000, "F"),
            (65, 0, K::Function, 96.6666666667, "A+"),
            (65, 3, K::Function, 94.7695183455, "A"),
            (65, 6, K::Function, 89.9692776719, "A-"),
            (65, 7, K::Function, 87.1283598435, "A-"),
            (65, 13, K::Function, 69.9988569469, "C+"),
            (65, 34, K::Function, 49.9997530882, "F"),
            (65, 35, K::Function, 49.0473857241, "F"),
            (65, 100, K::Function, 0.0000000000, "F"),
            (195, 0, K::Function, 90.0000000000, "A"),
            (195, 3, K::Function, 89.8472840757, "A-"),
            (195, 6, K::Function, 88.1079288500, "A-"),
            (195, 7, K::Function, 86.1003714359, "A-"),
            (195, 13, K::Function, 69.9078330157, "C+"),
            (195, 34, K::Function, 49.9800119888, "F"),
            (195, 35, K::Function, 49.0287302172, "F"),
            (195, 100, K::Function, 0.0000000000, "F"),
            (196, 0, K::Function, 89.7802197802, "A-"),
            (196, 3, K::Function, 89.6368791883, "A-"),
            (196, 6, K::Function, 87.9751165872, "A-"),
            (196, 7, K::Function, 86.0165522924, "A-"),
            (196, 13, K::Function, 69.8995012228, "C+"),
            (196, 34, K::Function, 49.9781972053, "F"),
            (196, 35, K::Function, 49.0270151508, "F"),
            (196, 100, K::Function, 0.0000000000, "F"),
            (286, 0, K::Function, 70.0000000000, "B-"),
            (286, 3, K::Function, 69.9942146367, "C+"),
            (286, 6, K::Function, 69.9078330157, "C+"),
            (286, 7, K::Function, 69.7501198027, "C+"),
            (286, 13, K::Function, 64.3237865499, "C"),
            (286, 34, K::Function, 48.4532634290, "F"),
            (286, 35, K::Function, 47.5812935650, "F"),
            (286, 100, K::Function, 0.0000000000, "F"),
            (481, 0, K::Function, 50.0000000000, "D"),
            (481, 3, K::Function, 49.9987500469, "F"),
            (481, 6, K::Function, 49.9800119888, "F"),
            (481, 7, K::Function, 49.9454371499, "F"),
            (481, 13, K::Function, 48.4532634290, "F"),
            (481, 34, K::Function, 40.5396442499, "F"),
            (481, 35, K::Function, 39.9653431078, "F"),
            (481, 100, K::Function, 0.0000000000, "F"),
            (482, 0, K::Function, 49.8974358974, "F"),
            (482, 3, K::Function, 49.8961936045, "F"),
            (482, 6, K::Function, 49.8775702893, "F"),
            (482, 7, K::Function, 49.8432067226, "F"),
            (482, 13, K::Function, 48.3596248705, "F"),
            (482, 34, K::Function, 40.4785655429, "F"),
            (482, 35, K::Function, 39.9059971089, "F"),
            (482, 100, K::Function, 0.0000000000, "F"),
            (700, 0, K::Function, 27.5384615385, "F"),
            (700, 3, K::Function, 27.5380508678, "F"),
            (700, 6, K::Function, 27.5318916455, "F"),
            (700, 7, K::Function, 27.5205128022, "F"),
            (700, 13, K::Function, 27.0119932441, "F"),
            (700, 34, K::Function, 23.7408319465, "F"),
            (700, 35, K::Function, 23.4661141389, "F"),
            (700, 100, K::Function, 0.0000000000, "F"),
            (968, 0, K::Function, 0.0512820513, "F"),
            (968, 3, K::Function, 0.0511255610, "F"),
            (968, 6, K::Function, 0.0487782953, "F"),
            (968, 7, K::Function, 0.0444407044, "F"),
            (968, 13, K::Function, 0.0000000000, "F"),
            (968, 34, K::Function, 0.0000000000, "F"),
            (968, 35, K::Function, 0.0000000000, "F"),
            (968, 100, K::Function, 0.0000000000, "F"),
            (1000, 0, K::Function, 0.0000000000, "F"),
            (1000, 3, K::Function, 0.0000000000, "F"),
            (1000, 6, K::Function, 0.0000000000, "F"),
            (1000, 7, K::Function, 0.0000000000, "F"),
            (1000, 13, K::Function, 0.0000000000, "F"),
            (1000, 34, K::Function, 0.0000000000, "F"),
            (1000, 35, K::Function, 0.0000000000, "F"),
            (1000, 100, K::Function, 0.0000000000, "F"),
            (5000, 0, K::Function, 0.0000000000, "F"),
            (5000, 3, K::Function, 0.0000000000, "F"),
            (5000, 6, K::Function, 0.0000000000, "F"),
            (5000, 7, K::Function, 0.0000000000, "F"),
            (5000, 13, K::Function, 0.0000000000, "F"),
            (5000, 34, K::Function, 0.0000000000, "F"),
            (5000, 35, K::Function, 0.0000000000, "F"),
            (5000, 100, K::Function, 0.0000000000, "F"),
            (0, 0, K::Struct, 100.0000000000, "A+"),
            (0, 6, K::Struct, 100.0000000000, "A+"),
            (0, 100, K::Struct, 100.0000000000, "A+"),
            (65, 0, K::Struct, 96.6666666667, "A+"),
            (65, 6, K::Struct, 96.6666666667, "A+"),
            (65, 100, K::Struct, 96.6666666667, "A+"),
            (195, 0, K::Struct, 90.0000000000, "A"),
            (195, 6, K::Struct, 90.0000000000, "A"),
            (195, 100, K::Struct, 90.0000000000, "A"),
            (196, 0, K::Struct, 89.7802197802, "A-"),
            (196, 6, K::Struct, 89.7802197802, "A-"),
            (196, 100, K::Struct, 89.7802197802, "A-"),
            (286, 0, K::Struct, 70.0000000000, "B-"),
            (286, 6, K::Struct, 70.0000000000, "B-"),
            (286, 100, K::Struct, 70.0000000000, "B-"),
            (481, 0, K::Struct, 50.0000000000, "D"),
            (481, 6, K::Struct, 50.0000000000, "D"),
            (481, 100, K::Struct, 50.0000000000, "D"),
            (482, 0, K::Struct, 49.8974358974, "F"),
            (482, 6, K::Struct, 49.8974358974, "F"),
            (482, 100, K::Struct, 49.8974358974, "F"),
            (700, 0, K::Struct, 27.5384615385, "F"),
            (700, 6, K::Struct, 27.5384615385, "F"),
            (700, 100, K::Struct, 27.5384615385, "F"),
            (968, 0, K::Struct, 0.0512820513, "F"),
            (968, 6, K::Struct, 0.0512820513, "F"),
            (968, 100, K::Struct, 0.0512820513, "F"),
            (1000, 0, K::Struct, 0.0000000000, "F"),
            (1000, 6, K::Struct, 0.0000000000, "F"),
            (1000, 100, K::Struct, 0.0000000000, "F"),
            (5000, 0, K::Struct, 0.0000000000, "F"),
            (5000, 6, K::Struct, 0.0000000000, "F"),
            (5000, 100, K::Struct, 0.0000000000, "F"),
        ];

        assert_eq!(GRID.len(), 121, "the grid must not shrink silently");
        for &(t, d, kind, want_score, want_grade) in GRID {
            let got = sc(t, d, kind);
            assert!(
                close(f64::from(got.score), want_score, 1e-4),
                "T={t} D={d} {kind:?}: score {} != python {want_score}",
                got.score
            );
            assert_eq!(
                got.grade.to_string(),
                want_grade,
                "T={t} D={d} {kind:?}: grade from score {}",
                got.score
            );
        }
    }

    // ---------------------------------------------------------------------
    // The bands
    // ---------------------------------------------------------------------

    /// §2.4's cut points, recomputed by inverting each axis with the other
    /// clean. These are what a letter now *means*, so a change to either curve
    /// that silently moves a grade boundary fails here.
    ///
    /// §2.4 also prints a `max T` column. It is **not** asserted, because it
    /// is not a maximum: it is `round(CL_max * 6.5)` at the real-valued
    /// boundary, so its `A+` entry of 98 tokens actually scores 94.97 (`A`).
    /// The `max CL` column below is the exact one.
    #[test]
    fn grade_cut_points_match_the_specification() {
        // (floor, max D, max CL) — §2.4, A+ through D.
        const CUTS: &[(&str, u32, u32)] = &[
            ("A+", 3, 15),
            ("A", 6, 30),
            ("A-", 7, 33),
            ("B+", 9, 37),
            ("B", 11, 40),
            ("B-", 13, 44),
            ("C+", 18, 51),
            ("C", 23, 59),
            ("C-", 28, 66),
            ("D", 34, 74),
        ];
        for &(grade, max_d, max_cl) in CUTS {
            let floor = Grade::from_variant_name(grade)
                .expect("§2.4 names only grades GRADE_VARIANTS lists")
                .score_band()
                .0;
            assert!(
                complexity_score(max_d) >= f64::from(floor),
                "{grade}: D = {max_d} should reach {floor}"
            );
            assert!(
                complexity_score(max_d + 1) < f64::from(floor),
                "{grade}: D = {} should not reach {floor}",
                max_d + 1
            );
            assert!(
                piecewise(f64::from(max_cl), &SIZE_ANCHORS) >= f64::from(floor),
                "{grade}: CL = {max_cl} should reach {floor}"
            );
            assert!(
                piecewise(f64::from(max_cl + 1), &SIZE_ANCHORS) < f64::from(floor),
                "{grade}: CL = {} should not reach {floor}",
                max_cl + 1
            );
        }
    }

    /// The complexity A-line and A+-line are the incumbent's own, retained
    /// decision for decision (§2.3). `cc <= 7` is `D <= 6`; `cc <= 4` is
    /// `D <= 3`. If someone later "fixes" `CX_ANCHORS` to McCabe's published
    /// 10, this is the test that says no.
    #[test]
    fn the_complexity_lines_are_the_incumbents_own() {
        assert_eq!(sc(0, 3, K::Function).grade, Grade::APlus); // cc = 4
        assert_eq!(sc(0, 4, K::Function).grade, Grade::A); // cc = 5
        assert_eq!(sc(0, 6, K::Function).grade, Grade::A); // cc = 7
        assert_eq!(sc(0, 7, K::Function).grade, Grade::AMinus); // cc = 8
                                                                // McCabe's 10 (D = 9) and NIST's 15 (D = 14) are both looser than this,
                                                                // and were declined. Both are already past the A line.
        assert_eq!(sc(0, 9, K::Function).grade, Grade::BPlus);
        assert_eq!(sc(0, 14, K::Function).grade, Grade::CPlus);
        // The incumbent's budget exhaustion, cc = 35, is the D floor.
        assert_eq!(sc(0, 34, K::Function).grade, Grade::D);
        assert_eq!(sc(0, 35, K::Function).grade, Grade::F);
    }

    // ---------------------------------------------------------------------
    // Shape properties
    // ---------------------------------------------------------------------

    /// Neither axis may ever pay. Adding a decision or a token can only ever
    /// leave the score alone or lower it — the property every ratchet built on
    /// this model depends on.
    #[test]
    fn the_score_is_monotone_non_increasing_on_both_axes() {
        for d in [0u32, 3, 6, 13, 34, 60] {
            let mut prev = f64::INFINITY;
            for t in (0u32..1_200).step_by(7) {
                let v = f64::from(sc(t, d, K::Function).score);
                assert!(v <= prev + 1e-6, "D = {d}: T = {t} scored up");
                prev = v;
            }
        }
        for t in [0u32, 65, 195, 286, 481, 900] {
            let mut prev = f64::INFINITY;
            for d in 0u32..120 {
                let v = f64::from(sc(t, d, K::Function).score);
                assert!(v <= prev + 1e-6, "T = {t}: D = {d} scored up");
                prev = v;
            }
        }
    }

    /// `N` is carried and never scored (§2.6). A nesting gateway would have
    /// changed 10 of 22,724 outcomes; shipping it as a component would repeat
    /// the dead-SATD defect. This is the vacuity guard that keeps it out.
    #[test]
    fn nesting_depth_is_carried_but_never_scored() {
        let base = ttg_score(
            TokenMeasures {
                tokens: 286,
                decisions: 7,
                max_nesting: 0,
            },
            K::Function,
        );
        for n in [1u32, 5, 9, 40] {
            let r = ttg_score(
                TokenMeasures {
                    tokens: 286,
                    decisions: 7,
                    max_nesting: n,
                },
                K::Function,
            );
            assert_eq!(r.n, n, "N was not carried");
            assert_eq!(r.score, base.score, "N = {n} moved the score");
            assert_eq!(r.grade, base.grade, "N = {n} moved the grade");
        }
    }

    /// `CL` is `T / 6.5` and nothing else, and it is reported so a reader can
    /// check the size axis by hand.
    #[test]
    fn comparable_lines_is_the_only_unit_conversion() {
        assert_eq!(TOKENS_PER_LINE, 6.5);
        for t in [0u32, 1, 65, 195, 286, 481, 1_000] {
            let r = sc(t, 0, K::Function);
            assert_eq!(r.cl, f64::from(t) / 6.5, "T = {t}");
            assert_eq!(r.t, t);
            assert_eq!(r.s_size, piecewise(r.cl, &SIZE_ANCHORS));
        }
        // The three anchors land on whole token counts, which is why the grid
        // above can assert exact band floors.
        assert_eq!(comparable_lines(195), 30.0);
        assert_eq!(comparable_lines(286), 44.0);
        assert_eq!(comparable_lines(481), 74.0);
    }

    /// The binding axis names the larger penalty, so a grade drop is
    /// explainable without re-deriving it.
    #[test]
    fn the_binding_axis_names_the_larger_penalty() {
        assert_eq!(sc(0, 0, K::Function).binding, Axis::Neither);
        assert_eq!(sc(0, 0, K::Struct).binding, Axis::Neither);
        assert_eq!(sc(500, 1, K::Function).binding, Axis::Size);
        assert_eq!(sc(10, 20, K::Function).binding, Axis::Complexity);
        assert_eq!(sc(195, 6, K::Function).binding, Axis::Both);
        assert_eq!(sc(500, 0, K::TypeAlias).binding, Axis::Size);

        // And it always agrees with the sub-scores it summarises.
        for d in [0u32, 2, 6, 13, 34] {
            for t in [0u32, 65, 195, 481] {
                let r = sc(t, d, K::Function);
                let s_cx = r.s_cx.unwrap_or(f64::NAN);
                match r.binding {
                    Axis::Complexity => assert!(s_cx < r.s_size, "T={t} D={d}"),
                    Axis::Size => assert!(r.s_size < s_cx, "T={t} D={d}"),
                    Axis::Both => assert!(s_cx == r.s_size && s_cx < 100.0, "T={t} D={d}"),
                    Axis::Neither => assert!(s_cx == 100.0 && r.s_size == 100.0, "T={t} D={d}"),
                }
            }
        }
    }

    /// The extremes are reachable and clamped: an empty definition is 100 and
    /// `F` is not merely theoretical.
    #[test]
    fn the_score_is_clamped_to_the_band_range() {
        let empty = sc(0, 0, K::Function);
        assert_eq!(empty.score, 100.0);
        assert_eq!(empty.grade, Grade::APlus);

        for kind in [K::Function, K::Struct] {
            let worst = sc(u32::MAX, u32::MAX, kind);
            assert_eq!(worst.score, 0.0, "{kind:?}");
            assert_eq!(worst.grade, Grade::F, "{kind:?}");
        }
        for t in (0u32..2_000).step_by(13) {
            for d in [0u32, 9, 44] {
                let s = sc(t, d, K::Function).score;
                assert!((0.0..=100.0).contains(&s), "T={t} D={d}: {s}");
            }
        }
    }
}
