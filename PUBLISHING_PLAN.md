# Publishing Plan for pmat

## Current Status
- Old name: `paiml-mcp-agent-toolkit` (last published: 0.26.3)
- New name: `pmat` (not yet published)
- Current version: 0.27.0

## Publishing Steps

### 1. Publish pmat 0.27.0
```bash
cd server
cargo publish --dry-run  # Test first
cargo publish            # Publish to crates.io
```

### 2. Create Deprecation Release for old name
After pmat is successfully published:

1. Create a new branch for deprecation:
```bash
git checkout -b deprecate-old-crate
```

2. Update server/Cargo.toml:
   - Change name back to "paiml-mcp-agent-toolkit"
   - Change version to "0.26.4"
   - Add deprecation warning in description

3. Add deprecation warning to main.rs or lib.rs:
```rust
#![deprecated(since = "0.26.4", note = "This crate has been renamed to 'pmat'. Please use 'pmat' instead.")]
```

4. Publish deprecation version:
```bash
cargo publish
```

### 3. Update Documentation
- Update all references to point to the new crate name
- Update installation instructions
- Update badges

## Benefits
- Clean new name that matches the binary
- Maintains compatibility for existing users (with deprecation warning)
- Clear migration path