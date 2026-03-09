# Work Management

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 14

## pmat work System

### Contract-Based Quality Enforcement

Each work item has a falsifiable contract:
```json
{
  "ticket_id": "PMAT-123",
  "claims": [
    {"claim": "Coverage >= 95%", "falsifiable": true, "evidence": "cargo llvm-cov"},
    {"claim": "No F-grade functions", "falsifiable": true, "evidence": "pmat comply"}
  ],
  "status": "in_progress"
}
```

### Commands

```bash
pmat work create "PMAT-123" --title "Fix memory leak"
pmat work status "PMAT-123"
pmat work complete "PMAT-123"  # Validates contract claims
pmat work list --status in_progress
```

### Quality Gate Integration

On `pmat work complete`:
1. Run all contract claims as assertions
2. If any claim fails, block completion
3. Generate evidence report

## Ticket Tracking

### Issue Reference Parsing

Supports patterns: `#123`, `PMAT-123`, `GH-123`
- Must `trim_matches` for parens: `(PMAT-472)` -> `PMAT-472`

### Fuzzy ID Matching

Short IDs resolve to full tickets:
- `123` matches `PMAT-123`
- Case-insensitive matching

## Roadmap & Todo

### Quality Gate Spec

```bash
pmat roadmap list              # Show all roadmap items
pmat roadmap add "Feature X"   # Add item with auto-priority
pmat roadmap complete "X"      # Mark complete, validate quality
```

## Storage

```
.pmat-work/
├── PMAT-123/
│   ├── contract.json    # Falsifiable claims
│   ├── evidence/        # Test results, coverage reports
│   └── notes.md         # Work notes
└── index.json           # Ticket index
```

## Key Files

| File | Purpose |
|------|---------|
| `src/cli/handlers/work_handler.rs` | Work command handler |
| `src/models/work.rs` | Work contract types |

## References

- Consolidated from: enhance-pmat-work, enhance-pmat-work-spec,
  improve-pmat-work, master-plan-pmat-work-system, roadmap-todo-quality-gate-spec
