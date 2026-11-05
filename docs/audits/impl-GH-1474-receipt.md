# Implementation receipt — GH-1474

Choice: the issue's "alias it explicitly, and say so in `--help`" option, not a refusal. `pmat context --format llm-optimized` is the documented invocation (repo CLAUDE.md, the `cli_struct.rs` help example, the readme-validate docs), so refusing it would break documented commands.

## Measured
- The defect is the issue's own measurement on pmat 3.42.0: `pmat context -q --format llm-optimized` is byte-identical to the markdown output. The code agrees: `context_generation.rs` routes `Markdown | LlmOptimized` to the same `generate_markdown_context`.
- New test `context_format_alias_tests::llm_optimized_is_documented_as_a_markdown_alias` asserts the value's `--help` text says "alias of markdown"; the old enum variant had no help text, so it fails on old code. NOT run locally: the lib test build never got a cargo-memcap slot, so CI is the proof.
- The unrun-tests ledger line moved for the one new lib test (24407/27534 -> 24408/27535 executed; unrun 3127 unchanged), regenerated with `pmat analyze unrun-tests --executed '' --write-ledger`. No `.unwrap()` added.

## Scope
`src/cli/enums/output_formats.rs` (help text on the variant, one test), a comment in `src/cli/handlers/utility_handlers/context_generation.rs`, the regenerated ledger line, this receipt. No behaviour change: output bytes are unchanged. No gate content touched.
