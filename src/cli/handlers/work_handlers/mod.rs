// Work handlers - split for file health (CB-040)
#![cfg_attr(coverage_nightly, coverage(off))]
include!("core_handlers.rs");
include!("ticket_handlers.rs");
