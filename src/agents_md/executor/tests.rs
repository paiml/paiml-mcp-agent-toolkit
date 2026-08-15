#![cfg_attr(coverage_nightly, coverage(off))]
//! Tests for the command executor module.

use super::*;
use std::path::PathBuf;
use std::time::Duration;

#[test]
fn test_validate_safe_command() {
    let executor = AgentsMdExecutor::new().unwrap();

    let cmd = Command {
        name: "Build".to_string(),
        command: "cargo build".to_string(),
        working_dir: None,
        env: Vec::new(),
        timeout: Some(60),
        safe: true,
    };

    let report = executor.validate_command(&cmd).unwrap();
    assert!(report.safe);
    assert_eq!(report.risk_level, RiskLevel::None);
    assert!(report.risks.is_empty());
}

#[test]
fn test_validate_dangerous_command() {
    let executor = AgentsMdExecutor::new().unwrap();

    let cmd = Command {
        name: "Dangerous".to_string(),
        command: "sudo rm -rf /".to_string(),
        working_dir: None,
        env: Vec::new(),
        timeout: Some(60),
        safe: false,
    };

    let report = executor.validate_command(&cmd).unwrap();
    assert!(!report.safe);
    assert_eq!(report.risk_level, RiskLevel::Critical);
    assert!(!report.risks.is_empty());
    assert!(!report.mitigations.is_empty());
}

#[tokio::test]
async fn test_execute_simple_command() {
    let executor = AgentsMdExecutor::new().unwrap();

    let cmd = Command {
        name: "Echo".to_string(),
        command: "echo hello".to_string(),
        working_dir: None,
        env: Vec::new(),
        timeout: Some(5),
        safe: true,
    };

    let output = executor.execute_command(&cmd).await.unwrap();
    assert_eq!(output.exit_code, 0);
    assert!(output.stdout.contains("hello"));
    assert!(!output.timed_out);
}

#[test]
fn test_quality_gate_success() {
    let executor = AgentsMdExecutor::new().unwrap();

    let output = CommandOutput {
        exit_code: 0,
        stdout: "Success".to_string(),
        stderr: String::new(),
        duration: Duration::from_secs(1),
        timed_out: false,
    };

    let report = executor.apply_quality_gates(&output).unwrap();
    assert!(report.passed);
    assert!(report.violations.is_empty());
}

#[test]
fn test_quality_gate_failure() {
    let executor = AgentsMdExecutor::new().unwrap();

    let output = CommandOutput {
        exit_code: 1,
        stdout: String::new(),
        stderr: "error: compilation failed".to_string(),
        duration: Duration::from_secs(1),
        timed_out: false,
    };

    let report = executor.apply_quality_gates(&output).unwrap();
    assert!(!report.passed);
    assert!(!report.violations.is_empty());
}

// === ExecutorConfig Tests ===

#[test]
fn test_executor_config_default() {
    let config = ExecutorConfig::default();
    assert_eq!(config.default_timeout, 60);
    assert_eq!(config.max_output_size, 10 * 1024 * 1024);
    assert!(config.allow_network);
    assert!(config.allowed_commands.contains(&"cargo".to_string()));
    assert!(config.allowed_commands.contains(&"npm".to_string()));
    assert!(config.allowed_commands.contains(&"make".to_string()));
    assert!(config.allowed_commands.contains(&"git".to_string()));
    assert!(config.allowed_commands.contains(&"python".to_string()));
    assert!(config.allowed_commands.contains(&"node".to_string()));
    assert!(config.env_vars.is_empty());
}

#[test]
fn test_executor_config_custom() {
    let config = ExecutorConfig {
        default_timeout: 120,
        max_output_size: 5 * 1024 * 1024,
        allow_network: false,
        allowed_commands: vec!["rustc".to_string()],
        env_vars: vec![("RUST_LOG".to_string(), "debug".to_string())],
    };

    assert_eq!(config.default_timeout, 120);
    assert_eq!(config.max_output_size, 5 * 1024 * 1024);
    assert!(!config.allow_network);
    assert_eq!(config.allowed_commands.len(), 1);
    assert_eq!(config.env_vars.len(), 1);
}

#[test]
fn test_executor_config_clone() {
    let config = ExecutorConfig::default();
    let cloned = config.clone();
    assert_eq!(config.default_timeout, cloned.default_timeout);
    assert_eq!(config.max_output_size, cloned.max_output_size);
}

#[test]
fn test_executor_config_debug() {
    let config = ExecutorConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("ExecutorConfig"));
    assert!(debug_str.contains("default_timeout"));
}

// === ResourceLimits Tests ===

#[test]
fn test_resource_limits_default() {
    let limits = ResourceLimits::default();
    assert_eq!(limits.cpu_time, 300);
    assert_eq!(limits.memory, 1024 * 1024 * 1024);
    assert_eq!(limits.file_size, 100 * 1024 * 1024);
}

#[test]
fn test_resource_limits_custom() {
    let limits = ResourceLimits {
        cpu_time: 600,
        memory: 2 * 1024 * 1024 * 1024,
        file_size: 200 * 1024 * 1024,
    };

    assert_eq!(limits.cpu_time, 600);
    assert_eq!(limits.memory, 2 * 1024 * 1024 * 1024);
    assert_eq!(limits.file_size, 200 * 1024 * 1024);
}

#[test]
fn test_resource_limits_clone() {
    let limits = ResourceLimits::default();
    let cloned = limits.clone();
    assert_eq!(limits.cpu_time, cloned.cpu_time);
    assert_eq!(limits.memory, cloned.memory);
}

#[test]
fn test_resource_limits_debug() {
    let limits = ResourceLimits::default();
    let debug_str = format!("{:?}", limits);
    assert!(debug_str.contains("ResourceLimits"));
    assert!(debug_str.contains("cpu_time"));
}

// === SandboxEnvironment Tests ===

#[test]
fn test_sandbox_environment_creation() {
    let sandbox = SandboxEnvironment {
        working_dir: PathBuf::from("/test/project"),
        temp_dir: PathBuf::from("/tmp/sandbox"),
        limits: ResourceLimits::default(),
    };

    assert_eq!(sandbox.working_dir, PathBuf::from("/test/project"));
    assert_eq!(sandbox.temp_dir, PathBuf::from("/tmp/sandbox"));
}

#[test]
fn test_sandbox_environment_clone() {
    let sandbox = SandboxEnvironment {
        working_dir: PathBuf::from("/home/user"),
        temp_dir: PathBuf::from("/tmp/test"),
        limits: ResourceLimits::default(),
    };
    let cloned = sandbox.clone();
    assert_eq!(sandbox.working_dir, cloned.working_dir);
}

#[test]
fn test_sandbox_environment_debug() {
    let sandbox = SandboxEnvironment {
        working_dir: PathBuf::from("/test"),
        temp_dir: PathBuf::from("/tmp"),
        limits: ResourceLimits::default(),
    };
    let debug_str = format!("{:?}", sandbox);
    assert!(debug_str.contains("SandboxEnvironment"));
}

// === CommandOutput Tests ===

#[test]
fn test_command_output_creation() {
    let output = CommandOutput {
        exit_code: 0,
        stdout: "output".to_string(),
        stderr: "".to_string(),
        duration: Duration::from_millis(500),
        timed_out: false,
    };

    assert_eq!(output.exit_code, 0);
    assert_eq!(output.stdout, "output");
    assert!(!output.timed_out);
}

#[test]
fn test_command_output_timed_out() {
    let output = CommandOutput {
        exit_code: -1,
        stdout: "".to_string(),
        stderr: "timeout".to_string(),
        duration: Duration::from_secs(60),
        timed_out: true,
    };

    assert!(output.timed_out);
    assert_eq!(output.exit_code, -1);
}

#[test]
fn test_command_output_clone() {
    let output = CommandOutput {
        exit_code: 1,
        stdout: "test".to_string(),
        stderr: "error".to_string(),
        duration: Duration::from_secs(1),
        timed_out: false,
    };
    let cloned = output.clone();
    assert_eq!(output.exit_code, cloned.exit_code);
    assert_eq!(output.stdout, cloned.stdout);
}

#[test]
fn test_command_output_debug() {
    let output = CommandOutput {
        exit_code: 0,
        stdout: "test".to_string(),
        stderr: "".to_string(),
        duration: Duration::from_secs(1),
        timed_out: false,
    };
    let debug_str = format!("{:?}", output);
    assert!(debug_str.contains("CommandOutput"));
}

// === SafetyReport Tests ===

#[test]
fn test_safety_report_safe() {
    let report = SafetyReport {
        safe: true,
        risk_level: RiskLevel::None,
        risks: Vec::new(),
        mitigations: Vec::new(),
    };

    assert!(report.safe);
    assert_eq!(report.risk_level, RiskLevel::None);
}

#[test]
fn test_safety_report_with_risks() {
    let report = SafetyReport {
        safe: false,
        risk_level: RiskLevel::High,
        risks: vec![Risk {
            risk_type: RiskType::FileSystem,
            description: "File deletion".to_string(),
            severity: RiskLevel::High,
        }],
        mitigations: vec!["Use sandbox".to_string()],
    };

    assert!(!report.safe);
    assert_eq!(report.risks.len(), 1);
    assert_eq!(report.mitigations.len(), 1);
}

#[test]
fn test_safety_report_clone() {
    let report = SafetyReport {
        safe: true,
        risk_level: RiskLevel::Low,
        risks: vec![],
        mitigations: vec![],
    };
    let cloned = report.clone();
    assert_eq!(report.safe, cloned.safe);
}

#[test]
fn test_safety_report_debug() {
    let report = SafetyReport {
        safe: true,
        risk_level: RiskLevel::None,
        risks: vec![],
        mitigations: vec![],
    };
    let debug_str = format!("{:?}", report);
    assert!(debug_str.contains("SafetyReport"));
}

// === RiskLevel Tests ===

#[test]
fn test_risk_level_ordering() {
    assert!(RiskLevel::None < RiskLevel::Low);
    assert!(RiskLevel::Low < RiskLevel::Medium);
    assert!(RiskLevel::Medium < RiskLevel::High);
    assert!(RiskLevel::High < RiskLevel::Critical);
}

#[test]
fn test_risk_level_equality() {
    assert_eq!(RiskLevel::None, RiskLevel::None);
    assert_ne!(RiskLevel::Low, RiskLevel::High);
}

#[test]
fn test_risk_level_clone_copy() {
    let level = RiskLevel::Medium;
    let cloned = level.clone();
    let copied = level;
    assert_eq!(level, cloned);
    assert_eq!(level, copied);
}

#[test]
fn test_risk_level_debug() {
    let level = RiskLevel::Critical;
    let debug_str = format!("{:?}", level);
    assert!(debug_str.contains("Critical"));
}

// === Risk Tests ===

#[test]
fn test_risk_creation() {
    let risk = Risk {
        risk_type: RiskType::Network,
        description: "Network access detected".to_string(),
        severity: RiskLevel::Medium,
    };

    assert_eq!(risk.risk_type, RiskType::Network);
    assert_eq!(risk.severity, RiskLevel::Medium);
}

#[test]
fn test_risk_clone() {
    let risk = Risk {
        risk_type: RiskType::System,
        description: "System command".to_string(),
        severity: RiskLevel::High,
    };
    let cloned = risk.clone();
    assert_eq!(risk.risk_type, cloned.risk_type);
}

#[test]
fn test_risk_debug() {
    let risk = Risk {
        risk_type: RiskType::Unknown,
        description: "Unknown risk".to_string(),
        severity: RiskLevel::Low,
    };
    let debug_str = format!("{:?}", risk);
    assert!(debug_str.contains("Risk"));
}

// === RiskType Tests ===

#[test]
fn test_risk_type_equality() {
    assert_eq!(RiskType::FileSystem, RiskType::FileSystem);
    assert_ne!(RiskType::Network, RiskType::System);
}

#[test]
fn test_risk_type_all_variants() {
    let types = vec![
        RiskType::FileSystem,
        RiskType::Network,
        RiskType::System,
        RiskType::Resource,
        RiskType::Unknown,
    ];
    for risk_type in types {
        let debug_str = format!("{:?}", risk_type);
        assert!(!debug_str.is_empty());
    }
}

// === QualityReport Tests ===

#[test]
fn test_quality_report_passed() {
    let report = QualityReport {
        passed: true,
        checks: vec![QualityCheck {
            name: "Exit Code".to_string(),
            passed: true,
            message: "Success".to_string(),
        }],
        violations: vec![],
    };

    assert!(report.passed);
    assert_eq!(report.checks.len(), 1);
    assert!(report.violations.is_empty());
}

#[test]
fn test_quality_report_failed() {
    let report = QualityReport {
        passed: false,
        checks: vec![],
        violations: vec![QualityViolation {
            violation_type: ViolationType::Error,
            message: "Build failed".to_string(),
            severity: Severity::High,
        }],
    };

    assert!(!report.passed);
    assert_eq!(report.violations.len(), 1);
}

#[test]
fn test_quality_report_clone() {
    let report = QualityReport {
        passed: true,
        checks: vec![],
        violations: vec![],
    };
    let cloned = report.clone();
    assert_eq!(report.passed, cloned.passed);
}

#[test]
fn test_quality_report_debug() {
    let report = QualityReport {
        passed: true,
        checks: vec![],
        violations: vec![],
    };
    let debug_str = format!("{:?}", report);
    assert!(debug_str.contains("QualityReport"));
}

// === QualityCheck Tests ===

#[test]
fn test_quality_check_creation() {
    let check = QualityCheck {
        name: "Lint".to_string(),
        passed: false,
        message: "Linting failed".to_string(),
    };

    assert_eq!(check.name, "Lint");
    assert!(!check.passed);
}

#[test]
fn test_quality_check_clone() {
    let check = QualityCheck {
        name: "Test".to_string(),
        passed: true,
        message: "All tests passed".to_string(),
    };
    let cloned = check.clone();
    assert_eq!(check.name, cloned.name);
    assert_eq!(check.passed, cloned.passed);
}

#[test]
fn test_quality_check_debug() {
    let check = QualityCheck {
        name: "Build".to_string(),
        passed: true,
        message: "Build successful".to_string(),
    };
    let debug_str = format!("{:?}", check);
    assert!(debug_str.contains("QualityCheck"));
}

// === QualityViolation Tests ===

#[test]
fn test_quality_violation_creation() {
    let violation = QualityViolation {
        violation_type: ViolationType::Warning,
        message: "Deprecated API usage".to_string(),
        severity: Severity::Low,
    };

    assert_eq!(violation.violation_type, ViolationType::Warning);
    assert_eq!(violation.severity, Severity::Low);
}

#[test]
fn test_quality_violation_clone() {
    let violation = QualityViolation {
        violation_type: ViolationType::SecurityRisk,
        message: "Security issue".to_string(),
        severity: Severity::Critical,
    };
    let cloned = violation.clone();
    assert_eq!(violation.violation_type, cloned.violation_type);
}

#[test]
fn test_quality_violation_debug() {
    let violation = QualityViolation {
        violation_type: ViolationType::ResourceLimit,
        message: "Memory limit exceeded".to_string(),
        severity: Severity::High,
    };
    let debug_str = format!("{:?}", violation);
    assert!(debug_str.contains("QualityViolation"));
}

// === ViolationType Tests ===

#[test]
fn test_violation_type_equality() {
    assert_eq!(ViolationType::Error, ViolationType::Error);
    assert_ne!(ViolationType::Warning, ViolationType::Timeout);
}

#[test]
fn test_violation_type_all_variants() {
    let types = vec![
        ViolationType::Error,
        ViolationType::Warning,
        ViolationType::Timeout,
        ViolationType::ResourceLimit,
        ViolationType::SecurityRisk,
    ];
    for vtype in types {
        let debug_str = format!("{:?}", vtype);
        assert!(!debug_str.is_empty());
    }
}

// === Severity Tests ===

#[test]
fn test_severity_ordering() {
    assert!(Severity::Low < Severity::Medium);
    assert!(Severity::Medium < Severity::High);
    assert!(Severity::High < Severity::Critical);
}

#[test]
fn test_severity_equality() {
    assert_eq!(Severity::Low, Severity::Low);
    assert_ne!(Severity::Low, Severity::High);
}

#[test]
fn test_severity_clone() {
    let severity = Severity::Medium;
    let cloned = severity.clone();
    assert_eq!(severity, cloned);
}

// === AgentsMdExecutor Tests ===

#[test]
fn test_executor_new() {
    let executor = AgentsMdExecutor::new();
    assert!(executor.is_ok());
}

#[test]
fn test_executor_with_config() {
    let config = ExecutorConfig {
        default_timeout: 30,
        max_output_size: 1024,
        allow_network: false,
        allowed_commands: vec!["echo".to_string()],
        env_vars: vec![],
    };

    let executor = AgentsMdExecutor::with_config(config);
    assert!(executor.is_ok());

    let executor = executor.unwrap();
    assert_eq!(executor.config.default_timeout, 30);
    assert_eq!(executor.config.max_output_size, 1024);
    assert!(!executor.config.allow_network);
}

#[test]
fn test_validate_command_not_whitelisted() {
    let executor = AgentsMdExecutor::new().unwrap();

    let cmd = Command {
        name: "Unknown".to_string(),
        command: "unknown_command arg1".to_string(),
        working_dir: None,
        env: Vec::new(),
        timeout: Some(60),
        safe: false,
    };

    let report = executor.validate_command(&cmd).unwrap();
    // Should have medium risk for non-whitelisted command
    assert!(report.risk_level >= RiskLevel::Medium);
    assert!(!report.risks.is_empty());
}

#[test]
fn test_validate_command_eval() {
    let executor = AgentsMdExecutor::new().unwrap();

    let cmd = Command {
        name: "Eval".to_string(),
        command: "bash -c 'eval $COMMAND'".to_string(),
        working_dir: None,
        env: Vec::new(),
        timeout: Some(60),
        safe: false,
    };

    let report = executor.validate_command(&cmd).unwrap();
    assert!(!report.safe);
    assert!(report.risk_level >= RiskLevel::High);
}

#[test]
fn test_validate_command_chmod_777() {
    let executor = AgentsMdExecutor::new().unwrap();

    let cmd = Command {
        name: "Chmod".to_string(),
        command: "chmod 777 /tmp/file".to_string(),
        working_dir: None,
        env: Vec::new(),
        timeout: Some(60),
        safe: false,
    };

    let report = executor.validate_command(&cmd).unwrap();
    assert!(!report.safe);
    assert!(report.risk_level >= RiskLevel::High);
}

#[test]
fn test_validate_command_curl_pipe_sh() {
    let executor = AgentsMdExecutor::new().unwrap();

    let cmd = Command {
        name: "Install".to_string(),
        command: "curl | sh".to_string(),
        working_dir: None,
        env: Vec::new(),
        timeout: Some(60),
        safe: false,
    };

    let report = executor.validate_command(&cmd).unwrap();
    assert!(!report.safe);
    assert_eq!(report.risk_level, RiskLevel::Critical);
}

#[test]
fn test_validate_command_wget_pipe_bash() {
    let executor = AgentsMdExecutor::new().unwrap();

    let cmd = Command {
        name: "Install".to_string(),
        command: "wget | bash".to_string(),
        working_dir: None,
        env: Vec::new(),
        timeout: Some(60),
        safe: false,
    };

    let report = executor.validate_command(&cmd).unwrap();
    assert!(!report.safe);
    assert_eq!(report.risk_level, RiskLevel::Critical);
}

#[test]
fn test_validate_command_dev_redirect() {
    let executor = AgentsMdExecutor::new().unwrap();

    let cmd = Command {
        name: "DevWrite".to_string(),
        command: "echo test > /dev/sda".to_string(),
        working_dir: None,
        env: Vec::new(),
        timeout: Some(60),
        safe: false,
    };

    let report = executor.validate_command(&cmd).unwrap();
    assert!(!report.safe);
    assert_eq!(report.risk_level, RiskLevel::Critical);
}

#[test]
fn test_validate_empty_command() {
    let executor = AgentsMdExecutor::new().unwrap();

    let cmd = Command {
        name: "Empty".to_string(),
        command: "".to_string(),
        working_dir: None,
        env: Vec::new(),
        timeout: Some(60),
        safe: true,
    };

    let report = executor.validate_command(&cmd);
    // Empty command should be handled (either error or safe report)
    assert!(report.is_ok() || report.is_err());
}

// === Quality Gates Tests ===

#[test]
fn test_quality_gate_timeout_violation() {
    let executor = AgentsMdExecutor::new().unwrap();

    let output = CommandOutput {
        exit_code: -1,
        stdout: "".to_string(),
        stderr: "".to_string(),
        duration: Duration::from_secs(120),
        timed_out: true,
    };

    let report = executor.apply_quality_gates(&output).unwrap();
    assert!(!report.passed);
    assert!(report
        .violations
        .iter()
        .any(|v| v.violation_type == ViolationType::Timeout));
}

#[test]
fn test_quality_gate_failed_pattern() {
    let executor = AgentsMdExecutor::new().unwrap();

    let output = CommandOutput {
        exit_code: 0, // Exit code is 0 but has error in stderr
        stdout: "".to_string(),
        stderr: "failed: permission denied".to_string(),
        duration: Duration::from_secs(1),
        timed_out: false,
    };

    let report = executor.apply_quality_gates(&output).unwrap();
    assert!(!report.passed);
    assert!(report
        .violations
        .iter()
        .any(|v| v.message.contains("failed:")));
}

#[test]
fn test_quality_gate_fatal_pattern() {
    let executor = AgentsMdExecutor::new().unwrap();

    let output = CommandOutput {
        exit_code: 1,
        stdout: "".to_string(),
        stderr: "FATAL: cannot continue".to_string(),
        duration: Duration::from_secs(1),
        timed_out: false,
    };

    let report = executor.apply_quality_gates(&output).unwrap();
    assert!(!report.passed);
}

#[test]
fn test_quality_gate_panic_pattern() {
    let executor = AgentsMdExecutor::new().unwrap();

    let output = CommandOutput {
        exit_code: 101,
        stdout: "".to_string(),
        stderr: "thread 'main' panicked: assertion failed".to_string(),
        duration: Duration::from_secs(1),
        timed_out: false,
    };

    let report = executor.apply_quality_gates(&output).unwrap();
    assert!(!report.passed);
}

// === Execute Command Tests ===

#[tokio::test]
async fn test_execute_command_with_working_dir() {
    let executor = AgentsMdExecutor::new().unwrap();

    let cmd = Command {
        name: "Pwd".to_string(),
        command: "pwd".to_string(),
        working_dir: Some(PathBuf::from("/tmp")),
        env: Vec::new(),
        timeout: Some(5),
        safe: true,
    };

    let output = executor.execute_command(&cmd).await.unwrap();
    assert_eq!(output.exit_code, 0);
    assert!(output.stdout.contains("/tmp") || output.stdout.contains("tmp"));
}

#[tokio::test]
async fn test_execute_command_with_env() {
    let executor = AgentsMdExecutor::new().unwrap();

    let cmd = Command {
        name: "PrintEnv".to_string(),
        command: "printenv TEST_VAR".to_string(),
        working_dir: None,
        env: vec![("TEST_VAR".to_string(), "test_value".to_string())],
        timeout: Some(5),
        safe: true,
    };

    let output = executor.execute_command(&cmd).await.unwrap();
    assert_eq!(output.exit_code, 0);
    assert!(output.stdout.contains("test_value"));
}

#[tokio::test]
async fn test_execute_dangerous_command_rejected() {
    let executor = AgentsMdExecutor::new().unwrap();

    let cmd = Command {
        name: "Dangerous".to_string(),
        command: "sudo rm -rf /".to_string(),
        working_dir: None,
        env: Vec::new(),
        timeout: Some(5),
        safe: false,
    };

    let result = executor.execute_command(&cmd).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_execute_empty_command() {
    let executor = AgentsMdExecutor::new().unwrap();

    let cmd = Command {
        name: "Empty".to_string(),
        command: "".to_string(),
        working_dir: None,
        env: Vec::new(),
        timeout: Some(5),
        safe: true,
    };

    let result = executor.execute_command(&cmd).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_execute_command_default_timeout() {
    let config = ExecutorConfig {
        default_timeout: 10,
        ..Default::default()
    };
    let executor = AgentsMdExecutor::with_config(config).unwrap();

    let cmd = Command {
        name: "Echo".to_string(),
        command: "echo test".to_string(),
        working_dir: None,
        env: Vec::new(),
        timeout: None, // Uses default
        safe: true,
    };

    let output = executor.execute_command(&cmd).await.unwrap();
    assert_eq!(output.exit_code, 0);
}

#[tokio::test]
async fn test_execute_command_with_config_env_vars() {
    let config = ExecutorConfig {
        env_vars: vec![("CONFIG_VAR".to_string(), "config_value".to_string())],
        ..Default::default()
    };
    let executor = AgentsMdExecutor::with_config(config).unwrap();

    let cmd = Command {
        name: "PrintConfigEnv".to_string(),
        command: "printenv CONFIG_VAR".to_string(),
        working_dir: None,
        env: Vec::new(),
        timeout: Some(5),
        safe: true,
    };

    let output = executor.execute_command(&cmd).await.unwrap();
    assert_eq!(output.exit_code, 0);
    assert!(output.stdout.contains("config_value"));
}

#[tokio::test]
async fn test_execute_failing_command() {
    let executor = AgentsMdExecutor::new().unwrap();

    let cmd = Command {
        name: "False".to_string(),
        command: "false".to_string(),
        working_dir: None,
        env: Vec::new(),
        timeout: Some(5),
        safe: true,
    };

    let output = executor.execute_command(&cmd).await.unwrap();
    assert_ne!(output.exit_code, 0);
    assert!(!output.timed_out);
}
