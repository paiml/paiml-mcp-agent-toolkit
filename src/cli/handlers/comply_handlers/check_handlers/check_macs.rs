//! MACS comply checks — CB-1650..CB-1658 (Component 32).
//!
//! Sub-spec: `docs/specifications/components/modern-agentic-coding-support.md`
//!
//! - CB-1651: Receipt Provenance Present — every schema_version>=2
//!   falsification receipt carries agent provenance (contracts/
//!   macs-provenance-v1.yaml). An unattributed v2 receipt is exactly the
//!   silent-crossing MACS F1 exists to prevent.
//! - CB-1653: Ladder Claim Drift — receipts closed above evidenced level
//!   fail; open over-claims warn (the MACS-005 gate blocks them at close).
//! - CB-1658: CoT Derivation Completeness — one obligation + one claim
//!   per step, fields verbatim (paraphrase drift is a failure).
//! - CB-1650: Skill Effort Pinned — every .claude/skills/**/{SKILL,skill}.md
//!   pins effort in {low,medium,high,xhigh}; session-only max/ultracode rejected.
//! - CB-1656: MCP Manifest Faithful — mcp.json advertises the live tool set.
//! - CB-1657: Doc Model Drift — no superseded model ids outside the registry.
//! - CB-1654: Refusal Events Acked — no ticket carries an unacknowledged
//!   Refusal event (MACS E5): a refusal-terminated turn must map to a
//!   paused ticket, never a completed (or quietly abandoned) one.
//!
//! AGY (`.agents/`) client config — PMAT-INIT-002 claims 4 and 5, GH #1031.
//! The read side of `pmat init --target agy`; see `check_macs_agy.rs` for the
//! Skip/Pass discipline all four share (a Skip must say it judged nothing).
//!
//! - CB-1663: AGY Directory Structure — every `.agents/**/*.json` parses,
//!   every `rules/*.md` is non-empty, every `skills/<name>/` has a SKILL.md.
//! - CB-1664: AGY Hooks Schema — `hooks.json` PreToolUse shape, and the
//!   referenced script exists and is executable (a missing one is a silent
//!   no-op: both clients fail open).
//! - CB-1665: AGY Skill Frontmatter — `skills/*/SKILL.md` declares `effort`
//!   (model-level tier) and `description`; the missing key is named.
//! - CB-1666: AGY MCP Client Config — `mcp_config.json` declares
//!   `mcpServers.<name>.command`/`args`, and pmat's own dead entrypoints
//!   (`cargo run …`, `--transport stdio`, bare `serve`) are rejected.

use super::types::*;
use std::path::Path;

include!("check_macs_provenance.rs");

include!("check_macs_ladder.rs");

include!("check_macs_derivation.rs");

include!("check_macs_skill_effort.rs");

include!("check_macs_artifacts.rs");

include!("check_macs_agy.rs");

include!("check_macs_agy_hooks.rs");

include!("check_macs_agy_skills.rs");

include!("check_macs_agy_mcp.rs");

include!("check_macs_tests_provenance.rs");

include!("check_macs_tests_ladder.rs");

include!("check_macs_tests_derivation.rs");

include!("check_macs_tests_skill_effort.rs");

include!("check_macs_tests_artifacts.rs");

include!("check_macs_tests_agy.rs");
