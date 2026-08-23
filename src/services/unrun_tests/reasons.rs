//! Why a set of tests is allowed to be unrun.
//!
//! Modelled on `orphan-ledger` in `.github/workflows/feature-matrix.yml`, which
//! requires every orphan FEATURE to be tested or explained. That job says
//! nothing about a TEST behind a feature no leg runs, which is how four
//! regression tests shipped in the 3.32.0 cycle having never executed once.
//!
//! A bucket with no entry here is a hard failure: "nobody noticed the test
//! existed" stops being a possible outcome. Every reason below is a measured
//! fact about this tree, not an intention. Where the reason recorded elsewhere
//! in the repository is WRONG, the correction is written here rather than the
//! wrong reason repeated.

/// `bucket -> reason`.
///
/// The bucket key names what a fix would have to change, in three shapes:
/// * a comma-joined feature set — the delta from a DEFAULT build that no
///   single CI leg supplies (`mcp-integration,mutation-testing` is a
///   COMBINATION, not two independently-covered flags);
/// * `not(f)` — the test needs `f` OFF and every leg has it ON;
/// * `<unsatisfiable>` — no feature assignment compiles it at all.
pub const REASONS: &[(&str, &str)] = &[
    (
        "<unsatisfiable>",
        "NOT a clean bill of health — the strongest finding in this ledger. \
         14 of these are `#[cfg(all(feature = \"F\", not(feature = \"F\")))]`: a \
         `test_..._without_feature` body written to cover the feature-OFF branch, \
         placed inside a module already gated ON that feature. The remaining 4 are \
         `#[cfg(any())]`, which is `false` by definition. No `--features` \
         invocation can ever compile any of them; only moving the bodies out of \
         the gated module can.",
    ),
    (
        "agent-daemon",
        "reachable from neither `default` nor `full`; orphan-ledger excludes it \
         as 'clippy state unremeasured since the mcp-integration fix'.",
    ),
    (
        "agents-md",
        "reachable from neither `default` nor `full`; orphan-ledger excludes it \
         as 'clippy state unremeasured since the mcp-integration fix' and already \
         records '+247 lib tests never run'.",
    ),
    (
        "analytics-gpu,analytics-simd",
        "ONE of the four tests this gate was built for, and still unrun. \
         orphan-ledger's stated reason — 'needs GPU hardware; ubuntu-latest has \
         none' — is measurably wrong about it: the single test behind \
         `analytics-gpu` asserts that `is_gpu_available()` is FALSE and that the \
         selector never picks `Backend::Gpu`, so it wants a GPU-LESS runner, which \
         is exactly what ubuntu-latest is. The real cost is the wgpu + pollster + \
         aprender-db/gpu compile a leg would add.",
    ),
    (
        "broken-tests",
        "deliberate non-compiling quarantine (#1023). These bodies do not build \
         under any feature set, so no leg can run them by construction.",
    ),
    (
        "broken-tests,mcp-integration",
        "quarantined (#1023); the quarantine alone is sufficient.",
    ),
    (
        "broken-tests,not(cpp-ast)",
        "quarantined (#1023); the quarantine alone is sufficient.",
    ),
    (
        "broken-tests,ruchy-ast",
        "quarantined (#1023); the quarantine alone is sufficient.",
    ),
    (
        "broken-tests,unified-protocol",
        "quarantined (#1023). `unified-protocol` IS a leg, so the quarantine is \
         the only thing keeping these unrun.",
    ),
    (
        "csharp-ast",
        "Sprint 46 removed the dependency and kept the flag so the surviving \
         `#[cfg]` guards compile; `csharp-ast` is in neither `default` nor `full`.",
    ),
    (
        "dap",
        "Debug Adapter Protocol; orphan-ledger has it 'lib-test delta unmeasured; \
         compile-checked only'. This ledger measures the delta: 231 tests.",
    ),
    (
        "dap,tui",
        "the DAP timeline terminal UI; both flags are compile-checked only.",
    ),
    (
        "deep-wasm,mcp-integration",
        "a COMBINATION no leg supplies: `full` reaches `deep-wasm` and the \
         `mcp-integration` leg reaches the other, and no invocation enables both.",
    ),
    (
        "demo,tui",
        "a COMBINATION no leg supplies: `full` reaches `demo`, nothing reaches \
         `tui` with it.",
    ),
    ("git-lib", "orphan-ledger has it 'compile-checked only'."),
    ("github-api", "orphan-ledger has it 'compile-checked only'."),
    (
        "java-ast",
        "Sprint 46 removed the dependency and kept the flag; in neither \
         `default` nor `full`.",
    ),
    (
        "java-ast,mcp-integration",
        "a COMBINATION no leg supplies; `java-ast` is a dep-less back-compat flag.",
    ),
    (
        "java-ast,mcp-integration,scala-ast",
        "the JVM cross-language MCP tests; a three-way combination no leg supplies.",
    ),
    (
        "kotlin-ast",
        "orphan-ledger has it 'lib-test delta unmeasured; compile-checked only'. \
         This ledger measures the delta: 32 tests.",
    ),
    (
        "not(mcp-http)",
        "the HTTP-ABSENT branch. `mcp-http` entered `default` in 3.32.0, so \
         every leg now compiles it in and nothing exercises the code that runs \
         when it is out — chiefly the `serve` refusal that used to print \
         '[HTTP NOT COMPILED IN this build]'. That path still ships for anyone \
         building `--no-default-features`, and it is now untested by every CI \
         leg. Recorded rather than deleted: the branch is reachable by a real \
         build configuration, so removing the tests would hide it instead of \
         covering it.",
    ),
    (
        "mcp-integration,mutation-testing",
        "a COMBINATION no leg supplies, and the reason a per-feature ledger \
         would have missed these: `full` enables `mutation-testing`, the \
         `mcp-integration` leg enables the other, and each feature therefore \
         looks covered while these 21 tests compile in neither.",
    ),
    (
        "mcp-integration,scala-ast",
        "a COMBINATION no leg supplies; `scala-ast` is a dep-less back-compat flag.",
    ),
    (
        "not(c-ast)",
        "needs `c-ast` OFF to cover the parser-absent branch; `c-ast` is in \
         `core-languages`, so every leg has it ON. Running it needs a \
         `--no-default-features` leg, which no job has.",
    ),
    (
        "not(cpp-ast)",
        "needs `cpp-ast` OFF; in `core-languages`, so ON in every leg.",
    ),
    (
        "not(python-ast)",
        "needs `python-ast` OFF; in `core-languages`, so ON in every leg.",
    ),
    (
        "not(viz)",
        "the `viz` stub types, compiled only when `viz` is OFF. `viz` is in \
         `default`, so every leg has it ON and no leg builds the stubs.",
    ),
    (
        "org-intelligence",
        "the feature survives for `pmat org localize`; its dependency did not \
         (aprender-orchestrate 0.41 removed the API). orphan-ledger \
         'compile-checked only'.",
    ),
    (
        "prometheus-metrics",
        "orphan-ledger has it 'compile-checked only'.",
    ),
    (
        "scala-ast",
        "Sprint 46 removed the dependency and kept the flag; in neither \
         `default` nor `full`.",
    ),
    (
        "simd",
        "orphan-ledger has it 'compile-checked only'. These are the \
         SIMD-equals-scalar property tests, so nothing checks that equivalence.",
    ),
    ("tui", "orphan-ledger has it 'compile-checked only'."),
];

#[must_use]
pub fn reason(bucket: &str) -> Option<&'static str> {
    REASONS.iter().find(|(b, _)| *b == bucket).map(|(_, r)| *r)
}
