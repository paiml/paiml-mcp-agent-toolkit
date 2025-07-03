# Release Process

This document outlines the complete release process for pmat, ensuring quality and consistency across all release channels.

## Release Channels

1. **GitHub Releases** - Pre-built binaries and source code
2. **Crates.io** - Rust package registry
3. **Docker Hub** - Container images (planned)

## Release Types

- **Major Release** (X.0.0) - Breaking changes, major features
- **Minor Release** (0.X.0) - New features, backward compatible
- **Patch Release** (0.0.X) - Bug fixes, documentation updates

## Release Workflow

### Step 1: Pre-Release Preparation

```bash
# 1. Ensure on master branch with latest changes
git checkout master
git pull origin master

# 2. Run comprehensive quality checks
make lint
make test-all
make coverage
pmat quality-gate --strict

# 3. Check for SATD
pmat analyze satd

# 4. Verify complexity limits
pmat analyze complexity --max-complexity 20
```

### Step 2: Version Update

```bash
# Update version in server/Cargo.toml
cd server
cargo set-version 0.26.4  # Use appropriate version

# Update lock file
cargo update -p pmat

# Commit version change
git add Cargo.toml Cargo.lock
git commit -m "chore: Bump version to v0.26.4"
```

### Step 3: Update Documentation

1. **RELEASE_NOTES.md**
   ```markdown
   ## v0.26.4 - 2025-07-02

   ### Features
   - Feature description

   ### Bug Fixes
   - Fix description

   ### Documentation
   - Documentation updates

   ### Internal
   - Internal improvements
   ```

2. **README.md**
   - Update version badges if needed
   - Update installation instructions
   - Add new features to feature list

3. **CHANGELOG.md** (if maintained)
   - Add detailed change log

### Step 4: Create Release Commit

```bash
# Stage all changes
git add RELEASE_NOTES.md README.md docs/

# Create release commit
git commit -m "release: Prepare v0.26.4

- Update release notes
- Update documentation
- Bump version in Cargo.toml

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"

# Push to master
git push origin master
```

### Step 5: Create Git Tag

```bash
# Create annotated tag
git tag -a v0.26.4 -m "Release v0.26.4

See RELEASE_NOTES.md for details"

# Push tag
git push origin v0.26.4
```

### Step 6: Build Release Artifacts

```bash
# Build for multiple platforms
make release-build

# This creates binaries in:
# - target/release/pmat (local platform)
# - target/x86_64-unknown-linux-gnu/release/pmat
# - target/aarch64-unknown-linux-gnu/release/pmat
# - target/x86_64-apple-darwin/release/pmat
# - target/aarch64-apple-darwin/release/pmat
# - target/x86_64-pc-windows-gnu/release/pmat
```

### Step 7: Publish to Crates.io

```bash
cd server

# Dry run first
cargo publish --dry-run

# Publish (use --no-verify if build.rs modifies files)
cargo publish --no-verify

# Verify publication
open https://crates.io/crates/pmat
```

### Step 8: Create GitHub Release

1. Go to [GitHub Releases](https://github.com/paiml/pmat/releases)
2. Click "Draft a new release"
3. Select the tag `v0.26.4`
4. Title: `v0.26.4`
5. Copy content from RELEASE_NOTES.md
6. Upload pre-built binaries:
   - `pmat-linux-x86_64.tar.gz`
   - `pmat-linux-aarch64.tar.gz`
   - `pmat-macos-x86_64.tar.gz`
   - `pmat-macos-aarch64.tar.gz`
   - `pmat-windows-x86_64.zip`
7. Check "Set as the latest release"
8. Publish release

### Step 9: Post-Release Verification

```bash
# 1. Test crates.io installation
cargo install pmat --force
pmat --version

# 2. Test binary downloads
curl -L https://github.com/paiml/pmat/releases/download/v0.26.4/pmat-linux-x86_64.tar.gz | tar xz
./pmat --version

# 3. Verify documentation
open https://docs.rs/pmat

# 4. Run smoke tests
pmat context
pmat analyze complexity
pmat quality-gate
```

### Step 10: Announcements

1. **Update README badges** - Ensure version badges show new version
2. **Community announcements** - Post in relevant forums/channels
3. **Update dependent projects** - Notify projects using pmat

## Automated Release Process

### GitHub Actions Workflow

The project includes `.github/workflows/release.yml` for automation:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  publish-crates-io:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      - name: Publish to crates.io
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
        run: |
          cd server
          cargo publish --no-verify

  build-binaries:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc
    # ... build and upload steps
```

## Release Checklist

### Pre-Release
- [ ] All tests pass (`make test-all`)
- [ ] Lints pass (`make lint`)
- [ ] Quality gate passes (`pmat quality-gate --strict`)
- [ ] No SATD present (`pmat analyze satd`)
- [ ] Complexity within limits
- [ ] Documentation updated
- [ ] Version bumped in Cargo.toml
- [ ] RELEASE_NOTES.md updated

### Release
- [ ] Git tag created and pushed
- [ ] Published to crates.io
- [ ] GitHub release created
- [ ] Binaries uploaded to release
- [ ] Docs.rs build successful

### Post-Release
- [ ] Installation from crates.io works
- [ ] Binary downloads work
- [ ] Documentation accessible
- [ ] Smoke tests pass
- [ ] Announcements made

## Emergency Procedures

### Yanking a Release

If a critical issue is found after release:

```bash
# Yank from crates.io
cargo yank --version 0.26.4

# Create patch release immediately
# Follow normal release process for 0.26.5
```

### Hotfix Process

1. Create hotfix branch from tag
   ```bash
   git checkout -b hotfix/v0.26.5 v0.26.4
   ```

2. Apply fix and test thoroughly

3. Fast-track through release process

4. Cherry-pick to master if applicable

## Version Naming Convention

- **Stable**: `0.26.4`
- **Pre-release**: `0.27.0-alpha.1`, `0.27.0-beta.1`, `0.27.0-rc.1`
- **Nightly**: `0.27.0-nightly.20250702`

## Release Cadence

- **Major**: As needed for breaking changes
- **Minor**: Monthly for new features
- **Patch**: As needed for bug fixes
- **Security**: Immediate release for security fixes

## Quality Gates

Every release must pass:

1. **Zero SATD** - No technical debt comments
2. **Complexity Limits** - All functions < 20 cyclomatic complexity
3. **Test Coverage** - Minimum 90% coverage
4. **Clean Lints** - All clippy pedantic/nursery warnings resolved
5. **Documentation** - All public APIs documented

## Release Notes Template

```markdown
## vX.Y.Z - YYYY-MM-DD

### 🎉 Features
- Brief description of new feature (#PR)

### 🐛 Bug Fixes
- Brief description of fix (#PR)

### 📚 Documentation
- Documentation improvements (#PR)

### 🔧 Internal
- Internal improvements (#PR)

### 🙏 Contributors
- @username - contribution description

**Full Changelog**: https://github.com/paiml/pmat/compare/vX.Y.Y...vX.Y.Z
```

## Rollback Procedure

If issues are discovered post-release:

1. **Assess Impact** - Determine severity and scope
2. **Communicate** - Notify users of known issues
3. **Yank if Critical** - Remove from crates.io if severe
4. **Prepare Fix** - Fast-track patch release
5. **Post-Mortem** - Document lessons learned

Remember: Quality over speed. Never rush a release!