//! KAIZEN-0165: Verify the 4 `pmat_*` AgentContextTools are wired via pmcp adapters.
//!
//! Before this fix, the adapter crate compiled but no `.tool()` call in
//! `simple_unified_server.rs` registered these tools — so MCP `tools/list`
//! returned only 16 core tools. Now the adapters exist and the server
//! registers them.
//!
//! These tests exercise the adapters directly (instead of booting a full
//! stdio server) so they run fast and survive `skip-slow-tests`.
//!
//! They assert:
//!   (a) The 4 adapter structs can be constructed from an `Arc<IndexManager>`.
//!   (b) `PmatIndexStatsHandler::handle(...)` returns non-stub JSON with a
//!       `manifest.function_count > 0` on a repo that has an existing
//!       `.pmat/context.idx` — proving the pmcp adapter actually delegates
//!       to the real tool.

use std::path::PathBuf;
use std::sync::Arc;

use pmat::mcp::tools::agent_context_tools::IndexManager;
use pmat::mcp_pmcp::agent_context_handlers::{
    PmatFindSimilarHandler, PmatGetFunctionHandler, PmatIndexStatsHandler, PmatQueryCodeHandler,
};
use pmcp::{RequestHandlerExtra, ToolHandler};
use serde_json::json;
use tokio_util::sync::CancellationToken;

fn test_extra() -> RequestHandlerExtra {
    RequestHandlerExtra::new("kaizen-0165-test".to_string(), CancellationToken::new())
}

fn pmat_project_root() -> PathBuf {
    // tests/modules/this.rs -> CARGO_MANIFEST_DIR is the worktree root.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn test_four_pmat_adapters_construct() {
    let mgr = Arc::new(IndexManager::new(pmat_project_root()));

    // Smoke: each newtype wrapper must be buildable from Arc<IndexManager>.
    let _q = PmatQueryCodeHandler::new(mgr.clone());
    let _g = PmatGetFunctionHandler::new(mgr.clone());
    let _s = PmatFindSimilarHandler::new(mgr.clone());
    let _i = PmatIndexStatsHandler::new(mgr.clone());
}

#[tokio::test]
#[ignore = "requires .pmat/context.idx; run with --ignored after `pmat index`"]
async fn test_pmat_index_stats_returns_non_stub() {
    let mgr = Arc::new(IndexManager::new(pmat_project_root()));
    let tool = PmatIndexStatsHandler::new(mgr);

    let result = tool
        .handle(json!({"rebuild": false}), test_extra())
        .await
        .expect("pmat_index_stats handler should not error on a pmat checkout");

    let fn_count = result["manifest"]["function_count"]
        .as_u64()
        .expect("response must include manifest.function_count");
    assert!(
        fn_count > 0,
        "index_stats returned stub (function_count={fn_count}); expected >0"
    );
}
