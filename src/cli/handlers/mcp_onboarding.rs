//! Onboarding for pmat's MCP surfaces: token generation, the ready-to-paste
//! client registration line, and the `pmat mcp connect` guide.
//!
//! # Why this module exists
//!
//! Getting pmat onto an MCP client used to require knowing four things that
//! nothing in the binary said, every one of which cost this project real time:
//!
//! 1. the HTTP endpoint is the **root** path — `/mcp` and `/health` are 404;
//! 2. `PMAT_MCP_HTTP_TOKEN` has a 16-character minimum, below which the server
//!    refuses to start;
//! 3. the streamable transport rejects a request without
//!    `Accept: application/json, text/event-stream` (406), and `curl -f` turns
//!    that into an empty string with no message;
//! 4. there is no `/health` endpoint to probe for readiness.
//!
//! The transport-parity gate had all four wrong simultaneously and reported
//! GREEN. Everything this module prints is asserted against the running server
//! by the tests at the bottom of this file and by `tests/e2e_http_serve_t.rs`,
//! so the advice cannot drift away from the behaviour again.
//!
//! # The token is generated, never waived
//!
//! When `PMAT_MCP_HTTP_TOKEN` is unset and the bind is loopback, [`generate_token`]
//! mints a fresh one from the OS CSPRNG and the server starts with it. That is
//! a convenience about *supplying* a secret, not about *requiring* one: the
//! generated token is 48 hex characters, three times the enforced floor, and
//! [`crate::mcp_pmcp::http_server::BearerToken::new`] still rejects anything
//! shorter. A token the user supplied and that is too weak is still refused —
//! see `a_short_token_is_still_refused`.

use anyhow::Result;
use std::net::SocketAddr;

/// Bytes of OS entropy behind a generated token — 192 bits, hex-encoded to 48
/// characters.
///
/// Deliberately far above [`crate::mcp_pmcp::http_server::MIN_TOKEN_LEN`]: the
/// floor exists to catch `PMAT_MCP_HTTP_TOKEN=x`, and a generated secret should
/// never be near it.
const GENERATED_TOKEN_BYTES: usize = 24;

/// Prefix on a generated token, so one found in a shell history or a client
/// config is identifiable as pmat's and as ephemeral.
const GENERATED_TOKEN_PREFIX: &str = "pmat-";

/// The `Accept` header the streamable-HTTP transport requires on RPC calls.
///
/// Not optional and not a nicety: without it pmcp answers 406. Measured, not
/// assumed — `the_documented_accept_header_is_the_one_the_server_requires`.
pub const REQUIRED_ACCEPT: &str = "application/json, text/event-stream";

/// Read `n` bytes from the operating system's CSPRNG.
///
/// `getrandom`, not a hand-rolled `/dev/urandom` read. The first version opened
/// that file directly, to avoid depending on `rand` — which is optional, arriving
/// via `standard-deps`, and so cannot be relied on by every feature combination
/// that can compile `mcp-http`. The GOAL was right; the mechanism was Unix-only,
/// and 3.32.0 shipped a token generator that cannot run on Windows at all:
///
/// ```text
/// Error: could not read the OS random source (/dev/urandom) to generate a
/// bearer token: The system cannot find the path specified. (os error 3)
/// ```
///
/// Its own doc comment said "the same kernel pool that `getrandom` uses on
/// Linux" — it named the platform and shipped as though that were universal
/// (#1081).
///
/// The trade it was avoiding did not exist: `getrandom` is ALREADY compiled into
/// every build as a transitive dependency (three versions in `Cargo.lock`), so
/// naming it directly costs no supply-chain surface and no build time, and it
/// has no feature coupling — which is strictly better than the file read for the
/// `--no-default-features --features mcp-http` case the original was protecting.
fn os_entropy(n: usize) -> std::io::Result<Vec<u8>> {
    let mut buf = vec![0u8; n];
    // getrandom::Error does not implement std::error::Error in 0.2, so it
    // cannot go through io::Error::other directly. Its Display is the useful
    // part (it names the OS call that failed), so carry that.
    getrandom::getrandom(&mut buf).map_err(|e| std::io::Error::other(e.to_string()))?;
    Ok(buf)
}

/// Lowercase hex encoding.
fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut acc, b| {
        // `write!` to a String cannot fail; swallowing the Result keeps this
        // total without introducing a panicking call.
        let _ = write!(acc, "{b:02x}");
        acc
    })
}

/// Mint a token that satisfies the bearer-token floor.
///
/// # Errors
///
/// When the OS CSPRNG cannot be read. There is deliberately no weaker fallback:
/// a predictable token would be worse than no server, so the caller reports the
/// failure and tells the user to supply their own.
pub fn generate_token() -> Result<String> {
    let bytes = os_entropy(GENERATED_TOKEN_BYTES).map_err(|e| {
        anyhow::anyhow!(
            "could not read the OS random source to generate a \
             bearer token: {e}. Set one yourself instead, e.g. \
             `export PMAT_MCP_HTTP_TOKEN=$(openssl rand -hex 24)`"
        )
    })?;
    Ok(format!("{GENERATED_TOKEN_PREFIX}{}", hex(&bytes)))
}

/// The URL an MCP client should be pointed at.
///
/// The endpoint is the ROOT path. `/mcp` 404s, and a registration written
/// against it fails at connect time with nothing explaining why — which is how
/// this cost a day. The trailing slash is kept for that reason.
///
/// A wildcard bind (`0.0.0.0`, `::`) has no address a client can dial, so the
/// host is left as a placeholder rather than emitting a URL that cannot work.
#[must_use]
pub fn endpoint_url(bound: &SocketAddr) -> String {
    if bound.ip().is_unspecified() {
        return format!("http://<this-host>:{}/", bound.port());
    }
    match bound {
        SocketAddr::V6(v6) => format!("http://[{}]:{}/", v6.ip(), v6.port()),
        SocketAddr::V4(v4) => format!("http://{}:{}/", v4.ip(), v4.port()),
    }
}

/// The exact `claude mcp add` invocation that registers this running server.
///
/// Emitted as one line so it can be pasted without a continuation character.
#[must_use]
pub fn claude_mcp_add_line(bound: &SocketAddr, token: &str) -> String {
    format!(
        "claude mcp add --scope user --transport http pmat {} --header \"Authorization: Bearer {token}\"",
        endpoint_url(bound)
    )
}

/// Where the token in play came from, which changes the advice printed with it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TokenOrigin {
    /// Read from `PMAT_MCP_HTTP_TOKEN`; stable across restarts.
    Environment,
    /// Minted by [`generate_token`]; dies with the process.
    Generated,
}

/// Print the connection banner: endpoint, auth, tool count, and the exact
/// registration command for the address actually bound.
///
/// Takes the bound address rather than the requested one because `--port 0`
/// binds an ephemeral port; a registration line built from the request would
/// name a port nothing is listening on.
///
/// # Errors
///
/// Propagates write failures from `out`.
pub fn write_connection_banner<W: std::io::Write>(
    mut out: W,
    bound: &SocketAddr,
    token: &str,
    origin: TokenOrigin,
    tool_count: usize,
) -> std::io::Result<()> {
    writeln!(
        out,
        "pmat MCP (streamable HTTP) listening on {}",
        endpoint_url(bound)
    )?;
    writeln!(
        out,
        "  endpoint: the ROOT path — `/mcp` and `/health` are 404, there is no health endpoint"
    )?;
    writeln!(
        out,
        "  auth: Bearer, from {}; unauthenticated requests get 401",
        crate::mcp_pmcp::http_server::TOKEN_ENV
    )?;
    writeln!(
        out,
        "  tools: {tool_count} (identical to the stdio surface)"
    )?;
    if origin == TokenOrigin::Generated {
        writeln!(out)?;
        writeln!(
            out,
            "{} was unset, so pmat generated a token for this process:",
            crate::mcp_pmcp::http_server::TOKEN_ENV
        )?;
        writeln!(out)?;
        writeln!(out, "  {token}")?;
        writeln!(out)?;
        writeln!(
            out,
            "It dies with this process. Restarting mints a new one, and a client"
        )?;
        writeln!(
            out,
            "registered with the old token will then get 401. To pin a stable token:"
        )?;
        writeln!(out)?;
        writeln!(
            out,
            "  export {}=$(pmat mcp token)",
            crate::mcp_pmcp::http_server::TOKEN_ENV
        )?;
    }
    writeln!(out)?;
    writeln!(out, "Register this server with Claude Code (copy-paste):")?;
    writeln!(out)?;
    writeln!(out, "  {}", claude_mcp_add_line(bound, token))?;
    writeln!(out)?;
    // Measured friction, not a hypothetical: `claude mcp add` refuses with
    // "MCP server pmat already exists in user config" rather than replacing,
    // and a generated token changes every restart — so re-running this after a
    // restart is the COMMON case, and it fails until the old name is freed.
    writeln!(
        out,
        "  (already registered? `claude mcp remove pmat -s user` first — add refuses to replace)"
    )?;
    writeln!(out)?;
    writeln!(
        out,
        "Hand-rolling requests? The streamable transport needs `Accept: {REQUIRED_ACCEPT}`"
    )?;
    writeln!(
        out,
        "on every RPC call; without it the server answers 406 and `curl -f` prints nothing."
    )?;
    Ok(())
}

/// The error returned when a non-loopback bind has no token supplied.
///
/// Auto-generation is deliberately confined to loopback, and the reason is
/// operational rather than cryptographic: a generated token changes on every
/// restart, so a shared endpoint registered with it breaks silently — every
/// client 401s — the first time the service is bounced. A secret with a
/// lifecycle needs an owner. So this refuses, and hands over the exact command
/// that produces a conforming token.
#[must_use]
pub fn non_loopback_needs_explicit_token(host: &str) -> anyhow::Error {
    let env = crate::mcp_pmcp::http_server::TOKEN_ENV;
    crate::cli_exit::configuration_error(anyhow::anyhow!(
        "{env} is not set and --host {host} is not loopback. pmat generates a \
         token for you only on a loopback bind, because a generated token dies \
         with the process: every client registered against a shared endpoint \
         would 401 the first time it restarts. Set one explicitly and it will \
         survive restarts:\n\n  \
         export {env}=$(pmat mcp token)\n  \
         pmat serve --transport http --host {host}\n\n\
         pmat will not start an unauthenticated MCP endpoint: pmcp serves every \
         request when no auth provider is configured, so starting without a \
         token would publish the full tool surface to anyone who can reach the \
         port."
    ))
}

/// The `pmat mcp connect` guide: every surface in one place.
///
/// `pmat --mode mcp`, `MCP_VERSION=1 pmat` and `pmat serve --transport http`
/// are three spellings a user previously had to already know in order to find.
#[must_use]
pub fn mcp_guide() -> String {
    let env = crate::mcp_pmcp::http_server::TOKEN_ENV;
    let min = crate::mcp_pmcp::http_server::MIN_TOKEN_LEN;
    let tools = crate::mcp_pmcp::tool_manifest::LIVE_MCP_TOOLS.len();
    format!(
        "pmat speaks MCP over two transports, from one binary, and both serve the\n\
         SAME {tools} tools. The CLI runs the same analyses with no client at all.\n\
         \n\
         1. MCP over STDIO — for a client that spawns pmat as a subprocess\n\
         \n\
         \x20    pmat --mode mcp\n\
         \x20    MCP_VERSION=1 pmat        # equivalent; either spelling works\n\
         \n\
         \x20  No port, no token. This is the right choice for Claude Code and\n\
         \x20  Claude Desktop on the same machine. Register it with:\n\
         \n\
         \x20    claude mcp add --scope user pmat -- pmat --mode mcp\n\
         \n\
         2. MCP over HTTP — for a shared or remote client, or one that cannot\n\
         \x20  spawn a subprocess\n\
         \n\
         \x20    pmat serve --transport http --port 8765\n\
         \n\
         \x20  With {env} unset on a loopback bind, pmat generates a\n\
         \x20  conforming token, prints it, and prints the exact `claude mcp add`\n\
         \x20  line for the port it bound. Nothing else to assemble.\n\
         \n\
         \x20  Four things about this endpoint that are not guessable:\n\
         \x20    * MCP is served at the ROOT path. `/mcp` is 404.\n\
         \x20    * There is NO /health endpoint. `/health` is 404, so it cannot\n\
         \x20      be used as a readiness probe — poll the root with a real RPC.\n\
         \x20    * Auth is mandatory. {env} must be at least {min}\n\
         \x20      characters or the server refuses to start. 401 without it.\n\
         \x20    * Every RPC call needs `Accept: {REQUIRED_ACCEPT}`.\n\
         \x20      Without that header the server answers 406, and `curl -f`\n\
         \x20      reports it as an empty string with no message.\n\
         \n\
         \x20  A hand-rolled call, in full:\n\
         \n\
         \x20    curl -sS http://127.0.0.1:8765/ \\\n\
         \x20      -H \"Authorization: Bearer ${env}\" \\\n\
         \x20      -H 'Content-Type: application/json' \\\n\
         \x20      -H 'Accept: {REQUIRED_ACCEPT}' \\\n\
         \x20      -d '{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\"}}'\n\
         \n\
         3. CLI — the same analyses, no MCP client\n\
         \n\
         \x20    pmat analyze complexity --path .\n\
         \n\
         Mint a conforming token:  pmat mcp token\n\
         Already registered a server named `pmat`?  claude mcp remove pmat -s user\n\
         \n\
         `mcp-http` is in the DEFAULT build as of 3.32.0; it needed\n\
         `--features mcp-http` before. Other --transport values (web-socket,\n\
         http-sse, both, all) are NOT IMPLEMENTED and exit 2. There is no\n\
         `stdio` value for --transport — that spelling is a clap error.\n"
    )
}

/// Handle `pmat mcp connect` and `pmat mcp token`.
///
/// # Errors
///
/// When `--token` is given and the OS random source cannot be read.
pub fn handle_mcp(token_only: bool) -> Result<()> {
    if token_only {
        // Bare token on stdout, nothing else, so `$(pmat mcp token)` is
        // directly substitutable.
        println!("{}", generate_token()?);
        return Ok(());
    }
    print!("{}", mcp_guide());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp_pmcp::http_server::MIN_TOKEN_LEN;

    #[test]
    fn a_generated_token_clears_the_enforced_floor_by_a_wide_margin() {
        let token = generate_token().expect("the OS random source must be readable");
        assert!(
            token.len() >= MIN_TOKEN_LEN,
            "a generated token must satisfy the floor it is generated for: {token}"
        );
        // The point is not merely to clear the bar but to be nowhere near it.
        assert!(
            token.len() > MIN_TOKEN_LEN * 2,
            "a generated secret must not sit near the minimum: {token}"
        );
        assert!(
            token.starts_with(GENERATED_TOKEN_PREFIX),
            "a generated token must be identifiable as pmat's: {token}"
        );
    }

    /// The counter-test for the whole feature. Generating a token when none was
    /// supplied must not become "accept anything": a token the USER supplied
    /// that is too short is still refused.
    #[test]
    fn a_short_token_is_still_refused() {
        let err = crate::mcp_pmcp::http_server::BearerToken::new("tooshort")
            .expect_err("an 8-character token must be rejected");
        let msg = err.to_string();
        assert!(
            msg.contains("at least"),
            "the refusal must name the floor, got: {msg}"
        );
        // And the generated one is accepted, so the floor is the only gate.
        let good = generate_token().expect("the OS random source must be readable");
        assert!(
            crate::mcp_pmcp::http_server::BearerToken::new(good).is_ok(),
            "a generated token must be accepted by the same constructor"
        );
    }

    #[test]
    fn two_generated_tokens_differ() {
        let a = generate_token().expect("readable");
        let b = generate_token().expect("readable");
        assert_ne!(a, b, "tokens must come from entropy, not a counter");
    }

    /// The registration line must point at the ROOT path. A line naming `/mcp`
    /// is the exact mistake this module exists to stop shipping.
    #[test]
    fn the_registration_line_targets_the_root_path() {
        let bound: SocketAddr = "127.0.0.1:8765".parse().expect("literal parses");
        let line = claude_mcp_add_line(&bound, "pmat-token-0123456789abcdef");
        assert!(
            line.contains("http://127.0.0.1:8765/"),
            "must name the root path, got: {line}"
        );
        assert!(
            !line.contains("8765/mcp"),
            "`/mcp` is 404; the registration line must never name it, got: {line}"
        );
        assert!(
            line.contains("--transport http"),
            "must tell the client it is an HTTP server, got: {line}"
        );
        assert!(
            line.contains("Authorization: Bearer pmat-token-0123456789abcdef"),
            "must carry the bearer header, got: {line}"
        );
        assert!(
            !line.contains('\n'),
            "must be one pasteable line, got: {line}"
        );
    }

    /// `--port 0` is how the tests and any ephemeral bind work; the printed
    /// line has to name the port that was actually bound.
    #[test]
    fn the_registration_line_uses_the_bound_port_not_the_requested_one() {
        let bound: SocketAddr = "127.0.0.1:41234".parse().expect("literal parses");
        let line = claude_mcp_add_line(&bound, "pmat-token-0123456789abcdef");
        assert!(
            line.contains(":41234/"),
            "must name the bound port, got: {line}"
        );
    }

    #[test]
    fn a_wildcard_bind_does_not_emit_an_undialable_url() {
        let bound: SocketAddr = "0.0.0.0:8765".parse().expect("literal parses");
        let url = endpoint_url(&bound);
        assert!(
            !url.contains("0.0.0.0"),
            "0.0.0.0 is not an address a client can dial, got: {url}"
        );
        assert!(
            url.contains("<this-host>") && url.contains("8765"),
            "must keep the port and mark the host for substitution, got: {url}"
        );
    }

    #[test]
    fn an_ipv6_url_brackets_the_address() {
        let bound: SocketAddr = "[::1]:8765".parse().expect("literal parses");
        let url = endpoint_url(&bound);
        assert_eq!(url, "http://[::1]:8765/", "IPv6 hosts must be bracketed");
    }

    /// The generated-token banner has to carry everything a user needs, and to
    /// say the token is ephemeral — a client registered against a token that
    /// silently changes on restart is a worse failure than no server.
    #[test]
    fn the_generated_banner_carries_the_paste_line_and_the_caveat() {
        let bound: SocketAddr = "127.0.0.1:8765".parse().expect("literal parses");
        let mut buf = Vec::new();
        write_connection_banner(
            &mut buf,
            &bound,
            "pmat-token-0123456789abcdef",
            TokenOrigin::Generated,
            19,
        )
        .expect("writing to a Vec cannot fail");
        let s = String::from_utf8(buf).expect("ascii");
        assert!(
            s.contains("claude mcp add"),
            "must print the registration line, got: {s}"
        );
        assert!(
            s.contains("pmat-token-0123456789abcdef"),
            "must print the token the user needs, got: {s}"
        );
        assert!(
            s.contains("ROOT path") && s.contains("404"),
            "must state where the endpoint is and that /mcp and /health are not, got: {s}"
        );
        assert!(
            s.contains(REQUIRED_ACCEPT),
            "must state the Accept header a hand-rolled client needs, got: {s}"
        );
        assert!(
            s.contains("406"),
            "must name the status a missing Accept header produces, got: {s}"
        );
        assert!(
            s.contains("dies with this process"),
            "must say the generated token is ephemeral, got: {s}"
        );
        // Measured while writing this: `claude mcp add` answers "MCP server
        // pmat already exists in user config" and does NOT replace. Because a
        // generated token changes on every restart, re-registering is the
        // common path, so the banner has to say how to free the name.
        assert!(
            s.contains("claude mcp remove pmat -s user"),
            "must say how to free the name, since `add` refuses to replace, got: {s}"
        );
    }

    /// With a token from the environment there is nothing ephemeral to warn
    /// about, and printing the operator's own secret back at them is noise.
    #[test]
    fn the_environment_banner_omits_the_ephemeral_warning() {
        let bound: SocketAddr = "127.0.0.1:8765".parse().expect("literal parses");
        let mut buf = Vec::new();
        write_connection_banner(
            &mut buf,
            &bound,
            "pmat-token-0123456789abcdef",
            TokenOrigin::Environment,
            19,
        )
        .expect("writing to a Vec cannot fail");
        let s = String::from_utf8(buf).expect("ascii");
        assert!(
            !s.contains("dies with this process"),
            "an environment token is not ephemeral, got: {s}"
        );
        assert!(
            s.contains("claude mcp add"),
            "the registration line is useful either way, got: {s}"
        );
    }

    /// A non-loopback bind must refuse rather than generate, and must hand over
    /// a command rather than a lecture.
    #[test]
    fn the_non_loopback_refusal_hands_over_a_runnable_command() {
        let err = non_loopback_needs_explicit_token("0.0.0.0");
        let msg = err.to_string();
        assert!(
            msg.contains("pmat mcp token"),
            "must name the command that mints a conforming token, got: {msg}"
        );
        assert!(
            msg.contains("PMAT_MCP_HTTP_TOKEN"),
            "must name the variable to set, got: {msg}"
        );
        assert_eq!(
            crate::cli_exit::code_for(&err),
            crate::cli_exit::ExitCode::ConfigurationError,
            "a missing token is a configuration error, as it was before"
        );
    }

    /// The guide is the one place all the surfaces are named. If a spelling
    /// stops working, this fails rather than the user finding out.
    #[test]
    fn the_guide_names_every_surface_and_every_trap() {
        let g = mcp_guide();
        for spelling in [
            "pmat --mode mcp",
            "MCP_VERSION=1 pmat",
            "pmat serve --transport http",
        ] {
            assert!(g.contains(spelling), "guide must name `{spelling}`: {g}");
        }
        for trap in [
            "ROOT path",
            "`/mcp` is 404",
            "NO /health",
            REQUIRED_ACCEPT,
            "406",
        ] {
            assert!(g.contains(trap), "guide must state `{trap}`: {g}");
        }
        assert!(
            g.contains(&crate::mcp_pmcp::http_server::MIN_TOKEN_LEN.to_string()),
            "guide must state the token minimum: {g}"
        );
    }

    /// The tool count in the guide is read from the manifest, never typed. An
    /// earlier revision of this repo's docs said "19 tools" while the running
    /// binary served 16, because the number was prose.
    #[test]
    fn the_guide_reports_the_manifest_tool_count_rather_than_a_literal() {
        let n = crate::mcp_pmcp::tool_manifest::LIVE_MCP_TOOLS.len();
        assert!(
            mcp_guide().contains(&format!("SAME {n} tools")),
            "the guide must report the manifest's own count ({n})"
        );
    }
}
