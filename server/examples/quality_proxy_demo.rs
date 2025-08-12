//! Example demonstrating the Quality Proxy service for enforcing code quality.
//!
//! This example shows how to use the quality proxy to intercept and validate
//! code changes before they are applied, ensuring all code meets quality standards.
//!
//! Run with:
//! ```bash
//! cargo run --example quality_proxy_demo
//! ```

use pmat::models::proxy::{ProxyMode, ProxyOperation, ProxyRequest, ProxyStatus, QualityConfig};
use pmat::services::quality_proxy::QualityProxyService;
use colored::Colorize;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{}", "Quality Proxy Demo".bold().blue());
    println!("{}", "==================".blue());
    println!();

    let service = QualityProxyService::new();

    // Demo 1: High-quality code in strict mode
    demo_high_quality_code(&service).await?;
    
    // Demo 2: Code with SATD in strict mode
    demo_satd_rejection(&service).await?;
    
    // Demo 3: Advisory mode with warnings
    demo_advisory_mode(&service).await?;
    
    // Demo 4: Auto-fix mode
    demo_auto_fix(&service).await?;
    
    // Demo 5: Complex code rejection
    demo_complexity_check(&service).await?;

    Ok(())
}

async fn demo_high_quality_code(service: &QualityProxyService) -> anyhow::Result<()> {
    println!("{}", "Demo 1: High-Quality Code (Strict Mode)".green().bold());
    println!("{}", "---------------------------------------".green());
    
    let code = r#"/// Calculate the factorial of a number
/// 
/// # Arguments
/// 
/// * `n` - The number to calculate factorial for
/// 
/// # Returns
/// 
/// The factorial of n
pub fn factorial(n: u32) -> u32 {
    match n {
        0 | 1 => 1,
        _ => n * factorial(n - 1),
    }
}"#;

    println!("Input code:");
    println!("{}", code.dimmed());
    println!();

    let request = ProxyRequest {
        operation: ProxyOperation::Write,
        file_path: "factorial.rs".to_string(),
        content: Some(code.to_string()),
        old_content: None,
        new_content: None,
        mode: ProxyMode::Strict,
        quality_config: QualityConfig::default(),
    };

    let response = service.proxy_operation(request).await?;
    
    print_response(&response);
    println!();
    
    Ok(())
}

async fn demo_satd_rejection(service: &QualityProxyService) -> anyhow::Result<()> {
    println!("{}", "Demo 2: SATD Rejection (Strict Mode)".yellow().bold());
    println!("{}", "------------------------------------".yellow());
    
    let code = r#"fn process_payment(amount: f64) -> Result<(), String> {
    // TODO: implement actual payment processing
    // FIXME: this is just a stub for now
    println!("Processing payment of ${}", amount);
    
    // HACK: always return success for testing
    Ok(())
}"#;

    println!("Input code:");
    println!("{}", code.dimmed());
    println!();

    let request = ProxyRequest {
        operation: ProxyOperation::Write,
        file_path: "payment.rs".to_string(),
        content: Some(code.to_string()),
        old_content: None,
        new_content: None,
        mode: ProxyMode::Strict,
        quality_config: QualityConfig::default(),
    };

    let response = service.proxy_operation(request).await?;
    
    print_response(&response);
    println!();
    
    Ok(())
}

async fn demo_advisory_mode(service: &QualityProxyService) -> anyhow::Result<()> {
    println!("{}", "Demo 3: Advisory Mode (Warnings Only)".cyan().bold());
    println!("{}", "-------------------------------------".cyan());
    
    let code = r#"pub fn calculate_discount(price: f64, percentage: f64) -> f64 {
    price * (1.0 - percentage / 100.0)
}"#;

    println!("Input code (missing documentation):");
    println!("{}", code.dimmed());
    println!();

    let request = ProxyRequest {
        operation: ProxyOperation::Write,
        file_path: "discount.rs".to_string(),
        content: Some(code.to_string()),
        old_content: None,
        new_content: None,
        mode: ProxyMode::Advisory,
        quality_config: QualityConfig::default(),
    };

    let response = service.proxy_operation(request).await?;
    
    print_response(&response);
    println!();
    
    Ok(())
}

async fn demo_auto_fix(service: &QualityProxyService) -> anyhow::Result<()> {
    println!("{}", "Demo 4: Auto-Fix Mode".magenta().bold());
    println!("{}", "--------------------".magenta());
    
    let code = r#"pub struct UserData {
    name: String,
    email: String,
}

fn validate_email(email: &str) -> bool {
    // TODO: add proper email validation
    email.contains("@")
}"#;

    println!("Input code (with SATD and missing docs):");
    println!("{}", code.dimmed());
    println!();

    let request = ProxyRequest {
        operation: ProxyOperation::Write,
        file_path: "user.rs".to_string(),
        content: Some(code.to_string()),
        old_content: None,
        new_content: None,
        mode: ProxyMode::AutoFix,
        quality_config: QualityConfig {
            allow_satd: false,
            require_docs: true,
            auto_format: true,
            ..Default::default()
        },
    };

    let response = service.proxy_operation(request).await?;
    
    print_response(&response);
    
    if response.refactoring_applied {
        println!();
        println!("{}", "Fixed code:".green());
        println!("{}", response.final_content.dimmed());
    }
    println!();
    
    Ok(())
}

async fn demo_complexity_check(service: &QualityProxyService) -> anyhow::Result<()> {
    println!("{}", "Demo 5: Complexity Check".red().bold());
    println!("{}", "------------------------".red());
    
    let code = r#"fn complex_calculation(a: i32, b: i32, c: i32, d: i32) -> i32 {
    let mut result = 0;
    
    if a > 0 {
        if b > 0 {
            if c > 0 {
                if d > 0 {
                    for i in 0..a {
                        if i % 2 == 0 {
                            for j in 0..b {
                                if j % 3 == 0 {
                                    result += i * j;
                                    if result > 100 {
                                        return result;
                                    }
                                }
                            }
                        }
                    }
                } else {
                    result = d * 2;
                }
            } else {
                result = c * 3;
            }
        } else {
            result = b * 4;
        }
    } else {
        result = a * 5;
    }
    
    result
}"#;

    println!("Input code (high complexity):");
    println!("{}", code.dimmed());
    println!();

    let request = ProxyRequest {
        operation: ProxyOperation::Write,
        file_path: "complex.rs".to_string(),
        content: Some(code.to_string()),
        old_content: None,
        new_content: None,
        mode: ProxyMode::Strict,
        quality_config: QualityConfig {
            max_complexity: 15,
            ..Default::default()
        },
    };

    let response = service.proxy_operation(request).await?;
    
    print_response(&response);
    println!();
    
    Ok(())
}

fn print_response(response: &pmat::models::proxy::ProxyResponse) {
    // Print status
    let status_str = match response.status {
        ProxyStatus::Accepted => "ACCEPTED".green(),
        ProxyStatus::Rejected => "REJECTED".red(),
        ProxyStatus::Modified => "MODIFIED".yellow(),
    };
    println!("Status: {}", status_str);
    
    // Print quality report
    println!("Quality Report:");
    println!("  Passed: {}", 
        if response.quality_report.passed { 
            "Yes".green() 
        } else { 
            "No".red() 
        }
    );
    
    // Print metrics
    println!("  Metrics:");
    println!("    Max Complexity: {}", response.quality_report.metrics.max_complexity);
    println!("    SATD Count: {}", response.quality_report.metrics.satd_count);
    println!("    Lint Violations: {}", response.quality_report.metrics.lint_violations);
    
    // Print violations
    if !response.quality_report.violations.is_empty() {
        println!("  Violations:");
        for violation in &response.quality_report.violations {
            let severity = match violation.severity {
                pmat::models::proxy::ViolationSeverity::Error => "ERROR".red(),
                pmat::models::proxy::ViolationSeverity::Warning => "WARNING".yellow(),
            };
            
            println!("    [{:>7}] {} at {}", 
                severity, 
                violation.message, 
                violation.location
            );
            
            if let Some(suggestion) = &violation.suggestion {
                println!("              Suggestion: {}", suggestion.dimmed());
            }
        }
    }
    
    // Print refactoring info
    if response.refactoring_applied {
        println!("  {}", "Automatic refactoring applied".green());
        if let Some(plan) = &response.refactoring_plan {
            println!("  Refactoring steps: {}", plan.len());
        }
    }
}