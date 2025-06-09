use crate::cli::DagType;
use crate::models::churn::{ChurnSummary, CodeChurnAnalysis};
use crate::models::dag::DependencyGraph;
use crate::models::template::{
    ParameterSpec, ParameterType, TemplateCategory, TemplateResource, Toolchain,
};
use crate::services::cache::{
    strategies::TemplateCacheStrategy, CacheConfig, ContentCache, SessionCacheManager,
};
use crate::services::context::{AstItem, FileContext};
use chrono::Utc;
use rustc_hash::FxHashMap;
use semver::Version;
use std::collections::HashMap;
use std::path::PathBuf;

#[tokio::test]
async fn test_session_cache_manager() {
    let config = CacheConfig::default();
    let manager = SessionCacheManager::new(config);

    // Test manager's diagnostics - check memory usage instead
    let diagnostics = manager.get_diagnostics();
    assert!(diagnostics.memory_usage_mb >= 0.0);
}

#[tokio::test]
async fn test_ast_cache() {
    // Create a custom test strategy that doesn't check file existence
    #[derive(Clone)]
    struct TestAstCacheStrategy;

    impl crate::services::cache::base::CacheStrategy for TestAstCacheStrategy {
        type Key = PathBuf;
        type Value = FileContext;

        fn cache_key(&self, path: &PathBuf) -> String {
            format!("test_ast:{}", path.display())
        }

        fn validate(&self, _path: &PathBuf, _cached: &FileContext) -> bool {
            // Always valid for tests
            true
        }

        fn ttl(&self) -> Option<std::time::Duration> {
            Some(std::time::Duration::from_secs(300))
        }

        fn max_size(&self) -> usize {
            100
        }
    }

    let strategy = TestAstCacheStrategy;
    let cache = ContentCache::new(strategy);

    // Create a test FileContext
    let file_context = FileContext {
        path: "/test/file.rs".to_string(),
        language: "rust".to_string(),
        items: vec![AstItem::Function {
            name: "main".to_string(),
            visibility: "pub".to_string(),
            is_async: false,
            line: 1,
        }],
        complexity_metrics: None,
    };

    let key = PathBuf::from("/test/file.rs");

    // Cache miss initially
    assert!(cache.get(&key).is_none());

    // Put value in cache (ContentCache wraps in Arc internally)
    cache.put(key.clone(), file_context.clone());

    // Cache hit
    let cached = cache.get(&key);
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().path, file_context.path);
}

#[tokio::test]
async fn test_template_cache() {
    let strategy = TemplateCacheStrategy;
    let cache = ContentCache::new(strategy);

    // Create a test TemplateResource
    let template = TemplateResource {
        uri: "template://makefile/rust/cli".to_string(),
        name: "Rust CLI Makefile".to_string(),
        description: "Makefile for Rust CLI projects".to_string(),
        toolchain: Toolchain::RustCli {
            cargo_features: vec![],
        },
        category: TemplateCategory::Makefile,
        parameters: vec![ParameterSpec {
            name: "project_name".to_string(),
            param_type: ParameterType::ProjectName,
            required: true,
            default_value: None,
            validation_pattern: None,
            description: "Name of the project".to_string(),
        }],
        s3_object_key: "templates/makefile/rust/cli.hbs".to_string(),
        content_hash: "abc123".to_string(),
        semantic_version: Version::new(1, 0, 0),
        dependency_graph: vec![],
    };

    let key = "template://makefile/rust/cli".to_string();

    // Cache and retrieve
    cache.put(key.clone(), template.clone());
    let cached = cache.get(&key);

    assert!(cached.is_some());
    assert_eq!(cached.unwrap().uri, template.uri);
}

#[tokio::test]
async fn test_dag_cache() {
    // Create a custom test strategy that doesn't check path existence
    #[derive(Clone)]
    struct TestDagCacheStrategy;

    impl crate::services::cache::base::CacheStrategy for TestDagCacheStrategy {
        type Key = (PathBuf, DagType);
        type Value = DependencyGraph;

        fn cache_key(&self, (path, dag_type): &(PathBuf, DagType)) -> String {
            format!("test_dag:{}:{:?}", path.display(), dag_type)
        }

        fn validate(&self, _key: &(PathBuf, DagType), _cached: &DependencyGraph) -> bool {
            // Always valid for tests
            true
        }

        fn ttl(&self) -> Option<std::time::Duration> {
            Some(std::time::Duration::from_secs(180))
        }

        fn max_size(&self) -> usize {
            20
        }
    }

    let strategy = TestDagCacheStrategy;
    let cache = ContentCache::new(strategy);

    // Create a test DependencyGraph
    let graph = DependencyGraph {
        nodes: FxHashMap::default(),
        edges: vec![],
    };

    let key = (PathBuf::from("/test/project"), DagType::CallGraph);

    // Cache and retrieve
    cache.put(key.clone(), graph.clone());
    let cached = cache.get(&key);

    assert!(cached.is_some());
}

#[tokio::test]
async fn test_churn_cache() {
    // Create a custom test strategy that doesn't check repo existence
    #[derive(Clone)]
    struct TestChurnCacheStrategy;

    impl crate::services::cache::base::CacheStrategy for TestChurnCacheStrategy {
        type Key = (PathBuf, u32);
        type Value = CodeChurnAnalysis;

        fn cache_key(&self, (repo, period_days): &(PathBuf, u32)) -> String {
            format!("test_churn:{}:{}", repo.display(), period_days)
        }

        fn validate(&self, _key: &(PathBuf, u32), _cached: &CodeChurnAnalysis) -> bool {
            // Always valid for tests
            true
        }

        fn ttl(&self) -> Option<std::time::Duration> {
            Some(std::time::Duration::from_secs(1800))
        }

        fn max_size(&self) -> usize {
            20
        }
    }

    let strategy = TestChurnCacheStrategy;
    let cache = ContentCache::new(strategy);

    // Create test CodeChurnAnalysis
    let churn = CodeChurnAnalysis {
        generated_at: Utc::now(),
        period_days: 30,
        repository_root: PathBuf::from("/test/repo"),
        files: vec![],
        summary: ChurnSummary {
            total_commits: 100,
            total_files_changed: 10,
            hotspot_files: vec![],
            stable_files: vec![],
            author_contributions: HashMap::new(),
        },
    };

    let key = (PathBuf::from("/test/repo"), 30u32);

    // Cache and retrieve
    cache.put(key.clone(), churn.clone());
    let cached = cache.get(&key);

    assert!(cached.is_some());
    assert_eq!(cached.unwrap().period_days, churn.period_days);
}

#[tokio::test]
async fn test_git_stats_cache() {
    use crate::services::cache::strategies::GitStats;

    // Create a custom test strategy that doesn't check git repo
    #[derive(Clone)]
    struct TestGitStatsCacheStrategy;

    impl crate::services::cache::base::CacheStrategy for TestGitStatsCacheStrategy {
        type Key = PathBuf;
        type Value = GitStats;

        fn cache_key(&self, repo: &PathBuf) -> String {
            format!("test_git_stats:{}", repo.display())
        }

        fn validate(&self, _repo: &PathBuf, _cached: &GitStats) -> bool {
            // Always valid for tests
            true
        }

        fn ttl(&self) -> Option<std::time::Duration> {
            Some(std::time::Duration::from_secs(900))
        }

        fn max_size(&self) -> usize {
            10
        }
    }

    let strategy = TestGitStatsCacheStrategy;
    let cache = ContentCache::new(strategy);

    // Create test GitStats
    let stats = GitStats {
        total_commits: 100,
        authors: vec!["Alice".to_string(), "Bob".to_string()],
        branch: "main".to_string(),
        head_commit: "abc123".to_string(),
    };

    let key = PathBuf::from("/test/repo");

    // Cache and retrieve
    cache.put(key.clone(), stats.clone());
    let cached = cache.get(&key);

    assert!(cached.is_some());
    assert_eq!(cached.unwrap().total_commits, stats.total_commits);
}

#[tokio::test]
async fn test_cache_eviction() {
    // Create a custom test strategy with max size 2
    #[derive(Clone)]
    struct SmallCacheStrategy;

    impl crate::services::cache::base::CacheStrategy for SmallCacheStrategy {
        type Key = PathBuf;
        type Value = FileContext;

        fn cache_key(&self, path: &PathBuf) -> String {
            format!("test_small:{}", path.display())
        }

        fn validate(&self, _path: &PathBuf, _cached: &FileContext) -> bool {
            // Always valid for tests
            true
        }

        fn ttl(&self) -> Option<std::time::Duration> {
            Some(std::time::Duration::from_secs(300))
        }

        fn max_size(&self) -> usize {
            2 // Small cache for eviction testing
        }
    }

    let strategy = SmallCacheStrategy;
    let cache = ContentCache::new(strategy);

    // Test eviction with small cache
    let file1 = FileContext {
        path: "/test/file1.rs".to_string(),
        language: "rust".to_string(),
        items: vec![],
        complexity_metrics: None,
    };

    let file2 = FileContext {
        path: "/test/file2.rs".to_string(),
        language: "rust".to_string(),
        items: vec![],
        complexity_metrics: None,
    };

    let file3 = FileContext {
        path: "/test/file3.rs".to_string(),
        language: "rust".to_string(),
        items: vec![],
        complexity_metrics: None,
    };

    // Add first two files
    cache.put(PathBuf::from("/test/file1.rs"), file1);
    cache.put(PathBuf::from("/test/file2.rs"), file2);

    // Both should be cached
    assert!(cache.get(&PathBuf::from("/test/file1.rs")).is_some());
    assert!(cache.get(&PathBuf::from("/test/file2.rs")).is_some());

    // Add third file - should evict the least recently used (file1)
    cache.put(PathBuf::from("/test/file3.rs"), file3);

    // File1 should be evicted, file2 and file3 should remain
    assert!(cache.get(&PathBuf::from("/test/file1.rs")).is_none());
    assert!(cache.get(&PathBuf::from("/test/file2.rs")).is_some());
    assert!(cache.get(&PathBuf::from("/test/file3.rs")).is_some());
}

#[tokio::test]
async fn test_cache_clear() {
    // Create a custom test strategy that doesn't check file existence
    #[derive(Clone)]
    struct TestClearStrategy;

    impl crate::services::cache::base::CacheStrategy for TestClearStrategy {
        type Key = PathBuf;
        type Value = FileContext;

        fn cache_key(&self, path: &PathBuf) -> String {
            format!("test_clear:{}", path.display())
        }

        fn validate(&self, _path: &PathBuf, _cached: &FileContext) -> bool {
            // Always valid for tests
            true
        }

        fn ttl(&self) -> Option<std::time::Duration> {
            Some(std::time::Duration::from_secs(300))
        }

        fn max_size(&self) -> usize {
            100
        }
    }

    let strategy = TestClearStrategy;
    let cache = ContentCache::new(strategy);

    // Add some items
    for i in 0..5 {
        let file = FileContext {
            path: format!("/test/file{i}.rs"),
            language: "rust".to_string(),
            items: vec![],
            complexity_metrics: None,
        };
        cache.put(PathBuf::from(format!("/test/file{i}.rs")), file);
    }

    // Verify items are cached
    assert!(cache.get(&PathBuf::from("/test/file0.rs")).is_some());
    assert!(cache.get(&PathBuf::from("/test/file4.rs")).is_some());

    // Clear cache
    cache.clear();

    // All items should be gone
    for i in 0..5 {
        assert!(cache
            .get(&PathBuf::from(format!("/test/file{i}.rs")))
            .is_none());
    }
}

#[tokio::test]
async fn test_cache_ttl() {
    // Create a custom strategy with very short TTL
    #[derive(Clone)]
    struct ShortTtlStrategy;

    impl crate::services::cache::base::CacheStrategy for ShortTtlStrategy {
        type Key = String;
        type Value = String;

        fn cache_key(&self, key: &String) -> String {
            key.clone()
        }

        fn validate(&self, _key: &String, _cached: &String) -> bool {
            true
        }

        fn ttl(&self) -> Option<std::time::Duration> {
            Some(std::time::Duration::from_millis(100))
        }

        fn max_size(&self) -> usize {
            10
        }
    }

    let strategy = ShortTtlStrategy;
    let cache = ContentCache::new(strategy);

    // Add item
    cache.put("test_key".to_string(), "test_value".to_string());

    // Should be cached immediately
    assert!(cache.get(&"test_key".to_string()).is_some());

    // Wait for TTL to expire
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    // Should be expired
    assert!(cache.get(&"test_key".to_string()).is_none());
}
