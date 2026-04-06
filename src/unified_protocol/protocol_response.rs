impl UnifiedResponse {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
            headers: HeaderMap::new(),
            body: Body::empty(),
            trace_id: Uuid::new_v4(),
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn ok() -> Self {
        Self::new(StatusCode::OK)
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_body(mut self, body: Body) -> Self {
        self.body = body;
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_json<T: Serialize>(self, data: &T) -> Result<Self, serde_json::Error> {
        let json = serde_json::to_vec(data)?;
        Ok(self
            .with_body(Body::from(json))
            .with_header("content-type", "application/json"))
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        if let (Ok(name), Ok(val)) = (
            key.parse::<http::HeaderName>(),
            value.parse::<http::HeaderValue>(),
        ) {
            self.headers.insert(name, val);
        }
        self
    }
}

impl IntoResponse for UnifiedResponse {
    fn into_response(self) -> Response {
        let mut response = Response::builder().status(self.status);

        for (key, value) in &self.headers {
            response = response.header(key, value);
        }

        response.body(self.body).unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Failed to build response"))
                .expect("internal error")
        })
    }
}
