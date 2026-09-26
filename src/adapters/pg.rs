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

use crate::adapters::blobs;
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

/// Enter a tenant's operator context for the life of this transaction, and resolve which docket
/// instance that context is serving.
///
/// Returns the `accounts.id` every write below must carry. See this module's header for why the
/// lookup is an interim, and why it raises instead of defaulting.
///
/// # Errors
/// Fails if the session variables cannot be set, or if the tenant has no docket account.
async fn enter(tx: &mut Transaction<'_, Postgres>, tenant: Uuid) -> Result<Uuid> {
    // Tenant and operator scope first, because resolving the third value needs them and not the
    // other way round: `accounts` FORCEs RLS and its policy reads `current_tenant()` AND
    // `has_operator_scope()` only. So this lookup is legal with two of the three set, and the
    // ordering is load-bearing rather than incidental.
    sqlx::query(
        "SELECT set_config('paridyum.tenant_id', $1, true), \
                set_config('paridyum.operator_scope', 'true', true)",
    )
    .bind(tenant.to_string())
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
    table: &str,
    record: &T,
) -> Result<()> {
    let sql =
        format!("INSERT INTO {table} (id, tenant_id, account_id, record) VALUES ($1, $2, $3, $4)");
    let mut tx = pool.begin().await.context("beginning a transaction")?;
    let account = enter(&mut tx, tenant).await?;
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
    table: &str,
) -> Result<Vec<T>> {
    let sql = format!("SELECT record FROM {table} ORDER BY chain_seq");
    let mut tx = pool.begin().await.context("beginning a transaction")?;
    enter(&mut tx, tenant).await?;
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
pub async fn append_register(pool: &PgPool, tenant: Uuid, entry: &Entry) -> Result<()> {
    append_chain(pool, tenant, "docket_register", entry).await
}

/// Read the register in issue order, verifying the seal before returning it.
///
/// # Errors
/// Fails if the read is refused, a row does not deserialise, or the chain does not verify.
pub async fn read_register(pool: &PgPool, tenant: Uuid) -> Result<Vec<Entry>> {
    read_chain(pool, tenant, "docket_register").await
}

/// Append one sealed identity record.
///
/// # Errors
/// As [`append_register`].
pub async fn append_identity(pool: &PgPool, tenant: Uuid, record: &Record) -> Result<()> {
    append_chain(pool, tenant, "docket_identities", record).await
}

/// Read the identity ledger in order, verifying the seal.
///
/// # Errors
/// As [`read_register`].
pub async fn read_identities(pool: &PgPool, tenant: Uuid) -> Result<Vec<Record>> {
    read_chain(pool, tenant, "docket_identities").await
}

/// Append one sealed personnel event.
///
/// # Errors
/// As [`append_register`].
pub async fn append_event(pool: &PgPool, tenant: Uuid, event: &Event) -> Result<()> {
    append_chain(pool, tenant, "docket_events", event).await
}

/// Read the personnel-event log in order, verifying the seal.
///
/// # Errors
/// As [`read_register`].
pub async fn read_events(pool: &PgPool, tenant: Uuid) -> Result<Vec<Event>> {
    read_chain(pool, tenant, "docket_events").await
}

/// Append one sealed signature (executed-document) record.
///
/// A separate chain from the register by design: a signature is a fact learned after issuance, so it
/// is a new record rather than a mutation of the issuance entry (`domain::signature`).
///
/// # Errors
/// As [`append_register`].
pub async fn append_signature(pool: &PgPool, tenant: Uuid, record: &Signature) -> Result<()> {
    append_chain(pool, tenant, "docket_signatures", record).await
}

/// Read the signature ledger in order, verifying the seal.
///
/// # Errors
/// As [`read_register`].
pub async fn read_signatures(pool: &PgPool, tenant: Uuid) -> Result<Vec<Signature>> {
    read_chain(pool, tenant, "docket_signatures").await
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
pub async fn upsert_person(pool: &PgPool, tenant: Uuid, person: &Person) -> Result<()> {
    let record = serde_json::to_value(&person.value).context("serialising a person document")?;
    let mut tx = pool.begin().await.context("beginning a transaction")?;
    let account = enter(&mut tx, tenant).await?;
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
pub async fn read_person(pool: &PgPool, tenant: Uuid, slug: &str) -> Result<Option<Person>> {
    let mut tx = pool.begin().await.context("beginning a transaction")?;
    enter(&mut tx, tenant).await?;
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
pub async fn list_persons(pool: &PgPool, tenant: Uuid) -> Result<Vec<Person>> {
    let mut tx = pool.begin().await.context("beginning a transaction")?;
    enter(&mut tx, tenant).await?;
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
pub async fn insert_person(pool: &PgPool, tenant: Uuid, person: &Person) -> Result<()> {
    let record = serde_json::to_value(&person.value).context("serialising a person document")?;
    let mut tx = pool.begin().await.context("beginning a transaction")?;
    let account = enter(&mut tx, tenant).await?;
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

async fn seal_and_append<T>(pool: &PgPool, tenant: Uuid, table: &str, record: T) -> Result<T>
where
    T: Chained + Serialize + DeserializeOwned + Send + Unpin + 'static,
{
    let mut tx = pool.begin().await.context("beginning a transaction")?;
    let account = enter(&mut tx, tenant).await?;
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
}

impl PgStore {
    /// A Postgres-backed store for `tenant`, driving async work on `runtime`.
    #[must_use]
    pub fn new(pool: PgPool, tenant: Uuid, runtime: tokio::runtime::Handle) -> Self {
        Self {
            pool,
            tenant,
            runtime,
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
            .block_on(read_person(&self.pool, self.tenant, slug))?
            .with_context(|| format!("no person record for {slug}"))
    }

    fn person_exists(&self, slug: &str) -> Result<bool> {
        Ok(self
            .runtime
            .block_on(read_person(&self.pool, self.tenant, slug))?
            .is_some())
    }

    fn slugs(&self) -> Result<Vec<String>> {
        Ok(self
            .runtime
            .block_on(list_persons(&self.pool, self.tenant))?
            .into_iter()
            .map(|p| p.slug)
            .collect())
    }

    fn create_person(&self, slug: &str, body: &str) -> Result<()> {
        let person = Self::body_to_person(slug, body)?;
        self.runtime
            .block_on(insert_person(&self.pool, self.tenant, &person))
    }

    fn rewrite_person(&self, slug: &str, body: &str) -> Result<()> {
        let person = Self::body_to_person(slug, body)?;
        self.runtime
            .block_on(upsert_person(&self.pool, self.tenant, &person))
    }

    fn ledger(&self) -> Result<Vec<Record>> {
        self.runtime
            .block_on(read_identities(&self.pool, self.tenant))
    }

    fn append_identity(&self, record: Record) -> Result<Record> {
        self.runtime.block_on(seal_and_append(
            &self.pool,
            self.tenant,
            "docket_identities",
            record,
        ))
    }

    fn register(&self) -> Result<Vec<Entry>> {
        self.runtime
            .block_on(read_register(&self.pool, self.tenant))
    }

    fn append_register(&self, entry: Entry) -> Result<Entry> {
        self.runtime.block_on(seal_and_append(
            &self.pool,
            self.tenant,
            "docket_register",
            entry,
        ))
    }

    fn events(&self) -> Result<Vec<Event>> {
        self.runtime.block_on(read_events(&self.pool, self.tenant))
    }

    fn append_event(&self, event: Event) -> Result<Event> {
        self.runtime.block_on(seal_and_append(
            &self.pool,
            self.tenant,
            "docket_events",
            event,
        ))
    }

    fn signatures(&self) -> Result<Vec<Signature>> {
        self.runtime
            .block_on(read_signatures(&self.pool, self.tenant))
    }

    fn append_signature(&self, record: Signature) -> Result<Signature> {
        self.runtime.block_on(seal_and_append(
            &self.pool,
            self.tenant,
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
async fn read_config(pool: &PgPool, tenant: Uuid, table: &str) -> Result<Vec<serde_json::Value>> {
    let sql = format!("SELECT record FROM {table} ORDER BY id");
    let mut tx = pool.begin().await.context("beginning a transaction")?;
    enter(&mut tx, tenant).await?;
    let rows: Vec<(sqlx::types::Json<serde_json::Value>,)> = sqlx::query_as(&sql)
        .fetch_all(&mut *tx)
        .await
        .with_context(|| format!("reading config table {table}"))?;
    tx.commit().await.context("committing")?;
    Ok(rows.into_iter().map(|(json,)| json.0).collect())
}

impl TenantStores for PgTenantStores {
    fn build(&self, tenant: Uuid) -> Result<RequestStores<'_>> {
        let store = PgStore::new(self.pool.clone(), tenant, self.handle.clone());
        let blobs = blobs::open(&self.paths, &tenant.to_string(), Some(self.handle.clone()))
            .context("opening the blob store (DOCKET_S3_* configured?)")?;
        // The tenant's configuration from its 0008 rows, or `None` when it has none yet — the served
        // dispatcher then falls back to the git registry until it is seeded (ADR-0099).
        let config =
            Registry::from_db(|table| self.handle.block_on(read_config(&self.pool, tenant, table)))
                .context("loading the tenant's configuration")?;
        Ok((Box::new(store), blobs, config))
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

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
        let (store, _blobs, config) = factory.build(tenant).expect("build a store for the tenant");
        assert_eq!(store.slugs().expect("list slugs"), Vec::<String>::new());
        // A fresh tenant has no config rows, so the factory returns None and the dispatcher would
        // fall back to the git registry.
        assert!(config.is_none(), "an unseeded tenant has no config");
    }
}
