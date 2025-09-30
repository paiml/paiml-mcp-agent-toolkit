# PMAT Claude Bridge

TypeScript bridge for Claude Agent SDK integration with PMAT.

## Architecture

This bridge implements the GREEN phase of EXTREME TDD methodology:
- Minimal implementation to pass tests
- No premature optimization
- Type-safe error handling
- Stdio IPC with atomic writes

## Structure

```
bridge/
├── src/
│   ├── index.ts       # Main bridge implementation
│   ├── types.ts       # Type definitions
│   └── transport.ts   # Stdio transport layer
├── dist/              # Compiled output
├── package.json       # Dependencies
└── tsconfig.json      # TypeScript configuration
```

## Building

```bash
cd bridge
npm install
npm run build
```

## Testing

```bash
npm test
```

## Performance Targets

- Initialization: <500ms
- P50 latency: <1ms
- P95 latency: <3ms
- Memory: <256MB

## Quality Gates

- Cyclomatic complexity: ≤15
- Test coverage: ≥95%
- Zero SATD policy enforced
- No TypeScript errors