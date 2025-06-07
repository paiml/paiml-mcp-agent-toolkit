# Post-Kaizen-022 Continuation Specification

## Executive Summary

Critical regressions detected: AST parsers retain 56-function match expressions (CC=260), CLI module contains 146 functions in single file (TDG 3.09), and MCP protocol lacks vectorized tool exposure. Metric accuracy compromised: uniform TDG values (2.44) indicate simplistic LOC multiplication, impossible cognitive:cyclomatic ratios (96:60), and dead code detection ignores FFI boundaries. This checklist guides remediation through verified computation engines.

## Current State Validation

- [x] **Verify 11 CLI commands operational**
  ```bash
  for cmd in duplicates defect-probability graph-metrics name-similarity \
             proof-annotations incremental-coverage symbol-table comprehensive \
             quality-gate makefile tdg; do
    pmat analyze $cmd --help >/dev/null 2>&1 || echo "FAILED: $cmd"
  done
  ```

- [x] **Baseline metrics captured**
  - AST parser complexity: `ast_cpp.rs` CC=260, `ast_c.rs` CC=190
  - CLI module: 273 symbols, 146 functions, TDG 3.09
  - Binary size: 13MB release build
  - Analysis performance: 143ms for 200 files

## Phase 0: Metric Accuracy Test Suite

### Setup Verification Framework
- [ ] Create `tests/metric_accuracy_suite.rs`:
  ```rust
  #[test]
  fn test_tdg_variance() {
      let files = vec!["simple.rs", "complex.rs", "medium.rs"];
      let tdgs: Vec<f64> = files.iter()
          .map(|f| calculate_tdg(f).value)
          .collect();
      let variance = statistical::variance(&tdgs);
      assert!(variance > 0.5, "TDG variance {:.3} too low", variance);
  }
  ```

- [ ] Create `tests/fixtures/` with known complexity:
  - [ ] `simple.rs`: Single function, no loops (expected CC=1)
  - [ ] `complex.rs`: Nested loops, high branching (expected CC>20)
  - [ ] `ffi_export.rs`: `#[no_mangle]` functions (should NOT be dead)

- [ ] Add complexity ratio validator:
  ```rust
  #[test]
  fn test_cognitive_bounds() {
      let ast = parse_file("server/src/cli/mod.rs");
      for func in ast.functions() {
          let ratio = func.cognitive as f64 / func.cyclomatic as f64;
          assert!(ratio >= 1.1 && ratio <= 2.0,
              "{}: impossible ratio {:.2}", func.name, ratio);
      }
  }
  ```

- [ ] Document current failures in `METRIC_FAILURES.md`:
  ```bash
  pmat analyze comprehensive server/ --json | \
    jq '.files[] | select(.tdg >= 2.43 and .tdg <= 2.45)' > uniform_tdg.json
  ```

## Phase 1: Verified Metrics Engine

### Day 1: TDG Multi-Factor Calculator
- [ ] Create `server/src/services/tdg_calculator.rs`
- [ ] Implement complexity variance calculation:
  ```rust
  fn compute_complexity_gradient(&self, ast: &UnifiedAst) -> ComplexityVariance {
      let complexities: Vec<u32> = ast.functions()
          .map(|f| f.complexity.cyclomatic)
          .collect();
      
      ComplexityVariance {
          mean: statistical::mean(&complexities),
          variance: statistical::variance(&complexities),
          gini: statistical::gini_coefficient(&complexities),
          percentile_90: statistical::percentile(&complexities, 0.9),
      }
  }
  ```

- [ ] Add coupling analyzer:
  ```rust
  impl CouplingAnalyzer {
      fn analyze(&self, file: &Path, ast: &UnifiedAst) -> CouplingMetrics {
          let imports = self.extract_imports(ast);
          let exports = self.extract_exports(ast);
          CouplingMetrics {
              afferent: self.count_incoming_dependencies(file, &imports),
              efferent: exports.len(),
              instability: efferent as f64 / (afferent + efferent) as f64,
          }
      }
  }
  ```

- [ ] Integrate git churn analysis:
  - [ ] Use `git2` crate for history traversal
  - [ ] Calculate monthly commit rate
  - [ ] Apply logarithmic normalization

- [ ] Test multi-factor TDG:
  ```bash
  cargo test tdg_calculator -- --nocapture
  # Verify: different files produce different TDG values
  ```

### Day 2: Verified Complexity Analyzer
- [ ] Create `server/src/services/verified_complexity.rs`
- [ ] Implement cognitive complexity per Sonar rules:
  ```rust
  fn compute_cognitive_weight(&self, node: &AstNode) -> u32 {
      match node.kind {
          AstKind::If | AstKind::Loop => {
              1 + self.nesting_level + 
              node.children().map(|c| self.compute_cognitive_weight(c)).sum()
          }
          AstKind::LogicalAnd | AstKind::LogicalOr => {
              1 + node.children().map(|c| self.compute_cognitive_weight(c)).sum()
          }
          _ => node.children().map(|c| self.compute_cognitive_weight(c)).sum()
      }
  }
  ```

- [ ] Add essential complexity (remove linear paths):
  ```rust
  fn compute_essential(&self, ast: &AstNode, cyclomatic: u32) -> u32 {
      let linear_paths = self.count_linear_paths(ast);
      cyclomatic.saturating_sub(linear_paths)
  }
  ```

- [ ] Implement Halstead metrics:
  - [ ] Count operators and operands
  - [ ] Calculate volume, difficulty, effort
  - [ ] Store in 16-byte struct

- [ ] Add debug assertions for sanity checks:
  ```rust
  debug_assert!(cognitive >= cyclomatic, "Cognitive < cyclomatic");
  debug_assert!(cognitive <= cyclomatic * 2, "Cognitive > 2x cyclomatic");
  debug_assert!(essential <= cyclomatic, "Essential > cyclomatic");
  ```

### Day 3: Dead Code Prover
- [ ] Create `server/src/services/dead_code_prover.rs`
- [ ] Implement reachability analyzer:
  ```rust
  impl ReachabilityAnalyzer {
      fn find_entry_points(&self, ast: &UnifiedAst) -> Vec<SymbolId> {
          ast.items().filter_map(|item| match item {
              Item::Main => Some(item.id),
              Item::Test => Some(item.id),
              Item::Benchmark => Some(item.id),
              Item::Binary => Some(item.id),
              _ => None
          }).collect()
      }
  }
  ```

- [ ] Add FFI reference tracker:
  - [ ] Scan for `#[no_mangle]` attributes
  - [ ] Check `extern "C"` blocks
  - [ ] Detect `wasm_bindgen` annotations
  - [ ] Find PyO3 `#[pyfunction]` markers

- [ ] Implement dynamic dispatch detector:
  ```rust
  impl DynamicDispatchAnalyzer {
      fn find_trait_object_usage(&self, symbol: SymbolId) -> Option<Usage> {
          // Check if symbol implements trait used in dyn Trait
          // Check if symbol address taken for fn pointer
          // Check if symbol in vtable
      }
  }
  ```

- [ ] Create proof generation:
  ```rust
  struct DeadCodeProof {
      item: SymbolId,
      proof_type: DeadCodeProofType,
      confidence: f64,
      evidence: Vec<Evidence>,
  }
  ```

## Phase 2: AST Parser Refactoring

### Day 4: C++ Parser Dispatch Table
- [ ] Create dispatch table infrastructure:
  ```rust
  static CPP_DISPATCH: Lazy<DashMap<&'static str, NodeMapper>> = Lazy::new(|| {
      let map = DashMap::with_capacity(64);
      // Register all 56 node types
      map
  });
  ```

- [ ] Extract mapper functions (56 total):
  - [ ] `map_function_definition`
  - [ ] `map_class_specifier`
  - [ ] `map_template_declaration`
  - [ ] `map_namespace_definition`
  - [ ] `map_method_declaration`
  - [ ] `map_constructor`
  - [ ] `map_destructor`
  - [ ] `map_field_declaration`
  - [ ] `map_enum_specifier`
  - [ ] `map_using_declaration`
  - [ ] ... (46 more)

- [ ] Update main dispatch logic:
  ```rust
  pub fn node_to_ast(&self, node: &Node, source: &str) -> Result<Option<UnifiedAstNode>> {
      if is_trivia(node.kind()) { return Ok(None); }
      
      if let Some(mapper) = CPP_DISPATCH.get(node.kind()) {
          let mut ast_node = mapper(node, source)?;
          self.post_process(&mut ast_node, node, source);
          Ok(Some(ast_node))
      } else {
          Ok(None)
      }
  }
  ```

- [ ] Verify complexity reduction:
  ```bash
  pmat analyze complexity server/src/services/ast_cpp.rs
  # Target: CC < 180 (from 260)
  ```

### Day 5: C Parser Dispatch Table
- [ ] Create C-specific dispatch table (fewer entries):
  ```rust
  static C_DISPATCH: Lazy<DashMap<&'static str, NodeMapper>> = Lazy::new(|| {
      let map = DashMap::with_capacity(32);
      // C has ~40 node types vs C++ ~60
      map
  });
  ```

- [ ] Extract C mapper functions (39 total):
  - [ ] `map_c_function`
  - [ ] `map_c_struct`
  - [ ] `map_c_enum`
  - [ ] `map_c_typedef`
  - [ ] `map_c_union`
  - [ ] ... (34 more)

- [ ] Verify complexity reduction:
  ```bash
  pmat analyze complexity server/src/services/ast_c.rs
  # Target: CC < 140 (from 190)
  ```

### Day 6: TypeScript Parser Decomposition
- [ ] Create pipeline architecture:
  ```rust
  pub struct TypeScriptAnalysisPipeline {
      symbol_extractor: SymbolExtractor,
      complexity_calculator: ComplexityCalculator,
      type_resolver: TypeResolver,
      import_analyzer: ImportAnalyzer,
  }
  ```

- [ ] Extract analysis stages:
  - [ ] `SymbolExtractor` (20 LOC)
  - [ ] `ComplexityCalculator` (30 LOC)
  - [ ] `TypeResolver` (40 LOC)
  - [ ] `ImportAnalyzer` (25 LOC)

- [ ] Update `analyze_with_swc` to use pipeline:
  ```rust
  pub fn analyze_with_swc(&self, source: &str) -> Result<FileContext> {
      let module = self.parse_module(source)?;
      self.pipeline.analyze(module)
  }
  ```

- [ ] Verify complexity reduction:
  ```bash
  pmat analyze complexity server/src/services/ast_typescript.rs
  # Target: CC < 100 (from 108)
  ```

## Phase 3: CLI Module Decomposition

### Day 7: Command Structure Setup
- [ ] Create directory structure:
  ```bash
  mkdir -p server/src/cli/commands/{analyze,generate,demo}
  mkdir -p server/src/cli/output
  ```

- [ ] Create base traits in `commands/mod.rs`:
  ```rust
  #[async_trait]
  pub trait CommandHandler: Send + Sync {
      type Args;
      type Output;
      async fn handle(&self, args: Self::Args) -> Result<Self::Output>;
  }
  ```

- [ ] Create `CommandRegistry`:
  ```rust
  pub struct CommandRegistry {
      analyze: HashMap<&'static str, Box<dyn AnalyzeHandler>>,
      generate: HashMap<&'static str, Box<dyn GenerateHandler>>,
  }
  ```

### Day 8: Extract Analyze Commands
- [ ] Create handler for each analyze subcommand:
  - [ ] `analyze/duplicates.rs` - `DuplicateHandler`
  - [ ] `analyze/defects.rs` - `DefectHandler`
  - [ ] `analyze/graph.rs` - `GraphMetricsHandler`
  - [ ] `analyze/complexity.rs` - `ComplexityHandler`
  - [ ] `analyze/comprehensive.rs` - `ComprehensiveHandler`
  - [ ] `analyze/name_similarity.rs` - `NameSimilarityHandler`
  - [ ] `analyze/symbol_table.rs` - `SymbolTableHandler`
  - [ ] `analyze/proof_annotations.rs` - `ProofHandler`
  - [ ] `analyze/incremental_coverage.rs` - `CoverageHandler`
  - [ ] `analyze/quality_gate.rs` - `QualityGateHandler`
  - [ ] `analyze/makefile.rs` - `MakefileHandler`
  - [ ] `analyze/tdg.rs` - `TdgHandler`
  - [ ] `analyze/dead_code.rs` - `DeadCodeHandler`
  - [ ] `analyze/satd.rs` - `SatdHandler`
  - [ ] `analyze/deep_context.rs` - `DeepContextHandler`

- [ ] Update main CLI to use registry:
  ```rust
  pub fn run(args: CliArgs) -> Result<()> {
      let registry = build_command_registry();
      registry.dispatch(args.command)
  }
  ```

### Day 9: Complete CLI Refactoring
- [ ] Extract generate commands:
  - [ ] `generate/template.rs`
  - [ ] `generate/context.rs`
  - [ ] `generate/mermaid.rs`

- [ ] Extract demo command:
  - [ ] `demo/runner.rs`

- [ ] Move formatting to `output/`:
  - [ ] `output/formatters.rs`
  - [ ] `output/progress.rs`

- [ ] Reduce `cli/mod.rs` to <100 LOC

- [ ] Verify complexity reduction:
  ```bash
  pmat analyze complexity server/src/cli/
  # Target: No file > 500 LOC, mod.rs < 100 LOC
  ```

## Phase 4: Deep Context Integration

### Day 10: Create Orchestrator
- [ ] Create `server/src/services/deep_context/orchestrator.rs`
- [ ] Implement parallel AST building:
  ```rust
  async fn build_unified_dag(&self, paths: &[PathBuf]) -> Result<Arc<AstDag>> {
      let semaphore = Arc::new(Semaphore::new(num_cpus::get() * 2));
      let dag = Arc::new(AstDag::new());
      
      let tasks: Vec<_> = paths.iter().map(|path| {
          let sem = semaphore.clone();
          let dag = dag.clone();
          spawn(async move {
              let _permit = sem.acquire().await?;
              let ast = parse_file(path).await?;
              dag.insert(path, ast);
              Ok::<(), Error>(())
          })
      }).collect();
      
      futures::future::try_join_all(tasks).await?;
      Ok(dag)
  }
  ```

- [ ] Integrate with `CodeIntelligence` engine:
  ```rust
  let request = AnalysisRequest {
      dag: dag.clone(),
      features: FeatureFlags::all(),
  };
  let report = self.intelligence.analyze_comprehensive(request).await?;
  ```

- [ ] Add cache integration:
  - [ ] Use `VectorizedCacheKey` for lookups
  - [ ] Cache analysis results
  - [ ] Implement cache invalidation

### Day 11: Wire Deep Context
- [ ] Update `analyze_deep_context` command handler
- [ ] Add performance instrumentation:
  ```rust
  let start = Instant::now();
  let result = orchestrator.analyze(config).await?;
  info!("Deep context analysis: {} files in {:?}", 
        result.file_count, start.elapsed());
  ```

- [ ] Test with large codebases:
  ```bash
  time pmat analyze deep-context ~/large-project --all-analyses
  # Target: <10s for 500 files
  ```

## Phase 5: Big-O Complexity Analysis

### Day 12: Data Structures
- [ ] Create `server/src/models/complexity_bound.rs`:
  ```rust
  #[repr(C, align(8))]
  pub struct ComplexityBound {
      pub class: BigOClass,           // 1 byte
      pub coefficient: u16,           // 2 bytes
      pub input_var: InputVariable,   // 1 byte
      pub confidence: u8,             // 1 byte (0-100%)
      pub flags: ComplexityFlags,     // 1 byte
      _padding: [u8; 2],              // 2 bytes
  }
  ```

- [ ] Update `NodeMetadata` enum:
  ```rust
  Function { 
      complexity: u16,
      param_count: u8,
      time_bound: ComplexityBound,      // 8 bytes
      space_bound: ComplexityBound,     // 8 bytes
      cache_complexity: CacheComplexity, // 8 bytes
  }
  ```

### Day 13: Pattern Recognition
- [ ] Create `server/src/services/complexity_patterns.rs`
- [ ] Implement pattern database:
  - [ ] Linear patterns (array iteration)
  - [ ] Quadratic patterns (nested loops)
  - [ ] Logarithmic patterns (binary search)
  - [ ] Linearithmic patterns (merge sort)
  - [ ] Exponential patterns (recursive fibonacci)

- [ ] Add pattern matcher:
  ```rust
  impl PatternMatcher {
      fn match_pattern(&self, ast: &UnifiedAstNode) -> Option<&ComplexityPattern> {
          for (name, pattern) in &self.patterns {
              if self.matches_ast(ast, &pattern.ast_pattern) {
                  return Some(pattern);
              }
          }
          None
      }
  }
  ```

### Day 14: Recurrence Solver
- [ ] Implement Master Theorem:
  ```rust
  fn solve_master_theorem(&self, a: u32, b: u32, k: u32, p: i32) -> ComplexityBound {
      let log_b_a = (a as f64).log(b as f64);
      match log_b_a.partial_cmp(&(k as f64)) {
          Some(Ordering::Less) => ComplexityBound::polynomial(k, p),
          Some(Ordering::Equal) => ComplexityBound::polynomial_log(k, p + 1),
          Some(Ordering::Greater) => ComplexityBound::polynomial(log_b_a),
          None => ComplexityBound::unknown(),
      }
  }
  ```

- [ ] Add recurrence extraction:
  ```rust
  fn extract_recurrence(&self, ast: &UnifiedAstNode) -> Option<RecurrenceRelation> {
      if !ast.is_recursive() { return None; }
      
      let calls = self.find_recursive_calls(ast);
      let work = self.compute_non_recursive_work(ast);
      
      Some(RecurrenceRelation { calls, work })
  }
  ```

### Day 15: Complexity Prover
- [ ] Create `server/src/services/complexity_prover.rs`
- [ ] Integrate Z3 for verification:
  ```rust
  fn verify_bound(&self, ast: &UnifiedAstNode, bound: ComplexityBound) 
      -> Result<ComplexityProof> 
  {
      let constraints = self.encode_verification_conditions(ast, bound);
      self.z3.push();
      
      for c in constraints {
          self.z3.assert(&c);
      }
      
      match self.z3.check() {
          SatResult::Unsat => Ok(ComplexityProof::Verified),
          SatResult::Sat => self.try_empirical_validation(ast, bound),
          SatResult::Unknown => Ok(ComplexityProof::Heuristic),
      }
  }
  ```

- [ ] Add empirical validator:
  - [ ] Generate worst-case inputs
  - [ ] Count operations
  - [ ] Fit curve to measurements

## Phase 6: Enhanced Reporting

### Day 16: Update Deep Context Format
- [ ] Modify `format_ast_analysis` in `deep_context.rs`:
  - [ ] Add TDG component breakdown
  - [ ] Include Big-O annotations
  - [ ] Show verification status
  - [ ] Add confidence levels

- [ ] Create complexity hotspot table:
  ```rust
  writeln!(output, "| Function | Time | Space | Confidence | Verified |");
  for func in &analysis.functions {
      writeln!(output, "| {} | {} | {} | {:.0}% | {} |",
          func.name,
          format_big_o(&func.time_bound),
          format_big_o(&func.space_bound),
          func.confidence * 100.0,
          if func.verified { "✓" } else { "✗" }
      );
  }
  ```

### Day 17: JSON Output for LLMs
- [ ] Create LLM-optimized format:
  ```rust
  #[derive(Serialize)]
  pub struct LLMComplexityReport {
      pub summary: ComplexitySummary,
      pub hotspots: Vec<Hotspot>,
      pub recommendations: Vec<Recommendation>,
      pub proofs: Vec<ComplexityProof>,
  }
  ```

- [ ] Add natural language descriptions:
  ```rust
  fn describe_complexity(&self, bound: &ComplexityBound) -> String {
      match bound.class {
          BigOClass::ON2 => format!(
              "Quadratic time complexity O(n²) with coefficient {}. \
               Performance degrades rapidly with input size.",
              bound.coefficient
          ),
          // ...
      }
  }
  ```

## Phase 7: MCP Protocol Updates

### Day 18: Tool Registration
- [ ] Update `handlers/tools.rs`:
  ```rust
  fn initialize_tool_registry() -> ToolRegistry {
      let mut registry = ToolRegistry::new();
      
      // Register all vectorized tools
      registry.register("detect_duplicates", handle_detect_duplicates);
      registry.register("predict_defects", handle_predict_defects);
      registry.register("analyze_graph_metrics", handle_graph_metrics);
      registry.register("find_similar_names", handle_name_similarity);
      registry.register("analyze_comprehensive", handle_comprehensive);
      registry.register("check_quality_gate", handle_quality_gate);
      registry.register("analyze_proof_annotations", handle_proof_annotations);
      registry.register("track_incremental_coverage", handle_incremental_coverage);
      registry.register("build_symbol_table", handle_symbol_table);
      registry.register("analyze_complexity_bounds", handle_complexity_bounds);
      registry.register("analyze_verified", handle_analyze_verified);
      
      registry
  }
  ```

- [ ] Implement handler functions:
  - [ ] `handle_detect_duplicates`
  - [ ] `handle_predict_defects`
  - [ ] `handle_graph_metrics`
  - [ ] `handle_name_similarity`
  - [ ] `handle_comprehensive`
  - [ ] `handle_quality_gate`
  - [ ] `handle_proof_annotations`
  - [ ] `handle_incremental_coverage`
  - [ ] `handle_symbol_table`
  - [ ] `handle_complexity_bounds`
  - [ ] `handle_analyze_verified`

- [ ] Test MCP exposure:
  ```bash
  echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | pmat
  # Should show 20+ tools
  ```

## Phase 8: Validation and Performance

### Day 19: Correctness Validation
- [ ] Run metric accuracy tests:
  ```bash
  cargo test metric_accuracy_suite -- --nocapture
  ```

- [ ] Verify TDG variance:
  ```bash
  pmat analyze tdg server/src/**/*.rs --json | \
    jq -r '.files[].tdg' | awk '{s+=$1; s2+=$1*$1; n++} 
    END {print "variance:", (s2/n)-(s/n)^2}'
  # Target: variance > 0.5
  ```

- [ ] Check complexity ratios:
  ```bash
  pmat analyze comprehensive server/ --json | \
    jq '.functions[] | 
    select(.cognitive > .cyclomatic * 2 or .cognitive < .cyclomatic * 1.1) | 
    .name' | wc -l
  # Target: 0 violations
  ```

- [ ] Validate FFI detection:
  ```bash
  echo '#[no_mangle] pub extern "C" fn test() {}' > ffi_test.rs
  pmat analyze dead-code ffi_test.rs --json | \
    jq '.dead_functions'
  # Target: 0 (not dead)
  ```

### Day 20: Performance Benchmarks
- [ ] Baseline performance:
  ```bash
  hyperfine --warmup 3 \
    'pmat analyze comprehensive server/src/services/' \
    --export-json baseline.json
  ```

- [ ] With verification enabled:
  ```bash
  hyperfine --warmup 3 \
    'pmat analyze comprehensive server/src/services/ --verify' \
    --export-json verified.json
  ```

- [ ] Compare results:
  ```bash
  scripts/compare_benchmarks.py baseline.json verified.json
  # Target: <20% overhead
  ```

- [ ] Memory usage:
  ```bash
  /usr/bin/time -v pmat analyze comprehensive server/ 2>&1 | \
    grep "Maximum resident set size"
  # Target: <1GB for 1000 files
  ```

- [ ] Cache effectiveness:
  ```bash
  pmat analyze comprehensive server/ --cache-stats
  # Target: >85% hit rate on second run
  ```

## Success Metrics

### Correctness
- [ ] TDG variance > 0.5 across similar files
- [ ] All cognitive/cyclomatic ratios between 1.1-2.0
- [ ] Zero false positives in dead code detection
- [ ] All defect predictions include confidence intervals
- [ ] Big-O annotations present in deep_context.md

### Performance
- [ ] Verification overhead < 20%
- [ ] Pattern cache hit rate > 85%
- [ ] Analysis throughput > 1000 files/second (8-core)
- [ ] Memory usage < 1GB for 1000-file projects

### Code Quality
- [ ] AST parser complexity reduced by >25%
- [ ] CLI module < 100 LOC
- [ ] No file exceeds 500 LOC
- [ ] All tests passing

