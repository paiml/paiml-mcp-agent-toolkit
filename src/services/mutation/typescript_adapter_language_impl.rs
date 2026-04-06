#[async_trait]
impl LanguageAdapter for TypeScriptAdapter {
    fn name(&self) -> &str {
        "typescript"
    }

    fn extensions(&self) -> &[&str] {
        &["ts", "tsx", "js", "jsx"]
    }

    #[cfg(feature = "typescript-ast")]
    async fn parse(&self, source: &str) -> Result<String> {
        let cm: Lrc<SourceMap> = Default::default();
        let fm = cm.new_source_file(FileName::Anon.into(), source.to_string());

        let lexer = Lexer::new(
            Syntax::Typescript(TsSyntax {
                tsx: true,
                decorators: true,
                ..Default::default()
            }),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );

        let mut parser = Parser::new_from(lexer);
        parser
            .parse_module()
            .map_err(|_| anyhow::anyhow!("Parse failed"))?;

        Ok(source.to_string())
    }

    #[cfg(not(feature = "typescript-ast"))]
    async fn parse(&self, source: &str) -> Result<String> {
        Ok(source.to_string())
    }

    async fn unparse(&self, ast: &str) -> Result<String> {
        Ok(ast.to_string())
    }

    fn mutation_operators(&self) -> Vec<Box<dyn MutationOperator>> {
        vec![
            Box::new(ArithmeticOperatorReplacement),
            Box::new(RelationalOperatorReplacement),
            Box::new(ConditionalOperatorReplacement),
            Box::new(UnaryOperatorReplacement),
        ]
    }

    async fn run_tests(&self, source_file: &Path) -> Result<TestRunResult> {
        // GREEN PHASE: Real test execution
        use std::time::Instant;
        use tokio::process::Command;

        // Find project root with package.json
        let project_root = find_package_json_root(source_file)
            .ok_or_else(|| anyhow::anyhow!("No package.json found"))?;

        // Detect test command from package.json
        let package_json_path = project_root.join("package.json");
        let package_json = tokio::fs::read_to_string(&package_json_path).await?;
        let test_cmd = detect_test_command(&package_json)?;

        // Run tests with timeout
        let start = Instant::now();
        let output = Command::new("npm")
            .arg("run")
            .arg(&test_cmd)
            .current_dir(project_root)
            .output()
            .await?;

        let execution_time_ms = start.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // Parse failures
        let failures = parse_test_failures(&stdout, &stderr);
        let passed = output.status.success();

        Ok(TestRunResult {
            passed,
            failures,
            execution_time_ms,
            stdout,
            stderr,
        })
    }
}
