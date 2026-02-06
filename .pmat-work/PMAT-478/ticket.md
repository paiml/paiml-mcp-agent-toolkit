# PMAT-478: pmat query Feature Parity with grep/rg/find/mdfind + Deeper Claude Integration

## Priority: High
## Status: Open
## Created: 2026-02-06

## Problem Statement

`pmat query` is the recommended code search tool per CLAUDE.md and project policy, but it lacks
several features that `grep`, `rg`, `find`, and `mdfind` (macOS) provide. This causes developers
and AI agents to fall back to raw grep/rg, bypassing quality-annotated search results.

Additionally, Claude Code sub-agents frequently ignore CLAUDE.md instructions to use `pmat query`
and default to grep/rg/find, undermining the sovereign search policy.

## Feature Parity Requirements

### Search Features (grep/rg parity)

1. **Negative search / exclusion patterns**
   - `pmat query "handler" --exclude "test"` — exclude results matching a pattern
   - `pmat query "cache" --exclude-file "*.test.rs"` — exclude specific file patterns
   - Equivalent to `grep -v` / `rg --invert-match`

2. **Regex search mode**
   - `pmat query --regex "fn\s+handle_\w+"` — full regex pattern matching
   - Support PCRE2 or Rust regex syntax
   - Equivalent to `grep -E` / `rg -e`

3. **Literal string search**
   - `pmat query --literal "unwrap()"` — exact string match, no semantic ranking
   - Useful for finding specific error messages, constants, etc.
   - Equivalent to `grep -F` / `rg -F`

4. **Case sensitivity control**
   - `pmat query "Error" --case-sensitive` (default: smart-case like rg)
   - `pmat query "error" --ignore-case`
   - Equivalent to `grep -i` / `rg -i` / `rg -S`

### File Discovery Features (find/mdfind parity)

5. **File glob search**
   - `pmat query --files "**/*.rs"` — find files by pattern
   - `pmat query --files "src/services/*.rs" --sort size` — sort by size, date, etc.
   - Equivalent to `find . -name "*.rs"` / `mdfind -name "*.rs"`

6. **File metadata search**
   - `pmat query --files --modified-after 2026-01-01` — find recently modified files
   - `pmat query --files --min-size 1000` — find large files
   - Equivalent to `find . -newer` / `mdfind "kMDItemContentModificationDate > ..."`

7. **Content + file combined search**
   - `pmat query "error" --in-files "src/services/**"` — search within file scope
   - `pmat query "handler" --file-type rust` — filter by language
   - Equivalent to `rg --glob` / `rg --type`

### Output Features

8. **Context lines**
   - `pmat query "panic" -A 3 -B 2` — show context around matches
   - Equivalent to `grep -A -B -C`

9. **Count mode**
   - `pmat query "unwrap" --count` — count matches per file
   - Equivalent to `grep -c` / `rg -c`

10. **Files-only mode**
    - `pmat query "handler" --files-with-matches` — show only matching file paths
    - Equivalent to `grep -l` / `rg -l`

## Claude Integration Improvements

### Problem: Sub-agents ignore CLAUDE.md policy

Claude Code sub-agents (Task tool agents) frequently:
- Use `grep -r` or `rg` instead of `pmat query`
- Use `find` or `ls -R` instead of `pmat query --files`
- Don't read or follow CLAUDE.md search policy
- Glob/Grep tools are "closer to hand" than invoking pmat CLI

### Solutions

11. **MCP tool priority hints**
    - Register `pmat_query_code` MCP tool with higher priority metadata
    - Add tool description emphasizing it as the PRIMARY search tool
    - Include examples in tool description

12. **Hook-based enforcement**
    - Pre-commit hook warns if new grep/rg usage added to CLAUDE.md or agent configs
    - Post-agent hook that detects grep/rg usage in agent transcripts and logs warnings

13. **Agent-specific instructions**
    - Generate `.claude/agent-instructions.md` with search policy
    - Include inline reminders in Task tool prompts
    - Add `pmat query` examples to each agent type's system prompt

14. **Compliance check (CB-131)**
    - New comply check: detect grep/rg/find usage in CLAUDE.md, AGENT.md, agent configs
    - Warn when these tools are referenced without `pmat query` alternative noted
    - Track adoption metrics: % of searches using pmat query vs grep/rg

## Acceptance Criteria

- [x] `pmat query --regex` works with Rust regex syntax
- [x] `pmat query --exclude` / `--exclude-file` filter results
- [x] `pmat query --literal` for exact string matching
- [ ] `pmat query --files` for file discovery (glob patterns)
- [x] `pmat query -A -B -C` for context lines
- [x] `pmat query --count` and `--files-with-matches` modes
- [x] `pmat query --case-sensitive` / `--ignore-case` case control
- [ ] CB-131 comply check for search tool enforcement
- [x] All features documented in --help
- [ ] CLAUDE.md updated with new flags
- [ ] Sub-agent compliance rate > 80% (measured via transcript analysis)

## Implementation Notes

- Use existing `pmat query` infrastructure (TF-IDF index, function index)
- Regex mode can bypass semantic ranking and use direct regex matching
- File discovery can leverage existing AST file traversal
- For Claude integration, focus on MCP tool registration and agent prompts
