#![cfg_attr(coverage_nightly, coverage(off))]
//! Asynchronous profiling with shadow stack instrumentation

use anyhow::Result;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::task::JoinHandle;
use wasmparser::{Operator, Payload};

use super::{HotFunction, InstructionMix, MemoryProfile, ProfilingReport};

// Types: ShadowStack, StackFrame, InstructionCategory, categorize_for_profiling
include!("profiler_types.rs");

// AsyncProfiler struct and implementation
include!("profiler_async.rs");

// ProfileAggregator struct and implementation
include!("profiler_aggregator.rs");

// Tests: property tests and async profiler coverage tests
include!("profiler_tests.rs");

// Tests: shadow stack, aggregator, and categorization tests
include!("profiler_tests_shadow_stack.rs");
