## Makefile Lint Feature

The codebase contains a full Makefile linter implementation in `./server/src/services/makefile_linter/` with:
- AST-based parser supporting GNU Make syntax
- Rule engine with checkmake-compatible rules (MinPhonyRule, UndefinedVariableRule, RecursiveExpansionRule)
- Performance analysis detecting expensive variable expansions

**Not exposed** - no CLI command, HTTP endpoint, or MCP tool. The linter exists but isn't wired to any user interface.

## Provability Feature

Two provability subsystems exist:
1. **Lightweight Provability Analyzer** (`lightweight_provability_analyzer.rs`):
    - Lattice-based abstract interpretation for nullability, aliasing, purity
    - Function-level proof summaries with verified properties
    - Incremental analysis support

2. **Proof Annotator** (`proof_annotator.rs`):
    - AST node annotation with proof metadata
    - Memory safety and thread safety verification hooks
    - Cache-aware proof collection

**Not exposed** - internal infrastructure only. The `ProofAnnotation` struct in `unified_ast.rs` suggests integration with AST analysis, but no user-facing commands access these capabilities.

Both features represent significant engineering investment in formal methods infrastructure that remains dormant in the current release.