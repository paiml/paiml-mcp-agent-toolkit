# Implementation receipt — PMAT-678 (#1274): `make publish-from-tag`

The ticket, verbatim:

> 3.38.0: cargo publish --dry-run --locked refused the tree because untracked .claude/agent-memory/
> counts as dirty; the operator moved it aside by hand. Structural fix: Makefile target
> publish-from-tag TAG= that refuses when TAG is absent or not on master, git worktree add --detach,
> env -u CARGO_REGISTRY_TOKEN cargo publish --dry-run --locked then cargo publish --locked, worktree
> removed. Belt: .claude/agent-memory/ in .gitignore; --locked on release-verify's cargo install.
> Falsifier: with an untracked file in the main tree the target still dry-runs clean from the
> worktree; a non-existent tag exits 2 before any cargo call; dropping --detach → the
> zero-untracked-files assertion RED.

**Why now.** `release.yml`'s header has named `make publish-from-tag TAG=vX.Y.Z` (PMAT-678) as the
crate's one publish path since that policy was written, but no such target was ever added:
`git log -S'publish-from-tag:' origin/master -- Makefile` is empty. The 3.42.0 release needs the
path the workflow names to exist (paiml/infra standing rules §8b, condition 4: "publish by the
repo's own publish path").

## What changed

| Part | Where | Behaviour |
|---|---|---|
| Target | `Makefile` `publish-from-tag` | `TAG` empty → usage, exit 2. Otherwise runs `scripts/publish-from-tag.sh "$(TAG)"`, passing `DRY_RUN`. |
| Script | `scripts/publish-from-tag.sh` | Exit 2 before any cargo call when TAG is empty, not `vX.Y.Z`, absent, not an ancestor of `origin/master` (fetched first), or when `[package] version` in `TAG:Cargo.toml` is not the tag's version. Then `git worktree add --detach` into a `mktemp -d` under `$TMPDIR`. Exit 3 if that worktree has any untracked or ignored entry. Then `env -u CARGO_REGISTRY_TOKEN cargo publish --dry-run --locked` inside it. `DRY_RUN=1` stops there. Otherwise, exit 3 on any non-ignored litter, then `env -u CARGO_REGISTRY_TOKEN cargo publish --locked`. An EXIT trap removes the worktree. It never passes `--allow-dirty`. |
| Belt | `.gitignore` | `.claude/agent-memory/` |
| Belt | `Makefile` `release-verify` | `cargo install pmat --locked --force` |

## One deviation: the `--detach` falsifier

The ticket says "dropping --detach → the zero-untracked-files assertion RED". That falsifier cannot
fire. `git worktree add <dir> <tag>` detaches HEAD at a tag with or without `--detach`, so removing
the flag changes nothing a test can observe. forjar found the same thing, and its
`scripts/publish-from-tag.sh` has a "MUTATION GUARD" note on it. The flag stays in the script.
The guard the audit pins instead is the one that matters: cargo runs inside a worktree of *this*
repository (arm 7: the shim's `git rev-parse --git-common-dir` is the repo's `.git`, and its
toplevel is not the main tree), and that worktree has no untracked or ignored file (arm 8).

## Evidence

`scripts/publish-from-tag-audit.sh` builds a throwaway repo with a bare origin and puts a shim
`cargo` first on PATH. The shim logs argv, git common dir, toplevel, `status --porcelain --ignored`
and whether `CARGO_REGISTRY_TOKEN` survived. `CARGO_REGISTRY_TOKEN=stale` is set in every run, and
`.claude/agent-memory/notes.md` is untracked in the main tree.

- 11/11 arms green. `--self-test` is RED against a no-op script.
- The ticket's two measurable falsifiers are arms 3 (non-existent tag, exit 2, cargo never called)
  and 6+8 (untracked file in the main tree, and the dry run is still clean from the worktree).

Mutation table. Each mutant is `sed` applied to the script, and each is killed by the named arms:

| Mutant | Red arms |
|---|---|
| M1 `git clone` + checkout instead of `git worktree add` | 7 |
| M2 drop `env -u CARGO_REGISTRY_TOKEN` | 9 |
| M3 never `cd` into the worktree (cargo runs in the main tree) | 7, 8 |
| M4 drop `--locked` | 6, 10 |
| M5 skip the origin/master ancestor check | 4 |
| M6 skip the version check | 5 |
| M7 no dry run | 6, 7, 8, 9, 10 |
| M8 keep the worktree | 11 |
| M9 add `--allow-dirty` | 10 |

A real dry run on this repository, `make publish-from-tag TAG=v3.41.1 DRY_RUN=1` (2026-09-27),
built the packaged crate in `$TMPDIR/pmat-publish-1UQRfO` in 1m 56s and ended:

```
   Uploading pmat v3.41.1 (/mnt/nvme-raid0/tmp/pmat1370/pubtmp/pmat-publish-1UQRfO)
warning: aborting upload due to dry run
dry run: pmat 3.41.1 packaged clean from v3.41.1; nothing published
```

The trap removed the worktree afterwards: `pubtmp/` is empty.
