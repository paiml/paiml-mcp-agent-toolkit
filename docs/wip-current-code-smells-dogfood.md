● 🔍 Top 5 Most Complex Files in Codebase

Based on the complexity analysis using our own MCP toolkit, here are the 5 most complex files that need refactoring attention:

1. ./server/src/services/context.rs

- 3 violations | Max Cyclomatic: 32 | Max Cognitive: 71
- Hotspot: format_context_as_markdown function (71 cognitive complexity!)
- Issue: Massive formatting function that needs decomposition

2. ./server/src/services/mermaid_generator.rs

- 2 violations | Max Cyclomatic: 25 | Max Cognitive: 33
- Hotspot: validate_mermaid_syntax function
- Issue: Complex validation logic for Mermaid diagrams

3. ./server/src/cli/mod.rs

- 4 violations | Max Cyclomatic: 24 | Max Cognitive: 30
- Hotspot: run function (main CLI entry point)
- Issue: Large command dispatching logic needs splitting

4. ./server/tests/documentation_examples.rs

- 4 violations | Max Cyclomatic: 23 | Max Cognitive: 57
- Hotspot: test_cli_examples_are_valid function
- Issue: Massive test function validating multiple CLI scenarios

5. ./server/src/unified_protocol/adapters/cli.rs

- 4 violations | Max Cyclomatic: 23 | Max Cognitive: 29
- Hotspot: CliAdapter::decode function
- Issue: Complex CLI command parsing and adaptation logic

🎯 Key Refactoring Targets

The format_context_as_markdown function in context.rs is the clear winner with 71 cognitive complexity - this should be the top priority for refactoring
into smaller, focused functions.

These results demonstrate our toolkit's dogfooding capabilities perfectly - we're using our own complexity analysis to identify technical debt in our own
codebase! 🚀

