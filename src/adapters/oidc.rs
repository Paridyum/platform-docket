//! The OIDC device authorization flow `docket login` runs (ADR-0097 §3).
//!
//! The CLI is a public client with no browser of its own, so it uses the OAuth 2.0 device
//! authorization grant (RFC 8628) against the control-plane instance (`id.paridyum.com`, the same
//! issuer `adapters::auth` verifies against). It asks the issuer for a device code, prints a URL and
//! a short user code for the operator to enter in a browser, then polls the token endpoint until the
//! sign-in is approved. The token that comes back is the operator token the served backend verifies
//! (ADR-0092) — audienced at the `paridyum-operations` project via the `…:aud` scope so it matches
//! the audience `auth.rs` pins.
//!
//! Everything here is pure request/response: [`request_device_code`] and [`poll_once`] do one HTTP
//! round-trip each and return a parsed outcome. The poll LOOP — the wait between attempts, honouring
//! `slow_down` — lives in the binary, so this module holds no timing and stays testable against a
//! local mock server.

use anyhow::{Context, Result, bail};
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::Value;
use std::fmt::Write as _;

/// The control-plane issuer to run the flow against.
pub const ENV_ISSUER: &str = "DOCKET_OIDC_ISSUER";
/// The device-flow public client id registered for the CLI. No default: a wrong client id fails
/// obscurely at the issuer, so an unset one is refused up front (ADR-0097 §3).
pub const ENV_CLIENT_ID: &str = "DOCKET_OIDC_CLIENT_ID";
/// The `paridyum-operations` project id, used to audience the token (`…:aud` scope).
pub const ENV_PROJECT_ID: &str = "DOCKET_OIDC_PROJECT_ID";

/// The issuer used when [`ENV_ISSUER`] is unset — the control plane `auth.rs` also pins.
pub const DEFAULT_ISSUER: &str = "https://id.paridyum.com";

/// The device-code grant type sent to the token endpoint (RFC 8628).
const DEVICE_GRANT: &str = "urn:ietf:params:oauth:grant-type:device_code";

/// How docket is configured to reach the issuer for a login.
#[derive(Debug, Clone)]
pub struct OidcConfig {
    /// The issuer base, e.g. `https://id.paridyum.com`. No trailing slash is assumed.
    pub issuer: String,
    /// The device-flow public client id. Required.
    pub client_id: String,
    /// The project id to audience the token for. Optional here, but a token minted without the
    /// `…:aud` scope will be refused by the served backend, which pins the audience (ADR-0092).
    pub project_id: Option<String>,
}

impl OidcConfig {
    /// Read the configuration from the environment.
    ///
    /// # Errors
    /// Fails when [`ENV_CLIENT_ID`] is unset or blank: there is deliberately no default client id.
    pub fn from_env() -> Result<Self> {
        let issuer = non_empty(ENV_ISSUER).unwrap_or_else(|| DEFAULT_ISSUER.to_owned());
        let client_id = non_empty(ENV_CLIENT_ID).with_context(|| {
            format!(
                "{ENV_CLIENT_ID} is unset. `docket login` needs the device-flow public client id \
                 registered for the CLI in paridyum-operations; there is no default."
            )
        })?;
        Ok(Self {
            issuer,
            client_id,
            project_id: non_empty(ENV_PROJECT_ID),
        })
    }

    fn scope(&self) -> String {
        let mut scope = String::from("openid profile");
        if let Some(project) = &self.project_id {
            // Zitadel's audience-reservation scope: it adds the project id to the token's `aud`,
            // which is what makes the served backend's audience check pass (ADR-0092).
            let _ = write!(scope, " urn:zitadel:iam:org:project:id:{project}:aud");
        }
        scope
    }

    fn device_endpoint(&self) -> String {
        format!(
            "{}/oauth/v2/device_authorization",
            self.issuer.trim_end_matches('/')
        )
    }

    fn token_endpoint(&self) -> String {
        format!("{}/oauth/v2/token", self.issuer.trim_end_matches('/'))
    }
}

fn non_empty(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.trim().is_empty())
}

/// What the device-authorization endpoint returned: the codes and how to poll.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DeviceAuthorization {
    /// The device code, presented back to the token endpoint while polling. Secret-ish; not printed.
    pub device_code: String,
    /// The short code the operator types into the browser.
    pub user_code: String,
    /// Where the operator goes to enter the code.
    pub verification_uri: String,
    /// A URL with the code pre-filled, where the issuer offers one.
    #[serde(default)]
    pub verification_uri_complete: Option<String>,
    /// Seconds until the device code expires.
    pub expires_in: i64,
    /// Minimum seconds between polls the issuer asks for.
    #[serde(default = "default_interval")]
    pub interval: u64,
}

const fn default_interval() -> u64 {
    5
}

/// A successful token response from the token endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TokenResponse {
    /// The operator access token. Sent as `Authorization: Bearer …`; never logged.
    pub access_token: String,
    /// Seconds until the access token expires.
    pub expires_in: i64,
    /// A refresh token, where the issuer returned one. Kept for a future healing slice.
    #[serde(default)]
    pub refresh_token: Option<String>,
}

/// The result of one poll of the token endpoint (RFC 8628 §3.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PollOutcome {
    /// The operator has not finished signing in; keep polling.
    Pending,
    /// Polling too fast; slow down and keep polling.
    SlowDown,
    /// The operator declined the sign-in.
    Denied,
    /// The device code expired before the operator finished; start over.
    Expired,
    /// The sign-in succeeded; here is the token.
    Approved(TokenResponse),
}

/// Ask the issuer for a device code.
///
/// # Errors
/// Fails if the request cannot be sent, the response is not success, or it is not the expected JSON.
pub fn request_device_code(http: &Client, cfg: &OidcConfig) -> Result<DeviceAuthorization> {
    let scope = cfg.scope();
    let resp = http
        .post(cfg.device_endpoint())
        .form(&[
            ("client_id", cfg.client_id.as_str()),
            ("scope", scope.as_str()),
        ])
        .send()
        .context("requesting a device code from the issuer")?;
    let status = resp.status();
    let body = resp.bytes().context("reading the device-code response")?;
    if !status.is_success() {
        bail!(
            "the issuer refused the device-code request ({}): {}",
            status.as_u16(),
            String::from_utf8_lossy(&body)
        );
    }
    serde_json::from_slice(&body).context("the device-code response was not the expected JSON")
}

/// Poll the token endpoint once.
///
/// # Errors
/// Fails if the request cannot be sent, or the endpoint returns an error this flow does not model.
pub fn poll_once(http: &Client, cfg: &OidcConfig, device_code: &str) -> Result<PollOutcome> {
    let resp = http
        .post(cfg.token_endpoint())
        .form(&[
            ("grant_type", DEVICE_GRANT),
            ("device_code", device_code),
            ("client_id", cfg.client_id.as_str()),
        ])
        .send()
        .context("polling the token endpoint")?;
    let status = resp.status();
    let body = resp.bytes().context("reading the token response")?;
    classify(status.is_success(), status.as_u16(), &body)
}

/// Turn one token-endpoint reply into a [`PollOutcome`]. Split out so it is unit-testable without a
/// socket, and so the set of modelled `error` codes is stated in one place.
fn classify(success: bool, status: u16, body: &[u8]) -> Result<PollOutcome> {
    if success {
        let token: TokenResponse =
            serde_json::from_slice(body).context("the token response was not the expected JSON")?;
        return Ok(PollOutcome::Approved(token));
    }
    let value: Value = serde_json::from_slice(body).unwrap_or(Value::Null);
    let error = value.get("error").and_then(Value::as_str).unwrap_or("");
    match error {
        "authorization_pending" => Ok(PollOutcome::Pending),
        "slow_down" => Ok(PollOutcome::SlowDown),
        "access_denied" => Ok(PollOutcome::Denied),
        "expired_token" => Ok(PollOutcome::Expired),
        other => bail!(
            "the token endpoint returned an error this flow does not handle ({status}): {}",
            if other.is_empty() {
                String::from_utf8_lossy(body).into_owned()
            } else {
                other.to_owned()
            }
        ),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use std::io::{Read as _, Write as _};
    use std::net::TcpListener;
    use std::sync::mpsc;

    /// A one-shot mock issuer: serves each canned response in order, one per connection, on loopback.
    /// Returns the base URL. The thread ends when the responses run out.
    fn mock_issuer(responses: Vec<(u16, &'static str)>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            tx.send(()).ok();
            for (status, body) in responses {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let reason = if status == 200 { "OK" } else { "Bad Request" };
                let response = format!(
                    "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes());
            }
        });
        rx.recv().expect("server started");
        format!("http://{addr}")
    }

    fn cfg(issuer: String) -> OidcConfig {
        OidcConfig {
            issuer,
            client_id: "cli-client".to_owned(),
            project_id: Some("390606436373103770".to_owned()),
        }
    }

    #[test]
    fn the_aud_scope_names_the_project() {
        let scope = cfg("x".to_owned()).scope();
        assert!(scope.contains("openid profile"), "{scope}");
        assert!(
            scope.contains("urn:zitadel:iam:org:project:id:390606436373103770:aud"),
            "{scope}"
        );
    }

    #[test]
    fn a_missing_project_id_omits_the_aud_scope() {
        let mut c = cfg("x".to_owned());
        c.project_id = None;
        assert_eq!(c.scope(), "openid profile");
    }

    #[test]
    fn a_device_code_response_parses() {
        let base = mock_issuer(vec![(
            200,
            r#"{"device_code":"dev-123","user_code":"WXYZ-1234",
                "verification_uri":"https://id.paridyum.com/device","expires_in":900,"interval":5}"#,
        )]);
        let http = Client::new();
        let da = request_device_code(&http, &cfg(base)).expect("device code");
        assert_eq!(da.device_code, "dev-123");
        assert_eq!(da.user_code, "WXYZ-1234");
        assert_eq!(da.interval, 5);
    }

    #[test]
    fn authorization_pending_then_success() {
        let http = Client::new();

        let pending = mock_issuer(vec![(400, r#"{"error":"authorization_pending"}"#)]);
        assert_eq!(
            poll_once(&http, &cfg(pending), "dev-123").expect("poll"),
            PollOutcome::Pending
        );

        let approved = mock_issuer(vec![(
            200,
            r#"{"access_token":"tok-abc","expires_in":3600,"refresh_token":"ref-xyz"}"#,
        )]);
        let outcome = poll_once(&http, &cfg(approved), "dev-123").expect("poll");
        assert!(matches!(outcome, PollOutcome::Approved(_)), "{outcome:?}");
        if let PollOutcome::Approved(t) = outcome {
            assert_eq!(t.access_token, "tok-abc");
            assert_eq!(t.expires_in, 3600);
            assert_eq!(t.refresh_token.as_deref(), Some("ref-xyz"));
        }
    }

    #[test]
    fn the_modelled_error_codes_map() {
        assert_eq!(
            classify(false, 400, br#"{"error":"slow_down"}"#).expect("c"),
            PollOutcome::SlowDown
        );
        assert_eq!(
            classify(false, 400, br#"{"error":"access_denied"}"#).expect("c"),
            PollOutcome::Denied
        );
        assert_eq!(
            classify(false, 400, br#"{"error":"expired_token"}"#).expect("c"),
            PollOutcome::Expired
        );
        assert!(classify(false, 500, br#"{"error":"server_error"}"#).is_err());
    }
}
