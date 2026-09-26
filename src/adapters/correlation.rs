//! The request correlation id: read from the edge, never invented per hop.
//!
//! ADR-0041 D3 requires a shared correlation id **from the first request**, because the audit trail
//! spans two systems that will never share a database — Zitadel's event log and ours — and
//! "retrofitting one across two systems means reconstructing history that was never written." Every
//! record docket has written so far has none, and those are not recoverable.
//!
//! ADR-0044 D1 fixes the wire format: **a W3C `traceparent`, or `X-Correlation-Id` where a client
//! cannot emit one**, generated once at the outermost boundary and *propagated unchanged*.
//!
//! ## This is the third implementation of one contract, and that is a cost, not a convenience
//!
//! `platform-tenancy/src/correlation.rs` and `@paridyum/auth`'s `correlation.ts` hold the same
//! rules. The duplication is deliberate and is named here rather than left to be discovered, because
//! the alternative today is worse:
//!
//! * `paridyum-auth` (`lib-auth-rust`) is the estate's shared **auth** crate. A correlation id is
//!   observability, not authorization, and putting it there because a crate happens to exist is the
//!   ANTI-0004 shape pointed the other way — a library acquiring a concern it has no business owning.
//! * A `lib-observability-rust` for one module is a repository, a version, a pin and a release path
//!   for eighty lines.
//!
//! **The trigger, stated so it fires rather than being remembered:** a *fourth* Rust implementation,
//! or any change to the parsing rules that has to be made in more than one place. Either one turns
//! this from a priced duplication into drift, and the answer then is a shared crate — the same shape
//! ADR-0092 D3 used for the JWKS verifier, which ADR-0107 eventually extracted.
//!
//! Recorded in `DECISION-DEBT.md`. The behaviour here is intentionally identical to the other two,
//! including which values are refused.
//!
//! ## Where the edge is for this tool
//!
//! Two answers, and both are correct:
//!
//! * **Served (ADR-0099).** The HTTP dispatcher reads the request's headers. A client's
//!   `traceparent` is the one that must survive.
//! * **CLI.** There is no inbound request: `docket issue` *is* the edge, so the id is minted once per
//!   invocation and every transaction in it shares one. Minting per *transaction* would be the
//!   per-hop minting D1 forbids, in the place it would be least visible — a single command producing
//!   four unrelated ids.

use uuid::Uuid;

/// Where the id came from. Carried so minting is observable rather than silent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// A W3C `traceparent`: the id is its `trace-id` field, unchanged.
    Traceparent,
    /// An `X-Correlation-Id` that matched the accepted shape, unchanged.
    Header,
    /// Nothing usable arrived, so this process was the edge and minted one.
    Minted,
}

impl Source {
    /// A short, stable token for a log line.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Traceparent => "traceparent",
            Self::Header => "x-correlation-id",
            Self::Minted => "minted",
        }
    }
}

/// A request's correlation id, and how it was obtained.
///
/// No constructor from a bare `String`: one is built by reading headers or by minting, so no call
/// site can put an arbitrary value into a record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorrelationId {
    value: String,
    source: Source,
}

impl CorrelationId {
    /// Resolve the id from a request's headers, minting one only if nothing usable arrived.
    ///
    /// `lookup` is a closure over whatever the caller's request type is — this tool's HTTP layer is
    /// its own (`adapters::http`), not `http::HeaderMap`, so a concrete header type here would tie
    /// this module to it.
    ///
    /// `traceparent` first, per ADR-0044 D1: a client that emits one is already in a trace this id
    /// must join rather than replace.
    pub fn from_headers<'a>(lookup: impl Fn(&str) -> Option<&'a str>) -> Self {
        if let Some(trace_id) = lookup("traceparent").and_then(trace_id_of) {
            return Self {
                value: trace_id,
                source: Source::Traceparent,
            };
        }
        if let Some(given) = lookup("x-correlation-id").filter(|v| is_acceptable(v)) {
            return Self {
                value: given.to_owned(),
                source: Source::Header,
            };
        }
        Self::mint()
    }

    /// Mint an id — for the CLI, which is its own edge, and as the fallback for a request that
    /// arrived without one.
    ///
    /// uuidv7 because ADR-0078 makes time-ordered ids the estate's default, and an audit stream is
    /// read in time order.
    #[must_use]
    pub fn mint() -> Self {
        Self {
            value: Uuid::now_v7().to_string(),
            source: Source::Minted,
        }
    }

    /// The id, as it is written to the session and to any audit record.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Where it came from.
    #[must_use]
    pub const fn source(&self) -> Source {
        self.source
    }
}

/// The `trace-id` of a W3C `traceparent`, or `None` if the header is not one.
///
/// `version-traceid-parentid-flags`. **Only the trace id is taken.** The parent id identifies this
/// hop's caller and changes per hop, so using it as the correlation id is exactly the mistake
/// ADR-0044 D1 describes, with the appearance of having propagated something.
///
/// Each refusal is the W3C spec's: fewer than four fields; version `ff`, reserved and invalid; a
/// trace id that is not 32 **lowercase** hex characters, because a case-insensitive read would make
/// two spellings of one trace two ids here; and an all-zero trace id, which is explicitly invalid and
/// is what a broken instrumentation library emits — accepting it would collapse every such request
/// onto a single id.
fn trace_id_of(header: &str) -> Option<String> {
    let mut fields = header.trim().split('-');
    let version = fields.next()?;
    let trace_id = fields.next()?;
    let _parent_id = fields.next()?;
    let _flags = fields.next()?;

    if version.len() != 2 || !version.bytes().all(is_lower_hex) || version == "ff" {
        return None;
    }
    if trace_id.len() != 32 || !trace_id.bytes().all(is_lower_hex) {
        return None;
    }
    if trace_id.bytes().all(|b| b == b'0') {
        return None;
    }
    Some(trace_id.to_owned())
}

/// Whether an `X-Correlation-Id` may be stored and logged as given.
///
/// 8–128 characters of ASCII alphanumerics, `-`, `_` or `.`: a uuid, a ULID, a hex trace id and most
/// proxy request ids pass; whitespace, control characters, newlines (log injection) and anything long
/// enough to be a payload do not. The value reaches `audit_events.correlation_id` and every log line
/// (ADR-0044 D2), so it is caller-controlled text in two places that are read during an incident.
///
/// A value outside the shape is **discarded, never truncated**: a truncated id is a *wrong* id, which
/// is worse than a fresh one, because it joins to nothing while looking like it should.
fn is_acceptable(value: &str) -> bool {
    (8..=128).contains(&value.len())
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.')
}

const fn is_lower_hex(b: u8) -> bool {
    b.is_ascii_digit() || b.is_ascii_lowercase() && b <= b'f'
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRACE: &str = "4bf92f3577b34da6a3ce929d0e0e4736";
    const PARENT: &str = "00f067aa0ba902b7";

    fn only(name: &'static str, value: &'static str) -> impl Fn(&str) -> Option<&'static str> {
        move |n: &str| if n == name { Some(value) } else { None }
    }

    #[test]
    fn a_traceparent_contributes_its_trace_id_unchanged() {
        let id = CorrelationId::from_headers(only(
            "traceparent",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
        ));
        assert_eq!(id.as_str(), TRACE);
        assert_eq!(id.source(), Source::Traceparent);
    }

    #[test]
    fn the_parent_id_is_never_the_correlation_id() {
        let id = CorrelationId::from_headers(only(
            "traceparent",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
        ));
        assert!(!id.as_str().contains(PARENT));
    }

    #[test]
    fn an_unknown_later_version_is_still_read() {
        let id = CorrelationId::from_headers(only(
            "traceparent",
            "01-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01-extra",
        ));
        assert_eq!(id.source(), Source::Traceparent);
    }

    #[test]
    fn an_invalid_traceparent_is_refused_and_one_is_minted() {
        for bad in [
            "",
            "00",
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7",
            "ff-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
            "00-00000000000000000000000000000000-00f067aa0ba902b7-01",
            "00-4BF92F3577B34DA6A3CE929D0E0E4736-00f067aa0ba902b7-01",
            "00-abc-00f067aa0ba902b7-01",
        ] {
            let id = CorrelationId::from_headers(
                |n: &str| {
                    if n == "traceparent" { Some(bad) } else { None }
                },
            );
            assert_eq!(id.source(), Source::Minted, "accepted {bad:?}");
        }
    }

    #[test]
    fn an_acceptable_fallback_header_is_taken_as_given() {
        let id = CorrelationId::from_headers(only(
            "x-correlation-id",
            "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
        ));
        assert_eq!(id.as_str(), "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee");
        assert_eq!(id.source(), Source::Header);
    }

    #[test]
    fn an_unacceptable_fallback_header_is_replaced_not_truncated() {
        for bad in [
            "short",
            "has space",
            "has\ttab",
            "semi;colon",
            "line\nbreak",
        ] {
            let id = CorrelationId::from_headers(|n: &str| {
                if n == "x-correlation-id" {
                    Some(bad)
                } else {
                    None
                }
            });
            assert_eq!(id.source(), Source::Minted, "accepted {bad:?}");
        }
    }

    #[test]
    fn a_traceparent_wins_over_the_fallback_header() {
        let id = CorrelationId::from_headers(|n: &str| match n {
            "traceparent" => Some("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"),
            "x-correlation-id" => Some("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"),
            _ => None,
        });
        assert_eq!(id.as_str(), TRACE);
    }

    #[test]
    fn nothing_usable_mints_a_non_empty_id() {
        let id = CorrelationId::from_headers(|_: &str| None);
        assert_eq!(id.source(), Source::Minted);
        // `audit_events` has CHECK (correlation_id <> ''), and the accepted shape is the one the
        // other two implementations use — a minted id must satisfy it.
        assert!(is_acceptable(id.as_str()));
    }

    #[test]
    fn a_minted_id_is_unique_per_call() {
        assert_ne!(CorrelationId::mint(), CorrelationId::mint());
    }
}
