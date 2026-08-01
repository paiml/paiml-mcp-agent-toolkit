//! Raw-line stdio transport that answers unparseable frames honestly.
//!
//! # Why this exists (defect #648, round 2)
//!
//! pmcp's [`StdioTransport`](pmcp::shared::StdioTransport) reads a line, hands
//! it to `parse_message`, and — if that fails — throws the raw bytes away and
//! returns a typed [`pmcp::Error`]. Everything above it therefore knows only
//! *that* a frame was rejected, never *which* frame. The v3.28.3 fix reacted to
//! that by emitting a single canned response for every bad frame:
//!
//! ```text
//! {"jsonrpc":"2.0","id":null,"error":{"code":-32700,
//!  "message":"Parse error: Transport error: Invalid message format: Invalid
//!             request: Protocol error: -32601 - Method not found: no/such/method"}}
//! ```
//!
//! Two things are wrong with that, and both are user-visible:
//!
//! 1. **`id` is always `null`.** A host that correlates responses by id never
//!    resolves its pending `id=2` promise, so the call hangs until the client's
//!    own timeout. Nothing in the frame is a lie, but the response is unusable.
//! 2. **The code and text are wrong for most inputs.** `tools/call` with
//!    `"params": null`, and `tools/call` with the `name` argument missing, both
//!    reported `Method not found: tools/call` — which is false; `tools/call`
//!    exists and works. pmcp collapses *every* request-parse failure into
//!    [`Error::method_not_found`] (`shared/protocol_helpers.rs::parse_request`
//!    ends `Err(Error::method_not_found(method))` whatever actually failed), so
//!    the operator is pointed at the wrong defect.
//!
//! The fix is to stop guessing: this transport owns the read side itself, keeps
//! the **raw line**, and classifies it with `serde_json` before answering. The id
//! is echoed because we can still see it, and the code is chosen from what the
//! frame actually says rather than from pmcp's collapsed error string.
//!
//! # Contract
//!
//! | input | id | code |
//! |---|---|---|
//! | unknown method, with an id | echoed | `-32601` |
//! | known method, params rejected | echoed | `-32602` |
//! | valid JSON object, no usable `method` | echoed | `-32600` |
//! | unparseable JSON / no recoverable id | `null` | `-32700` |
//! | notification (no `id`) or a response frame | *no reply at all* | — |
//!
//! Classification is a pure function of the raw bytes, so identical input always
//! produces identical output.

use async_trait::async_trait;
// pmcp's own JSON-RPC code table rather than a second copy of the numbers:
// `crate::mcp_integration::types::error_codes` says the same thing but is gated
// behind the optional `mcp-integration` feature, and two renderers of the same
// number must not be able to drift apart.
use pmcp::error::{ErrorCode, TransportError};
use pmcp::shared::{Transport, TransportMessage};
use serde_json::{Map, Value};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWriteExt, BufReader};

/// Methods `pmcp::types::ClientRequest` recognises.
///
/// That enum is `#[serde(tag = "method", content = "params")]`, so this list is
/// exactly its `#[serde(rename = "...")]` tags (pmcp 2.17) — it is *pmcp's*
/// method table, not a wish list of ours. Together with
/// [`PMCP_SERVER_METHODS`] it decides one thing only: whether a frame pmcp
/// rejected was rejected because the **method** is unknown (`-32601`) or because
/// the **params** were (`-32602`).
///
/// [`known_methods_are_real_pmcp_variants`] pins every entry against pmcp's
/// deserializer, so a rename upstream fails the tests rather than silently
/// downgrading every bad-params frame to "method not found".
///
/// Being absent from these lists is a graceful degradation (`-32601` instead of
/// `-32602`), never a hang: the id is echoed either way.
const PMCP_CLIENT_METHODS: &[&str] = &[
    "initialize",
    "tools/list",
    "tools/call",
    "prompts/list",
    "prompts/get",
    "resources/list",
    "resources/templates/list",
    "resources/read",
    "resources/subscribe",
    "resources/unsubscribe",
    "completion/complete",
    "logging/setLevel",
    "ping",
    // Also a ServerRequest variant; listed here because pmcp's `parse_request`
    // tries ClientRequest first and both carry the same payload type.
    "sampling/createMessage",
    "tasks/get",
    "tasks/result",
    "tasks/list",
    "tasks/cancel",
];

/// Methods only `pmcp::types::ServerRequest` recognises. See
/// [`PMCP_CLIENT_METHODS`].
const PMCP_SERVER_METHODS: &[&str] = &["roots/list", "elicitation/create"];

/// Is this a method pmcp's request parser knows at all?
fn is_known_method(method: &str) -> bool {
    PMCP_CLIENT_METHODS.contains(&method) || PMCP_SERVER_METHODS.contains(&method)
}

/// Methods pmcp deserializes with **no** `params` field at all.
///
/// Mirrors the special cases in `parse_client_request` / `parse_server_request`
/// so our diagnostic reproduces what pmcp actually attempted rather than a
/// different parse that would produce a different (i.e. invented) reason.
const PARAMLESS_METHODS: &[&str] = &["ping", "roots/list"];

/// What a frame pmcp could not parse deserves in reply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FrameVerdict {
    /// Send nothing. JSON-RPC 2.0 §4.1: a notification is never answered, and a
    /// response frame is not ours to answer either.
    Silent,
    /// Send this JSON-RPC error object.
    Error {
        /// The client's id, echoed so the host can resolve its pending promise.
        /// `Value::Null` only when the frame genuinely carries no usable id.
        id: Value,
        code: i32,
        message: String,
    },
}

impl FrameVerdict {
    /// Render as a complete JSON-RPC 2.0 frame, or `None` for [`Self::Silent`].
    pub(crate) fn to_frame(&self) -> Option<Value> {
        match self {
            Self::Silent => None,
            Self::Error { id, code, message } => Some(serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": code, "message": message },
            })),
        }
    }
}

/// Echo the client's `id` if it is one JSON-RPC allows (string, number, null).
///
/// Anything else (object, array, bool) is not a legal id and cannot be
/// correlated, so it degrades to `null` rather than being reflected verbatim.
fn echo_id(obj: &Map<String, Value>) -> Value {
    match obj.get("id") {
        Some(v @ (Value::String(_) | Value::Number(_))) => v.clone(),
        _ => Value::Null,
    }
}

/// Rebuild the request body pmcp itself tried to deserialize.
///
/// Mirrors `parse_client_request`: paramless methods get no `params` key, and a
/// `null` params becomes `{}` (which is exactly why `"params": null` on
/// `tools/call` fails — the empty object has no `name`).
fn probe_body(method: &str, params: Option<&Value>) -> Value {
    if PARAMLESS_METHODS.contains(&method) {
        return serde_json::json!({ "method": method });
    }
    match params {
        None | Some(Value::Null) => serde_json::json!({ "method": method, "params": {} }),
        Some(p) => serde_json::json!({ "method": method, "params": p }),
    }
}

/// Explain *why* the params were rejected, in pmcp's own words.
///
/// pmcp's `parse_request` discards the real serde error and reports
/// `Method not found: <method>` instead, so we re-run the same deserialization
/// here to recover the truth ("missing field `name`"). If that parse unexpectedly
/// succeeds the params are not what pmcp rejected, and we say only what we know
/// rather than inventing a reason.
fn invalid_params_message(method: &str, params: Option<&Value>) -> String {
    let body = probe_body(method, params);
    // Probe only the enum that owns the method: the other one would always fail
    // with "unknown variant", which would be a *fabricated* reason.
    let detail = if PMCP_SERVER_METHODS.contains(&method) {
        serde_json::from_value::<pmcp::types::ServerRequest>(body).err()
    } else {
        serde_json::from_value::<pmcp::types::ClientRequest>(body).err()
    };
    match detail {
        Some(e) => format!("Invalid params for {method}: {e}"),
        None => format!("Invalid params for {method}"),
    }
}

/// Classify a raw line pmcp refused, deciding id and error code from the bytes.
///
/// Pure: the same line always yields the same verdict.
pub(crate) fn classify_bad_frame(line: &[u8]) -> FrameVerdict {
    // Genuinely malformed: no JSON, therefore no recoverable id. This is the
    // ONLY case that deserves the null-id / -32700 pair the previous fix
    // emitted for everything.
    let value: Value = match serde_json::from_slice(line) {
        Ok(v) => v,
        Err(e) => {
            return FrameVerdict::Error {
                id: Value::Null,
                code: ErrorCode::PARSE_ERROR.as_i32(),
                message: format!("Parse error: {e}"),
            }
        }
    };

    let Some(obj) = value.as_object() else {
        return FrameVerdict::Error {
            id: Value::Null,
            code: ErrorCode::INVALID_REQUEST.as_i32(),
            message: "Invalid Request: a JSON-RPC frame must be a JSON object".to_string(),
        };
    };

    classify_object(obj)
}

/// Classify a frame that parsed as a JSON object. Split out to keep
/// [`classify_bad_frame`] under the cognitive-complexity ceiling.
fn classify_object(obj: &Map<String, Value>) -> FrameVerdict {
    let method = obj.get("method");

    // A result/error frame is a response to something we sent; JSON-RPC has no
    // reply-to-a-reply, so answering it would put a frame on the wire the host
    // never asked for.
    if method.is_none() && (obj.contains_key("result") || obj.contains_key("error")) {
        return FrameVerdict::Silent;
    }

    // No `id` key at all makes it a notification, and §4.1 forbids replying.
    // (Presence of an explicit `"id": null` still counts as a request, matching
    // pmcp's own `json_value.get("id").is_some()` classification.)
    if !obj.contains_key("id") {
        return FrameVerdict::Silent;
    }
    let id = echo_id(obj);

    let Some(method) = method.and_then(Value::as_str) else {
        return FrameVerdict::Error {
            id,
            code: ErrorCode::INVALID_REQUEST.as_i32(),
            message: "Invalid Request: \"method\" must be present and a string".to_string(),
        };
    };

    if is_known_method(method) {
        FrameVerdict::Error {
            id,
            code: ErrorCode::INVALID_PARAMS.as_i32(),
            message: invalid_params_message(method, obj.get("params")),
        }
    } else {
        FrameVerdict::Error {
            id,
            code: ErrorCode::METHOD_NOT_FOUND.as_i32(),
            message: format!("Method not found: {method}"),
        }
    }
}

/// Read one newline-delimited frame, cancel-safely.
///
/// Returns `Ok(None)` at EOF. The bytes live in the caller's persistent
/// `partial` buffer because pmcp's transport actor `select!`s on `receive()`
/// with the outbound arm `biased` first and **drops** the read future whenever a
/// response is ready. `AsyncBufReadExt::read_line` accumulates into a private
/// scratch buffer and loses everything on drop; `read_until` appends straight
/// into `partial`, so a dropped read resumes exactly where it stopped. (This
/// mirrors `StdioTransport::read_cancel_safe_line`, which is private to pmcp.)
///
/// An empty line yields `Some(vec![])` so the caller can answer it as the
/// malformed frame it is instead of silently swallowing client output.
///
/// A final line with no trailing newline is still returned: at EOF no more bytes
/// can arrive, so those bytes are a whole frame, not a partial one. pmcp drops
/// them, which means `printf '%s' '<bad frame>' | pmat` got no answer at all —
/// exactly the silence #648 is about.
async fn read_frame_line<R>(
    reader: &mut BufReader<R>,
    partial: &mut Vec<u8>,
) -> std::io::Result<Option<Vec<u8>>>
where
    R: AsyncRead + Unpin,
{
    loop {
        if let Some(idx) = partial.iter().position(|&b| b == b'\n') {
            return Ok(Some(take_line(partial, idx + 1)));
        }
        if reader.read_until(b'\n', partial).await? == 0 {
            if partial.is_empty() {
                return Ok(None);
            }
            let all = partial.len();
            return Ok(Some(take_line(partial, all)));
        }
    }
}

/// Drain `partial[..end]` as one frame, stripping a trailing `\n` and `\r`.
///
/// `end` is exclusive. It is `partial.len()` for the unterminated-final-line
/// case, where there is no newline to strip.
fn take_line(partial: &mut Vec<u8>, end: usize) -> Vec<u8> {
    let mut line: Vec<u8> = partial.drain(..end).collect();
    if line.last() == Some(&b'\n') {
        line.pop();
    }
    if line.last() == Some(&b'\r') {
        line.pop();
    }
    line
}

/// Read frames until one parses, answering (via `emit`) every one that does not.
///
/// Extracted from [`RawFrameStdioTransport::receive`] so the skip-and-answer loop
/// can be exercised against an in-memory reader in tests.
async fn next_parsable_frame<R, E>(
    reader: &mut BufReader<R>,
    partial: &mut Vec<u8>,
    emit: &mut E,
) -> std::io::Result<Option<TransportMessage>>
where
    R: AsyncRead + Unpin,
    E: FnMut(&FrameVerdict),
{
    while let Some(line) = read_frame_line(reader, partial).await? {
        match pmcp::shared::transport::parse_message(&line) {
            Ok(message) => return Ok(Some(message)),
            // The raw line is still in hand here — this is the whole point of
            // owning the read side. Classify it and keep reading; one bad frame
            // must not cost the session (the #648 round-1 fix, kept).
            Err(_) => emit(&classify_bad_frame(&line)),
        }
    }
    Ok(None)
}

/// Write an error frame to stdout **synchronously**.
///
/// Deliberately not `async`: this runs inside `receive()`, whose future the
/// transport actor drops the instant an outbound response is ready. An awaited
/// `write_all` cancelled mid-frame would leave a truncated JSON line on the
/// wire — a corrupt frame is worse than a slow one. A blocking write has no
/// await point to be cancelled at, and these frames are a few hundred bytes.
fn emit_error_frame(verdict: &FrameVerdict) {
    use std::io::Write;

    let Some(frame) = verdict.to_frame() else {
        return;
    };
    let Ok(mut bytes) = serde_json::to_vec(&frame) else {
        return;
    };
    bytes.push(b'\n');

    let mut out = std::io::stdout();
    if out.write_all(&bytes).is_ok() {
        let _ = out.flush();
    }
}

/// Registry listings whose array order must not depend on a hash seed, and the
/// field each one is ordered by.
///
/// MCP does not specify an order for any of these, which is exactly why the
/// server has to pick one: an unspecified order is not a licence to emit a
/// different one every time.
const ORDERED_REGISTRY_ARRAYS: &[(&str, &str)] = &[
    ("tools", "name"),
    ("prompts", "name"),
    ("resources", "uri"),
    ("resourceTemplates", "uriTemplate"),
];

/// Sort the registry listings inside an outgoing frame, in place.
///
/// DETERMINISM (round-3 sweep): `tools/list` is answered from pmcp's tool
/// registry, which is a `HashMap`, so the `tools` array came out in a
/// per-process random order — 8 server processes produced 8 DISTINCT orderings
/// (first three names went `["pdmt_deterministic_todos","git_operation",
/// "quality_gate"]`, then `["git_operation","analyze_dead_code",
/// "analyze_deep_context"]`, then `["scaffold_project",
/// "pdmt_deterministic_todos","generate_context"]`, …). It was stable WITHIN a
/// process, which is why in-process tests never caught it. The consequence is
/// on the record: v3.28.3's own commit claimed "5 identical runs produce
/// byte-identical output", and 5 runs of the repro session gave 5 different
/// md5s — localised to this array, because the same session with `tools/list`
/// removed WAS byte-identical five times over.
///
/// Sorting here, at the wire, is deliberate: it is the one place every response
/// passes through, so no future registry can reintroduce the defect, and pmcp's
/// own encoder still decides the wire format.
///
/// The bytes are left completely untouched unless a listing is actually present
/// and actually out of order, so no other frame can be reshaped (in particular
/// nothing else gets round-tripped through `serde_json::Value`).
pub(crate) fn order_registry_arrays(bytes: &mut Vec<u8>) {
    let Ok(mut frame) = serde_json::from_slice::<Value>(bytes) else {
        return;
    };
    let Some(result) = frame.get_mut("result").and_then(Value::as_object_mut) else {
        return;
    };

    let mut reordered = false;
    for (array_key, sort_key) in ORDERED_REGISTRY_ARRAYS {
        let Some(items) = result.get_mut(*array_key).and_then(Value::as_array_mut) else {
            continue;
        };
        let before: Vec<Option<String>> = items
            .iter()
            .map(|item| entry_sort_key(item, sort_key))
            .collect();
        // Entries without the key sort last, among themselves in arrival order
        // (a stable sort), rather than being reordered on a guess.
        items.sort_by(
            |a, b| match (entry_sort_key(a, sort_key), entry_sort_key(b, sort_key)) {
                (Some(x), Some(y)) => x.cmp(&y),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            },
        );
        let after: Vec<Option<String>> = items
            .iter()
            .map(|item| entry_sort_key(item, sort_key))
            .collect();
        reordered |= before != after;
    }

    if !reordered {
        return;
    }
    if let Ok(encoded) = serde_json::to_vec(&frame) {
        *bytes = encoded;
    }
}

/// The value an entry is ordered by, if it has one.
fn entry_sort_key(entry: &Value, sort_key: &str) -> Option<String> {
    entry.get(sort_key)?.as_str().map(ToString::to_string)
}

/// stdio transport that keeps the raw line long enough to answer it properly.
///
/// Drop-in replacement for `pmcp::shared::StdioTransport` in
/// [`crate::mcp_pmcp::SimpleUnifiedServer`]. Two deliberate differences:
///
/// * a frame pmcp cannot parse is answered here (see [`classify_bad_frame`])
///   and reading continues, so `receive()` only ever errors when the session is
///   genuinely over;
/// * **read-side EOF does not close the write side.** `StdioTransport` sets one
///   `closed` flag from its reader and then refuses every subsequent `send`,
///   which is what discarded responses the server had already committed to.
///   stdin and stdout are independent streams; a client that pipes a batch and
///   closes stdin is still reading stdout.
#[derive(Debug)]
pub(crate) struct RawFrameStdioTransport {
    stdin: BufReader<tokio::io::Stdin>,
    /// Persistent partial-line buffer; see [`read_frame_line`].
    partial: Vec<u8>,
    stdout: tokio::io::Stdout,
    /// Set by [`Transport::close`] only — never by read-side EOF.
    closed: bool,
    /// Set when stdin reaches EOF; stops further reads without touching writes.
    read_done: bool,
}

impl RawFrameStdioTransport {
    pub(crate) fn new() -> Self {
        Self {
            stdin: BufReader::new(tokio::io::stdin()),
            partial: Vec::new(),
            stdout: tokio::io::stdout(),
            closed: false,
            read_done: false,
        }
    }
}

#[async_trait]
impl Transport for RawFrameStdioTransport {
    async fn send(&mut self, message: TransportMessage) -> pmcp::Result<()> {
        if self.closed {
            return Err(TransportError::ConnectionClosed.into());
        }
        // pmcp's own encoder: the single source of truth for the wire format.
        let mut bytes = pmcp::shared::transport::serialize_message(&message)?;
        order_registry_arrays(&mut bytes);
        bytes.push(b'\n');
        self.stdout
            .write_all(&bytes)
            .await
            .map_err(|e| TransportError::Io(e.to_string()))?;
        self.stdout
            .flush()
            .await
            .map_err(|e| TransportError::Io(e.to_string()))?;
        Ok(())
    }

    async fn receive(&mut self) -> pmcp::Result<TransportMessage> {
        if self.closed || self.read_done {
            return Err(TransportError::ConnectionClosed.into());
        }
        let next = next_parsable_frame(&mut self.stdin, &mut self.partial, &mut emit_error_frame)
            .await
            .map_err(|e| TransportError::Io(e.to_string()))?;
        match next {
            Some(message) => Ok(message),
            None => {
                self.read_done = true;
                Err(TransportError::ConnectionClosed.into())
            }
        }
    }

    async fn close(&mut self) -> pmcp::Result<()> {
        self.closed = true;
        let _ = self.stdout.flush().await;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        !self.closed
    }

    fn transport_type(&self) -> &'static str {
        "stdio"
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    fn verdict(line: &str) -> FrameVerdict {
        classify_bad_frame(line.as_bytes())
    }

    fn parts(line: &str) -> (Value, i32, String) {
        match verdict(line) {
            FrameVerdict::Error { id, code, message } => (id, code, message),
            FrameVerdict::Silent => panic!("expected an error verdict for: {line}"),
        }
    }

    /// #648 consequence 1 (the serious one): the id MUST come back, or a host
    /// correlating by id never resolves its promise and the call hangs until
    /// the client times out. The shipped v3.28.3 answer was always
    /// `"id":null,"code":-32700` regardless of input.
    #[test]
    fn unknown_method_echoes_the_id_and_reports_method_not_found() {
        let (id, code, message) =
            parts(r#"{"jsonrpc":"2.0","id":2,"method":"no/such/method","params":{}}"#);
        assert_eq!(id, serde_json::json!(2), "the client's id must be echoed");
        assert_eq!(code, -32601, "an unknown method is -32601, not -32700");
        assert_eq!(message, "Method not found: no/such/method");
    }

    /// #648 consequence 2: `tools/call` exists and demonstrably works, so
    /// answering "Method not found: tools/call" points the operator at the
    /// wrong defect. Bad params are -32602.
    #[test]
    fn tools_call_with_null_params_is_invalid_params_not_method_not_found() {
        let (id, code, message) =
            parts(r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":null}"#);
        assert_eq!(id, serde_json::json!(2));
        assert_eq!(code, -32602, "params:null on a real method is -32602");
        assert!(
            !message.contains("Method not found"),
            "tools/call exists; claiming otherwise is false. got: {message}"
        );
        assert!(
            message.contains("name"),
            "the message must name the field that was missing, got: {message}"
        );
    }

    #[test]
    fn tools_call_missing_tool_name_is_invalid_params() {
        let (id, code, message) = parts(
            r#"{"jsonrpc":"2.0","id":"abc","method":"tools/call","params":{"arguments":{}}}"#,
        );
        assert_eq!(
            id,
            serde_json::json!("abc"),
            "string ids must be echoed verbatim"
        );
        assert_eq!(code, -32602);
        assert!(!message.contains("Method not found"), "got: {message}");
    }

    /// The one case where null/-32700 IS the honest answer.
    #[test]
    fn unparseable_line_is_parse_error_with_null_id() {
        let (id, code, message) = parts("this is not json");
        assert_eq!(id, Value::Null, "there is no id to recover here");
        assert_eq!(code, -32700);
        assert!(message.starts_with("Parse error"), "got: {message}");
    }

    #[test]
    fn empty_line_is_a_parse_error() {
        let (id, code, _) = parts("");
        assert_eq!(id, Value::Null);
        assert_eq!(code, -32700);
    }

    #[test]
    fn json_that_is_not_an_object_is_invalid_request() {
        let (id, code, _) = parts("[1,2,3]");
        assert_eq!(id, Value::Null);
        assert_eq!(code, -32600);
    }

    #[test]
    fn object_with_an_id_but_no_method_is_invalid_request_with_the_id_echoed() {
        let (id, code, _) = parts(r#"{"jsonrpc":"2.0","id":9,"paramz":{}}"#);
        assert_eq!(id, serde_json::json!(9));
        assert_eq!(code, -32600);
    }

    /// JSON-RPC 2.0 §4.1: a notification is never answered. Emitting a frame
    /// for one puts bytes on the wire the host never asked for.
    #[test]
    fn unparseable_notification_gets_no_reply() {
        assert_eq!(
            verdict(r#"{"jsonrpc":"2.0","method":"notifications/bogus"}"#),
            FrameVerdict::Silent
        );
    }

    #[test]
    fn a_response_frame_is_not_ours_to_answer() {
        assert_eq!(
            verdict(r#"{"jsonrpc":"2.0","id":1,"result":{"unknown":"shape"}}"#),
            FrameVerdict::Silent
        );
        assert_eq!(
            verdict(r#"{"jsonrpc":"2.0","id":1,"error":{"code":1,"message":"x"}}"#),
            FrameVerdict::Silent
        );
    }

    /// An id JSON-RPC does not allow cannot be correlated, so it must not be
    /// reflected as if it could.
    #[test]
    fn a_non_conforming_id_degrades_to_null() {
        let (id, code, _) = parts(r#"{"jsonrpc":"2.0","id":{"a":1},"method":"no/such"}"#);
        assert_eq!(id, Value::Null);
        assert_eq!(code, -32601);
    }

    /// Varying the input must vary the output (`output_derived_from_input`):
    /// the round-1 code returned one constant frame for all of these.
    #[test]
    fn distinct_inputs_produce_distinct_answers() {
        let answers: Vec<FrameVerdict> = [
            r#"{"jsonrpc":"2.0","id":1,"method":"no/such/method"}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":null}"#,
            r#"{"jsonrpc":"2.0","id":3,"method":"other/missing"}"#,
            "not json at all",
        ]
        .iter()
        .map(|l| classify_bad_frame(l.as_bytes()))
        .collect();

        for (i, a) in answers.iter().enumerate() {
            for (j, b) in answers.iter().enumerate() {
                assert!(
                    i == j || a != b,
                    "inputs {i} and {j} collapsed to the same answer: {a:?}"
                );
            }
        }
    }

    /// Nondeterminism in an analysis tool is a defect: five runs, one answer.
    #[test]
    fn classification_is_deterministic_across_five_runs() {
        let inputs = [
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":null}"#,
            r#"{"jsonrpc":"2.0","id":"x","method":"no/such/method"}"#,
            "garbage",
            r#"{"jsonrpc":"2.0","method":"notifications/bogus"}"#,
        ];
        for input in inputs {
            let first = classify_bad_frame(input.as_bytes());
            for run in 1..5 {
                assert_eq!(
                    classify_bad_frame(input.as_bytes()),
                    first,
                    "run {run} disagreed with run 0 for: {input}"
                );
            }
        }
    }

    /// Serde's wording for an unrecognised `#[serde(tag = "method")]` value.
    const TAG_ERROR: &str = "unknown variant";

    fn err_text<T>(r: Result<T, serde_json::Error>) -> String {
        r.err().map(|e| e.to_string()).unwrap_or_default()
    }

    fn client_rejects_tag(method: &str) -> bool {
        let body = probe_body(method, None);
        err_text(serde_json::from_value::<pmcp::types::ClientRequest>(body)).contains(TAG_ERROR)
    }

    fn server_rejects_tag(method: &str) -> bool {
        let body = probe_body(method, None);
        err_text(serde_json::from_value::<pmcp::types::ServerRequest>(body)).contains(TAG_ERROR)
    }

    /// Grounds [`PMCP_CLIENT_METHODS`] / [`PMCP_SERVER_METHODS`] in pmcp's own
    /// type system rather than in anybody's memory of the MCP spec: every entry
    /// must be a real variant tag of the enum it is filed under.
    ///
    /// The control case is deliberate — if serde ever changes the "unknown
    /// variant" wording, the control fails and this test goes red rather than
    /// silently passing on a probe that can no longer detect anything.
    #[test]
    fn known_methods_are_real_pmcp_variants() {
        assert!(
            client_rejects_tag("no/such/method") && server_rejects_tag("no/such/method"),
            "control case: a bogus method must be reported as an unknown variant, \
             otherwise this test proves nothing"
        );
        for method in PMCP_CLIENT_METHODS {
            assert!(
                !client_rejects_tag(method),
                "{method} is in PMCP_CLIENT_METHODS but pmcp::types::ClientRequest \
                 does not know it — bad-params frames for it would be answered -32601"
            );
        }
        for method in PMCP_SERVER_METHODS {
            assert!(
                !server_rejects_tag(method),
                "{method} is in PMCP_SERVER_METHODS but pmcp::types::ServerRequest \
                 does not know it"
            );
        }
    }

    /// The `-32602` reason must describe the params, never the *other* enum's
    /// "unknown variant" complaint — that would name a defect the client's
    /// frame does not have, which is the class of lie #648 is about.
    #[test]
    fn the_invalid_params_reason_never_blames_the_method() {
        for method in PMCP_CLIENT_METHODS.iter().chain(PMCP_SERVER_METHODS) {
            let message = invalid_params_message(method, Some(&serde_json::json!({"zzz": 1})));
            assert!(
                !message.contains(TAG_ERROR) && !message.contains("Method not found"),
                "{method}: the reason must describe the params, got: {message}"
            );
        }
    }

    // === frame reader / skip loop ===

    async fn drain(input: &str) -> (Vec<TransportMessage>, Vec<FrameVerdict>) {
        let mut reader = BufReader::new(std::io::Cursor::new(input.as_bytes().to_vec()));
        let mut partial = Vec::new();
        let mut verdicts = Vec::new();
        let mut messages = Vec::new();
        {
            let mut sink = |v: &FrameVerdict| verdicts.push(v.clone());
            while let Some(m) = next_parsable_frame(&mut reader, &mut partial, &mut sink)
                .await
                .expect("cursor never errors")
            {
                messages.push(m);
            }
        }
        (messages, verdicts)
    }

    /// The round-1 win that must not regress: a bad frame is answered and the
    /// *following* valid request still arrives.
    #[tokio::test]
    async fn a_bad_frame_does_not_cost_the_next_request() {
        let (messages, verdicts) = drain(concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"no/such/method\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
        ))
        .await;

        assert_eq!(verdicts.len(), 1, "exactly one bad frame answered");
        assert!(matches!(
            verdicts[0],
            FrameVerdict::Error { code: -32601, .. }
        ));
        assert_eq!(messages.len(), 1, "the valid request must still surface");
        assert!(matches!(messages[0], TransportMessage::Request { .. }));
    }

    /// Each bad frame gets its OWN answer, carrying its OWN id.
    #[tokio::test]
    async fn every_bad_frame_is_answered_with_its_own_id() {
        let (_, verdicts) = drain(concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"a/x\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"b/y\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":null}\n",
        ))
        .await;

        let ids: Vec<Value> = verdicts
            .iter()
            .filter_map(|v| v.to_frame())
            .map(|f| f["id"].clone())
            .collect();
        assert_eq!(
            ids,
            vec![
                serde_json::json!(1),
                serde_json::json!(2),
                serde_json::json!(3)
            ]
        );
    }

    /// The reader is dropped mid-line every time the actor has a response
    /// ready; losing those bytes would corrupt the next frame.
    #[tokio::test]
    async fn a_dropped_read_keeps_the_bytes_it_consumed() {
        use tokio::io::AsyncWriteExt as _;

        let (mut writer, reader) = tokio::io::duplex(64);
        let mut reader = BufReader::new(reader);
        let mut partial: Vec<u8> = Vec::new();

        writer.write_all(b"{\"half\"").await.unwrap();
        tokio::select! {
            _ = read_frame_line(&mut reader, &mut partial) => panic!("no newline yet"),
            () = tokio::time::sleep(std::time::Duration::from_millis(50)) => {}
        }
        assert_eq!(partial, b"{\"half\"");

        writer.write_all(b":1}\nnext\n").await.unwrap();
        let line = read_frame_line(&mut reader, &mut partial)
            .await
            .unwrap()
            .expect("a complete line");
        assert_eq!(line, b"{\"half\":1}", "no bytes lost across the drop");
    }

    #[tokio::test]
    async fn crlf_endings_are_stripped() {
        let (messages, verdicts) =
            drain("{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\",\"params\":{}}\r\n")
                .await;
        assert!(verdicts.is_empty(), "a CRLF frame is not malformed");
        assert_eq!(messages.len(), 1);
    }

    #[tokio::test]
    async fn eof_ends_the_read_loop() {
        let (messages, verdicts) = drain("").await;
        assert!(messages.is_empty());
        assert!(verdicts.is_empty());
    }

    /// `printf '%s' '<frame>'` (no trailing newline) is a complete frame once
    /// EOF proves no more bytes are coming. pmcp discards it, so such a client
    /// got no answer at all — the same silence #648 is about.
    #[tokio::test]
    async fn a_final_line_without_a_newline_is_still_a_frame() {
        let (messages, verdicts) =
            drain(r#"{"jsonrpc":"2.0","id":8,"method":"no/such/method"}"#).await;
        assert!(messages.is_empty());
        assert_eq!(verdicts.len(), 1, "the unterminated frame must be answered");
        let frame = verdicts[0].to_frame().expect("an error frame");
        assert_eq!(frame["id"], serde_json::json!(8));
        assert_eq!(frame["error"]["code"], serde_json::json!(-32601));
    }

    #[tokio::test]
    async fn a_valid_final_line_without_a_newline_still_parses() {
        let (messages, verdicts) =
            drain(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#).await;
        assert!(verdicts.is_empty());
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn silent_verdicts_render_no_frame() {
        assert_eq!(FrameVerdict::Silent.to_frame(), None);
    }

    #[test]
    fn error_verdicts_render_a_well_formed_jsonrpc_frame() {
        let frame = FrameVerdict::Error {
            id: serde_json::json!(7),
            code: -32601,
            message: "Method not found: x".to_string(),
        }
        .to_frame()
        .expect("an error verdict renders a frame");
        assert_eq!(frame["jsonrpc"], "2.0");
        assert_eq!(frame["id"], serde_json::json!(7));
        assert_eq!(frame["error"]["code"], serde_json::json!(-32601));
        assert_eq!(frame["error"]["message"], "Method not found: x");
    }

    // -- DETERMINISM: registry listings (round-3 sweep) --

    fn tools_frame(names: &[&str]) -> Vec<u8> {
        let tools: Vec<Value> = names
            .iter()
            .map(|n| serde_json::json!({ "name": n, "description": "d" }))
            .collect();
        serde_json::to_vec(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "result": { "tools": tools },
        }))
        .expect("frame serializes")
    }

    fn tool_names(bytes: &[u8]) -> Vec<String> {
        let frame: Value = serde_json::from_slice(bytes).expect("frame parses");
        frame["result"]["tools"]
            .as_array()
            .expect("tools array")
            .iter()
            .map(|t| t["name"].as_str().expect("name").to_string())
            .collect()
    }

    /// `tools/list` is answered from pmcp's registry, which is a `HashMap`, so
    /// the array came out in a per-process order: 8 server processes produced
    /// 8 DISTINCT orderings, and 5 runs of issue #648's own repro gave 5
    /// different md5 sums even after `sort`. The same session with `tools/list`
    /// removed WAS byte-identical five times over, which localised it here.
    ///
    /// Any arrival order must leave by one, sorted order.
    #[test]
    fn tools_list_is_sorted_whatever_order_it_arrives_in() {
        let arrivals = [
            vec![
                "pdmt_deterministic_todos",
                "git_operation",
                "quality_gate",
                "analyze_satd",
            ],
            vec![
                "git_operation",
                "analyze_satd",
                "pdmt_deterministic_todos",
                "quality_gate",
            ],
            vec![
                "quality_gate",
                "pdmt_deterministic_todos",
                "analyze_satd",
                "git_operation",
            ],
            vec![
                "analyze_satd",
                "quality_gate",
                "git_operation",
                "pdmt_deterministic_todos",
            ],
            vec![
                "git_operation",
                "quality_gate",
                "analyze_satd",
                "pdmt_deterministic_todos",
            ],
        ];

        let expected = vec![
            "analyze_satd".to_string(),
            "git_operation".to_string(),
            "pdmt_deterministic_todos".to_string(),
            "quality_gate".to_string(),
        ];

        for arrival in &arrivals {
            let mut bytes = tools_frame(arrival);
            order_registry_arrays(&mut bytes);
            assert_eq!(
                tool_names(&bytes),
                expected,
                "arrival order {arrival:?} must leave sorted"
            );
        }
    }

    /// Nothing else is reshaped: a frame with no registry listing, and a
    /// listing already in order, come out byte-for-byte unchanged.
    #[test]
    fn frames_without_a_registry_listing_are_left_byte_identical() {
        let untouched = [
            br#"{"jsonrpc":"2.0","id":1,"result":{}}"#.to_vec(),
            br#"{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"x"}}"#.to_vec(),
            br#"{"jsonrpc":"2.0","id":1e+20,"error":{"code":-32601,"message":"x"}}"#.to_vec(),
            b"not json at all".to_vec(),
            tools_frame(&["a_tool", "b_tool", "c_tool"]),
        ];
        for original in untouched {
            let mut bytes = original.clone();
            order_registry_arrays(&mut bytes);
            assert_eq!(
                bytes, original,
                "a frame that needs no reordering must not be rewritten"
            );
        }
    }

    /// Entries missing the sort key are kept, at the end, rather than dropped
    /// or reordered on a guess.
    #[test]
    fn entries_without_the_sort_key_are_kept() {
        let mut bytes = serde_json::to_vec(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "result": { "tools": [
                { "name": "zed" },
                { "no_name": true },
                { "name": "abe" },
            ]},
        }))
        .expect("serializes");
        order_registry_arrays(&mut bytes);

        let frame: Value = serde_json::from_slice(&bytes).expect("parses");
        let tools = frame["result"]["tools"].as_array().expect("array");
        assert_eq!(tools.len(), 3, "no entry may be dropped");
        assert_eq!(tools[0]["name"], "abe");
        assert_eq!(tools[1]["name"], "zed");
        assert_eq!(tools[2]["no_name"], serde_json::json!(true));
    }

    #[tokio::test]
    async fn transport_reports_stdio_and_closes_cleanly() {
        let mut t = RawFrameStdioTransport::new();
        assert_eq!(t.transport_type(), "stdio");
        assert!(t.is_connected());
        t.close().await.expect("close succeeds");
        assert!(!t.is_connected());
        // A closed transport refuses further reads; read-side EOF alone never
        // does this to the write side.
        assert!(t.receive().await.is_err());
    }
}
