# Implementation receipt — GH-1473

Cop ruling (pmat-cop, 2026-10-31): the refusal option (issue's second option) is accepted; sections are not implemented.

## Measured
- Reproduced on installed pmat 3.42.0: `pmat report -p . --md` with `--include-executive-summary` or `--include-recommendations` -> rc=0, body identical to base (18 lines each; only "Analysis Duration" differs). `--include-visualizations` -> rc=1.
- `cargo test --lib enhanced_reporting_handlers`: 11 passed, 0 failed (includes new `test_generate_report_refuses_the_section_flags`).
- `cargo test --lib report`: 821 passed, 0 failed, 2 ignored.
- `pmat verify --skip tests`: format, complexity, satd, clippy all ok.

## Round 2 (quorum round 1 NOT AGREED: 2 FAIL)
Both lanes were right: the clap args were `default_value_t = true`, so both flags were true on every plain `pmat report` and the refusal would have fired on every run. That is also why the reproduction above saw identical bodies: the base already had both true. Fix at the cause: `src/cli/commands/commands_enum/definition.rs` drops the two `default_value_t = true`, so the flags are false unless typed. New test `section_flags_are_false_unless_typed` parses `pmat report` through `Cli::try_parse_from` and asserts (false,false) with no flags, (true,true) when typed; it fails on the old default. Not measured locally (the lib test build never got a cargo-memcap slot); CI is the proof. CI's first run SIGABRTed this test on stack overflow (clap derive parse), so round 3 runs the parse inside `on_big_stack`.

## Round 3 (quorum round 2 NOT AGREED: 1 FAIL)
The `--include-recommendations` help said "always emitted", contradicting the error text (the report has no recommendations section); help corrected. The handler rustdoc still recommended the two flags in CLI examples and a `true,true` call; those examples and the parameter docs now say the flags are refused.

## Scope
Two source files: `src/cli/handlers/enhanced_reporting_handlers.rs` (new `reject_unimplemented_sections`, called beside `reject_unimplemented_visualizations`, plus two tests) and `src/cli/commands/commands_enum/definition.rs` (two clap defaults removed, help text says not implemented). No gate content touched.
