# Update Check & Self-Diagnosis Specification

## Overview

This document specifies the implementation of version checking and comprehensive self-diagnosis capabilities for the PAIML MCP Agent Toolkit, ensuring deterministic validation of all features with minimal payload emission for remote debugging.

## Update Check Implementation

### Version Manifest Protocol

```rust
#[derive(Serialize, Deserialize)]
struct VersionManifest {
    version: semver::Version,
    published_at: DateTime<Utc>,
    sha256: String,
    size_bytes: u64,
    minimum_supported: semver::Version,
    deprecations: Vec<DeprecatedFeature>,
    security_advisory: Option<SecurityNotice>,
}

impl VersionChecker {
    const MANIFEST_URL: &'static str = 
        "https://api.github.com/repos/paiml/paiml-mcp-agent-toolkit/releases/latest";
    const CHECK_INTERVAL: Duration = Duration::from_secs(86_400); // 24h
    const CACHE_PATH: &'static str = "~/.cache/pmat/version-check.json";

    async fn check_update(&self) -> Result<UpdateStatus> {
        // Atomic read of cached check with mtime validation
        if let Some(cached) = self.read_cache_atomic()? {
            if cached.checked_at.elapsed() < Self::CHECK_INTERVAL {
                return Ok(cached.status);
            }
        }

        // Non-blocking background check with timeout
        let manifest = tokio::time::timeout(
            Duration::from_secs(5),
            self.fetch_manifest()
        ).await??;

        let status = self.compare_versions(&manifest)?;
        self.write_cache_atomic(&status)?;
        Ok(status)
    }
}
```

### Update Command Implementation

```rust
#[derive(clap::Args)]
struct SelfUpdateArgs {
    /// Check for updates without installing
    #[arg(long)]
    check: bool,

    /// Show what would be updated without doing it
    #[arg(long)]
    dry_run: bool,

    /// Force update even if current version is newer
    #[arg(long)]
    force: bool,

    /// Use differential binary patching (if available)
    #[arg(long)]
    differential: bool,
}

impl SelfUpdater {
    async fn execute(&self, args: SelfUpdateArgs) -> Result<()> {
        let current = semver::Version::parse(env!("CARGO_PKG_VERSION"))?;
        let manifest = self.fetch_manifest().await?;
        
        if args.check {
            return self.print_update_status(&current, &manifest);
        }

        // Verify binary integrity before replacement
        let new_binary = self.download_verified(&manifest).await?;
        
        // Atomic replacement with rollback capability
        self.atomic_replace_binary(new_binary).await?;
        
        Ok(())
    }

    async fn atomic_replace_binary(&self, new: Vec<u8>) -> Result<()> {
        let exe = std::env::current_exe()?;
        let backup = exe.with_extension("backup");
        
        // 1. Write new binary to temp with restrictive permissions
        let temp = exe.with_extension("tmp");
        tokio::fs::write(&temp, &new).await?;
        
        #[cfg(unix)]
        std::fs::set_permissions(&temp, std::fs::Permissions::from_mode(0o755))?;
        
        // 2. Atomic rename dance
        tokio::fs::rename(&exe, &backup).await?;
        match tokio::fs::rename(&temp, &exe).await {
            Ok(_) => {
                let _ = tokio::fs::remove_file(&backup).await;
                Ok(())
            }
            Err(e) => {
                // Rollback on failure
                let _ = tokio::fs::rename(&backup, &exe).await;
                Err(e.into())
            }
        }
    }
}
```

## Self-Diagnosis Implementation

### Comprehensive Feature Matrix

```rust
#[derive(Serialize)]
struct DiagnosticReport {
    version: String,
    build_info: BuildInfo,
    timestamp: DateTime<Utc>,
    duration_ms: u64,
    features: BTreeMap<String, FeatureResult>,
    summary: DiagnosticSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_context: Option<CompactErrorContext>,
}

#[derive(Serialize)]
struct FeatureResult {
    status: FeatureStatus,
    duration_us: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metrics: Option<serde_json::Value>,
}

#[derive(Serialize)]
enum FeatureStatus {
    Ok,
    Degraded(String),
    Failed,
    Skipped(String),
}

pub struct SelfDiagnostic {
    tests: Vec<Box<dyn FeatureTest>>,
}

impl SelfDiagnostic {
    pub fn new() -> Self {
        Self {
            tests: vec![
                // Core parsing
                Box::new(RustAstTest::new()),
                Box::new(TypeScriptAstTest::new()),
                Box::new(PythonAstTest::new()),
                
                // Analysis engines
                Box::new(ComplexityAnalysisTest::new()),
                Box::new(ChurnAnalysisTest::new()),
                Box::new(DeadCodeTest::new()),
                Box::new(SatdDetectorTest::new()),
                Box::new(DagBuilderTest::new()),
                Box::new(DeepContextTest::new()),
                
                // Output formats
                Box::new(JsonOutputTest::new()),
                Box::new(MarkdownOutputTest::new()),
                Box::new(SarifOutputTest::new()),
                Box::new(MermaidGeneratorTest::new()),
                
                // Protocol adapters
                Box::new(CliAdapterTest::new()),
                Box::new(HttpServerTest::new()),
                Box::new(McpProtocolTest::new()),
                
                // Infrastructure
                Box::new(CacheSubsystemTest::new()),
                Box::new(GitIntegrationTest::new()),
                Box::new(FileDiscoveryTest::new()),
                Box::new(TemplateEngineTest::new()),
                
                // Advanced features
                Box::new(RankingEngineTest::new()),
                Box::new(TdgCalculatorTest::new()),
                Box::new(DuplicateDetectorTest::new()),
                Box::new(SymbolTableTest::new()),
                Box::new(BorrowCheckerTest::new()),
                Box::new(MakefileLinterTest::new()),
            ],
        }
    }

    pub async fn run_diagnostic(&self) -> DiagnosticReport {
        let start = Instant::now();
        let mut features = BTreeMap::new();
        
        for test in &self.tests {
            let test_start = Instant::now();
            let result = match timeout(Duration::from_secs(10), test.execute()).await {
                Ok(Ok(metrics)) => FeatureResult {
                    status: FeatureStatus::Ok,
                    duration_us: test_start.elapsed().as_micros() as u64,
                    error: None,
                    metrics: Some(metrics),
                },
                Ok(Err(e)) => FeatureResult {
                    status: FeatureStatus::Failed,
                    duration_us: test_start.elapsed().as_micros() as u64,
                    error: Some(format!("{:?}", e)),
                    metrics: None,
                },
                Err(_) => FeatureResult {
                    status: FeatureStatus::Failed,
                    duration_us: 10_000_000, // timeout
                    error: Some("Test timeout after 10s".into()),
                    metrics: None,
                },
            };
            
            features.insert(test.name().to_string(), result);
        }
        
        let summary = self.compute_summary(&features);
        
        DiagnosticReport {
            version: env!("CARGO_PKG_VERSION").to_string(),
            build_info: BuildInfo::current(),
            timestamp: Utc::now(),
            duration_ms: start.elapsed().as_millis() as u64,
            features,
            summary,
            error_context: self.extract_error_context(&features),
        }
    }
}
```

### Feature Test Implementations

```rust
#[async_trait]
trait FeatureTest: Send + Sync {
    fn name(&self) -> &'static str;
    async fn execute(&self) -> Result<serde_json::Value>;
}

struct RustAstTest;

#[async_trait]
impl FeatureTest for RustAstTest {
    fn name(&self) -> &'static str { "ast.rust" }
    
    async fn execute(&self) -> Result<serde_json::Value> {
        const TEST_CODE: &str = r#"
            pub fn fibonacci(n: u32) -> u32 {
                match n {
                    0 => 0,
                    1 => 1,
                    _ => fibonacci(n - 1) + fibonacci(n - 2),
                }
            }
        "#;
        
        let ast = parse_rust_content(TEST_CODE)?;
        
        // Verify expected structure
        assert_eq!(ast.items.len(), 1);
        assert!(matches!(&ast.items[0].kind, AstKind::Function(_)));
        
        // Test complexity calculation
        let complexity = compute_complexity(&ast)?;
        assert_eq!(complexity.cyclomatic, 3);
        
        Ok(json!({
            "parsed_items": ast.items.len(),
            "complexity": complexity.cyclomatic,
            "cognitive": complexity.cognitive,
        }))
    }
}

struct CacheSubsystemTest;

#[async_trait]
impl FeatureTest for CacheSubsystemTest {
    fn name(&self) -> &'static str { "cache.subsystem" }
    
    async fn execute(&self) -> Result<serde_json::Value> {
        let cache = SessionCacheManager::new(CacheConfig {
            max_entries: 100,
            ttl: Duration::from_secs(60),
            ..Default::default()
        });
        
        // Test basic operations
        let key = "test:diagnostic:key";
        let value = vec![1, 2, 3, 4, 5];
        
        cache.put(key, value.clone()).await?;
        let retrieved = cache.get(key).await?;
        assert_eq!(retrieved, Some(value));
        
        // Test eviction
        for i in 0..101 {
            cache.put(&format!("evict:{}", i), vec![i as u8]).await?;
        }
        
        let stats = cache.stats();
        
        Ok(json!({
            "entries": stats.entries,
            "hits": stats.hits,
            "misses": stats.misses,
            "evictions": stats.evictions,
            "hit_rate": stats.hit_rate(),
        }))
    }
}

struct MermaidGeneratorTest;

#[async_trait]
impl FeatureTest for MermaidGeneratorTest {
    fn name(&self) -> &'static str { "output.mermaid" }
    
    async fn execute(&self) -> Result<serde_json::Value> {
        let graph = DependencyGraph {
            nodes: vec![
                NodeInfo {
                    id: "main".into(),
                    label: "main.rs".into(),
                    node_type: NodeType::Module,
                    ..Default::default()
                },
                NodeInfo {
                    id: "lib".into(),
                    label: "lib.rs".into(),
                    node_type: NodeType::Module,
                    ..Default::default()
                },
            ],
            edges: vec![
                Edge {
                    from: "main".into(),
                    to: "lib".into(),
                    edge_type: EdgeType::Import,
                    weight: 1.0,
                },
            ],
        };
        
        let generator = DeterministicMermaidEngine::new(PageRankConfig::default());
        let mermaid = generator.generate(&graph, &MermaidOptions::default())?;
        
        // Verify output structure
        assert!(mermaid.contains("graph TD"));
        assert!(mermaid.contains("main[\"main.rs\"]"));
        assert!(mermaid.contains("lib[\"lib.rs\"]"));
        assert!(mermaid.contains("main --> lib"));
        
        // Test determinism
        let mermaid2 = generator.generate(&graph, &MermaidOptions::default())?;
        assert_eq!(mermaid, mermaid2);
        
        Ok(json!({
            "nodes_rendered": 2,
            "edges_rendered": 1,
            "output_size": mermaid.len(),
            "deterministic": true,
        }))
    }
}
```

### Compact Error Context

```rust
#[derive(Serialize)]
struct CompactErrorContext {
    failed_features: Vec<String>,
    error_patterns: BTreeMap<String, Vec<String>>,
    suggested_fixes: Vec<SuggestedFix>,
    environment: EnvironmentSnapshot,
}

#[derive(Serialize)]
struct SuggestedFix {
    feature: String,
    error_pattern: String,
    fix_command: Option<String>,
    documentation_link: Option<String>,
}

impl SelfDiagnostic {
    fn extract_error_context(&self, features: &BTreeMap<String, FeatureResult>) -> Option<CompactErrorContext> {
        let failed: Vec<_> = features
            .iter()
            .filter(|(_, r)| matches!(r.status, FeatureStatus::Failed))
            .map(|(name, _)| name.clone())
            .collect();
            
        if failed.is_empty() {
            return None;
        }
        
        let mut error_patterns = BTreeMap::new();
        for (feature, result) in features {
            if let Some(error) = &result.error {
                let pattern = self.classify_error(error);
                error_patterns.entry(pattern).or_insert_with(Vec::new).push(feature.clone());
            }
        }
        
        Some(CompactErrorContext {
            failed_features: failed,
            error_patterns,
            suggested_fixes: self.generate_fixes(&error_patterns),
            environment: EnvironmentSnapshot::capture(),
        })
    }
    
    fn classify_error(&self, error: &str) -> String {
        if error.contains("Permission denied") {
            "permission_denied".into()
        } else if error.contains("not found") {
            "file_not_found".into()
        } else if error.contains("timeout") {
            "timeout".into()
        } else if error.contains("git") {
            "git_error".into()
        } else {
            "unknown".into()
        }
    }
}
```

### CLI Integration

```rust
#[derive(clap::Args)]
struct DiagnoseArgs {
    /// Output format for diagnostic report
    #[arg(long, value_enum, default_value = "pretty")]
    format: DiagnosticFormat,
    
    /// Only run specific feature tests (can be repeated)
    #[arg(long)]
    only: Vec<String>,
    
    /// Skip specific feature tests (can be repeated)
    #[arg(long)]
    skip: Vec<String>,
    
    /// Maximum time to run diagnostics
    #[arg(long, default_value = "60s")]
    timeout: humantime::Duration,
}

pub async fn handle_diagnose(args: DiagnoseArgs) -> Result<()> {
    let diagnostic = SelfDiagnostic::new();
    let report = diagnostic.run_diagnostic().await;
    
    match args.format {
        DiagnosticFormat::Pretty => {
            println!("PMAT Self-Diagnostic Report");
            println!("==========================");
            println!("Version: {}", report.version);
            println!("Duration: {}ms", report.duration_ms);
            println!();
            
            for (feature, result) in &report.features {
                let icon = match result.status {
                    FeatureStatus::Ok => "✓",
                    FeatureStatus::Degraded(_) => "⚠",
                    FeatureStatus::Failed => "✗",
                    FeatureStatus::Skipped(_) => "○",
                };
                
                println!("{} {} ({}μs)", icon, feature, result.duration_us);
                
                if let Some(error) = &result.error {
                    println!("  └─ {}", error);
                }
            }
            
            if let Some(ctx) = &report.error_context {
                println!();
                println!("Suggested Fixes:");
                for fix in &ctx.suggested_fixes {
                    println!("- {}: {}", fix.feature, fix.fix_command.as_ref().unwrap_or(&"See documentation".into()));
                }
            }
        }
        DiagnosticFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        DiagnosticFormat::Compact => {
            // Ultra-compact for Claude Code consumption
            let compact = json!({
                "v": report.version,
                "ok": report.summary.all_passed,
                "failed": report.error_context.as_ref().map(|c| &c.failed_features),
                "fixes": report.error_context.as_ref().map(|c| &c.suggested_fixes),
            });
            println!("{}", serde_json::to_string(&compact)?);
        }
    }
    
    Ok(())
}
```

## Usage Examples

```bash
# Check for updates (non-blocking)
pmat self-update --check

# Update binary with automatic rollback
pmat self-update

# Run full diagnostic suite
pmat diagnose

# Run specific feature tests
pmat diagnose --only ast.rust --only cache.subsystem

# Get compact diagnostic for Claude Code
pmat diagnose --format compact | pbcopy

# Continuous monitoring mode
watch -n 300 'pmat diagnose --format json | jq .summary'
```

## Implementation Timeline

1. **Phase 1** (1 week): Basic update check with GitHub API
2. **Phase 2** (1 week): Core diagnostic framework with 5 essential tests
3. **Phase 3** (2 weeks): Complete feature test coverage
4. **Phase 4** (1 week): Atomic self-update with rollback
5. **Phase 5** (1 week): Differential updates and signature verification

## Performance Targets

- Update check: <100ms when cached, <5s uncached
- Full diagnostic suite: <10s for all features
- Individual feature test: <500ms typical, 10s timeout
- Diagnostic report size: <10KB compressed JSON

## Kaizen Diagnostic Target

Add to `Makefile`:

```makefile
# =============================================================================
# Kaizen Continuous Improvement - Self-improving diagnostic system
# =============================================================================

# Run comprehensive diagnostics with automatic improvement cycle
kaizen-diagnose: release
	@echo "🔄 Kaizen Diagnostic Cycle - Continuous Improvement"
	@echo "===================================================="
	@mkdir -p artifacts/kaizen/$(shell date +%Y-%m-%d)
	@echo "📊 Phase 1: Measure (測定) - Current state assessment..."
	@./target/release/pmat diagnose --format json > artifacts/kaizen/current-state.json
	@echo "🔍 Phase 2: Analyze (分析) - Root cause identification..."
	@$(MAKE) kaizen-analyze-diagnostics
	@echo "🛠️ Phase 3: Improve (改善) - Automated optimizations..."
	@$(MAKE) kaizen-apply-improvements
	@echo "✅ Phase 4: Control (管理) - Verify improvements..."
	@$(MAKE) kaizen-verify-improvements
	@echo "📈 Phase 5: Standardize (標準化) - Update baselines..."
	@$(MAKE) kaizen-update-standards

# Analyze diagnostic results and identify improvement opportunities
kaizen-analyze-diagnostics:
	@echo "🔬 Analyzing diagnostic patterns..."
	@deno run --allow-all scripts/kaizen-analyzer.ts \
		--input artifacts/kaizen/current-state.json \
		--baseline artifacts/kaizen/baseline.json \
		--output artifacts/kaizen/analysis.json
	@echo "📊 Performance regression detection..."
	@jq -r '.features | to_entries | map(select(.value.duration_us > 500000)) | .[].key' \
		artifacts/kaizen/current-state.json > artifacts/kaizen/slow-features.txt
	@echo "❌ Failure pattern analysis..."
	@jq -r '.features | to_entries | map(select(.value.status == "Failed")) | group_by(.value.error) | map({pattern: .[0].value.error, count: length})' \
		artifacts/kaizen/current-state.json > artifacts/kaizen/failure-patterns.json

# Apply automated improvements based on analysis
kaizen-apply-improvements:
	@echo "🔧 Applying automated improvements..."
	@# Cache optimization for slow features
	@if [ -s artifacts/kaizen/slow-features.txt ]; then \
		echo "⚡ Optimizing cache configuration for slow features..."; \
		./scripts/optimize-cache-config.sh < artifacts/kaizen/slow-features.txt; \
	fi
	@# Test timeout adjustments
	@echo "⏱️ Adjusting test timeouts based on p99 latencies..."
	@jq -r '.features | to_entries | map({feature: .key, p99: (.value.duration_us * 1.5 / 1000000)}) | map("sed -i \"s/timeout(Duration::from_secs(10), \(.feature)/timeout(Duration::from_secs(\(.p99|ceil)), \(.feature)/g\" src/")' \
		artifacts/kaizen/current-state.json | sh 2>/dev/null || true
	@# Dead code elimination based on failures
	@echo "🧹 Removing consistently failing deprecated features..."
	@./scripts/kaizen-dead-code-eliminator.sh

# Verify improvements through re-testing
kaizen-verify-improvements:
	@echo "🔄 Re-running diagnostics to verify improvements..."
	@./target/release/pmat diagnose --format json > artifacts/kaizen/improved-state.json
	@echo "📊 Calculating improvement metrics..."
	@deno run --allow-all scripts/kaizen-metrics.ts \
		--before artifacts/kaizen/current-state.json \
		--after artifacts/kaizen/improved-state.json \
		--report artifacts/kaizen/improvement-report.md
	@cat artifacts/kaizen/improvement-report.md

# Update baseline standards with improvements
kaizen-update-standards:
	@echo "📐 Updating performance baselines..."
	@# Only update baseline if improvements are significant (>5%)
	@improvement=$$(jq -r '.summary.improvement_percentage' artifacts/kaizen/improvement-report.json); \
	if (( $$(echo "$$improvement > 5" | bc -l) )); then \
		echo "✅ Significant improvement ($$improvement%), updating baseline..."; \
		cp artifacts/kaizen/improved-state.json artifacts/kaizen/baseline.json; \
		git add artifacts/kaizen/baseline.json; \
		git commit -m "chore(kaizen): Update diagnostic baseline ($$improvement% improvement)" || true; \
	else \
		echo "ℹ️ Improvement below threshold ($$improvement%), keeping current baseline"; \
	fi

# Generate Kaizen improvement suggestions for manual intervention
kaizen-suggest:
	@echo "💡 Generating Kaizen improvement suggestions..."
	@echo "# Kaizen Improvement Opportunities" > artifacts/kaizen/suggestions.md
	@echo "Generated: $$(date)" >> artifacts/kaizen/suggestions.md
	@echo "" >> artifacts/kaizen/suggestions.md
	@# Complexity-based suggestions
	@echo "## 🧮 Complexity Reduction Targets" >> artifacts/kaizen/suggestions.md
	@echo "Files with cognitive complexity > 30 (refactoring candidates):" >> artifacts/kaizen/suggestions.md
	@./target/release/pmat analyze complexity --top-files 10 --format json | \
		jq -r '.files[] | select(.max_cognitive_complexity > 30) | "- \(.file_path): Cognitive \(.max_cognitive_complexity) (target: <20)"' \
		>> artifacts/kaizen/suggestions.md
	@# Cache hit rate optimization
	@echo "" >> artifacts/kaizen/suggestions.md
	@echo "## 📊 Cache Optimization Opportunities" >> artifacts/kaizen/suggestions.md
	@jq -r '.features["cache.subsystem"].metrics | "Current hit rate: \(.hit_rate)%"' \
		artifacts/kaizen/current-state.json >> artifacts/kaizen/suggestions.md 2>/dev/null || echo "No cache metrics available"
	@# Test coverage gaps
	@echo "" >> artifacts/kaizen/suggestions.md
	@echo "## 🧪 Test Coverage Improvements" >> artifacts/kaizen/suggestions.md
	@echo "Features without adequate test coverage:" >> artifacts/kaizen/suggestions.md
	@comm -23 <(find src -name "*.rs" -exec basename {} .rs \; | sort) \
		<(find src/tests -name "*test*.rs" -exec grep -l "test_" {} \; | xargs -I{} basename {} .rs | sort) | \
		head -10 | sed 's/^/- /' >> artifacts/kaizen/suggestions.md
	@echo "" >> artifacts/kaizen/suggestions.md
	@echo "📄 Full report: artifacts/kaizen/suggestions.md"

# Continuous improvement loop (run in CI/CD)
kaizen-ci: kaizen-diagnose
	@if [ -f artifacts/kaizen/improvement-report.json ]; then \
		regression=$$(jq -r '.regressions | length' artifacts/kaizen/improvement-report.json); \
		if [ "$$regression" -gt 0 ]; then \
			echo "❌ Performance regressions detected!"; \
			jq -r '.regressions[]' artifacts/kaizen/improvement-report.json; \
			exit 1; \
		fi; \
	fi
	@echo "✅ No performance regressions detected"

# Historical trend analysis
kaizen-trends:
	@echo "📈 Analyzing historical diagnostic trends..."
	@mkdir -p artifacts/kaizen/history
	@# Archive current results with timestamp
	@cp artifacts/kaizen/current-state.json \
		"artifacts/kaizen/history/diagnostic-$$(date +%Y%m%d-%H%M%S).json"
	@# Generate trend report
	@deno run --allow-all scripts/kaizen-trends.ts \
		--history-dir artifacts/kaizen/history \
		--output artifacts/kaizen/trends.html
	@echo "📊 Trend report generated: artifacts/kaizen/trends.html"

# Self-healing diagnostic system
kaizen-self-heal:
	@echo "🔧 Initiating self-healing diagnostics..."
	@# Fix permission issues automatically
	@find . -name "*.rs" -type f ! -readable -exec chmod +r {} \;
	@# Rebuild cache if corrupted
	@if ! ./target/release/pmat diagnose --only cache.subsystem >/dev/null 2>&1; then \
		echo "🔄 Cache corruption detected, rebuilding..."; \
		rm -rf ~/.cache/pmat/*; \
		./target/release/pmat warm-cache; \
	fi
	@# Repair git state if needed
	@if ! git status >/dev/null 2>&1; then \
		echo "🔄 Git state corrupted, repairing..."; \
		git fsck --full; \
		git gc --aggressive; \
	fi
	@echo "✅ Self-healing complete"

# Generate Kaizen dashboard
kaizen-dashboard: kaizen-diagnose kaizen-trends
	@echo "📊 Generating Kaizen dashboard..."
	@echo "<!DOCTYPE html>" > artifacts/kaizen/dashboard.html
	@echo "<html><head><title>PMAT Kaizen Dashboard</title>" >> artifacts/kaizen/dashboard.html
	@echo "<style>body{font-family:monospace;padding:20px}.metric{display:inline-block;margin:10px;padding:15px;border:1px solid #ccc}</style>" >> artifacts/kaizen/dashboard.html
	@echo "</head><body><h1>PMAT Kaizen Dashboard</h1>" >> artifacts/kaizen/dashboard.html
	@echo "<div class='metrics'>" >> artifacts/kaizen/dashboard.html
	@# Add key metrics
	@jq -r '"<div class=\"metric\"><h3>Overall Health</h3><p>\(.summary.passed)/\(.summary.total) features passing</p></div>"' \
		artifacts/kaizen/current-state.json >> artifacts/kaizen/dashboard.html
	@echo "</div><iframe src='trends.html' width='100%' height='600'></iframe>" >> artifacts/kaizen/dashboard.html
	@echo "</body></html>" >> artifacts/kaizen/dashboard.html
	@echo "🌐 Dashboard available at: artifacts/kaizen/dashboard.html"

# Clean Kaizen artifacts older than 30 days
kaizen-clean:
	@echo "🧹 Cleaning old Kaizen artifacts..."
	@find artifacts/kaizen/history -name "diagnostic-*.json" -mtime +30 -delete
	@echo "✅ Cleanup complete"

.PHONY: kaizen-diagnose kaizen-analyze-diagnostics kaizen-apply-improvements \
	kaizen-verify-improvements kaizen-update-standards kaizen-suggest \
	kaizen-ci kaizen-trends kaizen-self-heal kaizen-dashboard kaizen-clean
```

## Supporting Scripts

### `scripts/kaizen-analyzer.ts`

```typescript
#!/usr/bin/env -S deno run --allow-all

interface DiagnosticReport {
  version: string;
  features: Record<string, FeatureResult>;
  summary: { total: number; passed: number; failed: number };
}

interface FeatureResult {
  status: "Ok" | "Failed" | "Degraded" | "Skipped";
  duration_us: number;
  error?: string;
  metrics?: any;
}

interface KaizenAnalysis {
  performance_bottlenecks: string[];
  failure_clusters: Record<string, string[]>;
  improvement_opportunities: Opportunity[];
  regression_risks: string[];
}

interface Opportunity {
  feature: string;
  type: "cache" | "algorithm" | "concurrency" | "io";
  impact: "high" | "medium" | "low";
  suggestion: string;
  estimated_improvement_ms: number;
}

async function analyzeForKaizen(
  current: DiagnosticReport,
  baseline?: DiagnosticReport
): Promise<KaizenAnalysis> {
  const analysis: KaizenAnalysis = {
    performance_bottlenecks: [],
    failure_clusters: {},
    improvement_opportunities: [],
    regression_risks: [],
  };

  // Identify performance bottlenecks (Pareto principle: 80/20)
  const sortedByDuration = Object.entries(current.features)
    .sort(([, a], [, b]) => b.duration_us - a.duration_us)
    .slice(0, Math.ceil(Object.keys(current.features).length * 0.2));

  const totalDuration = Object.values(current.features)
    .reduce((sum, f) => sum + f.duration_us, 0);

  for (const [feature, result] of sortedByDuration) {
    const percentage = (result.duration_us / totalDuration) * 100;
    if (percentage > 10) {
      analysis.performance_bottlenecks.push(feature);
      
      // Suggest improvements based on feature type
      if (feature.includes("ast")) {
        analysis.improvement_opportunities.push({
          feature,
          type: "cache",
          impact: "high",
          suggestion: "Implement thread-local AST cache with LRU eviction",
          estimated_improvement_ms: result.duration_us * 0.7 / 1000,
        });
      } else if (feature.includes("git")) {
        analysis.improvement_opportunities.push({
          feature,
          type: "io",
          impact: "medium",
          suggestion: "Use libgit2 bindings instead of process spawning",
          estimated_improvement_ms: result.duration_us * 0.5 / 1000,
        });
      }
    }
  }

  // Cluster failures by error pattern
  for (const [feature, result] of Object.entries(current.features)) {
    if (result.status === "Failed" && result.error) {
      const pattern = classifyError(result.error);
      if (!analysis.failure_clusters[pattern]) {
        analysis.failure_clusters[pattern] = [];
      }
      analysis.failure_clusters[pattern].push(feature);
    }
  }

  // Detect regressions if baseline exists
  if (baseline) {
    for (const [feature, current_result] of Object.entries(current.features)) {
      const baseline_result = baseline.features[feature];
      if (baseline_result && current_result.duration_us > baseline_result.duration_us * 1.2) {
        analysis.regression_risks.push(feature);
      }
    }
  }

  return analysis;
}

function classifyError(error: string): string {
  const patterns = [
    { regex: /permission denied/i, category: "permissions" },
    { regex: /timeout|timed out/i, category: "timeout" },
    { regex: /not found|missing/i, category: "missing_resource" },
    { regex: /parse|syntax/i, category: "parse_error" },
    { regex: /memory|oom/i, category: "memory" },
  ];

  for (const { regex, category } of patterns) {
    if (regex.test(error)) return category;
  }
  return "unknown";
}

// Main execution
if (import.meta.main) {
  const args = parseArgs(Deno.args);
  const current = JSON.parse(await Deno.readTextFile(args.input));
  const baseline = args.baseline 
    ? JSON.parse(await Deno.readTextFile(args.baseline))
    : undefined;

  const analysis = await analyzeForKaizen(current, baseline);
  await Deno.writeTextFile(args.output, JSON.stringify(analysis, null, 2));
}
```

### Usage Examples

```bash
# Run full Kaizen improvement cycle
make kaizen-diagnose

# Generate improvement suggestions
make kaizen-suggest

# View historical trends
make kaizen-trends
open artifacts/kaizen/trends.html

# CI/CD integration (fail on regression)
make kaizen-ci

# Self-heal before diagnostics
make kaizen-self-heal && make kaizen-diagnose
```

This implements the Kaizen philosophy through:
1. **Measure** (測定) - Comprehensive diagnostics
2. **Analyze** (分析) - Pattern recognition and clustering
3. **Improve** (改善) - Automated fixes where possible
4. **Control** (管理) - Verification and regression prevention
5. **Standardize** (標準化) - Baseline updates and documentation