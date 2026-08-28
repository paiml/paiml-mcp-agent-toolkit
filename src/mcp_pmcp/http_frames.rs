//! Honest JSON-RPC errors for the streamable-HTTP MCP transport.
//!
//! # Why this exists
//!
//! pmat ships two MCP transports and, until this module, only one of them
//! answered a bad frame truthfully. [`stdio_frames`](crate::mcp_pmcp::stdio_frames)
//! owns the read side of stdio and classifies every line before pmcp sees it
//! (#648); the streamable-HTTP transport added in EV-6 (#999) — in the DEFAULT
//! feature set since 3.32.0 — had no equivalent layer at all. The compliance
//! audit that rated the JSON-RPC transport "protocol compliant, no action"
//! measured stdio only, and could not have found this. The asymmetry is
//! checkable: `grep -cE '32700|32600|32602|32601' tests/e2e_http_serve_t.rs`
//! returned 0 while the same predicate over `stdio_frames.rs`'s test modules
//! returned 24. Nothing in CI could have caught either defect below.
//!
//! Two things were measurably wrong over HTTP, both of them defects pmat had
//! already named and fixed on the other transport:
//!
//! 1. **The `jsonrpc` member was never checked.** `{"jsonrpc":"1.0",…,
//!    "method":"tools/list"}` and `{"jsonrpc":"9.9",…}` were each answered
//!    HTTP 200 with the full tool listing — a reply in a protocol the client
//!    did not ask for. JSON-RPC 2.0 §4 makes a frame whose `jsonrpc` is not
//!    exactly `"2.0"` an Invalid Request whatever else it says.
//! 2. **Every client-side error collapsed to one code and a null id.** Unknown
//!    method, missing `method`, missing `jsonrpc` and rejected params on a
//!    *known* method all came back as HTTP 400 carrying parse-error/`"id":null`,
//!    with the true code surviving only as text inside the message string
//!    ("Protocol error: -32601 - Method not found"). A host that correlates
//!    responses by id never resolves its promise and waits out its own timeout
//!    — precisely the failure `stdio_frames` was written to eliminate, one
//!    transport over.
//!
//! # How
//!
//! This is plumbing, not a second classifier.
//! [`classify_bad_frame`] is already a pure `&[u8] -> FrameVerdict` function;
//! everything here does is get its verdict onto an HTTP response. A second copy
//! of the rules is exactly the drift that let the two transports disagree in
//! the first place, so there is deliberately no version test, no method table
//! and no id handling in this file.
//!
//! pmcp offers no hook that can *answer* a request from the request side, so
//! the guard works in two halves joined by `ServerHttpContext::request_id`:
//!
//! - `on_request` judges the raw body and, when it owes a reply, replaces the
//!   body with [`REFUSED_BODY`] so the tool surface cannot serve a frame that
//!   has already been refused, then parks the rendered frame.
//! - `on_response` recognises pmcp's manufactured parse-failure response and
//!   swaps in the parked frame at HTTP 200, where JSON-RPC application errors
//!   belong.
//!
//! The guard is wired into the transport config in
//! [`crate::mcp_pmcp::http_server`], never into `build_server`: that builder is
//! shared with stdio so the 19-tool surface cannot drift, and stdio already has
//! its own — better placed — classification layer.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use pmcp::error::ErrorCode;
use pmcp::server::http_middleware::{
    ServerHttpContext, ServerHttpMiddleware, ServerHttpMiddlewareChain, ServerHttpRequest,
    ServerHttpResponse,
};

use crate::mcp_pmcp::stdio_frames::{classify_bad_frame, FrameVerdict};

/// The body a judged frame is replaced with before pmcp ever parses it.
///
/// `{}` is chosen, not arbitrary: `pmcp::shared::transport::parse_message`
/// classifies a frame by shape and rejects an object carrying none of `method`,
/// `result` or `error` as "Unknown message type". Substituting it therefore
/// *guarantees* pmcp takes its parse-failure branch, which is the one branch
/// that reaches the response chain without first dispatching to a tool.
///
/// Leaving the original bytes in place is what let `{"jsonrpc":"1.0",…}` be
/// answered 200 with a full `tools/list`: judging a frame and then handing it
/// on unchanged judges nothing.
const REFUSED_BODY: &[u8] = b"{}";

/// How many judged frames may be waiting for their response at once.
///
/// The two hooks share only `ServerHttpContext::request_id`, and not every
/// request that enters `on_request` reaches `on_response`: pmcp runs
/// `validate_headers` between them and returns 406/415 without touching the
/// response chain. An unbounded map would therefore grow by one `Vec` per
/// malformed request from anyone who can reach the port — classification runs
/// before authentication because pmcp's chain does. Bounding the queue makes
/// that leak impossible by construction instead of by remembering to tidy up on
/// every path, which is the kind of promise that survives exactly until the
/// next early return is added upstream.
const PENDING_CAPACITY: usize = 64;

/// The status pmcp gives its manufactured parse-failure response, and the only
/// status this guard will overwrite.
const PARSE_FAILURE_STATUS: u16 = 400;

/// The status a JSON-RPC error frame is served with.
///
/// A JSON-RPC error is an *application* result, not a transport failure: the
/// request was delivered, understood as far as it could be, and answered. MCP
/// clients read the body; answering 400 makes a well-formed JSON-RPC reply look
/// like an HTTP-level rejection and encourages hosts to discard it unread.
const JSONRPC_ERROR_STATUS: u16 = 200;

/// The verdict this transport owes `body`, or `None` when the frame is pmcp's
/// to handle.
///
/// The rule is "pmcp refused the frame, OR its envelope is not JSON-RPC 2.0",
/// and both halves are read off the SAME classifier stdio uses, so the two
/// transports cannot drift apart.
///
/// The envelope half is read as an Invalid Request verdict rather than by
/// re-testing the `jsonrpc` member here, and that is exact rather than a
/// shortcut. [`classify_bad_frame`] reaches Invalid Request in exactly three
/// ways: a wrong or missing `jsonrpc`, a missing or non-string `method`, and a
/// frame that is JSON but not an object. pmcp's own parser already refuses the
/// second and third ("Unknown message type", and a `method` that will not
/// deserialize as a string), so they arrive through the `pmcp_refused` half
/// regardless. For a frame pmcp *accepted*, an Invalid Request verdict can only
/// mean the envelope — which is the one thing pmcp never looks at.
pub(crate) fn http_frame_verdict(body: &[u8]) -> Option<FrameVerdict> {
    let verdict = classify_bad_frame(body);
    let code = match &verdict {
        FrameVerdict::Error { code, .. } => *code,
        // A notification or a response frame. JSON-RPC 2.0 §4.1 forbids
        // answering either, and pmcp already ends both with 202 Accepted.
        FrameVerdict::Silent => return None,
    };
    let envelope_is_wrong = code == ErrorCode::INVALID_REQUEST.as_i32();
    let pmcp_refused = pmcp::shared::transport::parse_message(body).is_err();
    (envelope_is_wrong || pmcp_refused).then_some(verdict)
}

/// Carries each judged frame from `on_request` to `on_response`.
pub(crate) struct JsonRpcFrameGuard {
    pending: Mutex<VecDeque<(String, Vec<u8>)>>,
}

impl JsonRpcFrameGuard {
    /// A guard with an empty hand-off queue.
    pub(crate) fn new() -> Self {
        Self {
            pending: Mutex::new(VecDeque::with_capacity(PENDING_CAPACITY)),
        }
    }

    /// The hand-off queue, recovering from a poisoned lock rather than
    /// propagating it.
    ///
    /// Poisoning means some earlier holder panicked. What is behind this lock is
    /// a hand-off buffer, not an invariant: the worst a recovered guard can do
    /// is answer one request with a stale verdict, whereas re-panicking here
    /// would take down every subsequent request over a fault that had nothing
    /// to do with them.
    fn queue(&self) -> std::sync::MutexGuard<'_, VecDeque<(String, Vec<u8>)>> {
        match self.pending.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    /// Park `frame` until the response for `request_id` comes back.
    fn remember(&self, request_id: &str, frame: Vec<u8>) {
        let mut pending = self.queue();
        // `request_id` comes from the client's own `x-request-id` header when it
        // sends one, so ids are not guaranteed unique. Replacing keeps the
        // newest judgement instead of letting a stale one be handed to a later
        // request that happens to reuse the id.
        pending.retain(|(id, _)| id.as_str() != request_id);
        pending.push_back((request_id.to_string(), frame));
        while pending.len() > PENDING_CAPACITY {
            pending.pop_front();
        }
    }

    /// Remove and return the frame parked for `request_id`, if any.
    fn take(&self, request_id: &str) -> Option<Vec<u8>> {
        let mut pending = self.queue();
        let at = pending
            .iter()
            .position(|(id, _)| id.as_str() == request_id)?;
        pending.remove(at).map(|(_, frame)| frame)
    }

    #[cfg(test)]
    fn parked(&self) -> usize {
        self.queue().len()
    }
}

#[async_trait]
impl ServerHttpMiddleware for JsonRpcFrameGuard {
    async fn on_request(
        &self,
        request: &mut ServerHttpRequest,
        context: &ServerHttpContext,
    ) -> pmcp::Result<()> {
        let verdict = http_frame_verdict(&request.body);
        let Some(frame) = verdict.and_then(|v| v.to_frame_bytes()) else {
            return Ok(());
        };
        // Substitution, not rejection. Returning `Err` here would read as the
        // honest thing to do and would be wrong: pmcp collapses every middleware
        // error into HTTP 500 with an internal-error code, i.e. it would report
        // the client's malformed frame as a server fault. Overwriting the body
        // is what actually stops the tool surface from answering a frame that
        // has already been refused.
        request.body = REFUSED_BODY.to_vec();
        self.remember(&context.request_id, frame);
        Ok(())
    }

    async fn on_response(
        &self,
        response: &mut ServerHttpResponse,
        context: &ServerHttpContext,
    ) -> pmcp::Result<()> {
        // Taken unconditionally, before any decision below: this verdict has had
        // its one chance and must not outlive it whatever we do next.
        let Some(frame) = self.take(&context.request_id) else {
            return Ok(());
        };
        // The substituted body cannot parse, so the response for a judged frame
        // is always pmcp's manufactured parse failure. Checking that rather than
        // trusting the id alone closes the one case where the id is not enough:
        // `x-request-id` is client-supplied, so two concurrent requests may
        // share one, and a served result must never be replaced by an error.
        if response.status.as_u16() != PARSE_FAILURE_STATUS {
            return Ok(());
        }
        response.status = JSONRPC_ERROR_STATUS
            .try_into()
            .expect("200 is a valid HTTP status code");
        // pmcp builds its parse-failure response with an EMPTY header map, so
        // without this the frame goes out untyped and a strict client is
        // entitled to refuse to read it as JSON. Content-length is deliberately
        // NOT set here: pmcp derives it from the final body after this hook
        // runs, and a second, hand-computed copy could only ever disagree.
        response.add_header("content-type", "application/json");
        response.body = frame;
        Ok(())
    }

    /// Runs before anything else in the chain.
    ///
    /// Classification reads the body as the client sent it. Any middleware that
    /// rewrites a request must not get there first, or the guard would judge
    /// bytes the client never wrote.
    fn priority(&self) -> i32 {
        10
    }
}

/// The middleware chain to hand `StreamableHttpServerConfig::http_middleware`.
pub(crate) fn frame_guard_chain() -> Arc<ServerHttpMiddlewareChain> {
    let mut chain = ServerHttpMiddlewareChain::new();
    chain.add(Arc::new(JsonRpcFrameGuard::new()));
    Arc::new(chain)
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    /// Frames the HTTP transport must refuse, and what it owes each one.
    ///
    /// The same table drives `tests/e2e_http_serve_t.rs`, which asserts the
    /// codes and ids come back over a real socket from the shipped binary.
    /// Here the claim is narrower and sharper: whatever the classifier decides,
    /// this transport decides the same thing.
    const REFUSED_FRAMES: &[&str] = &[
        // The envelope defect pmcp cannot see: these parse, and were served.
        r#"{"jsonrpc":"1.0","id":1,"method":"tools/list","params":{}}"#,
        r#"{"jsonrpc":"9.9","id":"a","method":"tools/list","params":{}}"#,
        r#"{"id":2,"method":"tools/list","params":{}}"#,
        // The collapsed-code defects: each of these used to be one parse error.
        r#"{"jsonrpc":"2.0","id":3,"method":"no/such/method","params":{}}"#,
        r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":null}"#,
        r#"{"jsonrpc":"2.0","id":5}"#,
        r#"{"jsonrpc":"2.0","id":6,"method":"tools/list""#,
    ];

    /// Build a request without naming `http`'s types.
    ///
    /// pmat's `http` dependency is optional and is NOT enabled by `mcp-http`
    /// (only by `demo` and `unified-protocol`), so `Method`, `Uri`, `HeaderMap`
    /// and `StatusCode` cannot be spelled anywhere in this file. Every one of
    /// them implements `Default` — GET, `/`, empty and 200 OK respectively —
    /// and inference takes the type from the field each value lands in.
    fn request(body: &str) -> ServerHttpRequest {
        ServerHttpRequest::new(
            Default::default(),
            Default::default(),
            Default::default(),
            body.as_bytes().to_vec(),
        )
    }

    /// pmcp's manufactured parse-failure response: 400, no headers.
    fn parse_failure_response() -> ServerHttpResponse {
        let mut response = ServerHttpResponse::new(
            Default::default(),
            Default::default(),
            br#"{"error":"Invalid JSON: Unknown message type"}"#.to_vec(),
        );
        response.status = PARSE_FAILURE_STATUS
            .try_into()
            .expect("400 is a valid HTTP status code");
        response
    }

    /// The `(code, id)` pair a rendered frame carries — the two things a host
    /// needs and the two things the HTTP transport used to get wrong.
    fn code_and_id(frame: &[u8]) -> (i64, serde_json::Value) {
        let text = String::from_utf8_lossy(frame).into_owned();
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| format!("{e}: {text}"))
            .expect("the guard must emit valid JSON");
        let code = parsed
            .pointer("/error/code")
            .and_then(serde_json::Value::as_i64)
            .ok_or(text)
            .expect("the guard must emit a JSON-RPC error object carrying a code");
        (code, parsed.pointer("/id").cloned().unwrap_or_default())
    }

    /// The whole point of the module: one classifier, two transports. If these
    /// two ever disagree, the HTTP transport has grown a second opinion.
    #[test]
    fn every_refused_frame_gets_the_verdict_the_stdio_classifier_gives_it() {
        for frame in REFUSED_FRAMES {
            assert_eq!(
                http_frame_verdict(frame.as_bytes()),
                Some(classify_bad_frame(frame.as_bytes())),
                "the HTTP transport must owe `{frame}` exactly what stdio owes it"
            );
        }
    }

    /// The two defects, stated as codes rather than as "not what it used to be".
    #[test]
    fn a_wrong_jsonrpc_version_is_invalid_request_with_the_id_echoed() {
        let verdict = http_frame_verdict(REFUSED_FRAMES[0].as_bytes())
            .expect("a frame declaring jsonrpc 1.0 must not reach the tool surface");
        let bytes = verdict.to_frame_bytes().expect("an error renders a frame");
        assert_eq!(
            code_and_id(&bytes),
            (
                i64::from(ErrorCode::INVALID_REQUEST.as_i32()),
                serde_json::json!(1)
            ),
            "jsonrpc 1.0 is an Invalid Request, and the client's id must come back"
        );
    }

    #[test]
    fn an_unknown_method_is_method_not_found_not_a_parse_error() {
        let verdict = http_frame_verdict(REFUSED_FRAMES[3].as_bytes())
            .expect("an unknown method must be refused");
        let bytes = verdict.to_frame_bytes().expect("an error renders a frame");
        assert_eq!(
            code_and_id(&bytes),
            (
                i64::from(ErrorCode::METHOD_NOT_FOUND.as_i32()),
                serde_json::json!(3)
            ),
        );
    }

    #[test]
    fn rejected_params_on_a_known_method_are_invalid_params() {
        let verdict = http_frame_verdict(REFUSED_FRAMES[4].as_bytes())
            .expect("tools/call with null params must be refused");
        let bytes = verdict.to_frame_bytes().expect("an error renders a frame");
        assert_eq!(
            code_and_id(&bytes),
            (
                i64::from(ErrorCode::INVALID_PARAMS.as_i32()),
                serde_json::json!(4)
            ),
            "tools/call exists; blaming the method for bad params points the \
             operator at the wrong defect"
        );
    }

    #[test]
    fn unparseable_bytes_are_a_parse_error_with_a_null_id() {
        let verdict = http_frame_verdict(REFUSED_FRAMES[6].as_bytes())
            .expect("a truncated frame must be refused");
        let bytes = verdict.to_frame_bytes().expect("an error renders a frame");
        assert_eq!(
            code_and_id(&bytes),
            (
                i64::from(ErrorCode::PARSE_ERROR.as_i32()),
                serde_json::Value::Null
            ),
            "a null id is correct ONLY here, where there is no id to recover"
        );
    }

    /// The guard must not become a second gatekeeper. A frame pmcp accepts is
    /// pmcp's to answer.
    #[test]
    fn a_well_formed_request_is_left_for_the_tool_surface() {
        assert_eq!(
            http_frame_verdict(br#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#),
            None,
            "the guard must not intercept a request the server can serve"
        );
    }

    /// JSON-RPC 2.0 §4.1: neither of these is ours to answer, wrong version or
    /// not. pmcp ends both with 202 Accepted and must keep doing so.
    #[test]
    fn notifications_and_response_frames_are_not_answered() {
        for frame in [
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
            r#"{"jsonrpc":"1.0","method":"notifications/initialized"}"#,
            r#"{"jsonrpc":"2.0","id":1,"result":{}}"#,
        ] {
            assert_eq!(
                http_frame_verdict(frame.as_bytes()),
                None,
                "`{frame}` carries no id to answer to"
            );
        }
    }

    /// The substitution is the half that stops the 200-with-a-full-tool-listing.
    #[tokio::test]
    async fn a_judged_frame_never_reaches_pmcp_with_its_original_bytes() {
        let guard = JsonRpcFrameGuard::new();
        let context = ServerHttpContext::new("req-1".to_string());
        let mut req = request(REFUSED_FRAMES[0]);
        guard
            .on_request(&mut req, &context)
            .await
            .expect("the guard must not fail the request");
        assert_eq!(
            req.body, REFUSED_BODY,
            "the original bytes must not survive into pmcp, or it will serve them"
        );
        assert_eq!(
            guard.parked(),
            1,
            "the verdict must be waiting for the response"
        );
    }

    #[tokio::test]
    async fn the_verdict_replaces_the_parse_failure_response_at_200() {
        let guard = JsonRpcFrameGuard::new();
        let context = ServerHttpContext::new("req-2".to_string());
        let mut req = request(REFUSED_FRAMES[3]);
        guard
            .on_request(&mut req, &context)
            .await
            .expect("on_request");

        let mut response = parse_failure_response();
        guard
            .on_response(&mut response, &context)
            .await
            .expect("on_response");

        assert_eq!(
            response.status.as_u16(),
            JSONRPC_ERROR_STATUS,
            "a JSON-RPC application error is not an HTTP-level rejection"
        );
        assert_eq!(
            code_and_id(&response.body),
            (
                i64::from(ErrorCode::METHOD_NOT_FOUND.as_i32()),
                serde_json::json!(3)
            ),
        );
        assert_eq!(
            response.get_header("content-type"),
            Some("application/json")
        );
        assert_eq!(
            guard.parked(),
            0,
            "the verdict must not outlive its response"
        );
    }

    /// `x-request-id` is client-supplied, so two live requests can share one.
    /// A served result must never be overwritten by somebody else's error.
    #[tokio::test]
    async fn a_response_that_is_not_the_parse_failure_is_left_untouched() {
        let guard = JsonRpcFrameGuard::new();
        let context = ServerHttpContext::new("shared-id".to_string());
        let mut req = request(REFUSED_FRAMES[0]);
        guard
            .on_request(&mut req, &context)
            .await
            .expect("on_request");

        let served = br#"{"jsonrpc":"2.0","id":1,"result":{"tools":[]}}"#.to_vec();
        let mut response =
            ServerHttpResponse::new(Default::default(), Default::default(), served.clone());
        guard
            .on_response(&mut response, &context)
            .await
            .expect("on_response");

        assert_eq!(
            response.body, served,
            "a served result must not be rewritten"
        );
        assert_eq!(
            guard.parked(),
            0,
            "the verdict must be dropped even when it is not used, or it leaks"
        );
    }

    /// pmcp returns 406/415 from header validation *between* the two hooks, so
    /// a request can enter `on_request` and never reach `on_response`. Anyone
    /// who can reach the port can send those, unauthenticated.
    #[tokio::test]
    async fn the_hand_off_queue_cannot_grow_without_bound() {
        let guard = JsonRpcFrameGuard::new();
        for n in 0..PENDING_CAPACITY * 4 {
            let context = ServerHttpContext::new(format!("req-{n}"));
            let mut req = request(REFUSED_FRAMES[0]);
            guard
                .on_request(&mut req, &context)
                .await
                .expect("on_request");
        }
        assert_eq!(
            guard.parked(),
            PENDING_CAPACITY,
            "a response that never arrives must cost a bounded amount of memory"
        );
    }

    #[tokio::test]
    async fn a_reused_request_id_replaces_rather_than_accumulates() {
        let guard = JsonRpcFrameGuard::new();
        let context = ServerHttpContext::new("same".to_string());
        for frame in [REFUSED_FRAMES[0], REFUSED_FRAMES[3]] {
            let mut req = request(frame);
            guard
                .on_request(&mut req, &context)
                .await
                .expect("on_request");
        }
        assert_eq!(guard.parked(), 1, "one id must hold at most one verdict");

        let mut response = parse_failure_response();
        guard
            .on_response(&mut response, &context)
            .await
            .expect("on_response");
        assert_eq!(
            code_and_id(&response.body).0,
            i64::from(ErrorCode::METHOD_NOT_FOUND.as_i32()),
            "the newest judgement wins, not the stalest"
        );
    }

    /// The chain is what `serve` actually installs; an empty one would make
    /// every assertion above unreachable in production.
    #[test]
    fn the_installed_chain_is_not_empty() {
        assert!(
            format!("{:?}", frame_guard_chain()).contains("middleware_count: 1"),
            "the guard must actually be in the chain handed to pmcp"
        );
    }
}
