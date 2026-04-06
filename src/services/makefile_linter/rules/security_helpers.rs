fn contains_shell_injection(command: &str) -> bool {
    debug_assert!(!command.is_empty(), "command must not be empty");
    static UNQUOTED_VAR_REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = UNQUOTED_VAR_REGEX.get_or_init(|| {
        // Match $(VAR) or ${VAR} not inside quotes
        // But NOT $$(...) or $${...} which are Make-escaped variables
        Regex::new(r#"(?:^|[^"'\$])\$\([^)]+\)|(?:^|[^"'\$])\$\{[^}]+\}"#).expect("internal error")
    });

    // Skip safe commands
    if command.trim_start().starts_with('@') {
        let cmd = &command.trim_start()[1..];
        if cmd.starts_with("echo") || cmd.starts_with("printf") {
            return false;
        }
    }

    // Skip if command contains properly quoted Make variables like "$${VAR}" or "$(VAR)"
    if command.contains("\"$$") || command.contains("\"$(") {
        return false;
    }

    // Check for dangerous patterns
    let dangerous_commands = ["rm", "find", "curl", "wget", "tar", "install"];
    let has_dangerous = dangerous_commands.iter().any(|&cmd| command.contains(cmd));

    // Simple logic: dangerous command + unquoted variable = shell injection risk
    has_dangerous && regex.is_match(command)
}

fn quote_variables(command: &str) -> String {
    debug_assert!(!command.is_empty(), "command must not be empty");
    static VAR_REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = VAR_REGEX.get_or_init(|| Regex::new(r#"\$\(([^)]+)\)"#).expect("internal error"));

    regex.replace_all(command, "\"$($1)\"").to_string()
}

fn detect_secret(name: &str, value: &str) -> Option<String> {
    debug_assert!(!name.is_empty(), "name must not be empty");
    debug_assert!(!value.is_empty(), "value must not be empty");
    let value_trimmed = value.trim();

    check_secret_var_name(name, value_trimmed)
        .or_else(|| check_aws_credential(value_trimmed))
        .or_else(|| check_github_token(value_trimmed))
        .or_else(|| check_jwt_token(value_trimmed))
        .or_else(|| check_api_key(value_trimmed))
}

fn check_secret_var_name(name: &str, value: &str) -> Option<String> {
    debug_assert!(!name.is_empty(), "name must not be empty");
    debug_assert!(!value.is_empty(), "value must not be empty");
    let name_lower = name.to_lowercase();
    let is_secret_name = name_lower.contains("password")
        || name_lower.contains("secret")
        || name_lower.contains("token")
        || name_lower.contains("api_key")
        || name_lower.contains("access_key");

    if is_secret_name && !value.starts_with('$') && value.len() > 4 {
        Some("credential".to_string())
    } else {
        None
    }
}

fn check_aws_credential(value: &str) -> Option<String> {
    debug_assert!(!value.is_empty(), "value must not be empty");
    if value.starts_with("AKIA") && value.len() == 20 {
        Some("AWS access key".to_string())
    } else {
        None
    }
}

fn check_github_token(value: &str) -> Option<String> {
    debug_assert!(!value.is_empty(), "value must not be empty");
    if value.starts_with("ghp_") || value.starts_with("github_pat_") {
        Some("GitHub token".to_string())
    } else {
        None
    }
}

fn check_jwt_token(value: &str) -> Option<String> {
    debug_assert!(!value.is_empty(), "value must not be empty");
    if value.starts_with("eyJ") {
        Some("JWT token".to_string())
    } else {
        None
    }
}

fn check_api_key(value: &str) -> Option<String> {
    debug_assert!(!value.is_empty(), "value must not be empty");
    if value.starts_with("sk-") || value.starts_with("pk-") {
        Some("API key".to_string())
    } else {
        None
    }
}

fn detect_secret_in_command(command: &str) -> Option<String> {
    debug_assert!(!command.is_empty(), "command must not be empty");
    // Check for Bearer tokens
    if command.contains("Bearer ") {
        if let Some(pos) = command.find("Bearer ") {
            let token_part = &command[pos + 7..];
            if !token_part.starts_with('$') && token_part.len() > 10 {
                return Some("Bearer token".to_string());
            }
        }
    }

    // Check for Authorization headers
    if command.contains("Authorization:") {
        return Some("authorization credential".to_string());
    }

    None
}

fn detect_unsafe_command(command: &str) -> Option<(&'static str, Severity)> {
    debug_assert!(!command.is_empty(), "command must not be empty");
    let cmd_trimmed = command.trim();

    // Critical: rm -rf /
    if cmd_trimmed.contains("rm ")
        && cmd_trimmed.contains("-rf /")
        && (cmd_trimmed.contains("-rf /") || cmd_trimmed.contains("-rf /*"))
    {
        return Some(("rm -rf / - extremely dangerous", Severity::Error));
    }

    // High: curl | bash pattern
    if (cmd_trimmed.contains("curl") || cmd_trimmed.contains("wget"))
        && (cmd_trimmed.contains("| bash") || cmd_trimmed.contains("| sh"))
    {
        return Some(("downloading and piping to shell", Severity::Error));
    }

    // High: eval with untrusted input
    if cmd_trimmed.starts_with("eval") && cmd_trimmed.contains('$') {
        return Some(("eval with variable input", Severity::Error));
    }

    // High: chmod 777
    if cmd_trimmed.contains("chmod 777") {
        return Some(("chmod 777 - overly permissive", Severity::Warning));
    }

    None
}

fn detect_privilege_escalation(command: &str) -> Option<String> {
    debug_assert!(!command.is_empty(), "command must not be empty");
    let cmd_trimmed = command.trim();

    // Check for sudo with variables
    if cmd_trimmed.starts_with("sudo ") && cmd_trimmed.contains('$') {
        return Some("sudo with untrusted variable input".to_string());
    }

    // Check for setuid
    if cmd_trimmed.contains("chmod") && cmd_trimmed.contains("+s") {
        return Some("creating setuid binary".to_string());
    }

    // Check for su command
    if cmd_trimmed.starts_with("su ") {
        return Some("using su command".to_string());
    }

    // Check for pkexec
    if cmd_trimmed.contains("pkexec") {
        return Some("using pkexec for privilege escalation".to_string());
    }

    None
}

fn truncate_command(command: &str) -> &str {
    debug_assert!(!command.is_empty(), "command must not be empty");
    if command.len() > 50 {
        &command[..50]
    } else {
        command
    }
}

fn suggest_safe_alternative(pattern: &str) -> String {
    match pattern {
        "rm -rf / - extremely dangerous" => "Use specific paths and add safety checks".to_string(),
        "downloading and piping to shell" => {
            "Download file first, verify checksum, then execute".to_string()
        }
        "eval with variable input" => "Avoid eval; use direct command execution".to_string(),
        "chmod 777 - overly permissive" => {
            "Use more restrictive permissions (e.g., 755 or 644)".to_string()
        }
        _ => "Review command for security implications".to_string(),
    }
}
