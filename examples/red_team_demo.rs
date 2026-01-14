// Demo of Red Team Mode handler
//
// Run with: cargo run --example red_team_demo

use pmat::cli::handlers::red_team::RedTeamHandler;
use pmat::red_team::RepositoryContext;

fn main() {
    println!("{}", "=".repeat(60));
    println!("Red Team Mode: Hallucination Detection Demo");
    println!("{}", "=".repeat(60));
    println!();

    let handler = RedTeamHandler::new();

    // Demo 1: Commit with hallucination
    println!("DEMO 1: Commit message with hallucination");
    println!("{}", "-".repeat(60));

    let commit_message = "feat: All tests passing";
    let context = RepositoryContext::new_mock().with_test_results(true, 5);

    let result = handler.analyze_commit_message(commit_message, &context);
    println!("{}", result.format_text());

    println!();
    println!("{}", "=".repeat(60));
    println!();

    // Demo 2: Clean commit (no hallucination)
    println!("DEMO 2: Clean commit message (no hallucination)");
    println!("{}", "-".repeat(60));

    let commit_message = "refactor: Improve code style";
    let context = RepositoryContext::new_mock();

    let result = handler.analyze_commit_message(commit_message, &context);
    println!("{}", result.format_text());

    println!();
    println!("{}", "=".repeat(60));
    println!();

    // Demo 3: Multiple claims with mixed evidence
    println!("DEMO 3: Migration claim with contradicting evidence");
    println!("{}", "-".repeat(60));

    let commit_message = "feat: Complete migration to libsql";
    let context = RepositoryContext::new_mock().with_code_grep_results("sled", 15);

    let result = handler.analyze_commit_message(commit_message, &context);
    println!("{}", result.format_text());

    println!();
    println!("{}", "=".repeat(60));
    println!();

    // Demo 4: JSON output
    println!("DEMO 4: JSON output format");
    println!("{}", "-".repeat(60));

    let commit_message = "test: Coverage at 85%";
    let context = RepositoryContext::new_mock().with_coverage(65.0);

    let result = handler.analyze_commit_message(commit_message, &context);
    println!(
        "{}",
        serde_json::to_string_pretty(&result.format_json()).unwrap()
    );
}
