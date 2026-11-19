# Release Process

How pmat reaches users. Source of truth: `.github/workflows/release.yml` (header
comment), `scripts/publish-from-tag.sh`, `scripts/release-check.sh`, `CHANGELOG.md`.
Last verified against v3.45.1.

## Policy

- No workflow ever publishes the crate. The crates.io token never enters GitHub
  secrets, and `--allow-dirty` is never used.
- The crate is published from a detached worktree of the tag, on the release
  workstation, with `make publish-from-tag`.
- Nothing is tagged or published off a red clean-room gate.

## Channels

1. **Git tag** `vX.Y.Z` on `master`.
2. **GitHub release** `pmat X.Y.Z`, created by `release.yml` as a prerelease, with
   binaries attached by `binary-release.yml`.
3. **crates.io** `pmat`, published by `make publish-from-tag`.

## Steps

### 1. Version bump PR

Branch off `master` (`master` is protected; no direct pushes). In one PR:

- `Cargo.toml` `[package] version`, and `Cargo.lock` to match.
- `CHANGELOG.md`: move `[Unreleased]` entries under `## [X.Y.Z] - YYYY-MM-DD`.
- `docs/roadmaps/entries/GH-<n>.yaml` for the ticket (CB-2115 turns master red
  without it).
- The lock must honor `rust-version` in `Cargo.toml`. v3.45.0 failed here: a
  lock needing rustc 1.95 did not build on 1.93.

Merge it through the normal gates (`ci / gate`, `workspace-test`), squash.

### 2. Pre-tag checks

Before tagging, on the merged commit:

- `master` is green.
- The last mutation (diff) nightly verdict is green. It is a release gate, not a
  merge gate (BSE-06).

### 3. Tag

```bash
git checkout master && git pull --ff-only origin master
git tag -a vX.Y.Z -m "pmat X.Y.Z"
git push origin vX.Y.Z
```

The tag version must equal `Cargo.toml` (release.yml and `publish-from-tag.sh`
both refuse a mismatch) and the tagged commit must be on `origin/master`.
A published tag is never deleted, moved or re-cut. If a tag's release fails,
fix forward with the next patch version (v3.45.0 was tagged, never published,
and superseded by v3.45.1).

### 4. release.yml (triggered by the tag)

Runs on `[self-hosted, clean-room]`. Jobs, in order, each gating the next:

1. `create-release`: parses the tag, checks tag == `Cargo.toml` version. Creates
   nothing.
2. `gate`: the fleet `unified-gate.yml` clean-room gate at the tagged sha.
3. `verify`: `cargo package --locked` (the package builds from its own tarball).
4. `prerelease`: only after 2 and 3 are green. `gh release create --prerelease
   --verify-tag --generate-notes`, then dispatches `binary-release.yml` (builds
   and attaches binaries) and `post-release.yml` (tag == `Cargo.toml`, MSRV
   check). A release made with `GITHUB_TOKEN` raises no `release: published`
   event, which is why the dispatch is explicit.

A red `gate` or `verify` creates no release. Re-run against an existing tag with
`workflow_dispatch` (input `tag`).

### 5. Publish from the tag

On the release workstation, after `release.yml` is green:

```bash
make publish-from-tag TAG=vX.Y.Z DRY_RUN=1   # dry run: packages, publishes nothing
make publish-from-tag TAG=vX.Y.Z             # real publish
```

The script refuses (exit 2) a malformed or missing tag, a tag not on
`origin/master`, or a version mismatch, and exits 3 on a dirty worktree. It runs
under `env -u CARGO_REGISTRY_TOKEN` and uses cargo's own credentials file.

### 6. Promote to Latest

The prerelease is promoted only after the crate is on crates.io:

```bash
gh release edit vX.Y.Z --prerelease=false --latest
```

### 7. Verify

```bash
make release-check                       # version in Cargo.toml: tagged, released, on crates.io
gh release list --limit 3                # vX.Y.Z must read "Latest", not "Pre-release"
gh release view vX.Y.Z --json isPrerelease,isDraft   # both false
cargo search pmat | head -1              # crates.io shows X.Y.Z
cargo install pmat --locked --force && pmat --version
make dogfood-published VERSION=X.Y.Z     # install from crates.io, run the release gate on it
```

`release-check` exits 1 naming the first missing channel (tag, GitHub release,
crates.io) and 3 when a read failed (not measured, never "missing"). The
`nightly` pre-release in the list is the rolling nightly build, not a release.

## Failure handling

- **Gate or verify red:** no release exists. Fix on `master` through a PR, bump
  the patch version, tag again with the new version.
- **Published version is broken:** a bad version cannot be un-published. Fix
  forward with a patch release. Yanking follows the standing operator rules
  (`cargo yank --version X.Y.Z`), not this document.
- **Binaries missing from the release:** `gh workflow run binary-release.yml -f
  tag=vX.Y.Z`.

## Version naming

`X.Y.Z` stable. Patch for fixes, minor for features, major for breaking changes.
