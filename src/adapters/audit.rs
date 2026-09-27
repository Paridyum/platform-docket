//! What docket records about who touched a tenant's records. ADR-0044, ADR-0110.
//!
//! The four sealed chains say what a document **is**. They do not say who **looked**. A read of a
//! person record leaves no trace in them, and a refused write leaves less — which is the half of
//! ADR-0044 decision 6 the chains were never going to cover:
//!
//!   "What is recorded, at minimum: who (subject, tenant, account), what (action, resource), when,
//!    where from, the correlation id, and the outcome — **including denials.** A trail recording only
//!    successful access answers 'what happened' and not 'what was attempted', and the second is the
//!    security question."
//!
//! ## One event per operation, which is ADR-0110 and is why this is not in the store
//!
//! ADR-0110 decision 1: **one audit event per operation a caller requested.** A served request is one
//! operation; a CLI invocation is one operation; the queries each causes are not. `docket check` opens
//! several transactions and reads four chains, a person store and a registry — audited per store call
//! that is tens of rows for one command, and nobody asked for a `SELECT` on `docket_signatures`.
//!
//! So the writer is called from the **dispatcher**, where the operation is known, and not from
//! `PgStore`, which only ever sees queries. That also keeps the `Store` trait — which `FileStore`
//! satisfies too — free of a parameter it has no use for.
//!
//! ## ⛔ FILE MODE IS NOT AUDITED, and it cannot be
//!
//! `audit_events` lives in the tenancy database. `Backend::Fixed` — the CLI's local store and the
//! loopback break-glass path — has no database at all, and no tenant in the sense the column means.
//! So a `docket issue` run against local files writes no audit row, and there is nowhere for it to go
//! that a tenant could read.
//!
//! This is stated rather than papered over with a local log file, which would be a second audit store
//! the tenant cannot see — and an audit trail only the auditee can read is the thing D5 exists to
//! replace. Recorded in `DECISION-DEBT.md`; the closing condition is that file mode stops being a path
//! to a tenant's records, not that it grows a log.

use anyhow::Result;
use uuid::Uuid;

use crate::adapters::correlation::CorrelationId;

/// Whether the operation was permitted, refused, or broke.
///
/// Matches `audit_events_outcome_is_known` in tenancy migration 0010. **`Denied` and `Failed` are
/// distinct on purpose:** denied is the system working — a boundary held — and failed is the system
/// not working. Collapsing them puts every outage into the security review.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The operation was performed.
    Allowed,
    /// A boundary refused it. **The row ADR-0044 D6 exists for.**
    Denied,
    /// It could not be completed for a reason the caller did not cause.
    Failed,
}

impl Outcome {
    /// The value the column takes.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allowed => "allowed",
            Self::Denied => "denied",
            Self::Failed => "failed",
        }
    }

    /// The outcome an HTTP status implies.
    ///
    /// **401 and 403 are the only denials.** A 400, a 404 or a 409 means the operation was permitted
    /// and the request was wrong — recording those as denials would fill a security review with
    /// typos, and the question "what was refused" would stop being answerable. A 5xx is `Failed`,
    /// because the caller did not cause it.
    ///
    /// A 404 is `Allowed` even though nothing was returned, and that reads oddly until you ask what
    /// the row is for: the operation *was* served, and "this tenant asked for a person who does not
    /// exist" is a fact worth having rather than a refusal.
    #[must_use]
    pub const fn of_status(status: u16) -> Self {
        match status {
            401 | 403 => Self::Denied,
            500..=599 => Self::Failed,
            _ => Self::Allowed,
        }
    }
}

/// One thing a caller asked docket to do.
///
/// `action` and `object` are owned strings rather than `&'static str` — unlike `platform-tenancy`'s,
/// because docket's operation name is derived from the request path at runtime. [`operation_of`] is
/// the only thing that builds them, and it maps to a **closed set** rather than interpolating the
/// path, so a caller cannot write its own vocabulary into the column (ADR-0110 accepted cost 2).
#[derive(Debug, Clone)]
pub struct Event {
    /// What was done, in docket's own vocabulary — `document.issue`, `person.read`.
    pub action: String,
    /// What it was done to — the noun the caller would use.
    pub object: String,
    /// Whether it was allowed, refused, or broke.
    pub outcome: Outcome,
    /// The Zitadel subject that acted. Never a display name.
    pub actor: String,
    /// Where from, if the request said. Never fabricated.
    pub source: Option<String>,
    /// The client, if the request said.
    pub client: Option<String>,
}

/// The operation a method and path describe, as `(action, object)`.
///
/// **A closed vocabulary, not the path interpolated into a string.** ADR-0110 accepted cost 2 makes an
/// operation name a contract — something a query against the trail is written against — and a path
/// echoed into the column would make every new route a silent addition to that contract, and every
/// path segment a caller supplies part of it.
///
/// An unrecognised route is `request.unknown`, which is honest: a 404 for a path this server does not
/// serve is still an attempt worth recording, and inventing a name for it from the path is how the
/// column acquires caller-controlled text.
#[must_use]
pub fn operation_of(method: &str, segments: &[&str]) -> (String, String) {
    // `api` is stripped: every route carries it and a prefix present on every row is noise.
    let path: Vec<&str> = segments
        .iter()
        .copied()
        .skip_while(|s| *s == "api")
        .collect();
    let (action, object) = match (method, path.as_slice()) {
        ("GET", ["health"]) => ("health.read", "health"),
        ("GET", ["persons"]) => ("person.list", "persons"),
        ("GET", ["persons", _]) => ("person.read", "person"),
        ("POST", ["persons"]) => ("person.create", "person"),
        ("PUT", ["persons", _]) => ("person.rewrite", "person"),
        ("GET", ["register"]) => ("register.read", "register"),
        ("GET", ["identities"]) => ("identity.read", "identities"),
        ("GET", ["events"]) => ("event.read", "events"),
        ("GET", ["signatures"]) => ("signature.read", "signatures"),
        ("POST", ["issue", ..]) => ("document.issue", "document"),
        ("POST", ["sign", ..]) => ("document.sign", "signature"),
        _ => ("request.unknown", "request"),
    };
    (action.to_owned(), object.to_owned())
}

/// The SQL one audit row takes, with its parameters bound by the caller.
///
/// Kept here rather than in `pg.rs` so the column list and `Event` cannot drift apart, and so the one
/// place that knows the audit schema is the one place named for it.
pub(crate) const INSERT: &str = "INSERT INTO audit_events \
     (tenant_id, correlation_id, actor, action, object, outcome, \
      source, client, operator_scope, actor_region) \
   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)";

/// Everything the row needs that is not the event itself.
///
/// `account_id` is deliberately **absent**, and that is not an omission. ADR-0110 decision 3 wants one
/// event per silo an operation touches — and docket's served requests are dispatched before an account
/// is resolved (`enter()` resolves it inside the transaction, per instance). A row claiming an account
/// the dispatcher had not yet looked up would be a guess about which silo was touched. `NULL` means
/// tenant-wide, which is what a request that has not yet reached an account honestly is.
///
/// Closing that is D-73: the operation would have to carry the account back out of the store.
#[derive(Debug, Clone, Copy)]
pub struct Context<'a> {
    /// The tenant whose records the operation touched.
    pub tenant: Uuid,
    /// The request's correlation id (ADR-0044 D1) — read from the edge, never minted here.
    pub correlation: &'a CorrelationId,
    /// The region this instance serves from (RFC-0014).
    pub region: &'a str,
    /// Whether the caller held operator scope, recorded on the row as a fact about this request.
    ///
    /// Carried here, from the dispatcher, rather than written as a literal in the sink: the sink
    /// cannot know it, and 0010 records the column "never recomputed", so a constant that became
    /// false one day would falsify history nobody could correct. The dispatcher knows it because it
    /// holds a `VerifiedOperator`, which `OperatorVerifier::verify` produces only after the
    /// `operator` role check passes.
    pub operator_scope: bool,
}

/// Where an audit row goes, so the dispatcher can write one without holding a pool.
///
/// A trait because `Backend::Fixed` has no database: the served backend implements it over the tenancy
/// database, and the fixed one does not implement it at all — which is why the dispatcher's call site
/// is `Option`-shaped rather than a no-op sink. **A no-op sink that silently discarded audit rows is
/// the failure this trait exists to make impossible to write by accident.**
pub trait Sink: Sync {
    /// Append one event.
    ///
    /// # Errors
    /// Fails if the row cannot be written. The caller decides what that means — see the dispatcher,
    /// which logs and serves rather than converting a served response into a 500.
    fn record(&self, ctx: Context<'_>, event: &Event) -> Result<()>;
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn only_401_and_403_are_denials() {
        assert_eq!(Outcome::of_status(401), Outcome::Denied);
        assert_eq!(Outcome::of_status(403), Outcome::Denied);
        // A typo is not a refusal. If these were denials, "what was refused" would stop being a
        // question the trail can answer.
        assert_eq!(Outcome::of_status(400), Outcome::Allowed);
        assert_eq!(Outcome::of_status(404), Outcome::Allowed);
        assert_eq!(Outcome::of_status(409), Outcome::Allowed);
    }

    #[test]
    fn a_server_fault_is_failed_not_denied() {
        assert_eq!(Outcome::of_status(500), Outcome::Failed);
        assert_eq!(Outcome::of_status(503), Outcome::Failed);
    }

    #[test]
    fn success_is_allowed() {
        assert_eq!(Outcome::of_status(200), Outcome::Allowed);
        assert_eq!(Outcome::of_status(201), Outcome::Allowed);
    }

    #[test]
    fn the_outcome_strings_match_the_check_constraint() {
        // `audit_events_outcome_is_known` permits exactly these three. The enum and the constraint are
        // in different languages and nothing else compares them.
        assert_eq!(Outcome::Allowed.as_str(), "allowed");
        assert_eq!(Outcome::Denied.as_str(), "denied");
        assert_eq!(Outcome::Failed.as_str(), "failed");
    }

    #[test]
    fn the_api_prefix_is_stripped_and_routes_map_to_a_closed_vocabulary() {
        assert_eq!(
            operation_of("GET", &["api", "persons", "asha-verma"]),
            ("person.read".to_owned(), "person".to_owned())
        );
        assert_eq!(
            operation_of("GET", &["persons"]),
            ("person.list".to_owned(), "persons".to_owned())
        );
        assert_eq!(
            operation_of("POST", &["api", "issue", "CRT", "asha-verma"]),
            ("document.issue".to_owned(), "document".to_owned())
        );
    }

    #[test]
    fn an_unknown_route_does_not_put_the_path_in_the_column() {
        // The path is caller-controlled text and the column is read during an incident. An attempt on
        // an unserved route is recorded as an attempt, not as whatever the caller typed.
        let (action, object) = operation_of("GET", &["api", "../../etc/passwd", "%00"]);
        assert_eq!(action, "request.unknown");
        assert_eq!(object, "request");
        assert!(!action.contains("passwd") && !object.contains("passwd"));
    }

    #[test]
    fn a_known_path_with_an_unknown_method_is_unknown_rather_than_guessed() {
        assert_eq!(
            operation_of("DELETE", &["api", "persons", "asha-verma"]),
            ("request.unknown".to_owned(), "request".to_owned())
        );
    }
}
