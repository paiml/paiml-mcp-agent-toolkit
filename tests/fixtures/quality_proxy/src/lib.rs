//! Warning-free, dependency-free Rust for the quality proxy's lint stage.
//!
//! The proxy copies this crate's manifest and lock file into a private temp
//! directory and overwrites this file with the content under review, so what
//! matters here is that the crate is trivially lintable: no dependencies, one
//! documented public item, nothing that makes clippy think.

/// Greet `name`.
///
/// Documented on purpose — the proxy's documentation gate flags an
/// undocumented `pub fn`, and this fixture must be a clean baseline.
#[must_use]
pub fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}
