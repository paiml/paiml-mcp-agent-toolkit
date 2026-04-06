impl McpServer {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(
        agent_registry: Arc<AgentRegistry>,
        config: ServerConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let capabilities = ServerCapabilities {
            experimental: None,
            logging: if config.enable_logging {
                Some(LoggingCapabilities {
                    level: "info".to_string(),
                })
            } else {
                None
            },
            prompts: Some(PromptsCapability {
                list_changed: Some(true),
            }),
            resources: Some(ResourcesCapability {
                subscribe: Some(true),
                list_changed: Some(true),
            }),
            tools: Some(ToolsCapability {
                list_changed: Some(true),
            }),
        };

        let server_info = ServerInfo {
            name: config.name.clone(),
            version: config.version.clone(),
            protocol_version: MCP_VERSION.to_string(),
        };

        let context = Arc::new(McpContext {
            server_info,
            capabilities,
            tools: Arc::new(RwLock::new(ToolRegistry::new())),
            resources: Arc::new(RwLock::new(ResourceRegistry::new())),
            prompts: Arc::new(RwLock::new(PromptRegistry::new())),
            agent_registry,
        });

        Ok(Self {
            context,
            config,
            shutdown: Arc::new(tokio::sync::Notify::new()),
        })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn run_tcp(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(&self.config.bind_address).await?;
        println!("MCP Server listening on {}", self.config.bind_address);

        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.config.max_connections));

        loop {
            tokio::select! {
                accept = listener.accept() => {
                    let (stream, addr) = accept?;

                    let permit = semaphore.clone().acquire_owned().await?;
                    let context = self.context.clone();
                    let config = self.config.clone();

                    tokio::spawn(async move {
                        let transport = Arc::new(TcpTransport::new(stream));
                        let session = McpSession::new(context, transport.clone());

                        if let Err(e) = handle_session(session, config).await {
                            eprintln!("Session error from {}: {}", addr, e);
                        }

                        drop(permit);
                    });
                }
                _ = self.shutdown.notified() => {
                    println!("Shutting down MCP server");
                    break;
                }
            }
        }

        Ok(())
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn run_unix(&self) -> Result<(), Box<dyn std::error::Error>> {
        let socket_path = self
            .config
            .unix_socket
            .as_ref()
            .ok_or("Unix socket path not configured")?;

        // Remove existing socket file
        let _ = std::fs::remove_file(socket_path);

        let listener = UnixListener::bind(socket_path)?;
        println!("MCP Server listening on Unix socket: {}", socket_path);

        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.config.max_connections));

        loop {
            tokio::select! {
                accept = listener.accept() => {
                    let (stream, _) = accept?;

                    let permit = semaphore.clone().acquire_owned().await?;
                    let context = self.context.clone();
                    let config = self.config.clone();

                    tokio::spawn(async move {
                        let transport = Arc::new(UnixTransport::new(stream));
                        let session = McpSession::new(context, transport.clone());

                        if let Err(e) = handle_session(session, config).await {
                            eprintln!("Session error: {}", e);
                        }

                        drop(permit);
                    });
                }
                _ = self.shutdown.notified() => {
                    println!("Shutting down MCP server");
                    break;
                }
            }
        }

        Ok(())
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn run_stdio(&self) -> Result<(), Box<dyn std::error::Error>> {
        let transport = Arc::new(StdioTransport::new());
        let session = McpSession::new(self.context.clone(), transport);

        handle_session(session, self.config.clone()).await
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn shutdown(&self) {
        self.shutdown.notify_waiters();
    }
}
