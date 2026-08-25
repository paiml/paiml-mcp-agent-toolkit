// CB-2104: `pmat comply numeric-claims` — numbers a repository writes down
// about itself, judged against each other.
//
// Included from comply_handlers/mod.rs, which owns the imports.
//
// This handler is deliberately thin. Everything that decides anything lives in
// src/services/numeric_claims/, where it can be driven from string literals;
// what is here is argument plumbing, the `.pmat.yaml` enable switch, and the
// one line that turns a report into a process exit code.

/// The id `.pmat.yaml` addresses this rule by.
///
/// Registered in `default_checks` at severity Warning — NOT Error. An advisory
/// check declared Error would join the roster CB-2100 verifies reachability
/// for and would fail a `pmat comply check`, which is precisely the promise
/// this rule was built not to break.
const NUMERIC_CLAIMS_RULE_ID: &str = "cb-2104";

/// Report numbers this repository contradicts itself about.
///
/// WARN only. Findings exit 0, whatever their count — the check is advisory and
/// never blocks. Exit 2 means UNMEASURABLE: the corpus could not be read, or
/// the check failed its own self-test against the committed fixture. That
/// distinction is the point of the design: "I analysed 12,693 numbers and found
/// nothing" must never be byte-identical to "`git ls-files` returned nothing
/// and I analysed nothing".
///
/// `--min-sites` below the default and `--include-generated` both weaken the
/// rule, and both make it print the measured precision cost. The warnings come
/// from the rule itself rather than from here, so a caller that forgets to
/// print them is the only way to lose them.
fn handle_numeric_claims(
    project_path: &Path,
    format: NumericClaimsFormat,
    min_sites: usize,
    include_generated: bool,
) -> Result<()> {
    use crate::models::comply_config::PmatYamlConfig;
    use crate::services::numeric_claims::{
        census,
        cohort::{CohortConfig, Guards},
        render,
    };

    if min_sites == 0 {
        anyhow::bail!(
            "--min-sites must be at least 1: a cohort spanning no files is not a replicated claim"
        );
    }

    let yaml = PmatYamlConfig::load(project_path).unwrap_or_default();
    if !yaml.comply.is_check_enabled(NUMERIC_CLAIMS_RULE_ID) {
        // A DISABLED document, not a zeroed census: `--format json` must stay
        // pure JSON on stdout, and "nothing was scanned" must never render as
        // "scanned, and clean".
        match format {
            NumericClaimsFormat::Json => {
                println!("{}", render::disabled_json(NUMERIC_CLAIMS_RULE_ID)?)
            }
            NumericClaimsFormat::Text => {
                print!("{}", render::disabled_text(NUMERIC_CLAIMS_RULE_ID))
            }
        }
        return Ok(());
    }

    let cfg = CohortConfig {
        min_sites,
        guards: Guards {
            generated: !include_generated,
            ..Guards::default()
        },
        ..CohortConfig::default()
    };

    let report = census::run(project_path, &cfg);
    match format {
        NumericClaimsFormat::Json => println!("{}", render::json(&report)?),
        NumericClaimsFormat::Text => print!("{}", render::text(&report)),
    }

    // The whole exit policy is one call on the report, so the CLI cannot
    // disagree with the tests about what a finding costs.
    let code = report.exit_code();
    if code != 0 {
        // `process::exit` runs no destructors, so the report is flushed by hand
        // rather than trusted to a line buffer.
        use std::io::Write;
        let _ = std::io::stdout().flush();
        std::process::exit(code);
    }
    Ok(())
}
