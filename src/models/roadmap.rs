#![cfg_attr(coverage_nightly, coverage(off))]
// Roadmap data models for unified GitHub/YAML workflow
//
// Supports both GitHub-first and YAML-first workflows with write-through synchronization.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Types: Roadmap, RoadmapItem, Phase, Subtask, ItemType, Priority, serde helpers
include!("roadmap_types.rs");

// ItemStatus enum with custom Deserialize and fuzzy alias matching
include!("roadmap_status.rs");

// impl Roadmap + impl RoadmapItem methods
include!("roadmap_impl.rs");

#[cfg(test)]
include!("roadmap_tests.rs");
