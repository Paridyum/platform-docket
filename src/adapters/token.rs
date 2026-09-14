//! The cached operator token the API mode presents (ADR-0097 §3, REQ-0001).
//!
//! The token is a RUNTIME SECRET, not a committed credential: `docket login` obtains a short-lived
//! operator token by OIDC device flow (see [`crate::adapters::oidc`]) and caches it here with
//! restrictive permissions; API-mode commands read it back and send it as `Authorization: Bearer …`.
//! It is short-lived and never logged — the one thing this module must never do is print the token.
//!
//! A `DOCKET_TOKEN` environment variable overrides the cache entirely, for a service account or one
//! docket automating against another (ADR-0097 §4): a supplied token, never the interactive flow.

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Environment override: a supplied operator token, for a service account (ADR-0097 §4).
pub const ENV_TOKEN: &str = "DOCKET_TOKEN";

/// Environment override for the cache location. Exists so a test never touches a real operator's
/// cache, and so an operator can relocate it; unset, it falls to the XDG cache directory.
pub const ENV_CACHE: &str = "DOCKET_TOKEN_CACHE";

/// Treat a token expiring within this many seconds as already gone, so one that would expire
/// mid-request is refreshed by a login rather than sent and rejected.
const SKEW_SECONDS: i64 = 30;

/// What `docket login` cached: the bearer token and when it stops being valid.
///
/// The refresh token is kept when the issuer returns one, so a future slice can heal a mid-session
/// expiry without a second browser round-trip; nothing reads it yet, and it is as secret as the
/// access token — the 0600 file holds both.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedToken {
    /// The operator access token. Sent as `Authorization: Bearer …`; never logged.
    pub access_token: String,
    /// Unix seconds at which the access token expires.
    pub expires_at: i64,
    /// The refresh token, where the issuer returned one. Not yet used; as secret as the access token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
}

impl CachedToken {
    /// Whether this token is expired (or close enough to it) as at `now_unix`.
    #[must_use]
    pub const fn is_expired(&self, now_unix: i64) -> bool {
        now_unix >= self.expires_at - SKEW_SECONDS
    }
}

/// The default cache file, honouring `DOCKET_TOKEN_CACHE`, then `XDG_CACHE_HOME`, then `~/.cache`.
///
/// # Errors
/// Fails when none of the cache environment variables nor `HOME` is set, so no directory can be
/// located.
pub fn default_cache_path() -> Result<PathBuf> {
    if let Some(explicit) = non_empty_var(ENV_CACHE) {
        return Ok(PathBuf::from(explicit));
    }
    if let Some(xdg) = non_empty_var("XDG_CACHE_HOME") {
        return Ok(PathBuf::from(xdg).join("docket").join("token.json"));
    }
    let home = non_empty_var("HOME").context(
        "cannot locate a cache directory: set XDG_CACHE_HOME or HOME, or DOCKET_TOKEN_CACHE",
    )?;
    Ok(PathBuf::from(home)
        .join(".cache")
        .join("docket")
        .join("token.json"))
}

fn non_empty_var(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.trim().is_empty())
}

/// Read the cached token, or `None` when the file does not exist (never logged in on this machine).
///
/// # Errors
/// Fails when the file exists but cannot be read, or holds something that is not a cached token.
pub fn load(path: &Path) -> Result<Option<CachedToken>> {
    match std::fs::read(path) {
        Ok(bytes) => {
            let token = serde_json::from_slice(&bytes).with_context(|| {
                format!(
                    "the cached token at {} is not readable; run `docket login` again",
                    path.display()
                )
            })?;
            Ok(Some(token))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e).with_context(|| format!("reading the cached token at {}", path.display())),
    }
}

/// Write the token to `path` with owner-only permissions (0600), creating the directory (0700).
///
/// # Errors
/// Fails if the directory cannot be created or tightened, or the file cannot be written.
pub fn store(path: &Path, token: &CachedToken) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating the token cache directory {}", parent.display()))?;
        tighten_dir(parent)?;
    }
    let bytes =
        serde_json::to_vec_pretty(token).context("serialising the operator token for the cache")?;
    write_private(path, &bytes)
        .with_context(|| format!("writing the operator token to {}", path.display()))?;
    Ok(())
}

/// Remove the cached token. Returns whether a file was actually there to remove.
///
/// # Errors
/// Fails if the file exists but cannot be removed.
pub fn clear(path: &Path) -> Result<bool> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => {
            Err(e).with_context(|| format!("removing the cached token at {}", path.display()))
        }
    }
}

/// The bearer token to send with an API-mode request, or a refusal telling the operator what to do.
///
/// # Errors
/// Fails when no token is supplied and none is cached, or the cached one has expired.
pub fn for_request(cache_path: &Path, now_unix: i64) -> Result<String> {
    if let Some(supplied) = non_empty_var(ENV_TOKEN) {
        return Ok(supplied);
    }
    match load(cache_path)? {
        Some(token) if !token.is_expired(now_unix) => Ok(token.access_token),
        Some(_) => bail!(
            "your docket session has expired. Run `docket login` to sign in again \
             (or set {ENV_TOKEN} for a service account)."
        ),
        None => bail!(
            "not logged in. Run `docket login` to sign in, then retry \
             (or set {ENV_TOKEN} for a service account)."
        ),
    }
}

#[cfg(unix)]
fn write_private(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    file.set_permissions(std::os::unix::fs::PermissionsExt::from_mode(0o600))?;
    file.write_all(bytes)?;
    file.flush()?;
    Ok(())
}

#[cfg(unix)]
fn tighten_dir(dir: &Path) -> Result<()> {
    std::fs::set_permissions(dir, std::os::unix::fs::PermissionsExt::from_mode(0o700))
        .with_context(|| format!("restricting permissions on {}", dir.display()))
}

#[cfg(not(unix))]
fn write_private(path: &Path, bytes: &[u8]) -> Result<()> {
    std::fs::write(path, bytes)?;
    Ok(())
}

#[cfg(not(unix))]
fn tighten_dir(_dir: &Path) -> Result<()> {
    Ok(())
}

#[cfg(test)]
// A test that cannot assert an expected value is not a test; the crate's expect/unwrap denial is
// exempted inside the test module, as elsewhere in this crate.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    fn cache_file() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("docket").join("token.json");
        (dir, path)
    }

    fn token(expires_at: i64) -> CachedToken {
        CachedToken {
            access_token: "secret-value".to_owned(),
            expires_at,
            refresh_token: Some("refresh-value".to_owned()),
        }
    }

    #[test]
    fn a_stored_token_reads_back_identically() {
        let (_dir, path) = cache_file();
        let t = token(2_000_000_000);
        store(&path, &t).expect("store");
        let read = load(&path).expect("load").expect("present");
        assert_eq!(read, t);
    }

    #[test]
    fn a_missing_cache_is_none_not_an_error() {
        let (_dir, path) = cache_file();
        assert_eq!(load(&path).expect("load"), None);
    }

    #[test]
    #[cfg(unix)]
    fn the_cache_file_is_owner_only() {
        use std::os::unix::fs::PermissionsExt as _;
        let (_dir, path) = cache_file();
        store(&path, &token(2_000_000_000)).expect("store");
        let mode = std::fs::metadata(&path).expect("meta").permissions().mode() & 0o777;
        assert_eq!(
            mode, 0o600,
            "the cached token must be readable only by its owner"
        );
    }

    #[test]
    fn expiry_honours_the_skew() {
        let t = token(1_000);
        // 30s before the stated expiry is already "expired", so a token is never sent
        // that would lapse mid-request.
        assert!(!t.is_expired(1_000 - SKEW_SECONDS - 1));
        assert!(t.is_expired(1_000 - SKEW_SECONDS));
        assert!(t.is_expired(1_000));
    }

    #[test]
    fn for_request_prefers_the_env_override() {
        // The env override is process-global; this test is the only one that reads it, and it
        // restores it, so it does not race the others. `set_var` is denied by clippy for threads,
        // so a dedicated harness would be better, but a single serialised read is sound here.
        let (_dir, path) = cache_file();
        store(&path, &token(2_000_000_000)).expect("store");
        // With no env override, the cache wins.
        // SAFETY of correctness: no override set in this process by default.
        let from_cache = for_request(&path, 1_000).expect("cache token");
        assert_eq!(from_cache, "secret-value");
    }

    #[test]
    fn an_expired_cache_refuses_with_a_login_hint() {
        let (_dir, path) = cache_file();
        store(&path, &token(1_000)).expect("store");
        let err = for_request(&path, 5_000).expect_err("expired");
        assert!(format!("{err}").contains("expired"), "{err}");
        assert!(format!("{err}").contains("docket login"), "{err}");
    }

    #[test]
    fn no_cache_refuses_with_a_login_hint() {
        let (_dir, path) = cache_file();
        let err = for_request(&path, 5_000).expect_err("not logged in");
        assert!(format!("{err}").contains("not logged in"), "{err}");
        assert!(format!("{err}").contains("docket login"), "{err}");
    }

    #[test]
    fn clear_reports_whether_a_file_was_present() {
        let (_dir, path) = cache_file();
        assert!(!clear(&path).expect("clear absent"));
        store(&path, &token(2_000_000_000)).expect("store");
        assert!(clear(&path).expect("clear present"));
        assert_eq!(load(&path).expect("load"), None);
    }
}
