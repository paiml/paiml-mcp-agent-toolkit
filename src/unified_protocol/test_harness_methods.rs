impl TestHarness {
    /// Create a new instance.
    pub fn new() -> Self {
        // Create service with default implementations
        let service = UnifiedService::new()
            .with_template_service(DefaultTemplateService)
            .with_analysis_service(DefaultAnalysisService);

        Self {
            service,
            mcp_adapter: McpAdapter::new(),
            http_adapter: HttpAdapter::new("127.0.0.1:0".parse().unwrap()),
            cli_adapter: CliAdapter::new(),
            test_results: Arc::new(Mutex::new(TestResults::default())),
        }
    }

    /// Configure the test harness with custom services
    pub fn with_template_service<T: TemplateService + 'static>(mut self, service: T) -> Self {
        self.service = self.service.with_template_service(service);
        self
    }

    /// With analysis service.
    pub fn with_analysis_service<A: AnalysisService + 'static>(mut self, service: A) -> Self {
        self.service = self.service.with_analysis_service(service);
        self
    }

    /// Test a single endpoint across all protocols
    pub async fn test_endpoint<
        T: Serialize,
        R: for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
    >(
        &self,
        test_name: &str,
        method: &str,
        path: &str,
        body: T,
    ) -> Result<(), TestError> {
        info!("Running test: {}", test_name);

        let mut results = self.test_results.lock().await;
        results.tests_run += 1;

        // Test all protocols
        let protocols = [Protocol::Mcp, Protocol::Http, Protocol::Cli];
        let mut responses = HashMap::with_capacity(64);

        for protocol in &protocols {
            match self.test_protocol(protocol, method, path, &body).await {
                Ok(response) => {
                    responses.insert(*protocol, response);
                }
                Err(e) => {
                    results
                        .protocol_failures
                        .entry(*protocol)
                        .or_insert_with(Vec::new)
                        .push(format!("{}: {}", test_name, e));
                    debug!("Protocol {} failed for test {}: {}", protocol, test_name, e);
                }
            }
        }

        // Check protocol equivalence
        if responses.len() >= 2 {
            let protocol_pairs: Vec<_> = responses.keys().collect();
            for i in 0..protocol_pairs.len() {
                for j in i + 1..protocol_pairs.len() {
                    let p1 = *protocol_pairs[i];
                    let p2 = *protocol_pairs[j];

                    if let (Some(r1), Some(r2)) = (responses.get(&p1), responses.get(&p2)) {
                        let normalized_r1 = self.normalize_response(r1, p1);
                        let normalized_r2 = self.normalize_response(r2, p2);

                        if normalized_r1 != normalized_r2 {
                            results.equivalence_failures.push(EquivalenceFailure {
                                test_name: test_name.to_string(),
                                protocols: (p1, p2),
                                expected: normalized_r1.clone(),
                                actual: normalized_r2.clone(),
                                difference: format!(
                                    "Protocols {} and {} returned different results",
                                    p1, p2
                                ),
                            });
                        }
                    }
                }
            }
        }

        if responses.len() == protocols.len() {
            results.tests_passed += 1;
            Ok(())
        } else {
            results.tests_failed += 1;
            Err(TestError::ProtocolFailure(format!(
                "Test {} failed on {} protocols",
                test_name,
                protocols.len() - responses.len()
            )))
        }
    }

    /// Test template generation across all protocols
    pub async fn test_template_generation(&self) -> Result<(), TestError> {
        let request_body = json!({
            "template_uri": "template://makefile/rust/cli",
            "parameters": {
                "project_name": "test-project"
            }
        });

        self.test_endpoint::<_, Value>(
            "template_generation",
            "POST",
            "/api/v1/generate",
            request_body,
        )
        .await
    }

    /// Test template listing across all protocols
    pub async fn test_template_listing(&self) -> Result<(), TestError> {
        self.test_endpoint::<_, Value>("template_listing", "GET", "/api/v1/templates", json!({}))
            .await
    }

    /// Test complexity analysis across all protocols
    pub async fn test_complexity_analysis(&self) -> Result<(), TestError> {
        let request_body = json!({
            "project_path": ".",
            "toolchain": "rust",
            "format": "json"
        });

        self.test_endpoint::<_, Value>(
            "complexity_analysis",
            "POST",
            "/api/v1/analyze/complexity",
            request_body,
        )
        .await
    }

    /// Test error handling consistency across protocols
    pub async fn test_error_handling(&self) -> Result<(), TestError> {
        // Test with non-existent template
        let request_body = json!({
            "template_uri": "template://nonexistent/template",
            "parameters": {}
        });

        // For error cases, we expect all protocols to fail consistently
        let protocols = [Protocol::Mcp, Protocol::Http, Protocol::Cli];
        let mut error_responses = HashMap::with_capacity(64);

        for protocol in &protocols {
            match self
                .test_protocol(protocol, "POST", "/api/v1/generate", &request_body)
                .await
            {
                Ok(_) => {
                    return Err(TestError::UnexpectedSuccess(format!(
                        "Protocol {} should have failed but succeeded",
                        protocol
                    )));
                }
                Err(e) => {
                    error_responses.insert(*protocol, e.to_string());
                }
            }
        }

        // Verify all protocols failed (which is expected)
        if error_responses.len() == protocols.len() {
            info!("Error handling test passed - all protocols failed as expected");
            Ok(())
        } else {
            Err(TestError::InconsistentBehavior(
                "Not all protocols failed consistently".to_string(),
            ))
        }
    }

    /// Run the complete test suite
    pub async fn run_test_suite(&self) -> TestSuiteResults {
        info!("Starting unified protocol test suite");

        let tests = vec![
            (
                "template_generation",
                Box::new(|| Box::pin(self.test_template_generation())) as TestFunction,
            ),
            (
                "template_listing",
                Box::new(|| Box::pin(self.test_template_listing())),
            ),
            (
                "complexity_analysis",
                Box::new(|| Box::pin(self.test_complexity_analysis())),
            ),
            (
                "error_handling",
                Box::new(|| Box::pin(self.test_error_handling())),
            ),
        ];

        let mut suite_results = TestSuiteResults::new();

        for (test_name, test_fn) in tests {
            info!("Running test: {}", test_name);
            match test_fn().await {
                Ok(()) => {
                    suite_results.passed.push(test_name.to_string());
                    info!("✅ Test {} passed", test_name);
                }
                Err(e) => {
                    suite_results
                        .failed
                        .push((test_name.to_string(), e.to_string()));
                    info!("❌ Test {} failed: {}", test_name, e);
                }
            }
        }

        let final_results = self.test_results.lock().await.clone();
        suite_results.summary = Some(final_results);

        info!(
            "Test suite completed: {} passed, {} failed",
            suite_results.passed.len(),
            suite_results.failed.len()
        );

        suite_results
    }

    /// Test a specific protocol with given parameters
    async fn test_protocol<T: Serialize>(
        &self,
        protocol: &Protocol,
        method: &str,
        path: &str,
        body: &T,
    ) -> Result<Value, TestError> {
        let unified_request = UnifiedRequest::new(
            method
                .parse()
                .map_err(|e| TestError::InvalidRequest(format!("Invalid method: {}", e)))?,
            path.to_string(),
        )
        .with_body(Body::from(serde_json::to_vec(body)?))
        .with_header("content-type", "application/json")
        .with_extension("protocol", *protocol);

        let response = self
            .service
            .process_request(unified_request)
            .await
            .map_err(|e| TestError::ServiceError(e.to_string()))?;

        if response.status.is_success() {
            let body_bytes = axum::body::to_bytes(response.body, usize::MAX)
                .await
                .map_err(|e| {
                    TestError::ResponseError(format!("Failed to read response body: {}", e))
                })?;

            serde_json::from_slice(&body_bytes)
                .map_err(|e| TestError::ResponseError(format!("Invalid JSON response: {}", e)))
        } else {
            Err(TestError::HttpError(
                response.status,
                String::from_utf8_lossy(
                    &axum::body::to_bytes(response.body, usize::MAX)
                        .await
                        .unwrap_or_default(),
                )
                .to_string(),
            ))
        }
    }

    /// Normalize responses for protocol comparison
    fn normalize_response(&self, response: &Value, protocol: Protocol) -> Value {
        let mut normalized = response.clone();

        // Remove protocol-specific fields
        match protocol {
            Protocol::Mcp => {
                // Remove JSON-RPC specific fields
                if let Value::Object(ref mut obj) = normalized {
                    obj.remove("jsonrpc");
                    obj.remove("id");
                }
            }
            Protocol::Http => {
                // HTTP responses are already normalized
            }
            Protocol::Cli => {
                // CLI responses might have different formatting
                if let Value::Object(ref mut obj) = normalized {
                    obj.remove("exit_code");
                }
            }
            Protocol::WebSocket => {
                // WebSocket normalization (if implemented)
            }
        }

        normalized
    }
}

impl Default for TestHarness {
    fn default() -> Self {
        Self::new()
    }
}
