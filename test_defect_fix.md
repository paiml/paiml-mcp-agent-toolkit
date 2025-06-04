# Deep Context Analysis

## Executive Summary

Generated: 2025-06-04 14:32:27.365443203 UTC
Version: 0.21.0
Analysis Time: 4.16s
Cache Hit Rate: 0.0%

## Quality Scorecard

- **Overall Health**: ⚠️ (75.0/100)
- **Maintainability Index**: 70.0
- **Technical Debt**: 40.0 hours estimated

## Project Structure

```
└── /
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
    ├── .paiml-display.yaml
    ├── Cargo.toml
    ├── .gitignore
    ├── deep_context.md
    ├── CONTEXT_GENERATION_RELEASE_NOTES.md
    ├── RELEASE_NOTES.md
    ├── docs/
    │   ├── protocol-agnostic-demo-harness.md
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
    │   ├── speed-up-tests-spec.md
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
    │   ├── minimal-remote-github-spec.md
    │   ├── complexity-spec.md
    │   ├── bugs/
    │   │   ├── enhance-report-jun2-spec.md
    │   │   ├── demo-hot-fix-v20.md
    │   │   ├── mermaid-empty-bug.md
    │   │   ├── deep-context-satd-integration-bug.md
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
    │   ├── wip-current-code-smells-dogfood.md
    │   ├── rust-docs-spec.md
    │   ├── deterministic-graphs-mmd-spec.md
    │   ├── simplify-demo-spec.md
    │   ├── extend-web-protocol-spec.md
    │   ├── simplify-graph-query-spec.md
    │   ├── replace-make-context-spec.md
    │   ├── readme-dogfood-spec.md
    │   ├── demo-v2-spec.md
    │   └── archive/
    │       └── SPEC.md
    ├── .git/
    ├── deep_context_fixed.md
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
    ├── artifacts/
    │   ├── README.md
    │   ├── dogfooding/
    │   │   ├── complexity-2025-05-30.md
    │   │   ├── churn-2025-05-30.md
    │   │   ├── ast-context-2025-05-30.md
    │   │   ├── server-info-2025-05-30.md
    │   │   └── dag-2025-05-30.mmd
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
    ├── .idea/
    │   └── workspace.xml
    ├── Makefile
    ├── Cargo.lock
    ├── server/
    │   ├── assets/
    │   │   ├── demo/
    │   │   │   ├── style.min.css
    │   │   │   ├── app.min.js
    │   │   │   └── favicon.ico
    │   │   └── vendor/
    │   ├── post-complexity.json
    │   ├── proptest-regressions/
    │   │   └── services/
    │   │       └── mermaid_property_tests.txt
    │   ├── Cargo.toml
    │   ├── outdated-deps.json
    │   ├── .gitignore
    │   ├── src/
    │   │   ├── bin/
    │   │   │   └── pmat.rs
    │   │   ├── proptest-regressions/
    │   │   │   └── cli_property_tests.txt
    │   │   ├── demo/
    │   │   │   ├── adapters/
    │   │   │   │   ├── cli.rs
    │   │   │   │   ├── mod.rs
    │   │   │   │   ├── http.rs
    │   │   │   │   └── mcp.rs
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
    │   │   │   ├── prompts.rs
    │   │   │   ├── cli_tests.rs
    │   │   │   ├── binary_size.rs
    │   │   │   ├── ast_e2e.rs
    │   │   │   ├── cli_integration_full.rs
    │   │   │   ├── analyze_cli_tests.rs
    │   │   │   ├── template_resources.rs
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
    │   │   │   ├── cli_simple_tests.rs
    │   │   │   ├── error_handling.rs
    │   │   │   ├── mcp_protocol.rs
    │   │   │   ├── deep_context_simplified_tests.rs
    │   │   │   ├── demo_comprehensive_tests.rs
    │   │   │   ├── resources.rs
    │   │   │   ├── error.rs
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
    │   │   │   └── args.rs
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
    │   │   │   ├── tdg.rs
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
    │   │       ├── context.rs
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
    │   │       ├── git_analysis.rs
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
    │   │       ├── dag_builder.rs
    │   │       ├── canonical_query.rs
    │   │       ├── unified_ast_engine.rs
    │   │       ├── fixed_graph_builder.rs
    │   │       └── ast_python.rs
    │   ├── default_16411311994742027426_0_3168394.profraw
    │   ├── default_16411311994742027426_0_3168786.profraw
    │   ├── default_16411311994742027426_0_3168492.profraw
    │   ├── post-metrics.json
    │   ├── default_16411311994742027426_0_3168443.profraw
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
    │   │   ├── complexity_metrics.rs
    │   │   ├── cli_comprehensive_integration.rs
    │   │   ├── ast_dag_mermaid_pipeline.rs
    │   │   ├── e2e/
    │   │   │   ├── README.md
    │   │   │   ├── installation.test.ts
    │   │   │   └── mcp_protocol.test.ts
    │   │   ├── bin_integration.rs
    │   │   └── generate_mermaid_example.rs
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
    │   ├── Makefile
    │   ├── baseline-complexity-sorted.json
    │   ├── default_16411311994742027426_0_3168590.profraw
    │   ├── server/
    │   │   └── coverage/
    │   │       └── html/
    │   ├── default_16411311994742027426_0_3168688.profraw
    │   ├── scripts/
    │   │   ├── docker-setup.ts
    │   │   ├── test-mcp-e2e.ts
    │   │   └── test-installation.ts
    │   ├── baseline-dag.mmd
    │   ├── coverage/
    │   │   └── html/
    │   ├── deno.json
    │   ├── baseline-complexity.json
    │   ├── baseline-metrics.json
    │   ├── .build-info.json
    │   ├── installer.sh
    │   ├── baseline-metrics-no-meta.json
    │   ├── default_16411311994742027426_0_3168737.profraw
    │   └── .cargo/
    │       └── config.toml
    ├── scripts/
    │   ├── install.sh
    │   ├── dogfood-readme.ts
    │   ├── README.md
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
    │   ├── install.test.ts
    │   ├── validate-naming.ts
    │   ├── test-coverage-summary.md
    │   ├── run-fuzzing.ts
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
    ├── baseline-dag.mmd
    ├── baseline-complexity.json
    ├── SYMBOLIC_AI_RELEASE_NOTES.md
    ├── coverage_deno/
    │   ├── b0036dd6-7813-48c8-bd36-bb0f1696d1f5.json
    │   ├── cce7ae72-0d57-4b75-8927-16be4bed6990.json
    │   ├── lcov.info
    │   ├── a9fdf1f6-5869-463f-be84-b50638667f2d.json
    │   ├── 9b8bc7e8-42c2-4eee-b3ef-ab00e4c1fbfb.json
    │   ├── 3a0819a9-66cb-4336-9f15-99a07755b52c.json
    │   ├── 84f3c683-f9a5-4151-8eef-c9be2d6efc34.json
    │   ├── 7d8d90bd-3bd8-4c4f-b3ed-72188d5f3398.json
    │   ├── 3007ab93-8d2e-468e-8d40-5e30d1f56f82.json
    │   ├── 281cfb88-2d35-4a38-a873-e4eea1d44f1d.json
    │   ├── 49b164b4-2776-4173-8365-46ccf9c7ae0e.json
    │   ├── html/
    │   │   ├── mermaid-validator.ts.html
    │   │   ├── lib/
    │   │   │   ├── install-utils.ts.html
    │   │   │   ├── create-release-utils.ts.html
    │   │   │   └── index.html
    │   │   ├── validate-demo-assets.ts.html
    │   │   └── index.html
    │   ├── a2653698-7a71-44d7-8fd6-d5b90563d580.json
    │   ├── 633b7f83-dd80-4a66-a83f-d8116bb5a9b0.json
    │   ├── 97601ecf-5eea-4207-960d-7c99a77466e6.json
    │   ├── 66c96b75-6087-4dd1-ae2b-a03ec80837dc.json
    │   ├── 7271bb19-cc9e-4218-9e23-0d154fc7452a.json
    │   ├── a75d5b01-3b13-4807-9e0a-e635eaa3584f.json
    │   └── 83e1e2f5-d172-403b-801d-72f4d53195e2.json
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
    ├── fuzz/
    │   ├── Cargo.toml
    │   ├── fuzz_targets/
    │   │   ├── fuzz_mermaid_performance.rs
    │   │   ├── fuzz_dag_builder.rs
    │   │   ├── fuzz_mermaid_generation.rs
    │   │   └── fuzz_mermaid_escaping.rs
    │   ├── Cargo.lock
    │   └── target/
    ├── target/
    ├── .config/
    │   └── nextest.toml
    └── .cargo/
        └── config.toml

📊 Total Files: 416, Total Size: 76971984 bytes
```

## Enhanced AST Analysis

### ./assets/demo/app.js

**Language:** javascript
**Total Symbols:** 5
**Functions:** 5 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `formatFileSize` (public) at line 1
  - `formatDuration` (public) at line 1
  - `refreshData (async)` (public) at line 1
  - `updateMetricsDisplay` (public) at line 1
  - `exportReport (async)` (public) at line 1
### ./assets/vendor/mermaid-10.6.1.min.js

**Language:** javascript
**Total Symbols:** 8275
**Functions:** 8249 | **Structs:** 26 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `xA` (public) at line 1
  - `G7` (public) at line 1
  - `nt` (public) at line 1
  - `Q2` (public) at line 1
  - `gxe` (public) at line 1
  - `pxe` (public) at line 1
  - `bxe` (public) at line 1
  - `EU` (public) at line 1
  - `wxe` (public) at line 1
  - `Zft` (public) at line 1
  - ... and 8239 more functions

**Structs:**
  - `$Lt` (public) with 0 fields at line 1
  - `TDt` (public) with 0 fields at line 1
  - `iCe` (public) with 0 fields at line 1
  - `aCe` (public) with 0 fields at line 1
  - `mCe` (public) with 0 fields at line 1
  - ... and 21 more structs
### ./assets/project-state.d.ts

**Language:** typescript
**Total Symbols:** 1
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 1 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Traits:**
  - `ProjectState` (public) at line 1
### ./server/assets/demo/app.min.js

**Language:** javascript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./server/assets/vendor/d3.min.js

**Language:** javascript
**Total Symbols:** 897
**Functions:** 886 | **Structs:** 11 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `n` (public) at line 1
  - `e` (public) at line 1
  - `r` (public) at line 1
  - `u` (public) at line 1
  - `i` (public) at line 1
  - `o` (public) at line 1
  - `d` (public) at line 1
  - `p` (public) at line 1
  - `g` (public) at line 1
  - `y` (public) at line 1
  - ... and 876 more functions

**Structs:**
  - `T` (public) with 0 fields at line 1
  - `InternMap` (public) with 0 fields at line 1
  - `InternSet` (public) with 0 fields at line 1
  - `Su` (public) with 0 fields at line 1
  - `Ru` (public) with 0 fields at line 1
  - ... and 6 more structs
### ./server/assets/vendor/gridjs.min.js

**Language:** javascript
**Total Symbols:** 74
**Functions:** 74 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `n` (public) at line 1
  - `e` (public) at line 1
  - `r` (public) at line 1
  - `o` (public) at line 1
  - `i` (public) at line 1
  - `u` (public) at line 1
  - `s` (public) at line 1
  - `a` (public) at line 1
  - `b` (public) at line 1
  - `w` (public) at line 1
  - ... and 64 more functions
### ./server/assets/vendor/mermaid.min.js

**Language:** javascript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
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
### ./server/src/demo/adapters/mod.rs

**Language:** rust
**Total Symbols:** 3
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
### ./server/src/demo/adapters/http.rs

**Language:** rust
**Total Symbols:** 22
**Functions:** 4 | **Structs:** 6 | **Enums:** 2 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 10

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

**Imports:** 10 import statements
### ./server/src/demo/adapters/mcp.rs

**Language:** rust
**Total Symbols:** 24
**Functions:** 5 | **Structs:** 9 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 9

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

**Imports:** 9 import statements
### ./server/src/demo/mod.rs

**Language:** rust
**Total Symbols:** 44
**Functions:** 22 | **Structs:** 4 | **Enums:** 3 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 15

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
  - ... and 12 more functions

**Structs:**
  - `DemoConfig` (private) with 3 fields (derives: derive) at line 1
  - `DemoAnalyzer` (private) with 2 fields at line 1
  - `ProtocolTrace` (private) with 2 fields (derives: derive) at line 1
  - `DemoArgs` (public) with 16 fields (derives: derive) at line 1

**Enums:**
  - `AnalysisResults` (private) with 3 variants at line 1
  - `DemoOutput` (private) with 3 variants at line 1
  - `Protocol` (public) with 4 variants at line 1

**Imports:** 15 import statements
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
### ./server/src/demo/runner.rs

**Language:** rust
**Total Symbols:** 24
**Functions:** 3 | **Structs:** 4 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 17

**Functions:**
  - `resolve_repository` (public) at line 1
  - `resolve_repo_spec` (private) at line 1
  - `detect_repository` (public) at line 1

**Structs:**
  - `DemoRunner` (public) with 2 fields at line 1
  - `DemoStep` (public) with 7 fields (derives: derive) at line 1
  - `DemoReport` (public) with 4 fields (derives: derive) at line 1
  - `Component` (private) with 4 fields (derives: derive) at line 1

**Imports:** 17 import statements
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
### ./server/src/demo/templates.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
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
### ./server/src/tests/ast_e2e.rs

**Language:** rust
**Total Symbols:** 23
**Functions:** 13 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 10

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

**Imports:** 10 import statements
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
### ./server/src/tests/ast_regression_test.rs

**Language:** rust
**Total Symbols:** 6
**Functions:** 2 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Functions:**
  - `test_ast_analysis_not_empty_regression (async)` (private) at line 1
  - `test_deep_context_includes_ast_analysis (async)` (private) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
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
### ./server/src/tests/cli_property_tests.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
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
### ./server/src/tests/fixtures/sample.ts

**Language:** typescript
**Total Symbols:** 13
**Functions:** 4 | **Structs:** 4 | **Enums:** 1 | **Traits:** 2 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Functions:**
  - `processData` (public) at line 1
  - `fetchRemoteData (async)` (public) at line 1
  - `calculateSum` (public) at line 1
  - `asyncOperation (async)` (public) at line 1

**Structs:**
  - `UserRole` (public) with 0 fields at line 1
  - `UserService` (public) with 2 fields at line 1
  - `ApiResponse` (public) with 0 fields at line 1
  - `Repository` (public) with 1 field at line 1

**Enums:**
  - `StatusCode` (public) with 3 variants at line 1

**Traits:**
  - `User` (public) at line 1
  - `AdminUser` (public) at line 1

**Key Imports:**
  - `fs/promises` at line 1
  - `path` at line 1
### ./server/src/tests/fixtures/sample.js

**Language:** javascript
**Total Symbols:** 5
**Functions:** 4 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `calculateAverage` (public) at line 1
  - `fetchUserData (async)` (public) at line 1
  - `formatDate` (public) at line 1
  - `processAsync (async)` (public) at line 1

**Structs:**
  - `DataProcessor` (public) with 0 fields at line 1
### ./server/src/tests/fixtures/sample.py

**Language:** python
**Total Symbols:** 17
**Functions:** 9 | **Structs:** 2 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 6

**Functions:**
  - `get_display_name` (public) at line 1
  - `_internal_method` (private) at line 1
  - `__init__` (private) at line 1
  - `get_user (async)` (public) at line 1
  - `create_user (async)` (public) at line 1
  - `list_users` (public) at line 1
  - `process_data` (public) at line 1
  - `fetch_remote_data (async)` (public) at line 1
  - `_private_helper` (private) at line 1

**Structs:**
  - `User` (public) with 4 fields at line 1
  - `UserService` (public) with 0 fields at line 1

**Key Imports:**
  - `os` at line 1
  - `sys` at line 1
  - `typing.List` at line 1
  - `typing.Optional` at line 1
  - `typing.Dict` at line 1
  - `dataclasses.dataclass` at line 1
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
### ./server/src/cli/mod.rs

**Language:** rust
**Total Symbols:** 163
**Functions:** 89 | **Structs:** 3 | **Enums:** 17 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 54

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
  - ... and 79 more functions

**Structs:**
  - `Cli` (restricted) with 6 fields (derives: derive) at line 1
  - `EarlyCliArgs` (public) with 4 fields (derives: derive) at line 1
  - `DeepContextConfigParams` (private) with 10 fields (derives: derive) at line 1

**Enums:**
  - `Mode` (restricted) with 2 variants at line 1
  - `ExecutionMode` (public) with 2 variants at line 1
  - `Commands` (public) with 9 variants at line 1
  - `AnalyzeCommands` (public) with 8 variants at line 1
  - `ContextFormat` (public) with 2 variants at line 1
  - ... and 12 more enums

**Imports:** 54 import statements
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
### ./server/src/unified_protocol/adapters/cli.rs

**Language:** rust
**Total Symbols:** 34
**Functions:** 19 | **Structs:** 3 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 11

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
  - ... and 9 more functions

**Structs:**
  - `CliAdapter` (public) with 0 fields at line 1
  - `CliInput` (public) with 3 fields at line 1
  - `CliRunner` (public) with 1 field at line 1

**Enums:**
  - `CliOutput` (public) with 2 variants at line 1

**Imports:** 11 import statements
### ./server/src/unified_protocol/adapters/mod.rs

**Language:** rust
**Total Symbols:** 3
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
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
### ./server/src/unified_protocol/service.rs

**Language:** rust
**Total Symbols:** 75
**Functions:** 16 | **Structs:** 32 | **Enums:** 0 | **Traits:** 2 | **Impls:** 0 | **Modules:** 0 | **Imports:** 25

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
  - ... and 6 more functions

**Structs:**
  - `UnifiedService` (public) with 3 fields (derives: derive) at line 1
  - `AppState` (public) with 3 fields (derives: derive) at line 1
  - `ServiceMetrics` (public) with 3 fields (derives: derive) at line 1
  - `DefaultTemplateService` (public) with 0 fields (derives: derive) at line 1
  - `DefaultAnalysisService` (public) with 0 fields (derives: derive) at line 1
  - ... and 27 more structs

**Traits:**
  - `TemplateService` (public) at line 1
  - `AnalysisService` (public) at line 1

**Imports:** 25 import statements
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
### ./server/src/models/mod.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
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
  - `Language` (public) with 4 variants at line 1
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
### ./server/src/models/error.rs

**Language:** rust
**Total Symbols:** 3
**Functions:** 0 | **Structs:** 0 | **Enums:** 2 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 1

**Enums:**
  - `TemplateError` (public) with 10 variants at line 1
  - `AnalysisError` (public) with 5 variants at line 1

**Key Imports:**
  - `use statement` at line 1
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
### ./server/src/utils/mod.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
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
### ./server/src/handlers/tools.rs

**Language:** rust
**Total Symbols:** 109
**Functions:** 58 | **Structs:** 10 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 41

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
  - ... and 48 more functions

**Structs:**
  - `ValidationResult` (private) with 2 fields at line 1
  - `AnalyzeCodeChurnArgs` (private) with 3 fields (derives: derive) at line 1
  - `AnalyzeComplexityArgs` (private) with 7 fields (derives: derive) at line 1
  - `AnalyzeDagArgs` (private) with 5 fields (derives: derive) at line 1
  - `GenerateContextArgs` (private) with 7 fields (derives: derive) at line 1
  - ... and 5 more structs

**Imports:** 41 import statements
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
### ./server/src/services/context.rs

**Language:** rust
**Total Symbols:** 56
**Functions:** 39 | **Structs:** 5 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 11

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

**Imports:** 11 import statements
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
### ./server/src/services/mod.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
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
### ./server/src/services/deep_context.rs

**Language:** rust
**Total Symbols:** 99
**Functions:** 11 | **Structs:** 35 | **Enums:** 14 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 39

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
  - `analyze_satd (async)` (private) at line 1
  - ... and 1 more functions

**Structs:**
  - `DeepContextConfig` (public) with 10 fields (derives: derive) at line 1
  - `ComplexityThresholds` (public) with 2 fields (derives: derive) at line 1
  - `DeepContext` (public) with 8 fields (derives: derive) at line 1
  - `ContextMetadata` (public) with 5 fields (derives: derive) at line 1
  - `CacheStats` (public) with 3 fields (derives: derive) at line 1
  - ... and 30 more structs

**Enums:**
  - `AnalysisType` (public) with 7 variants at line 1
  - `DagType` (public) with 4 variants at line 1
  - `CacheStrategy` (public) with 3 variants at line 1
  - `NodeType` (public) with 2 variants at line 1
  - `ConfidenceLevel` (public) with 3 variants at line 1
  - ... and 9 more enums

**Imports:** 39 import statements
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
### ./server/src/services/tdg_calculator.rs

**Language:** rust
**Total Symbols:** 13
**Functions:** 2 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 10

**Functions:**
  - `test_tdg_calculation (async)` (private) at line 1
  - `test_tdg_distribution` (private) at line 1

**Structs:**
  - `TDGCalculator` (public) with 3 fields at line 1

**Imports:** 10 import statements
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
### ./server/src/services/ast_typescript.rs

**Language:** rust
**Total Symbols:** 17
**Functions:** 7 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 9

**Functions:**
  - `analyze_typescript_file_with_complexity (async)` (public) at line 1
  - `analyze_typescript_file_with_complexity_cached (async)` (public) at line 1
  - `analyze_javascript_file_with_complexity (async)` (public) at line 1
  - `analyze_typescript_file (async)` (public) at line 1
  - `analyze_typescript_file_with_classifier (async)` (public) at line 1
  - `analyze_javascript_file (async)` (public) at line 1
  - `analyze_javascript_file_with_classifier (async)` (public) at line 1

**Structs:**
  - `TypeScriptVisitor` (private) with 10 fields at line 1

**Imports:** 9 import statements
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
### ./server/src/services/duplicate_detector.rs

**Language:** rust
**Total Symbols:** 25
**Functions:** 7 | **Structs:** 8 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 9

**Functions:**
  - `euclidean_distance` (private) at line 1
  - `compute_rabin_fingerprint` (private) at line 1
  - `compute_alpha_normalized_hash` (private) at line 1
  - `compute_minhash_signature` (private) at line 1
  - `test_lsh_basic` (private) at line 1
  - `test_ann_index` (private) at line 1
  - `test_duplicate_detector` (private) at line 1

**Structs:**
  - `VectorizedLSH` (public) with 4 fields at line 1
  - `ANNIndex` (public) with 2 fields at line 1
  - `UniversalFeatureExtractor` (public) with 3 fields at line 1
  - `CloneReport` (public) with 5 fields (derives: derive) at line 1
  - `CloneGroup` (public) with 4 fields (derives: derive) at line 1
  - ... and 3 more structs

**Enums:**
  - `CloneType` (public) with 4 variants at line 1

**Imports:** 9 import statements
### ./server/src/services/git_clone.rs

**Language:** rust
**Total Symbols:** 17
**Functions:** 2 | **Structs:** 4 | **Enums:** 1 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 10

**Functions:**
  - `test_parse_github_urls (async)` (private) at line 1
  - `test_cache_key_generation (async)` (private) at line 1

**Structs:**
  - `CloneProgress` (public) with 4 fields (derives: derive) at line 1
  - `ClonedRepo` (public) with 3 fields (derives: derive) at line 1
  - `GitCloner` (public) with 4 fields (derives: derive) at line 1
  - `ParsedGitHubUrl` (public) with 2 fields (derives: derive) at line 1

**Enums:**
  - `CloneError` (public) with 6 variants at line 1

**Imports:** 10 import statements
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
### ./server/src/services/file_discovery.rs

**Language:** rust
**Total Symbols:** 23
**Functions:** 6 | **Structs:** 4 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 13

**Functions:**
  - `test_file_discovery_basic` (private) at line 1
  - `test_external_repo_filtering` (private) at line 1
  - `test_max_depth_limit` (private) at line 1
  - `test_custom_ignore_patterns` (private) at line 1
  - `test_file_extension_filtering` (private) at line 1
  - `test_discovery_stats` (private) at line 1

**Structs:**
  - `FileDiscoveryConfig` (public) with 6 fields (derives: derive) at line 1
  - `ProjectFileDiscovery` (public) with 3 fields at line 1
  - `DiscoveryStats` (public) with 4 fields (derives: derive) at line 1
  - `ExternalRepoFilter` (public) with 1 field at line 1

**Imports:** 13 import statements
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
### ./server/src/services/makefile_linter/rules/checkmake.rs

**Language:** rust
**Total Symbols:** 22
**Functions:** 11 | **Structs:** 6 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `check_undefined_in_text` (private) at line 1
  - `is_automatic_var` (private) at line 1
  - `is_function_call` (private) at line 1
  - `test_min_phony_rule` (private) at line 1
  - `test_phony_declared_rule` (private) at line 1
  - `test_max_body_length_rule` (private) at line 1
  - `test_timestamp_expanded_rule` (private) at line 1
  - `test_undefined_variable_rule` (private) at line 1
  - `test_portability_rule` (private) at line 1
  - `test_is_automatic_var` (private) at line 1
  - ... and 1 more functions

**Structs:**
  - `MinPhonyRule` (public) with 2 fields at line 1
  - `PhonyDeclaredRule` (public) with 1 field at line 1
  - `MaxBodyLengthRule` (public) with 2 fields at line 1
  - `TimestampExpandedRule` (public) with 0 fields at line 1
  - `UndefinedVariableRule` (public) with 0 fields at line 1
  - ... and 1 more structs

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
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
### ./server/src/services/cache/manager.rs

**Language:** rust
**Total Symbols:** 13
**Functions:** 0 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 12

**Structs:**
  - `SessionCacheManager` (public) with 8 fields at line 1

**Imports:** 12 import statements
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
### ./server/src/services/cache/config.rs

**Language:** rust
**Total Symbols:** 3
**Functions:** 0 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Structs:**
  - `CacheConfig` (public) with 14 fields (derives: derive) at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
### ./server/src/services/cache/persistent.rs

**Language:** rust
**Total Symbols:** 12
**Functions:** 0 | **Structs:** 2 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 10

**Structs:**
  - `PersistentCacheEntry` (private) with 3 fields (derives: derive) at line 1
  - `PersistentCache` (public) with 4 fields at line 1

**Imports:** 10 import statements
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
### ./server/src/services/rust_borrow_checker.rs

**Language:** rust
**Total Symbols:** 20
**Functions:** 4 | **Structs:** 2 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 14

**Functions:**
  - `test_rust_borrow_checker_creation (async)` (private) at line 1
  - `test_rust_borrow_checker_collect (async)` (private) at line 1
  - `test_memory_safety_annotation` (private) at line 1
  - `test_thread_safety_annotation` (private) at line 1

**Structs:**
  - `RustBorrowChecker` (public) with 2 fields (derives: derive) at line 1
  - `CollectionState` (private) with 3 fields at line 1

**Imports:** 14 import statements
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
  - `FileAst` (public) with 4 variants at line 1

**Imports:** 10 import statements
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
### ./server/src/services/ast_python.rs

**Language:** rust
**Total Symbols:** 11
**Functions:** 4 | **Structs:** 1 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 6

**Functions:**
  - `analyze_python_file_with_complexity (async)` (public) at line 1
  - `analyze_python_file (async)` (public) at line 1
  - `analyze_python_file_with_classifier (async)` (public) at line 1
  - `extract_python_items` (private) at line 1

**Structs:**
  - `PythonComplexityVisitor` (private) with 8 fields at line 1

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
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
### ./server/tests/demo_integration.rs

**Language:** rust
**Total Symbols:** 13
**Functions:** 6 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 7

**Functions:**
  - `test_demo_mode_in_current_directory` (private) at line 1
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
### ./server/tests/e2e/installation.test.ts

**Language:** typescript
**Total Symbols:** 4
**Functions:** 1 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `testBinaryExecution (async)` (public) at line 1

**Key Imports:**
  - `https://deno.land/std@0.210.0/assert/mod.ts` at line 1
  - `https://deno.land/std@0.210.0/path/mod.ts` at line 1
  - `https://deno.land/std@0.210.0/testing/bdd.ts` at line 1
### ./server/tests/e2e/mcp_protocol.test.ts

**Language:** typescript
**Total Symbols:** 5
**Functions:** 0 | **Structs:** 1 | **Enums:** 0 | **Traits:** 2 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Structs:**
  - `McpClient` (public) with 5 fields at line 1

**Traits:**
  - `JsonRpcRequest` (public) at line 1
  - `JsonRpcResponse` (public) at line 1

**Key Imports:**
  - `https://deno.land/std@0.208.0/assert/mod.ts` at line 1
  - `https://deno.land/std@0.208.0/testing/bdd.ts` at line 1
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
### ./server/scripts/docker-setup.ts

**Language:** typescript
**Total Symbols:** 8
**Functions:** 5 | **Structs:** 0 | **Enums:** 0 | **Traits:** 1 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Functions:**
  - `runCommand (async)` (public) at line 1
  - `checkDockerStatus (async)` (public) at line 1
  - `installDocker (async)` (public) at line 1
  - `fixDockerPermissions (async)` (public) at line 1
  - `main (async)` (public) at line 1

**Traits:**
  - `DockerStatus` (public) at line 1

**Key Imports:**
  - `https://deno.land/std@0.208.0/fs/mod.ts` at line 1
  - `https://deno.land/std@0.208.0/flags/mod.ts` at line 1
### ./server/scripts/test-mcp-e2e.ts

**Language:** typescript
**Total Symbols:** 5
**Functions:** 1 | **Structs:** 1 | **Enums:** 0 | **Traits:** 2 | **Impls:** 0 | **Modules:** 0 | **Imports:** 1

**Functions:**
  - `runE2ETests (async)` (public) at line 1

**Structs:**
  - `McpClient` (public) with 5 fields at line 1

**Traits:**
  - `JsonRpcRequest` (public) at line 1
  - `JsonRpcResponse` (public) at line 1

**Key Imports:**
  - `https://deno.land/std@0.208.0/assert/mod.ts` at line 1
### ./server/scripts/test-installation.ts

**Language:** typescript
**Total Symbols:** 7
**Functions:** 5 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Functions:**
  - `testBinaryExecution (async)` (public) at line 1
  - `testMCPProtocol (async)` (public) at line 1
  - `testInstallationPaths (async)` (public) at line 1
  - `testCurlInstallation (async)` (public) at line 1
  - `main (async)` (public) at line 1

**Key Imports:**
  - `https://deno.land/std@0.210.0/assert/mod.ts` at line 1
  - `https://deno.land/std@0.210.0/path/mod.ts` at line 1
### ./scripts/dogfood-readme.ts

**Language:** typescript
**Total Symbols:** 2
**Functions:** 2 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `ensureDirectoryExists (async)` (public) at line 1
  - `runDeepContextAnalysis (async)` (public) at line 1
### ./scripts/generate-fuzz-corpus.ts

**Language:** typescript
**Total Symbols:** 9
**Functions:** 4 | **Structs:** 0 | **Enums:** 0 | **Traits:** 3 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Functions:**
  - `generateCorpus (async)` (public) at line 1
  - `generateDeepGraph` (public) at line 1
  - `generateWideGraph` (public) at line 1
  - `serializeInput` (public) at line 1

**Traits:**
  - `FuzzNode` (public) at line 1
  - `FuzzEdge` (public) at line 1
  - `FuzzInput` (public) at line 1

**Key Imports:**
  - `https://deno.land/std@0.208.0/fs/mod.ts` at line 1
  - `https://deno.land/std@0.208.0/path/mod.ts` at line 1
### ./scripts/install.integration.test.ts

**Language:** typescript
**Total Symbols:** 2
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Key Imports:**
  - `https://deno.land/std@0.210.0/assert/mod.ts` at line 1
  - `https://deno.land/std@0.210.0/testing/bdd.ts` at line 1
### ./scripts/ast-mermaid-integration.test.ts

**Language:** typescript
**Total Symbols:** 5
**Functions:** 0 | **Structs:** 1 | **Enums:** 0 | **Traits:** 2 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Structs:**
  - `ASTMermaidGenerator` (public) with 0 fields at line 1

**Traits:**
  - `ASTNode` (public) at line 1
  - `CodeAST` (public) at line 1

**Key Imports:**
  - `https://deno.land/std@0.208.0/assert/mod.ts` at line 1
  - `./mermaid-validator.ts` at line 1
### ./scripts/create-release.ts

**Language:** typescript
**Total Symbols:** 2
**Functions:** 1 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 1

**Functions:**
  - `main (async)` (public) at line 1

**Key Imports:**
  - `./lib/create-release-utils.ts` at line 1
### ./scripts/mermaid-validator.ts

**Language:** typescript
**Total Symbols:** 6
**Functions:** 0 | **Structs:** 1 | **Enums:** 0 | **Traits:** 3 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Structs:**
  - `MermaidValidator` (public) with 3 fields at line 1

**Traits:**
  - `ValidationResult` (public) at line 1
  - `ValidationError` (public) at line 1
  - `BatchValidationResult` (public) at line 1

**Key Imports:**
  - `https://deno.land/std@0.208.0/flags/mod.ts` at line 1
  - `https://deno.land/std@0.208.0/fs/walk.ts` at line 1
### ./scripts/update-version.ts

**Language:** typescript
**Total Symbols:** 9
**Functions:** 7 | **Structs:** 0 | **Enums:** 0 | **Traits:** 2 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `parseVersion` (public) at line 1
  - `formatVersion` (public) at line 1
  - `getCurrentVersion (async)` (public) at line 1
  - `bumpVersion` (public) at line 1
  - `updateVersionInFile (async)` (public) at line 1
  - `updateAllVersions (async)` (public) at line 1
  - `main (async)` (public) at line 1

**Traits:**
  - `VersionUpdate` (public) at line 1
  - `Version` (public) at line 1
### ./scripts/validate-github-actions-status.ts

**Language:** typescript
**Total Symbols:** 10
**Functions:** 6 | **Structs:** 0 | **Enums:** 0 | **Traits:** 4 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `fetchGitHubAPI (async)` (public) at line 1
  - `getLatestWorkflowRuns (async)` (public) at line 1
  - `getWorkflowJobs (async)` (public) at line 1
  - `formatStatus` (public) at line 1
  - `formatDate` (public) at line 1
  - `validateGitHubActionsStatus (async)` (public) at line 1

**Traits:**
  - `WorkflowRun` (public) at line 1
  - `Job` (public) at line 1
  - `WorkflowRunsResponse` (public) at line 1
  - `JobsResponse` (public) at line 1
### ./scripts/test-curl-install.ts

**Language:** typescript
**Total Symbols:** 2
**Functions:** 2 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `test (async)` (public) at line 1
  - `main (async)` (public) at line 1
### ./scripts/validate-demo-assets.ts

**Language:** typescript
**Total Symbols:** 12
**Functions:** 9 | **Structs:** 0 | **Enums:** 0 | **Traits:** 1 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Functions:**
  - `extractHtmlFromRust` (public) at line 1
  - `extractJavaScriptFromHtml` (public) at line 1
  - `extractCssFromHtml` (public) at line 1
  - `validateJavaScript (async)` (public) at line 1
  - `validateCss` (public) at line 1
  - `validateHtml` (public) at line 1
  - `formatWebAssets` (public) at line 1
  - `processRustFile (async)` (public) at line 1
  - `main (async)` (public) at line 1

**Traits:**
  - `ValidationResult` (public) at line 1

**Key Imports:**
  - `https://deno.land/std@0.208.0/fs/mod.ts` at line 1
  - `https://deno.land/std@0.208.0/flags/mod.ts` at line 1
### ./scripts/create-release.test.ts

**Language:** typescript
**Total Symbols:** 5
**Functions:** 3 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Functions:**
  - `detectPlatform` (public) at line 1
  - `getVersion (async)` (public) at line 1
  - `isRunningInGitHubActions` (public) at line 1

**Key Imports:**
  - `https://deno.land/std@0.210.0/testing/asserts.ts` at line 1
  - `https://deno.land/std@0.210.0/testing/bdd.ts` at line 1
### ./scripts/dogfood-readme-integration.test.ts

**Language:** typescript
**Total Symbols:** 2
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Key Imports:**
  - `https://deno.land/std@0.208.0/assert/mod.ts` at line 1
  - `./mermaid-validator.ts` at line 1
### ./scripts/test-workflow-dag.ts

**Language:** typescript
**Total Symbols:** 9
**Functions:** 2 | **Structs:** 1 | **Enums:** 0 | **Traits:** 4 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Functions:**
  - `runWorkflowTests (async)` (public) at line 1
  - `testVersionMismatchScenario` (public) at line 1

**Structs:**
  - `WorkflowSimulator` (public) with 6 fields at line 1

**Traits:**
  - `WorkflowTestResult` (public) at line 1
  - `WorkflowStep` (public) at line 1
  - `WorkflowJob` (public) at line 1
  - `WorkflowContent` (public) at line 1

**Key Imports:**
  - `https://deno.land/std@0.210.0/yaml/mod.ts` at line 1
  - `https://deno.land/std@0.210.0/path/mod.ts` at line 1
### ./scripts/mermaid-validator.test.ts

**Language:** typescript
**Total Symbols:** 2
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Key Imports:**
  - `https://deno.land/std@0.208.0/assert/mod.ts` at line 1
  - `./mermaid-validator.ts` at line 1
### ./scripts/install.test.ts

**Language:** typescript
**Total Symbols:** 4
**Functions:** 2 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Functions:**
  - `detectPlatform` (public) at line 1
  - `detectPlatformWithInputs` (public) at line 1

**Key Imports:**
  - `https://deno.land/std@0.210.0/assert/mod.ts` at line 1
  - `https://deno.land/std@0.210.0/testing/bdd.ts` at line 1
### ./scripts/validate-naming.ts

**Language:** typescript
**Total Symbols:** 8
**Functions:** 7 | **Structs:** 0 | **Enums:** 0 | **Traits:** 1 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `runCommand (async)` (public) at line 1
  - `checkCargoToml (async)` (public) at line 1
  - `checkSourceCode (async)` (public) at line 1
  - `checkGitHubWorkflows (async)` (public) at line 1
  - `checkDocumentation (async)` (public) at line 1
  - `checkMakefiles (async)` (public) at line 1
  - `runValidation (async)` (public) at line 1

**Traits:**
  - `ValidationResult` (public) at line 1
### ./scripts/run-fuzzing.ts

**Language:** typescript
**Total Symbols:** 13
**Functions:** 8 | **Structs:** 0 | **Enums:** 0 | **Traits:** 1 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Functions:**
  - `checkCargoFuzz (async)` (public) at line 1
  - `installCargoFuzz (async)` (public) at line 1
  - `generateCorpusIfNeeded (async)` (public) at line 1
  - `runFuzzer (async)` (public) at line 1
  - `generateCoverage (async)` (public) at line 1
  - `checkCommand (async)` (public) at line 1
  - `countFiles (async)` (public) at line 1
  - `main (async)` (public) at line 1

**Traits:**
  - `FuzzerConfig` (public) at line 1

**Key Imports:**
  - `https://deno.land/std@0.208.0/fs/mod.ts` at line 1
  - `https://deno.land/std@0.208.0/path/mod.ts` at line 1
  - `https://deno.land/std@0.208.0/flags/mod.ts` at line 1
  - `https://deno.land/std@0.208.0/fmt/colors.ts` at line 1
### ./scripts/validate-demo-assets.test.ts

**Language:** typescript
**Total Symbols:** 2
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Key Imports:**
  - `https://deno.land/std@0.208.0/assert/mod.ts` at line 1
  - `./validate-demo-assets.ts` at line 1
### ./scripts/validate-docs.ts

**Language:** typescript
**Total Symbols:** 6
**Functions:** 3 | **Structs:** 0 | **Enums:** 0 | **Traits:** 1 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Functions:**
  - `validateFile (async)` (public) at line 1
  - `shouldValidateFile` (public) at line 1
  - `validateProject (async)` (public) at line 1

**Traits:**
  - `ValidationError` (public) at line 1

**Key Imports:**
  - `https://deno.land/std@0.220.0/fs/walk.ts` at line 1
  - `https://deno.land/std@0.220.0/path/mod.ts` at line 1
### ./scripts/deep-context.ts

**Language:** typescript
**Total Symbols:** 29
**Functions:** 10 | **Structs:** 1 | **Enums:** 0 | **Traits:** 13 | **Impls:** 0 | **Modules:** 0 | **Imports:** 5

**Functions:**
  - `log` (public) at line 1
  - `generateProjectTree (async)` (public) at line 1
  - `buildTree (async)` (public) at line 1
  - `parseRustFile (async)` (public) at line 1
  - `visitTypeScriptNode` (public) at line 1
  - `parseTypeScriptFile` (public) at line 1
  - `parseMakefile (async)` (public) at line 1
  - `findProjectFiles (async)` (public) at line 1
  - `generateOutput` (public) at line 1
  - `main (async)` (public) at line 1

**Structs:**
  - `LogLevel` (public) with 0 fields at line 1

**Traits:**
  - `ParsedArgs` (public) at line 1
  - `RustFunction` (public) at line 1
  - `RustStruct` (public) at line 1
  - `RustEnum` (public) at line 1
  - `RustImpl` (public) at line 1
  - ... and 8 more traits

**Key Imports:**
  - `https://deno.land/std@0.208.0/flags/mod.ts` at line 1
  - `https://deno.land/std@0.208.0/fs/walk.ts` at line 1
  - `https://deno.land/std@0.208.0/path/mod.ts` at line 1
  - `https://deno.land/std@0.208.0/fmt/colors.ts` at line 1
  - `https://esm.sh/typescript@5.3.3` at line 1
### ./scripts/update-rust-docs.ts

**Language:** typescript
**Total Symbols:** 8
**Functions:** 0 | **Structs:** 1 | **Enums:** 0 | **Traits:** 6 | **Impls:** 0 | **Modules:** 0 | **Imports:** 1

**Structs:**
  - `RustDocsUpdater` (public) with 0 fields at line 1

**Traits:**
  - `Metrics` (public) at line 1
  - `CoverageMetrics` (public) at line 1
  - `PerformanceMetrics` (public) at line 1
  - `ComplexityMetrics` (public) at line 1
  - `BinarySizeMetrics` (public) at line 1
  - ... and 1 more traits

**Key Imports:**
  - `https://deno.land/std@0.208.0/path/mod.ts` at line 1
### ./scripts/lib/install-utils.ts

**Language:** typescript
**Total Symbols:** 10
**Functions:** 9 | **Structs:** 0 | **Enums:** 0 | **Traits:** 1 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `detectPlatform` (public) at line 1
  - `getLatestVersion (async)` (public) at line 1
  - `constructDownloadUrl` (public) at line 1
  - `downloadFile (async)` (public) at line 1
  - `extractTarball (async)` (public) at line 1
  - `ensureDirectoryExists (async)` (public) at line 1
  - `isInPath` (public) at line 1
  - `verifyInstallation (async)` (public) at line 1
  - `stripVersionPrefix` (public) at line 1

**Traits:**
  - `InstallConfig` (public) at line 1
### ./scripts/lib/create-release-utils.test.ts

**Language:** typescript
**Total Symbols:** 3
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Key Imports:**
  - `https://deno.land/std@0.210.0/testing/asserts.ts` at line 1
  - `https://deno.land/std@0.210.0/testing/bdd.ts` at line 1
  - `./create-release-utils.ts` at line 1
### ./scripts/lib/create-release-utils-integration.test.ts

**Language:** typescript
**Total Symbols:** 3
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Key Imports:**
  - `https://deno.land/std@0.210.0/testing/asserts.ts` at line 1
  - `https://deno.land/std@0.210.0/testing/bdd.ts` at line 1
  - `./create-release-utils.ts` at line 1
### ./scripts/lib/install-utils.test.ts

**Language:** typescript
**Total Symbols:** 5
**Functions:** 2 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `downloadFile (async)` (public) at line 1
  - `extractTarball (async)` (public) at line 1

**Key Imports:**
  - `https://deno.land/std@0.210.0/testing/asserts.ts` at line 1
  - `https://deno.land/std@0.210.0/testing/bdd.ts` at line 1
  - `./install-utils.ts` at line 1
### ./scripts/lib/create-release-utils.ts

**Language:** typescript
**Total Symbols:** 10
**Functions:** 10 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `getVersion (async)` (public) at line 1
  - `runCommand (async)` (public) at line 1
  - `checkGhCli (async)` (public) at line 1
  - `isRunningInGitHubActions` (public) at line 1
  - `detectPlatform` (public) at line 1
  - `createTarball (async)` (public) at line 1
  - `generateReleaseNotes` (public) at line 1
  - `checkProjectStructure (async)` (public) at line 1
  - `buildReleaseBinary (async)` (public) at line 1
  - `createGitHubRelease (async)` (public) at line 1
### ./scripts/install.ts

**Language:** typescript
**Total Symbols:** 9
**Functions:** 9 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `error` (public) at line 1
  - `info` (public) at line 1
  - `warn` (public) at line 1
  - `detectPlatform` (public) at line 1
  - `getLatestVersion (async)` (public) at line 1
  - `downloadFile (async)` (public) at line 1
  - `extractTarGz (async)` (public) at line 1
  - `install (async)` (public) at line 1
  - `main (async)` (public) at line 1
### ./scripts/mcp-install.ts

**Language:** typescript
**Total Symbols:** 18
**Functions:** 12 | **Structs:** 0 | **Enums:** 0 | **Traits:** 2 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Functions:**
  - `runCommand (async)` (public) at line 1
  - `getSourceFilesHash (async)` (public) at line 1
  - `getBuildInfo (async)` (public) at line 1
  - `saveBuildInfo (async)` (public) at line 1
  - `findInstalledBinary (async)` (public) at line 1
  - `checkIfRebuildNeeded (async)` (public) at line 1
  - `checkInstallationStatus (async)` (public) at line 1
  - `uninstallMcpServer (async)` (public) at line 1
  - `buildAndInstall (async)` (public) at line 1
  - `smartInstall (async)` (public) at line 1
  - ... and 2 more functions

**Traits:**
  - `InstallLocation` (public) at line 1
  - `BuildInfo` (public) at line 1

**Key Imports:**
  - `https://deno.land/std@0.208.0/flags/mod.ts` at line 1
  - `https://deno.land/std@0.208.0/fs/mod.ts` at line 1
  - `https://deno.land/std@0.208.0/path/mod.ts` at line 1
  - `https://deno.land/std@0.208.0/fs/walk.ts` at line 1
### ./scripts/archive/generate-from-project-state.ts

**Language:** typescript
**Total Symbols:** 8
**Functions:** 8 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `generateGitHubBadge` (public) at line 1
  - `generateStaticBadge` (public) at line 1
  - `generateLicenseBadge` (public) at line 1
  - `generateBadges` (public) at line 1
  - `generateInstallerUrl` (public) at line 1
  - `generateReleaseUrl` (public) at line 1
  - `generateRepoUrl` (public) at line 1
  - `generateClaudeConfig` (public) at line 1
### ./scripts/archive/cleanup-releases.ts

**Language:** typescript
**Total Symbols:** 6
**Functions:** 4 | **Structs:** 0 | **Enums:** 0 | **Traits:** 2 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `getGitHubData (async)` (public) at line 1
  - `getLocalTags (async)` (public) at line 1
  - `_getCommitForTag (async)` (public) at line 1
  - `main (async)` (public) at line 1

**Traits:**
  - `GitHubRelease` (public) at line 1
  - `GitHubWorkflowRun` (public) at line 1
### ./scripts/archive/dogfood-readme-deprecated.ts

**Language:** typescript
**Total Symbols:** 5
**Functions:** 0 | **Structs:** 1 | **Enums:** 0 | **Traits:** 3 | **Impls:** 0 | **Modules:** 0 | **Imports:** 1

**Structs:**
  - `MCPAgentToolkitDogfooder` (public) with 0 fields at line 1

**Traits:**
  - `AnalysisResult` (public) at line 1
  - `ProjectMetrics` (public) at line 1
  - `ServerInfoResponse` (public) at line 1

**Key Imports:**
  - `https://deno.land/std@0.208.0/path/mod.ts` at line 1
### ./scripts/archive/dead-scripts/download-mermaid.ts

**Language:** typescript
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./scripts/archive/dead-scripts/docker-setup.ts

**Language:** typescript
**Total Symbols:** 8
**Functions:** 5 | **Structs:** 0 | **Enums:** 0 | **Traits:** 1 | **Impls:** 0 | **Modules:** 0 | **Imports:** 2

**Functions:**
  - `runCommand (async)` (public) at line 1
  - `checkDockerStatus (async)` (public) at line 1
  - `installDocker (async)` (public) at line 1
  - `fixDockerPermissions (async)` (public) at line 1
  - `main (async)` (public) at line 1

**Traits:**
  - `DockerStatus` (public) at line 1

**Key Imports:**
  - `https://deno.land/std@0.208.0/fs/mod.ts` at line 1
  - `https://deno.land/std@0.208.0/flags/mod.ts` at line 1
### ./scripts/archive/dead-scripts/mcp-install-deterministic.ts

**Language:** typescript
**Total Symbols:** 14
**Functions:** 10 | **Structs:** 0 | **Enums:** 0 | **Traits:** 1 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `runCommand (async)` (public) at line 1
  - `analyzeSystemState (async)` (public) at line 1
  - `cleanEverything (async)` (public) at line 1
  - `fixCargoToml (async)` (public) at line 1
  - `buildBinary (async)` (public) at line 1
  - `installBinary (async)` (public) at line 1
  - `configureClaude (async)` (public) at line 1
  - `verifyInstallation (async)` (public) at line 1
  - `printState` (public) at line 1
  - `main (async)` (public) at line 1

**Traits:**
  - `SystemState` (public) at line 1

**Key Imports:**
  - `https://deno.land/std@0.208.0/flags/mod.ts` at line 1
  - `https://deno.land/std@0.208.0/fs/mod.ts` at line 1
  - `https://deno.land/std@0.208.0/path/mod.ts` at line 1
### ./scripts/archive/cleanup-test-artifacts.ts

**Language:** typescript
**Total Symbols:** 2
**Functions:** 2 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `cleanupTestArtifacts` (public) at line 1
  - `setupTestCleanup` (public) at line 1
### ./scripts/archive/verify-demo-binary-size.ts

**Language:** typescript
**Total Symbols:** 4
**Functions:** 3 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 1

**Functions:**
  - `runCommand (async)` (public) at line 1
  - `getBinarySize (async)` (public) at line 1
  - `main (async)` (public) at line 1

**Key Imports:**
  - `https://deno.land/std@0.208.0/path/mod.ts` at line 1
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
### ./fuzz/target/debug/build/typenum-0616c43aaf37d1ed/out/tests.rs

**Language:** rust
**Total Symbols:** 1746
**Functions:** 1743 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `test_0_BitAnd_0` (private) at line 1
  - `test_0_BitOr_0` (private) at line 1
  - `test_0_BitXor_0` (private) at line 1
  - `test_0_Shl_0` (private) at line 1
  - `test_0_Shr_0` (private) at line 1
  - `test_0_Add_0` (private) at line 1
  - `test_0_Mul_0` (private) at line 1
  - `test_0_Pow_0` (private) at line 1
  - `test_0_Min_0` (private) at line 1
  - `test_0_Max_0` (private) at line 1
  - ... and 1733 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
### ./fuzz/target/debug/build/mime_guess-d7ef9d079883a20c/out/mime_types_generated.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./fuzz/target/debug/build/paiml-mcp-agent-toolkit-2cdaceb83e818e9a/out/compressed_templates.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
### ./fuzz/target/debug/build/rustpython-parser-6f745e97e5b489c8/out/keywords.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./fuzz/target/debug/build/paiml-mcp-agent-toolkit-530832308607b316/out/compressed_templates.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
### ./fuzz/target/debug/build/crunchy-a6e6d11cb39ae881/out/lib.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 4 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `invalid_range` (private) at line 1
  - `start_at_one_with_step` (private) at line 1
  - `start_at_one` (private) at line 1
  - `test_all` (private) at line 1
### ./fuzz/target/debug/build/unicode_names2-294286d27e2b0fd5/out/generated_alias.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./fuzz/target/debug/build/unicode_names2-294286d27e2b0fd5/out/generated_phf.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./fuzz/target/debug/build/unicode_names2-294286d27e2b0fd5/out/generated.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./fuzz/target/debug/build/paiml-mcp-agent-toolkit-99643dca77d04bab/out/compressed_templates.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
### ./target/release/build/selectors-ee530316e6750045/out/ascii_case_insensitive_html_attributes.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/release/build/typenum-72296a600ceccf65/out/tests.rs

**Language:** rust
**Total Symbols:** 1746
**Functions:** 1743 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `test_0_BitAnd_0` (private) at line 1
  - `test_0_BitOr_0` (private) at line 1
  - `test_0_BitXor_0` (private) at line 1
  - `test_0_Shl_0` (private) at line 1
  - `test_0_Shr_0` (private) at line 1
  - `test_0_Add_0` (private) at line 1
  - `test_0_Mul_0` (private) at line 1
  - `test_0_Pow_0` (private) at line 1
  - `test_0_Min_0` (private) at line 1
  - `test_0_Max_0` (private) at line 1
  - ... and 1733 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
### ./target/release/build/rustpython-parser-e41da5e286a38697/out/keywords.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/release/build/paiml-mcp-agent-toolkit-6380e95b85563dd4/out/compressed_templates.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
### ./target/release/build/unicode_names2-32174d55335b5665/out/generated_alias.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/release/build/unicode_names2-32174d55335b5665/out/generated_phf.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/release/build/unicode_names2-32174d55335b5665/out/generated.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/release/build/mime_guess-272b486d2f4c5b77/out/mime_types_generated.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/release/build/rustpython-parser-8c62b913e77fdafa/out/keywords.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/release/build/crunchy-537635466b587b11/out/lib.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 4 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `invalid_range` (private) at line 1
  - `start_at_one_with_step` (private) at line 1
  - `start_at_one` (private) at line 1
  - `test_all` (private) at line 1
### ./target/release/build/typenum-c86c01010d9c1a20/out/tests.rs

**Language:** rust
**Total Symbols:** 1746
**Functions:** 1743 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `test_0_BitAnd_0` (private) at line 1
  - `test_0_BitOr_0` (private) at line 1
  - `test_0_BitXor_0` (private) at line 1
  - `test_0_Shl_0` (private) at line 1
  - `test_0_Shr_0` (private) at line 1
  - `test_0_Add_0` (private) at line 1
  - `test_0_Mul_0` (private) at line 1
  - `test_0_Pow_0` (private) at line 1
  - `test_0_Min_0` (private) at line 1
  - `test_0_Max_0` (private) at line 1
  - ... and 1733 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
### ./target/release/build/markup5ever-380850c1ba707c73/out/named_entities.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/release/build/markup5ever-380850c1ba707c73/out/generated.rs

**Language:** rust
**Total Symbols:** 3
**Functions:** 0 | **Structs:** 3 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Structs:**
  - `LocalNameStaticSet` (public) with 0 fields (derives: derive) at line 1
  - `PrefixStaticSet` (public) with 0 fields (derives: derive) at line 1
  - `NamespaceStaticSet` (public) with 0 fields (derives: derive) at line 1
### ./target/release/build/mime_guess-722f4fcab743aa58/out/mime_types_generated.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/release/build/crunchy-6f9353c3b0485b1e/out/lib.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 4 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `invalid_range` (private) at line 1
  - `start_at_one_with_step` (private) at line 1
  - `start_at_one` (private) at line 1
  - `test_all` (private) at line 1
### ./target/release/build/unicode_names2-c5e717df397148ca/out/generated_alias.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/release/build/unicode_names2-c5e717df397148ca/out/generated_phf.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/release/build/unicode_names2-c5e717df397148ca/out/generated.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/release/build/paiml-mcp-agent-toolkit-ba8b8b8053dfc4d0/out/compressed_templates.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
### ./target/debug/build/crunchy-6262cea67f8d0af7/out/lib.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 4 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `invalid_range` (private) at line 1
  - `start_at_one_with_step` (private) at line 1
  - `start_at_one` (private) at line 1
  - `test_all` (private) at line 1
### ./target/debug/build/paiml-mcp-agent-toolkit-3a78d45bda1602ba/out/compressed_templates.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
### ./target/debug/build/paiml-mcp-agent-toolkit-a356411944bef768/out/compressed_templates.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
### ./target/debug/build/selectors-4239a7a594686f2a/out/ascii_case_insensitive_html_attributes.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/crunchy-4b58ade15d894be5/out/lib.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 4 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `invalid_range` (private) at line 1
  - `start_at_one_with_step` (private) at line 1
  - `start_at_one` (private) at line 1
  - `test_all` (private) at line 1
### ./target/debug/build/rustpython-parser-c1d2d0652104a28f/out/keywords.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/paiml-mcp-agent-toolkit-5a242e91bd514a61/out/compressed_templates.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
### ./target/debug/build/paiml-mcp-agent-toolkit-73809624c115cac1/out/compressed_templates.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
### ./target/debug/build/mime_guess-433191ff6b2a7f43/out/mime_types_generated.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/selectors-8af8ec855eae68d6/out/ascii_case_insensitive_html_attributes.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/typenum-0616c43aaf37d1ed/out/tests.rs

**Language:** rust
**Total Symbols:** 1746
**Functions:** 1743 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `test_0_BitAnd_0` (private) at line 1
  - `test_0_BitOr_0` (private) at line 1
  - `test_0_BitXor_0` (private) at line 1
  - `test_0_Shl_0` (private) at line 1
  - `test_0_Shr_0` (private) at line 1
  - `test_0_Add_0` (private) at line 1
  - `test_0_Mul_0` (private) at line 1
  - `test_0_Pow_0` (private) at line 1
  - `test_0_Min_0` (private) at line 1
  - `test_0_Max_0` (private) at line 1
  - ... and 1733 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
### ./target/debug/build/typenum-8705cd5d2675414c/out/tests.rs

**Language:** rust
**Total Symbols:** 1746
**Functions:** 1743 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `test_0_BitAnd_0` (private) at line 1
  - `test_0_BitOr_0` (private) at line 1
  - `test_0_BitXor_0` (private) at line 1
  - `test_0_Shl_0` (private) at line 1
  - `test_0_Shr_0` (private) at line 1
  - `test_0_Add_0` (private) at line 1
  - `test_0_Mul_0` (private) at line 1
  - `test_0_Pow_0` (private) at line 1
  - `test_0_Min_0` (private) at line 1
  - `test_0_Max_0` (private) at line 1
  - ... and 1733 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
### ./target/debug/build/selectors-1f29f90f1229f560/out/ascii_case_insensitive_html_attributes.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/markup5ever-9d0619cffbbbf7c6/out/named_entities.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/markup5ever-9d0619cffbbbf7c6/out/generated.rs

**Language:** rust
**Total Symbols:** 3
**Functions:** 0 | **Structs:** 3 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Structs:**
  - `LocalNameStaticSet` (public) with 0 fields (derives: derive) at line 1
  - `PrefixStaticSet` (public) with 0 fields (derives: derive) at line 1
  - `NamespaceStaticSet` (public) with 0 fields (derives: derive) at line 1
### ./target/debug/build/mime_guess-d7ef9d079883a20c/out/mime_types_generated.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/unicode_names2-daadbdd698b54514/out/generated_alias.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/unicode_names2-daadbdd698b54514/out/generated_phf.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/unicode_names2-daadbdd698b54514/out/generated.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/unicode_names2-de7aaec59ba2bee9/out/generated_alias.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/unicode_names2-de7aaec59ba2bee9/out/generated_phf.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/unicode_names2-de7aaec59ba2bee9/out/generated.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/paiml-mcp-agent-toolkit-e3f485a864cc50f6/out/compressed_templates.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
### ./target/debug/build/unicode_names2-3b910bb8cd402128/out/generated_alias.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/unicode_names2-3b910bb8cd402128/out/generated_phf.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/unicode_names2-3b910bb8cd402128/out/generated.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/rustpython-parser-7a1fd0832ac95676/out/keywords.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/crunchy-40daccb50a4f1806/out/lib.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 4 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `invalid_range` (private) at line 1
  - `start_at_one_with_step` (private) at line 1
  - `start_at_one` (private) at line 1
  - `test_all` (private) at line 1
### ./target/debug/build/rustpython-parser-6f745e97e5b489c8/out/keywords.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/rustpython-parser-e09af0319a8df0f6/out/keywords.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/typenum-b51e0394aa43b0ab/out/tests.rs

**Language:** rust
**Total Symbols:** 1746
**Functions:** 1743 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `test_0_BitAnd_0` (private) at line 1
  - `test_0_BitOr_0` (private) at line 1
  - `test_0_BitXor_0` (private) at line 1
  - `test_0_Shl_0` (private) at line 1
  - `test_0_Shr_0` (private) at line 1
  - `test_0_Add_0` (private) at line 1
  - `test_0_Mul_0` (private) at line 1
  - `test_0_Pow_0` (private) at line 1
  - `test_0_Min_0` (private) at line 1
  - `test_0_Max_0` (private) at line 1
  - ... and 1733 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
### ./target/debug/build/mime_guess-51219a052f766396/out/mime_types_generated.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/selectors-4dbb82da9b2628aa/out/ascii_case_insensitive_html_attributes.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/mime_guess-b36cece9b11e9324/out/mime_types_generated.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/crunchy-a6e6d11cb39ae881/out/lib.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 4 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `invalid_range` (private) at line 1
  - `start_at_one_with_step` (private) at line 1
  - `start_at_one` (private) at line 1
  - `test_all` (private) at line 1
### ./target/debug/build/unicode_names2-71c9bb5b15ef890e/out/generated_alias.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/unicode_names2-71c9bb5b15ef890e/out/generated_phf.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/unicode_names2-71c9bb5b15ef890e/out/generated.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/paiml-mcp-agent-toolkit-e64ee1aabc2197a1/out/compressed_templates.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
### ./target/debug/build/paiml-mcp-agent-toolkit-4eff13697bc57eac/out/compressed_templates.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
### ./target/debug/build/paiml-mcp-agent-toolkit-52d68413db53ccad/out/compressed_templates.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
### ./target/debug/build/markup5ever-0a07d5f6746add9e/out/named_entities.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/markup5ever-0a07d5f6746add9e/out/generated.rs

**Language:** rust
**Total Symbols:** 3
**Functions:** 0 | **Structs:** 3 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Structs:**
  - `LocalNameStaticSet` (public) with 0 fields (derives: derive) at line 1
  - `PrefixStaticSet` (public) with 0 fields (derives: derive) at line 1
  - `NamespaceStaticSet` (public) with 0 fields (derives: derive) at line 1
### ./target/debug/build/unicode_names2-294286d27e2b0fd5/out/generated_alias.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/unicode_names2-294286d27e2b0fd5/out/generated_phf.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/unicode_names2-294286d27e2b0fd5/out/generated.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/typenum-70b58b9af9edf813/out/tests.rs

**Language:** rust
**Total Symbols:** 1746
**Functions:** 1743 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `test_0_BitAnd_0` (private) at line 1
  - `test_0_BitOr_0` (private) at line 1
  - `test_0_BitXor_0` (private) at line 1
  - `test_0_Shl_0` (private) at line 1
  - `test_0_Shr_0` (private) at line 1
  - `test_0_Add_0` (private) at line 1
  - `test_0_Mul_0` (private) at line 1
  - `test_0_Pow_0` (private) at line 1
  - `test_0_Min_0` (private) at line 1
  - `test_0_Max_0` (private) at line 1
  - ... and 1733 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
### ./target/debug/build/markup5ever-ecc608dea4413a88/out/named_entities.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/markup5ever-ecc608dea4413a88/out/generated.rs

**Language:** rust
**Total Symbols:** 3
**Functions:** 0 | **Structs:** 3 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Structs:**
  - `LocalNameStaticSet` (public) with 0 fields (derives: derive) at line 1
  - `PrefixStaticSet` (public) with 0 fields (derives: derive) at line 1
  - `NamespaceStaticSet` (public) with 0 fields (derives: derive) at line 1
### ./target/debug/build/typenum-2e7e72dbb708f88e/out/tests.rs

**Language:** rust
**Total Symbols:** 1746
**Functions:** 1743 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `test_0_BitAnd_0` (private) at line 1
  - `test_0_BitOr_0` (private) at line 1
  - `test_0_BitXor_0` (private) at line 1
  - `test_0_Shl_0` (private) at line 1
  - `test_0_Shr_0` (private) at line 1
  - `test_0_Add_0` (private) at line 1
  - `test_0_Mul_0` (private) at line 1
  - `test_0_Pow_0` (private) at line 1
  - `test_0_Min_0` (private) at line 1
  - `test_0_Max_0` (private) at line 1
  - ... and 1733 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
### ./target/debug/build/paiml-mcp-agent-toolkit-b2559051a3059eb3/out/compressed_templates.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
### ./target/debug/build/rustpython-parser-0b145ec5d5dc9579/out/keywords.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/markup5ever-e3f53f601332663c/out/named_entities.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/debug/build/markup5ever-e3f53f601332663c/out/generated.rs

**Language:** rust
**Total Symbols:** 3
**Functions:** 0 | **Structs:** 3 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Structs:**
  - `LocalNameStaticSet` (public) with 0 fields (derives: derive) at line 1
  - `PrefixStaticSet` (public) with 0 fields (derives: derive) at line 1
  - `NamespaceStaticSet` (public) with 0 fields (derives: derive) at line 1
### ./target/debug/build/mime_guess-d5c98e1139e53e1f/out/mime_types_generated.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/llvm-cov-target/debug/build/paiml-mcp-agent-toolkit-c714e36b2b2eaa5c/out/compressed_templates.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 4

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
### ./target/llvm-cov-target/debug/build/selectors-95212bf9bb3b8251/out/ascii_case_insensitive_html_attributes.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/llvm-cov-target/debug/build/mime_guess-665e9edc00a94481/out/mime_types_generated.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/llvm-cov-target/debug/build/markup5ever-3cd0004c8e57608b/out/named_entities.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/llvm-cov-target/debug/build/markup5ever-3cd0004c8e57608b/out/generated.rs

**Language:** rust
**Total Symbols:** 3
**Functions:** 0 | **Structs:** 3 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Structs:**
  - `LocalNameStaticSet` (public) with 0 fields (derives: derive) at line 1
  - `PrefixStaticSet` (public) with 0 fields (derives: derive) at line 1
  - `NamespaceStaticSet` (public) with 0 fields (derives: derive) at line 1
### ./target/llvm-cov-target/debug/build/crunchy-c377f7e94b1940db/out/lib.rs

**Language:** rust
**Total Symbols:** 4
**Functions:** 4 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0

**Functions:**
  - `invalid_range` (private) at line 1
  - `start_at_one_with_step` (private) at line 1
  - `start_at_one` (private) at line 1
  - `test_all` (private) at line 1
### ./target/llvm-cov-target/debug/build/rustpython-parser-fe744392a3696a50/out/keywords.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/llvm-cov-target/debug/build/typenum-32fc9a673cbfb5e6/out/tests.rs

**Language:** rust
**Total Symbols:** 1746
**Functions:** 1743 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 3

**Functions:**
  - `test_0_BitAnd_0` (private) at line 1
  - `test_0_BitOr_0` (private) at line 1
  - `test_0_BitXor_0` (private) at line 1
  - `test_0_Shl_0` (private) at line 1
  - `test_0_Shr_0` (private) at line 1
  - `test_0_Add_0` (private) at line 1
  - `test_0_Mul_0` (private) at line 1
  - `test_0_Pow_0` (private) at line 1
  - `test_0_Min_0` (private) at line 1
  - `test_0_Max_0` (private) at line 1
  - ... and 1733 more functions

**Key Imports:**
  - `use statement` at line 1
  - `use statement` at line 1
  - `use statement` at line 1
### ./target/llvm-cov-target/debug/build/unicode_names2-94aa83e4b53a7879/out/generated_alias.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/llvm-cov-target/debug/build/unicode_names2-94aa83e4b53a7879/out/generated_phf.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
### ./target/llvm-cov-target/debug/build/unicode_names2-94aa83e4b53a7879/out/generated.rs

**Language:** rust
**Total Symbols:** 0
**Functions:** 0 | **Structs:** 0 | **Enums:** 0 | **Traits:** 0 | **Impls:** 0 | **Modules:** 0 | **Imports:** 0
## Complexity Hotspots

| Function | File | Cyclomatic | Cognitive |
|----------|------|------------|-----------|

## Code Churn Analysis

**Summary:**
- Total Commits: 0
- Files Changed: 0

**Top Changed Files:**
| File | Commits | Authors |
|------|---------|---------|

## Technical Debt Analysis

**SATD Summary:**

## Dead Code Analysis

**Summary:**
- Dead Functions: 0
- Total Dead Lines: 0

## Defect Probability Analysis

**Risk Assessment:**
- Total Defects Predicted: 0
- Defect Density: 0.00 defects per 1000 lines

---
Generated by deep-context v0.21.0
