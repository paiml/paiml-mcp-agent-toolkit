# release 3.38.0 — post-publish dog-food of the crate crates.io serves

| field | value |
|---|---|
| tag | `v3.38.0` on master 9e494f95f (merge of PR #1197); GitHub release created with `gh release create --verify-tag` |
| publish | `env -u CARGO_REGISTRY_TOKEN cargo publish --locked` from the clean tree at 9e494f95f: "Published pmat v3.38.0 at registry `crates-io`", exit 0; preceded by `cargo publish --dry-run --locked` exit 0 (4948 files, 8.6 MiB compressed, packaged crate builds in 1 m 36 s) |
| crates.io | `max_stable_version=3.38.0`, `newest_version=3.38.0` |
| docs.rs | `status.json` for 3.38.0: `doc_status: true` |
| binary-release.yml | run 34016448458 success, 12 release assets (aarch64/x86_64 × darwin/linux-gnu/linux-musl, each with `.sha256`) |
| post-release.yml | run 34016448406 success: `MSRV verification` ✓, `Verify published release` ✓ |
| docker-publish.yml | failure, "Username and password required" — fails on every tag since v3.34.0 (v3.36.0, v3.37.0 identical); no Docker Hub secrets; pre-existing, not part of this release |

## Install from the registry

```
cargo install pmat --version 3.38.0 --locked --root /mnt/nvme-raid0/agent-wt/pmat-338-install
   Downloaded pmat v3.38.0 … Installed package `pmat v3.38.0` (executable `pmat`)   exit 0
```

## Fixture (one existing row `PMAT-010`), installed binary only

| step | exit | observed |
|---|---|---|
| `work validate` (clean) | 0 | — |
| `work add "first"` | 0 | `Created ticket: PMAT-011`, lock file `11` |
| `work add "second"` | 0 | `Created ticket: PMAT-012`, lock file `12` |
| `work add "third"` | 0 | roadmap ids `PMAT-010 PMAT-011 PMAT-012 PMAT-013` — distinct, sequential |
| `work validate` (after adds) | 0 | — |
| append a second `- id: PMAT-011` row, `work validate` | 1 | `duplicate id PMAT-011 at docs/roadmaps/roadmap.yaml:21, docs/roadmaps/roadmap.yaml:69` |
| `work add` on that duplicated roadmap | 0 | it parses, so the allocator accepts it and mints the next number — `add` does not enforce what `validate` rejects (follow-up) |
| append `status: bogus`, `work validate` | 1 | `roadmap.yaml:101:3: roadmap[6]: unknown status 'bogus' (did you mean 'todo'?)` |
| `work add` on the unparseable roadmap | 1 | refused, nothing written |
| `work validate --path /nonexistent` | 1 | — |
| `work validate --help` | — | carries the `Exit codes` paragraph |

The workstation's own `~/.cargo/bin/pmat` was then upgraded with `cargo install pmat --version 3.38.0 --locked --force` ("Replaced package `pmat v3.37.0` with `pmat v3.38.0`", exit 0).

## Two publish traps, for the next cut

1. `cargo publish` refused the tree with "10 files in the working directory contain changes that were not yet committed": the untracked, un-ignored `.claude/agent-memory/` (the paiml-implement subagents' memory). It was moved aside for the dry run and the publish and restored afterwards; `--allow-dirty` was not used. A `.gitignore` entry is the durable fix.
2. The PR Title Check wants a subject that starts with a letter (`chore(release): bump to 3.38.0 (…)`), not `3.38.0 — …`.
