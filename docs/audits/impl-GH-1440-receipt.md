# GH-1440 — FLOW-03: `pmat work add` refuses an untriaged ticket (receipt)

## The ticket, verbatim (issue #1440 body; the roadmap row carries only the title)

> FLOW-03 ⊇ #1370. #1370 (owner: pmat1370) already covers fragment-only write + `roadmap sync --check`. This issue is ONLY the delta:
>
> 1. `pmat work add` refuses without `--epic <issue#>` (an open issue labelled `epic` in the same repo) and `--priority P0|P1|P2|P3` — defects and P0 are never refused for lack of a milestone (spec §0.4).
> 2. On success it creates the GitHub sub-issue link (epic → new ticket) and writes `epic:` + the priority into the fragment; the two must agree (FR-3 / F-3b).
> 3. The fragment carries `kind:`.
>
> Falsifiable claim: today `pmat work add "x" --github-issue N` succeeds with no epic and no P-label (measured in paiml/infra 2026-09-25, PMAT-1115). DoD: contract `pmat-work-add-triaged-v1`; RED→GREEN falsifier; release + forjar pin fleet-wide. Depends on #1370 landing first.

## How each clause is met

| Clause | Where |
|---|---|
| 1. refuses without `--epic` (open, labelled `epic`, same repo) | `gate` + `check_epic`, `src/cli/handlers/work_handlers/work_add_triage.rs`; audit arms 4, 5, 6 |
| 1. refuses without `--priority P0..P3` | `gate`; P0..P3 are aliases on `WorkPriority`; audit arm 2; unit test `work_add_triaged_priority_accepts_p0_to_p3` |
| 1. no milestone is ever asked for | no milestone check exists; audit arm 10 (P0, no milestone, accepted) |
| 2. creates the sub-issue link | `ensure_linked` runs BEFORE the row is written; audit arm 8 (exactly one POST, child's REST id); arm 9 (idempotent); arm 11 (failed link ⇒ roadmap byte-identical) |
| 2. writes `epic:` + priority, which agree with the link | `epic: E` is written only after GitHub holds E → child; arms 8, 12 |
| 3. the fragment carries `kind:` | `--kind` or a `kind:<x>` tag is required, since a `kind:` cannot be carried unless one is known; `tags_with_kind` adds it once; arms 3, 10, 12 |
| contract `pmat-work-add-triaged-v1` | `contracts/pmat-work-add-triaged-v1.yaml`, bound in `contracts/binding.yaml` |
| RED→GREEN falsifier | `scripts/work-add-triaged-audit.sh` |

## Evidence (measured)

- `scripts/work-add-triaged-audit.sh`:
  - on this branch's build: 13/13 arms green;
  - on the released pmat 3.41.1: RED;
  - `--self-test` against a no-op binary: RED.
- `cargo test --lib work` reports 1308 passed, 0 failed, 3 ignored. That count includes 16 new `work_add_triaged_*` tests, and each refusal test asserts zero GitHub writes.
- "Release + forjar pin fleet-wide" is out of this PR's scope. It follows the next pmat release.
