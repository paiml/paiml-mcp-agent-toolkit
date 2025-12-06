//! WebSocket Protocol - Binary messaging
//!
//! Replaces Server-Sent Events (SSE) with WebSocket for bidirectional
//! communication. Uses MessagePack for efficient binary encoding.

use crate::state::SystemMetrics;
use serde::{Deserialize, Serialize};

/// Incoming WebSocket message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum WsMessage {
    /// System metrics update
    Metrics(MetricsPayload),
    /// Hotspots update
    Hotspots(HotspotsPayload),
    /// DAG update
    Dag(DagPayload),
    /// Connection status
    Status(StatusPayload),
    /// Error message
    Error(ErrorPayload),
}

/// Outgoing WebSocket command
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum WsCommand {
    /// Subscribe to a channel
    Subscribe { channel: String },
    /// Unsubscribe from a channel
    Unsubscribe { channel: String },
    /// Request data refresh
    Refresh { target: String },
    /// Analyze a file
    Analyze { path: String },
}

/// Metrics payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsPayload {
    pub cpu: f64,
    pub memory: f64,
    pub connections: u32,
}

impl From<MetricsPayload> for SystemMetrics {
    fn from(p: MetricsPayload) -> Self {
        Self {
            cpu_usage: p.cpu,
            memory_usage: p.memory,
            active_connections: p.connections,
        }
    }
}

/// Hotspots payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotsPayload {
    pub items: Vec<HotspotItem>,
}

/// Single hotspot item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotItem {
    pub file: String,
    pub complexity: u32,
    pub churn: u32,
    pub score: f64,
}

/// DAG payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagPayload {
    pub nodes: Vec<DagNodeData>,
    pub edges: Vec<DagEdgeData>,
}

/// DAG node data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagNodeData {
    pub id: String,
    pub label: String,
}

/// DAG edge data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagEdgeData {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

/// Status payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusPayload {
    pub connected: bool,
    pub server_time: u64,
}

/// Error payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub code: String,
    pub message: String,
}

/// Encode to MessagePack binary format
pub fn to_msgpack<T: Serialize>(value: &T) -> Result<Vec<u8>, ProtocolError> {
    // Simple JSON-based encoding for now (can upgrade to rmp-serde later)
    serde_json::to_vec(value).map_err(|e| ProtocolError::EncodeError(e.to_string()))
}

/// Decode from MessagePack binary format
pub fn from_msgpack<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T, ProtocolError> {
    serde_json::from_slice(bytes).map_err(|e| ProtocolError::DecodeError(e.to_string()))
}

/// Protocol error
#[derive(Debug, Clone)]
pub enum ProtocolError {
    EncodeError(String),
    DecodeError(String),
    ConnectionError(String),
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EncodeError(e) => write!(f, "Encode error: {}", e),
            Self::DecodeError(e) => write!(f, "Decode error: {}", e),
            Self::ConnectionError(e) => write!(f, "Connection error: {}", e),
        }
    }
}

impl std::error::Error for ProtocolError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_message_deserialize_metrics() {
        let json = r#"{"type":"metrics","data":{"cpu":50.0,"memory":60.0,"connections":5}}"#;
        let msg: WsMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, WsMessage::Metrics(_)));

        if let WsMessage::Metrics(payload) = msg {
            assert_eq!(payload.cpu, 50.0);
            assert_eq!(payload.memory, 60.0);
            assert_eq!(payload.connections, 5);
        }
    }

    #[test]
    fn test_ws_command_serialize_subscribe() {
        let cmd = WsCommand::Subscribe { channel: "metrics".to_string() };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("subscribe"));
        assert!(json.contains("metrics"));
    }

    #[test]
    fn test_ws_command_serialize_refresh() {
        let cmd = WsCommand::Refresh { target: "hotspots".to_string() };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("refresh"));
    }

    #[test]
    fn test_msgpack_roundtrip() {
        let metrics = SystemMetrics {
            cpu_usage: 45.5,
            memory_usage: 60.0,
            active_connections: 3,
        };
        let bytes = to_msgpack(&metrics).unwrap();
        let decoded: SystemMetrics = from_msgpack(&bytes).unwrap();
        assert_eq!(metrics.cpu_usage, decoded.cpu_usage);
        assert_eq!(metrics.memory_usage, decoded.memory_usage);
        assert_eq!(metrics.active_connections, decoded.active_connections);
    }

    #[test]
    fn test_metrics_payload_conversion() {
        let payload = MetricsPayload {
            cpu: 50.0,
            memory: 70.0,
            connections: 10,
        };
        let metrics: SystemMetrics = payload.into();
        assert_eq!(metrics.cpu_usage, 50.0);
        assert_eq!(metrics.memory_usage, 70.0);
        assert_eq!(metrics.active_connections, 10);
    }
}
