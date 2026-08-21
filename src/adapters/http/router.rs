//! Method and path to handler. Holds no rules.
//!
//! Every arm calls one `application::` function and maps its result to a wire
//! type in `dto`. If an arm ever needs an `if` about what a person owes, that
//! `if` belongs in `application::` — see ANTI-0004 and §2 of the design.

use crate::adapters::clock::Clock;
use crate::adapters::http::dto;
use crate::adapters::http::request::Request;
use crate::adapters::http::response::Response;
use crate::adapters::registry::Registry;
use crate::adapters::store;
use crate::application::{archive, check, comms, next, personnel, roster};
use crate::config::Paths;
use crate::domain::identity::Standing;
use anyhow::Result;
use serde::Serialize;

/// Everything a handler needs, assembled once at startup.
pub struct Context<'a> {
    /// Where the store lives.
    pub paths: &'a Paths,
    /// The registry, loaded once. Nothing here writes to it (§8).
    pub registry: &'a Registry,
    /// Injected, never `SystemTime::now` — STD-0004, and it makes retention
    /// arithmetic testable on a fixed date.
    pub clock: &'a dyn Clock,
}

/// Dispatch one request.
///
/// Never returns `Err`: a handler that fails becomes a 500 carrying the reason,
/// because a connection that closes with nothing written tells the operator only
/// that something is broken. The error text is the one the domain produced.
#[must_use]
pub fn route(ctx: &Context<'_>, req: &Request) -> Response {
    let segments = req.segments();
    let parts: Vec<&str> = segments;

    // Read-only routes only, so far. Writes arrive with operator resolution,
    // which §10 of the design sequences BEFORE the first write for the obvious
    // reason: a write that cannot name who made it is not auditable.
    let result = match (req.method.as_str(), parts.as_slice()) {
        ("GET", ["api", "health"]) => Ok(json(&Health {
            ok: true,
            version: env!("CARGO_PKG_VERSION"),
        })),
        ("GET", ["api", "roster"]) => roster_handler(ctx, req),
        ("GET", ["api", "person", slug]) => person_handler(ctx, slug),
        ("GET", ["api", "check"]) => check_handler(ctx),
        ("GET", ["api", "retention"]) => retention_handler(ctx),
        ("GET", ["api", "emails", slug]) => emails_handler(ctx, slug),

        // A known path with the wrong method is a 405, not a 404. The distinction
        // matters to whoever is debugging the console: 404 sends them looking for
        // a typo in a path that is correct.
        (
            _,
            ["api", "health" | "roster" | "check" | "retention"] | ["api", "person" | "emails", _],
        ) => {
            return Response::error(
                405,
                &format!("{} is not allowed on {}", req.method, req.path),
            );
        }

        _ => return Response::error(404, &format!("no route for {} {}", req.method, req.path)),
    };

    match result {
        Ok(response) => response,
        Err(e) => {
            // `{e:#}` prints the whole anyhow chain. The context the domain
            // attached is the useful part — "ledger names class X, which is not
            // declared" is actionable and "internal server error" is not.
            Response::error(status_for(&e), &format!("{e:#}"))
        }
    }
}

/// `/api/health` payload.
#[derive(Debug, Serialize)]
struct Health {
    ok: bool,
    version: &'static str,
}

/// Serialise, or report the failure to serialise rather than hiding it.
fn json<T: Serialize>(value: &T) -> Response {
    match serde_json::to_vec(value) {
        Ok(body) => Response::json(body),
        Err(e) => Response::error(500, &format!("could not serialise the response: {e}")),
    }
}

/// A 404 carries a marker so the dispatcher can tell it from a real failure.
///
/// Matching on a message is exactly the fragile thing this avoids elsewhere, so
/// the marker is a private constant compared against itself rather than prose
/// anyone might reword.
const NOT_FOUND: &str = "docket::http::not_found";

fn status_for(e: &anyhow::Error) -> u16 {
    if e.chain().any(|c| c.to_string() == NOT_FOUND) {
        404
    } else {
        500
    }
}

fn not_found(what: &str) -> anyhow::Error {
    anyhow::anyhow!(NOT_FOUND).context(what.to_owned())
}

fn roster_handler(ctx: &Context<'_>, req: &Request) -> Result<Response> {
    let all = roster::build(ctx.paths, ctx.registry)?;

    // `?standing=active` filters, exactly as `docket roster --standing` does.
    // Parsed here rather than in `application::`, because which query parameters
    // exist is a property of this transport and not of the roster.
    let wanted = req.query.as_deref().and_then(|q| {
        q.split('&')
            .find_map(|pair| pair.strip_prefix("standing="))
            .map(str::to_owned)
    });

    let lines: Vec<&roster::Line> = all
        .iter()
        .filter(|l| wanted.as_deref().is_none_or(|s| l.standing.as_str() == s))
        .collect();

    // Tallies come from the WHOLE roster, never the filtered view. A filtered
    // page showing "1 active" because one row is displayed would misreport the
    // organisation's size.
    let (active, separated, erased) = roster::tally(&all);

    Ok(json(&dto::Roster {
        people: lines.into_iter().map(dto::RosterLine::from).collect(),
        active,
        separated,
        erased,
    }))
}

fn person_handler(ctx: &Context<'_>, slug: &str) -> Result<Response> {
    // Membership is decided from the ledger rather than by catching an error and
    // reading its message, so "unknown person" and "the ledger is unreadable"
    // cannot be confused. One returns 404 and the other 500, and conflating them
    // would send an operator hunting for a typo during a disk failure.
    let ledger = store::ledger(ctx.paths)?;
    if !ledger.iter().any(|r| r.slug == slug) {
        return Err(not_found(&format!("no person with slug {slug}")));
    }

    let dossier = personnel::dossier(ctx.paths, ctx.registry, slug)?;
    let erased = dossier.identity.standing == Standing::Erased;

    // An erased person has no record on disk, so there is no stage to read and
    // nothing outstanding. Returning empty lists is correct rather than an
    // omission: the identifier survives as a tombstone and the workflow does not.
    let (steps, blockers) = if erased {
        (Vec::new(), Vec::new())
    } else {
        let person = store::person(ctx.paths, slug)?;
        let gates =
            personnel::unmet_gates(ctx.paths, ctx.registry, &dossier.identity, person.stage())?;
        let steps = next::steps(ctx.paths, ctx.registry, slug)?;
        (steps, gates)
    };

    Ok(json(&dto::Person {
        person_id: dossier.identity.person_id.to_string(),
        engagement_id: dossier.identity.engagement_id.to_string(),
        slug: dossier.identity.slug.clone(),
        class: dossier.identity.class.clone(),
        standing: dossier.identity.standing.as_str().to_owned(),
        created_on: dossier.identity.created_on.clone(),
        standing_changed_on: dossier.identity.standing_changed_on.clone(),
        erased,
        documents: dossier
            .documents
            .iter()
            .map(|(code, serial, issued_on)| dto::Document {
                code: code.clone(),
                serial: serial.clone(),
                issued_on: issued_on.clone(),
            })
            .collect(),
        open_access: dossier.open_access.clone(),
        history: dossier
            .history
            .iter()
            .map(|(at, kind, detail)| dto::HistoryEntry {
                at: at.clone(),
                kind: kind.clone(),
                detail: detail.clone(),
            })
            .collect(),
        violations: dossier.violations.iter().map(ToString::to_string).collect(),
        steps: steps.iter().map(dto::Step::from).collect(),
        blockers: blockers.iter().map(dto::Blocker::from).collect(),
    }))
}

fn check_handler(ctx: &Context<'_>) -> Result<Response> {
    let findings = check::run(ctx.paths, ctx.registry)?;
    Ok(json(&dto::Check::from(&findings)))
}

fn retention_handler(ctx: &Context<'_>) -> Result<Response> {
    let (due, unevaluated) = archive::retention_due(ctx.paths, ctx.registry, ctx.clock.today()?)?;
    Ok(json(&dto::Retention {
        overdue: due
            .iter()
            .map(|o| dto::Overdue {
                serial: o.serial.clone(),
                person_id: o.person_id.clone(),
                class: o.class.clone(),
                keep_for: o.keep_for.clone(),
                basis: o.basis.clone(),
                since: o.since.clone(),
                days: o.days,
            })
            .collect(),
        unevaluated,
    }))
}

fn emails_handler(ctx: &Context<'_>, slug: &str) -> Result<Response> {
    let ledger = store::ledger(ctx.paths)?;
    if !ledger.iter().any(|r| r.slug == slug) {
        return Err(not_found(&format!("no person with slug {slug}")));
    }
    let due = comms::due(ctx.paths, ctx.registry, slug)?;
    let messages: Vec<dto::Message> = due.iter().map(dto::Message::from).collect();
    Ok(json(&messages))
}
