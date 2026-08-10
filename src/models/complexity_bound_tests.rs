// Unit tests and property tests for complexity bound types.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_bound_size() {
        // Check the struct size - may be 8 or 16 bytes depending on platform and serde
        let size = std::mem::size_of::<ComplexityBound>();
        assert!(
            size == 8 || size == 16,
            "ComplexityBound size is {size} bytes"
        );
        assert_eq!(std::mem::align_of::<ComplexityBound>(), 8);
    }

    #[test]
    fn test_cache_complexity_size() {
        // Ensure cache complexity is 8 bytes
        assert_eq!(std::mem::size_of::<CacheComplexity>(), 8);
        assert_eq!(std::mem::align_of::<CacheComplexity>(), 8);
    }

    #[test]
    fn test_big_o_ordering() {
        assert!(BigOClass::Constant.is_better_than(&BigOClass::Linear));
        assert!(BigOClass::Linear.is_better_than(&BigOClass::Quadratic));
        assert!(BigOClass::Logarithmic.is_better_than(&BigOClass::Linear));
        assert!(BigOClass::Linearithmic.is_better_than(&BigOClass::Quadratic));
    }

    #[test]
    fn test_complexity_bound_creation() {
        let bound = ComplexityBound::linear()
            .with_confidence(90)
            .with_flags(ComplexityFlags::PROVEN | ComplexityFlags::TIGHT_BOUND);

        assert_eq!(bound.class, BigOClass::Linear);
        assert_eq!(bound.confidence, 90);
        assert!(bound.flags.is_proven());
        assert_eq!(bound.notation(), "O(n)");
    }

    #[test]
    fn test_growth_estimation() {
        let linear = ComplexityBound::linear();
        let quadratic = ComplexityBound::quadratic();

        assert_eq!(linear.estimate_operations(100.0), 100.0);
        assert_eq!(quadratic.estimate_operations(100.0), 10000.0);
    }

    /// Display for BigOClass covers all 9 variants via notation().
    #[test]
    fn test_big_o_class_display_all_variants() {
        assert_eq!(format!("{}", BigOClass::Constant), "O(1)");
        assert_eq!(format!("{}", BigOClass::Logarithmic), "O(log n)");
        assert_eq!(format!("{}", BigOClass::Linear), "O(n)");
        assert_eq!(format!("{}", BigOClass::Linearithmic), "O(n log n)");
        assert_eq!(format!("{}", BigOClass::Quadratic), "O(n²)");
        assert_eq!(format!("{}", BigOClass::Cubic), "O(n³)");
        assert_eq!(format!("{}", BigOClass::Exponential), "O(2^n)");
        assert_eq!(format!("{}", BigOClass::Factorial), "O(n!)");
        assert_eq!(format!("{}", BigOClass::Unknown), "O(?)");
    }

    /// Display for InputVariable covers all 5 variants.
    #[test]
    fn test_input_variable_display_all_variants() {
        assert_eq!(format!("{}", InputVariable::N), "n");
        assert_eq!(format!("{}", InputVariable::M), "m");
        assert_eq!(format!("{}", InputVariable::K), "k");
        assert_eq!(format!("{}", InputVariable::D), "d");
        assert_eq!(format!("{}", InputVariable::Custom), "x");
    }

    /// growth_factor covers every BigOClass arm, including the
    /// Stirling-approximation branch inside Factorial (n > 20).
    #[test]
    fn test_big_o_class_growth_factor_all_variants() {
        assert_eq!(BigOClass::Constant.growth_factor(100.0), 1.0);
        assert_eq!(BigOClass::Logarithmic.growth_factor(4.0), 2.0);
        assert_eq!(BigOClass::Linear.growth_factor(100.0), 100.0);
        assert_eq!(BigOClass::Linearithmic.growth_factor(4.0), 4.0 * 2.0);
        assert_eq!(BigOClass::Quadratic.growth_factor(10.0), 100.0);
        assert_eq!(BigOClass::Cubic.growth_factor(10.0), 1000.0);
        assert_eq!(BigOClass::Exponential.growth_factor(3.0), 8.0);
        // Factorial small-n branch: 5! = 120
        assert_eq!(BigOClass::Factorial.growth_factor(5.0), 120.0);
        // Factorial Stirling branch: n > 20 hits the approximation arm
        assert!(BigOClass::Factorial.growth_factor(25.0).is_finite());
        // Unknown maps to NaN
        assert!(BigOClass::Unknown.growth_factor(10.0).is_nan());
    }

    /// ComplexityFlags covers new/with/has/is_worst_case/is_proven.
    #[test]
    fn test_complexity_flags_builder_and_queries() {
        let empty = ComplexityFlags::new();
        assert!(!empty.has(ComplexityFlags::WORST_CASE));
        assert!(!empty.is_worst_case());
        assert!(!empty.is_proven());

        let flags = ComplexityFlags::new()
            .with(ComplexityFlags::WORST_CASE)
            .with(ComplexityFlags::PROVEN)
            .with(ComplexityFlags::TIGHT_BOUND);
        assert!(flags.has(ComplexityFlags::WORST_CASE));
        assert!(flags.has(ComplexityFlags::PROVEN));
        assert!(flags.has(ComplexityFlags::TIGHT_BOUND));
        assert!(flags.is_worst_case());
        assert!(flags.is_proven());
        assert!(!flags.has(ComplexityFlags::EMPIRICAL));
    }

    #[test]
    fn test_master_theorem() {
        // Test case: T(n) = 2T(n/2) + O(n) -> O(n log n)
        let recurrence = RecurrenceRelation {
            recursive_calls: vec![RecursiveCall {
                division_factor: 2,
                size_reduction: 0,
                count: 2,
            }],
            work_per_call: ComplexityBound::linear(),
            base_case_size: 1,
        };

        let solution = recurrence.solve_master_theorem();
        assert!(solution.is_some());
        assert_eq!(solution.unwrap().class, BigOClass::Linearithmic);
    }

    /// ComplexityBound::default() delegates to Self::unknown().
    #[test]
    fn test_complexity_bound_default_is_unknown() {
        let bound: ComplexityBound = Default::default();
        let unknown = ComplexityBound::unknown();
        assert_eq!(bound.class, unknown.class);
        assert_eq!(bound.confidence, unknown.confidence);
    }

    /// Display for ComplexityBound (core.rs:280) formats as
    /// "{notation} ({confidence}% confidence)".
    #[test]
    fn test_complexity_bound_display_formats_notation_and_confidence() {
        let bound = ComplexityBound::linear().with_confidence(85);
        let rendered = format!("{bound}");
        assert!(
            rendered.contains("85%") && rendered.contains("confidence"),
            "Display must include confidence number + label, got {rendered:?}"
        );
        assert_eq!(rendered, format!("{} (85% confidence)", bound.notation()));
    }

    /// CacheComplexity::default() (core.rs:301) delegates to new(0, 1, Unknown).
    #[test]
    fn test_cache_complexity_default_is_zero_miss_one_penalty_unknown() {
        let cache: CacheComplexity = Default::default();
        assert_eq!(cache.hit_ratio, 0);
        assert_eq!(cache.miss_penalty, 1);
        assert_eq!(cache.working_set, BigOClass::Unknown);
    }

    /// The MCP `analyze_big_o` payload carried this struct verbatim, so
    /// clients received `"_padding":[0,0]` — `#[repr(C)]` alignment filler —
    /// alongside the measurements. Padding is layout, never data.
    #[test]
    fn test_serialised_bound_carries_no_alignment_padding() {
        let json = serde_json::to_string(&ComplexityBound::quadratic()).expect("serialise");
        assert!(
            !json.contains("_padding"),
            "alignment filler must not be serialised: {json}"
        );
        // The measurements themselves still round-trip.
        let back: ComplexityBound = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(back.class, BigOClass::Quadratic);
        assert_eq!(back.confidence, ComplexityBound::quadratic().confidence);
    }
}

