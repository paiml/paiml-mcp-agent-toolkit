# Canonical Version Updates Specification for Rust Crates

**Status**: Specification v1.0  
**Created**: 2025-08-14  
**Priority**: HIGH  
**Category**: Release Engineering  
**Enforcement Mode**: STRICT  

## Executive Summary

This specification defines the canonical approach for version management in Rust crates, specifically for the `pmat` project. It incorporates industry best practices, automated tooling, and the PDMT (Pragmatic Deterministic MCP Templating) approach to ensure consistent, reliable, and semantic-versioning-compliant releases.

## Problem Statement

The recent release accidentally created version 2.0.1 instead of 2.3.1, demonstrating the need for:
- Automated version validation
- Workspace-aware version synchronization
- Pre-release quality gates
- Deterministic release workflows

## Canonical Rust Crate Version Management

### Core Principles

1. **Semantic Versioning (SemVer) Compliance**
   - MAJOR.MINOR.PATCH format strictly enforced
   - MAJOR: Incompatible API changes
   - MINOR: Backward-compatible functionality additions
   - PATCH: Backward-compatible bug fixes
   - Pre-1.0: Left-shifted rules (0.x.y → 0.x+1.0 can break API)

2. **Workspace Version Synchronization**
   - Single source of truth in workspace `Cargo.toml`
   - All member crates inherit workspace version
   - Automated propagation to all manifests

3. **Immutable Releases**
   - Published versions are permanent
   - No force-pushing tags
   - No republishing with same version

4. **Automated Validation**
   - cargo-semver-checks for API compatibility
   - cargo-deny for dependency auditing
   - Quality gates before version bump

## PDMT Deterministic Todo Generation

### Pre-Release Quality Requirements

```yaml
quality_requirements:
  complexity:
    max_cyclomatic: 20
    max_cognitive: 15
  coverage:
    minimum: 80%
    doctests: 100%
    property_tests: required
  debt:
    satd_tolerance: 0
    todo_comments: 0
  dependencies:
    security_vulnerabilities: 0
    outdated_allowed: 5
```

### Deterministic Release Todos

```json
{
  "release_todos": {
    "seed": 42,
    "granularity": "high",
    "priority": "critical",
    "enforcement_mode": "strict",
    "todos": [
      {
        "id": "pre-release-001",
        "title": "Validate Current Version State",
        "description": "Ensure version consistency across workspace",
        "validation_commands": [
          "grep '^version' Cargo.toml server/Cargo.toml | uniq -c | grep -q '2 version'",
          "cargo metadata --no-deps --format-version 1 | jq -r '.packages[].version' | uniq -c"
        ],
        "success_criteria": "All versions match",
        "estimated_time": "5m",
        "dependencies": []
      },
      {
        "id": "pre-release-002",
        "title": "Run SemVer Compatibility Check",
        "description": "Verify API compatibility for version bump type",
        "validation_commands": [
          "cargo install cargo-semver-checks --locked",
          "cargo semver-checks check-release"
        ],
        "success_criteria": "No breaking changes for patch/minor",
        "estimated_time": "10m",
        "dependencies": ["pre-release-001"]
      },
      {
        "id": "pre-release-003",
        "title": "Execute Full Quality Gate",
        "description": "Run comprehensive quality validation",
        "validation_commands": [
          "make lint",
          "make test",
          "pmat quality-gate --fail-on-violation"
        ],
        "success_criteria": "All quality gates pass",
        "estimated_time": "15m",
        "dependencies": ["pre-release-001"]
      },
      {
        "id": "pre-release-004",
        "title": "Audit Dependencies",
        "description": "Check for security vulnerabilities and updates",
        "validation_commands": [
          "cargo audit",
          "cargo outdated --root-deps-only",
          "cargo tree --duplicate"
        ],
        "success_criteria": "No critical vulnerabilities",
        "estimated_time": "5m",
        "dependencies": []
      },
      {
        "id": "pre-release-005",
        "title": "Update CHANGELOG.md",
        "description": "Document all changes since last release",
        "validation_commands": [
          "git log --oneline $(git describe --tags --abbrev=0)..HEAD",
          "grep -q '## \\[Unreleased\\]' CHANGELOG.md"
        ],
        "success_criteria": "CHANGELOG has unreleased section",
        "estimated_time": "10m",
        "dependencies": ["pre-release-003"]
      },
      {
        "id": "release-001",
        "title": "Determine Version Bump Type",
        "description": "Analyze changes to determine major/minor/patch",
        "validation_commands": [
          "cargo semver-checks check-release --baseline-version $(git describe --tags --abbrev=0)"
        ],
        "success_criteria": "Version bump type identified",
        "estimated_time": "5m",
        "dependencies": ["pre-release-002"]
      },
      {
        "id": "release-002",
        "title": "Update Version in Workspace",
        "description": "Bump version in workspace root Cargo.toml",
        "validation_commands": [
          "sed -i 's/^version = .*/version = \"NEW_VERSION\"/' Cargo.toml",
          "sed -i 's/^version = .*/version = \"NEW_VERSION\"/' server/Cargo.toml",
          "cargo check --all"
        ],
        "success_criteria": "All crates compile with new version",
        "estimated_time": "5m",
        "dependencies": ["release-001"]
      },
      {
        "id": "release-003",
        "title": "Create Release Commit",
        "description": "Commit version changes with conventional format",
        "validation_commands": [
          "git add -A",
          "git commit -m 'chore: release vNEW_VERSION'",
          "git log --oneline -1"
        ],
        "success_criteria": "Clean commit with version tag",
        "estimated_time": "2m",
        "dependencies": ["release-002", "pre-release-005"]
      },
      {
        "id": "release-004",
        "title": "Create Git Tag",
        "description": "Tag the release commit",
        "validation_commands": [
          "git tag -a vNEW_VERSION -m 'Release vNEW_VERSION'",
          "git tag --verify vNEW_VERSION 2>/dev/null || git show vNEW_VERSION"
        ],
        "success_criteria": "Tag created and points to correct commit",
        "estimated_time": "2m",
        "dependencies": ["release-003"]
      },
      {
        "id": "release-005",
        "title": "Build Release Artifacts",
        "description": "Build optimized binaries for all targets",
        "validation_commands": [
          "cargo build --release --locked",
          "cargo test --release --locked",
          "strip target/release/pmat"
        ],
        "success_criteria": "Binaries built and optimized",
        "estimated_time": "20m",
        "dependencies": ["release-003"]
      },
      {
        "id": "release-006",
        "title": "Publish to crates.io",
        "description": "Publish the crate to crates.io registry",
        "validation_commands": [
          "cargo publish --dry-run",
          "cargo package --list",
          "cargo publish"
        ],
        "success_criteria": "Crate published successfully",
        "estimated_time": "5m",
        "dependencies": ["release-005"]
      },
      {
        "id": "release-007",
        "title": "Push to GitHub",
        "description": "Push commits and tags to GitHub",
        "validation_commands": [
          "git push origin master",
          "git push origin vNEW_VERSION"
        ],
        "success_criteria": "Code and tags on GitHub",
        "estimated_time": "2m",
        "dependencies": ["release-004"]
      },
      {
        "id": "release-008",
        "title": "Create GitHub Release",
        "description": "Create release with notes and artifacts",
        "validation_commands": [
          "gh release create vNEW_VERSION --title 'vNEW_VERSION' --notes-file RELEASE_NOTES.md",
          "gh release upload vNEW_VERSION target/release/pmat"
        ],
        "success_criteria": "GitHub release with artifacts",
        "estimated_time": "5m",
        "dependencies": ["release-007", "release-005"]
      },
      {
        "id": "post-release-001",
        "title": "Verify Installation Methods",
        "description": "Test all installation methods work",
        "validation_commands": [
          "cargo install pmat --force",
          "pmat --version | grep -q NEW_VERSION",
          "curl -sSfL install.sh | bash && pmat --version"
        ],
        "success_criteria": "All installation methods work",
        "estimated_time": "10m",
        "dependencies": ["release-006", "release-008"]
      },
      {
        "id": "post-release-002",
        "title": "Update Documentation",
        "description": "Update README and docs with new version",
        "validation_commands": [
          "sed -i 's/v[0-9]\\+\\.[0-9]\\+\\.[0-9]\\+/vNEW_VERSION/g' README.md",
          "grep -q 'vNEW_VERSION' README.md"
        ],
        "success_criteria": "Documentation updated",
        "estimated_time": "5m",
        "dependencies": ["post-release-001"]
      },
      {
        "id": "post-release-003",
        "title": "Monitor Release Health",
        "description": "Check for issues post-release",
        "validation_commands": [
          "gh workflow view CI --json conclusion",
          "cargo search pmat | grep -q NEW_VERSION",
          "gh issue list --label bug --created '>1 hour ago'"
        ],
        "success_criteria": "No critical issues reported",
        "estimated_time": "10m",
        "dependencies": ["post-release-001"]
      }
    ]
  }
}
```

## Tooling Requirements

### Essential Tools

1. **cargo-release** (Recommended)
   ```toml
   [workspace.metadata.release]
   sign-commit = true
   sign-tag = true
   push-remote = "origin"
   release = true
   pre-release-replacements = [
     {file="README.md", search="pmat = \"[0-9.]+\"", replace="pmat = \"{{version}}\""},
     {file="CHANGELOG.md", search="Unreleased", replace="{{version}}"}
   ]
   ```

2. **cargo-semver-checks**
   ```bash
   cargo install cargo-semver-checks
   cargo semver-checks check-release
   ```

3. **cargo-audit**
   ```bash
   cargo install cargo-audit
   cargo audit
   ```

4. **cargo-outdated**
   ```bash
   cargo install cargo-outdated
   cargo outdated --root-deps-only
   ```

## Automation Configuration

### GitHub Actions Workflow

```yaml
name: Canonical Release
on:
  workflow_dispatch:
    inputs:
      bump_type:
        type: choice
        options: [patch, minor, major, auto]
        
jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Install release tools
        run: |
          cargo install cargo-release cargo-semver-checks cargo-audit
      
      - name: Quality gates
        run: |
          make lint
          make test
          cargo audit
          
      - name: Determine version bump
        id: version
        run: |
          if [ "${{ inputs.bump_type }}" = "auto" ]; then
            cargo semver-checks check-release || echo "bump=major" >> $GITHUB_OUTPUT
            # Logic to determine minor vs patch
          else
            echo "bump=${{ inputs.bump_type }}" >> $GITHUB_OUTPUT
          fi
          
      - name: Release
        run: |
          cargo release ${{ steps.version.outputs.bump }} --execute
```

### Makefile Integration

```makefile
.PHONY: release-patch release-minor release-major

release-patch:
	@$(MAKE) pre-release-checks
	cargo release patch --execute

release-minor:
	@$(MAKE) pre-release-checks
	cargo release minor --execute

release-major:
	@$(MAKE) pre-release-checks
	cargo release major --execute

pre-release-checks:
	cargo semver-checks check-release
	cargo audit
	cargo outdated --exit-code 1
	$(MAKE) lint
	$(MAKE) test
	pmat quality-gate --fail-on-violation
```

## Version Numbering Rules

### Pre-1.0 Development
- 0.0.x: Alpha/experimental
- 0.x.0: Beta/stabilizing API
- 0.x.y: Bug fixes and minor features

### Post-1.0 Stability
- x.0.0: Major breaking changes
- x.y.0: New features, backward compatible
- x.y.z: Bug fixes only

### Special Cases
- Release candidates: x.y.z-rc.N
- Beta releases: x.y.z-beta.N
- Nightly builds: x.y.z-nightly.YYYYMMDD

## Common Pitfalls to Avoid

1. **Version Regression**: Never decrease version numbers (2.3.0 → 2.0.1)
2. **Workspace Desync**: Always update all workspace members
3. **Missing Changelog**: Every release needs documentation
4. **Skipping Quality Gates**: Never bypass pre-release checks
5. **Force Pushing Tags**: Tags should be immutable
6. **Publishing Without Testing**: Always verify locally first
7. **Forgetting Dependencies**: Update internal workspace dependencies

## Recovery Procedures

### If Wrong Version Published
1. **Cannot unpublish from crates.io** (by design)
2. Immediately publish a new patch version
3. Mark the bad version as yanked: `cargo yank --version x.y.z`
4. Update documentation to skip bad version

### If CI/CD Fails Mid-Release
1. Check git tags: `git tag -l`
2. Check crates.io: `cargo search pmat`
3. Resume from appropriate todo step
4. Use `--no-verify` flags cautiously

## Quality Metrics

### Release Health Indicators
- Zero failed CI runs post-release
- No yanked versions in 6 months
- All quality gates passing
- SemVer compliance 100%
- Installation success rate >99%

### Version Velocity Targets
- Patch releases: Weekly if needed
- Minor releases: Bi-weekly
- Major releases: Quarterly maximum

## Implementation Checklist

- [ ] Install cargo-release toolchain
- [ ] Configure workspace.metadata.release
- [ ] Set up GitHub Actions workflow
- [ ] Create release Makefile targets
- [ ] Document in CONTRIBUTING.md
- [ ] Train team on new process
- [ ] Monitor first 3 releases
- [ ] Adjust based on lessons learned

## References

- [Cargo Book: SemVer Compatibility](https://doc.rust-lang.org/cargo/reference/semver.html)
- [cargo-release Documentation](https://github.com/crate-ci/cargo-release)
- [cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Keep a Changelog](https://keepachangelog.com/)

## Conclusion

This specification provides a deterministic, quality-enforced approach to version management for Rust crates. By following these canonical practices and using the PDMT-generated todos, we ensure consistent, reliable releases that maintain backward compatibility and high quality standards.

The key innovation is treating version updates as a series of validated, deterministic todos with clear success criteria and quality gates at every step. This prevents the version regression issues we experienced and ensures every release meets our extreme quality standards.