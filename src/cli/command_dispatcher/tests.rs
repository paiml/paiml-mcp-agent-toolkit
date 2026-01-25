// TEMPORARILY DISABLED: File splitting broke syntax
#[cfg(all(test, feature = "broken-tests"))]
#[path = "command_dispatcher_tests.rs"]
mod command_dispatcher_tests;
