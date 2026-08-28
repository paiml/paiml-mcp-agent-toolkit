//! Streamable-HTTP MCP transport (EV-6, #999).
//!
//! Serves the same tool surface as the stdio server — `build_server` is shared,
//! so "which tools does pmat advertise" keeps one answer — over pmcp's
//! `StreamableHttpServer`.
//!
//! # Why streamable HTTP specifically
//!
//! The Google Antigravity Agent registers remote MCP servers as
//! `{"type": "mcp_server", "name": ..., "url": ...}` and supports **streamable
//! HTTP only** — SSE is explicitly unsupported — so stdio cannot reach it. Its
//! egress proxy also blocks loopback, so the endpoint has to be a reachable
//! allowlisted host rather than `localhost`.
//!
//! # It fails closed
//!
//! [`serve`] REFUSES TO START without a token. That is deliberate and it is the
//! whole point of the module: pmcp's HTTP layer calls
//! `server.get_auth_provider()` and, when there is none, falls through to
//! `extract_auth_from_proxy_headers` and returns `Ok(None)` — i.e. it serves
//! every request. A naive wiring would satisfy the sentence "unauthenticated
//! request → 401" in a ticket while doing the opposite in production, because
//! nothing would ever be unauthenticated. Requiring the token at construction
//! makes that state unreachable rather than merely discouraged.
//!
//! # It answers bad frames honestly
//!
//! The transport config installs
//! [`http_frames::frame_guard_chain`](crate::mcp_pmcp::http_frames::frame_guard_chain).
//! Without it this transport had no JSON-RPC error layer at all: pmcp classifies
//! a frame by shape and never reads the `jsonrpc` member, so a frame declaring
//! version 1.0 was answered 200 with the full tool listing, and every
//! client-side fault came back as one parse error with `"id":null`. stdio has
//! had that layer since #648 and the guard reuses its classifier, so the two
//! transports answer the same bytes the same way.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{bail, Result};
use pmcp::server::auth::{AuthContext, AuthProvider};

/// Environment variable carrying the bearer token.
///
/// A token belongs in the environment, not in a flag: an argument is visible in
/// `ps` and lands in shell history. On Antigravity the bearer is injected at the
/// egress proxy via `network.allowlist[].transform`, so it never sits inside the
/// sandbox at all — that is the sovereign-safe shape and this variable is the
/// server's half of it.
pub const TOKEN_ENV: &str = "PMAT_MCP_HTTP_TOKEN";

/// Shortest token this will accept.
///
/// Not security theatre: the failure mode being prevented is a deployment that
/// sets `PMAT_MCP_HTTP_TOKEN=x`, passes its smoke test, and is effectively open.
///
/// Public so the help text and the generated-token path state the SAME floor
/// this constructor enforces. `serve --help` used to describe the minimum in
/// prose, which is how a documented number drifts from an enforced one.
pub const MIN_TOKEN_LEN: usize = 16;

/// Bearer-token auth over a single shared secret.
#[derive(Clone)]
pub struct BearerToken {
    token: Arc<String>,
}

impl BearerToken {
    /// Reject a token too short to be worth having.
    pub fn new(token: impl Into<String>) -> Result<Self> {
        let token = token.into();
        if token.len() < MIN_TOKEN_LEN {
            bail!(
                "{TOKEN_ENV} must be at least {MIN_TOKEN_LEN} characters; got {}",
                token.len()
            );
        }
        Ok(Self {
            token: Arc::new(token),
        })
    }

    /// Read the token from the environment.
    pub fn from_env() -> Result<Self> {
        match std::env::var(TOKEN_ENV) {
            Ok(t) => Self::new(t),
            // Exit code DECLARED, not inferred. This message used to reach
            // `ExitCode::ConfigurationError` (4) because the substring "config"
            // appears in "no auth provider is configured" — a clause about
            // pmcp's behaviour, in a message about a missing token. The help at
            // commands_enum/definition.rs:933 documents exit 4 as a contract,
            // so the code is kept and now stated here, where it cannot be
            // changed by editing prose.
            Err(_) => Err(crate::cli_exit::configuration_error(anyhow::anyhow!(
                "{TOKEN_ENV} is not set. `pmat serve` will not start an unauthenticated MCP \
                 endpoint: pmcp serves every request when no auth provider is configured, so \
                 starting without a token would publish the full tool surface to anyone who \
                 can reach the port."
            ))),
        }
    }

    /// Constant-time comparison.
    ///
    /// `==` on `String` short-circuits at the first differing byte, which leaks
    /// a matching prefix to anyone who can time the response. The cost here is
    /// nil and the alternative is a real, if unglamorous, oracle.
    fn matches(&self, candidate: &str) -> bool {
        let expected = self.token.as_bytes();
        let got = candidate.as_bytes();
        if expected.len() != got.len() {
            return false;
        }
        let mut diff = 0u8;
        for (a, b) in expected.iter().zip(got.iter()) {
            diff |= a ^ b;
        }
        diff == 0
    }
}

/// Redacted on purpose. `#[derive(Debug)]` would print the shared secret into
/// any log line, panic message or `expect` that formats this value — and the
/// test below formats it.
impl std::fmt::Debug for BearerToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BearerToken")
            .field("token", &"<redacted>")
            .finish()
    }
}

#[async_trait::async_trait]
impl AuthProvider for BearerToken {
    async fn validate_request(
        &self,
        authorization_header: Option<&str>,
    ) -> pmcp::error::Result<Option<AuthContext>> {
        // `None` here means "no credentials", and pmcp turns a validation error
        // into 401. Returning Ok(None) instead would be a pass.
        let header = authorization_header.ok_or_else(|| {
            pmcp::error::Error::Internal("missing Authorization header".to_string())
        })?;
        let presented = header.strip_prefix("Bearer ").ok_or_else(|| {
            pmcp::error::Error::Internal("Authorization header is not a Bearer token".to_string())
        })?;
        if !self.matches(presented.trim()) {
            return Err(pmcp::error::Error::Internal(
                "invalid bearer token".to_string(),
            ));
        }
        Ok(Some(AuthContext {
            subject: "pmat-mcp-http".to_string(),
            scopes: vec![],
            ..AuthContext::default()
        }))
    }

    fn auth_scheme(&self) -> &'static str {
        "Bearer"
    }

    fn is_required(&self) -> bool {
        true
    }
}

/// Largest request body this transport will read, in bytes (4 MiB).
///
/// The cap itself is not new — pmcp has always enforced one before any JSON
/// parsing, which is why the unbounded-body hypothesis about this transport is
/// false. What is new is that pmat states the number. It used to arrive through
/// `..Default::default()`, so the only place it existed was pmcp's `limits.rs`:
/// a limit pmat depends on, cannot name, and would not notice changing under
/// it. Naming it here also means a pmcp bump cannot silently move it.
#[cfg(feature = "mcp-http")]
pub const MAX_REQUEST_BYTES: usize = 4 * 1024 * 1024;

/// The transport configuration, separated from [`serve`] so it can be asserted
/// on without binding a socket.
///
/// Two of its four fields are load-bearing and neither is visible from the
/// outside until something goes wrong, which is why they are pinned by a test
/// rather than left to a reading of this function.
#[cfg(feature = "mcp-http")]
fn transport_config() -> pmcp::server::streamable_http_server::StreamableHttpServerConfig {
    pmcp::server::streamable_http_server::StreamableHttpServerConfig {
        session_id_generator: None, // stateless
        enable_json_response: true,
        // Without this pmat has NO JSON-RPC error layer on HTTP. pmcp classifies
        // a frame by shape and never reads the `jsonrpc` member, so
        // `{"jsonrpc":"1.0",…,"method":"tools/list"}` was answered 200 with the
        // full tool listing, and every client-side fault — unknown method,
        // missing `method`, rejected params on a known method — collapsed into
        // one parse error with `"id":null`, leaving hosts that correlate by id
        // waiting out their own timeouts. stdio has had this layer since #648;
        // the guard reuses its classifier rather than adding a second one.
        //
        // It belongs HERE and not in `build_server`: that builder is shared with
        // stdio precisely so the tool surface cannot drift, and stdio does its
        // classification lower down, where it still holds the raw line.
        http_middleware: Some(crate::mcp_pmcp::http_frames::frame_guard_chain()),
        // Stated rather than inherited. This cap is real — it is enforced before
        // any JSON parsing — but it arrived through `..Default::default()`, so
        // the only place the number existed was pmcp's `limits.rs`. A limit pmat
        // relies on and cannot name is a limit pmat cannot be said to have
        // chosen.
        max_request_bytes: MAX_REQUEST_BYTES,
        ..Default::default()
    }
}

/// Start the streamable-HTTP MCP endpoint. Returns the bound address and the
/// server task.
///
/// Stateless by construction (`session_id_generator: None`): MCP revision
/// 2026-07-28 removes the initialize/initialized handshake and
/// `Mcp-Session-Id` entirely, and server-held session state is the pattern it
/// dropped. Starting stateless means the transport does not have to be
/// re-architected when pmat moves to that revision.
#[cfg(feature = "mcp-http")]
pub async fn serve(
    addr: SocketAddr,
    auth: BearerToken,
) -> Result<(SocketAddr, tokio::task::JoinHandle<()>)> {
    use pmcp::server::streamable_http_server::StreamableHttpServer;

    let provider: Arc<dyn AuthProvider> = Arc::new(auth);
    let server =
        crate::mcp_pmcp::simple_unified_server::SimpleUnifiedServer::build_server(Some(provider))
            .map_err(|e| anyhow::anyhow!("building the MCP tool surface failed: {e}"))?;

    let http = StreamableHttpServer::with_config(
        addr,
        Arc::new(tokio::sync::Mutex::new(server)),
        transport_config(),
    );
    http.start()
        .await
        .map_err(|e| anyhow::anyhow!("starting the streamable-HTTP MCP server failed: {e}"))
}

/// The build without the feature: say so rather than pretend.
#[cfg(not(feature = "mcp-http"))]
pub async fn serve(
    _addr: SocketAddr,
    _auth: BearerToken,
) -> Result<(SocketAddr, tokio::task::JoinHandle<()>)> {
    bail!(
        "this pmat was built without the `mcp-http` feature, so the streamable-HTTP MCP \
         transport is not compiled in. Rebuild with `--features mcp-http`."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_short_token_is_refused() {
        assert!(
            BearerToken::new("tooshort").is_err(),
            "a token short enough to guess must not be accepted"
        );
        assert!(BearerToken::new("0123456789abcdef").is_ok());
    }

    /// The invariant: no token, no server. Asserted because the failure mode is
    /// silent — pmcp serves every request when no auth provider is configured,
    /// so a `serve` that started without one would look healthy and be open.
    #[test]
    fn without_a_token_the_server_refuses_to_start() {
        let saved = std::env::var(TOKEN_ENV).ok();
        std::env::remove_var(TOKEN_ENV);
        let got = BearerToken::from_env();
        if let Some(v) = saved {
            std::env::set_var(TOKEN_ENV, v);
        }
        let err = got.expect_err("must refuse without a token").to_string();
        assert!(
            err.contains("will not start an unauthenticated MCP endpoint"),
            "the error must say why, got: {err}"
        );
    }

    #[tokio::test]
    async fn a_request_without_credentials_is_rejected() {
        let auth = BearerToken::new("0123456789abcdef0123").expect("token");
        assert!(
            auth.validate_request(None).await.is_err(),
            "no Authorization header must not authenticate"
        );
        assert!(
            auth.validate_request(Some("Basic 0123456789abcdef0123"))
                .await
                .is_err(),
            "a non-Bearer scheme must not authenticate"
        );
        assert!(
            auth.validate_request(Some("Bearer wrongwrongwrongwrong"))
                .await
                .is_err(),
            "a wrong token must not authenticate"
        );
        assert!(
            auth.validate_request(Some("Bearer 0123456789abcdef0123"))
                .await
                .expect("valid token")
                .is_some(),
            "the correct token must authenticate"
        );
    }

    /// A prefix match must not pass, and must not be distinguishable by length.
    #[test]
    fn comparison_is_length_checked_and_constant_time() {
        let auth = BearerToken::new("0123456789abcdef0123").expect("token");
        assert!(
            !auth.matches("0123456789abcdef012"),
            "prefix must not match"
        );
        assert!(
            !auth.matches("0123456789abcdef01234"),
            "suffix must not match"
        );
        assert!(auth.matches("0123456789abcdef0123"));
    }

    #[test]
    fn auth_is_required_and_advertises_bearer() {
        let auth = BearerToken::new("0123456789abcdef0123").expect("token");
        assert!(auth.is_required(), "optional auth is not auth");
        assert_eq!(auth.auth_scheme(), "Bearer");
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(all(test, feature = "mcp-http"))]
mod transport_config_tests {
    //! The config literal is where the HTTP transport's protocol behaviour is
    //! actually decided, and every field in it fails silently: a missing
    //! middleware chain does not error, it just serves malformed frames as if
    //! they were fine. So the fields are asserted rather than read.
    use super::*;

    /// Without a chain here, pmat has NO JSON-RPC error layer on HTTP —
    /// `..Default::default()` supplies `http_middleware: None`, which is how
    /// `{"jsonrpc":"1.0",…}` came to be answered 200 with the full tool listing.
    #[test]
    fn the_frame_guard_is_installed_and_is_not_an_empty_chain() {
        let config = transport_config();
        let chain = config
            .http_middleware
            .expect("the HTTP transport must carry the JSON-RPC frame guard");
        assert!(
            format!("{chain:?}").contains("middleware_count: 1"),
            "an empty chain is wired-but-inert; got {chain:?}"
        );
    }

    /// Stated, not inherited. The cap is enforced by pmcp before any parsing,
    /// but until it was written down here the value lived only in pmcp.
    #[test]
    fn the_request_body_cap_is_named_by_pmat() {
        assert_eq!(transport_config().max_request_bytes, MAX_REQUEST_BYTES);
        assert_eq!(
            MAX_REQUEST_BYTES,
            4 * 1024 * 1024,
            "changing the cap is a deliberate act; say so here"
        );
    }

    /// Both of these were already true and both are load-bearing: sessions are
    /// the pattern MCP 2026-07-28 drops, and SSE responses would defeat the
    /// JSON body every assertion in this module reads.
    #[test]
    fn the_transport_stays_stateless_and_answers_with_json() {
        let config = transport_config();
        assert!(config.session_id_generator.is_none());
        assert!(config.enable_json_response);
    }
}

#[cfg(all(test, feature = "mcp-http"))]
mod http_e2e_tests {
    //! The invariant EV-6 exists for, asserted against a REAL bound socket
    //! rather than against the provider in isolation: pmcp only consults the
    //! auth provider if one is wired, and the whole hazard is a wiring that
    //! looks right and serves open.
    use super::*;

    async fn start() -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
        let auth = BearerToken::new("0123456789abcdef0123").expect("token");
        serve("127.0.0.1:0".parse().expect("addr"), auth)
            .await
            .expect("server starts")
    }

    #[tokio::test]
    async fn unauthenticated_request_is_401_and_a_valid_token_is_not() {
        let (addr, handle) = start().await;
        // pmcp mounts the MCP endpoint at the ROOT of the server it starts
        // (`build_mcp_router` routes POST/GET/DELETE on "/"), not at /mcp. An
        // earlier version of this test used /mcp, got 404, and would have
        // "passed" a 401 assertion against a path that does not exist.
        let url = format!("http://{addr}/");
        let client = reqwest::Client::new();
        // Streamable HTTP negotiates content before it authenticates: without
        // this the server answers 406 and the 401 assertion below would be
        // testing content negotiation, not auth.
        const ACCEPT: &str = "application/json, text/event-stream";
        let body = serde_json::json!({
            "jsonrpc":"2.0","id":1,"method":"tools/list","params":{}
        });

        let no_auth = client
            .post(&url)
            .header("Accept", ACCEPT)
            .json(&body)
            .send()
            .await
            .expect("request");
        assert_eq!(
            no_auth.status().as_u16(),
            401,
            "an unauthenticated request must be refused, not served"
        );

        let bad = client
            .post(&url)
            .header("Accept", ACCEPT)
            .header("Authorization", "Bearer wrongwrongwrongwrong")
            .json(&body)
            .send()
            .await
            .expect("request");
        assert_eq!(bad.status().as_u16(), 401, "a wrong token must be refused");

        let good = client
            .post(&url)
            .header("Accept", ACCEPT)
            .header("Authorization", "Bearer 0123456789abcdef0123")
            .json(&body)
            .send()
            .await
            .expect("request");
        assert!(
            good.status().is_success(),
            "the correct token must be served, got {}",
            good.status()
        );
        handle.abort();
    }
}
