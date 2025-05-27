use installer_macro::shell_installer;
use std::fmt;

pub struct ShellContext;

#[derive(Debug)]
pub enum Error {
    UnsupportedPlatform(String),
    DownloadFailed(String),
    ChecksumMismatch { expected: String, actual: String },
    InstallFailed(String),
    CommandFailed(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnsupportedPlatform(platform) => {
                write!(f, "Unsupported platform: {}", platform)
            }
            Error::DownloadFailed(msg) => write!(f, "Download failed: {}", msg),
            Error::ChecksumMismatch { expected, actual } => {
                write!(
                    f,
                    "Checksum mismatch: expected {}, got {}",
                    expected, actual
                )
            }
            Error::InstallFailed(msg) => write!(f, "Installation failed: {}", msg),
            Error::CommandFailed(msg) => write!(f, "Command failed: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl ShellContext {
    #[inline(always)]
    pub fn command(&self, cmd: &'static str, args: &[&str]) -> Result<String, Error> {
        // This is analyzed at compile time and transformed to shell
        match std::process::Command::new(cmd).args(args).output() {
            Ok(output) => Ok(String::from_utf8_lossy(&output.stdout).into_owned()),
            Err(e) => Err(Error::CommandFailed(format!("{}: {}", cmd, e))),
        }
    }

    #[inline(always)]
    pub fn test_dir(&self, path: &str) -> bool {
        std::path::Path::new(path).is_dir()
    }
}

#[shell_installer]
pub fn install_paiml_mcp_agent_toolkit(ctx: &ShellContext, args: &[String]) -> Result<(), Error> {
    // Parse arguments
    let install_dir = args
        .first()
        .map(String::as_str)
        .unwrap_or("${HOME}/.local/bin");
    let version = args
        .get(1)
        .map(String::as_str)
        .unwrap_or(env!("CARGO_PKG_VERSION"));

    // Platform detection
    let os = ctx.command("uname", &["-s"])?;
    let arch = ctx.command("uname", &["-m"])?;

    let platform = match (os.trim(), arch.trim()) {
        ("Linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("Linux", "aarch64") => "aarch64-unknown-linux-gnu",
        ("Darwin", "x86_64") => "x86_64-apple-darwin",
        ("Darwin", "aarch64" | "arm64") => "aarch64-apple-darwin",
        (os, arch) => return Err(Error::UnsupportedPlatform(format!("{}-{}", os, arch))),
    };

    // Construct URLs (readonly for security)
    ctx.command("readonly", &["BASE_URL=https://github.com/paiml/paiml-mcp-agent-toolkit/releases/download"])?;
    let base_url = "https://github.com/paiml/paiml-mcp-agent-toolkit/releases/download";
    let binary_url = format!(
        "{}/v{}/paiml-mcp-agent-toolkit-{}.tar.gz",
        base_url, version, platform
    );
    let checksum_url = format!("{}.sha256", binary_url);

    // Create temporary directory
    let temp_dir = ctx.command("mktemp", &["-d"])?;
    let temp_dir = temp_dir.trim();

    // Set cleanup trap
    ctx.command("trap", &[&format!("rm -rf {}", temp_dir), "EXIT"])?;

    // Download binary
    ctx.command(
        "curl",
        &[
            "-sSfL",
            "--max-time",
            "300",
            "--retry",
            "3",
            "-o",
            &format!("{}/archive.tar.gz", temp_dir),
            &binary_url,
        ],
    )?;

    // Download and verify checksum
    let expected_checksum = ctx.command("curl", &["-sSfL", &checksum_url])?;
    let expected_checksum =
        expected_checksum
            .split_whitespace()
            .next()
            .ok_or_else(|| Error::ChecksumMismatch {
                expected: "none".into(),
                actual: "parse_failed".into(),
            })?;

    let actual_checksum = ctx.command("sha256sum", &[&format!("{}/archive.tar.gz", temp_dir)])?;
    let actual_checksum =
        actual_checksum
            .split_whitespace()
            .next()
            .ok_or_else(|| Error::ChecksumMismatch {
                expected: expected_checksum.into(),
                actual: "compute_failed".into(),
            })?;

    if expected_checksum != actual_checksum {
        return Err(Error::ChecksumMismatch {
            expected: expected_checksum.into(),
            actual: actual_checksum.into(),
        });
    }

    // Extract archive
    ctx.command(
        "tar",
        &[
            "-xzf",
            &format!("{}/archive.tar.gz", temp_dir),
            "-C",
            temp_dir,
        ],
    )?;

    // Create install directory if needed
    if !ctx.test_dir(install_dir) {
        ctx.command("mkdir", &["-p", install_dir])?;
    }

    // Atomic installation using rename
    ctx.command(
        "mv",
        &[
            "-f",
            &format!("{}/paiml-mcp-agent-toolkit", temp_dir),
            &format!("{}/paiml-mcp-agent-toolkit", install_dir),
        ],
    )?;

    // Set executable permissions
    ctx.command(
        "chmod",
        &["755", &format!("{}/paiml-mcp-agent-toolkit", install_dir)],
    )?;

    // Cleanup is handled by trap
    Ok(())
}
