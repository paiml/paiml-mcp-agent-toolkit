//! Tests for unified protocol error module

use super::*;

#[test]
fn test_unified_error_variants() {
    // Test Io error
    let io_err = UnifiedError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
    assert!(matches!(io_err, UnifiedError::Io(_)));
    assert!(format!("{}", io_err).contains("IO error"));
    
    // Test Serialization error
    let ser_err = UnifiedError::Serialization("test error".to_string());
    assert!(matches!(ser_err, UnifiedError::Serialization(_)));
    
    // Test Protocol error
    let proto_err = UnifiedError::Protocol("protocol error".to_string());
    assert!(matches!(proto_err, UnifiedError::Protocol(_)));
    
    // Test NotFound error
    let not_found = UnifiedError::NotFound("resource".to_string());
    assert!(matches!(not_found, UnifiedError::NotFound(_)));
    
    // Test BadRequest error
    let bad_req = UnifiedError::BadRequest("invalid input".to_string());
    assert!(matches!(bad_req, UnifiedError::BadRequest(_)));
    
    // Test Internal error
    let internal = UnifiedError::Internal("internal error".to_string());
    assert!(matches!(internal, UnifiedError::Internal(_)));
    
    // Test Timeout error
    let timeout = UnifiedError::Timeout;
    assert!(matches!(timeout, UnifiedError::Timeout));
    
    // Test Unauthorized error
    let unauth = UnifiedError::Unauthorized;
    assert!(matches!(unauth, UnifiedError::Unauthorized));
}

#[test]
fn test_unified_error_from_io_error() {
    let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "no access");
    let unified_error: UnifiedError = io_error.into();
    assert!(matches!(unified_error, UnifiedError::Io(_)));
}

#[test]
fn test_unified_error_from_serde_json_error() {
    let json_err = serde_json::from_str::<serde_json::Value>("invalid json");
    assert!(json_err.is_err());
    let unified_error: UnifiedError = json_err.unwrap_err().into();
    assert!(matches!(unified_error, UnifiedError::Serialization(_)));
}

#[test]
fn test_unified_error_display() {
    let errors = vec![
        UnifiedError::NotFound("user".to_string()),
        UnifiedError::BadRequest("invalid id".to_string()),
        UnifiedError::Internal("server error".to_string()),
        UnifiedError::Timeout,
        UnifiedError::Unauthorized,
    ];
    
    for error in errors {
        let error_string = format!("{}", error);
        assert!(!error_string.is_empty());
    }
}

#[test]
fn test_unified_error_is_retryable() {
    // These should be retryable
    assert!(UnifiedError::Timeout.is_retryable());
    assert!(UnifiedError::Io(std::io::Error::new(std::io::ErrorKind::Interrupted, "")).is_retryable());
    
    // These should not be retryable
    assert!(!UnifiedError::BadRequest("".to_string()).is_retryable());
    assert!(!UnifiedError::NotFound("".to_string()).is_retryable());
    assert!(!UnifiedError::Unauthorized.is_retryable());
}

#[test]
fn test_unified_error_status_code() {
    use http::StatusCode;
    
    assert_eq!(UnifiedError::NotFound("".to_string()).status_code(), StatusCode::NOT_FOUND);
    assert_eq!(UnifiedError::BadRequest("".to_string()).status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(UnifiedError::Unauthorized.status_code(), StatusCode::UNAUTHORIZED);
    assert_eq!(UnifiedError::Timeout.status_code(), StatusCode::REQUEST_TIMEOUT);
    assert_eq!(UnifiedError::Internal("".to_string()).status_code(), StatusCode::INTERNAL_SERVER_ERROR);
}