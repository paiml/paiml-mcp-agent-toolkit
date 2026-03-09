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

## Key Files

| File | Purpose |
|------|---------|
| `src/cli/handlers/validate_readme_handler.rs` | README validation |
| `src/services/doc_validator.rs` | Link and reference checking |

## References

- Consolidated from: CLI_MCP_DOCUMENTATION_ENFORCEMENT, documentation-accuracy-enforcement,
  documentation-accuracy-enforcement-toyota-way-addendum, doc-validate
