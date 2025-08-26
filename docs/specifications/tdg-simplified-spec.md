# TDG (Technical Debt Grading) Scoring Tool Specification

## Executive Summary

The TDG scoring tool provides quantitative measurement of code quality, technical debt, and maintainability. Originally embedded in `pmat analyze`, TDG becomes a standalone subcommand for evaluating AI-generated code quality and comparing implementations.

## Command Structure

```bash
# Score single file
pmat tdg <file>

# Score project directory
pmat tdg <directory>

# Compare two files/directories
pmat tdg compare <source1> <source2>

# Custom configuration
pmat tdg --config tdg.toml <file>

# Output formats
pmat tdg --json <file>
pmat tdg --markdown <file>
pmat tdg --quiet <file>  # Score only
```

## Scoring Algorithm

### Core Metrics (100 points total)

```rust
pub struct TdgScore {
    // Primary orthogonal metrics - no overlap
    structural_complexity: f32,   // 25 points - Pure graph metrics
    semantic_complexity: f32,      // 20 points - Type/logic complexity  
    duplication_ratio: f32,        // 20 points - Code redundancy
    coupling_score: f32,           // 15 points - Dependencies
    doc_coverage: f32,             // 10 points - Documentation
    consistency_score: f32,        // 10 points - Style uniformity
    
    // Metadata
    total: f32,                   // 0-100
    grade: Grade,                  // A+ to F
    confidence: f32,               // 0-1 based on language support
    language: Language,
}

pub enum Grade {
    APLus,  // 95-100: Production excellence
    A,      // 90-94:  High quality
    AMinus, // 85-89:  Good quality
    BPlus,  // 80-84:  Above average
    B,      // 75-79:  Average
    BMinus, // 70-74:  Below average
    CPlus,  // 65-69:  Poor
    C,      // 60-64:  Very poor
    CMinus, // 55-59:  Problematic
    D,      // 50-54:  Severe issues
    F,      // 0-49:   Failing
}

// Penalty attribution to prevent double-counting
pub struct PenaltyAttribution {
    source_metric: MetricCategory,
    amount: f32,
    applied_to: HashSet<MetricCategory>,
}

impl TdgScorer {
    fn apply_penalty(&mut self, issue: CodeIssue) -> PenaltyAttribution {
        let primary_category = issue.primary_impact();
        let penalty = self.calculate_penalty(&issue);
        
        // Apply only to primary category
        self.scores[primary_category] -= penalty.amount;
        
        // Track to prevent re-penalization
        PenaltyAttribution {
            source_metric: primary_category,
            amount: penalty.amount,
            applied_to: HashSet::from([primary_category]),
        }
    }
}
```

### Structural Complexity (25 points)

```rust
impl StructuralComplexityScorer {
    fn score(&self, ast: &SyntaxTree) -> f32 {
        let mut points = 25.0;
        
        // Control flow graph complexity
        let cfg = build_control_flow_graph(ast);
        let cyclomatic = cfg.cyclomatic_complexity();
        
        // Logarithmic penalty for high complexity
        if cyclomatic > 10 {
            let penalty = ((cyclomatic as f32).ln() - 10_f32.ln()) * 2.0;
            points -= penalty.min(15.0);
        }
        
        // Nesting depth (max 5 point penalty)
        let max_nesting = ast.max_nesting_depth();
        if max_nesting > 3 {
            points -= ((max_nesting - 3) as f32).min(5.0);
        }
        
        // Number of branches (max 5 point penalty)
        let branch_count = cfg.branch_nodes().count();
        if branch_count > 20 {
            points -= ((branch_count - 20) as f32 * 0.1).min(5.0);
        }
        
        points.max(0.0)
    }
}
```

### Semantic Complexity (20 points)

```rust
impl SemanticComplexityScorer {
    fn score(&self, ast: &SyntaxTree) -> f32 {
        let mut points = 20.0;
        
        // Cognitive complexity (Sonar method)
        let cognitive = self.calculate_cognitive_complexity(ast);
        if cognitive > 15 {
            points -= ((cognitive - 15) as f32 * 0.5).min(10.0);
        }
        
        // Type complexity (generic depth, trait bounds)
        let type_complexity = self.analyze_type_complexity(ast);
        points -= (type_complexity * 2.0).min(5.0);
        
        // Expression complexity (nested ternaries, lambdas)
        let expr_complexity = self.analyze_expression_complexity(ast);
        points -= (expr_complexity * 2.0).min(5.0);
        
        points.max(0.0)
    }
    
    fn calculate_cognitive_complexity(&self, ast: &SyntaxTree) -> u32 {
        let mut complexity = 0;
        let mut nesting_level = 0;
        
        ast.walk(|node| {
            match node.kind() {
                "if_statement" | "while_loop" | "for_loop" => {
                    complexity += 1 + nesting_level;
                    nesting_level += 1;
                }
                "else_clause" => complexity += 1,
                "break" | "continue" => complexity += 1 + nesting_level,
                "catch_clause" => complexity += 1 + nesting_level,
                "binary_expression" if is_logical(node) => {
                    complexity += 1;
                }
                _ => {}
            }
            
            if closes_scope(node) {
                nesting_level = nesting_level.saturating_sub(1);
            }
        });
        
        complexity
    }
}
```

### Duplication Detection (20 points)

```rust
pub struct DuplicationDetector {
    min_token_sequence: usize,  // 50 tokens minimum
    similarity_threshold: f32,   // 0.85 similarity
}

impl DuplicationDetector {
    fn score(&self, ast: &SyntaxTree) -> f32 {
        let mut points = 20.0;
        
        // Type I: Exact clones (excluding whitespace/comments)
        let exact_clones = self.find_exact_clones(ast);
        
        // Type II: Renamed clones (parameter/variable names differ)
        let renamed_clones = self.find_renamed_clones(ast);
        
        // Type III: Modified clones (statements added/removed)
        let modified_clones = self.find_modified_clones(ast);
        
        let total_tokens = ast.token_count();
        let duplicate_tokens = exact_clones.total_tokens() 
            + renamed_clones.total_tokens() * 0.8
            + modified_clones.total_tokens() * 0.5;
        
        let duplication_ratio = duplicate_tokens as f32 / total_tokens as f32;
        
        // Progressive penalty
        points -= (duplication_ratio * 40.0).min(20.0);
        
        points.max(0.0)
    }
    
    fn find_renamed_clones(&self, ast: &SyntaxTree) -> CloneSet {
        // Normalize identifiers to detect Type II clones
        let normalized = self.normalize_ast(ast);
        let sequences = self.extract_token_sequences(&normalized, self.min_token_sequence);
        
        // Use suffix array for O(n log n) detection
        let suffix_array = build_suffix_array(&sequences);
        let clones = self.find_similar_sequences(&suffix_array, self.similarity_threshold);
        
        CloneSet::from(clones)
    }
    
    fn normalize_ast(&self, ast: &SyntaxTree) -> NormalizedAst {
        let mut normalized = ast.clone();
        
        normalized.visit_mut(|node| {
            match node.kind() {
                "identifier" if !is_type_name(node) => {
                    node.set_text("$VAR");
                }
                "string_literal" => {
                    node.set_text("$STR");
                }
                "number_literal" => {
                    node.set_text("$NUM");
                }
                _ => {}
            }
        });
        
        normalized
    }
}
```

### Coupling Analysis (15 points)

```rust
impl CouplingAnalyzer {
    fn score(&self, module: &Module) -> f32 {
        let mut points = 15.0;
        
        // Afferent coupling (incoming dependencies)
        let afferent = self.calculate_afferent_coupling(module);
        
        // Efferent coupling (outgoing dependencies)  
        let efferent = self.calculate_efferent_coupling(module);
        
        // Instability: I = Ce / (Ca + Ce)
        let instability = efferent as f32 / (afferent + efferent).max(1) as f32;
        
        // Distance from main sequence
        let abstractness = self.calculate_abstractness(module);
        let distance = (instability + abstractness - 1.0).abs();
        
        // Penalize high coupling and distance from main sequence
        if afferent + efferent > 10 {
            points -= ((afferent + efferent - 10) as f32 * 0.3).min(7.0);
        }
        
        points -= (distance * 8.0).min(8.0);
        
        points.max(0.0)
    }
    
    fn calculate_dependency_depth(&self, module: &Module) -> usize {
        let graph = self.build_dependency_graph(module);
        
        // Find longest path in DAG
        let topo_order = graph.topological_sort();
        let mut depths = HashMap::new();
        
        for node in topo_order {
            let max_pred_depth = graph.predecessors(node)
                .map(|pred| depths[pred])
                .max()
                .unwrap_or(0);
            depths.insert(node, max_pred_depth + 1);
        }
        
        *depths.values().max().unwrap_or(&0)
    }
}
```

### Documentation Coverage (10 points)

```rust
impl DocumentationScorer {
    fn score(&self, ast: &SyntaxTree, lang: Language) -> f32 {
        let adapter = self.get_language_adapter(lang);
        
        // Extract documentation based on language conventions
        let docs = adapter.extract_documentation(ast);
        
        // Count documentable items
        let public_items = adapter.find_public_items(ast);
        let documented_items = public_items.iter()
            .filter(|item| docs.has_documentation(item))
            .count();
        
        let coverage = documented_items as f32 / public_items.len().max(1) as f32;
        let mut points = coverage * 7.0;
        
        // Bonus for examples in documentation (max 2 points)
        let example_count = docs.count_examples();
        points += (example_count as f32 * 0.5).min(2.0);
        
        // Bonus for module-level documentation (1 point)
        if docs.has_module_documentation() {
            points += 1.0;
        }
        
        points.min(10.0)
    }
}

trait LanguageDocAdapter {
    fn extract_documentation(&self, ast: &SyntaxTree) -> Documentation;
    fn find_public_items(&self, ast: &SyntaxTree) -> Vec<SyntaxNode>;
}

struct RustDocAdapter;
impl LanguageDocAdapter for RustDocAdapter {
    fn extract_documentation(&self, ast: &SyntaxTree) -> Documentation {
        let mut docs = Documentation::new();
        
        ast.walk(|node| {
            if node.kind() == "line_comment" && node.text().starts_with("///") {
                docs.add_doc_comment(node);
            } else if node.kind() == "block_comment" && node.text().starts_with("/**") {
                docs.add_doc_comment(node);
            }
        });
        
        docs
    }
}
```

### Consistency Analysis (10 points)

```rust
impl ConsistencyAnalyzer {
    fn score(&self, ast: &SyntaxTree, lang: Language) -> f32 {
        let mut points = 10.0;
        let rules = self.get_language_rules(lang);
        
        // Naming convention consistency
        let naming_violations = self.check_naming_consistency(ast, &rules);
        points -= (naming_violations as f32 * 0.2).min(4.0);
        
        // Import organization
        let import_issues = self.check_import_organization(ast, &rules);
        points -= (import_issues as f32 * 0.3).min(2.0);
        
        // Pattern consistency (graduated scoring)
        let pattern_score = self.analyze_pattern_consistency(ast);
        points -= ((1.0 - pattern_score) * 4.0).min(4.0);
        
        points.max(0.0)
    }
    
    fn check_naming_consistency(&self, ast: &SyntaxTree, rules: &LanguageRules) -> u32 {
        let mut violations = 0;
        
        // Use tree-sitter queries for language-agnostic analysis
        let query = rules.naming_query();
        let captures = query.captures(ast.root_node());
        
        for capture in captures {
            let expected_style = match capture.name {
                "function_name" => rules.function_style(),
                "type_name" => rules.type_style(),
                "constant_name" => rules.constant_style(),
                "variable_name" => rules.variable_style(),
                _ => continue,
            };
            
            if !matches_style(capture.node.text(), expected_style) {
                violations += 1;
            }
        }
        
        violations
    }
    
    fn analyze_pattern_consistency(&self, ast: &SyntaxTree) -> f32 {
        // Detect inconsistent patterns (e.g., mixed error handling styles)
        let patterns = self.extract_patterns(ast);
        let consistency_scores = vec![
            self.error_handling_consistency(&patterns),
            self.null_check_consistency(&patterns),
            self.loop_style_consistency(&patterns),
            self.conditional_style_consistency(&patterns),
        ];
        
        // Return average consistency
        consistency_scores.iter().sum::<f32>() / consistency_scores.len() as f32
    }
}

pub struct LanguageRules {
    language: Language,
    naming_conventions: HashMap<NodeKind, NamingStyle>,
    import_rules: ImportRules,
    queries: HashMap<String, TreeSitterQuery>,
}

impl LanguageRules {
    fn for_language(lang: Language) -> Self {
        match lang {
            Language::Rust => Self::rust_rules(),
            Language::Python => Self::python_rules(),
            Language::JavaScript => Self::javascript_rules(),
            Language::Go => Self::go_rules(),
            _ => Self::default_rules(),
        }
    }
    
    fn rust_rules() -> Self {
        let mut naming = HashMap::new();
        naming.insert(NodeKind::Function, NamingStyle::SnakeCase);
        naming.insert(NodeKind::Type, NamingStyle::PascalCase);
        naming.insert(NodeKind::Constant, NamingStyle::ScreamingSnakeCase);
        
        LanguageRules {
            language: Language::Rust,
            naming_conventions: naming,
            import_rules: ImportRules::rust(),
            queries: Self::load_rust_queries(),
        }
    }
}
```

## Language Support Architecture

```rust
pub trait LanguageAdapter: Send + Sync {
    fn detect(&self, file: &Path) -> bool;
    fn parse(&self, source: &str) -> Result<SyntaxTree>;
    fn complexity_calculator(&self) -> Box<dyn ComplexityCalculator>;
    fn doc_extractor(&self) -> Box<dyn DocExtractor>;
    fn naming_rules(&self) -> LanguageRules;
    fn confidence(&self) -> f32;  // 0-1 scoring confidence for this language
}

pub struct LanguageRegistry {
    adapters: HashMap<Language, Box<dyn LanguageAdapter>>,
}

impl LanguageRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            adapters: HashMap::new(),
        };
        
        // Register adapters with confidence levels
        registry.register(Language::Rust, RustAdapter::new(), 1.0);
        registry.register(Language::Python, PythonAdapter::new(), 0.95);
        registry.register(Language::JavaScript, JsAdapter::new(), 0.90);
        registry.register(Language::Go, GoAdapter::new(), 0.95);
        registry.register(Language::TypeScript, TsAdapter::new(), 0.90);
        registry.register(Language::Java, JavaAdapter::new(), 0.85);
        registry.register(Language::C, CAdapter::new(), 0.80);
        registry.register(Language::Cpp, CppAdapter::new(), 0.75);
        
        registry
    }
}

impl TdgAnalyzer {
    pub fn analyze(&self, file: &Path) -> Result<TdgScore> {
        let language = self.detect_language(file)?;
        let adapter = self.registry.get_adapter(language)?;
        let confidence = adapter.confidence();
        
        let source = fs::read_to_string(file)?;
        let ast = adapter.parse(&source)?;
        
        // Score with language-specific adapters
        let mut score = TdgScore::default();
        score.language = language;
        score.confidence = confidence;
        
        // Each scorer uses the appropriate language adapter
        score.structural_complexity = self.structural_scorer.score(&ast);
        score.semantic_complexity = self.semantic_scorer.score(&ast);
        score.duplication_ratio = self.duplication_detector.score(&ast);
        score.coupling_score = self.coupling_analyzer.score(&ast);
        score.doc_coverage = self.doc_scorer.score(&ast, language);
        score.consistency_score = self.consistency_analyzer.score(&ast, language);
        
        score.calculate_total();
        Ok(score)
    }
}
```

## Output Formats

### Human-Readable Output

```
╭─────────────────────────────────────────────────╮
│  TDG Score Report: src/analyzer.rs              │
├─────────────────────────────────────────────────┤
│  Overall Score: 82.5/100 (B+)                  │
│  Language: Rust (confidence: 100%)             │
│                                                 │
│  📊 Breakdown:                                  │
│  ├─ Structural:     21.5/25  ████████░░        │
│  ├─ Semantic:       16.0/20  ████████░░        │
│  ├─ Duplication:    17.0/20  ████████░░        │
│  ├─ Coupling:       12.0/15  ████████░░        │
│  ├─ Documentation:   8.0/10  ████████░░        │
│  └─ Consistency:     8.0/10  ████████░░        │
│                                                 │
│  🔍 Key Issues (no double penalties):          │
│  • High cyclomatic complexity: 15 (line 45)    │
│  • Code duplication: 12% Type-II clones        │
│  • Missing docs: 3 public functions            │
│                                                 │
│  ✨ Strengths:                                  │
│  • Low coupling (I=0.3, D=0.1)                 │
│  • Consistent naming conventions               │
│  • No cognitive complexity issues              │
╰─────────────────────────────────────────────────╯
```

### JSON Output

```json
{
  "file": "src/analyzer.rs",
  "language": "rust",
  "confidence": 1.0,
  "score": {
    "total": 82.5,
    "grade": "B+",
    "breakdown": {
      "structural_complexity": {
        "score": 21.5,
        "max": 25,
        "details": {
          "cyclomatic_complexity": 15,
          "max_nesting": 3,
          "branch_count": 18
        }
      },
      "semantic_complexity": {
        "score": 16.0,
        "max": 20,
        "details": {
          "cognitive_complexity": 12,
          "type_complexity": 0.2,
          "expression_complexity": 0.3
        }
      },
      "duplication": {
        "score": 17.0,
        "max": 20,
        "details": {
          "exact_clones": 0.02,
          "renamed_clones": 0.08,
          "modified_clones": 0.05,
          "total_ratio": 0.12
        }
      },
      "coupling": {
        "score": 12.0,
        "max": 15,
        "details": {
          "afferent": 3,
          "efferent": 5,
          "instability": 0.625,
          "distance": 0.125
        }
      },
      "documentation": {
        "score": 8.0,
        "max": 10,
        "coverage": 0.70,
        "examples": 2
      },
      "consistency": {
        "score": 8.0,
        "max": 10,
        "naming_violations": 2,
        "pattern_consistency": 0.85
      }
    }
  },
  "penalties_applied": [
    {
      "issue": "high_cyclomatic_complexity",
      "category": "structural_complexity",
      "amount": 3.5
    },
    {
      "issue": "code_duplication",
      "category": "duplication",
      "amount": 3.0
    }
  ],
  "timestamp": "2024-01-17T10:30:00Z"
}
```

### Comparison Output

```
╭─────────────────────────────────────────────────╮
│  TDG Comparison: ai_v1.rs vs ai_v2.rs          │
├─────────────────────────────────────────────────┤
│                     ai_v1.rs   ai_v2.rs    Δ   │
│  Overall Score:       72.5      85.0    +12.5  │
│  Grade:                B-        A-       ↑2   │
│  Confidence:         100%      100%            │
│                                                 │
│  Structural:          18.0      23.5    +5.5   │
│  Semantic:            14.0      17.0    +3.0   │
│  Duplication:         15.0      18.5    +3.5   │
│  Coupling:            11.0      13.0    +2.0   │
│  Documentation:        7.5       7.0    -0.5   │
│  Consistency:          7.0       6.0    -1.0   │
│                                                 │
│  Winner: ai_v2.rs (17.2% improvement)          │
│                                                 │
│  Key Improvements:                             │
│  • Reduced cyclomatic complexity (15→8)        │
│  • Lower code duplication (18%→9%)             │
│  • Better semantic clarity                     │
│                                                 │
│  Minor Regressions:                            │
│  • Documentation coverage decreased            │
│  • Import organization degraded                │
╰─────────────────────────────────────────────────╯
```

## Configuration File

```toml
# tdg.toml - Custom scoring configuration
[weights]
structural_complexity = 25
semantic_complexity = 20
duplication = 20
coupling = 15
documentation = 10
consistency = 10

[thresholds]
max_cyclomatic_complexity = 10
max_cognitive_complexity = 15
max_nesting_depth = 3
min_token_sequence = 50
similarity_threshold = 0.85
max_coupling = 10
min_doc_coverage = 0.8

[penalties]
# Logarithmic penalties for exponential problems
complexity_penalty_base = "logarithmic"
duplication_penalty_curve = "linear"
coupling_penalty_curve = "quadratic"

[language_overrides.python]
# Python-specific adjustments
max_cognitive_complexity = 20  # Higher due to comprehensions
min_doc_coverage = 0.9  # Stricter for docstrings

[language_overrides.go]
# Go-specific adjustments
enforce_error_check = true
max_function_length = 40  # Go prefers smaller functions
```

## Implementation Architecture

```rust
pub struct TdgAnalyzer {
    config: TdgConfig,
    registry: LanguageRegistry,
    scorers: ScorerSet,
    cache: DashMap<PathBuf, CachedScore>,
    penalty_tracker: PenaltyTracker,
}

impl TdgAnalyzer {
    pub fn analyze_file(&self, path: &Path) -> Result<TdgScore> {
        // Check cache with file hash
        let file_hash = hash_file(path)?;
        if let Some(cached) = self.cache.get(path) {
            if cached.hash == file_hash {
                return Ok(cached.score.clone());
            }
        }
        
        // Detect language and get adapter
        let language = detect_language(path)?;
        let adapter = self.registry.get_adapter(language)?;
        
        // Parse with language-specific parser
        let source = fs::read_to_string(path)?;
        let ast = adapter.parse(&source)?;
        
        // Initialize penalty tracker
        let mut tracker = PenaltyTracker::new();
        
        // Score with orthogonal metrics
        let mut score = TdgScore::default();
        score.language = language;
        score.confidence = adapter.confidence();
        
        // Apply each scorer with penalty tracking
        for scorer in &self.scorers {
            let metric_score = scorer.score(&ast, &mut tracker)?;
            score.set_metric(scorer.category(), metric_score);
        }
        
        score.calculate_total();
        
        // Cache result
        self.cache.insert(path.to_path_buf(), CachedScore {
            score: score.clone(),
            hash: file_hash,
        });
        
        Ok(score)
    }
    
    pub fn analyze_project(&self, dir: &Path) -> Result<ProjectScore> {
        let files = self.discover_files(dir)?;
        let mut scores = Vec::new();
        
        // Parallel analysis with rayon
        let results: Vec<_> = files
            .par_iter()
            .map(|file| self.analyze_file(file))
            .collect();
        
        for result in results {
            scores.push(result?);
        }
        
        Ok(ProjectScore::aggregate(scores))
    }
    
    pub fn compare(&self, path1: &Path, path2: &Path) -> Result<Comparison> {
        let score1 = if path1.is_dir() {
            self.analyze_project(path1)?.average()
        } else {
            self.analyze_file(path1)?
        };
        
        let score2 = if path2.is_dir() {
            self.analyze_project(path2)?.average()
        } else {
            self.analyze_file(path2)?
        };
        
        Ok(Comparison::new(score1, score2))
    }
}

// Penalty tracking to prevent double-counting
pub struct PenaltyTracker {
    applied: HashMap<IssueId, PenaltyAttribution>,
}

impl PenaltyTracker {
    pub fn apply(&mut self, issue: CodeIssue) -> Option<f32> {
        if self.applied.contains_key(&issue.id()) {
            return None;  // Already penalized
        }
        
        let penalty = issue.calculate_penalty();
        self.applied.insert(issue.id(), PenaltyAttribution {
            source_metric: issue.primary_category(),
            amount: penalty,
            applied_to: HashSet::from([issue.primary_category()]),
        });
        
        Some(penalty)
    }
}
```

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_no_double_penalties() {
        let code = r#"
            fn complex_function(x: i32) -> i32 {
                // High complexity function
                if x > 0 {
                    if x > 10 {
                        if x > 20 {
                            if x > 30 {
                                return x * 2;
                            }
                        }
                    }
                }
                x
            }
        "#;
        
        let analyzer = TdgAnalyzer::new();
        let score = analyzer.analyze_str(code).unwrap();
        
        // Verify only structural complexity is penalized
        assert!(score.structural_complexity < 25.0);
        assert_eq!(score.semantic_complexity, 20.0); // No cascade
        assert_eq!(score.coupling_score, 15.0); // No cascade
    }
    
    #[test]
    fn test_duplication_detection() {
        let code = r#"
            fn process_a(x: i32) -> i32 {
                let result = x * 2;
                if result > 100 {
                    return result + 10;
                }
                result
            }
            
            fn process_b(y: i32) -> i32 {
                let result = y * 2;
                if result > 100 {
                    return result + 10;
                }
                result
            }
        "#;
        
        let analyzer = TdgAnalyzer::new();
        let score = analyzer.analyze_str(code).unwrap();
        
        // Should detect Type-II clone (renamed variables)
        assert!(score.duplication_ratio < 20.0);
        let details = score.duplication_details();
        assert!(details.renamed_clones > 0.0);
    }
    
    #[test]
    fn test_language_detection() {
        let files = vec![
            ("test.rs", Language::Rust),
            ("test.py", Language::Python),
            ("test.js", Language::JavaScript),
            ("test.go", Language::Go),
            ("test.ts", Language::TypeScript),
        ];
        
        for (filename, expected) in files {
            let detected = detect_language(Path::new(filename)).unwrap();
            assert_eq!(detected, expected);
        }
    }
    
    #[test]
    fn test_graduated_scoring() {
        let analyzer = TdgAnalyzer::new();
        
        // Test graduated complexity penalties
        let scores: Vec<_> = (5..=25).step_by(5)
            .map(|complexity| {
                let code = generate_function_with_complexity(complexity);
                analyzer.analyze_str(&code).unwrap().structural_complexity
            })
            .collect();
        
        // Verify logarithmic penalty curve
        for i in 1..scores.len() {
            let delta = scores[i-1] - scores[i];
            assert!(delta > 0.0); // Score decreases
            if i > 1 {
                let prev_delta = scores[i-2] - scores[i-1];
                assert!(delta < prev_delta); // Decreasing penalty rate
            }
        }
    }
    
    #[test]
    fn test_cache_performance() {
        let analyzer = TdgAnalyzer::new();
        let path = Path::new("test.rs");
        
        // First call - no cache
        let start = Instant::now();
        let score1 = analyzer.analyze_file(path).unwrap();
        let uncached_time = start.elapsed();
        
        // Second call - cached
        let start = Instant::now();
        let score2 = analyzer.analyze_file(path).unwrap();
        let cached_time = start.elapsed();
        
        assert_eq!(score1, score2);
        assert!(cached_time < uncached_time / 10); // 10x faster
    }
}
```

## Performance Requirements

- Single file analysis: <50ms
- Project analysis (1000 files): <5s with parallel processing
- Comparison: <100ms
- Cache hit: <1ms
- Memory usage: O(n) where n is file size

## Quality Requirements

- Test coverage: ≥80%
- Max cyclomatic complexity: 10
- Documentation coverage: 100% for public API
- Zero clippy warnings with pedantic lints
- Zero unsafe blocks
- Memory safety guaranteed via Rust ownership

## CLI Integration

```rust
// In pmat/src/cli.rs
#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...
    
    /// Grade technical debt and code quality
    Tdg {
        /// File or directory to analyze
        path: PathBuf,
        
        #[command(subcommand)]
        command: Option<TdgCommand>,
        
        /// Output format
        #[arg(long, value_enum)]
        format: Option<OutputFormat>,
        
        /// Configuration file
        #[arg(long)]
        config: Option<PathBuf>,
        
        /// Quiet mode (score only)
        #[arg(short, long)]
        quiet: bool,
        
        /// Minimum grade to pass (for CI)
        #[arg(long)]
        min_grade: Option<Grade>,
    },
}

#[derive(Subcommand)]
enum TdgCommand {
    /// Compare two files or directories
    Compare {
        source1: PathBuf,
        source2: PathBuf,
    },
}

impl TdgCommand {
    pub fn execute(self) -> Result<()> {
        let analyzer = TdgAnalyzer::from_config(self.config)?;
        
        match self.command {
            Some(TdgCommand::Compare { source1, source2 }) => {
                let comparison = analyzer.compare(&source1, &source2)?;
                self.output_comparison(comparison)?;
            }
            None => {
                let score = if self.path.is_dir() {
                    analyzer.analyze_project(&self.path)?
                } else {
                    analyzer.analyze_file(&self.path)?
                };
                
                if let Some(min_grade) = self.min_grade {
                    if score.grade < min_grade {
                        return Err(anyhow!("Grade {} below minimum {}", 
                            score.grade, min_grade));
                    }
                }
                
                self.output_score(score)?;
            }
        }
        
        Ok(())
    }
}
```

## Deliverables

1. **Core Implementation** (3 days)
    - Orthogonal scoring system with penalty tracking
    - Language-agnostic analysis via tree-sitter
    - Graduated scoring algorithms
    - Caching and performance optimization

2. **Language Support** (2 days)
    - Adapters for 8+ languages
    - Language-specific rules and queries
    - Confidence scoring per language

3. **CLI Integration** (1 day)
    - Subcommand implementation
    - Output formatters (human, JSON, markdown)
    - CI/CD grade enforcement

4. **Testing** (1 day)
    - Unit tests (>80% coverage)
    - Integration tests for each language
    - Performance benchmarks
    - No-double-penalty verification

5. **Documentation** (1 day)
    - User guide with examples
    - API documentation
    - Language support matrix
    - Configuration reference

## Success Criteria

- Accurately grades AI-generated code without double penalties
- Provides graduated, nuanced scoring
- Language-agnostic with high confidence for major languages
- Maintains <50ms performance for typical files
- Achieves 80% test coverage with max complexity of 10
- Zero technical debt in implementation

---

*TDG: Orthogonal, language-aware code quality measurement*