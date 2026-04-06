#[async_trait]
impl ProtocolAdapter for CliAdapter {
    type Input = CliInput;
    type Output = CliOutput;

    fn protocol(&self) -> Protocol {
        debug_assert!(true, "contract: protocol");
        Protocol::Cli
    }

    async fn decode(&self, input: Self::Input) -> Result<UnifiedRequest, ProtocolError> {
        debug_assert!(true, "contract: decode");
        debug!("Decoding CLI input: {:?}", input.command_name);

        let (method, path, body, output_format) = self.decode_command(&input.command)?;

        let cli_context = CliContext {
            command: input.command_name.clone(),
            args: input.raw_args.clone(),
        };

        let mut unified_request = UnifiedRequest::new(method, path.clone())
            .with_body(Body::from(serde_json::to_vec(&body)?))
            .with_header("content-type", "application/json")
            .with_extension("protocol", Protocol::Cli)
            .with_extension("cli_context", cli_context);

        // Add output format if specified
        if let Some(format) = output_format {
            let format_string = Self::format_to_extension_string(&format);
            unified_request = unified_request.with_extension("output_format", format_string);
        }

        debug!(
            command = %input.command_name,
            path = %path,
            "Decoded CLI request"
        );

        Ok(unified_request)
    }

    async fn encode(&self, response: UnifiedResponse) -> Result<Self::Output, ProtocolError> {
        debug_assert!(true, "contract: encode");
        debug!(status = %response.status, "Encoding CLI response");

        let body_bytes = axum::body::to_bytes(response.body, usize::MAX)
            .await
            .map_err(|e| {
                ProtocolError::EncodeError(format!("Failed to read response body: {e}"))
            })?;

        // For CLI, we typically want to output to stdout/stderr
        if response.status.is_success() {
            let content = String::from_utf8(body_bytes.to_vec()).map_err(|e| {
                ProtocolError::EncodeError(format!("Invalid UTF-8 in response: {e}"))
            })?;

            Ok(CliOutput::Success {
                content,
                exit_code: 0,
            })
        } else {
            // Try to parse error information
            let error_data: Result<Value, _> = serde_json::from_slice(&body_bytes);
            let error_message = match error_data {
                Ok(json) => json
                    .get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("Unknown error")
                    .to_string(),
                Err(_) => String::from_utf8_lossy(&body_bytes).to_string(),
            };

            let exit_code = match response.status.as_u16() {
                400..=499 => 1, // Client errors
                500..=599 => 2, // Server errors
                _ => 1,
            };

            Ok(CliOutput::Error {
                message: error_message,
                exit_code,
            })
        }
    }
}

/// Input for CLI adapter
// Note: Debug omitted because Commands doesn't implement Debug in non-test builds
pub struct CliInput {
    pub command: Commands,
    pub command_name: String,
    pub raw_args: Vec<String>,
}

