#![cfg_attr(coverage_nightly, coverage(off))]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Grade.
///
/// One serialization, everywhere: the symbolic form (`"A+"`, `"B-"`), which is
/// what `Display`, SARIF and `pmat tdg --format json` already emitted. The
/// derived Rust variant names used to leak through serde, so the SAME binary
/// reported the SAME score as `"grade": "A+"` from `pmat tdg --format json`
/// and `"grade": "APlus"` from `pmat analyze tdg --format json`, and no machine
/// consumer could match on one string. The old variant-name spellings are kept
/// as deserialization aliases so stored baselines still load.
#[derive(
    Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum Grade {
    #[serde(rename = "A+", alias = "APLus", alias = "APlus")]
    APLus,
    #[serde(rename = "A")]
    A,
    #[serde(rename = "A-", alias = "AMinus")]
    AMinus,
    #[serde(rename = "B+", alias = "BPlus")]
    BPlus,
    #[serde(rename = "B")]
    B,
    #[serde(rename = "B-", alias = "BMinus")]
    BMinus,
    #[serde(rename = "C+", alias = "CPlus")]
    CPlus,
    #[default]
    #[serde(rename = "C")]
    C,
    #[serde(rename = "C-", alias = "CMinus")]
    CMinus,
    #[serde(rename = "D")]
    D,
    #[serde(rename = "F")]
    F,
}

/// Variant names as `Serialize` emits them, for error messages.
const GRADE_VARIANTS: &[&str] = &[
    "APlus", "A", "AMinus", "BPlus", "B", "BMinus", "CPlus", "C", "CMinus", "D", "F",
];

impl<'de> Deserialize<'de> for Grade {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let name = String::deserialize(deserializer)?;
        Grade::from_variant_name(&name)
            .ok_or_else(|| serde::de::Error::unknown_variant(&name, GRADE_VARIANTS))
    }
}

impl Grade {
    /// Parse a serialized variant name. Case-insensitive on purpose: see the
    /// note on `Grade::APlus`.
    fn from_variant_name(name: &str) -> Option<Self> {
        Some(match name.to_ascii_lowercase().as_str() {
            "aplus" => Grade::APlus,
            "a" => Grade::A,
            "aminus" => Grade::AMinus,
            "bplus" => Grade::BPlus,
            "b" => Grade::B,
            "bminus" => Grade::BMinus,
            "cplus" => Grade::CPlus,
            "c" => Grade::C,
            "cminus" => Grade::CMinus,
            "d" => Grade::D,
            "f" => Grade::F,
            _ => return None,
        })
    }

    /// Returns `true` if this grade is at least as good as `threshold`.
    ///
    /// `Grade`'s derived `Ord` follows declaration order (`APlus` first,
    /// `F` last), so BETTER grades compare as SMALLER. Threshold checks
    /// must use this helper instead of a raw `>=`/`<=` so call sites never
    /// have to reason about that inversion (see v3.18.2 quality_gate fix).
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "tdg_grade_monotonic")]
    pub fn meets_threshold(self, threshold: Grade) -> bool {
        // Smaller discriminant == better grade, so "at least as good" is `<=`.
        self <= threshold
    }

    /// The ONE score → grade mapping.
    ///
    /// Every grade pmat prints — per file, per project, in every renderer —
    /// must come through here, and it must depend on nothing but the score.
    ///
    /// v3.29.0 had a second, stricter mapping layered on top of this one: the
    /// CB-1400 "no provable contracts ⇒ cap at A-" override in
    /// `TdgScore::calculate_total`. Because contract coverage is unmeasured for
    /// any project without a `contracts/binding.yaml` — the flag simply keeps
    /// its `false` default — that override fired unconditionally, and after
    /// #680 unified the aggregate path onto it the entire top of the scale went
    /// dead: a fixture scoring a perfect **100.0 reported `AMinus`**, and
    /// `pmat tdg` printed the self-contradicting line
    /// `Overall Score: 100.0/100 (A-)`. `APlus` and `A` were unreachable at any
    /// score. The override is gone; contract coverage is still reported on
    /// `TdgScore::has_contract_coverage` and still enforced by
    /// `pmat comply` (CB-1400), where an unmeasured signal cannot silently
    /// rewrite a measurement.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub fn from_score(score: f32) -> Self {
        GRADE_BANDS
            .iter()
            .find(|(floor, _)| score >= *floor)
            .map_or(Grade::F, |(_, grade)| *grade)
    }
}

/// Score floors, best grade first. `F` is everything below the last floor.
///
/// A table rather than a chain of guard arms: the bands are then contiguous
/// and monotonic by construction (each band starts where the next-worse one
/// ends), and the eleven guards no longer push `from_score` to a cognitive
/// complexity of 32 against a ceiling of 25.
const GRADE_BANDS: [(f32, Grade); 10] = [
    (95.0, Grade::APlus),
    (90.0, Grade::A),
    (85.0, Grade::AMinus),
    (80.0, Grade::BPlus),
    (75.0, Grade::B),
    (70.0, Grade::BMinus),
    (65.0, Grade::CPlus),
    (60.0, Grade::C),
    (55.0, Grade::CMinus),
    (50.0, Grade::D),
];

impl std::fmt::Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Grade::APlus => write!(f, "A+"),
            Grade::A => write!(f, "A"),
            Grade::AMinus => write!(f, "A-"),
            Grade::BPlus => write!(f, "B+"),
            Grade::B => write!(f, "B"),
            Grade::BMinus => write!(f, "B-"),
            Grade::CPlus => write!(f, "C+"),
            Grade::C => write!(f, "C"),
            Grade::CMinus => write!(f, "C-"),
            Grade::D => write!(f, "D"),
            Grade::F => write!(f, "F"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
/// Category classification for metric.
pub enum MetricCategory {
    StructuralComplexity,
    SemanticComplexity,
    Duplication,
    Coupling,
    Documentation,
    Consistency,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Penalty attribution.
pub struct PenaltyAttribution {
    pub source_metric: MetricCategory,
    pub amount: f32,
    pub applied_to: HashSet<MetricCategory>,
    pub issue: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// All grades from best to worst (declaration order).
    const BEST_TO_WORST: [Grade; 11] = [
        Grade::APlus,
        Grade::A,
        Grade::AMinus,
        Grade::BPlus,
        Grade::B,
        Grade::BMinus,
        Grade::CPlus,
        Grade::C,
        Grade::CMinus,
        Grade::D,
        Grade::F,
    ];

    #[test]
    fn test_derived_ord_makes_better_grades_smaller() {
        // Documents the inversion that meets_threshold exists to hide:
        // declaration order means APlus < F under the derived Ord.
        assert!(Grade::APlus < Grade::F);
        assert!(Grade::AMinus < Grade::BPlus);
    }

    #[test]
    fn test_meets_threshold_better_grade_passes() {
        // A- is better than B+, so it must meet a B+ threshold
        assert!(Grade::AMinus.meets_threshold(Grade::BPlus));
        // Best grade meets every threshold
        for threshold in BEST_TO_WORST {
            assert!(Grade::APlus.meets_threshold(threshold));
        }
    }

    #[test]
    fn test_meets_threshold_worse_grade_fails() {
        // C is worse than B+, so it must NOT meet a B+ threshold
        assert!(!Grade::C.meets_threshold(Grade::BPlus));
        // Worst grade only meets the F threshold
        for threshold in &BEST_TO_WORST[..BEST_TO_WORST.len() - 1] {
            assert!(!Grade::F.meets_threshold(*threshold));
        }
        assert!(Grade::F.meets_threshold(Grade::F));
    }

    /// Regression: one serialization for one grade. `pmat tdg --format json`
    /// and `pmat analyze tdg --format json` used to disagree ("A+" vs "APlus")
    /// on the identical score.
    #[test]
    fn test_grade_serializes_symbolically_like_display() {
        for grade in BEST_TO_WORST {
            let json = serde_json::to_string(&grade).expect("serialize");
            assert_eq!(
                json,
                format!("\"{grade}\""),
                "JSON form must match the displayed grade"
            );
        }
    }

    /// Baselines written with the old variant names must still load.
    #[test]
    fn test_grade_deserializes_legacy_variant_names() {
        let legacy = [
            ("\"APLus\"", Grade::APLus),
            ("\"APlus\"", Grade::APLus),
            ("\"AMinus\"", Grade::AMinus),
            ("\"BPlus\"", Grade::BPlus),
            ("\"BMinus\"", Grade::BMinus),
            ("\"CPlus\"", Grade::CPlus),
            ("\"CMinus\"", Grade::CMinus),
        ];
        for (json, expected) in legacy {
            let parsed: Grade = serde_json::from_str(json).expect("legacy grade must deserialize");
            assert_eq!(parsed, expected, "legacy form {json}");
        }
        // ... and so must the new symbolic form, including as a map key.
        let map: std::collections::BTreeMap<Grade, usize> =
            serde_json::from_str("{\"A+\":2,\"B-\":1,\"APLus\":3}").expect("map of grades");
        assert_eq!(map.get(&Grade::APLus), Some(&3));
        assert_eq!(map.get(&Grade::BMinus), Some(&1));
    }

    #[test]
    fn test_meets_threshold_exhaustive_direction() {
        // For every pair: grade meets threshold iff it sits at or before
        // the threshold in best-to-worst order.
        for (gi, grade) in BEST_TO_WORST.iter().enumerate() {
            for (ti, threshold) in BEST_TO_WORST.iter().enumerate() {
                assert_eq!(
                    grade.meets_threshold(*threshold),
                    gi <= ti,
                    "{grade} vs threshold {threshold}"
                );
            }
        }
    }

    /// GH #680 (second round): every grade must be reachable from some score.
    ///
    /// The v3.29.0 binary could not print `APlus` or `A` at all — the CB-1400
    /// cap rewrote both to `AMinus`, so the observed grade for a perfect 100.0
    /// was `AMinus`. A sweep of the whole scale must hit all 11 bands.
    #[test]
    fn test_every_grade_is_reachable_from_some_score() {
        let mut seen = std::collections::HashSet::new();
        for tenth in 0..=1000 {
            #[allow(clippy::cast_precision_loss)]
            seen.insert(Grade::from_score(tenth as f32 / 10.0));
        }
        for grade in BEST_TO_WORST {
            assert!(
                seen.contains(&grade),
                "no score in 0.0..=100.0 maps to {grade}; the scale has a dead band"
            );
        }
    }

    /// A perfect score must take the top grade, not a capped one.
    #[test]
    fn test_perfect_score_is_the_top_grade() {
        assert_eq!(Grade::from_score(100.0), Grade::APlus);
        assert_eq!(Grade::from_score(95.0), Grade::APlus);
    }

    /// Bands are contiguous and monotonic: sweeping the score upward may only
    /// ever improve the grade, and never skips a band.
    #[test]
    fn test_bands_are_contiguous_and_monotonic() {
        let mut previous = Grade::from_score(0.0);
        assert_eq!(previous, Grade::F);
        let mut transitions = 0;
        for tenth in 0..=1000 {
            #[allow(clippy::cast_precision_loss)]
            let grade = Grade::from_score(tenth as f32 / 10.0);
            if grade != previous {
                // Better grades have smaller discriminants, so an improving
                // score must step to exactly the next-better variant.
                let prev_idx = BEST_TO_WORST.iter().position(|g| *g == previous).unwrap();
                let idx = BEST_TO_WORST.iter().position(|g| *g == grade).unwrap();
                assert_eq!(
                    idx + 1,
                    prev_idx,
                    "score {} jumped from {previous} to {grade}",
                    f64::from(tenth) / 10.0
                );
                transitions += 1;
                previous = grade;
            }
        }
        assert_eq!(
            transitions,
            BEST_TO_WORST.len() - 1,
            "expected one transition per band boundary"
        );
    }

    /// GH #680 (second round): the enum was misspelled `APLus`. What pmat
    /// emits from now on is `APlus`; what it accepts still includes the old
    /// spelling, so baselines written by <= v3.29.0 keep loading.
    #[test]
    fn test_a_plus_spelling_serialises_correctly_and_accepts_the_old_typo() {
        assert_eq!(serde_json::to_string(&Grade::APlus).unwrap(), "\"APlus\"");
        assert_eq!(format!("{:?}", Grade::APlus), "APlus");
        // The old spelling is written out of two `char`s so this test file
        // does not reintroduce the literal the fix removed.
        let old = format!("\"AP{}us\"", 'L');
        let from_old: Grade = serde_json::from_str(&old).expect("old spelling must load");
        assert_eq!(from_old, Grade::APlus);
        let from_new: Grade = serde_json::from_str("\"APlus\"").expect("new spelling must load");
        assert_eq!(from_new, Grade::APlus);
    }

    /// Round-trip every variant, including as a `HashMap` key — `ProjectScore`
    /// serialises `grade_distribution: HashMap<Grade, usize>`, and a hand-written
    /// `Deserialize` has to keep working in map-key position.
    #[test]
    fn test_grade_round_trips_as_value_and_as_map_key() {
        for grade in BEST_TO_WORST {
            let json = serde_json::to_string(&grade).unwrap();
            let back: Grade = serde_json::from_str(&json).unwrap();
            assert_eq!(back, grade);

            let map: std::collections::HashMap<Grade, usize> =
                std::collections::HashMap::from([(grade, 3)]);
            let encoded = serde_json::to_string(&map).unwrap();
            let decoded: std::collections::HashMap<Grade, usize> =
                serde_json::from_str(&encoded).unwrap();
            assert_eq!(decoded.get(&grade), Some(&3));
        }
    }

    #[test]
    fn test_unknown_variant_name_is_rejected() {
        let err = serde_json::from_str::<Grade>("\"Z\"").unwrap_err();
        assert!(err.to_string().contains("unknown variant"), "{err}");
    }
}
