//! Deterministic aggregation of category scores.
//!
//! # Why this module exists (#687)
//!
//! `pmat rust-project-score --format json` was not byte-reproducible on an
//! unchanged project. Five runs produced two distinct md5 sums; the *only*
//! textual difference between them was
//!
//! ```text
//! "percentage": 28.001373626373628   vs   28.001373626373624
//! ```
//!
//! (observed 9x `…628` / 6x `…624` over fifteen runs, while `total_earned`
//! stayed bit-identical at `63.785714285714285` and `total_possible` at
//! `279.0`).
//!
//! Root cause: category scores live in a [`HashMap`], whose iteration order is
//! randomised per process. The normalized percentage is the mean of the
//! per-category percentages, so the addends were folded in a *different order*
//! on every invocation — and floating point addition is not associative, so the
//! result wobbled by one ULP. That made an unchanged project diff on roughly
//! every other run, which defeats the whole point of checking the JSON into a
//! CI baseline.
//!
//! Everything in this module therefore folds in a **fixed, name-sorted order**
//! and rounds the emitted figures to [`SCORE_DECIMALS`] places, so identical
//! input produces byte-identical output. Every renderer (text, markdown, json,
//! yaml) must go through these helpers so the renderers cannot disagree about
//! the same number.

use super::models::CategoryScore;
use std::collections::HashMap;

/// Decimal places kept in emitted score figures.
///
/// Six places is far more precision than a 0-100 quality score can justify,
/// while being coarse enough to swallow the ULP-level wobble that #687 was
/// about (`28.001373626373628` and `28.001373626373624` both round to
/// `28.001374`).
pub const SCORE_DECIMALS: i32 = 6;

/// Round a score to [`SCORE_DECIMALS`] places.
///
/// Non-finite inputs are returned untouched — rounding NaN/inf would invent a
/// number that was never measured.
#[must_use]
pub fn round_score(value: f64) -> f64 {
    if !value.is_finite() {
        return value;
    }
    let factor = 10_f64.powi(SCORE_DECIMALS);
    (value * factor).round() / factor
}

/// Category `(name, score)` pairs in deterministic (name-sorted) order.
///
/// This is the single ordering used by *every* renderer, so the JSON category
/// array matches the text and markdown tables.
#[must_use]
pub fn sorted_categories(
    categories: &HashMap<String, CategoryScore>,
) -> Vec<(&String, &CategoryScore)> {
    let mut ordered: Vec<(&String, &CategoryScore)> = categories.iter().collect();
    ordered.sort_by(|a, b| a.0.cmp(b.0));
    ordered
}

/// The percentage at or above which a category is rendered as passing (the `✓`
/// tier in the text renderer).
pub const PASSING_PERCENTAGE: f64 = 90.0;

/// True when `--failures-only` should hide this category.
///
/// #943: the flag was a no-op, so this predicate did not exist and the four
/// renderers had nothing to share. It lives here, next to the fold every
/// renderer already goes through, so text, markdown, json and yaml cannot
/// disagree about which categories are failures.
///
/// A category that is **not applicable** is not hidden: N/A means "not
/// measured", which is not a pass, and hiding it would leave the reported
/// denominator unexplainable.
#[must_use]
pub fn hidden_by_failures_only(category: &CategoryScore) -> bool {
    category.applicable && category.percentage() >= PASSING_PERCENTAGE
}

/// Category pairs in name-sorted order, optionally dropping passing categories.
#[must_use]
pub fn sorted_categories_filtered(
    categories: &HashMap<String, CategoryScore>,
    failures_only: bool,
) -> Vec<(&String, &CategoryScore)> {
    sorted_categories(categories)
        .into_iter()
        .filter(|(_, cat)| !(failures_only && hidden_by_failures_only(cat)))
        .collect()
}

/// Deterministic sum over categories, folded in name-sorted order.
fn sorted_sum<F>(categories: &HashMap<String, CategoryScore>, mut value_of: F) -> f64
where
    F: FnMut(&CategoryScore) -> Option<f64>,
{
    let mut total = 0.0_f64;
    for (_, cat) in sorted_categories(categories) {
        if let Some(v) = value_of(cat) {
            total += v;
        }
    }
    round_score(total)
}

/// Points earned across every category (applicable or not), name-sorted fold.
#[must_use]
pub fn total_earned(categories: &HashMap<String, CategoryScore>) -> f64 {
    sorted_sum(categories, |cat| Some(cat.earned))
}

/// Points earned across **applicable** categories only (#237: N/A categories
/// must not pollute the totals).
#[must_use]
pub fn applicable_earned(categories: &HashMap<String, CategoryScore>) -> f64 {
    sorted_sum(categories, |cat| cat.applicable.then_some(cat.earned))
}

/// Maximum points across **applicable** categories only.
#[must_use]
pub fn applicable_possible(categories: &HashMap<String, CategoryScore>) -> f64 {
    sorted_sum(categories, |cat| cat.applicable.then_some(cat.max))
}

/// Normalized 0-100 score: the mean of the per-category percentages of the
/// applicable categories, folded in name-sorted order and rounded.
///
/// The name-sorted fold is the actual #687 fix; the rounding is belt and
/// braces so that a future change of aggregation strategy cannot silently
/// reintroduce a ULP-level diff.
#[must_use]
pub fn normalized_percentage(categories: &HashMap<String, CategoryScore>) -> f64 {
    let applicable: Vec<&CategoryScore> = sorted_categories(categories)
        .into_iter()
        .map(|(_, cat)| cat)
        .filter(|cat| cat.applicable)
        .collect();

    if applicable.is_empty() {
        return 0.0;
    }

    let mut sum_pcts = 0.0_f64;
    for cat in &applicable {
        // A zero-max category cannot lose points, so it counts as complete.
        sum_pcts += if cat.max > 0.0 {
            (cat.earned / cat.max) * 100.0
        } else {
            100.0
        };
    }

    round_score(sum_pcts / applicable.len() as f64)
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    /// Category set whose per-category percentages are non-terminating in
    /// binary, so their sum genuinely depends on the fold order. Modelled on
    /// the real pmat run that produced the #687 report (11 categories,
    /// normalized ≈ 28.0).
    fn wobbly_categories() -> HashMap<String, CategoryScore> {
        let raw: [(&str, f64, f64); 11] = [
            ("Build Performance", 4.0, 15.0),
            ("Code Quality", 7.0, 26.0),
            ("Dependency Health", 5.0, 12.0),
            ("Documentation", 11.0, 15.0),
            ("Formal Verification", 1.0, 16.0),
            ("GPU/SIMD Quality", 3.0, 10.0),
            ("Known Defects", 13.0, 20.0),
            ("Performance & Benchmarking", 7.0, 10.0),
            ("Reproducibility", 2.0, 15.0),
            ("Rust Tooling & CI/CD", 91.0, 130.0),
            ("Testing Excellence", 3.0, 20.0),
        ];
        raw.iter()
            .map(|(name, earned, max)| ((*name).to_string(), CategoryScore::new(*earned, *max)))
            .collect()
    }

    #[test]
    fn test_round_score_trims_ulp_wobble() {
        // The two values #687 actually observed must collapse to one.
        assert_eq!(
            round_score(28.001_373_626_373_628),
            round_score(28.001_373_626_373_624)
        );
    }

    #[test]
    fn test_round_score_passes_non_finite_through() {
        assert!(round_score(f64::NAN).is_nan());
        assert_eq!(round_score(f64::INFINITY), f64::INFINITY);
    }

    #[test]
    fn test_sorted_categories_is_alphabetical() {
        let cats = wobbly_categories();
        let names: Vec<&str> = sorted_categories(&cats)
            .into_iter()
            .map(|(n, _)| n.as_str())
            .collect();
        let mut expected = names.clone();
        expected.sort_unstable();
        assert_eq!(names, expected);
    }

    /// #687 regression: identical input must give a bit-identical percentage.
    ///
    /// Each iteration builds a *fresh* `HashMap`, which gets its own
    /// `RandomState` and therefore its own iteration order — the in-process
    /// stand-in for "run the binary again". A single run proves nothing, so
    /// this runs 25 of them.
    #[test]
    fn test_normalized_percentage_is_bit_stable_across_25_maps() {
        let first = normalized_percentage(&wobbly_categories());
        for i in 0..25 {
            let again = normalized_percentage(&wobbly_categories());
            assert_eq!(
                first.to_bits(),
                again.to_bits(),
                "percentage wobbled on iteration {i}: {first:?} vs {again:?}"
            );
        }
    }

    #[test]
    fn test_totals_are_bit_stable_across_25_maps() {
        let earned = total_earned(&wobbly_categories());
        let app_earned = applicable_earned(&wobbly_categories());
        let app_possible = applicable_possible(&wobbly_categories());
        for i in 0..25 {
            let cats = wobbly_categories();
            assert_eq!(total_earned(&cats).to_bits(), earned.to_bits(), "iter {i}");
            assert_eq!(
                applicable_earned(&cats).to_bits(),
                app_earned.to_bits(),
                "iter {i}"
            );
            assert_eq!(
                applicable_possible(&cats).to_bits(),
                app_possible.to_bits(),
                "iter {i}"
            );
        }
    }

    #[test]
    fn test_non_applicable_categories_excluded_from_applicable_totals() {
        let mut cats = HashMap::new();
        cats.insert("A".to_string(), CategoryScore::new(5.0, 10.0));
        cats.insert("B".to_string(), CategoryScore::not_applicable(20.0));
        assert_eq!(applicable_earned(&cats), 5.0);
        assert_eq!(applicable_possible(&cats), 10.0);
        // total_earned counts every category, N/A ones contribute 0 earned.
        assert_eq!(total_earned(&cats), 5.0);
        // Only "A" is applicable, so the mean is A's own percentage.
        assert_eq!(normalized_percentage(&cats), 50.0);
    }

    #[test]
    fn test_normalized_percentage_empty_is_zero() {
        let cats: HashMap<String, CategoryScore> = HashMap::new();
        assert_eq!(normalized_percentage(&cats), 0.0);
    }

    #[test]
    fn test_zero_max_category_counts_as_complete() {
        let mut cats = HashMap::new();
        cats.insert("Zero".to_string(), CategoryScore::new(0.0, 0.0));
        assert_eq!(normalized_percentage(&cats), 100.0);
    }
}
