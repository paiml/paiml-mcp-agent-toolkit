//! Five Whys Root Cause Analysis Example
//!
//! This example demonstrates how to use pmat's five-whys command for
//! Toyota Way root cause analysis with evidence-based debugging.
//!
//! Run with: `cargo run --example five_whys_demo`
//!
//! # Features Demonstrated
//!
//! 1. Five Whys methodology (Toyota Production System)
//! 2. Evidence-based hypothesis generation
//! 3. PMAT tool integration (complexity, SATD, TDG, churn)
//! 4. Confidence scoring for root causes
//! 5. Prioritized recommendations
//!
//! # CLI Usage
//!
//! ```bash
//! # Basic usage (5 iterations, text output)
//! pmat five-whys "Stack overflow in parser"
//!
//! # Short aliases
//! pmat why "Memory leak in cache"
//! pmat debug-whys "Test failures"
//!
//! # Custom depth (1-10 iterations)
//! pmat five-whys "Performance regression" --depth 3
//!
//! # Output formats
//! pmat five-whys "Bug in login" --format text     # Human-readable (default)
//! pmat five-whys "Bug in login" --format json     # For CI/CD
//! pmat five-whys "Bug in login" --format markdown # For documentation
//!
//! # Auto-analyze suspected files
//! pmat five-whys "Crash on startup" --auto-analyze
//!
//! # With deep context file
//! pmat five-whys "API timeout" --context deep_context.md
//!
//! # Save analysis
//! pmat five-whys "Error 500" --output analysis.md --format markdown
//! ```

fn main() {
    println!("PMAT Five Whys Root Cause Analysis Demo");
    println!("{}", "=".repeat(60));

    // Example 1: Toyota Way methodology
    println!("\nExample 1: Toyota Way Five Whys Methodology");
    println!("{}", "-".repeat(40));
    demonstrate_methodology();

    // Example 2: Evidence sources
    println!("\nExample 2: Evidence Sources");
    println!("{}", "-".repeat(40));
    demonstrate_evidence_sources();

    // Example 3: Confidence scoring
    println!("\nExample 3: Confidence Scoring");
    println!("{}", "-".repeat(40));
    demonstrate_confidence_scoring();

    // Example 4: Example analysis
    println!("\nExample 4: Example Analysis");
    println!("{}", "-".repeat(40));
    demonstrate_example_analysis();

    // Example 5: Integration with development workflow
    println!("\nExample 5: Development Workflow Integration");
    println!("{}", "-".repeat(40));
    demonstrate_workflow_integration();

    println!("\n{}", "=".repeat(60));
    println!("Five Whys demo completed!");
}

/// Demonstrate the Toyota Way Five Whys methodology
fn demonstrate_methodology() {
    println!(
        "
Toyota Production System - Five Whys:

The Five Whys is a iterative interrogative technique used to explore
the cause-and-effect relationships underlying a particular problem.

## Core Principles

1. Genchi Genbutsu (Go and See)
   - Gather evidence from the actual code/system
   - Don't guess, investigate

2. Jidoka (Automation with Human Touch)
   - PMAT tools automatically gather evidence
   - Human judgment interprets results

3. Kaizen (Continuous Improvement)
   - Learn from root causes
   - Prevent future occurrences

4. Nemawashi (Building Consensus)
   - Transparent reasoning chain
   - Shareable analysis reports

## Example Chain

  Problem: Application crashes on startup

  Why 1: Memory allocation fails
  Why 2: Heap size exceeded
  Why 3: Large cache initialized on startup
  Why 4: Cache pre-loads entire database
  Why 5: No lazy loading implemented
         ^ ROOT CAUSE

  Root Cause: Missing lazy loading for cache initialization
  Solution: Implement lazy loading for database cache
"
    );
}

/// Demonstrate evidence sources
fn demonstrate_evidence_sources() {
    println!(
        "
Five Whys Evidence Sources:

PMAT automatically gathers evidence from these tools:

## 1. Complexity Analysis (25% weight)
   - Cyclomatic complexity violations (threshold: 20)
   - Cognitive complexity
   - Function length
   - Nesting depth

## 2. TDG Scoring (25% weight)
   - Test coverage metrics
   - Test quality indicators
   - Missing test detection

## 3. SATD Detection (20% weight)
   - TODO markers
   - FIXME comments
   - HACK annotations
   - Technical debt indicators

## 4. Git Churn (20% weight)
   - Commit frequency
   - Lines changed over time
   - File instability metrics

## 5. Dead Code (10% weight)
   - Unused functions
   - Unreachable code
   - Orphaned modules

## Evidence Gathering Process

  1. Parse issue description for file/function names
  2. Run PMAT analysis tools on suspected files
  3. Correlate findings with hypothesis
  4. Calculate confidence scores
  5. Generate recommendations
"
    );
}

/// Demonstrate confidence scoring
fn demonstrate_confidence_scoring() {
    println!(
        "
Confidence Scoring (0.0 - 1.0):

Each hypothesis receives a confidence score based on evidence:

## Score Calculation

  confidence = (
      complexity_weight * complexity_score +
      tdg_weight * tdg_score +
      satd_weight * satd_score +
      churn_weight * churn_score +
      dead_code_weight * dead_code_score
  )

## Severity Multipliers

  - Critical findings: 1.5x
  - High findings: 1.2x
  - Medium findings: 1.0x
  - Low findings: 0.8x

## Interpretation

  0.8 - 1.0 : High confidence - Strong evidence supports this hypothesis
  0.5 - 0.8 : Medium confidence - Some evidence, needs verification
  0.2 - 0.5 : Low confidence - Weak evidence, consider alternatives
  0.0 - 0.2 : Very low - Unlikely to be the root cause

## Example

  Hypothesis: \"Complexity in payment module causes bugs\"

  Evidence:
    - Complexity score: 0.9 (function with CC=45)
    - TDG score: 0.7 (60% test coverage)
    - SATD score: 0.8 (3 TODO markers)
    - Churn score: 0.6 (modified 15 times)
    - Dead code: 0.2 (no dead code)

  Confidence = 0.25*0.9 + 0.25*0.7 + 0.20*0.8 + 0.20*0.6 + 0.10*0.2
             = 0.225 + 0.175 + 0.16 + 0.12 + 0.02
             = 0.70 (Medium-High confidence)
"
    );
}

/// Demonstrate an example analysis
fn demonstrate_example_analysis() {
    println!(
        "
Example Five Whys Analysis:

  $ pmat five-whys \"Stack overflow in parser module\"

  Five Whys Root Cause Analysis
  ================================================

  Issue: Stack overflow in parser module

  Why 1: Why does the parser cause stack overflow?
  -----------------------------------------------
  Hypothesis: Recursive parsing without depth limit
  Evidence:
    - parser.rs has cyclomatic complexity of 42
    - 3 FIXME comments about recursion
    - File changed 28 times in last month
  Confidence: 0.85 (High)

  Why 2: Why is there no depth limit?
  -----------------------------------------------
  Hypothesis: Original design didn't anticipate deep nesting
  Evidence:
    - No test cases for deeply nested input
    - TODO: \"add recursion guard\" in parser.rs:234
  Confidence: 0.72 (Medium-High)

  Why 3: Why wasn't deep nesting anticipated?
  -----------------------------------------------
  Hypothesis: Specification unclear about max depth
  Evidence:
    - spec.md doesn't mention depth limits
    - Similar parsers have 1000-node limit
  Confidence: 0.65 (Medium)

  Why 4: Why is specification unclear?
  -----------------------------------------------
  Hypothesis: Copied from prototype without review
  Evidence:
    - Git blame shows bulk import
    - No review comments on original PR
  Confidence: 0.55 (Medium)

  Why 5: ROOT CAUSE
  -----------------------------------------------
  Root Cause: Lack of specification review process
  Confidence: 0.85 (High)

  Recommendations:
  ================================================
  1. [IMMEDIATE] Add recursion depth guard in parser.rs
  2. [SHORT-TERM] Add test cases for edge nesting depths
  3. [LONG-TERM] Establish specification review process
  4. [LONG-TERM] Add complexity monitoring to CI/CD
"
    );
}

/// Demonstrate workflow integration
fn demonstrate_workflow_integration() {
    println!(
        "
Development Workflow Integration:

## Bug Triage Process

  1. Bug reported -> Create issue
  2. Run Five Whys analysis:
     $ pmat five-whys \"<bug description>\" --output issue-123-analysis.md
  3. Attach analysis to issue
  4. Review and validate root cause
  5. Fix based on recommendations

## CI/CD Integration

```yaml
# .github/workflows/bug-analysis.yml
name: Bug Analysis
on:
  issues:
    types: [labeled]

jobs:
  analyze:
    if: contains(github.event.issue.labels.*.name, 'bug')
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Analyze Bug
        run: |
          TITLE=\"${{ github.event.issue.title }}\"
          pmat five-whys \"$TITLE\" \\
            --format markdown \\
            --output analysis.md \\
            --auto-analyze

      - name: Comment on Issue
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const analysis = fs.readFileSync('analysis.md', 'utf8');
            github.rest.issues.createComment({{
              owner: context.repo.owner,
              repo: context.repo.repo,
              issue_number: context.issue.number,
              body: '## Automated Root Cause Analysis\\n\\n' + analysis
            }});
```

## Post-Mortem Template

Use Five Whys output for post-mortems:

  1. Incident summary (issue description)
  2. Root cause chain (5 whys)
  3. Evidence (PMAT findings)
  4. Remediation (recommendations)
  5. Prevention (process improvements)
"
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_example_runs() {
        super::main();
    }
}
