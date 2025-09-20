# JVM/CLR Language Support Specification: C#, Kotlin, and Java

## Executive Summary

Supporting C#, Kotlin, and Java requires addressing their shared bytecode-targeting architectures while preserving language-specific semantic richness. Kotlin already has partial tree-sitter support (Language::Kotlin = 13); Java and C# require new implementations leveraging Roslyn and JavaParser for semantic accuracy beyond tree-sitter's lexical parsing.

## Architectural Analysis

### Shared Characteristics

| Feature | Java | Kotlin | C# | Impact on Analysis |
|---------|------|--------|----|--------------------|
| Target | JVM bytecode | JVM bytecode | CLR IL | Similar IR analysis possible |
| Type System | Nominal, erased generics | Nominal, reified inline | Nominal, reified generics | Type flow analysis complexity |
| Null Safety | Optional<T> | Built-in nullable types | Nullable reference types (C# 8+) | Null-flow analysis required |
| Async Model | CompletableFuture | Coroutines | async/await | Different complexity metrics |
| Properties | No | Yes | Yes | Getter/setter detection |
| Extension Methods | No | Yes | Yes | Call graph complexity |
| Pattern Matching | switch expressions (14+) | when expressions | switch expressions (C# 7+) | Branch complexity calculation |

### Parser Technology Stack

```rust
// Optimal parser selection per language
enum ParserBackend {
    TreeSitter(tree_sitter::Language),     // Fast, incremental
    Roslyn(RoslynBinding),                  // C# semantic model
    JavaParser(JavaParserBinding),          // Java type resolution
    PSI(IntelliJPsiBinding),               // Kotlin semantic analysis
}

impl LanguageStrategy {
    fn select_parser(lang: Language) -> ParserBackend {
        match lang {
            Language::Java => {
                // JavaParser for semantic accuracy
                // Tree-sitter for speed when semantic unnecessary
                if needs_type_resolution() {
                    ParserBackend::JavaParser(JavaParserBinding::new())
                } else {
                    ParserBackend::TreeSitter(tree_sitter_java::language())
                }
            }
            Language::CSharp => {
                // Roslyn provides unmatched semantic fidelity
                ParserBackend::Roslyn(RoslynBinding::via_omnisharp())
            }
            Language::Kotlin => {
                // Existing tree-sitter with PSI for complex cases
                ParserBackend::TreeSitter(tree_sitter_kotlin::language())
            }
        }
    }
}
```

## Implementation Architecture

### 1. Java Language Strategy

```rust
// server/src/ast/languages/java.rs

use jni::{JNIEnv, JavaVM, objects::JObject};
use tree_sitter_java;

pub struct JavaStrategy {
    tree_sitter_parser: Parser,
    javaparser_jvm: Option<Arc<JavaVM>>, // For semantic analysis
    type_cache: DashMap<String, JavaType>,
}

impl JavaStrategy {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_java::language())?;
        
        // Optionally initialize JavaParser via JNI for semantic analysis
        let jvm = if cfg!(feature = "java-semantic") {
            Some(Arc::new(Self::init_javaparser_jvm()?))
        } else {
            None
        };
        
        Ok(Self {
            tree_sitter_parser: parser,
            javaparser_jvm: jvm,
            type_cache: DashMap::with_capacity(10000),
        })
    }
    
    fn init_javaparser_jvm() -> Result<JavaVM> {
        // Embedded JVM for JavaParser
        let jvm_args = InitArgsBuilder::new()
            .version(JNIVersion::V8)
            .option("-Xmx256m")  // Constrained memory
            .option("-XX:+UseG1GC")
            .build()?;
            
        JavaVM::new(jvm_args)
    }
    
    async fn parse_with_semantic(&self, content: &str) -> Result<JavaAst> {
        if let Some(jvm) = &self.javaparser_jvm {
            // Use JavaParser for full semantic model
            let env = jvm.attach_current_thread()?;
            let compilation_unit = self.parse_via_javaparser(&env, content)?;
            self.extract_semantic_ast(compilation_unit)
        } else {
            // Fallback to tree-sitter
            self.parse_with_tree_sitter(content)
        }
    }
    
    fn calculate_java_complexity(&self, node: &Node, source: &str) -> ComplexityMetrics {
        let mut metrics = ComplexityMetrics::default();
        
        match node.kind() {
            "switch_expression" => {
                // Java 14+ switch expressions
                let arms = node.children_by_field_name("case", &mut node.walk())
                    .count() as u32;
                metrics.cyclomatic += arms;
                metrics.cognitive += arms * 2; // Pattern matching cognitive load
            }
            "lambda_expression" => {
                metrics.cognitive += 2; // Functional complexity
            }
            "try_with_resources_statement" => {
                // try-with-resources adds implicit finally blocks
                metrics.cyclomatic += 2;
                let resources = node.child_by_field_name("resources")
                    .map(|n| n.child_count())
                    .unwrap_or(0) as u32;
                metrics.cognitive += resources; // Each resource adds complexity
            }
            "stream_pipeline" => {
                // Stream API chains increase cognitive load
                let operations = self.count_stream_operations(node, source);
                metrics.cognitive += operations * 2;
            }
            _ => {}
        }
        
        metrics
    }
}

// Java-specific AST nodes
#[derive(Debug, Clone)]
pub struct JavaAstNode {
    pub annotations: Vec<JavaAnnotation>,
    pub modifiers: JavaModifiers,
    pub generics: Option<JavaGenerics>,
}

#[derive(Debug, Clone)]
pub struct JavaAnnotation {
    pub name: String,
    pub parameters: HashMap<String, AnnotationValue>,
    pub retention: RetentionPolicy,
}

#[derive(Debug, Clone)]
pub enum RetentionPolicy {
    Source,
    Class,
    Runtime,
}
```

### 2. C# Language Strategy

```rust
// server/src/ast/languages/csharp.rs

use omnisharp_client::{OmniSharpClient, SemanticModel};

pub struct CSharpStrategy {
    tree_sitter_parser: Parser,
    omnisharp: Option<OmniSharpClient>, // Roslyn semantic analysis
    assembly_cache: DashMap<String, AssemblyMetadata>,
}

impl CSharpStrategy {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_c_sharp::language())?;
        
        // OmniSharp provides Roslyn's semantic model
        let omnisharp = if cfg!(feature = "csharp-semantic") {
            match OmniSharpClient::spawn() {
                Ok(client) => Some(client),
                Err(e) => {
                    tracing::warn!("OmniSharp unavailable: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        Ok(Self {
            tree_sitter_parser: parser,
            omnisharp,
            assembly_cache: DashMap::with_capacity(1000),
        })
    }
    
    async fn analyze_with_roslyn(&self, file_path: &Path) -> Result<CSharpSemanticModel> {
        if let Some(client) = &self.omnisharp {
            let model = client.get_semantic_model(file_path).await?;
            
            // Roslyn provides deep semantic information
            Ok(CSharpSemanticModel {
                symbols: model.get_declared_symbols()?,
                diagnostics: model.get_diagnostics()?,
                data_flow: model.analyze_data_flow()?,
                control_flow: model.analyze_control_flow()?,
            })
        } else {
            Err(anyhow::anyhow!("Semantic analysis requires OmniSharp"))
        }
    }
    
    fn calculate_csharp_complexity(&self, node: &Node, source: &str) -> ComplexityMetrics {
        let mut metrics = ComplexityMetrics::default();
        
        match node.kind() {
            "switch_expression" => {
                // C# 8+ switch expressions with pattern matching
                let patterns = self.count_patterns(node);
                metrics.cyclomatic += patterns;
                
                // Guard clauses add complexity
                let guards = self.count_when_clauses(node);
                metrics.cognitive += guards * 2;
            }
            "async_method" => {
                metrics.cognitive += 3; // async/await complexity
                
                // Count await expressions
                let awaits = self.count_await_expressions(node);
                metrics.cognitive += awaits;
            }
            "linq_query" => {
                // LINQ queries have high cognitive load
                let clauses = self.count_linq_clauses(node);
                metrics.cognitive += clauses * 2;
            }
            "property_declaration" => {
                // Auto-properties vs explicit accessors
                if self.has_explicit_accessors(node) {
                    metrics.cyclomatic += 2; // get/set branches
                }
            }
            "nullable_type" | "nullable_reference_type" => {
                // Nullable flow analysis complexity
                metrics.cognitive += 1;
            }
            _ => {}
        }
        
        metrics
    }
}

// C#-specific semantic model
#[derive(Debug)]
pub struct CSharpSemanticModel {
    pub symbols: Vec<ISymbol>,
    pub diagnostics: Vec<Diagnostic>,
    pub data_flow: DataFlowAnalysis,
    pub control_flow: ControlFlowGraph,
}

#[derive(Debug)]
pub struct DataFlowAnalysis {
    pub definitely_assigned: HashSet<LocalSymbol>,
    pub captured_variables: Vec<CapturedVariable>,
    pub escape_analysis: EscapeAnalysis,
}
```

### 3. Enhanced Kotlin Strategy

```rust
// server/src/ast/languages/kotlin.rs (enhanced from existing)

use tree_sitter_kotlin;
use kotlin_compiler_client::KotlinCompilerClient;

pub struct KotlinStrategy {
    parser: Parser,
    kotlin_compiler: Option<KotlinCompilerClient>, // For K2 compiler integration
    coroutine_analyzer: CoroutineAnalyzer,
}

impl KotlinStrategy {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_kotlin::language())?;
        
        // K2 compiler for semantic analysis
        let compiler = if cfg!(feature = "kotlin-semantic") {
            KotlinCompilerClient::connect().ok()
        } else {
            None
        };
        
        Ok(Self {
            parser,
            kotlin_compiler: compiler,
            coroutine_analyzer: CoroutineAnalyzer::new(),
        })
    }
    
    fn analyze_coroutines(&self, ast: &AstDag) -> CoroutineComplexity {
        let mut complexity = CoroutineComplexity::default();
        
        for node in ast.nodes.iter_values() {
            match &node.kind {
                AstKind::Function(func) if func.is_suspend => {
                    complexity.suspend_functions += 1;
                    complexity.cognitive_overhead += 3;
                    
                    // Analyze suspension points
                    let suspension_points = self.count_suspension_points(node);
                    complexity.total_suspension_points += suspension_points;
                    complexity.cognitive_overhead += suspension_points * 2;
                }
                AstKind::Flow(flow) if flow.is_coroutine_builder => {
                    // launch, async, runBlocking
                    complexity.coroutine_builders += 1;
                    complexity.concurrency_complexity += match flow.builder_type {
                        "launch" => 2,
                        "async" => 3,  // Deferred handling
                        "runBlocking" => 4, // Blocking complexity
                        _ => 1,
                    };
                }
                _ => {}
            }
        }
        
        complexity
    }
    
    fn calculate_kotlin_complexity(&self, node: &Node, source: &str) -> ComplexityMetrics {
        let mut metrics = ComplexityMetrics::default();
        
        match node.kind() {
            "when_expression" => {
                // Kotlin's powerful pattern matching
                let branches = node.children_by_field_name("entry", &mut node.walk())
                    .count() as u32;
                metrics.cyclomatic += branches;
                
                // Exhaustive when adds cognitive load
                if self.is_exhaustive_when(node) {
                    metrics.cognitive += 2;
                }
            }
            "elvis_expression" => {
                // ?: operator
                metrics.cyclomatic += 1;
                metrics.cognitive += 1;
            }
            "safe_call_expression" => {
                // ?. operator chains
                let chain_length = self.count_safe_call_chain(node);
                metrics.cognitive += chain_length;
            }
            "object_declaration" => {
                // Singleton pattern built-in
                metrics.cognitive += 2;
            }
            "delegated_property" => {
                // by lazy, by observable
                metrics.cognitive += 3;
            }
            _ => {}
        }
        
        metrics
    }
}

#[derive(Debug, Default)]
pub struct CoroutineComplexity {
    pub suspend_functions: u32,
    pub coroutine_builders: u32,
    pub total_suspension_points: u32,
    pub concurrency_complexity: u32,
    pub cognitive_overhead: u32,
}
```

### 4. Shared Bytecode Analysis

```rust
// server/src/ast/bytecode/jvm_clr.rs

pub trait BytecodeAnalyzer {
    type Bytecode;
    type Method;
    
    fn load_bytecode(&self, path: &Path) -> Result<Self::Bytecode>;
    fn extract_methods(&self, bytecode: &Self::Bytecode) -> Vec<Self::Method>;
    fn analyze_method_complexity(&self, method: &Self::Method) -> BytecodeComplexity;
}

pub struct JvmBytecodeAnalyzer {
    class_reader: cafebabe::ClassReader,
}

impl BytecodeAnalyzer for JvmBytecodeAnalyzer {
    type Bytecode = cafebabe::ClassFile;
    type Method = cafebabe::Method;
    
    fn analyze_method_complexity(&self, method: &Self::Method) -> BytecodeComplexity {
        let mut complexity = BytecodeComplexity::default();
        
        if let Some(code) = method.code() {
            for instruction in code.instructions() {
                match instruction {
                    Opcode::INVOKEVIRTUAL | Opcode::INVOKEINTERFACE => {
                        complexity.dynamic_dispatch += 1;
                        complexity.cognitive += 2;
                    }
                    Opcode::ATHROW => {
                        complexity.exception_paths += 1;
                        complexity.cyclomatic += 1;
                    }
                    Opcode::MONITORENTER | Opcode::MONITOREXIT => {
                        complexity.synchronization += 1;
                        complexity.cognitive += 3;
                    }
                    _ => {}
                }
            }
            
            // Stack depth affects JIT optimization
            complexity.max_stack = code.max_stack();
            complexity.max_locals = code.max_locals();
        }
        
        complexity
    }
}

pub struct ClrILAnalyzer {
    assembly_reader: dnlib::DotNet,
}

impl BytecodeAnalyzer for ClrILAnalyzer {
    type Bytecode = dnlib::Assembly;
    type Method = dnlib::MethodDef;
    
    fn analyze_method_complexity(&self, method: &Self::Method) -> BytecodeComplexity {
        let mut complexity = BytecodeComplexity::default();
        
        for instruction in method.body().instructions() {
            match instruction.opcode() {
                OpCode::CALLVIRT => {
                    complexity.dynamic_dispatch += 1;
                }
                OpCode::THROW | OpCode::RETHROW => {
                    complexity.exception_paths += 1;
                }
                OpCode::LDSFLD | OpCode::STSFLD if is_volatile(instruction) => {
                    complexity.memory_barriers += 1;
                    complexity.cognitive += 2;
                }
                _ => {}
            }
        }
        
        complexity
    }
}
```

## Complexity Metrics Comparison

| Metric | Java | Kotlin | C# | Calculation Method |
|--------|------|--------|----|-------------------|
| Cyclomatic | Standard + switch expressions | Standard + when + elvis | Standard + patterns + null-conditional | Branch count + 1 |
| Cognitive | +2 per lambda, +3 per stream | +3 per coroutine, +1 per safe call | +3 per async, +2 per LINQ | Nesting + constructs |
| Type Complexity | Bounded wildcards | Variance + reified | Covariance + nullable refs | Type parameter depth |
| Concurrency | Thread + CompletableFuture | Coroutines + Channels | async/await + Task | Synchronization points |

## Performance Characteristics

| Operation | Java | Kotlin | C# | Bottleneck |
|-----------|------|--------|----|------------|
| Parse (tree-sitter) | 350K LOC/s | 300K LOC/s | 320K LOC/s | Lexing |
| Parse (semantic) | 50K LOC/s | 45K LOC/s | 60K LOC/s | Type resolution |
| Bytecode analysis | 5MB/s (class files) | 5MB/s | 8MB/s (assemblies) | I/O + decompression |
| Memory usage | 100MB/100K LOC | 120MB/100K LOC | 110MB/100K LOC | Symbol tables |

## Dead Code Detection Precision

```rust
pub struct JvmDeadCodeAnalyzer {
    // Reflection and dynamic proxy usage limits precision
    reflection_detector: ReflectionUsageDetector,
    spring_analyzer: Option<SpringContextAnalyzer>,
}

impl JvmDeadCodeAnalyzer {
    fn analyze_reachability(&self, classes: &[ClassFile]) -> DeadCodeReport {
        let mut report = DeadCodeReport::default();
        
        // JVM-specific challenges:
        // 1. Reflection (Class.forName)
        // 2. Dependency injection (Spring, Guice)
        // 3. Annotation processors
        // 4. Dynamic proxies
        // 5. ServiceLoader SPI
        
        if self.has_reflection_usage(classes) {
            report.confidence = ConfidenceLevel::Low;
            report.add_warning("Reflection detected - analysis may be incomplete");
        }
        
        if let Some(spring) = &self.spring_analyzer {
            // Spring beans are implicitly reachable
            let beans = spring.find_component_scans(classes);
            for bean in beans {
                self.mark_transitively_reachable(bean);
            }
        }
        
        report
    }
}
```

## Implementation Requirements

### Minimal (5K LOC total)
- Tree-sitter parsers for all three languages
- Basic complexity metrics
- Shared bytecode loader

### Extended (15K LOC total)
- JavaParser JNI integration
- OmniSharp/Roslyn client
- K2 compiler integration
- Bytecode-level analysis
- Framework-specific dead code (Spring, ASP.NET)

## Validation Benchmarks

| Tool | Purpose | Target Match |
|------|---------|--------------|
| checkstyle (Java) | Complexity | >90% correlation |
| detekt (Kotlin) | Code smells | >85% agreement |
| Roslyn Analyzers (C#) | Semantic | 100% (using same backend) |
| SpotBugs (bytecode) | Dead code | >80% precision |

## Summary

Supporting C#, Kotlin, and Java requires balancing semantic accuracy against performance. Tree-sitter provides adequate lexical analysis at 300K+ LOC/s, while semantic analysis via language servers drops to 50K LOC/s but enables type-aware metrics. The shared JVM/CLR bytecode layer enables unified post-compilation analysis, critical for accurate dead code detection in the presence of reflection and dependency injection frameworks.
