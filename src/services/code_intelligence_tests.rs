#[test]
fn test_analysis_request_cache_key() {
    let req = AnalysisRequest {
        project_path: "/test/project".to_string(),
        analysis_types: vec![AnalysisType::DuplicateDetection],
        include_patterns: vec![],
        exclude_patterns: vec![],
        max_depth: None,
        parallel: true,
    };

    let key1 = req.cache_key();
    let key2 = req.cache_key();

    assert_eq!(key1, key2);
}

#[tokio::test]
async fn test_unified_cache() {
    let cache = UnifiedCache::new(10);

    let report = AnalysisReport {
        duplicates: None,
        dead_code: None,
        complexity_metrics: None,
        dependency_graph: None,
        defect_predictions: None,
        graph_metrics: None,
        timestamp: Utc::now(),
    };

    cache.put("test_key".to_string(), report.clone()).await;

    let cached = cache.get("test_key").await;
    assert!(cached.is_some());
}

#[tokio::test]
async fn test_code_intelligence_creation() {
    let intelligence = CodeIntelligence::new();
    let (nodes, gen) = intelligence.get_dag_stats().await;

    assert_eq!(nodes, 0);
    assert_eq!(gen, 0);
}

/// Test UnifiedCache with zero capacity (validates unwrap fix at line 180)
#[tokio::test]
async fn test_unified_cache_zero_capacity() {
    // Zero capacity should default to 1 (NonZeroUsize requirement)
    let cache = UnifiedCache::new(0);

    let report = AnalysisReport {
        duplicates: None,
        dead_code: None,
        complexity_metrics: None,
        dependency_graph: None,
        defect_predictions: None,
        graph_metrics: None,
        timestamp: Utc::now(),
    };

    // Cache should work with capacity 1 (defaulted from 0)
    cache.put("key1".to_string(), report.clone()).await;

    // First item should be retrievable
    assert!(cache.get("key1").await.is_some());

    // Adding second item should evict first (LRU with capacity 1)
    let report2 = AnalysisReport {
        duplicates: None,
        dead_code: None,
        complexity_metrics: None,
        dependency_graph: None,
        defect_predictions: None,
        graph_metrics: None,
        timestamp: Utc::now(),
    };
    cache.put("key2".to_string(), report2).await;

    // First item should be evicted
    assert!(cache.get("key1").await.is_none());
    // Second item should be present
    assert!(cache.get("key2").await.is_some());
}

/// Test UnifiedCache with valid non-zero capacities
#[tokio::test]
async fn test_unified_cache_valid_capacities() {
    // Test capacity 1
    let cache1 = UnifiedCache::new(1);
    let report = AnalysisReport {
        duplicates: None,
        dead_code: None,
        complexity_metrics: None,
        dependency_graph: None,
        defect_predictions: None,
        graph_metrics: None,
        timestamp: Utc::now(),
    };
    cache1.put("test".to_string(), report.clone()).await;
    assert!(cache1.get("test").await.is_some());

    // Test capacity 100
    let cache100 = UnifiedCache::new(100);
    cache100.put("test".to_string(), report.clone()).await;
    assert!(cache100.get("test").await.is_some());

    // Test large capacity
    let cache_large = UnifiedCache::new(10_000);
    cache_large.put("test".to_string(), report).await;
    assert!(cache_large.get("test").await.is_some());
}

/// Test UnifiedCache LRU eviction behavior
#[tokio::test]
async fn test_unified_cache_lru_eviction() {
    let cache = UnifiedCache::new(2); // Capacity 2

    let report1 = AnalysisReport {
        duplicates: None,
        dead_code: None,
        complexity_metrics: None,
        dependency_graph: None,
        defect_predictions: None,
        graph_metrics: None,
        timestamp: Utc::now(),
    };

    let report2 = report1.clone();
    let report3 = report1.clone();

    // Fill cache to capacity
    cache.put("key1".to_string(), report1).await;
    cache.put("key2".to_string(), report2).await;

    // Both should be present
    assert!(cache.get("key1").await.is_some());
    assert!(cache.get("key2").await.is_some());

    // Add third item - should evict least recently used (key1)
    cache.put("key3".to_string(), report3).await;

    // key1 should be evicted
    assert!(cache.get("key1").await.is_none());
    // key2 and key3 should be present
    assert!(cache.get("key2").await.is_some());
    assert!(cache.get("key3").await.is_some());
}

// === AnalysisType tests ===

#[test]
fn test_analysis_type_equality() {
    assert_eq!(
        AnalysisType::DuplicateDetection,
        AnalysisType::DuplicateDetection
    );
    assert_ne!(
        AnalysisType::DuplicateDetection,
        AnalysisType::DeadCodeAnalysis
    );
}

#[test]
fn test_analysis_type_clone() {
    let t = AnalysisType::ComplexityMetrics;
    let cloned = t;
    assert_eq!(t, cloned);
}

#[test]
fn test_analysis_type_debug() {
    let t = AnalysisType::DependencyGraph;
    let debug_str = format!("{:?}", t);
    assert!(debug_str.contains("DependencyGraph"));
}

#[test]
fn test_analysis_type_all_variants() {
    let types = vec![
        AnalysisType::DuplicateDetection,
        AnalysisType::DeadCodeAnalysis,
        AnalysisType::ComplexityMetrics,
        AnalysisType::DependencyGraph,
        AnalysisType::DefectPrediction,
        AnalysisType::NameSimilarity,
    ];
    assert_eq!(types.len(), 6);
}

// === AnalysisRequest tests ===

#[test]
fn test_analysis_request_creation() {
    let req = AnalysisRequest {
        project_path: "/test/path".to_string(),
        analysis_types: vec![AnalysisType::DuplicateDetection],
        include_patterns: vec!["*.rs".to_string()],
        exclude_patterns: vec!["target/".to_string()],
        max_depth: Some(5),
        parallel: true,
    };

    assert_eq!(req.project_path, "/test/path");
    assert_eq!(req.analysis_types.len(), 1);
    assert!(req.parallel);
}

#[test]
fn test_analysis_request_clone() {
    let req = AnalysisRequest {
        project_path: "/test".to_string(),
        analysis_types: vec![AnalysisType::DeadCodeAnalysis],
        include_patterns: vec![],
        exclude_patterns: vec![],
        max_depth: None,
        parallel: false,
    };

    let cloned = req.clone();
    assert_eq!(req.project_path, cloned.project_path);
    assert_eq!(req.analysis_types, cloned.analysis_types);
}

#[test]
fn test_analysis_request_cache_key_different_paths() {
    let req1 = AnalysisRequest {
        project_path: "/path/one".to_string(),
        analysis_types: vec![AnalysisType::DuplicateDetection],
        include_patterns: vec![],
        exclude_patterns: vec![],
        max_depth: None,
        parallel: true,
    };

    let req2 = AnalysisRequest {
        project_path: "/path/two".to_string(),
        analysis_types: vec![AnalysisType::DuplicateDetection],
        include_patterns: vec![],
        exclude_patterns: vec![],
        max_depth: None,
        parallel: true,
    };

    assert_ne!(req1.cache_key(), req2.cache_key());
}

#[test]
fn test_analysis_request_cache_key_different_types() {
    let req1 = AnalysisRequest {
        project_path: "/test".to_string(),
        analysis_types: vec![AnalysisType::DuplicateDetection],
        include_patterns: vec![],
        exclude_patterns: vec![],
        max_depth: None,
        parallel: true,
    };

    let req2 = AnalysisRequest {
        project_path: "/test".to_string(),
        analysis_types: vec![AnalysisType::DeadCodeAnalysis],
        include_patterns: vec![],
        exclude_patterns: vec![],
        max_depth: None,
        parallel: true,
    };

    assert_ne!(req1.cache_key(), req2.cache_key());
}

#[test]
fn test_analysis_request_cache_key_length() {
    let req = AnalysisRequest {
        project_path: "/test".to_string(),
        analysis_types: vec![],
        include_patterns: vec![],
        exclude_patterns: vec![],
        max_depth: None,
        parallel: false,
    };

    let key = req.cache_key();
    assert_eq!(key.len(), 64); // SHA256 hex = 64 chars
}
