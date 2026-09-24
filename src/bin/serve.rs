//! The docket served backend (ADR-0093): configure from the environment, fetch the control-plane
//! JWKS once, open the Postgres store, and serve the REAL operator API — the same
//! `adapters::http` router the console already speaks, now over Postgres with token authentication.
//! The everyday interface for all of Paridyum's HR staff; the CLI (`docket`) remains as a break-glass
//! admin path against the same database (ADR-0093 §3).
//!
//! This is not a second server: it builds the same `router::Context` the CLI's loopback `serve`
//! builds, differing only in its store (a `PgStore`, not a `FileStore`) and its authentication (a
//! verified operator token, ADR-0092, not the attribution header). The transport, the routes, the
//! rules and the wire contract are `adapters::http`, unchanged (design §5).
//!
//! Configuration is environment-only (RUN-0004: secrets arrive decrypted at deploy, never read from
//! a file here):
//!
//! ```text
//! DOCKET_DATABASE_URL   the docket database (sqlx)
//! DOCKET_OIDC_ISSUER    the control-plane issuer, e.g. https://id.paridyum.com
//! DOCKET_OIDC_JWKS      JWKS URL (default: {issuer}/oauth/v2/keys)
//! DOCKET_OIDC_AUDIENCE  the paridyum-operations project id a token must be audienced for
//! DOCKET_TENANT_ID      the tenant whose HR records this instance serves (a uuid)
//! DOCKET_BIND           listen address (default 0.0.0.0:8080)
//! DOCKET_ROOT           where registry/ lives (default: the current directory)
//!
//! PARIDYUM_BACKUPS_CONFIRMED  "1" once an off-host backup destination exists AND a restore has
//!                             been rehearsed (ADR-0064 D3). Until then every route that writes a
//!                             person record answers 503; reads are unaffected.
//! ```

use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::path::Path;
use tool_docket::adapters::auth::{OperatorVerifier, keys_from_jwks};
use tool_docket::adapters::clock::SystemClock;
use tool_docket::adapters::http::server::{self, Backend, Serving};
use tool_docket::adapters::pg::{self, PgTenantStores};
use tool_docket::adapters::registry::Registry;
use tool_docket::config::Paths;

fn env(key: &str) -> Result<String> {
    std::env::var(key).with_context(|| format!("required environment variable {key} is unset"))
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // A multi-threaded runtime the process owns for its whole life. The served backend is
    // thread-per-request and synchronous (design §2); `PgStore` drives sqlx to completion on THIS
    // runtime's handle from each request thread, so it must outlive every request — hence built here
    // and kept in `main`, never entered.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("building the tokio runtime")?;

    let issuer = env("DOCKET_OIDC_ISSUER")?;
    let jwks_url =
        std::env::var("DOCKET_OIDC_JWKS").unwrap_or_else(|_| format!("{issuer}/oauth/v2/keys"));
    let audience = env("DOCKET_OIDC_AUDIENCE")?;
    // No DOCKET_TENANT_ID: the served backend derives the tenant from each operator token's org
    // (ADR-0099/ADR-0082), so one deployment serves every tenant. The break-glass CLI still names
    // its one tenant explicitly; only the served path became multi-tenant.
    let bind: SocketAddr = std::env::var("DOCKET_BIND")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_owned())
        .parse()
        .context("DOCKET_BIND is not a socket address, e.g. 0.0.0.0:8080")?;
    let root = std::env::var("DOCKET_ROOT").unwrap_or_else(|_| ".".to_owned());
    let database_url = env("DOCKET_DATABASE_URL")?;

    // Fetch the signing keys once at startup, off the request path (ADR-0047 D4). A periodic refresh
    // and rotation-on-unknown-kid is the next slice (SOP). The pool is opened on the same runtime.
    let (keys, pool) = runtime.block_on(async {
        let jwks: serde_json::Value = reqwest::get(&jwks_url)
            .await
            .with_context(|| format!("fetching JWKS from {jwks_url}"))?
            .json()
            .await
            .context("the JWKS response was not JSON")?;
        let keys = keys_from_jwks(&jwks);
        anyhow::ensure!(!keys.is_empty(), "no usable keys in the JWKS at {jwks_url}");
        let pool = pg::pool(&database_url).await?;
        Ok::<_, anyhow::Error>((keys, pool))
    })?;
    tracing::info!(key_count = keys.len(), issuer, "loaded control-plane JWKS");

    let verifier = OperatorVerifier::new(keys, issuer, audience);
    let registry = Registry::load(&Paths::resolve(Path::new(&root)))
        .context("loading registry/ (is DOCKET_ROOT pointing at the repository?)")?;
    // The served backend's blob store is Garage/S3 (design §11, ADR-0094), the whole reason a
    // document rendered by `onboard` over the API is now durable across a pod restart: the bytes go to
    // the bucket, not the ephemeral pod filesystem. Keys are scoped by tenant. The async S3 client is
    // driven on the same runtime the PgStore uses. `paths` remains for the remaining filesystem
    // artifacts (generated indexes, templates).
    let paths = Paths::resolve(Path::new(&root));
    let clock = SystemClock;
    // The record + blob stores are built PER REQUEST from the token's tenant (design §11, ADR-0099).
    let stores = PgTenantStores::new(pool, runtime.handle().clone(), paths.clone());

    // ADR-0064 D3 / ADR-0068 §5 / ADR-0093 §5, and migration 0006's own header: schema and
    // SYNTHETIC rows now, the first REAL personnel record only once an off-host backup destination
    // exists and a restore has been rehearsed. `web-hiring` has enforced the identical gate on its
    // apply route since it shipped; docket — which is what actually writes personnel records — has
    // enforced nothing until now.
    //
    // Exactly "1", never a truthy string. `PARIDYUM_BACKUPS_CONFIRMED=false` reads as confirmation
    // to any check that tests for presence, and that is the shape of accident this gate exists to
    // stop.
    let backups_confirmed = std::env::var("PARIDYUM_BACKUPS_CONFIRMED").as_deref() == Ok("1");
    if backups_confirmed {
        tracing::warn!(
            "PARIDYUM_BACKUPS_CONFIRMED=1: real personnel records may be written. This asserts an \
             off-host backup destination AND a rehearsed restore (ADR-0064 D3)."
        );
    } else {
        tracing::info!(
            "backup gate CLOSED: person writes return 503 until PARIDYUM_BACKUPS_CONFIRMED=1. \
             Reads and synthetic use are unaffected."
        );
    }

    let serving = Serving {
        // The verifier's presence is exactly what permits a routable bind (design §5, server.rs).
        backend: Backend::PerTenant { stores: &stores },
        paths: &paths,
        registry: &registry,
        verifier: Some(&verifier),
        clock: &clock,
        backups_confirmed,
    };

    tracing::info!(addr = %bind, "docket serving the real operator API over Postgres (multi-tenant)");
    server::serve(&serving, bind).context("the server exited with an error")?;
    Ok(())
}
