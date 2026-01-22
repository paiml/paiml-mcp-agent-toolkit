// Storage implementation - split for file health (CB-040)

include!("storage_impl.rs");

#[cfg(test)]
#[path = "storage_tests.rs"]
mod tests;
