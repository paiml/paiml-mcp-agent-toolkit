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

// --- Tests ---
include!("work_ledger_tests.rs");
