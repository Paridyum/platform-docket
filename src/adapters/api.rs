//! The API-mode client: docket as an authenticated HTTP client of a remote docket (ADR-0097 §1).
//!
//! With `DOCKET_API_URL` set, the CLI does not run the application layer or touch a store: it calls
//! the served backend's `/api/*` endpoints (the same ones `console-docket` speaks) with an operator
//! token, and renders the JSON. This module is the transport and the rendering — nothing here
//! decides workflow, exactly as the server's router does not.
//!
//! The wire types are mirrored here rather than shared with `adapters::http::dto`: the served shapes
//! are a published contract, and a client that re-used the server's structs would silently track a
//! field rename that a real client would not (the same deliberate duplication ADR-0093 keeps between
//! docket and console-docket). The `render_*` functions are pure `-> String`, so the output an
//! operator sees is unit-testable without a server.

use anyhow::{Context, Result, anyhow};
use reqwest::blocking::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt::Write as _;

// ---------------------------------------------------------------------------
// Wire types received. Mirror of `adapters::http::dto`, deserialized here.
// ---------------------------------------------------------------------------

/// One roster row.
#[derive(Debug, Clone, Deserialize)]
pub struct RosterLine {
    /// Permanent, opaque person identifier.
    pub person_id: String,
    /// Identifier of the current engagement.
    pub engagement_id: String,
    /// Filename the record is kept under.
    pub slug: String,
    /// Person class of this engagement.
    pub class: String,
    /// Current standing, as the ledger records it.
    pub standing: String,
    /// Name, where the record still exists.
    #[serde(default)]
    pub name: Option<String>,
    /// Documents issued to this person.
    #[serde(default)]
    pub documents: usize,
}

/// The roster and its tallies.
#[derive(Debug, Clone, Deserialize)]
pub struct Roster {
    /// One row per engagement.
    pub people: Vec<RosterLine>,
    /// Active engagements across the whole roster.
    pub active: usize,
    /// Separated engagements across the whole roster.
    pub separated: usize,
    /// Erased engagements across the whole roster.
    pub erased: usize,
}

/// One issued document.
#[derive(Debug, Clone, Deserialize)]
pub struct Document {
    /// Document code.
    pub code: String,
    /// Serial of the document.
    pub serial: String,
    /// ISO date it was issued.
    pub issued_on: String,
}

/// One event from a person's history.
#[derive(Debug, Clone, Deserialize)]
pub struct HistoryEntry {
    /// When it happened.
    pub at: String,
    /// What kind of event it was.
    pub kind: String,
    /// Free text the event carried, where it carried any.
    #[serde(default)]
    pub detail: Option<String>,
}

/// Why a person may not leave their current stage yet.
#[derive(Debug, Clone, Deserialize)]
pub struct Blocker {
    /// Document code the stage requires executed.
    pub code: String,
    /// Document title, for the message.
    pub title: String,
    /// Why it is not satisfied.
    pub reason: String,
}

/// One outstanding step, in workflow order.
#[derive(Debug, Clone, Deserialize)]
pub struct Step {
    /// What it is, in a sentence.
    pub what: String,
    /// Why it is next.
    pub why: String,
    /// The equivalent CLI command, where one exists.
    #[serde(default)]
    pub command: Option<String>,
}

/// One check finding.
#[derive(Debug, Clone, Deserialize)]
pub struct Finding {
    /// Whether this must block, or merely be reported.
    pub blocking: bool,
    /// What is wrong, in one line.
    pub message: String,
}

/// Everything `check` looked at and found.
#[derive(Debug, Clone, Deserialize)]
pub struct Check {
    /// Problems, blocking first.
    pub findings: Vec<Finding>,
    /// How many findings block.
    pub errors: usize,
    /// Documents counted.
    pub documents: usize,
    /// Stages declared.
    pub stages: usize,
    /// Entries in the register.
    pub entries: usize,
}

/// A person's page.
// `person_id` is the domain's name for the identifier, matching the served `dto::Person`; renaming
// it here would make the client read `person_id` on a roster row and `id` on a person.
#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, Deserialize)]
pub struct Person {
    /// Permanent, opaque person identifier.
    pub person_id: String,
    /// Identifier of the current engagement.
    pub engagement_id: String,
    /// Filename the record is kept under.
    pub slug: String,
    /// Person class of this engagement.
    pub class: String,
    /// Current standing, as the ledger records it.
    pub standing: String,
    /// ISO date the person identifier was first allocated.
    pub created_on: String,
    /// ISO date standing last changed.
    pub standing_changed_on: String,
    /// True once the record is gone and only the identifier remains.
    pub erased: bool,
    /// Documents issued to this person.
    pub documents: Vec<Document>,
    /// Systems granted and not revoked.
    pub open_access: Vec<String>,
    /// Event history, oldest first.
    pub history: Vec<HistoryEntry>,
    /// Ways the record does not match what its class declares.
    pub violations: Vec<String>,
    /// What to do next, most urgent first.
    pub steps: Vec<Step>,
    /// Why this person may not leave their current stage yet.
    pub blockers: Vec<Blocker>,
}

/// One document past its retention period.
#[derive(Debug, Clone, Deserialize)]
pub struct Overdue {
    /// Serial of the document.
    pub serial: String,
    /// Permanent, opaque person identifier.
    pub person_id: String,
    /// Person class of this engagement.
    pub class: String,
    /// How long that class keeps it, in prose.
    pub keep_for: String,
    /// Which date the clock ran from.
    pub basis: String,
    /// The date the clock started from.
    pub since: String,
    /// Days elapsed since then.
    pub days: i64,
}

/// Documents past their retention period, and the ones that could not be judged.
#[derive(Debug, Clone, Deserialize)]
pub struct Retention {
    /// Documents past their retention period.
    pub overdue: Vec<Overdue>,
    /// Retention periods this tool could not parse, and therefore did not judge.
    pub unevaluated: Vec<String>,
}

/// One attachment a message names.
#[derive(Debug, Clone, Deserialize)]
pub struct Attachment {
    /// Document code.
    pub code: String,
    /// Document title, for the message.
    pub title: String,
    /// Serial, or a note that it has not been issued.
    pub serial: String,
}

/// One message that is due, rendered ready to copy.
#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    /// Which message this is.
    pub id: String,
    /// Stage it belongs to.
    pub stage: String,
    /// Who it goes to.
    pub to: String,
    /// The address, where the record holds one.
    #[serde(default)]
    pub address: Option<String>,
    /// When to send it.
    pub when: String,
    /// Filled subject.
    pub subject: String,
    /// Filled body.
    pub body: String,
    /// Attachments the message names, with their serials.
    pub attachments: Vec<Attachment>,
    /// Attachments the message names that have not been issued.
    pub missing: Vec<String>,
}

/// One enrolment question, from `GET /api/form/{class}`.
#[derive(Debug, Clone, Deserialize)]
pub struct Question {
    /// Dotted path the answer is written to.
    pub path: String,
    /// Field group, used to order and head the questions.
    pub group: String,
    /// The declared purpose, shown as help.
    pub purpose: String,
    /// Whether the class requires an answer.
    pub required: bool,
    /// Every value this field accepts, or empty for free text.
    #[serde(default)]
    pub choices: Vec<String>,
    /// A safe starting value, where the registry declares one.
    #[serde(default)]
    pub default: Option<String>,
}

/// The enrolment form for a class.
#[derive(Debug, Clone, Deserialize)]
pub struct Form {
    /// The class these questions belong to.
    pub class: String,
    /// Questions in catalogue order, grouped by `group`.
    pub questions: Vec<Question>,
}

/// What a write did.
#[derive(Debug, Clone, Deserialize)]
pub struct Wrote {
    /// Plain sentence naming what changed, for an operator to read.
    pub did: String,
    /// Documents issued by this call, if any.
    #[serde(default)]
    pub issued: Vec<Document>,
    /// Where the person now is, if the stage changed.
    #[serde(default)]
    pub stage: Option<String>,
}

/// A write that was refused, and why.
#[derive(Debug, Clone, Deserialize)]
pub struct Refused {
    /// What was attempted.
    pub attempted: String,
    /// Why it may not happen yet.
    #[serde(default)]
    pub blockers: Vec<Blocker>,
    /// Refusals that are not stage gates.
    #[serde(default)]
    pub refusals: Vec<String>,
}

/// The result of a write over the API: the server did it, or understood and declined.
#[derive(Debug, Clone)]
pub enum PostOutcome {
    /// The server performed the write.
    Wrote(Wrote),
    /// The server understood the request and declined it (HTTP 409).
    Refused(Refused),
}

// ---------------------------------------------------------------------------
// Wire types sent. Serialized here; must match `adapters::http::dto`.
// ---------------------------------------------------------------------------

/// `POST /api/person`.
#[derive(Debug, Serialize)]
pub struct EnrolRequest {
    /// Filename the record is filed under.
    pub slug: String,
    /// Person class. Decides what may be collected at all.
    pub class: String,
    /// Answers, keyed by the dotted path they are written to.
    pub answers: BTreeMap<String, String>,
}

/// `POST /api/person/{slug}/sign`.
#[derive(Debug, Serialize)]
pub struct SignRequest {
    /// Serial of the document, e.g. `PAR/NDA/2026/0001`.
    pub serial: String,
    /// How it was signed: `wet-ink` | `aadhaar-esign` | `electronic`.
    pub method: String,
    /// ISO date the person signed.
    pub signed_on: String,
    /// ISO date Paridyum received it. Defaults, server-side, to the signing date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub received_on: Option<String>,
    /// Path to the scan. Not supported over the API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan: Option<String>,
}

/// `POST /api/person/{slug}/advance`.
#[derive(Debug, Serialize)]
pub struct AdvanceRequest {
    /// Advance despite unmet gates.
    pub force: bool,
}

// ---------------------------------------------------------------------------
// The client.
// ---------------------------------------------------------------------------

/// An authenticated HTTP client of a remote docket's `/api/*` (ADR-0097).
pub struct ApiClient {
    base: String,
    token: String,
    http: Client,
}

/// One raw reply: the status and the body, before it is interpreted.
struct Reply {
    status: u16,
    body: Vec<u8>,
}

impl ApiClient {
    /// Build a client for `base` (e.g. `https://docket.paridyum.com`) presenting `token`.
    ///
    /// # Errors
    /// Fails if the underlying HTTP client cannot be built.
    pub fn new(base: &str, token: String) -> Result<Self> {
        let http = Client::builder()
            .build()
            .context("building the HTTP client for the docket API")?;
        Ok(Self {
            base: base.trim_end_matches('/').to_owned(),
            token,
            http,
        })
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }

    fn run(&self, rb: RequestBuilder) -> Result<Reply> {
        let resp = rb
            .bearer_auth(&self.token)
            .send()
            .context("the request to the docket API failed (is DOCKET_API_URL reachable?)")?;
        let status = resp.status().as_u16();
        let body = resp
            .bytes()
            .context("reading the docket API response body")?
            .to_vec();
        Ok(Reply { status, body })
    }

    /// Turn a non-2xx reply into an error, hinting at `docket login` on a 401.
    fn error_for(reply: &Reply) -> anyhow::Error {
        if reply.status == 401 {
            return anyhow!(
                "the docket API rejected the token (401). Run `docket login` to sign in again."
            );
        }
        let message = serde_json::from_slice::<Value>(&reply.body)
            .ok()
            .and_then(|v| {
                v.get("error")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
            })
            .unwrap_or_else(|| String::from_utf8_lossy(&reply.body).into_owned());
        anyhow!("the docket API returned {}: {message}", reply.status)
    }

    fn read_json<T: serde::de::DeserializeOwned>(reply: &Reply) -> Result<T> {
        if (200..300).contains(&reply.status) {
            serde_json::from_slice(&reply.body)
                .context("the docket API response was not the expected JSON")
        } else {
            Err(Self::error_for(reply))
        }
    }

    fn read_post(reply: &Reply) -> Result<PostOutcome> {
        if (200..300).contains(&reply.status) {
            Ok(PostOutcome::Wrote(
                serde_json::from_slice(&reply.body)
                    .context("the docket API response was not the expected JSON")?,
            ))
        } else if reply.status == 409 {
            Ok(PostOutcome::Refused(
                serde_json::from_slice(&reply.body)
                    .context("the docket API refusal was not the expected JSON")?,
            ))
        } else {
            Err(Self::error_for(reply))
        }
    }

    /// `GET /api/roster`, optionally filtered by standing.
    ///
    /// # Errors
    /// Fails on a transport error or a non-2xx response.
    pub fn roster(&self, standing: Option<&str>) -> Result<Roster> {
        let path = match standing {
            Some(s) => format!("/api/roster?standing={s}"),
            None => "/api/roster".to_owned(),
        };
        Self::read_json(&self.run(self.http.get(self.url(&path)))?)
    }

    /// `GET /api/person/{slug}`.
    ///
    /// # Errors
    /// Fails on a transport error or a non-2xx response.
    pub fn person(&self, slug: &str) -> Result<Person> {
        Self::read_json(&self.run(self.http.get(self.url(&format!("/api/person/{slug}"))))?)
    }

    /// `GET /api/emails/{slug}`.
    ///
    /// # Errors
    /// Fails on a transport error or a non-2xx response.
    pub fn emails(&self, slug: &str) -> Result<Vec<Message>> {
        Self::read_json(&self.run(self.http.get(self.url(&format!("/api/emails/{slug}"))))?)
    }

    /// `GET /api/check`.
    ///
    /// # Errors
    /// Fails on a transport error or a non-2xx response.
    pub fn check(&self) -> Result<Check> {
        Self::read_json(&self.run(self.http.get(self.url("/api/check")))?)
    }

    /// `GET /api/retention`.
    ///
    /// # Errors
    /// Fails on a transport error or a non-2xx response.
    pub fn retention(&self) -> Result<Retention> {
        Self::read_json(&self.run(self.http.get(self.url("/api/retention")))?)
    }

    /// `GET /api/form/{class}`.
    ///
    /// # Errors
    /// Fails on a transport error or a non-2xx response.
    pub fn form(&self, class: &str) -> Result<Form> {
        Self::read_json(&self.run(self.http.get(self.url(&format!("/api/form/{class}"))))?)
    }

    /// `POST /api/person`.
    ///
    /// # Errors
    /// Fails on a transport error or a non-2xx, non-409 response.
    pub fn enrol(&self, request: &EnrolRequest) -> Result<PostOutcome> {
        Self::read_post(&self.run(self.http.post(self.url("/api/person")).json(request))?)
    }

    /// `POST /api/person/{slug}/onboard`.
    ///
    /// # Errors
    /// Fails on a transport error or a non-2xx, non-409 response.
    pub fn onboard(&self, slug: &str) -> Result<PostOutcome> {
        Self::read_post(
            &self.run(
                self.http
                    .post(self.url(&format!("/api/person/{slug}/onboard"))),
            )?,
        )
    }

    /// `POST /api/person/{slug}/sign`.
    ///
    /// # Errors
    /// Fails on a transport error or a non-2xx, non-409 response.
    pub fn sign(&self, slug: &str, request: &SignRequest) -> Result<PostOutcome> {
        Self::read_post(
            &self.run(
                self.http
                    .post(self.url(&format!("/api/person/{slug}/sign")))
                    .json(request),
            )?,
        )
    }

    /// `POST /api/person/{slug}/advance`.
    ///
    /// # Errors
    /// Fails on a transport error or a non-2xx, non-409 response.
    pub fn advance(&self, slug: &str, request: &AdvanceRequest) -> Result<PostOutcome> {
        Self::read_post(
            &self.run(
                self.http
                    .post(self.url(&format!("/api/person/{slug}/advance")))
                    .json(request),
            )?,
        )
    }
}

// ---------------------------------------------------------------------------
// Rendering. Pure `-> String`, so what an operator sees is testable.
// ---------------------------------------------------------------------------

/// Render the roster as the CLI's `docket roster` does.
#[must_use]
pub fn render_roster(roster: &Roster) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{:<16} {:<18} {:<10} {:<10} {:<20} {:>4}",
        "PERSON", "ENGAGEMENT", "CLASS", "STANDING", "NAME", "DOCS"
    );
    for l in &roster.people {
        let _ = writeln!(
            out,
            "{:<16} {:<18} {:<10} {:<10} {:<20} {:>4}",
            l.person_id,
            l.engagement_id,
            l.class,
            l.standing,
            l.name.as_deref().unwrap_or("--"),
            l.documents
        );
    }
    let _ = writeln!(
        out,
        "\n{} engagements: {} active, {} separated, {} erased",
        roster.people.len(),
        roster.active,
        roster.separated,
        roster.erased
    );
    out
}

/// Render a person's status page, as `docket status` does.
#[must_use]
pub fn render_status(person: &Person) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{}  ({})", person.person_id, person.slug);
    let _ = writeln!(
        out,
        "engagement {}  class {}",
        person.engagement_id, person.class
    );
    let _ = writeln!(out, "standing: {}\n", person.standing);
    let _ = writeln!(out, "documents ({})", person.documents.len());
    for d in &person.documents {
        let _ = writeln!(out, "  {}  {:<22} {}", d.code, d.serial, d.issued_on);
    }
    if person.steps.is_empty() {
        let _ = writeln!(out, "\nnothing outstanding");
    } else {
        let _ = writeln!(out, "\noutstanding ({}):", person.steps.len());
        for s in &person.steps {
            let _ = writeln!(out, "  - {}", s.what);
        }
    }
    out
}

/// Render everything held about one person, as `docket record` does.
#[must_use]
pub fn render_record(person: &Person) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{}  ({})", person.person_id, person.slug);
    let _ = writeln!(out, "engagement {}", person.engagement_id);
    let _ = writeln!(out, "class      {}", person.class);
    let _ = writeln!(
        out,
        "standing   {}  since {}",
        person.standing, person.standing_changed_on
    );
    let _ = writeln!(out, "created    {}", person.created_on);

    let _ = writeln!(out, "\ndocuments ({})", person.documents.len());
    for d in &person.documents {
        let _ = writeln!(out, "  {}  {:<22} {}", d.code, d.serial, d.issued_on);
    }

    let _ = writeln!(out, "\naccess still open ({})", person.open_access.len());
    for a in &person.open_access {
        let _ = writeln!(out, "  {a}");
    }

    let _ = writeln!(out, "\nhistory ({})", person.history.len());
    for h in &person.history {
        let _ = writeln!(
            out,
            "  {}  {:<20} {}",
            h.at,
            h.kind,
            h.detail.as_deref().unwrap_or("")
        );
    }

    if person.violations.is_empty() {
        let _ = writeln!(
            out,
            "\nrecord matches what class \"{}\" declares",
            person.class
        );
    } else {
        let _ = writeln!(
            out,
            "\n{} problems with the record:",
            person.violations.len()
        );
        for v in &person.violations {
            let _ = writeln!(out, "  {v}");
        }
    }
    out
}

/// Render the next steps, as `docket next` does. `done` is true when nothing is outstanding.
#[must_use]
pub fn render_next(person: &Person) -> (String, bool) {
    let mut out = String::new();
    if person.steps.is_empty() {
        let _ = writeln!(out, "{}: nothing to do", person.slug);
        return (out, true);
    }
    for (i, step) in person.steps.iter().enumerate() {
        let _ = writeln!(out, "{}. {}", i + 1, step.what);
        let _ = writeln!(out, "   why: {}", step.why);
        if let Some(cmd) = &step.command {
            let _ = writeln!(out, "   run: {cmd}");
        }
        let _ = writeln!(out);
    }
    (out, false)
}

/// Render the due messages, as `docket emails` does. `incomplete` is true when any names a document
/// that is not issued.
#[must_use]
pub fn render_emails(messages: &[Message]) -> (String, bool) {
    let mut out = String::new();
    let mut incomplete = false;
    for m in messages {
        let _ = writeln!(out, "{}", "=".repeat(78));
        let _ = writeln!(out, "[{}]  stage {}  ->  {}", m.id, m.stage, m.to);
        let _ = writeln!(out, "WHEN     {}", m.when);
        let _ = writeln!(
            out,
            "TO       {}",
            m.address.as_deref().unwrap_or("(no address recorded)")
        );
        let _ = writeln!(out, "SUBJECT  {}", m.subject);
        if m.attachments.is_empty() {
            let _ = writeln!(out, "ATTACH   (none)");
        } else {
            for a in &m.attachments {
                let _ = writeln!(out, "ATTACH   {}  {:<48} {}", a.code, a.title, a.serial);
            }
        }
        let _ = writeln!(out, "{}", "-".repeat(78));
        let _ = writeln!(out, "{}", m.body.trim_end());
        if !m.missing.is_empty() {
            incomplete = true;
            let _ = writeln!(
                out,
                "\n!! {} names {} document(s) that are not issued: {}",
                m.id,
                m.missing.len(),
                m.missing.join(" ")
            );
        }
    }
    let _ = writeln!(out, "{}", "=".repeat(78));
    (out, incomplete)
}

/// Render the check findings, as `docket check` does.
#[must_use]
pub fn render_check(check: &Check) -> String {
    let mut out = String::new();
    for f in &check.findings {
        if f.blocking {
            let _ = writeln!(out, "ERROR  {}", f.message);
        } else {
            let _ = writeln!(out, "warn   {}", f.message);
        }
    }
    let _ = writeln!(
        out,
        "\n{} documents, {} stages, {} register entries, {} errors",
        check.documents, check.stages, check.entries, check.errors
    );
    out
}

/// Render the retention report, as `docket retention` does.
#[must_use]
pub fn render_retention(retention: &Retention) -> String {
    let mut out = String::new();
    for u in &retention.unevaluated {
        let _ = writeln!(out, "unevaluated  {u}");
    }
    if retention.overdue.is_empty() {
        let _ = writeln!(out, "\nnothing past its retention period");
        return out;
    }
    for d in &retention.overdue {
        let _ = writeln!(
            out,
            "OVERDUE  {}  {}  class {} keeps it {} -- {} {} ({} days ago)",
            d.serial, d.person_id, d.class, d.keep_for, d.basis, d.since, d.days
        );
    }
    let _ = writeln!(
        out,
        "\n{} document(s) past retention.",
        retention.overdue.len()
    );
    out
}

/// Render a completed write.
#[must_use]
pub fn render_wrote(wrote: &Wrote) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{}", wrote.did);
    for d in &wrote.issued {
        let _ = writeln!(out, "  {}  {}  {}", d.code, d.serial, d.issued_on);
    }
    if let Some(stage) = &wrote.stage {
        let _ = writeln!(out, "now at stage: {stage}");
    }
    out
}

/// Render a refusal, as the CLI reports one.
#[must_use]
pub fn render_refused(refused: &Refused) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "refused: {}", refused.attempted);
    for b in &refused.blockers {
        let _ = writeln!(out, "  - {} ({}): {}", b.code, b.title, b.reason);
    }
    for r in &refused.refusals {
        let _ = writeln!(out, "  - {r}");
    }
    out
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use std::io::{Read as _, Write as _};
    use std::net::TcpListener;
    use std::sync::mpsc;

    /// A one-shot mock server: serves one canned reply, capturing the request line and headers so a
    /// test can assert the bearer token was sent. Returns (base URL, receiver of the raw request).
    fn mock_server(status: u16, body: &'static str) -> (String, mpsc::Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        let (ready_tx, ready_rx) = mpsc::channel();
        let (req_tx, req_rx) = mpsc::channel();
        std::thread::spawn(move || {
            ready_tx.send(()).ok();
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 8192];
                let n = stream.read(&mut buf).unwrap_or(0);
                req_tx
                    .send(String::from_utf8_lossy(&buf[..n]).into_owned())
                    .ok();
                let reason = if (200..300).contains(&status) {
                    "OK"
                } else {
                    "Error"
                };
                let response = format!(
                    "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes());
            }
        });
        ready_rx.recv().expect("server started");
        (format!("http://{addr}"), req_rx)
    }

    #[test]
    fn a_get_sends_the_bearer_token_and_parses() {
        let body = r#"{"people":[{"person_id":"P1","engagement_id":"E1","slug":"amy",
            "class":"intern","standing":"active","name":"Amy","documents":2}],
            "active":1,"separated":0,"erased":0}"#;
        let (base, requests) = mock_server(200, body);
        let client = ApiClient::new(&base, "tok-123".to_owned()).expect("client");
        let roster = client.roster(Some("active")).expect("roster");
        assert_eq!(roster.people.len(), 1);
        assert_eq!(roster.active, 1);
        let raw = requests.recv().expect("request captured");
        assert!(raw.contains("GET /api/roster?standing=active"), "{raw}");
        assert!(raw.contains("authorization: Bearer tok-123"), "{raw}");
    }

    #[test]
    fn a_401_hints_at_login() {
        let (base, _r) = mock_server(401, r#"{"error":"nope"}"#);
        let client = ApiClient::new(&base, "stale".to_owned()).expect("client");
        let err = client.check().expect_err("401");
        assert!(format!("{err}").contains("docket login"), "{err}");
    }

    #[test]
    fn a_server_error_surfaces_the_error_field() {
        let (base, _r) = mock_server(500, r#"{"error":"ledger names class X"}"#);
        let client = ApiClient::new(&base, "tok".to_owned()).expect("client");
        let err = client.retention().expect_err("500");
        assert!(format!("{err}").contains("ledger names class X"), "{err}");
    }

    #[test]
    fn a_post_returns_wrote_on_success() {
        let (base, _r) = mock_server(200, r#"{"did":"advanced amy from a to b","stage":"b"}"#);
        let client = ApiClient::new(&base, "tok".to_owned()).expect("client");
        let outcome = client
            .advance("amy", &AdvanceRequest { force: false })
            .expect("advance");
        assert!(matches!(outcome, PostOutcome::Wrote(_)), "{outcome:?}");
        if let PostOutcome::Wrote(w) = outcome {
            assert_eq!(w.stage.as_deref(), Some("b"));
        }
    }

    #[test]
    fn a_post_returns_refused_on_409() {
        let body = r#"{"attempted":"advance amy","blockers":[{"code":"NDA","title":"NDA",
            "reason":"not signed"}],"refusals":[]}"#;
        let (base, _r) = mock_server(409, body);
        let client = ApiClient::new(&base, "tok".to_owned()).expect("client");
        let outcome = client
            .advance("amy", &AdvanceRequest { force: false })
            .expect("advance");
        assert!(matches!(outcome, PostOutcome::Refused(_)), "{outcome:?}");
        if let PostOutcome::Refused(r) = outcome {
            assert_eq!(r.blockers.len(), 1);
        }
    }

    #[test]
    fn render_roster_shows_the_tally() {
        let roster = Roster {
            people: vec![RosterLine {
                person_id: "P1".to_owned(),
                engagement_id: "E1".to_owned(),
                slug: "amy".to_owned(),
                class: "intern".to_owned(),
                standing: "active".to_owned(),
                name: Some("Amy".to_owned()),
                documents: 2,
            }],
            active: 1,
            separated: 0,
            erased: 0,
        };
        let text = render_roster(&roster);
        assert!(text.contains("Amy"), "{text}");
        assert!(text.contains("1 active, 0 separated, 0 erased"), "{text}");
    }

    #[test]
    fn render_refused_lists_the_blockers() {
        let refused = Refused {
            attempted: "advance amy".to_owned(),
            blockers: vec![Blocker {
                code: "NDA".to_owned(),
                title: "Non-disclosure agreement".to_owned(),
                reason: "not recorded as signed".to_owned(),
            }],
            refusals: vec![],
        };
        let text = render_refused(&refused);
        assert!(text.contains("refused: advance amy"), "{text}");
        assert!(text.contains("NDA"), "{text}");
    }
}
