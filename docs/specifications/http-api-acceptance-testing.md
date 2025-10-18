# HTTP API Acceptance Testing Specification
**Version**: 1.0  
**Date**: 2025-09-10  
**Status**: Implementation Required  
**Coverage Target**: 100% of HTTP endpoints, methods, and integration scenarios

## 1. Overview

This specification defines comprehensive acceptance testing for the `pmat` HTTP API server, ensuring 100% coverage of all HTTP endpoints, REST API compliance, WebSocket functionality, and integration scenarios. Every HTTP interface must be tested for correctness, performance, security, and API compatibility.

## 2. HTTP API Architecture

### 2.1 API Design Principles
- **RESTful Design**: Follows REST architectural principles
- **JSON Communication**: Request/response bodies use JSON format
- **HTTP Status Codes**: Proper use of HTTP status codes for all scenarios
- **WebSocket Support**: Real-time streaming for long-running operations
- **Content Negotiation**: Supports multiple response formats (JSON, HTML, CSV)

### 2.2 API Versioning
- **v1 APIs**: Current stable API version under `/api/v1/`
- **Legacy APIs**: Backwards compatible endpoints under `/api/`
- **Deprecation Policy**: Clear deprecation notices and migration paths

### 2.3 Testing Methodology
- **API Contract Testing**: Validate request/response schemas
- **Integration Testing**: End-to-end HTTP request/response testing
- **Performance Testing**: Load testing and response time validation
- **Security Testing**: Authentication, authorization, and input validation
- **Cross-Platform Testing**: Test across different HTTP clients and browsers

## 3. HTTP Endpoints Coverage Matrix

### 3.1 Dashboard and UI Endpoints
| Endpoint | Method | Coverage | Test Cases | Error Cases | Performance |
|----------|--------|----------|------------|-------------|-------------|
| `/` | GET | ⏳ | 8 | 3 | ⏳ |
| `/vendor/*` | GET | ⏳ | 12 | 5 | ⏳ |
| `/demo.*` | GET | ⏳ | 6 | 3 | ⏳ |

### 3.2 Core API v1 Endpoints (Legacy)
| Endpoint | Method | Coverage | Test Cases | Error Cases | Performance |
|----------|--------|----------|------------|-------------|-------------|
| `/api/summary` | GET | ⏳ | 10 | 6 | ⏳ |
| `/api/metrics` | GET | ⏳ | 12 | 7 | ⏳ |
| `/api/hotspots` | GET | ⏳ | 9 | 5 | ⏳ |
| `/api/dag` | GET | ⏳ | 8 | 4 | ⏳ |
| `/api/system-diagram` | GET | ⏳ | 7 | 4 | ⏳ |
| `/api/analysis` | GET | ⏳ | 15 | 8 | ⏳ |
| `/api/recommendations` | GET | ⏳ | 11 | 6 | ⏳ |
| `/api/polyglot` | GET | ⏳ | 13 | 7 | ⏳ |
| `/api/showcase` | GET | ⏳ | 9 | 5 | ⏳ |

### 3.3 Enhanced API v1 Endpoints (Current)
| Endpoint | Method | Coverage | Test Cases | Error Cases | Performance |
|----------|--------|----------|------------|-------------|-------------|
| `/api/v1/analysis/architecture` | GET | ⏳ | 18 | 9 | ⏳ |
| `/api/v1/analysis/defects` | GET | ⏳ | 16 | 8 | ⏳ |
| `/api/v1/analysis/statistics` | GET | ⏳ | 14 | 7 | ⏳ |
| `/api/v1/analysis/diagram` | GET | ⏳ | 12 | 6 | ⏳ |
| `/api/v1/analysis/stream` | GET/WebSocket | ⏳ | 20 | 10 | ⏳ |

### 3.4 Analysis Trigger Endpoints (POST Operations)
| Endpoint | Method | Coverage | Test Cases | Error Cases | Performance |
|----------|--------|----------|------------|-------------|-------------|
| `/api/v1/analysis/trigger` | POST | ⏳ | 15 | 8 | ⏳ |
| `/api/v1/projects` | POST | ⏳ | 12 | 7 | ⏳ |
| `/api/v1/templates/generate` | POST | ⏳ | 18 | 9 | ⏳ |
| `/api/v1/quality-gate/check` | POST | ⏳ | 14 | 8 | ⏳ |

### 3.5 WebSocket Endpoints
| Endpoint | Protocol | Coverage | Test Cases | Error Cases | Performance |
|----------|----------|----------|------------|-------------|-------------|
| `/ws/analysis` | WebSocket | ⏳ | 12 | 6 | ⏳ |
| `/ws/progress` | WebSocket | ⏳ | 8 | 4 | ⏳ |
| `/ws/notifications` | WebSocket | ⏳ | 10 | 5 | ⏳ |

## 4. HTTP Method Coverage

### 4.1 HTTP Methods Testing
| Method | Usage | Test Cases | Authentication | Content Types |
|--------|-------|------------|----------------|---------------|
| GET | Data retrieval | 25 | ⏳ | JSON, HTML, CSV |
| POST | Data creation/triggers | 15 | ⏳ | JSON, Form Data |
| PUT | Data updates | 8 | ⏳ | JSON |
| DELETE | Resource deletion | 5 | ⏳ | N/A |
| HEAD | Metadata only | 10 | ⏳ | Headers only |
| OPTIONS | CORS preflight | 8 | ⏳ | Headers only |

### 4.2 Content Type Support
- **Request Content Types**: `application/json`, `multipart/form-data`, `application/x-www-form-urlencoded`
- **Response Content Types**: `application/json`, `text/html`, `text/csv`, `text/plain`, `image/svg+xml`
- **WebSocket**: Binary and text message support

## 5. Test Implementation Structure

### 5.1 Test Organization
```
server/tests/http_acceptance/
├── test_api_contracts.rs          # API schema validation
├── test_dashboard_endpoints.rs    # UI and dashboard routes
├── test_core_api_v1.rs           # Legacy API endpoints
├── test_enhanced_api_v1.rs       # Current API endpoints
├── test_websocket_endpoints.rs   # WebSocket functionality
├── test_authentication.rs        # Auth and authorization
├── test_error_handling.rs        # HTTP error scenarios
├── test_performance.rs           # Load and performance testing
├── test_security.rs              # Security and validation
├── test_cors.rs                  # CORS and cross-origin
├── test_content_negotiation.rs   # Content type handling
└── helpers/
    ├── http_test_client.rs       # HTTP test client
    ├── websocket_test_client.rs  # WebSocket test client
    ├── api_validators.rs         # Response validation helpers
    ├── performance_helpers.rs    # Performance testing utilities
    └── test_data.rs              # Test fixtures and data
```

### 5.2 HTTP Test Client Implementation
```rust
pub struct HttpTestClient {
    base_url: String,
    client: reqwest::Client,
    auth_token: Option<String>,
}

impl HttpTestClient {
    pub async fn new(base_url: &str) -> Result<Self>;
    pub async fn get(&self, path: &str) -> Result<reqwest::Response>;
    pub async fn post(&self, path: &str, body: Value) -> Result<reqwest::Response>;
    pub async fn put(&self, path: &str, body: Value) -> Result<reqwest::Response>;
    pub async fn delete(&self, path: &str) -> Result<reqwest::Response>;
    pub async fn head(&self, path: &str) -> Result<reqwest::Response>;
    pub async fn options(&self, path: &str) -> Result<reqwest::Response>;
    pub fn with_auth_token(mut self, token: String) -> Self;
    pub fn with_header(self, key: &str, value: &str) -> Self;
}
```

### 5.3 WebSocket Test Client Implementation
```rust
pub struct WebSocketTestClient {
    connection: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    url: String,
}

impl WebSocketTestClient {
    pub async fn connect(url: &str) -> Result<Self>;
    pub async fn send_message(&mut self, message: Value) -> Result<()>;
    pub async fn receive_message(&mut self) -> Result<Value>;
    pub async fn close(&mut self) -> Result<()>;
    pub async fn ping(&mut self) -> Result<()>;
}
```

## 6. Test Case Specifications

### 6.1 API Contract Test Template
```rust
#[tokio::test]
async fn test_api_contract_compliance() {
    // Arrange
    let client = HttpTestClient::new("http://localhost:8080").await.unwrap();
    let expected_schema = load_api_schema("analysis_summary.json");
    
    // Act
    let response = client.get("/api/v1/analysis/summary").await.unwrap();
    
    // Assert HTTP Compliance
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
    assert!(response.headers().contains_key("x-response-time"));
    
    // Assert API Contract
    let body: Value = response.json().await.unwrap();
    assert_json_schema_compliance(&body, &expected_schema);
    assert_required_fields_present(&body);
    assert_field_types_correct(&body);
    
    // Assert Performance
    assert_response_time_acceptable(&response, Duration::from_secs(5));
}
```

### 6.2 Critical Test Scenarios

#### 6.2.1 Happy Path Testing
- **Valid Requests**: All endpoints with correct parameters and headers
- **Content Negotiation**: Request different response formats (JSON, CSV, HTML)
- **Parameter Variations**: Test with optional parameters, defaults, and edge values
- **Authentication**: Valid authentication tokens and sessions

#### 6.2.2 Error Path Testing
- **Invalid Endpoints**: Non-existent URLs return proper 404 responses
- **Method Not Allowed**: Unsupported HTTP methods return 405 responses
- **Bad Requests**: Malformed JSON and invalid parameters return 400 responses
- **Authentication Failures**: Missing or invalid tokens return 401 responses
- **Authorization Failures**: Insufficient permissions return 403 responses
- **Rate Limiting**: Excessive requests trigger rate limiting with 429 responses

#### 6.2.3 WebSocket Testing
- **Connection Establishment**: WebSocket handshake and upgrade
- **Message Exchange**: Bidirectional message sending and receiving
- **Connection Persistence**: Long-lived connections and heartbeat
- **Error Handling**: Connection failures and reconnection logic
- **Message Ordering**: Proper ordering of streamed messages

#### 6.2.4 Integration Testing
- **Multi-step Workflows**: Trigger analysis → Poll status → Get results
- **State Management**: Sessions and request correlation across calls
- **File Upload**: Project upload and analysis triggering
- **Download Responses**: Report generation and file downloads

## 7. Performance Requirements

### 7.1 Response Time Targets
| Endpoint Category | Max Response Time | Concurrent Users | Notes |
|-------------------|-------------------|------------------|--------|
| Dashboard/UI | 2 seconds | 100 | Static content caching |
| API Summary | 5 seconds | 50 | Cached analysis results |
| API Analysis | 30 seconds | 20 | Complex analysis operations |
| WebSocket Messages | 1 second | 100 | Real-time streaming |
| File Operations | 10 seconds | 30 | Upload/download operations |

### 7.2 Load Testing Requirements
- **Concurrent Users**: Support 100 concurrent API requests
- **Request Rate**: Handle 1000 requests per minute sustained
- **Memory Usage**: Stay under 4GB during peak load
- **Connection Limits**: Support 500 concurrent WebSocket connections

### 7.3 Performance Testing Scenarios
```rust
#[tokio::test]
async fn test_load_performance() {
    let clients = create_concurrent_clients(100).await;
    let start_time = Instant::now();
    
    // Execute concurrent requests
    let responses = join_all(
        clients.iter().map(|client| client.get("/api/summary"))
    ).await;
    
    let duration = start_time.elapsed();
    
    // Assert all requests succeeded
    for response in responses {
        assert_eq!(response.status(), 200);
        assert!(response.headers().get("x-response-time").is_some());
    }
    
    // Assert performance requirements
    assert!(duration < Duration::from_secs(10));
    assert_memory_usage_within_limits();
}
```

## 8. Security Testing

### 8.1 Input Validation
- **SQL Injection**: Test all string parameters for SQL injection attempts
- **XSS Prevention**: Validate proper HTML escaping in responses
- **Path Traversal**: Prevent file system access outside intended directories
- **JSON Injection**: Validate proper JSON parsing and sanitization

### 8.2 Authentication and Authorization
- **Token Validation**: Verify proper JWT or API key validation
- **Session Management**: Test session creation, validation, and expiration
- **Role-Based Access**: Verify proper authorization for different user roles
- **Rate Limiting**: Test API rate limiting and abuse prevention

### 8.3 HTTPS and Transport Security
- **TLS Configuration**: Verify proper TLS setup and certificate validation
- **HSTS Headers**: Ensure HTTP Strict Transport Security headers
- **Secure Cookies**: Validate secure cookie flags and settings
- **CORS Configuration**: Test Cross-Origin Resource Sharing policies

## 9. API Documentation Testing

### 9.1 OpenAPI Specification
- **Schema Validation**: Ensure API responses match OpenAPI schema
- **Documentation Accuracy**: Verify examples and descriptions are correct
- **Version Consistency**: Ensure API versions match documentation
- **Deprecation Notices**: Validate deprecation warnings and migration guides

### 9.2 Generated Documentation
```yaml
# Example OpenAPI validation test
openapi: 3.0.0
info:
  title: PMAT Analysis API
  version: 2.79.0
paths:
  /api/v1/analysis/summary:
    get:
      responses:
        '200':
          description: Analysis summary
          content:
            application/json:
              schema:
                type: object
                properties:
                  status:
                    type: string
                  results:
                    type: object
```

## 10. Cross-Platform Compatibility

### 10.1 Browser Testing
- **Modern Browsers**: Chrome, Firefox, Safari, Edge
- **JavaScript APIs**: Fetch API, WebSocket API compatibility
- **CORS Handling**: Proper cross-origin request handling
- **WebSocket Support**: Browser WebSocket compatibility

### 10.2 HTTP Client Testing
- **cURL**: Command-line HTTP testing
- **Postman**: API testing tool compatibility
- **Programming Languages**: Python requests, Node.js fetch, etc.
- **Mobile Clients**: iOS and Android HTTP client compatibility

## 11. Monitoring and Observability

### 11.1 Logging Validation
- **Request Logging**: All HTTP requests are properly logged
- **Error Logging**: Errors include sufficient context for debugging
- **Performance Logging**: Response times and resource usage tracked
- **Security Logging**: Authentication failures and suspicious activity logged

### 11.2 Metrics and Health Checks
- **Health Endpoints**: `/health`, `/ready`, `/metrics` endpoints
- **Prometheus Metrics**: Proper metrics format and labels
- **Application Metrics**: Business logic metrics (analysis counts, etc.)
- **Infrastructure Metrics**: CPU, memory, network usage

## 12. Implementation Phases

### 12.1 Phase 1: Core API Testing (Week 1)
- Implement HTTP test client framework
- Test basic endpoints (dashboard, summary, metrics)
- Test error handling for common scenarios
- Basic performance and security testing
- Target: 30% coverage

### 12.2 Phase 2: Advanced API Features (Week 2)
- Test all v1 API endpoints thoroughly
- WebSocket functionality testing
- Authentication and authorization testing
- Content negotiation testing
- Target: 60% coverage

### 12.3 Phase 3: Integration and Performance (Week 3)
- End-to-end workflow testing
- Load testing and performance optimization
- Cross-platform compatibility testing
- Security hardening and testing
- Target: 85% coverage

### 12.4 Phase 4: Production Readiness (Week 4)
- API documentation validation
- Monitoring and observability testing
- Disaster recovery and error handling
- Final security audit and compliance
- Target: 100% coverage

## 13. Continuous Integration

### 13.1 Automated Testing
```bash
# Daily HTTP API acceptance tests
cargo test http_acceptance --release -- --nocapture

# API contract testing with schema validation
cargo test api_contracts --release

# Performance regression testing
cargo test http_performance --release

# Security vulnerability scanning
cargo test http_security --release
```

### 13.2 Quality Gates
- **API Contract Compliance**: All responses must match OpenAPI schema
- **Performance Requirements**: All endpoints must meet response time targets
- **Security Standards**: All security tests must pass
- **Error Handling**: All error scenarios must return appropriate responses

## 14. Success Criteria

### 14.1 Functional Excellence
- **✅ 100% Endpoint Coverage**: Every HTTP endpoint comprehensively tested
- **✅ API Contract Compliance**: All responses match documented schemas
- **✅ Error Handling**: Proper HTTP status codes and error messages
- **✅ WebSocket Functionality**: Real-time streaming works reliably

### 14.2 Performance Excellence
- **✅ Response Time Compliance**: All endpoints meet performance targets
- **✅ Load Handling**: Supports required concurrent user load
- **✅ Resource Efficiency**: Reasonable CPU and memory usage under load
- **✅ Scalability**: Graceful degradation under high load

### 14.3 Security Excellence
- **✅ Input Validation**: All inputs properly validated and sanitized
- **✅ Authentication/Authorization**: Proper access controls implemented
- **✅ Transport Security**: HTTPS and secure communication
- **✅ Attack Resistance**: Resistant to common web attacks

---

**Implementation Status**: ⏳ **PENDING IMPLEMENTATION**
**Target Completion**: Sprint 93 (4 weeks)
**Responsibility**: Development Team + DevOps + Security Team
**Success Metric**: 100% HTTP API coverage with enterprise-grade reliability and security