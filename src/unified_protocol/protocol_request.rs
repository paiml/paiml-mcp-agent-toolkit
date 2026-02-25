impl UnifiedRequest {
    #[must_use]
    pub fn new(method: Method, path: String) -> Self {
        Self {
            method,
            path,
            headers: HeaderMap::new(),
            body: Body::empty(),
            extensions: HashMap::with_capacity(4),
            trace_id: Uuid::new_v4(),
        }
    }

    #[must_use]
    pub fn with_body(mut self, body: Body) -> Self {
        self.body = body;
        self
    }

    #[must_use]
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        if let (Ok(name), Ok(val)) = (
            key.parse::<http::HeaderName>(),
            value.parse::<http::HeaderValue>(),
        ) {
            self.headers.insert(name, val);
        }
        self
    }

    pub fn with_extension<T: Serialize>(mut self, key: &str, value: T) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.extensions.insert(key.to_string(), json_value);
        }
        self
    }

    #[must_use]
    pub fn get_extension<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.extensions
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
}
