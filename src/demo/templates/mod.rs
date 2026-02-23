#![cfg_attr(coverage_nightly, coverage(off))]
//! Templates for the demo web interface.
//!
//! Contains HTML and CSS templates used by the demo server to render
//! interactive reports. These are validated by scripts/validate-demo-assets.ts

mod css;
mod html;
#[cfg(test)]
mod tests;

pub use css::CSS_DARK_THEME;
pub use html::HTML_TEMPLATE;
