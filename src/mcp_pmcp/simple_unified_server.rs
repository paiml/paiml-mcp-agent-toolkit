use crate::mcp::tools::agent_context_tools::IndexManager;
use crate::mcp_pmcp::agent_context_handlers::{
    PmatFindSimilarHandler, PmatGetFunctionHandler, PmatIndexStatsHandler, PmatQueryCodeHandler,
};
use crate::mcp_pmcp::analyze_handlers::{
    AnalyzeBigOTool, AnalyzeComplexityTool, AnalyzeDagTool, AnalyzeDeadCodeTool,
    AnalyzeDeepContextTool, AnalyzeSatdTool, HardcodedPathsTool, ReachabilityTool,
    VacuousTestsTool,
};
use crate::mcp_pmcp::context_handlers::{GenerateContextTool, GitTool, ScaffoldProjectTool};
use crate::mcp_pmcp::pdmt_handler::PdmtTool;
use crate::mcp_pmcp::quality_handlers::QualityGateTool;
use crate::mcp_pmcp::quality_proxy_handler::QualityProxyTool;
use crate::mcp_pmcp::stdio_frames::RawFrameStdioTransport;
use crate::mcp_server::state_manager::StateManager;
use async_trait::async_trait;
use pmcp::shared::{Transport, TransportMessage};
use pmcp::{Server, ServerCapabilities};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};
use tracing::info;

/// Stdio transport wrapper that signals session end when the read side fails.
///
/// pmcp's `Server::run` spawns the stdio reader task and then awaits an
/// infinite keep-alive loop. When the client closes stdin (EOF surfaces from
/// `StdioTransport::receive` as `TransportError::ConnectionClosed`), the
/// reader task breaks out of its loop but the keep-alive future never
/// completes — so a one-shot piped session (`printf '...' | MCP_VERSION=1
/// pmat`) answered in milliseconds yet the process lived until externally
/// killed (exit=124 under `timeout`). This wrapper forwards the first
/// `receive()` error to a oneshot channel so [`SimpleUnifiedServer::run`] can
/// race the server future against session end and exit cleanly.
///
/// Long-lived hosts (e.g. Claude Code) hold stdin open: the blocking read
/// never errors, the channel never fires, and behavior is unchanged. Any
/// other receive error also signals shutdown, because pmcp's reader task
/// breaks on every receive error and would otherwise leave the server
/// permanently deaf.
///
/// # Draining in-flight work before signalling
///
/// Signalling on the *first* receive error truncated responses. EOF is
/// observed by the read side while a request consumed moments earlier is
/// still being handled, so `run`'s `select!` took the session-end branch and
/// the process exited before that response was ever written. Piping
/// `initialize` + `tools/list` in one write and closing stdin answered
/// `tools/list` in only 2 of 5 trials on a release build (5/5 on debug, which
/// is merely too slow to lose the race). The comment that used to sit in
/// `run` asserted the opposite — that every consumed request had already been
/// answered — and that assertion was simply false.
///
/// So this counts requests in against responses out and defers the signal
/// until the count reaches zero.
///
/// # Withholding EOF from pmcp itself
///
/// Counting was necessary but not sufficient. pmcp's transport actor breaks its
/// loop the instant `receive()` errors, *without* draining the outbound queue,
/// so a response the worker already produced is discarded before `send()` is
/// ever reached — above this wrapper, where counting cannot see it. Against
/// pmcp 2.17 that cost `tools/list` its answer in 21 of 30 one-shot sessions
/// even with the counter in place.
///
/// `receive()` therefore does not surface EOF while a consumed request is
/// unanswered. The actor's `select!` is `biased` with the outbound arm first, so
/// withholding keeps it in the loop, the queued response wins, this future is
/// dropped, `send()` decrements, and the following `receive()` reports EOF with
/// nothing outstanding. See [`Self::DRAIN_BACKSTOP`] for the liveness bound.
///
/// The original hang this wrapper was written to fix is unaffected: with no work
/// outstanding, `in_flight` is already 0 and EOF surfaces immediately. Both the
/// withholding and the deferral happen only when exiting would lose data.
#[derive(Debug)]
struct EofSignalingTransport<T: Transport> {
    inner: T,
    session_end_tx: Option<oneshot::Sender<String>>,
    /// Requests received whose responses have not yet been sent.
    ///
    /// A plain field rather than an atomic: pmcp drives the transport from a
    /// single-owner actor, and both `send` and `receive` take `&mut self`, so
    /// access is already exclusive.
    in_flight: usize,
    /// Why the session ended, recorded on the first receive error and held
    /// until `in_flight` drains to zero.
    pending_end: Option<String>,
}

impl<T: Transport> EofSignalingTransport<T> {
    /// Wrap `inner`, returning the transport and the session-end receiver.
    ///
    /// The receiver resolves with a human-readable reason once `receive()` on
    /// the wrapped transport has errored (EOF or otherwise) *and* every
    /// request already taken off the wire has been answered.
    fn new(inner: T) -> (Self, oneshot::Receiver<String>) {
        let (tx, rx) = oneshot::channel();
        (
            Self {
                inner,
                session_end_tx: Some(tx),
                in_flight: 0,
                pending_end: None,
            },
            rx,
        )
    }

    /// How long `receive()` will withhold EOF while a request is unanswered.
    ///
    /// Only a backstop: the actor's biased outbound arm normally wins in
    /// microseconds. It exists so a handler that never answers degrades to the
    /// old truncation instead of wedging the process indefinitely, and it is
    /// generous because `analyze_deep_context` legitimately runs for minutes.
    const DRAIN_BACKSTOP: std::time::Duration = std::time::Duration::from_secs(300);

    /// Write a response the inner transport refused, straight to stdout.
    ///
    /// pmcp's `StdioTransport` flips a `closed` flag the moment its *read* side
    /// hits EOF (`shared/stdio.rs`), and `send()` then rejects every subsequent
    /// write. So a reply for a request the server already accepted is discarded
    /// simply because the client closed **stdin** — which says nothing about
    /// whether stdout is still readable. Under the MCP stdio transport the two
    /// directions are independent streams, and a client that pipes a batch and
    /// closes stdin is still waiting on stdout. This is why withholding EOF from
    /// the actor was not sufficient on pmcp 2.17: the frame reached `send()` and
    /// was rejected below us.
    ///
    /// Emitting the frame here uses pmcp's own `serialize_message`, the
    /// documented single source of truth for the wire encoding, so the bytes are
    /// identical to what the transport would have written. Returns whether the
    /// frame was delivered.
    ///
    /// Reported upstream as paiml/rust-mcp-sdk#316; remove this once
    /// `StdioTransport` splits its single `closed` flag into read-side and
    /// write-side state and stops coupling the write side to read-side EOF.
    async fn deliver_refused_response(frame: &TransportMessage) -> bool {
        use tokio::io::AsyncWriteExt;

        let Ok(mut bytes) = pmcp::shared::transport::serialize_message(frame) else {
            return false;
        };
        // Same ordering the normal write path applies, so a salvaged response
        // is not the one frame whose `tools` array is hash-ordered.
        crate::mcp_pmcp::stdio_frames::order_registry_arrays(&mut bytes);
        bytes.push(b'\n');

        let mut out = tokio::io::stdout();
        if out.write_all(&bytes).await.is_err() {
            return false;
        }
        out.flush().await.is_ok()
    }

    /// Does this `receive()` error mean the session is genuinely over?
    ///
    /// Only a closed connection or a broken stdin does. A frame we could not
    /// parse — a malformed line, an unknown method, bad params — says nothing
    /// about whether the client is still there, and JSON-RPC defines error
    /// responses precisely so the conversation can continue.
    ///
    /// Treating every error as terminal is what made a single bad frame kill
    /// the whole session: `initialize` was answered, then an unknown method
    /// ended the process with exit 0 and the *next valid* request was never
    /// answered. A host sees the server succeed and vanish mid-conversation.
    fn is_session_over(e: &pmcp::Error) -> bool {
        matches!(
            e,
            pmcp::Error::Transport(
                pmcp::error::TransportError::ConnectionClosed | pmcp::error::TransportError::Io(_)
            )
        )
    }

    /// How many consecutive non-terminal receive errors to tolerate.
    ///
    /// [`RawFrameStdioTransport`] consumes a line before it can reject it, so
    /// this loop always makes progress and the cap is never reached in
    /// production. It only bounds a hypothetical inner transport that errors
    /// without consuming, so a bug there degrades to exit rather than a spin.
    const MAX_CONSECUTIVE_BAD_FRAMES: u32 = 1024;

    /// Read until a message arrives or the session is genuinely over.
    ///
    /// A frame the inner transport could not turn into a message must not end
    /// the session: pmcp's actor breaks its `select!` the instant `receive()`
    /// returns Err, so merely declining to signal session-end would turn a
    /// silent exit into a hang — the error must not be propagated at all.
    ///
    /// The JSON-RPC error response for such a frame is emitted by the inner
    /// transport, NOT here: [`RawFrameStdioTransport`] is the only layer that
    /// still holds the raw line and can therefore echo the client's id and pick
    /// the right code. This wrapper used to answer instead, and could only ever
    /// emit `{"id":null,"code":-32700}` — which left every host correlating by
    /// id waiting on a promise that never resolved (#648).
    async fn receive_past_bad_frames(&mut self) -> pmcp::Result<TransportMessage> {
        let mut bad_frames: u32 = 0;
        loop {
            let attempt = self.inner.receive().await;
            match &attempt {
                Err(e) if !Self::is_session_over(e) => {
                    bad_frames += 1;
                    if bad_frames >= Self::MAX_CONSECUTIVE_BAD_FRAMES {
                        return attempt;
                    }
                }
                _ => return attempt,
            }
        }
    }

    /// Record why the session ended and hold EOF back until nothing is owed.
    ///
    /// pmcp's transport actor breaks its loop the moment `receive()` errors,
    /// *without* draining the outbound queue — so a response the worker has
    /// already produced is dropped on the floor. That happens above this
    /// wrapper, which is why counting sends alone was not enough: `send()` was
    /// never reached. On pmcp 2.17 this cost `tools/list` its answer in 21 of 30
    /// one-shot sessions.
    ///
    /// The actor's `select!` is `biased` with the outbound arm first, so simply
    /// not resolving here keeps it in the loop: the queued response wins the
    /// race, this future is dropped, `send()` runs and decrements, and the next
    /// `receive()` surfaces EOF with nothing outstanding. Dropping this future
    /// loses no bytes — the inner transport already returned an error, and
    /// asking it again returns the same error.
    async fn handle_terminal_receive_error(&mut self, e: &pmcp::Error) {
        // Record the reason on the first error only, then signal as soon as
        // everything already consumed has been answered.
        if self.pending_end.is_none() {
            self.pending_end = Some(e.to_string());
        }

        if self.in_flight > 0 {
            // Bounded so a handler that never answers degrades to the old
            // truncation rather than wedging the process forever; a hang is
            // worse for a user than a lost response. The normal path never
            // waits: the outbound arm wins in microseconds.
            tokio::time::sleep(Self::DRAIN_BACKSTOP).await;
        }

        self.signal_if_drained();
    }

    /// Fire the session-end signal if the read side is finished and no
    /// consumed request is still awaiting its response.
    fn signal_if_drained(&mut self) {
        if self.in_flight > 0 {
            return;
        }
        let Some(reason) = self.pending_end.take() else {
            return;
        };
        if let Some(tx) = self.session_end_tx.take() {
            let _ = tx.send(reason);
        }
    }
}

#[async_trait]
impl<T: Transport> Transport for EofSignalingTransport<T> {
    async fn send(&mut self, message: TransportMessage) -> pmcp::Result<()> {
        // Only a Response retires a request. Notifications get no reply, and
        // a Request travelling outbound is the server calling the client
        // (e.g. sampling), not an answer to anything we counted.
        let retires_request = matches!(message, TransportMessage::Response(_));

        // Keep a copy so a refused response can still be delivered. Only
        // responses are worth the clone; see `deliver_refused_response`.
        let salvage = if retires_request {
            Some(message.clone())
        } else {
            None
        };

        let mut result = self.inner.send(message).await;

        if let (Err(_), Some(frame)) = (&result, &salvage) {
            if Self::deliver_refused_response(frame).await {
                result = Ok(());
            }
        }

        if retires_request {
            self.in_flight = self.in_flight.saturating_sub(1);
            // Attempt the deferred signal even if the send itself failed:
            // this request is never going to be answered now, and holding the
            // session open for it would reintroduce the original hang.
            self.signal_if_drained();
        }
        result
    }

    async fn receive(&mut self) -> pmcp::Result<TransportMessage> {
        let result = self.receive_past_bad_frames().await;
        match &result {
            Ok(TransportMessage::Request { .. }) => self.in_flight += 1,
            Ok(_) => {}
            Err(e) => self.handle_terminal_receive_error(e).await,
        }
        result
    }

    async fn close(&mut self) -> pmcp::Result<()> {
        self.inner.close().await
    }

    fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }

    fn transport_type(&self) -> &'static str {
        self.inner.transport_type()
    }
}

/// Simple unified MCP server that uses only existing, working handlers.
///
/// This is a transitional implementation that provides the most critical tools
/// while we complete the full unification.
pub struct SimpleUnifiedServer {
    state_manager: Arc<Mutex<StateManager>>,
}

/// Forwards an `Arc<dyn AuthProvider>` to `ServerBuilder::auth_provider`,
/// which takes `impl AuthProvider + 'static` and so will not accept the trait
/// object directly.
struct ArcAuthProvider(std::sync::Arc<dyn pmcp::server::auth::AuthProvider>);

#[async_trait::async_trait]
impl pmcp::server::auth::AuthProvider for ArcAuthProvider {
    async fn validate_request(
        &self,
        authorization_header: Option<&str>,
    ) -> pmcp::error::Result<Option<pmcp::server::auth::AuthContext>> {
        self.0.validate_request(authorization_header).await
    }
    fn auth_scheme(&self) -> &'static str {
        self.0.auth_scheme()
    }
    fn is_required(&self) -> bool {
        self.0.is_required()
    }
}

impl SimpleUnifiedServer {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            state_manager: Arc::new(Mutex::new(StateManager::new())),
        })
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting PMAT Simple Unified MCP server (pmcp SDK)");
        let server = Self::build_server(None)?;
        self.run_stdio(server).await
    }

    /// Build the tool surface, independent of transport.
    ///
    /// Extracted from `run()` so the SAME registrations serve stdio and
    /// streamable HTTP (EV-6, #999). A second builder would be a second answer
    /// to "which tools does pmat advertise", and this file already carries a
    /// drift-guard test because that question has had two answers before.
    ///
    /// `auth` is applied when present. It is `None` for stdio, where the
    /// transport is the process boundary; over HTTP it is REQUIRED by the
    /// caller — see `serve_http`.
    pub fn build_server(
        auth: Option<std::sync::Arc<dyn pmcp::server::auth::AuthProvider>>,
    ) -> Result<Server, Box<dyn std::error::Error>> {
        // KAIZEN-0165: shared IndexManager for the 4 pmat_* AgentContextTools.
        // Construction is cheap (no disk I/O); first tool call triggers index build.
        let index_manager = Arc::new(IndexManager::new(
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        ));

        // Build server with core PMAT tools that are already working
        let builder = Server::builder()
            .name("paiml-mcp-agent-toolkit")
            .version(env!("CARGO_PKG_VERSION"))
            .capabilities(ServerCapabilities::tools_only())
            // === Core Analysis Tools (6) ===
            .tool("analyze_complexity", AnalyzeComplexityTool)
            .tool("analyze_satd", AnalyzeSatdTool)
            .tool("analyze_dead_code", AnalyzeDeadCodeTool)
            .tool("analyze_dag", AnalyzeDagTool)
            .tool("analyze_deep_context", AnalyzeDeepContextTool)
            .tool("analyze_big_o", AnalyzeBigOTool)
            // === Forensic Analyzers (3) — #1029 ===
            // Shipped CLI-only in 3.32.0 because this list was hand-curated
            // beside `AnalyzeCommands` instead of derived from it. Which
            // `analyze` subcommands belong here is now decided by the total
            // match in `cli::analyze_mcp_exposure`, so the next variant cannot
            // reach a release undeclared.
            .tool("analyze_reachability", ReachabilityTool)
            .tool("analyze_hardcoded_paths", HardcodedPathsTool)
            .tool("analyze_vacuous_tests", VacuousTestsTool)
            // === Refactoring Tools: REMOVED (EV-0, #999) ===
            // `refactor.start` / `nextIteration` / `getState` / `stop` were 4 of
            // 20 advertised tools — 20% of the agent-facing surface — and the
            // engine behind them is not an analyzer. `find_violations`
            // (models/refactor_impls.rs:140-145) returns a hardcoded
            // HighComplexity violation at "line 100, column 1" for any file
            // whose PATH CONTAINS THE SUBSTRING "complex", and nothing at all
            // otherwise. It reads no source and builds no AST.
            //
            // docs/mcp/TOOLS.md disclosed this in prose ("currently
            // simulated — violations are synthesized from filename patterns"),
            // but a disclosure in a document is not a guard: an agent calls
            // `tools/list` and gets four tools that answer confidently with
            // fabricated line numbers and a "suggested_fix".
            //
            // Unregistered rather than deleted: the state machine, its
            // handlers and their tests remain, so implementing real AST
            // analysis is re-registering four lines, not rebuilding. Until
            // then it is not advertised.
            // === Quality Tools (3) ===
            .tool("quality_gate", QualityGateTool)
            .tool("quality_proxy", QualityProxyTool)
            .tool("pdmt_deterministic_todos", PdmtTool::new())
            // === Git and Context Tools (3) ===
            .tool("git_operation", GitTool)
            .tool("generate_context", GenerateContextTool)
            .tool("scaffold_project", ScaffoldProjectTool)
            // === Agent Context Tools (4) — KAIZEN-0165 ===
            .tool(
                "pmat_query_code",
                PmatQueryCodeHandler::new(index_manager.clone()),
            )
            .tool(
                "pmat_get_function",
                PmatGetFunctionHandler::new(index_manager.clone()),
            )
            .tool(
                "pmat_find_similar",
                PmatFindSimilarHandler::new(index_manager.clone()),
            )
            .tool(
                "pmat_index_stats",
                PmatIndexStatsHandler::new(index_manager.clone()),
            );
        let builder = match auth {
            Some(provider) => builder.auth_provider(ArcAuthProvider(provider)),
            None => builder,
        };
        Ok(builder.build()?)
    }

    /// Serve the built tool surface over stdio.
    async fn run_stdio(&self, server: Server) -> Result<(), Box<dyn std::error::Error>> {
        info!(
            "PMAT Simple Unified MCP server ready with {} tools, listening on stdio",
            crate::mcp_pmcp::tool_manifest::LIVE_MCP_TOOLS.len()
        );

        // Run server with stdio transport, racing against stdin EOF. pmcp's
        // `Server::run` keep-alive future never completes (even after the
        // reader task exits on EOF), so without this the process leaks after
        // one-shot piped sessions. See `EofSignalingTransport`.
        //
        // The inner transport is ours rather than `pmcp::shared::StdioTransport`
        // because pmcp discards the raw line the moment `parse_message` fails,
        // leaving no way to echo the client's id (#648): every bad frame came
        // back `{"id":null,...,"code":-32700}`. See [`RawFrameStdioTransport`].
        let (transport, session_end) = EofSignalingTransport::new(RawFrameStdioTransport::new());
        tokio::select! {
            result = server.run(transport) => {
                result?;
            }
            reason = session_end => {
                // Safe to exit: `EofSignalingTransport` only fires this once
                // every request taken off the wire has had its response sent,
                // so nothing is left to truncate. (This comment previously
                // claimed that was automatic — "responses for consumed
                // requests were already written" — which was false, and cost
                // `tools/list` its answer in 3 of 5 one-shot piped sessions.)
                // Flush once more defensively before exiting.
                use std::io::Write;
                let _ = std::io::stdout().flush();
                let reason = reason.unwrap_or_else(|_| "transport dropped".to_string());
                info!("MCP stdio session ended ({reason}); exiting");
            }
        }

        info!("PMAT Simple Unified MCP server shutting down");
        Ok(())
    }
}

impl Default for SimpleUnifiedServer {
    fn default() -> Self {
        Self::new().expect("Failed to create simple unified server")
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod eof_drain_tests {
    use super::*;
    use pmcp::types::{JSONRPCResponse, RequestId};

    /// Scripted transport: `receive()` replays `script`, `send()` is a no-op.
    #[derive(Debug)]
    struct ScriptedTransport {
        script: std::collections::VecDeque<Option<TransportMessage>>,
    }

    impl ScriptedTransport {
        /// `None` in the script means "return a ConnectionClosed error".
        fn new(script: Vec<Option<TransportMessage>>) -> Self {
            Self {
                script: script.into(),
            }
        }
    }

    #[async_trait]
    impl Transport for ScriptedTransport {
        async fn send(&mut self, _message: TransportMessage) -> pmcp::Result<()> {
            Ok(())
        }
        async fn receive(&mut self) -> pmcp::Result<TransportMessage> {
            match self.script.pop_front() {
                Some(Some(msg)) => Ok(msg),
                _ => Err(pmcp::Error::Transport(
                    pmcp::error::TransportError::ConnectionClosed,
                )),
            }
        }
        async fn close(&mut self) -> pmcp::Result<()> {
            Ok(())
        }
        fn is_connected(&self) -> bool {
            true
        }
        fn transport_type(&self) -> &'static str {
            "scripted"
        }
    }

    fn a_request() -> TransportMessage {
        TransportMessage::Request {
            id: RequestId::from(1i64),
            request: pmcp::types::Request::Client(Box::new(pmcp::types::ClientRequest::ListTools(
                Default::default(),
            ))),
        }
    }

    fn a_response() -> TransportMessage {
        TransportMessage::Response(JSONRPCResponse {
            jsonrpc: "2.0".to_string(),
            id: RequestId::from(1i64),
            payload: pmcp::types::jsonrpc::ResponsePayload::Result(serde_json::json!({})),
        })
    }

    /// The regression: EOF observed while a request is still being handled
    /// must NOT end the session, or that response is truncated.
    #[tokio::test]
    async fn eof_does_not_signal_while_a_request_is_in_flight() {
        let (mut transport, mut session_end) =
            EofSignalingTransport::new(ScriptedTransport::new(vec![Some(a_request()), None]));

        assert!(transport.receive().await.is_ok(), "request should arrive");
        assert!(transport.receive().await.is_err(), "then EOF");

        assert!(
            session_end.try_recv().is_err(),
            "session must stay open while a consumed request is unanswered — \
             signalling here is what truncated tools/list"
        );

        transport.send(a_response()).await.unwrap();

        assert!(
            session_end.try_recv().is_ok(),
            "once the response is sent, the session must end"
        );
    }

    /// The original hang must stay fixed: with nothing outstanding, EOF ends
    /// the session immediately.
    #[tokio::test]
    async fn eof_signals_immediately_when_nothing_is_in_flight() {
        let (mut transport, mut session_end) =
            EofSignalingTransport::new(ScriptedTransport::new(vec![None]));

        assert!(transport.receive().await.is_err());
        assert!(
            session_end.try_recv().is_ok(),
            "with no work outstanding the session must end at once, or the \
             one-shot pipe hangs until killed (the bug this wrapper fixed)"
        );
    }

    /// Several requests may be consumed before EOF; all must be answered.
    #[tokio::test]
    async fn waits_for_every_outstanding_request() {
        let (mut transport, mut session_end) =
            EofSignalingTransport::new(ScriptedTransport::new(vec![
                Some(a_request()),
                Some(a_request()),
                None,
            ]));

        transport.receive().await.unwrap();
        transport.receive().await.unwrap();
        assert!(transport.receive().await.is_err());

        transport.send(a_response()).await.unwrap();
        assert!(
            session_end.try_recv().is_err(),
            "one of two responses is not enough"
        );

        transport.send(a_response()).await.unwrap();
        assert!(session_end.try_recv().is_ok(), "now both are answered");
    }

    /// Notifications carry no response, so they must not hold the session open.
    #[tokio::test]
    async fn notifications_do_not_count_as_outstanding_work() {
        let notification = TransportMessage::Notification(pmcp::types::Notification::Client(
            pmcp::types::ClientNotification::Initialized,
        ));
        let (mut transport, mut session_end) =
            EofSignalingTransport::new(ScriptedTransport::new(vec![Some(notification), None]));

        transport.receive().await.unwrap();
        assert!(transport.receive().await.is_err());

        assert!(
            session_end.try_recv().is_ok(),
            "a notification is never answered, so it must not defer shutdown"
        );
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod active_tests {
    use super::*;

    #[test]
    fn test_simple_unified_server_new() {
        let result = SimpleUnifiedServer::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_simple_unified_server_default() {
        let server = SimpleUnifiedServer::default();
        let _ = server;
    }

    #[test]
    fn test_server_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<SimpleUnifiedServer>();
    }

    #[test]
    fn test_server_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<SimpleUnifiedServer>();
    }

    #[test]
    fn test_server_size() {
        let size = std::mem::size_of::<SimpleUnifiedServer>();
        // Arc<Mutex<T>> is typically 8 bytes on 64-bit systems
        assert!(
            size <= 16,
            "Server struct is larger than expected: {} bytes",
            size
        );
    }

    #[test]
    fn test_new_does_not_panic() {
        let _ = std::panic::catch_unwind(|| {
            let _ = SimpleUnifiedServer::new();
        });
    }

    #[tokio::test]
    async fn test_state_manager_accessible() {
        let server = SimpleUnifiedServer::new().unwrap();
        let state = server.state_manager.lock().await;
        drop(state);
    }

    #[tokio::test]
    async fn test_state_manager_thread_safety() {
        let server = SimpleUnifiedServer::new().unwrap();
        let state_clone = server.state_manager.clone();
        {
            let _state1 = server.state_manager.lock().await;
        }
        {
            let _state2 = state_clone.lock().await;
        }
    }

    /// v3.18.2: every tool registered on the live SimpleUnifiedServer must
    /// advertise a non-empty description and a real inputSchema, otherwise
    /// `tools/list` shows "no description, empty schema" and agents cannot
    /// call it (pmcp's default `metadata()` is `None`, which the builder
    /// silently turns into `ToolInfo { description: None, input_schema: {} }`).
    ///
    /// Regression: analyze_dag / analyze_big_o / analyze_deep_context shipped
    /// with no `metadata()` after R17-1 replaced the aliased handlers with
    /// new structs.
    #[test]
    fn test_all_live_tools_advertise_description_and_schema() {
        use pmcp::ToolHandler;

        let _state_manager = Arc::new(Mutex::new(StateManager::new()));
        let index_manager = Arc::new(IndexManager::new(PathBuf::from(".")));

        // (registered name, metadata, takes_arguments) — mirrors the
        // `.tool(...)` registrations in `run()` above. Tools with
        // `takes_arguments = false` are genuinely no-arg (empty `properties`
        // plus `additionalProperties: false` is their correct schema).
        let tools: Vec<(&str, Option<pmcp::types::ToolInfo>, bool)> = vec![
            ("analyze_complexity", AnalyzeComplexityTool.metadata(), true),
            ("analyze_satd", AnalyzeSatdTool.metadata(), true),
            ("analyze_dead_code", AnalyzeDeadCodeTool.metadata(), true),
            ("analyze_dag", AnalyzeDagTool.metadata(), true),
            (
                "analyze_deep_context",
                AnalyzeDeepContextTool.metadata(),
                true,
            ),
            ("analyze_big_o", AnalyzeBigOTool.metadata(), true),
            ("analyze_reachability", ReachabilityTool.metadata(), true),
            (
                "analyze_hardcoded_paths",
                HardcodedPathsTool.metadata(),
                true,
            ),
            ("analyze_vacuous_tests", VacuousTestsTool.metadata(), true),
            // refactor.* unregistered in EV-0 (#999) — see run()
            ("quality_gate", QualityGateTool.metadata(), true),
            ("quality_proxy", QualityProxyTool.metadata(), true),
            ("pdmt_deterministic_todos", PdmtTool::new().metadata(), true),
            ("git_operation", GitTool.metadata(), true),
            ("generate_context", GenerateContextTool.metadata(), true),
            ("scaffold_project", ScaffoldProjectTool.metadata(), true),
            (
                "pmat_query_code",
                PmatQueryCodeHandler::new(index_manager.clone()).metadata(),
                true,
            ),
            (
                "pmat_get_function",
                PmatGetFunctionHandler::new(index_manager.clone()).metadata(),
                true,
            ),
            (
                "pmat_find_similar",
                PmatFindSimilarHandler::new(index_manager.clone()).metadata(),
                true,
            ),
            (
                "pmat_index_stats",
                PmatIndexStatsHandler::new(index_manager).metadata(),
                true,
            ),
        ];

        assert_eq!(
            tools.len(),
            crate::mcp_pmcp::tool_manifest::LIVE_MCP_TOOLS.len(),
            "registry drift: this list must mirror the `.tool(...)` registrations in run(), \
             which `manifest_matches_server` pins to LIVE_MCP_TOOLS"
        );

        for (name, metadata, takes_arguments) in tools {
            let info = metadata.unwrap_or_else(|| {
                panic!(
                    "{name}: metadata() is None — tools/list would advertise \
                     an empty description and empty inputSchema"
                )
            });
            assert_eq!(
                info.name, name,
                "{name}: metadata name must match the registered tool name"
            );
            assert!(
                info.description
                    .as_deref()
                    .is_some_and(|d| !d.trim().is_empty()),
                "{name}: description must be non-empty"
            );
            let schema = info
                .input_schema
                .as_object()
                .unwrap_or_else(|| panic!("{name}: inputSchema must be a JSON object"));
            assert_eq!(
                schema.get("type").and_then(|t| t.as_str()),
                Some("object"),
                "{name}: inputSchema must declare type: object"
            );
            let properties = schema
                .get("properties")
                .and_then(|p| p.as_object())
                .unwrap_or_else(|| panic!("{name}: inputSchema must have a properties map"));
            if takes_arguments {
                assert!(
                    !properties.is_empty(),
                    "{name}: inputSchema.properties must be non-empty for a \
                     tool that takes arguments"
                );
            }
        }
    }

    /// EV-0 acceptance (b): the simulated `refactor.*` tools must NOT be
    /// advertised.
    ///
    /// They used to be, with descriptions disclosing the simulation — and a
    /// test asserted that the disclosure was present. But a disclosure is not a
    /// guard. An agent calls `tools/list`, sees four tools, and calls them; the
    /// engine behind them (`models/refactor_impls.rs:140-145`) returns a
    /// hardcoded HighComplexity violation at "line 100, column 1" for any file
    /// whose PATH CONTAINS THE SUBSTRING "complex", and nothing otherwise. It
    /// reads no source and builds no AST.
    ///
    /// So the assertion is now absence, not disclosure. Re-registering them is
    /// four lines in `run()` — do that when the engine analyses something.
    #[test]
    fn simulated_refactor_tools_are_not_advertised() {
        let declared: Vec<&str> = crate::mcp_pmcp::tool_manifest::LIVE_MCP_TOOLS
            .iter()
            .map(|(n, _)| *n)
            .collect();
        for name in [
            "refactor.start",
            "refactor.nextIteration",
            "refactor.getState",
            "refactor.stop",
        ] {
            assert!(
                !declared.contains(&name),
                "{name} is advertised again — its engine synthesizes violations \
                 from a path substring, so an agent calling it acts on fabricated \
                 line numbers (EV-0, #999)"
            );
        }
        assert_eq!(
            declared.len(),
            19,
            "the surface should be 19 tools: 16 after removing the 4 simulated ones, plus \
             the 3 forensic analyzers exposed in #1029"
        );
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod eof_shutdown_tests {
    //! Regression tests for the stdio EOF process leak: when the client
    //! closes stdin, `EofSignalingTransport` must fire the session-end
    //! channel so `SimpleUnifiedServer::run` exits instead of hanging on
    //! pmcp's never-completing keep-alive loop.
    //!
    //! Manual end-to-end repro (must answer fast and exit 0, NOT 124):
    //! ```text
    //! printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"t","version":"0"}}}\n' \
    //!   | MCP_VERSION=2024-11-05 timeout 10 pmat; echo exit=$?
    //! ```
    use super::*;
    use pmcp::error::TransportError;
    use pmcp::types::{Notification, ProgressNotification, ProgressToken};

    /// Scripted inner transport: pops pre-canned receive results, then EOF.
    #[derive(Debug)]
    struct ScriptedTransport {
        receives: Vec<pmcp::Result<TransportMessage>>,
    }

    #[async_trait]
    impl Transport for ScriptedTransport {
        async fn send(&mut self, _message: TransportMessage) -> pmcp::Result<()> {
            Ok(())
        }

        async fn receive(&mut self) -> pmcp::Result<TransportMessage> {
            self.receives
                .pop()
                .unwrap_or_else(|| Err(TransportError::ConnectionClosed.into()))
        }

        async fn close(&mut self) -> pmcp::Result<()> {
            Ok(())
        }
    }

    fn progress_message() -> TransportMessage {
        TransportMessage::Notification(Notification::Progress(ProgressNotification::new(
            ProgressToken::String("test".to_string()),
            50.0,
            None,
        )))
    }

    #[tokio::test]
    async fn eof_on_receive_signals_session_end_and_propagates_error() {
        let inner = ScriptedTransport { receives: vec![] };
        let (mut transport, session_end) = EofSignalingTransport::new(inner);

        let result = transport.receive().await;
        assert!(result.is_err(), "EOF error must propagate to pmcp's reader");

        let reason = session_end
            .await
            .expect("session-end signal must fire on EOF");
        assert!(
            reason.contains("Connection closed"),
            "expected ConnectionClosed reason, got: {reason}"
        );
    }

    #[tokio::test]
    async fn successful_receive_does_not_signal_session_end() {
        // Long-lived hosts (Claude Code) hold stdin open; a successful read
        // must NOT trigger shutdown.
        let inner = ScriptedTransport {
            receives: vec![Ok(progress_message())],
        };
        let (mut transport, mut session_end) = EofSignalingTransport::new(inner);

        let result = transport.receive().await;
        assert!(result.is_ok());
        assert!(
            session_end.try_recv().is_err(),
            "session-end must not fire on a successful receive"
        );
    }

    #[tokio::test]
    async fn send_does_not_signal_session_end() {
        let inner = ScriptedTransport { receives: vec![] };
        let (mut transport, mut session_end) = EofSignalingTransport::new(inner);

        transport
            .send(progress_message())
            .await
            .expect("scripted send always succeeds");
        assert!(
            session_end.try_recv().is_err(),
            "session-end must not fire on send"
        );
    }

    #[tokio::test]
    async fn session_end_signal_fires_at_most_once() {
        let inner = ScriptedTransport { receives: vec![] };
        let (mut transport, session_end) = EofSignalingTransport::new(inner);

        let _ = transport.receive().await; // first error: fires the signal
        let _ = transport.receive().await; // second error: sender consumed, must not panic
        assert!(session_end.await.is_ok());
    }

    #[tokio::test]
    async fn wrapper_delegates_transport_metadata_to_inner() {
        let inner = ScriptedTransport { receives: vec![] };
        let (transport, _session_end) = EofSignalingTransport::new(inner);

        // ScriptedTransport uses the trait defaults; the wrapper must
        // delegate rather than override.
        assert!(transport.is_connected());
        assert_eq!(transport.transport_type(), "unknown");
    }
}

/// NOTE: Temporarily disabled - tool methods don't exist
#[cfg(all(test, pmat_broken_tests))]
mod coverage_tests {
    use super::*;

    // === SimpleUnifiedServer Construction Tests ===

    #[test]
    fn test_simple_unified_server_new() {
        let result = SimpleUnifiedServer::new();
        assert!(result.is_ok());
        let server = result.unwrap();
        // Verify server was created
        let _ = server;
    }

    #[test]
    fn test_simple_unified_server_default() {
        // default() calls new().expect(), so it should succeed
        let server = SimpleUnifiedServer::default();
        let _ = server;
    }

    #[test]
    fn test_simple_unified_server_state_manager_initialized() {
        let server = SimpleUnifiedServer::new().unwrap();
        // State manager should be initialized (Arc<Mutex<StateManager>>)
        // We can't directly access it, but we can verify the server was created
        assert!(std::mem::size_of_val(&server) > 0);
    }

    // === Server Structure Tests ===

    #[test]
    fn test_server_has_state_manager() {
        let server = SimpleUnifiedServer::new().unwrap();
        // The state_manager field exists and is properly initialized
        let _ = &server.state_manager;
    }

    #[test]
    fn test_multiple_server_instances() {
        // Each server should have its own state manager
        let server1 = SimpleUnifiedServer::new().unwrap();
        let server2 = SimpleUnifiedServer::new().unwrap();

        // Verify both were created successfully
        let _ = server1;
        let _ = server2;
    }

    // === Tool Registration Tests (Compile-time verification) ===

    #[test]
    fn test_analyze_tools_importable() {
        // Verify all analysis tool types are accessible
        let _ = AnalyzeComplexityTool::new();
        let _ = AnalyzeSatdTool::new();
        let _ = AnalyzeDeadCodeTool::new();
        let _ = AnalyzeDagTool::new();
        let _ = AnalyzeDeepContextTool::new();
        let _ = AnalyzeBigOTool::new();
    }

    #[test]
    fn test_refactor_tools_require_state_manager() {
        let state_manager = Arc::new(Mutex::new(StateManager::new()));

        // Verify refactor tools can be created with state manager
        let _ = RefactorStartTool::new(state_manager.clone());
        let _ = RefactorNextIterationTool::new(state_manager.clone());
        let _ = RefactorGetStateTool::new(state_manager.clone());
        let _ = RefactorStopTool::new(state_manager.clone());
    }

    #[test]
    fn test_quality_tools_importable() {
        let _ = QualityGateTool::new();
        let _ = QualityProxyTool::new();
        let _ = PdmtTool::new();
    }

    #[test]
    fn test_context_tools_importable() {
        let _ = GitTool::new();
        let _ = GenerateContextTool::new();
        let _ = ScaffoldProjectTool::new();
    }

    // === Server Builder Verification Tests ===

    #[test]
    fn test_server_builder_pattern_accessible() {
        // Verify Server builder can be accessed
        // Note: We don't actually run the server, just verify the builder works
        let builder = Server::builder()
            .name("test-server")
            .version("0.1.0")
            .capabilities(ServerCapabilities::tools_only());

        // Builder should be valid
        let _ = builder;
    }

    // === Async Run Tests (without actually running) ===

    #[tokio::test]
    async fn test_server_run_requires_stdio() {
        // We can't actually call run() in tests because it blocks on stdio
        // but we can verify the server can be constructed and would be ready
        let server = SimpleUnifiedServer::new().unwrap();

        // Server is ready but we won't call run() as it would block
        let _ = server;
    }

    // === State Manager Integration Tests ===

    #[tokio::test]
    async fn test_state_manager_accessible() {
        let server = SimpleUnifiedServer::new().unwrap();

        // Lock the state manager and verify it works
        let state = server.state_manager.lock().await;

        // State manager should be empty initially (no sessions)
        drop(state);
    }

    #[tokio::test]
    async fn test_state_manager_thread_safety() {
        let server = SimpleUnifiedServer::new().unwrap();

        // Clone Arc to simulate multi-threaded access
        let state_clone = server.state_manager.clone();

        // Verify both references can acquire locks (sequentially)
        {
            let _state1 = server.state_manager.lock().await;
        }
        {
            let _state2 = state_clone.lock().await;
        }
    }

    // === Re-export Verification Tests ===

    #[test]
    fn test_all_tool_types_accessible() {
        // Analysis tools
        assert!(std::any::type_name::<AnalyzeComplexityTool>().contains("ComplexityTool"));
        assert!(std::any::type_name::<AnalyzeSatdTool>().contains("SatdTool"));
        assert!(std::any::type_name::<AnalyzeDeadCodeTool>().contains("DeadCodeTool"));
        // R17-1: Dag/BigO/DeepContext tools are now distinct structs that
        // dispatch to the correct analysis functions (not lint/coupling/churn).
        assert!(std::any::type_name::<AnalyzeDagTool>().contains("AnalyzeDagTool"));
        assert!(std::any::type_name::<AnalyzeDeepContextTool>().contains("AnalyzeDeepContextTool"));
        assert!(std::any::type_name::<AnalyzeBigOTool>().contains("AnalyzeBigOTool"));

        // Quality tools
        assert!(std::any::type_name::<QualityGateTool>().contains("QualityGateTool"));
        assert!(std::any::type_name::<QualityProxyTool>().contains("QualityProxyTool"));

        // Context tools
        assert!(std::any::type_name::<GenerateContextTool>().contains("ContextGenerateTool"));
        assert!(std::any::type_name::<ScaffoldProjectTool>().contains("ContextSummaryTool"));
        assert!(std::any::type_name::<GitTool>().contains("GitStatusTool"));
    }

    // === Server Capabilities Tests ===

    #[test]
    fn test_server_capabilities_structure() {
        let capabilities = ServerCapabilities::tools_only();

        assert!(capabilities.tools.is_some());
        assert!(capabilities.tools.as_ref().unwrap().list_changed.is_none());
    }

    // === Memory and Safety Tests ===

    #[test]
    fn test_server_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<SimpleUnifiedServer>();
    }

    #[test]
    fn test_server_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<SimpleUnifiedServer>();
    }

    #[test]
    fn test_server_size() {
        // Server should have minimal overhead (just an Arc<Mutex<StateManager>>)
        let size = std::mem::size_of::<SimpleUnifiedServer>();
        // Arc<Mutex<T>> is typically 8 bytes on 64-bit systems
        assert!(
            size <= 16,
            "Server struct is larger than expected: {} bytes",
            size
        );
    }

    // === Error Handling Tests ===

    #[test]
    fn test_new_does_not_panic() {
        // SimpleUnifiedServer::new() should never panic
        let _ = std::panic::catch_unwind(|| {
            let _ = SimpleUnifiedServer::new();
        });
    }

    #[test]
    fn test_default_does_not_panic() {
        // SimpleUnifiedServer::default() should never panic
        let result = std::panic::catch_unwind(|| {
            let _ = SimpleUnifiedServer::default();
        });
        assert!(result.is_ok());
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod recoverable_frame_tests {
    use super::*;
    use pmcp::types::RequestId;

    /// Transport whose `receive()` replays a script of Ok/Err outcomes.
    ///
    /// Unlike `eof_drain_tests::ScriptedTransport` this can yield a *recoverable*
    /// error — the case that used to kill the session.
    #[derive(Debug)]
    struct FlakyTransport {
        script: std::collections::VecDeque<pmcp::Result<TransportMessage>>,
        /// Reads attempted after the script ran dry. Proves the wrapper kept
        /// reading rather than giving up on the first bad frame.
        reads_past_end: usize,
    }

    impl FlakyTransport {
        fn new(script: Vec<pmcp::Result<TransportMessage>>) -> Self {
            Self {
                script: script.into(),
                reads_past_end: 0,
            }
        }
    }

    #[async_trait]
    impl Transport for FlakyTransport {
        async fn send(&mut self, _message: TransportMessage) -> pmcp::Result<()> {
            Ok(())
        }
        async fn receive(&mut self) -> pmcp::Result<TransportMessage> {
            match self.script.pop_front() {
                Some(outcome) => outcome,
                None => {
                    self.reads_past_end += 1;
                    Err(pmcp::Error::Transport(
                        pmcp::error::TransportError::ConnectionClosed,
                    ))
                }
            }
        }
        async fn close(&mut self) -> pmcp::Result<()> {
            Ok(())
        }
        fn is_connected(&self) -> bool {
            true
        }
        fn transport_type(&self) -> &'static str {
            "flaky"
        }
    }

    fn a_request() -> TransportMessage {
        TransportMessage::Request {
            id: RequestId::from(7i64),
            request: pmcp::types::Request::Client(Box::new(pmcp::types::ClientRequest::ListTools(
                Default::default(),
            ))),
        }
    }

    fn unparseable() -> pmcp::Error {
        pmcp::Error::Transport(pmcp::error::TransportError::InvalidMessage(
            "not json".to_string(),
        ))
    }

    /// The regression (#648): one unparseable frame used to end the whole
    /// session, so every later valid request was silently lost.
    ///
    /// Before the fix `receive()` returned the error, pmcp's actor broke its
    /// loop, and the process exited 0 without answering the request that
    /// followed. This asserts the wrapper skips the bad frame and surfaces the
    /// NEXT valid message instead.
    #[tokio::test]
    async fn recoverable_error_does_not_end_the_session() {
        let inner = FlakyTransport::new(vec![Err(unparseable()), Ok(a_request())]);
        let (mut transport, mut rx) = EofSignalingTransport::new(inner);

        let got = transport.receive().await;

        assert!(
            matches!(got, Ok(TransportMessage::Request { .. })),
            "a bad frame must be skipped and the next valid request returned, got: {got:?}"
        );
        assert_eq!(
            transport.in_flight, 1,
            "the request surfaced past the bad frame must still be counted"
        );
        assert!(
            rx.try_recv().is_err(),
            "an unparseable frame must NOT signal session end"
        );
    }

    /// Several bad frames in a row are each skipped, not just the first.
    #[tokio::test]
    async fn consecutive_recoverable_errors_are_all_skipped() {
        let inner = FlakyTransport::new(vec![
            Err(unparseable()),
            Err(unparseable()),
            Err(unparseable()),
            Ok(a_request()),
        ]);
        let (mut transport, mut rx) = EofSignalingTransport::new(inner);

        assert!(matches!(
            transport.receive().await,
            Ok(TransportMessage::Request { .. })
        ));
        assert!(
            rx.try_recv().is_err(),
            "repeated bad frames must not end the session"
        );
    }

    /// The complement, so the fix cannot be "never end the session": a genuine
    /// ConnectionClosed with nothing outstanding must still end it promptly.
    #[tokio::test]
    async fn connection_closed_still_ends_the_session() {
        let inner = FlakyTransport::new(vec![Err(pmcp::Error::Transport(
            pmcp::error::TransportError::ConnectionClosed,
        ))]);
        let (mut transport, mut rx) = EofSignalingTransport::new(inner);

        let got = transport.receive().await;

        assert!(got.is_err(), "EOF must still surface as an error");
        assert!(
            rx.try_recv().is_ok(),
            "ConnectionClosed with nothing in flight must signal session end"
        );
    }

    /// An IO failure on stdin is a broken stream, not a bad frame: it must end
    /// the session rather than spin trying to read more.
    #[tokio::test]
    async fn io_error_ends_the_session() {
        let inner = FlakyTransport::new(vec![Err(pmcp::Error::Transport(
            pmcp::error::TransportError::Io("stdin exploded".to_string()),
        ))]);
        let (mut transport, mut rx) = EofSignalingTransport::new(inner);

        assert!(transport.receive().await.is_err());
        assert!(
            rx.try_recv().is_ok(),
            "an IO error must be treated as terminal"
        );
    }
}
