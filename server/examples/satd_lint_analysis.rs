//! Example demonstrating SATD and lint-hotspot analysis via different protocols
//! 
//! This example shows how to use the newly added SATD and lint-hotspot
//! functionality through CLI, HTTP, and MCP protocols.

use anyhow::Result;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 SATD and Lint Hotspot Analysis Example\n");
    
    let project_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| ".".to_string());
    
    // Example 1: Using CLI directly (for demonstration, we'll show the commands)
    println!("1️⃣ CLI Usage Examples:");
    println!("   # Analyze SATD (Self-Admitted Technical Debt)");
    println!("   pmat analyze satd --project-path {}", project_path);
    println!("   pmat analyze satd --strict --critical-only");
    println!();
    println!("   # Find lint hotspots");
    println!("   pmat analyze lint-hotspot --project-path {}", project_path);
    println!("   pmat analyze lint-hotspot --top-files 10 --min-violations 5");
    println!();
    
    // Example 2: HTTP API usage
    println!("2️⃣ HTTP API Usage:");
    println!("   POST http://localhost:8080/api/v1/analyze/satd");
    println!("   Body: {}", json!({
        "project_path": project_path,
        "strict": false,
        "exclude_tests": true,
        "critical_only": false,
        "format": "json"
    }).to_string());
    println!();
    println!("   POST http://localhost:8080/api/v1/analyze/lint-hotspot");
    println!("   Body: {}", json!({
        "project_path": project_path,
        "top_files": 10,
        "min_violations": 1,
        "format": "json"
    }).to_string());
    println!();
    
    // Example 3: MCP tool usage
    println!("3️⃣ MCP Tool Usage (for AI agents):");
    println!("   Tool: analyze_satd");
    println!("   Arguments: {}", json!({
        "project_path": project_path,
        "strict": false,
        "exclude_tests": true,
        "critical_only": false,
        "format": "summary"
    }).to_string());
    println!();
    println!("   Tool: analyze_lint_hotspot");
    println!("   Arguments: {}", json!({
        "project_path": project_path,
        "top_files": 10,
        "min_violations": 1,
        "include": "**/*.rs",
        "exclude": "**/tests/**"
    }).to_string());
    println!();
    
    // Example 4: Practical integration example
    println!("4️⃣ Practical Integration Example:");
    println!("   Here's how you might use these in a CI/CD pipeline:\n");
    
    println!("   ```yaml");
    println!("   # .github/workflows/quality.yml");
    println!("   - name: Check for technical debt");
    println!("     run: |");
    println!("       pmat analyze satd --critical-only --format json > satd.json");
    println!("       if [ $(jq '.total_debt_items' satd.json) -gt 0 ]; then");
    println!("         echo '::warning::Critical technical debt found!'");
    println!("         jq '.items[] | \"\\(.file):\\(.line) - \\(.text)\"' satd.json");
    println!("       fi");
    println!();
    println!("   - name: Find lint hotspots");
    println!("     run: |");
    println!("       pmat analyze lint-hotspot --top-files 5 --format json > hotspots.json");
    println!("       echo '::notice::Top files with lint issues:'");
    println!("       jq '.hotspots[] | \"\\(.file_path): \\(.violations) violations\"' hotspots.json");
    println!("   ```");
    println!();
    
    // Example 5: Quality gate integration
    println!("5️⃣ Quality Gate Integration:");
    println!("   You can now include SATD and lint metrics in quality gates:");
    println!();
    println!("   pmat quality-gate \\");
    println!("     --checks complexity,satd,lint,dead-code \\");
    println!("     --max-satd-items 10 \\");
    println!("     --max-lint-density 5.0 \\");
    println!("     --fail-on-violation");
    println!();
    
    println!("✨ These new features help maintain code quality by:");
    println!("   • Tracking technical debt through code comments");
    println!("   • Identifying files that accumulate the most lint violations");
    println!("   • Providing consistent analysis across CLI, HTTP, and MCP interfaces");
    println!("   • Enabling automated quality checks in CI/CD pipelines");
    
    Ok(())
}