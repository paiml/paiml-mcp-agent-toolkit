# Documentation Accuracy Enforcement: Toyota Way Addendum

**Status**: Draft Enhancement
**Type**: Specification Enhancement
**Created**: 2025-10-18
**Parent Spec**: `documentation-accuracy-enforcement.md`
**Priority**: P0
**Review**: Toyota Way Principles Applied

---

## Executive Summary

This addendum enhances the Documentation Accuracy Enforcement specification by applying Toyota Way principles more deeply. Based on peer review, we address long-term maintainability, intelligent validation workflows, broader evidence sources, trend analysis, extensibility, and refined confidence scoring.

## Toyota Way Enhancements

### Enhancement 1: Self-Documentation & Recursive Validation (Long-Term Philosophy)

**Problem**: The validator itself is a complex system that needs documentation and validation.

**Toyota Way Principle**: Base management decisions on a long-term philosophy, even at the expense of short-term financial goals.

#### Solution: Validator Self-Validation

```rust
/// Self-validation module for the DocAccuracyValidator
///
/// Applies validation recursively to the validator's own documentation
pub struct SelfValidator {
    /// The main validator instance
    validator: DocAccuracyValidator,

    /// Configuration for self-validation
    config: SelfValidationConfig,
}

#[derive(Debug, Clone)]
pub struct SelfValidationConfig {
    /// Validate validator's own source documentation
    pub validate_own_docs: bool,

    /// Minimum code coverage for validator components
    pub min_coverage: f32,

    /// Require validator components to have examples
    pub require_examples: bool,
}

impl SelfValidator {
    /// Validate the validator's own documentation
    ///
    /// # Toyota Way - Jidoka
    ///
    /// This is "building in quality" at a meta-level. The validator validates
    /// its own claims about its capabilities.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let self_validator = SelfValidator::new(validator);
    /// let results = self_validator.validate_self().await?;
    ///
    /// // The validator's own documentation must pass its own checks
    /// assert_eq!(results.status, ValidationStatus::Verified);
    /// ```
    pub async fn validate_self(&self) -> Result<SelfValidationReport> {
        let mut issues = Vec::new();

        // 1. Validate validator's own Rust documentation
        issues.extend(self.validate_rust_docs().await?);

        // 2. Validate specification document
        issues.extend(self.validate_spec_doc().await?);

        // 3. Validate examples in documentation
        issues.extend(self.validate_examples().await?);

        // 4. Check test coverage
        issues.extend(self.validate_coverage().await?);

        Ok(SelfValidationReport {
            total_checks: issues.len(),
            passed: issues.iter().filter(|i| i.passed).count(),
            issues,
        })
    }

    async fn validate_rust_docs(&self) -> Result<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // Get all public APIs in validator crate
        let validator_files = glob("server/src/services/doc_validator/**/*.rs")?;

        for file in validator_files {
            let content = tokio::fs::read_to_string(&file).await?;

            // Extract doc comments
            let doc_claims = self.extract_doc_comments(&content, &file);

            // Validate each doc claim against the actual implementation
            for claim in doc_claims {
                let result = self.validator.validate_claim(&claim).await?;

                if !matches!(result.status, ValidationStatus::Verified) {
                    issues.push(ValidationIssue {
                        file: file.clone(),
                        line: claim.line_number,
                        message: format!(
                            "Validator's own documentation contains unverified claim: {}",
                            claim.text
                        ),
                        passed: false,
                    });
                }
            }
        }

        Ok(issues)
    }

    async fn validate_examples(&self) -> Result<Vec<ValidationIssue>> {
        // Run cargo test --doc to ensure all examples compile and run
        let output = tokio::process::Command::new("cargo")
            .args(&["test", "--doc", "--package", "pmat-doc-validator"])
            .output()
            .await?;

        if !output.status.success() {
            return Ok(vec![ValidationIssue {
                file: PathBuf::from("validator/src/lib.rs"),
                line: 0,
                message: "Validator's documentation examples failed to run".to_string(),
                passed: false,
            }]);
        }

        Ok(vec![])
    }
}

/// Self-validation report
#[derive(Debug, Clone)]
pub struct SelfValidationReport {
    pub total_checks: usize,
    pub passed: usize,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub file: PathBuf,
    pub line: usize,
    pub message: String,
    pub passed: bool,
}
```

#### Integration

```bash
# Run self-validation as part of CI
pmat validate-readme --self-validate

# Pre-commit hook includes self-validation
if [ -f "server/src/services/doc_validator/**/*.rs" ]; then
    pmat validate-readme --self-validate
fi
```

---

### Enhancement 2: Intelligent Re-Validation (Jidoka - Stop the Line)

**Problem**: Code changes can invalidate documentation without the documentation file being modified.

**Toyota Way Principle**: Create continuous process flow to bring problems to the surface (Jidoka).

#### Solution: Dependency-Based Validation

```rust
/// Dependency analyzer for intelligent re-validation
pub struct DependencyAnalyzer {
    /// Deep context containing dependency graph
    deep_context: DeepContextData,

    /// Git integration for change detection
    git_analyzer: GitAnalyzer,
}

impl DependencyAnalyzer {
    /// Identify documentation files affected by code changes
    ///
    /// # Toyota Way - Jidoka
    ///
    /// This prevents defects from escaping by re-validating documentation
    /// even when only code changes, stopping the line before bad docs propagate.
    ///
    /// # Algorithm
    ///
    /// 1. Analyze git diff to find changed functions/files
    /// 2. Query dependency graph to find documentation references
    /// 3. Return list of documentation files to re-validate
    ///
    /// # Examples
    ///
    /// ```rust
    /// let analyzer = DependencyAnalyzer::new(deep_context, git);
    /// let changed_files = git.get_changed_files("HEAD~1", "HEAD")?;
    /// let docs_to_validate = analyzer.affected_documentation(&changed_files)?;
    ///
    /// // If server/src/cli/mod.rs changed, README.md might reference it
    /// assert!(docs_to_validate.contains(&PathBuf::from("README.md")));
    /// ```
    pub fn affected_documentation(&self, changed_files: &[PathBuf]) -> Result<Vec<PathBuf>> {
        let mut affected_docs = HashSet::new();

        for file in changed_files {
            // Get all functions/types changed in this file
            let changed_entities = self.extract_changed_entities(file)?;

            // Query which documentation files reference these entities
            for entity in &changed_entities {
                let referencing_docs = self.deep_context.find_documentation_references(entity)?;
                affected_docs.extend(referencing_docs);
            }
        }

        Ok(affected_docs.into_iter().collect())
    }

    fn extract_changed_entities(&self, file: &PathBuf) -> Result<Vec<Entity>> {
        let diff = self.git_analyzer.get_diff_for_file(file)?;
        let mut entities = Vec::new();

        // Parse diff to find changed function signatures
        for change in diff.hunks {
            if change.is_function_signature_change() {
                let func_name = change.extract_function_name()?;
                entities.push(Entity::Function(func_name));
            }
        }

        Ok(entities)
    }
}

/// Deep context extension for documentation references
impl DeepContextData {
    /// Find all documentation files that reference a given entity
    ///
    /// # Examples
    ///
    /// ```rust
    /// let entity = Entity::Function("analyze_complexity".to_string());
    /// let docs = deep_context.find_documentation_references(&entity)?;
    ///
    /// // Returns ["README.md", "CLAUDE.md"] if they mention this function
    /// ```
    pub fn find_documentation_references(&self, entity: &Entity) -> Result<Vec<PathBuf>> {
        let mut docs = Vec::new();

        // Scan all markdown files for entity mentions
        for doc_file in &self.documentation_files {
            let content = std::fs::read_to_string(doc_file)?;

            if self.contains_reference(&content, entity) {
                docs.push(doc_file.clone());
            }
        }

        Ok(docs)
    }

    fn contains_reference(&self, content: &str, entity: &Entity) -> bool {
        match entity {
            Entity::Function(name) => {
                // Check for function references: `function_name(`, function_name:
                let patterns = [
                    format!("`{}(`", name),
                    format!("{}(", name),
                    format!("{}:", name),
                ];
                patterns.iter().any(|p| content.contains(p))
            }
            Entity::File(path) => {
                content.contains(&path.display().to_string())
            }
            Entity::Module(name) => {
                content.contains(name)
            }
            _ => false,
        }
    }
}
```

#### Pre-commit Hook Integration

```bash
#!/bin/bash
# Enhanced pre-commit hook with intelligent re-validation

# Get changed files (both code and docs)
CHANGED_FILES=$(git diff --cached --name-only)

# Identify code changes
CODE_CHANGES=$(echo "$CHANGED_FILES" | grep -E '\\.(rs|ts|js|py)$' || true)

# Identify doc changes
DOC_CHANGES=$(echo "$CHANGED_FILES" | grep -E '(README|CLAUDE|GEMINI|AGENT)\\.md$' || true)

# If code changed, find affected documentation
if [ -n "$CODE_CHANGES" ]; then
    echo "🔍 Code changes detected - analyzing documentation dependencies..."

    AFFECTED_DOCS=$(pmat analyze-doc-deps --changed-files $CODE_CHANGES)

    # Combine explicitly changed docs with dependency-affected docs
    ALL_DOCS_TO_VALIDATE=$(echo -e "$DOC_CHANGES\\n$AFFECTED_DOCS" | sort -u)

    if [ -n "$ALL_DOCS_TO_VALIDATE" ]; then
        echo "📚 Re-validating affected documentation:"
        echo "$ALL_DOCS_TO_VALIDATE"

        pmat validate-readme --targets $ALL_DOCS_TO_VALIDATE --fail-on-error
    fi
fi
```

---

### Enhancement 3: Real-Time IDE Feedback (Jidoka - Immediate Detection)

**Problem**: Waiting for pre-commit hook provides late feedback.

**Toyota Way Principle**: Build in quality at the source (Jidoka), with immediate feedback like an andon cord.

#### Solution: Language Server Protocol (LSP) Integration

```rust
/// LSP server for real-time documentation validation
pub struct DocValidatorLsp {
    /// The main validator
    validator: Arc<DocAccuracyValidator>,

    /// Client connection
    client: Client,

    /// Debounce timer for validation
    debounce: Duration,
}

impl LanguageServer for DocValidatorLsp {
    async fn did_change(&self, params: DidChangeTextDocumentParams) -> Result<()> {
        // Debounce to avoid validating on every keystroke
        tokio::time::sleep(self.debounce).await;

        let uri = params.text_document.uri;
        let content = self.get_document_content(&uri).await?;

        // Extract claims from the current document
        let claims = self.validator.claim_extractor.extract(
            &content,
            PathBuf::from(uri.path())
        )?;

        let mut diagnostics = Vec::new();

        // Validate each claim in real-time
        for claim in claims {
            let result = self.validator.hallucination_detector.validate_claim(&claim).await?;

            if !matches!(result.status, ValidationStatus::Verified) {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position::new(claim.line_number as u32, 0),
                        end: Position::new(claim.line_number as u32, claim.text.len() as u32),
                    },
                    severity: Some(match result.status {
                        ValidationStatus::Contradiction => DiagnosticSeverity::ERROR,
                        ValidationStatus::Unverified => DiagnosticSeverity::WARNING,
                        _ => DiagnosticSeverity::INFORMATION,
                    }),
                    message: result.error_message.unwrap_or_else(|| {
                        format!("Confidence: {:.2}", result.confidence)
                    }),
                    source: Some("pmat-doc-validator".to_string()),
                    ..Default::default()
                });
            }
        }

        // Send diagnostics to IDE
        self.client.publish_diagnostics(uri, diagnostics, None).await;

        Ok(())
    }
}
```

#### VS Code Extension

```typescript
// VS Code extension for PMAT documentation validation
import * as vscode from 'vscode';
import { LanguageClient } from 'vscode-languageclient/node';

export function activate(context: vscode.ExtensionContext) {
    // Start LSP server
    const serverOptions = {
        command: 'pmat',
        args: ['lsp', '--validator', 'doc-accuracy']
    };

    const client = new LanguageClient(
        'pmatDocValidator',
        'PMAT Documentation Validator',
        serverOptions,
        {
            documentSelector: [
                { scheme: 'file', pattern: '**/README.md' },
                { scheme: 'file', pattern: '**/CLAUDE.md' },
                { scheme: 'file', pattern: '**/GEMINI.md' },
                { scheme: 'file', pattern: '**/AGENT.md' }
            ]
        }
    );

    client.start();
}
```

---

### Enhancement 4: Extended Code Facts Database (Genchi Genbutsu - Broader Evidence)

**Problem**: AST alone doesn't capture performance, behavior, or non-functional requirements.

**Toyota Way Principle**: "Go and see" to thoroughly understand the situation - gather evidence from multiple sources.

#### Solution: Multi-Source Evidence Integration

```rust
/// Enhanced code facts database with multiple evidence sources
pub struct EnhancedCodeFactDatabase {
    /// AST-based facts (existing)
    ast_facts: HashMap<Entity, Vec<CodeFact>>,

    /// Performance benchmark facts
    benchmark_facts: HashMap<Entity, Vec<BenchmarkFact>>,

    /// Test coverage facts
    coverage_facts: HashMap<Entity, Vec<CoverageFact>>,

    /// Git history facts (temporal evidence)
    history_facts: HashMap<Entity, Vec<HistoryFact>>,

    /// Documentation facts (existing docs)
    doc_facts: HashMap<Entity, Vec<DocFact>>,
}

/// Performance benchmark fact
#[derive(Debug, Clone)]
pub struct BenchmarkFact {
    pub entity: Entity,
    pub benchmark_name: String,
    pub median_time_ns: u64,
    pub throughput: Option<f64>,
    pub source_file: PathBuf,
}

/// Test coverage fact
#[derive(Debug, Clone)]
pub struct CoverageFact {
    pub entity: Entity,
    pub line_coverage: f32,
    pub branch_coverage: f32,
    pub has_tests: bool,
    pub test_count: usize,
}

/// Git history fact (temporal evidence)
#[derive(Debug, Clone)]
pub struct HistoryFact {
    pub entity: Entity,
    pub last_modified: SystemTime,
    pub modification_count: usize,
    pub last_modifier: String,
    pub is_stable: bool, // Low churn rate
}

impl EnhancedCodeFactDatabase {
    /// Build from multiple sources
    ///
    /// # Toyota Way - Genchi Genbutsu
    ///
    /// "Go and see" means gathering evidence from all available sources,
    /// not just the AST. This creates a richer, more complete picture.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let db = EnhancedCodeFactDatabase::builder()
    ///     .with_deep_context("deep_context.md")
    ///     .with_benchmarks("target/criterion")
    ///     .with_coverage("target/coverage.json")
    ///     .with_git_history(".")
    ///     .build()
    ///     .await?;
    ///
    /// // Query returns facts from all sources
    /// let facts = db.query_comprehensive(&Entity::Function("analyze".to_string()))?;
    /// assert!(facts.has_benchmark_data);
    /// assert!(facts.has_coverage_data);
    /// ```
    pub async fn build_from_sources(
        deep_context_path: &Path,
        benchmarks_dir: Option<&Path>,
        coverage_report: Option<&Path>,
        git_repo: Option<&Path>,
    ) -> Result<Self> {
        let mut db = Self::default();

        // 1. Load AST facts (existing)
        let deep_context = std::fs::read_to_string(deep_context_path)?;
        db.ast_facts = Self::parse_ast_facts(&deep_context)?;

        // 2. Load benchmark facts
        if let Some(bench_dir) = benchmarks_dir {
            db.benchmark_facts = Self::parse_criterion_benchmarks(bench_dir).await?;
        }

        // 3. Load coverage facts
        if let Some(cov_path) = coverage_report {
            db.coverage_facts = Self::parse_coverage_report(cov_path).await?;
        }

        // 4. Load git history facts
        if let Some(repo_path) = git_repo {
            db.history_facts = Self::analyze_git_history(repo_path).await?;
        }

        Ok(db)
    }

    async fn parse_criterion_benchmarks(bench_dir: &Path) -> Result<HashMap<Entity, Vec<BenchmarkFact>>> {
        let mut facts = HashMap::new();

        // Parse Criterion's JSON output
        for entry in glob(&format!("{}/**/estimates.json", bench_dir.display()))? {
            let json_path = entry?;
            let data: serde_json::Value = serde_json::from_str(
                &tokio::fs::read_to_string(&json_path).await?
            )?;

            // Extract function name from benchmark
            if let Some(func_name) = Self::extract_function_from_benchmark_name(&json_path) {
                let entity = Entity::Function(func_name);

                facts.entry(entity.clone()).or_insert_with(Vec::new).push(BenchmarkFact {
                    entity,
                    benchmark_name: json_path.file_name().unwrap().to_string_lossy().to_string(),
                    median_time_ns: data["median"]["point_estimate"].as_f64().unwrap_or(0.0) as u64,
                    throughput: data.get("throughput").and_then(|t| t.as_f64()),
                    source_file: json_path.clone(),
                });
            }
        }

        Ok(facts)
    }

    async fn parse_coverage_report(cov_path: &Path) -> Result<HashMap<Entity, Vec<CoverageFact>>> {
        // Parse cargo-llvm-cov JSON output
        let json: serde_json::Value = serde_json::from_str(
            &tokio::fs::read_to_string(cov_path).await?
        )?;

        let mut facts = HashMap::new();

        for file in json["data"].as_array().unwrap_or(&vec![]) {
            for function in file["functions"].as_array().unwrap_or(&vec![]) {
                let func_name = function["name"].as_str().unwrap();
                let entity = Entity::Function(func_name.to_string());

                facts.entry(entity.clone()).or_insert_with(Vec::new).push(CoverageFact {
                    entity,
                    line_coverage: function["execution_count"].as_f64().unwrap_or(0.0) as f32,
                    branch_coverage: function["branches_covered"].as_f64().unwrap_or(0.0) as f32,
                    has_tests: function["execution_count"].as_u64().unwrap_or(0) > 0,
                    test_count: function["execution_count"].as_usize().unwrap_or(0),
                });
            }
        }

        Ok(facts)
    }

    /// Validate claims using comprehensive evidence
    ///
    /// # Examples
    ///
    /// ```rust
    /// let claim = Claim {
    ///     text: "PMAT can analyze Ruby code efficiently (sub-second)".to_string(),
    ///     claim_type: ClaimType::Capability,
    ///     entities: vec![Entity::Language("Ruby".to_string())],
    ///     // ...
    /// };
    ///
    /// let result = db.validate_with_evidence(&claim).await?;
    ///
    /// // Result includes benchmark evidence to support "efficiently" claim
    /// assert!(result.evidence.contains_benchmark);
    /// assert!(result.evidence.benchmark_time_ns < 1_000_000_000); // < 1 second
    /// ```
    pub async fn validate_with_evidence(&self, claim: &Claim) -> Result<ComprehensiveValidationResult> {
        let mut evidence = ComprehensiveEvidence::default();

        // Collect evidence from all sources
        for entity in &claim.entities {
            if let Some(ast) = self.ast_facts.get(entity) {
                evidence.ast_evidence.extend(ast.clone());
            }

            if let Some(bench) = self.benchmark_facts.get(entity) {
                evidence.benchmark_evidence.extend(bench.clone());
            }

            if let Some(cov) = self.coverage_facts.get(entity) {
                evidence.coverage_evidence.extend(cov.clone());
            }

            if let Some(hist) = self.history_facts.get(entity) {
                evidence.history_evidence.extend(hist.clone());
            }
        }

        // Validate claim against comprehensive evidence
        let status = self.determine_status(&claim, &evidence)?;
        let confidence = self.calculate_comprehensive_confidence(&evidence)?;

        Ok(ComprehensiveValidationResult {
            claim: claim.clone(),
            status,
            evidence,
            confidence,
        })
    }

    fn determine_status(&self, claim: &Claim, evidence: &ComprehensiveEvidence) -> Result<ValidationStatus> {
        // Example: Check performance claims
        if claim.text.contains("efficient") || claim.text.contains("fast") {
            if evidence.benchmark_evidence.is_empty() {
                return Ok(ValidationStatus::Inconclusive);
            }

            // Heuristic: "efficient" means < 100ms
            let avg_time = evidence.benchmark_evidence.iter()
                .map(|b| b.median_time_ns)
                .sum::<u64>() / evidence.benchmark_evidence.len() as u64;

            if avg_time > 100_000_000 { // > 100ms
                return Ok(ValidationStatus::Contradiction);
            }
        }

        // Default: use AST evidence
        if evidence.ast_evidence.is_empty() {
            Ok(ValidationStatus::Unverified)
        } else {
            Ok(ValidationStatus::Verified)
        }
    }

    fn calculate_comprehensive_confidence(&self, evidence: &ComprehensiveEvidence) -> Result<f32> {
        let mut confidence = 0.0;
        let mut weight_sum = 0.0;

        // Weight different evidence types
        if !evidence.ast_evidence.is_empty() {
            confidence += 0.5; // AST evidence: 50% confidence
            weight_sum += 1.0;
        }

        if !evidence.benchmark_evidence.is_empty() {
            confidence += 0.3; // Benchmark evidence: 30% confidence boost
            weight_sum += 1.0;
        }

        if !evidence.coverage_evidence.is_empty() {
            confidence += 0.2; // Coverage evidence: 20% confidence boost
            weight_sum += 1.0;
        }

        if weight_sum > 0.0 {
            Ok(confidence / weight_sum)
        } else {
            Ok(0.0)
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ComprehensiveEvidence {
    pub ast_evidence: Vec<CodeFact>,
    pub benchmark_evidence: Vec<BenchmarkFact>,
    pub coverage_evidence: Vec<CoverageFact>,
    pub history_evidence: Vec<HistoryFact>,
}

#[derive(Debug, Clone)]
pub struct ComprehensiveValidationResult {
    pub claim: Claim,
    pub status: ValidationStatus,
    pub evidence: ComprehensiveEvidence,
    pub confidence: f32,
}
```

---

### Enhancement 5: Trend Analysis & Error Categorization (Hansei & Kaizen)

**Problem**: Current system flags errors but doesn't facilitate learning from them.

**Toyota Way Principle**: Become a learning organization through relentless reflection (Hansei) and continuous improvement (Kaizen).

#### Solution: Validation Metrics & Trend Dashboard

```rust
/// Metrics collector for validation results
pub struct ValidationMetrics {
    /// Database for storing historical metrics
    db: MetricsDatabase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMetricsSnapshot {
    pub timestamp: SystemTime,
    pub total_claims: usize,
    pub verified: usize,
    pub contradictions: usize,
    pub unverified: usize,
    pub broken_links: usize,
    pub inconclusive: usize,

    /// Error categorization
    pub errors_by_category: HashMap<ErrorCategory, usize>,

    /// Errors by file
    pub errors_by_file: HashMap<PathBuf, usize>,

    /// Average confidence score
    pub avg_confidence: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Hallucinated capability (claimed feature doesn't exist)
    HallucinatedCapability,

    /// Outdated API signature
    OutdatedApiSignature,

    /// Broken internal file reference
    BrokenInternalReference,

    /// External link 404
    ExternalLink404,

    /// Unverifiable performance claim
    UnverifiablePerformance,

    /// Missing test evidence
    MissingTestEvidence,
}

impl ValidationMetrics {
    /// Record validation results for trend analysis
    ///
    /// # Toyota Way - Hansei
    ///
    /// Recording metrics enables reflection on what types of errors occur most
    /// frequently, guiding improvement efforts.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let metrics = ValidationMetrics::new("./metrics.db");
    /// metrics.record_results(&validation_results).await?;
    ///
    /// // Later, analyze trends
    /// let trend = metrics.analyze_trend(Duration::from_days(30)).await?;
    /// println!("Top error category: {:?}", trend.most_common_error);
    /// ```
    pub async fn record_results(&self, results: &[ValidationResult]) -> Result<()> {
        let mut snapshot = ValidationMetricsSnapshot {
            timestamp: SystemTime::now(),
            total_claims: results.len(),
            verified: 0,
            contradictions: 0,
            unverified: 0,
            broken_links: 0,
            inconclusive: 0,
            errors_by_category: HashMap::new(),
            errors_by_file: HashMap::new(),
            avg_confidence: 0.0,
        };

        for result in results {
            match result.status {
                ValidationStatus::Verified => snapshot.verified += 1,
                ValidationStatus::Contradiction => {
                    snapshot.contradictions += 1;
                    let category = self.categorize_error(result);
                    *snapshot.errors_by_category.entry(category).or_insert(0) += 1;
                }
                ValidationStatus::Unverified => snapshot.unverified += 1,
                ValidationStatus::NotFound => snapshot.broken_links += 1,
                ValidationStatus::Inconclusive => snapshot.inconclusive += 1,
                _ => {}
            }

            if !matches!(result.status, ValidationStatus::Verified) {
                *snapshot.errors_by_file.entry(result.claim.source_file.clone()).or_insert(0) += 1;
            }

            snapshot.avg_confidence += result.confidence;
        }

        snapshot.avg_confidence /= results.len() as f32;

        self.db.insert_snapshot(&snapshot).await?;

        Ok(())
    }

    fn categorize_error(&self, result: &ValidationResult) -> ErrorCategory {
        // Heuristics for categorizing errors
        if result.claim.text.contains("can ") || result.claim.text.contains("supports ") {
            return ErrorCategory::HallucinatedCapability;
        }

        if result.claim.claim_type == ClaimType::Api {
            return ErrorCategory::OutdatedApiSignature;
        }

        if result.claim.claim_type == ClaimType::ExternalRef {
            return ErrorCategory::ExternalLink404;
        }

        if result.claim.text.contains("efficient") || result.claim.text.contains("fast") {
            return ErrorCategory::UnverifiablePerformance;
        }

        ErrorCategory::BrokenInternalReference
    }

    /// Analyze validation trends over time
    ///
    /// # Toyota Way - Kaizen
    ///
    /// Trend analysis identifies systemic issues that should be addressed
    /// through continuous improvement.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let trend = metrics.analyze_trend(Duration::from_days(30)).await?;
    ///
    /// if trend.most_common_error == ErrorCategory::OutdatedApiSignature {
    ///     println!("Insight: Need better tooling around refactoring");
    /// }
    /// ```
    pub async fn analyze_trend(&self, period: Duration) -> Result<TrendAnalysis> {
        let cutoff = SystemTime::now() - period;
        let snapshots = self.db.get_snapshots_since(cutoff).await?;

        if snapshots.is_empty() {
            return Ok(TrendAnalysis::default());
        }

        // Aggregate error categories
        let mut category_counts: HashMap<ErrorCategory, usize> = HashMap::new();
        for snapshot in &snapshots {
            for (category, count) in &snapshot.errors_by_category {
                *category_counts.entry(category.clone()).or_insert(0) += count;
            }
        }

        let most_common_error = category_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(cat, _)| cat.clone());

        // Calculate trend direction (improving or declining?)
        let first_half = &snapshots[..snapshots.len() / 2];
        let second_half = &snapshots[snapshots.len() / 2..];

        let first_error_rate = first_half.iter()
            .map(|s| (s.contradictions + s.unverified) as f32 / s.total_claims as f32)
            .sum::<f32>() / first_half.len() as f32;

        let second_error_rate = second_half.iter()
            .map(|s| (s.contradictions + s.unverified) as f32 / s.total_claims as f32)
            .sum::<f32>() / second_half.len() as f32;

        let trend_direction = if second_error_rate < first_error_rate {
            TrendDirection::Improving
        } else if second_error_rate > first_error_rate {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        };

        Ok(TrendAnalysis {
            period_days: period.as_secs() / 86400,
            total_snapshots: snapshots.len(),
            most_common_error,
            trend_direction,
            error_rate_change: second_error_rate - first_error_rate,
            recommendations: self.generate_recommendations(&category_counts, &trend_direction),
        })
    }

    fn generate_recommendations(
        &self,
        category_counts: &HashMap<ErrorCategory, usize>,
        trend: &TrendDirection,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Find most common error
        if let Some((category, count)) = category_counts.iter().max_by_key(|(_, c)| *c) {
            match category {
                ErrorCategory::OutdatedApiSignature => {
                    recommendations.push(
                        "High rate of outdated API signatures detected. Consider: \n\
                         1. Implementing automated API change detection\n\
                         2. Adding refactoring notifications to documentation\n\
                         3. Running doc validation more frequently".to_string()
                    );
                }
                ErrorCategory::HallucinatedCapability => {
                    recommendations.push(
                        "Hallucinated capabilities detected. Consider: \n\
                         1. Strengthening claim extraction patterns\n\
                         2. Lowering semantic similarity threshold\n\
                         3. Reviewing documentation writing guidelines".to_string()
                    );
                }
                ErrorCategory::BrokenInternalReference => {
                    recommendations.push(
                        "Broken internal references detected. Consider: \n\
                         1. Implementing file move detection\n\
                         2. Auto-updating references on refactoring\n\
                         3. Using IDE refactoring tools more consistently".to_string()
                    );
                }
                _ => {}
            }
        }

        if matches!(trend, TrendDirection::Declining) {
            recommendations.push(
                "Quality is declining. Consider: \n\
                 1. Running team retrospective on documentation practices\n\
                 2. Increasing validation frequency\n\
                 3. Adding documentation review to PR process".to_string()
            );
        }

        recommendations
    }
}

#[derive(Debug, Clone)]
pub struct TrendAnalysis {
    pub period_days: u64,
    pub total_snapshots: usize,
    pub most_common_error: Option<ErrorCategory>,
    pub trend_direction: TrendDirection,
    pub error_rate_change: f32,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrendDirection {
    Improving,
    Stable,
    Declining,
}
```

#### CLI Integration

```bash
# View validation metrics
pmat validate-readme --metrics --period 30d

# Output:
# 📊 Documentation Validation Metrics (Last 30 Days)
# ================================================
#
# Total Validations: 45
# Trend: Improving ↗ (-5.2% error rate)
#
# Most Common Error: Outdated API Signature (23 occurrences)
#
# Recommendations:
# 1. High rate of outdated API signatures detected
#    - Consider implementing automated API change detection
#    - Add refactoring notifications to documentation
#
# Error Breakdown:
# - Outdated API Signature:        23 (51%)
# - Broken Internal Reference:     12 (27%)
# - Hallucinated Capability:        8 (18%)
# - External Link 404:              2 (4%)
```

---

### Enhancement 6: Extensible Claim Extraction (Team Empowerment)

**Problem**: Teams have project-specific documentation conventions that aren't covered by default patterns.

**Toyota Way Principle**: Respect for people - empower teams to improve the process.

#### Solution: Configurable Claim Patterns

```rust
/// Claim extractor with configurable patterns
pub struct ConfigurableClaimExtractor {
    /// Built-in patterns
    builtin_patterns: Vec<ClaimPattern>,

    /// Custom patterns loaded from configuration
    custom_patterns: Vec<ClaimPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimPattern {
    /// Pattern name
    pub name: String,

    /// Regex pattern
    pub pattern: String,

    /// Claim type this pattern detects
    pub claim_type: ClaimType,

    /// Capture group for claim text
    pub text_group: usize,

    /// Capture groups for entities
    pub entity_groups: Vec<EntityCapture>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityCapture {
    pub group_index: usize,
    pub entity_type: String, // "function", "file", "language", etc.
}

impl ConfigurableClaimExtractor {
    /// Load claim patterns from configuration file
    ///
    /// # Toyota Way - Respect for People
    ///
    /// Teams can define their own patterns without modifying core code,
    /// empowering them to adapt the tool to their needs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let extractor = ConfigurableClaimExtractor::from_config(".pmat/claim-patterns.toml")?;
    /// let claims = extractor.extract(doc_content, file_path)?;
    /// ```
    pub fn from_config(config_path: &Path) -> Result<Self> {
        let config_content = std::fs::read_to_string(config_path)?;
        let config: ClaimPatternsConfig = toml::from_str(&config_content)?;

        Ok(Self {
            builtin_patterns: Self::default_patterns(),
            custom_patterns: config.patterns,
        })
    }

    fn default_patterns() -> Vec<ClaimPattern> {
        vec![
            ClaimPattern {
                name: "capability_can".to_string(),
                pattern: r"PMAT can ([^.]+)".to_string(),
                claim_type: ClaimType::Capability,
                text_group: 0,
                entity_groups: vec![],
            },
            ClaimPattern {
                name: "command_run".to_string(),
                pattern: r"Run `pmat ([^`]+)`".to_string(),
                claim_type: ClaimType::Command,
                text_group: 0,
                entity_groups: vec![
                    EntityCapture {
                        group_index: 1,
                        entity_type: "command".to_string(),
                    }
                ],
            },
            // ... more builtin patterns
        ]
    }

    pub fn extract(&self, content: &str, source_file: PathBuf) -> Result<Vec<Claim>> {
        let mut claims = Vec::new();

        // Apply all patterns (builtin + custom)
        for pattern in self.builtin_patterns.iter().chain(&self.custom_patterns) {
            let regex = Regex::new(&pattern.pattern)?;

            for (line_num, line) in content.lines().enumerate() {
                if let Some(captures) = regex.captures(line) {
                    let text = captures.get(pattern.text_group).unwrap().as_str().to_string();

                    // Extract entities from capture groups
                    let mut entities = Vec::new();
                    for entity_cap in &pattern.entity_groups {
                        if let Some(cap) = captures.get(entity_cap.group_index) {
                            entities.push(self.parse_entity(cap.as_str(), &entity_cap.entity_type)?);
                        }
                    }

                    claims.push(Claim {
                        source_file: source_file.clone(),
                        line_number: line_num + 1,
                        text,
                        claim_type: pattern.claim_type.clone(),
                        entities,
                    });
                }
            }
        }

        Ok(claims)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaimPatternsConfig {
    patterns: Vec<ClaimPattern>,
}
```

#### Configuration File Example

```toml
# .pmat/claim-patterns.toml
# Custom claim extraction patterns for this project

[[patterns]]
name = "performance_benchmark"
pattern = "Performance: ([a-zA-Z_]+) completes in < ([0-9]+)ms"
claim_type = "Capability"
text_group = 0
entity_groups = [
    { group_index = 1, entity_type = "function" }
]

[[patterns]]
name = "test_coverage"
pattern = "([a-zA-Z_]+) has ([0-9]+)% test coverage"
claim_type = "Capability"
text_group = 0
entity_groups = [
    { group_index = 1, entity_type = "module" }
]

[[patterns]]
name = "api_contract"
pattern = "API contract: `([a-zA-Z_]+)\\(([^)]+)\\) -> ([^`]+)`"
claim_type = "Api"
text_group = 0
entity_groups = [
    { group_index = 1, entity_type = "function" }
]
```

---

### Enhancement 7: Refined Confidence Scoring (Evidence Quality)

**Problem**: Current confidence score doesn't reflect the quality and type of evidence.

**Toyota Way Principle**: Base decisions on thorough data analysis (evidence quality matters).

#### Solution: Multi-Factor Confidence Scoring

```rust
/// Refined confidence calculator
pub struct ConfidenceCalculator {
    /// Weights for different evidence types
    weights: ConfidenceWeights,
}

#[derive(Debug, Clone)]
pub struct ConfidenceWeights {
    /// Direct AST match (highest confidence)
    pub ast_exact_match: f32,

    /// AST semantic match (high confidence)
    pub ast_semantic_match: f32,

    /// Benchmark evidence
    pub benchmark: f32,

    /// Test coverage evidence
    pub coverage: f32,

    /// Git history evidence (temporal)
    pub history: f32,

    /// Semantic similarity (variable confidence)
    pub semantic_base: f32,
}

impl Default for ConfidenceWeights {
    fn default() -> Self {
        Self {
            ast_exact_match: 1.0,      // 100% confidence
            ast_semantic_match: 0.85,   // 85% confidence
            benchmark: 0.7,             // 70% confidence
            coverage: 0.6,              // 60% confidence
            history: 0.5,               // 50% confidence
            semantic_base: 0.4,         // 40% base confidence
        }
    }
}

impl ConfidenceCalculator {
    /// Calculate refined confidence score
    ///
    /// # Algorithm
    ///
    /// Confidence is calculated as a weighted combination of:
    /// 1. Evidence type quality (AST > Benchmark > Coverage > History)
    /// 2. Semantic similarity score
    /// 3. Evidence freshness (recent = higher confidence)
    /// 4. Evidence consensus (multiple sources agreeing)
    ///
    /// # Examples
    ///
    /// ```rust
    /// let calc = ConfidenceCalculator::default();
    ///
    /// let result = ComprehensiveValidationResult {
    ///     evidence: ComprehensiveEvidence {
    ///         ast_evidence: vec![/* exact match */],
    ///         benchmark_evidence: vec![/* supporting data */],
    ///         // ...
    ///     },
    ///     // ...
    /// };
    ///
    /// let confidence = calc.calculate(&result)?;
    /// assert!(confidence > 0.9); // High confidence from multiple sources
    /// ```
    pub fn calculate(&self, result: &ComprehensiveValidationResult) -> Result<f32> {
        let mut confidence_scores = Vec::new();
        let mut weights = Vec::new();

        // 1. AST evidence
        if !result.evidence.ast_evidence.is_empty() {
            let ast_conf = self.calculate_ast_confidence(&result.evidence.ast_evidence, &result.claim)?;
            confidence_scores.push(ast_conf.score);
            weights.push(ast_conf.weight);
        }

        // 2. Benchmark evidence
        if !result.evidence.benchmark_evidence.is_empty() {
            let bench_conf = self.calculate_benchmark_confidence(
                &result.evidence.benchmark_evidence,
                &result.claim
            )?;
            confidence_scores.push(bench_conf);
            weights.push(self.weights.benchmark);
        }

        // 3. Coverage evidence
        if !result.evidence.coverage_evidence.is_empty() {
            let cov_conf = self.calculate_coverage_confidence(&result.evidence.coverage_evidence)?;
            confidence_scores.push(cov_conf);
            weights.push(self.weights.coverage);
        }

        // 4. History evidence (freshness check)
        if !result.evidence.history_evidence.is_empty() {
            let hist_conf = self.calculate_history_confidence(&result.evidence.history_evidence)?;
            confidence_scores.push(hist_conf);
            weights.push(self.weights.history);
        }

        // Calculate weighted average
        if confidence_scores.is_empty() {
            return Ok(0.0);
        }

        let weighted_sum: f32 = confidence_scores.iter()
            .zip(&weights)
            .map(|(score, weight)| score * weight)
            .sum();

        let weight_sum: f32 = weights.iter().sum();

        Ok(weighted_sum / weight_sum)
    }

    fn calculate_ast_confidence(
        &self,
        ast_facts: &[CodeFact],
        claim: &Claim
    ) -> Result<AstConfidence> {
        // Check for exact match first
        for fact in ast_facts {
            if Self::is_exact_match(&fact.description, &claim.text) {
                return Ok(AstConfidence {
                    score: self.weights.ast_exact_match,
                    weight: self.weights.ast_exact_match,
                });
            }
        }

        // Fall back to semantic similarity
        let similarities: Vec<f32> = ast_facts.iter()
            .map(|fact| self.semantic_similarity(&fact.description, &claim.text))
            .collect();

        let max_similarity = similarities.iter().cloned().fold(0.0f32, f32::max);

        Ok(AstConfidence {
            score: self.weights.ast_semantic_match * max_similarity,
            weight: self.weights.ast_semantic_match,
        })
    }

    fn is_exact_match(fact: &str, claim: &str) -> bool {
        // Normalize and compare
        let fact_norm = fact.to_lowercase().replace(|c: char| !c.is_alphanumeric(), "");
        let claim_norm = claim.to_lowercase().replace(|c: char| !c.is_alphanumeric(), "");

        claim_norm.contains(&fact_norm) || fact_norm.contains(&claim_norm)
    }

    fn calculate_benchmark_confidence(
        &self,
        benchmarks: &[BenchmarkFact],
        claim: &Claim
    ) -> Result<f32> {
        // If claim mentions performance, benchmark data is highly relevant
        if claim.text.contains("fast") || claim.text.contains("efficient") {
            // Extract performance threshold from claim if possible
            if let Some(threshold_ms) = Self::extract_performance_threshold(&claim.text) {
                // Check if benchmarks meet threshold
                let meets_threshold = benchmarks.iter()
                    .all(|b| b.median_time_ns < (threshold_ms * 1_000_000) as u64);

                if meets_threshold {
                    return Ok(0.95); // Very high confidence
                } else {
                    return Ok(0.1); // Low confidence (contradicts claim)
                }
            }
        }

        // Benchmark exists but doesn't directly validate claim
        Ok(0.5)
    }

    fn extract_performance_threshold(text: &str) -> Option<u64> {
        // Extract "< 100ms" style thresholds
        let re = Regex::new(r"< ?(\d+)\s*(ms|milliseconds?)").ok()?;
        let cap = re.captures(text)?;
        let ms = cap.get(1)?.as_str().parse::<u64>().ok()?;
        Some(ms)
    }
}

#[derive(Debug, Clone)]
struct AstConfidence {
    score: f32,
    weight: f32,
}
```

---

## Implementation Priority

Based on Toyota Way impact:

1. **P0 - Immediate**: Self-Validation (Enhancement 1)
2. **P0 - Immediate**: Intelligent Re-Validation (Enhancement 2)
3. **P1 - Sprint 1**: Extended Code Facts Database (Enhancement 4)
4. **P1 - Sprint 1**: Refined Confidence Scoring (Enhancement 7)
5. **P2 - Sprint 2**: Trend Analysis (Enhancement 5)
6. **P2 - Sprint 2**: Extensible Patterns (Enhancement 6)
7. **P3 - Future**: IDE Integration (Enhancement 3)

## Success Metrics

- **Jidoka**: 100% of documentation changes validated before commit
- **Genchi Genbutsu**: 3+ evidence sources per validation
- **Hansei**: Monthly trend reports reviewed in retrospectives
- **Kaizen**: Declining error rate month-over-month
- **Respect**: Teams contribute 5+ custom patterns within 3 months

---

**Next Steps**: Incorporate these enhancements into the main specification and implementation roadmap.
