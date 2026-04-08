/// Local demo server.
pub struct LocalDemoServer {
    port: u16,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

impl std::fmt::Debug for LocalDemoServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalDemoServer")
            .field("port", &self.port)
            .field("shutdown_tx", &"<oneshot sender>")
            .finish()
    }
}

impl LocalDemoServer {
    #[cfg(feature = "demo")]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn spawn(initial_content: DemoContent) -> Result<(Self, u16)> {
        Self::spawn_with_results(initial_content, None, None, None).await
    }

    #[cfg(feature = "demo")]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn spawn_with_results(
        initial_content: DemoContent,
        complexity_report: Option<crate::services::complexity::ComplexityReport>,
        churn_analysis: Option<crate::models::churn::CodeChurnAnalysis>,
        dependency_graph: Option<DependencyGraph>,
    ) -> Result<(Self, u16)> {
        // Bind to ephemeral port
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();

        // Create initial state
        let state = Arc::new(RwLock::new(DemoState {
            repository: std::path::PathBuf::from("."),
            analysis_results: AnalysisResults {
                files_analyzed: initial_content.files_analyzed,
                avg_complexity: initial_content.avg_complexity,
                tech_debt_hours: initial_content.tech_debt_hours,
                complexity_report: complexity_report.unwrap_or_default(),
                churn_analysis: churn_analysis.unwrap_or_default(),
                dependency_graph: dependency_graph.unwrap_or_default(),
                tdg_summary: None, // Will be populated during analysis
            },
            mermaid_cache: Arc::new(DashMap::new()),
            system_diagram: initial_content.system_diagram.clone(),
        }));

        // Spawn accept loop with bounded concurrency
        tokio::spawn(async move {
            let semaphore = Arc::new(Semaphore::new(100)); // Max 100 concurrent connections

            loop {
                tokio::select! {
                    accept_result = listener.accept() => {
                        if let Ok((stream, _)) = accept_result {
                            let permit = semaphore
                                .clone()
                                .acquire_owned()
                                .await
                                .expect("Semaphore should always be available");
                            let state = Arc::clone(&state);

                            tokio::spawn(async move {
                                if let Err(e) = handle_connection(stream, state).await {
                                    eprintln!("Connection error: {e}");
                                }
                                drop(permit);
                            });
                        }
                    }
                    _ = &mut shutdown_rx => {
                        break;
                    }
                }
            }
        });

        Ok((Self { port, shutdown_tx }, port))
    }

    #[cfg(not(feature = "demo"))]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn spawn(_initial_content: DemoContent) -> Result<(Self, u16)> {
        anyhow::bail!("Demo mode not available. Build with --features demo")
    }

    #[cfg(not(feature = "demo"))]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn spawn_with_results(
        _initial_content: DemoContent,
        _complexity_report: Option<crate::services::complexity::ComplexityReport>,
        _churn_analysis: Option<crate::models::churn::CodeChurnAnalysis>,
        _dependency_graph: Option<DependencyGraph>,
    ) -> Result<(Self, u16)> {
        anyhow::bail!("Demo mode not available. Build with --features demo")
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Port.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Shutdown.
    pub fn shutdown(self) {
        let _ = self.shutdown_tx.send(());
    }
}

#[cfg(feature = "demo")]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
async fn handle_connection(mut stream: TcpStream, state: Arc<RwLock<DemoState>>) -> Result<()> {
    let mut buffer = BytesMut::with_capacity(4096);
    stream.read_buf(&mut buffer).await?;

    // Parse HTTP request
    let request = parse_minimal_request(&buffer)?;

    // Use router to handle request
    let response = super::router::handle_request(&request.path, &state);

    // Write response with zero-copy
    let response_bytes = serialize_response(response);
    stream.write_all(&response_bytes).await?;
    stream.flush().await?;

    Ok(())
}

#[cfg(feature = "demo")]
#[derive(Debug)]
struct MinimalRequest {
    path: String,
}

#[cfg(feature = "demo")]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn parse_minimal_request(buffer: &[u8]) -> Result<MinimalRequest> {
    let request_str = std::str::from_utf8(buffer)?;
    let first_line = request_str
        .lines()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Empty request"))?;

    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(anyhow::anyhow!("Invalid request line"));
    }

    Ok(MinimalRequest {
        path: parts[1].to_string(),
    })
}

#[cfg(feature = "demo")]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn serialize_response(response: Response<Bytes>) -> Vec<u8> {
    let mut output = Vec::new();

    // Status line
    output.extend_from_slice(b"HTTP/1.1 ");
    output.extend_from_slice(response.status().as_str().as_bytes());
    output.extend_from_slice(b" ");
    output.extend_from_slice(
        response
            .status()
            .canonical_reason()
            .unwrap_or("Unknown")
            .as_bytes(),
    );
    output.extend_from_slice(b"\r\n");

    // Headers
    for (name, value) in response.headers() {
        output.extend_from_slice(name.as_str().as_bytes());
        output.extend_from_slice(b": ");
        output.extend_from_slice(value.as_bytes());
        output.extend_from_slice(b"\r\n");
    }

    // Content-Length
    output.extend_from_slice(b"Content-Length: ");
    output.extend_from_slice(response.body().len().to_string().as_bytes());
    output.extend_from_slice(b"\r\n");

    // End of headers
    output.extend_from_slice(b"\r\n");

    // Body
    output.extend_from_slice(response.body());

    output
}
