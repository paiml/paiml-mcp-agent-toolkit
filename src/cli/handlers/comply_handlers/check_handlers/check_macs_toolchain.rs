// Included from check_macs.rs — do NOT add `use` imports or `#!` attributes here.

/// CB-1667: a Rust project pins its toolchain (KZ-001 a).
///
/// An unpinned `stable` picks up each new rustc the day it ships, so `main` can
/// go red with no code change. A Rust project (has a `Cargo.toml`) must carry a
/// `rust-toolchain.toml` whose `channel` is an exact version (`1.98.1`) or a
/// dated nightly (`nightly-2026-10-01`). `stable`, `beta` and bare `nightly`
/// float and fail. This reads the file only; it never runs rustup.
pub(crate) fn check_toolchain_pinned(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1667: Pinned Rust Toolchain";
    let mk = |status, message: String| ComplianceCheck {
        name: name.to_string(),
        status,
        message,
        severity: Severity::Error,
    };
    if !project_path.join("Cargo.toml").is_file() {
        return skip_check(name, "no Cargo.toml — not a Rust project, 0 toolchains judged");
    }
    let path = project_path.join("rust-toolchain.toml");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return mk(
            CheckStatus::Fail,
            "no rust-toolchain.toml — pin an exact channel (e.g. channel = \"1.98.1\")".to_string(),
        );
    };
    let Some(channel) = toolchain_channel(&text) else {
        return mk(
            CheckStatus::Fail,
            "rust-toolchain.toml names no `channel` under [toolchain]".to_string(),
        );
    };
    if toolchain_channel_is_pinned(&channel) {
        mk(CheckStatus::Pass, format!("toolchain pinned to {channel}"))
    } else {
        mk(
            CheckStatus::Fail,
            format!("channel \"{channel}\" floats — pin an exact version or a dated nightly"),
        )
    }
}

/// The `channel` value of the `[toolchain]` table, if any (line scan: the file
/// is one flat table, and the optional `toml` crate is not always compiled in).
fn toolchain_channel(text: &str) -> Option<String> {
    let mut in_toolchain = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_toolchain = line == "[toolchain]";
        } else if in_toolchain {
            if let Some(rest) = line.strip_prefix("channel") {
                let v = rest.trim_start().strip_prefix('=')?.trim();
                let v = v.split('#').next()?.trim().trim_matches(|c| c == '"' || c == '\'');
                return (!v.is_empty()).then(|| v.to_string());
            }
        }
    }
    None
}

/// `1.98.1` / `1.98` style versions, or `nightly-YYYY-MM-DD` / `beta-YYYY-MM-DD`.
fn toolchain_channel_is_pinned(channel: &str) -> bool {
    let is_version = !channel.is_empty()
        && channel.split('.').count() >= 2
        && channel.split('.').all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()));
    let dated = ["nightly-", "beta-"]
        .iter()
        .find_map(|p| channel.strip_prefix(p))
        .is_some_and(|d| {
            let parts: Vec<&str> = d.split('-').collect();
            parts.len() == 3
                && parts[0].len() == 4
                && parts[1].len() == 2
                && parts[2].len() == 2
                && parts.iter().all(|p| p.bytes().all(|b| b.is_ascii_digit()))
        });
    is_version || dated
}
