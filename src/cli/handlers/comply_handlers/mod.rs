// Comply handlers - split for file health (CB-040)
include!("check_handlers.rs");
include!("migrate_handlers.rs");

// CB-050/CB-060 detection logic
pub mod comply_cb_detect;

#[cfg(test)]
#[path = "comply_handlers_tests.rs"]
mod tests;

#[cfg(test)]
mod falsification_tests;
