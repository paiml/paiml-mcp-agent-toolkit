# Curl Installation Specification: TypeScript-First with Shell Fallback

## 1. Executive Summary

This specification defines a TypeScript-first installation system using Deno for primary execution, with a minimal POSIX shell wrapper for environments without Deno. The architecture provides the best of both worlds: type safety and modern tooling when available, with universal compatibility through shell fallback.

### Key Metrics
- **Primary installer**: ~200 LOC TypeScript (Deno)
- **Shell wrapper**: ~150 LOC POSIX sh (fallback)
- **Runtime dependencies**: None (Deno) or curl/tar (shell)
- **Type safety**: Full TypeScript validation
- **Cross-platform**: Linux, macOS, Windows support

## 2. Architecture

### 2.1 Dual-Mode Design

```
User runs: curl -fsSL .../install.sh | sh
                       │
                       ▼
              ┌─────────────────┐
              │  Shell Wrapper  │
              │  (install.sh)   │
              └────────┬────────┘
                       │
                  Deno exists?
                    /     \
                  Yes      No
                  /         \
         ┌──────▼──────┐  ┌──▼──────────┐
         │   Deno      │  │    Shell    │
         │ TypeScript  │  │  Fallback   │
         │ Installer   │  │  Installer  │
         └─────────────┘  └─────────────┘
```

### 2.2 Project Structure

```
paiml-mcp-agent-toolkit/
├── scripts/
│   ├── install.ts           # Primary TypeScript installer
│   ├── install.sh           # Shell wrapper with fallback
│   ├── create-release.ts    # Release creation script
│   └── test-curl-install.ts # Installation testing
└── docs/
    └── curl-install-spec.md # This document
```

## 3. Implementation

### 3.1 Shell Wrapper (install.sh)

The shell wrapper serves two purposes:
1. Detect and delegate to Deno if available
2. Provide fallback installation for environments without Deno

```bash
#!/bin/sh
# MCP Agent Toolkit Installation Script - Shell wrapper for Deno installer

set -euf

# Check if Deno is installed
if command -v deno >/dev/null 2>&1; then
    # Use the Deno installer
    exec deno run --allow-net --allow-read --allow-write --allow-env --allow-run \
        https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.ts "$@"
fi

# Fallback to shell implementation if Deno is not available
echo "Deno not found, using shell installer..."

# ... minimal POSIX shell implementation ...
```

### 3.2 TypeScript Installer (install.ts)

The primary installer leverages Deno's capabilities:

```typescript
#!/usr/bin/env -S deno run --allow-net --allow-read --allow-write --allow-env --allow-run

// Type-safe configuration
interface Config {
  repo: string;
  binaryName: string;
  installDir: string;
}

// Platform detection with proper types
function detectOS(): "linux" | "macos" | "windows" {
  switch (Deno.build.os) {
    case "linux": return "linux";
    case "darwin": return "macos";
    case "windows": return "windows";
    default: throw new Error(`Unsupported OS: ${Deno.build.os}`);
  }
}

// Async/await for clean error handling
async function install(version?: string): Promise<void> {
  const os = detectOS();
  const arch = detectArch();
  
  // Modern fetch API
  const response = await fetch(downloadUrl);
  if (!response.ok) {
    throw new Error(`Download failed: ${response.status}`);
  }
  
  // Built-in temp directory management
  const tmpDir = await Deno.makeTempDir();
  try {
    // ... installation logic ...
  } finally {
    await Deno.remove(tmpDir, { recursive: true });
  }
}
```

### 3.3 Benefits of TypeScript-First Approach

#### Type Safety
```typescript
// Compile-time guarantees
type Platform = "linux" | "macos" | "windows";
type Arch = "x86_64" | "aarch64";

// Exhaustive checking
function getPlatformBinary(platform: Platform): string {
  switch (platform) {
    case "linux": return "paiml-linux";
    case "macos": return "paiml-darwin";
    case "windows": return "paiml.exe";
    // TypeScript error if platform added without handler
  }
}
```

#### Modern Error Handling
```typescript
class InstallError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly details?: Record<string, unknown>
  ) {
    super(message);
    this.name = "InstallError";
  }
}

// Structured error reporting
throw new InstallError(
  "Checksum verification failed",
  "CHECKSUM_MISMATCH",
  { expected: expectedHash, actual: actualHash }
);
```

#### Cross-Platform Abstraction
```typescript
// Deno provides consistent APIs across platforms
await Deno.chmod(binaryPath, 0o755);  // Works on Windows too
await Deno.env.get("HOME");           // Cross-platform env access
await Deno.makeTempDir();              // Platform-appropriate temp dirs
```

## 4. Testing Infrastructure

### 4.1 Automated Testing (test-curl-install.ts)

```typescript
async function testInstallation(): Promise<void> {
  const tests = [
    testScriptFetch,
    testScriptValidation,
    testPlatformDetection,
    testGitHubAPIAccess,
    testDownloadURLConstruction,
  ];
  
  for (const test of tests) {
    const result = await test();
    console.log(`${result.passed ? "✅" : "❌"} ${result.name}`);
  }
}
```

### 4.2 Release Automation (create-release.ts)

```typescript
async function createRelease(): Promise<void> {
  // Type-safe version extraction
  const version = await getVersionFromCargoToml();
  
  // Platform-specific binary creation
  const { os, arch } = detectPlatform();
  const tarball = await createTarball(binaryName, os, arch);
  
  // GitHub release via gh CLI
  await runCommand("gh", [
    "release", "create", `v${version}`,
    "--title", `v${version}`,
    "--notes", releaseNotes,
    tarball,
  ]);
}
```

## 5. Security Properties

### 5.1 TypeScript Advantages

1. **No shell injection**: String interpolation is safe by default
2. **Validated inputs**: TypeScript enforces input validation
3. **Secure defaults**: Deno requires explicit permissions
4. **Memory safety**: No buffer overflows or memory corruption

### 5.2 Permission Model

```typescript
// Deno requires explicit permissions
// --allow-net: Download from GitHub
// --allow-read: Read existing installation
// --allow-write: Write binary and temp files
// --allow-env: Read HOME and PATH
// --allow-run: Execute tar and verification commands
```

## 6. Comparison with Previous Approach

| Aspect            | Rust-Generated Shell | TypeScript/Deno     | Advantage              |
|-------------------|---------------------|---------------------|------------------------|
| Development Speed | Complex macro system | Direct TypeScript   | TypeScript: 10x faster |
| Type Safety       | Compile-time only    | Runtime + compile   | Equal                  |
| Debugging         | Generated code       | Source maps         | TypeScript: Better DX  |
| Dependencies      | Rust toolchain       | Deno (optional)     | TypeScript: Simpler    |
| Bundle Size       | 2KB shell            | 5KB TS + 2KB shell  | Shell: Smaller         |
| Error Messages    | Basic shell errors   | Structured errors   | TypeScript: Clearer    |
| Cross-platform    | Manual handling      | Built-in APIs       | TypeScript: Cleaner    |

## 7. Migration Benefits

### 7.1 Developer Experience
- **Faster iteration**: Direct TypeScript editing vs. macro compilation
- **Better debugging**: Stack traces and source maps
- **Familiar tooling**: VS Code, debuggers, linters work out-of-box

### 7.2 Maintenance
- **Single language**: All scripts in TypeScript/Deno
- **No macro complexity**: Direct code is easier to understand
- **Standard patterns**: Async/await, try/catch, modules

### 7.3 User Experience
- **Fallback compatibility**: Shell wrapper ensures universal support
- **Better errors**: Structured error messages with context
- **Modern features**: Progress indicators, colored output, etc.

## 8. Implementation Checklist

- [x] Create TypeScript installer (install.ts)
- [x] Create shell wrapper with fallback (install.sh)
- [x] Create test script (test-curl-install.ts)
- [x] Create release script (create-release.ts)
- [x] Format all TypeScript files
- [x] Update documentation
- [ ] Test on multiple platforms
- [ ] Create first GitHub release
- [ ] Verify curl installation flow

## 9. Usage Examples

### Basic Installation
```bash
# Primary method (uses Deno if available)
curl -fsSL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh

# Force Deno installer
curl -fsSL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.ts | deno run --allow-all -

# Specific version
curl -fsSL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh -s v0.1.0
```

### Testing
```bash
# Run installation tests
./scripts/test-curl-install.ts

# Create a release
./scripts/create-release.ts
```

## 10. Conclusion

The TypeScript-first approach with shell fallback provides the optimal balance:
- **Modern development**: TypeScript, async/await, proper error handling
- **Universal compatibility**: Shell fallback for environments without Deno
- **Simplified maintenance**: One language for all scripts
- **Better user experience**: Structured errors, cross-platform support

This aligns with the "ONE way" principle by standardizing on Deno/TypeScript for all scripting while maintaining backward compatibility through a minimal shell wrapper.