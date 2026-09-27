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

use sqlx::PgPool;
use tool_docket::adapters::pg;
use uuid::Uuid;

/// Seed a fresh tenant and its default docket account, and return the tenant's id.
pub async fn seed_tenant(test_url: &str, name: &str) -> Uuid {
    let admin_url =
        std::env::var("DOCKET_ADMIN_DATABASE_URL").unwrap_or_else(|_| test_url.to_owned());
    let admin: PgPool = pg::pool(&admin_url).await.expect("connect (admin)");
    let tenant = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO tenants (id, slug, name, zitadel_org_id, region) VALUES ($1, $2, $3, $4, 'in')",
    )
    .bind(tenant)
    .bind(format!("test-{tenant}"))
    .bind(name)
    .bind(tenant.simple().to_string())
    .execute(&admin)
    .await
    .expect("seed a fresh tenant");
    sqlx::query(
        "INSERT INTO accounts (tenant_id, product, slug, display_name) \
         VALUES ($1, 'docket', 'default', 'Docket')",
    )
    .bind(tenant)
    .execute(&admin)
    .await
    .expect("seed the tenant's docket account");
    admin.close().await;
    tenant
}
