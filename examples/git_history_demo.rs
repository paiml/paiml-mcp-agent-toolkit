//! Demo: Git History RAG in action
//! Run with: cargo run --example git_history_demo

use pmat::services::git_history::{
    ChangeType, CommitInfo, DocumentMetadata, FileChange, GitHistoryIndex, GitHistorySearchEngine,
    GitSearchOptions, RankedDocument, RrfFusion,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Git History RAG Demo ===\n");

    // 1. Create an in-memory index
    let mut index = GitHistoryIndex::in_memory()?;
    println!("✓ Created GitHistoryIndex");

    // 2. Simulate some commits (in production, these come from git2)
    let commits = vec![
        CommitInfo {
            hash: "a".repeat(40),
            message_subject: "Fix null pointer exception in parser".to_string(),
            message_body: Some("Handle edge case when input buffer is empty".to_string()),
            author_name: "Alice".to_string(),
            author_email: "alice@example.com".to_string(),
            timestamp: 1700000000,
            is_merge: false,
            is_fix: true,
            is_feat: false,
            issue_refs: vec!["#123".to_string()],
            files: vec![FileChange {
                path: "src/parser.rs".to_string(),
                change_type: ChangeType::Modified,
                lines_added: 15,
                lines_deleted: 3,
            }],
        },
        CommitInfo {
            hash: "b".repeat(40),
            message_subject: "Add dark mode support to UI".to_string(),
            message_body: Some("Users can toggle dark mode in settings".to_string()),
            author_name: "Bob".to_string(),
            author_email: "bob@example.com".to_string(),
            timestamp: 1700100000,
            is_merge: false,
            is_fix: false,
            is_feat: true,
            issue_refs: vec!["#456".to_string()],
            files: vec![FileChange {
                path: "src/ui/theme.rs".to_string(),
                change_type: ChangeType::Modified,
                lines_added: 100,
                lines_deleted: 10,
            }],
        },
        CommitInfo {
            hash: "c".repeat(40),
            message_subject: "Fix memory leak in cache module".to_string(),
            message_body: Some("Clear expired entries periodically".to_string()),
            author_name: "Alice".to_string(),
            author_email: "alice@example.com".to_string(),
            timestamp: 1700200000,
            is_merge: false,
            is_fix: true,
            is_feat: false,
            issue_refs: vec![],
            files: vec![FileChange {
                path: "src/cache.rs".to_string(),
                change_type: ChangeType::Modified,
                lines_added: 25,
                lines_deleted: 5,
            }],
        },
        CommitInfo {
            hash: "d".repeat(40),
            message_subject: "Refactor error handling for better diagnostics".to_string(),
            message_body: None,
            author_name: "Charlie".to_string(),
            author_email: "charlie@example.com".to_string(),
            timestamp: 1700300000,
            is_merge: false,
            is_fix: false,
            is_feat: false,
            issue_refs: vec![],
            files: vec![FileChange {
                path: "src/error.rs".to_string(),
                change_type: ChangeType::Added,
                lines_added: 150,
                lines_deleted: 0,
            }],
        },
    ];

    // 3. Insert commits
    let count = index.insert_commits(&commits)?;
    println!("✓ Indexed {} commits", count);

    // 4. Create search engine
    let mut engine = GitHistorySearchEngine::new(&index);
    println!("✓ Created search engine\n");

    // 5. Search for "fix bug error"
    println!("--- Search: \"fix bug error\" ---");
    let results = engine.search("fix bug error", GitSearchOptions::default())?;
    for (i, r) in results.iter().take(3).enumerate() {
        println!(
            "  {}. {} (score: {:.3})",
            i + 1,
            r.commit.message_subject,
            r.relevance_score
        );
        println!(
            "     Author: {}, Files: {:?}",
            r.commit.author_name, r.files
        );
    }

    // 6. Search with filter: only fixes
    println!("\n--- Search: \"memory\" (only fixes) ---");
    let options = GitSearchOptions {
        only_fixes: true,
        ..Default::default()
    };
    let results = engine.search("memory", options)?;
    for r in &results {
        println!(
            "  • {} (is_fix: {})",
            r.commit.message_subject, r.commit.is_fix
        );
    }

    // 7. Demonstrate RRF Fusion
    println!("\n--- RRF Fusion Demo ---");
    let fusion = RrfFusion::new();

    // Simulate code search results
    let code_results = vec![
        RankedDocument {
            id: "src/error.rs:handle_error".to_string(),
            original_score: 0.9,
            source: "code".to_string(),
            metadata: DocumentMetadata::default(),
        },
        RankedDocument {
            id: "src/parser.rs:parse".to_string(),
            original_score: 0.7,
            source: "code".to_string(),
            metadata: DocumentMetadata::default(),
        },
    ];

    // Git results (from our search above)
    let git_results: Vec<RankedDocument> = results
        .iter()
        .map(|r| RankedDocument {
            id: format!(
                "{}:{}",
                r.commit.hash[..7].to_string(),
                r.commit.message_subject
            ),
            original_score: r.relevance_score,
            source: "git".to_string(),
            metadata: DocumentMetadata {
                path: r.files.first().cloned().unwrap_or_default(),
                name: r.commit.message_subject.clone(),
                line_or_timestamp: r.commit.timestamp,
                related_commits: vec![r.commit.hash.clone()],
            },
        })
        .collect();

    let fused = fusion.fuse(vec![("code", code_results), ("git", git_results)], 5);

    println!("  Fused results (code + git):");
    for (i, r) in fused.iter().enumerate() {
        println!(
            "    {}. {} (RRF: {:.4}, source: {})",
            i + 1,
            r.id.chars().take(50).collect::<String>(),
            r.rrf_score,
            r.primary_source
        );
    }

    println!("\n✓ Git History RAG demo complete!");
    Ok(())
}
