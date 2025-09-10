//! TDD Test Suite for Zero Warnings Sprint 87
//! 
//! This test validates that we achieve ABSOLUTE ZERO warnings.
//! NO EXCUSES. Every warning must be eliminated.

#[cfg(test)]
mod zero_warnings_tests {
    use std::process::Command;

    /// RED Test: Verify we can detect warnings
    #[test]
    fn test_warnings_detection_works() {
        let output = Command::new("cargo")
            .args(&["check", "--quiet"])
            .output()
            .expect("Failed to run cargo check");
        
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // Should detect warnings initially
        assert!(stderr.contains("warning:"), "Warning detection mechanism must work");
    }

    /// GREEN Test: Verify ZERO warnings after fixes
    #[test]
    fn test_absolute_zero_warnings() {
        let output = Command::new("cargo")
            .args(&["check", "--quiet"])
            .output()
            .expect("Failed to run cargo check");
        
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // Filter out build script warnings (pmat@2.77.0:)
        let actual_warnings: Vec<&str> = stderr
            .lines()
            .filter(|line| line.contains("warning:"))
            .filter(|line| !line.contains("pmat@2.77.0:"))
            .collect();
        
        // ABSOLUTE ZERO TOLERANCE
        assert_eq!(
            actual_warnings.len(), 
            0, 
            "ZERO warnings required. Found: {:#?}", 
            actual_warnings
        );
    }

    /// TDD Test Categories - Each must pass
    
    #[test]
    fn test_no_unused_variable_warnings() {
        let warnings = get_warnings();
        let unused_vars: Vec<_> = warnings
            .iter()
            .filter(|w| w.contains("unused variable"))
            .collect();
        
        assert_eq!(unused_vars.len(), 0, "No unused variables allowed: {:#?}", unused_vars);
    }

    #[test]
    fn test_no_dead_code_warnings() {
        let warnings = get_warnings();
        let dead_code: Vec<_> = warnings
            .iter()
            .filter(|w| w.contains("never used") || w.contains("never read") || w.contains("never constructed"))
            .collect();
        
        assert_eq!(dead_code.len(), 0, "No dead code allowed: {:#?}", dead_code);
    }

    #[test]
    fn test_no_private_interface_warnings() {
        let warnings = get_warnings();
        let private_interfaces: Vec<_> = warnings
            .iter()
            .filter(|w| w.contains("more private than"))
            .collect();
        
        assert_eq!(private_interfaces.len(), 0, "No private interface violations: {:#?}", private_interfaces);
    }

    #[test]
    fn test_no_lifetime_warnings() {
        let warnings = get_warnings();
        let lifetime_warnings: Vec<_> = warnings
            .iter()
            .filter(|w| w.contains("lifetime") || w.contains("elided"))
            .collect();
        
        assert_eq!(lifetime_warnings.len(), 0, "No lifetime warnings allowed: {:#?}", lifetime_warnings);
    }

    fn get_warnings() -> Vec<String> {
        let output = Command::new("cargo")
            .args(&["check", "--quiet"])
            .output()
            .expect("Failed to run cargo check");
        
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        stderr
            .lines()
            .filter(|line| line.contains("warning:"))
            .filter(|line| !line.contains("pmat@2.77.0:"))
            .map(|s| s.to_string())
            .collect()
    }
}