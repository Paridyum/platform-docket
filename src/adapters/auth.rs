//! Verifying an operator's Zitadel token (ADR-0092).
//!
//! The served console forwards a bearer token from the control-plane instance (id.paridyum.com).
//! This validates it against that instance's JWKS, pins the issuer and the `paridyum-operations`
//! audience — a correctly-signed token minted for another application is otherwise accepted
//! (ADR-0047 D5 / ADR-0070 D4) — requires the `operator` role, and returns the opaque subject. The
//! subject is then bound to a docket operator by `registry::Registry::by_subject` (ADR-0092 §2), on
//! the permanent id rather than the mutable loginname so a rename cannot reattribute the ledger.
//!
//! This is the estate's second in-crate RSA/JWKS verifier (platform-identity has the first);
//! ADR-0092 §3 records that a shared Rust `lib-auth` waits for a third consumer rather than being
//! extracted speculatively.

use crate::domain::tenant::derive_tenant_id;
use anyhow::{Context, Result, bail};
use jsonwebtoken::{DecodingKey, Validation, decode, decode_header};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

/// The Zitadel claim carrying the signing-in user's organization id — present when the
/// `urn:zitadel:iam:user:resourceowner` scope is requested (oidc.rs / web-admin do). ADR-0099
/// derives the tenant uuid from it (ADR-0082), which is the RLS context for the request.
const RESOURCEOWNER_ID_CLAIM: &str = "urn:zitadel:iam:user:resourceowner:id";

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
}

/// Validates operator tokens against the control-plane instance's published keys.
pub struct OperatorVerifier {
    keys: BTreeMap<String, DecodingKey>,
    issuer: String,
    audience: String,
}

impl OperatorVerifier {
    /// Build a verifier from `kid`-keyed decoding keys and the pins it enforces.
    #[must_use]
    pub fn new(keys: BTreeMap<String, DecodingKey>, issuer: String, audience: String) -> Self {
        Self {
            keys,
            issuer,
            audience,
        }
    }

    /// Verify a bearer token, returning the operator subject or an error naming the first failure.
    ///
    /// # Errors
    /// Fails on a missing or unknown key id, a bad signature, the wrong issuer or audience, an
    /// expired token, a token without the `operator` role, or one carrying no subject.
    pub fn verify(&self, token: &str) -> Result<VerifiedOperator> {
        let token = token.strip_prefix(BEARER).unwrap_or(token).trim();
        let header = decode_header(token).context("the token header is undecodable")?;
        let kid = header.kid.context("the token header carries no key id")?;
        let key = self
            .keys
            .get(&kid)
            .context("no published key matches the token's key id")?;

        let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.set_issuer(&[self.issuer.as_str()]);
        validation.set_audience(&[self.audience.as_str()]);
        let data = decode::<Value>(token, key, &validation).context("the token was rejected")?;

        if !roles_from_claims(&data.claims).contains(OPERATOR_ROLE) {
            bail!("the token does not carry the '{OPERATOR_ROLE}' role");
        }
        let subject = data
            .claims
            .get("sub")
            .and_then(Value::as_str)
            .context("the token carries no subject")?
            .to_owned();
        let org = data
            .claims
            .get(RESOURCEOWNER_ID_CLAIM)
            .and_then(Value::as_str)
            .context(
                "the token carries no resourceowner (org id). The CLI/console must request the \
                 `urn:zitadel:iam:user:resourceowner` scope so docket can derive the tenant \
                 (ADR-0099); a multi-tenant request cannot be defaulted to one.",
            )?;
        let tenant = derive_tenant_id(org)?;
        Ok(VerifiedOperator { subject, tenant })
    }
}

/// Pull the roles out of Zitadel's `urn:zitadel:iam:org:project:{projectId}:roles` claim, whose key
/// carries the project id and whose value is an object keyed by role name.
fn roles_from_claims(claims: &Value) -> BTreeSet<String> {
    let Some(map) = claims.as_object() else {
        return BTreeSet::new();
    };
    for (key, value) in map {
        if !(key.starts_with("urn:zitadel:iam:org:project:") && key.ends_with(":roles")) {
            continue;
        }
        if let Some(roles) = value.as_object() {
            return roles.keys().cloned().collect();
        }
    }
    BTreeSet::new()
}

/// Parse a JWKS document into `kid`-keyed decoding keys. Extracted from the binary's startup fetch
/// so the parsing is unit-testable and stays off the request path (ADR-0047 D4).
#[must_use]
pub fn keys_from_jwks(jwks: &Value) -> BTreeMap<String, DecodingKey> {
    let mut out = BTreeMap::new();
    let Some(keys) = jwks.get("keys").and_then(Value::as_array) else {
        return out;
    };
    for key in keys {
        let (Some(kid), Some(n), Some(e)) = (
            key.get("kid").and_then(Value::as_str),
            key.get("n").and_then(Value::as_str),
            key.get("e").and_then(Value::as_str),
        ) else {
            continue;
        };
        if let Ok(dk) = DecodingKey::from_rsa_components(n, e) {
            out.insert(kid.to_owned(), dk);
        }
    }
    out
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
    use serde_json::json;

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
        assert!(v.verify(&token).is_ok());
    }

    #[test]
    fn a_token_without_the_operator_role_is_refused() {
        let (enc, v) = keypair();
        let mut claims = operator_claims();
        claims["urn:zitadel:iam:org:project:390606436373103770:roles"] = json!({ "viewer": {} });
        assert!(v.verify(&sign(&enc, &claims)).is_err());
    }

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

    #[test]
    fn an_unknown_key_id_is_refused() {
        let (enc, v) = keypair();
        let mut header = Header::new(jsonwebtoken::Algorithm::RS256);
        header.kid = Some("unknown".to_owned());
        let token = encode(&header, &operator_claims(), &enc).expect("sign");
        assert!(v.verify(&token).is_err());
    }
}
