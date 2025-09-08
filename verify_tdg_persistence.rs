use pmat::tdg::{TdgAnalyzer, TdgConfig};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== TDG Persistence Verification ===\n");
    
    // Create analyzer with storage
    let analyzer = TdgAnalyzer::with_storage(TdgConfig::default())?;
    
    // Check storage statistics
    if let Some(stats) = analyzer.get_storage_stats() {
        println!("Storage Statistics:");
        println!("  Total entries: {}", stats.total_entries);
        println!("  Hot cache: {} entries", stats.hot_entries);
        println!("  Warm storage: {} entries", stats.warm_entries);
        println!("  Cold storage: {} entries", stats.cold_entries);
        println!("  Compression ratio: {:.1}%", stats.compression_ratio * 100.0);
        println!();
    }
    
    // Try to retrieve a previously analyzed file
    let test_path = Path::new("server/src/tdg/storage.rs");
    println!("Checking for cached score of: {}", test_path.display());
    
    if let Ok(Some(score)) = analyzer.get_stored_score(test_path).await {
        println!("✅ Found cached score!");
        println!("  Score: {:.1}/100", score.total);
        println!("  Grade: {:?}", score.grade);
        println!("\nTDG is successfully dogfooding with persistent storage!");
    } else {
        println!("❌ No cached score found");
        println!("Analyzing file now...");
        
        let score = analyzer.analyze_file(test_path).await?;
        println!("  Score: {:.1}/100", score.total);
        println!("  Grade: {:?}", score.grade);
        println!("\nScore has been stored for future retrieval.");
    }
    
    Ok(())
}