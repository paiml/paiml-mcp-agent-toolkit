# Deep Context Analysis

## Executive Summary

Generated: 2025-06-05 21:59:52.297663561 UTC
Version: 0.21.0
Analysis Time: 29.91s
Cache Hit Rate: 0.0%

## Quality Scorecard

- **Overall Health**: ⚠️ (75.0/100)
- **Maintainability Index**: 70.0
- **Technical Debt**: 40.0 hours estimated

## Project Structure

```
└── /
    ├── deep_context_with_complexity.md
    ├── test-deep-context-3.md
    ├── assets/
    │   ├── README.md
    │   ├── demo/
    │   │   ├── app.js
    │   │   ├── favicon.ico
    │   │   └── style.css
    │   ├── vendor/
    │   ├── project-state.json
    │   ├── project-state.d.ts
    │   └── demo.gif
    ├── README.md
    ├── CLAUDE.md
    ├── test_defect_fix.md
    ├── .paiml-display.yaml
    ├── Cargo.toml
    ├── .gitignore
    ├── deep_context.md
    ├── CONTEXT_GENERATION_RELEASE_NOTES.md
    ├── test-deep-context-2.md
    ├── KAIZEN_BINARY_PERFORMANCE.md
    ├── RELEASE_NOTES.md
    ├── docs/
    │   ├── protocol-agnostic-demo-harness.md
    │   ├── post-0.2-quality-updates-v2.md
    │   ├── deep-context-with-reporting-extended-spec.md
    │   ├── enhancement-top-files-flag.md
    │   ├── minimial-tdg-spec.md
    │   ├── satd-spec.md
    │   ├── dag-vec-v2-spec.md
    │   ├── system-status/
    │   │   ├── may31-deep-context.md
    │   │   ├── may31-2025-post-deterministic-mmd.md
    │   │   └── may31-2025-post-satd.md
    │   ├── ratatui-interactive-mode-spec.md
    │   ├── validate-mermaid-js-spec.md
    │   ├── tdg-integration-spec.md
    │   ├── post-0.2-quality-updates.md
    │   ├── dead-code-redux-spec.md
    │   ├── quickcheck-addition-spec.md
    │   ├── clap-testing-qa-spec.md
    │   ├── speed-up-tests-spec.md
    │   ├── qa-edge-github-remote-projects.md
    │   ├── implementation-summary.md
    │   ├── binary-size-improvement-spec.md
    │   ├── cli-spec.md
    │   ├── mcp-tests-spec.md
    │   ├── comprehensive-cli-tests-spec.md
    │   ├── self-documenting-enhanced-mermaid-testing-spec.md
    │   ├── deep-context-report-spec.md
    │   ├── v2-spec.md
    │   ├── demo-mode-spec.md
    │   ├── makefile-linter-rust-spec.md
    │   ├── pmat-binary-conversion-spec.md
    │   ├── deep-context.md
    │   ├── ast-enhanced-with-verifiable-proofs-spec.md
    │   ├── single-shot-context-spec.md
    │   ├── post-0.2-quality-updates-v3.md
    │   ├── qa-provability-release-spec.md
    │   ├── dupe-code-redux-spec.md
    │   ├── minimal-remote-github-spec.md
    │   ├── complexity-spec.md
    │   ├── bugs/
    │   │   ├── enhance-report-jun2-spec.md
    │   │   ├── demo-hot-fix-v20.md
    │   │   ├── mermaid-empty-bug.md
    │   │   ├── deep-context-satd-integration-bug.md
    │   │   ├── fix-deep-context-report-bug.md
    │   │   ├── archived/
    │   │   │   ├── github-actions-release-fixed.md
    │   │   │   ├── mermaid-bug-fixed.md
    │   │   │   ├── github-actions-release.doc
    │   │   │   ├── one-shot-context-bug-resolved.md
    │   │   │   ├── github-actions-release.md
    │   │   │   └── mermaid-bug-test-report.md
    │   │   ├── mermaid-empty-dag-generation.md
    │   │   └── annotated-ast-bugs-june4.md
    │   ├── curl-install-spec.md
    │   ├── grid-js-implementation-summary.md
    │   ├── dag-spec.md
    │   ├── dead-code-metrics-spec.md
    │   ├── cli-mcp.md
    │   ├── mermaid-graph-spec.md
    │   ├── qa-tui-mode-spec.md
    │   ├── qa-v2-checklist.md
    │   ├── wip-current-code-smells-dogfood.md
    │   ├── rust-docs-spec.md
    │   ├── deterministic-graphs-mmd-spec.md
    │   ├── missing-features-spec.md
    │   ├── diagnose-update-spec.md
    │   ├── simplify-demo-spec.md
    │   ├── auto-detect-make-readme-spec.md
    │   ├── extend-web-protocol-spec.md
    │   ├── qa-v2-pmat.md
    │   ├── simplify-graph-query-spec.md
    │   ├── replace-make-context-spec.md
    │   ├── readme-dogfood-spec.md
    │   ├── demo-v2-spec.md
    │   └── archive/
    │       └── SPEC.md
    ├── deep_context_with_tdg.md
    ├── .git/
    ├── deep_context_fixed.md
    ├── KAIZEN_LINT_IMPROVEMENTS.md
    ├── test_final_verification.md
    ├── rust-docs/
    │   ├── coverage.md
    │   ├── metrics-summary.json
    │   ├── mcp-protocol.md
    │   ├── cli-reference.md
    │   ├── http-api.md
    │   ├── architecture.md
    │   ├── performance.md
    │   └── technical-debt-gradient.md
    ├── kaizen-lint-config.toml
    ├── artifacts/
    │   ├── README.md
    │   ├── dogfooding/
    │   │   ├── complexity-2025-05-30.md
    │   │   ├── churn-2025-05-30.md
    │   │   ├── ast-context-2025-05-30.md
    │   │   ├── server-info-2025-05-30.md
    │   │   └── dag-2025-05-30.mmd
    │   ├── kaizen/
    │   │   └── kaizen-metrics.json
    │   ├── templates/
    │   │   ├── README.md
    │   │   └── rust-makefile-example.mk
    │   └── mermaid/
    │       ├── README.md
    │       ├── src-architecture.mmd
    │       ├── ast-generated/
    │       │   ├── simple/
    │       │   │   └── codebase-modules.mmd
    │       │   └── styled/
    │       │       └── service-interactions.mmd
    │       ├── non-code/
    │       │   ├── simple/
    │       │   │   └── architecture-overview.mmd
    │       │   └── styled/
    │       │       └── workflow-styled.mmd
    │       ├── fixtures/
    │       │   ├── complex_styled_standard.mmd
    │       │   ├── actual-paiml-high-level-system-diagram.mmd
    │       │   ├── INVALID_example_diagram.mmd
    │       │   └── reference_standard.mmd
    │       └── current_project_dag.mmd
    ├── kaizen-analysis-results.md
    ├── current_deep_context.md
    ├── .idea/
    │   ├── workspace.xml
    │   └── paiml-mcp-agent-toolkit.iml
    ├── Makefile
    ├── kaizen-test-config.toml
    ├── Cargo.lock
    ├── server/
    │   ├── default_16411311994742027426_0_561498.profraw
    │   ├── assets/
    │   │   ├── demo/
    │   │   │   ├── style.min.css
    │   │   │   ├── app.min.js
    │   │   │   └── favicon.ico
    │   │   └── vendor/
    │   ├── default_16411311994742027426_0_596783.profraw
    │   ├── post-complexity.json
    │   ├── default_16411311994742027426_0_762270.profraw
    │   ├── proptest-regressions/
    │   │   └── services/
    │   │       └── mermaid_property_tests.txt
    │   ├── default_16411311994742027426_0_596773.profraw
    │   ├── default_16411311994742027426_0_410892.profraw
    │   ├── .clippy.toml
    │   ├── Cargo.toml
    │   ├── outdated-deps.json
    │   ├── .gitignore
    │   ├── src/
    │   │   ├── bin/
    │   │   │   └── pmat.rs
    │   │   ├── proptest-regressions/
    │   │   │   └── cli_property_tests.txt
    │   │   ├── testing/
    │   │   │   ├── mod.rs
    │   │   │   ├── properties.rs
    │   │   │   └── arbitrary.rs
    │   │   ├── demo/
    │   │   │   ├── adapters/
    │   │   │   │   ├── cli.rs
    │   │   │   │   ├── mod.rs
    │   │   │   │   ├── http.rs
    │   │   │   │   ├── mcp.rs
    │   │   │   │   └── tui.rs
    │   │   │   ├── mod.rs
    │   │   │   ├── router.rs
    │   │   │   ├── export.rs
    │   │   │   ├── protocol_harness.rs
    │   │   │   ├── config.rs
    │   │   │   ├── server.rs
    │   │   │   ├── runner.rs
    │   │   │   ├── assets.rs
    │   │   │   └── templates.rs
    │   │   ├── tests/
    │   │   │   ├── code_smell_comprehensive_tests.rs
    │   │   │   ├── kaizen_reliability_patterns.rs
    │   │   │   ├── prompts.rs
    │   │   │   ├── cli_tests.rs
    │   │   │   ├── project_meta_integration_test.rs
    │   │   │   ├── binary_size.rs
    │   │   │   ├── ast_e2e.rs
    │   │   │   ├── cli_integration_full.rs
    │   │   │   ├── analyze_cli_tests.rs
    │   │   │   ├── template_resources.rs
    │   │   │   ├── dead_code_verification.rs
    │   │   │   ├── ast_regression_test.rs
    │   │   │   ├── additional_coverage.rs
    │   │   │   ├── cache_comprehensive_tests.rs
    │   │   │   ├── deep_context_tests.rs
    │   │   │   ├── cli_comprehensive_tests.rs
    │   │   │   ├── cache.rs
    │   │   │   ├── cli_property_tests.rs
    │   │   │   ├── churn.rs
    │   │   │   ├── helpers.rs
    │   │   │   ├── template_rendering.rs
    │   │   │   ├── http_adapter_tests.rs
    │   │   │   ├── clap_command_structure_tests.rs
    │   │   │   ├── cli_simple_tests.rs
    │   │   │   ├── error_handling.rs
    │   │   │   ├── mcp_protocol.rs
    │   │   │   ├── deep_context_simplified_tests.rs
    │   │   │   ├── demo_comprehensive_tests.rs
    │   │   │   ├── resources.rs
    │   │   │   ├── clap_env_var_tests.rs
    │   │   │   ├── clap_argument_parsing_tests.rs
    │   │   │   ├── complexity_distribution_verification.rs
    │   │   │   ├── error.rs
    │   │   │   ├── kaizen_test_optimizations.rs
    │   │   │   ├── build_naming_validation.rs
    │   │   │   ├── tools.rs
    │   │   │   ├── fixtures/
    │   │   │   │   ├── sample.ts
    │   │   │   │   ├── sample.js
    │   │   │   │   └── sample.py
    │   │   │   ├── lib.rs
    │   │   │   ├── claude_code_e2e.rs
    │   │   │   ├── e2e_full_coverage.rs
    │   │   │   ├── models.rs
    │   │   │   └── unified_protocol_tests.rs
    │   │   ├── cli/
    │   │   │   ├── mod.rs
    │   │   │   ├── args.rs
    │   │   │   └── diagnose.rs
    │   │   ├── unified_protocol/
    │   │   │   ├── adapters/
    │   │   │   │   ├── cli.rs
    │   │   │   │   ├── mod.rs
    │   │   │   │   ├── http.rs
    │   │   │   │   └── mcp.rs
    │   │   │   ├── mod.rs
    │   │   │   ├── test_harness.rs
    │   │   │   ├── error.rs
    │   │   │   └── service.rs
    │   │   ├── models/
    │   │   │   ├── deep_context_config.rs
    │   │   │   ├── tdg.rs
    │   │   │   ├── project_meta.rs
    │   │   │   ├── mod.rs
    │   │   │   ├── dead_code.rs
    │   │   │   ├── dag.rs
    │   │   │   ├── churn.rs
    │   │   │   ├── template.rs
    │   │   │   ├── unified_ast.rs
    │   │   │   ├── mcp.rs
    │   │   │   └── error.rs
    │   │   ├── lib.rs
    │   │   ├── utils/
    │   │   │   ├── mod.rs
    │   │   │   └── helpers.rs
    │   │   ├── stateless_server.rs
    │   │   ├── handlers/
    │   │   │   ├── prompts.rs
    │   │   │   ├── mod.rs
    │   │   │   ├── resources.rs
    │   │   │   ├── tools.rs
    │   │   │   └── initialize.rs
    │   │   └── services/
    │   │       ├── code_intelligence.rs
    │   │       ├── file_classifier.rs
    │   │       ├── lightweight_provability_analyzer.rs
    │   │       ├── context.rs
    │   │       ├── quality_gates.rs
    │   │       ├── proof_annotator.rs
    │   │       ├── mod.rs
    │   │       ├── mermaid_property_tests.rs
    │   │       ├── deep_context.rs
    │   │       ├── ranking.rs
    │   │       ├── complexity.rs
    │   │       ├── tdg_calculator.rs
    │   │       ├── deterministic_mermaid_engine.rs
    │   │       ├── ast_typescript.rs
    │   │       ├── ast_rust.rs
    │   │       ├── duplicate_detector.rs
    │   │       ├── git_clone.rs
    │   │       ├── symbol_table.rs
    │   │       ├── readme_compressor.rs
    │   │       ├── incremental_coverage_analyzer.rs
    │   │       ├── mermaid_generator.rs
    │   │       ├── file_discovery.rs
    │   │       ├── makefile_linter/
    │   │       │   ├── mod.rs
    │   │       │   ├── parser.rs
    │   │       │   ├── rules/
    │   │       │   │   ├── mod.rs
    │   │       │   │   ├── checkmake.rs
    │   │       │   │   └── performance.rs
    │   │       │   └── ast.rs
    │   │       ├── satd_detector.rs
    │   │       ├── dogfooding_engine.rs
    │   │       ├── semantic_naming.rs
    │   │       ├── dead_code_analyzer.rs
    │   │       ├── template_service.rs
    │   │       ├── artifact_writer.rs
    │   │       ├── project_meta_detector.rs
    │   │       ├── git_analysis.rs
    │   │       ├── makefile_compressor.rs
    │   │       ├── embedded_templates.rs
    │   │       ├── old_cache.rs
    │   │       ├── ast_strategies.rs
    │   │       ├── cache/
    │   │       │   ├── strategies.rs
    │   │       │   ├── persistent_manager.rs
    │   │       │   ├── manager.rs
    │   │       │   ├── mod.rs
    │   │       │   ├── content_cache.rs
    │   │       │   ├── base.rs
    │   │       │   ├── diagnostics.rs
    │   │       │   ├── cache_trait.rs
    │   │       │   ├── config.rs
    │   │       │   └── persistent.rs
    │   │       ├── defect_probability.rs
    │   │       ├── rust_borrow_checker.rs
    │   │       ├── renderer.rs
    │   │       ├── ast_based_dependency_analyzer.rs
    │   │       ├── dag_builder.rs
    │   │       ├── canonical_query.rs
    │   │       ├── unified_ast_engine.rs
    │   │       ├── fixed_graph_builder.rs
    │   │       └── ast_python.rs
    │   ├── default_16411311994742027426_0_3168394.profraw
    │   ├── default_16411311994742027426_0_561513.profraw
    │   ├── default_16411311994742027426_0_3168786.profraw
    │   ├── default_16411311994742027426_0_3168492.profraw
    │   ├── post-metrics.json
    │   ├── default_16411311994742027426_0_3168443.profraw
    │   ├── kaizen-metrics.json
    │   ├── paiml-mcp-agent-toolkit
    │   ├── post-dag.mmd
    │   ├── default_16411311994742027426_0_3168541.profraw
    │   ├── deno.lock
    │   ├── Dockerfile
    │   ├── post-metrics-no-meta.json
    │   ├── default_16411311994742027426_0_3168639.profraw
    │   ├── mcp.json
    │   ├── build.rs
    │   ├── tests/
    │   │   ├── mermaid_spec_compliance.rs.skip
    │   │   ├── git_clone_validation.rs
    │   │   ├── demo_integration.rs
    │   │   ├── export_integration.rs
    │   │   ├── enhanced_dag_integration.rs
    │   │   ├── mermaid_empty_bug_fix_test.rs
    │   │   ├── cli_documentation_sync.rs
    │   │   ├── config_integration.rs
    │   │   ├── demo_web_integration.rs
    │   │   ├── determinism_tests.rs
    │   │   ├── mcp_documentation_sync.rs
    │   │   ├── generate_mermaid_test.rs
    │   │   ├── mermaid_artifact_tests.rs
    │   │   ├── execution_mode.rs
    │   │   ├── services_integration.rs
    │   │   ├── demo_core_extraction.rs
    │   │   ├── documentation_examples.rs
    │   │   ├── demo_e2e_integration.rs
    │   │   ├── fixtures/
    │   │   │   └── test_artifacts.rs
    │   │   ├── complexity_metrics.rs
    │   │   ├── cli_comprehensive_integration.rs
    │   │   ├── ast_dag_mermaid_pipeline.rs
    │   │   ├── e2e/
    │   │   │   ├── README.md
    │   │   │   ├── installation.test.ts
    │   │   │   └── mcp_protocol.test.ts
    │   │   ├── bin_integration.rs
    │   │   └── generate_mermaid_example.rs
    │   ├── default_16411311994742027426_0_410873.profraw
    │   ├── post-complexity-sorted.json
    │   ├── templates/
    │   │   ├── readme/
    │   │   │   ├── rust/
    │   │   │   │   ├── cli.hbs
    │   │   │   │   └── cli.json
    │   │   │   ├── python-uv/
    │   │   │   │   ├── cli.hbs
    │   │   │   │   └── cli.json
    │   │   │   └── deno/
    │   │   │       ├── cli.hbs
    │   │   │       └── cli.json
    │   │   ├── makefile/
    │   │   │   ├── rust/
    │   │   │   │   ├── cli.hbs
    │   │   │   │   └── cli.json
    │   │   │   ├── python-uv/
    │   │   │   │   ├── cli.hbs
    │   │   │   │   └── cli.json
    │   │   │   └── deno/
    │   │   │       ├── cli.hbs
    │   │   │       └── cli.json
    │   │   └── gitignore/
    │   │       ├── rust/
    │   │       │   ├── cli.hbs
    │   │       │   └── cli.json
    │   │       ├── python-uv/
    │   │       │   ├── cli.hbs
    │   │       │   └── cli.json
    │   │       └── deno/
    │   │           ├── cli.hbs
    │   │           └── cli.json
    │   ├── test_output.md
    │   ├── setup_localstack.sh
    │   ├── default_16411311994742027426_0_561511.profraw
    │   ├── get-docker.sh
    │   ├── artifacts/
    │   │   └── mermaid/
    │   │       ├── README.md
    │   │       ├── ast-generated/
    │   │       │   ├── simple/
    │   │       │   │   └── codebase-modules.mmd
    │   │       │   └── styled/
    │   │       │       └── service-interactions.mmd
    │   │       └── non-code/
    │   │           ├── simple/
    │   │           │   └── architecture-overview.mmd
    │   │           └── styled/
    │   │               └── workflow-styled.mmd
    │   ├── benches/
    │   │   └── critical_path.rs
    │   ├── default_16411311994742027426_0_410867.profraw
    │   ├── examples/
    │   ├── default_16411311994742027426_0_762217.profraw
    │   ├── baseline-complexity-sorted.json
    │   ├── default_16411311994742027426_0_3168590.profraw
    │   ├── server/
    │   │   └── coverage/
    │   │       └── html/
    │   ├── default_16411311994742027426_0_3168688.profraw
    │   ├── default_16411311994742027426_0_596753.profraw
    │   ├── scripts/
    │   │   ├── docker-setup.ts
    │   │   ├── test-mcp-e2e.ts
    │   │   └── test-installation.ts
    │   ├── default_16411311994742027426_0_762241.profraw
    │   ├── baseline-dag.mmd
    │   ├── coverage/
    │   │   └── html/
    │   ├── deno.json
    │   ├── default_16411311994742027426_0_410859.profraw
    │   ├── default_16411311994742027426_0_561533.profraw
    │   ├── default_16411311994742027426_0_762232.profraw
    │   ├── baseline-complexity.json
    │   ├── baseline-metrics.json
    │   ├── .build-info.json
    │   ├── installer.sh
    │   ├── default_16411311994742027426_0_596792.profraw
    │   ├── baseline-metrics-no-meta.json
    │   ├── default_16411311994742027426_0_3168737.profraw
    │   ├── test_regex.rs
    │   ├── fuzz/
    │   │   └── fuzz_targets/
    │   │       └── fuzz_github_urls.rs
    │   └── .cargo/
    │       └── config.toml
    ├── scripts/
    │   ├── install.sh
    │   ├── dogfood-readme.ts
    │   ├── README.md
    │   ├── complexity-distribution.sh
    │   ├── generate-fuzz-corpus.ts
    │   ├── install.integration.test.ts
    │   ├── ast-mermaid-integration.test.ts
    │   ├── create-release.ts
    │   ├── mermaid-validator.ts
    │   ├── update-version.ts
    │   ├── validate-github-actions-status.ts
    │   ├── test-curl-install.ts
    │   ├── validate-demo-assets.ts
    │   ├── create-release.test.ts
    │   ├── dogfood-readme-integration.test.ts
    │   ├── test-workflow-dag.ts
    │   ├── mermaid-validator.test.ts
    │   ├── dead-code-calibration.sh
    │   ├── install.test.ts
    │   ├── validate-naming.ts
    │   ├── test-coverage-summary.md
    │   ├── run-fuzzing.ts
    │   ├── qa-verification-status.sh
    │   ├── validate-demo-assets.test.ts
    │   ├── validate-docs.ts
    │   ├── deep-context.ts
    │   ├── update-rust-docs.ts
    │   ├── lib/
    │   │   ├── install-utils.ts
    │   │   ├── create-release-utils.test.ts
    │   │   ├── create-release-utils-integration.test.ts
    │   │   ├── install-utils.test.ts
    │   │   └── create-release-utils.ts
    │   ├── install.ts
    │   ├── mcp-install.ts
    │   └── archive/
    │       ├── README.md
    │       ├── generate-from-project-state.ts
    │       ├── cleanup-releases.ts
    │       ├── dogfood-readme-deprecated.ts
    │       ├── dead-scripts/
    │       │   ├── download-mermaid.ts
    │       │   ├── docker-setup.ts
    │       │   └── mcp-install-deterministic.ts
    │       ├── cleanup-test-artifacts.ts
    │       └── verify-demo-binary-size.ts
    ├── paiml-mcp-agent-toolkit-x86_64-unknown-linux-gnu.tar.gz
    ├── baseline-dag.mmd
    ├── final_verification.md
    ├── test-deep-context.md
    ├── baseline-complexity.json
    ├── SYMBOLIC_AI_RELEASE_NOTES.md
    ├── coverage_deno/
    │   ├── 54608034-2f46-476c-b7d1-7bd77d402a32.json
    │   ├── b6edb193-5214-400f-957b-40379d83b9d7.json
    │   ├── 28b94866-d42f-4a8b-8969-0508b0c5c62e.json
    │   ├── 607aa840-8c0d-41d9-ba01-c4b0c9208b86.json
    │   ├── ac3ac8f9-4fae-48b9-9108-ebb37a77c30d.json
    │   ├── d5bd4df7-0f2a-42a4-b465-8633f027d894.json
    │   ├── 7c0d5148-4239-4e0c-a63a-3ae58f7e9bfd.json
    │   ├── 68725d37-8ceb-4707-8937-a2d17a20f186.json
    │   └── da56b935-9c17-4dde-9f33-ef66a92fd632.json
    ├── baseline-metrics.json
    ├── .github/
    │   ├── dependabot.yml
    │   ├── workflows/
    │   │   ├── release.yml
    │   │   ├── benchmark.yml
    │   │   ├── README.md
    │   │   ├── code-quality.yml
    │   │   ├── simple-release.yml
    │   │   ├── dependencies.yml
    │   │   ├── cargo-dist.yml
    │   │   ├── main.yml
    │   │   ├── auto-tag-release.yml
    │   │   ├── pr-checks.yml
    │   │   ├── create-release.yml
    │   │   ├── ci.yml
    │   │   └── automated-release.yml
    │   └── CONTRIBUTING.md
    ├── KAIZEN_TEST_IMPROVEMENTS.md
    ├── nonexistent-platform.tar.gz
    ├── fuzz/
    │   ├── Cargo.toml
    │   ├── fuzz_targets/
    │   │   ├── fuzz_mermaid_performance.rs
    │   │   ├── fuzz_dag_builder.rs
    │   │   ├── fuzz_mermaid_generation.rs
    │   │   ├── fuzz_mermaid_escaping.rs
    │   │   └── fuzz_github_urls.rs
    │   ├── Cargo.lock
    │   └── target/
    ├── target/
    ├── .config/
    │   └── nextest.toml
    ├── -platform.tar.gz
    └── .cargo/
        └── config.toml

📊 Total Files: 484, Total Size: 168167990 bytes
```

## Enhanced AST Analysis

### ./CLAUDE.md

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.96

**TDG Severity:** Normal

### ./CONTEXT_GENERATION_RELEASE_NOTES.md

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.96

**TDG Severity:** Normal

### ./KAIZEN_BINARY_PERFORMANCE.md

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.01

**TDG Severity:** Normal

### ./KAIZEN_LINT_IMPROVEMENTS.md

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.97

**TDG Severity:** Normal

### ./KAIZEN_TEST_IMPROVEMENTS.md

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.97

**TDG Severity:** Normal

### ./RELEASE_NOTES.md

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.97

**TDG Severity:** Normal

### ./SYMBOLIC_AI_RELEASE_NOTES.md

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.96

**TDG Severity:** Normal

### ./artifacts/dogfooding/ast-context-2025-05-30.md

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.20

**TDG Severity:** Normal

### ./artifacts/dogfooding/churn-2025-05-30.md

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.96

**TDG Severity:** Normal

### ./artifacts/dogfooding/complexity-2025-05-30.md

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.99

**TDG Severity:** Normal

### ./artifacts/dogfooding/server-info-2025-05-30.md

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.96

**TDG Severity:** Normal

### ./artifacts/templates/rust-makefile-example.mk

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.64

**TDG Severity:** Normal

### ./assets/demo/app.js

**Language:** javascript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.35

**TDG Severity:** Normal

### ./assets/project-state.d.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.67

**TDG Severity:** Normal

### ./assets/project-state.json

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.96

**TDG Severity:** Normal

### ./baseline-complexity.json

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.15

**TDG Severity:** Normal

### ./baseline-metrics.json

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.15

**TDG Severity:** Normal

### ./final_verification.md

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.09

**TDG Severity:** Normal

### ./fuzz/fuzz_targets/fuzz_dag_builder.rs

**Language:** rust
**Total Symbols:** 17
**Functions:** 8 | **Structs:** 2 | **Enums:** 2 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `convert_to_project_context` (private) at line 1
  - `convert_file` (private) at line 1
  - `sanitize_path` (private) at line 1
  - `convert_ast_item` (private) at line 1
  - `sanitize_name` (private) at line 1
  - `sanitize_path_import` (private) at line 1
  - `convert_visibility` (private) at line 1
  - `assert_dag_invariants` (private) at line 1

**Structs:**
  - `FuzzProject` (private) with 1 field (derives: derive) at line 1
  - `FuzzFile` (private) with 2 fields (derives: derive) at line 1

**Enums:**
  - `FuzzAstItem` (private) with 6 variants at line 1
  - `FuzzVisibility` (private) with 3 variants at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.11

**TDG Severity:** Normal

### ./fuzz/fuzz_targets/fuzz_github_urls.rs

**Language:** rust
**Total Symbols:** 3
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.22

**TDG Severity:** Normal

### ./fuzz/fuzz_targets/fuzz_mermaid_escaping.rs

**Language:** rust
**Total Symbols:** 10
**Functions:** 4 | **Structs:** 2 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Functions:**
  - `build_escape_test_graph` (private) at line 1
  - `build_label_with_special_chars` (private) at line 1
  - `assert_proper_escaping` (private) at line 1
  - `assert_valid_mermaid_syntax` (private) at line 1

**Structs:**
  - `EscapeFuzzInput` (private) with 1 field (derives: derive) at line 1
  - `FuzzLabel` (private) with 8 fields (derives: derive) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.24

**TDG Severity:** Normal

### ./fuzz/fuzz_targets/fuzz_mermaid_generation.rs

**Language:** rust
**Total Symbols:** 17
**Functions:** 6 | **Structs:** 5 | **Enums:** 2 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Functions:**
  - `build_dependency_graph` (private) at line 1
  - `sanitize_id` (private) at line 1
  - `convert_node_type` (private) at line 1
  - `convert_edge_type` (private) at line 1
  - `assert_invariants` (private) at line 1
  - `has_unescaped_pipes` (private) at line 1

**Structs:**
  - `FuzzInput` (private) with 3 fields (derives: derive) at line 1
  - `FuzzNode` (private) with 6 fields (derives: derive) at line 1
  - `FuzzEdge` (private) with 3 fields (derives: derive) at line 1
  - `FuzzOptions` (private) with 4 fields (derives: derive) at line 1
  - `ConvertedGraph` (private) with 2 fields at line 1

**Enums:**
  - `FuzzNodeType` (private) with 5 variants at line 1
  - `FuzzEdgeType` (private) with 5 variants at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.89

**TDG Severity:** Warning

### ./fuzz/fuzz_targets/fuzz_mermaid_performance.rs

**Language:** rust
**Total Symbols:** 8
**Functions:** 2 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `build_performance_graph` (private) at line 1
  - `assert_performance_bounds` (private) at line 1

**Structs:**
  - `PerfFuzzInput` (private) with 4 fields (derives: derive) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 0.82

**TDG Severity:** Normal

### ./kaizen-analysis-results.md

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.96

**TDG Severity:** Normal

### ./kaizen-lint-config.toml

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.97

**TDG Severity:** Normal

### ./kaizen-test-config.toml

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.97

**TDG Severity:** Normal

### ./rust-docs/architecture.md

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.79

**TDG Severity:** Warning

### ./rust-docs/cli-reference.md

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.98

**TDG Severity:** Normal

### ./rust-docs/coverage.md

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.96

**TDG Severity:** Normal

### ./rust-docs/http-api.md

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.13

**TDG Severity:** Normal

### ./rust-docs/mcp-protocol.md

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.09

**TDG Severity:** Normal

### ./rust-docs/metrics-summary.json

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.97

**TDG Severity:** Normal

### ./rust-docs/performance.md

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.05

**TDG Severity:** Normal

### ./rust-docs/technical-debt-gradient.md

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.48

**TDG Severity:** Normal

### ./scripts/archive/cleanup-releases.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.34

**TDG Severity:** Normal

### ./scripts/archive/cleanup-test-artifacts.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.18

**TDG Severity:** Normal

### ./scripts/archive/dead-scripts/docker-setup.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.70

**TDG Severity:** Warning

### ./scripts/archive/dead-scripts/download-mermaid.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.98

**TDG Severity:** Normal

### ./scripts/archive/dead-scripts/mcp-install-deterministic.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 2.23

**TDG Severity:** Warning

### ./scripts/archive/dogfood-readme-deprecated.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.27

**TDG Severity:** Normal

### ./scripts/archive/generate-from-project-state.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.05

**TDG Severity:** Normal

### ./scripts/archive/verify-demo-binary-size.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.07

**TDG Severity:** Normal

### ./scripts/ast-mermaid-integration.test.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 2.09

**TDG Severity:** Warning

### ./scripts/complexity-distribution.sh

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.13

**TDG Severity:** Normal

### ./scripts/create-release.test.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.16

**TDG Severity:** Normal

### ./scripts/create-release.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.73

**TDG Severity:** Normal

### ./scripts/dead-code-calibration.sh

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.00

**TDG Severity:** Normal

### ./scripts/deep-context.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 2.43

**TDG Severity:** Warning

### ./scripts/dogfood-readme-integration.test.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.44

**TDG Severity:** Normal

### ./scripts/dogfood-readme.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.06

**TDG Severity:** Normal

### ./scripts/generate-fuzz-corpus.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.88

**TDG Severity:** Normal

### ./scripts/install.integration.test.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.90

**TDG Severity:** Normal

### ./scripts/install.sh

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.18

**TDG Severity:** Normal

### ./scripts/install.test.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.59

**TDG Severity:** Warning

### ./scripts/install.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.31

**TDG Severity:** Normal

### ./scripts/lib/create-release-utils-integration.test.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.91

**TDG Severity:** Normal

### ./scripts/lib/create-release-utils.test.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.34

**TDG Severity:** Normal

### ./scripts/lib/create-release-utils.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.92

**TDG Severity:** Normal

### ./scripts/lib/install-utils.test.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.33

**TDG Severity:** Normal

### ./scripts/lib/install-utils.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.94

**TDG Severity:** Normal

### ./scripts/mcp-install.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 2.07

**TDG Severity:** Warning

### ./scripts/mermaid-validator.test.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.19

**TDG Severity:** Normal

### ./scripts/mermaid-validator.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 2.38

**TDG Severity:** Warning

### ./scripts/qa-verification-status.sh

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.20

**TDG Severity:** Normal

### ./scripts/run-fuzzing.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.74

**TDG Severity:** Warning

### ./scripts/test-coverage-summary.md

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.97

**TDG Severity:** Normal

### ./scripts/test-curl-install.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.87

**TDG Severity:** Normal

### ./scripts/test-workflow-dag.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.67

**TDG Severity:** Warning

### ./scripts/update-rust-docs.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.59

**TDG Severity:** Warning

### ./scripts/update-version.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.98

**TDG Severity:** Normal

### ./scripts/validate-demo-assets.test.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.04

**TDG Severity:** Normal

### ./scripts/validate-demo-assets.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.66

**TDG Severity:** Warning

### ./scripts/validate-docs.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.21

**TDG Severity:** Normal

### ./scripts/validate-github-actions-status.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.33

**TDG Severity:** Normal

### ./scripts/validate-naming.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.77

**TDG Severity:** Warning

### ./server/Dockerfile

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.99

**TDG Severity:** Normal

### ./server/baseline-complexity-sorted.json

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.15

**TDG Severity:** Normal

### ./server/baseline-complexity.json

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.13

**TDG Severity:** Normal

### ./server/baseline-metrics-no-meta.json

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.14

**TDG Severity:** Normal

### ./server/baseline-metrics.json

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.14

**TDG Severity:** Normal

### ./server/benches/critical_path.rs

**Language:** rust
**Total Symbols:** 9
**Functions:** 4 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `benchmark_cli_parsing` (private) at line 1
  - `benchmark_template_generation` (private) at line 1
  - `benchmark_dag_generation` (private) at line 1
  - `benchmark_context_generation` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.57

**TDG Severity:** Warning

### ./server/build.rs

**Language:** rust
**Total Symbols:** 36
**Functions:** 27 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 9

**Functions:**
  - `main` (private) at line 1
  - `verify_dependency_versions` (private) at line 1
  - `download_and_compress_assets` (private) at line 1
  - `setup_asset_directories` (private) at line 1
  - `get_asset_definitions` (private) at line 1
  - `process_assets` (private) at line 1
  - `should_skip_asset` (private) at line 1
  - `ensure_asset_downloaded` (private) at line 1
  - `download_asset` (private) at line 1
  - `handle_download_failure` (private) at line 1
  - ... and 17 more functions

**Imports:** 9 import statements

**Technical Debt Gradient:** 2.00

**TDG Severity:** Warning

### ./server/deno.json

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.65

**TDG Severity:** Normal

### ./server/fuzz/fuzz_targets/fuzz_github_urls.rs

**Language:** rust
**Total Symbols:** 3
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.22

**TDG Severity:** Normal

### ./server/get-docker.sh

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 2.04

**TDG Severity:** Warning

### ./server/installer.sh

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.07

**TDG Severity:** Normal

### ./server/mcp.json

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.97

**TDG Severity:** Normal

### ./server/outdated-deps.json

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.64

**TDG Severity:** Normal

### ./server/post-complexity-sorted.json

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.15

**TDG Severity:** Normal

### ./server/post-complexity.json

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.13

**TDG Severity:** Normal

### ./server/post-metrics-no-meta.json

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.14

**TDG Severity:** Normal

### ./server/post-metrics.json

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.14

**TDG Severity:** Normal

### ./server/scripts/docker-setup.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.38

**TDG Severity:** Normal

### ./server/scripts/test-installation.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.95

**TDG Severity:** Normal

### ./server/scripts/test-mcp-e2e.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.92

**TDG Severity:** Normal

### ./server/setup_localstack.sh

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.66

**TDG Severity:** Normal

### ./server/src/bin/pmat.rs

**Language:** rust
**Total Symbols:** 10
**Functions:** 3 | **Structs:** 0 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 6

**Functions:**
  - `detect_execution_mode` (private) at line 1
  - `init_tracing` (private) at line 1
  - `main (async)` (private) at line 1

**Enums:**
  - `ExecutionMode` (private) with 2 variants at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.06

**TDG Severity:** Normal

### ./server/src/cli/args.rs

**Language:** rust
**Total Symbols:** 8
**Functions:** 5 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `validate_params` (public) at line 1
  - `validate_type` (private) at line 1
  - `value_type_name` (private) at line 1
  - `expand_env_vars` (public) at line 1
  - `parse_key_val` (public) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.21

**TDG Severity:** Normal

### ./server/src/cli/diagnose.rs

**Language:** rust
**Total Symbols:** 34
**Functions:** 2 | **Structs:** 17 | **Enums:** 2 | **Traits:** 1 | **Impls:** 0 | **Modules:** 0 | **Imports:** 12

**Functions:**
  - `handle_diagnose (async)` (public) at line 1
  - `print_pretty_report` (private) at line 1

**Structs:**
  - `DiagnoseArgs` (public) with 4 fields (derives: derive) at line 1
  - `DiagnosticReport` (public) with 7 fields (derives: derive) at line 1
  - `BuildInfo` (public) with 4 fields (derives: derive) at line 1
  - `FeatureResult` (public) with 4 fields (derives: derive) at line 1
  - `DiagnosticSummary` (public) with 7 fields (derives: derive) at line 1
  - ... and 12 more structs

**Enums:**
  - `DiagnosticFormat` (public) with 3 variants at line 1
  - `FeatureStatus` (public) with 4 variants at line 1

**Traits:**
  - `FeatureTest` (public) at line 1

**Imports:** 12 import statements

**Technical Debt Gradient:** 1.76

**TDG Severity:** Warning

### ./server/src/cli/mod.rs

**Language:** rust
**Total Symbols:** 170
**Functions:** 94 | **Structs:** 3 | **Enums:** 18 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 55

**Functions:**
  - `parse_early_for_tracing` (public) at line 1
  - `run (async)` (public) at line 1
  - `execute_command (async)` (private) at line 1
  - `execute_analyze_command (async)` (private) at line 1
  - `execute_demo_command (async)` (private) at line 1
  - `handle_generate (async)` (private) at line 1
  - `handle_scaffold (async)` (private) at line 1
  - `handle_list (async)` (private) at line 1
  - `handle_search (async)` (private) at line 1
  - `handle_validate (async)` (private) at line 1
  - ... and 84 more functions

**Structs:**
  - `Cli` (restricted) with 6 fields (derives: derive) at line 1
  - `EarlyCliArgs` (public) with 4 fields (derives: derive) at line 1
  - `DeepContextConfigParams` (private) with 10 fields (derives: derive) at line 1

**Enums:**
  - `Mode` (restricted) with 2 variants at line 1
  - `ExecutionMode` (public) with 2 variants at line 1
  - `Commands` (public) with 10 variants at line 1
  - `AnalyzeCommands` (public) with 9 variants at line 1
  - `ContextFormat` (public) with 2 variants at line 1
  - ... and 13 more enums

**Imports:** 55 import statements

**Technical Debt Gradient:** 2.90

**TDG Severity:** Critical

### ./server/src/demo/adapters/cli.rs

**Language:** rust
**Total Symbols:** 19
**Functions:** 4 | **Structs:** 4 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 10

**Functions:**
  - `test_cli_adapter_metadata (async)` (private) at line 1
  - `test_cli_request_from_value` (private) at line 1
  - `test_cache_key_generation` (private) at line 1
  - `test_api_trace_creation` (private) at line 1

**Structs:**
  - `CliDemoAdapter` (public) with 0 fields at line 1
  - `CliRequest` (public) with 4 fields (derives: derive) at line 1
  - `CliResponse` (public) with 6 fields (derives: derive) at line 1
  - `CliApiTrace` (public) with 6 fields (derives: derive) at line 1

**Enums:**
  - `CliDemoError` (public) with 4 variants at line 1

**Imports:** 10 import statements

**Technical Debt Gradient:** 1.14

**TDG Severity:** Normal

### ./server/src/demo/adapters/http.rs

**Language:** rust
**Total Symbols:** 24
**Functions:** 4 | **Structs:** 6 | **Enums:** 2 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 12

**Functions:**
  - `test_http_adapter_metadata (async)` (private) at line 1
  - `test_http_request_from_value` (private) at line 1
  - `test_api_introspection (async)` (private) at line 1
  - `test_context_analysis (async)` (private) at line 1

**Structs:**
  - `HttpDemoAdapter` (public) with 0 fields at line 1
  - `HttpRequest` (public) with 6 fields (derives: derive) at line 1
  - `HttpResponse` (public) with 4 fields (derives: derive) at line 1
  - `HttpRequestInfo` (public) with 4 fields (derives: derive) at line 1
  - `HttpEndpoint` (public) with 5 fields (derives: derive) at line 1
  - ... and 1 more structs

**Enums:**
  - `HttpResponseBody` (public) with 4 variants at line 1
  - `HttpDemoError` (public) with 6 variants at line 1

**Imports:** 12 import statements

**Technical Debt Gradient:** 1.23

**TDG Severity:** Normal

### ./server/src/demo/adapters/mcp.rs

**Language:** rust
**Total Symbols:** 26
**Functions:** 5 | **Structs:** 9 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 11

**Functions:**
  - `test_mcp_adapter_metadata (async)` (private) at line 1
  - `test_mcp_request_from_value` (private) at line 1
  - `test_demo_analyze (async)` (private) at line 1
  - `test_unknown_method (async)` (private) at line 1
  - `test_error_conversion` (private) at line 1

**Structs:**
  - `McpDemoAdapter` (public) with 0 fields at line 1
  - `McpRequest` (public) with 4 fields (derives: derive) at line 1
  - `McpResponse` (public) with 4 fields (derives: derive) at line 1
  - `McpError` (public) with 3 fields (derives: derive) at line 1
  - `DemoAnalyzeParams` (public) with 3 fields (derives: derive) at line 1
  - ... and 4 more structs

**Enums:**
  - `McpDemoError` (public) with 7 variants at line 1

**Imports:** 11 import statements

**Technical Debt Gradient:** 1.29

**TDG Severity:** Normal

### ./server/src/demo/adapters/mod.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 0.96

**TDG Severity:** Normal

### ./server/src/demo/adapters/tui.rs

**Language:** rust
**Total Symbols:** 31
**Functions:** 0 | **Structs:** 14 | **Enums:** 5 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 12

**Structs:**
  - `TuiState` (private) with 6 fields (derives: derive) at line 1
  - `AnalysisResults` (private) with 3 fields (derives: derive) at line 1
  - `Hotspot` (private) with 4 fields (derives: derive) at line 1
  - `FileInfo` (private) with 3 fields (derives: derive) at line 1
  - `NodeInfo` (private) with 3 fields (derives: derive) at line 1
  - ... and 9 more structs

**Enums:**
  - `ControlFlow` (private) with 2 variants at line 1
  - `PanelId` (private) with 3 variants at line 1
  - `Severity` (private) with 3 variants at line 1
  - `UpdateType` (private) with 5 variants at line 1
  - `TuiDemoError` (public) with 6 variants at line 1

**Imports:** 12 import statements

**Technical Debt Gradient:** 1.87

**TDG Severity:** Warning

### ./server/src/demo/assets.rs

**Language:** rust
**Total Symbols:** 10
**Functions:** 4 | **Structs:** 1 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Functions:**
  - `get_asset` (public) at line 1
  - `get_asset` (public) at line 1
  - `decompress_asset` (public) at line 1
  - `get_asset_hash` (public) at line 1

**Structs:**
  - `EmbeddedAsset` (public) with 3 fields (derives: derive) at line 1

**Enums:**
  - `AssetEncoding` (public) with 2 variants at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.06

**TDG Severity:** Normal

### ./server/src/demo/config.rs

**Language:** rust
**Total Symbols:** 22
**Functions:** 3 | **Structs:** 9 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 9

**Functions:**
  - `test_default_config` (private) at line 1
  - `test_load_from_yaml` (private) at line 1
  - `test_config_manager (async)` (private) at line 1

**Structs:**
  - `DisplayConfig` (public) with 4 fields (derives: derive) at line 1
  - `PanelConfig` (public) with 4 fields (derives: derive) at line 1
  - `DependencyPanelConfig` (public) with 3 fields (derives: derive) at line 1
  - `ComplexityPanelConfig` (public) with 2 fields (derives: derive) at line 1
  - `ChurnPanelConfig` (public) with 2 fields (derives: derive) at line 1
  - ... and 4 more structs

**Enums:**
  - `GroupingStrategy` (public) with 3 variants at line 1

**Imports:** 9 import statements

**Technical Debt Gradient:** 1.33

**TDG Severity:** Normal

### ./server/src/demo/export.rs

**Language:** rust
**Total Symbols:** 29
**Functions:** 5 | **Structs:** 9 | **Enums:** 0 | **Traits:** 1 | **Impls:** 0 | **Modules:** 0 | **Imports:** 14

**Functions:**
  - `create_export_report` (public) at line 1
  - `create_full_export_report` (public) at line 1
  - `test_markdown_export` (private) at line 1
  - `test_json_export` (private) at line 1
  - `test_export_service` (private) at line 1

**Structs:**
  - `ExportReport` (public) with 14 fields (derives: derive) at line 1
  - `ComplexityAnalysis` (public) with 4 fields (derives: derive) at line 1
  - `ChurnAnalysis` (public) with 2 fields (derives: derive) at line 1
  - `ChurnFile` (public) with 3 fields (derives: derive) at line 1
  - `ProjectSummary` (public) with 4 fields (derives: derive) at line 1
  - ... and 4 more structs

**Traits:**
  - `Exporter` (public) at line 1

**Imports:** 14 import statements

**Technical Debt Gradient:** 1.37

**TDG Severity:** Normal

### ./server/src/demo/mod.rs

**Language:** rust
**Total Symbols:** 46
**Functions:** 23 | **Structs:** 4 | **Enums:** 3 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 16

**Functions:**
  - `run_demo (async)` (public) at line 1
  - `load_demo_config` (private) at line 1
  - `create_analyzer` (private) at line 1
  - `run_analyses (async)` (private) at line 1
  - `generate_output` (private) at line 1
  - `handle_protocol_output (async)` (private) at line 1
  - `build_protocol_request` (private) at line 1
  - `format_and_print_output` (private) at line 1
  - `print_api_metadata (async)` (private) at line 1
  - `run_all_protocols (async)` (private) at line 1
  - ... and 13 more functions

**Structs:**
  - `DemoConfig` (private) with 3 fields (derives: derive) at line 1
  - `DemoAnalyzer` (private) with 2 fields at line 1
  - `ProtocolTrace` (private) with 2 fields (derives: derive) at line 1
  - `DemoArgs` (public) with 16 fields (derives: derive) at line 1

**Enums:**
  - `AnalysisResults` (private) with 4 variants at line 1
  - `DemoOutput` (private) with 4 variants at line 1
  - `Protocol` (public) with 5 variants at line 1

**Imports:** 16 import statements

**Technical Debt Gradient:** 2.24

**TDG Severity:** Warning

### ./server/src/demo/protocol_harness.rs

**Language:** rust
**Total Symbols:** 26
**Functions:** 3 | **Structs:** 11 | **Enums:** 2 | **Traits:** 1 | **Impls:** 0 | **Modules:** 0 | **Imports:** 9

**Functions:**
  - `test_demo_engine_creation (async)` (private) at line 1
  - `test_context_cache (async)` (private) at line 1
  - `test_trace_store (async)` (private) at line 1

**Structs:**
  - `ProtocolMetadata` (public) with 7 fields (derives: derive) at line 1
  - `DemoEngine` (public) with 4 fields at line 1
  - `DemoConfig` (public) with 4 fields (derives: derive) at line 1
  - `ContextCache` (public) with 2 fields at line 1
  - `CacheEntry` (public) with 5 fields (derives: derive) at line 1
  - ... and 6 more structs

**Enums:**
  - `AnalysisStatus` (public) with 5 variants at line 1
  - `DemoError` (public) with 7 variants at line 1

**Traits:**
  - `DemoProtocol` (public) at line 1

**Imports:** 9 import statements

**Technical Debt Gradient:** 1.21

**TDG Severity:** Normal

### ./server/src/demo/router.rs

**Language:** rust
**Total Symbols:** 11
**Functions:** 3 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 7

**Functions:**
  - `build_router` (private) at line 1
  - `handle_request` (public) at line 1
  - `handle_request` (public) at line 1

**Structs:**
  - `Router` (public) with 1 field at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.16

**TDG Severity:** Normal

### ./server/src/demo/runner.rs

**Language:** rust
**Total Symbols:** 27
**Functions:** 7 | **Structs:** 4 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 16

**Functions:**
  - `resolve_repository` (public) at line 1
  - `resolve_repo_spec` (private) at line 1
  - `get_canonical_path` (private) at line 1
  - `find_git_root` (private) at line 1
  - `is_interactive_environment` (private) at line 1
  - `read_repository_path_from_user` (private) at line 1
  - `detect_repository` (public) at line 1

**Structs:**
  - `DemoRunner` (public) with 2 fields at line 1
  - `DemoStep` (public) with 7 fields (derives: derive) at line 1
  - `DemoReport` (public) with 4 fields (derives: derive) at line 1
  - `Component` (private) with 4 fields (derives: derive) at line 1

**Imports:** 16 import statements

**Technical Debt Gradient:** 2.53

**TDG Severity:** Critical

### ./server/src/demo/server.rs

**Language:** rust
**Total Symbols:** 53
**Functions:** 28 | **Structs:** 7 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 18

**Functions:**
  - `handle_connection (async)` (private) at line 1
  - `parse_minimal_request` (private) at line 1
  - `serialize_response` (private) at line 1
  - `serve_dashboard` (restricted) at line 1
  - `serve_static_asset` (restricted) at line 1
  - `serve_static_asset` (restricted) at line 1
  - `serve_architecture_analysis` (restricted) at line 1
  - `serve_defect_analysis` (restricted) at line 1
  - `serve_statistics_analysis` (restricted) at line 1
  - `serve_system_diagram` (restricted) at line 1
  - ... and 18 more functions

**Structs:**
  - `DemoContent` (public) with 10 fields (derives: derive) at line 1
  - `Hotspot` (public) with 3 fields (derives: derive) at line 1
  - `DemoState` (public) with 4 fields (derives: derive) at line 1
  - `AnalysisResults` (public) with 7 fields (derives: derive) at line 1
  - `LocalDemoServer` (public) with 2 fields at line 1
  - ... and 2 more structs

**Imports:** 18 import statements

**Technical Debt Gradient:** 1.60

**TDG Severity:** Warning

### ./server/src/demo/templates.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.09

**TDG Severity:** Normal

### ./server/src/handlers/initialize.rs

**Language:** rust
**Total Symbols:** 6
**Functions:** 2 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Functions:**
  - `handle_initialize (async)` (public) at line 1
  - `handle_tools_list (async)` (public) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.09

**TDG Severity:** Normal

### ./server/src/handlers/mod.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 1 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `handle_request (async)` (public) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.01

**TDG Severity:** Normal

### ./server/src/handlers/prompts.rs

**Language:** rust
**Total Symbols:** 6
**Functions:** 2 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Functions:**
  - `handle_prompts_list (async)` (public) at line 1
  - `handle_prompt_get (async)` (public) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.13

**TDG Severity:** Normal

### ./server/src/handlers/resources.rs

**Language:** rust
**Total Symbols:** 8
**Functions:** 2 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 6

**Functions:**
  - `handle_resource_list (async)` (public) at line 1
  - `handle_resource_read (async)` (public) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.09

**TDG Severity:** Normal

### ./server/src/handlers/tools.rs

**Language:** rust
**Total Symbols:** 123
**Functions:** 65 | **Structs:** 12 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 46

**Functions:**
  - `handle_tool_call (async)` (public) at line 1
  - `parse_tool_call_params` (private) at line 1
  - `dispatch_tool_call (async)` (private) at line 1
  - `is_template_tool` (private) at line 1
  - `is_analysis_tool` (private) at line 1
  - `handle_template_tools (async)` (private) at line 1
  - `handle_analysis_tools (async)` (private) at line 1
  - `handle_generate_template (async)` (private) at line 1
  - `handle_list_templates (async)` (private) at line 1
  - `handle_validate_template (async)` (private) at line 1
  - ... and 55 more functions

**Structs:**
  - `ValidationResult` (private) with 2 fields at line 1
  - `AnalyzeCodeChurnArgs` (private) with 3 fields (derives: derive) at line 1
  - `AnalyzeComplexityArgs` (private) with 7 fields (derives: derive) at line 1
  - `AnalyzeDagArgs` (private) with 5 fields (derives: derive) at line 1
  - `GenerateContextArgs` (private) with 7 fields (derives: derive) at line 1
  - ... and 7 more structs

**Imports:** 46 import statements

**Technical Debt Gradient:** 2.82

**TDG Severity:** Critical

### ./server/src/lib.rs

**Language:** rust
**Total Symbols:** 18
**Functions:** 1 | **Structs:** 2 | **Enums:** 0 | **Traits:** 1 | **Impls:** 0 | **Modules:** 0 | **Imports:** 14

**Functions:**
  - `run_mcp_server (async)` (public) at line 1

**Structs:**
  - `S3Client` (public) with 0 fields at line 1
  - `TemplateServer` (public) with 5 fields at line 1

**Traits:**
  - `TemplateServerTrait` (public) at line 1

**Imports:** 14 import statements

**Technical Debt Gradient:** 1.24

**TDG Severity:** Normal

### ./server/src/models/churn.rs

**Language:** rust
**Total Symbols:** 8
**Functions:** 0 | **Structs:** 3 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Structs:**
  - `CodeChurnAnalysis` (public) with 5 fields (derives: derive) at line 1
  - `FileChurnMetrics` (public) with 9 fields (derives: derive) at line 1
  - `ChurnSummary` (public) with 5 fields (derives: derive) at line 1

**Enums:**
  - `ChurnOutputFormat` (public) with 4 variants at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.03

**TDG Severity:** Normal

### ./server/src/models/dag.rs

**Language:** rust
**Total Symbols:** 7
**Functions:** 0 | **Structs:** 3 | **Enums:** 2 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Structs:**
  - `DependencyGraph` (public) with 2 fields (derives: derive) at line 1
  - `NodeInfo` (public) with 7 fields (derives: derive) at line 1
  - `Edge` (public) with 4 fields (derives: derive) at line 1

**Enums:**
  - `NodeType` (public) with 5 variants at line 1
  - `EdgeType` (public) with 5 variants at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.07

**TDG Severity:** Normal

### ./server/src/models/dead_code.rs

**Language:** rust
**Total Symbols:** 20
**Functions:** 9 | **Structs:** 5 | **Enums:** 2 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Functions:**
  - `test_file_dead_code_metrics_creation` (private) at line 1
  - `test_dead_code_item_creation` (private) at line 1
  - `test_dead_code_type_variants` (private) at line 1
  - `test_confidence_levels` (private) at line 1
  - `test_dead_code_ranking_result` (private) at line 1
  - `test_dead_code_summary_from_files` (private) at line 1
  - `test_dead_code_analysis_config_default` (private) at line 1
  - `test_file_metrics_add_different_item_types` (private) at line 1
  - `test_score_calculation_with_different_confidence_levels` (private) at line 1

**Structs:**
  - `FileDeadCodeMetrics` (public) with 11 fields (derives: derive) at line 1
  - `DeadCodeItem` (public) with 4 fields (derives: derive) at line 1
  - `DeadCodeRankingResult` (public) with 4 fields (derives: derive) at line 1
  - `DeadCodeSummary` (public) with 8 fields (derives: derive) at line 1
  - `DeadCodeAnalysisConfig` (public) with 3 fields (derives: derive) at line 1

**Enums:**
  - `ConfidenceLevel` (public) with 3 variants at line 1
  - `DeadCodeType` (public) with 4 variants at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.15

**TDG Severity:** Normal

### ./server/src/models/deep_context_config.rs

**Language:** rust
**Total Symbols:** 17
**Functions:** 10 | **Structs:** 2 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `default_dead_code_threshold` (private) at line 1
  - `default_cyclomatic_warning` (private) at line 1
  - `default_cyclomatic_error` (private) at line 1
  - `default_cognitive_warning` (private) at line 1
  - `default_cognitive_error` (private) at line 1
  - `test_default_config_validation` (private) at line 1
  - `test_entry_point_validation` (private) at line 1
  - `test_threshold_validation` (private) at line 1
  - `test_entry_point_detection` (private) at line 1
  - `test_config_serialization` (private) at line 1

**Structs:**
  - `DeepContextConfig` (public) with 6 fields (derives: derive) at line 1
  - `ComplexityThresholds` (public) with 4 fields (derives: derive) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 2.01

**TDG Severity:** Warning

### ./server/src/models/error.rs

**Language:** rust
**Total Symbols:** 3
**Functions:** 0 | **Structs:** 0 | **Enums:** 2 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 1

**Enums:**
  - `TemplateError` (public) with 10 variants at line 1
  - `AnalysisError` (public) with 5 variants at line 1

**Key Imports:**
  - `use statement` at line 1

**Technical Debt Gradient:** 1.01

**TDG Severity:** Normal

### ./server/src/models/mcp.rs

**Language:** rust
**Total Symbols:** 15
**Functions:** 0 | **Structs:** 13 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Structs:**
  - `McpRequest` (public) with 4 fields (derives: derive) at line 1
  - `McpResponse` (public) with 4 fields (derives: derive) at line 1
  - `McpError` (public) with 3 fields (derives: derive) at line 1
  - `ToolCallParams` (public) with 2 fields (derives: derive) at line 1
  - `GenerateTemplateArgs` (public) with 2 fields (derives: derive) at line 1
  - ... and 8 more structs

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.02

**TDG Severity:** Normal

### ./server/src/models/mod.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.96

**TDG Severity:** Normal

### ./server/src/models/project_meta.rs

**Language:** rust
**Total Symbols:** 10
**Functions:** 0 | **Structs:** 7 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Structs:**
  - `MetaFile` (public) with 3 fields (derives: derive) at line 1
  - `CompressedMakefile` (public) with 4 fields (derives: derive) at line 1
  - `MakeTarget` (public) with 3 fields (derives: derive) at line 1
  - `CompressedReadme` (public) with 3 fields (derives: derive) at line 1
  - `CompressedSection` (public) with 2 fields (derives: derive) at line 1
  - ... and 2 more structs

**Enums:**
  - `MetaFileType` (public) with 2 variants at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.06

**TDG Severity:** Normal

### ./server/src/models/tdg.rs

**Language:** rust
**Total Symbols:** 15
**Functions:** 2 | **Structs:** 9 | **Enums:** 2 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Functions:**
  - `test_tdg_severity_from_value` (private) at line 1
  - `test_tdg_config_default` (private) at line 1

**Structs:**
  - `TDGScore` (public) with 5 fields (derives: derive) at line 1
  - `TDGComponents` (public) with 5 fields (derives: derive) at line 1
  - `TDGConfig` (public) with 7 fields (derives: derive) at line 1
  - `TDGSummary` (public) with 8 fields (derives: derive) at line 1
  - `TDGHotspot` (public) with 4 fields (derives: derive) at line 1
  - ... and 4 more structs

**Enums:**
  - `TDGSeverity` (public) with 3 variants at line 1
  - `RecommendationType` (public) with 7 variants at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.05

**TDG Severity:** Normal

### ./server/src/models/template.rs

**Language:** rust
**Total Symbols:** 9
**Functions:** 0 | **Structs:** 4 | **Enums:** 3 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Structs:**
  - `TemplateResource` (public) with 10 fields (derives: derive) at line 1
  - `ParameterSpec` (public) with 6 fields (derives: derive) at line 1
  - `GeneratedTemplate` (public) with 4 fields (derives: derive) at line 1
  - `TemplateResponse` (public) with 1 field (derives: derive) at line 1

**Enums:**
  - `Toolchain` (public) with 3 variants at line 1
  - `TemplateCategory` (public) with 4 variants at line 1
  - `ParameterType` (public) with 6 variants at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.07

**TDG Severity:** Normal

### ./server/src/models/unified_ast.rs

**Language:** rust
**Total Symbols:** 36
**Functions:** 4 | **Structs:** 10 | **Enums:** 15 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 7

**Functions:**
  - `test_node_size` (private) at line 1
  - `test_node_alignment` (private) at line 1
  - `test_node_flags` (private) at line 1
  - `test_ast_dag` (private) at line 1

**Structs:**
  - `NodeFlags` (public) with 1 field (derives: derive) at line 1
  - `ProofAnnotation` (public) with 11 fields (derives: derive) at line 1
  - `Location` (public) with 2 fields (derives: derive) at line 1
  - `Span` (public) with 2 fields (derives: derive) at line 1
  - `BytePos` (public) with 1 field (derives: derive) at line 1
  - ... and 5 more structs

**Enums:**
  - `Language` (public) with 10 variants at line 1
  - `AstKind` (public) with 8 variants at line 1
  - `FunctionKind` (public) with 7 variants at line 1
  - `ClassKind` (public) with 6 variants at line 1
  - `VarKind` (public) with 5 variants at line 1
  - ... and 10 more enums

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.16

**TDG Severity:** Normal

### ./server/src/services/artifact_writer.rs

**Language:** rust
**Total Symbols:** 26
**Functions:** 6 | **Structs:** 7 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 12

**Functions:**
  - `test_artifact_writer_creation` (private) at line 1
  - `test_directory_structure_creation` (private) at line 1
  - `test_atomic_write_with_hash` (private) at line 1
  - `test_artifact_tree_writing` (private) at line 1
  - `test_integrity_verification` (private) at line 1
  - `test_statistics` (private) at line 1

**Structs:**
  - `ArtifactWriter` (public) with 2 fields at line 1
  - `ArtifactMetadata` (public) with 5 fields (derives: derive) at line 1
  - `VerificationReport` (public) with 4 fields (derives: derive) at line 1
  - `IntegrityFailure` (public) with 3 fields (derives: derive) at line 1
  - `ArtifactStatistics` (public) with 5 fields (derives: derive) at line 1
  - ... and 2 more structs

**Enums:**
  - `ArtifactType` (public) with 5 variants at line 1

**Imports:** 12 import statements

**Technical Debt Gradient:** 1.75

**TDG Severity:** Warning

### ./server/src/services/ast_based_dependency_analyzer.rs

**Language:** rust
**Total Symbols:** 26
**Functions:** 2 | **Structs:** 10 | **Enums:** 4 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 10

**Functions:**
  - `test_rust_dependency_analysis (async)` (private) at line 1
  - `test_typescript_dependency_analysis (async)` (private) at line 1

**Structs:**
  - `AstBasedDependencyAnalyzer` (public) with 2 fields at line 1
  - `DependencyAnalysis` (public) with 3 fields (derives: derive) at line 1
  - `Dependency` (public) with 5 fields (derives: derive) at line 1
  - `Location` (public) with 3 fields (derives: derive) at line 1
  - `BoundaryViolation` (public) with 4 fields (derives: derive) at line 1
  - ... and 5 more structs

**Enums:**
  - `ImportType` (public) with 6 variants at line 1
  - `ViolationType` (public) with 3 variants at line 1
  - `ArchitectureLayer` (public) with 4 variants at line 1
  - `Scope` (private) with 3 variants at line 1

**Imports:** 10 import statements

**Technical Debt Gradient:** 1.97

**TDG Severity:** Warning

### ./server/src/services/ast_python.rs

**Language:** rust
**Total Symbols:** 19
**Functions:** 12 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 6

**Functions:**
  - `read_file_content (async)` (private) at line 1
  - `parse_python_content` (private) at line 1
  - `calculate_complexity_metrics` (private) at line 1
  - `check_file_classification` (private) at line 1
  - `extract_ast_items` (private) at line 1
  - `analyze_python_file_with_complexity (async)` (public) at line 1
  - `analyze_python_file (async)` (public) at line 1
  - `analyze_python_file_with_classifier (async)` (public) at line 1
  - `extract_python_items` (private) at line 1
  - `create_function_item` (private) at line 1
  - ... and 2 more functions

**Structs:**
  - `PythonComplexityVisitor` (private) with 8 fields at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 2.43

**TDG Severity:** Warning

### ./server/src/services/ast_rust.rs

**Language:** rust
**Total Symbols:** 11
**Functions:** 4 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 6

**Functions:**
  - `analyze_rust_file_with_complexity (async)` (public) at line 1
  - `analyze_rust_file_with_complexity_and_classifier (async)` (public) at line 1
  - `analyze_rust_file (async)` (public) at line 1
  - `analyze_rust_file_with_classifier (async)` (public) at line 1

**Structs:**
  - `RustComplexityVisitor` (private) with 10 fields at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 2.23

**TDG Severity:** Warning

### ./server/src/services/ast_strategies.rs

**Language:** rust
**Total Symbols:** 13
**Functions:** 0 | **Structs:** 5 | **Enums:** 0 | **Traits:** 1 | **Impls:** 0 | **Modules:** 0 | **Imports:** 7

**Structs:**
  - `RustAstStrategy` (public) with 0 fields at line 1
  - `TypeScriptAstStrategy` (public) with 0 fields at line 1
  - `JavaScriptAstStrategy` (public) with 0 fields at line 1
  - `PythonAstStrategy` (public) with 0 fields at line 1
  - `StrategyRegistry` (public) with 1 field at line 1

**Traits:**
  - `AstStrategy` (public) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.05

**TDG Severity:** Normal

### ./server/src/services/ast_typescript.rs

**Language:** rust
**Total Symbols:** 10
**Functions:** 6 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Functions:**
  - `analyze_typescript_file_with_complexity (async)` (public) at line 1
  - `analyze_typescript_file_with_complexity_cached (async)` (public) at line 1
  - `analyze_typescript_file (async)` (public) at line 1
  - `analyze_javascript_file (async)` (public) at line 1
  - `analyze_typescript_file_with_classifier (async)` (public) at line 1
  - `analyze_javascript_file_with_classifier (async)` (public) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.10

**TDG Severity:** Normal

### ./server/src/services/cache/base.rs

**Language:** rust
**Total Symbols:** 7
**Functions:** 0 | **Structs:** 2 | **Enums:** 0 | **Traits:** 1 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Structs:**
  - `CacheEntry` (public) with 5 fields (derives: derive) at line 1
  - `CacheStats` (public) with 4 fields (derives: derive) at line 1

**Traits:**
  - `CacheStrategy` (public) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.03

**TDG Severity:** Normal

### ./server/src/services/cache/cache_trait.rs

**Language:** rust
**Total Symbols:** 5
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 1 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Traits:**
  - `AstCacheManager` (public) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.00

**TDG Severity:** Normal

### ./server/src/services/cache/config.rs

**Language:** rust
**Total Symbols:** 3
**Functions:** 0 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Structs:**
  - `CacheConfig` (public) with 14 fields (derives: derive) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.20

**TDG Severity:** Normal

### ./server/src/services/cache/content_cache.rs

**Language:** rust
**Total Symbols:** 9
**Functions:** 0 | **Structs:** 2 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 7

**Structs:**
  - `ContentCache` (public) with 4 fields at line 1
  - `CacheMetrics` (public) with 5 fields (derives: derive) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.41

**TDG Severity:** Normal

### ./server/src/services/cache/diagnostics.rs

**Language:** rust
**Total Symbols:** 9
**Functions:** 1 | **Structs:** 4 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Functions:**
  - `format_prometheus_metrics` (public) at line 1

**Structs:**
  - `CacheDiagnostics` (public) with 7 fields (derives: derive) at line 1
  - `CacheStatsSnapshot` (public) with 6 fields (derives: derive) at line 1
  - `CacheEffectiveness` (public) with 4 fields (derives: derive) at line 1
  - `CacheDiagnosticReport` (public) with 3 fields (derives: derive) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.25

**TDG Severity:** Normal

### ./server/src/services/cache/manager.rs

**Language:** rust
**Total Symbols:** 13
**Functions:** 0 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 12

**Structs:**
  - `SessionCacheManager` (public) with 8 fields at line 1

**Imports:** 12 import statements

**Technical Debt Gradient:** 1.38

**TDG Severity:** Normal

### ./server/src/services/cache/mod.rs

**Language:** rust
**Total Symbols:** 8
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 8

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 0.96

**TDG Severity:** Normal

### ./server/src/services/cache/persistent.rs

**Language:** rust
**Total Symbols:** 12
**Functions:** 0 | **Structs:** 2 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 10

**Structs:**
  - `PersistentCacheEntry` (private) with 3 fields (derives: derive) at line 1
  - `PersistentCache` (public) with 4 fields at line 1

**Imports:** 10 import statements

**Technical Debt Gradient:** 2.47

**TDG Severity:** Warning

### ./server/src/services/cache/persistent_manager.rs

**Language:** rust
**Total Symbols:** 8
**Functions:** 0 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 7

**Structs:**
  - `PersistentCacheManager` (public) with 5 fields at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.13

**TDG Severity:** Normal

### ./server/src/services/cache/strategies.rs

**Language:** rust
**Total Symbols:** 15
**Functions:** 0 | **Structs:** 6 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 9

**Structs:**
  - `AstCacheStrategy` (public) with 0 fields (derives: derive) at line 1
  - `TemplateCacheStrategy` (public) with 0 fields (derives: derive) at line 1
  - `DagCacheStrategy` (public) with 0 fields (derives: derive) at line 1
  - `ChurnCacheStrategy` (public) with 0 fields (derives: derive) at line 1
  - `GitStatsCacheStrategy` (public) with 0 fields (derives: derive) at line 1
  - ... and 1 more structs

**Imports:** 9 import statements

**Technical Debt Gradient:** 1.71

**TDG Severity:** Warning

### ./server/src/services/canonical_query.rs

**Language:** rust
**Total Symbols:** 28
**Functions:** 9 | **Structs:** 10 | **Enums:** 3 | **Traits:** 1 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `detect_architectural_components` (private) at line 1
  - `infer_component_relationships` (private) at line 1
  - `aggregate_component_metrics` (private) at line 1
  - `generate_styled_architecture_diagram` (private) at line 1
  - `sanitize_component_id` (private) at line 1
  - `humanize_component_name` (private) at line 1
  - `collect_component_nodes` (private) at line 1
  - `merge_coupled_components` (private) at line 1
  - `calculate_graph_diameter` (private) at line 1

**Structs:**
  - `AnalysisContext` (public) with 5 fields (derives: derive) at line 1
  - `CallGraph` (public) with 2 fields (derives: derive) at line 1
  - `CallNode` (public) with 4 fields (derives: derive) at line 1
  - `CallEdge` (public) with 4 fields (derives: derive) at line 1
  - `QueryResult` (public) with 2 fields (derives: derive) at line 1
  - ... and 5 more structs

**Enums:**
  - `CallNodeType` (public) with 5 variants at line 1
  - `CallEdgeType` (public) with 5 variants at line 1
  - `ComponentEdgeType` (public) with 4 variants at line 1

**Traits:**
  - `CanonicalQuery` (public) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.53

**TDG Severity:** Warning

### ./server/src/services/code_intelligence.rs

**Language:** rust
**Total Symbols:** 27
**Functions:** 4 | **Structs:** 10 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 12

**Functions:**
  - `analyze_dag_enhanced (async)` (public) at line 1
  - `test_analysis_request_cache_key` (private) at line 1
  - `test_unified_cache (async)` (private) at line 1
  - `test_code_intelligence_creation (async)` (private) at line 1

**Structs:**
  - `AnalysisRequest` (public) with 6 fields (derives: derive) at line 1
  - `AnalysisReport` (public) with 7 fields (derives: derive) at line 1
  - `ComplexityReport` (public) with 3 fields (derives: derive) at line 1
  - `ComplexityHotspot` (public) with 4 fields (derives: derive) at line 1
  - `DependencyGraphReport` (public) with 4 fields (derives: derive) at line 1
  - ... and 5 more structs

**Enums:**
  - `AnalysisType` (public) with 6 variants at line 1

**Imports:** 12 import statements

**Technical Debt Gradient:** 1.58

**TDG Severity:** Warning

### ./server/src/services/complexity.rs

**Language:** rust
**Total Symbols:** 23
**Functions:** 5 | **Structs:** 11 | **Enums:** 1 | **Traits:** 1 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `compute_complexity_cache_key` (public) at line 1
  - `aggregate_results` (public) at line 1
  - `format_complexity_summary` (public) at line 1
  - `format_complexity_report` (public) at line 1
  - `format_as_sarif` (public) at line 1

**Structs:**
  - `ComplexityMetrics` (public) with 4 fields (derives: derive) at line 1
  - `FileComplexityMetrics` (public) with 4 fields (derives: derive) at line 1
  - `FunctionComplexity` (public) with 4 fields (derives: derive) at line 1
  - `ClassComplexity` (public) with 5 fields (derives: derive) at line 1
  - `ComplexityThresholds` (public) with 6 fields (derives: derive) at line 1
  - ... and 6 more structs

**Enums:**
  - `Violation` (public) with 2 variants at line 1

**Traits:**
  - `ComplexityRule` (public) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.93

**TDG Severity:** Warning

### ./server/src/services/context.rs

**Language:** rust
**Total Symbols:** 57
**Functions:** 39 | **Structs:** 5 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 12

**Functions:**
  - `analyze_rust_file (async)` (public) at line 1
  - `analyze_rust_file_with_cache (async)` (public) at line 1
  - `analyze_project (async)` (public) at line 1
  - `analyze_rust_file_with_persistent_cache (async)` (public) at line 1
  - `analyze_project_with_cache (async)` (public) at line 1
  - `build_gitignore` (private) at line 1
  - `scan_and_analyze_files (async)` (private) at line 1
  - `analyze_file_by_toolchain (async)` (private) at line 1
  - `analyze_deno_file (async)` (private) at line 1
  - `build_project_summary (async)` (private) at line 1
  - ... and 29 more functions

**Structs:**
  - `ProjectContext` (public) with 3 fields (derives: derive) at line 1
  - `ProjectSummary` (public) with 7 fields (derives: derive) at line 1
  - `FileContext` (public) with 4 fields (derives: derive) at line 1
  - `RustVisitor` (private) with 2 fields at line 1
  - `GroupedItems` (private) with 6 fields at line 1

**Enums:**
  - `AstItem` (public) with 7 variants at line 1

**Imports:** 12 import statements

**Technical Debt Gradient:** 2.50

**TDG Severity:** Warning

### ./server/src/services/dag_builder.rs

**Language:** rust
**Total Symbols:** 9
**Functions:** 4 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Functions:**
  - `filter_call_edges` (public) at line 1
  - `filter_import_edges` (public) at line 1
  - `filter_inheritance_edges` (public) at line 1
  - `prune_graph_pagerank` (public) at line 1

**Structs:**
  - `DagBuilder` (public) with 4 fields at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 2.02

**TDG Severity:** Warning

### ./server/src/services/dead_code_analyzer.rs

**Language:** rust
**Total Symbols:** 37
**Functions:** 9 | **Structs:** 12 | **Enums:** 2 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 14

**Functions:**
  - `test_hierarchical_bitset` (private) at line 1
  - `test_dead_code_analyzer` (private) at line 1
  - `test_vtable_resolver` (private) at line 1
  - `test_reference_edge_creation` (private) at line 1
  - `test_reference_node_creation` (private) at line 1
  - `test_dead_code_analyzer_with_entry_points` (private) at line 1
  - `test_coverage_data_creation` (private) at line 1
  - `test_cross_lang_reference_graph` (private) at line 1
  - `test_analyze_with_ranking (async)` (private) at line 1

**Structs:**
  - `HierarchicalBitSet` (public) with 2 fields at line 1
  - `CrossLangReferenceGraph` (public) with 3 fields (derives: derive) at line 1
  - `ReferenceEdge` (public) with 4 fields (derives: derive) at line 1
  - `ReferenceNode` (public) with 3 fields (derives: derive) at line 1
  - `VTableResolver` (public) with 2 fields at line 1
  - ... and 7 more structs

**Enums:**
  - `ReferenceType` (public) with 6 variants at line 1
  - `DeadCodeType` (public) with 5 variants at line 1

**Imports:** 14 import statements

**Technical Debt Gradient:** 2.50

**TDG Severity:** Critical

### ./server/src/services/deep_context.rs

**Language:** rust
**Total Symbols:** 136
**Functions:** 24 | **Structs:** 43 | **Enums:** 14 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 55

**Functions:**
  - `analyze_ast_contexts (async)` (private) at line 1
  - `analyze_single_file (async)` (private) at line 1
  - `detect_language` (private) at line 1
  - `analyze_rust_file (async)` (private) at line 1
  - `analyze_typescript_file (async)` (private) at line 1
  - `analyze_python_file (async)` (private) at line 1
  - `analyze_complexity (async)` (private) at line 1
  - `analyze_churn (async)` (private) at line 1
  - `analyze_dead_code (async)` (private) at line 1
  - `analyze_file_for_dead_code` (private) at line 1
  - ... and 14 more functions

**Structs:**
  - `DeepContextConfig` (public) with 10 fields (derives: derive) at line 1
  - `ComplexityThresholds` (public) with 2 fields (derives: derive) at line 1
  - `DeepContext` (public) with 11 fields (derives: derive) at line 1
  - `DeepContextResult` (public) with 16 fields (derives: derive) at line 1
  - `AstSummary` (public) with 6 fields (derives: derive) at line 1
  - ... and 38 more structs

**Enums:**
  - `AnalysisType` (public) with 8 variants at line 1
  - `DagType` (public) with 4 variants at line 1
  - `CacheStrategy` (public) with 3 variants at line 1
  - `NodeType` (public) with 2 variants at line 1
  - `ConfidenceLevel` (public) with 3 variants at line 1
  - ... and 9 more enums

**Imports:** 55 import statements

**Technical Debt Gradient:** 2.88

**TDG Severity:** Critical

### ./server/src/services/defect_probability.rs

**Language:** rust
**Total Symbols:** 13
**Functions:** 4 | **Structs:** 5 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `interpolate_cdf` (private) at line 1
  - `test_defect_probability_calculation` (private) at line 1
  - `test_cdf_interpolation` (private) at line 1
  - `test_project_analysis` (private) at line 1

**Structs:**
  - `DefectProbabilityCalculator` (public) with 1 field (derives: derive) at line 1
  - `DefectWeights` (public) with 4 fields (derives: derive) at line 1
  - `FileMetrics` (public) with 9 fields (derives: derive) at line 1
  - `DefectScore` (public) with 5 fields (derives: derive) at line 1
  - `ProjectDefectAnalysis` (public) with 5 fields (derives: derive) at line 1

**Enums:**
  - `RiskLevel` (public) with 3 variants at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.60

**TDG Severity:** Warning

### ./server/src/services/deterministic_mermaid_engine.rs

**Language:** rust
**Total Symbols:** 18
**Functions:** 7 | **Structs:** 1 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 9

**Functions:**
  - `test_pagerank_determinism` (private) at line 1
  - `test_mermaid_output_determinism` (private) at line 1
  - `test_sanitize_id` (private) at line 1
  - `test_escape_mermaid_label` (private) at line 1
  - `test_is_service_module` (private) at line 1
  - `test_complexity_styling` (private) at line 1
  - `test_empty_graph` (private) at line 1

**Structs:**
  - `DeterministicMermaidEngine` (public) with 2 fields at line 1

**Enums:**
  - `ComplexityBucket` (private) with 3 variants at line 1

**Imports:** 9 import statements

**Technical Debt Gradient:** 2.07

**TDG Severity:** Warning

### ./server/src/services/dogfooding_engine.rs

**Language:** rust
**Total Symbols:** 20
**Functions:** 3 | **Structs:** 5 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 12

**Functions:**
  - `test_ast_context_generation (async)` (private) at line 1
  - `test_combined_metrics_generation (async)` (private) at line 1
  - `test_server_info_generation` (private) at line 1

**Structs:**
  - `DogfoodingEngine` (public) with 1 field at line 1
  - `FileContext` (public) with 6 fields (derives: derive) at line 1
  - `ChurnMetrics` (public) with 5 fields (derives: derive) at line 1
  - `FileHotspot` (public) with 4 fields (derives: derive) at line 1
  - `DagMetrics` (public) with 6 fields (derives: derive) at line 1

**Imports:** 12 import statements

**Technical Debt Gradient:** 1.38

**TDG Severity:** Normal

### ./server/src/services/duplicate_detector.rs

**Language:** rust
**Total Symbols:** 28
**Functions:** 5 | **Structs:** 12 | **Enums:** 3 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 8

**Functions:**
  - `test_token_hash` (private) at line 1
  - `test_minhash_similarity` (private) at line 1
  - `test_feature_extraction` (private) at line 1
  - `test_duplicate_detection` (private) at line 1
  - `test_shingle_generation` (private) at line 1

**Structs:**
  - `Token` (public) with 2 fields (derives: derive) at line 1
  - `MinHashSignature` (public) with 1 field (derives: derive) at line 1
  - `CodeFragment` (public) with 12 fields (derives: derive) at line 1
  - `CloneInstance` (public) with 7 fields (derives: derive) at line 1
  - `CloneGroup` (public) with 7 fields (derives: derive) at line 1
  - ... and 7 more structs

**Enums:**
  - `Language` (public) with 4 variants at line 1
  - `CloneType` (public) with 3 variants at line 1
  - `TokenKind` (public) with 7 variants at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 2.43

**TDG Severity:** Warning

### ./server/src/services/embedded_templates.rs

**Language:** rust
**Total Symbols:** 19
**Functions:** 12 | **Structs:** 2 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `convert_to_template_resource` (private) at line 1
  - `parse_template_category` (private) at line 1
  - `parse_toolchain` (private) at line 1
  - `convert_embedded_parameters` (private) at line 1
  - `convert_embedded_parameter` (private) at line 1
  - `parse_parameter_type` (private) at line 1
  - `convert_json_value_to_string` (private) at line 1
  - `build_s3_object_key` (private) at line 1
  - `get_category_path` (private) at line 1
  - `list_templates (async)` (public) at line 1
  - ... and 2 more functions

**Structs:**
  - `EmbeddedTemplateMetadata` (private) with 7 fields (derives: derive) at line 1
  - `EmbeddedParameter` (private) with 5 fields (derives: derive) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.18

**TDG Severity:** Normal

### ./server/src/services/file_classifier.rs

**Language:** rust
**Total Symbols:** 26
**Functions:** 9 | **Structs:** 7 | **Enums:** 2 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 8

**Functions:**
  - `calculate_shannon_entropy` (private) at line 1
  - `test_vendor_detection_determinism` (private) at line 1
  - `test_performance_on_large_files` (private) at line 1
  - `test_entropy_calculation` (private) at line 1
  - `test_binary_detection` (private) at line 1
  - `test_line_length_detection` (private) at line 1
  - `test_rust_target_directory_filtering` (private) at line 1
  - `test_additional_build_artifacts` (private) at line 1
  - `test_debug_reporter` (private) at line 1

**Structs:**
  - `FileClassifierConfig` (public) with 3 fields (derives: derive) at line 1
  - `FileClassifier` (public) with 4 fields (derives: derive) at line 1
  - `DebugReporter` (public) with 3 fields (derives: derive) at line 1
  - `DebugEvent` (public) with 6 fields (derives: derive) at line 1
  - `DebugReport` (public) with 3 fields (derives: derive) at line 1
  - ... and 2 more structs

**Enums:**
  - `ParseDecision` (public) with 2 variants at line 1
  - `SkipReason` (public) with 7 variants at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 2.42

**TDG Severity:** Warning

### ./server/src/services/file_discovery.rs

**Language:** rust
**Total Symbols:** 26
**Functions:** 7 | **Structs:** 4 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 14

**Functions:**
  - `test_file_discovery_basic` (private) at line 1
  - `test_external_repo_filtering` (private) at line 1
  - `test_max_depth_limit` (private) at line 1
  - `test_custom_ignore_patterns` (private) at line 1
  - `test_file_extension_filtering` (private) at line 1
  - `test_file_categorization` (private) at line 1
  - `test_discovery_stats` (private) at line 1

**Structs:**
  - `FileDiscoveryConfig` (public) with 6 fields (derives: derive) at line 1
  - `ProjectFileDiscovery` (public) with 3 fields at line 1
  - `DiscoveryStats` (public) with 4 fields (derives: derive) at line 1
  - `ExternalRepoFilter` (public) with 1 field at line 1

**Enums:**
  - `FileCategory` (public) with 6 variants at line 1

**Imports:** 14 import statements

**Technical Debt Gradient:** 2.50

**TDG Severity:** Warning

### ./server/src/services/fixed_graph_builder.rs

**Language:** rust
**Total Symbols:** 13
**Functions:** 3 | **Structs:** 4 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `create_test_graph` (private) at line 1
  - `test_deterministic_build` (private) at line 1
  - `test_node_limit` (private) at line 1

**Structs:**
  - `GraphConfig` (public) with 3 fields (derives: derive) at line 1
  - `FixedGraph` (public) with 2 fields (derives: derive) at line 1
  - `FixedNode` (public) with 4 fields (derives: derive) at line 1
  - `FixedGraphBuilder` (public) with 3 fields at line 1

**Enums:**
  - `GroupingStrategy` (public) with 3 variants at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.93

**TDG Severity:** Warning

### ./server/src/services/git_analysis.rs

**Language:** rust
**Total Symbols:** 10
**Functions:** 0 | **Structs:** 3 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 7

**Structs:**
  - `GitAnalysisService` (public) with 0 fields at line 1
  - `FileStats` (private) with 6 fields at line 1
  - `CommitInfo` (private) with 3 fields at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.60

**TDG Severity:** Warning

### ./server/src/services/git_clone.rs

**Language:** rust
**Total Symbols:** 20
**Functions:** 3 | **Structs:** 4 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 12

**Functions:**
  - `test_parse_github_urls (async)` (private) at line 1
  - `test_validate_github_name (async)` (private) at line 1
  - `test_cache_key_generation (async)` (private) at line 1

**Structs:**
  - `CloneProgress` (public) with 4 fields (derives: derive) at line 1
  - `ClonedRepo` (public) with 3 fields (derives: derive) at line 1
  - `GitCloner` (public) with 4 fields (derives: derive) at line 1
  - `ParsedGitHubUrl` (public) with 2 fields (derives: derive) at line 1

**Enums:**
  - `CloneError` (public) with 6 variants at line 1

**Imports:** 12 import statements

**Technical Debt Gradient:** 2.38

**TDG Severity:** Warning

### ./server/src/services/incremental_coverage_analyzer.rs

**Language:** rust
**Total Symbols:** 21
**Functions:** 2 | **Structs:** 10 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 9

**Functions:**
  - `extract_function_name` (private) at line 1
  - `test_incremental_coverage (async)` (private) at line 1

**Structs:**
  - `IncrementalCoverageAnalyzer` (public) with 4 fields at line 1
  - `FileId` (public) with 2 fields (derives: derive) at line 1
  - `AstNode` (public) with 2 fields (derives: derive) at line 1
  - `FunctionInfo` (public) with 4 fields (derives: derive) at line 1
  - `CoverageUpdate` (public) with 3 fields (derives: derive) at line 1
  - ... and 5 more structs

**Imports:** 9 import statements

**Technical Debt Gradient:** 2.07

**TDG Severity:** Warning

### ./server/src/services/lightweight_provability_analyzer.rs

**Language:** rust
**Total Symbols:** 22
**Functions:** 3 | **Structs:** 7 | **Enums:** 4 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 8

**Functions:**
  - `test_nullability_lattice` (private) at line 1
  - `test_property_domain_join` (private) at line 1
  - `test_incremental_analysis (async)` (private) at line 1

**Structs:**
  - `LightweightProvabilityAnalyzer` (public) with 3 fields at line 1
  - `FunctionId` (public) with 3 fields (derives: derive) at line 1
  - `ProofSummary` (public) with 4 fields (derives: derive) at line 1
  - `VerifiedProperty` (public) with 3 fields (derives: derive) at line 1
  - `PropertyDomain` (public) with 4 fields (derives: derive) at line 1
  - ... and 2 more structs

**Enums:**
  - `PropertyType` (public) with 6 variants at line 1
  - `NullabilityLattice` (public) with 5 variants at line 1
  - `AliasLattice` (public) with 5 variants at line 1
  - `PurityLattice` (public) with 6 variants at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.76

**TDG Severity:** Warning

### ./server/src/services/makefile_compressor.rs

**Language:** rust
**Total Symbols:** 13
**Functions:** 6 | **Structs:** 2 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `extract_package_name` (private) at line 1
  - `test_compress_basic_makefile` (private) at line 1
  - `test_compress_rust_makefile` (private) at line 1
  - `test_recipe_summarization` (private) at line 1
  - `test_dependency_extraction` (private) at line 1
  - `test_toolchain_detection` (private) at line 1

**Structs:**
  - `MakefileCompressor` (public) with 4 fields at line 1
  - `ParsedTarget` (private) with 2 fields (derives: derive) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 2.40

**TDG Severity:** Warning

### ./server/src/services/makefile_linter/ast.rs

**Language:** rust
**Total Symbols:** 22
**Functions:** 11 | **Structs:** 6 | **Enums:** 3 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Functions:**
  - `test_makefile_ast_creation` (private) at line 1
  - `test_add_node` (private) at line 1
  - `test_find_rules_by_target` (private) at line 1
  - `test_get_phony_targets` (private) at line 1
  - `test_source_span` (private) at line 1
  - `test_count_targets` (private) at line 1
  - `test_count_phony_targets` (private) at line 1
  - `test_has_pattern_rules` (private) at line 1
  - `test_uses_automatic_variables` (private) at line 1
  - `test_get_variables` (private) at line 1
  - ... and 1 more functions

**Structs:**
  - `MakefileAst` (public) with 3 fields (derives: derive) at line 1
  - `MakefileNode` (public) with 4 fields (derives: derive) at line 1
  - `RecipeLine` (public) with 2 fields (derives: derive) at line 1
  - `RecipePrefixes` (public) with 3 fields (derives: derive) at line 1
  - `SourceSpan` (public) with 4 fields (derives: derive) at line 1
  - ... and 1 more structs

**Enums:**
  - `MakefileNodeKind` (public) with 10 variants at line 1
  - `NodeData` (public) with 5 variants at line 1
  - `AssignmentOp` (public) with 5 variants at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.47

**TDG Severity:** Normal

### ./server/src/services/makefile_linter/mod.rs

**Language:** rust
**Total Symbols:** 19
**Functions:** 9 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 10

**Functions:**
  - `lint_makefile (async)` (public) at line 1
  - `calculate_quality_score` (private) at line 1
  - `test_calculate_quality_score_perfect` (private) at line 1
  - `test_calculate_quality_score_with_errors` (private) at line 1
  - `test_calculate_quality_score_with_warnings` (private) at line 1
  - `test_calculate_quality_score_minimum` (private) at line 1
  - `test_lint_result_methods` (private) at line 1
  - `test_lint_makefile_async (async)` (private) at line 1
  - `test_lint_makefile_file_not_found (async)` (private) at line 1

**Imports:** 10 import statements

**Technical Debt Gradient:** 1.09

**TDG Severity:** Normal

### ./server/src/services/makefile_linter/parser.rs

**Language:** rust
**Total Symbols:** 21
**Functions:** 16 | **Structs:** 1 | **Enums:** 2 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Functions:**
  - `test_parser_new` (private) at line 1
  - `test_parse_empty_file` (private) at line 1
  - `test_parse_simple_rule` (private) at line 1
  - `test_parse_variable` (private) at line 1
  - `test_parse_comment` (private) at line 1
  - `test_parse_pattern_rule` (private) at line 1
  - `test_parse_phony_rule` (private) at line 1
  - `test_parse_double_colon_rule` (private) at line 1
  - `test_parse_include` (private) at line 1
  - `test_parse_recipe_with_prefixes` (private) at line 1
  - ... and 6 more functions

**Structs:**
  - `MakefileParser` (public) with 5 fields (derives: derive) at line 1

**Enums:**
  - `ParseError` (public) with 4 variants at line 1
  - `LineType` (private) with 2 variants at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 2.40

**TDG Severity:** Warning

### ./server/src/services/makefile_linter/rules/checkmake.rs

**Language:** rust
**Total Symbols:** 28
**Functions:** 14 | **Structs:** 8 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `check_undefined_in_text` (private) at line 1
  - `extract_var_name` (private) at line 1
  - `should_check_variable` (private) at line 1
  - `create_undefined_violation` (private) at line 1
  - `is_automatic_var` (private) at line 1
  - `is_function_call` (private) at line 1
  - `test_min_phony_rule` (private) at line 1
  - `test_phony_declared_rule` (private) at line 1
  - `test_max_body_length_rule` (private) at line 1
  - `test_timestamp_expanded_rule` (private) at line 1
  - ... and 4 more functions

**Structs:**
  - `MinPhonyRule` (public) with 2 fields at line 1
  - `PhonyDeclaredRule` (public) with 1 field at line 1
  - `MaxBodyLengthRule` (public) with 2 fields at line 1
  - `TimestampExpandedRule` (public) with 0 fields at line 1
  - `UndefinedVariableRule` (public) with 0 fields at line 1
  - ... and 3 more structs

**Enums:**
  - `VarRefType` (private) with 3 variants at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 2.43

**TDG Severity:** Warning

### ./server/src/services/makefile_linter/rules/mod.rs

**Language:** rust
**Total Symbols:** 20
**Functions:** 8 | **Structs:** 6 | **Enums:** 1 | **Traits:** 1 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Functions:**
  - `test_severity_ordering` (private) at line 1
  - `test_violation_creation` (private) at line 1
  - `test_rule_registry_new` (private) at line 1
  - `test_rule_registry_register` (private) at line 1
  - `test_check_all_empty_ast` (private) at line 1
  - `test_check_all_sorting` (private) at line 1
  - `test_default_trait_implementation` (private) at line 1
  - `test_makefile_with_phony_targets` (private) at line 1

**Structs:**
  - `Violation` (public) with 5 fields (derives: derive) at line 1
  - `LintResult` (public) with 3 fields (derives: derive) at line 1
  - `RuleRegistry` (public) with 1 field (derives: derive) at line 1
  - `TestRule` (private) with 0 fields at line 1
  - `TestRule` (private) with 0 fields at line 1
  - ... and 1 more structs

**Enums:**
  - `Severity` (public) with 4 variants at line 1

**Traits:**
  - `MakefileRule` (public) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.14

**TDG Severity:** Normal

### ./server/src/services/makefile_linter/rules/performance.rs

**Language:** rust
**Total Symbols:** 15
**Functions:** 9 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `extract_var_refs` (private) at line 1
  - `count_var_usage` (private) at line 1
  - `is_function_call` (private) at line 1
  - `is_automatic_var` (private) at line 1
  - `test_recursive_expansion_rule` (private) at line 1
  - `test_extract_var_refs` (private) at line 1
  - `test_count_var_usage` (private) at line 1
  - `test_expensive_propagation` (private) at line 1
  - `test_multiple_targets_with_expensive_prereq` (private) at line 1

**Structs:**
  - `RecursiveExpansionRule` (public) with 1 field at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 2.40

**TDG Severity:** Warning

### ./server/src/services/mermaid_generator.rs

**Language:** rust
**Total Symbols:** 40
**Functions:** 28 | **Structs:** 2 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 10

**Functions:**
  - `load_reference_standard` (private) at line 1
  - `load_complex_styled_standard` (private) at line 1
  - `load_invalid_example` (private) at line 1
  - `validate_mermaid_syntax` (private) at line 1
  - `validate_mermaid_directive` (private) at line 1
  - `validate_content_not_empty` (private) at line 1
  - `validate_no_raw_angle_brackets` (private) at line 1
  - `validate_node_definitions` (private) at line 1
  - `test_reference_standards_are_valid` (private) at line 1
  - `test_invalid_example_is_correctly_identified` (private) at line 1
  - ... and 18 more functions

**Structs:**
  - `MermaidGenerator` (public) with 2 fields at line 1
  - `MermaidOptions` (public) with 4 fields (derives: derive) at line 1

**Imports:** 10 import statements

**Technical Debt Gradient:** 2.27

**TDG Severity:** Warning

### ./server/src/services/mermaid_property_tests.rs

**Language:** rust
**Total Symbols:** 8
**Functions:** 5 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `node_id_strategy` (private) at line 1
  - `label_strategy` (private) at line 1
  - `node_type_strategy` (private) at line 1
  - `edge_type_strategy` (private) at line 1
  - `node_strategy` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.85

**TDG Severity:** Warning

### ./server/src/services/mod.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.96

**TDG Severity:** Normal

### ./server/src/services/old_cache.rs

**Language:** rust
**Total Symbols:** 8
**Functions:** 4 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Functions:**
  - `get_metadata (async)` (public) at line 1
  - `put_metadata (async)` (public) at line 1
  - `get_content (async)` (public) at line 1
  - `put_content (async)` (public) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.10

**TDG Severity:** Normal

### ./server/src/services/project_meta_detector.rs

**Language:** rust
**Total Symbols:** 16
**Functions:** 5 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 10

**Functions:**
  - `test_detect_metadata_files (async)` (private) at line 1
  - `test_detect_various_makefile_variants (async)` (private) at line 1
  - `test_detect_various_readme_variants (async)` (private) at line 1
  - `test_max_depth_limitation (async)` (private) at line 1
  - `test_file_read_timeout (async)` (private) at line 1

**Structs:**
  - `ProjectMetaDetector` (public) with 1 field at line 1

**Imports:** 10 import statements

**Technical Debt Gradient:** 1.41

**TDG Severity:** Normal

### ./server/src/services/proof_annotator.rs

**Language:** rust
**Total Symbols:** 24
**Functions:** 3 | **Structs:** 6 | **Enums:** 1 | **Traits:** 1 | **Impls:** 0 | **Modules:** 0 | **Imports:** 13

**Functions:**
  - `test_proof_annotator_basic (async)` (private) at line 1
  - `test_proof_annotator_parallel_sources (async)` (private) at line 1
  - `test_proof_cache (async)` (private) at line 1

**Structs:**
  - `ProofCollectionResult` (public) with 3 fields (derives: derive) at line 1
  - `CollectionMetrics` (public) with 4 fields (derives: derive) at line 1
  - `ProofCache` (public) with 2 fields (derives: derive) at line 1
  - `ProofAnnotator` (public) with 3 fields at line 1
  - `CacheStats` (public) with 2 fields (derives: derive) at line 1
  - ... and 1 more structs

**Enums:**
  - `ProofCollectionError` (public) with 4 variants at line 1

**Traits:**
  - `ProofSource` (public) at line 1

**Imports:** 13 import statements

**Technical Debt Gradient:** 1.75

**TDG Severity:** Warning

### ./server/src/services/quality_gates.rs

**Language:** rust
**Total Symbols:** 21
**Functions:** 5 | **Structs:** 5 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 10

**Functions:**
  - `calculate_complexity_entropy` (private) at line 1
  - `create_test_deep_context_result` (private) at line 1
  - `test_qa_verification_dead_code` (private) at line 1
  - `test_qa_verification_complexity` (private) at line 1
  - `test_qa_report_generation` (private) at line 1

**Structs:**
  - `QAVerificationResult` (public) with 6 fields (derives: derive) at line 1
  - `DeadCodeVerification` (public) with 4 fields (derives: derive) at line 1
  - `ComplexityVerification` (public) with 5 fields (derives: derive) at line 1
  - `ProvabilityVerification` (public) with 4 fields (derives: derive) at line 1
  - `QAVerification` (public) with 1 field at line 1

**Enums:**
  - `VerificationStatus` (public) with 3 variants at line 1

**Imports:** 10 import statements

**Technical Debt Gradient:** 1.78

**TDG Severity:** Warning

### ./server/src/services/ranking.rs

**Language:** rust
**Total Symbols:** 58
**Functions:** 37 | **Structs:** 7 | **Enums:** 0 | **Traits:** 1 | **Impls:** 0 | **Modules:** 0 | **Imports:** 13

**Functions:**
  - `rank_files_vectorized` (public) at line 1
  - `rank_files_by_complexity` (public) at line 1
  - `create_test_file_metrics` (private) at line 1
  - `test_empty_file_list (async)` (private) at line 1
  - `test_limit_exceeds_files (async)` (private) at line 1
  - `test_vectorized_ranking` (private) at line 1
  - `test_composite_complexity_score_ordering` (private) at line 1
  - `test_composite_complexity_score_default` (private) at line 1
  - `test_composite_complexity_score_equality` (private) at line 1
  - `test_churn_score_default_and_ordering` (private) at line 1
  - ... and 27 more functions

**Structs:**
  - `RankingEngine` (public) with 2 fields at line 1
  - `CompositeComplexityScore` (public) with 5 fields (derives: derive) at line 1
  - `ChurnScore` (public) with 5 fields (derives: derive) at line 1
  - `DuplicationScore` (public) with 6 fields (derives: derive) at line 1
  - `ComplexityRanker` (public) with 3 fields at line 1
  - ... and 2 more structs

**Traits:**
  - `FileRanker` (public) at line 1

**Imports:** 13 import statements

**Technical Debt Gradient:** 1.85

**TDG Severity:** Warning

### ./server/src/services/readme_compressor.rs

**Language:** rust
**Total Symbols:** 15
**Functions:** 7 | **Structs:** 3 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `test_compress_basic_readme` (private) at line 1
  - `test_section_scoring` (private) at line 1
  - `test_truncate_intelligently` (private) at line 1
  - `test_extract_project_description` (private) at line 1
  - `test_markdown_parsing` (private) at line 1
  - `test_feature_extraction` (private) at line 1
  - `test_compress_section_with_budget` (private) at line 1

**Structs:**
  - `ReadmeCompressor` (public) with 2 fields at line 1
  - `Section` (private) with 5 fields (derives: derive) at line 1
  - `List` (private) with 1 field (derives: derive) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 2.40

**TDG Severity:** Warning

### ./server/src/services/renderer.rs

**Language:** rust
**Total Symbols:** 15
**Functions:** 9 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `render_template` (public) at line 1
  - `test_template_renderer_new` (private) at line 1
  - `test_render_template_simple` (private) at line 1
  - `test_render_template_with_current_timestamp` (private) at line 1
  - `test_render_template_with_helpers` (private) at line 1
  - `test_render_template_missing_variable` (private) at line 1
  - `test_render_template_error` (private) at line 1
  - `test_render_template_with_conditionals` (private) at line 1
  - `test_render_template_preserves_original_context` (private) at line 1

**Structs:**
  - `TemplateRenderer` (public) with 1 field at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.10

**TDG Severity:** Normal

### ./server/src/services/rust_borrow_checker.rs

**Language:** rust
**Total Symbols:** 19
**Functions:** 4 | **Structs:** 2 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 13

**Functions:**
  - `test_rust_borrow_checker_creation (async)` (private) at line 1
  - `test_rust_borrow_checker_collect (async)` (private) at line 1
  - `test_memory_safety_annotation` (private) at line 1
  - `test_thread_safety_annotation` (private) at line 1

**Structs:**
  - `RustBorrowChecker` (public) with 2 fields (derives: derive) at line 1
  - `CollectionState` (private) with 3 fields at line 1

**Imports:** 13 import statements

**Technical Debt Gradient:** 1.88

**TDG Severity:** Warning

### ./server/src/services/satd_detector.rs

**Language:** rust
**Total Symbols:** 29
**Functions:** 7 | **Structs:** 10 | **Enums:** 3 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 9

**Functions:**
  - `test_pattern_classification` (private) at line 1
  - `test_context_hash_stability` (private) at line 1
  - `test_extract_from_content (async)` (private) at line 1
  - `test_comment_extraction` (private) at line 1
  - `test_directory_analysis (async)` (private) at line 1
  - `test_severity_adjustment` (private) at line 1
  - `test_metrics_generation` (private) at line 1

**Structs:**
  - `SATDDetector` (public) with 2 fields at line 1
  - `TechnicalDebt` (public) with 7 fields (derives: derive) at line 1
  - `SATDAnalysisResult` (public) with 5 fields (derives: derive) at line 1
  - `SATDSummary` (public) with 5 fields (derives: derive) at line 1
  - `AstContext` (public) with 6 fields (derives: derive) at line 1
  - ... and 5 more structs

**Enums:**
  - `DebtCategory` (public) with 6 variants at line 1
  - `Severity` (public) with 4 variants at line 1
  - `AstNodeType` (public) with 5 variants at line 1

**Imports:** 9 import statements

**Technical Debt Gradient:** 2.45

**TDG Severity:** Warning

### ./server/src/services/semantic_naming.rs

**Language:** rust
**Total Symbols:** 9
**Functions:** 4 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Functions:**
  - `test_path_to_module_rust` (private) at line 1
  - `test_path_to_module_python` (private) at line 1
  - `test_get_semantic_name_priority` (private) at line 1
  - `test_clean_id` (private) at line 1

**Structs:**
  - `SemanticNamer` (public) with 1 field (derives: derive) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.15

**TDG Severity:** Normal

### ./server/src/services/symbol_table.rs

**Language:** rust
**Total Symbols:** 13
**Functions:** 4 | **Structs:** 2 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 7

**Functions:**
  - `test_symbol_table_insertion_and_lookup` (private) at line 1
  - `test_relative_location_resolution` (private) at line 1
  - `test_qualified_name_parsing` (private) at line 1
  - `test_symbol_table_builder` (private) at line 1

**Structs:**
  - `SymbolTable` (public) with 2 fields (derives: derive) at line 1
  - `SymbolTableBuilder` (public) with 1 field at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.41

**TDG Severity:** Normal

### ./server/src/services/tdg_calculator.rs

**Language:** rust
**Total Symbols:** 14
**Functions:** 2 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 11

**Functions:**
  - `test_tdg_calculation (async)` (private) at line 1
  - `test_tdg_distribution` (private) at line 1

**Structs:**
  - `TDGCalculator` (public) with 4 fields at line 1

**Imports:** 11 import statements

**Technical Debt Gradient:** 2.29

**TDG Severity:** Warning

### ./server/src/services/template_service.rs

**Language:** rust
**Total Symbols:** 27
**Functions:** 12 | **Structs:** 6 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 9

**Functions:**
  - `get_template_content (async)` (public) at line 1
  - `generate_template (async)` (public) at line 1
  - `generate_context (async)` (private) at line 1
  - `list_templates (async)` (public) at line 1
  - `list_all_resources (async)` (public) at line 1
  - `parse_template_uri` (private) at line 1
  - `build_template_prefix` (private) at line 1
  - `extract_filename` (private) at line 1
  - `validate_parameters` (private) at line 1
  - `scaffold_project (async)` (public) at line 1
  - ... and 2 more functions

**Structs:**
  - `ScaffoldResult` (public) with 2 fields (derives: derive) at line 1
  - `GeneratedFile` (public) with 3 fields (derives: derive) at line 1
  - `ScaffoldError` (public) with 2 fields (derives: derive) at line 1
  - `SearchResult` (public) with 3 fields (derives: derive) at line 1
  - `ValidationResult` (public) with 2 fields (derives: derive) at line 1
  - ... and 1 more structs

**Imports:** 9 import statements

**Technical Debt Gradient:** 2.05

**TDG Severity:** Warning

### ./server/src/services/unified_ast_engine.rs

**Language:** rust
**Total Symbols:** 23
**Functions:** 3 | **Structs:** 9 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 10

**Functions:**
  - `test_deterministic_artifact_generation (async)` (private) at line 1
  - `test_path_to_module_name` (private) at line 1
  - `test_is_source_file` (private) at line 1

**Structs:**
  - `UnifiedAstEngine` (public) with 2 fields at line 1
  - `LanguageParsers` (public) with 0 fields at line 1
  - `AstForest` (public) with 1 field (derives: derive) at line 1
  - `ModuleNode` (public) with 4 fields (derives: derive) at line 1
  - `ModuleMetrics` (public) with 4 fields (derives: derive) at line 1
  - ... and 4 more structs

**Enums:**
  - `FileAst` (public) with 9 variants at line 1

**Imports:** 10 import statements

**Technical Debt Gradient:** 2.45

**TDG Severity:** Warning

### ./server/src/stateless_server.rs

**Language:** rust
**Total Symbols:** 8
**Functions:** 0 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 7

**Structs:**
  - `StatelessTemplateServer` (public) with 1 field at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.06

**TDG Severity:** Normal

### ./server/src/testing/arbitrary.rs

**Language:** rust
**Total Symbols:** 15
**Functions:** 8 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 7

**Functions:**
  - `arbitrary_identifier` (private) at line 1
  - `arbitrary_type_name` (private) at line 1
  - `arbitrary_module_path` (private) at line 1
  - `arbitrary_import_list` (private) at line 1
  - `arbitrary_bounded_vec` (private) at line 1
  - `test_ast_node_arbitrary_terminates` (private) at line 1
  - `test_dependency_graph_has_nodes` (private) at line 1
  - `test_symbol_arbitrary_valid_name` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.64

**TDG Severity:** Warning

### ./server/src/testing/mod.rs

**Language:** rust
**Total Symbols:** 2
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 0.96

**TDG Severity:** Normal

### ./server/src/testing/properties.rs

**Language:** rust
**Total Symbols:** 40
**Functions:** 26 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 14

**Functions:**
  - `prop_ast_parsing_deterministic` (private) at line 1
  - `prop_complexity_calculation_deterministic` (private) at line 1
  - `prop_dag_builder_deterministic` (private) at line 1
  - `prop_cache_get_put_coherence` (private) at line 1
  - `prop_cache_invalidation_consistency` (private) at line 1
  - `prop_request_response_symmetry` (private) at line 1
  - `prop_protocol_error_handling_consistency` (private) at line 1
  - `prop_concurrent_ast_analysis_safety` (private) at line 1
  - `prop_cache_concurrent_access_safety` (private) at line 1
  - `prop_dag_acyclic_invariant` (private) at line 1
  - ... and 16 more functions

**Imports:** 14 import statements

**Technical Debt Gradient:** 1.70

**TDG Severity:** Warning

### ./server/src/tests/additional_coverage.rs

**Language:** rust
**Total Symbols:** 8
**Functions:** 3 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `test_churn_output_format` (private) at line 1
  - `test_cli_validate_params` (private) at line 1
  - `test_additional_model_coverage` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.04

**TDG Severity:** Normal

### ./server/src/tests/analyze_cli_tests.rs

**Language:** rust
**Total Symbols:** 11
**Functions:** 7 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Functions:**
  - `test_analyze_churn_command_parsing` (private) at line 1
  - `test_analyze_churn_with_all_options` (private) at line 1
  - `test_analyze_churn_format_options` (private) at line 1
  - `test_analyze_churn_invalid_format` (private) at line 1
  - `test_analyze_churn_short_flags` (private) at line 1
  - `test_analyze_subcommand_help` (private) at line 1
  - `test_analyze_churn_help` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.30

**TDG Severity:** Normal

### ./server/src/tests/ast_e2e.rs

**Language:** rust
**Total Symbols:** 24
**Functions:** 13 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 11

**Functions:**
  - `test_analyze_python_file_comprehensive (async)` (private) at line 1
  - `test_python_class_field_count (async)` (private) at line 1
  - `test_python_import_parsing (async)` (private) at line 1
  - `test_python_file_not_found (async)` (private) at line 1
  - `test_python_invalid_syntax (async)` (private) at line 1
  - `test_analyze_typescript_file_comprehensive (async)` (private) at line 1
  - `test_analyze_javascript_file (async)` (private) at line 1
  - `test_typescript_class_field_count (async)` (private) at line 1
  - `test_tsx_file_detection (async)` (private) at line 1
  - `test_jsx_file_detection (async)` (private) at line 1
  - ... and 3 more functions

**Imports:** 11 import statements

**Technical Debt Gradient:** 1.85

**TDG Severity:** Warning

### ./server/src/tests/ast_regression_test.rs

**Language:** rust
**Total Symbols:** 5
**Functions:** 2 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `test_ast_analysis_not_empty_regression (async)` (private) at line 1
  - `test_deep_context_includes_ast_analysis (async)` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.02

**TDG Severity:** Normal

### ./server/src/tests/binary_size.rs

**Language:** rust
**Total Symbols:** 8
**Functions:** 5 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `binary_size_regression` (private) at line 1
  - `feature_size_impact` (private) at line 1
  - `template_compression_works` (private) at line 1
  - `startup_time_regression` (private) at line 1
  - `memory_usage_baseline` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.11

**TDG Severity:** Normal

### ./server/src/tests/build_naming_validation.rs

**Language:** rust
**Total Symbols:** 10
**Functions:** 8 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Functions:**
  - `test_cargo_build_has_single_correct_binary` (private) at line 1
  - `test_no_old_package_references` (private) at line 1
  - `test_no_old_binary_references_in_workflows` (private) at line 1
  - `test_correct_binary_name_in_workflows` (private) at line 1
  - `test_no_wrong_repo_urls_in_workflows` (private) at line 1
  - `test_workspace_aware_cargo_commands_in_makefile` (private) at line 1
  - `test_cargo_lock_only_in_root` (private) at line 1
  - `test_build_script_workspace_aware` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.35

**TDG Severity:** Normal

### ./server/src/tests/cache.rs

**Language:** rust
**Total Symbols:** 27
**Functions:** 9 | **Structs:** 7 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 11

**Functions:**
  - `test_session_cache_manager (async)` (private) at line 1
  - `test_ast_cache (async)` (private) at line 1
  - `test_template_cache (async)` (private) at line 1
  - `test_dag_cache (async)` (private) at line 1
  - `test_churn_cache (async)` (private) at line 1
  - `test_git_stats_cache (async)` (private) at line 1
  - `test_cache_eviction (async)` (private) at line 1
  - `test_cache_clear (async)` (private) at line 1
  - `test_cache_ttl (async)` (private) at line 1

**Structs:**
  - `TestAstCacheStrategy` (private) with 0 fields (derives: derive) at line 1
  - `TestDagCacheStrategy` (private) with 0 fields (derives: derive) at line 1
  - `TestChurnCacheStrategy` (private) with 0 fields (derives: derive) at line 1
  - `TestGitStatsCacheStrategy` (private) with 0 fields (derives: derive) at line 1
  - `SmallCacheStrategy` (private) with 0 fields (derives: derive) at line 1
  - ... and 2 more structs

**Imports:** 11 import statements

**Technical Debt Gradient:** 1.16

**TDG Severity:** Normal

### ./server/src/tests/cache_comprehensive_tests.rs

**Language:** rust
**Total Symbols:** 23
**Functions:** 16 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 7

**Functions:**
  - `test_cache_config_default_values` (private) at line 1
  - `test_cache_config_custom_values` (private) at line 1
  - `test_cache_config_serialization` (private) at line 1
  - `test_cache_stats_snapshot_creation` (private) at line 1
  - `test_cache_stats_snapshot_zero_requests` (private) at line 1
  - `test_cache_stats_snapshot_serialization` (private) at line 1
  - `test_cache_effectiveness_structure` (private) at line 1
  - `test_cache_effectiveness_serialization` (private) at line 1
  - `test_cache_diagnostics_structure` (private) at line 1
  - `test_cache_stats_hit_rate_calculation` (private) at line 1
  - ... and 6 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.06

**TDG Severity:** Normal

### ./server/src/tests/churn.rs

**Language:** rust
**Total Symbols:** 21
**Functions:** 11 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 10

**Functions:**
  - `create_test_analysis` (private) at line 1
  - `test_churn_score_calculation` (private) at line 1
  - `test_output_format_parsing` (private) at line 1
  - `test_format_churn_summary` (private) at line 1
  - `test_format_churn_markdown` (private) at line 1
  - `test_format_churn_csv` (private) at line 1
  - `test_no_git_repository_error` (private) at line 1
  - `test_parse_commit_line` (private) at line 1
  - `test_multiple_commits_and_files` (private) at line 1
  - `test_churn_score_edge_cases` (private) at line 1
  - ... and 1 more functions

**Imports:** 10 import statements

**Technical Debt Gradient:** 1.12

**TDG Severity:** Normal

### ./server/src/tests/clap_argument_parsing_tests.rs

**Language:** rust
**Total Symbols:** 36
**Functions:** 28 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 8

**Functions:**
  - `test_numeric_argument_coercion` (private) at line 1
  - `test_path_argument_coercion` (private) at line 1
  - `test_enum_argument_coercion` (private) at line 1
  - `test_boolean_flag_coercion` (private) at line 1
  - `test_optional_argument_coercion` (private) at line 1
  - `test_vec_argument_coercion` (private) at line 1
  - `test_numeric_range_validation` (private) at line 1
  - `test_enum_validation` (private) at line 1
  - `test_path_validation` (private) at line 1
  - `test_mutually_exclusive_flags` (private) at line 1
  - ... and 18 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 2.50

**TDG Severity:** Critical

### ./server/src/tests/clap_command_structure_tests.rs

**Language:** rust
**Total Symbols:** 25
**Functions:** 20 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `test_derive_parser_propagation` (private) at line 1
  - `test_binary_name_detection` (private) at line 1
  - `test_global_args_accessible` (private) at line 1
  - `test_subcommand_hierarchy` (private) at line 1
  - `test_propagate_version` (private) at line 1
  - `test_help_generation` (private) at line 1
  - `test_env_var_support` (private) at line 1
  - `test_command_aliases` (private) at line 1
  - `test_required_args_validation` (private) at line 1
  - `test_global_flags_precedence` (private) at line 1
  - ... and 10 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.49

**TDG Severity:** Normal

### ./server/src/tests/clap_env_var_tests.rs

**Language:** rust
**Total Symbols:** 32
**Functions:** 21 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 11

**Functions:**
  - `test_rust_log_env_var` (private) at line 1
  - `test_env_var_precedence` (private) at line 1
  - `test_empty_env_var` (private) at line 1
  - `test_env_var_unset` (private) at line 1
  - `test_env_var_with_special_characters` (private) at line 1
  - `test_env_var_unicode` (private) at line 1
  - `test_env_var_with_verbose_flags` (private) at line 1
  - `test_multiple_env_vars` (private) at line 1
  - `test_env_var_parsing_errors` (private) at line 1
  - `test_explicit_none_vs_env_var` (private) at line 1
  - ... and 11 more functions

**Imports:** 11 import statements

**Technical Debt Gradient:** 2.06

**TDG Severity:** Warning

### ./server/src/tests/claude_code_e2e.rs

**Language:** rust
**Total Symbols:** 27
**Functions:** 21 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `create_test_server` (private) at line 1
  - `create_tool_request` (private) at line 1
  - `test_claude_code_rust_cli_workflow (async)` (private) at line 1
  - `test_claude_code_all_languages_scaffold (async)` (private) at line 1
  - `create_scaffold_test_cases` (private) at line 1
  - `test_toolchain_scaffolding (async)` (private) at line 1
  - `create_scaffold_request` (private) at line 1
  - `validate_scaffold_response` (private) at line 1
  - `verify_generated_files` (private) at line 1
  - `process_generated_file` (private) at line 1
  - ... and 11 more functions

**Structs:**
  - `GeneratedFileFlags` (private) with 3 fields at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.43

**TDG Severity:** Normal

### ./server/src/tests/cli_comprehensive_tests.rs

**Language:** rust
**Total Symbols:** 36
**Functions:** 31 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `test_generate_command_full_parsing` (private) at line 1
  - `test_generate_command_aliases` (private) at line 1
  - `test_generate_missing_required_args` (private) at line 1
  - `test_scaffold_command_parsing` (private) at line 1
  - `test_scaffold_template_delimiter` (private) at line 1
  - `test_scaffold_default_parallel` (private) at line 1
  - `test_list_command_all_formats` (private) at line 1
  - `test_list_command_filters` (private) at line 1
  - `test_list_default_format` (private) at line 1
  - `test_search_command_parsing` (private) at line 1
  - ... and 21 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.82

**TDG Severity:** Warning

### ./server/src/tests/cli_integration_full.rs

**Language:** rust
**Total Symbols:** 21
**Functions:** 8 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 13

**Functions:**
  - `test_cli_run_generate_to_file (async)` (private) at line 1
  - `test_cli_generate_template_direct (async)` (private) at line 1
  - `test_cli_list_templates_direct (async)` (private) at line 1
  - `test_cli_search_templates_direct (async)` (private) at line 1
  - `test_cli_validate_template_direct (async)` (private) at line 1
  - `test_cli_scaffold_project_direct (async)` (private) at line 1
  - `test_cli_context_generation (async)` (private) at line 1
  - `test_cli_churn_analysis (async)` (private) at line 1

**Imports:** 13 import statements

**Technical Debt Gradient:** 1.23

**TDG Severity:** Normal

### ./server/src/tests/cli_property_tests.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.53

**TDG Severity:** Warning

### ./server/src/tests/cli_simple_tests.rs

**Language:** rust
**Total Symbols:** 9
**Functions:** 5 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Functions:**
  - `test_validate_params_basic` (private) at line 1
  - `test_expand_env_vars_basic` (private) at line 1
  - `test_output_format_enum` (private) at line 1
  - `test_context_format_enum` (private) at line 1
  - `test_commands_construction` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.00

**TDG Severity:** Normal

### ./server/src/tests/cli_tests.rs

**Language:** rust
**Total Symbols:** 38
**Functions:** 25 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 13

**Functions:**
  - `test_validate_params_all_valid` (private) at line 1
  - `test_validate_params_missing_required` (private) at line 1
  - `test_validate_params_unknown_parameter` (private) at line 1
  - `test_validate_params_type_validation` (private) at line 1
  - `test_expand_env_vars` (private) at line 1
  - `test_expand_env_vars_no_vars` (private) at line 1
  - `test_expand_env_vars_multiple_occurrences` (private) at line 1
  - `create_test_server (async)` (private) at line 1
  - `test_generate_command_to_stdout (async)` (private) at line 1
  - `test_generate_command_to_file (async)` (private) at line 1
  - ... and 15 more functions

**Imports:** 13 import statements

**Technical Debt Gradient:** 1.31

**TDG Severity:** Normal

### ./server/src/tests/code_smell_comprehensive_tests.rs

**Language:** rust
**Total Symbols:** 37
**Functions:** 26 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 10

**Functions:**
  - `test_cross_reference_tracking` (private) at line 1
  - `test_entry_point_detection` (private) at line 1
  - `test_dynamic_dispatch_resolution` (private) at line 1
  - `test_hierarchical_bitset_optimization` (private) at line 1
  - `test_confidence_scoring` (private) at line 1
  - `test_coverage_integration` (private) at line 1
  - `test_multi_language_comment_parsing` (private) at line 1
  - `test_contextual_classification` (private) at line 1
  - `test_severity_scoring` (private) at line 1
  - `test_complexity_integration` (private) at line 1
  - ... and 16 more functions

**Structs:**
  - `PurityAnalysis` (private) with 3 fields at line 1

**Imports:** 10 import statements

**Technical Debt Gradient:** 2.03

**TDG Severity:** Warning

### ./server/src/tests/complexity_distribution_verification.rs

**Language:** rust
**Total Symbols:** 10
**Functions:** 7 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Functions:**
  - `verify_complexity_distribution` (private) at line 1
  - `calculate_entropy` (private) at line 1
  - `calculate_coefficient_of_variation` (private) at line 1
  - `test_entropy_calculation` (private) at line 1
  - `test_complexity_distribution_verification` (private) at line 1
  - `test_coefficient_of_variation` (private) at line 1
  - `generate_realistic_functions` (private) at line 1

**Structs:**
  - `ComplexityDistributionConfig` (private) with 3 fields (derives: derive) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.22

**TDG Severity:** Normal

### ./server/src/tests/dead_code_verification.rs

**Language:** rust
**Total Symbols:** 10
**Functions:** 6 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Functions:**
  - `verify_entry_point_detection` (private) at line 1
  - `verify_cross_language_references` (private) at line 1
  - `verify_dead_code_detection` (private) at line 1
  - `verify_zero_dead_code_edge_cases` (private) at line 1
  - `verify_closure_and_dynamic_dispatch` (private) at line 1
  - `test_coverage_integration` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.17

**TDG Severity:** Normal

### ./server/src/tests/deep_context_simplified_tests.rs

**Language:** rust
**Total Symbols:** 16
**Functions:** 10 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 6

**Functions:**
  - `test_deep_context_config_default_values` (private) at line 1
  - `test_deep_context_analyzer_creation` (private) at line 1
  - `test_metadata_creation` (private) at line 1
  - `test_quality_scorecard_calculations` (private) at line 1
  - `test_defect_summary_aggregation` (private) at line 1
  - `test_prioritized_recommendations` (private) at line 1
  - `test_cross_language_references` (private) at line 1
  - `test_template_provenance_tracking` (private) at line 1
  - `test_analysis_type_equality` (private) at line 1
  - `test_enum_variants_complete` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.09

**TDG Severity:** Normal

### ./server/src/tests/deep_context_tests.rs

**Language:** rust
**Total Symbols:** 21
**Functions:** 13 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 8

**Functions:**
  - `test_deep_context_config_default_values (async)` (private) at line 1
  - `test_deep_context_analyzer_creation (async)` (private) at line 1
  - `test_discovery_simple_project (async)` (private) at line 1
  - `test_discovery_with_excludes (async)` (private) at line 1
  - `test_metadata_creation (async)` (private) at line 1
  - `test_quality_scorecard_calculations (async)` (private) at line 1
  - `test_defect_summary_aggregation (async)` (private) at line 1
  - `test_defect_hotspot_ranking (async)` (private) at line 1
  - `test_prioritized_recommendations (async)` (private) at line 1
  - `test_cross_language_references (async)` (private) at line 1
  - ... and 3 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.11

**TDG Severity:** Normal

### ./server/src/tests/demo_comprehensive_tests.rs

**Language:** rust
**Total Symbols:** 28
**Functions:** 17 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 11

**Functions:**
  - `test_demo_runner_creation (async)` (private) at line 1
  - `test_demo_step_structure (async)` (private) at line 1
  - `test_demo_report_structure (async)` (private) at line 1
  - `test_detect_repository_git_repo (async)` (private) at line 1
  - `test_detect_repository_cargo_project (async)` (private) at line 1
  - `test_detect_repository_nodejs_project (async)` (private) at line 1
  - `test_detect_repository_python_project (async)` (private) at line 1
  - `test_detect_repository_pyproject_toml (async)` (private) at line 1
  - `test_detect_repository_with_readme (async)` (private) at line 1
  - `test_detect_repository_empty_directory (async)` (private) at line 1
  - ... and 7 more functions

**Imports:** 11 import statements

**Technical Debt Gradient:** 1.16

**TDG Severity:** Normal

### ./server/src/tests/e2e_full_coverage.rs

**Language:** rust
**Total Symbols:** 14
**Functions:** 9 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `test_mcp_server_e2e_coverage (async)` (private) at line 1
  - `test_cli_main_binary_version` (private) at line 1
  - `test_cli_main_binary_help` (private) at line 1
  - `test_cli_subcommand_help` (private) at line 1
  - `test_cli_mode_list_templates` (private) at line 1
  - `test_cli_generate_validation_error` (private) at line 1
  - `test_cli_search_templates` (private) at line 1
  - `test_cli_invalid_command` (private) at line 1
  - `test_cli_analyze_churn` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.13

**TDG Severity:** Normal

### ./server/src/tests/error.rs

**Language:** rust
**Total Symbols:** 12
**Functions:** 10 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Functions:**
  - `test_template_not_found_error` (private) at line 1
  - `test_invalid_uri_error` (private) at line 1
  - `test_validation_error` (private) at line 1
  - `test_render_error` (private) at line 1
  - `test_not_found_error` (private) at line 1
  - `test_s3_error` (private) at line 1
  - `test_invalid_utf8_error` (private) at line 1
  - `test_cache_error` (private) at line 1
  - `test_json_error` (private) at line 1
  - `test_io_error` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.01

**TDG Severity:** Normal

### ./server/src/tests/error_handling.rs

**Language:** rust
**Total Symbols:** 18
**Functions:** 15 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `test_template_error_display` (private) at line 1
  - `test_template_not_found_error` (private) at line 1
  - `test_not_found_error` (private) at line 1
  - `test_render_error` (private) at line 1
  - `test_validation_error` (private) at line 1
  - `test_invalid_utf8_error` (private) at line 1
  - `test_error_to_mcp_code` (private) at line 1
  - `test_parameter_spec_creation` (private) at line 1
  - `test_parameter_spec_with_default` (private) at line 1
  - `test_error_debug_representation` (private) at line 1
  - ... and 5 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.13

**TDG Severity:** Normal

### ./server/src/tests/fixtures/sample.js

**Language:** javascript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.66

**TDG Severity:** Normal

### ./server/src/tests/fixtures/sample.py

**Language:** python
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.70

**TDG Severity:** Normal

### ./server/src/tests/fixtures/sample.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.68

**TDG Severity:** Normal

### ./server/src/tests/helpers.rs

**Language:** rust
**Total Symbols:** 13
**Functions:** 10 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `test_snake_case_helper` (private) at line 1
  - `test_kebab_case_helper` (private) at line 1
  - `test_pascal_case_helper` (private) at line 1
  - `test_current_year_helper` (private) at line 1
  - `test_current_date_helper` (private) at line 1
  - `test_helper_error_handling` (private) at line 1
  - `test_empty_string_handling` (private) at line 1
  - `test_helper_with_non_string_parameter` (private) at line 1
  - `test_pascal_case_preserves_existing_capitalization` (private) at line 1
  - `test_year_and_date_helpers_consistency` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.18

**TDG Severity:** Normal

### ./server/src/tests/http_adapter_tests.rs

**Language:** rust
**Total Symbols:** 17
**Functions:** 11 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 6

**Functions:**
  - `test_http_adapter_creation (async)` (private) at line 1
  - `test_http_adapter_bind (async)` (private) at line 1
  - `test_http_output_creation (async)` (private) at line 1
  - `test_http_context_creation (async)` (private) at line 1
  - `test_http_context_with_no_remote_addr (async)` (private) at line 1
  - `test_http_context_with_no_user_agent (async)` (private) at line 1
  - `test_http_context_empty (async)` (private) at line 1
  - `test_protocol_adapter_trait` (private) at line 1
  - `test_http_status_code_variations (async)` (private) at line 1
  - `test_http_response_with_json (async)` (private) at line 1
  - ... and 1 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.23

**TDG Severity:** Normal

### ./server/src/tests/kaizen_reliability_patterns.rs

**Language:** rust
**Total Symbols:** 23
**Functions:** 13 | **Structs:** 2 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 8

**Functions:**
  - `kaizen_retry (async)` (public) at line 1
  - `poka_yoke_timeout (async)` (public) at line 1
  - `fast_assert_eq` (public) at line 1
  - `should_skip_expensive_operation` (public) at line 1
  - `create_minimal_test_data` (public) at line 1
  - `test_kaizen_retry_succeeds_first_attempt (async)` (private) at line 1
  - `test_kaizen_retry_succeeds_after_retries (async)` (private) at line 1
  - `test_poka_yoke_timeout_success (async)` (private) at line 1
  - `test_poka_yoke_timeout_failure (async)` (private) at line 1
  - `test_jidoka_test_setup_cleanup` (private) at line 1
  - ... and 3 more functions

**Structs:**
  - `JidokaTestSetup` (public) with 1 field at line 1
  - `TestStateInspector` (public) with 3 fields at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.48

**TDG Severity:** Normal

### ./server/src/tests/kaizen_test_optimizations.rs

**Language:** rust
**Total Symbols:** 24
**Functions:** 10 | **Structs:** 5 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 8

**Functions:**
  - `fast_unit_test_setup` (public) at line 1
  - `fast_temp_dir` (public) at line 1
  - `generate_test_data` (public) at line 1
  - `fast_proptest_config` (public) at line 1
  - `small_string_strategy` (public) at line 1
  - `small_vec_strategy` (public) at line 1
  - `test_kaizen_runner_tracks_metrics (async)` (private) at line 1
  - `test_fast_temp_dir_creation` (private) at line 1
  - `test_mock_heavy_operation_performance` (private) at line 1
  - `test_concurrent_test_execution (async)` (private) at line 1

**Structs:**
  - `TestMetrics` (public) with 5 fields (derives: derive) at line 1
  - `SlowTest` (public) with 3 fields (derives: derive) at line 1
  - `FlakyTest` (public) with 3 fields (derives: derive) at line 1
  - `KaizenTestRunner` (public) with 2 fields at line 1
  - `MockHeavyOperation` (public) with 0 fields at line 1

**Enums:**
  - `TestCategory` (public) with 4 variants at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.26

**TDG Severity:** Normal

### ./server/src/tests/lib.rs

**Language:** rust
**Total Symbols:** 21
**Functions:** 12 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 9

**Functions:**
  - `test_template_server_new (async)` (private) at line 1
  - `test_template_server_trait_implementation (async)` (private) at line 1
  - `test_template_server_deprecated_methods (async)` (private) at line 1
  - `test_warm_cache (async)` (private) at line 1
  - `test_template_server_cache_initialization (async)` (private) at line 1
  - `test_template_server_cache_sizes (async)` (private) at line 1
  - `test_warm_cache_templates (async)` (private) at line 1
  - `test_template_server_trait_via_methods (async)` (private) at line 1
  - `test_type_aliases` (private) at line 1
  - `test_s3_client_instantiation` (private) at line 1
  - ... and 2 more functions

**Imports:** 9 import statements

**Technical Debt Gradient:** 1.10

**TDG Severity:** Normal

### ./server/src/tests/mcp_protocol.rs

**Language:** rust
**Total Symbols:** 15
**Functions:** 10 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `create_test_server` (private) at line 1
  - `create_request` (private) at line 1
  - `test_handle_initialize (async)` (private) at line 1
  - `test_handle_list_tools (async)` (private) at line 1
  - `test_handle_list_resources (async)` (private) at line 1
  - `test_handle_call_tool_generate_template (async)` (private) at line 1
  - `test_handle_call_tool_invalid_tool (async)` (private) at line 1
  - `test_handle_call_tool_missing_parameters (async)` (private) at line 1
  - `test_handle_invalid_method (async)` (private) at line 1
  - `test_protocol_version_default (async)` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.10

**TDG Severity:** Normal

### ./server/src/tests/models.rs

**Language:** rust
**Total Symbols:** 7
**Functions:** 4 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `test_toolchain_priority` (private) at line 1
  - `test_toolchain_as_str` (private) at line 1
  - `test_template_category_serialization` (private) at line 1
  - `test_parameter_type_serialization` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.03

**TDG Severity:** Normal

### ./server/src/tests/project_meta_integration_test.rs

**Language:** rust
**Total Symbols:** 11
**Functions:** 4 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 7

**Functions:**
  - `test_full_metadata_compression_pipeline (async)` (private) at line 1
  - `test_metadata_integration (async)` (private) at line 1
  - `create_test_project` (private) at line 1
  - `create_minimal_project` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.15

**TDG Severity:** Normal

### ./server/src/tests/prompts.rs

**Language:** rust
**Total Symbols:** 14
**Functions:** 9 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `create_test_server` (private) at line 1
  - `create_request` (private) at line 1
  - `test_handle_prompts_list (async)` (private) at line 1
  - `test_handle_prompt_get_rust_project (async)` (private) at line 1
  - `test_handle_prompt_get_deno_project (async)` (private) at line 1
  - `test_handle_prompt_get_python_project (async)` (private) at line 1
  - `test_handle_prompt_get_missing_params (async)` (private) at line 1
  - `test_handle_prompt_get_invalid_params (async)` (private) at line 1
  - `test_handle_prompt_get_unknown_prompt (async)` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.08

**TDG Severity:** Normal

### ./server/src/tests/resources.rs

**Language:** rust
**Total Symbols:** 13
**Functions:** 8 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `create_test_server` (private) at line 1
  - `create_request` (private) at line 1
  - `test_handle_resource_list (async)` (private) at line 1
  - `test_handle_resource_read_success (async)` (private) at line 1
  - `test_handle_resource_read_missing_params (async)` (private) at line 1
  - `test_handle_resource_read_invalid_params (async)` (private) at line 1
  - `test_handle_resource_read_not_found (async)` (private) at line 1
  - `test_handle_resource_read_all_templates (async)` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.12

**TDG Severity:** Normal

### ./server/src/tests/template_rendering.rs

**Language:** rust
**Total Symbols:** 11
**Functions:** 9 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Functions:**
  - `test_render_rust_cli_makefile` (private) at line 1
  - `test_render_python_uv_makefile` (private) at line 1
  - `test_render_deno_typescript_makefile` (private) at line 1
  - `test_render_readme_template` (private) at line 1
  - `test_render_gitignore_template` (private) at line 1
  - `test_render_with_conditionals` (private) at line 1
  - `test_render_with_missing_parameters` (private) at line 1
  - `test_render_with_nested_loops` (private) at line 1
  - `test_render_with_string_helpers` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.01

**TDG Severity:** Normal

### ./server/src/tests/template_resources.rs

**Language:** rust
**Total Symbols:** 12
**Functions:** 10 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Functions:**
  - `create_test_server` (private) at line 1
  - `test_list_all_templates (async)` (private) at line 1
  - `test_filter_templates_by_prefix (async)` (private) at line 1
  - `test_get_template_metadata (async)` (private) at line 1
  - `test_get_template_content (async)` (private) at line 1
  - `test_invalid_template_uri (async)` (private) at line 1
  - `test_template_categories (async)` (private) at line 1
  - `test_template_toolchains (async)` (private) at line 1
  - `test_template_parameter_types (async)` (private) at line 1
  - `test_rust_template_parameters (async)` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.18

**TDG Severity:** Normal

### ./server/src/tests/tools.rs

**Language:** rust
**Total Symbols:** 22
**Functions:** 17 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `create_test_server` (private) at line 1
  - `create_request` (private) at line 1
  - `test_handle_tool_call_missing_params (async)` (private) at line 1
  - `test_handle_tool_call_invalid_params (async)` (private) at line 1
  - `test_list_templates_all (async)` (private) at line 1
  - `test_list_templates_by_toolchain (async)` (private) at line 1
  - `test_list_templates_by_category (async)` (private) at line 1
  - `test_validate_template_valid (async)` (private) at line 1
  - `test_validate_template_missing_required (async)` (private) at line 1
  - `test_validate_template_unknown_parameter (async)` (private) at line 1
  - ... and 7 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.25

**TDG Severity:** Normal

### ./server/src/tests/unified_protocol_tests.rs

**Language:** rust
**Total Symbols:** 18
**Functions:** 13 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `test_unified_service_creation (async)` (private) at line 1
  - `test_app_state_default (async)` (private) at line 1
  - `test_service_metrics_creation (async)` (private) at line 1
  - `test_service_metrics_increment (async)` (private) at line 1
  - `test_service_metrics_duration_tracking (async)` (private) at line 1
  - `test_service_metrics_error_tracking (async)` (private) at line 1
  - `test_protocol_context_http_only` (private) at line 1
  - `test_protocol_context_mcp_only` (private) at line 1
  - `test_app_error_types` (private) at line 1
  - `test_app_error_status_codes` (private) at line 1
  - ... and 3 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.10

**TDG Severity:** Normal

### ./server/src/unified_protocol/adapters/cli.rs

**Language:** rust
**Total Symbols:** 35
**Functions:** 20 | **Structs:** 3 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 11

**Functions:**
  - `format_to_string` (private) at line 1
  - `churn_format_to_string` (private) at line 1
  - `complexity_format_to_string` (private) at line 1
  - `dag_type_to_string` (private) at line 1
  - `dead_code_format_to_string` (private) at line 1
  - `satd_format_to_string` (private) at line 1
  - `satd_severity_to_string` (private) at line 1
  - `deep_context_format_to_string` (private) at line 1
  - `deep_context_dag_type_to_string` (private) at line 1
  - `deep_context_cache_strategy_to_string` (private) at line 1
  - ... and 10 more functions

**Structs:**
  - `CliAdapter` (public) with 0 fields at line 1
  - `CliInput` (public) with 3 fields at line 1
  - `CliRunner` (public) with 1 field at line 1

**Enums:**
  - `CliOutput` (public) with 2 variants at line 1

**Imports:** 11 import statements

**Technical Debt Gradient:** 1.83

**TDG Severity:** Warning

### ./server/src/unified_protocol/adapters/http.rs

**Language:** rust
**Total Symbols:** 32
**Functions:** 12 | **Structs:** 4 | **Enums:** 2 | **Traits:** 1 | **Impls:** 0 | **Modules:** 0 | **Imports:** 13

**Functions:**
  - `handle_connection (async)` (private) at line 1
  - `process_http_request (async)` (private) at line 1
  - `convert_hyper_to_http_input (async)` (private) at line 1
  - `collect_request_body (async)` (private) at line 1
  - `decode_http_input (async)` (private) at line 1
  - `handle_unified_request (async)` (private) at line 1
  - `encode_unified_response (async)` (private) at line 1
  - `serve_http_connection (async)` (private) at line 1
  - `test_http_adapter_creation` (private) at line 1
  - `test_http_response_builder (async)` (private) at line 1
  - ... and 2 more functions

**Structs:**
  - `HttpAdapter` (public) with 2 fields at line 1
  - `HttpStreamAdapter` (public) with 2 fields at line 1
  - `HttpServer` (public) with 2 fields at line 1
  - `HttpResponseBuilder` (public) with 0 fields at line 1

**Enums:**
  - `HttpInput` (public) with 2 variants at line 1
  - `HttpOutput` (public) with 1 variant at line 1

**Traits:**
  - `HttpServiceHandler` (public) at line 1

**Imports:** 13 import statements

**Technical Debt Gradient:** 1.36

**TDG Severity:** Normal

### ./server/src/unified_protocol/adapters/mcp.rs

**Language:** rust
**Total Symbols:** 23
**Functions:** 7 | **Structs:** 5 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 10

**Functions:**
  - `test_json_rpc_request_creation` (private) at line 1
  - `test_json_rpc_notification` (private) at line 1
  - `test_json_rpc_response_success` (private) at line 1
  - `test_json_rpc_response_error` (private) at line 1
  - `test_mcp_adapter_decode (async)` (private) at line 1
  - `test_mcp_adapter_encode_success (async)` (private) at line 1
  - `test_standard_json_rpc_errors` (private) at line 1

**Structs:**
  - `McpAdapter` (public) with 1 field at line 1
  - `JsonRpcRequest` (public) with 4 fields (derives: derive) at line 1
  - `JsonRpcResponse` (public) with 4 fields (derives: derive) at line 1
  - `JsonRpcError` (public) with 3 fields (derives: derive) at line 1
  - `McpReader` (public) with 1 field at line 1

**Enums:**
  - `McpInput` (public) with 2 variants at line 1

**Imports:** 10 import statements

**Technical Debt Gradient:** 1.20

**TDG Severity:** Normal

### ./server/src/unified_protocol/adapters/mod.rs

**Language:** rust
**Total Symbols:** 3
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 0.96

**TDG Severity:** Normal

### ./server/src/unified_protocol/error.rs

**Language:** rust
**Total Symbols:** 20
**Functions:** 8 | **Structs:** 3 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 8

**Functions:**
  - `extract_protocol_from_context` (private) at line 1
  - `set_protocol_context` (public) at line 1
  - `clear_protocol_context` (public) at line 1
  - `test_app_error_status_codes` (private) at line 1
  - `test_mcp_error_codes` (private) at line 1
  - `test_error_types` (private) at line 1
  - `test_protocol_context (async)` (private) at line 1
  - `test_error_to_protocol_response (async)` (private) at line 1

**Structs:**
  - `McpError` (public) with 3 fields (derives: derive) at line 1
  - `HttpErrorResponse` (public) with 3 fields (derives: derive) at line 1
  - `CliErrorResponse` (public) with 3 fields (derives: derive) at line 1

**Enums:**
  - `AppError` (public) with 14 variants at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.22

**TDG Severity:** Normal

### ./server/src/unified_protocol/mod.rs

**Language:** rust
**Total Symbols:** 24
**Functions:** 3 | **Structs:** 7 | **Enums:** 2 | **Traits:** 1 | **Impls:** 0 | **Modules:** 0 | **Imports:** 11

**Functions:**
  - `test_unified_request_creation` (private) at line 1
  - `test_unified_response_creation` (private) at line 1
  - `test_protocol_display` (private) at line 1

**Structs:**
  - `UnifiedRequest` (public) with 6 fields (derives: derive) at line 1
  - `UnifiedResponse` (public) with 4 fields (derives: derive) at line 1
  - `AdapterRegistry` (public) with 1 field (derives: derive) at line 1
  - `AdapterWrapper` (private) with 1 field at line 1
  - `McpContext` (public) with 2 fields (derives: derive) at line 1
  - ... and 2 more structs

**Enums:**
  - `Protocol` (public) with 4 variants at line 1
  - `ProtocolError` (public) with 7 variants at line 1

**Traits:**
  - `ProtocolAdapter` (public) at line 1

**Imports:** 11 import statements

**Technical Debt Gradient:** 1.24

**TDG Severity:** Normal

### ./server/src/unified_protocol/service.rs

**Language:** rust
**Total Symbols:** 86
**Functions:** 18 | **Structs:** 38 | **Enums:** 0 | **Traits:** 2 | **Impls:** 0 | **Modules:** 0 | **Imports:** 28

**Functions:**
  - `list_templates (async)` (public) at line 1
  - `get_template (async)` (public) at line 1
  - `generate_template (async)` (public) at line 1
  - `analyze_complexity (async)` (public) at line 1
  - `analyze_complexity_get (async)` (public) at line 1
  - `analyze_churn (async)` (public) at line 1
  - `analyze_dag (async)` (public) at line 1
  - `generate_context (async)` (public) at line 1
  - `analyze_dead_code (async)` (public) at line 1
  - `analyze_deep_context (async)` (public) at line 1
  - ... and 8 more functions

**Structs:**
  - `UnifiedService` (public) with 3 fields (derives: derive) at line 1
  - `AppState` (public) with 3 fields (derives: derive) at line 1
  - `ServiceMetrics` (public) with 3 fields (derives: derive) at line 1
  - `DefaultTemplateService` (public) with 0 fields (derives: derive) at line 1
  - `DefaultAnalysisService` (public) with 0 fields (derives: derive) at line 1
  - ... and 33 more structs

**Traits:**
  - `TemplateService` (public) at line 1
  - `AnalysisService` (public) at line 1

**Imports:** 28 import statements

**Technical Debt Gradient:** 1.45

**TDG Severity:** Normal

### ./server/src/unified_protocol/test_harness.rs

**Language:** rust
**Total Symbols:** 22
**Functions:** 5 | **Structs:** 4 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 12

**Functions:**
  - `test_harness_creation (async)` (private) at line 1
  - `test_template_generation_endpoint (async)` (private) at line 1
  - `test_error_handling_consistency (async)` (private) at line 1
  - `test_protocol_equivalence (async)` (private) at line 1
  - `test_suite_results` (private) at line 1

**Structs:**
  - `TestHarness` (public) with 5 fields at line 1
  - `TestResults` (public) with 5 fields (derives: derive) at line 1
  - `EquivalenceFailure` (public) with 5 fields (derives: derive) at line 1
  - `TestSuiteResults` (public) with 3 fields (derives: derive) at line 1

**Enums:**
  - `TestError` (public) with 8 variants at line 1

**Imports:** 12 import statements

**Technical Debt Gradient:** 1.96

**TDG Severity:** Warning

### ./server/src/utils/helpers.rs

**Language:** rust
**Total Symbols:** 22
**Functions:** 18 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Functions:**
  - `snake_case_helper` (public) at line 1
  - `kebab_case_helper` (public) at line 1
  - `pascal_case_helper` (public) at line 1
  - `current_year_helper` (public) at line 1
  - `current_date_helper` (public) at line 1
  - `to_snake_case` (private) at line 1
  - `to_kebab_case` (private) at line 1
  - `to_pascal_case` (private) at line 1
  - `test_to_snake_case_basic` (private) at line 1
  - `test_to_snake_case_edge_cases` (private) at line 1
  - ... and 8 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.15

**TDG Severity:** Normal

### ./server/src/utils/mod.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.96

**TDG Severity:** Normal

### ./server/templates/gitignore/deno/cli.json

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.64

**TDG Severity:** Normal

### ./server/templates/gitignore/python-uv/cli.json

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.64

**TDG Severity:** Normal

### ./server/templates/gitignore/rust/cli.json

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.71

**TDG Severity:** Normal

### ./server/templates/makefile/deno/cli.json

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.66

**TDG Severity:** Normal

### ./server/templates/makefile/python-uv/cli.json

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.67

**TDG Severity:** Normal

### ./server/templates/makefile/rust/cli.json

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.68

**TDG Severity:** Normal

### ./server/templates/readme/deno/cli.json

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.67

**TDG Severity:** Normal

### ./server/templates/readme/python-uv/cli.json

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.67

**TDG Severity:** Normal

### ./server/templates/readme/rust/cli.json

**Language:** unknown
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.71

**TDG Severity:** Normal

### ./server/test_regex.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.02

**TDG Severity:** Normal

### ./server/tests/ast_dag_mermaid_pipeline.rs

**Language:** rust
**Total Symbols:** 11
**Functions:** 8 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `create_test_rust_project` (private) at line 1
  - `test_ast_to_dag_metadata_propagation (async)` (private) at line 1
  - `test_pipeline_determinism (async)` (private) at line 1
  - `test_pipeline_with_complex_project (async)` (private) at line 1
  - `test_edge_budget_enforcement (async)` (private) at line 1
  - `test_individual_file_analysis (async)` (private) at line 1
  - `test_mermaid_output_quality (async)` (private) at line 1
  - `test_metadata_serialization` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.19

**TDG Severity:** Normal

### ./server/tests/bin_integration.rs

**Language:** rust
**Total Symbols:** 6
**Functions:** 4 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Functions:**
  - `test_binary_version_flag` (private) at line 1
  - `test_binary_json_rpc_initialize` (private) at line 1
  - `test_binary_invalid_json` (private) at line 1
  - `test_binary_multiple_requests` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.19

**TDG Severity:** Normal

### ./server/tests/cli_comprehensive_integration.rs

**Language:** rust
**Total Symbols:** 36
**Functions:** 29 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 6

**Functions:**
  - `test_generate_makefile_e2e` (private) at line 1
  - `test_generate_missing_required_params` (private) at line 1
  - `test_generate_invalid_template_uri` (private) at line 1
  - `test_generate_to_stdout` (private) at line 1
  - `test_scaffold_parallel_generation` (private) at line 1
  - `test_list_json_output_schema` (private) at line 1
  - `test_list_table_output` (private) at line 1
  - `test_list_yaml_output` (private) at line 1
  - `test_list_filtered_by_toolchain` (private) at line 1
  - `test_list_filtered_by_category` (private) at line 1
  - ... and 19 more functions

**Structs:**
  - `ErrorCase` (private) with 2 fields at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.65

**TDG Severity:** Warning

### ./server/tests/cli_documentation_sync.rs

**Language:** rust
**Total Symbols:** 14
**Functions:** 8 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `parse_documented_cli_commands` (private) at line 1
  - `parse_cli_help_output` (private) at line 1
  - `get_binary_path` (private) at line 1
  - `test_cli_commands_match_documentation` (private) at line 1
  - `test_cli_subcommands_match_documentation` (private) at line 1
  - `test_cli_options_match_documentation` (private) at line 1
  - `test_no_undocumented_commands` (private) at line 1
  - `test_documentation_examples_are_valid` (private) at line 1

**Structs:**
  - `DocumentedCommand` (private) with 5 fields (derives: derive) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 2.34

**TDG Severity:** Warning

### ./server/tests/complexity_metrics.rs

**Language:** rust
**Total Symbols:** 24
**Functions:** 15 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 9

**Functions:**
  - `test_complexity_metrics_creation` (private) at line 1
  - `test_complexity_metrics_default` (private) at line 1
  - `test_function_complexity_creation` (private) at line 1
  - `test_class_complexity_creation` (private) at line 1
  - `test_file_complexity_metrics_creation` (private) at line 1
  - `test_compute_complexity_cache_key` (private) at line 1
  - `test_aggregate_results_empty` (private) at line 1
  - `test_aggregate_results_with_data` (private) at line 1
  - `test_format_complexity_summary` (private) at line 1
  - `test_format_complexity_report` (private) at line 1
  - ... and 5 more functions

**Imports:** 9 import statements

**Technical Debt Gradient:** 1.12

**TDG Severity:** Normal

### ./server/tests/config_integration.rs

**Language:** rust
**Total Symbols:** 11
**Functions:** 5 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 6

**Functions:**
  - `test_config_loading_from_file (async)` (private) at line 1
  - `test_config_hot_reload (async)` (private) at line 1
  - `test_config_default_values (async)` (private) at line 1
  - `test_config_accessor_methods (async)` (private) at line 1
  - `test_invalid_config_file` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.18

**TDG Severity:** Normal

### ./server/tests/demo_core_extraction.rs

**Language:** rust
**Total Symbols:** 14
**Functions:** 6 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 8

**Functions:**
  - `test_demo_runner_as_library (async)` (private) at line 1
  - `test_config_manager_as_library (async)` (private) at line 1
  - `test_export_service_as_library (async)` (private) at line 1
  - `test_programmatic_demo_with_custom_config (async)` (private) at line 1
  - `test_export_formats_discovery` (private) at line 1
  - `test_end_to_end_library_usage (async)` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.25

**TDG Severity:** Normal

### ./server/tests/demo_e2e_integration.rs

**Language:** rust
**Total Symbols:** 24
**Functions:** 9 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 14

**Functions:**
  - `create_test_repository` (private) at line 1
  - `test_demo_server_happy_path (async)` (private) at line 1
  - `test_api_contract_compliance (async)` (private) at line 1
  - `test_concurrent_requests (async)` (private) at line 1
  - `test_performance_assertions (async)` (private) at line 1
  - `test_error_handling (async)` (private) at line 1
  - `test_analysis_pipeline_integrity (async)` (private) at line 1
  - `test_data_source_indicators (async)` (private) at line 1
  - `test_mermaid_diagram_rendering (async)` (private) at line 1

**Structs:**
  - `DemoServer` (private) with 3 fields at line 1

**Imports:** 14 import statements

**Technical Debt Gradient:** 2.50

**TDG Severity:** Critical

### ./server/tests/demo_integration.rs

**Language:** rust
**Total Symbols:** 13
**Functions:** 6 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 7

**Functions:**
  - `test_demo_mode_in_test_directory` (private) at line 1
  - `test_demo_mode_with_json_output` (private) at line 1
  - `test_demo_mode_with_specific_path` (private) at line 1
  - `test_demo_increases_test_coverage` (private) at line 1
  - `test_demo_runner_execution (async)` (private) at line 1
  - `test_repository_detection` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.24

**TDG Severity:** Normal

### ./server/tests/demo_web_integration.rs

**Language:** rust
**Total Symbols:** 13
**Functions:** 8 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `test_demo_server_startup_and_shutdown (async)` (private) at line 1
  - `test_demo_server_api_endpoints (async)` (private) at line 1
  - `test_demo_server_static_assets (async)` (private) at line 1
  - `test_demo_server_concurrent_requests (async)` (private) at line 1
  - `test_demo_server_response_headers (async)` (private) at line 1
  - `test_demo_content_rendering (async)` (private) at line 1
  - `test_demo_server_starts (async)` (private) at line 1
  - `test_demo_content_from_analysis` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.28

**TDG Severity:** Normal

### ./server/tests/determinism_tests.rs

**Language:** rust
**Total Symbols:** 38
**Functions:** 17 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 21

**Functions:**
  - `test_unified_ast_engine_determinism (async)` (private) at line 1
  - `test_mermaid_generation_determinism` (private) at line 1
  - `test_dogfooding_engine_determinism (async)` (private) at line 1
  - `test_artifact_writer_determinism (async)` (private) at line 1
  - `test_pagerank_numerical_stability` (private) at line 1
  - `test_hash_collision_resistance` (private) at line 1
  - `test_file_ordering_stability (async)` (private) at line 1
  - `test_edge_case_determinism` (private) at line 1
  - `test_concurrent_generation_determinism (async)` (private) at line 1
  - `create_test_project (async)` (private) at line 1
  - ... and 7 more functions

**Imports:** 21 import statements

**Technical Debt Gradient:** 1.81

**TDG Severity:** Warning

### ./server/tests/documentation_examples.rs

**Language:** rust
**Total Symbols:** 24
**Functions:** 20 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Functions:**
  - `get_binary_path` (private) at line 1
  - `test_cli_examples_are_valid` (private) at line 1
  - `process_bash_code_block` (private) at line 1
  - `should_skip_line` (private) at line 1
  - `has_complex_shell_features` (private) at line 1
  - `is_non_toolkit_command` (private) at line 1
  - `handle_multiline_command` (private) at line 1
  - `validate_command` (private) at line 1
  - `validate_binary_path` (private) at line 1
  - `validate_command_arguments` (private) at line 1
  - ... and 10 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.78

**TDG Severity:** Warning

### ./server/tests/e2e/installation.test.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 0.98

**TDG Severity:** Normal

### ./server/tests/e2e/mcp_protocol.test.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Technical Debt Gradient:** 1.27

**TDG Severity:** Normal

### ./server/tests/enhanced_dag_integration.rs

**Language:** rust
**Total Symbols:** 8
**Functions:** 4 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Functions:**
  - `test_enhanced_dag_analysis` (private) at line 1
  - `test_enhanced_analysis_backward_compatibility` (private) at line 1
  - `test_enhanced_flags_combinations` (private) at line 1
  - `test_enhanced_dag_help` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.16

**TDG Severity:** Normal

### ./server/tests/execution_mode.rs

**Language:** rust
**Total Symbols:** 18
**Functions:** 11 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 7

**Functions:**
  - `detect_execution_mode_test` (private) at line 1
  - `test_execution_mode_detection_with_mcp_version` (private) at line 1
  - `test_execution_mode_detection_without_mcp_version` (private) at line 1
  - `test_env_filter_creation` (private) at line 1
  - `test_server_creation_logic` (private) at line 1
  - `test_mcp_version_environment_variable` (private) at line 1
  - `test_argument_count_behavior` (private) at line 1
  - `test_async_runtime_setup (async)` (private) at line 1
  - `test_tracing_initialization` (private) at line 1
  - `test_terminal_detection` (private) at line 1
  - ... and 1 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.11

**TDG Severity:** Normal

### ./server/tests/export_integration.rs

**Language:** rust
**Total Symbols:** 20
**Functions:** 10 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 10

**Functions:**
  - `create_test_export_report` (private) at line 1
  - `create_test_churn_analysis` (private) at line 1
  - `test_markdown_export` (private) at line 1
  - `test_json_export_pretty` (private) at line 1
  - `test_json_export_compact` (private) at line 1
  - `test_sarif_export` (private) at line 1
  - `test_export_service` (private) at line 1
  - `test_export_service_save_to_file` (private) at line 1
  - `test_create_export_report` (private) at line 1
  - `test_export_without_optional_data` (private) at line 1

**Imports:** 10 import statements

**Technical Debt Gradient:** 1.17

**TDG Severity:** Normal

### ./server/tests/fixtures/test_artifacts.rs

**Language:** rust
**Total Symbols:** 6
**Functions:** 4 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Functions:**
  - `create_test_artifacts` (public) at line 1
  - `create_rust_artifacts` (private) at line 1
  - `create_python_artifacts` (private) at line 1
  - `create_typescript_artifacts` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 2.41

**TDG Severity:** Warning

### ./server/tests/generate_mermaid_example.rs

**Language:** rust
**Total Symbols:** 5
**Functions:** 1 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Functions:**
  - `generate_example_mermaid_diagram` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.06

**TDG Severity:** Normal

### ./server/tests/generate_mermaid_test.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 1 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `generate_test_mermaid` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.04

**TDG Severity:** Normal

### ./server/tests/git_clone_validation.rs

**Language:** rust
**Total Symbols:** 12
**Functions:** 10 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Functions:**
  - `test_github_url_parsing_comprehensive` (private) at line 1
  - `test_github_url_parsing_invalid` (private) at line 1
  - `test_cache_key_generation` (private) at line 1
  - `test_cache_key_uniqueness` (private) at line 1
  - `test_url_normalization` (private) at line 1
  - `test_security_boundaries` (private) at line 1
  - `test_fuzzer_identified_security_issues` (private) at line 1
  - `test_edge_case_handling` (private) at line 1
  - `test_clone_timeout (async)` (private) at line 1
  - `test_round_trip_parsing` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.43

**TDG Severity:** Normal

### ./server/tests/mcp_documentation_sync.rs

**Language:** rust
**Total Symbols:** 19
**Functions:** 8 | **Structs:** 3 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 8

**Functions:**
  - `parse_documented_mcp_tools` (private) at line 1
  - `get_binary_path` (private) at line 1
  - `send_mcp_request` (private) at line 1
  - `test_mcp_tools_match_documentation` (private) at line 1
  - `test_mcp_tool_schemas_match_documentation` (private) at line 1
  - `test_mcp_methods_match_documentation` (private) at line 1
  - `test_mcp_error_codes_are_complete` (private) at line 1
  - `test_no_undocumented_mcp_tools` (private) at line 1

**Structs:**
  - `DocumentedTool` (private) with 4 fields (derives: derive) at line 1
  - `McpResponse` (private) with 4 fields (derives: derive) at line 1
  - `ToolDefinition` (private) with 3 fields (derives: derive) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.86

**TDG Severity:** Warning

### ./server/tests/mermaid_artifact_tests.rs

**Language:** rust
**Total Symbols:** 23
**Functions:** 14 | **Structs:** 2 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 6

**Functions:**
  - `get_artifact_specs` (private) at line 1
  - `generate_simple_architecture` (private) at line 1
  - `generate_styled_workflow` (private) at line 1
  - `generate_ast_simple` (private) at line 1
  - `generate_ast_styled` (private) at line 1
  - `validate_simple_diagram` (private) at line 1
  - `validate_styled_diagram` (private) at line 1
  - `validate_ast_diagram` (private) at line 1
  - `validate_complexity_styled` (private) at line 1
  - `test_generate_all_artifacts` (private) at line 1
  - ... and 4 more functions

**Structs:**
  - `MermaidArtifactSpec` (public) with 5 fields (derives: derive) at line 1
  - `DiagramMetrics` (private) with 4 fields (derives: derive) at line 1

**Enums:**
  - `ArtifactCategory` (public) with 4 variants at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.75

**TDG Severity:** Warning

### ./server/tests/mermaid_empty_bug_fix_test.rs

**Language:** rust
**Total Symbols:** 9
**Functions:** 6 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `test_regression_empty_nodes_bug` (private) at line 1
  - `test_mermaid_label_escaping` (private) at line 1
  - `test_node_types_have_labels` (private) at line 1
  - `test_complexity_styled_diagram_has_labels` (private) at line 1
  - `test_empty_graph_doesnt_crash` (private) at line 1
  - `test_special_characters_in_node_ids` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1

**Technical Debt Gradient:** 1.26

**TDG Severity:** Normal

### ./server/tests/services_integration.rs

**Language:** rust
**Total Symbols:** 26
**Functions:** 9 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 17

**Functions:**
  - `test_execution_mode_detection` (private) at line 1
  - `test_cli_run_generate_command (async)` (private) at line 1
  - `test_ast_rust_analysis (async)` (private) at line 1
  - `test_complexity_service` (private) at line 1
  - `test_dag_builder` (private) at line 1
  - `test_mcp_handlers (async)` (private) at line 1
  - `test_binary_main_logic (async)` (private) at line 1
  - `test_cli_functions (async)` (private) at line 1
  - `test_ast_error_handling (async)` (private) at line 1

**Imports:** 17 import statements

**Technical Debt Gradient:** 1.50

**TDG Severity:** Normal

## Complexity Hotspots

| Function | File | Cyclomatic | Cognitive |
|----------|------|------------|-----------|
| `ProjectFileDiscovery::categorize_file` | `./server/src/services/file_discovery.rs` | 20 | 24 |
| `handle_analyze_complexity` | `./server/src/cli/mod.rs` | 19 | 24 |
| `DemoReport::render_step_highlights` | `./server/src/demo/runner.rs` | 19 | 28 |
| `MakefileCompressor::detect_toolchain` | `./server/src/services/makefile_compressor.rs` | 19 | 20 |
| `validate_node_definitions` | `./server/src/services/mermaid_generator.rs` | 19 | 27 |
| `handle_analyze_satd` | `./server/src/cli/mod.rs` | 18 | 26 |
| `DeadCodeAnalyzer::build_reference_graph_from_dep_graph` | `./server/src/services/dead_code_analyzer.rs` | 18 | 29 |
| `LightweightProvabilityAnalyzer::compute_confidence` | `./server/src/services/lightweight_provability_analyzer.rs` | 18 | 19 |
| `mcp_endpoint` | `./server/src/unified_protocol/service.rs` | 18 | 18 |
| `format_prioritized_recommendations` | `./server/src/cli/mod.rs` | 17 | 39 |

## Code Churn Analysis

**Summary:**
- Total Commits: 1597
- Files Changed: 507

**Top Changed Files:**
| File | Commits | Authors |
|------|---------|---------|
| `server/Cargo.toml` | 103 | 2 |
| `Cargo.toml` | 84 | 2 |
| `Cargo.lock` | 70 | 3 |
| `assets/project-state.json` | 59 | 2 |
| `README.md` | 50 | 1 |
| `installer-macro/Cargo.toml` | 42 | 2 |
| `Makefile` | 32 | 1 |
| `server/src/cli/mod.rs` | 24 | 1 |
| `.github/workflows/automated-release.yml` | 20 | 1 |
| `CLAUDE.md` | 19 | 1 |

## Technical Debt Analysis

**SATD Summary:**

## Dead Code Analysis

**Summary:**
- Dead Functions: 0
- Total Dead Lines: 225

**Top Files with Dead Code:**
| File | Dead Lines | Dead Functions |
|------|------------|----------------|
| `./assets/demo/app.js` | 0 | 0 |
| `./assets/project-state.d.ts` | 0 | 0 |
| `./fuzz/fuzz_targets/fuzz_dag_builder.rs` | 0 | 0 |
| `./fuzz/fuzz_targets/fuzz_github_urls.rs` | 0 | 0 |
| `./fuzz/fuzz_targets/fuzz_mermaid_escaping.rs` | 0 | 0 |
| `./scripts/archive/cleanup-releases.ts` | 0 | 0 |
| `./scripts/archive/cleanup-test-artifacts.ts` | 0 | 0 |
| `./scripts/archive/dead-scripts/docker-setup.ts` | 0 | 0 |
| `./scripts/archive/dead-scripts/download-mermaid.ts` | 0 | 0 |
| `./scripts/archive/dead-scripts/mcp-install-deterministic.ts` | 0 | 0 |

## Defect Probability Analysis

**Risk Assessment:**
- Total Defects Predicted: 262
- Defect Density: 10.55 defects per 1000 lines

---
Generated by deep-context v0.21.0

