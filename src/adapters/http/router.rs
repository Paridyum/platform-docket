//! Method and path to handler. Holds no rules.
//!
//! Every arm calls one `application::` function and maps its result to a wire
//! type in `dto`. If an arm ever needs an `if` about what a person owes, that
//! `if` belongs in `application::` — see ANTI-0004 and §2 of the design.

use crate::adapters::auth::OperatorVerifier;
use crate::adapters::blobs::Blobs;
use crate::adapters::clock::Clock;
use crate::adapters::http::dto;
use crate::adapters::http::operator;
use crate::adapters::http::request::Request;
use crate::adapters::http::response::Response;
use crate::adapters::registry::Registry;
use crate::adapters::store::Store;
use crate::application::{archive, check, comms, enrol, issue, next, personnel, roster, sign};
use crate::config::Paths;
use crate::domain::identity::Standing;
use crate::domain::signature::Method;
use anyhow::Result;
use serde::Serialize;

/// Everything a handler needs, assembled once at startup.
pub struct Context<'a> {
    /// The record store — Postgres on the served backend, files on the CLI (ADR-0093, the port). The
    /// four sealed chains and the person document reach it through this; blobs still go via `paths`.
    pub store: &'a dyn Store,
    /// The BLOB store — Garage/S3 on the served backend, files on the CLI (design §11, ADR-0094).
    /// Document BYTES (issued PDFs, packs, scans) reach it through this port; the register records the
    /// blob KEY. Generated indexes, templates and `.gitignore` reads still go via `paths`.
    pub blobs: &'a dyn Blobs,
    /// Where the remaining filesystem artifacts live: packs' output dir, generated indexes, templates
    /// (design §4). Document bytes moved to `blobs` (§11).
    pub paths: &'a Paths,
    /// The registry, loaded once. Nothing here writes to it (§8).
    pub registry: &'a Registry,
    /// Verifies operator tokens on the served backend (ADR-0092). `None` on the loopback/break-glass
    /// backend, where the attribution header stands in and SSH is the authentication (design §5).
    pub verifier: Option<&'a OperatorVerifier>,
    /// Injected, never `SystemTime::now` — STD-0004, and it makes retention
    /// arithmetic testable on a fixed date. `Sync` so the thread-per-request server can share the
    /// context across request threads (the store and verifier are already `Sync`).
    pub clock: &'a (dyn Clock + Sync),
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

    // Every write resolves an operator first and refuses without one — see
    // `operator::resolve`, which is also what makes a cross-site form unable to
    // forge one of these.
    let result = match (req.method.as_str(), parts.as_slice()) {
        ("GET", ["api", "health"]) => Ok(health()),
        ("GET", ["api", "roster"]) => roster_handler(ctx, req),
        ("GET", ["api", "person", slug]) => person_handler(ctx, slug),
        ("GET", ["api", "check"]) => check_handler(ctx),
        ("GET", ["api", "retention"]) => retention_handler(ctx),
        ("GET", ["api", "emails", slug]) => emails_handler(ctx, slug),
        ("GET", ["api", "form", class]) => form_handler(ctx, class),
        ("GET", ["api", "operators"]) => Ok(json(
            &ctx.registry
                .operators
                .iter()
                .filter(|o| o.active)
                .map(dto::Operator::from)
                .collect::<Vec<_>>(),
        )),

        ("POST", ["api", "person"]) => enrol_handler(ctx, req),
        ("POST", ["api", "person", slug, "onboard"]) => onboard_handler(ctx, req, slug),
        ("POST", ["api", "person", slug, "sign"]) => sign_handler(ctx, req, slug),
        ("POST", ["api", "person", slug, "advance"]) => advance_handler(ctx, req, slug),

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
///
/// `contract` is the one a client should act on. `version` moves whenever anything
/// ships; `contract` moves only when the shape of a response changes in a way a
/// client would read wrong, which is the question a client is actually asking.
#[derive(Debug, Serialize)]
struct Health {
    ok: bool,
    version: &'static str,
    contract: u32,
}

/// Serialise, or report the failure to serialise rather than hiding it.
/// The unauthenticated health response (`GET /api/health`). Public so the served boundary can
/// answer the k8s probe without a token and without building a tenant store (ADR-0099): health
/// names no tenant, so it must not be behind the tenant-from-token gate the rest of the API is.
#[must_use]
pub fn health() -> Response {
    json(&Health {
        ok: true,
        version: env!("CARGO_PKG_VERSION"),
        contract: dto::CONTRACT,
    })
}

fn json<T: Serialize>(value: &T) -> Response {
    match serde_json::to_vec(value) {
        Ok(body) => Response::json(body),
        Err(e) => Response::error(500, &format!("could not serialise the response: {e}")),
    }
}

/// Markers letting the dispatcher tell a category of refusal from a real failure.
///
/// Matching on a message is the fragile thing this avoids elsewhere, so these are
/// private constants compared against themselves rather than prose anyone might
/// reword. A handler attaches one as context; nothing parses English.
const NOT_FOUND: &str = "docket::http::not_found";

/// The request was understood and the caller may not do it.
///
/// Distinct from 500 because the two send an operator to different places: 403
/// means fix the request, 500 means the tool or the store is broken. An
/// authorisation failure reported as 500 reads as an outage.
pub const NOT_PERMITTED: &str = "docket::http::not_permitted";

/// The request carried no valid credentials.
///
/// Distinct from 403 (`NOT_PERMITTED`) because the remedies differ: 401 means present a valid token,
/// 403 means you are known but not allowed. The served backend returns 401 for a missing or
/// unverifiable operator token (ADR-0092) and 403 for a verified subject bound to no operator.
pub const NOT_AUTHENTICATED: &str = "docket::http::not_authenticated";

fn status_for(e: &anyhow::Error) -> u16 {
    let marked = |m: &str| e.chain().any(|c| c.to_string() == m);
    if marked(NOT_FOUND) {
        404
    } else if marked(NOT_AUTHENTICATED) {
        401
    } else if marked(NOT_PERMITTED) {
        403
    } else {
        500
    }
}

fn not_found(what: &str) -> anyhow::Error {
    anyhow::anyhow!(NOT_FOUND).context(what.to_owned())
}

fn roster_handler(ctx: &Context<'_>, req: &Request) -> Result<Response> {
    let all = roster::build(ctx.store, ctx.registry)?;

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
    let ledger = ctx.store.ledger()?;
    if !ledger.iter().any(|r| r.slug == slug) {
        return Err(not_found(&format!("no person with slug {slug}")));
    }

    let dossier = personnel::dossier(ctx.store, ctx.registry, slug, None)?;
    let erased = dossier.identity.standing == Standing::Erased;

    // An erased person has no record on disk, so there is no stage to read and
    // nothing outstanding. Returning empty lists is correct rather than an
    // omission: the identifier survives as a tombstone and the workflow does not.
    let (steps, blockers) = if erased {
        (Vec::new(), Vec::new())
    } else {
        let person = ctx.store.person(slug)?;
        let gates = personnel::unmet_gates(
            ctx.store,
            ctx.registry,
            &dossier.identity,
            person.stage(),
            person.class(),
        )?;
        // No company picker in the console yet; an ambiguous person is refused
        // rather than resolved by recency (ADR-0077 D3).
        let steps = next::steps(ctx.store, ctx.paths, ctx.registry, slug, None)?;
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
    let findings = check::run(ctx.store, ctx.blobs, ctx.paths, ctx.registry)?;
    Ok(json(&dto::Check::from(&findings)))
}

fn retention_handler(ctx: &Context<'_>) -> Result<Response> {
    let (due, unevaluated) = archive::retention_due(ctx.store, ctx.registry, ctx.clock.today()?)?;
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
    let ledger = ctx.store.ledger()?;
    if !ledger.iter().any(|r| r.slug == slug) {
        return Err(not_found(&format!("no person with slug {slug}")));
    }
    let due = comms::due(ctx.store, ctx.registry, slug, None)?;
    let messages: Vec<dto::Message> = due.iter().map(dto::Message::from).collect();
    Ok(json(&messages))
}

/// A refusal is a 409, not a 400 and not a 500.
///
/// The distinction is the whole point of this API: 400 means the request was
/// wrong, 500 means we are broken, and 409 means the request was understood
/// perfectly and the domain declined. Only the third is information the operator
/// can act on, and flattening the three is how a UI comes to render a refusal as
/// an error and invite a retry.
const REFUSED: u16 = 409;

/// Parse a JSON body, or say what was wrong with it.
fn body<T: serde::de::DeserializeOwned>(req: &Request) -> Result<T> {
    if req.body.is_empty() {
        anyhow::bail!("this route needs a JSON body and none was sent");
    }
    serde_json::from_slice(&req.body)
        .map_err(|e| anyhow::anyhow!("could not read the request body as JSON: {e}"))
}

/// `GET /api/form/{class}` — the enrolment questions for a class.
///
/// Generated from `registry/fields.yaml`, which is the same source the CLI's
/// interview reads. The form and the prompts therefore cannot diverge: a field
/// declared in the registry appears in both with its stated purpose as help, and
/// a field the class forbids appears in neither. That is the property worth
/// having — a form written by hand is a second, drifting statement of what may be
/// collected, and over-collection is a DPDP breach rather than untidiness.
fn form_handler(ctx: &Context<'_>, class_name: &str) -> Result<Response> {
    let Some(class) = ctx.registry.catalogue.class(class_name) else {
        return Err(not_found(&format!("no person class named {class_name}")));
    };
    let questions = crate::domain::personnel::questions(&ctx.registry.catalogue, class);
    Ok(json(&dto::Form {
        class: class_name.to_owned(),
        questions: questions.iter().map(dto::Question::from).collect(),
    }))
}

/// `POST /api/person` — open a record and allocate an identifier.
///
/// Writes the record from the answers, then allocates. That order matches
/// `docket new --interactive` exactly, and it matters: `personnel::create`
/// appends to the identity ledger, and a ledger entry for a record that failed to
/// write would be a permanent row pointing at nothing.
fn enrol_handler(ctx: &Context<'_>, req: &Request) -> Result<Response> {
    let by = operator::resolve(ctx.registry, ctx.verifier, req)?;
    let ask: dto::EnrolRequest = body(req)?;

    if ctx.registry.catalogue.class(&ask.class).is_none() {
        anyhow::bail!("no person class named {}", ask.class);
    }

    let yaml = enrol::to_yaml(ctx.registry, &ask.class, &ask.answers);
    enrol::write_record(ctx.store, &ask.slug, &yaml)?;
    let record = personnel::create(
        ctx.store,
        ctx.paths,
        ctx.registry,
        &ask.slug,
        &ask.class,
        // The console has no company picker yet, so it opens records at the
        // default company. Explicit rather than implied: the moment the console
        // needs to open a Paridyum Cyber record, this line is what must change,
        // and a caller that silently inherited a default would not say so.
        ctx.registry.default_company().unwrap_or("PAR"),
        ctx.clock.today()?,
        by,
    )?;

    Ok(json(&dto::Wrote {
        did: format!(
            "opened a record for {} as {} and allocated {}",
            record.slug, record.class, record.person_id
        ),
        issued: Vec::new(),
        stage: None,
    }))
}

/// `POST /api/person/{slug}/onboard` — issue everything owed at this stage.
fn onboard_handler(ctx: &Context<'_>, req: &Request, slug: &str) -> Result<Response> {
    let by = operator::resolve(ctx.registry, ctx.verifier, req)?;
    let person = ctx.store.person(slug)?;

    // Refusals first, and NOT forceable from here.
    //
    // The CLI has `--force` for the judgement calls. This route deliberately does
    // not expose it: §8 forbids an interface that offers a way past a refusal, and
    // a console button that issues defective documents is worse than no button.
    // An operator who genuinely means to override uses the CLI, where the flag is
    // typed out and visible in shell history.
    let refusals = issue::refusals(ctx.registry, &person);
    if !refusals.is_empty() {
        return Ok(refused(
            &format!("issue the documents owed by {slug}"),
            Vec::new(),
            refusals,
        ));
    }

    let who = personnel::identify(ctx.store, slug)?;
    let register = ctx.store.register()?;

    // Refuse BEFORE rendering anything, naming the fields that are missing.
    //
    // Without this the first document reaches pdflatex, which fails with
    // "Undefined control sequence" and a 500 — a LaTeX diagnostic for what is
    // really an incomplete record, and no indication of which field to fill.
    // Found by enrolling through the API with a partial set of answers, which is
    // exactly what a half-completed form sends.
    //
    // The names are record paths rather than LaTeX commands (D-44), so the
    // refusal says which lines of people/<slug>.yaml to edit. This route reads
    // them straight through; the translation happens in `unsupplied_fields`.
    let unrenderable = issue::unrenderable(ctx.paths, ctx.registry, &person, &who, &register)?;
    if !unrenderable.is_empty() {
        let reasons = unrenderable
            .iter()
            .map(|(code, fields)| {
                format!("{code} needs these record fields: {}", fields.join(", "))
            })
            .collect();
        return Ok(refused(
            &format!("issue the documents owed by {slug}"),
            Vec::new(),
            reasons,
        ));
    }

    let todo = issue::outstanding(ctx.registry, &person, &who, &register)?;
    let today = ctx.clock.today()?;

    let by_code = ctx.registry.lifecycle.by_code()?;
    let mut issued = Vec::new();
    for code in &todo {
        let Some(doc) = by_code.get(code.as_str()) else {
            anyhow::bail!("{code}: declared by a stage but not by documents");
        };
        let entry = issue::one(
            ctx.store,
            ctx.blobs,
            ctx.paths,
            ctx.registry,
            &person,
            doc,
            today,
            by,
        )?;
        issued.push(dto::Document {
            code: entry.code.clone(),
            serial: entry.serial.clone(),
            issued_on: entry.issued_on.clone(),
        });
    }

    Ok(json(&dto::Wrote {
        did: if issued.is_empty() {
            format!("nothing was owed by {slug} at this stage")
        } else {
            format!("issued {} document(s) for {slug}", issued.len())
        },
        issued,
        stage: None,
    }))
}

/// `POST /api/person/{slug}/sign` — record a document coming back executed.
fn sign_handler(ctx: &Context<'_>, req: &Request, slug: &str) -> Result<Response> {
    let by = operator::resolve(ctx.registry, ctx.verifier, req)?;
    let ask: dto::SignRequest = body(req)?;

    // The serial must belong to this person. Without this check a request could
    // record a signature against anybody's document through anybody's URL, and
    // the signature ledger would be right while the audit trail was misleading.
    let who = personnel::identify(ctx.store, slug)?;
    let register = ctx.store.register()?;
    let owned = register
        .iter()
        .any(|e| e.serial == ask.serial && e.person_id == who.person_id);
    if !owned {
        anyhow::bail!(
            "{} was not issued to {slug}; recording it here would file a signature \
             against the wrong person",
            ask.serial
        );
    }

    let method = Method::parse(&ask.method)?;
    let signed_on = crate::adapters::clock::parse_iso(&ask.signed_on)?;
    let received_on = match ask.received_on.as_deref() {
        Some(d) => crate::adapters::clock::parse_iso(d)?,
        None => signed_on,
    };
    let scan = ask.scan.as_deref().map(std::path::Path::new);

    let signed = sign::record(
        ctx.store,
        ctx.blobs,
        ctx.registry,
        &ask.serial,
        method,
        signed_on,
        received_on,
        scan,
        by,
    )?;

    Ok(json(&dto::Wrote {
        did: format!(
            "recorded {} ({}) as signed on {}",
            signed.record.serial, signed.title, signed.record.signed_on
        ),
        issued: Vec::new(),
        stage: None,
    }))
}

/// `POST /api/person/{slug}/advance` — move to the next lifecycle stage.
fn advance_handler(ctx: &Context<'_>, req: &Request, slug: &str) -> Result<Response> {
    let by = operator::resolve(ctx.registry, ctx.verifier, req)?;
    // A body is optional here; absent means `force: false`.
    let ask: dto::AdvanceRequest = if req.body.is_empty() {
        dto::AdvanceRequest { force: false }
    } else {
        body(req)?
    };

    let (from, to, gates) = personnel::advance(
        ctx.store,
        ctx.paths,
        ctx.registry,
        slug,
        ctx.clock.today()?,
        ask.force,
        by,
    )?;

    // A non-empty `gates` with a successful call means the move did NOT happen.
    // `personnel::advance` returns the gates instead of moving, which is why this
    // is a refusal rather than an error.
    if !gates.is_empty() {
        return Ok(refused(
            &format!("advance {slug} from {from} to {to}"),
            gates.iter().map(dto::Blocker::from).collect(),
            Vec::new(),
        ));
    }

    Ok(json(&dto::Wrote {
        did: format!("advanced {slug} from {from} to {to}"),
        issued: Vec::new(),
        stage: Some(to),
    }))
}

/// A refusal: understood, declined, and said so in the shape the UI already reads.
fn refused(attempted: &str, blockers: Vec<dto::Blocker>, refusals: Vec<String>) -> Response {
    let payload = dto::Refused {
        attempted: attempted.to_owned(),
        blockers,
        refusals,
    };
    match serde_json::to_vec(&payload) {
        Ok(bytes) => Response {
            status: REFUSED,
            body: bytes,
        },
        Err(e) => Response::error(500, &format!("could not serialise the refusal: {e}")),
    }
}
