# Cargo Publishing Guide

This guide documents the process for publishing new releases of pmat to crates.io.

## Prerequisites

1. **Cargo Account**: Create an account at [crates.io](https://crates.io)
2. **API Token**: Generate a token from your [account settings](https://crates.io/settings/tokens)
3. **Token Storage**: Store securely in `~/.cargo/credentials` or environment variable
4. **Ownership**: Ensure you have publish rights to the crate

## Token Configuration

### Option 1: Cargo Credentials File (Recommended)

```bash
# Login with your token
cargo login YOUR_API_TOKEN

# This creates/updates ~/.cargo/credentials
```

### Option 2: Environment Variable

```bash
# Add to ~/.bashrc or ~/.zshrc
export CARGO_REGISTRY_TOKEN="YOUR_API_TOKEN"
```

### Option 3: GitHub Secrets (for CI/CD)

Add `CARGO_REGISTRY_TOKEN` to repository secrets for automated publishing.

## Pre-Publishing Checklist

1. **Version Update**
   ```bash
   # Update version in server/Cargo.toml
   cd server
   cargo set-version 0.26.4  # or next version
   ```

2. **Quality Verification**
   ```bash
   # Run all quality checks
   make lint
   make test-all
   pmat quality-gate --strict
   ```

3. **Documentation Update**
   - Update `RELEASE_NOTES.md` with changes
   - Update README.md if needed
   - Verify all docs are current

4. **Dry Run**
   ```bash
   cd server
   cargo publish --dry-run
   ```

## Publishing Process

### Manual Publishing

```bash
# From server directory
cd server

# Publish to crates.io
cargo publish

# Note: If build.rs modifies files, use --no-verify
cargo publish --no-verify
```

### Automated Publishing (GitHub Actions)

The project includes automated publishing via GitHub Actions:

1. Create a new release on GitHub
2. Tag with version (e.g., `v0.26.4`)
3. GitHub Action automatically publishes to crates.io

## Common Issues and Solutions

### Issue: Build Script Modifications

**Error**: "Source directory was modified by build.rs during cargo publish"

**Solution**: Use `--no-verify` flag
```bash
cargo publish --no-verify
```

**Note**: This occurs because our build.rs downloads and compresses vendor assets.

### Issue: Version Already Exists

**Error**: "Version X.Y.Z already exists"

**Solution**: Increment version in Cargo.toml and try again

### Issue: Large Package Size

**Solution**: Check `exclude` field in Cargo.toml:
```toml
exclude = [
    "*.log",
    "test_*",
    "temp_*",
    "*.profraw",
    ".pmat-cache",
    "proptest-regressions",
    "target",
    "assets/vendor/*"  # Exclude downloaded assets
]
```

### Issue: Missing Required Fields

Ensure Cargo.toml has all required fields:
```toml
[package]
name = "pmat"
version = "0.26.3"
edition = "2021"
authors = ["Your Name <email@example.com>"]
description = "Zero-configuration AI context generation..."
documentation = "https://docs.rs/pmat"
homepage = "https://github.com/paiml/pmat"
repository = "https://github.com/paiml/pmat"
license = "MIT OR Apache-2.0"
readme = "README.md"
keywords = ["mcp", "code-analysis", "refactoring", "quality", "ai"]
categories = ["command-line-utilities", "development-tools"]
```

## Post-Publishing Steps

1. **Verify Publication**
   ```bash
   # Check crates.io page
   open https://crates.io/crates/pmat
   
   # Verify installation works
   cargo install pmat
   ```

2. **Update Documentation**
   - Docs.rs will automatically build documentation
   - Verify at https://docs.rs/pmat

3. **Create GitHub Release**
   - Tag the commit with version
   - Upload pre-built binaries
   - Include release notes

4. **Announce Release**
   - Update project README
   - Post to relevant communities
   - Update dependent projects

## Version Management

### Semantic Versioning

- **MAJOR** (1.0.0): Breaking API changes
- **MINOR** (0.27.0): New features, backward compatible
- **PATCH** (0.26.4): Bug fixes, no API changes

### Pre-release Versions

For testing:
```toml
version = "0.27.0-alpha.1"
```

## Security Considerations

1. **Never commit tokens** to version control
2. **Use environment variables** in CI/CD
3. **Rotate tokens regularly**
4. **Limit token scope** if possible

## Troubleshooting

### Debug Publishing Issues

```bash
# Verbose output
RUST_LOG=debug cargo publish --dry-run

# Check package contents
cargo package --list

# Verify locally
cargo package
cd target/package/pmat-*
cargo build
```

### Getting Help

- Check [cargo publish docs](https://doc.rust-lang.org/cargo/commands/cargo-publish.html)
- Visit [crates.io help](https://crates.io/help)
- Open issue on GitHub

## Automation Script

Example script for automated releases:

```bash
#!/bin/bash
# scripts/publish-crate.sh

set -e

VERSION=$1
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

echo "Publishing version $VERSION"

# Update version
cd server
cargo set-version $VERSION

# Run quality checks
make lint
make test-fast
pmat quality-gate --strict

# Publish
cargo publish --no-verify

echo "Successfully published v$VERSION to crates.io"
```

## Best Practices

1. **Test thoroughly** before publishing
2. **Document all changes** in release notes
3. **Use --dry-run** first
4. **Keep dependencies updated**
5. **Monitor download statistics**
6. **Respond to user issues quickly**

Remember: Once published, versions cannot be deleted, only yanked!