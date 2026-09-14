//! Who is making this request, and whether they may.

use crate::adapters::auth::OperatorVerifier;
use crate::adapters::http::request::Request;
use crate::adapters::http::router::{NOT_AUTHENTICATED, NOT_PERMITTED};
use crate::adapters::registry::Registry;
use anyhow::{Context, Result, anyhow};

/// Header naming the operator. Also the CSRF defence — see [`resolve`].
pub const OPERATOR_HEADER: &str = "x-docket-operator";

/// The bearer-token header the served backend authenticates from (ADR-0092).
pub const AUTHORIZATION_HEADER: &str = "authorization";

/// Resolve the operator, choosing the auth mechanism by whether a verifier is present.
///
/// This is the §5 invariant made mechanical: a served backend is configured with a token verifier
/// and authenticates every write from a control-plane Bearer token (ADR-0092); a loopback backend
/// (the CLI's `docket serve`, break-glass, SSH-fronted) has no verifier and falls back to the
/// attribution header. The two never mix — where a verifier is present the header is not even read,
/// so a served route cannot degrade to an unverified claim (design §5).
///
/// # Errors
/// Token path: missing or unverifiable token → 401; a verified subject bound to no operator → 403.
/// Header path: a missing, blank, or unknown operator → 403.
pub fn resolve<'a>(
    registry: &'a Registry,
    verifier: Option<&OperatorVerifier>,
    req: &Request,
) -> Result<&'a str> {
    match verifier {
        Some(verifier) => resolve_token(registry, verifier, req),
        None => resolve_header(registry, req),
    }
}

/// The served path: a verified control-plane token bound to a docket operator (ADR-0092).
fn resolve_token<'a>(
    registry: &'a Registry,
    verifier: &OperatorVerifier,
    req: &Request,
) -> Result<&'a str> {
    let Some(header) = req.header(AUTHORIZATION_HEADER) else {
        return Err(anyhow!(NOT_AUTHENTICATED).context(
            "this write carries no bearer token. The served backend authenticates every write \
             from a control-plane operator token (ADR-0092); send it as `Authorization: Bearer …`.",
        ));
    };
    // A bad signature, wrong issuer/audience, missing role or expiry is a 401: the caller must
    // present a valid token, which is a different remedy from being a known-but-unbound subject.
    let proven = verifier
        .verify(header)
        .map_err(|e| anyhow!(NOT_AUTHENTICATED).context(format!("{e}")))
        .context("the bearer token was not accepted")?;
    // A correctly-signed token whose subject is bound to no active operator is a 403: authenticated,
    // but not permitted — the subject must be bound in registry/operators.yaml (ADR-0092 §2).
    let operator = registry
        .by_subject(&proven.subject)
        .map_err(|e| anyhow!(NOT_PERMITTED).context(format!("{e}")))
        .context("the verified subject is bound to no operator")?;
    Ok(&operator.id)
}

/// Resolve the operator for a write, or refuse.
///
/// # What this is, and what it is not
///
/// This is **attribution**, not authentication. It is exactly as strong as the
/// CLI's `--by`: a claimed identifier, checked against `registry/operators.yaml`
/// so it must name a real operator, but not proven. What proves it is SSH — the
/// server binds loopback only, so reaching this at all means holding a key to the
/// machine (see `server::require_loopback`).
///
/// Anyone who can reach the socket can therefore claim to be any operator in the
/// registry. That is not a regression: it is the CLI's position, where anyone with
/// a shell can pass any `--by`. Per-operator authorisation waits for
/// `platform-identity` (ADR-0047), and is recorded as D-38.
///
/// It is refused rather than defaulted, because a default is exactly what made
/// every event in the log say "docket".
///
/// # Why this doubles as CSRF protection
///
/// A cross-site HTML form cannot set a custom request header. Requiring one on
/// every write means a page in the operator's browser — reachable by script from
/// any other page in that browser, which loopback and SSH do not prevent — cannot
/// forge a write. The header has to be present, so a form post can never satisfy
/// it. That replaces the CSRF token the original design specified, which assumed
/// forms this process would serve itself.
///
/// # Errors
/// Fails if the header is absent, blank, or names an operator the registry does
/// not declare.
fn resolve_header<'a>(registry: &'a Registry, req: &Request) -> Result<&'a str> {
    let Some(claimed) = req.header(OPERATOR_HEADER) else {
        return Err(anyhow!(NOT_PERMITTED).context(format!(
            "this write has no operator. Send {OPERATOR_HEADER} naming someone in \
             registry/operators.yaml: a write that cannot say who made it is not auditable, \
             and every entry in the ledger records its author."
        )));
    };
    let claimed = claimed.trim();
    if claimed.is_empty() {
        // An EMPTY header is unset, not an operator called "". The CLI learned
        // the same lesson about a blank environment variable.
        return Err(anyhow!(NOT_PERMITTED).context(format!(
            "{OPERATOR_HEADER} is blank, which is not an operator"
        )));
    }
    let operator = registry
        .operator(claimed)
        .map_err(|e| anyhow!(NOT_PERMITTED).context(format!("{e}")))
        .context("the operator named is not one this tool accepts")?;
    Ok(&operator.id)
}

#[cfg(test)]
// The lint configuration denies expect/unwrap for the crate, because a panic in a
// running tool is a defect. In a test an assertion IS the point.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use crate::adapters::http::request;
    use crate::config::Paths;
    use std::path::Path;

    fn registry() -> Registry {
        Registry::load(&Paths::resolve(Path::new(env!("CARGO_MANIFEST_DIR"))))
            .expect("registry loads")
    }

    fn req(raw: &str) -> Request {
        request::parse(raw.as_bytes()).expect("parses")
    }

    #[test]
    fn a_declared_operator_resolves() {
        let reg = registry();
        // Taken from the registry rather than hardcoded, so this test does not
        // fail the day an operator leaves.
        let known = reg
            .operators
            .iter()
            .find(|o| o.active)
            .map(|o| o.id.clone())
            .expect("at least one active operator is declared");
        let r = req(&format!(
            "POST /api/x HTTP/1.1\r\nX-Docket-Operator: {known}\r\n\r\n"
        ));
        // No verifier: the loopback/break-glass path authenticates from the attribution header.
        assert_eq!(resolve(&reg, None, &r).expect("resolves"), known);
    }

    #[test]
    fn a_write_with_no_operator_header_is_refused() {
        let err =
            resolve(&registry(), None, &req("POST /api/x HTTP/1.1\r\n\r\n")).expect_err("refused");
        let text = format!("{err}");
        assert!(text.contains("no operator"), "{text}");
        // The message must say what to do, not just that it failed.
        assert!(text.contains("operators.yaml"), "{text}");
    }

    #[test]
    fn a_blank_operator_header_is_not_an_operator() {
        let err = resolve(
            &registry(),
            None,
            &req("POST /api/x HTTP/1.1\r\nX-Docket-Operator:   \r\n\r\n"),
        )
        .expect_err("refused");
        assert!(format!("{err}").contains("blank"));
    }

    #[test]
    fn an_operator_the_registry_does_not_declare_is_refused() {
        assert!(
            resolve(
                &registry(),
                None,
                &req("POST /api/x HTTP/1.1\r\nX-Docket-Operator: nobody\r\n\r\n")
            )
            .is_err()
        );
    }

    #[test]
    fn with_a_verifier_the_header_no_longer_authenticates() {
        // The served path does not read the attribution header at all: a request that carries only
        // X-Docket-Operator and no token is unauthenticated (401), never a claim (design §5).
        let verifier = OperatorVerifier::new(
            std::collections::BTreeMap::new(),
            "https://id.paridyum.com".to_owned(),
            "operations".to_owned(),
        );
        let err = resolve(
            &registry(),
            Some(&verifier),
            &req("POST /api/x HTTP/1.1\r\nX-Docket-Operator: rohan\r\n\r\n"),
        )
        .expect_err("no token, so refused");
        // The 401 marker rides in the error chain (where `status_for` reads it), not the top display.
        assert!(
            err.chain().any(|c| c.to_string() == NOT_AUTHENTICATED),
            "missing token is a 401"
        );
    }

    #[test]
    fn with_a_verifier_a_garbage_token_is_401() {
        let verifier = OperatorVerifier::new(
            std::collections::BTreeMap::new(),
            "https://id.paridyum.com".to_owned(),
            "operations".to_owned(),
        );
        let err = resolve(
            &registry(),
            Some(&verifier),
            &req("POST /api/x HTTP/1.1\r\nAuthorization: Bearer not-a-token\r\n\r\n"),
        )
        .expect_err("unverifiable, so refused");
        assert!(
            err.chain().any(|c| c.to_string() == NOT_AUTHENTICATED),
            "{err:#}"
        );
    }

    #[test]
    fn the_header_name_is_matched_case_insensitively() {
        let reg = registry();
        let known = reg
            .operators
            .iter()
            .find(|o| o.active)
            .map(|o| o.id.clone())
            .expect("an active operator");
        // Clients differ on casing, and a write refused over capitalisation would
        // read as an authorisation failure.
        let r = req(&format!(
            "POST /api/x HTTP/1.1\r\nx-DOCKET-operator: {known}\r\n\r\n"
        ));
        assert!(resolve(&reg, None, &r).is_ok());
    }
}
