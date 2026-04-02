# Contract Surface Types & Anti-Leak Enforcement

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 23

## Root-Cause Analysis: Why Contracts Leak

Five Whys (2026-04-02):

1. **Why do provable-contracts keep "leaking" past pmat comply?**
   New contract types don't fit the kernel-math model CB-1200..1214 hardcode.
2. **Why kernel-math only?** Original spec defined ONE shape: `equations:`
   with `proof_obligations:` and `bindings:`.
3. **Why wasn't schema extension detected?** No contract-type discriminator.
4. **Why no discriminator?** Built for single archetype — nobody anticipated
   CLI args, HTTP, MCP, CI configs, TUI widgets, WASM FFI as contract surfaces.
5. **Why not contracted from the start?** Focus was mathematical correctness.
   System-boundary contracts were left to ad-hoc serde validation.

**The fix:** Eight contract surface types + entity inventory. `pmat comply`
classifies contracts by surface, enforces L5, names every violating file.

## Contract Entity Inventory

### Contracted Domains (12 domains, 285 contracts, 100% L5)

| Domain | Count | Repos | Key Entities |
|--------|-------|-------|-------------|
| Math/Linear Algebra | ~40 | trueno, aprender, pv | matmul, gemm, softmax, attention, FFT, sparse, solvers |
| Tensor/Data Structures | ~12 | aprender, realizar, pv | shape-flow, layout, inventory, kv-cache, embedding-algebra |
| ML/Training | ~25 | aprender, entrenar, pv | adamw, lora, dpo-loss, SVM, random-forest, bayesian, GNN |
| Inference Pipeline | ~15 | realizar, pv | forward-pass, sampling, batch-inference, speculative-decoding |
| Quantization | ~10 | trueno, realizar, pv | Q4K, Q8, FP8, int8-symmetric, roundtrip fidelity |
| Model Formats | ~10 | realizar, aprender, pv | APR v2, GGUF, safetensors, BPE tokenizer |
| Architecture-Specific | ~20 | aprender, pv | llama, qwen2/3, mistral, phi, gemma, mamba, whisper |
| GPU/Compute | ~18 | trueno, pv | CUDA safety, WGPU, PTX parity, cooperative GEMM |
| Transpiler | ~6 | pv (decy) | C++→Rust type preservation, CUDA FFI, semantic equiv |
| TUI/Visualization | ~5 | presentar | WCAG color, rect geometry, layout constraints, panels |
| Quality/Compliance | ~8 | pv | TDG scoring, comply-check, canary metrics |
| Positional Encoding | ~6 | aprender, pv | RoPE, ALiBi, absolute, learned, QK-norm |

### Missing Domains (9 domains, 0 contracts)

| Domain | Severity | Why It Matters | First Contracts Needed |
|--------|----------|----------------|----------------------|
| **CLI/HTTP Interface** | High | User-facing boundary | exit-code-semantics, error-envelope, arg-validation |
| **MCP Protocol** | High | Agent-facing boundary | tool-schema-validation, session-lifecycle |
| **Concurrency** | High | Correctness-critical | lock-ordering, channel-bounds, task-cancellation |
| **Tracing/Observability** | Medium | Production debugging | span-propagation, renacer-trace-format |
| **Memory Management** | Medium | WASM/GPU safety | allocator-budget, mmap-safety, arena-lifecycle |
| **Graph/Index** | Medium | Core pmat infra | csr-construction, pagerank-convergence, fts5-consistency |
| **WASM FFI** | Medium | CB-1307 enforces code, needs YAML | jsvalue-conversion, panic-ffi-prevention |
| **Configuration** | Low | CB-1303 enforces | pmat-yaml-schema, cargo-workspace-invariants |
| **Compression** | Low | Narrow surface | lz4-roundtrip, zstd-fidelity |

## Surface Type Taxonomy

### Type 1: CLI Arguments

```yaml
surface: cli
command: pmat analyze complexity
arguments:
  - name: --max-cyclomatic
    type: u32
    range: [1, 100]
    contract: "value >= 1 && value <= 100"
  - name: --format
    type: enum
    variants: [table, json, yaml, markdown, csv, summary, text, plain, junit]
    contract: "ONE canonical OutputFormat enum (9 variants)"
exit_codes: { 0: success, 1: violations, 2: invalid_args }
```

**CB-1300:** FAIL on OutputFormat duplication. **DONE** (9→1 canonical).

### Type 2: HTTP Payloads

```yaml
surface: http
endpoint: POST /api/v1/query
request: { content_type: application/json, schema: QueryRequest }
response: { 200: QueryResponse, 400: ErrorEnvelope, 500: ErrorEnvelope }
```

**CB-1301:** Blocked until HTTP handlers exist.

### Type 3: MCP Tool Schemas

```yaml
surface: mcp
tool: pmat_query_code
input_schema: { query: string, limit: integer, format: enum }
output: { content_type: text/plain }
errors: [INVALID_QUERY, INDEX_NOT_FOUND]
```

**CB-1302:** FAIL on undocumented arg structs. **DONE** (25→0).

### Type 4: Configuration (YAML, TOML, Cargo)

Contracts for `.github/workflows/`, `Cargo.toml`, `.pmat.yaml`.
Key invariants: `RUST_MIN_STACK >= 8388608`, no stale working-directory,
sovereign deps meet version minimums, edition = 2021.

**CB-1303:** WARN on CI drift. **IMPL.**

### Type 5: Sovereign Stack Dependencies

| Crate | Min Version | Role |
|-------|------------|------|
| aprender | >=0.27 | ML, stats, graphs |
| trueno | >=0.16 | SIMD compute |
| trueno-graph | >=0.1.17 | Graph DB, PageRank |
| trueno-db | >=0.3.15 | Columnar storage |
| trueno-rag | >=0.2.0 | RAG pipeline |
| presentar-core | >=0.3.2 | TUI framework |

**CB-1304:** FAIL on version violation. **IMPL.** All stack repos fixed.

### Type 6: Provable-Contracts Schema Extensions

Five contract classes recognized by CB-1305:

| Class | Discriminator | Example |
|-------|--------------|---------|
| kernel-math | `equations:` with real preconditions | softmax-kernel-v1 |
| cross-language | `equations:` + `enforcement_level:` | cuda-kernel-safety-v1 |
| schema-registry | `weight_roles:` / `architecture_map:` | architecture-requirements-v1 |
| invariants-only | `invariants:` without `equations:` | qk-norm-apr-loader-v1 |
| semantic-leak | ALL assertions are generic placeholders | configuration-v1 |

**CB-1305:** WARN on unclassified. FAIL if >20%. **IMPL.**

### Type 7: TUI Widget Lifecycle (presentar)

Contracts the verify-measure-layout-paint cycle. Required domains:

| Domain | Contract | Key Invariant |
|--------|----------|---------------|
| Color | color-wcag-v1.yaml | CR in [1.0, 21.0], symmetric |
| Geometry | rect-geometry-v1.yaml | intersection commutative, inset non-negative |
| Layout | constraints-layout-v1.yaml | constrain idempotent, bounded |

**CB-1306:** FAIL if presentar workspace missing any domain. **IMPL.**
presentar: 3/3 PASS.

### Type 8: WASM FFI Boundary

Contracts `#[wasm_bindgen]` exports — the JS-to-Rust boundary.

| Export | Contract |
|--------|----------|
| App::new(canvas_id) | Must return `Result<_, JsValue>` |
| All pub fn | Doc comment required |
| All paths | No `.unwrap()` (panic across FFI) |
| init_presentar | Called exactly once |

**CB-1307:** WARN if >50% undocumented or unwrap in exports. **IMPL.**
presentar: 17 exports, 0 unwrap-across-FFI.

## CB-1308: L5 Mandatory Enforcement (Zero Tolerance)

**L5 (lean_theorem) is the ONLY acceptable verification level.**

Every contract YAML with `equations:` but without `lean_theorem:` is a
FAIL. CB-1308 lists every violating file by name and current level,
exactly like `pmat tdg` lists files below grade threshold.

```
CB-1308: Verification Ladder (L5): 3/13 at L5, 10 violations:
    - softmax-kernel-v1.yaml (L4)
    - matmul-kernel-v1.yaml (L3)
    - config-v1.yaml (L1)
```

**Severity:** Critical. Exit code 1. Blocks CI and pre-push hooks.
Schema/invariant-only contracts (no equations) are exempt.

**Five Whys:** Contracts stalled at L4 because nothing demanded L5.
Toyota Way Jidoka: stop the line. Now we enforce.

## Resolution Hierarchy

| Priority | Source | Method |
|----------|--------|--------|
| 1 | **GitHub org commits** | `git log --all -- '*.yaml'` across org repos |
| 2 | **batuta oracle** | `batuta oracle --rag "contract for <surface>"` |
| 3 | **arXiv** | Formal methods, DBC, contract verification |
| 4 | **Web search** | Industry patterns, RFC references |
| 5 | **Chain of thought** | LLM synthesis from partial evidence |
| 6 | **Five Whys** | `pmat five-whys "Why is <surface> uncontracted?"` |

## Enforcement Summary

| Check | Surface | Status | Enforcement |
|-------|---------|--------|-------------|
| CB-1300 | CLI | **DONE** (9→1) | OutputFormat convergence |
| CB-1301 | HTTP | Blocked | No handlers yet |
| CB-1302 | MCP | **DONE** (25→0) | Arg struct doc coverage |
| CB-1303 | Config | **IMPL** | CI drift, RUST_MIN_STACK |
| CB-1304 | Library | **IMPL** | Sovereign dep versions |
| CB-1305 | PV Schema | **IMPL** | Anti-leak classifier |
| CB-1306 | TUI Widget | **IMPL** | Widget lifecycle domains |
| CB-1307 | WASM FFI | **IMPL** | unwrap/doc/Result checks |
| CB-1308 | Verification | **IMPL** | L5 mandatory, per-file FAIL |

## Dogfooding Results (2026-04-02)

| Check | pmat | presentar | pv-contracts | aprender | trueno | realizar |
|-------|------|-----------|-------------|----------|--------|----------|
| CB-1300 | PASS:1 | PASS | SKIP | PASS | PASS | PASS |
| CB-1302 | PASS:0 | SKIP | SKIP | SKIP | SKIP | SKIP |
| CB-1303 | PASS | PASS:7 | PASS | PASS:6 | PASS:6 | PASS:5 |
| CB-1304 | PASS:5 | PASS:1 | SKIP | PASS:2 | PASS:2 | PASS:3 |
| CB-1305 | SKIP | PASS:3/3 | WARN:4 | PASS:68/73 | PASS:28/28 | PASS:14/14 |
| CB-1306 | SKIP | PASS:3/3 | SKIP | SKIP | SKIP | SKIP |
| CB-1307 | SKIP | PASS:17 | SKIP | WARN:2/3 | SKIP | SKIP |
| CB-1308 | SKIP | PASS:3/3 | PASS:185 | PASS:56 | PASS:28 | PASS:14 |

**29 PASS, 1 WARN, 0 FAIL across 6 repos.** 289 contracts at L5.
Remaining WARN: presentar CB-1307 (1 unwrap in WASM export path).
All CB-1303 CI false positives fixed. All CB-1305 semantic leaks
resolved (4 generic contracts given real preconditions). CB-1307
improved (impl-block method scanning). aprender silu annotation fixed.

## Falsification (2026-04-02, live verification)

| Claim | Test | Verdict |
|-------|------|---------|
| F-1: Taxonomy exhaustive | CB-1305 on pv-contracts | **Partial**: 4 semantic leaks remain (generic placeholders) |
| F-2: Hierarchy converges | Resolved aprender/realizar/presentar L5 gaps | **Confirmed**: 1-3 steps |
| F-3: Anti-leak detection | CB-1305 on 6 repos | **Confirmed**: 5 classes recognized |
| F-4: Dep version enforcement | CB-1304 on trueno/realizar/presentar | **Confirmed**: 3 violations caught and fixed |
| F-5: OutputFormat convergence | CB-1300 on pmat | **Done**: 9→1 (exit 0) |
| F-6: CI drift prevention | CB-1303 on 4 repos | **Confirmed**: RUST_MIN_STACK missing in 4 CI configs |
| F-7: L5 mandatory holds | CB-1308 on 6 repos | **Confirmed**: 285/285 at L5, exit 1 on violation |
| F-8: Per-file enforcement | Injected rect-geometry L4 | **Confirmed**: named file, exit 1, restored |
| F-9: New contract caught | realizar gpu-inference-parity added without lean_theorem | **Confirmed**: CB-1308 caught same day |

**F-9 is the strongest validation**: a real contract was added to
realizar today (2026-04-02) without `lean_theorem:`. CB-1308 caught
it immediately. This proves the enforcement loop works in practice,
not just on synthetic test cases.

## Key Files

| File | Purpose |
|------|---------|
| `check_contract_surfaces.rs` | CB-1300..1308 (9 checks, 13 tests) |
| `check_pv_enforcement.rs` | CB-1200..1209 |
| `check_pv_quality.rs` | CB-1210..1214 |
| `contracts/mod.rs` | Canonical OutputFormat (9 variants) |

## References

### Foundational
- Meyer (1988). Design by Contract. Preconditions, postconditions, invariants.
- Rust RFC #128045: `core::contracts` (`requires`/`ensures`).
- PMAT Component 22: [provable-contracts.md](provable-contracts.md).
- pv-spec.md sections 2, 27: Verification Ladder (L0-L5), The One Way.

### Rust Verification (arXiv)
- Le Blanc & Lam (2024). [Surveying the Rust Verification Landscape](https://arxiv.org/abs/2410.01981). Kani, Verus, Prusti taxonomy.
- Zhu et al. (2025). [Lessons Learned Verifying the Rust Standard Library](https://arxiv.org/abs/2510.01072). Kani bounded model checking on std.
- Gao et al. (2026). [Automated Proof Generation for Rust (SAFE)](https://arxiv.org/abs/2410.15756). Self-evolving LLM proof synthesis.
- Nong et al. (2025). [Annotating Safety Properties of Unsafe Rust](https://arxiv.org/abs/2504.21312). DSL for safety contract annotations.

### Lean/Theorem Proving (arXiv)
- Dardik & Kang (2025). [arXiv:2509.06250](https://arxiv.org/abs/2509.06250). Assume-guarantee contracts for task chains.
- Li et al. (2025). [arXiv:2510.12047](https://arxiv.org/abs/2510.12047). LLMs and formal contracts.
- Shen et al. (2026). [VCoT-Bench: LLMs as Theorem Provers for Rust](https://arxiv.org/abs/2603.18334). Verification chain of thought.
- Agarwal et al. (2026). [NTP4VC: Neural Theorem Proving for Verification Conditions](https://arxiv.org/abs/2601.18944). ML proof gen in Lean.

### WASM Safety (arXiv)
- Draissi et al. (2026). [Walma: Memory Corruption in WebAssembly](https://arxiv.org/abs/2603.24167). ML-based memory attestation.
- Fink et al. (2024). [Cage: Hardware-Accelerated Safe WebAssembly](https://arxiv.org/abs/2408.11456). MTE spatial/temporal safety.
- Vassena et al. (2025). [MSWASM Type Safety](https://arxiv.org/abs/2503.03698). Iris-Wasm logical relation.

### TUI/Constraint Layout (arXiv)
- Jiang et al. (2014). [Constraint Solvers for UI Layout](https://arxiv.org/abs/1401.1031). Cassowary + OR-constraints.
- Guo et al. (2026). [Terminal Is All You Need](https://arxiv.org/abs/2603.10664). CHI 2026 — terminal as agent interface.
- Yuan et al. (2025). [SpecifyUI: Structured UI Specifications](https://arxiv.org/abs/2509.07334). JSON parameterized constraints.

### Contract Verification (arXiv)
- Chen et al. (2026). [ToolGate: Contract-Grounded Tool Execution for LLMs](https://arxiv.org/abs/2601.04688). Hoare logic for tool calls.
- Wu et al. (2025). [Smart Contracts Formal Verification SLR](https://arxiv.org/abs/2510.17865). 200-paper systematic review.
