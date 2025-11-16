//! BUG-012: Multi-Language CLI Support Example
//!
//! Demonstrates the language override functionality for `pmat context` command.
//!
//! This example shows how to:
//! 1. Override language detection with a single language
//! 2. Specify multiple languages for analysis
//! 3. Use case-insensitive language names
//! 4. Handle invalid language specifications
//!
//! # Usage
//!
//! ```bash
//! cargo run --example bug_012_multi_language_cli
//! ```
//!
//! # Related
//!
//! - Bug Report: bug-reports/012-missing-multi-language-support.md
//! - Implementation: server/src/services/language_override.rs
//! - Tests: server/tests/bug_012_multi_language_cli_tests.rs
//! - Integration: BUG-011 (Enhanced Language Detection)

use anyhow::Result;
use pmat::services::language_override::{
    get_effective_languages, has_override, normalize_language_name, validate_language_support,
    LanguageOverride,
};
use tempfile::TempDir;

fn main() -> Result<()> {
    println!("==================================================");
    println!("BUG-012: Multi-Language CLI Support Demonstration");
    println!("==================================================\n");

    // Example 1: Single Language Override
    println!("📝 Example 1: Single Language Override");
    println!("   Command: pmat context --language python\n");

    let project = create_demo_project()?;

    let override_opts = LanguageOverride {
        language: Some("python".to_string()),
        languages: None,
    };

    let langs = get_effective_languages(&override_opts, project.path())?;
    println!("   ✅ Result: Analyzing as '{}'", langs[0]);
    println!("   📊 Override active: {}\n", has_override(&override_opts));

    // Example 2: Multiple Languages Override
    println!("📝 Example 2: Multiple Languages Override");
    println!("   Command: pmat context --languages rust,python,typescript\n");

    let override_opts = LanguageOverride {
        language: None,
        languages: Some(vec![
            "rust".to_string(),
            "python".to_string(),
            "typescript".to_string(),
        ]),
    };

    let langs = get_effective_languages(&override_opts, project.path())?;
    println!("   ✅ Result: Analyzing {} languages", langs.len());
    for (i, lang) in langs.iter().enumerate() {
        println!("      {}. {}", i + 1, lang);
    }
    println!();

    // Example 3: Case-Insensitive Language Names
    println!("📝 Example 3: Case-Insensitive Language Names");
    println!("   Command: pmat context --language PYTHON\n");

    let normalized_python = normalize_language_name("PYTHON")?;
    let normalized_rust = normalize_language_name("Rust")?;
    let normalized_cpp = normalize_language_name("cpp")?;

    println!("   ✅ Normalization results:");
    println!("      'PYTHON'  → '{}'", normalized_python);
    println!("      'Rust'    → '{}'", normalized_rust);
    println!("      'cpp'     → '{}'", normalized_cpp);
    println!();

    // Example 4: Language Validation
    println!("📝 Example 4: Language Support Validation");
    println!("   Checking supported languages...\n");

    let test_languages = vec![
        ("rust", true),
        ("python", true),
        ("typescript", true),
        ("fortran", false),
        ("cobol", false),
    ];

    println!("   🔍 Validation results:");
    for (lang, should_pass) in test_languages {
        match validate_language_support(lang) {
            Ok(_) if should_pass => println!("      ✅ '{}' - SUPPORTED", lang),
            Ok(_) => println!("      ⚠️  '{}' - Unexpected support", lang),
            Err(_) if !should_pass => println!("      ❌ '{}' - Not supported (expected)", lang),
            Err(e) => println!("      ❌ '{}' - Error: {}", lang, e),
        }
    }
    println!();

    // Example 5: Auto-Detection Fallback
    println!("📝 Example 5: Auto-Detection Fallback (No Override)");
    println!("   Command: pmat context\n");

    let override_opts = LanguageOverride {
        language: None,
        languages: None,
    };

    let langs = get_effective_languages(&override_opts, project.path())?;
    println!(
        "   ✅ Result: Auto-detected as '{}' (from BUG-011)",
        langs[0]
    );
    println!("   📊 Override active: {}\n", has_override(&override_opts));

    // Example 6: Priority Demonstration
    println!("📝 Example 6: Override Priority");
    println!("   When both --language and --languages are specified:\n");

    let override_opts = LanguageOverride {
        language: Some("c".to_string()),           // Higher priority
        languages: Some(vec!["rust".to_string()]), // Ignored
    };

    let langs = get_effective_languages(&override_opts, project.path())?;
    println!("   🔝 Priority order:");
    println!("      1. --language (single)    → USED");
    println!("      2. --languages (multiple) → IGNORED");
    println!("      3. Auto-detection         → IGNORED");
    println!("   ✅ Result: '{}'", langs[0]);
    println!();

    // Summary
    println!("==================================================");
    println!("✅ BUG-012 Implementation Complete");
    println!("==================================================");
    println!();
    println!("📋 Features Demonstrated:");
    println!("   ✓ Single language override (--language)");
    println!("   ✓ Multiple language override (--languages)");
    println!("   ✓ Case-insensitive language names");
    println!("   ✓ Language validation with helpful errors");
    println!("   ✓ Auto-detection fallback (BUG-011)");
    println!("   ✓ Correct priority: single > multiple > auto");
    println!();
    println!("📖 Documentation:");
    println!("   - Bug Report: bug-reports/012-missing-multi-language-support.md");
    println!("   - Tests: server/tests/bug_012_multi_language_cli_tests.rs");
    println!("   - Module: server/src/services/language_override.rs");
    println!();
    println!("🔧 Usage:");
    println!("   pmat context --language rust");
    println!("   pmat context --languages rust,python,typescript");
    println!("   pmat context --language Python  # Case-insensitive");

    Ok(())
}

/// Create a demo polyglot project for testing
fn create_demo_project() -> Result<TempDir> {
    use std::fs;

    let temp = TempDir::new()?;
    let base = temp.path();

    // Create Rust files
    fs::create_dir_all(base.join("src"))?;
    fs::write(
        base.join("src/main.rs"),
        "fn main() { println!(\"Hello from Rust\"); }",
    )?;
    fs::write(base.join("Cargo.toml"), "[package]\nname = \"demo\"\n")?;

    // Create Python files
    fs::write(base.join("script.py"), "print('Hello from Python')")?;
    fs::write(base.join("pyproject.toml"), "[project]\nname = \"demo\"\n")?;

    // Create TypeScript files
    fs::write(base.join("app.ts"), "console.log('Hello from TypeScript');")?;
    fs::write(base.join("package.json"), "{\"name\": \"demo\"}")?;

    Ok(temp)
}
