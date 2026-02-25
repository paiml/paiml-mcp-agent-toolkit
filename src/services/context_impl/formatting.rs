// Formatting module for context output (split for file health compliance CB-040)
//
// This file defines shared types used across formatting sub-modules and
// delegates implementation to include files:
//   - formatting_core.rs:  ProjectContext markdown formatting + item formatters
//   - formatting_deep.rs:  DeepContext markdown formatting + quality scorecard
//   - formatting_tests.rs: Unit tests for visitor and formatting

struct GroupedItems<'a> {
    functions: Vec<&'a AstItem>,
    structs: Vec<&'a AstItem>,
    enums: Vec<&'a AstItem>,
    traits: Vec<&'a AstItem>,
    impls: Vec<&'a AstItem>,
    modules: Vec<&'a AstItem>,
}

// ProjectContext formatting: format_context_as_markdown and helpers
include!("formatting_core.rs");

// DeepContext formatting: format_deep_context_as_markdown and helpers
include!("formatting_deep.rs");

// Unit tests for visitor and formatting functions
include!("formatting_tests.rs");
