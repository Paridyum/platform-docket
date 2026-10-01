//! Tenant seeding shared by the Postgres integration suites (`pg_store.rs`, `served_api.rs`).
//!
//! ONE COPY, because the copies rotted. Both suites seeded tenants with the schema as it was at
//! migration 0006 — no `region` (NOT NULL since tenancy 0011) and no docket account (which `enter()`
//! requires since 0012) — and both skipped silently without a database, so nobody saw it. The next
//! schema change to `tenants` or `accounts` is made here once.
//!
//! THROUGH AN ADMIN CONNECTION (`DOCKET_ADMIN_DATABASE_URL`, falling back to the test's own URL), so a
//! suite does not depend on the app role being able to write the control-plane registry. The
//! harness's grants — mirrored from platform-tenancy's probe — happen to be broader than that; the
//! deployment's need not be.

use sqlx::{Connection, PgConnection};
use uuid::Uuid;

/// Seed a fresh tenant and its default docket account, and return the tenant's id.
pub async fn seed_tenant(test_url: &str, name: &str) -> Uuid {
    let admin_url =
        std::env::var("DOCKET_ADMIN_DATABASE_URL").unwrap_or_else(|_| test_url.to_owned());
    // One connection, not a pool: two statements, then closed.
    let mut admin = PgConnection::connect(&admin_url)
        .await
        .expect("connect (admin)");
    // The rows below are written through THIS connection, so it is the one the scratch guard must
    // check — not only the test's own pool. A leftover DOCKET_ADMIN_DATABASE_URL pointing at a real
    // database would otherwise seed a test tenant into the real control-plane tables.
    if let Ok(scratch) = std::env::var("DOCKET_PROBE_DB") {
        let (db,): (String,) = sqlx::query_as("SELECT current_database()")
            .fetch_one(&mut admin)
            .await
            .expect("current_database (admin)");
        assert_eq!(
            db, scratch,
            "the admin connection is not the named scratch database"
        );
    }
    let tenant = Uuid::new_v4();
    // Every tenant has an owning org since tenancy's 2026-09-28 rebuild (knowledge ADR-0114 D4), so
    // the org comes first. Both registry rows are personless (ADR-0117 D3): the tenant's display name
    // lives with the tenant, in `tenant_settings`, beside its locale declarations (ADR-0122 D4).
    let org = Uuid::new_v4();
    sqlx::query("INSERT INTO orgs (id, region) VALUES ($1, 'in')")
        .bind(org)
        .execute(&mut admin)
        .await
        .expect("seed a fresh org");
    sqlx::query(
        "INSERT INTO tenants (id, owner_org_id, slug, zitadel_org_id, region) \
         VALUES ($1, $2, $3, $4, 'in')",
    )
    .bind(tenant)
    .bind(org)
    .bind(format!("test-{tenant}"))
    .bind(tenant.simple().to_string())
    .execute(&mut admin)
    .await
    .expect("seed a fresh tenant");
    // `tenant_settings` and `accounts` are tenant-scoped and FORCE row-level security, which binds the
    // table owner too — so the admin connection writes them in the new tenant's operator session, as
    // provisioning does. Session-level (`is_local = false`) on this one connection, closed below, so
    // it cannot leak into a pooled connection. Without it every suite failed here with "paridyum.
    // tenant_id is not set" as soon as CI first reached a real database (2026-09-30).
    sqlx::query(
        "SELECT set_config('paridyum.tenant_id', $1, false), \
                set_config('paridyum.account_id', '00000000-0000-0000-0000-000000000000', false), \
                set_config('paridyum.operator_scope', 'true', false)",
    )
    .bind(tenant.to_string())
    .execute(&mut admin)
    .await
    .expect("enter the new tenant's operator session");
    sqlx::query(
        "INSERT INTO tenant_settings \
           (tenant_id, display_name, supported_locales, content_locale, locale, timezone, currency) \
         VALUES ($1, $2, ARRAY['en', 'ar'], 'en', 'en-IN', 'Asia/Kolkata', 'INR')",
    )
    .bind(tenant)
    .bind(name)
    .execute(&mut admin)
    .await
    .expect("seed the tenant's declarations");
    // ADR-0078 D2 says no INSERT names the surrogate key — the column's `uuidv7()` default mints it.
    // `accounts.id` HAS no default until tenancy 0014, and these suites must run against the schema
    // docket is deployed into (0014 pending on the cluster today), so this is the ADR's own case of a
    // missing default being fixed, not a second call site: a v7 key (ADR-0078 D1), named only because
    // the default does not exist yet. Remove the `id` here once 0014 is applied everywhere it runs.
    sqlx::query(
        "INSERT INTO accounts (id, tenant_id, product, slug, display_name) \
         VALUES ($1, $2, 'docket', 'default', 'Docket')",
    )
    .bind(Uuid::now_v7())
    .bind(tenant)
    .execute(&mut admin)
    .await
    .expect("seed the tenant's docket account");
    admin.close().await.expect("close (admin)");
    tenant
}
