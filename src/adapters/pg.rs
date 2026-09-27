//! The Postgres store: docket's records and sealed chains in the database (ADR-0093, RFC-0039 D1).
//!
//! docket's store is the database, not the filesystem. Each sealed record is stored VERBATIM as
//! JSONB (platform-tenancy migration 0006) so its SHA-256 seal verifies unchanged — the
//! record -> JSON -> record path proven in `domain::register`/`identity`/`event`. This adapter is
//! async (sqlx, ADR-0005): the served backend awaits it, and the break-glass CLI `block_on`s it
//! (ADR-0093 §3), one implementation for both callers.
//!
//! EVERY OPERATION SETS THE RLS SESSION VARIABLES first, inside its transaction. Since migration
//! 0012/0014 that is THREE values, not two: `paridyum.tenant_id`, `paridyum.account_id` and
//! `paridyum.operator_scope`. The accessors RAISE when unset rather than defaulting (ADR-0038), so
//! an operation that forgot one is refused, never silently widened. They are set LOCAL to the
//! transaction, so a pooled connection cannot carry one request's identity into the next.
//!
//! WHY THE ACCOUNT IS HERE AT ALL. ADR-0104 makes a product instance an account: a tenant may run
//! two dockets, and their person records must not mix. Migration 0012 put `account_id NOT NULL` on
//! all five person-bearing tables and 0014 put it in their policies. This adapter set two session
//! values and inserted none of it, so on the day 0012 applied every write here began failing — the
//! retrofit was written without its only writer being updated (T-G.19, D-70).
//!
//! THE RESOLUTION IS AN INTERIM AND IS DELIBERATELY NOT A DEFAULT. ADR-0104 D5 addresses instances
//! by path — `<tenant-host>/<product>/<slug>` — and this repository has no such routing yet. Until
//! it does, `enter` looks up the tenant's `slug = 'default'` docket account, which ADR-0104 cost 2
//! explicitly permits ("a default instance is still an account, never a null"). It RAISES when that
//! row is absent. It must never acquire a fallback: a nil uuid or an `unwrap_or_default` here is
//! the widening-by-absence GLO-0005 and `db.operator_scope_is_an_explicit_grant` exist to forbid,
//! and it would silently write one instance's people into another's.

use crate::adapters::audit;
use crate::adapters::blobs;
use crate::adapters::correlation::CorrelationId;
use crate::adapters::http::server::{RequestStores, TenantStores};
use crate::adapters::registry::Registry;
use crate::adapters::store::Store;
use crate::config::Paths;
use crate::domain::chain::{Chained, seal, verify};
use crate::domain::event::Event;
use crate::domain::identity::Record;
use crate::domain::person::Person;
use crate::domain::register::Entry;
use crate::domain::signature::Record as Signature;
use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde::de::DeserializeOwned;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

/// Open a connection pool to the docket database.
///
/// # Errors
/// Fails if the URL is unreachable or authentication is refused.
pub async fn pool(url: &str) -> Result<PgPool> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await
        .context("connecting to the docket database")
}

/// The session values every docket transaction carries, as ONE literal: the tenant and the request's
/// correlation id (ADR-0038, ADR-0044 D1). `$1` is the tenant, `$2` the correlation id. A macro so the
/// operator variant below is built FROM it rather than retyping it — a new session value added here
/// reaches both, and cannot be added to one and missed in the other.
macro_rules! request_session_sql {
    () => {
        "SELECT set_config('paridyum.tenant_id', $1, true), \
         set_config('paridyum.correlation_id', $2, true)"
    };
}

/// The request session — what the audit sink sets, and all it sets (see `Sink::record` below).
const REQUEST_SESSION: &str = request_session_sql!();

/// [`REQUEST_SESSION`] plus operator scope, in one statement: what `enter()` needs to read
/// `accounts`. Same parameters.
const OPERATOR_SESSION: &str = concat!(
    request_session_sql!(),
    ", set_config('paridyum.operator_scope', 'true', true)"
);

/// Begin a transaction carrying the request session and nothing more. The audit sink writes through
/// this, and its live test reads through it — so the test exercises the session the sink actually
/// sets, not a copy of it.
async fn request_transaction(
    pool: &PgPool,
    tenant: Uuid,
    correlation: &CorrelationId,
) -> Result<Transaction<'static, Postgres>> {
    let mut tx = pool
        .begin()
        .await
        .context("beginning the request transaction")?;
    sqlx::query(REQUEST_SESSION)
        .bind(tenant.to_string())
        .bind(correlation.as_str())
        .execute(&mut *tx)
        .await
        .context("setting the request session values")?;
    Ok(tx)
}

/// Enter a tenant's operator context for the life of this transaction, and resolve which docket
/// instance that context is serving.
///
/// Returns the `accounts.id` every write below must carry. See this module's header for why the
/// lookup is an interim, and why it raises instead of defaulting.
///
/// # Errors
/// Fails if the session variables cannot be set, or if the tenant has no docket account.
async fn enter(
    tx: &mut Transaction<'_, Postgres>,
    tenant: Uuid,
    correlation: &CorrelationId,
) -> Result<Uuid> {
    // Tenant and operator scope first, because resolving the account needs them and not the other
    // way round: `accounts` FORCEs RLS and its policy reads `current_tenant()` AND
    // `has_operator_scope()` only. So this lookup is legal with two of the four set, and the
    // ordering is load-bearing rather than incidental.
    //
    // The correlation id goes in the same statement as those two, because it is needed no later and
    // costs no round trip. It is the one value here that no policy reads: it grants and refuses
    // nothing (ADR-0044 D1, RFC-0045 §D1). `audit_events` is `NOT NULL` on it and
    // `paridyum.current_correlation()` raises rather than returning NULL, so a path that writes an
    // audit event without it fails loudly instead of writing a row that joins to nothing.
    //
    // NOT MINTED HERE, and that is the point of taking it as a parameter. A minted-per-transaction
    // id would turn one `docket issue` into four unrelated ids — the per-hop minting ADR-0044 D1
    // forbids, in the place it would be least visible.
    sqlx::query(OPERATOR_SESSION)
        .bind(tenant.to_string())
        .bind(correlation.as_str())
        .execute(&mut **tx)
        .await
        .context("setting the tenant RLS session variables")?;

    let found: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM accounts WHERE tenant_id = $1 AND product = 'docket' AND slug = 'default'",
    )
    .bind(tenant)
    .fetch_optional(&mut **tx)
    .await
    .context("resolving this tenant's docket account")?;

    let Some((account,)) = found else {
        bail!(
            "tenant {tenant} has no docket account (product='docket', slug='default'). \
             Refusing rather than inventing one: an account is the boundary between two instances \
             of docket under one tenant (ADR-0104), and a write that guesses it puts one \
             instance's people into another's. Provision the account first."
        );
    };

    sqlx::query("SELECT set_config('paridyum.account_id', $1, true)")
        .bind(account.to_string())
        .execute(&mut **tx)
        .await
        .context("setting the account RLS session variable")?;

    Ok(account)
}

// ── The four sealed chains ───────────────────────────────────────────────────────────────────
//
// register, identity, personnel-event and signature records are all `Chained` and all stored the
// same way, so the append and the verified read are written once here and named per chain below. The
// table name
// is a compile-time literal from this module, never caller input, so `format!` into the SQL carries
// no injection.

async fn append_chain<T: Serialize + Sync + 'static>(
    pool: &PgPool,
    tenant: Uuid,
    correlation: &CorrelationId,
    table: &str,
    record: &T,
) -> Result<()> {
    let sql =
        format!("INSERT INTO {table} (id, tenant_id, account_id, record) VALUES ($1, $2, $3, $4)");
    let mut tx = pool.begin().await.context("beginning a transaction")?;
    let account = enter(&mut tx, tenant, correlation).await?;
    sqlx::query(&sql)
        .bind(Uuid::new_v4())
        .bind(tenant)
        .bind(account)
        .bind(sqlx::types::Json(record))
        .execute(&mut *tx)
        .await
        .with_context(|| format!("appending to {table}"))?;
    tx.commit().await.context("committing")?;
    Ok(())
}

async fn read_chain<T: Chained + DeserializeOwned + Send + Unpin + 'static>(
    pool: &PgPool,
    tenant: Uuid,
    correlation: &CorrelationId,
    table: &str,
) -> Result<Vec<T>> {
    let sql = format!("SELECT record FROM {table} ORDER BY chain_seq");
    let mut tx = pool.begin().await.context("beginning a transaction")?;
    enter(&mut tx, tenant, correlation).await?;
    let rows: Vec<(sqlx::types::Json<T>,)> = sqlx::query_as(&sql)
        .fetch_all(&mut *tx)
        .await
        .with_context(|| format!("reading {table}"))?;
    tx.commit().await.context("committing")?;
    let records: Vec<T> = rows.into_iter().map(|(json,)| json.0).collect();
    let faults = verify(&records).with_context(|| format!("verifying the {table} chain"))?;
    if !faults.is_empty() {
        let detail = faults
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("; ");
        bail!("the {table} chain has {} fault(s): {detail}", faults.len());
    }
    Ok(records)
}

/// Append one sealed register (issued-document) entry. Append-only is enforced by the database.
///
/// # Errors
/// Fails if the entry cannot be serialised or the insert is refused (RLS, or an attempted rewrite).
pub async fn append_register(
    pool: &PgPool,
    tenant: Uuid,
    correlation: &CorrelationId,
    entry: &Entry,
) -> Result<()> {
    append_chain(pool, tenant, correlation, "docket_register", entry).await
}

/// Read the register in issue order, verifying the seal before returning it.
///
/// # Errors
/// Fails if the read is refused, a row does not deserialise, or the chain does not verify.
pub async fn read_register(
    pool: &PgPool,
    tenant: Uuid,
    correlation: &CorrelationId,
) -> Result<Vec<Entry>> {
    read_chain(pool, tenant, correlation, "docket_register").await
}

/// Append one sealed identity record.
///
/// # Errors
/// As [`append_register`].
pub async fn append_identity(
    pool: &PgPool,
    tenant: Uuid,
    correlation: &CorrelationId,
    record: &Record,
) -> Result<()> {
    append_chain(pool, tenant, correlation, "docket_identities", record).await
}

/// Read the identity ledger in order, verifying the seal.
///
/// # Errors
/// As [`read_register`].
pub async fn read_identities(
    pool: &PgPool,
    tenant: Uuid,
    correlation: &CorrelationId,
) -> Result<Vec<Record>> {
    read_chain(pool, tenant, correlation, "docket_identities").await
}

/// Append one sealed personnel event.
///
/// # Errors
/// As [`append_register`].
pub async fn append_event(
    pool: &PgPool,
    tenant: Uuid,
    correlation: &CorrelationId,
    event: &Event,
) -> Result<()> {
    append_chain(pool, tenant, correlation, "docket_events", event).await
}

/// Read the personnel-event log in order, verifying the seal.
///
/// # Errors
/// As [`read_register`].
pub async fn read_events(
    pool: &PgPool,
    tenant: Uuid,
    correlation: &CorrelationId,
) -> Result<Vec<Event>> {
    read_chain(pool, tenant, correlation, "docket_events").await
}

/// Append one sealed signature (executed-document) record.
///
/// A separate chain from the register by design: a signature is a fact learned after issuance, so it
/// is a new record rather than a mutation of the issuance entry (`domain::signature`).
///
/// # Errors
/// As [`append_register`].
pub async fn append_signature(
    pool: &PgPool,
    tenant: Uuid,
    correlation: &CorrelationId,
    record: &Signature,
) -> Result<()> {
    append_chain(pool, tenant, correlation, "docket_signatures", record).await
}

/// Read the signature ledger in order, verifying the seal.
///
/// # Errors
/// As [`read_register`].
pub async fn read_signatures(
    pool: &PgPool,
    tenant: Uuid,
    correlation: &CorrelationId,
) -> Result<Vec<Signature>> {
    read_chain(pool, tenant, correlation, "docket_signatures").await
}

// ── The mutable person record ────────────────────────────────────────────────────────────────
//
// A person is a dynamic document (person-classes/fields.yaml), stored whole as JSONB and keyed by
// its slug within the tenant. It is the one docket record that is edited rather than appended, so
// the write is an upsert. The value is docket's `serde_yaml::Value`; JSON is a lossless carrier for
// the maps, strings, booleans and numbers a person document holds, so it round-trips through JSONB.

/// Create or replace a person record.
///
/// # Errors
/// Fails if the document cannot be serialised or the write is refused (RLS).
pub async fn upsert_person(
    pool: &PgPool,
    tenant: Uuid,
    correlation: &CorrelationId,
    person: &Person,
) -> Result<()> {
    let record = serde_json::to_value(&person.value).context("serialising a person document")?;
    let mut tx = pool.begin().await.context("beginning a transaction")?;
    let account = enter(&mut tx, tenant, correlation).await?;
    // ON CONFLICT names (account_id, slug), which is the constraint migration 0012 left behind when
    // it replaced docket_persons_slug_unique. Naming (tenant_id, slug) here raises "no unique or
    // exclusion constraint matching" — the same class of break as the missing column, and it would
    // have survived a compile.
    sqlx::query(
        "INSERT INTO docket_persons (id, tenant_id, account_id, slug, record) \
         VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (account_id, slug) DO UPDATE SET record = EXCLUDED.record, updated_at = now()",
    )
    .bind(Uuid::new_v4())
    .bind(tenant)
    .bind(account)
    .bind(&person.slug)
    .bind(sqlx::types::Json(&record))
    .execute(&mut *tx)
    .await
    .context("upserting a person record")?;
    tx.commit().await.context("committing")?;
    Ok(())
}

fn person_from(slug: String, record: serde_json::Value) -> Result<Person> {
    let value = serde_yaml::to_value(record).context("a stored person document is not a value")?;
    Ok(Person::new(slug, value))
}

/// Read one person record, or `None` if the tenant has no person by that slug.
///
/// # Errors
/// Fails if the read is refused or the stored document does not convert back to a value.
pub async fn read_person(
    pool: &PgPool,
    tenant: Uuid,
    correlation: &CorrelationId,
    slug: &str,
) -> Result<Option<Person>> {
    let mut tx = pool.begin().await.context("beginning a transaction")?;
    enter(&mut tx, tenant, correlation).await?;
    let row: Option<(sqlx::types::Json<serde_json::Value>,)> =
        sqlx::query_as("SELECT record FROM docket_persons WHERE slug = $1")
            .bind(slug)
            .fetch_optional(&mut *tx)
            .await
            .context("reading a person record")?;
    tx.commit().await.context("committing")?;
    row.map(|(json,)| person_from(slug.to_owned(), json.0))
        .transpose()
}

/// Read every person record for the tenant, by slug — the roster's backing read.
///
/// # Errors
/// Fails if the read is refused or a stored document does not convert back to a value.
pub async fn list_persons(
    pool: &PgPool,
    tenant: Uuid,
    correlation: &CorrelationId,
) -> Result<Vec<Person>> {
    let mut tx = pool.begin().await.context("beginning a transaction")?;
    enter(&mut tx, tenant, correlation).await?;
    let rows: Vec<(String, sqlx::types::Json<serde_json::Value>)> =
        sqlx::query_as("SELECT slug, record FROM docket_persons ORDER BY slug")
            .fetch_all(&mut *tx)
            .await
            .context("reading the roster")?;
    tx.commit().await.context("committing")?;
    rows.into_iter()
        .map(|(slug, json)| person_from(slug, json.0))
        .collect()
}

/// Insert a new person record, failing if the slug already exists (the create path).
///
/// # Errors
/// Fails if a record already exists for the slug, or the write is refused.
pub async fn insert_person(
    pool: &PgPool,
    tenant: Uuid,
    correlation: &CorrelationId,
    person: &Person,
) -> Result<()> {
    let record = serde_json::to_value(&person.value).context("serialising a person document")?;
    let mut tx = pool.begin().await.context("beginning a transaction")?;
    let account = enter(&mut tx, tenant, correlation).await?;
    let done = sqlx::query(
        "INSERT INTO docket_persons (id, tenant_id, account_id, slug, record) \
         VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (account_id, slug) DO NOTHING",
    )
    .bind(Uuid::new_v4())
    .bind(tenant)
    .bind(account)
    .bind(&person.slug)
    .bind(sqlx::types::Json(&record))
    .execute(&mut *tx)
    .await
    .context("inserting a person record")?;
    if done.rows_affected() == 0 {
        bail!("a record already exists for {}", person.slug);
    }
    tx.commit().await.context("committing")?;
    Ok(())
}

// ── Atomic seal-and-append ───────────────────────────────────────────────────────────────────
//
// The file store is single-writer, so read-last, seal-onto-it, append cannot interleave. Postgres is
// not: two operators enrolling at once would each read the same tail and seal onto it, forking the
// chain — and the append-only table would accept both. So the port's append is one transaction that
// takes a per-(tenant, chain) advisory lock, reads the committed tail under it, seals, and inserts.
// Concurrent appends to the same chain queue on the lock and each seals onto the other's committed
// end; appends to different chains or different tenants do not contend. This is the concurrency
// correctness ADR-0093 §decision-2 moves from a file lock into the database.

async fn seal_and_append<T>(
    pool: &PgPool,
    tenant: Uuid,
    correlation: &CorrelationId,
    table: &str,
    record: T,
) -> Result<T>
where
    T: Chained + Serialize + DeserializeOwned + Send + Unpin + 'static,
{
    let mut tx = pool.begin().await.context("beginning a transaction")?;
    let account = enter(&mut tx, tenant, correlation).await?;
    // The lock key carries the ACCOUNT as well as the tenant, because since ADR-0104 a chain is
    // per-instance: two dockets under one tenant have two independent registers, and the tail this
    // transaction reads is bounded by the account predicate in 0014's policies. Keying on tenant
    // alone would still be *safe* — it over-locks, serialising two instances that never contend —
    // but it would say something untrue about what the lock protects, and the next person to widen
    // it would have no way to tell over-locking from a deliberate invariant.
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(format!("docket:{tenant}:{account}:{table}"))
        .execute(&mut *tx)
        .await
        .context("locking the chain for append")?;
    let tail_sql = format!("SELECT record FROM {table} ORDER BY chain_seq DESC LIMIT 1");
    let last: Option<(sqlx::types::Json<T>,)> = sqlx::query_as(&tail_sql)
        .fetch_optional(&mut *tx)
        .await
        .with_context(|| format!("reading the tail of {table}"))?;
    let previous: Option<T> = last.map(|(json,)| json.0);
    let sealed = seal(record, previous.as_ref()).context("sealing onto the chain")?;
    let insert_sql =
        format!("INSERT INTO {table} (id, tenant_id, account_id, record) VALUES ($1, $2, $3, $4)");
    sqlx::query(&insert_sql)
        .bind(Uuid::new_v4())
        .bind(tenant)
        .bind(account)
        .bind(sqlx::types::Json(&sealed))
        .execute(&mut *tx)
        .await
        .with_context(|| format!("appending to {table}"))?;
    tx.commit().await.context("committing")?;
    Ok(sealed)
}

// ── PgStore: the Store port over Postgres ──────────────────────────────────────────────────────
//
// The served backend's implementation of `adapters::store::Store` (ADR-0093 port design). The trait
// is synchronous — the whole application layer is — and sqlx is async, so each method drives the
// async operation to completion on a shared runtime the process owns. The callers are blocking
// threads (the served backend is thread-per-request, the CLI is synchronous), never inside the
// runtime, so `block_on` is the sanctioned bridge and not a nested-runtime hazard.

/// docket's record store backed by Postgres, for one tenant (ADR-0093).
pub struct PgStore {
    pool: PgPool,
    tenant: Uuid,
    runtime: tokio::runtime::Handle,
    /// The correlation id of the request (or CLI invocation) this store serves.
    ///
    /// **Held on the store rather than passed per method**, because a `PgStore` is built for the life
    /// of ONE request (`PgTenantStores::build`) or one CLI invocation. That makes the store the
    /// natural scope for it, and it means the `Store` trait — which the file-backed implementation
    /// also satisfies, and which knows nothing about sessions — does not grow a parameter it has no
    /// use for.
    correlation: CorrelationId,
}

impl PgStore {
    /// A Postgres-backed store for `tenant`, driving async work on `runtime`.
    #[must_use]
    pub fn new(
        pool: PgPool,
        tenant: Uuid,
        runtime: tokio::runtime::Handle,
        correlation: CorrelationId,
    ) -> Self {
        Self {
            pool,
            tenant,
            runtime,
            correlation,
        }
    }

    /// Parse a person body (the same YAML the file store writes) into a document.
    fn body_to_person(slug: &str, body: &str) -> Result<Person> {
        let value: serde_yaml::Value =
            serde_yaml::from_str(body).context("the person body is not valid YAML")?;
        Ok(Person::new(slug.to_owned(), value))
    }
}

impl Store for PgStore {
    fn person(&self, slug: &str) -> Result<Person> {
        self.runtime
            .block_on(read_person(
                &self.pool,
                self.tenant,
                &self.correlation,
                slug,
            ))?
            .with_context(|| format!("no person record for {slug}"))
    }

    fn person_exists(&self, slug: &str) -> Result<bool> {
        Ok(self
            .runtime
            .block_on(read_person(
                &self.pool,
                self.tenant,
                &self.correlation,
                slug,
            ))?
            .is_some())
    }

    fn slugs(&self) -> Result<Vec<String>> {
        Ok(self
            .runtime
            .block_on(list_persons(&self.pool, self.tenant, &self.correlation))?
            .into_iter()
            .map(|p| p.slug)
            .collect())
    }

    fn create_person(&self, slug: &str, body: &str) -> Result<()> {
        let person = Self::body_to_person(slug, body)?;
        self.runtime.block_on(insert_person(
            &self.pool,
            self.tenant,
            &self.correlation,
            &person,
        ))
    }

    fn rewrite_person(&self, slug: &str, body: &str) -> Result<()> {
        let person = Self::body_to_person(slug, body)?;
        self.runtime.block_on(upsert_person(
            &self.pool,
            self.tenant,
            &self.correlation,
            &person,
        ))
    }

    fn ledger(&self) -> Result<Vec<Record>> {
        self.runtime
            .block_on(read_identities(&self.pool, self.tenant, &self.correlation))
    }

    fn append_identity(&self, record: Record) -> Result<Record> {
        self.runtime.block_on(seal_and_append(
            &self.pool,
            self.tenant,
            &self.correlation,
            "docket_identities",
            record,
        ))
    }

    fn register(&self) -> Result<Vec<Entry>> {
        self.runtime
            .block_on(read_register(&self.pool, self.tenant, &self.correlation))
    }

    fn append_register(&self, entry: Entry) -> Result<Entry> {
        self.runtime.block_on(seal_and_append(
            &self.pool,
            self.tenant,
            &self.correlation,
            "docket_register",
            entry,
        ))
    }

    fn events(&self) -> Result<Vec<Event>> {
        self.runtime
            .block_on(read_events(&self.pool, self.tenant, &self.correlation))
    }

    fn append_event(&self, event: Event) -> Result<Event> {
        self.runtime.block_on(seal_and_append(
            &self.pool,
            self.tenant,
            &self.correlation,
            "docket_events",
            event,
        ))
    }

    fn signatures(&self) -> Result<Vec<Signature>> {
        self.runtime
            .block_on(read_signatures(&self.pool, self.tenant, &self.correlation))
    }

    fn append_signature(&self, record: Signature) -> Result<Signature> {
        self.runtime.block_on(seal_and_append(
            &self.pool,
            self.tenant,
            &self.correlation,
            "docket_signatures",
            record,
        ))
    }

    fn reseal_events(&self, _events: Vec<Event>) -> Result<()> {
        // Erase rewrites the event log and re-seals it, which means DELETEing chain rows. The
        // append-only trigger (migration 0006's `paridyum.forbid_mutation()`) forbids that, and
        // lifting it needs a platform-tenancy migration — a narrowly-scoped, SECURITY DEFINER delete
        // exception (design §6) — that is cross-repo and MUST NOT be made from here. So the Postgres
        // store refuses erasure with a clear error rather than silently failing the trigger.
        // Tracked as debt (DECISION-DEBT.md, DEBT-docket-pg-erase).
        bail!(
            "erasing on the Postgres store needs migration 0007 (append-only delete exception); \
             tracked as debt. Use the break-glass CLI on the file store to honour this erasure."
        )
    }
}

/// Builds a Postgres record store and a Garage/S3 blob store for a request's tenant (ADR-0099). The
/// served backend holds one and calls it once per request, since the tenant comes from the operator
/// token rather than a fixed env. Owns the pool; drives blob I/O on the shared runtime.
pub struct PgTenantStores {
    pool: PgPool,
    handle: tokio::runtime::Handle,
    paths: Paths,
}

impl PgTenantStores {
    /// A factory over `pool`, driving async work on `handle`, with `paths` for the blob store.
    #[must_use]
    pub fn new(pool: PgPool, handle: tokio::runtime::Handle, paths: Paths) -> Self {
        Self {
            pool,
            handle,
            paths,
        }
    }
}

/// Read every `record` in a tenant's config table (ADR-0099), under its operator RLS context. The
/// table name comes from `Registry::from_db`'s fixed constants, never caller input, so the `format!`
/// carries no injection — the same property the chain reads above rely on.
async fn read_config(
    pool: &PgPool,
    tenant: Uuid,
    correlation: &CorrelationId,
    table: &str,
) -> Result<Vec<serde_json::Value>> {
    let sql = format!("SELECT record FROM {table} ORDER BY id");
    let mut tx = pool.begin().await.context("beginning a transaction")?;
    enter(&mut tx, tenant, correlation).await?;
    let rows: Vec<(sqlx::types::Json<serde_json::Value>,)> = sqlx::query_as(&sql)
        .fetch_all(&mut *tx)
        .await
        .with_context(|| format!("reading config table {table}"))?;
    tx.commit().await.context("committing")?;
    Ok(rows.into_iter().map(|(json,)| json.0).collect())
}

/// The audit sink over the tenancy database (ADR-0110, ADR-0044).
///
/// **Its own transaction, always**, unlike `platform-tenancy`'s two-entry writer. There, a successful
/// read and its audit row commit together because the read's transaction is still open. Here the
/// operation has already finished — the dispatcher has a `Response` in hand — so there is nothing left
/// to be atomic with, and a failed row must not be able to undo work that already succeeded.
impl audit::Sink for PgTenantStores {
    fn record(&self, ctx: audit::Context<'_>, event: &audit::Event) -> Result<()> {
        self.handle.block_on(async {
            // Only the session values this transaction reads. `enter()` is not used: it resolves a
            // docket account, and an audit row is written for the OPERATION rather than for a silo the
            // dispatcher may not have reached (see audit::Context).
            //
            // NO ACCOUNT AND NO OPERATOR SCOPE, deliberately. `audit_events_append` (migration 0010)
            // checks `tenant_id = current_tenant()` and nothing else, and this transaction only
            // inserts — so neither value is read. An earlier version set the account to the nil uuid
            // and the scope to `true` "so nothing raises": a sentinel standing in for an absent
            // account is the widening-by-absence GLO-0005 forbids, and if a later statement ever read
            // them in this transaction, it would read every account's rows. Unset, a future read
            // CANNOT WIDEN: `current_account()` raises when a policy evaluates it (ADR-0038), and RLS
            // evaluates it on every row a scan reaches — so a read either raises or, when no row is
            // reached, returns nothing. It never returns another account's rows.
            //
            // Through `request_transaction`, which the live test also uses, so the test reads under
            // exactly the session this sink writes under.
            let mut tx = request_transaction(&self.pool, ctx.tenant, ctx.correlation)
                .await
                .context("opening the audit transaction")?;

            sqlx::query(audit::INSERT)
                .bind(ctx.tenant)
                .bind(ctx.correlation.as_str())
                .bind(&event.actor)
                .bind(&event.action)
                .bind(&event.object)
                .bind(event.outcome.as_str())
                .bind(event.source.as_deref())
                .bind(event.client.as_deref())
                // From the context, which the dispatcher fills from what it verified — never a literal
                // here, where the fact is not known (see audit::Context::operator_scope).
                .bind(ctx.operator_scope)
                .bind(ctx.region)
                .execute(&mut *tx)
                .await
                .context("appending the audit event")?;

            tx.commit().await.context("committing the audit event")?;
            Ok(())
        })
    }
}

impl TenantStores for PgTenantStores {
    fn build(&self, tenant: Uuid, correlation: &CorrelationId) -> Result<RequestStores<'_>> {
        let store = PgStore::new(
            self.pool.clone(),
            tenant,
            self.handle.clone(),
            correlation.clone(),
        );
        let blobs = blobs::open(&self.paths, &tenant.to_string(), Some(self.handle.clone()))
            .context("opening the blob store (DOCKET_S3_* configured?)")?;
        // The tenant's configuration from its 0008 rows, or `None` when it has none yet — the served
        // dispatcher then falls back to the git registry until it is seeded (ADR-0099).
        let config = Registry::from_db(|table| {
            self.handle
                .block_on(read_config(&self.pool, tenant, correlation, table))
        })
        .context("loading the tenant's configuration")?;
        Ok((Box::new(store), blobs, config))
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    /// docket's audit writer against the real `audit_events` policies — docket's own code, not a
    /// retyped copy of its SQL. `scripts/probe-audit-writer.sh` runs it against a scratch database with
    /// every tenancy migration replayed, as a login role subject to RLS (a superuser bypasses RLS and
    /// would prove nothing).
    ///
    /// It proves the sink's three claims: the row is written with only the tenant and the correlation
    /// id in the session; the row's `operator_scope` is the context's value, not a literal; and a read
    /// under the session the sink sets cannot widen.
    ///
    /// **SCRATCH ONLY, BY CHECK.** `audit_events` is append-only — a trigger refuses UPDATE and DELETE
    /// — so a row written here is permanent. The test therefore runs only when `DOCKET_AUDIT_PROBE_DB`
    /// names the database it connects to AND that name ends in `_probe`; `DOCKET_DATABASE_URL` alone,
    /// which `seed-config.py` also uses for real databases, is never enough. Skipped, it says so on
    /// stderr rather than passing silently.
    #[test]
    fn the_audit_sink_writes_under_the_append_policy_alone() {
        let (Ok(url), Ok(tenant), Ok(scratch)) = (
            std::env::var("DOCKET_DATABASE_URL"),
            std::env::var("DOCKET_TEST_TENANT"),
            std::env::var("DOCKET_AUDIT_PROBE_DB"),
        ) else {
            eprintln!(
                "SKIPPED the_audit_sink_writes_under_the_append_policy_alone — run \
                 scripts/probe-audit-writer.sh; this path is invisible to every other test"
            );
            return;
        };
        assert!(
            scratch.ends_with("_probe"),
            "DOCKET_AUDIT_PROBE_DB must name a *_probe database"
        );
        let tenant: Uuid = tenant.parse().expect("DOCKET_TEST_TENANT is a uuid");
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let pool = runtime.block_on(pool(&url)).expect("connect");
        let (connected,): (String,) = runtime
            .block_on(sqlx::query_as("SELECT current_database()").fetch_one(&pool))
            .expect("current_database");
        assert_eq!(
            connected, scratch,
            "connected to a database other than the named scratch one"
        );

        let dir = tempfile::tempdir().expect("tempdir");
        let sink = PgTenantStores::new(
            pool.clone(),
            runtime.handle().clone(),
            Paths::resolve(dir.path()),
        );
        let correlation = CorrelationId::mint();
        let event = audit::Event {
            action: "person.read".into(),
            object: "persons".into(),
            outcome: audit::Outcome::Allowed,
            actor: "probe-subject".into(),
            source: Some("203.0.113.9".into()),
            client: Some("probe/1.0".into()),
        };
        // `false`, deliberately: a sink that bound a literal `true` would still write a row, and
        // only a value it could not have invented tells the two apart.
        audit::Sink::record(
            &sink,
            audit::Context {
                tenant,
                correlation: &correlation,
                region: "in",
                operator_scope: false,
            },
            &event,
        )
        .expect("the row is written with only tenant and correlation in the session");

        runtime.block_on(async {
            // Read it back as an operator's console would: tenant, an account, operator scope.
            let mut tx = pool.begin().await.expect("begin");
            sqlx::query(OPERATOR_SESSION)
                .bind(tenant.to_string())
                .bind(correlation.as_str())
                .execute(&mut *tx)
                .await
                .expect("operator session");
            sqlx::query("SELECT set_config('paridyum.account_id', $1, true)")
                .bind(Uuid::new_v4().to_string())
                .execute(&mut *tx)
                .await
                .expect("account");
            let rows: Vec<(bool,)> =
                sqlx::query_as("SELECT operator_scope FROM audit_events WHERE correlation_id = $1")
                    .bind(correlation.as_str())
                    .fetch_all(&mut *tx)
                    .await
                    .expect("read back");
            assert_eq!(
                rows,
                vec![(false,)],
                "one row, carrying the context's operator_scope"
            );
            tx.rollback().await.expect("rollback");

            // Under the SAME session the sink writes under — the same function builds it — a read
            // reaches the row just written, evaluates the account predicate, and raises.
            let mut tx = request_transaction(&pool, tenant, &correlation)
                .await
                .expect("request transaction");
            let widened = sqlx::query_as::<_, (i64,)>("SELECT count(*) FROM audit_events")
                .fetch_one(&mut *tx)
                .await;
            assert!(
                widened.is_err(),
                "a read in the audit session answered: {widened:?}"
            );
        });
    }

    // Runs only when DOCKET_DATABASE_URL is set (like the pg_store integration tests). A plain
    // `#[test]`: PgStore block_on's on the handle from THIS thread, outside the runtime.
    #[test]
    fn the_factory_builds_a_tenant_scoped_store() {
        let Ok(url) = std::env::var("DOCKET_DATABASE_URL") else {
            return;
        };
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let pool = runtime.block_on(pool(&url)).expect("connect");
        let dir = tempfile::tempdir().expect("tempdir");
        let factory =
            PgTenantStores::new(pool, runtime.handle().clone(), Paths::resolve(dir.path()));

        // A fresh tenant that never had a row: the factory builds a store bound to it, and a read
        // returns empty rather than another tenant's data — the store is tenant-scoped.
        let tenant = Uuid::new_v4();
        let (store, _blobs, config) = factory
            .build(tenant, &CorrelationId::mint())
            .expect("build a store for the tenant");
        assert_eq!(store.slugs().expect("list slugs"), Vec::<String>::new());
        // A fresh tenant has no config rows, so the factory returns None and the dispatcher would
        // fall back to the git registry.
        assert!(config.is_none(), "an unseeded tenant has no config");
    }
}
