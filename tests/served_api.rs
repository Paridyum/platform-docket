//! End to end: an authenticated operator creates and lists a person THROUGH the REAL served router,
//! the Postgres store and RLS (ADR-0093 + ADR-0092). This ties the token verifier, the pg adapter and
//! the `adapters::http` router together on the happy path — each piece has its own tests; this proves
//! they work as one, over the same router and wire contract the console speaks. It also checks the two
//! auth refusals the served path must make (missing token → 401, unknown subject → 403). Runs only
//! when `DOCKET_DATABASE_URL` is set (migrations 0001–0006 applied); otherwise it skips.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]

use jsonwebtoken::{DecodingKey, EncodingKey, Header, encode};
use rsa::RsaPrivateKey;
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use tool_docket::adapters::auth::OperatorVerifier;
use tool_docket::adapters::clock::{FixedClock, parse_iso};
use tool_docket::adapters::http::request;
use tool_docket::adapters::http::router::{self, Context};
use tool_docket::adapters::pg::{self, PgStore};
use tool_docket::adapters::registry::Registry;
use tool_docket::config::Paths;
use uuid::Uuid;

const ISS: &str = "https://id.test";
const AUD: &str = "ops-project";
// rohan's control-plane subject, bound in registry/operators.yaml (ADR-0092 §2).
const ROHAN_SUB: &str = "390506086174098663";

/// Sign an operator token for `subject` with the test key.
fn token(enc: &EncodingKey, subject: &str) -> String {
    let mut header = Header::new(jsonwebtoken::Algorithm::RS256);
    header.kid = Some("k1".to_owned());
    let claims = json!({
        "iss": ISS, "sub": subject, "aud": AUD, "exp": 9_999_999_999u64,
        "urn:zitadel:iam:org:project:ops:roles": { "operator": {} },
    });
    format!("Bearer {}", encode(&header, &claims, enc).expect("sign"))
}

/// Route one raw HTTP request through the real router and return (status, body).
fn call(ctx: &Context<'_>, raw: &str) -> (u16, Value) {
    let req = request::parse(raw.as_bytes()).expect("request parses");
    let resp = router::route(ctx, &req);
    let body = if resp.body.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&resp.body).expect("json body")
    };
    (resp.status, body)
}

#[test]
fn an_operator_creates_and_lists_a_person_through_the_real_router() {
    let Ok(url) = std::env::var("DOCKET_DATABASE_URL") else {
        eprintln!("skipping: DOCKET_DATABASE_URL not set");
        return;
    };

    // A process-owned multi-threaded runtime: `PgStore` block_on's on its handle from THIS thread,
    // which is outside the runtime, so the bridge is valid (never a nested runtime). The test is a
    // plain `#[test]` for exactly that reason — a `#[tokio::test]` would run the router on a runtime
    // worker and the block_on would panic.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("runtime");

    // A throwaway signing keypair; the verifier trusts its public half.
    let key = RsaPrivateKey::new(&mut rand::thread_rng(), 2048).expect("key");
    let enc = EncodingKey::from_rsa_pem(key.to_pkcs8_pem(LineEnding::LF).expect("priv").as_bytes())
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
    let verifier = OperatorVerifier::new(keys, ISS.to_owned(), AUD.to_owned());
    let paths = Paths::resolve(std::path::Path::new("."));
    let registry = Registry::load(&paths).expect("registry");
    let clock = FixedClock(parse_iso("2026-09-14").expect("date"));

    let pool = runtime.block_on(pg::pool(&url)).expect("connect");
    let tenant = Uuid::new_v4();
    runtime
        .block_on(
            sqlx::query(
                "INSERT INTO tenants (id, slug, name, zitadel_org_id) VALUES ($1, $2, $3, $4)",
            )
            .bind(tenant)
            .bind(format!("test-{tenant}"))
            .bind("Served e2e")
            .bind(tenant.simple().to_string())
            .execute(&pool),
        )
        .expect("seed tenant");

    let store = PgStore::new(pool, tenant, runtime.handle().clone());
    let ctx = Context {
        store: &store,
        paths: &paths,
        registry: &registry,
        verifier: Some(&verifier),
        clock: &clock,
    };

    // A write with NO token is 401 — the served path authenticates every write (ADR-0092), and the
    // handler never runs unauthenticated.
    let body = json!({ "slug": "served-test-person", "class": "intern" }).to_string();
    let (status, _) = call(
        &ctx,
        &format!(
            "POST /api/person HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        ),
    );
    assert_eq!(status, 401, "a write with no token is unauthenticated");

    // A valid token for a subject bound to NO operator is 403 — authenticated, but not permitted.
    let stranger = token(&enc, "no-such-subject");
    let (status, _) = call(
        &ctx,
        &format!(
            "POST /api/person HTTP/1.1\r\nAuthorization: {stranger}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        ),
    );
    assert_eq!(status, 403, "an unbound subject is forbidden");

    // A valid, bound operator token creates the person through the real enrol handler.
    let rohan = token(&enc, ROHAN_SUB);
    let (status, _) = call(
        &ctx,
        &format!(
            "POST /api/person HTTP/1.1\r\nAuthorization: {rohan}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        ),
    );
    assert_eq!(status, 200, "the operator opened a record");

    // The roster, read back through the real router, shows exactly that person for this tenant.
    let (status, roster) = call(
        &ctx,
        &format!("GET /api/roster HTTP/1.1\r\nAuthorization: {rohan}\r\n\r\n"),
    );
    assert_eq!(status, 200);
    let slugs: Vec<&str> = roster["people"]
        .as_array()
        .expect("people array")
        .iter()
        .filter_map(|p| p.get("slug").and_then(Value::as_str))
        .collect();
    assert_eq!(
        slugs,
        ["served-test-person"],
        "the created person is on the roster, and only them (RLS scopes to this tenant)"
    );
}
