# Implementation receipt — GH-1473

Cop ruling (pmat-cop, 2026-10-31): the refusal option (issue's second option) is accepted; sections are not implemented.

## Measured
- Reproduced on installed pmat 3.42.0: `pmat report -p . --md` with `--include-executive-summary` or `--include-recommendations` -> rc=0, body identical to base (18 lines each; only "Analysis Duration" differs). `--include-visualizations` -> rc=1.
- `cargo test --lib enhanced_reporting_handlers`: 11 passed, 0 failed (includes new `test_generate_report_refuses_the_section_flags`).
- `cargo test --lib report`: 821 passed, 0 failed, 2 ignored.
- `pmat verify --skip tests`: format, complexity, satd, clippy all ok.

## Scope
One source file, `src/cli/handlers/enhanced_reporting_handlers.rs`: new `reject_unimplemented_sections`, called beside `reject_unimplemented_visualizations`, plus one test. No gate content touched.
