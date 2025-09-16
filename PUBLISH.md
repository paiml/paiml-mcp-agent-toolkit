# Publishing pmat to crates.io

## Current Status
- Version 2.87.1 is ready for publication
- GitHub release created: https://github.com/paiml/paiml-mcp-agent-toolkit/releases/tag/v2.87.1
- All code changes committed and pushed

## Issue with Current Token
The token in ~/.zshrc (`CARGO_REGISTRY_TOKEN`) appears to be invalid or expired.
Error: "403 Forbidden: authentication failed"

## Steps to Publish

1. **Get a new crates.io token**:
   - Go to https://crates.io/settings/tokens
   - Login as the crate owner (noahgift)
   - Generate a new token with publish permissions
   - Name it something like "pmat-publish-2025"

2. **Save the token**:
   ```bash
   cargo login <your-new-token>
   ```
   Or update in ~/.zshrc:
   ```bash
   export CARGO_REGISTRY_TOKEN="<your-new-token>"
   ```

3. **Publish the crate**:
   ```bash
   cargo publish --package pmat
   ```

4. **Verify publication**:
   ```bash
   # Wait a minute for crates.io to index
   cargo search pmat --limit 1
   # Should show: pmat = "2.87.1"
   ```

5. **Install and test**:
   ```bash
   cargo install pmat --force
   pmat --version  # Should show 2.87.1
   ```

## What's New in 2.87.1

### Fixed
- **Dependency Compatibility**: Resolved cargo install failures
  - Upgraded swc packages from 0.x to 14.x/24.x series
  - Fixed all swc API breaking changes
  - Pinned serde to 1.0.219 to work around swc_common issue

### Updated Dependencies
- tokio: 1.47
- pmcp: 1.4.2
- swc_ecma_parser: 24.0
- swc_common: 14.0
- tree-sitter: 0.22

The build is fully tested and working locally. Once you have a valid token,
publishing should complete successfully.