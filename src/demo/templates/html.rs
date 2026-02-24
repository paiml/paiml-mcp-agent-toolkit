#![cfg_attr(coverage_nightly, coverage(off))]
/// HTML templates for the demo web interface
/// These are validated by scripts/validate-demo-assets.ts
///
/// Split into logical sections:
/// - html_styles.rs: DOCTYPE, head, and CSS styles (477 lines)
/// - html_body.rs: HTML body markup and layout (276 lines)
/// - html_scripts.rs: JavaScript logic and closing tags (431 lines)
pub const HTML_TEMPLATE: &str = concat!(
    include_str!("html_styles.rs"),
    include_str!("html_body.rs"),
    include_str!("html_scripts.rs"),
);
