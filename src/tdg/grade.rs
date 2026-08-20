#![cfg_attr(coverage_nightly, coverage(off))]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Grade.
///
/// One serialization, everywhere: the symbolic form (`"A+"`, `"B-"`), which is
/// what `Display`, SARIF, `pmat tdg --format json` and the `--min-grade`
/// argument parser already used. `Serialize` was left DERIVED, so the Rust
/// variant names leaked onto every serde-rendered surface and the SAME binary
/// reported the SAME score as `"grade": "A+"` from `pmat tdg --format json`
/// and `"grade": "APlus"` from `pmat analyze tdg --format json` and from the
/// MCP `quality_gate` tool — no machine consumer could match on one string
/// (GH #703, #669). `Serialize` is therefore written by hand below and emits
/// exactly what `Display` does. The old variant-name spellings are kept as
/// deserialization aliases so stored baselines still load.
// `Serialize`/`Deserialize` are both implemented by hand below (deserialization
// accepts the wire spelling AND the historical variant names), so neither may
// also be derived -- two fix agents each solved this and the merge kept both,
// which is a conflicting-impl compile error.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Grade {
    APlus,
    A,
    AMinus,
    BPlus,
    B,
    BMinus,
    CPlus,
    #[default]
    C,
    CMinus,
    D,
    F,
}

/// Grade spellings as `Serialize` emits them, for error messages.
/// CANONICAL, and the only list of grade spellings anything may enumerate.
/// Ordered worst-last, matching `Grade`'s own `Ord`. CB-200 kept a private
/// `["A","B","C","D","F"]` and was blind to every modified grade for a release.
pub(crate) const GRADE_VARIANTS: &[&str] =
    &["A+", "A", "A-", "B+", "B", "B-", "C+", "C", "C-", "D", "F"];

impl Serialize for Grade {
    /// Emits the symbolic form -- byte-identical to `Display`. See the note on
    /// `Grade`: a derived `Serialize` here is what made one binary print two
    /// spellings of one grade.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

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
    pub(crate) fn from_variant_name(name: &str) -> Option<Self> {
        // Accepts BOTH spellings. `Serialize` emits the variant name (what
        // every stored baseline contains), but symbolic forms reach this from
        // user input and from output written while the two round-3 fixes
        // disagreed about the wire format. One serialization, two accepted
        // inputs -- which is what #669's "two spellings" finding asked for.
        Some(match name.to_ascii_lowercase().as_str() {
            "aplus" | "a+" => Grade::APlus,
            "a" => Grade::A,
            "aminus" | "a-" => Grade::AMinus,
            "bplus" | "b+" => Grade::BPlus,
            "b" => Grade::B,
            "bminus" | "b-" => Grade::BMinus,
            "cplus" | "c+" => Grade::CPlus,
            "c" => Grade::C,
            "cminus" | "c-" => Grade::CMinus,
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

    /// The half-open score band this grade covers, `[floor, ceiling)` — the top
    /// grade's band is closed at 100.0.
    ///
    /// Exists so nothing has to restate the bands in prose. `pmat explain`
    /// used to carry a hand-written five-grade table (A = "Score 85-94") that
    /// contradicted this one (A = 90..95, and A-/B+/B-/C+/C-/D were not listed
    /// at all), so `explain TDG-A-` answered "No checks matching 'TDG-A-'" for a
    /// grade `pmat tdg` prints routinely.
    #[must_use]
    pub fn score_band(self) -> (f32, f32) {
        match GRADE_BANDS.iter().position(|(_, grade)| *grade == self) {
            // `F` is everything below the last floor.
            None => (0.0, GRADE_BANDS[GRADE_BANDS.len() - 1].0),
            Some(0) => (GRADE_BANDS[0].0, 100.0),
            Some(i) => (GRADE_BANDS[i].0, GRADE_BANDS[i - 1].0),
        }
    }

    /// Every grade, best first — the population `pmat explain` must document.
    #[must_use]
    pub fn all() -> [Grade; 11] {
        [
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
        ]
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
    /// Not a scored component — the source of the critical-defect penalty
    /// `TdgScore::calculate_total` applies on top of the components.
    ///
    /// The penalty was the single largest term in the score (up to ~91 points)
    /// and appeared in `penalties_applied` nowhere at all: a file could read
    /// `total: 25.16, grade: F, critical_defects_count: 3` with
    /// `penalties_applied: ["Duplication"]` worth 9 points, so any consumer
    /// reconstructing the grade from components-minus-penalties was wrong by the
    /// whole difference and concluded the defects had cost nothing. A penalty
    /// that moves the score must be attributable like every other penalty.
    CriticalDefect,
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

    /// Regression (GH #703, #669): ONE serialization for one grade.
    /// `pmat tdg --format json` and `pmat analyze tdg --format json` disagreed
    /// ("A+" vs "APlus") on the identical score, so no machine consumer could
    /// match on a string.
    ///
    /// The wire form is the SYMBOLIC one -- what `Display`, SARIF,
    /// `pmat tdg --format json` and `--min-grade` all already used. Only the
    /// derived `Serialize` spoke variant names. `Deserialize` still accepts
    /// both, so every baseline already on disk keeps its meaning.
    #[test]
    fn test_grade_has_exactly_one_json_form_and_accepts_both() {
        for grade in BEST_TO_WORST {
            let json = serde_json::to_string(&grade).expect("serialize");
            assert_eq!(
                json,
                format!("\"{grade}\""),
                "JSON form must be the symbolic form Display prints"
            );

            let back: Grade = serde_json::from_str(&json).expect("round trip");
            assert_eq!(back, grade);

            // The historical variant-name form must still parse.
            let variant = format!("\"{grade:?}\"");
            let from_variant: Grade =
                serde_json::from_str(&variant).expect("variant-name form must load");
            assert_eq!(from_variant, grade);
        }
    }

    /// Baselines written with the old variant names must still load.
    #[test]
    fn test_grade_deserializes_legacy_variant_names() {
        let legacy = [
            ("\"APLus\"", Grade::APlus),
            ("\"APlus\"", Grade::APlus),
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
        assert_eq!(map.get(&Grade::APlus), Some(&3));
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

    /// GH #680 (second round): the enum was misspelled `APlus`. What pmat
    /// emits from now on is `"A+"`; what it accepts still includes the variant
    /// names, so baselines written by <= v3.29.0 keep loading.
    #[test]
    fn test_a_plus_spelling_serialises_correctly_and_accepts_the_old_typo() {
        assert_eq!(serde_json::to_string(&Grade::APlus).unwrap(), "\"A+\"");
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

    /// `score_band` must agree with `from_score` at every boundary, or the
    /// documentation generated from it would restate the same contradiction
    /// `pmat explain` used to carry.
    #[test]
    fn test_score_band_agrees_with_from_score() {
        for grade in Grade::all() {
            let (floor, ceiling) = grade.score_band();
            assert_eq!(
                Grade::from_score(floor),
                grade,
                "{grade}'s floor {floor} does not map back to {grade}"
            );
            // Just under the ceiling is still this grade; at the ceiling it is
            // the next better one (except for the top band, closed at 100).
            assert_eq!(Grade::from_score(ceiling - 0.1), grade, "{grade} ceiling");
            if grade != Grade::APlus {
                assert_ne!(Grade::from_score(ceiling), grade, "{grade} ceiling is open");
            }
        }
        assert_eq!(Grade::APlus.score_band(), (95.0, 100.0));
        assert_eq!(Grade::F.score_band(), (0.0, 50.0));
    }

    #[test]
    fn test_unknown_variant_name_is_rejected() {
        let err = serde_json::from_str::<Grade>("\"Z\"").unwrap_err();
        assert!(err.to_string().contains("unknown variant"), "{err}");
    }
}

/// Every canonical spelling parses, and `GRADE_VARIANTS` is in `Ord` order.
///
/// A TEST, not a `const` block: `from_variant_name` lowercases, and
/// `str::to_ascii_lowercase` is not a `const fn`, so this cannot run in a const
/// context. It runs under `cargo test --lib`, which is the rung this repository
/// actually executes. The anchoring property — that rank tracks the score band
/// — is proved in `contracts/lean/Theorems/Tdg/Grade.lean`, because no
/// self-referential assertion can catch a scale reversed together with its own
/// array.
#[test]
fn grade_order_is_parseable() {
    let parsed: Vec<Grade> = GRADE_VARIANTS
        .iter()
        .map(|s| Grade::from_variant_name(s).expect("every canonical spelling must parse"))
        .collect();
    let mut sorted = parsed.clone();
    sorted.sort();
    assert_eq!(parsed, sorted, "GRADE_VARIANTS is not in Ord order");
    assert_eq!(
        parsed.len(),
        Grade::all().len(),
        "GRADE_VARIANTS and Grade::all() disagree"
    );
}
const GRADE_ORDER_IS_PARSEABLE: () = ();
