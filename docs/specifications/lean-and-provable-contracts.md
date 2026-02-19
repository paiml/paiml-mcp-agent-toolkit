# Lean 4 First-Class Language Support

**Status**: Active
**Version**: 1.0
**Created**: 2026-02-19

## Overview

Lean 4 is a proof assistant and programming language with dependent types. Unlike other supported languages, Lean 4's quality metric isn't just code complexity — it's **proof completeness** (`sorry` count, theorem coverage). This specification defines first-class support for Lean 4 in PMAT.

## Language Taxonomy

Lean 4 constructs fall into these categories:

| Construct | AST Type | Description |
|-----------|----------|-------------|
| `def` | Function | Regular definitions |
| `noncomputable def` | Function | Non-constructive definitions |
| `theorem` | Function | Proven propositions |
| `lemma` | Function | Helper propositions |
| `structure` | Struct | Record types |
| `class` | Struct | Type classes |
| `inductive` | Struct | Inductive data types |
| `abbrev` | Function | Type abbreviations |
| `axiom` | Function | Unproven axioms |
| `opaque` | Function | Opaque definitions |
| `instance` | Function | Type class instances |
| `namespace` | Module | Scope grouping |

## Proof Quality Metrics

- **sorry count**: Number of incomplete proofs (lower = better)
- **tactic nesting depth**: Depth of `by` tactic blocks
- **axiom usage**: Number of custom axioms (risk indicator)
- **theorem/lemma ratio**: Proven propositions vs total definitions

## Parser Strategy

**Phase 1 (current)**: Pattern-based parsing (like Go, Shell, PHP)
- Line-by-line keyword matching
- No external tree-sitter dependency
- Sufficient for AST item extraction and proof metrics

**Phase 2 (future)**: tree-sitter-lean4 integration
- Full AST with dependent type resolution
- Tactic-level analysis

## Detection

| Signal | Confidence Boost |
|--------|-----------------|
| `lakefile.lean` | +90 |
| `lean-toolchain` | +90 |
| `.lean` extension | standard |

## Feature Gate

- Feature: `lean-ast` (empty feature, pattern-based)
- Included in: `all-languages`, `extended-languages`

## Integration with Falsification System

Lean 4 projects enable the `FormalProofVerification` falsification method:
- Counts `sorry` occurrences across `.lean` files
- Compares against `max_sorry_count` threshold (default: 0)
- Blocks completion when `require_proof_verification` is enabled
