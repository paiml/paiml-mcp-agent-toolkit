# release 3.39.0 — post-publish dog-food of the crate crates.io serves

| field | value |
|---|---|
| tag / master | `v3.39.0` → `62562ccd9`; post-cut fixups merged as #1206 (8b6bf64de) behind the tag |
| publish | `env -u CARGO_REGISTRY_TOKEN cargo publish --locked` from a detached worktree of `v3.39.0` (`git status --porcelain --ignored` empty): `Packaged 4970 files, 43.1MiB (8.6MiB compressed)` … `Published pmat v3.39.0 at registry crates-io`, exit 0; dry run exit 0 beforehand |
| crates.io | `max_stable_version=3.39.0`, `newest_version=3.39.0` |
| AD-01 `make release-check` | exit 0: `3.39.0 is tagged, released and on crates.io` |
| AD-02 `make dogfood-published VERSION=3.39.0` | exit 0, `GO: pmat 3.39.0 — 13 checks, 0 failure(s)`, wall 210 s; receipt `docs/audits/release-3.39.0-dogfood-published.md` |
| docs.rs | `status.json` for 3.39.0: `doc_status: true` (polled, not rerun) |
| GitHub release | prerelease created by hand (fleet gate cannot pass pmat's tree until PMAT-687), 12 assets from `binary-release.yml` (run 34038788621 success), `post-release.yml` success (run 34038788628); promoted with `gh release edit v3.39.0 --prerelease=false` |
| infra | paiml/infra#458 merged (eb223af90): lambda-labs 3.37.0→3.39.0, intel 3.37.0→3.39.0, mini 3.31.0→3.39.0, gx10 `nightly`→`v3.39.0`. On this host `forjar apply -f machines/lambda-labs/forjar.yaml` would apply **141 changes (123 create, 18 destroy)** — the machine's state has drifted far beyond the pin — so only `-r stack-tool-pmat` was applied here; the full apply is an operator decision. `forjar drift` reported "No drift detected" over **0 resources inspected** — a vacuous check, recorded. |
| flag-efficacy comparison | `make gate-flag-efficacy-full` on the v3.38.0 tag in a detached worktree: identical summary and identical 18-flag no-op list — the Phase-3 red predates 3.39.0 (PMAT-688) |

## Install from the registry (A3/A4)

```
cargo install pmat --version 3.39.0 --locked --root /mnt/nvme-raid0/agent-wt/pmat-scratch-339
   Downloaded pmat v3.39.0 … Installed package `pmat v3.39.0` (executable `pmat`)   exit 0
```

## Fixture table — the crates.io-installed binary only (temp git repo, one existing row `PMAT-010` + a comment, an unknown key and a flow-style row)

| step | exit | observed |
|---|---|---|
| `work validate` (clean) | 0 | — |
| `work add` ×2 | 0, 0 | ids `PMAT-012`, `PMAT-013`; **untouched bytes identical** (the new file starts with the old bytes) |
| id authority | — | `.git/pmat/roadmap-id.lock` reads `13`; no `roadmap.yaml.lock` sibling |
| two worktrees `work add` | 0, 0 | `PMAT-015` (wt2) and `PMAT-014` (wt1) — distinct |
| append a second `- id: PMAT-011`; `work validate` | 1 | — |
| `work add` on that roadmap (WA) | 1 | refused, nothing written |
| append `status: bogus`; `work validate` / `work add` | 1 / 1 | — |
| `work validate --path /nonexistent` | 1 | — |
| `work cot derive` on the #1200 step | 0 | `statement: "apr compare-hf --offline hangs — must not occur after fix"` |
| `work cot derive` on a hollow step | 1 | no artifact written |
| `work validate --help` | — | carries the `Exit codes` paragraph |

Deferred behaviour rows: DS (skill listed once as `pmat-dogfood`) — not in 3.39.0 (PMAT-677, 3.40.0).
