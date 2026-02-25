use super::*;
use crate::agents::registry::AgentRegistry;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, UnixListener};

// Structs: McpServer, ServerConfig, impl Default for ServerConfig
include!("server_types.rs");

// impl McpServer: registration methods (register_defaults, register_agent_tools, etc.)
include!("server_registration.rs");

// Transport implementations: handle_session, TcpTransport, UnixTransport, StdioTransport
include!("server_transport.rs");

// impl McpServer: new(), run_tcp(), run_unix(), run_stdio(), shutdown()
include!("server_runtime.rs");

// Unit tests
include!("server_tests.rs");
