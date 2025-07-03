# Deprecation Notice

The `paiml-mcp-agent-toolkit` crate has been renamed to `pmat` starting from version 0.27.0.

## For Users

Please update your `Cargo.toml`:

```toml
# Old (deprecated)
[dependencies]
paiml-mcp-agent-toolkit = "0.26.3"

# New
[dependencies]
pmat = "0.27.0"
```

## For CLI Users

The binary has been renamed from `paiml-mcp-agent-toolkit` to `pmat`:

```bash
# Old
paiml-mcp-agent-toolkit context

# New
pmat context
```

## Migration

1. Update your dependency in `Cargo.toml`
2. Update any scripts or CI/CD pipelines to use `pmat` instead of `paiml-mcp-agent-toolkit`
3. The API remains the same - only the package and binary names have changed

## Support

The old package name will receive one final update (0.26.4) with this deprecation notice, but all new features and fixes will only be published under the `pmat` name.

For more information, visit: https://github.com/paiml/paiml-mcp-agent-toolkit