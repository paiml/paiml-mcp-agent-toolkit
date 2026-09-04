# Implementation receipt — PMAT-656 (AD-04: Quorum Review Skill)

Spec: `docs/specifications/agentic-delivery-pmat.md` §3.1 / §5.2 / §9.4. Epic #1153.
Branch `PMAT-656-quorum-review`. Bundle side: `~/src/paiml-implement` branch `ad04-quorum-review`
(`skills/quorum-review/{SKILL.md,quorum-review.sh,pmat-merge}`, `install.sh`, `verify.sh`).

## What changed

- **Bundle** — `quorum-review.sh --base <ref> --ticket <id> [--pr n] [--width 1..10] [--out f]`: one prompt
  (diff, ticket, receipt, refutation doctrine) to N agy lanes (`--sandbox`, `agy/quorum-schema.json`);
  artifact `docs/audits/quorum-<ticket>.json` with `agreed`, judged `head`, per-lane verdict/summary/findings;
  raw lane outputs under `<artifact>.lanes/`. `pmat-merge <pr> --auto …` refuses (exit 1, names the file)
  without an agreeing artifact for the PR's current head; non-auto passes through. `verify.sh` gains
  `quorum-review-exec`. Without `agy` the script stops and says to run sequential Claude reviews.
- **pmat** — `scripts/quorum-review-audit.sh` (five offline legs through a stub `gh`; `--clean`/`--planted`
  read the live artifacts), `contracts/quorum-review-v1.yaml` (pv lint PASS), spec §9.4 note, this receipt.

## Verification (orchestrator runs; every number re-measured, none copied from a lane)

| check | result |
|---|---|
| RED — skill absent (the 3.36.0 state): `QUORUM_SKILL_DIR=/nonexistent bash scripts/quorum-review-audit.sh` | exit 1, `✗ helper present (missing: /nonexistent/pmat-merge)` |
| GREEN — installed skill, five offline legs | exit 0: no artifact → refused naming `docs/audits/quorum-<ticket>.json`, merge not called; other head / `agreed=false` → refused; agreeing artifact → `gh pr merge --auto` called; non-auto passthrough |
| named mutation M1 (helper ignores `agreed`) | RED on the other-head / agreed=false leg |
| named mutation M2 (helper ignores `head`) | RED on the same leg |
| verdict commit (paiml-implement#6): an artifact for the parent counts when the head commit only adds `docs/audits/quorum-*.json`; a head that also changes code is refused | both legs GREEN |
| named mutation M3 (helper accepts the parent unconditionally) | RED on the verdict-commit control |
| live helper against PR #1174 (armed, head 8815540e0), no artifact | exit 1, refused; auto-merge state unchanged (`merge` before and after) |
| live helper, fabricated agreeing artifact for that head | passed through to `gh pr merge --auto --merge` (idempotent on an already-armed PR); `agreed=false` variant refused |
| **planted contradiction** (branch `PMAT-655-planted`, 535e3ccc3: a test claiming the trailer is advisory under strict) | 3 lanes, 3 FAIL, not agreed; lane 3 names `commit_enforcement_tests.rs:149` — "asserts the opposite of the ticket … vacuous assertion" |
| **clean diff** (AD-03 branch e668cf180 against master) | 3 lanes, **3 FAIL — the reviewers were right**: `--strict` did not reach the pre-commit generator (`hook_generation.rs`, read only `[hooks] strict`), and `hooks verify --fix` / `hooks update` / the comply auto-install re-generated hooks with `strict=false`; plus a dead `PMAT_TASK_ID_PATTERN` export and a wrong test count in the receipt. All fixed on the AD-03 branch with two regression tests. The re-run after the fix is `docs/audits/quorum-PMAT-655-after-fix.json` here |
| first run of both controls | parser took the schema object agy echoes back for a verdict (`verdict={'type':'string',…}`); fixed — a verdict must be an enum string and findings a list; raw lanes now persisted |
| AD-04 quorum on this PR's head 445dbacb5 | 3 FAIL, cited: the roadmap carried PMAT-654/655 rows from the seed checkout (removed — 654 is on master, 655 rides #1175); the contract still said `a.head == pr.head` while the helper and leg 4c accept the parent for a docs-only verdict commit (contract now states the exception); F4 named `quorum-PMAT-655.json` where the file is `quorum-PMAT-655-after-fix.json` (fixed). The artifact that arms the PR is the next run |
| AD-04 quorum on head 0bdc937c8 | 3 FAIL, cited: the seed checkout's serializer had reordered PMAT-653 and normalised PMAT-654's fields, so master's own rows appeared in the diff (roadmap rebuilt as master plus this ticket's block: +17 lines, nothing else); one receipt sentence still named `quorum-PMAT-655.json` (fixed) |
| `pv lint contracts/quorum-review-v1.yaml` | PASS (0 errors, 0 warnings) |
| bashrs on the three scripts | 0 errors (SEC010/SEC011 findings resolved by validating the ticket id and removing bare-variable `rm -rf`) |

**Clean diff → three PASS, demonstrated after the fix.** The AD-03 branch was re-reviewed once the
four findings were fixed (scratch rebase onto the AD-02 branch so the diff is AD-03 alone; head
a7484a1cf): **3 lanes, 3 PASS, agreed** — `docs/audits/quorum-PMAT-655-after-fix.json`. The refuted
first run is kept as `docs/audits/quorum-PMAT-655-first-run-refuted.json` and the planted control as
`docs/audits/quorum-PMAT-655-planted-control.json`. A quorum that passes everything would have been
the vacuous outcome; this one refused a branch `pmat verify` and the author had both accepted, then
passed it once it was right.

## Gates

| gate | result |
|---|---|
| `pmat verify` (docs + scripts + contract; no Rust) | see PR body |
| `make gate-artifact` | see PR body |
| pv contract same PR | `contracts/quorum-review-v1.yaml` |
| named mutation RED | M1, M2, M3 above |

## Jidoka / scope
- The skill's own first run mis-parsed agy's output (fixed in the same change, both controls re-run).
- Two self-kills of the orchestrator's shell (`pkill -f` pattern matching a restart command in the same
  call) and one false "verify still running" reading — `pgrep -f 'verif[y] --format'` matched the agy lanes,
  whose argv carries the prompt text. Match on the binary path, never on a phrase the prompt may contain.

Verdict: **DONE** — bundle PR paiml/paiml-implement#5 merged (8bca212); this PR carries the acceptance, contract, receipt and the three lane artifacts.
