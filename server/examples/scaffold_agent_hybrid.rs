//! Hybrid agent scaffolding example - Deterministic core with AI wrapper
//!
//! This advanced example demonstrates the hybrid architecture pattern where
//! a deterministic core provides guaranteed correctness while an AI wrapper
//! adds flexibility and natural language processing capabilities.

#![allow(unused_variables)]

use anyhow::Result;
use pmat::scaffold::agent::{
    AgentContextBuilder, CoreSpec, FallbackStrategy, ModelType, QualityLevel, VerificationMethod,
    WrapperSpec,
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║      Hybrid Agent Architecture Demonstration         ║");
    println!("║   Deterministic Core + Probabilistic AI Wrapper      ║");
    println!("╚══════════════════════════════════════════════════════╝");
    println!();

    // Example 1: Code Analysis Hybrid Agent
    println!("📊 Example 1: Hybrid Code Analyzer");
    println!("─────────────────────────────────");
    create_hybrid_code_analyzer().await?;

    // Example 2: Verification-First Agent
    println!("\n🔐 Example 2: Verification-First Agent");
    println!("──────────────────────────────────────");
    create_verification_agent().await?;

    // Example 3: Fallback Safety Agent
    println!("\n🛡️  Example 3: Fallback Safety Agent");
    println!("────────────────────────────────────");
    create_fallback_agent().await?;

    println!("\n═══════════════════════════════════════════════════════");
    println!("Key Concepts Demonstrated:");
    println!("  1. Deterministic core ensures correctness");
    println!("  2. AI wrapper provides flexibility");
    println!("  3. Verification methods guarantee properties");
    println!("  4. Fallback strategies handle AI failures");
    println!("  5. Confidence thresholds control AI usage");
    println!("═══════════════════════════════════════════════════════");

    Ok(())
}

/// Hybrid Code Analyzer with deterministic AST analysis + AI insights
async fn create_hybrid_code_analyzer() -> Result<()> {
    // Configure the deterministic core
    // Note: In a real implementation, invariants would be configured
    // For this example, we'll describe them conceptually
    let core_spec = CoreSpec {
        verification_method: VerificationMethod::PropertyTests,
        max_complexity: 10,
        invariants: Vec::new(), // Would contain actual invariant implementations
    };

    let invariant_descriptions = vec![
        "AST parsing must be total (never fail)",
        "Complexity calculation must be deterministic",
        "SATD detection must have zero false negatives",
    ];

    // Configure the AI wrapper
    let wrapper_spec = WrapperSpec {
        model_type: ModelType::Claude,
        fallback_strategy: FallbackStrategy::Deterministic,
        confidence_threshold: 0.95,
    };

    let _context = AgentContextBuilder::new("hybrid_code_analyzer", "hybrid")
        .with_deterministic_core(core_spec.clone())
        .with_probabilistic_wrapper(wrapper_spec.clone())
        .with_quality_level(QualityLevel::Extreme)
        .build()?;

    println!("Configuration:");
    println!("  Deterministic Core:");
    println!("    • AST-based analysis (guaranteed correct)");
    println!("    • McCabe complexity (deterministic)");
    println!("    • Pattern matching for SATD");
    println!("    • Verification: Property-based testing");
    println!("    • Max complexity: {}", core_spec.max_complexity);

    println!("\n  AI Wrapper (Claude):");
    println!("    • Natural language explanations");
    println!("    • Code improvement suggestions");
    println!("    • Semantic understanding");
    println!(
        "    • Confidence threshold: {}",
        wrapper_spec.confidence_threshold
    );
    println!("    • Fallback: Use deterministic core only");

    println!("\n  Invariants Enforced:");
    for description in &invariant_descriptions {
        println!("    ✓ {}", description);
    }

    println!("\n  Example Flow:");
    println!("    1. User: 'Analyze this code for issues'");
    println!("    2. Core: Parse AST, calculate metrics (100% reliable)");
    println!("    3. AI: Generate insights if confidence > 0.95");
    println!("    4. Fallback: Return core metrics if AI fails");

    Ok(())
}

/// Verification-First Agent with formal proofs
async fn create_verification_agent() -> Result<()> {
    let core_spec = CoreSpec {
        verification_method: VerificationMethod::FormalProof,
        max_complexity: 5,      // Very low for provability
        invariants: Vec::new(), // Would contain formal invariant specifications
    };

    let wrapper_spec = WrapperSpec {
        model_type: ModelType::GPT4,
        fallback_strategy: FallbackStrategy::Error, // Fail fast if AI unavailable
        confidence_threshold: 0.98,                 // Very high threshold
    };

    println!("Configuration:");
    println!("  Verification Method: Formal Proof");
    println!("  Why Formal Verification?");
    println!("    • Mathematical guarantees of correctness");
    println!("    • Suitable for critical systems");
    println!("    • Eliminates entire classes of bugs");

    println!("\n  Core Properties:");
    println!(
        "    • Max complexity: {} (for tractable proofs)",
        core_spec.max_complexity
    );
    println!("    • State space: Finite and bounded");
    println!("    • Transitions: Proven correct");

    println!("\n  AI Enhancement:");
    println!("    • Model: GPT-4 for advanced reasoning");
    println!("    • Purpose: Explain proofs in natural language");
    println!(
        "    • Threshold: {} (very conservative)",
        wrapper_spec.confidence_threshold
    );

    println!("\n  Course Exercise Ideas:");
    println!("    1. Prove termination of a sorting algorithm");
    println!("    2. Verify invariants of a state machine");
    println!("    3. Model check concurrent operations");

    Ok(())
}

/// Fallback Safety Agent demonstrating graceful degradation
async fn create_fallback_agent() -> Result<()> {
    let core_spec = CoreSpec {
        verification_method: VerificationMethod::ModelChecking,
        max_complexity: 8,
        invariants: Vec::new(), // Would contain temporal logic formulas
    };

    // Local model with default value fallback
    let wrapper_spec = WrapperSpec {
        model_type: ModelType::Local("./models/llama2".to_string()),
        fallback_strategy: FallbackStrategy::DefaultValue,
        confidence_threshold: 0.90,
    };

    println!("Configuration:");
    println!("  Fallback Strategy Pattern:");
    println!("    Level 1: Try local AI model");
    println!("    Level 2: Use deterministic core");
    println!("    Level 3: Return safe default value");

    println!("\n  Model Checking Verification:");
    println!("    • Exhaustive state exploration");
    println!("    • Temporal logic properties");
    println!("    • Counterexample generation");

    println!("\n  Safety Guarantees:");
    println!("    ✓ Never crashes or panics");
    println!("    ✓ Always produces valid output");
    println!("    ✓ Graceful degradation under load");
    println!("    ✓ Audit trail for debugging");

    println!("\n  Real-World Scenarios:");
    println!("    • Network failures → Use cached AI responses");
    println!("    • Model unavailable → Pure deterministic mode");
    println!("    • Low confidence → Return conservative defaults");
    println!("    • High latency → Quick deterministic response");

    println!("\n  Implementation Pattern:");
    println!("    ```rust");
    println!("    match ai_wrapper.process(input).await {{");
    println!("        Ok(result) if result.confidence > 0.90 => result.value,");
    println!("        Ok(_) => deterministic_core.process(input),");
    println!("        Err(_) => default_safe_value(),");
    println!("    }}");
    println!("    ```");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pmat::scaffold::agent::hybrid::{Invariant, InvariantSeverity};

    #[test]
    fn test_hybrid_examples() {
        // Test that examples demonstrate key concepts
        let core = CoreSpec {
            verification_method: VerificationMethod::PropertyTests,
            max_complexity: 10,
            invariants: vec![Invariant {
                name: "test".to_string(),
                description: "Test invariant".to_string(),
                severity: InvariantSeverity::Error,
            }],
        };

        assert_eq!(core.max_complexity, 10);
        assert!(matches!(
            core.verification_method,
            VerificationMethod::PropertyTests
        ));
    }
}
