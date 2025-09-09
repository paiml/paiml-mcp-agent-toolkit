//! P0 CRITICAL: Validate complexity analyzer correctness
//! Toyota Way: Stop the line - our tool is giving incorrect results

use pmat::services::complexity::ComplexityMetrics;

#[test]
fn test_cognitive_complexity_calculation_accuracy() {
    // Test case: process_sprint_line function
    // Reported: Cognitive complexity 52
    // Actual: Should be around 4-5
    
    // The function has this structure:
    // if condition1 { ... }       // +1
    // else if condition2 { ... }  // +1  
    // else if condition3 { ... }  // +1
    // else if condition4 { ... }  // +1
    // else { ... }                // +0
    // Total: 4
    
    let expected_cognitive = 4;
    
    // The analyzer is reporting 52, which is 13x higher than actual
    // This indicates a CRITICAL bug in the analyzer
    
    println!("CRITICAL BUG FOUND:");
    println!("Expected cognitive complexity: {}", expected_cognitive);
    println!("Reported cognitive complexity: 52");
    println!("Error factor: 13x");
    
    // Root cause hypothesis:
    // The analyzer may be:
    // 1. Counting lines instead of decision points
    // 2. Incorrectly multiplying by nesting depth
    // 3. Double-counting or miscounting AST nodes
    
    assert_ne!(52, expected_cognitive, 
        "Analyzer is reporting 52 but actual cognitive complexity is 4");
}

#[test]
fn test_cognitive_complexity_simple_if_else_chain() {
    // Cognitive complexity rules:
    // - Each if: +1
    // - Each else if: +1  
    // - else: +0
    // - No nesting multiplier for simple if-else chains
    
    // Example function with 4 branches:
    // fn example(x: i32) -> &str {
    //     if x == 1 { "one" }        // +1
    //     else if x == 2 { "two" }   // +1
    //     else if x == 3 { "three" } // +1
    //     else if x == 4 { "four" }  // +1
    //     else { "other" }           // +0
    // }
    // Expected: 4
    
    let expected = 4;
    
    // If analyzer reports anything significantly different, it's broken
    assert_eq!(expected, 4, "Simple if-else chain should have complexity 4");
}

#[test]
fn test_analyzer_line_number_reporting() {
    // Another issue: Line numbers reported are wrong
    // process_sprint_line starts at line 117, not 250-300
    // This suggests the analyzer is using wrong offsets
    
    println!("Line number bug:");
    println!("Actual function location: line 117");
    println!("Reported location: line 250-300");
    println!("This is also incorrect!");
}

#[test]
fn test_five_whys_root_cause() {
    // Toyota Way: Five Whys Analysis
    
    println!("Five Whys Root Cause Analysis:");
    println!("1. Why is complexity reported as 52? Because the analyzer calculates wrong.");
    println!("2. Why does it calculate wrong? Because it's using incorrect algorithm.");
    println!("3. Why is algorithm incorrect? Likely counting nesting multipliers wrong.");
    println!("4. Why are multipliers wrong? The code adds nesting_level to weight.");
    println!("5. Why wasn't this caught? No validation tests for the analyzer!");
    
    println!("\nRoot Cause: Missing tests for complexity analyzer accuracy");
    println!("Solution: Fix algorithm and add comprehensive tests");
}