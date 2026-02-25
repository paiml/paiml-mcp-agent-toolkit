#![cfg_attr(coverage_nightly, coverage(off))]
use std::net::SocketAddr;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use http_body_util::BodyExt;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use serde::Serialize;
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info};

use crate::unified_protocol::{
    HttpContext, Protocol, ProtocolAdapter, ProtocolError, UnifiedRequest, UnifiedResponse,
};

include!("http_adapter.rs");
include!("http_server.rs");
include!("http_response_builder.rs");
include!("http_tests.rs");
include!("http_extended_tests.rs");
