use proptest::prelude::*;

// SIMD/Scalar equivalence property tests for Trueno integration
mod simd_equivalence_tests {
    use proptest::prelude::*;

    const EPSILON: f64 = 1e-6;

    /// Scalar implementation of cosine similarity for dense vectors
    /// This is the reference implementation that SIMD must match
    fn cosine_similarity_scalar(v1: &[f64], v2: &[f64]) -> f64 {
        if v1.len() != v2.len() || v1.is_empty() {
            return 0.0;
        }

        let mut dot_product = 0.0;
        let mut norm1 = 0.0;
        let mut norm2 = 0.0;

        for i in 0..v1.len() {
            dot_product += v1[i] * v2[i];
            norm1 += v1[i] * v1[i];
            norm2 += v2[i] * v2[i];
        }

        if norm1 > 0.0 && norm2 > 0.0 {
            dot_product / (norm1.sqrt() * norm2.sqrt())
        } else {
            0.0
        }
    }

    /// Scalar implementation of entropy calculation
    /// This is the reference implementation that SIMD must match
    fn entropy_scalar(probabilities: &[f64]) -> f64 {
        let mut entropy = 0.0;
        for &p in probabilities {
            if p > 0.0 {
                entropy -= p * p.log2();
            }
        }
        entropy
    }

    /// SIMD implementation of cosine similarity using Trueno
    /// Uses Trueno's Vector type with auto-selected SIMD backend
    #[cfg(feature = "simd")]
    fn cosine_similarity_simd(v1: &[f64], v2: &[f64]) -> f64 {
        use trueno::Vector;

        if v1.len() != v2.len() || v1.is_empty() {
            return 0.0;
        }

        // Convert f64 to f32 for Trueno (Trueno uses f32 for SIMD efficiency)
        let v1_f32: Vec<f32> = v1.iter().map(|&x| x as f32).collect();
        let v2_f32: Vec<f32> = v2.iter().map(|&x| x as f32).collect();

        // Create Trueno vectors with auto-selected backend
        let vec1 = Vector::from_slice(&v1_f32);
        let vec2 = Vector::from_slice(&v2_f32);

        // Compute dot product and norms using SIMD
        let dot = match vec1.dot(&vec2) {
            Ok(d) => d as f64,
            Err(_) => return 0.0,
        };

        let norm1 = match vec1.norm_l2() {
            Ok(n) => n as f64,
            Err(_) => return 0.0,
        };

        let norm2 = match vec2.norm_l2() {
            Ok(n) => n as f64,
            Err(_) => return 0.0,
        };

        if norm1 > 0.0 && norm2 > 0.0 {
            dot / (norm1 * norm2)
        } else {
            0.0
        }
    }

    /// SIMD implementation of entropy using Trueno
    /// Uses Trueno's Vector::log2() for vectorized Shannon entropy calculation.
    /// H = -Σ p * log2(p)
    #[cfg(feature = "simd")]
    fn entropy_simd(probabilities: &[f64]) -> f64 {
        use trueno::Vector;

        if probabilities.is_empty() {
            return 0.0;
        }

        // Convert to f32, replacing zeros with 1.0 (so log2(1) = 0, contributing nothing)
        let probs_f32: Vec<f32> = probabilities
            .iter()
            .map(|&p| if p > 0.0 { p as f32 } else { 1.0 })
            .collect();

        let probs_vec = Vector::from_slice(&probs_f32);

        // Compute log2(p) for all elements
        let log2_vec = match probs_vec.log2() {
            Ok(v) => v,
            Err(_) => return entropy_scalar(probabilities),
        };

        // Compute p * log2(p)
        let p_log_p = match probs_vec.mul(&log2_vec) {
            Ok(v) => v,
            Err(_) => return entropy_scalar(probabilities),
        };

        // Sum and negate: H = -Σ p * log2(p)
        match p_log_p.sum() {
            Ok(sum) => -(sum as f64),
            Err(_) => entropy_scalar(probabilities),
        }
    }

    // Generate valid probability distributions
    fn probability_distribution(size: usize) -> impl Strategy<Value = Vec<f64>> {
        prop::collection::vec(1.0f64..100.0, size..=size).prop_map(|v| {
            let sum: f64 = v.iter().sum();
            v.iter().map(|&x| x / sum).collect()
        })
    }

    // Generate non-zero vectors for cosine similarity
    fn non_zero_vector(size: usize) -> impl Strategy<Value = Vec<f64>> {
        prop::collection::vec(-100.0f64..100.0, size..=size)
            .prop_filter("at least one non-zero element", |v| {
                v.iter().any(|&x| x.abs() > 1e-10)
            })
    }

    proptest! {
        /// Property: Cosine similarity is symmetric
        #[test]
        fn cosine_similarity_symmetric(
            v1 in non_zero_vector(100),
            v2 in non_zero_vector(100)
        ) {
            let sim_12 = cosine_similarity_scalar(&v1, &v2);
            let sim_21 = cosine_similarity_scalar(&v2, &v1);
            prop_assert!((sim_12 - sim_21).abs() < EPSILON,
                "Symmetry violated: {} vs {}", sim_12, sim_21);
        }

        /// Property: Cosine similarity of identical vectors is 1.0
        #[test]
        fn cosine_similarity_identical(v in non_zero_vector(100)) {
            let sim = cosine_similarity_scalar(&v, &v);
            prop_assert!((sim - 1.0).abs() < EPSILON,
                "Self-similarity should be 1.0, got {}", sim);
        }

        /// Property: Cosine similarity is bounded [-1, 1]
        #[test]
        fn cosine_similarity_bounded(
            v1 in non_zero_vector(100),
            v2 in non_zero_vector(100)
        ) {
            let sim = cosine_similarity_scalar(&v1, &v2);
            prop_assert!((-1.0 - EPSILON..=1.0 + EPSILON).contains(&sim),
                "Similarity out of bounds: {}", sim);
        }

        /// Property: Entropy is non-negative
        #[test]
        fn entropy_non_negative(probs in probability_distribution(50)) {
            let entropy = entropy_scalar(&probs);
            prop_assert!(entropy >= -EPSILON,
                "Entropy should be non-negative, got {}", entropy);
        }

        /// Property: Uniform distribution has maximum entropy
        #[test]
        fn entropy_maximum_for_uniform(size in 10usize..100) {
            let uniform: Vec<f64> = vec![1.0 / size as f64; size];
            let max_entropy = (size as f64).log2();
            let computed = entropy_scalar(&uniform);
            prop_assert!((computed - max_entropy).abs() < EPSILON,
                "Uniform entropy should be log2({}), got {}", size, computed);
        }
    }

    /// RED TEST: SIMD cosine similarity must match scalar
    /// This test will FAIL until Trueno SIMD is implemented
    #[test]
    #[cfg(feature = "simd")]
    fn simd_cosine_similarity_matches_scalar() {
        use proptest::test_runner::{Config, TestRunner};

        let mut runner = TestRunner::new(Config::with_cases(1000));

        runner
            .run(&(non_zero_vector(256), non_zero_vector(256)), |(v1, v2)| {
                let scalar_result = cosine_similarity_scalar(&v1, &v2);
                let simd_result = cosine_similarity_simd(&v1, &v2);

                let diff = (scalar_result - simd_result).abs();
                prop_assert!(
                    diff < EPSILON,
                    "SIMD/scalar mismatch: scalar={}, simd={}, diff={}",
                    scalar_result,
                    simd_result,
                    diff
                );
                Ok(())
            })
            .expect("SIMD cosine similarity must match scalar within epsilon");
    }

    /// GREEN TEST: SIMD entropy must match scalar
    /// Now passes with Trueno v0.2.1's log2() support
    #[test]
    #[cfg(feature = "simd")]
    fn simd_entropy_matches_scalar() {
        use proptest::test_runner::{Config, TestRunner};

        let mut runner = TestRunner::new(Config::with_cases(1000));

        runner
            .run(&probability_distribution(256), |probs| {
                let scalar_result = entropy_scalar(&probs);
                let simd_result = entropy_simd(&probs);

                let diff = (scalar_result - simd_result).abs();
                // `entropy_simd` computes through trueno's f32 SIMD path, so it
                // cannot be held to an ABSOLUTE f64 bound. Measured counterexample:
                //     scalar = 7.732883734404172   (f64)
                //     simd   = 7.732882499694824   (f32)
                //     diff   = 1.2347e-6
                // The f32 ulp at |H| ~ 7.73 is 2^-20 ~ 9.5e-7, so EPSILON (1e-6)
                // was barely ONE ulp — it demanded precision f32 does not have.
                // The relative error is 1.60e-7 = 1.34 x f32::EPSILON, i.e. about
                // as accurate as f32 gets. The test was the defect, not the kernel.
                //
                // It survived because proptest must draw a near-uniform 256-way
                // distribution to push H high enough for one ulp to exceed 1e-6;
                // at lower entropy the absolute bound happened to hold.
                //
                // Scale to the magnitude and to f32's mantissa, with 8 ulps of
                // headroom for the 256-term summation.
                let tol = 8.0 * f64::from(f32::EPSILON) * scalar_result.abs().max(1.0);
                prop_assert!(
                    diff <= tol,
                    "SIMD/scalar mismatch beyond f32 precision: scalar={}, simd={}, diff={}, tol={}",
                    scalar_result,
                    simd_result,
                    diff,
                    tol
                );
                Ok(())
            })
            .expect("SIMD entropy must match scalar within epsilon");
    }

    /// Test various vector sizes for SIMD alignment edge cases
    #[test]
    #[cfg(feature = "simd")]
    fn simd_handles_various_sizes() {
        // Test sizes that aren't multiples of 4 (SIMD lane width)
        let sizes = [1, 3, 5, 7, 15, 17, 31, 33, 63, 65, 127, 129, 255, 257];

        for &size in &sizes {
            // Test cosine similarity
            let v1: Vec<f64> = (0..size).map(|i| (i as f64).sin()).collect();
            let v2: Vec<f64> = (0..size).map(|i| (i as f64).cos()).collect();

            let scalar = cosine_similarity_scalar(&v1, &v2);
            let simd = cosine_similarity_simd(&v1, &v2);

            assert!(
                (scalar - simd).abs() < EPSILON,
                "Cosine size {} mismatch: scalar={}, simd={}",
                size,
                scalar,
                simd
            );

            // Test entropy with various sizes
            let raw: Vec<f64> = (0..size).map(|i| (i as f64 + 1.0)).collect();
            let sum: f64 = raw.iter().sum();
            let probs: Vec<f64> = raw.iter().map(|&x| x / sum).collect();

            let scalar_entropy = entropy_scalar(&probs);
            let simd_entropy = entropy_simd(&probs);

            assert!(
                (scalar_entropy - simd_entropy).abs() < EPSILON,
                "Entropy size {} mismatch: scalar={}, simd={}",
                size,
                scalar_entropy,
                simd_entropy
            );
        }
    }

    /// Test empty and degenerate cases
    #[test]
    fn edge_cases() {
        // Empty vectors
        assert_eq!(cosine_similarity_scalar(&[], &[]), 0.0);

        // Single element
        assert!((cosine_similarity_scalar(&[1.0], &[1.0]) - 1.0).abs() < EPSILON);

        // Zero vector
        assert_eq!(cosine_similarity_scalar(&[0.0, 0.0], &[1.0, 1.0]), 0.0);

        // Orthogonal vectors
        let orth = cosine_similarity_scalar(&[1.0, 0.0], &[0.0, 1.0]);
        assert!(
            orth.abs() < EPSILON,
            "Orthogonal vectors should have ~0 similarity"
        );

        // Opposite vectors
        let opp = cosine_similarity_scalar(&[1.0, 1.0], &[-1.0, -1.0]);
        assert!(
            (opp + 1.0).abs() < EPSILON,
            "Opposite vectors should have -1 similarity"
        );
    }

    /// Benchmark-ready function that can be used to measure SIMD vs scalar performance
    #[test]
    fn baseline_performance_scalar() {
        let size = 10000;
        let v1: Vec<f64> = (0..size).map(|i| (i as f64).sin()).collect();
        let v2: Vec<f64> = (0..size).map(|i| (i as f64).cos()).collect();

        // Warmup and verify
        let result = cosine_similarity_scalar(&v1, &v2);
        assert!(result.is_finite(), "Result should be finite");
    }
}
