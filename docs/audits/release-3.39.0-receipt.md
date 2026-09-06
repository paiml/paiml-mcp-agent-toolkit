# release receipt — 3.39.0 (PUBLISH-READY; Noah publishes)

| field | value |
|---|---|
| master at cut | `62562ccd9` = merge of PR #1204 (release cut, head 7e4bb4743: check-runs success=44 skipped=5 failure=0, reruns=0) |
| tag | `v3.39.0` → `62562ccd9` (annotated, tag object ffff99022) |
| GitHub prerelease | created by hand with `gh release create --prerelease --verify-tag --notes-file <CHANGELOG [3.39.0]>`; `binary-release.yml` (run 34038788621) and `post-release.yml` (run 34038788628) fired on `release: published`; `release.yml` ran on the tag push (run 34038786778) and will fail at the fleet lint-gate (PMAT-687) — a red gate creates no release, and the release already existed by hand |
| publish dry run | detached worktree of `v3.39.0` (`git status --porcelain` empty): `env -u CARGO_REGISTRY_TOKEN cargo publish --dry-run --locked` → `Packaged 4970 files, 43.1MiB (8.6MiB compressed)`, verified build 2 m 11 s, `aborting upload due to dry run`, **exit 0** |
| publish | **NOT DONE — stops here by direction.** Noah: `WT=$(mktemp -d)/wt && git worktree add --detach "$WT" v3.39.0 && (cd "$WT" && env -u CARGO_REGISTRY_TOKEN cargo publish --locked) && git worktree remove "$WT"`; then `gh release edit v3.39.0 --prerelease=false`; then merge paiml/infra#458 and `forjar apply -f machines/<host>/forjar.yaml` on lambda-labs, intel, mini, gx10 |
| version string for aprender `scripts/pmat_bin.sh` | **3.39.0** |
| local install | `~/.cargo/bin/pmat` = 3.39.0 built from the release branch (7e4bb4743) — the aprender C0-3 session can use it now |
| infra pin PR | paiml/infra#458 — lambda-labs 3.37.0→3.39.0, intel 3.37.0→3.39.0, mini 3.31.0→3.39.0, gx10 `nightly`→`v3.39.0`; merge after crates.io serves 3.39.0 |

## What 3.39.0 carries (all merged on the required checks)
| PR | tickets | merged head check-runs | reruns |
|---|---|---|---|
| #1181 AD-09 docs | PMAT-663 | 803d0547f: success=43 skipped=5 | 1 (`ci / coverage` flake, #1202 class 1) |
| #1203 release path | PMAT-675 | 0f7a7be43: success=40+; coverage rerun | 1 (`ci / coverage` flake, #1202 class 2) |
| #1201 urgent | PMAT-685 (#1200), PMAT-676, PMAT-679, PMAT-680 | be3f18c8d: success=43 skipped=5 | 0 |
| #1204 release cut | PMAT-682, PMAT-686 | 7e4bb4743: success=44 skipped=5 | 0 |
| #1206 fixups (behind the tag) | PMAT-682/675 | in CI at receipt time | — |

## Gates on the tagged sha (62562ccd9 / its branch head)
| gate | command | result |
|---|---|---|
| lib tests (touched suites) | `cargo test --lib -- work_add_ work_validate_duplicate roadmap_text roadmap_service roadmap_id_authority work_cot tests_macs_derivation cot_derive release_workflow_tests` | 89 passed [V] |
| metrics ratchet | `the_committed_ratchet_holds_at_head` | ok; unwrap 20336 / panic 781 = baselines [V] |
| `pmat verify --skip tests` | every commit | ok [V] |
| validate-book | `make validate-book` | PASS, 4 chapters, controls OK; wall 0.7 s — an already-built pmat was exercised, not a fresh build [V] |
| clean-room (local mode A: detached worktree, CARGO_BUILD_JOBS=2) | `cargo package --locked` (verify from the tarball) | exit 0, 4 m 10 s [V] |
| clean-room doctests | `cargo test --doc --locked -- --test-threads=4` | 286 passed, 0 failed, 58 ignored [V] |
| fleet clean-room gate (unified-gate) | probe run 34034967876 on v3.38.0 | `create-release` ✓, `gate / lint-gate` ✗ "Banned path scan" on pmat's own analyzer fixtures — **the fleet gate cannot pass pmat's tree until PMAT-687**; `clean_room=local-modeA` for this release |
| `cargo publish --dry-run --locked` (detached worktree of the tag) | see above | exit 0 [V] |

## Fixture table — locally installed 3.39.0 (crates.io install is Noah's post-publish step)
| step | result |
|---|---|
| `work validate` clean | exit 0 |
| `work add` ×2 | exit 0, ids PMAT-012, PMAT-013; **untouched entries byte-identical** (new file starts with the old bytes; a comment, an unknown key and a flow-style row untouched) |
| id authority | lock + high-water mark at `.git/pmat/roadmap-id.lock` (reads 13); no `roadmap.yaml.lock` sibling |
| two worktrees `work add` | wt2 → PMAT-014, wt1 → PMAT-015 (distinct) |
| duplicated id | `work validate` exit 1; `work add` refused: `duplicate id PMAT-011 at …/roadmap.yaml:9, …/roadmap.yaml:58` |
| unparseable | `work validate` 1, `work add` 1 (nothing written) |
| `work cot derive` on the #1200 step | exit 0, `statement:` = the claim; hollow step → exit 1, nothing written |

## Mutation controls (all observed RED then restored; orchestrator re-ran each)
cot fallback removed → 3 fallback tests + the ten-contract fixture test RED, **in CI** run 34023329875; validator call removed from add → refuse test RED; add routed through the whole-file writer → append test RED; every-ref term dropped → other-refs test RED; `continue-on-error` planted on verify → workflow test RED.

## Dispositions
| item | disposition |
|---|---|
| #1181 | merged |
| #1201, #1203, #1204 | merged (this release) |
| #1206 | open, auto-merge armed (fixups behind the tag) |
| #1179 (PMAT-662 docs) | open — master-merge prepared in its worktree (e2fe300b6), pushed after #1206/#1207 merge; quorum then merge |
| #1180 (PMAT-661), #1184 (PMAT-666), #1177 (PMAT-657) | **closed** with five-whys lines in their bodies (branches kept; revive under PMAT-684 in 3.40.0) — HRQ rows: human-account PRs closed by the run |
| #1200 | complete — fixed by #1201 (auto-closes) |
| #1198 (bot: release-check on 3.38.0) | complete — 3.38.0 shipped; the bot's own check passes on master |
| #1202 (coverage flakes) | defer(3.40.0) — two classes recorded, one rerun each on #1181 and #1203 |
| 47 open issues + 57 non-completed tickets | dispositioned in `docs/audits/dispositions-3.39.0.json` (worker af560bd751fd4507c, sonnet; every `complete` cites a merged PR/commit whose ancestry the orchestrator re-checked for the 11 PRs named): complete: 19 | defer(3.40.0): 47 | defer(3.41.0): 35 | reject: 3. `complete` tickets marked completed in the roadmap through the release binary. HRQ (21 human-authored rows a human closes): #1029, #1035, #1153, #1159, #1169, #1193, #1200, PMAT-502, PMAT-630, PMAT-635, PMAT-642, PMAT-645, PMAT-648, PMAT-652, PMAT-655, PMAT-660, PMAT-664, PMAT-656, PMAT-665, PMAT-672, PMAT-682 |
| rejected | #1035 (Superseded: this is the 12-repo stack-audit root-cause umbrella whose …); #1153 (Superseded: the 25 verified defects this tracking issue references are…); PMAT-502 (Roadmap status is already 'cancelled'; no further action needed.…) |
| PMAT-677 (DS), PMAT-678 (PP), PMAT-681 (GD), PMAT-683 (MB), PMAT-684 (RV), PMAT-687 | deferred(3.40.0) — the tag was cut on the URGENT #1200 pivot before DS/PP/GD; PP's `publish-from-tag` recipe is the inline command above |

## Budget
`K=240`, `K̂=302 [C]` (basis L11-L12); orchestrator turns ≈ 232 at receipt time (andon threshold 192 crossed during the URGENT pivot; the operator's second brief reset the scope); subagents: peak 1 live, 5 workers (2 resumed once), 5 delegates; denials 2 (hook: gh while a slot was held); `reruns` 2 (both #1202 flakes, on docs/workflow PRs; 0 on the release PRs).

verdict: PUBLISH-READY — nothing published; tag and prerelease in place; Noah publishes from a detached worktree of `v3.39.0`.

RELEASE-3.39.0-RECEIPT-END
