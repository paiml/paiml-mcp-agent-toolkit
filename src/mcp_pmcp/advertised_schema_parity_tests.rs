//! Every parameter a tool HONOURS must be a parameter it ADVERTISES.
//!
//! REGRESSION: `analyze_satd` read `include_tests` — it selects whether test
//! files and `#[cfg(test)]` blocks are scanned, so it moves the count — while
//! `tools/list` advertised only `{paths, include_resolved}`:
//!
//! ```text
//! tools/list  analyze_satd.inputSchema.properties -> paths, include_resolved
//! handler     SatdArgs                            -> paths, include_resolved, include_tests
//!
//! analyze_satd {"paths":[fixture]}                       -> total_satd 2
//! analyze_satd {"paths":[fixture],"include_tests":true}  -> total_satd 3
//! ```
//!
//! A hidden parameter means two callers sending the documented arguments can
//! get different answers, and the schema gives neither of them a way to find
//! out why. Either advertise it or stop honouring it; this one is advertised,
//! because the behaviour is wanted (it is `pmat analyze satd --include-tests`,
//! #997, reaching MCP, and `analyze_dead_code` next door already advertises the
//! identical flag).
//!
//! The sweep below is the drift guard: it scrapes the field list off each
//! handler's `#[derive(Deserialize)]` args struct and requires the tool's own
//! `metadata()` to name every one of them. It only sees TOP-LEVEL fields —
//! nested config objects (`quality_config`) are checked as a single property.

use pmcp::ToolHandler;
use std::collections::BTreeSet;

/// Field names an args struct deserializes, scraped from its declaration.
///
/// Deliberately reads the SOURCE rather than a hand-written list: a
/// hand-written list is a third copy of the same fact and would drift exactly
/// the way the schema did.
fn args_fields(source: &str, struct_name: &str) -> BTreeSet<String> {
    let needle = format!("struct {struct_name} {{");
    let start = source
        .find(&needle)
        .unwrap_or_else(|| panic!("{struct_name} is not declared in the source given"));

    let mut fields = BTreeSet::new();
    for line in source[start + needle.len()..].lines().skip(1) {
        let line = line.trim();
        if line == "}" {
            return fields;
        }
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        let name = line
            .split(':')
            .next()
            .unwrap_or_default()
            .trim()
            .trim_start_matches("pub ")
            .trim();
        if !name.is_empty() {
            fields.insert(name.to_string());
        }
    }
    panic!("{struct_name}'s declaration is not closed by a bare `}}`");
}

/// Properties a tool's `metadata()` advertises to `tools/list`.
fn advertised(tool: &dyn ToolHandler) -> BTreeSet<String> {
    let info = tool
        .metadata()
        .expect("a registered tool must advertise metadata");
    info.input_schema["properties"]
        .as_object()
        .expect("inputSchema must carry a properties object")
        .keys()
        .cloned()
        .collect()
}

/// A field a handler parses ON PURPOSE without advertising it, and why.
///
/// One entry only, and it is the opposite of a hidden parameter:
/// `analyze_deep_context` parses `include_patterns` so it can REFUSE it —
/// dropping the field from the struct would make serde ignore it in silence
/// again, which is how it spent a release advertised as "accepted but not yet
/// applied as a filter" while being wired to nothing.
const DELIBERATELY_UNADVERTISED: &[(&str, &str)] = &[("analyze_deep_context", "include_patterns")];

fn assert_parity(tool: &dyn ToolHandler, source: &str, struct_name: &str) {
    let name = tool
        .metadata()
        .expect("a registered tool must advertise metadata")
        .name;
    let advertised = advertised(tool);

    for field in args_fields(source, struct_name) {
        if DELIBERATELY_UNADVERTISED.contains(&(name.as_str(), field.as_str())) {
            assert!(
                !advertised.contains(&field),
                "{name} advertises {field}, which is on the deliberately-refused list"
            );
            continue;
        }
        assert!(
            advertised.contains(&field),
            "{name} honours `{field}` ({struct_name}) but tools/list advertises only {advertised:?} \
             — a hidden parameter that changes the answer"
        );
    }
}

const ANALYZE_COMPLEXITY_SRC: &str = include_str!("analyze_complexity_handler.rs");
const ANALYZE_DEBT_SRC: &str = include_str!("analyze_debt_handlers.rs");
const ANALYZE_METRICS_SRC: &str = include_str!("analyze_metrics_handlers.rs");
const ANALYZE_TDG_SRC: &str = include_str!("analyze_tdg_tool_handlers.rs");
const ANALYZE_FORENSICS_SRC: &str = include_str!("analyze_forensics_handlers.rs");
const QUALITY_SRC: &str = include_str!("quality_handlers.rs");
const QUALITY_PROXY_SRC: &str = include_str!("quality_proxy_handler.rs");
const PDMT_SRC: &str = include_str!("pdmt_handler.rs");
const CONTEXT_SRC: &str = include_str!("context_handlers_context.rs");
const GIT_SRC: &str = include_str!("context_handlers_git.rs");
const TDG_SRC: &str = include_str!("tdg_handlers.rs");

/// THE defect: `include_tests` moves the number and was invisible in
/// `tools/list`.
#[test]
fn analyze_satd_advertises_the_include_tests_it_honours() {
    use crate::mcp_pmcp::analyze_handlers::SatdTool;

    let advertised = advertised(&SatdTool::new());
    assert!(
        advertised.contains("include_tests"),
        "analyze_satd honours include_tests but advertises only {advertised:?}"
    );

    // Advertised as the boolean it actually is, and described — an entry that
    // says nothing is not documentation.
    let schema = SatdTool::new().metadata().expect("metadata").input_schema;
    let property = &schema["properties"]["include_tests"];
    assert_eq!(property["type"], serde_json::json!("boolean"));
    assert!(
        property["description"]
            .as_str()
            .is_some_and(|d| d.contains("test")),
        "include_tests must be described, got {property}"
    );

    // It stays OPTIONAL: advertising it must not start requiring it.
    let required: Vec<&str> = schema["required"]
        .as_array()
        .expect("required")
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect();
    assert_eq!(required, vec!["paths"]);
}

/// The reason it is advertised rather than un-honoured: it genuinely changes
/// the answer. If this ever stops being true, the fix is to delete the
/// parameter, not to document a no-op.
#[tokio::test]
async fn include_tests_changes_the_satd_answer_so_it_has_to_be_documented() {
    use crate::mcp_pmcp::tool_functions::analyze_satd;

    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("src")).expect("src");
    std::fs::create_dir_all(dir.path().join("tests")).expect("tests");
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname=\"f\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
    )
    .expect("manifest");
    std::fs::write(
        dir.path().join("src/lib.rs"),
        "// TODO: production marker\npub fn f() -> i32 { 1 }\n",
    )
    .expect("lib");
    std::fs::write(
        dir.path().join("tests/it.rs"),
        "// TODO: integration-test marker\n#[test] fn t() { assert_eq!(1, 1); }\n",
    )
    .expect("it");

    let paths = [dir.path().to_path_buf()];
    let off = analyze_satd(&paths, false, false).await.expect("satd off");
    let on = analyze_satd(&paths, false, true).await.expect("satd on");

    let count = |v: &serde_json::Value| v["results"]["total_satd"].as_u64();
    assert!(
        count(&on) > count(&off),
        "include_tests changed nothing ({:?} vs {:?}) — then it must not be a parameter at all",
        count(&off),
        count(&on)
    );
}

/// The sweep: no other registered tool may hold a parameter back.
#[test]
fn no_registered_tool_honours_an_unadvertised_parameter() {
    use crate::mcp_pmcp::analyze_handlers::{
        AnalyzeBigOTool, AnalyzeDagTool, AnalyzeDeepContextTool, ComplexityTool, DeadCodeTool,
        HardcodedPathsTool, ReachabilityTool, SatdTool, TdgCompareTool, TdgTool, VacuousTestsTool,
    };
    use crate::mcp_pmcp::context_handlers::{
        ContextGenerateTool, ContextSummaryTool, GitStatusTool,
    };
    use crate::mcp_pmcp::pdmt_handler::PdmtTool;
    use crate::mcp_pmcp::quality_handlers::QualityGateTool;
    use crate::mcp_pmcp::quality_proxy_handler::QualityProxyTool;
    use crate::mcp_pmcp::tdg_handlers::{
        TdgAnalyzeWithStorageTool, TdgConfigureStorageTool, TdgHealthCheckTool,
        TdgPerformanceMetricsTool, TdgStorageManagementTool, TdgSystemDiagnosticsTool,
    };

    assert_parity(
        &ComplexityTool::new(),
        ANALYZE_COMPLEXITY_SRC,
        "ComplexityArgs",
    );
    assert_parity(&SatdTool::new(), ANALYZE_DEBT_SRC, "SatdArgs");
    assert_parity(&DeadCodeTool::new(), ANALYZE_DEBT_SRC, "DeadCodeArgs");
    assert_parity(
        &AnalyzeDagTool::new(),
        ANALYZE_METRICS_SRC,
        "AnalyzeDagArgs",
    );
    assert_parity(
        &AnalyzeBigOTool::new(),
        ANALYZE_METRICS_SRC,
        "AnalyzeBigOArgs",
    );
    assert_parity(
        &AnalyzeDeepContextTool::new(),
        ANALYZE_METRICS_SRC,
        "AnalyzeDeepContextArgs",
    );
    // #1029: the three forensic analyzers, registered in this cycle.
    assert_parity(
        &ReachabilityTool::new(),
        ANALYZE_FORENSICS_SRC,
        "ReachabilityArgs",
    );
    assert_parity(
        &HardcodedPathsTool::new(),
        ANALYZE_FORENSICS_SRC,
        "HardcodedPathsArgs",
    );
    assert_parity(
        &VacuousTestsTool::new(),
        ANALYZE_FORENSICS_SRC,
        "VacuousTestsArgs",
    );
    assert_parity(&TdgTool::new(), ANALYZE_TDG_SRC, "TdgArgs");
    assert_parity(&TdgCompareTool::new(), ANALYZE_TDG_SRC, "TdgCompareArgs");
    assert_parity(&QualityGateTool::new(), QUALITY_SRC, "QualityGateArgs");
    assert_parity(&QualityProxyTool, QUALITY_PROXY_SRC, "QualityProxyInput");
    assert_parity(&PdmtTool::new(), PDMT_SRC, "PdmtInput");
    assert_parity(
        &ContextGenerateTool::new(),
        CONTEXT_SRC,
        "ContextGenerateArgs",
    );
    assert_parity(
        &ContextSummaryTool::new(),
        CONTEXT_SRC,
        "ContextSummaryArgs",
    );
    assert_parity(&GitStatusTool::new(), GIT_SRC, "GitStatusArgs");
    assert_parity(
        &TdgSystemDiagnosticsTool::new(),
        TDG_SRC,
        "TdgSystemDiagnosticsArgs",
    );
    assert_parity(
        &TdgStorageManagementTool::new(),
        TDG_SRC,
        "TdgStorageManagementArgs",
    );
    assert_parity(
        &TdgAnalyzeWithStorageTool::new(),
        TDG_SRC,
        "TdgAnalyzeWithStorageArgs",
    );
    assert_parity(
        &TdgPerformanceMetricsTool::new(),
        TDG_SRC,
        "TdgPerformanceMetricsArgs",
    );
    assert_parity(
        &TdgConfigureStorageTool::new(),
        TDG_SRC,
        "TdgConfigureStorageArgs",
    );
    assert_parity(&TdgHealthCheckTool::new(), TDG_SRC, "TdgHealthCheckArgs");
}

/// The scraper has to be able to FAIL, or the sweep above is decoration.
#[test]
fn the_field_scraper_reads_real_declarations() {
    let fields = args_fields(ANALYZE_DEBT_SRC, "SatdArgs");
    assert_eq!(
        fields.iter().map(String::as_str).collect::<Vec<_>>(),
        vec!["include_resolved", "include_tests", "paths"],
        "the scraper must see every field, attributes and doc comments skipped"
    );

    // A field the schema does not name is what the sweep is looking for: prove
    // the comparison rejects one rather than passing everything.
    let advertised = advertised(&crate::mcp_pmcp::analyze_handlers::SatdTool::new());
    assert!(
        !advertised.contains("no_such_parameter"),
        "sanity: the schema must not contain an invented property"
    );
    assert!(
        args_fields(
            "struct Fake {\n    paths: Vec<String>,\n    no_such_parameter: bool,\n}",
            "Fake"
        )
        .contains("no_such_parameter"),
        "the scraper must surface an undocumented field for the sweep to catch"
    );
}
