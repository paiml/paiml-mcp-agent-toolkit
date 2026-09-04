# Release receipt — pmat 3.36.0 (the first publication since 3.34.0)

| leg | result |
|---|---|
| merge | PR #1171 → master `d2aede8ea` (CRUX-04 already on master; #1159 fix, AD-01 and the agentic-delivery spec cherry-picked into the release train — #1167/#1170/#1168 closed as superseded) |
| tag / GitHub release | `v3.36.0` on `d2aede8ea`; release id 382267859; `binary-release.yml` completed/success, 12 assets (6 targets × tar.gz + sha256) |
| pre-publish gates (release head 1158ae48e) | `pmat verify`: exit 0 (format, satd, clippy, tests measured; complexity withdrawn on the clean tree) · `make validate-book`: PASS with the falsification control armed (Ch05/07/13/14) · `scripts/dogfood-use.sh` with the rc binary: 13 checks, 0 failures · `cargo package`: 8.93 MB (< 10 MB) |
| packaged-artifact probe (tagged, clean checkout) | self-test fired 5 probes first; P1–P7 all PASS: clean, tagged, no `--allow-dirty`/`--no-verify` publish, recorded sha `d2aede8ea0a9` exists, README ships, artifact builds standalone, downstream crate links it, `cargo install` from the artifact works, README examples exist, registry had no 3.36.0 yet |
| publish | `env -u CARGO_REGISTRY_TOKEN cargo publish --locked` from the tagged checkout → `Published pmat v3.36.0`; crates.io: 3.36.0 yanked=false size=8931399 created=2026-09-03T19:23:58 |
| **post-publish dog-food of the bytes crates.io serves** | `cargo install pmat --version 3.36.0 --locked` into a throwaway root → `pmat 3.36.0` · `scripts/dogfood-use.sh` against the INSTALLED binary: **13 checks, 0 failure(s)** |
| installed `--version` | `commit: unknown`, `worktree: unknown` — the crates.io build has no git metadata (CRUX-21, spec §8.21, still open); the receipt therefore pins the artifact by the registry's size/created stamp above, not by a commit line |
| AD-01 release-check on master | RED on the merge push at 19:01Z (before the tag): opened #1172 · GREEN on the dispatch after tag + release + crate existed (run 33796369689) · local `scripts/release-check.sh` on master: `release-check: 3.36.0 is tagged, released and on crates.io` |
| docs.rs | `doc_status=true` for 3.36.0 (built within ~25 min of publish; HTTP 200) |

The post-publish dog-food above was run by hand; the scripted form (AD-02, `scripts/dogfood-published.sh 3.36.0`) wrote [`release-3.36.0-dogfood-published.md`](release-3.36.0-dogfood-published.md) with the same result (13 checks, 0 failures) and pins the artifact by the registry's size and stamp.

Not checked: the Docker channel (#1122 — publishes from a different version; not part of this release's gates). The post-publish steps were run by hand from `/mnt/nvme-raid0/agent-wt/pmat-release/publish-3.36.0.sh`; AD-02 (`scripts/dogfood-published.sh`) is the next item and turns this receipt into a script's output.
