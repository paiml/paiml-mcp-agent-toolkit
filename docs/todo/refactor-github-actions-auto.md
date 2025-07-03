# Specification: `pmat refactor auto --github-action`

## Executive Summary

This specification defines a deterministic, incremental refactoring system for GitHub Actions workflows that leverages AST-based transformations, semantic patching, and selective LLM assistance. The architecture prioritizes correctness through reversible transformations, minimizes API calls through content-addressed caching, and maintains workflow semantics through incremental validation.

## System Architecture

### Core Components

```rust
pub struct GitHubActionRefactorer {
    /// Deterministic AST transformations operating on parsed YAML structure
    syntax_transformer: SyntaxTransformer,
    
    /// Context-aware semantic patches with precondition predicates
    semantic_patcher: SemanticPatcher,
    
    /// LLM advisor for ambiguous transformations requiring domain knowledge
    llm_advisor: Option<LLMAdvisor>,
    
    /// Content-addressed cache with probabilistic eviction
    cache: Arc<HighPerformanceCache>,
    
    /// Rate-limited GitHub API client with exponential backoff
    github: Arc<RateLimitedClient>,
    
    /// Incremental workflow validator using actionlint and custom rules
    validator: WorkflowValidator,
}
```

### Architectural Principles

1. **Separation of Concerns**: Deterministic transformations are isolated from probabilistic inference
2. **Fail-Fast Semantics**: Each transformation is validated incrementally with rollback capability
3. **Cache-Oblivious Design**: All GitHub API operations are cached with content-addressed keys
4. **Concurrent I/O**: Diagnostic phase leverages `tokio::join!` for parallel API calls
5. **Type Safety**: External process invocation replaced with structured API clients

## Data Models

### Workflow Representation

```rust
/// Immutable, structural sharing AST representation of YAML workflows
pub struct YamlDocument {
    root: Arc<YamlNode>,
    version: u64, // For optimistic concurrency control
}

pub enum YamlNode {
    Mapping(Arc<IndexMap<String, YamlNode>>),
    Sequence(Arc<Vec<YamlNode>>),
    Scalar(String),
    Null,
}

impl YamlDocument {
    /// Zero-copy path queries using JSONPath-like syntax
    pub fn find_all(&self, path: &str) -> impl Iterator<Item = &YamlNode> {
        PathQuery::parse(path).execute(&self.root)
    }
    
    /// Create checkpoint for rollback semantics
    pub fn checkpoint(&self) -> DocumentCheckpoint {
        DocumentCheckpoint {
            root: Arc::clone(&self.root),
            version: self.version,
        }
    }
}
```

### Diagnostic Context

```rust
pub struct DiagnosticContext {
    /// Parsed failure information with surrounding context
    pub failure: WorkflowFailure,
    
    /// Parsed workflow document
    pub workflow: YamlDocument,
    
    /// Project context from `pmat context` analysis
    pub project_context: ProjectContext,
    
    /// GitHub API rate limit budget
    pub api_budget: u32,
    
    /// Optional run metadata for advanced diagnostics
    pub metadata: Option<RunMetadata>,
}

pub struct WorkflowFailure {
    pub job_name: String,
    pub step_index: usize,
    pub error_message: String,
    pub context_lines: Vec<String>, // ±50 lines around failure
    pub failure_type: FailureType,
}

pub enum FailureType {
    SyntaxError { line: usize, column: usize },
    ActionNotFound { action: String },
    ScriptError { exit_code: i32 },
    DependencyFailure { missing: String },
    PermissionDenied { resource: String },
    Unknown,
}
```

## Transformation Pipeline

### Phase 1: Concurrent Diagnostics

```rust
impl GitHubActionRefactorer {
    pub async fn diagnose(&self, run_url: &Url) -> Result<DiagnosticContext> {
        let (owner, repo, run_id) = self.github.parse_run_url(run_url)?;
        
        // Parallel I/O with structured concurrency
        let ((logs, failure), workflow, project_context, api_limits) = tokio::try_join!(
            // Group related operations
            async {
                let logs = self.github.fetch_logs(&owner, &repo, run_id).await?;
                let failure = FailureParser::new()
                    .with_context_window(50)
                    .parse(&logs)?;
                Ok::<_, AppError>((logs, failure))
            },
            self.github.fetch_workflow_content(&owner, &repo, run_id),
            self.analyze_project_context(&owner, &repo),
            self.github.get_rate_limit_status()
        )?;
        
        Ok(DiagnosticContext {
            failure,
            workflow: YamlDocument::parse(&workflow)?,
            project_context,
            api_budget: api_limits.remaining,
            metadata: None, // Populated on-demand
        })
    }
}
```

### Phase 2: Deterministic Transformations

```rust
pub trait TransformRule: Send + Sync {
    fn name(&self) -> &'static str;
    fn apply(&self, doc: &mut YamlDocument) -> Option<AppliedTransformation>;
}

pub struct AppliedTransformation {
    pub rule: &'static str,
    pub changes: Vec<Change>,
    pub reversible: bool,
}

/// Example: SHA-based version pinning with caching
pub struct VersionPinningRule {
    cache: Arc<HighPerformanceCache>,
    github: Arc<RateLimitedClient>,
}

impl TransformRule for VersionPinningRule {
    fn name(&self) -> &'static str { "version-pinning" }
    
    fn apply(&self, doc: &mut YamlDocument) -> Option<AppliedTransformation> {
        let mut changes = Vec::new();
        let runtime = tokio::runtime::Handle::current();
        
        for uses_node in doc.find_all_mut("jobs.*.steps[*].uses") {
            if let YamlNode::Scalar(ref action_ref) = uses_node {
                if let Some(unpinned) = ActionReference::parse(action_ref) {
                    if unpinned.is_floating_tag() {
                        // Block on async operation within sync context
                        match runtime.block_on(self.resolve_to_sha(&unpinned)) {
                            Ok(sha) => {
                                let pinned = format!("{}@{}", unpinned.repository, sha);
                                *uses_node = YamlNode::Scalar(pinned.clone());
                                changes.push(Change::VersionPinned {
                                    from: action_ref.clone(),
                                    to: pinned,
                                });
                            }
                            Err(e) => {
                                tracing::warn!("Failed to resolve {}: {}", action_ref, e);
                            }
                        }
                    }
                }
            }
        }
        
        (!changes.is_empty()).then(|| AppliedTransformation {
            rule: self.name(),
            changes,
            reversible: true,
        })
    }
}

impl VersionPinningRule {
    async fn resolve_to_sha(&self, action: &ActionReference) -> Result<String> {
        let cache_key = format!("sha:{}@{}", action.repository, action.ref_spec);
        
        self.cache.get_or_compute(cache_key.as_bytes(), || async {
            // Use GraphQL for efficient batch resolution
            let query = r#"
                query ResolveRef($owner: String!, $name: String!, $ref: String!) {
                    repository(owner: $owner, name: $name) {
                        object(expression: $ref) {
                            ... on Commit { oid }
                            ... on Tag {
                                target {
                                    ... on Commit { oid }
                                }
                            }
                        }
                    }
                }
            "#;
            
            let (owner, name) = action.repository.split_once('/')
                .ok_or_else(|| anyhow!("Invalid repository format"))?;
            
            let variables = json!({
                "owner": owner,
                "name": name,
                "ref": action.ref_spec,
            });
            
            let response = self.github.graphql(query, variables).await?;
            let sha = response["data"]["repository"]["object"]["oid"]
                .as_str()
                .or_else(|| response["data"]["repository"]["object"]["target"]["oid"].as_str())
                .ok_or_else(|| anyhow!("Failed to extract SHA from GraphQL response"))?;
            
            Ok(CacheEntry::new(
                sha.as_bytes().to_vec(),
                Duration::from_secs(3600), // 1-hour TTL
            ))
        }).await.map(|entry| String::from_utf8(entry.data).unwrap())
    }
}
```

### Phase 3: Semantic Patching

```rust
pub struct SemanticPatch {
    pub name: &'static str,
    pub precondition: Box<dyn Fn(&DiagnosticContext) -> bool + Send + Sync>,
    pub transform: Box<dyn Fn(&mut YamlDocument, &DiagnosticContext) -> Result<Vec<Change>> + Send + Sync>,
}

impl SemanticPatcher {
    pub fn build_patch_set(&self) -> Vec<SemanticPatch> {
        vec![
            // Rust-specific caching optimization
            SemanticPatch {
                name: "rust-cache-injection",
                precondition: Box::new(|ctx| {
                    ctx.project_context.primary_language == Language::Rust
                        && !ctx.workflow.contains_path("jobs.*.steps[*].uses", "actions/cache@*")
                        && !ctx.workflow.contains_path("jobs.*.steps[*].uses", "Swatinem/rust-cache@*")
                }),
                transform: Box::new(|doc, _ctx| {
                    let cache_step = YamlNode::from_str(r#"
                        name: Cache cargo artifacts
                        uses: Swatinem/rust-cache@v2
                        with:
                          cache-on-failure: true
                    "#)?;
                    
                    // Insert after checkout step
                    doc.insert_after("jobs.*.steps[?(@.uses =~ /actions\\/checkout.*/)]", cache_step)?;
                    
                    Ok(vec![Change::StepInserted {
                        job: "all",
                        position: "after-checkout",
                        description: "Rust dependency caching",
                    }])
                }),
            },
            
            // Security: Minimal permission scoping
            SemanticPatch {
                name: "permission-minimization",
                precondition: Box::new(|ctx| {
                    ctx.workflow.get_permissions().map_or(true, |perms| perms.is_write_all())
                }),
                transform: Box::new(|doc, ctx| {
                    let required = analyze_required_permissions(&ctx.workflow)?;
                    doc.set_root_permissions(required)?;
                    
                    Ok(vec![Change::PermissionsScoped {
                        from: "write-all",
                        to: format!("{:?}", required),
                    }])
                }),
            },
        ]
    }
}
```

### Phase 4: Incremental Application

```rust
impl IncrementalRefactorer {
    pub async fn refactor(&self, mut ctx: DiagnosticContext) -> Result<RefactoredWorkflow> {
        let mut workflow = ctx.workflow.clone();
        let mut applied_changes = Vec::new();
        let mut validation_cache = ValidationCache::new();
        
        // Phase 1: Apply deterministic transformations
        for rule in &self.transformer.rules {
            if let Some(transformation) = rule.apply(&mut workflow) {
                // Incremental validation with caching
                match self.validator.validate_transformation(&workflow, &transformation, &mut validation_cache).await {
                    Ok(ValidationResult::Valid) => {
                        applied_changes.push(transformation);
                    }
                    Ok(ValidationResult::Invalid(reasons)) => {
                        tracing::info!("Skipping {} due to: {:?}", rule.name(), reasons);
                        workflow.rollback(&transformation)?;
                    }
                    Err(e) => {
                        tracing::warn!("Validation error for {}: {}", rule.name(), e);
                        workflow.rollback(&transformation)?;
                    }
                }
            }
        }
        
        // Phase 2: Apply semantic patches
        for patch in self.patcher.build_patch_set() {
            if !(patch.precondition)(&ctx) {
                continue;
            }
            
            let checkpoint = workflow.checkpoint();
            match (patch.transform)(&mut workflow, &ctx) {
                Ok(changes) if !changes.is_empty() => {
                    if self.validator.validate_workflow(&workflow).await?.is_valid() {
                        applied_changes.push(AppliedTransformation {
                            rule: patch.name,
                            changes,
                            reversible: true,
                        });
                    } else {
                        workflow.restore(checkpoint)?;
                    }
                }
                Ok(_) => {} // No changes needed
                Err(e) => {
                    tracing::warn!("Patch {} failed: {}", patch.name, e);
                    workflow.restore(checkpoint)?;
                }
            }
        }
        
        // Phase 3: Check if primary failure is resolved
        let failure_resolved = self.validator
            .simulate_workflow_execution(&workflow, &ctx.failure)
            .await?
            .is_success();
        
        // Phase 4: LLM assistance only if necessary
        if !failure_resolved && self.llm_advisor.is_some() {
            let advisor = self.llm_advisor.as_ref().unwrap();
            
            // Highly constrained prompt focusing only on the specific failure
            let suggestion = advisor.suggest_minimal_fix(
                &ctx.failure,
                &workflow,
                &applied_changes,
            ).await?;
            
            // Apply suggestion with validation
            if let Some(transformation) = suggestion.to_transformation() {
                let checkpoint = workflow.checkpoint();
                workflow.apply_transformation(&transformation)?;
                
                if self.validator.validate_workflow(&workflow).await?.is_valid() {
                    applied_changes.push(transformation);
                } else {
                    workflow.restore(checkpoint)?;
                }
            }
        }
        
        // Generate comprehensive report
        let validation_report = self.validator.generate_report(&workflow).await?;
        
        Ok(RefactoredWorkflow {
            content: workflow.to_yaml()?,
            applied_changes,
            validation_report,
            metrics: RefactoringMetrics {
                deterministic_fixes: applied_changes.iter()
                    .filter(|t| !t.rule.starts_with("llm-"))
                    .count(),
                semantic_patches: applied_changes.iter()
                    .filter(|t| t.rule.contains("patch"))
                    .count(),
                llm_assists: applied_changes.iter()
                    .filter(|t| t.rule.starts_with("llm-"))
                    .count(),
                api_calls_saved: validation_cache.cache_hits(),
                total_duration: ctx.start_time.elapsed(),
            },
        })
    }
}
```

## Performance Architecture

### High-Performance Cache

```rust
pub struct HighPerformanceCache {
    /// Sharded to reduce lock contention
    shards: [Arc<DashMap<Blake3Hash, CacheEntry>>; 16],
    
    /// Global statistics for monitoring
    stats: Arc<CacheStats>,
    
    /// Background task for TTL enforcement
    ttl_enforcer: JoinHandle<()>,
}

#[derive(Clone)]
pub struct CacheEntry {
    pub data: Arc<[u8]>,
    pub expires_at: Instant,
    pub hit_count: AtomicU32,
}

impl HighPerformanceCache {
    pub async fn get_or_compute<F, Fut>(&self, key: &[u8], compute: F) -> Result<CacheEntry>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<CacheEntry>>,
    {
        let hash = blake3::hash(key);
        let shard_idx = (hash.as_bytes()[0] & 0x0F) as usize;
        let shard = &self.shards[shard_idx];
        
        // Fast path: cache hit
        if let Some(entry) = shard.get(&hash) {
            if entry.expires_at > Instant::now() {
                entry.hit_count.fetch_add(1, Ordering::Relaxed);
                self.stats.hits.fetch_add(1, Ordering::Relaxed);
                return Ok(entry.clone());
            }
        }
        
        // Slow path: compute and cache
        self.stats.misses.fetch_add(1, Ordering::Relaxed);
        
        // Use async-aware mutex to prevent thundering herd
        let _lock = self.computation_locks[shard_idx].lock().await;
        
        // Double-check after acquiring lock
        if let Some(entry) = shard.get(&hash) {
            if entry.expires_at > Instant::now() {
                return Ok(entry.clone());
            }
        }
        
        let entry = compute().await?;
        
        // Probabilistic eviction with LFU bias
        if shard.len() >= SHARD_CAPACITY {
            self.evict_lfu_entry(shard).await;
        }
        
        shard.insert(hash, entry.clone());
        Ok(entry)
    }
    
    async fn evict_lfu_entry(&self, shard: &DashMap<Blake3Hash, CacheEntry>) {
        // Sample k entries and evict the least frequently used
        const SAMPLE_SIZE: usize = 5;
        
        let sample: Vec<_> = shard.iter()
            .take(SAMPLE_SIZE)
            .map(|entry| (entry.key().clone(), entry.hit_count.load(Ordering::Relaxed)))
            .collect();
        
        if let Some((victim_key, _)) = sample.iter().min_by_key(|(_, hits)| *hits) {
            shard.remove(victim_key);
            self.stats.evictions.fetch_add(1, Ordering::Relaxed);
        }
    }
}
```

### Rate-Limited GitHub Client

```rust
pub struct RateLimitedClient {
    client: Octocrab,
    rate_limiter: Arc<TokenBucket>,
    retry_policy: RetryPolicy,
}

impl RateLimitedClient {
    pub async fn graphql<T: DeserializeOwned>(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<T> {
        self.rate_limiter.acquire(1).await?;
        
        let request = self.client
            .graphql(query)
            .variables(variables);
        
        // Exponential backoff with jitter
        retry_with_policy(&self.retry_policy, || async {
            match request.send().await {
                Ok(response) => Ok(response),
                Err(e) if e.is_rate_limit() => {
                    let retry_after = e.retry_after_seconds().unwrap_or(60);
                    Err(RetryableError::RateLimit { retry_after })
                }
                Err(e) => Err(RetryableError::Permanent(e.into())),
            }
        }).await
    }
}
```

## Testing Strategy

### Property-Based Testing

```rust
#[cfg(test)]
mod properties {
    use proptest::prelude::*;
    
    proptest! {
        /// Transformations preserve workflow semantics
        #[test]
        fn transformations_preserve_job_structure(
            workflow in arb_valid_workflow()
        ) {
            let original_jobs = extract_job_names(&workflow);
            let mut transformed = workflow.clone();
            
            let refactorer = create_test_refactorer();
            refactorer.transformer.transform(&mut transformed);
            
            let transformed_jobs = extract_job_names(&transformed);
            
            prop_assert_eq!(
                original_jobs, 
                transformed_jobs,
                "Job structure must be preserved"
            );
        }
        
        /// Version pinning is deterministic
        #[test]
        fn version_pinning_is_deterministic(
            workflow in arb_workflow_with_actions(),
            seed in 0u64..u64::MAX
        ) {
            let rule = VersionPinningRule::new_with_mock_resolver(seed);
            
            let mut w1 = workflow.clone();
            let mut w2 = workflow.clone();
            
            rule.apply(&mut w1);
            rule.apply(&mut w2);
            
            prop_assert_eq!(w1, w2, "Same input must produce same output");
        }
    }
}
```

### Regression Corpus

```rust
#[tokio::test]
async fn test_known_failure_patterns() {
    let test_cases = [
        ("node_version_mismatch", include_str!("../corpus/node_version_mismatch.yml")),
        ("missing_permissions", include_str!("../corpus/missing_permissions.yml")),
        ("deprecated_actions", include_str!("../corpus/deprecated_actions.yml")),
        ("cache_key_collision", include_str!("../corpus/cache_key_collision.yml")),
    ];
    
    let refactorer = create_production_refactorer();
    
    for (name, workflow_yaml) in test_cases {
        let mut workflow = YamlDocument::parse(workflow_yaml)
            .expect(&format!("Failed to parse {}", name));
        
        let ctx = create_mock_context(&workflow);
        let result = refactorer.refactor(ctx).await
            .expect(&format!("Refactoring failed for {}", name));
        
        // Validate the refactored workflow
        assert!(
            result.validation_report.is_valid(),
            "Refactored {} failed validation: {:?}",
            name,
            result.validation_report
        );
        
        // Ensure at least one meaningful change was made
        assert!(
            !result.applied_changes.is_empty(),
            "No changes applied to {}",
            name
        );
    }
}
```

### Performance Benchmarks

```rust
#[bench]
fn bench_yaml_parsing(b: &mut Bencher) {
    let yaml = include_str!("../benches/large_workflow.yml");
    b.iter(|| {
        YamlDocument::parse(black_box(yaml)).unwrap()
    });
}

#[bench]
fn bench_transformation_pipeline(b: &mut Bencher) {
    let workflow = YamlDocument::parse(include_str!("../benches/complex_workflow.yml")).unwrap();
    let transformer = SyntaxTransformer::default();
    
    b.iter(|| {
        let mut work = workflow.clone();
        transformer.transform(black_box(&mut work))
    });
}

#[tokio::test]
async fn bench_cache_performance() {
    let cache = HighPerformanceCache::new();
    let keys: Vec<_> = (0..10000)
        .map(|i| format!("key-{}", i).into_bytes())
        .collect();
    
    // Measure cache performance under concurrent load
    let start = Instant::now();
    
    let handles: Vec<_> = keys.iter()
        .cycle()
        .take(100_000)
        .map(|key| {
            let cache = cache.clone();
            let key = key.clone();
            tokio::spawn(async move {
                cache.get_or_compute(&key, || async {
                    Ok(CacheEntry::new(vec![42; 100], Duration::from_secs(60)))
                }).await
            })
        })
        .collect();
    
    futures::future::join_all(handles).await;
    
    let elapsed = start.elapsed();
    let ops_per_sec = 100_000.0 / elapsed.as_secs_f64();
    
    assert!(ops_per_sec > 50_000.0, "Cache throughput below threshold: {:.0} ops/sec", ops_per_sec);
}
```

## CLI Interface

```bash
# Basic usage
pmat refactor auto --github-action <URL>

# Dry run with detailed change preview
pmat refactor auto --github-action <URL> --dry-run

# Apply only deterministic transformations
pmat refactor auto --github-action <URL> --deterministic-only

# Use specific rule categories
pmat refactor auto --github-action <URL> --rules security,performance,caching

# Output to specific file
pmat refactor auto --github-action <URL> --output workflow-fixed.yml

# JSON output with metrics
pmat refactor auto --github-action <URL> --format json --metrics

# Use custom cache directory
pmat refactor auto --github-action <URL> --cache-dir ~/.pmat/cache

# Verbose logging for debugging
pmat refactor auto --github-action <URL> -vvv

# Use specific GitHub token
pmat refactor auto --github-action <URL> --github-token $GITHUB_TOKEN
```

## Success Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| **P99 Latency (Deterministic)** | < 3s | Time excluding API calls and LLM |
| **P99 Latency (End-to-End)** | < 10s | Total time including all operations |
| **Deterministic Resolution Rate** | > 75% | Failures fixed without LLM |
| **Incremental Success Rate** | > 95% | Runs producing valid improvements |
| **Cache Hit Rate** | > 85% | GitHub API cache effectiveness |
| **Memory Usage (RSS)** | < 150MB | Peak memory during operation |
| **Transformation Correctness** | 100% | No workflow breakage |
| **API Rate Limit Compliance** | 100% | No 429 errors |

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum RefactoringError {
    #[error("Failed to parse GitHub URL: {0}")]
    InvalidUrl(String),
    
    #[error("GitHub API error: {0}")]
    GitHubApi(#[from] octocrab::Error),
    
    #[error("Workflow parsing failed: {0}")]
    YamlParse(#[from] serde_yaml::Error),
    
    #[error("Validation failed: {0}")]
    Validation(ValidationError),
    
    #[error("All transformations failed to improve workflow")]
    NoImprovementPossible,
    
    #[error("Rate limit exceeded, retry after {retry_after}s")]
    RateLimitExceeded { retry_after: u64 },
}

impl RefactoringError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, 
            Self::GitHubApi(_) | 
            Self::RateLimitExceeded { .. }
        )
    }
}
```

## Integration Requirements

### Prerequisites

- `actionlint` binary in PATH for workflow validation
- GitHub token with `repo` and `actions` scopes
- `pmat` installation for project context analysis

### Configuration

```toml
# ~/.config/pmat/refactor.toml
[github]
token_env = "GITHUB_TOKEN"
max_retries = 3
timeout_seconds = 30

[cache]
directory = "~/.cache/pmat/github"
max_size_mb = 500
ttl_hours = 24

[rules]
enabled = ["security", "performance", "best-practices"]
disabled = []

[llm]
provider = "openai"
model = "gpt-4-turbo"
max_tokens = 500
temperature = 0.3
```
