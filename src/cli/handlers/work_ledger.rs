#![cfg_attr(coverage_nightly, coverage(off))]
//! Falsification Ledger: Append-only receipt tracking for work completion
//!
//! Every `pmat work complete` produces an immutable FalsificationReceipt that:
//! - Records what was tested, what passed/failed, and any overrides
//! - Gates work completion (stale or failing receipts block)
//! - Appends to a global JSONL ledger for audit trails
//!
//! Storage layout:
//! ```text
//! .pmat-work/
//! ├── {item-id}/
//! │   ├── contract.json
//! │   └── falsification/
//! │       └── receipt-2026-02-14T10-30-00Z.json
//! └── ledger.jsonl
//! ```

use super::work_falsification::{ClaimResult, FalsificationReport};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use uuid::Uuid;

// --- Types: enums, structs, constants ---
include!("work_ledger_types.rs");

// --- FalsificationReceipt + free functions ---
include!("work_ledger_receipt.rs");

// --- FalsificationLedger service ---
include!("work_ledger_service.rs");

// --- ULTRA-002: agent file claims (.pmat-work/claims.jsonl) ---
include!("work_agent_claims.rs");
include!("work_agent_claims_handlers.rs");
include!("work_agent_claims_render.rs");

// --- ULTRA-003: triage coverage accounting (.pmat-work/triage.jsonl) ---
include!("work_agent_triage.rs");
include!("work_agent_triage_handlers.rs");

// --- MACS-019: delegation handoff + provenance boundary (#985) ---
include!("work_ledger_delegate.rs");

// --- Subcommand routing for both ---
include!("work_agent_dispatch.rs");

// --- Tests ---
include!("work_ledger_tests.rs");
include!("work_agent_claims_tests.rs");
include!("work_agent_triage_tests.rs");
include!("work_ledger_delegate_tests.rs");
