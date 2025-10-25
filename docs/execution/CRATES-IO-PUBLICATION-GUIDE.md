# Guide: Publishing PMAT to crates.io

This document outlines the process for publishing PMAT to crates.io and updating the npm package.

## Prerequisites

- Cargo account with publish rights
- npm account with publish rights
- Clean build with all tests passing
- Release notes and CHANGELOG.md updated
- Version numbers updated in all files

## Step 1: Final Verification

Before publishing, run these verification steps:

```bash
# Clean and rebuild
cd /home/noah/src/paiml-mcp-agent-toolkit
cargo clean
cargo build

# Run clippy
cargo clippy --workspace -- -D warnings

# Run tests (focusing on critical tests)
cargo test --lib -- --skip ignored_tests

# Validate book examples
make validate-book
```

## Step 2: Package Creation

Create the package to verify it builds correctly:

```bash
# Create the package
cd /home/noah/src/paiml-mcp-agent-toolkit
cargo package --manifest-path server/Cargo.toml --allow-dirty --no-verify

# Verify the package contents
cargo package --list --manifest-path server/Cargo.toml --allow-dirty
```

## Step 3: Publishing to crates.io

Once verification is complete, publish to crates.io:

```bash
# Login to crates.io if needed
cargo login

# Publish the package
cd /home/noah/src/paiml-mcp-agent-toolkit/server
cargo publish --allow-dirty
```

## Step 4: Update npm Package

After crates.io publication, update and publish the npm package:

```bash
# Install the rust binary from crates.io
cargo install pmat

# Copy the binary to npm package directory
cd /home/noah/src/paiml-mcp-agent-toolkit/npm-package
node install.js

# Verify the package
npm pack

# Publish to npm
npm publish
```

## Step 5: Create GitHub Release

Create a GitHub release to make the new version official:

```bash
# Create a git tag
git tag -a v2.171.1 -m "Release v2.171.1 - C/C++ Language Support"

# Push the tag
git push origin v2.171.1

# Create the GitHub release using gh CLI
gh release create v2.171.1 \
  --title "PMAT v2.171.1 - C/C++ Language Support" \
  --notes-file docs/releases/RELEASE-v2.171.1.md
```

## Step 6: Update Distribution Packages

After the GitHub release, update all distribution packages:

1. **Homebrew**
   ```bash
   cd /home/noah/src/paiml-mcp-agent-toolkit/homebrew
   ./update-formula.sh 2.171.1
   ```

2. **AUR**
   ```bash
   cd /home/noah/src/paiml-mcp-agent-toolkit/aur
   ./update-package.sh 2.171.1
   ```

3. **Debian/Ubuntu**
   ```bash
   cd /home/noah/src/paiml-mcp-agent-toolkit/debian
   ./build-deb.sh 2.171.1
   ```

4. **Chocolatey**
   ```bash
   cd /home/noah/src/paiml-mcp-agent-toolkit/chocolatey
   ./build-package.ps1 2.171.1
   ```

## Troubleshooting

### Common Issues with crates.io Publication

1. **Package Verification Failed**
   - Use `--no-verify` flag for packages with complex build scripts
   - Ensure all dependencies are correctly specified

2. **Version Conflict**
   - Ensure the version hasn't been published before
   - Use `cargo yank` to remove a problematic version if needed

3. **Size Limits**
   - crates.io has a 10MB limit for the .crate file
   - Remove large assets, test data, or examples if needed

### Common Issues with npm Publication

1. **Binary Not Found**
   - Ensure the installation script correctly locates the binary
   - Check permissions on the binary

2. **Version Conflict**
   - Ensure the npm version matches crates.io version
   - Use `npm unpublish` if needed for a problematic version

## Post-Publication Verification

After publishing, verify the installation works from both registries:

```bash
# Verify crates.io installation
cargo uninstall pmat
cargo install pmat
pmat --version

# Verify npm installation
npm uninstall -g pmat-agent
npm install -g pmat-agent
pmat --version
```

## Additional Resources

- [Cargo Publishing Documentation](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [npm Publishing Documentation](https://docs.npmjs.com/packages-and-modules/contributing-packages-to-the-registry)
- [GitHub Release Documentation](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository)