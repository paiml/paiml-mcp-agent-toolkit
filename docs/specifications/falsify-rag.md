# RAG-Powered Popperian Falsification for Specs and Tickets

**Version**: 1.0
**Created**: 2026-02-14
**Status**: SPECIFICATION (Ready for Implementation)
**Methodology**: EXTREME TDD + Toyota Way + Popperian Falsification
**Extends**: `git-history-rag-integration.md`, `documentation-accuracy-enforcement.md`
**Command**: `pmat falsify` (alias: `pmat falsify-spec`)

---

## Executive Summary

Generalize PMAT's existing Popperian falsification (`pmat work complete`) and documentation accuracy enforcement (`pmat validate-readme`) into a unified RAG-powered falsification engine that can falsify **any structured claim** — specifications, tickets, roadmap items, commit messages, and PR descriptions — by cross-referencing claims against the codebase, git history, other specs, and runtime evidence.

**Core Insight**: `pmat validate-readme` answers "is this doc accurate?" (confirmation). `pmat falsify` answers "what evidence **contradicts** this claim?" (falsification). The epistemological difference matters: falsification is strictly more powerful because it actively searches for disconfirming evidence rather than confirming matches.

---

## Toyota Way Principles

| Principle | Japanese | Application |
|-----------|----------|-------------|
| **Jidoka** | 自働化 | Stop the line when a spec claim is falsified — don't ship contradicted specs |
| **Genchi Genbutsu** | 現地現物 | Go to the actual code, git history, test results — never trust claims at face value |
| **Kaizen** | 改善 | Each falsification run improves spec quality incrementally |
| **Muda** | 無駄 | Eliminate waste of implementing against stale/contradicted specs |
| **Poka-Yoke** | ポカヨケ | Error-proof the spec pipeline — catch contradictions before they become code |
| **Andon Cord** | アンドン | Pull the cord when falsification rate exceeds threshold |

---

## 1. Problem Statement

### 1.1 Current State

Three separate systems with overlapping but incomplete falsification:

| System | What it falsifies | Limitation |
|--------|-------------------|------------|
| `pmat validate-readme` | README/CLAUDE.md claims | Only docs, no specs/tickets, confirmation-biased |
| `pmat work complete` | Work contract claims | Only work items, predefined claim types |
| `trueno-ptx-debug` | `FalsificationRegistry` tests | Only PTX-specific, not generalized |

**Gaps**:
- Specifications (`docs/specifications/*.md`) are **never falsified** against the codebase
- Tickets/roadmap items drift from reality with no detection
- No RAG-powered semantic search for disconfirming evidence
- No temporal falsification (spec written before refactor → spec is stale)
- No spec-to-spec contradiction detection
- Claims extracted manually rather than automatically from structured documents

### 1.2 Desired State

A single `pmat falsify` command that:
1. Extracts falsifiable claims from any document (spec, ticket, roadmap entry)
2. Uses RAG (trueno-rag + pmat query) to search for **disconfirming** evidence
3. Applies temporal analysis (git churn) to detect staleness
4. Cross-references specs against each other for contradictions
5. Produces a falsification report with evidence chains
6. Integrates into CI/CD as a quality gate

---

## 2. Scientific Foundation

### 2.1 Popperian Falsification (Philosophy of Science)

**[Popper-1959]** Popper, K.R. "The Logic of Scientific Discovery." Routledge, 1959.
- **Principle**: A claim is scientific only if it is falsifiable — there must exist a possible observation that would prove it wrong
- **Application**: Every spec claim must have a corresponding falsification test. Claims without falsification methods are flagged as "unfalsifiable" (a quality defect)

**[Lakatos-1978]** Lakatos, I. "The Methodology of Scientific Research Programmes." Cambridge University Press, 1978.
- **Principle**: Sophisticated falsification considers auxiliary hypotheses — a failed test might falsify the test setup, not the core theory
- **Application**: When RAG finds contradicting evidence, we score confidence rather than binary pass/fail. A low-confidence contradiction might indicate the search missed context, not that the spec is wrong

### 2.2 Information Retrieval for Falsification

**[SIGIR-2022]** Formal, T., et al. "SPLADE v2: Sparse Lexical and Expansion Model for Information Retrieval." ACM SIGIR, 2022.
- **Finding**: Hybrid retrieval (dense + sparse) outperforms single-method for evidence retrieval
- **Application**: Use trueno-rag BM25 + vector search for disconfirming evidence

**[ACL-2023]** Min, S., et al. "FActScore: Fine-grained Atomic Evaluation of Factual Precision in Long Form Text Generation." ACL, 2023.
- **Finding**: Decomposing text into atomic facts enables precise verification
- **Application**: Decompose spec sections into atomic claims before falsification

**[NeurIPS-2024]** Wei, J., et al. "Measuring Faithfulness in Chain-of-Thought Reasoning." NeurIPS, 2024.
- **Finding**: Chain-of-thought explanations can be unfaithful to actual reasoning; evidence chains must be grounded
- **Application**: Every falsification verdict must cite specific code locations, not abstract reasoning

### 2.3 Temporal Consistency in Software Documentation

**[ICSE-2021]** Wen, F., et al. "An Empirical Study of Documentation Decay in Open-Source Projects." ICSE, 2021.
- **Finding**: 25-30% of documentation becomes stale within 6 months of the code it describes being modified
- **Application**: Churn-weighted staleness scoring — if code has high churn since spec was written, spec has higher falsification priority

**[MSR-2023]** Aghajani, E., et al. "Software Documentation: The Practitioners' Perspective." MSR, 2023.
- **Finding**: Most documentation issues are "what" problems (incorrect facts) not "how" problems (unclear instructions)
- **Application**: Focus claim extraction on factual assertions rather than procedural guidance

---

## 3. Architecture

### 3.1 Pipeline Overview

```
┌─────────────────┐
│  Input Document  │  spec.md, ticket.yaml, roadmap.yaml, PR description
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Claim Extractor  │  Decomposes document into atomic falsifiable claims
│ (ClaimExtractor) │  Reuses src/red_team/claim_extractor.rs patterns
└────────┬────────┘
         │  Vec<FalsifiableClaim>
         ▼
┌─────────────────────────────────────────────────────────┐
│              RAG Falsification Engine                     │
│                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────┐ │
│  │ Code Search  │  │ Git History  │  │ Spec Cross-Ref│ │
│  │ (trueno-rag) │  │ (-G fusion)  │  │ (spec index)  │ │
│  └──────┬───────┘  └──────┬───────┘  └───────┬───────┘ │
│         │                  │                   │         │
│         ▼                  ▼                   ▼         │
│  ┌─────────────────────────────────────────────────┐    │
│  │  Evidence Aggregator (RRF multi-source fusion)  │    │
│  └─────────────────────┬───────────────────────────┘    │
│                        │                                 │
│  ┌─────────────────────▼───────────────────────────┐    │
│  │  Falsification Scorer                            │    │
│  │  - Contradiction score (0.0-1.0)                 │    │
│  │  - Staleness score (churn-weighted)              │    │
│  │  - Confidence score (evidence quality)           │    │
│  └─────────────────────┬───────────────────────────┘    │
└────────────────────────┼────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│                 Falsification Report                     │
│  - Per-claim verdict (survived / falsified / stale)     │
│  - Evidence chains with file:line citations             │
│  - Spec health score                                    │
│  - Recommended actions                                  │
└─────────────────────────────────────────────────────────┘
```

### 3.2 Core Types

```rust
/// Input: a document to falsify
pub struct FalsificationTarget {
    /// Source path (spec file, ticket URL, roadmap entry)
    pub source: TargetSource,

    /// Raw content
    pub content: String,

    /// Document type affects claim extraction strategy
    pub doc_type: DocumentType,
}

pub enum TargetSource {
    /// Local file path
    File(PathBuf),
    /// Roadmap YAML entry (key path)
    RoadmapEntry { file: PathBuf, key: String },
    /// Inline text (e.g., from stdin or MCP tool call)
    Inline(String),
}

pub enum DocumentType {
    /// docs/specifications/*.md — structured spec with requirements
    Specification,
    /// Roadmap YAML entry — status claims, completion claims
    RoadmapEntry,
    /// Commit message — "fix X", "add Y" claims
    CommitMessage,
    /// PR description — summary claims about changes
    PullRequest,
    /// CLAUDE.md / README.md — capability claims (delegates to existing validator)
    AgentDoc,
    /// Free-form markdown — best-effort claim extraction
    Generic,
}

/// A single falsifiable claim extracted from a document
pub struct FalsifiableClaim {
    /// Unique ID within this falsification run
    pub id: String,

    /// The claim text, normalized to a testable assertion
    pub assertion: String,

    /// Original text from the document
    pub original_text: String,

    /// Source location
    pub source_location: SourceLocation,

    /// Claim category determines falsification strategy
    pub category: ClaimCategory,

    /// Entities referenced (functions, files, modules, metrics)
    pub entities: Vec<Entity>,

    /// Whether the claim is absolute ("all", "zero", "always")
    pub is_absolute: bool,

    /// Numeric threshold if present ("95% coverage", ">100 tests")
    pub numeric_value: Option<NumericClaim>,
}

pub enum ClaimCategory {
    /// "Function X does Y" — falsify by finding X and checking behavior
    CodeBehavior,

    /// "File X exists at path Y" — falsify by checking filesystem
    PathReference,

    /// "Module X has complexity < N" — falsify by measuring
    MetricClaim,

    /// "Feature X was added in commit Y" — falsify via git history
    TemporalClaim,

    /// "Spec A requires X" contradicts "Spec B forbids X"
    CrossSpecClaim,

    /// "Coverage is 95%" — falsify by running coverage
    QuantitativeClaim,

    /// "Architecture follows pattern X" — falsify by structural analysis
    ArchitecturalClaim,

    /// "No unsafe code in module X" — falsify by AST search
    AbsenceClaim,

    /// Claims that cannot be mechanically falsified
    Unfalsifiable,
}

pub struct NumericClaim {
    pub value: f64,
    pub unit: String,        // "percent", "count", "seconds", "bytes"
    pub comparator: Comparator, // Gt, Lt, Eq, Gte, Lte
}

pub enum Comparator { Gt, Lt, Eq, Gte, Lte }
```

### 3.3 Evidence Types

```rust
/// Evidence found during falsification
pub struct Evidence {
    /// Where this evidence was found
    pub source: EvidenceSource,

    /// How strongly this evidence contradicts the claim (0.0 = supports, 1.0 = contradicts)
    pub contradiction_score: f64,

    /// Confidence in the evidence itself (0.0 = uncertain, 1.0 = definitive)
    pub confidence: f64,

    /// Human-readable explanation
    pub explanation: String,

    /// Code citation if applicable
    pub citation: Option<CodeCitation>,
}

pub enum EvidenceSource {
    /// Found via semantic code search (trueno-rag)
    CodeSearch { query: String, rank: usize },

    /// Found via git history search (-G)
    GitHistory { commit: String, message: String },

    /// Found in another spec (cross-reference)
    SpecCrossRef { spec_path: PathBuf, line: usize },

    /// Found via AST analysis (pmat context)
    AstAnalysis { file: PathBuf, function: String },

    /// Found via metric measurement
    MetricMeasurement { metric: String, actual: f64, claimed: f64 },

    /// Temporal: code changed significantly after spec was written
    TemporalDrift { spec_date: chrono::NaiveDate, code_churn: f64 },
}

pub struct CodeCitation {
    pub file: PathBuf,
    pub line: usize,
    pub snippet: String, // 3-5 lines of context
}
```

### 3.4 Falsification Verdict

```rust
pub struct FalsificationVerdict {
    pub claim: FalsifiableClaim,
    pub status: VerdictStatus,
    pub evidence: Vec<Evidence>,

    /// Aggregate contradiction score across all evidence
    pub contradiction_score: f64,

    /// Aggregate confidence
    pub confidence: f64,

    /// Staleness score (0.0 = fresh, 1.0 = code completely rewritten since spec)
    pub staleness: f64,
}

pub enum VerdictStatus {
    /// Claim survived falsification — no contradicting evidence found
    Survived,

    /// Claim actively contradicted by evidence
    Falsified,

    /// Claim references code that has changed significantly — likely stale
    Stale,

    /// Claim could not be tested (no falsification method available)
    Unfalsifiable,

    /// Evidence found but inconclusive (low confidence)
    Inconclusive,
}

pub struct FalsificationReport {
    pub target: FalsificationTarget,
    pub verdicts: Vec<FalsificationVerdict>,
    pub summary: ReportSummary,
}

pub struct ReportSummary {
    pub total_claims: usize,
    pub survived: usize,
    pub falsified: usize,
    pub stale: usize,
    pub unfalsifiable: usize,
    pub inconclusive: usize,

    /// Overall spec health (0.0-1.0)
    /// Formula: survived / (total - unfalsifiable)
    pub health_score: f64,

    /// Staleness index (0.0-1.0)
    /// Formula: avg staleness across all claims
    pub staleness_index: f64,
}
```

---

## 4. Falsification Strategies

### 4.1 Code Behavior Claims

**Trigger**: Claim mentions function names, "handles", "parses", "returns", "validates"

```
Claim: "The TDG analyzer assigns grades A-F based on complexity thresholds"

Falsification strategy:
1. pmat query "TDG" "grade" "assign" --include-source --limit 10
2. Extract actual grading logic from source
3. Check if grades A-F exist (not just A-D, for example)
4. Check if complexity is actually used (not just line count)
5. Verdict: If grading uses different criteria → Falsified with evidence
```

### 4.2 Path Reference Claims

**Trigger**: Claim contains file paths, module paths, `src/`, `docs/`

```
Claim: "Configuration is defined in server/src/services/configuration_service.rs"

Falsification strategy:
1. Check if file exists at path
2. If exists, check if it actually defines configuration (not just imports it)
3. If file was renamed/moved, find the new location
4. Verdict: File missing → Falsified; File exists but different purpose → Falsified
```

### 4.3 Metric Claims

**Trigger**: Claim contains numbers, percentages, comparisons

```
Claim: "Coverage exceeds 95% across all modules"

Falsification strategy:
1. Run pmat query --coverage-gaps --limit 5
2. Check if any module is below 95%
3. A single module below threshold falsifies the "all modules" claim
4. Verdict: Module X at 87% → Falsified with specific evidence
```

### 4.4 Temporal Claims

**Trigger**: Claim references time, versions, commits, "added in", "since"

```
Claim: "Kotlin support was added in Sprint 42"

Falsification strategy:
1. pmat query "kotlin" -G --limit 10
2. Find earliest commit mentioning Kotlin
3. Cross-reference commit date with Sprint 42 dates
4. Verdict: If Kotlin commits predate Sprint 42 → Falsified
```

### 4.5 Cross-Spec Contradiction

**Trigger**: Claim makes assertions about constraints, limits, requirements

```
Spec A: "Maximum dependency count: 3,000"
Spec B: "Dependency count is unlimited for sovereign stack projects"

Falsification strategy:
1. Index all specs into a spec-specific RAG index
2. For each constraint claim, search for contradicting constraints
3. Use semantic similarity to find claims about the same concept
4. Verdict: Contradicting constraints found → Both specs flagged
```

### 4.6 Absence Claims

**Trigger**: Claim asserts something does NOT exist ("no unsafe", "zero panics", "no dependencies on X")

```
Claim: "Zero unsafe blocks in the parser module"

Falsification strategy:
1. pmat query --literal "unsafe" --exclude-tests in parser module files
2. A single match falsifies the claim
3. Absence claims are the easiest to falsify — one counterexample suffices
4. Verdict: Found unsafe block at parser/tokenizer.rs:142 → Falsified
```

### 4.7 Staleness Detection

**Trigger**: Applied to ALL claims as a secondary check

```
Claim: "The context module uses HashMap for O(1) lookups" (written 2025-06-01)

Staleness strategy:
1. Identify files referenced by the claim
2. pmat query "context" --churn
3. If context.rs has >50% churn since 2025-06-01, flag as potentially stale
4. Staleness doesn't falsify — it raises priority for re-verification
```

---

## 5. Claim Extraction

### 5.1 Strategy per Document Type

#### Specifications (`docs/specifications/*.md`)

Structured extraction using markdown headers as scope:

```
## Section Header → scope context
- Bullet points → individual claims
- Code blocks → API contract claims (function signatures, struct definitions)
- Tables → metric claims (one per row)
- "MUST", "SHALL", "NEVER" → absolute claims (highest falsification priority)
```

#### Roadmap YAML

```yaml
# roadmap.yaml entry:
- id: PMAT-500
  title: "Add Kotlin support"
  status: complete
  # Claims extracted:
  # 1. Feature "Kotlin support" exists in codebase (status=complete)
  # 2. Feature matches title description
```

#### Commit Messages

```
fix: resolve stack overflow in recursive parser (#1234)

Claims extracted:
1. There was a stack overflow (historical claim — falsify via issue #1234)
2. It was in a recursive parser (code claim — falsify via pmat query)
3. It is now resolved (current state claim — falsify via test/reproduction)
```

### 5.2 Claim Extraction Pipeline

Reuse and extend `src/red_team/claim_extractor.rs`:

```rust
impl ClaimExtractor {
    /// Extract claims from a specification document
    pub fn extract_from_spec(&self, content: &str) -> Vec<FalsifiableClaim> {
        let mut claims = Vec::new();

        // Phase 1: Structural extraction (headers, bullets, tables, code blocks)
        claims.extend(self.extract_structural_claims(content));

        // Phase 2: RFC-2119 keyword extraction (MUST, SHALL, SHOULD, NEVER)
        claims.extend(self.extract_rfc2119_claims(content));

        // Phase 3: Numeric claim extraction (percentages, counts, thresholds)
        claims.extend(self.extract_numeric_claims(content));

        // Phase 4: Path reference extraction (file paths, module paths)
        claims.extend(self.extract_path_claims(content));

        // Phase 5: Entity extraction (function names, struct names, enum variants)
        claims.extend(self.extract_entity_claims(content));

        // Deduplicate overlapping claims
        self.deduplicate(claims)
    }
}
```

### 5.3 RFC-2119 Keyword Priority

| Keyword | Falsification Priority | Rationale |
|---------|----------------------|-----------|
| MUST / SHALL / REQUIRED | P0 (Critical) | Absolute requirement — single counterexample falsifies |
| MUST NOT / SHALL NOT | P0 (Critical) | Absolute prohibition — single occurrence falsifies |
| SHOULD / RECOMMENDED | P1 (High) | Strong expectation — needs pattern of violation |
| SHOULD NOT | P1 (High) | Strong discouragement — needs pattern of occurrence |
| MAY / OPTIONAL | P2 (Low) | No falsification needed — claim is already hedged |

---

## 6. RAG Integration

### 6.1 Index Architecture

Extend the dual-index design from `git-history-rag-integration.md` with a third index:

```
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│  Code Index      │  │  Git Index       │  │  Spec Index      │
│  (trueno-rag)    │  │  (commit msgs)   │  │  (spec claims)   │
│                  │  │                  │  │                  │
│  BM25 + Vector   │  │  BM25 + Vector   │  │  BM25 + Vector   │
│  per-function    │  │  per-commit      │  │  per-claim       │
└────────┬─────────┘  └────────┬─────────┘  └────────┬─────────┘
         │                     │                      │
         └─────────────────────┼──────────────────────┘
                               │
                    ┌──────────▼──────────┐
                    │  RRF Fusion Layer   │
                    │  (k=60, per-source  │
                    │   weight tuning)    │
                    └─────────────────────┘
```

**Spec Index** is new. It indexes:
- Extracted claims from all `docs/specifications/*.md`
- Each claim gets an embedding (trueno-rag BM25 + TF-IDF vector)
- Cross-spec queries find related claims across different specs
- Updated on spec file modification (same staleness rules as code index)

### 6.2 Disconfirming Search Strategy

Standard RAG retrieves the most **similar** results. Falsification RAG must also find **contradicting** results. Two approaches:

#### Approach A: Negation-Augmented Query

```rust
/// For claim "parser handles UTF-8 correctly":
/// Query 1 (standard): "parser UTF-8 handling"
/// Query 2 (negated):  "parser UTF-8 error" OR "parser ASCII only" OR "parser encoding bug"
fn generate_disconfirming_queries(claim: &FalsifiableClaim) -> Vec<String> {
    let mut queries = vec![claim.assertion.clone()];

    // Add negation variants
    if claim.assertion.contains("handles") {
        queries.push(claim.assertion.replace("handles", "fails to handle"));
        queries.push(claim.assertion.replace("handles", "does not support"));
    }
    if claim.assertion.contains("all") {
        queries.push(claim.assertion.replace("all", "some") + " missing");
    }

    // Add fault-pattern variants
    queries.push(format!("{} bug", claim.key_concept()));
    queries.push(format!("{} panic unwrap", claim.key_concept()));
    queries.push(format!("{} todo fixme", claim.key_concept()));

    queries
}
```

#### Approach B: Evidence Polarity Classification

After retrieving top-K results for the standard query, classify each result's **polarity** relative to the claim:

```rust
enum EvidencePolarity {
    /// Evidence supports the claim (function exists, does what spec says)
    Supporting,
    /// Evidence contradicts the claim (function behaves differently)
    Contradicting,
    /// Evidence is about the same topic but neither supports nor contradicts
    Neutral,
}

/// Classify polarity using structural heuristics (no LLM needed):
fn classify_polarity(claim: &FalsifiableClaim, evidence: &SearchResult) -> EvidencePolarity {
    // Path claim + file doesn't exist → Contradicting
    // Metric claim + measured value differs → Contradicting
    // Absence claim + entity found → Contradicting
    // Function claim + function found with matching signature → Supporting
    // ...
}
```

**Decision**: Use both approaches. Approach A broadens recall. Approach B refines precision. No LLM dependency — all heuristic-based for determinism and speed.

### 6.3 Churn-Weighted Staleness

```rust
/// Calculate staleness of a claim based on code churn since spec was written
fn calculate_staleness(
    claim: &FalsifiableClaim,
    spec_modified: chrono::NaiveDate,
    referenced_files: &[PathBuf],
    churn_data: &ChurnIndex,
) -> f64 {
    if referenced_files.is_empty() {
        return 0.0; // Can't measure staleness without file references
    }

    let mut max_staleness = 0.0f64;
    for file in referenced_files {
        if let Some(churn) = churn_data.get_churn_since(file, spec_modified) {
            // churn.score is 0.0-1.0 based on commit frequency and line changes
            max_staleness = max_staleness.max(churn.score);
        }
    }

    max_staleness
}
```

---

## 7. Command Interface

### 7.1 CLI

```bash
# Falsify a single spec
pmat falsify docs/specifications/trueno-o1-context-tdg-integration.md

# Falsify all specs
pmat falsify docs/specifications/

# Falsify a roadmap entry
pmat falsify docs/roadmaps/roadmap.yaml --entry PMAT-500

# Falsify with verbose evidence chains
pmat falsify spec.md --verbose

# Falsify with specific strategies only
pmat falsify spec.md --strategies code-behavior,path-reference,staleness

# JSON output for CI/CD
pmat falsify spec.md --format json -o falsification-report.json

# JUnit output for CI integration
pmat falsify spec.md --format junit -o falsification-results.xml

# Only show falsified/stale claims (skip survived)
pmat falsify spec.md --failures-only

# Set staleness threshold (default: 0.5)
pmat falsify spec.md --staleness-threshold 0.3

# Cross-spec contradiction check across all specs
pmat falsify docs/specifications/ --cross-spec

# Falsify stdin (pipe from ticket system)
echo "Parser handles all UTF-8 input correctly" | pmat falsify --stdin

# Dry run: extract claims only, don't falsify
pmat falsify spec.md --dry-run
```

### 7.2 MCP Tool

```json
{
  "name": "pmat_falsify",
  "description": "Falsify claims in a spec or ticket against the codebase using RAG-powered evidence search",
  "inputSchema": {
    "type": "object",
    "properties": {
      "target": {
        "type": "string",
        "description": "Path to spec file, or inline claim text"
      },
      "strategies": {
        "type": "array",
        "items": { "type": "string" },
        "description": "Falsification strategies to apply"
      },
      "failures_only": {
        "type": "boolean",
        "default": false
      }
    },
    "required": ["target"]
  }
}
```

### 7.3 Integration with `pmat work`

Extend `FalsificationMethod` enum (in `work_contract.rs`):

```rust
pub enum FalsificationMethod {
    // ... existing variants ...

    /// RAG-powered spec falsification (New in v3.0)
    SpecFalsification {
        spec_path: PathBuf,
        strategies: Vec<FalsificationStrategy>,
    },

    /// Cross-spec contradiction detection (New in v3.0)
    CrossSpecContradiction {
        spec_paths: Vec<PathBuf>,
    },
}
```

---

## 8. Output Format

### 8.1 Text Output (Default)

```
Falsifying: docs/specifications/trueno-o1-context-tdg-integration.md
Extracted: 14 falsifiable claims (3 P0, 7 P1, 4 P2)

[1/14] P0 "CSR graphs provide O(1) symbol lookups"
       Strategy: CodeBehavior + MetricClaim
       Evidence: src/services/tdg/tdg_graph.rs:45 — uses HashMap (O(1) amortized, not O(1) worst-case)
       Verdict: SURVIVED (nitpick: amortized vs worst-case, but claim is reasonable)
       Confidence: 0.92

[2/14] P0 "8/8 tests passing for ProjectContextGraph"
       Strategy: QuantitativeClaim
       Evidence: cargo test context_graph — found 8 tests, 8 passing
       Verdict: SURVIVED
       Confidence: 1.00

[3/14] P1 "Every analyze_project_with_cache() builds a ProjectContextGraph"
       Strategy: CodeBehavior + AbsenceClaim
       Evidence: src/services/context.rs:565 — conditional: only builds graph if features enabled
       Verdict: FALSIFIED — claim says "every" but code shows conditional execution
       Confidence: 0.87
       Staleness: 0.34 (context.rs has moderate churn)

...

Summary:
  Total claims:    14
  Survived:        10 (71.4%)
  Falsified:        2 (14.3%)
  Stale:            1 (7.1%)
  Inconclusive:     1 (7.1%)
  Unfalsifiable:    0

  Spec health:     0.83
  Staleness index: 0.21

  Recommended actions:
  1. Fix claim #3: add "when graph features are enabled" qualifier
  2. Update claim #9: file path changed from old_path to new_path
  3. Re-verify claim #12: referenced code has 67% churn since spec was written
```

### 8.2 JSON Output

```json
{
  "target": "docs/specifications/trueno-o1-context-tdg-integration.md",
  "timestamp": "2026-02-14T10:30:00Z",
  "summary": {
    "total_claims": 14,
    "survived": 10,
    "falsified": 2,
    "stale": 1,
    "inconclusive": 1,
    "unfalsifiable": 0,
    "health_score": 0.83,
    "staleness_index": 0.21
  },
  "verdicts": [
    {
      "id": "claim-001",
      "priority": "P0",
      "assertion": "CSR graphs provide O(1) symbol lookups",
      "original_text": "trueno-graph provides CSR graph database for O(1) symbol lookups",
      "source_location": { "file": "trueno-o1-context-tdg-integration.md", "line": 12 },
      "status": "survived",
      "contradiction_score": 0.08,
      "confidence": 0.92,
      "staleness": 0.15,
      "evidence": [
        {
          "source": "code_search",
          "query": "CSR graph symbol lookup",
          "file": "src/services/tdg/tdg_graph.rs",
          "line": 45,
          "snippet": "pub fn get_symbol(&self, name: &str) -> Option<&Symbol> {\n    self.node_map.get(name)\n}",
          "contradiction_score": 0.08,
          "confidence": 0.95,
          "explanation": "Uses HashMap::get which is O(1) amortized — consistent with claim"
        }
      ]
    }
  ]
}
```

---

## 9. Implementation Plan

### Phase 1: Claim Extraction (Week 1)

1. Extend `src/red_team/claim_extractor.rs` with `extract_from_spec()` method
2. Add RFC-2119 keyword detection (MUST/SHALL/SHOULD/MAY)
3. Add path reference extraction regex
4. Add numeric claim extraction
5. Add markdown structural parsing (headers as scope, bullets as claims)
6. Tests: 15+ unit tests for each extraction pattern

### Phase 2: Core Falsification Engine (Week 2)

1. Create `src/services/falsification/` module
2. Implement `FalsificationEngine` with strategy dispatch
3. Implement `CodeBehaviorStrategy` — uses `pmat query` internally
4. Implement `PathReferenceStrategy` — filesystem + AST validation
5. Implement `MetricClaimStrategy` — delegates to existing metric collectors
6. Implement `AbsenceClaimStrategy` — uses `pmat query --literal`
7. Tests: Integration tests with real spec files

### Phase 3: RAG Integration (Week 3)

1. Build spec index (third index alongside code and git indexes)
2. Implement disconfirming query generation (negation-augmented)
3. Implement evidence polarity classification
4. Implement RRF fusion across code + git + spec indexes
5. Implement churn-weighted staleness scoring
6. Tests: Falsification against known-stale specs

### Phase 4: Cross-Spec Contradiction (Week 4)

1. Index all spec claims into spec-specific RAG index
2. For each constraint claim, query for contradicting constraints
3. Implement semantic similarity threshold for "same concept" detection
4. Produce contradiction pairs with evidence
5. Tests: Plant known contradictions across test specs

### Phase 5: CLI + MCP + CI Integration (Week 5)

1. Add `pmat falsify` CLI subcommand (clap)
2. Add `pmat_falsify` MCP tool
3. Add `--format json/junit` output formatters
4. Integrate into `pmat work complete` as new `FalsificationMethod::SpecFalsification`
5. Add pre-commit hook option
6. Tests: End-to-end CLI tests

---

## 10. Integration with Existing Systems

### 10.1 Reuse Matrix

| Existing Component | Reused For | Location |
|-------------------|------------|----------|
| `ClaimExtractor` | Claim extraction foundation | `src/red_team/claim_extractor.rs` |
| `DocAccuracyValidator` | Path reference + entity validation | `src/services/hallucination_detector.rs` |
| `CodeFactDatabase` | Ground truth from codebase | `src/services/hallucination_detector.rs` |
| `TursoVectorDB` (trueno-rag backed) | Semantic search for evidence | `src/services/semantic/turso_vector_db.rs` |
| `Bm25SearchEngine` | Keyword search for evidence | `src/services/semantic/hybrid_search.rs` |
| `ChurnIndex` (--churn) | Staleness scoring | `src/services/semantic/` |
| `GitHistoryIndex` (-G) | Temporal claim falsification | `src/services/semantic/` |
| `WorkContract` + `FalsificationMethod` | Integration point for pmat work | `src/cli/handlers/work_contract.rs` |
| `run_falsification_tests` | Execution model for claims | `src/cli/handlers/work_falsification.rs` |

### 10.2 New Components

| Component | Purpose | Depends On |
|-----------|---------|------------|
| `SpecClaimExtractor` | Extract claims from specs (extends ClaimExtractor) | `red_team/claim_extractor.rs` |
| `FalsificationEngine` | Orchestrate falsification strategies | trueno-rag, pmat query |
| `SpecIndex` | RAG index of spec claims for cross-ref | trueno-rag |
| `EvidencePolarityClassifier` | Determine if evidence supports/contradicts | Heuristic rules |
| `DisconfirmingQueryGenerator` | Generate negation-augmented queries | Linguistic rules |
| `StalenessScorer` | Churn-weighted staleness calculation | ChurnIndex |
| `FalsificationReporter` | Text/JSON/JUnit output | Existing reporter patterns |

---

## 11. Success Criteria

### 11.1 Functional

| # | Criterion | Measurement |
|---|-----------|-------------|
| F1 | Extracts ≥80% of falsifiable claims from a spec | Manual audit of 5 specs |
| F2 | Falsifies known-stale specs with ≥90% recall | Plant 10 stale claims, detect ≥9 |
| F3 | Detects planted cross-spec contradictions at ≥95% | Plant 20 contradictions, detect ≥19 |
| F4 | Zero false positives on a known-good spec | Run against verified-current spec |
| F5 | Path reference claims checked with 100% accuracy | All path claims verified against fs |
| F6 | Metric claims measured, not just searched | Coverage/complexity claims actually run tools |

### 11.2 Performance

| # | Criterion | Target |
|---|-----------|--------|
| P1 | Single spec falsification | <30 seconds |
| P2 | All specs falsification (100+ specs) | <5 minutes |
| P3 | Claim extraction (no falsification) | <1 second per spec |
| P4 | Cross-spec contradiction check | <2 minutes for all specs |
| P5 | Spec index build/update | <10 seconds incremental |

### 11.3 Quality

| # | Criterion | Target |
|---|-----------|--------|
| Q1 | Test coverage for falsification engine | ≥95% |
| Q2 | TDG grade for new modules | ≥B average |
| Q3 | No new `unsafe` blocks | Zero |
| Q4 | Falsification of the falsifier (meta-test) | `pmat falsify` run against this spec passes |

---

## 12. Falsification of This Spec (Meta-Test)

Per Popperian methodology, this spec itself must be falsifiable. Falsifiable claims in this document:

| # | Claim | How to Falsify |
|---|-------|----------------|
| M1 | "Extends `git-history-rag-integration.md`" | Check file exists |
| M2 | "Reuses `src/red_team/claim_extractor.rs`" | Check file exists and has `Claim` struct |
| M3 | "existing `FalsificationMethod` enum" | Check enum exists in `work_contract.rs` |
| M4 | "`trueno-rag` provides BM25 + vector search" | Check `Bm25SearchEngine` and `TursoVectorDB` exist |
| M5 | "Single spec falsification < 30 seconds" | Benchmark after implementation |
| M6 | "Claim extraction < 1 second per spec" | Benchmark after implementation |
| M7 | "`DocAccuracyValidator` exists" | Check `hallucination_detector.rs` |

All meta-claims M1-M4 and M7 are verified against the current codebase as of spec creation date. M5-M6 are future-testable performance claims.

---

## 13. Mandatory Falsification Ledger (Andon Cord for `pmat work`)

### 13.1 Problem

Currently `pmat work complete` can close work items through two paths:

1. **Contract path**: `WorkContract::exists()` → `run_contract_tests()` → but failures can be overridden with `--override-claims` + `--ticket`
2. **Legacy path**: `run_legacy_falsification()` → **warnings only, never blocks**

Neither path requires RAG-powered spec falsification, and neither persists an immutable audit trail. A work item can be closed without any record of what was falsified, what survived, and what was overridden.

### 13.2 Design: Falsification Ledger

Every falsification run — whether from `pmat falsify`, `pmat work complete`, or CI — produces a **FalsificationReceipt** that is persisted to the ledger. Work items CANNOT be closed without a receipt.

#### Storage

```
.pmat-work/
├── <item-id>/
│   ├── contract.json          # Existing: work contract
│   ├── falsification/         # NEW: falsification audit trail
│   │   ├── receipt-2026-02-14T10:30:00Z.json   # Immutable receipt
│   │   ├── receipt-2026-02-14T11:45:00Z.json   # Re-run after fixes
│   │   └── latest -> receipt-2026-02-14T11:45:00Z.json  # Symlink
│   └── ...
└── ledger.jsonl               # NEW: append-only global ledger
```

**Dual storage**:
- Per-item `falsification/` directory: detailed receipts with full evidence
- Global `ledger.jsonl`: append-only log of all falsification events across all work items (for querying, reporting, trend analysis)

#### FalsificationReceipt

```rust
/// Immutable record of a falsification run
/// Once written, MUST NOT be modified (append-only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalsificationReceipt {
    /// Unique receipt ID (UUID v7 — time-ordered)
    pub receipt_id: String,

    /// Work item this falsification is for
    pub work_item_id: String,

    /// Git SHA at time of falsification (tamper-evident)
    pub git_sha: String,

    /// Timestamp (RFC 3339)
    pub timestamp: String,

    /// Who/what triggered the falsification
    pub trigger: FalsificationTrigger,

    /// What was falsified (spec files, contract claims, etc.)
    pub targets: Vec<FalsificationTargetSummary>,

    /// Aggregate results
    pub summary: ReceiptSummary,

    /// Per-claim verdicts (full evidence)
    pub verdicts: Vec<FalsificationVerdict>,

    /// Claims that were overridden (with ticket reference)
    pub overrides: Vec<ClaimOverride>,

    /// SHA-256 hash of this receipt's content (excluding this field)
    /// Enables tamper detection in the ledger
    pub content_hash: String,
}

pub enum FalsificationTrigger {
    /// pmat work complete
    WorkComplete { item_id: String },
    /// pmat falsify <path>
    ManualCli { path: String },
    /// CI/CD pipeline
    CiPipeline { pipeline_id: String, job_id: String },
    /// MCP tool call
    McpTool { session_id: String },
    /// Pre-commit hook
    PreCommit,
}

pub struct ReceiptSummary {
    pub total_claims: usize,
    pub survived: usize,
    pub falsified: usize,
    pub stale: usize,
    pub overridden: usize,
    pub unfalsifiable: usize,
    pub health_score: f64,

    /// Whether this receipt allows work completion
    /// true iff: falsified == 0 || all falsified claims are in overrides
    pub allows_completion: bool,
}

pub struct ClaimOverride {
    /// Which claim was overridden
    pub claim_id: String,

    /// Ticket reference (mandatory for overrides)
    pub ticket: String,

    /// Who approved the override
    pub approved_by: Option<String>,

    /// Why the override is justified
    pub reason: String,
}
```

#### Global Ledger Entry (JSONL)

```rust
/// Compact entry for the append-only global ledger
/// One line per falsification run
#[derive(Debug, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub receipt_id: String,
    pub timestamp: String,
    pub work_item_id: String,
    pub git_sha: String,
    pub trigger: FalsificationTrigger,
    pub total_claims: usize,
    pub survived: usize,
    pub falsified: usize,
    pub overridden: usize,
    pub health_score: f64,
    pub allows_completion: bool,
    pub content_hash: String,
}
```

### 13.3 Gate: `pmat work complete` REQUIRES Receipt

Modify `handle_work_complete` in `src/cli/handlers/work_handlers/core_handlers.rs`:

```rust
pub async fn handle_work_complete(
    id: String,
    skip_quality: bool,
    override_claims: Option<Vec<String>>,
    ticket: Option<String>,
    path: Option<PathBuf>,
) -> Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    // ... existing setup ...

    // NEW: Check for falsification receipt BEFORE allowing completion
    let receipt = get_or_create_falsification_receipt(
        &project_path,
        &item,
        &override_claims,
        &ticket,
    ).await?;

    if !receipt.summary.allows_completion {
        print_blocked_by_falsification(&receipt);
        anyhow::bail!(
            "Work blocked: falsification ledger shows {} unresolved falsified claims. \
             Fix the claims or use --override-claims with --ticket to acknowledge.",
            receipt.summary.falsified - receipt.summary.overridden
        );
    }

    // Persist receipt to ledger (append-only)
    persist_receipt(&project_path, &receipt)?;
    append_to_ledger(&project_path, &receipt)?;

    // ... existing completion logic ...
}
```

#### Receipt Resolution Flow

```
pmat work complete PMAT-500
         │
         ▼
    ┌────────────────────────┐
    │ Check for valid receipt │
    │ in .pmat-work/PMAT-500/│
    │ falsification/latest   │
    └────────────┬───────────┘
                 │
         ┌───────┴───────┐
         │               │
    No receipt      Receipt exists
         │               │
         ▼               ▼
    ┌──────────┐   ┌──────────────────┐
    │ Run full │   │ Check freshness: │
    │ pmat     │   │ receipt.git_sha  │
    │ falsify  │   │ == HEAD?         │
    └────┬─────┘   └────────┬─────────┘
         │            ┌─────┴──────┐
         │          Fresh        Stale
         │            │            │
         │            ▼            ▼
         │      ┌──────────┐  ┌──────────┐
         │      │ Use      │  │ Re-run   │
         │      │ existing │  │ pmat     │
         │      │ receipt  │  │ falsify  │
         │      └────┬─────┘  └────┬─────┘
         │           │             │
         └───────────┼─────────────┘
                     │
                     ▼
            ┌─────────────────┐
            │ allows_completion│
            │ == true?         │
            └────────┬────────┘
               ┌─────┴─────┐
             Yes           No
               │             │
               ▼             ▼
          ┌─────────┐  ┌──────────────────┐
          │ Persist │  │ BLOCK completion  │
          │ receipt │  │ Show falsified    │
          │ + close │  │ claims + evidence │
          │ item    │  │ Require --override│
          └─────────┘  └──────────────────┘
```

### 13.4 Receipt Freshness

A receipt is **fresh** if:
1. `receipt.git_sha == git rev-parse HEAD` (no new commits since falsification)
2. `receipt.timestamp` is within 24 hours (configurable via `.pmat-metrics.toml`)

If stale, `pmat work complete` automatically re-runs falsification. This prevents the pattern of "run falsification, make changes, close without re-falsifying."

### 13.5 Legacy Mode Deprecation

The current legacy path (`run_legacy_falsification`) becomes a **hard error** instead of a warning:

```rust
// BEFORE (current):
async fn run_contract_falsification(...) -> Result<()> {
    if !WorkContract::exists(project_path, item_id) {
        println!("ℹ️  No work contract found (legacy mode)");
        return run_legacy_falsification(...).await; // warnings only
    }
    // ...
}

// AFTER:
async fn run_contract_falsification(...) -> Result<()> {
    if !WorkContract::exists(project_path, item_id) {
        anyhow::bail!(
            "No work contract found for {}. Run 'pmat work start {}' first.\n\
             Work contracts with falsification receipts are MANDATORY.\n\
             Legacy mode has been deprecated.",
            id, id
        );
    }
    // ...
}
```

**Migration**: Existing work items without contracts get a one-time auto-generated contract on next `pmat work complete`, with a warning that future items must use `pmat work start`.

### 13.6 Override Accountability Chain

When claims are overridden, the ledger creates an **accountability chain**:

```
Override requires:
1. --override-claims "claim-003,claim-007"    (which claims)
2. --ticket "GH-1234"                         (why it's acceptable)
3. Receipt records: who, when, what, why       (audit trail)
```

The override is recorded in the receipt with the ticket reference. This means:
- Every overridden falsification is traceable to a ticket
- `pmat ledger audit` can report all overrides across the project's history
- Trend analysis: "are we overriding more claims over time?" (quality signal)

### 13.7 Ledger Query Commands

```bash
# View falsification history for a work item
pmat ledger show PMAT-500

# View all overrides across all work items
pmat ledger overrides

# View trend: health scores over time
pmat ledger trend --last 30

# Audit: find work items completed without fresh receipts
pmat ledger audit --check-freshness

# Export ledger for external analysis
pmat ledger export --format csv -o falsification-audit.csv

# Verify ledger integrity (check content hashes)
pmat ledger verify
```

### 13.8 Ledger Types

```rust
/// Ledger service for querying falsification history
pub struct FalsificationLedger {
    /// Path to .pmat-work/ledger.jsonl
    ledger_path: PathBuf,

    /// Path to .pmat-work/ root
    work_root: PathBuf,
}

impl FalsificationLedger {
    /// Append a receipt to the global ledger (append-only, never modify)
    pub fn append(&self, receipt: &FalsificationReceipt) -> Result<()>;

    /// Load all entries (streaming — doesn't load full receipts)
    pub fn entries(&self) -> Result<impl Iterator<Item = LedgerEntry>>;

    /// Load full receipt for a specific run
    pub fn get_receipt(&self, receipt_id: &str) -> Result<FalsificationReceipt>;

    /// Get latest receipt for a work item
    pub fn latest_receipt(&self, work_item_id: &str) -> Result<Option<FalsificationReceipt>>;

    /// Check if a valid (fresh) receipt exists for a work item
    pub fn has_fresh_receipt(
        &self,
        work_item_id: &str,
        current_sha: &str,
        max_age: Duration,
    ) -> Result<bool>;

    /// Get all overrides across all work items
    pub fn all_overrides(&self) -> Result<Vec<(LedgerEntry, Vec<ClaimOverride>)>>;

    /// Compute health score trend over time
    pub fn health_trend(&self, days: usize) -> Result<Vec<(NaiveDate, f64)>>;

    /// Verify integrity of all receipts (hash check)
    pub fn verify_integrity(&self) -> Result<IntegrityReport>;
}

pub struct IntegrityReport {
    pub total_receipts: usize,
    pub valid: usize,
    pub tampered: usize,
    pub missing: usize,
    pub tampered_ids: Vec<String>,
}
```

---

## 14. Interaction with Existing `pmat work` Lifecycle

### 14.1 Updated Lifecycle

```
pmat work start PMAT-500
  │  Creates WorkContract in .pmat-work/PMAT-500/contract.json
  │  Captures baseline metrics
  │
  ▼  (developer works on the item)
  │
pmat falsify docs/specifications/relevant-spec.md
  │  Produces FalsificationReceipt
  │  Persists to .pmat-work/PMAT-500/falsification/
  │  Appends to .pmat-work/ledger.jsonl
  │  (can be run multiple times — each run creates new receipt)
  │
  ▼
pmat work complete PMAT-500
  │  1. Check for fresh falsification receipt
  │  2. If no receipt or stale → auto-run pmat falsify
  │  3. If receipt.allows_completion → proceed
  │  4. If blocked → show evidence, require --override-claims + --ticket
  │  5. Persist final receipt to ledger
  │  6. Mark item complete in roadmap
  │
  ▼
.pmat-work/ledger.jsonl  ← immutable audit trail of all falsification events
```

### 14.2 Auto-Detection of Relevant Specs

When `pmat work complete` runs and no explicit spec is provided, it auto-detects relevant specs:

```rust
/// Find specs that are likely related to this work item
fn find_relevant_specs(
    project_path: &Path,
    work_item: &RoadmapItem,
) -> Result<Vec<PathBuf>> {
    let specs_dir = project_path.join("docs/specifications");
    let mut relevant = Vec::new();

    // Strategy 1: Specs modified since work started
    let contract = WorkContract::load(project_path, &work_item.id)?;
    let modified_since = git_files_modified_since(&contract.baseline_commit)?;
    for file in &modified_since {
        if file.starts_with("docs/specifications/") {
            relevant.push(project_path.join(file));
        }
    }

    // Strategy 2: Specs referenced in commit messages since baseline
    let commits = git_commits_since(&contract.baseline_commit)?;
    for commit in &commits {
        for spec in extract_spec_references(&commit.message) {
            let path = specs_dir.join(spec);
            if path.exists() && !relevant.contains(&path) {
                relevant.push(path);
            }
        }
    }

    // Strategy 3: RAG search for specs related to work item title
    if relevant.is_empty() {
        // Fallback: semantic search across spec index
        let results = spec_index.search(&work_item.title, 5)?;
        for result in results {
            relevant.push(result.path);
        }
    }

    Ok(relevant)
}
```

### 14.3 Configuration

```toml
# .pmat-metrics.toml additions

[falsification]
# Require falsification receipt for work completion (default: true)
require_receipt = true

# Maximum receipt age before re-falsification required
receipt_max_age_hours = 24

# Minimum health score to allow completion (default: 0.7)
min_health_score = 0.7

# Strategies to run by default
default_strategies = ["code-behavior", "path-reference", "metric-claim", "staleness"]

# Whether to auto-detect relevant specs on work complete
auto_detect_specs = true

# Maximum number of specs to falsify per work item (performance guard)
max_specs_per_item = 10

# Enable cross-spec contradiction checking on work complete
cross_spec_on_complete = false
```

---

## 15. Updated Implementation Plan

Amend Phase 5 and add Phase 6:

### Phase 5: CLI + MCP + CI Integration (Week 5) — AMENDED

1. Add `pmat falsify` CLI subcommand (clap)
2. Add `pmat_falsify` MCP tool
3. Add `--format json/junit` output formatters
4. **Add `FalsificationReceipt` and `FalsificationLedger` types**
5. **Add receipt persistence (per-item + global ledger.jsonl)**
6. **Integrate receipt check into `handle_work_complete`**
7. **Deprecate legacy falsification path**
8. Tests: End-to-end CLI tests

### Phase 6: Ledger Commands + Accountability (Week 6) — NEW

1. Add `pmat ledger show` subcommand
2. Add `pmat ledger overrides` subcommand
3. Add `pmat ledger trend` subcommand
4. Add `pmat ledger audit` subcommand
5. Add `pmat ledger verify` (integrity check)
6. Add `pmat ledger export` (CSV/JSON)
7. Add auto-detection of relevant specs in `pmat work complete`
8. Tests: Ledger integrity tests, freshness tests, override accountability tests

---

## 16. Updated Success Criteria

### 16.1 Ledger-Specific Criteria

| # | Criterion | Measurement |
|---|-----------|-------------|
| L1 | `pmat work complete` BLOCKS without a fresh receipt | Integration test: complete without receipt → error |
| L2 | Receipt freshness check detects stale receipts | Create receipt, make commit, check → stale |
| L3 | Overrides require --ticket (no anonymous overrides) | Attempt override without ticket → error |
| L4 | Ledger is append-only (no modification of past entries) | Verify no write path modifies existing entries |
| L5 | Content hash detects tampering | Modify receipt, run verify → tampered |
| L6 | Legacy mode produces hard error, not warning | Integration test: complete without contract → error |
| L7 | Auto-detection finds relevant specs | Start work, modify spec, complete → spec detected |
| L8 | Ledger trend shows health scores over time | Create 5 receipts, query trend → 5 data points |
