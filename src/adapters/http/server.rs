//! The listener: binding loopback only, and one connection at a time.

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

/// Refuse to listen on anything but a loopback address.
///
/// This is the whole access-control model, so it is a function with tests rather
/// than a default someone can pass a flag around. §4 of the design: SSH keys are
/// the authentication and the operating system is the identity, which holds only
/// while the socket cannot be reached from off the machine. The moment this binds
/// a routable interface, the application has no authentication at all — every
/// caller becomes a trusted operator.
///
/// When an authentication adapter exists (ADR-0047, `platform-identity`), it is
/// added beside this rather than replacing it — ADR-0054.
///
/// # Errors
/// Fails if the address is not a loopback address.
pub fn require_loopback(addr: IpAddr) -> Result<()> {
    if addr.is_loopback() {
        return Ok(());
    }
    bail!(
        "refusing to bind {addr}: this server has no authentication of its own, so it \
         must not be reachable from another machine. Reach it over an SSH forward \
         instead: ssh -L 8080:127.0.0.1:8080 <host>"
    )
}

/// Serve until the process is stopped.
///
/// # Errors
/// Fails if the address is not loopback, or if the port cannot be bound.
pub fn serve(ctx: &Context<'_>, port: u16) -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    require_loopback(addr.ip())?;

    let listener = TcpListener::bind(addr)
        .with_context(|| format!("could not bind {addr}; is another docket already serving?"))?;

    // Printed rather than logged: this is a foreground process an operator starts
    // and watches, and the SSH command is the thing they actually need next.
    println!("docket serving on http://{addr}");
    println!("reach it from another machine with:  ssh -L {port}:127.0.0.1:{port} <this-host>");
    // D-41: the limit stated where an operator meets it, rather than only in a
    // ledger nobody reads at the moment a screen appears to hang. One connection
    // at a time is deliberate -- it is what keeps four append-only hash chains
    // safe without file locking -- and the visible symptom of a slow route is
    // indistinguishable from a crash unless it was said in advance.
    println!();
    println!("ONE REQUEST AT A TIME, deliberately: this process is the only writer to four");
    println!("append-only hash chains. A slow route -- issuing a pack runs the typesetter --");
    println!("blocks every other operator's screen until it finishes. That is waiting, not a");
    println!("hang. This process also holds the store lock, so CLI writes are refused while");
    println!("it runs (ADR-0072).");

    for incoming in listener.incoming() {
        match incoming {
            Ok(stream) => {
                // One connection at a time, deliberately.
                //
                // This process is the only writer to four append-only hash chains,
                // and that is what makes them safe without file locking (§4). A
                // thread per connection would make two concurrent writes possible
                // and a torn chain the failure. Three HR operators clicking
                // through screens do not need concurrency, and a serialised
                // server is one whose correctness is arguable from its shape.
                if let Err(e) = handle(ctx, stream) {
                    // A failed connection must never stop the server. The usual
                    // cause is a client that hung up mid-request, which is not an
                    // event worth ending the process over.
                    eprintln!("connection error: {e:#}");
                }
            }
            Err(e) => eprintln!("accept failed: {e:#}"),
        }
    }
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
    fn loopback_addresses_are_accepted() {
        assert!(require_loopback("127.0.0.1".parse().expect("v4")).is_ok());
        assert!(require_loopback("::1".parse().expect("v6")).is_ok());
        // The whole 127/8 range is loopback, not just 127.0.0.1.
        assert!(require_loopback("127.0.2.2".parse().expect("v4")).is_ok());
    }

    #[test]
    fn a_routable_address_is_refused_with_the_reason_and_the_alternative() {
        let err = require_loopback("0.0.0.0".parse().expect("v4")).expect_err("refused");
        let text = format!("{err}");
        assert!(text.contains("no authentication"), "{text}");
        assert!(text.contains("ssh -L"), "{text}");
    }

    #[test]
    fn a_lan_address_is_refused() {
        // The exact mistake console-docket's `next dev` makes, which is how this
        // rule earned its test.
        assert!(require_loopback("10.12.9.25".parse().expect("v4")).is_err());
        assert!(require_loopback("192.168.1.4".parse().expect("v4")).is_err());
    }

    #[test]
    fn every_unspecified_address_is_refused() {
        assert!(require_loopback("0.0.0.0".parse().expect("v4")).is_err());
        assert!(require_loopback("::".parse().expect("v6")).is_err());
    }
}
