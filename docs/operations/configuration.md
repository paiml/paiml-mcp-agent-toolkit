# Configuration Guide

*Reference: SPECIFICATION.md Section 36*

## Overview

PMAT supports flexible configuration through multiple sources with clear precedence rules. Configuration can be provided via files, environment variables, or command-line arguments.

## Configuration Sources

### Precedence Order (Highest to Lowest)

1. Command-line arguments
2. Environment variables
3. Project-local `pmat.toml`
4. User config `~/.config/pmat/config.toml`
5. System config `/etc/pmat/config.toml`
6. Built-in defaults

## Configuration File Format

### pmat.toml Structure

```toml
# Project-specific PMAT configuration
version = "1.0"

[project]
name = "my-project"
root = "."
exclude = ["target/", "node_modules/", "*.test.rs"]

[quality]
max_cyclomatic = 20
max_cognitive = 15
max_satd = 0
min_coverage = 80
require_docs = true
enforce_pre_commit = true

[analysis]
languages = ["rust", "typescript", "python"]
parallel = true
cache_enabled = true
cache_dir = ".pmat-cache"

[refactor]
target_complexity = 10
preserve_comments = true
format_after = true
incremental = true

[mcp]
enabled = true
port = 3000
host = "127.0.0.1"
max_connections = 100

[logging]
level = "info"
format = "pretty"  # pretty, json, compact
file = "pmat.log"
rotate = "daily"

[performance]
threads = 0  # 0 = auto-detect
memory_limit = "2GB"
timeout = 300  # seconds

[output]
default_format = "human"  # human, json, sarif, markdown
color = "auto"  # always, never, auto
verbose = false
```

## Environment Variables

All configuration options can be set via environment variables:

```bash
# Format: PMAT_<SECTION>_<KEY>
export PMAT_QUALITY_MAX_CYCLOMATIC=15
export PMAT_LOGGING_LEVEL=debug
export PMAT_MCP_PORT=3001
export PMAT_OUTPUT_FORMAT=json

# Special variables
export PMAT_CONFIG=/path/to/custom/config.toml
export PMAT_NO_CONFIG=1  # Ignore all config files
```

## Command-Line Override

```bash
# Override specific options
pmat analyze complexity \
  --max-cyclomatic 25 \
  --format json \
  --no-cache

# Use custom config file
pmat --config ./custom-pmat.toml quality-gate

# Ignore all config files
pmat --no-config analyze satd
```

## Profile-Based Configuration

### Development vs Production

```toml
# pmat.toml
[profile.dev]
quality.max_cyclomatic = 30
logging.level = "debug"
performance.threads = 1

[profile.prod]
quality.max_cyclomatic = 15
logging.level = "warn"
performance.threads = 0

[profile.ci]
quality.enforce_pre_commit = true
output.format = "json"
logging.file = "ci-run.log"
```

Activate profile:
```bash
pmat --profile dev analyze complexity
# Or via environment
export PMAT_PROFILE=prod
```

## Dynamic Configuration

### Runtime Reloading

```rust
use notify::{Watcher, RecursiveMode};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ConfigManager {
    config: Arc<RwLock<Config>>,
    watcher: notify::RecommendedWatcher,
}

impl ConfigManager {
    pub fn watch_for_changes(&mut self) {
        let config = self.config.clone();
        
        self.watcher.watch("pmat.toml", RecursiveMode::NonRecursive)
            .expect("Failed to watch config");
        
        // On change, reload config
        tokio::spawn(async move {
            let new_config = Config::load().await?;
            *config.write().await = new_config;
            info!("Configuration reloaded");
        });
    }
}
```

## Validation

### Schema Validation

```rust
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Serialize, Validate)]
pub struct QualityConfig {
    #[validate(range(min = 1, max = 100))]
    pub max_cyclomatic: u32,
    
    #[validate(range(min = 1, max = 100))]
    pub max_cognitive: u32,
    
    #[validate(range(min = 0, max = 100))]
    pub min_coverage: u8,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config: Config = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }
}
```

## Migration

### Version Compatibility

```toml
# pmat.toml
version = "1.0"  # Config schema version

[_meta]
min_pmat_version = "2.5.0"
max_pmat_version = "3.0.0"
```

Migration handling:
```rust
fn migrate_config(old: OldConfig) -> Result<Config> {
    match old.version.as_str() {
        "0.1" => migrate_v0_1_to_v1_0(old),
        "0.2" => migrate_v0_2_to_v1_0(old),
        "1.0" => Ok(Config::from(old)),
        v => Err(format!("Unsupported config version: {}", v)),
    }
}
```

## Team Settings

### Shared Configuration

```bash
# Check in team config
git add pmat.toml
git commit -m "Team quality standards"

# Individual overrides (git-ignored)
echo "pmat.local.toml" >> .gitignore
cat > pmat.local.toml << EOF
[logging]
level = "trace"
EOF
```

### CI/CD Integration

```yaml
# .github/workflows/quality.yml
env:
  PMAT_PROFILE: ci
  PMAT_QUALITY_MAX_CYCLOMATIC: 20
  PMAT_OUTPUT_FORMAT: json
```

## Default Values

Built-in defaults for all options:

```rust
impl Default for Config {
    fn default() -> Self {
        Self {
            quality: QualityConfig {
                max_cyclomatic: 20,
                max_cognitive: 15,
                max_satd: 0,
                min_coverage: 80,
                require_docs: false,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "pretty".to_string(),
            },
            // ... other defaults
        }
    }
}
```

## Troubleshooting

### Debug Configuration Loading

```bash
# Show effective configuration
pmat config show

# Validate configuration
pmat config validate

# Show config source for each value
pmat config show --sources

# Test config without running
pmat --dry-run --config test.toml analyze
```

### Common Issues

1. **Config not found**: Check file path and permissions
2. **Invalid values**: Run `pmat config validate`
3. **Env var ignored**: Check spelling (PMAT_SECTION_KEY)
4. **Override not working**: Check precedence order

## Best Practices

1. **Version control pmat.toml**: Share team standards
2. **Use profiles**: Separate dev/prod settings
3. **Document custom settings**: Add comments in TOML
4. **Validate in CI**: Run `pmat config validate`
5. **Keep defaults reasonable**: Most projects shouldn't need config
6. **Use env vars in CI**: Easier than file management

## Related Documentation

- [Quality Standards](../quality/standards.md)
- [Error Handling](./error-handling.md)
- [SPECIFICATION.md Section 36](../SPECIFICATION.md#36-configuration)