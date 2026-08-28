# CB-2104 self-test fixture

`pmat comply numeric-claims` runs itself against this corpus **before** it scans
the real one, on every invocation. If it does not recover 4/4 of the planted
defects with 0 false positives over the innocent half, the run is UNMEASURABLE
(exit 2) and the real result is not printed at all.

That is the control the check would otherwise lack: a rule that has silently
stopped firing and a repository that is genuinely clean produce the same empty
output, and only a corpus that MUST fire can separate them.

The files are `include_str!`-ed into the binary by
`src/services/numeric_claims/census.rs`, so the fixture travels with the
executable and works when scanning any repository, not only this one. They are
also tracked here so the planted defects are reviewable as source. Nothing
scans them from disk at run time.

## The four planted defects — one per rule family

| # | rule | file | what is planted |
|---|---|---|---|
| 1 | C1 SELF-BREACH | `planted/.pmat-metrics.toml:41` | `max_unwrap_calls = 100  # Current: 570` — a ceiling whose own comment reports a 5.7x breach. Verbatim from this repository at `2f15cab92`. |
| 2 | C5 NAMED CROSS-REFERENCE | `planted/binary_size.rs:7` | `50 * 1024 * 1024` (= 52,428,800) annotated "aligned with .pmat-metrics.toml binary_max_bytes", which is 50,000,000. |
| 3 | C4 UNJUSTIFIED DIVERGENCE | `planted/codecov.yml:6` | `threshold: 95%` where both sibling `codecov.yml` files say `2%` — a 47.5x divergence with no stated reason. |
| 4 | R1 REPLICATED DIVERGENT CLAIM | `planted/crate-*/README.md:3` | one sentence template across seven files, five saying `70 workspace crates` and two saying `75`. |

## The innocent half — 36 numbers that must never fire

`innocent/site{0..7}.{md,toml,yaml,rs}` carry the 26 hand-planted innocent
numbers (HTTP statuses, ports, a seed, a year, an edition, an arXiv id, a
section heading, a table row, a past-state record, a policy target, …). Each
class is replicated across **eight** files with a 6/2 value split — the exact
shape R1 hunts — so silence here is a property of the framing rules and not of
a quiet corpus. An innocent control that could not fire even if the rules
rotted would measure nothing.

`innocent/derivations.rs` carries the ten hand-audited correct derivations,
where the annotation computes the value beside it out of parts.

## Editing this fixture

Changing these bytes changes what the check proves about itself. Each planted
defect is pinned by rule, path prefix and quantity in the `PLANTED` table in
`src/services/numeric_claims/census.rs` — not by line number, so moving a line
within a fixture file is safe and deleting the defect is not. The line numbers
in the table above are documentation and were checked by hand, not by a test.

A fixture edit that removes a defect fails
`census_tests.rs::the_self_test_recovers_all_four_planted_defects` rather than
quietly weakening every future run, and
`census_tests.rs::removing_a_planted_defect_fails_the_self_test` ablates each
one in turn to prove the self-test can still go red at all.
