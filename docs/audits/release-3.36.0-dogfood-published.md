# Post-publish dog-food — pmat 3.36.0

| leg | result |
|---|---|
| P7 registry | `pmat 3.36.0` on crates.io, not yanked; size 8931399 bytes, created 2026-09-03T19:23:58.838498Z |
| P5 install | `cargo install pmat --version 3.36.0 --locked` into a throwaway root: executable present |
| --version | `pmat 3.36.0` — no commit line (CRUX-21: the crates.io build carries no git metadata) |
| --help | answered |
| release gate | `scripts/dogfood-use.sh` against the INSTALLED binary: 13 checks, 0 failure(s) |

Written by `scripts/dogfood-published.sh` (AD-02, docs/specifications/agentic-delivery-pmat.md section 3.6). The registry size and stamp pin the artifact; a local build cannot forge them.
