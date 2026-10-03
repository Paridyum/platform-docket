//! Verifying an operator's Zitadel token (ADR-0092).
//!
//! The served console forwards a bearer token from the control-plane instance (id.paridyum.com).
//! **The verification is `paridyum-auth`** (`lib-auth-rust`) since 2026-09-24: this module was the
//! estate's second in-crate RSA/JWKS verifier and ADR-0092 §3 recorded it as debt until a third
//! consumer appeared. `platform-tenancy` was the third; ADR-0107 settled the name and the crate
//! exists, so what remains here is docket's own policy, which a shared library must not hold
//! (ANTI-0004):
//!
//! - the issuer and audience this tool pins — one plane, `id.paridyum.com`, and the
//!   `paridyum-operations` project, so a tenant-plane token is refused outright rather than
//!   downgraded (ADR-0047 D5 / ADR-0070 D4);
//! - the `operator` **role** requirement (ADR-0092 D1);
//! - deriving the tenant uuid from the `resourceowner` claim (ADR-0082 / ADR-0099).
//!
//! The subject is then bound to a docket operator by `registry::Registry::by_subject` (ADR-0092 §2),
//! on the permanent id rather than the mutable loginname so a rename cannot reattribute the ledger.

use crate::domain::tenant::derive_tenant_id;
use anyhow::{Context, Result, bail};
use jsonwebtoken::DecodingKey;
use paridyum_auth::{JwksVerifier, Pins, Plane, TokenVerifier as _};
use std::collections::BTreeMap;
use uuid::Uuid;

// Re-exported so `serve.rs` parses a fetched JWKS without naming the shared crate: the fetch stays
// here (a startup concern of the deployable, ADR-0047 D4) and the parsing is the library's.
pub use paridyum_auth::keys_from_jwks;

/// The role a token must carry to operate docket.
const OPERATOR_ROLE: &str = "operator";
const BEARER: &str = "Bearer ";

/// A verified operator token: the opaque control-plane subject it proves, having passed signature,
/// issuer, audience, expiry and the operator-role check. Bind it to a docket operator by subject.
#[derive(Debug, PartialEq, Eq)]
pub struct VerifiedOperator {
    /// The Zitadel `sub` — opaque and permanent (ADR-0092 §2).
    pub subject: String,
    /// The tenant this request acts within, derived from the token's `resourceowner` org id
    /// (ADR-0099 / ADR-0082). The RLS context for every store call in the request; one deployment
    /// serves many tenants and this is what keeps them apart.
    pub tenant: Uuid,
    /// Whether the token carried the `operator` role — read off the verified claims, not assumed.
    /// `verify` refuses a token without it today, so this is `true` for every value that exists; it
    /// is a field anyway so that what the audit row records is what was verified, and the day the
    /// role check is relaxed the value follows the claim instead of a literal elsewhere.
    pub operator_scope: bool,
    /// The plane the token was issued on. Only a control-plane operator — Paridyum's own staff — may
    /// fall back to the git registry, which is Paridyum's configuration (ADR-0099 D2, ADR-0130 D6).
    pub plane: Plane,
}

/// Validates operator tokens against the control-plane instance's published keys.
pub struct OperatorVerifier(JwksVerifier);

impl OperatorVerifier {
    /// Build a verifier from `kid`-keyed decoding keys and the pins it enforces.
    ///
    /// One plane, deliberately: a token from the tenant instance is not an operator, and pinning a
    /// single issuer is what refuses it rather than accepting it with fewer privileges.
    #[must_use]
    pub fn new(keys: BTreeMap<String, DecodingKey>, issuer: String, audience: String) -> Self {
        Self(JwksVerifier::new(
            keys,
            Pins::single_plane(Plane::Control, issuer, audience),
        ))
    }

    /// Verify a bearer token, returning the operator subject and tenant, or an error naming the
    /// first failure.
    ///
    /// # Errors
    /// Fails on a missing or unknown key id, a bad signature, the wrong issuer or audience, an
    /// expired token, a token without the `operator` role, or one carrying no `resourceowner`.
    pub fn verify(&self, token: &str) -> Result<VerifiedOperator> {
        // Lenient about the `Bearer ` prefix, as this adapter has always been. The shared crate's
        // `strip_bearer` is strict and is used by the platform services; tightening this caller
        // while moving its code would be a behaviour change smuggled in as a refactor.
        let token = token.strip_prefix(BEARER).unwrap_or(token).trim();
        let principal = self
            .0
            .verify(token)
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("the token was rejected")?;

        if !principal.has_role(OPERATOR_ROLE) {
            bail!("the token does not carry the '{OPERATOR_ROLE}' role");
        }
        let org = principal.organization_id().context(
            "the token carries no resourceowner (org id). The CLI/console must request the \
             `urn:zitadel:iam:user:resourceowner` scope so docket can derive the tenant \
             (ADR-0099); a multi-tenant request cannot be defaulted to one.",
        )?;
        let tenant = derive_tenant_id(org)?;
        Ok(VerifiedOperator {
            subject: principal.subject().to_owned(),
            tenant,
            operator_scope: principal.has_role(OPERATOR_ROLE),
            plane: Plane::Control,
        })
    }
}

#[cfg(test)]
// A test that cannot assert an expected value is not a test; the crate's expect/unwrap denial is
// exempted inside the test module, as elsewhere.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use jsonwebtoken::{EncodingKey, Header, encode};
    use rsa::RsaPrivateKey;
    use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
    use serde_json::{Value, json};

    const ISS: &str = "https://id.paridyum.com";
    const AUD: &str = "390606436373103770"; // the paridyum-operations project (control plane)

    // A throwaway keypair generated per test run — nothing embedded, so no secret to scan for.
    fn keypair() -> (EncodingKey, OperatorVerifier) {
        let key = RsaPrivateKey::new(&mut rand::thread_rng(), 2048).expect("generate key");
        let priv_pem = key.to_pkcs8_pem(LineEnding::LF).expect("private pem");
        let pub_pem = key
            .to_public_key()
            .to_public_key_pem(LineEnding::LF)
            .expect("public pem");
        let enc = EncodingKey::from_rsa_pem(priv_pem.as_bytes()).expect("encoding key");
        let mut keys = BTreeMap::new();
        keys.insert(
            "k1".to_owned(),
            DecodingKey::from_rsa_pem(pub_pem.as_bytes()).expect("decoding key"),
        );
        (
            enc,
            OperatorVerifier::new(keys, ISS.to_owned(), AUD.to_owned()),
        )
    }

    fn sign(enc: &EncodingKey, claims: &Value) -> String {
        let mut header = Header::new(jsonwebtoken::Algorithm::RS256);
        header.kid = Some("k1".to_owned());
        encode(&header, claims, enc).expect("sign")
    }

    fn operator_claims() -> Value {
        json!({
            "iss": ISS, "sub": "390506086174098663", "aud": AUD, "exp": 9_999_999_999u64,
            "urn:zitadel:iam:org:project:390606436373103770:roles": { "operator": {} },
            // Paridyum's org id; derives the tenant 68445ee9-… (ADR-0082).
            "urn:zitadel:iam:user:resourceowner:id": "389623583590057140",
        })
    }

    #[test]
    fn a_valid_operator_token_yields_its_subject_and_tenant() {
        let (enc, v) = keypair();
        let out = v.verify(&sign(&enc, &operator_claims())).expect("verifies");
        assert_eq!(out.subject, "390506086174098663");
        assert_eq!(
            out.plane,
            Plane::Control,
            "this verifier pins the control plane"
        );
        assert_eq!(
            out.tenant,
            Uuid::parse_str("68445ee9-175a-58a9-834e-a2f0c9c9b124").unwrap()
        );
    }

    #[test]
    fn a_token_without_a_resourceowner_is_refused() {
        let (enc, v) = keypair();
        let mut claims = operator_claims();
        claims
            .as_object_mut()
            .unwrap()
            .remove("urn:zitadel:iam:user:resourceowner:id");
        assert!(v.verify(&sign(&enc, &claims)).is_err());
    }

    #[test]
    fn a_bearer_prefix_is_accepted() {
        let (enc, v) = keypair();
        let token = format!("Bearer {}", sign(&enc, &operator_claims()));
        assert!(
            v.verify(&token).is_ok(),
            "this adapter has always tolerated the prefix, and still must"
        );
    }

    #[test]
    fn a_token_without_the_operator_role_is_refused() {
        let (enc, v) = keypair();
        let mut claims = operator_claims();
        claims["urn:zitadel:iam:org:project:390606436373103770:roles"] = json!({ "viewer": {} });
        assert!(v.verify(&sign(&enc, &claims)).is_err());
    }

    // The next three assert THIS TOOL'S PINS rather than the library's logic: that docket pins one
    // plane and the operations project, and therefore refuses what it should. The library tests
    // cover the verification itself.
    #[test]
    fn the_wrong_issuer_is_refused() {
        let (enc, v) = keypair();
        let mut claims = operator_claims();
        claims["iss"] = json!("https://auth.paridyum.com");
        assert!(
            v.verify(&sign(&enc, &claims)).is_err(),
            "a tenant-plane token is not an operator"
        );
    }

    #[test]
    fn the_wrong_audience_is_refused() {
        let (enc, v) = keypair();
        let mut claims = operator_claims();
        claims["aud"] = json!("some-other-project");
        assert!(
            v.verify(&sign(&enc, &claims)).is_err(),
            "a token minted for another app is refused (ADR-0047 D5)"
        );
    }

    #[test]
    fn an_expired_token_is_refused() {
        let (enc, v) = keypair();
        let mut claims = operator_claims();
        claims["exp"] = json!(1_000u64);
        assert!(v.verify(&sign(&enc, &claims)).is_err());
    }
}
