# impl receipt — PMAT-687 [train/FG]: fleet gate admissibility for a pmat tag

**Ticket:** PMAT-687 (kind:code, carried from 3.39.0) · **Branch:** `PMAT-687-fleet-gate` · **Orchestrator:** Fable (direct; workflow edits are orchestrator-only). Paired change in the fleet: paiml/.github#65.

## Mechanism (five-whys terminal)
`release.yml` could never go green on a pmat tag: `gate / lint-gate`'s "Banned path scan" greps every tracked `*.rs *.toml *.sh` for four workstation prefixes, and pmat's own banned-path analyzer (`src/services/hardcoded_paths.rs`) names those strings in its recognisers and test fixtures (14 lines), plus one comment in `check.rs` quoted an absolute checkout path. PMAT-686 scrubbed 17 other files at 3.39.0 and pinned these two as debt because editing the analyzer trips `pmat verify`'s complexity gate on its pre-existing `classify` (cognitive 33 > 25). So 3.39.0 shipped `clean_room=local-modeA` with a prerelease created by hand — a producer gating itself.

## Fix (after the quorum)
The first cut excluded the analyzer's file fleet-side (paiml/.github#65). The 3-lane quorum returned **do-not-merge** on that diff (lanes 4faaed6e-1f62-4323-902c-ce0b8db0c802, 963b24c9-66bc-4eeb-a3b2-c676d7fdb37c; one lane unparsed) with two findings the orchestrator verified: (1) `grep -vE "$EXCLUDE"` in the scan runs on `path:line:content`, so an unanchored fragment masks any line anywhere whose *content* contains it — an exclusion blinds more than one file; (2) the 14 hits were comments and `#[cfg(test)]` fixtures, so the honest fix the ticket named first (pay the complexity debt, rebuild the fixtures) was available. #65 was closed with that five-whys and the masking class was reported to the fleet as a follow-up.

The landed fix: `classify` split into four helpers and `candidates` / `feed` / `braces_outside_literals` refactored below the cognitive-25 gate with no behaviour change (the file's 23 tests are the contract and pass unchanged; `pmat analyze complexity` errors `[]`, was 4); the fixtures name `/home/alice` (any named user is machine-specific to the analyzer — the fixture never needed this workstation) and the doc comments say `<user>`; `check.rs:693`'s comment scrubbed; the mirror test excludes nothing, pins nothing, and reads bytes lossily like `git grep` (a non-UTF-8 file is scanned, not skipped). The fleet's own grep, reproduced locally with `unified-gate.yml`'s EXCLUDE, finds 0 lines for each of the four prefixes.

## RED → GREEN
- RED 039217d74: mirror test with the pins removed fails on exactly one line: `check.rs:693` (the redacted literal). Discrimination: it did not name the analyzer file's 14 lines because that commit still modelled an exclusion; the final mirror models none and is green because the literals are gone, not hidden.
- GREEN 75e2e128e (scrub) + the refactor commit: `cargo test --lib -- hardcoded_paths fleet_banned` → 23 passed; `TMPDIR=/tmp/tmpx pmat verify --skip tests --format json` → ok=true (format, complexity, satd, clippy all measured true).
- Anti-vacuity: plant a banned prefix in any tracked `src/*.rs` and the mirror names it (the RED commit is that observation on check.rs).

## Workflow probe (PO-G3)
A probe on v3.39.0 cannot show this (that tree still carries the literals — quorum lane 1). The falsifier is the first tag carrying this change: on `v3.40.0`, `release.yml`'s `gate / lint-gate` must pass the banned-path scan and the run must reach `verify`. Recorded in the 3.40.0 release receipt (gate table, `clean_room=`).

Orchestrator turns: 11 (incl. the quorum, the closed fleet PR, and one worker cut off by the API limit whose green tree the orchestrator committed)

verdict: GREEN

IMPL-PMAT-687-RECEIPT-END
