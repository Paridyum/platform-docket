//! docket's records survive a full round trip THROUGH Postgres — the four sealed chains keep their
//! seal, the mutable person record upserts and reads back, and RLS isolates tenants (ADR-0093,
//! RFC-0039 D1). Runs only when `DOCKET_DATABASE_URL` points at a database with the tenancy migrations
//! applied; otherwise it says SKIPPED and returns, so `cargo test` needs no database.
//! `scripts/probe-audit-writer.sh` replays the tenancy migrations into a scratch database and runs it —
//! the full stream by default, or the subset `TENANCY_MIGRATIONS` names, which is how it proves docket
//! against the schema it is about to be deployed into.

// A test that cannot assert an expected value is not a test — the crate's expect/unwrap denial is
// exempted here exactly as it is inside the unit-test modules.
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]

mod common;

use platform_docket::adapters::correlation::CorrelationId;
use platform_docket::adapters::pg::{self, PgStore};
use platform_docket::adapters::store::Store;
use platform_docket::domain::chain::{seal, verify};
use platform_docket::domain::event::{Event, Kind};
use platform_docket::domain::identity::{EngagementId, PersonId, Record, Standing};
use platform_docket::domain::person::Person;
use platform_docket::domain::register::{Entry, verify_chain};
use platform_docket::domain::signature::{Method, Record as Signature};
use uuid::Uuid;

/// One id for the whole test, the way a CLI invocation has one (ADR-0044 D1). A fresh id per call
/// would be per-transaction minting, which is what the production paths are written not to do.
fn cid() -> CorrelationId {
    CorrelationId::mint()
}

fn entry(seq: u32) -> Entry {
    Entry {
        serial: format!("PAR/NDA/2026/{seq:04}"),
        code: "NDA".to_owned(),
        seq,
        year: 2026,
        person_id: PersonId::new(1).expect("a valid person id"),
        engagement_id: EngagementId::new("PAR", "INT", 2026, 1).expect("a valid engagement id"),
        slug: "asha-verma".to_owned(),
        document: "nda".to_owned(),
        title: "Non-Disclosure Agreement".to_owned(),
        kind: "signed".to_owned(),
        issued_on: "2026-08-19".to_owned(),
        retention: "contract".to_owned(),
        template_sha256: "a".repeat(64),
        pdf_sha256: "b".repeat(64),
        pdf_path: format!("issued/asha-verma/{seq}.pdf"),
        texlive: "2026".to_owned(),
        engine: None,
        engine_version: None,
        signed_by: vec!["cto".to_owned()],
        prev_sha256: String::new(),
        entry_sha256: String::new(),
    }
}

fn record(pseq: u32, standing: Standing) -> Record {
    Record {
        person_id: PersonId::new(pseq).expect("a valid person id"),
        engagement_id: EngagementId::new("PAR", "INT", 2026, 1).expect("a valid engagement id"),
        slug: "asha-verma".to_owned(),
        class: "intern".to_owned(),
        created_on: "2026-08-19".to_owned(),
        standing,
        standing_changed_on: "2026-08-19".to_owned(),
        prev_sha256: String::new(),
        entry_sha256: String::new(),
    }
}

fn evt(kind: Kind) -> Event {
    Event {
        person_id: PersonId::new(1).expect("a valid person id"),
        at: "2026-08-19".to_owned(),
        kind,
        detail: None,
        recorded_by: "cto".to_owned(),
        prev_sha256: String::new(),
        entry_sha256: String::new(),
    }
}

fn sig(serial: &str, method: Method) -> Signature {
    Signature {
        serial: serial.to_owned(),
        signed_on: "2026-09-01".to_owned(),
        method,
        scan_sha256: None,
        scan_path: None,
        received_on: "2026-09-02".to_owned(),
        recorded_by: "cto".to_owned(),
        prev_sha256: String::new(),
        entry_sha256: String::new(),
    }
}

fn as_json<T: serde::Serialize>(items: &[T]) -> Vec<serde_json::Value> {
    items
        .iter()
        .map(|e| serde_json::to_value(e).expect("serialise"))
        .collect()
}

/// The database URL if the test should run, or `None` to skip — saying so, because a skipped
/// integration suite that prints nothing reads exactly like a passing one.
fn url() -> Option<String> {
    let url = std::env::var("DOCKET_DATABASE_URL").ok();
    if url.is_none() {
        eprintln!(
            "SKIPPED pg_store — DOCKET_DATABASE_URL not set; scripts/probe-audit-writer.sh runs it"
        );
    }
    url
}

/// Seed a fresh tenant so each test is isolated and re-runnable against a persistent database.
async fn fresh_tenant(url: &str) -> Uuid {
    common::seed::seed_tenant(url, "Integration test").await
}

#[tokio::test]
async fn register_round_trips_through_postgres_with_the_seal_intact() {
    let Some(url) = url() else {
        return;
    };
    let pool = pg::pool(&url).await.expect("connect");
    let tenant = fresh_tenant(&url).await;

    let mut sealed: Vec<Entry> = Vec::new();
    for i in 1..=3 {
        let e = entry(i).seal(sealed.last()).expect("seal");
        pg::append_register(&pool, tenant, &cid(), &e)
            .await
            .expect("append");
        sealed.push(e);
    }

    let read = pg::read_register(&pool, tenant, &cid())
        .await
        .expect("read");
    assert_eq!(read.len(), 3);
    assert!(
        verify_chain(&read).expect("verifiable").is_empty(),
        "seal intact through Postgres"
    );
    assert_eq!(
        as_json(&read),
        as_json(&sealed),
        "records unchanged by Postgres"
    );

    let elsewhere = pg::read_register(&pool, fresh_tenant(&url).await, &cid())
        .await
        .expect("read other");
    assert!(elsewhere.is_empty(), "RLS isolates the register by tenant");
}

#[tokio::test]
async fn the_identity_chain_round_trips_through_postgres() {
    let Some(url) = url() else {
        return;
    };
    let pool = pg::pool(&url).await.expect("connect");
    let tenant = fresh_tenant(&url).await;

    let mut sealed: Vec<Record> = Vec::new();
    for (i, standing) in [Standing::Active, Standing::Active, Standing::Erased]
        .into_iter()
        .enumerate()
    {
        let r = seal(
            record(u32::try_from(i + 1).expect("small"), standing),
            sealed.last(),
        )
        .expect("seal");
        pg::append_identity(&pool, tenant, &cid(), &r)
            .await
            .expect("append");
        sealed.push(r);
    }

    let read = pg::read_identities(&pool, tenant, &cid())
        .await
        .expect("read");
    assert_eq!(read.len(), 3);
    assert!(
        verify(&read).expect("verifiable").is_empty(),
        "seal intact (incl. Standing enum)"
    );
    assert_eq!(as_json(&read), as_json(&sealed));
}

#[tokio::test]
async fn the_event_chain_round_trips_through_postgres() {
    let Some(url) = url() else {
        return;
    };
    let pool = pg::pool(&url).await.expect("connect");
    let tenant = fresh_tenant(&url).await;

    let mut sealed: Vec<Event> = Vec::new();
    for kind in [Kind::Created, Kind::StageChanged, Kind::Separated] {
        let e = seal(evt(kind), sealed.last()).expect("seal");
        pg::append_event(&pool, tenant, &cid(), &e)
            .await
            .expect("append");
        sealed.push(e);
    }

    let read = pg::read_events(&pool, tenant, &cid()).await.expect("read");
    assert_eq!(read.len(), 3);
    assert!(
        verify(&read).expect("verifiable").is_empty(),
        "seal intact (incl. Kind enum)"
    );
    assert_eq!(as_json(&read), as_json(&sealed));
}

#[tokio::test]
async fn the_signature_chain_round_trips_through_postgres() {
    let Some(url) = url() else {
        return;
    };
    let pool = pg::pool(&url).await.expect("connect");
    let tenant = fresh_tenant(&url).await;

    let mut sealed: Vec<Signature> = Vec::new();
    for (serial, method) in [
        ("PAR/NDA/2026/0001", Method::WetInk),
        ("PAR/IPA/2026/0001", Method::AadhaarEsign),
        ("PAR/OFR/2026/0001", Method::Electronic),
    ] {
        let s = seal(sig(serial, method), sealed.last()).expect("seal");
        pg::append_signature(&pool, tenant, &cid(), &s)
            .await
            .expect("append");
        sealed.push(s);
    }

    let read = pg::read_signatures(&pool, tenant, &cid())
        .await
        .expect("read");
    assert_eq!(read.len(), 3);
    assert!(
        verify(&read).expect("verifiable").is_empty(),
        "seal intact (incl. Method enum)"
    );
    assert_eq!(as_json(&read), as_json(&sealed));

    let elsewhere = pg::read_signatures(&pool, fresh_tenant(&url).await, &cid())
        .await
        .expect("read other");
    assert!(elsewhere.is_empty(), "RLS isolates signatures by tenant");
}

// The PgStore port tests are SYNCHRONOUS: they call the `Store` trait exactly as the served backend
// and the CLI do (a blocking thread driving async work on a runtime handle), which is the path the
// integration must actually prove. They build their own runtime and seed a tenant through it.

#[test]
fn the_pg_store_port_seals_every_chain_and_manages_a_person() {
    let Some(url) = url() else {
        return;
    };
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    let (pool, tenant) = rt.block_on(async {
        let pool = pg::pool(&url).await.expect("connect");
        let tenant = fresh_tenant(&url).await;
        (pool, tenant)
    });
    let store = PgStore::new(pool, tenant, rt.handle().clone(), cid());

    // The person: create is create-only; read, list and rewrite behave through the port.
    assert!(!store.person_exists("asha-verma").expect("exists"));
    store
        .create_person("asha-verma", "legal_name: Asha Verma\nclass: intern\n")
        .expect("create");
    assert!(store.person_exists("asha-verma").expect("exists"));
    assert!(
        store.create_person("asha-verma", "x: 1\n").is_err(),
        "create refuses to overwrite"
    );
    assert_eq!(
        store
            .person("asha-verma")
            .expect("read")
            .string("legal_name"),
        Some("Asha Verma")
    );
    store
        .rewrite_person("asha-verma", "legal_name: Asha K Verma\nclass: intern\n")
        .expect("rewrite");
    assert_eq!(
        store
            .person("asha-verma")
            .expect("read")
            .string("legal_name"),
        Some("Asha K Verma")
    );
    assert_eq!(store.slugs().expect("slugs"), vec!["asha-verma".to_owned()]);

    // Every chain seals through the port and verifies on read.
    for i in 1..=3 {
        store.append_register(entry(i)).expect("register");
    }
    for (i, standing) in [Standing::Active, Standing::Active, Standing::Erased]
        .into_iter()
        .enumerate()
    {
        store
            .append_identity(record(u32::try_from(i + 1).expect("small"), standing))
            .expect("identity");
    }
    store.append_event(evt(Kind::Created)).expect("event");
    store
        .append_signature(sig("PAR/NDA/2026/0001", Method::WetInk))
        .expect("signature");

    let reg = store.register().expect("register read");
    assert_eq!(reg.len(), 3);
    assert!(
        verify_chain(&reg).expect("verifiable").is_empty(),
        "the register verifies through the port"
    );
    assert_eq!(
        reg[1].prev_sha256, reg[0].entry_sha256,
        "the port sealed it in order"
    );
    let ledger = store.ledger().expect("ledger");
    assert_eq!(ledger.len(), 3);
    assert!(verify(&ledger).expect("verifiable").is_empty());
    assert_eq!(store.events().expect("events").len(), 1);
    assert_eq!(store.signatures().expect("signatures").len(), 1);
}

#[test]
fn concurrent_appends_to_one_chain_do_not_fork_it() {
    let Some(url) = url() else {
        return;
    };
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    let (pool, tenant) = rt.block_on(async {
        let pool = pg::pool(&url).await.expect("connect");
        let tenant = fresh_tenant(&url).await;
        (pool, tenant)
    });
    // Eight threads append to the SAME event chain at once. Without the per-chain advisory lock in
    // seal_and_append, two would read the same tail and seal onto it, forking the chain — and the
    // append-only table would keep both. verify() would then find the fork.
    let store = std::sync::Arc::new(PgStore::new(pool, tenant, rt.handle().clone(), cid()));
    let mut handles = Vec::new();
    for _ in 0..8 {
        let store = std::sync::Arc::clone(&store);
        handles.push(std::thread::spawn(move || {
            store.append_event(evt(Kind::Created)).expect("append");
        }));
    }
    for handle in handles {
        handle.join().expect("join");
    }
    let events = store.events().expect("read");
    assert_eq!(events.len(), 8, "every concurrent append landed");
    assert!(
        verify(&events).expect("verifiable").is_empty(),
        "the chain is linear despite eight concurrent writers — the lock prevented a fork"
    );
}

#[tokio::test]
async fn a_person_upserts_reads_and_lists() {
    let Some(url) = url() else {
        return;
    };
    let pool = pg::pool(&url).await.expect("connect");
    let tenant = fresh_tenant(&url).await;

    let doc = |name: &str| -> Person {
        let value: serde_yaml::Value =
            serde_yaml::from_str(&format!("legal_name: {name}\nclass: intern\n")).expect("yaml");
        Person::new("asha-verma".to_owned(), value)
    };

    // insert
    pg::upsert_person(&pool, tenant, &cid(), &doc("Asha Verma"))
        .await
        .expect("insert");
    let got = pg::read_person(&pool, tenant, &cid(), "asha-verma")
        .await
        .expect("read");
    assert_eq!(
        got.expect("present").string("legal_name"),
        Some("Asha Verma")
    );

    // update (the mutable record — same slug upserts)
    pg::upsert_person(&pool, tenant, &cid(), &doc("Asha K Verma"))
        .await
        .expect("update");
    let got = pg::read_person(&pool, tenant, &cid(), "asha-verma")
        .await
        .expect("read");
    assert_eq!(
        got.expect("present").string("legal_name"),
        Some("Asha K Verma"),
        "upsert replaced"
    );

    // roster lists it once, not twice
    let roster = pg::list_persons(&pool, tenant, &cid()).await.expect("list");
    assert_eq!(roster.len(), 1, "one person after an upsert, not two");

    // RLS: another tenant sees no person
    let none = pg::read_person(&pool, fresh_tenant(&url).await, &cid(), "asha-verma")
        .await
        .expect("read other");
    assert!(none.is_none(), "RLS isolates persons by tenant");
}

/// The factory against the real schema: a tenant with no docket account is REFUSED — loudly, never
/// defaulted (ADR-0104 cost 2) — and a provisioned tenant builds a store that sees nothing of any
/// other tenant's. Scratch-only (`DOCKET_PROBE_DB` must name the `*_probe` database it connects to),
/// because it seeds rows; `scripts/probe-audit-writer.sh` provides the database and the admin URL.
///
/// Moved here from a unit test in `src/adapters/pg.rs` so it seeds through `common::seed`, the one
/// copy — the unit test had its own, and the review that moved it found the two had already diverged.
/// A plain `#[test]`: `PgStore` calls `block_on` on the handle from THIS thread, outside the runtime.
#[test]
fn the_factory_refuses_an_unprovisioned_tenant_and_scopes_a_provisioned_one() {
    use platform_docket::adapters::http::server::TenantStores;
    use platform_docket::adapters::pg::PgTenantStores;
    use platform_docket::config::Paths;

    let (Some(url), Ok(scratch), Ok(_admin)) = (
        url(),
        std::env::var("DOCKET_PROBE_DB"),
        std::env::var("DOCKET_ADMIN_DATABASE_URL"),
    ) else {
        eprintln!(
            "SKIPPED the_factory_refuses_an_unprovisioned_tenant_and_scopes_a_provisioned_one — it \
             seeds rows, so it needs DOCKET_PROBE_DB (a *_probe database) and DOCKET_ADMIN_DATABASE_URL; \
             scripts/probe-audit-writer.sh sets both"
        );
        return;
    };
    assert!(
        scratch.ends_with("_probe"),
        "DOCKET_PROBE_DB must name a *_probe database"
    );
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let pool = runtime.block_on(pg::pool(&url)).expect("connect");
    let (connected,): (String,) = runtime
        .block_on(sqlx::query_as("SELECT current_database()").fetch_one(&pool))
        .expect("current_database");
    assert_eq!(
        connected, scratch,
        "connected to a database other than the named scratch one"
    );
    let dir = tempfile::tempdir().expect("tempdir");
    let factory = PgTenantStores::new(pool, runtime.handle().clone(), Paths::resolve(dir.path()));

    // A tenant with no docket account: refused, and the refusal names what is missing.
    let Err(refusal) = factory.build(Uuid::new_v4(), &cid()) else {
        unreachable!("a tenant with no docket account built a store");
    };
    assert!(
        format!("{refusal:#}").contains("has no docket account"),
        "the refusal did not name the missing account: {refusal:#}"
    );

    // A provisioned tenant: the store reads empty — nothing of another tenant's — and no config.
    let tenant = runtime.block_on(fresh_tenant(&url));
    let (store, _blobs, config) = factory
        .build(tenant, &cid())
        .expect("build a store for the provisioned tenant");
    assert_eq!(store.slugs().expect("list slugs"), Vec::<String>::new());
    assert!(config.is_none(), "an unseeded tenant has no config");
}
