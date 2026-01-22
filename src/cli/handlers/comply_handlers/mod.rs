// Comply handlers - split for file health (CB-040)
include!("check_handlers.rs");
include!("migrate_handlers.rs");

#[cfg(test)]
#[path = "comply_handlers_tests.rs"]
mod tests;
