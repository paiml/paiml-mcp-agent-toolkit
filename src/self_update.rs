//! Sovereign self-update (aprender EPIC #4232, pmat#1425).
//!
//! `pmat update [--check]` and the daily startup check come from the shared
//! `aprender-update` crate; this module only declares the product.

/// Release asset template; must match `binary-release.yml`
/// (`pmat-${TAG}-${TARGET}.tar.gz` plus a `.sha256` sibling).
pub const ASSET_TEMPLATE: &str = "{bin}-{tag}-{target}.tar.gz";

/// The product descriptor handed to the shared updater.
pub const PRODUCT: sovereign_update::Product = sovereign_update::Product {
    bin: "pmat",
    repo: "paiml/paiml-mcp-agent-toolkit",
    version: env!("CARGO_PKG_VERSION"),
    build_sha: None,
    release_asset: Some(ASSET_TEMPLATE),
    nightly: false,
};

/// True when argv asks for the `update` subcommand.
#[must_use]
pub fn is_update_request(args: &[String]) -> bool {
    args.get(1).is_some_and(|a| a == "update")
}

/// Handle `pmat update ...` (exits) and run the daily startup check otherwise.
pub fn run(args: &[String]) {
    if is_update_request(args) {
        std::process::exit(sovereign_update::update_main(&PRODUCT, &args[2..]));
    }
    sovereign_update::startup(&PRODUCT, args);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn product_names_pmat_and_its_repo() {
        assert_eq!(PRODUCT.bin, "pmat");
        assert_eq!(PRODUCT.repo, "paiml/paiml-mcp-agent-toolkit");
        assert_eq!(PRODUCT.version, env!("CARGO_PKG_VERSION"));
        assert!(!PRODUCT.nightly);
    }

    #[test]
    fn asset_template_matches_binary_release_workflow() {
        let wf = include_str!("../.github/workflows/binary-release.yml");
        assert!(wf.contains(r#"ARCHIVE="pmat-${VERSION}-${TARGET}""#));
        assert_eq!(PRODUCT.release_asset, Some("{bin}-{tag}-{target}.tar.gz"));
    }

    #[test]
    fn update_request_detection_is_exact() {
        assert!(is_update_request(&argv(&["pmat", "update"])));
        assert!(is_update_request(&argv(&["pmat", "update", "--check"])));
        assert!(!is_update_request(&argv(&["pmat"])));
        assert!(!is_update_request(&argv(&["pmat", "analyze", "update"])));
        assert!(!is_update_request(&argv(&["pmat", "updates"])));
    }
}
