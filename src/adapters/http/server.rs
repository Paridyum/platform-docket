//! The listener: binding conditionally on auth, one thread per request.
//!
//! ADR-0093 §decision-2 moved docket's write concurrency from a single-writer file lock into the
//! database: the served backend is thread-per-request, and correctness under concurrent writers is
//! the `Store` port's atomic, advisory-locked seal-append (`adapters::pg`), not this listener's
//! serialisation. So the "one request at a time" loop that protected the file chains is retired.

use crate::adapters::http::request;
use crate::adapters::http::response::Response;
use crate::adapters::http::router::{self, Context};
use anyhow::{Context as _, Result, bail};
use std::io::Write;
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::time::Duration;

/// How long a connection may take to send its request, or to read its response.
///
/// Without this a client that opens a socket and says nothing holds a thread
/// forever, and enough of them stop the server accepting anything. Five seconds
/// is generous for a request travelling over loopback or an SSH forward.
const IO_TIMEOUT: Duration = Duration::from_secs(5);

/// Refuse a routable bind unless the server can authenticate its callers.
///
/// This is the design §5 invariant made mechanical. A loopback bind is always allowed: SSH keys are
/// the authentication and the operating system is the identity, which holds while the socket cannot
/// be reached from off the machine. A ROUTABLE bind is allowed ONLY when a token verifier is present
/// (`has_verifier`): the served backend authenticates every write from a control-plane operator
/// token (ADR-0092), so it may be reached from another machine. A routable bind with no verifier is
/// refused — that is a server with no authentication of its own exposed to the network, which is the
/// exact failure ADR-0072's loopback rule existed to prevent.
///
/// # Errors
/// Fails if `addr` is routable and no verifier is present.
pub fn permit_bind(addr: IpAddr, has_verifier: bool) -> Result<()> {
    if addr.is_loopback() || has_verifier {
        return Ok(());
    }
    bail!(
        "refusing to bind {addr}: this backend has no token verifier, so it has no authentication \
         of its own and must not be reachable from another machine. Either bind a loopback address \
         and reach it over an SSH forward (ssh -L 8080:127.0.0.1:8080 <host>), or configure the \
         OIDC verifier so the served backend can authenticate operators (ADR-0092)."
    )
}

/// Serve until the process is stopped, a thread per accepted connection.
///
/// # Errors
/// Fails if the bind is refused ([`permit_bind`]) or the port cannot be bound.
pub fn serve(ctx: &Context<'_>, addr: SocketAddr) -> Result<()> {
    permit_bind(addr.ip(), ctx.verifier.is_some())?;

    let listener = TcpListener::bind(addr)
        .with_context(|| format!("could not bind {addr}; is another docket already serving?"))?;

    // Printed rather than logged: an operator starts this in the foreground and watches it.
    println!("docket serving on http://{addr}");
    if ctx.verifier.is_some() {
        println!("authenticating every write from a control-plane operator token (ADR-0092)");
    } else {
        println!("no token verifier: loopback only, SSH is the authentication (ADR-0054)");
        println!(
            "reach it from another machine with:  ssh -L {p}:127.0.0.1:{p} <this-host>",
            p = addr.port()
        );
    }

    // One thread per connection. Concurrent writers are safe because the `Store` port seals every
    // append atomically under a per-chain lock (ADR-0093 §decision-2); this listener no longer
    // carries that responsibility. A scoped thread borrows `ctx` for the life of the accept loop, so
    // no `'static`/`Arc` is forced on the handler wiring.
    std::thread::scope(|scope| {
        for incoming in listener.incoming() {
            match incoming {
                Ok(stream) => {
                    scope.spawn(move || {
                        if let Err(e) = handle(ctx, stream) {
                            // A failed connection must never stop the server. The usual cause is a
                            // client that hung up mid-request, which is not worth ending over.
                            eprintln!("connection error: {e:#}");
                        }
                    });
                }
                Err(e) => eprintln!("accept failed: {e:#}"),
            }
        }
    });
    Ok(())
}

/// Read one request, route it, write one response.
fn handle(ctx: &Context<'_>, mut stream: TcpStream) -> Result<()> {
    stream.set_read_timeout(Some(IO_TIMEOUT))?;
    stream.set_write_timeout(Some(IO_TIMEOUT))?;

    // A parse failure is answered with 400 rather than by dropping the connection.
    // A client that gets no answer cannot tell a malformed request from a crashed
    // server, and the difference is the first thing worth knowing.
    let response = match request::parse(&stream) {
        Ok(req) => router::route(ctx, &req),
        Err(e) => Response::error(400, &format!("{e:#}")),
    };

    response.write_to(&mut stream)?;
    stream.flush()?;
    Ok(())
}

#[cfg(test)]
// The lint configuration denies expect/unwrap/indexing for the crate, because a
// panic in a running tool is a defect. In a test an assertion IS the point, and
// the same convention is used by every other test module in this crate.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn loopback_binds_without_a_verifier() {
        assert!(permit_bind("127.0.0.1".parse().expect("v4"), false).is_ok());
        assert!(permit_bind("::1".parse().expect("v6"), false).is_ok());
        // The whole 127/8 range is loopback, not just 127.0.0.1.
        assert!(permit_bind("127.0.2.2".parse().expect("v4"), false).is_ok());
    }

    #[test]
    fn a_routable_bind_needs_a_verifier() {
        // Without a verifier a routable bind is refused, with the reason and the SSH alternative.
        let err = permit_bind("0.0.0.0".parse().expect("v4"), false).expect_err("refused");
        let text = format!("{err}");
        assert!(text.contains("no authentication"), "{text}");
        assert!(text.contains("ssh -L"), "{text}");
        // With a verifier the same routable bind is allowed — the served backend authenticates.
        assert!(permit_bind("0.0.0.0".parse().expect("v4"), true).is_ok());
    }

    #[test]
    fn a_lan_address_without_a_verifier_is_refused() {
        // The exact mistake console-docket's `next dev` makes, which is how this rule earned a test.
        assert!(permit_bind("10.12.9.25".parse().expect("v4"), false).is_err());
        assert!(permit_bind("192.168.1.4".parse().expect("v4"), false).is_err());
        // …but permitted once a verifier is configured.
        assert!(permit_bind("10.12.9.25".parse().expect("v4"), true).is_ok());
    }

    #[test]
    fn every_unspecified_address_without_a_verifier_is_refused() {
        assert!(permit_bind("0.0.0.0".parse().expect("v4"), false).is_err());
        assert!(permit_bind("::".parse().expect("v6"), false).is_err());
    }
}
