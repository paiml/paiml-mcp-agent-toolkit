# C-Languages AST Support Specification (Revised)

## Executive Summary

This specification defines the integration of C, C++, and Cython AST analysis into the PAIML MCP Agent Toolkit. Based on rigorous academic review, we adopt a phased implementation strategy that acknowledges the fundamental challenges of cross-language static analysis while providing immediate value through incremental delivery. The design leverages established program analysis techniques, explicitly choosing tractable approximations over theoretical completeness.

## 1. Core Design Principles

### 1.1 Foundational Constraints

Our approach acknowledges three fundamental limitations in static analysis:

1. **Rice's Theorem**: Most interesting properties about programs are undecidable
2. **Pointer Aliasing**: May-alias analysis is NP-hard in the general case
3. **Path Explosion**: Path-sensitive analysis faces exponential state spaces

We address these through:
- **Sound Approximation**: Over-approximate reachability (fewer false dead code reports)
- **Monotone Frameworks**: Ensure analysis convergence
- **Abstract Interpretation**: Use abstract domains for tractable analysis

### 1.2 Phased Implementation Strategy

```
Phase 1 (Months 1-3): Single-Language C Analysis
Phase 2 (Months 4-6): C++ Extension with Pointer Analysis
Phase 3 (Months 7-9): Inter-language C/C++ Analysis
Phase 4 (Months 10-12): Cython Integration
```

## 2. Parser Infrastructure

### 2.1 Dependencies and Configuration

```toml
[dependencies]
# Core parsing
tree-sitter = "0.20"
tree-sitter-c = "0.20.6"
tree-sitter-cpp = "0.20.0"
tree-sitter-python = "0.20.4"

# Analysis infrastructure
petgraph = "0.6"  # Graph algorithms
fixedbitset = "0.4"  # Efficient bit vectors
rustc-hash = "1.1"  # Fast hashing

# Language-specific
cpp_demangle = "0.4"
gimli = "0.28"  # DWARF parsing for debug info
goblin = "0.7"  # Object file parsing

[dev-dependencies]
proptest = "1.0"  # Property-based testing
criterion = "0.5"  # Benchmarking
```

### 2.2 Preprocessor Strategy

```rust
// server/src/services/preprocessor.rs
pub struct PreprocessorConfig {
    pub mode: PreprocessorMode,
    pub definitions: HashMap<String, MacroDefinition>,
    pub include_paths: Vec<PathBuf>,
}

pub enum PreprocessorMode {
    /// Use external preprocessor (accurate but slower)
    External { command: PathBuf },  // e.g., "gcc -E"
    
    /// Minimal built-in preprocessor (fast but limited)
    Builtin { 
        handle_includes: bool,
        expand_macros: bool,
    },
    
    /// Hybrid: external for complex files, builtin for simple
    Adaptive { 
        complexity_threshold: usize,
        fallback: Box<PreprocessorMode>,
    },
}

impl Preprocessor {
    /// Preprocess with caching based on include graph hash
    pub async fn preprocess(&self, path: &Path) -> Result<PreprocessedFile> {
        let include_hash = self.compute_include_hash(path)?;
        
        if let Some(cached) = self.cache.get(&include_hash) {
            return Ok(cached);
        }
        
        match self.config.mode {
            PreprocessorMode::External { ref command } => {
                self.preprocess_external(path, command).await
            },
            PreprocessorMode::Builtin { .. } => {
                self.preprocess_builtin(path).await
            },
            PreprocessorMode::Adaptive { threshold, ref fallback } => {
                let complexity = self.estimate_complexity(path)?;
                if complexity > threshold {
                    self.preprocess_with_mode(path, fallback).await
                } else {
                    self.preprocess_builtin(path).await
                }
            },
        }
    }
}
```

## 3. Phase 1: C Language Analysis

### 3.1 Control Flow Graph Construction

```rust
// server/src/services/cfg_builder.rs
pub struct ControlFlowGraph {
    pub entry: NodeId,
    pub exit: NodeId,
    pub nodes: IndexMap<NodeId, CfgNode>,
    pub edges: Vec<CfgEdge>,
    pub dominators: DominatorTree,
}

#[derive(Debug, Clone)]
pub struct CfgNode {
    pub id: NodeId,
    pub kind: CfgNodeKind,
    pub ast_node: Option<AstNodeId>,
    pub predecessors: Vec<NodeId>,
    pub successors: Vec<NodeId>,
}

#[derive(Debug, Clone)]
pub enum CfgNodeKind {
    Entry,
    Exit,
    Statement(Statement),
    Condition { true_branch: NodeId, false_branch: NodeId },
    Switch { cases: Vec<(Option<i64>, NodeId)>, default: Option<NodeId> },
    Label(String),
    Goto(String),
}

impl CfgBuilder {
    pub fn build_cfg(ast: &CAst) -> Result<ControlFlowGraph> {
        let mut builder = Self::new();
        
        // First pass: collect all labels for goto resolution
        let labels = builder.collect_labels(ast)?;
        
        // Second pass: build basic blocks
        let blocks = builder.build_basic_blocks(ast)?;
        
        // Third pass: connect blocks and resolve gotos
        builder.connect_blocks(&blocks, &labels)?;
        
        // Compute dominator tree for optimization
        let dominators = builder.compute_dominators()?;
        
        Ok(ControlFlowGraph {
            entry: builder.entry_node,
            exit: builder.exit_node,
            nodes: builder.nodes,
            edges: builder.edges,
            dominators,
        })
    }
    
    /// Handle C's non-structured control flow
    fn resolve_goto(&mut self, goto: &GotoStatement, labels: &HashMap<String, NodeId>) -> Result<()> {
        match labels.get(&goto.label) {
            Some(&target) => {
                self.add_edge(goto.node_id, target, CfgEdgeKind::Goto);
                Ok(())
            },
            None => Err(AnalysisError::UnresolvedGoto(goto.label.clone())),
        }
    }
}
```

### 3.2 Path-Insensitive Memory Analysis

Following the reviewer's recommendation, we start with tractable path-insensitive analysis:

```rust
// server/src/services/memory_analysis_c.rs
pub struct PathInsensitiveMemoryAnalyzer {
    lattice: MemoryStateLattice,
    transfer: TransferFunctions,
}

/// Abstract domain for memory states
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryState {
    Unallocated,
    Allocated { size: AbstractSize },
    Freed,
    Unknown,  // Top element
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbstractSize {
    Constant(usize),
    Symbolic(String),
    Unknown,
}

impl MemoryStateLattice {
    /// Join operation for the lattice
    pub fn join(&self, a: &MemoryState, b: &MemoryState) -> MemoryState {
        use MemoryState::*;
        match (a, b) {
            (Unallocated, Unallocated) => Unallocated,
            (Allocated { size: s1 }, Allocated { size: s2 }) => {
                Allocated { size: self.join_sizes(s1, s2) }
            },
            (Freed, Freed) => Freed,
            _ => Unknown,  // Conservative approximation
        }
    }
    
    /// Partial order for convergence
    pub fn less_equal(&self, a: &MemoryState, b: &MemoryState) -> bool {
        matches!((a, b), 
            (Unallocated, Unallocated) |
            (Unallocated, Unknown) |
            (Allocated { .. }, Unknown) |
            (Freed, Freed) |
            (Freed, Unknown) |
            (_, Unknown)
        )
    }
}

impl PathInsensitiveMemoryAnalyzer {
    pub fn analyze_function(&self, func: &CFunction, cfg: &ControlFlowGraph) -> MemoryAnalysisResult {
        let mut state = HashMap::<AllocationSite, MemoryState>::new();
        let mut warnings = Vec::new();
        
        // Initialize all allocation sites to Unallocated
        for site in self.find_allocation_sites(func) {
            state.insert(site, MemoryState::Unallocated);
        }
        
        // Fixed-point iteration
        let mut changed = true;
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 100;
        
        while changed && iterations < MAX_ITERATIONS {
            changed = false;
            iterations += 1;
            
            // Process nodes in topological order
            for node_id in cfg.topological_order() {
                let old_state = state.clone();
                self.transfer.apply(&cfg.nodes[&node_id], &mut state);
                
                // Check for convergence
                if state != old_state {
                    changed = true;
                }
            }
        }
        
        // Detect issues in final state
        for (site, mem_state) in &state {
            match mem_state {
                MemoryState::Allocated { .. } if self.is_function_exit(&site.location) => {
                    warnings.push(MemoryWarning::PotentialLeak { site: site.clone() });
                },
                _ => {},
            }
        }
        
        MemoryAnalysisResult {
            final_states: state,
            warnings,
            converged: iterations < MAX_ITERATIONS,
        }
    }
}
```

### 3.3 Intraprocedural Complexity Analysis

```rust
// server/src/services/complexity_c.rs
pub struct CComplexityAnalyzer {
    cfg: ControlFlowGraph,
    ast: CAst,
}

impl CComplexityAnalyzer {
    pub fn analyze(&self) -> CComplexityMetrics {
        CComplexityMetrics {
            cyclomatic: self.calculate_cyclomatic(),
            cognitive: self.calculate_cognitive(),
            goto_complexity: self.calculate_goto_penalty(),
            macro_complexity: self.estimate_macro_complexity(),
            pointer_depth: self.analyze_pointer_depth(),
            nesting_depth: self.calculate_max_nesting(),
        }
    }
    
    /// McCabe's V(G) = E - N + 2P
    fn calculate_cyclomatic(&self) -> u32 {
        let edges = self.cfg.edges.len() as u32;
        let nodes = self.cfg.nodes.len() as u32;
        let connected_components = 1u32;  // Single function
        
        edges - nodes + 2 * connected_components
    }
    
    /// Goto statements significantly increase complexity
    fn calculate_goto_penalty(&self) -> u32 {
        self.cfg.edges.iter()
            .filter(|e| matches!(e.kind, CfgEdgeKind::Goto))
            .count() as u32 * 3  // Each goto multiplies complexity
    }
    
    /// Conservative estimate of macro expansion complexity
    fn estimate_macro_complexity(&self) -> u32 {
        self.ast.macro_expansions.iter()
            .map(|expansion| {
                match expansion.kind {
                    MacroKind::ObjectLike => 1,
                    MacroKind::FunctionLike { param_count } => param_count + 1,
                    MacroKind::Variadic => 3,  // Variadic macros are complex
                }
            })
            .sum()
    }
}
```

## 4. Phase 2: C++ Analysis with Pointer Analysis

### 4.1 Steensgaard's Pointer Analysis

Following the reviewer's recommendation, we implement Steensgaard's algorithm for scalable pointer analysis:

```rust
// server/src/services/pointer_analysis.rs
pub struct SteensgaardAnalysis {
    /// Union-find structure for equivalence classes
    points_to_sets: UnionFind<AbstractLocation>,
    /// Constraints collected from the program
    constraints: Vec<PointerConstraint>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum AbstractLocation {
    /// Stack variable
    StackVar { function: FunctionId, name: String },
    /// Heap allocation site
    HeapSite { id: AllocationSiteId },
    /// Global variable
    Global { name: String },
    /// Function (for function pointers)
    Function { id: FunctionId },
    /// Field of a struct
    Field { base: Box<AbstractLocation>, offset: FieldOffset },
    /// Unknown location (top element)
    Unknown,
}

#[derive(Debug, Clone)]
pub enum PointerConstraint {
    /// p = &q
    AddressOf { ptr: AbstractLocation, pointee: AbstractLocation },
    /// p = q
    Copy { from: AbstractLocation, to: AbstractLocation },
    /// p = *q
    Load { ptr: AbstractLocation, pointee: AbstractLocation },
    /// *p = q
    Store { ptr: AbstractLocation, value: AbstractLocation },
}

impl SteensgaardAnalysis {
    /// O(n α(n)) algorithm where α is inverse Ackermann
    pub fn analyze(&mut self, program: &Program) -> PointsToGraph {
        // Step 1: Collect constraints
        for function in &program.functions {
            self.collect_constraints(function);
        }
        
        // Step 2: Solve constraints using union-find
        for constraint in &self.constraints {
            match constraint {
                PointerConstraint::AddressOf { ptr, pointee } => {
                    // ptr points to pointee
                    self.points_to_sets.union(ptr, pointee);
                },
                PointerConstraint::Copy { from, to } => {
                    // to and from point to same set
                    let from_target = self.points_to_sets.find(from);
                    let to_target = self.points_to_sets.find(to);
                    self.points_to_sets.union(from_target, to_target);
                },
                PointerConstraint::Load { ptr, pointee } => {
                    // pointee = *ptr, so unify pointee with ptr's target
                    let ptr_target = self.points_to_sets.find(ptr);
                    self.points_to_sets.union(pointee, ptr_target);
                },
                PointerConstraint::Store { ptr, value } => {
                    // *ptr = value, so unify ptr's target with value
                    let ptr_target = self.points_to_sets.find(ptr);
                    self.points_to_sets.union(ptr_target, value);
                },
            }
        }
        
        // Step 3: Build points-to graph from equivalence classes
        self.build_points_to_graph()
    }
    
    fn collect_constraints(&mut self, function: &Function) {
        for stmt in &function.statements {
            match stmt {
                Statement::Assignment { lhs, rhs } => {
                    self.handle_assignment(lhs, rhs);
                },
                Statement::Call { target, args, .. } => {
                    // Function pointer calls require points-to info
                    if let Some(fp) = self.as_function_pointer(target) {
                        self.constraints.push(PointerConstraint::Load {
                            ptr: fp,
                            pointee: AbstractLocation::Unknown,
                        });
                    }
                },
                _ => {},
            }
        }
    }
}
```

### 4.2 C++ Virtual Method Resolution

```rust
// server/src/services/cpp_analysis.rs
pub struct VirtualMethodResolver {
    class_hierarchy: ClassHierarchy,
    vtable_layouts: HashMap<ClassId, VTableLayout>,
    pointer_analysis: Arc<PointsToGraph>,
}

#[derive(Debug, Clone)]
pub struct VTableLayout {
    pub class_id: ClassId,
    pub entries: Vec<VTableEntry>,
    pub offset_to_top: isize,
    pub typeinfo: TypeInfo,
}

#[derive(Debug, Clone)]
pub enum VTableEntry {
    VirtualMethod { decl: MethodId, impl_: MethodId },
    VirtualBase { offset: isize },
    Thunk { target: MethodId, adjustment: isize },
}

impl VirtualMethodResolver {
    pub fn resolve_virtual_call(
        &self, 
        receiver: &Expression,
        method_name: &str,
        arg_types: &[Type],
    ) -> CallTargets {
        // Step 1: Get possible receiver types from pointer analysis
        let receiver_types = self.get_possible_types(receiver);
        
        // Step 2: For each possible type, look up method in vtable
        let mut targets = HashSet::new();
        
        for class_type in receiver_types {
            if let Some(vtable) = self.vtable_layouts.get(&class_type) {
                // Find method in vtable considering covariant return types
                if let Some(entry) = self.lookup_virtual_method(vtable, method_name, arg_types) {
                    targets.insert(entry.impl_);
                }
            }
        }
        
        CallTargets {
            definite: targets.len() == 1,
            possible: targets,
            reason: if targets.is_empty() {
                ResolutionFailure::NoMatchingMethod
            } else {
                ResolutionFailure::None
            },
        }
    }
    
    /// Use pointer analysis to determine possible dynamic types
    fn get_possible_types(&self, expr: &Expression) -> HashSet<ClassId> {
        let abstract_loc = self.expression_to_location(expr);
        let points_to_set = self.pointer_analysis.get_points_to_set(&abstract_loc);
        
        points_to_set.iter()
            .filter_map(|loc| self.location_to_type(loc))
            .filter_map(|ty| self.extract_class_type(&ty))
            .collect()
    }
}
```

### 4.3 Template Instantiation Analysis

```rust
// server/src/services/template_analysis.rs
pub struct TemplateInstantiator {
    /// Cache of instantiated templates
    instantiation_cache: HashMap<TemplateInstanceKey, Arc<InstantiatedTemplate>>,
    /// Track recursive instantiation depth
    depth_limit: usize,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct TemplateInstanceKey {
    template_id: TemplateId,
    arguments: Vec<CanonicalType>,
}

impl TemplateInstantiator {
    pub fn instantiate(
        &mut self,
        template: &Template,
        args: &[TemplateArgument],
    ) -> Result<Arc<InstantiatedTemplate>> {
        // Canonicalize arguments for cache lookup
        let key = self.canonicalize_key(template.id, args)?;
        
        // Check cache first
        if let Some(cached) = self.instantiation_cache.get(&key) {
            return Ok(Arc::clone(cached));
        }
        
        // Prevent infinite recursion
        if self.current_depth() > self.depth_limit {
            return Err(AnalysisError::TemplateInstantiationDepthExceeded);
        }
        
        // Perform substitution
        let instantiated = self.substitute_template(template, args)?;
        
        // Cache result
        let arc = Arc::new(instantiated);
        self.instantiation_cache.insert(key, Arc::clone(&arc));
        
        Ok(arc)
    }
    
    /// SFINAE-aware substitution
    fn substitute_template(
        &mut self,
        template: &Template,
        args: &[TemplateArgument],
    ) -> Result<InstantiatedTemplate> {
        let mut substitution_map = self.build_substitution_map(template, args)?;
        
        match template.kind {
            TemplateKind::Class(ref class_template) => {
                // Substitute in class members
                let members = self.substitute_members(&class_template.members, &substitution_map)?;
                
                // Check constraints (C++20 concepts)
                if let Some(constraints) = &class_template.constraints {
                    self.check_constraints(constraints, &substitution_map)?;
                }
                
                Ok(InstantiatedTemplate::Class(InstantiatedClass {
                    template_id: template.id,
                    arguments: args.to_vec(),
                    members,
                    vtable: self.build_vtable(&members)?,
                }))
            },
            TemplateKind::Function(ref func_template) => {
                // Function template instantiation with overload resolution
                self.instantiate_function_template(func_template, &substitution_map)
            },
            TemplateKind::Variable(ref var_template) => {
                // Variable template (C++14)
                self.instantiate_variable_template(var_template, &substitution_map)
            },
        }
    }
}
```

## 5. Phase 3: Cross-Language Symbol Resolution

### 5.1 Symbol Resolution Strategy

```rust
// server/src/services/symbol_resolver.rs
pub struct CrossLanguageSymbolResolver {
    /// Symbol tables per compilation unit
    compilation_units: HashMap<PathBuf, SymbolTable>,
    /// Name demangler for C++
    demangler: CppDemangler,
    /// ABI-specific rules
    abi_rules: AbiRules,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub id: SymbolId,
    pub name: SymbolName,
    pub kind: SymbolKind,
    pub linkage: Linkage,
    pub visibility: Visibility,
    pub defined_in: Option<PathBuf>,
    pub referenced_from: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum SymbolName {
    /// Plain C name
    C(String),
    /// Mangled C++ name
    CppMangled { mangled: String, demangled: String },
    /// Cython generated name
    Cython { python_name: String, c_name: String },
}

impl CrossLanguageSymbolResolver {
    /// Resolve symbols across compilation units without a linker
    pub fn resolve_symbols(&mut self) -> Result<ResolutionResult> {
        let mut unresolved = Vec::new();
        let mut resolved = HashMap::new();
        let mut conflicts = Vec::new();
        
        // Phase 1: Collect all exported symbols
        let mut exports: HashMap<String, Vec<(SymbolId, &Symbol)>> = HashMap::new();
        
        for (path, table) in &self.compilation_units {
            for symbol in table.exported_symbols() {
                let canonical_name = self.canonicalize_name(&symbol.name)?;
                exports.entry(canonical_name)
                    .or_default()
                    .push((symbol.id, symbol));
            }
        }
        
        // Phase 2: Resolve imports
        for (path, table) in &self.compilation_units {
            for import in table.imported_symbols() {
                let canonical_name = self.canonicalize_name(&import.name)?;
                
                match exports.get(&canonical_name) {
                    Some(candidates) if candidates.len() == 1 => {
                        // Unique match
                        resolved.insert(import.id, candidates[0].0);
                    },
                    Some(candidates) if candidates.len() > 1 => {
                        // Multiple definitions - need disambiguation
                        match self.disambiguate(import, candidates) {
                            Ok(target) => resolved.insert(import.id, target),
                            Err(e) => conflicts.push(SymbolConflict {
                                symbol: import.id,
                                candidates: candidates.iter().map(|(id, _)| *id).collect(),
                                reason: e,
                            }),
                        };
                    },
                    None => {
                        // No definition found
                        unresolved.push(UnresolvedSymbol {
                            id: import.id,
                            name: canonical_name,
                            referenced_from: path.clone(),
                        });
                    },
                }
            }
        }
        
        Ok(ResolutionResult {
            resolved,
            unresolved,
            conflicts,
        })
    }
    
    /// Apply ABI-specific rules for name canonicalization
    fn canonicalize_name(&self, name: &SymbolName) -> Result<String> {
        match name {
            SymbolName::C(s) => Ok(s.clone()),
            SymbolName::CppMangled { mangled, .. } => {
                // Demangle according to ABI
                self.demangler.demangle(mangled)
            },
            SymbolName::Cython { c_name, .. } => {
                // Cython uses C ABI
                Ok(c_name.clone())
            },
        }
    }
}
```

### 5.2 Link-Time Analysis Without Linking

```rust
// server/src/services/whole_program_analysis.rs
pub struct WholeProgram{
    /// All compilation units
    units: Vec<CompilationUnit>,
    /// Cross-unit symbol resolution
    symbol_resolver: CrossLanguageSymbolResolver,
    /// Interprocedural call graph
    call_graph: CallGraph,
    /// Pointer analysis results
    points_to: Arc<PointsToGraph>,
}

impl WholeProgram{
    /// Build whole-program view without actual linking
    pub fn analyze(project: &Project) -> Result<Self> {
        // Step 1: Parse all compilation units in parallel
        let units = project.files
            .par_iter()
            .map(|file| Self::analyze_unit(file))
            .collect::<Result<Vec<_>>>()?;
        
        // Step 2: Resolve symbols across units
        let mut symbol_resolver = CrossLanguageSymbolResolver::new();
        for unit in &units {
            symbol_resolver.add_unit(&unit)?;
        }
        let resolution = symbol_resolver.resolve_symbols()?;
        
        // Step 3: Build interprocedural call graph
        let mut call_graph_builder = CallGraphBuilder::new(&resolution);
        for unit in &units {
            call_graph_builder.add_unit(&unit)?;
        }
        
        // Step 4: Run whole-program pointer analysis
        let pointer_analyzer = SteensgaardAnalysis::new();
        let points_to = pointer_analyzer.analyze_whole_program(&units)?;
        
        // Step 5: Refine call graph with pointer analysis
        call_graph_builder.refine_with_pointer_analysis(&points_to)?;
        let call_graph = call_graph_builder.build();
        
        Ok(Self {
            units,
            symbol_resolver,
            call_graph,
            points_to: Arc::new(points_to),
        })
    }
    
    /// Interprocedural reachability for dead code detection
    pub fn compute_reachability(&self) -> ReachabilityAnalysis {
        let mut reachable = FixedBitSet::with_capacity(self.call_graph.num_functions());
        let mut work_list = Vec::new();
        
        // Initialize with entry points
        for entry in self.find_entry_points() {
            reachable.insert(entry.0);
            work_list.push(entry);
        }
        
        // Fixed-point computation
        while let Some(func_id) = work_list.pop() {
            // Direct calls
            for callee in self.call_graph.direct_callees(func_id) {
                if !reachable[callee.0] {
                    reachable.insert(callee.0);
                    work_list.push(callee);
                }
            }
            
            // Indirect calls through function pointers
            for call_site in self.call_graph.indirect_call_sites(func_id) {
                let targets = self.resolve_indirect_call(&call_site);
                for target in targets {
                    if !reachable[target.0] {
                        reachable.insert(target.0);
                        work_list.push(target);
                    }
                }
            }
        }
        
        ReachabilityAnalysis { reachable }
    }
}
```

## 6. Phase 4: Cython Integration

### 6.1 Cython Type System Bridge

```rust
// server/src/services/cython_analysis.rs
pub struct CythonTypeSystem {
    /// Maps between Python types and C types
    type_mappings: HashMap<PyType, CType>,
    /// Inferred types for expressions
    type_environment: TypeEnvironment,
    /// GIL state tracking
    gil_tracker: GilStateTracker,
}

#[derive(Debug, Clone)]
pub enum CythonType {
    /// Pure Python type
    Python(PyType),
    /// C type in Cython
    C(CType),
    /// Fused type (generic)
    Fused { name: String, options: Vec<CythonType> },
    /// Memory view type
    MemoryView { 
        base: Box<CythonType>, 
        ndim: usize,
        layout: MemoryLayout 
    },
}

#[derive(Debug, Clone)]
pub enum MemoryLayout {
    C,          // C-contiguous
    Fortran,    // Fortran-contiguous  
    Strided,    // Any strided
    Direct,     // Direct access (::1)
}

impl CythonTypeSystem {
    /// Infer types using Hindley-Milner style inference
    pub fn infer_types(&mut self, module: &CythonModule) -> Result<TypedModule> {
        let mut constraints = Vec::new();
        
        // Generate constraints from declarations
        for decl in &module.declarations {
            match decl {
                CythonDecl::Cdef { var, type_annotation, .. } => {
                    if let Some(ann) = type_annotation {
                        constraints.push(TypeConstraint::Equal(
                            TypeVar::Variable(var.clone()),
                            self.parse_annotation(ann)?,
                        ));
                    }
                },
                CythonDecl::Cpdef { func, params, return_type, .. } => {
                    // Function has both Python and C signatures
                    self.generate_function_constraints(func, params, return_type, &mut constraints)?;
                },
                _ => {},
            }
        }
        
        // Solve constraints
        let substitution = self.solve_constraints(constraints)?;
        
        // Apply substitution
        Ok(self.apply_substitution(&module, &substitution)?)
    }
    
    /// Track GIL state for nogil analysis
    pub fn analyze_gil_safety(&self, func: &CythonFunction) -> GilAnalysis {
        let mut state = GilState::Held;
        let mut violations = Vec::new();
        
        for stmt in &func.body {
            match stmt {
                Statement::With { context, body } if context == "nogil" => {
                    let old_state = state;
                    state = GilState::Released;
                    
                    // Check body for GIL-requiring operations
                    for inner in body {
                        if let Some(violation) = self.check_gil_requirement(inner, state) {
                            violations.push(violation);
                        }
                    }
                    
                    state = old_state;
                },
                Statement::Call { target, .. } => {
                    // Check if function requires GIL
                    if self.requires_gil(target) && state == GilState::Released {
                        violations.push(GilViolation {
                            location: stmt.span.clone(),
                            operation: format!("Call to {} requires GIL", target),
                        });
                    }
                },
                _ => {},
            }
        }
        
        GilAnalysis {
            safe: violations.is_empty(),
            violations,
            can_release_gil: self.can_release_gil(&func),
        }
    }
}
```

### 6.2 Buffer Protocol Analysis

```rust
// server/src/services/buffer_protocol.rs
pub struct BufferProtocolAnalyzer {
    type_system: Arc<CythonTypeSystem>,
}

impl BufferProtocolAnalyzer {
    pub fn analyze_memory_view(&self, view_expr: &MemoryViewExpr) -> Result<BufferAnalysis> {
        let base_type = self.type_system.get_type(&view_expr.base)?;
        
        // Parse memory view syntax: type[:,:,::1]
        let dimensions = self.parse_dimensions(&view_expr.slice_spec)?;
        
        // Determine access patterns
        let access_pattern = dimensions.iter()
            .map(|dim| match dim {
                SliceSpec::Full => AccessPattern::Sequential,
                SliceSpec::Strided(1) => AccessPattern::Direct,
                SliceSpec::Strided(n) => AccessPattern::Strided(*n),
            })
            .collect();
        
        // Check for efficient access patterns
        let is_c_contiguous = self.is_c_contiguous(&access_pattern);
        let allows_nogil = base_type.is_numeric() && is_c_contiguous;
        
        Ok(BufferAnalysis {
            base_type,
            dimensions: dimensions.len(),
            access_pattern,
            is_c_contiguous,
            allows_nogil,
            estimated_cache_behavior: self.estimate_cache_behavior(&access_pattern),
        })
    }
    
    /// Generate efficient Rust equivalent for transpilation
    pub fn suggest_rust_type(&self, buffer: &BufferAnalysis) -> RustType {
        match (buffer.base_type, buffer.dimensions, buffer.is_c_contiguous) {
            (CythonType::C(CType::Double), 1, true) => {
                RustType::Slice(Box::new(RustType::F64))
            },
            (CythonType::C(CType::Double), 2, true) => {
                RustType::Custom("ndarray::ArrayView2<f64>".to_string())
            },
            _ => RustType::Custom("Vec<u8>".to_string()), // Fallback
        }
    }
}
```

## 7. Abstract Interpretation Framework

### 7.1 Generic Abstract Domain Infrastructure

```rust
// server/src/analysis/abstract_interpretation.rs
pub trait AbstractDomain: Clone + Eq + Debug {
    /// Bottom element (unreachable)
    fn bottom() -> Self;
    
    /// Top element (unknown)
    fn top() -> Self;
    
    /// Partial order
    fn less_equal(&self, other: &Self) -> bool;
    
    /// Join (least upper bound)
    fn join(&self, other: &Self) -> Self;
    
    /// Meet (greatest lower bound)  
    fn meet(&self, other: &Self) -> Self;
    
    /// Widening (for convergence)
    fn widen(&self, other: &Self) -> Self {
        // Default: same as join
        self.join(other)
    }
    
    /// Narrowing (for precision)
    fn narrow(&self, other: &Self) -> Self {
        // Default: same as meet
        self.meet(other)
    }
}

pub struct AbstractInterpreter<D: AbstractDomain> {
    domain: PhantomData<D>,
    widening_delay: usize,
}

impl<D: AbstractDomain> AbstractInterpreter<D> {
    pub fn analyze_function(
        &self,
        func: &Function,
        cfg: &ControlFlowGraph,
        transfer: &impl TransferFunction<D>,
    ) -> Result<Solution<D>> {
        let mut state: HashMap<NodeId, D> = HashMap::new();
        let mut work_list: BinaryHeap<WorkItem> = BinaryHeap::new();
        let mut iteration_count: HashMap<NodeId, usize> = HashMap::new();
        
        // Initialize entry
        state.insert(cfg.entry, transfer.initial_state());
        work_list.push(WorkItem { node: cfg.entry, priority: 0 });
        
        while let Some(WorkItem { node, .. }) = work_list.pop() {
            let current_state = state.get(&node).cloned().unwrap_or_else(D::bottom);
            
            // Apply transfer function
            let new_state = transfer.apply(&cfg.nodes[&node], &current_state)?;
            
            // Process successors
            for &succ in &cfg.nodes[&node].successors {
                let old_succ_state = state.get(&succ).cloned().unwrap_or_else(D::bottom);
                
                // Join with predecessor
                let mut joined = old_succ_state.join(&new_state);
                
                // Apply widening if needed
                let succ_iterations = iteration_count.entry(succ).or_insert(0);
                if *succ_iterations > self.widening_delay {
                    joined = old_succ_state.widen(&joined);
                }
                *succ_iterations += 1;
                
                // Check if state changed
                if !joined.less_equal(&old_succ_state) {
                    state.insert(succ, joined);
                    work_list.push(WorkItem { 
                        node: succ, 
                        priority: cfg.reverse_postorder[&succ],
                    });
                }
            }
        }
        
        Ok(Solution { states: state })
    }
}
```

### 7.2 Interval Domain for Bounds Checking

```rust
// server/src/analysis/domains/interval.rs
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Interval {
    pub min: Bound,
    pub max: Bound,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Bound {
    NegInf,
    Finite(i64),
    PosInf,
}

impl AbstractDomain for Interval {
    fn bottom() -> Self {
        Interval { 
            min: Bound::PosInf, 
            max: Bound::NegInf,
        }
    }
    
    fn top() -> Self {
        Interval {
            min: Bound::NegInf,
            max: Bound::PosInf,
        }
    }
    
    fn less_equal(&self, other: &Self) -> bool {
        self.min >= other.min && self.max <= other.max
    }
    
    fn join(&self, other: &Self) -> Self {
        Interval {
            min: self.min.min(other.min.clone()),
            max: self.max.max(other.max.clone()),
        }
    }
    
    fn widen(&self, other: &Self) -> Self {
        Interval {
            min: if other.min < self.min { Bound::NegInf } else { self.min.clone() },
            max: if other.max > self.max { Bound::PosInf } else { self.max.clone() },
        }
    }
}

pub struct IntervalAnalysis;

impl TransferFunction<Interval> for IntervalAnalysis {
    fn apply(&self, node: &CfgNode, state: &Interval) -> Result<Interval> {
        match &node.kind {
            CfgNodeKind::Statement(stmt) => match stmt {
                Statement::Assignment { lhs, rhs } => {
                    // Evaluate RHS in current state
                    let rhs_interval = self.evaluate_expr(rhs, state)?;
                    Ok(rhs_interval)
                },
                Statement::ArrayAccess { array, index } => {
                    // Check bounds
                    let index_interval = self.evaluate_expr(index, state)?;
                    let array_bounds = self.get_array_bounds(array)?;
                    
                    if !self.definitely_in_bounds(&index_interval, &array_bounds) {
                        // Potential out-of-bounds access
                        self.report_warning(BoundsWarning {
                            location: stmt.span.clone(),
                            index_range: index_interval,
                            array_size: array_bounds,
                        });
                    }
                    
                    Ok(state.clone())
                },
                _ => Ok(state.clone()),
            },
            _ => Ok(state.clone()),
        }
    }
}
```

## 8. Testing Strategy

### 8.1 Property-Based Testing for Analysis Soundness

```rust
#[cfg(test)]
mod analysis_properties {
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn pointer_analysis_soundness(program in arb_program()) {
            let analysis = SteensgaardAnalysis::new();
            let result = analysis.analyze(&program);
            
            // Property 1: Points-to sets form equivalence classes
            for (p1, p2) in program.all_pointer_pairs() {
                if result.alias(p1, p2) {
                    prop_assert!(result.alias(p2, p1), "Symmetry violated");
                    
                    // Transitivity
                    for p3 in program.all_pointers() {
                        if result.alias(p2, p3) {
                            prop_assert!(result.alias(p1, p3), "Transitivity violated");
                        }
                    }
                }
            }
            
            // Property 2: Conservative approximation
            let precise = AndersenAnalysis::new().analyze(&program);
            for (p1, p2) in program.all_pointer_pairs() {
                if precise.alias(p1, p2) {
                    prop_assert!(result.alias(p1, p2), 
                        "Steensgaard should over-approximate Andersen");
                }
            }
        }
        
        #[test]
        fn cfg_construction_preserves_semantics(func in arb_c_function()) {
            let cfg = CfgBuilder::build(&func)?;
            
            // All paths from entry to exit represent valid executions
            for path in cfg.all_paths(cfg.entry, cfg.exit) {
                let trace = execute_path(&func, &path);
                prop_assert!(trace.is_valid(), "CFG contains invalid path");
            }
            
            // All reachable statements are in CFG
            for stmt in func.statements() {
                if stmt.is_reachable() {
                    prop_assert!(cfg.contains_statement(stmt), 
                        "Reachable statement missing from CFG");
                }
            }
        }
    }
}
```

### 8.2 Regression Tests for Known Patterns

```rust
#[test]
fn test_complex_goto_patterns() {
    // Duff's device
    let code = r#"
        void duff_device(int *to, int *from, int count) {
            int n = (count + 7) / 8;
            switch (count % 8) {
                case 0: do { *to++ = *from++;
                case 7:      *to++ = *from++;
                case 6:      *to++ = *from++;
                case 5:      *to++ = *from++;
                case 4:      *to++ = *from++;
                case 3:      *to++ = *from++;
                case 2:      *to++ = *from++;
                case 1:      *to++ = *from++;
                } while (--n > 0);
            }
        }
    "#;
    
    let analyzer = CComplexityAnalyzer::new();
    let metrics = analyzer.analyze_string(code).unwrap();
    
    // Duff's device should have high complexity
    assert!(metrics.cyclomatic > 10);
    assert!(metrics.goto_complexity > 0); // Implicit gotos in switch
}

#[test]
fn test_template_sfinae_patterns() {
    let code = r#"
        template<typename T>
        auto has_iterator_helper(int) -> decltype(
            std::declval<T>().begin(), 
            std::declval<T>().end(), 
            std::true_type{});
        
        template<typename T>
        std::false_type has_iterator_helper(...);
        
        template<typename T>
        struct has_iterator : decltype(has_iterator_helper<T>(0)) {};
    "#;
    
    let analyzer = CppTemplateAnalyzer::new();
    let result = analyzer.analyze_string(code).unwrap();
    
    assert_eq!(result.sfinae_patterns.len(), 1);
    assert!(result.requires_substitution_failure_handling);
}
```

## 9. Performance Optimization

### 9.1 Incremental Analysis Architecture

```rust
// server/src/incremental/mod.rs
pub struct IncrementalAnalyzer {
    /// Previous analysis results
    previous_state: AnalysisState,
    /// Dependency graph between functions
    dependencies: DependencyGraph,
    /// File modification tracker
    file_tracker: FileTracker,
}

impl IncrementalAnalyzer {
    pub fn analyze_changes(&mut self, changes: &[FileChange]) -> Result<AnalysisDelta> {
        let mut affected = HashSet::new();
        let mut work_list = Vec::new();
        
        // Phase 1: Identify directly affected functions
        for change in changes {
            match change {
                FileChange::Modified { path, regions } => {
                    let affected_funcs = self.find_affected_functions(path, regions)?;
                    affected.extend(affected_funcs);
                    work_list.extend(affected_funcs);
                },
                FileChange::Created { path } => {
                    // New file - analyze completely
                    let new_funcs = self.analyze_new_file(path)?;
                    affected.extend(new_funcs);
                },
                FileChange::Deleted { path } => {
                    // Remove from analysis
                    self.remove_file_analysis(path)?;
                },
            }
        }
        
        // Phase 2: Propagate changes through dependency graph
        while let Some(func) = work_list.pop() {
            for dependent in self.dependencies.dependents_of(func) {
                if affected.insert(dependent) {
                    work_list.push(dependent);
                }
            }
        }
        
        // Phase 3: Reanalyze affected functions
        let mut delta = AnalysisDelta::new();
        for func in affected {
            let old_result = self.previous_state.get(func);
            let new_result = self.analyze_function(func)?;
            
            if old_result != Some(&new_result) {
                delta.changed.insert(func, new_result);
            }
        }
        
        Ok(delta)
    }
}
```

### 9.2 Parallel Analysis Pipeline

```rust
pub struct ParallelAnalyzer {
    thread_pool: ThreadPool,
    work_stealing_queue: WorkStealingQueue<AnalysisTask>,
}

impl ParallelAnalyzer {
    pub fn analyze_project(&self, project: &Project) -> Result<ProjectAnalysis> {
        // Stage 1: Parse all files in parallel (embarrassingly parallel)
        let parsed_files: Vec<_> = project.files
            .par_iter()
            .map(|file| self.parse_file(file))
            .collect::<Result<_>>()?;
        
        // Stage 2: Build symbol tables (requires coordination)
        let symbol_tables = self.build_symbol_tables_parallel(&parsed_files)?;
        
        // Stage 3: Intraprocedural analysis (parallel per function)
        let local_analyses: HashMap<_, _> = parsed_files
            .par_iter()
            .flat_map(|file| file.functions.par_iter())
            .map(|func| {
                let analysis = self.analyze_function_local(func)?;
                Ok((func.id, analysis))
            })
            .collect::<Result<_>>()?;
        
        // Stage 4: Interprocedural analysis (requires serialization)
        let whole_program = self.build_whole_program_view(&local_analyses)?;
        
        // Stage 5: Cross-language analysis
        let cross_language = self.analyze_cross_language(&whole_program)?;
        
        Ok(ProjectAnalysis {
            files: parsed_files,
            local_analyses,
            whole_program,
            cross_language,
        })
    }
}
```

## 10. Revised Implementation Timeline

### Phase 1: Foundation (Months 1-3)
- **Month 1**: Parser infrastructure, C AST extraction, CFG construction
- **Month 2**: Path-insensitive memory analysis, intraprocedural complexity metrics
- **Month 3**: Testing framework, property-based tests, initial optimizations

### Phase 2: C++ Support (Months 4-6)
- **Month 4**: C++ parser integration, class hierarchy analysis, vtable construction
- **Month 5**: Steensgaard pointer analysis implementation and validation
- **Month 6**: Template instantiation, SFINAE handling, overload resolution

### Phase 3: Interprocedural Analysis (Months 7-9)
- **Month 7**: Symbol resolution without linker, name demangling
- **Month 8**: Interprocedural call graph, refined with pointer analysis
- **Month 9**: Whole-program dead code detection, cross-unit optimization

### Phase 4: Cython Integration (Months 10-12)
- **Month 10**: Cython parser, type inference, GIL analysis
- **Month 11**: Buffer protocol, memory view optimization
- **Month 12**: Cross-language FFI analysis, transpilation support

This revised specification acknowledges the fundamental complexity of the problem while providing a realistic path to implementation. The phased approach ensures each component is thoroughly tested before building upon it, and the explicit algorithm choices (Steensgaard over Andersen, path-insensitive before path-sensitive) reflect practical engineering trade-offs between precision and scalability.

## 11. File Discovery and .gitignore Integration

### 11.1 Ripgrep-Style Ignore Patterns

The C/C++ AST analysis must respect repository ignore patterns following ripgrep's approach:

```rust
// File discovery must use the ignore crate with proper configuration
let mut builder = WalkBuilder::new(&project_root);
builder
    .standard_filters(true)         // Enable .gitignore, .ignore, etc.
    .hidden(true)                   // Skip hidden files by default
    .parents(true)                  // Check parent dirs for ignore files
    .git_ignore(true)              // Respect .gitignore
    .git_global(true)              // Respect global gitignore
    .git_exclude(true)             // Respect .git/info/exclude
    .add_custom_ignore_filename(".paimlignore"); // Custom ignore file
```

### 11.2 Build Artifact Filtering

The analysis must skip common C/C++ build artifacts:

```rust
fn is_c_build_artifact(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    
    // Object files and libraries
    if path_str.ends_with(".o") || path_str.ends_with(".a") || 
       path_str.ends_with(".so") || path_str.ends_with(".dylib") ||
       path_str.ends_with(".dll") || path_str.ends_with(".lib") {
        return true;
    }
    
    // Build directories
    if path_str.contains("/build/") || path_str.contains("/cmake-build-") ||
       path_str.contains("/out/") || path_str.contains("/.build/") {
        return true;
    }
    
    // CMake artifacts
    if path_str.contains("/CMakeFiles/") || path_str.ends_with("CMakeCache.txt") {
        return true;
    }
    
    false
}
```

### 11.3 Integration Testing Requirements

```rust
#[test]
fn test_c_analysis_respects_gitignore() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create source files
    fs::write(temp_dir.path().join("main.c"), "int main() {}").unwrap();
    fs::write(temp_dir.path().join("test.c"), "void test() {}").unwrap();
    
    // Create build artifacts
    fs::create_dir(temp_dir.path().join("build")).unwrap();
    fs::write(temp_dir.path().join("build/main.o"), "binary").unwrap();
    
    // Create .gitignore
    fs::write(temp_dir.path().join(".gitignore"), "build/\n*.o\n").unwrap();
    
    // Run analysis
    let analyzer = CAnalyzer::new();
    let files = analyzer.discover_files(temp_dir.path()).unwrap();
    
    // Should only find source files, not build artifacts
    assert_eq!(files.len(), 2);
    assert!(files.iter().any(|f| f.ends_with("main.c")));
    assert!(files.iter().any(|f| f.ends_with("test.c")));
    assert!(!files.iter().any(|f| f.ends_with(".o")));
}
```