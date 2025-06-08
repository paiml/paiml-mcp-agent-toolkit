# Project Context

## Project Structure

- **Language**: rust
- **Total Files**: 12
- **Total Functions**: 31
- **Total Structs**: 61
- **Total Enums**: 34
- **Total Traits**: 0

## Files

### server/src/models/churn.rs

- use statement
- use statement
- use statement
- use statement
- CodeChurnAnalysis
- FileChurnMetrics
- ChurnSummary
- ChurnOutputFormat

### server/src/models/complexity_bound.rs

- use statement
- use statement
- BigOClass
- InputVariable
- ComplexityFlags
- ComplexityBound
- CacheComplexity
- RecurrenceRelation
- RecursiveCall
- ComplexityProofType
- use statement
- test_complexity_bound_size
- test_cache_complexity_size
- test_big_o_ordering
- test_complexity_bound_creation
- test_growth_estimation
- test_master_theorem

### server/src/models/dag.rs

- use statement
- use statement
- DependencyGraph
- NodeInfo
- Edge
- NodeType
- EdgeType

### server/src/models/dead_code.rs

- use statement
- use statement
- FileDeadCodeMetrics
- ConfidenceLevel
- DeadCodeItem
- DeadCodeType
- DeadCodeRankingResult
- DeadCodeSummary
- DeadCodeAnalysisConfig
- use statement
- use statement
- test_file_dead_code_metrics_creation
- test_dead_code_item_creation
- test_dead_code_type_variants
- test_confidence_levels
- test_dead_code_ranking_result
- test_dead_code_summary_from_files
- test_dead_code_analysis_config_default
- test_file_metrics_add_different_item_types
- test_score_calculation_with_different_confidence_levels

### server/src/models/deep_context_config.rs

- use statement
- use statement
- DeepContextConfig
- ComplexityThresholds
- default_dead_code_threshold
- default_cyclomatic_warning
- default_cyclomatic_error
- default_cognitive_warning
- default_cognitive_error
- use statement
- use statement
- use statement
- test_default_config_validation
- test_entry_point_validation
- test_threshold_validation
- test_entry_point_detection
- test_config_serialization

### server/src/models/error.rs

- use statement
- use statement
- TemplateError
- AnalysisError
- PmatError
- use statement
- use statement
- use statement
- use statement
- use statement
- use statement
- use statement
- use statement
- use statement
- ErrorSeverity

### server/src/models/mcp.rs

- use statement
- use statement
- McpRequest
- McpResponse
- McpError
- ToolCallParams
- GenerateTemplateArgs
- ListTemplatesArgs
- ResourceReadParams
- ValidateTemplateArgs
- ScaffoldProjectArgs
- SearchTemplatesArgs
- PromptGetParams
- Prompt
- PromptArgument

### server/src/models/mod.rs


### server/src/models/project_meta.rs

- use statement
- use statement
- MetaFileType
- MetaFile
- CompressedMakefile
- MakeTarget
- CompressedReadme
- CompressedSection
- BuildInfo
- ProjectOverview

### server/src/models/tdg.rs

- use statement
- TDGScore
- TDGComponents
- TDGSeverity
- TDGConfig
- TDGSummary
- TDGHotspot
- TDGAnalysis
- TDGRecommendation
- RecommendationType
- TDGDistribution
- TDGBucket
- use statement
- test_tdg_severity_from_value
- test_tdg_config_default

### server/src/models/template.rs

- use statement
- use statement
- TemplateResource
- Toolchain
- TemplateCategory
- ParameterSpec
- ParameterType
- GeneratedTemplate
- TemplateResponse

### server/src/models/unified_ast.rs

- use statement
- use statement
- use statement
- use statement
- use statement
- use statement
- Language
- NodeFlags
- AstKind
- FunctionKind
- ClassKind
- VarKind
- ImportKind
- ExprKind
- StmtKind
- TypeKind
- ModuleKind
- MacroKind
- ProofAnnotation
- PropertyType
- ConfidenceLevel
- VerificationMethod
- EvidenceType
- Location
- Span
- BytePos
- QualifiedName
- RelativeLocation
- UnifiedAstNode
- ColumnStore
- AstDag
- LanguageParsers
- use statement
- test_node_size
- test_node_alignment
- test_node_flags
- test_ast_dag

