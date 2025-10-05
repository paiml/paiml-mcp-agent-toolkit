# TICKET-PMAT-5031: Add `pmat scaffold wasm` CLI Command

**Status**: GREEN
**Priority**: P0
**Complexity**: 2
**Estimated Time**: 30 minutes
**Dependencies**: Existing scaffold module, TemplateType::Wasm, TICKET-PMAT-5030
**Sprint**: Sprint 19 - CLI Integration & Dogfooding

## Objective

Add `pmat scaffold wasm` subcommand to create WebAssembly projects. The scaffolding infrastructure exists (TemplateType::Wasm, WasmFramework), we just need to add the CLI variant and wire it up.

## Success Criteria

- [ ] `pmat scaffold wasm --name my-wasm --framework wasm-labs` creates a working WASM project
- [ ] Support both WasmLabs and PureWasm frameworks
- [ ] All CLI arguments properly passed to scaffold engine
- [ ] Quality gates enforced on scaffolded projects
- [ ] All quality gates pass (complexity <10, coverage >80%, no SATD)

## Current State

**Already Exists:**
- `TemplateType::Wasm` with `WasmFramework` enum (config.rs:19)
- `WasmFramework::WasmLabs` and `WasmFramework::PureWasm` (config.rs:32-35)
- Scaffolding engine in `server/src/scaffold/`
- Template registry with wasm templates

**Missing:**
- `ScaffoldCommands::Wasm` variant
- Wire-up in command_structure.rs
- Handler function (if needed)

## Test Strategy

### Unit Tests
- [ ] `test_scaffold_wasm_command_structure` - Verify command args
- [ ] `test_wasm_framework_parsing` - Parse framework names

### Integration Tests
- [ ] `integration_scaffold_wasm_labs` - Create WasmLabs project
- [ ] `integration_scaffold_pure_wasm` - Create PureWasm project
- [ ] `integration_scaffolded_wasm_builds` - Verify wasm-pack build succeeds

## Quality Gates

- [ ] Cyclomatic complexity <10 for all functions
- [ ] Cognitive complexity <15 for all functions
- [ ] Line coverage >80%
- [ ] Branch coverage >80%
- [ ] 0 SATD violations
- [ ] 0 clippy warnings
- [ ] All tests pass

## Implementation Plan

### Phase 1: Add ScaffoldCommands::Wasm Variant

```rust
// server/src/cli/commands.rs (after Agent variant)

pub enum ScaffoldCommands {
    // ... existing variants ...

    /// Scaffold a WebAssembly project
    Wasm {
        /// Project name
        #[arg(short, long)]
        name: String,

        /// WASM framework (wasm-labs, pure-wasm)
        #[arg(short, long, default_value = "wasm-labs")]
        framework: String,

        /// Features to include (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        features: Vec<String>,

        /// Quality level (standard, strict, extreme)
        #[arg(short = 'q', long, default_value = "strict")]
        quality: String,

        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Overwrite existing directory
        #[arg(long)]
        force: bool,

        /// Show what would be generated without creating files
        #[arg(long)]
        dry_run: bool,
    },

    // ... ListTemplates, ValidateTemplate ...
}
```

### Phase 2: Create Handler Function

```rust
// server/src/cli/handlers/generation_handlers.rs

/// Parameters for WASM scaffolding
pub struct ScaffoldWasmParams {
    pub name: String,
    pub framework: String,
    pub features: Vec<String>,
    pub quality: String,
    pub output: Option<PathBuf>,
    pub force: bool,
    pub dry_run: bool,
}

/// Handle WASM scaffolding command
///
/// # Complexity
/// - Time: O(n) where n is project size
/// - Cyclomatic: 4
pub async fn handle_scaffold_wasm(params: ScaffoldWasmParams) -> Result<()> {
    let ScaffoldWasmParams {
        name,
        framework,
        features,
        quality,
        output,
        force,
        dry_run,
    } = params;

    // Parse framework
    let wasm_framework = match framework.as_str() {
        "wasm-labs" => WasmFramework::WasmLabs,
        "pure-wasm" => WasmFramework::PureWasm,
        _ => return Err(anyhow::anyhow!("Unknown WASM framework: {}", framework)),
    };

    // Parse features
    let parsed_features: Vec<Feature> = features
        .iter()
        .filter_map(|f| match f.as_str() {
            "logging" => Some(Feature::Logging),
            "metrics" => Some(Feature::Metrics),
            "tracing" => Some(Feature::Tracing),
            _ => None,
        })
        .collect();

    // Create scaffold config
    let config = ScaffoldConfig {
        project_name: name.clone(),
        template_type: TemplateType::Wasm {
            based_on: wasm_framework,
        },
        features: parsed_features,
        quality_gates: match quality.as_str() {
            "extreme" => QualityGateConfig::extreme_tdd(),
            _ => QualityGateConfig::default(),
        },
    };

    if dry_run {
        println!("Would create WASM project: {}", name);
        println!("  Framework: {}", framework);
        println!("  Quality: {}", quality);
        return Ok(());
    }

    // Use scaffold engine
    let engine = ScaffoldEngine::new()?;
    engine.validate_config(&config)?;

    let output_dir = output.unwrap_or_else(|| PathBuf::from("."));
    let project_dir = output_dir.join(&name);

    if project_dir.exists() && !force {
        return Err(anyhow::anyhow!(
            "Directory {} already exists. Use --force to overwrite",
            project_dir.display()
        ));
    }

    engine.scaffold(config)?;

    println!("✅ Created WASM project: {}", name);
    println!("  Location: {}", project_dir.display());
    println!("  Framework: {}", framework);
    println!("\nNext steps:");
    println!("  cd {}", name);
    println!("  wasm-pack build");
    println!("  wasm-pack test --headless --firefox");

    Ok(())
}
```

### Phase 3: Wire Up in command_structure.rs

```rust
// server/src/cli/command_structure.rs

Commands::Scaffold { command } => {
    match command {
        ScaffoldCommands::Project { ... } => { ... }
        ScaffoldCommands::Agent { ... } => { ... }

        ScaffoldCommands::Wasm {
            name,
            framework,
            features,
            quality,
            output,
            force,
            dry_run,
        } => {
            let params = super::handlers::ScaffoldWasmParams {
                name,
                framework,
                features,
                quality,
                output,
                force,
                dry_run,
            };
            super::handlers::handle_scaffold_wasm(params).await
        }

        ScaffoldCommands::ListTemplates => { ... }
        ScaffoldCommands::ValidateTemplate { ... } => { ... }
    }
}
```

### Phase 4: Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scaffold_wasm_command() {
        let cmd = ScaffoldCommands::Wasm {
            name: "my-wasm".to_string(),
            framework: "wasm-labs".to_string(),
            features: vec!["logging".to_string()],
            quality: "strict".to_string(),
            output: None,
            force: false,
            dry_run: false,
        };

        match cmd {
            ScaffoldCommands::Wasm { name, framework, .. } => {
                assert_eq!(name, "my-wasm");
                assert_eq!(framework, "wasm-labs");
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    #[ignore] // Integration test
    async fn integration_scaffold_wasm_dry_run() {
        let params = ScaffoldWasmParams {
            name: "test-wasm".to_string(),
            framework: "wasm-labs".to_string(),
            features: vec![],
            quality: "standard".to_string(),
            output: None,
            force: false,
            dry_run: true,
        };

        let result = handle_scaffold_wasm(params).await;
        assert!(result.is_ok());
    }
}
```

## Complexity Analysis

Functions with complexity:
- `handle_scaffold_wasm`: CC=4 (framework match, quality match, exists check, dry-run)
- Command match arm: CC=2 (simple routing)

All functions under CC=10 threshold ✓

## Verification Commands

```bash
# WasmLabs framework
pmat scaffold wasm --name my-wasm --framework wasm-labs

# Pure WASM
pmat scaffold wasm --name pure-wasm --framework pure-wasm

# With features
pmat scaffold wasm --name feature-wasm --framework wasm-labs \
  --features logging,metrics

# Dry run
pmat scaffold wasm --name test-wasm --framework wasm-labs --dry-run

# Extreme quality
pmat scaffold wasm --name strict-wasm --framework wasm-labs --quality extreme

# Verify it builds
cd my-wasm
wasm-pack build
wasm-pack test --headless --firefox
```

## Files to Create/Modify

### Modified Files
- `server/src/cli/commands.rs` - Add Wasm variant to ScaffoldCommands
- `server/src/cli/handlers/generation_handlers.rs` - Add handle_scaffold_wasm
- `server/src/cli/handlers/mod.rs` - Export ScaffoldWasmParams and handler
- `server/src/cli/command_structure.rs` - Wire up Wasm variant

## Risk Assessment

**Low Risk:**
- Scaffolding engine already supports WASM
- WasmFramework enum already defined
- Following same pattern as Agent command (TICKET-PMAT-5030)

**Mitigation:**
- Dry-run mode for safe testing
- Force flag prevents accidental overwrites
- Integration tests verify end-to-end

## Notes

This ticket follows the same pattern as TICKET-PMAT-5030:

**Infrastructure Already Complete:**
1. Scaffolding engine supports WASM
2. WasmFramework enum defined
3. Template registry with WASM templates
4. Quality gate enforcement

**What We're Adding:**
- CLI command variant (~20 lines)
- Handler function (~60 lines)
- Wire-up in dispatcher (~15 lines)
- Tests

**Value:**
- Developers can scaffold WASM projects in <5 minutes
- WasmLabs best practices baked in
- Quality gates from day one
- Consistent with agent scaffolding

**Reference Implementation:**
- `../wasm-labs` - WASM project best practices

**TDD Cycle Duration**: Estimated 30 minutes for RED → GREEN → REFACTOR
