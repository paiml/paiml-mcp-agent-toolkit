## QA Checklist: Clap CLI Functionality Mapping

### 1. Command Structure Validation

#### 1.1 Root Command Parser
```rust
// Verify Cli struct derives and attributes
- [ ] #[derive(Parser)] propagates to all subcommands
- [ ] #[command(version, about)] renders in --help
- [ ] #[command(propagate_version = true)] applies recursively
- [ ] Binary name detection: env!("CARGO_PKG_NAME")
- [ ] Global args accessible from all subcommands
```

#### 1.2 Subcommand Hierarchy
```rust
// Expected structure from cli/mod.rs:
Commands
├── Generate (TemplateCommands)
├── Analyze (AnalyzeCommands)
│   ├── Churn
│   ├── Complexity
│   ├── Dag
│   ├── DeadCode
│   └── DeepContext
├── Demo
└── Template (TemplateCommands)
    ├── Generate
    ├── List
    ├── Search
    └── Validate
```

### 2. Argument Parsing Correctness

#### 2.1 Type Coercion & Validation
```rust
// Test matrix for parse_from_str implementations:
- [ ] PathBuf: Tilde expansion (~/ → /home/user/)
- [ ] bool: --flag vs --no-flag precedence
- [ ] Vec<String>: Multiple values via repeated flags
- [ ] HashMap: Key=value parsing via parse_key_val()
- [ ] Enum variants: Case-insensitive matching
- [ ] Duration: "30s", "5m", "1h" parsing
```

#### 2.2 Custom Validators
```rust
// From cli/args.rs validation pipeline:
- [ ] validate_params(): Required params present
- [ ] validate_type(): String/bool/number enforcement
- [ ] File existence: PathBuf args must exist
- [ ] URL format: Regex validation for --repo-url
- [ ] Numeric ranges: --limit 1..=1000
- [ ] Mutual exclusivity: --json excludes --format
```

### 3. Environment Variable Integration

#### 3.1 Expansion Behavior
```rust
// expand_env_vars() test cases:
- [ ] $VAR → value
- [ ] ${VAR} → value  
- [ ] ${VAR:-default} → default if unset
- [ ] ${VAR:=default} → set and return default
- [ ] Escaped \$VAR → literal $VAR
- [ ] Nested ${FOO_${BAR}} → error
```

#### 3.2 Precedence Rules
```
Priority order (highest to lowest):
1. Explicit CLI argument
2. Environment variable  
3. Config file value
4. Default from #[arg(default_value)]
```

### 4. Help Text Generation

#### 4.1 Command Documentation
```rust
// Verify help strings from #[command(about)]:
- [ ] Short help (-h): Single line descriptions
- [ ] Long help (--help): Extended with examples
- [ ] Subcommand help: pmat analyze --help
- [ ] Context-aware help: Shows valid subcommands only
- [ ] Error suggestions: "Did you mean 'generate'?"
```

#### 4.2 Argument Documentation
```rust
// Per-argument help validation:
- [ ] Required markers: <REQUIRED> vs [OPTIONAL]
- [ ] Type hints: --limit <NUMBER>
- [ ] Default values shown: [default: 10]
- [ ] Env var hints: [env: PMAT_CACHE_DIR]
- [ ] Possible values: --format <json|yaml|toml>
```

### 5. Error Handling & Recovery

#### 5.1 Parse Error Messages
```rust
// Expected error quality:
- [ ] Missing required: "required argument '--template' not provided"
- [ ] Invalid type: "invalid value '3.14' for '--limit': not a valid integer"
- [ ] Unknown arg: "unexpected argument '--colour' found, did you mean '--color'?"
- [ ] Subcommand typo: "unrecognized subcommand 'genrate', did you mean 'generate'?"
- [ ] Value validation: "3000 is not in range 1..=1000"
```

#### 5.2 Error Context Preservation
```rust
// Clap error wrapping chain:
ParseError
  → CliError::Validation
    → AppError::InvalidInput
      → JSON-RPC error code -32602
```

### 6. Complex Argument Scenarios

#### 6.1 Positional vs Named
```rust
// Test precedence and mixing:
- [ ] pmat generate makefile rust → positional parsing
- [ ] pmat generate --template=makefile --toolchain=rust → named
- [ ] Mixed: pmat generate makefile --toolchain=rust
- [ ] Trailing args: pmat analyze -- --internal-flag
- [ ] Varargs: pmat demo --args key1=val1 key2=val2
```

#### 6.2 Conditional Requirements
```rust
// requires_if/conflicts_with logic:
- [ ] --output requires --format
- [ ] --parallel conflicts with --sequential  
- [ ] --repo-url requires_if analyze subcommand
- [ ] group!(output) exactly one of --stdout/--file
- [ ] --all-features implies --no-default-features=false
```

### 7. Subcommand-Specific Features

#### 7.1 Analyze Subcommands
```rust
// AnalyzeCommands enum mapping:
analyze churn:
  - [ ] --limit validates > 0
  - [ ] --format enum restricts to json/csv/markdown
  - [ ] --output PathBuf creates parent dirs

analyze complexity:
  - [ ] --threshold f32 in range 0.0..100.0
  - [ ] --sort-by validates field names
  - [ ] Multiple --exclude patterns accumulate

analyze deep-context:
  - [ ] 10 boolean flags toggle features
  - [ ] --repo-url XOR local path detection
  - [ ] --cache-strategy affects performance
```

#### 7.2 Template Commands
```rust
// Template URI parsing:
- [ ] template://category/toolchain/name
- [ ] s3://bucket/path validation  
- [ ] file:///absolute/path checking
- [ ] Relative path resolution
- [ ] Invalid URI scheme rejection
```

### 8. Demo Mode Arguments

#### 8.1 Protocol Selection
```rust
// --protocol flag behavior:
- [ ] Default: "all" runs all protocols
- [ ] Single: --protocol=http  
- [ ] Multiple: --protocol=http,mcp
- [ ] Invalid: --protocol=ftp → error
- [ ] Case normalization: HTTP → http
```

#### 8.2 Display Configuration
```rust
// Complex nested config:
- [ ] --panel-width constraints: 80..=300
- [ ] --update-interval Duration parsing
- [ ] --config-file YAML deserialization
- [ ] Config merge precedence
- [ ] Partial config updates
```

### 9. Performance & Memory Boundaries

#### 9.1 Argument Count Limits
```rust
// Stress test boundaries:
- [ ] 1000 --exclude patterns → Vec allocation
- [ ] 100KB single argument → stack overflow check
- [ ] 10MB total argv → parser performance
- [ ] Recursive expansion depth limit: 10
- [ ] Pathological regex in validators
```

#### 9.2 Parser Initialization Cost
```rust
// Benchmark targets:
Cold start: < 5ms
Subcommand dispatch: < 1ms  
Help generation: < 10ms
Error formatting: < 2ms
Shell completion: < 50ms
```

### 10. Integration Testing Matrix

#### 10.1 Shell Compatibility
```bash
# POSIX compliance:
- [ ] Quoted args: pmat generate "makefile with spaces"
- [ ] Escaped quotes: pmat --name="John \"Doc\" Smith"
- [ ] Glob expansion: pmat analyze src/**/*.rs
- [ ] Tilde in quotes: pmat --path "~/project"
- [ ] UTF-8 args: pmat --author="José García"
```

#### 10.2 Platform-Specific Behavior
```rust
// OS-dependent parsing:
#[cfg(windows)]
- [ ] Path separator: --path C:\Users\project
- [ ] Case insensitive: --FORMAT vs --format
- [ ] UNC paths: \\server\share\file

#[cfg(unix)]  
- [ ] Signal handling during parse
- [ ] TTY detection for color output
- [ ] Permission bits on --output paths
```

### 11. Advanced Clap Features

#### 11.1 Derive API Completeness
```rust
// Verify all derive helpers:
- [ ] #[arg(short, long)] generates both -v/--verbose
- [ ] #[arg(alias = "rm")] for command aliases
- [ ] #[arg(hide = true)] for internal flags
- [ ] #[arg(exclusive = true)] prevents other args
- [ ] #[command(arg_required_else_help)] behavior
```

#### 11.2 Builder Pattern Fallback
```rust
// For dynamic CLI construction:
- [ ] Command::new() interop with derive
- [ ] Runtime subcommand injection
- [ ] Conditional argument addition
- [ ] Plugin system argument discovery
- [ ] Merge multiple command definitions
```

### 12. Regression Test Suite

#### 12.1 Version Migration
```rust
// Clap v3 → v4 compatibility:
- [ ] AppSettings → command attributes
- [ ] Arg::with_name → Arg::new
- [ ] SubCommand → Command migration
- [ ] Validator fn signature changes
- [ ] Error type hierarchies
```

#### 12.2 Known Edge Cases
```rust
// Historical bugs to prevent:
- [ ] Double-dash after subcommand
- [ ] Unicode normalization in args
- [ ] Negative numbers vs flags (-1 vs -v)
- [ ] Empty string arguments ""
- [ ] Null bytes in arguments
- [ ] Symlink resolution for argv[0]
```

### Test Execution Protocol
```rust
#[cfg(test)]
mod clap_integration_tests {
    use assert_cmd::Command;
    use predicates::prelude::*;
    
    #[test]
    fn test_full_command_matrix() {
        let permutations = generate_arg_permutations();
        for args in permutations {
            Command::cargo_bin("pmat")
                .args(&args)
                .assert()
                .code(predicate::in_iter(0..=2));
        }
    }
}
```

### Validation Metrics
```
Coverage targets:
- Argument combinations: >95%
- Error paths: 100%
- Help text accuracy: 100%
- Platform variants: 3 OS minimum
- Clap feature usage: >80%
```