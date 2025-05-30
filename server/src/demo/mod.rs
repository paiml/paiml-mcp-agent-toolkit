#[cfg(any(test, feature = "demo-dev"))]
pub mod runner;

#[cfg(any(test, feature = "demo-dev"))]
pub use runner::{detect_repository, DemoReport, DemoRunner, DemoStep};

use anyhow::Result;

#[cfg(not(any(test, feature = "demo-dev")))]
pub fn run_demo(_args: DemoArgs) -> Result<()> {
    anyhow::bail!("Demo mode not available in release builds")
}

#[cfg(any(test, feature = "demo-dev"))]
pub async fn run_demo(
    args: DemoArgs,
    server: std::sync::Arc<crate::stateless_server::StatelessTemplateServer>,
) -> Result<()> {
    use crate::cli::{ExecutionMode, OutputFormat};

    let repo = detect_repository(args.path)?;
    let mut runner = DemoRunner::new(server);
    let report = runner.execute(repo).await?;

    let output = match args.format {
        OutputFormat::Table | OutputFormat::Yaml => report.render(ExecutionMode::Cli),
        OutputFormat::Json => report.render(ExecutionMode::Mcp),
    };

    println!("{}", output);
    Ok(())
}

#[derive(Debug, Clone)]
pub struct DemoArgs {
    pub path: Option<std::path::PathBuf>,
    pub format: crate::cli::OutputFormat,
}
