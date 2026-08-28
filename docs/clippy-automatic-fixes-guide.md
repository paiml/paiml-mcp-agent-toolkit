# PMAT Clippy Fix Preview — Guide

> **This tool does not fix anything.** It previews clippy warnings that clear a
> confidence bar. It never modifies your source. If you want fixes applied, run
> `cargo clippy --fix`.
>
> The name of this page is kept for the links that point at it. The behaviour it
> describes is preview-only.

Every claim below was checked against the code it describes, at the revision that
closed [#1086](https://github.com/paiml/paiml-mcp-agent-toolkit/issues/1086). The
previous version of this page documented a command (`pmat fix clippy`), a
confidence model (0.0–1.0 scores from five weighted factors), a set of flags
(`--interactive`, `--category`, `--incremental`, `--explain`, …), an MCP tool
(`fix_clippy`), a config table (`[quality.gates.clippy]`) and a benchmark table
— **none of which exist**. It has been replaced rather than corrected.

## The command

```bash
pmat analyze clippy [--path DIR] [--confidence LEVEL] [--fix-codes CODES] [-o FILE]
```

`pmat fix clippy` does not exist — `pmat fix` is the Oracle PDCA loop, and
`pmat fix clippy` exits with `error: unrecognized subcommand`.

### Flags

| Flag | Default | Effect |
|------|---------|--------|
| `-p`, `--path <DIR>` | `.` | Directory to run `cargo clippy` in |
| `-c`, `--confidence <high\|medium\|low>` | `high` | Minimum confidence a lint must reach to be listed |
| `--fix-codes <A,B>` | *(none)* | Restrict the listing to these lint codes |
| `-o`, `--output <FILE>` | *(stdout)* | Write the JSON report to a file |
| `--dry-run` | — | **Accepted and ignored.** Preview is the only mode |
| `--perf` | — | **Accepted and ignored.** Bound as `perf: _perf` in the route and discarded |
| `--project-path <DIR>` | *(hidden)* | Deprecated alias for `--path` |

`--dry-run` is retained so existing invocations keep parsing. Both values produce
the same output.

## What it actually does

1. Runs `cargo clippy --message-format=json` in `--path`.
   If that command exits non-zero, `analyze clippy` returns an error instead of a
   report — so a directory without a `Cargo.toml`, or a crate that fails to
   compile, produces no listing at all. **The tool is Rust-only.**
2. Parses each `compiler-message` line into a diagnostic. Lines with no lint code
   or no primary source span (cargo's `compiler-artifact`, `build-finished` and
   `N warnings emitted` records) are rejected rather than counted.
3. Rates each diagnostic High / Medium / Low and drops everything below
   `--confidence`.
4. Prints the survivors. **No file is written**, except the report itself when
   `-o` is given.

There is no `fs::write` anywhere in `src/services/clippy_fix/`. The fix
transform (`ClippyFixEngine::apply_fix_internal`) builds a modified copy of the
source in memory, and nothing writes that copy anywhere. It is also not safe to
write: for `clippy::needless_return` it is `source.replace("return ", "")` over
the whole file, ignoring the span the diagnostic carries, so it strikes that
substring inside string literals and comments too.

Until #1086 the response nevertheless said `"action": "applied"` with a non-zero
`successful_fixes`, a `success_rate` of `100.0` and a named `fixed_files` list,
over a byte-identical tree. Those four keys are gone; `action` is now the
constant `"previewed"`.

## Confidence

Confidence is not a score. It is one of three levels, read from a **hardcoded
seven-entry table** (`ClippyFixEngine::init_confidence_rules`):

| Level | Lints |
|-------|-------|
| High | `clippy::needless_return`, `clippy::redundant_clone`, `clippy::unnecessary_wraps` |
| Medium | `clippy::manual_map`, `clippy::single_match` |
| Low | `clippy::needless_lifetimes`, `clippy::complex_lifetime` |

Any lint outside that table falls to `default_confidence`, which returns Medium
if the diagnostic carries a suggestion and Low otherwise. **In practice it is
always Low**: the JSON parser (`ClippyDiagnostic::parse_json_value`) sets
`suggestion: None` unconditionally and never reads cargo's suggested
replacement, so no parsed diagnostic has one.

The consequence is worth stating plainly, because it surprises everyone:

> At the default `--confidence high`, only the three High lints above can ever
> appear. Every other warning in your crate is filtered out.

This is why the response reports `diagnostics_found` and `diagnostics_eligible`
separately, and why a run that filters everything out says so instead of
reporting a clean crate. Use `--confidence low` to see the whole list.

## Configuration

**None is read.** `ClippyFixEngine::new()` takes no arguments and the confidence
table is a literal in the source. Nothing in the clippy path opens a config file,
so `pmat.toml [clippy]`, `.pmat/clippy-rules.yaml` and `[quality.gates.clippy]`
are all inert. (`pmat config` has no `set` subcommand either.)

To change which lints are eligible, edit `init_confidence_rules` in
`src/services/clippy_fix/clippy_fix_engine.rs`, or filter at the call site with
`--fix-codes`.

## MCP

`analyze clippy` is **not** exposed as an MCP tool. The function
`mcp_pmcp::tools::auto_clippy_fix` lives under the MCP tools directory, but it is
registered in no tool inventory; its only caller is the CLI route in
`src/cli/handlers/analysis_handlers/advanced_routes.rs`. There is no `fix_clippy`
tool to call from an MCP client.

## Output

```json
{
  "action": "previewed",
  "diagnostics_eligible": 0,
  "diagnostics_filtered_out": 76,
  "diagnostics_found": 76,
  "message": "⚠️ clippy reported 76 diagnostic(s), and none met the required confidence (High) — 76 left untouched. This is NOT a clean result; re-run with --confidence low to see them.",
  "min_confidence": "High",
  "results": {
    "preview_only": true,
    "previewed": [],
    "total_previewed": 0
  }
}
```

Key by key:

| Key | Meaning |
|-----|---------|
| `action` | Always `"previewed"`. Nothing else is possible; nothing is written |
| `diagnostics_found` | What `cargo clippy` reported, before any filtering |
| `diagnostics_eligible` | How many cleared `--confidence` and `--fix-codes` |
| `diagnostics_filtered_out` | The difference. A large value is not a clean crate |
| `results.previewed[]` | `file`, `line`, `code`, `message`, `confidence` per eligible diagnostic |

Removed in #1086, and deliberately absent rather than zeroed — a key that does
not exist cannot be misread: `successful_fixes`, `success_rate`, `fixed_files`,
`would_fix`, `dry_run`.

## Using it in CI

Treat the output as a report, not an action. A pipeline that ran
`pmat analyze clippy` and trusted `successful_fixes` would have reported a clean
auto-fix over an untouched tree — that is exactly what #1086 was.

```bash
# List every warning pmat can classify, and fail if any are eligible.
pmat analyze clippy --confidence low -o clippy-preview.json
jq -e '.diagnostics_eligible == 0' clippy-preview.json
```

To actually change code, use cargo:

```bash
cargo clippy --fix --allow-dirty
```

## Troubleshooting

**"It reports zero diagnostics but `cargo clippy` shows dozens."**
Check `diagnostics_found` against `diagnostics_eligible`. At the default
`--confidence high` only three lints are eligible; everything else is filtered.
Re-run with `--confidence low`.

**"It errors instead of printing a report."**
`cargo clippy` exited non-zero in `--path`. Run it there by hand: no
`Cargo.toml`, a compile error, or deny-level lints all produce this.

**"Nothing changed in my files."**
Nothing ever does. See the top of this page.

---

**Status**: preview only — see [#1086](https://github.com/paiml/paiml-mcp-agent-toolkit/issues/1086).
