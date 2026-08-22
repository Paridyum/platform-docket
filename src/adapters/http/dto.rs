//! The wire shapes. What the console receives, stated once and explicitly.
//!
//! These exist rather than `#[derive(Serialize)]` on the `application::` types
//! deliberately. Those types are internal, and a field renamed there would
//! silently rename a field in a published API — the sort of change that compiles,
//! passes every test in this repository, and breaks a client nobody ran.
//!
//! The cost is a mapping function per type. The mapping is where a wire contract
//! is either honoured or visibly changed, so paying it is the point.

use crate::application::{check, comms, next, personnel, roster};
use crate::domain::identity::Standing;
use serde::Serialize;

/// One roster row.
#[derive(Debug, Serialize)]
pub struct RosterLine {
    /// Permanent, opaque person identifier. Names the archive folder.
    pub person_id: String,
    /// Identifier of the current engagement. Carries class and period.
    pub engagement_id: String,
    /// Filename the record is kept under, while it exists. Never an identity.
    pub slug: String,
    /// Person class of this engagement.
    pub class: String,
    /// Current standing, as the ledger records it.
    pub standing: String,
    /// Name, where the record still exists.
    pub name: Option<String>,
    /// Role, where the record still exists.
    pub role: Option<String>,
    /// Engagement start, where recorded.
    pub start_date: Option<String>,
    /// Engagement end, where recorded.
    pub end_date: Option<String>,
    /// Documents issued to this person.
    pub documents: usize,
    /// True once the record is gone and only the identifier remains.
    ///
    /// Derived here rather than left for the console to infer from
    /// `slug == "(erased)"`. A client comparing against a sentinel string is a
    /// client that breaks when the sentinel changes, and an erased person
    /// displayed as though their record were merely empty would be a serious
    /// misreading of a DPDP erasure.
    pub erased: bool,
}

impl From<&roster::Line> for RosterLine {
    fn from(l: &roster::Line) -> Self {
        Self {
            person_id: l.person_id.clone(),
            engagement_id: l.engagement_id.clone(),
            slug: l.slug.clone(),
            class: l.class.clone(),
            standing: l.standing.as_str().to_owned(),
            name: l.name.clone(),
            role: l.role.clone(),
            start_date: l.start_date.clone(),
            end_date: l.end_date.clone(),
            documents: l.documents,
            erased: matches!(l.standing, Standing::Erased),
        }
    }
}

/// The roster, with the tallies the CLI also prints.
#[derive(Debug, Serialize)]
pub struct Roster {
    /// One row per engagement, from the ledger.
    pub people: Vec<RosterLine>,
    /// Active engagements across the whole roster, never the filtered view.
    pub active: usize,
    /// Separated engagements across the whole roster.
    pub separated: usize,
    /// Erased engagements across the whole roster.
    pub erased: usize,
}

/// One outstanding step, in workflow order.
#[derive(Debug, Serialize)]
pub struct Step {
    /// What it is, in a sentence.
    pub what: String,
    /// Why it is next.
    pub why: String,
    /// The equivalent CLI command, where one exists.
    ///
    /// Carried to the console on purpose: it is what an operator falls back to
    /// when a screen is missing or wrong, and it keeps the two interfaces visibly
    /// the same workflow rather than two workflows that happen to agree.
    pub command: Option<String>,
}

impl From<&next::Step> for Step {
    fn from(s: &next::Step) -> Self {
        Self {
            what: s.what.clone(),
            why: s.why.clone(),
            command: s.command.clone(),
        }
    }
}

/// One check finding.
///
/// `blocking` is carried as the boolean the domain uses rather than flattened
/// into a severity string. A client that has to compare against `"error"` or
/// `"warning"` is a client that silently downgrades a blocking finding the day
/// that spelling changes.
#[derive(Debug, Serialize)]
pub struct Finding {
    /// Whether this must block, or merely be reported.
    pub blocking: bool,
    /// What is wrong, in one line.
    pub message: String,
}

impl From<&check::Finding> for Finding {
    fn from(f: &check::Finding) -> Self {
        Self {
            blocking: f.blocking,
            message: f.message.clone(),
        }
    }
}

/// Everything `check` looked at, and everything it found.
#[derive(Debug, Serialize)]
pub struct Check {
    /// Problems, blocking first.
    pub findings: Vec<Finding>,
    /// How many findings block.
    pub errors: usize,
    /// Documents issued to this person.
    pub documents: usize,
    /// Stages declared.
    pub stages: usize,
    /// Entries in the register.
    pub entries: usize,
}

impl From<&check::Findings> for Check {
    fn from(f: &check::Findings) -> Self {
        Self {
            findings: f.items.iter().map(Finding::from).collect(),
            errors: f.errors(),
            documents: f.documents,
            stages: f.stages,
            entries: f.entries,
        }
    }
}

/// One message that is due, rendered ready to copy.
///
/// Rendered, never sent. D-27 and §8 of the design: this tool does not send mail,
/// and a console button labelled "send" would be a lie about what happened.
#[derive(Debug, Serialize)]
pub struct Message {
    /// Which message this is.
    pub id: String,
    /// Stage it belongs to.
    pub stage: String,
    /// Who it goes to.
    pub to: String,
    /// The address, where the record holds one.
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
    ///
    /// Non-empty means this message must not go out yet. Kept separate from
    /// `attachments` so the console cannot render a missing one as present.
    pub missing: Vec<String>,
}

/// One attachment a message names.
#[derive(Debug, Serialize)]
pub struct Attachment {
    /// Document code.
    pub code: String,
    /// Document title, for the message.
    pub title: String,
    /// Serial, or a note that it has not been issued.
    pub serial: String,
}

impl From<&comms::Rendered> for Message {
    fn from(m: &comms::Rendered) -> Self {
        Self {
            id: m.id.clone(),
            stage: m.stage.clone(),
            to: m.to.as_str().to_owned(),
            address: m.address.clone(),
            when: m.when.clone(),
            subject: m.subject.clone(),
            body: m.body.clone(),
            attachments: m
                .attachments
                .iter()
                .map(|(code, title, serial)| Attachment {
                    code: code.clone(),
                    title: title.clone(),
                    serial: serial.clone(),
                })
                .collect(),
            missing: m.missing.clone(),
        }
    }
}

/// A person's page: who they are, what they hold, and what they still owe.
//
// `person_id` repeats the type name, which clippy::struct_field_names flags. Kept:
// it is the domain's name for the identifier, in `identity::Record` and in every
// other wire type here. Renaming it to `id` for one struct would make the console
// read `person_id` on a roster row and `id` on a person, which is a worse API than
// a repeated word.
#[allow(clippy::struct_field_names)]
#[derive(Debug, Serialize)]
pub struct Person {
    /// Permanent, opaque person identifier. Names the archive folder.
    pub person_id: String,
    /// Identifier of the current engagement. Carries class and period.
    pub engagement_id: String,
    /// Filename the record is kept under, while it exists. Never an identity.
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
    /// What to do next, most urgent first. The ordering IS the workflow.
    pub steps: Vec<Step>,
    /// Why this person may not leave their current stage yet.
    pub blockers: Vec<Blocker>,
}

/// One issued document.
#[derive(Debug, Serialize)]
pub struct Document {
    /// Document code.
    pub code: String,
    /// Serial of the document.
    pub serial: String,
    /// ISO date it was issued.
    pub issued_on: String,
}

/// One event from the person's history.
#[derive(Debug, Serialize)]
pub struct HistoryEntry {
    /// When it happened.
    pub at: String,
    /// What kind of event it was.
    pub kind: String,
    /// Free text the event carried, where it carried any.
    pub detail: Option<String>,
}

/// Why a person may not leave their current stage yet.
///
/// Every entry is an unmet requirement — `application::personnel::unmet_gates`
/// returns only the ones that block. There is deliberately no `met` field to
/// invert: §8 of the design forbids an interface that renders these as advisory,
/// and the signature gate exists to stop access being granted before the NDA is
/// back. A non-empty `blockers` means the action is refused, not discouraged.
#[derive(Debug, Serialize)]
pub struct Blocker {
    /// Document code the stage requires executed.
    pub code: String,
    /// Document title, for the message.
    pub title: String,
    /// Why it is not satisfied.
    pub reason: String,
}

impl From<&personnel::Gate> for Blocker {
    fn from(g: &personnel::Gate) -> Self {
        Self {
            code: g.code.clone(),
            title: g.title.clone(),
            reason: g.reason.clone(),
        }
    }
}

/// One person who may operate this tool.
///
/// Sent so the console can offer a chooser rather than asking an operator to type
/// their own id. Inactive operators are not sent at all: a name that cannot be
/// used is a name someone will pick and then be refused for.
#[derive(Debug, Serialize)]
pub struct Operator {
    /// Short id, sent back as `X-Docket-Operator` on every write.
    pub id: String,
    /// Name recorded in the log.
    pub name: String,
    /// Their role, for the report.
    pub role: String,
}

impl From<&crate::adapters::registry::Operator> for Operator {
    fn from(o: &crate::adapters::registry::Operator) -> Self {
        Self {
            id: o.id.clone(),
            name: o.name.clone(),
            role: o.role.clone(),
        }
    }
}

/// Documents past their retention period, and the ones that could not be judged.
///
/// `unevaluated` is not an empty-list-shaped afterthought. `retention.yaml`
/// states periods in prose because the basis matters more than the arithmetic, so
/// a period this tool cannot parse is reported as unevaluated rather than assumed
/// compliant — silently treating "deleted within 30 days" as satisfied is how
/// over-retention becomes invisible. A console that renders only `overdue` would
/// undo that.
#[derive(Debug, Serialize)]
pub struct Retention {
    /// Documents past their retention period.
    pub overdue: Vec<Overdue>,
    /// Retention periods this tool could not parse, and therefore did not judge.
    ///
    /// Reported rather than assumed compliant. Never render this as an empty
    /// state: a period like "deleted within 30 days" that silently counted as
    /// satisfied is how over-retention becomes invisible.
    pub unevaluated: Vec<String>,
}

/// One document past its retention period.
#[derive(Debug, Serialize)]
pub struct Overdue {
    /// Serial of the document.
    pub serial: String,
    /// Permanent, opaque person identifier. Names the archive folder.
    pub person_id: String,
    /// Person class of this engagement.
    pub class: String,
    /// How long that class keeps it, in prose, as `retention.yaml` states it.
    pub keep_for: String,
    /// Which date the clock ran from — issue, engagement end, or never.
    pub basis: String,
    /// The date the clock started from.
    pub since: String,
    /// Days elapsed since then.
    pub days: i64,
}

// ---------------------------------------------------------------------------
// Writes. Request bodies in, results out.
// ---------------------------------------------------------------------------

use serde::Deserialize;
use std::collections::BTreeMap;

/// `POST /api/person` — open a record.
#[derive(Debug, Deserialize)]
pub struct EnrolRequest {
    /// Filename the record is filed under. Never an identity.
    pub slug: String,
    /// Person class. Decides what may be collected at all.
    pub class: String,
    /// Answers, keyed by the dotted path they are written to.
    ///
    /// `BTreeMap` rather than a struct because the questions are generated from
    /// `registry/fields.yaml` — a typed struct here would have to be edited every
    /// time a field is declared, and would silently drop answers to fields it did
    /// not know about.
    #[serde(default)]
    pub answers: BTreeMap<String, String>,
}

/// `POST /api/person/{slug}/sign` — record a document coming back executed.
#[derive(Debug, Deserialize)]
pub struct SignRequest {
    /// Serial of the document, e.g. `PAR/NDA/2026/0001`.
    pub serial: String,
    /// How it was signed: `wet-ink` | `aadhaar-esign` | `electronic`.
    pub method: String,
    /// ISO date the person signed.
    pub signed_on: String,
    /// ISO date Paridyum received it. Defaults to the signing date.
    pub received_on: Option<String>,
    /// Path to the scan. Required for wet-ink.
    pub scan: Option<String>,
}

/// `POST /api/person/{slug}/advance` — move to the next lifecycle stage.
#[derive(Debug, Deserialize)]
pub struct AdvanceRequest {
    /// Advance despite unmet gates.
    ///
    /// Present because the CLI has it, and absent from the console's UI on
    /// purpose: §8 forbids an interface that offers a way past the signature
    /// gate. A caller that sets this is doing something deliberate at a level
    /// below the screens.
    #[serde(default)]
    pub force: bool,
}

/// What a write did.
#[derive(Debug, Serialize)]
pub struct Wrote {
    /// Plain sentence naming what changed, for an operator to read.
    pub did: String,
    /// Documents issued by this call, if any.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub issued: Vec<Document>,
    /// Where the person now is, if the stage changed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
}

/// A write that was refused, and why.
///
/// Distinct from an error: nothing went wrong, the domain declined. The blockers
/// are the same shape the person page renders, so a refused write and a held
/// person read identically in the console.
#[derive(Debug, Serialize)]
pub struct Refused {
    /// What was attempted.
    pub attempted: String,
    /// Why it may not happen yet.
    pub blockers: Vec<Blocker>,
    /// Refusals that are not stage gates — an unfilled record, a missing forum.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub refusals: Vec<String>,
}

/// The version of the wire contract this build speaks.
///
/// ## Why a number separate from the package version
///
/// `CARGO_PKG_VERSION` moves when anything ships. This moves only when the shape
/// of a response changes in a way that an existing `console-docket` would read
/// wrong or fail to read at all — so a console can decide, from one cheap request,
/// whether it is talking to a build it understands.
///
/// ## The failure it exists to end
///
/// A `docket serve` left running from before a change served the old shape
/// indefinitely. The console refused the response, correctly, and the operator got
/// a redacted Server Component error: a React error code and a digest. The cause
/// was a binary nobody had rebuilt, and nothing in the system could say so — the
/// diagnosis was "almost always this means", which is a guess printed confidently.
///
/// ## When to change it
///
/// **Bump on any change that removes a field, renames one, changes a field's type,
/// or adds a field an existing console would require.** Adding a field that older
/// consoles ignore does not need a bump; adding one they must have does, because
/// `console-docket` validates the shape it reads and refuses what is missing.
///
/// Do not reuse a number and do not go backwards. It is not semver — there is no
/// meaning in the gap between 2 and 3 beyond "these are different".
///
/// **There is deliberately no test asserting this constant's value.** One was
/// written — `assert!(CONTRACT >= 2)` — and clippy correctly rejected it as an
/// assertion with a constant value: it is folded at compile time and can never
/// fail at runtime, so it would have been a test that verifies nothing while
/// appearing to guard the rule. The rule lives here, in the history below, and in
/// review. A real guard would have to compare this against the value on the
/// previous commit, which is a CI check and not a unit test; it has not been
/// written because lowering this number is not a mistake anyone makes by accident.
///
/// ## History
///
/// - **1** — the shape as first served.
/// - **2** — `Question` gained `choices` and `default`. This is the change whose
///   absence produced the incident above.
pub const CONTRACT: u32 = 2;

/// One question the enrolment form must ask.
#[derive(Debug, Serialize)]
pub struct Question {
    /// Dotted path the answer is written to.
    pub path: String,
    /// Field group, used to order and head the questions.
    pub group: String,
    /// The declared purpose, shown as help.
    ///
    /// Always sent. If a field's purpose cannot be written down it should not
    /// exist, so there is never a reason to hide it from the person being asked.
    pub purpose: String,
    /// Whether the class requires an answer.
    pub required: bool,
    /// Every value this field accepts, or empty for free text.
    ///
    /// Sent so a console can offer the same closed list the CLI interview offers.
    /// It is empty for most fields and that is not an omission: a name, an
    /// institution and an address have no enumerable set, and a client that
    /// invented one would stop an operator recording the truth.
    ///
    /// A client MUST NOT infer a set from a field's name. `engagement.start_date`
    /// looks like a date and this is deliberately silent about that, because
    /// whether "on joining" is a valid answer belongs here and not to a browser.
    pub choices: Vec<String>,
    /// A safe starting value, where the registry declares one.
    ///
    /// The same default the interview offers on a blank answer. A client may
    /// prefill it; it is not a value the operator has agreed to, so a client that
    /// prefills MUST let it be changed and cleared.
    pub default: Option<String>,
}

impl From<&crate::domain::personnel::Question> for Question {
    fn from(q: &crate::domain::personnel::Question) -> Self {
        Self {
            path: q.path.clone(),
            group: q.group.clone(),
            purpose: q.purpose.clone(),
            required: q.required,
            choices: q.choices.clone(),
            default: q.default.clone(),
        }
    }
}

/// The enrolment form for a class, generated rather than written.
#[derive(Debug, Serialize)]
pub struct Form {
    /// The class these questions belong to.
    pub class: String,
    /// Questions in catalogue order, grouped by `group`.
    pub questions: Vec<Question>,
}
