//! The listener: binding conditionally on auth, one thread per request.
//!
//! ADR-0093 §decision-2 moved docket's write concurrency from a single-writer file lock into the
//! database: the served backend is thread-per-request, and correctness under concurrent writers is
//! the `Store` port's atomic, advisory-locked seal-append (`adapters::pg`), not this listener's
//! serialisation. So the "one request at a time" loop that protected the file chains is retired.

use crate::adapters::auth::OperatorVerifier;
use crate::adapters::blobs::Blobs;
use crate::adapters::clock::Clock;
use crate::adapters::correlation::CorrelationId;
use crate::adapters::http::operator::AUTHORIZATION_HEADER;
use crate::adapters::http::request::{self, Request};
use crate::adapters::http::response::Response;
use crate::adapters::http::router::{self, Context};
use crate::adapters::registry::Registry;
use crate::adapters::store::Store;
use crate::config::Paths;
use anyhow::{Context as _, Result, bail};
use std::io::Write;
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::time::Duration;
use uuid::Uuid;

/// What one request resolves to before it is routed: its record store, its blob store, and the
/// tenant's configuration (`Some` when seeded, `None` to fall back to the git registry). The blob
/// store may borrow the factory, hence the lifetime.
pub type RequestStores<'a> = (Box<dyn Store>, Box<dyn Blobs + 'a>, Option<Registry>);

/// Builds the tenant-scoped record and blob stores for one request. The served, multi-tenant path
/// holds one of these (ADR-0099): the tenant comes from the operator token, so the store cannot be
/// built once at startup as it was single-tenant. The served binary implements it over the Postgres
/// pool and Garage; the CLI/loopback path uses a fixed store and never needs it.
pub trait TenantStores: Sync {
    /// The record store, blob store, and per-tenant configuration for `tenant`, for the life of one
    /// request. The record store owns its handles; the blob store may borrow the factory (a
    /// filesystem blob store references its paths — `blobs::open`), so it is tied to `&self`, which
    /// outlives every request in the loop. The config is `Some` when the tenant has been seeded
    /// (ADR-0099), and `None` when it has not — the dispatcher then falls back to the git registry.
    ///
    /// `correlation` is the request's id (ADR-0044 D1), read from its headers by the dispatcher and
    /// passed in rather than minted here: a store that minted its own would give one request an id
    /// nothing else in the estate shares, which is the failure D1 names.
    ///
    /// # Errors
    /// Fails if the stores cannot be opened (a bad pool or missing S3 configuration) or the config
    /// cannot be read.
    fn build(&self, tenant: Uuid, correlation: &CorrelationId) -> Result<RequestStores<'_>>;
}

/// How a request's record/blob stores are obtained.
pub enum Backend<'a> {
    /// One fixed store for one tenant — the loopback/break-glass backend and the CLI. No token; the
    /// attribution header and SSH stand in (design §5).
    Fixed {
        /// The record store (four sealed chains + the person document).
        store: &'a dyn Store,
        /// The blob store (issued PDFs, packs, scans).
        blobs: &'a dyn Blobs,
    },
    /// A store built per request from the operator token's tenant (ADR-0099). The served backend.
    PerTenant {
        /// Builds the record + blob stores for the request's derived tenant.
        stores: &'a dyn TenantStores,
    },
}

/// Everything the served loop needs. The tenant-independent bits are shared across every request;
/// the store is either fixed or built per request from the token (`Backend`).
pub struct Serving<'a> {
    /// How the request's record/blob stores are obtained (fixed, or per-tenant).
    pub backend: Backend<'a>,
    /// Filesystem artifacts (pack output, generated indexes, templates) not held in the stores.
    pub paths: &'a Paths,
    /// The registry (companies, operators, lifecycle) — loaded once; nothing here writes it.
    pub registry: &'a Registry,
    /// Verifies operator tokens (ADR-0092). `None` on the loopback/break-glass backend.
    pub verifier: Option<&'a OperatorVerifier>,
    /// Injected clock (STD-0004); `Sync` so it is shared across request threads.
    pub clock: &'a (dyn Clock + Sync),
    /// Whether an off-host backup destination exists and a restore has been rehearsed
    /// (`PARIDYUM_BACKUPS_CONFIRMED=1`). While false, every route that would write a person record
    /// refuses with 503 — ADR-0068 §5 and ADR-0093 §5, and migration 0006's own header.
    ///
    /// Resolved at startup and injected, never read from the environment inside a handler, for the
    /// reason stated on `clock`: a rule that reads its own configuration is a rule no test can put
    /// in the other position.
    pub backups_confirmed: bool,
}

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
pub fn serve(s: &Serving<'_>, addr: SocketAddr) -> Result<()> {
    permit_bind(addr.ip(), s.verifier.is_some())?;

    let listener = TcpListener::bind(addr)
        .with_context(|| format!("could not bind {addr}; is another docket already serving?"))?;

    // Printed rather than logged: an operator starts this in the foreground and watches it.
    println!("docket serving on http://{addr}");
    if s.verifier.is_some() {
        println!(
            "authenticating every request and deriving its tenant from the token (ADR-0092/0099)"
        );
    } else {
        println!("no token verifier: loopback only, SSH is the authentication (ADR-0054)");
        println!(
            "reach it from another machine with:  ssh -L {p}:127.0.0.1:{p} <this-host>",
            p = addr.port()
        );
    }

    // One thread per connection. Concurrent writers are safe because the `Store` port seals every
    // append atomically under a per-chain lock (ADR-0093 §decision-2); this listener no longer
    // carries that responsibility. A scoped thread borrows `s` for the life of the accept loop, so
    // no `'static`/`Arc` is forced on the handler wiring.
    std::thread::scope(|scope| {
        for incoming in listener.incoming() {
            match incoming {
                Ok(stream) => {
                    scope.spawn(move || {
                        if let Err(e) = handle(s, stream) {
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
fn handle(s: &Serving<'_>, mut stream: TcpStream) -> Result<()> {
    stream.set_read_timeout(Some(IO_TIMEOUT))?;
    stream.set_write_timeout(Some(IO_TIMEOUT))?;

    // A parse failure is answered with 400 rather than by dropping the connection.
    // A client that gets no answer cannot tell a malformed request from a crashed
    // server, and the difference is the first thing worth knowing.
    let response = match request::parse(&stream) {
        Ok(req) => dispatch(s, &req),
        Err(e) => Response::error(400, &format!("{e:#}")),
    };

    response.write_to(&mut stream)?;
    stream.flush()?;
    Ok(())
}

/// Obtain the request's store and route it. On the multi-tenant backend this verifies the operator
/// token, derives the tenant (ADR-0099), and builds the store for THAT tenant before the handlers
/// touch it — so RLS scopes every query and one deployment serves many tenants. On the fixed backend
/// the store is the one it was built with.
fn dispatch(s: &Serving<'_>, req: &Request) -> Response {
    match &s.backend {
        Backend::Fixed { store, blobs } => {
            let ctx = Context {
                store: *store,
                blobs: *blobs,
                paths: s.paths,
                registry: s.registry,
                verifier: s.verifier,
                clock: s.clock,
                backups_confirmed: s.backups_confirmed,
            };
            router::route(&ctx, req)
        }
        Backend::PerTenant { stores } => {
            // Health names no tenant and the k8s probe sends no token; answer it before the gate.
            if req.method == "GET" && req.segments().as_slice() == ["api", "health"] {
                return router::health();
            }
            let Some(verifier) = s.verifier else {
                return Response::error(500, "the multi-tenant backend has no verifier configured");
            };
            let Some(header) = req.header(AUTHORIZATION_HEADER) else {
                return Response::error(
                    401,
                    "this request carries no bearer token. The served backend derives the tenant \
                     and the operator from a control-plane token (ADR-0092/0099); send it as \
                     `Authorization: Bearer …`.",
                );
            };
            let tenant = match verifier.verify(header) {
                Ok(op) => op.tenant,
                Err(e) => return Response::error(401, &format!("{e:#}")),
            };
            // ADR-0044 D1: the id comes off the request, and is minted here only when the client
            // sent nothing usable. `req.header` matches case-insensitively (see request.rs), which is
            // what makes `traceparent` from any client reach this.
            let correlation = CorrelationId::from_headers(|name| req.header(name));
            let (store, blobs, config) = match stores.build(tenant, &correlation) {
                Ok(built) => built,
                Err(e) => {
                    return Response::error(
                        500,
                        &format!("could not open the tenant store: {e:#}"),
                    );
                }
            };
            // The tenant's own config if it is seeded (ADR-0099), else the git registry as a fallback
            // until it is — the line that makes the git registry a seed, not the source of truth.
            let registry = config.as_ref().unwrap_or(s.registry);
            let ctx = Context {
                store: store.as_ref(),
                blobs: blobs.as_ref(),
                paths: s.paths,
                registry,
                verifier: s.verifier,
                clock: s.clock,
                backups_confirmed: s.backups_confirmed,
            };
            router::route(&ctx, req)
        }
    }
}

#[cfg(test)]
// The lint configuration denies expect/unwrap/indexing for the crate, because a
// panic in a running tool is a defect. In a test an assertion IS the point, and
// the same convention is used by every other test module in this crate.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use crate::adapters::clock::SystemClock;
    use crate::adapters::registry::Registry;
    use crate::domain::event::Event;
    use crate::domain::identity::Record as IdentityRecord;
    use crate::domain::person::Person;
    use crate::domain::register::Entry;
    use crate::domain::signature::Record as Signature;
    use anyhow::bail;
    use jsonwebtoken::{DecodingKey, EncodingKey, Header, encode};
    use rsa::RsaPrivateKey;
    use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
    use serde_json::json;
    use std::collections::BTreeMap;
    use std::io::Read;
    use std::net::{TcpListener, TcpStream};

    const ISS: &str = "https://id.test";
    const AUD: &str = "ops-project";

    /// A record store that holds nothing — the routes exercised here (health, operators) never touch
    /// it, so every method answers empty rather than panicking.
    struct NullStore;
    impl Store for NullStore {
        fn person(&self, _: &str) -> Result<Person> {
            bail!("null store")
        }
        fn person_exists(&self, _: &str) -> Result<bool> {
            Ok(false)
        }
        fn slugs(&self) -> Result<Vec<String>> {
            Ok(Vec::new())
        }
        fn create_person(&self, _: &str, _: &str) -> Result<()> {
            Ok(())
        }
        fn rewrite_person(&self, _: &str, _: &str) -> Result<()> {
            Ok(())
        }
        fn ledger(&self) -> Result<Vec<IdentityRecord>> {
            Ok(Vec::new())
        }
        fn append_identity(&self, r: IdentityRecord) -> Result<IdentityRecord> {
            Ok(r)
        }
        fn register(&self) -> Result<Vec<Entry>> {
            Ok(Vec::new())
        }
        fn append_register(&self, e: Entry) -> Result<Entry> {
            Ok(e)
        }
        fn events(&self) -> Result<Vec<Event>> {
            Ok(Vec::new())
        }
        fn append_event(&self, e: Event) -> Result<Event> {
            Ok(e)
        }
        fn signatures(&self) -> Result<Vec<Signature>> {
            Ok(Vec::new())
        }
        fn append_signature(&self, s: Signature) -> Result<Signature> {
            Ok(s)
        }
        fn reseal_events(&self, _: Vec<Event>) -> Result<()> {
            Ok(())
        }
    }

    struct NullBlobs;
    impl Blobs for NullBlobs {
        fn put(&self, _: &str, _: &[u8]) -> Result<()> {
            Ok(())
        }
        fn get(&self, _: &str) -> Result<Vec<u8>> {
            bail!("null blobs")
        }
        fn exists(&self, _: &str) -> Result<bool> {
            Ok(false)
        }
        fn delete(&self, _: &str) -> Result<()> {
            Ok(())
        }
    }

    /// A factory that ignores the tenant and returns the null stores — enough to prove the served
    /// backend builds a store per request and routes through it.
    struct FakeStores;
    impl TenantStores for FakeStores {
        fn build(&self, _tenant: Uuid, _correlation: &CorrelationId) -> Result<RequestStores<'_>> {
            // No config: the dispatcher falls back to the git registry, which is what these tests
            // route against (GET /api/operators reads it).
            Ok((Box::new(NullStore), Box::new(NullBlobs), None))
        }
    }

    fn verifier_and_signer() -> (OperatorVerifier, EncodingKey) {
        let key = RsaPrivateKey::new(&mut rand::thread_rng(), 2048).expect("key");
        let enc =
            EncodingKey::from_rsa_pem(key.to_pkcs8_pem(LineEnding::LF).expect("priv").as_bytes())
                .expect("enc");
        let mut keys = BTreeMap::new();
        keys.insert(
            "k1".to_owned(),
            DecodingKey::from_rsa_pem(
                key.to_public_key()
                    .to_public_key_pem(LineEnding::LF)
                    .expect("pub")
                    .as_bytes(),
            )
            .expect("dec"),
        );
        (
            OperatorVerifier::new(keys, ISS.to_owned(), AUD.to_owned()),
            enc,
        )
    }

    fn operator_bearer(enc: &EncodingKey) -> String {
        let mut header = Header::new(jsonwebtoken::Algorithm::RS256);
        header.kid = Some("k1".to_owned());
        let claims = json!({
            "iss": ISS, "sub": "op-subject", "aud": AUD, "exp": 9_999_999_999u64,
            "urn:zitadel:iam:org:project:ops:roles": { "operator": {} },
            "urn:zitadel:iam:user:resourceowner:id": "389623583590057140",
        });
        format!("Bearer {}", encode(&header, &claims, enc).expect("sign"))
    }

    fn registry() -> Registry {
        Registry::load(&Paths::resolve(std::path::Path::new(env!(
            "CARGO_MANIFEST_DIR"
        ))))
        .expect("load the crate's registry")
    }

    fn parse_req(raw: &str) -> Request {
        request::parse(raw.as_bytes()).expect("parse")
    }

    #[test]
    fn fixed_backend_routes_to_the_store() {
        let reg = registry();
        let clock = SystemClock;
        let paths = Paths::resolve(std::path::Path::new("."));
        let (store, blobs) = (NullStore, NullBlobs);
        let s = Serving {
            backend: Backend::Fixed {
                store: &store,
                blobs: &blobs,
            },
            paths: &paths,
            registry: &reg,
            verifier: None,
            clock: &clock,
            backups_confirmed: true,
        };
        let resp = dispatch(
            &s,
            &parse_req("GET /api/health HTTP/1.1\r\nHost: x\r\n\r\n"),
        );
        assert_eq!(resp.status, 200);
    }

    fn pertenant<'a>(
        verifier: Option<&'a OperatorVerifier>,
        stores: &'a FakeStores,
        reg: &'a Registry,
        paths: &'a Paths,
        clock: &'a SystemClock,
    ) -> Serving<'a> {
        Serving {
            backend: Backend::PerTenant { stores },
            paths,
            registry: reg,
            verifier,
            clock,
            // The existing suite asserts routing and auth, not the data gate, so it runs with
            // backups confirmed. The gate's own tests set it false explicitly.
            backups_confirmed: true,
        }
    }

    #[test]
    fn health_is_public_on_the_multi_tenant_backend() {
        let (reg, clock, paths, stores) = (
            registry(),
            SystemClock,
            Paths::resolve(std::path::Path::new(".")),
            FakeStores,
        );
        let (verifier, _enc) = verifier_and_signer();
        let s = pertenant(Some(&verifier), &stores, &reg, &paths, &clock);
        // No token, yet health answers — the probe never carries one.
        assert_eq!(
            dispatch(&s, &parse_req("GET /api/health HTTP/1.1\r\n\r\n")).status,
            200
        );
    }

    #[test]
    fn a_request_without_a_token_is_401() {
        let (reg, clock, paths, stores) = (
            registry(),
            SystemClock,
            Paths::resolve(std::path::Path::new(".")),
            FakeStores,
        );
        let (verifier, _enc) = verifier_and_signer();
        let s = pertenant(Some(&verifier), &stores, &reg, &paths, &clock);
        assert_eq!(
            dispatch(&s, &parse_req("GET /api/operators HTTP/1.1\r\n\r\n")).status,
            401
        );
    }

    #[test]
    fn a_bad_token_is_401() {
        let (reg, clock, paths, stores) = (
            registry(),
            SystemClock,
            Paths::resolve(std::path::Path::new(".")),
            FakeStores,
        );
        let (verifier, _enc) = verifier_and_signer();
        let s = pertenant(Some(&verifier), &stores, &reg, &paths, &clock);
        let resp = dispatch(
            &s,
            &parse_req("GET /api/operators HTTP/1.1\r\nAuthorization: Bearer nonsense\r\n\r\n"),
        );
        assert_eq!(resp.status, 401);
    }

    #[test]
    fn a_multi_tenant_backend_without_a_verifier_is_500() {
        let (reg, clock, paths, stores) = (
            registry(),
            SystemClock,
            Paths::resolve(std::path::Path::new(".")),
            FakeStores,
        );
        let s = pertenant(None, &stores, &reg, &paths, &clock);
        assert_eq!(
            dispatch(&s, &parse_req("GET /api/operators HTTP/1.1\r\n\r\n")).status,
            500
        );
    }

    #[test]
    fn a_valid_token_builds_a_store_and_routes() {
        let (reg, clock, paths, stores) = (
            registry(),
            SystemClock,
            Paths::resolve(std::path::Path::new(".")),
            FakeStores,
        );
        let (verifier, enc) = verifier_and_signer();
        let s = pertenant(Some(&verifier), &stores, &reg, &paths, &clock);
        let raw = format!(
            "GET /api/operators HTTP/1.1\r\nAuthorization: {}\r\n\r\n",
            operator_bearer(&enc)
        );
        assert_eq!(dispatch(&s, &parse_req(&raw)).status, 200);
    }

    // ── The off-host backup gate (ADR-0064 D3, ADR-0068 §5, ADR-0093 §5) ────────────────────
    //
    // Migration 0006's header states the condition and, until 2026-09-20, nothing in docket
    // enforced it. `web-hiring` had enforced the identical rule on its apply route since it
    // shipped, so the gate held for the surface that COLLECTS an application and not for the one
    // that ENROLS a person.
    //
    // Each of these is watched in both positions. A 503 that is never observed turning into
    // something else is indistinguishable from a route that is simply broken.

    fn gated<'a>(
        stores: &'a FakeStores,
        reg: &'a Registry,
        paths: &'a Paths,
        clock: &'a SystemClock,
        verifier: Option<&'a OperatorVerifier>,
        backups_confirmed: bool,
    ) -> Serving<'a> {
        Serving {
            backend: Backend::PerTenant { stores },
            paths,
            registry: reg,
            verifier,
            clock,
            backups_confirmed,
        }
    }

    #[test]
    fn a_person_write_is_refused_while_the_backup_gate_is_shut() {
        let (reg, clock, paths, stores) = (
            registry(),
            SystemClock,
            Paths::resolve(std::path::Path::new(".")),
            FakeStores,
        );
        let (verifier, enc) = verifier_and_signer();
        let s = gated(&stores, &reg, &paths, &clock, Some(&verifier), false);

        // AUTHENTICATED, and that is the whole point. A tokenless request is 401 regardless of the
        // backups, so refusing one proves nothing about this gate. What must be true is that a
        // VERIFIED operator — someone with every right to enrol a person — is refused instead of
        // succeeding.
        let bearer = operator_bearer(&enc);
        for path in [
            "POST /api/person",
            "POST /api/person/x/onboard",
            "POST /api/person/x/sign",
            "POST /api/person/x/advance",
        ] {
            let raw = format!("{path} HTTP/1.1\r\nAuthorization: {bearer}\r\n\r\n");
            let resp = dispatch(&s, &parse_req(&raw));
            assert_eq!(resp.status, 503, "expected 503 for: {path}");
            let body = String::from_utf8_lossy(&resp.body);
            assert!(
                body.contains("PARIDYUM_BACKUPS_CONFIRMED"),
                "the refusal must name what would lift it, got: {body}"
            );
        }
    }

    #[test]
    fn opening_the_gate_stops_the_503() {
        // The other position, and the reason this test exists: with the gate open the SAME verified
        // request must stop being 503. That is what distinguishes "the gate refused it" from "this
        // route was never reachable".
        let (reg, clock, paths, stores) = (
            registry(),
            SystemClock,
            Paths::resolve(std::path::Path::new(".")),
            FakeStores,
        );
        let (verifier, enc) = verifier_and_signer();
        let raw = format!(
            "POST /api/person HTTP/1.1\r\nAuthorization: {}\r\n\r\n",
            operator_bearer(&enc)
        );

        let shut = gated(&stores, &reg, &paths, &clock, Some(&verifier), false);
        assert_eq!(dispatch(&shut, &parse_req(&raw)).status, 503);

        let open = gated(&stores, &reg, &paths, &clock, Some(&verifier), true);
        assert_ne!(
            dispatch(&open, &parse_req(&raw)).status,
            503,
            "the gate is open; 503 must not survive it"
        );
    }

    #[test]
    fn an_unauthenticated_write_is_401_not_503_even_with_the_gate_shut() {
        // Ordering, asserted rather than assumed. On the served backend the tenant is derived from
        // the token, so authentication necessarily precedes this gate — and that is correct: a
        // tokenless caller has not earned the right to learn the estate's operational state.
        let (reg, clock, paths, stores) = (
            registry(),
            SystemClock,
            Paths::resolve(std::path::Path::new(".")),
            FakeStores,
        );
        let (verifier, _enc) = verifier_and_signer();
        let s = gated(&stores, &reg, &paths, &clock, Some(&verifier), false);
        assert_eq!(
            dispatch(&s, &parse_req("POST /api/person HTTP/1.1\r\n\r\n")).status,
            401
        );
    }

    #[test]
    fn reads_are_unaffected_by_the_backup_gate() {
        // A gate that made the whole API inert would be turned off, and then it protects nothing.
        // Synthetic rows and every read stay available with the gate shut — which is exactly what
        // migration 0006 permits: "Schema and SYNTHETIC rows now".
        let (reg, clock, paths, stores) = (
            registry(),
            SystemClock,
            Paths::resolve(std::path::Path::new(".")),
            FakeStores,
        );
        let s = gated(&stores, &reg, &paths, &clock, None, false);
        assert_eq!(
            dispatch(&s, &parse_req("GET /api/health HTTP/1.1\r\n\r\n")).status,
            200
        );
    }

    #[test]
    fn handle_reads_one_request_and_writes_its_response() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        let reg = registry();
        let clock = SystemClock;
        let paths = Paths::resolve(std::path::Path::new("."));
        let (store, blobs) = (NullStore, NullBlobs);
        let s = Serving {
            backend: Backend::Fixed {
                store: &store,
                blobs: &blobs,
            },
            paths: &paths,
            registry: &reg,
            verifier: None,
            clock: &clock,
            backups_confirmed: true,
        };
        let client = std::thread::spawn(move || {
            let mut c = TcpStream::connect(addr).expect("connect");
            c.write_all(b"GET /api/health HTTP/1.1\r\nHost: x\r\n\r\n")
                .expect("write");
            let mut buf = Vec::new();
            c.read_to_end(&mut buf).expect("read");
            String::from_utf8_lossy(&buf).into_owned()
        });
        let (server_stream, _peer) = listener.accept().expect("accept");
        handle(&s, server_stream).expect("handle answers the request");
        let response = client.join().expect("client thread");
        assert!(response.contains("200"), "{response}");
    }

    #[test]
    fn serve_accepts_a_connection_and_answers_it() {
        // A free port: bind, read the port, drop the listener, hand the port to serve().
        let port = TcpListener::bind("127.0.0.1:0")
            .expect("bind")
            .local_addr()
            .expect("addr")
            .port();
        // `serve` never returns (the accept loop), so it runs on a spawned thread over leaked,
        // 'static data. The test process ends with the thread; nextest runs each test in its own
        // process, so nothing is left behind.
        let s: &'static Serving<'static> = Box::leak(Box::new(Serving {
            backend: Backend::Fixed {
                store: Box::leak(Box::new(NullStore)),
                blobs: Box::leak(Box::new(NullBlobs)),
            },
            paths: Box::leak(Box::new(Paths::resolve(std::path::Path::new(".")))),
            registry: Box::leak(Box::new(registry())),
            verifier: None,
            clock: Box::leak(Box::new(SystemClock)),
            backups_confirmed: true,
        }));
        std::thread::spawn(move || {
            let _ = serve(s, SocketAddr::from(([127, 0, 0, 1], port)));
        });
        // Busy-retry the connect (thread::sleep is disallowed): serve binds synchronously at the top,
        // so the socket is listening within a few attempts.
        let mut response = String::new();
        for _ in 0..10_000 {
            let Ok(mut c) = TcpStream::connect(("127.0.0.1", port)) else {
                continue;
            };
            c.write_all(b"GET /api/health HTTP/1.1\r\nHost: x\r\n\r\n")
                .expect("write");
            let mut buf = Vec::new();
            c.read_to_end(&mut buf).expect("read");
            response = String::from_utf8_lossy(&buf).into_owned();
            break;
        }
        assert!(response.contains("200"), "{response}");
    }

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
