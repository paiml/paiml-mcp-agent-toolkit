# impl receipt — PMAT-651 (AD-01: a merged release must have become a release)

| field | value |
|---|---|
| ticket | PMAT-651 (agentic-delivery-pmat.md §3.5 / §9.1; motivating incident: 3.35.0 merged 2026-09-02, never published) |
| branch | PMAT-651-release-check (PR #1170, superseded — its commit dbf6bce66 rides the 3.36.0 release PR #1171 as d706db92a) |
| phase gate | `pmat verify --format json` on the branch: exit 0 (format, satd, clippy, tests measured; complexity withdrawn — no Rust changed) |
| DoD gate | the release train's CI on #1171 (required checks) |
| quorum | never (`--quorum never` per the invocation) |
| subagents | 0 |

## Verification (orchestrator reruns)

| check | result |
|---|---|
| `bash scripts/release-check.sh` on master de234a4c2 | **exit 1** — `Cargo.toml says 3.35.0 but no tag v3.35.0` (the incident, as a red gate) |
| `bash scripts/release-check.sh --self-test` | **SELF-TEST PASSED**: a 9.9.9 fixture fails the tag leg, then — with a planted tag — the release leg |
| `bashrs lint scripts/release-check.sh` | 0 errors (a `rm -rf` on a variable path replaced by a guarded cleanup after SEC011) |
| `quality-gate.yml` parses; jobs `score, provable-ladder, release-check`; `release-check` has `if: github.event_name != 'pull_request'`, GitHub-hosted runner, daily schedule | measured with a YAML parse |
| `make release-check` | runs the script |
| the job on master after 3.36.0 is tagged, released and published | must go GREEN — recorded in `docs/audits/release-3.36.0-dogfood.md` |

## Named mutation
A check that hardcodes `3.35.0` passes the incident and fails the self-test's 9.9.9 control (the version is read from `Cargo.toml`). A stray tag `vv2.88.0` in the repository sorted above every number under `sort -V`; the script filters to numeric tags — without the filter the "latest tag" was 2.88.0 and the check compared against the wrong baseline.

## Decisions taken as the conservative option ([A])
- [A] Not a required PR check: a PR is not a release, and a merge must not wait on the registry or the runner fleet.
- [A] Three channels in the order tag → GitHub release → crates.io; the first missing one is named; Docker (#1122) is a later fourth leg.

## Verdict
DONE when #1171 merges and the job is green on the published 3.36.0 (see the release receipt).
