    // ============ TRUENO-RAG-2-COSINE: SIMD Cosine Similarity Tests ============
    // RED Phase: These tests define the expected behavior of SIMD-accelerated cosine similarity

    /// Test that SIMD cosine similarity matches scalar implementation for identical vectors
    #[test]
    fn test_simd_cosine_similarity_identical_vectors() {
        let v1 = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let v2 = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];

        let scalar = TursoVectorDB::cosine_similarity(&v1, &v2);
        let simd = TursoVectorDB::cosine_similarity_simd(&v1, &v2);

        assert!(
            (scalar - simd).abs() < 0.0001,
            "SIMD should match scalar for identical vectors: scalar={}, simd={}",
            scalar,
            simd
        );
        assert!(
            (simd - 1.0).abs() < 0.0001,
            "Identical vectors should have similarity 1.0"
        );
    }

    /// Test that SIMD cosine similarity matches scalar for orthogonal vectors
    #[test]
    fn test_simd_cosine_similarity_orthogonal_vectors() {
        let v1 = vec![1.0f32, 0.0, 0.0, 0.0];
        let v2 = vec![0.0f32, 1.0, 0.0, 0.0];

        let scalar = TursoVectorDB::cosine_similarity(&v1, &v2);
        let simd = TursoVectorDB::cosine_similarity_simd(&v1, &v2);

        assert!(
            (scalar - simd).abs() < 0.0001,
            "SIMD should match scalar for orthogonal vectors"
        );
        assert!(
            simd.abs() < 0.0001,
            "Orthogonal vectors should have similarity 0.0"
        );
    }

    /// Test that SIMD cosine similarity handles large vectors (OpenAI embedding dimension)
    #[test]
    fn test_simd_cosine_similarity_large_vectors() {
        // OpenAI ada-002 embedding dimension is 1536
        let v1: Vec<f32> = (0..1536).map(|i| (i as f32 * 0.001).sin()).collect();
        let v2: Vec<f32> = (0..1536).map(|i| (i as f32 * 0.001).cos()).collect();

        let scalar = TursoVectorDB::cosine_similarity(&v1, &v2);
        let simd = TursoVectorDB::cosine_similarity_simd(&v1, &v2);

        assert!(
            (scalar - simd).abs() < 0.001,
            "SIMD should match scalar for large vectors: scalar={}, simd={}",
            scalar,
            simd
        );
    }

    /// Test that SIMD handles non-SIMD-aligned vector sizes (not multiple of 4 or 8)
    #[test]
    fn test_simd_cosine_similarity_unaligned_sizes() {
        // Test various sizes that aren't multiples of SIMD lane width
        for size in [1, 3, 5, 7, 9, 13, 17, 33, 65, 129] {
            let v1: Vec<f32> = (0..size).map(|i| i as f32 * 0.1).collect();
            let v2: Vec<f32> = (0..size).map(|i| (i as f32 * 0.1).powi(2)).collect();

            let scalar = TursoVectorDB::cosine_similarity(&v1, &v2);
            let simd = TursoVectorDB::cosine_similarity_simd(&v1, &v2);

            assert!(
                (scalar - simd).abs() < 0.001,
                "SIMD should match scalar for size {}: scalar={}, simd={}",
                size,
                scalar,
                simd
            );
        }
    }

    /// Test SIMD handles empty vectors
    #[test]
    fn test_simd_cosine_similarity_empty_vectors() {
        let v1: Vec<f32> = vec![];
        let v2: Vec<f32> = vec![];

        let simd = TursoVectorDB::cosine_similarity_simd(&v1, &v2);
        assert!(
            simd == 0.0 || simd.is_nan(),
            "Empty vectors should return 0 or NaN"
        );
    }

    /// Test SIMD handles zero vectors
    #[test]
    fn test_simd_cosine_similarity_zero_vectors() {
        let v1 = vec![0.0f32, 0.0, 0.0, 0.0];
        let v2 = vec![1.0f32, 2.0, 3.0, 4.0];

        let simd = TursoVectorDB::cosine_similarity_simd(&v1, &v2);
        assert!(simd == 0.0, "Zero vector should result in 0.0 similarity");
    }

    /// Test SIMD handles negative values
    #[test]
    fn test_simd_cosine_similarity_opposite_vectors() {
        let v1 = vec![1.0f32, 2.0, 3.0, 4.0];
        let v2 = vec![-1.0f32, -2.0, -3.0, -4.0];

        let scalar = TursoVectorDB::cosine_similarity(&v1, &v2);
        let simd = TursoVectorDB::cosine_similarity_simd(&v1, &v2);

        assert!(
            (scalar - simd).abs() < 0.0001,
            "SIMD should match scalar for opposite vectors"
        );
        assert!(
            (simd + 1.0).abs() < 0.0001,
            "Opposite vectors should have similarity -1.0"
        );
    }

    /// Property test: SIMD and scalar should always produce equivalent results
    #[test]
    fn test_simd_scalar_equivalence_property() {
        use std::f32::consts::PI;

        // Generate 100 random test cases
        for seed in 0..100 {
            let size = 64 + (seed % 200); // Sizes from 64 to 263
            let v1: Vec<f32> = (0..size)
                .map(|i| ((i as f32 + seed as f32) * 0.1).sin())
                .collect();
            let v2: Vec<f32> = (0..size)
                .map(|i| ((i as f32 + seed as f32) * PI * 0.1).cos())
                .collect();

            let scalar = TursoVectorDB::cosine_similarity(&v1, &v2);
            let simd = TursoVectorDB::cosine_similarity_simd(&v1, &v2);

            assert!(
                (scalar - simd).abs() < 0.001,
                "Seed {}, size {}: scalar={}, simd={}, diff={}",
                seed,
                size,
                scalar,
                simd,
                (scalar - simd).abs()
            );
        }
    }

    /// Performance baseline test (documents expected speedup)
    #[test]
    fn test_simd_cosine_similarity_performance_baseline() {
        use std::time::Instant;

        // Large vectors to measure performance
        let v1: Vec<f32> = (0..10000).map(|i| (i as f32 * 0.0001).sin()).collect();
        let v2: Vec<f32> = (0..10000).map(|i| (i as f32 * 0.0001).cos()).collect();

        // Warmup
        let _ = TursoVectorDB::cosine_similarity(&v1, &v2);
        let _ = TursoVectorDB::cosine_similarity_simd(&v1, &v2);

        // Benchmark scalar
        let scalar_start = Instant::now();
        for _ in 0..1000 {
            let _ = TursoVectorDB::cosine_similarity(&v1, &v2);
        }
        let scalar_duration = scalar_start.elapsed();

        // Benchmark SIMD
        let simd_start = Instant::now();
        for _ in 0..1000 {
            let _ = TursoVectorDB::cosine_similarity_simd(&v1, &v2);
        }
        let simd_duration = simd_start.elapsed();

        // Document the speedup (soft assertion - just log it)
        let speedup = scalar_duration.as_nanos() as f64 / simd_duration.as_nanos() as f64;
        eprintln!(
            "SIMD cosine similarity speedup: {:.2}x (scalar: {:?}, simd: {:?})",
            speedup, scalar_duration, simd_duration
        );

        // Verify correctness
        let scalar = TursoVectorDB::cosine_similarity(&v1, &v2);
        let simd = TursoVectorDB::cosine_similarity_simd(&v1, &v2);
        assert!((scalar - simd).abs() < 0.001);
    }
