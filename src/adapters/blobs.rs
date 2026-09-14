//! The blob store: docket's document BYTES, behind one synchronous port (design §11, ADR-0094).
//!
//! [`Store`](crate::adapters::store::Store) moved docket's *records* to Postgres and left the *bytes*
//! — issued PDFs, packs, signed scans, the plaintext a vault seals — on the filesystem. That is a
//! sharp edge once docket is served: a served pod's filesystem is ephemeral, so a PDF rendered by
//! `onboard` over the API is lost when the pod restarts. This port ends that interim exactly the way
//! the record port did: one synchronous trait with two implementations — [`FileBlobs`] (the CLI's
//! offline/break-glass path, unchanged behaviour) and [`S3Blobs`] (Garage, the in-cluster S3 store
//! ADR-0094 chose).
//!
//! A blob is addressed by an opaque **key** — the relative path the register already records as
//! `pdf_path` (e.g. `issued/<person_id>/<file>.pdf`), or the same shape for scans and packs. The key
//! is frozen the first time it is written, because it lives inside an already-sealed register entry;
//! a later reshuffle would orphan every historical reference (the discipline the register imposes).
//!
//! [`S3Blobs`] uses `rusty-s3` to SigV4-sign a request URL and the crate's existing `reqwest`
//! (rustls) to send it — no new TLS stack, ADR-0005 held. `rusty-s3` is a pure signer with no HTTP or
//! TLS of its own. The client is async, so — like `PgStore` — it drives each request to completion on
//! a shared runtime with `block_on`; the callers are blocking threads (the served backend is
//! thread-per-request, the CLI is synchronous), never inside the runtime, so this is the sanctioned
//! bridge and not a nested-runtime hazard.

use crate::config::Paths;
use anyhow::{Context, Result, bail};
use std::fs;
use std::time::Duration;

/// How long a signed S3 URL is valid. The request is sent immediately after signing on the same
/// blocking thread, so this only has to cover one in-cluster round trip; ninety seconds is generous.
const SIGN_TTL: Duration = Duration::from_secs(90);

/// docket's blob store: document bytes behind one key (design §11).
///
/// The key is the register's `pdf_path` and the analogous relative path for scans and packs. `Sync`
/// is a supertrait for the same reason the record port carries it — a `&dyn Blobs` is shared across
/// the served backend's per-request threads.
pub trait Blobs: Sync {
    /// Store `bytes` under `key`, overwriting any existing object at that key.
    ///
    /// # Errors
    /// Fails if the write is refused.
    fn put(&self, key: &str, bytes: &[u8]) -> Result<()>;

    /// Read the bytes stored under `key`.
    ///
    /// # Errors
    /// Fails if the key is absent or the read is refused.
    fn get(&self, key: &str) -> Result<Vec<u8>>;

    /// Whether an object exists at `key`, without reading it.
    ///
    /// # Errors
    /// Fails only if existence cannot be determined.
    fn exists(&self, key: &str) -> Result<bool>;

    /// Delete the object at `key`. Idempotent: deleting an absent key is not an error, because erase
    /// (ADR-0069) must be able to run over a partially-populated store without failing on a gap.
    ///
    /// # Errors
    /// Fails if the delete is refused.
    fn delete(&self, key: &str) -> Result<()>;
}

/// The filesystem blob store: bytes under `store/<key>` (the CLI's offline/break-glass path).
///
/// This is the behaviour docket has always had, now behind the [`Blobs`] port: a key maps to
/// `paths.store.join(key)`, so `issued/<person_id>/<file>.pdf` lands exactly where `keep_pdf` used to
/// put it and every historical `pdf_path` still resolves. The CLI keeps it as its default, so the
/// offline path is unchanged (the existing suite is the proof).
pub struct FileBlobs<'a> {
    paths: &'a Paths,
}

impl<'a> FileBlobs<'a> {
    /// A file blob store rooted at `paths`.
    #[must_use]
    pub fn new(paths: &'a Paths) -> Self {
        Self { paths }
    }
}

impl Blobs for FileBlobs<'_> {
    fn put(&self, key: &str, bytes: &[u8]) -> Result<()> {
        let dest = self.paths.store.join(key);
        if let Some(dir) = dest.parent() {
            fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        }
        fs::write(&dest, bytes).with_context(|| format!("writing {}", dest.display()))
    }

    fn get(&self, key: &str) -> Result<Vec<u8>> {
        let src = self.paths.store.join(key);
        fs::read(&src).with_context(|| format!("reading {}", src.display()))
    }

    fn exists(&self, key: &str) -> Result<bool> {
        Ok(self.paths.store.join(key).exists())
    }

    fn delete(&self, key: &str) -> Result<()> {
        let path = self.paths.store.join(key);
        if path.exists() {
            fs::remove_file(&path).with_context(|| format!("deleting {}", path.display()))?;
        }
        Ok(())
    }
}

/// The S3 configuration docket reads from the environment, matching the `docket-s3` secret keys
/// (ADR-0094): `endpoint`, `bucket`, `access_key_id`, `secret_access_key`, `region`.
#[derive(Debug, Clone)]
pub struct S3Config {
    endpoint: String,
    bucket: String,
    access_key_id: String,
    secret_access_key: String,
    region: String,
}

impl S3Config {
    /// Read the S3 configuration from the environment, or `None` when it is not configured.
    ///
    /// "Configured" means the endpoint AND bucket are both set and non-empty; that is the selector
    /// (design §11): present → [`S3Blobs`], absent → [`FileBlobs`]. When the endpoint/bucket are set
    /// but a credential or the region is missing, that is a misconfiguration, not "use files", so it
    /// is reported as an error rather than silently falling back.
    ///
    /// # Errors
    /// Fails if the endpoint/bucket are set but a required credential or the region is missing.
    pub fn from_env() -> Result<Option<Self>> {
        let get = |k: &str| {
            std::env::var(k)
                .ok()
                .map(|v| v.trim().to_owned())
                .filter(|v| !v.is_empty())
        };
        let (Some(endpoint), Some(bucket)) = (get("DOCKET_S3_ENDPOINT"), get("DOCKET_S3_BUCKET"))
        else {
            return Ok(None);
        };
        let require = |k: &str| {
            get(k).with_context(|| {
                format!(
                    "DOCKET_S3_ENDPOINT is set, so the blob store is Garage/S3 and {k} is required"
                )
            })
        };
        Ok(Some(Self {
            endpoint,
            bucket,
            access_key_id: require("DOCKET_S3_ACCESS_KEY_ID")?,
            secret_access_key: require("DOCKET_S3_SECRET_ACCESS_KEY")?,
            region: get("DOCKET_S3_REGION").unwrap_or_else(|| "garage".to_owned()),
        }))
    }
}

/// The Garage/S3 blob store, one bucket, keys prefixed by the tenant (design §11, ADR-0094).
///
/// Selected when the S3 config is present. Every key is prefixed with the tenant id before it reaches
/// the bucket, so one tenant's keys can never collide with another's (the tenant-scoping invariant);
/// the prefix is an addressing detail of this layer and is NOT part of the `pdf_path` recorded in the
/// register, so already-recorded keys are unchanged.
pub struct S3Blobs {
    bucket: rusty_s3::Bucket,
    credentials: rusty_s3::Credentials,
    client: reqwest::Client,
    runtime: tokio::runtime::Handle,
    prefix: String,
}

impl S3Blobs {
    /// Build an S3 blob store from `config`, scoping keys under `tenant`, driving requests on
    /// `runtime`.
    ///
    /// `tenant` is the empty string when there is no tenant (the loopback/break-glass CLI): keys are
    /// then unprefixed, which is the single-tenant filesystem shape lifted into one bucket.
    ///
    /// # Errors
    /// Fails if the endpoint is not a valid URL or is missing a host.
    pub fn new(config: &S3Config, tenant: &str, runtime: tokio::runtime::Handle) -> Result<Self> {
        let endpoint = config.endpoint.parse().with_context(|| {
            format!(
                "DOCKET_S3_ENDPOINT={:?} is not a valid URL",
                config.endpoint
            )
        })?;
        // Path-style addressing: Garage serves `endpoint/bucket/key`, not virtual-hosted subdomains.
        let bucket = rusty_s3::Bucket::new(
            endpoint,
            rusty_s3::UrlStyle::Path,
            config.bucket.clone(),
            config.region.clone(),
        )
        .map_err(|e| anyhow::anyhow!("building the S3 bucket: {e:?}"))?;
        let credentials =
            rusty_s3::Credentials::new(&config.access_key_id, &config.secret_access_key);
        let client = reqwest::Client::builder()
            .build()
            .context("building the S3 HTTP client")?;
        Ok(Self {
            bucket,
            credentials,
            client,
            runtime,
            prefix: tenant.to_owned(),
        })
    }

    /// The tenant-scoped object key: `<tenant>/<key>`, or `<key>` when there is no tenant.
    fn scoped(&self, key: &str) -> String {
        if self.prefix.is_empty() {
            key.to_owned()
        } else {
            format!("{}/{key}", self.prefix)
        }
    }
}

impl Blobs for S3Blobs {
    fn put(&self, key: &str, bytes: &[u8]) -> Result<()> {
        use rusty_s3::S3Action as _;
        let object = self.scoped(key);
        let url = rusty_s3::actions::PutObject::new(&self.bucket, Some(&self.credentials), &object)
            .sign(SIGN_TTL);
        self.runtime.block_on(async {
            let resp = self
                .client
                .put(url)
                .body(bytes.to_vec())
                .send()
                .await
                .with_context(|| format!("PUT {object}"))?;
            let status = resp.status();
            if !status.is_success() {
                bail!("PUT {object} failed: {status}");
            }
            Ok(())
        })
    }

    fn get(&self, key: &str) -> Result<Vec<u8>> {
        use rusty_s3::S3Action as _;
        let object = self.scoped(key);
        let url = rusty_s3::actions::GetObject::new(&self.bucket, Some(&self.credentials), &object)
            .sign(SIGN_TTL);
        self.runtime.block_on(async {
            let resp = self
                .client
                .get(url)
                .send()
                .await
                .with_context(|| format!("GET {object}"))?;
            let status = resp.status();
            if !status.is_success() {
                bail!("GET {object} failed: {status}");
            }
            let bytes = resp
                .bytes()
                .await
                .with_context(|| format!("reading the body of GET {object}"))?;
            Ok(bytes.to_vec())
        })
    }

    fn exists(&self, key: &str) -> Result<bool> {
        use rusty_s3::S3Action as _;
        let object = self.scoped(key);
        let url =
            rusty_s3::actions::HeadObject::new(&self.bucket, Some(&self.credentials), &object)
                .sign(SIGN_TTL);
        self.runtime.block_on(async {
            let resp = self
                .client
                .head(url)
                .send()
                .await
                .with_context(|| format!("HEAD {object}"))?;
            let status = resp.status();
            if status.is_success() {
                Ok(true)
            } else if status == reqwest::StatusCode::NOT_FOUND {
                Ok(false)
            } else {
                bail!("HEAD {object} failed: {status}")
            }
        })
    }

    fn delete(&self, key: &str) -> Result<()> {
        use rusty_s3::S3Action as _;
        let object = self.scoped(key);
        let url =
            rusty_s3::actions::DeleteObject::new(&self.bucket, Some(&self.credentials), &object)
                .sign(SIGN_TTL);
        self.runtime.block_on(async {
            let resp = self
                .client
                .delete(url)
                .send()
                .await
                .with_context(|| format!("DELETE {object}"))?;
            let status = resp.status();
            // S3 delete is idempotent: an absent key returns 204/404, both fine.
            if status.is_success() || status == reqwest::StatusCode::NOT_FOUND {
                Ok(())
            } else {
                bail!("DELETE {object} failed: {status}")
            }
        })
    }
}

/// Whether the S3 blob store is configured (endpoint AND bucket both set).
///
/// The selector, kept separate from [`open`] so a caller can decide up front whether it needs a
/// runtime handle for the async S3 client.
#[must_use]
pub fn s3_configured() -> bool {
    let set = |k: &str| std::env::var(k).is_ok_and(|v| !v.trim().is_empty());
    set("DOCKET_S3_ENDPOINT") && set("DOCKET_S3_BUCKET")
}

/// Choose the blob store the same way the record store is chosen (design §11): S3 config present →
/// [`S3Blobs`], else [`FileBlobs`].
///
/// `tenant` scopes S3 keys by prefix; pass the empty string on the loopback/break-glass CLI.
/// `runtime` drives the async S3 client and is required only when S3 is configured.
///
/// # Errors
/// Fails if S3 is configured but a required setting is missing, no runtime handle was supplied, or
/// the bucket cannot be built.
pub fn open<'a>(
    paths: &'a Paths,
    tenant: &str,
    runtime: Option<tokio::runtime::Handle>,
) -> Result<Box<dyn Blobs + 'a>> {
    match S3Config::from_env()? {
        Some(config) => {
            let runtime = runtime.context(
                "the S3 blob store is configured but no tokio runtime is available to drive it",
            )?;
            Ok(Box::new(S3Blobs::new(&config, tenant, runtime)?))
        }
        None => Ok(Box::new(FileBlobs::new(paths))),
    }
}

#[cfg(test)]
// The crate denies expect/unwrap/indexing; an assertion IS the point of a test, so they are exempted
// here as in every other test module.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::{Blobs, FileBlobs, S3Blobs};
    use crate::config::Paths;
    use sha2::{Digest as _, Sha256};
    use std::collections::BTreeMap;
    use std::fmt::Write as _;
    use std::io::{Read as _, Write as _};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};

    fn sha256_hex(bytes: &[u8]) -> String {
        let mut h = Sha256::new();
        h.update(bytes);
        let mut out = String::new();
        for b in h.finalize() {
            let _ = write!(out, "{b:02x}");
        }
        out
    }

    #[test]
    fn file_blobs_round_trips_and_preserves_the_digest() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = Paths::resolve(dir.path());
        let blobs = FileBlobs::new(&paths);

        let key = "issued/PAR-INT-2026-0001/NDA-0001.pdf";
        let bytes = b"%PDF-1.7 a genuine-enough document".to_vec();
        let digest = sha256_hex(&bytes);

        assert!(!blobs.exists(key).expect("exists"));
        blobs.put(key, &bytes).expect("put");
        assert!(blobs.exists(key).expect("exists"));

        // The seal-preservation analogue: bytes verify against the recorded digest after a round trip.
        let read = blobs.get(key).expect("get");
        assert_eq!(
            sha256_hex(&read),
            digest,
            "digest preserved through the blob store"
        );

        // The key maps to the exact filesystem path the register's pdf_path resolves to.
        assert!(paths.store.join(key).exists(), "key maps to store/<key>");

        blobs.delete(key).expect("delete");
        assert!(
            !blobs.exists(key).expect("exists"),
            "erase deletes the blob"
        );
        // Idempotent: deleting an absent key is not an error.
        blobs.delete(key).expect("delete-absent");
    }

    /// A minimal in-memory S3 over a real `TcpListener`: enough of the verbs to exercise `S3Blobs`
    /// end to end (put/get/exists/delete and the digest round trip) without Garage. It ignores the
    /// `SigV4` query string — it is testing our request shape and byte handling, not re-verifying the
    /// signer — and keys objects by the request path so tenant prefixing is observable.
    fn spawn_mock_s3(store: Arc<Mutex<BTreeMap<String, Vec<u8>>>>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut buf = vec![0u8; 65536];
                let n = stream.read(&mut buf).unwrap_or(0);
                let req = &buf[..n];
                // Split head/body on the blank line.
                let split = req.windows(4).position(|w| w == b"\r\n\r\n");
                let (head, body) = match split {
                    Some(i) => (&req[..i], &req[i + 4..]),
                    None => (req, &b""[..]),
                };
                let head_str = String::from_utf8_lossy(head);
                let mut lines = head_str.lines();
                let request_line = lines.next().unwrap_or("");
                let mut parts = request_line.split_whitespace();
                let method = parts.next().unwrap_or("");
                let target = parts.next().unwrap_or("");
                let path = target.split('?').next().unwrap_or("").to_owned();
                // Content-Length lets us read any body bytes that did not fit the first read.
                let content_length: usize = head_str
                    .lines()
                    .find_map(|l| {
                        l.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .map(|v| v.trim().parse().unwrap_or(0))
                    })
                    .unwrap_or(0);
                let mut body = body.to_vec();
                while body.len() < content_length {
                    let mut more = vec![0u8; 65536];
                    let m = stream.read(&mut more).unwrap_or(0);
                    if m == 0 {
                        break;
                    }
                    body.extend_from_slice(&more[..m]);
                }

                let mut map = store.lock().expect("lock");
                let response: Vec<u8> = match method {
                    "PUT" => {
                        map.insert(path, body);
                        b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n".to_vec()
                    }
                    "GET" => match map.get(&path) {
                        Some(bytes) => {
                            let mut r = format!(
                                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n",
                                bytes.len()
                            )
                            .into_bytes();
                            r.extend_from_slice(bytes);
                            r
                        }
                        None => b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_vec(),
                    },
                    "HEAD" => {
                        if map.contains_key(&path) {
                            b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n".to_vec()
                        } else {
                            b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_vec()
                        }
                    }
                    "DELETE" => {
                        map.remove(&path);
                        b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n".to_vec()
                    }
                    _ => b"HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\n\r\n".to_vec(),
                };
                drop(map);
                let _ = stream.write_all(&response);
                let _ = stream.flush();
            }
        });
        format!("http://{addr}")
    }

    #[test]
    fn s3_blobs_round_trips_preserves_the_digest_and_scopes_by_tenant() {
        let store: Arc<Mutex<BTreeMap<String, Vec<u8>>>> = Arc::new(Mutex::new(BTreeMap::new()));
        let endpoint = spawn_mock_s3(Arc::clone(&store));
        // Multi-thread, matching production: `S3Blobs` drives reqwest via `Handle::block_on`, which
        // needs the runtime's IO driver on a worker thread — a current-thread runtime would not pump
        // it and the request would hang.
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("runtime");

        let config = super::S3Config {
            endpoint,
            bucket: "docket".to_owned(),
            access_key_id: "key".to_owned(),
            secret_access_key: "secret".to_owned(),
            region: "garage".to_owned(),
        };
        let tenant = "11111111-1111-1111-1111-111111111111";
        let blobs = S3Blobs::new(&config, tenant, runtime.handle().clone()).expect("s3 blobs");

        let key = "issued/PAR-INT-2026-0001/NDA-0001.pdf";
        let bytes = b"%PDF-1.7 a genuine-enough document".to_vec();
        let digest = sha256_hex(&bytes);

        assert!(!blobs.exists(key).expect("exists"));
        blobs.put(key, &bytes).expect("put");
        assert!(blobs.exists(key).expect("exists"));

        let read = blobs.get(key).expect("get");
        assert_eq!(
            sha256_hex(&read),
            digest,
            "digest preserved through S3 round trip"
        );

        // Tenant-scoping invariant: the object landed under the tenant prefix in the bucket,
        // path-style => /<bucket>/<tenant>/<key>.
        let map = store.lock().expect("lock");
        let expected_path = format!("/docket/{tenant}/{key}");
        assert!(
            map.contains_key(&expected_path),
            "object keyed by tenant-scoped path; keys present: {:?}",
            map.keys().collect::<Vec<_>>()
        );
        drop(map);

        blobs.delete(key).expect("delete");
        assert!(
            !blobs.exists(key).expect("exists"),
            "erase deletes the blob"
        );
        blobs.delete(key).expect("delete-absent is idempotent");
    }
}
