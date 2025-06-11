//! Tests for unified protocol

use super::*;
use super::error::UnifiedError;
use serde_json::json;

#[test]
fn test_unified_error_variants() {
    let io_error = UnifiedError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
    assert!(matches!(io_error, UnifiedError::Io(_)));
    
    let json_error = UnifiedError::Json(serde_json::Error::custom("test"));
    assert!(matches!(json_error, UnifiedError::Json(_)));
    
    let protocol_error = UnifiedError::Protocol("test error".to_string());
    assert!(matches!(protocol_error, UnifiedError::Protocol(_)));
    
    let internal_error = UnifiedError::Internal("internal error".to_string());
    assert!(matches!(internal_error, UnifiedError::Internal(_)));
}

#[test]
fn test_unified_request_creation() {
    let req = UnifiedRequest {
        method: "test_method".to_string(),
        params: json!({"key": "value"}),
        context: RequestContext {
            request_id: "test123".to_string(),
            timestamp: std::time::SystemTime::now(),
            source: ProtocolSource::Cli,
        },
    };
    
    assert_eq!(req.method, "test_method");
    assert_eq!(req.context.request_id, "test123");
    assert!(matches!(req.context.source, ProtocolSource::Cli));
}

#[test]
fn test_unified_response_creation() {
    let resp = UnifiedResponse {
        result: Some(json!({"status": "ok"})),
        error: None,
        metadata: ResponseMetadata {
            request_id: "test123".to_string(),
            duration_ms: 100,
            cache_hit: false,
        },
    };
    
    assert!(resp.result.is_some());
    assert!(resp.error.is_none());
    assert_eq!(resp.metadata.request_id, "test123");
    assert_eq!(resp.metadata.duration_ms, 100);
    assert!(!resp.metadata.cache_hit);
}

#[test]
fn test_protocol_source_variants() {
    let sources = vec![
        ProtocolSource::Cli,
        ProtocolSource::Http,
        ProtocolSource::Mcp,
    ];
    
    for source in sources {
        match source {
            ProtocolSource::Cli => assert_eq!(format!("{:?}", source), "Cli"),
            ProtocolSource::Http => assert_eq!(format!("{:?}", source), "Http"),
            ProtocolSource::Mcp => assert_eq!(format!("{:?}", source), "Mcp"),
        }
    }
}

#[test]
fn test_request_context_default() {
    let ctx = RequestContext::default();
    assert!(!ctx.request_id.is_empty());
    assert!(matches!(ctx.source, ProtocolSource::Cli));
}

#[test]
fn test_response_metadata_default() {
    let meta = ResponseMetadata::default();
    assert!(!meta.request_id.is_empty());
    assert_eq!(meta.duration_ms, 0);
    assert!(!meta.cache_hit);
}