# Sprint 49 Implementation Plan: Technical Debt Reduction

This document details the specific implementation steps for the technical debt reduction work outlined in the Sprint 49 plan.

## Phase 1: High Severity Violations

### 1. Mutation Executor Resilience (`server/src/services/mutation/executor.rs`)

#### Current Issues:
- File restoration issues when process is interrupted (SIGINT/Ctrl+C)
- No resumable testing capability
- Temporary files may remain on errors

#### Implementation Plan:

1. **Enhance the backup/restore mechanism:**
   - Convert the backup/restore functions to use a RAII pattern with `Drop` implementation
   - Create a `MutantGuard` struct that restores files on scope exit

```rust
struct MutantGuard {
    original_path: PathBuf,
    backup_path: PathBuf,
}

impl MutantGuard {
    async fn new(path: &Path) -> Result<Self> {
        let backup_path = path.with_extension("pmat_backup");
        fs::copy(path, &backup_path).await?;
        Ok(Self {
            original_path: path.to_path_buf(),
            backup_path,
        })
    }
}

impl Drop for MutantGuard {
    fn drop(&mut self) {
        // Use blocking FS operations in Drop to ensure they complete
        if let Err(e) = std::fs::copy(&self.backup_path, &self.original_path) {
            eprintln!("Error restoring file: {}", e);
        }
        let _ = std::fs::remove_file(&self.backup_path);
    }
}
```

2. **Add signal handler for graceful shutdown:**
   - Implement a signal handler using `tokio::signal` to catch SIGINT
   - On signal, cancel running mutation tests and restore files

```rust
pub async fn execute_mutants_with_cancellation(&self, mutants: &[Mutant]) -> Result<Vec<MutationResult>> {
    // Set up cancellation token
    let (cancel_tx, mut cancel_rx) = tokio::sync::mpsc::channel(1);
    
    // Set up signal handler
    let cancel_tx_clone = cancel_tx.clone();
    tokio::spawn(async move {
        if let Ok(()) = tokio::signal::ctrl_c().await {
            println!("Received Ctrl+C, gracefully stopping...");
            let _ = cancel_tx_clone.send(()).await;
        }
    });
    
    // Execute mutants with cancellation
    // ...
}
```

3. **Add state persistence for resumable testing:**
   - Save mutation test progress to disk periodically
   - Add option to resume from last saved state

```rust
#[derive(Serialize, Deserialize)]
struct MutationTestState {
    completed_mutants: Vec<MutationResult>,
    pending_mutants: Vec<Mutant>,
}

impl MutantExecutor {
    async fn save_state(&self, state: &MutationTestState, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(state)?;
        fs::write(path, json).await?;
        Ok(())
    }
    
    async fn load_state(&self, path: &Path) -> Result<MutationTestState> {
        let json = fs::read_to_string(path).await?;
        let state = serde_json::from_str(&json)?;
        Ok(state)
    }
    
    // New method to resume testing
    pub async fn resume_mutants(&self, state_path: &Path) -> Result<Vec<MutationResult>> {
        let state = self.load_state(state_path).await?;
        let mut results = state.completed_mutants;
        let pending = self.execute_mutants(&state.pending_mutants).await?;
        results.extend(pending);
        Ok(results)
    }
}
```

### 2. Distributed Testing Safety (`server/src/services/mutation/distributed.rs`)

#### Current Issues:
- Temp files may remain after worker cancellation
- No proper monitoring of worker state
- Incomplete worker error boundaries

#### Implementation Plan:

1. **Improve worker temp file management:**
   - Use a temp directory per worker with automatic cleanup
   - Add a RAII-based cleanup mechanism 

```rust
struct WorkerTempFile {
    path: PathBuf,
}

impl WorkerTempFile {
    fn new(worker_id: usize, mutant_id: usize) -> Self {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join(format!("pmat_w{}_{}.rs", worker_id, mutant_id));
        Self { path }
    }
    
    async fn write(&self, content: &str) -> Result<()> {
        fs::write(&self.path, content).await?;
        Ok(())
    }
}

impl Drop for WorkerTempFile {
    fn drop(&mut self) {
        // Use blocking FS to ensure cleanup
        let _ = std::fs::remove_file(&self.path);
    }
}
```

2. **Add worker state monitoring:**
   - Track worker health with heartbeats
   - Automatically restart failed workers
   - Add detailed logging of worker state

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum WorkerState {
    Idle,
    Processing,
    Completed,
    Failed,
}

struct WorkerMetrics {
    state: WorkerState,
    processed: usize,
    last_heartbeat: std::time::Instant,
    errors: Vec<String>,
}

// Track workers in DistributedExecutor
struct WorkerTracker {
    workers: Vec<WorkerMetrics>,
}

impl WorkerTracker {
    fn new(worker_count: usize) -> Self {
        let workers = (0..worker_count)
            .map(|_| WorkerMetrics {
                state: WorkerState::Idle,
                processed: 0,
                last_heartbeat: std::time::Instant::now(),
                errors: Vec::new(),
            })
            .collect();
        Self { workers }
    }
    
    fn update_heartbeat(&mut self, worker_id: usize) {
        if let Some(worker) = self.workers.get_mut(worker_id) {
            worker.last_heartbeat = std::time::Instant::now();
        }
    }
    
    fn detect_stalled_workers(&self, timeout: std::time::Duration) -> Vec<usize> {
        let now = std::time::Instant::now();
        self.workers.iter().enumerate()
            .filter_map(|(id, w)| {
                if w.state == WorkerState::Processing && now.duration_since(w.last_heartbeat) > timeout {
                    Some(id)
                } else {
                    None
                }
            })
            .collect()
    }
}
```

3. **Improve progress tracking:**
   - Add detailed metrics per worker
   - Add ETA calculation
   - Enhance mutation score calculation

```rust
impl MutationProgress {
    // Add ETA calculation
    pub fn estimated_completion_time(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        if self.completed == 0 || self.total == 0 {
            return None;
        }
        
        let now = chrono::Utc::now();
        let avg_time_per_mutant = self.elapsed_seconds as f64 / self.completed as f64;
        let remaining = self.total - self.completed;
        let remaining_seconds = (remaining as f64 * avg_time_per_mutant) as i64;
        
        Some(now + chrono::Duration::seconds(remaining_seconds))
    }
    
    // Add mutation score with confidence interval
    pub fn mutation_score_with_confidence(&self) -> (f64, f64) {
        if self.killed + self.survived == 0 {
            return (0.0, 0.0);
        }
        
        let score = self.mutation_score();
        
        // 95% confidence interval using Wilson score interval
        let n = (self.killed + self.survived) as f64;
        let z = 1.96; // 95% confidence
        let p = self.killed as f64 / n;
        
        let interval = z * ((p * (1.0 - p)) / n).sqrt();
        
        (score, interval * 100.0) // Return as percentage
    }
}
```

### 3. Deep WASM Analysis (`server/src/services/deep_wasm/service.rs`)

#### Current Issues:
- Missing WebAssembly analyzer functions
- Incomplete WAT file support
- Missing WASM function extraction

#### Implementation Plan:

1. **Implement WebAssembly analyzer:**
   - Complete the TODO in the bytecode analysis section
   - Add proper disassembly support for WASM functions
   - Implement pattern detection across modules

```rust
impl BytecodeAnalyzer {
    // Implement disassembly support
    pub fn disassemble_function(&self, bytes: &[u8], func_idx: usize) -> DeepWasmResult<DisassembledFunction> {
        // Parse and disassemble function
        let mut parser = wasmparser::Parser::new(0);
        let mut payload_iter = parser.parse_all(bytes);
        
        let mut code_section = None;
        while let Some(Ok(payload)) = payload_iter.next() {
            match payload {
                wasmparser::Payload::CodeSectionEntry(body) => {
                    if func_idx == parser.current_position() {
                        code_section = Some(body);
                        break;
                    }
                }
                _ => {}
            }
        }
        
        if let Some(body) = code_section {
            let locals = body.get_locals_reader()?;
            let code = body.get_operators_reader()?;
            
            let mut disassembled = DisassembledFunction {
                function_index: func_idx,
                name: None,
                local_count: 0,
                instructions: vec![],
            };
            
            // Parse locals
            let mut locals_count = 0;
            for _ in 0..locals.get_count() {
                let (count, _) = locals.read()?;
                locals_count += count;
            }
            disassembled.local_count = locals_count;
            
            // Parse instructions
            let mut instructions = Vec::new();
            for op in code.into_iter() {
                let op = op?;
                instructions.push(format!("{:?}", op));
            }
            disassembled.instructions = instructions;
            
            Ok(disassembled)
        } else {
            Err(DeepWasmError::Analysis(format!("Function {} not found", func_idx)))
        }
    }
}
```

2. **Add WAT text format support:**
   - Implement WAT parser for text format WebAssembly
   - Extract functions and complexity metrics

```rust
impl WasmInspector {
    pub fn inspect_wat_file(&self, path: &Path) -> DeepWasmResult<WasmModuleAnalysis> {
        let content = fs::read_to_string(path)
            .map_err(DeepWasmError::Io)?;
            
        // Basic WAT analysis using regex patterns
        let function_count = self.count_wat_functions(&content);
        let exported_functions = self.count_wat_exports(&content);
        
        // Convert WAT to WASM for deeper analysis if needed
        // For now, we'll use simple metrics
        
        Ok(WasmModuleAnalysis {
            module_size_bytes: content.len(),
            function_count,
            exported_functions,
            max_complexity: self.estimate_wat_complexity(&content),
            has_dwarf: false,
            has_source_map: false,
        })
    }
    
    fn count_wat_functions(&self, content: &str) -> usize {
        let re = regex::Regex::new(r"\(func\s").unwrap();
        re.find_iter(content).count()
    }
    
    fn count_wat_exports(&self, content: &str) -> usize {
        let re = regex::Regex::new(r"\(export\s").unwrap();
        re.find_iter(content).count()
    }
    
    fn estimate_wat_complexity(&self, content: &str) -> usize {
        // Count control flow instructions as proxy for complexity
        let control_patterns = [
            "if", "else", "loop", "block", "br_if", "br_table",
            "call_indirect"
        ];
        
        let mut max_complexity = 0;
        let mut current_complexity = 0;
        
        for line in content.lines() {
            if line.contains("(func") {
                current_complexity = 1; // Base complexity
            } else if line.contains("(end)") {
                max_complexity = max_complexity.max(current_complexity);
                current_complexity = 0;
            } else {
                for pattern in &control_patterns {
                    if line.contains(pattern) {
                        current_complexity += 1;
                    }
                }
            }
        }
        
        max_complexity
    }
}
```

3. **Implement WASM function extraction:**
   - Add function table analysis
   - Track import/export patterns
   - Correlate with source code

```rust
impl DeepWasmService {
    // Add function table analysis
    fn analyze_function_table(&self, wasm_bytes: &[u8]) -> DeepWasmResult<Vec<FunctionTableEntry>> {
        let mut parser = wasmparser::Parser::new(0);
        let mut payload_iter = parser.parse_all(wasm_bytes);
        
        let mut entries = Vec::new();
        let mut function_count = 0;
        
        while let Some(Ok(payload)) = payload_iter.next() {
            match payload {
                wasmparser::Payload::FunctionSection(reader) => {
                    function_count = reader.get_count();
                },
                wasmparser::Payload::ExportSection(reader) => {
                    for export in reader.into_iter() {
                        let export = export?;
                        if let wasmparser::ExternalKind::Function = export.kind {
                            entries.push(FunctionTableEntry {
                                index: export.index as usize,
                                name: Some(export.name.to_string()),
                                is_exported: true,
                                is_imported: false,
                                signature: None,
                            });
                        }
                    }
                },
                wasmparser::Payload::ImportSection(reader) => {
                    for import in reader.into_iter() {
                        let import = import?;
                        if let wasmparser::ImportSectionEntryType::Function(_) = import.ty {
                            entries.push(FunctionTableEntry {
                                index: function_count,
                                name: Some(format!("{}.{}", import.module, import.name)),
                                is_exported: false,
                                is_imported: true,
                                signature: None,
                            });
                            function_count += 1;
                        }
                    }
                },
                _ => {}
            }
        }
        
        Ok(entries)
    }
}
```

## Phase 2: Context.rs Improvements

### Language Analyzer Implementation Plan

For each language analyzer in `context.rs`, we'll follow this pattern:

1. Check if there's an existing implementation in `server/src/services/languages/`
2. If yes, uncomment and test the code
3. If not, implement a minimal analyzer

The implementations will share this common approach:

```rust
fn analyze_X_file(path: &Path) -> Result<FileContext, TemplateError> {
    // Read file
    let content = tokio::fs::read_to_string(path).await
        .map_err(TemplateError::Io)?;
        
    // Extract imports and functions using regex patterns
    let mut items = Vec::new();
    
    // Add imports (if applicable)
    for line in content.lines() {
        if let Some(import) = extract_X_import(line) {
            items.push(import);
        }
    }
    
    // Add functions
    for (name, line) in extract_X_functions(&content) {
        items.push(AstItem::Function {
            name,
            visibility: "public".to_string(),
            is_async: false,
            line,
        });
    }
    
    // Create FileContext
    Ok(FileContext {
        path: path.display().to_string(),
        language: "X".to_string(),
        items,
        complexity_metrics: None,
    })
}
```

### Specific Language Implementation Details:

#### C/C++ Analyzer:

```rust
fn analyze_c_file(path: &Path) -> Result<FileContext, TemplateError> {
    // Read file
    let content = tokio::fs::read_to_string(path).await
        .map_err(TemplateError::Io)?;
        
    let mut items = Vec::new();
    
    // Find includes using regex
    let include_regex = regex::Regex::new(r#"#include\s+["<]([^">]+)[">]"#).unwrap();
    for cap in include_regex.captures_iter(&content) {
        items.push(AstItem::Import {
            module: cap[1].to_string(),
            items: vec![],
            alias: None,
            line: 0, // Line info requires more context
        });
    }
    
    // Find function declarations
    let function_regex = regex::Regex::new(r"(\w+)\s+(\w+)\s*\([^)]*\)\s*\{").unwrap();
    for cap in function_regex.captures_iter(&content) {
        items.push(AstItem::Function {
            name: cap[2].to_string(),
            visibility: "public".to_string(),
            is_async: false,
            line: 0,
        });
    }
    
    Ok(FileContext {
        path: path.display().to_string(),
        language: "c".to_string(),
        items,
        complexity_metrics: None,
    })
}
```

#### Ruby Analyzer:

```rust
fn analyze_ruby_file(path: &Path) -> Result<FileContext, TemplateError> {
    let content = tokio::fs::read_to_string(path).await
        .map_err(TemplateError::Io)?;
        
    let mut items = Vec::new();
    
    // Find require/include statements
    let require_regex = regex::Regex::new(r#"require(?:_relative)?\s+['"]([^'"]+)['"]\s*$"#).unwrap();
    for cap in require_regex.captures_iter(&content) {
        items.push(AstItem::Import {
            module: cap[1].to_string(),
            items: vec![],
            alias: None,
            line: 0,
        });
    }
    
    // Find method definitions
    let method_regex = regex::Regex::new(r"^\s*def\s+(\w+)").unwrap();
    for (line_num, line) in content.lines().enumerate() {
        if let Some(cap) = method_regex.captures(line) {
            items.push(AstItem::Function {
                name: cap[1].to_string(),
                visibility: "public".to_string(), 
                is_async: false,
                line: line_num + 1,
            });
        }
    }
    
    // Find class definitions
    let class_regex = regex::Regex::new(r"^\s*class\s+(\w+)").unwrap();
    for (line_num, line) in content.lines().enumerate() {
        if let Some(cap) = class_regex.captures(line) {
            items.push(AstItem::Struct {
                name: cap[1].to_string(),
                visibility: "public".to_string(),
                fields_count: 0, // We'd need deeper parsing to count fields
                derives: Vec::new(),
                line: line_num + 1,
            });
        }
    }
    
    Ok(FileContext {
        path: path.display().to_string(),
        language: "ruby".to_string(),
        items,
        complexity_metrics: None,
    })
}
```

#### Shell Script Analyzer:

```rust
fn analyze_shell_file(path: &Path) -> Result<FileContext, TemplateError> {
    let content = tokio::fs::read_to_string(path).await
        .map_err(TemplateError::Io)?;
        
    let mut items = Vec::new();
    
    // Find function definitions
    let function_regex = regex::Regex::new(r"^\s*(\w+)\s*\(\)\s*\{").unwrap();
    for (line_num, line) in content.lines().enumerate() {
        if let Some(cap) = function_regex.captures(line) {
            items.push(AstItem::Function {
                name: cap[1].to_string(),
                visibility: "public".to_string(),
                is_async: false,
                line: line_num + 1,
            });
        }
    }
    
    // Find source/include statements
    let source_regex = regex::Regex::new(r"^\s*source\s+([^\s;]+)").unwrap();
    for (line_num, line) in content.lines().enumerate() {
        if let Some(cap) = source_regex.captures(line) {
            items.push(AstItem::Import {
                module: cap[1].to_string(),
                items: vec![],
                alias: None,
                line: line_num + 1,
            });
        }
    }
    
    Ok(FileContext {
        path: path.display().to_string(),
        language: "shell".to_string(),
        items,
        complexity_metrics: None,
    })
}
```

## Phase 3: Deep Context Enhancements

### Context.rs and Deep Context Integration:

1. **Connect language analyzers to deep context:**
   - Update deep_context.rs to use the new language analyzers
   - Add multi-language project support

```rust
impl DeepContext {
    // Add method to analyze multi-language projects
    pub async fn analyze_multi_language_project(
        &self, 
        root_path: &Path, 
        language_preferences: &[&str]
    ) -> Result<ProjectContext> {
        let mut combined_context = ProjectContext {
            project_type: "multi-language".to_string(),
            files: Vec::new(),
            summary: ProjectSummary::default(),
        };
        
        // Detect and analyze files by language
        for language in language_preferences {
            let extensions = self.get_extensions_for_language(language);
            for ext in extensions {
                // Find files with this extension
                let glob_pattern = format!("**/*.{}", ext);
                let files = glob::glob(&glob_pattern)?;
                
                for entry in files {
                    if let Ok(path) = entry {
                        // Analyze file using appropriate analyzer
                        if let Some(file_context) = analyze_file_by_extension(&path).await {
                            combined_context.files.push(file_context);
                        }
                    }
                }
            }
        }
        
        // Build combined summary
        combined_context.summary = self.build_multi_language_summary(&combined_context.files);
        
        Ok(combined_context)
    }
    
    fn get_extensions_for_language(&self, language: &str) -> Vec<&str> {
        match language.to_lowercase().as_str() {
            "rust" => vec!["rs"],
            "typescript" => vec!["ts", "tsx"],
            "javascript" => vec!["js", "jsx", "mjs"],
            "python" => vec!["py", "pyi"],
            "c" => vec!["c", "h"],
            "cpp" => vec!["cpp", "cc", "cxx", "hpp", "hxx"],
            "ruby" => vec!["rb"],
            "shell" => vec!["sh", "bash"],
            "go" => vec!["go"],
            "java" => vec!["java"],
            "csharp" => vec!["cs"],
            "kotlin" => vec!["kt", "kts"],
            "swift" => vec!["swift"],
            "wasm" => vec!["wat", "wasm"],
            // Add more languages as needed
            _ => vec![],
        }
    }
    
    fn build_multi_language_summary(&self, files: &[FileContext]) -> ProjectSummary {
        let mut summary = ProjectSummary::default();
        
        // Count files
        summary.total_files = files.len();
        
        // Count items by type
        for file in files {
            for item in &file.items {
                match item {
                    AstItem::Function { .. } => summary.total_functions += 1,
                    AstItem::Struct { .. } => summary.total_structs += 1,
                    AstItem::Enum { .. } => summary.total_enums += 1,
                    AstItem::Trait { .. } => summary.total_traits += 1,
                    AstItem::Impl { .. } => summary.total_impls += 1,
                    _ => {}
                }
            }
        }
        
        summary
    }
}
```

2. **Add quality metrics for non-Rust codebases:**
   - Implement language-specific complexity calculations
   - Add multi-language summary metrics

```rust
impl DeepContext {
    // Add method to calculate quality metrics for multi-language projects
    pub fn calculate_multi_language_quality(&self, context: &ProjectContext) -> QualityScorecard {
        let mut scorecard = QualityScorecard::default();
        
        // Count metrics by language
        let mut metrics_by_language: HashMap<String, LanguageMetrics> = HashMap::new();
        
        for file in &context.files {
            let metrics = metrics_by_language
                .entry(file.language.clone())
                .or_insert_with(LanguageMetrics::default);
                
            metrics.file_count += 1;
            metrics.function_count += file.items.iter()
                .filter(|item| matches!(item, AstItem::Function { .. }))
                .count();
                
            // Add complexity if available
            if let Some(ref complexity) = file.complexity_metrics {
                metrics.total_cyclomatic += complexity.cyclomatic_complexity;
                metrics.total_cognitive += complexity.cognitive_complexity;
                metrics.max_cyclomatic = metrics.max_cyclomatic.max(complexity.cyclomatic_complexity);
                metrics.max_cognitive = metrics.max_cognitive.max(complexity.cognitive_complexity);
            }
        }
        
        // Calculate overall metrics
        let total_files = context.files.len();
        let mut overall_complexity = 0.0;
        let mut overall_maintainability = 0.0;
        
        for (_, metrics) in &metrics_by_language {
            // Weight by proportion of codebase
            let weight = metrics.file_count as f64 / total_files as f64;
            
            // Calculate language-specific scores
            let language_complexity = if metrics.function_count > 0 {
                (metrics.total_cyclomatic as f64 / metrics.function_count as f64) * 10.0
            } else {
                0.0
            };
            
            let language_maintainability = 100.0 - (language_complexity * 5.0);
            
            // Add weighted contribution
            overall_complexity += language_complexity * weight;
            overall_maintainability += language_maintainability * weight;
        }
        
        // Set scorecard values
        scorecard.complexity_score = 100.0 - (overall_complexity * 10.0).min(100.0);
        scorecard.maintainability_index = overall_maintainability;
        scorecard.modularity_score = 70.0; // Default for multi-language
        
        // Estimate technical debt hours based on complexity
        scorecard.technical_debt_hours = context.summary.total_functions as f64 * 0.2 * 
                                        (overall_complexity / 10.0);
        
        // Overall health is average of other scores
        scorecard.overall_health = (scorecard.complexity_score + 
                                   scorecard.maintainability_index + 
                                   scorecard.modularity_score) / 3.0;
        
        scorecard
    }
}
```

## Documentation and Testing

1. **Unit tests for language analyzers:**
   - Create test files for each language
   - Verify correct parsing and AST extraction
   - Test language detection logic

2. **Integration tests:**
   - Test multi-language project analysis
   - Verify distributed mutation testing
   - Test WebAssembly analysis

3. **Documentation updates:**
   - Update language support documentation
   - Document mutation testing improvements
   - Document WebAssembly analysis capabilities

## Timeline and Tasks

### Phase 1 (Days 1-3):
- Day 1: Implement MutantGuard and RAII pattern for file restoration
- Day 2: Add signal handling and resumable testing
- Day 3: Implement WebAssembly analyzer improvements

### Phase 2 (Days 4-7):
- Day 4: Implement C/C++ and Ruby analyzers
- Day 5: Implement Shell scripts and Functional language analyzers
- Day 6: Implement WebAssembly analyzer
- Day 7: Test and validate all language analyzers

### Phase 3 (Days 8-9):
- Day 8: Implement multi-language project support
- Day 9: Add quality metrics for non-Rust codebases

### Documentation and Testing (Day 10):
- Write tests for all new functionality
- Update documentation
- Verify technical debt reduction