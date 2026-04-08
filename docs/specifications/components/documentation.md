# Documentation

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 15

## Documentation Accuracy Enforcement

### Zero Hallucination Policy

All documentation (README.md, CLAUDE.md, GEMINI.md, AGENT.md) must be validated:

```bash
pmat context --output deep_context.md --format llm-optimized
pmat validate-readme \
    --targets README.md CLAUDE.md GEMINI.md AGENT.md \
    --deep-context deep_context.md \
    --fail-on-contradiction --verbose
```

### Validation Checks

| Check | Description |
|-------|-------------|
| Capability claims | Verified against AST (functions exist) |
| File path references | Verified against filesystem |
| Function references | Verified against index |
| External URLs | 404 detection via HTTP HEAD |
| Version numbers | Cross-referenced with Cargo.toml |

### Academic Foundations

- Semantic Entropy (Nature 2024): Detects hallucinated claims
- MIND framework (IJCAI 2025): Multi-dimensional inconsistency detection

## CLI/MCP Documentation Enforcement

Quality gate for documentation completeness:
- All CLI commands must have --help text
- All MCP tools must have descriptions
- All HTTP endpoints must have schema docs

## Link Validation

```bash
pmat doc-validate --check-links README.md
```

- Validates internal links (relative paths)
- Validates external links (HTTP HEAD requests)
- Reports broken anchors and 404s

## Toyota Way Addendum

Documentation follows Toyota Way principles:
- **Genchi Genbutsu**: Go see the actual code, don't guess
- **Jidoka**: Stop-the-line on documentation errors
- **Kaizen**: Continuous improvement of docs with each release

## Enforcement Points

| Hook | Check | Blocking |
|------|-------|----------|
| Pre-commit | Link validation | Warning |
| CI pipeline | Full accuracy validation | Error |
| Release | All docs validated | Blocking |

## PMAT Book: Falsified Provable Contracts

### Schema

Each book chapter has a provable contract YAML in `contracts/` following
the aprender pattern. The contract defines **what must be falsifiable**
about the chapter — no claim exists without a test.

```
contracts/
  pmat-book-build-v1.yaml       # Build + deploy contract (mdbook, CI)
  pmat-book-ch05-v1.yaml        # Ch 5: Analyze Suite (critical)
  pmat-book-ch07-v1.yaml        # Ch 7: Quality Gates (critical)
  pmat-book-ch13-v1.yaml        # Ch 14/file:ch13: Multi-Language (critical)
  pmat-book-ch14-v1.yaml        # Ch 35/file:ch35: Semantic Search (critical)
```

**IMPORTANT**: Chapter numbers in SUMMARY.md don't match filenames.
The `chapter_file` field is the authoritative path. `summary_chapter`
records the display number. Falsification caught this mismatch.

### Contract Structure (Per Chapter)

```yaml
metadata:
  contract: pmat-book-ch{NN}
  title: "Chapter Title"
  chapter_file: ch{NN}-00-topic.md
  critical: true/false

preconditions:
  - "pmat binary installed"
  - "pmat {command} --help exits 0"

postconditions:
  - "All CLI examples in chapter produce valid output"

falsification:
  - condition: "pmat {command} exits non-zero on test project"
    severity: P0
    action: reject_chapter
  - condition: "Chapter claims feature not in pmat --help"
    severity: P0
    action: reject_chapter
```

### Enforcement

| Gate | Check | Action |
|------|-------|--------|
| `make validate-book` | Critical chapters (5, 7, 13, 14) | Blocks release |
| Pre-push hook | `pmat comply check` includes CB-148 | Warning |
| CI | `mdbook build` + falsification tests | Blocks merge |

### Falsification Principles (Popperian)

1. **Every claim is testable**: If a chapter says "pmat query supports
   --contracts", there's a falsification test that runs the command.
2. **P0 = reject**: Any P0 falsification failure blocks the chapter.
3. **Examples are code**: CLI examples in chapters are extracted and
   run as tests. Stale output = P1 failure.
4. **No stubs**: Chapters with <50 words are rejected (P0).

## Key Files

| File | Purpose |
|------|---------|
| `src/cli/handlers/readme_validate_handlers/` | README validation (module) |
| `src/services/doc_validator.rs` | Link and reference checking |
| `contracts/pmat-book-build-v1.yaml` | Book build/deploy contract |
| `contracts/pmat-book-ch*.yaml` | Per-chapter falsification contracts |

## References

- Consolidated from: CLI_MCP_DOCUMENTATION_ENFORCEMENT, documentation-accuracy-enforcement,
  documentation-accuracy-enforcement-toyota-way-addendum, doc-validate
