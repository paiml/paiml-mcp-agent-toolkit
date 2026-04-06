/// Write output to file or stdout
async fn write_context_output(output: Option<PathBuf>, content: &str) -> Result<()> {
    debug_assert!(!content.is_empty(), "content must not be empty");
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, content).await?;
        eprintln!("✅ Context written to: {}", output_path.display());
    } else {
        println!("{content}");
    }
    Ok(())
}
