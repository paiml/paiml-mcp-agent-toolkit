# Infrastructure

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 18

## Pre-Commit Hooks

### Quality Gate Enforcement

```bash
pmat hooks install --tdg-enforcement
```

Hooks validate:
- Complexity thresholds (cyclomatic <=30, cognitive <=25)
- TDG grade gate (configurable minimum)
- O(1) metric freshness (<7 days staleness)
- Documentation link integrity

### Enhanced Runner Ecosystem

- Parallel hook execution for performance
- Cascading failure detection
- Emergency bypass: `git commit --no-verify`

## bashrs Linting

### Shell/Makefile Quality

```bash
bashrs lint Makefile
bashrs lint Makefile
```

Checks:
- SC2086/SC2046/SC2116: Shell quoting issues
- DET003: Non-determinism detection
- IDEM002: Idempotency violations
- SEC008: Security vulnerabilities

## Project Scaffolding

```bash
pmat scaffold rust-lib --name my-lib
pmat scaffold rust-cli --name my-cli
```

Templates include:
- Cargo.toml with batuta stack dependencies
- Pre-configured `.pmat.yaml`
- CI pipeline templates
- TDG enforcement hooks

## Session Recording

### .pmat Format

Capture and replay analysis sessions:
```bash
pmat record start
# ... run analyses ...
pmat record stop --output session.pmat
pmat record replay session.pmat
```

## Oracle RAG System

```bash
batuta oracle --rag "your question here"
```

Knowledge system powered by RAG:
- Indexes all stack documentation
- Semantic search across batuta ecosystem
- Auto-updates via post-commit hooks

## Red Team Mode

Adversarial testing for quality assurance:
```bash
pmat red-team --target src/services/
```

- Generates adversarial inputs
- Tests error handling paths
- Validates panic-freedom

## Prompt System

PMAT prompt specification for model input control:
- Context window management
- Token budget optimization
- Prompt templating for analysis tasks

## Kaizen Improvements

### File Caching (Round 4)

Sub-100ms file operations:
- LRU cache for frequently accessed files
- Memory-mapped I/O for large files
- Parallel directory scanning

### Tracing & Bug Discovery

Integration of tracing with TDG and git expansion:
- Distributed tracing across analysis phases
- Bug pattern correlation with git history
- Automated root cause suggestion

## Key Files

| File | Purpose |
|------|---------|
| `.pmat.yaml` | Project configuration |
| `scripts/` | Installation and utility scripts |
| `Makefile` | Build, test, deploy targets |
| `.github/workflows/` | CI pipeline configuration |

## References

- Consolidated from: pre-commit-hooks-spec, enhance-runner-ecosystem,
  Makefile-lint-enhance, scaffold-maintain-spec, contract-refactoring-plan,
  unified-contract-by-design, pmat-recording-format, pmat-oracle-specification,
  red-team-mode-spec, prompt-spec, compute-brick-support,
  kaizen-round-4-file-caching-plan, tracing-bug-discovery-tdg-git-expansion-spec,
  universal-reporting-data-science-ascii-viz, qdd-tool-specification,
  learning-system-ideas, refactoring-specification, unified-quality-driven-development-tool
