# TICKET-PMAT-5030: Wire up `pmat scaffold agent` CLI Command

**Status**: GREEN
**Priority**: P0
**Complexity**: 2
**Estimated Time**: 30 minutes
**Dependencies**: Existing scaffold::agent module, handle_scaffold_agent function
**Sprint**: Sprint 19 - CLI Integration & Dogfooding

## Objective

Connect the existing `handle_scaffold_agent` function to the CLI dispatcher so that `pmat scaffold agent` command works end-to-end. The scaffolding engine already exists - we just need to wire it up.

## Success Criteria

- [ ] `pmat scaffold agent --name my-agent --template mcp-server` creates a working agent
- [ ] All CLI arguments properly passed to handler
- [ ] Interactive mode works
- [ ] Dry-run mode works
- [ ] Quality gates enforced on scaffolded projects
- [ ] All quality gates pass (complexity <10, coverage >80%, no SATD)

## Current State

**Already Exists:**
- `ScaffoldCommands::Agent` enum variant with all arguments (commands.rs:3073)
- `handle_scaffold_agent()` function fully implemented (generation_handlers.rs:173)
- Complete scaffolding engine in `server/src/scaffold/agent/`
- Templates and quality level enforcement

**Missing:**
- Wire-up in command_structure.rs (currently returns error at line 97-103)
- Integration tests

## Test Strategy

### Unit Tests
- [ ] `test_scaffold_agent_params_conversion` - Convert CLI args to params
- [ ] `test_scaffold_agent_dry_run` - Dry run mode
- [ ] `test_scaffold_agent_quality_levels` - Quality level validation

### Integration Tests
- [ ] `integration_scaffold_basic_agent` - Create basic MCP agent
- [ ] `integration_scaffold_hybrid_agent` - Create hybrid agent
- [ ] `integration_scaffolded_agent_builds` - Verify cargo build succeeds

## Quality Gates

- [ ] Cyclomatic complexity <10 for all functions
- [ ] Cognitive complexity <15 for all functions
- [ ] Line coverage >80%
- [ ] Branch coverage >80%
- [ ] 0 SATD violations
- [ ] 0 clippy warnings
- [ ] All tests pass

## Implementation Plan

### Phase 1: Update command_structure.rs

```rust
// server/src/cli/command_structure.rs (line 97-103)

Commands::Scaffold { command } => {
    match command {
        ScaffoldCommands::Project { ... } => { ... }

        ScaffoldCommands::Agent {
            name,
            template,
            features,
            quality,
            output,
            force,
            dry_run,
            interactive,
            deterministic_core,
            probabilistic_wrapper,
        } => {
            let params = super::handlers::ScaffoldAgentParams {
                name,
                template,
                features,
                quality,
                output,
                force,
                dry_run,
                interactive,
                deterministic_core,
                probabilistic_wrapper,
            };
            super::handlers::handle_scaffold_agent(params).await
        }
    }
}
```

### Phase 2: Add Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::commands::ScaffoldCommands;
    use std::path::PathBuf;

    #[test]
    fn test_scaffold_agent_basic() {
        let cmd = ScaffoldCommands::Agent {
            name: "test-agent".to_string(),
            template: "mcp-server".to_string(),
            features: vec![],
            quality: "strict".to_string(),
            output: None,
            force: false,
            dry_run: false,
            interactive: false,
            deterministic_core: None,
            probabilistic_wrapper: None,
        };

        // Verify command structure is correct
        match cmd {
            ScaffoldCommands::Agent { name, template, .. } => {
                assert_eq!(name, "test-agent");
                assert_eq!(template, "mcp-server");
            }
            _ => panic!("Wrong command variant"),
        }
    }

    #[test]
    #[ignore] // Integration test
    async fn integration_scaffold_agent_dry_run() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        let params = ScaffoldAgentParams {
            name: "test-agent".to_string(),
            template: "mcp-server".to_string(),
            features: vec![],
            quality: "strict".to_string(),
            output: Some(temp_dir.path().to_path_buf()),
            force: false,
            dry_run: true,
            interactive: false,
            deterministic_core: None,
            probabilistic_wrapper: None,
        };

        let result = handle_scaffold_agent(params).await;
        assert!(result.is_ok());

        // Dry run should not create files
        let agent_dir = temp_dir.path().join("test-agent");
        assert!(!agent_dir.exists());
    }

    #[test]
    #[ignore] // Integration test - requires scaffolding engine
    async fn integration_scaffold_mcp_agent() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        let params = ScaffoldAgentParams {
            name: "test-mcp-agent".to_string(),
            template: "mcp-server".to_string(),
            features: vec!["logging".to_string()],
            quality: "standard".to_string(),
            output: Some(temp_dir.path().to_path_buf()),
            force: false,
            dry_run: false,
            interactive: false,
            deterministic_core: None,
            probabilistic_wrapper: None,
        };

        let result = handle_scaffold_agent(params).await;
        assert!(result.is_ok());

        // Verify structure created
        let agent_dir = temp_dir.path().join("test-mcp-agent");
        assert!(agent_dir.exists());
        assert!(agent_dir.join("Cargo.toml").exists());
        assert!(agent_dir.join("src").exists());
    }
}
```

## Complexity Analysis

Functions with complexity:
- Command match arm: CC=2 (Project + Agent branches)

All functions under CC=10 threshold ✓

## Verification Commands

```bash
# Basic scaffolding
pmat scaffold agent --name my-agent --template mcp-server

# With features
pmat scaffold agent --name calc-agent --template calculator --features logging,telemetry

# Dry run
pmat scaffold agent --name test-agent --template mcp-server --dry-run

# Interactive mode
pmat scaffold agent --interactive

# Hybrid agent
pmat scaffold agent --name hybrid-agent --template hybrid \
  --deterministic-core state-machine \
  --probabilistic-wrapper llm

# Extreme quality
pmat scaffold agent --name strict-agent --template mcp-server --quality extreme

# Verify it builds
cd my-agent
cargo build
cargo test
```

## Files to Modify

### Modified Files
- `server/src/cli/command_structure.rs` - Wire up Agent variant (line 97-103)

### Test Files
- Add integration tests to verify end-to-end scaffolding

## Risk Assessment

**Low Risk:**
- Scaffolding engine already exists and tested
- Handler function already implemented
- Just wiring up existing components

**Mitigation:**
- Integration tests verify end-to-end
- Dry-run mode for safe testing
- Force flag prevents accidental overwrites

## Notes

This ticket is simpler than it first appears because:

**Already Complete:**
1. Scaffolding engine (`server/src/scaffold/agent/`)
2. Handler function (`handle_scaffold_agent`)
3. CLI argument structure (`ScaffoldCommands::Agent`)
4. Template system
5. Quality level enforcement

**What's Missing:**
- 10 lines of code to connect Agent variant to handler
- Integration tests to verify it works

**Value:**
- Developers can scaffold new agents in <5 minutes
- Quality gates enforced from day one
- Consistent project structure
- Best practices baked in

**TDD Cycle Duration**: Estimated 30 minutes for RED → GREEN → REFACTOR
