// Helper functions for HTTP request parsing and routing

fn parse_http_request(raw: &[u8]) -> Result<HttpRequest, ProtocolError> {
    debug_assert!(!raw.is_empty(), "raw must not be empty");
    let request_str = String::from_utf8_lossy(raw);
    let lines: Vec<&str> = request_str.lines().collect();

    validate_request_lines(&lines)?;
    let (method, path) = parse_request_line(lines[0])?;
    let (headers, body_start) = parse_headers(&lines);
    let body = parse_body(&lines, body_start);

    Ok(HttpRequest {
        method,
        path,
        headers,
        body,
    })
}

fn validate_request_lines(lines: &[&str]) -> Result<(), ProtocolError> {
    debug_assert!(!lines.is_empty(), "lines must not be empty");
    if lines.is_empty() {
        Err(ProtocolError::InvalidParams("Empty request".to_string()))
    } else {
        Ok(())
    }
}

fn parse_request_line(line: &str) -> Result<(String, String), ProtocolError> {
    debug_assert!(!line.is_empty(), "line must not be empty");
    let request_line: Vec<&str> = line.split_whitespace().collect();

    if request_line.len() < 2 {
        return Err(ProtocolError::InvalidParams(
            "Invalid request line".to_string(),
        ));
    }

    Ok((request_line[0].to_string(), request_line[1].to_string()))
}

fn parse_headers(lines: &[&str]) -> (HashMap<String, String>, usize) {
    debug_assert!(!lines.is_empty(), "lines must not be empty");
    let mut headers = HashMap::new();
    let mut body_start = 0;

    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.is_empty() {
            body_start = i + 1;
            break;
        }

        if let Some((key, value)) = line.split_once(": ") {
            headers.insert(key.to_string(), value.to_string());
        }
    }

    (headers, body_start)
}

fn parse_body(lines: &[&str], body_start: usize) -> Value {
    debug_assert!(!lines.is_empty(), "lines must not be empty");
    if body_start >= lines.len() {
        return Value::Null;
    }

    let body_str = lines[body_start..].join("\n");
    serde_json::from_str(&body_str).unwrap_or(Value::Null)
}

fn route_to_operation(path: &str, method: &str) -> Result<Operation, ProtocolError> {
    debug_assert!(!path.is_empty(), "path must not be empty");
    debug_assert!(!method.is_empty(), "method must not be empty");
    match (method, path) {
        ("GET" | "POST", "/analyze/complexity") => {
            Ok(Operation::AnalyzeComplexity(ComplexityParams {
                file_path: None,
                max_cyclomatic: None,
                max_cognitive: None,
            }))
        }
        ("GET" | "POST", "/analyze/satd") => Ok(Operation::AnalyzeSatd(SatdParams {
            file_path: None,
            strict: false,
        })),
        ("GET" | "POST", "/analyze/dead-code") => Ok(Operation::AnalyzeDeadCode(DeadCodeParams {
            file_path: None,
            include_tests: false,
        })),
        ("POST", "/quality/gate") => Ok(Operation::QualityGate(QualityGateParams {
            file_path: None,
            fail_on_violation: false,
        })),
        _ => Err(ProtocolError::UnknownMethod(format!("{method} {path}"))),
    }
}
