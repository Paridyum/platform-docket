//! Producing a document, allocating its serial and recording it.

use crate::adapters::{clock, registry::Registry, store, typeset};
use crate::config::Paths;
use crate::domain::definitions::Definitions;
use crate::domain::escape;
use crate::domain::lifecycle::Document;
use crate::domain::person::{Person, blockers};
use crate::domain::register::{Entry, sha256};
use crate::domain::serial::{self, Serial, next_seq};
use anyhow::{Context, Result, bail};
use std::fs;

/// Build the definitions a document is rendered against.
///
/// Order matters: organisation facts first, then signatories, then the
/// person, then the document's own date and reference. Later values win,
/// which is what lets a person record override an organisational default.
pub fn definitions(
    registry: &Registry,
    person: Option<&Person>,
    reference: Option<&Serial>,
    date: time::Date,
) -> Definitions {
    let mut defs = Definitions::new();

    if let Some(org) = registry.org_node() {
        defs.absorb("dkOrg", org);
    }
    // The class reads these short names for the letterhead.
    for (from, to) in [
        ("dkOrgName", "dkOrgName"),
        ("dkOrgTagline", "dkTagline"),
        ("dkOrgAddress", "dkAddress"),
        ("dkOrgEmail", "dkEmail"),
        ("dkOrgWeb", "dkWeb"),
        ("dkOrgLegalStatus", "dkLegalStatus"),
    ] {
        defs.alias(from, to);
    }

    for s in registry.active_signatories() {
        let id = escape::command_name(&s.id);
        defs.set(format!("dkSig{id}Name"), &s.legal_name);
        defs.set(format!("dkSig{id}Designation"), &s.designation);
        defs.set(
            format!("dkSig{id}Short"),
            s.short.as_deref().unwrap_or(&s.id),
        );
    }

    if let Some(p) = person {
        defs.absorb("dkP", &p.value);
        // Dates print in long form on the document, not as ISO.
        for (field, name) in [
            ("engagement.start_date", "dkPEngagementStartDate"),
            ("engagement.end_date", "dkPEngagementEndDate"),
            ("identity.date_of_birth", "dkPIdentityDateOfBirth"),
        ] {
            if let Some(raw) = p.string(field)
                && let Ok(d) = clock::parse_iso(raw)
            {
                defs.set(name, &clock::to_long(d));
            }
        }
        // Pronouns live at `identity.pronouns` in the record, which flattens
        // to \dkPIdentityPronouns*. The templates read the shorter
        // \dkPPronouns*, so alias across BEFORE defaulting -- without this
        // the default silently wins and a person is misgendered on a
        // document they keep.
        for part in ["Subject", "Object", "Possessive"] {
            defs.alias(
                &format!("dkPIdentityPronouns{part}"),
                &format!("dkPPronouns{part}"),
            );
        }
        // Never inferred from a name; a wrong guess is worse than the neutral
        // default, so the default is neutral.
        defs.set_default("dkPPronounsSubject", "they");
        defs.set_default("dkPPronounsObject", "them");
        defs.set_default("dkPPronounsPossessive", "their");
    }

    defs.set("dkDocDate", &clock::to_long(date));
    defs.set("dkDocRef", reference.map_or("NOT ISSUED", Serial::as_str));
    // Overwritten with the real code at issuance. A draft must not carry one
    // that looks checkable.
    defs.set("dkDocVerify", "NOT ISSUED");
    defs
}

/// Add what the register and the event log already know about this person.
///
/// The offboarding checklist used to present a generic list of systems to
/// tick, while the event log knew exactly which ones were still granted.
/// Printing the derived answer turns the checklist from a reminder into
/// evidence, and it cannot disagree with the log because it comes from it.
///
/// # Errors
/// Fails if the register or event log cannot be read.
pub fn add_history(defs: &mut Definitions, paths: &Paths, person: &Person) -> Result<()> {
    let issued = store::register(paths)?;
    let who = crate::application::personnel::identify(paths, &person.slug).ok();
    let mine: Vec<&Entry> = issued
        .iter()
        .filter(|e| who.as_ref().is_some_and(|w| e.person_id == w.person_id))
        .collect();
    let mut docs = String::new();
    for e in &mine {
        docs.push_str("\\item ");
        docs.push_str(&escape::tex(&e.code));
        docs.push_str(" --- ");
        docs.push_str(&escape::tex(&e.title));
        docs.push_str(" \\hfill \\texttt{");
        docs.push_str(&escape::tex(&e.serial));
        docs.push('}');
    }
    defs.set_raw(
        "dkPDocsIssuedItems",
        if docs.is_empty() {
            "\\item ~".to_owned()
        } else {
            docs
        },
    );
    defs.set_raw("dkPDocsIssuedCount", mine.len().to_string());

    // A record with no identifier simply has no recorded history, which is
    // not an error at render time.
    let open = match &who {
        Some(record) => {
            let log = store::events(paths)?;
            crate::domain::event::open_access(&log, &record.person_id)
        }
        None => Vec::new(),
    };
    let mut items = String::new();
    for a in &open {
        items.push_str("\\item ");
        items.push_str(&escape::tex(a));
    }
    defs.set_raw(
        "dkPAccessOpenItems",
        if items.is_empty() {
            "\\item \\textit{nothing recorded as still granted}".to_owned()
        } else {
            items
        },
    );
    defs.set_raw("dkPAccessOpenCount", open.len().to_string());
    Ok(())
}

/// Issue one document: render it, keep the PDF, and seal a register entry.
///
/// # Errors
/// Fails if the template is missing, the serial format is unset,
/// typesetting fails, or the register cannot be written.
pub fn one(
    paths: &Paths,
    registry: &Registry,
    person: &Person,
    document: &Document,
    date: time::Date,
    by: &str,
) -> Result<Entry> {
    one_with(paths, registry, person, document, date, None, by)
}

/// Issue one document, with extra definitions only that document needs.
///
/// The pack cover sheet is the reason this exists: it lists the contents of the
/// pack, which cannot be known from the registry alone. Passing those in beats
/// either a special case inside `definitions` or a second render path.
///
/// # Errors
/// As `one`.
pub fn one_with(
    paths: &Paths,
    registry: &Registry,
    person: &Person,
    document: &Document,
    date: time::Date,
    extra: Option<&Definitions>,
    by: &str,
) -> Result<Entry> {
    // The register keys on identifiers, so a document cannot be issued for a
    // record that has none. That is not a limitation: a document with no
    // identified recipient is exactly what the register exists to prevent.
    let who = crate::application::personnel::identify(paths, &person.slug)?;
    let template = paths.root.join(&document.template);
    if !template.exists() {
        bail!(
            "{}: template not found at {}",
            document.code,
            template.display()
        );
    }

    let format = registry
        .serial_format()
        .context("org.reference.format is unset; a serial cannot be allocated")?;
    let year = date.year();
    let existing = store::register(paths)?;
    let used: Vec<u32> = existing
        .iter()
        .filter(|e| e.code == document.code && e.year == year)
        .map(|e| e.seq)
        .collect();
    let seq = next_seq(&used);
    let serial = Serial::render(format, &document.code, year, seq)?;

    // Read the template first: its hash feeds the verification code, which
    // must be printed ON the document and therefore known before rendering.
    let template_bytes =
        fs::read(&template).with_context(|| format!("reading {}", template.display()))?;
    let template_hash = sha256(&template_bytes);
    let issued_on = clock::to_iso(date)?;

    // Recorded on the entry: a durable artifact says what produced it.
    let engine = typeset::detect()?;
    let engine_version = engine.version();

    let work = tempfile::tempdir().context("creating a working directory")?;
    let mut defs = definitions(registry, Some(person), Some(&serial), date);
    defs.set(
        "dkDocVerify",
        &serial::verification_code(serial.as_str(), &person.slug, &issued_on, &template_hash),
    );
    add_history(&mut defs, paths, person)?;
    if let Some(extra) = extra {
        defs.merge(extra);
    }
    // Identify the file itself, not just its contents.
    defs.set("dkDocTitle", &document.title);
    defs.set(
        "dkDocSubject",
        &format!(
            "{} issued to {} under engagement {} on {}",
            serial, who.person_id, who.engagement_id, issued_on
        ),
    );
    defs.set(
        "dkDocKeywords",
        &format!(
            "Paridyum; {}; {}; {}; {}",
            document.code, serial, who.person_id, who.engagement_id
        ),
    );
    let meta = document_metadata(document, &serial, &who, &issued_on);
    let pdf = typeset::render(
        paths,
        &typeset::Job {
            template: &template,
            out_dir: work.path(),
            definitions: &defs,
            date,
            draft: false,
            metadata: meta,
        },
    )?;

    let bytes = fs::read(&pdf).with_context(|| format!("reading {}", pdf.display()))?;
    let file_name = format!("{}-{}.pdf", document.code, serial.file_stem());
    // Named by the PERSON identifier, not the slug: this directory becomes a
    // cloud folder, and a folder named after a person discloses who Paridyum
    // engages to anyone who can read the listing (ADR-0051).
    let stored = store::keep_pdf(paths, who.person_id.as_str(), &file_name, &pdf)?;

    let entry = register_entry(
        &RegisterEntry {
            serial: &serial,
            seq,
            year,
            person: &who,
            document,
            issued_on: &issued_on,
            template_hash: &template_hash,
            pdf_sha256: &sha256(&bytes),
            stored: &stored,
            engine: engine.as_str(),
            engine_version: &engine_version,
        },
        existing.last(),
    )?;

    store::append(paths, &entry)?;

    // The log carried access and separation but never issuance, so a person's
    // history was missing the documents themselves.
    crate::application::personnel::record_event(
        paths,
        &who.person_id,
        crate::domain::event::Kind::DocumentIssued,
        Some(&format!("{} {serial}", document.code)),
        by,
        date,
    )?;
    Ok(entry)
}

/// The facts a register entry is built from.
///
/// A struct rather than nine positional arguments: `clippy.toml` caps them at
/// eight, and at nine the call site stops being readable anyway.
struct RegisterEntry<'a> {
    serial: &'a Serial,
    seq: u32,
    year: i32,
    person: &'a crate::domain::identity::Record,
    document: &'a Document,
    issued_on: &'a str,
    template_hash: &'a str,
    pdf_sha256: &'a str,
    stored: &'a str,
    engine: &'a str,
    engine_version: &'a str,
}

/// Build and seal the register entry for one issuance.
fn register_entry(f: &RegisterEntry<'_>, previous: Option<&Entry>) -> Result<Entry> {
    Entry {
        serial: f.serial.to_string(),
        code: f.document.code.clone(),
        seq: f.seq,
        year: f.year,
        person_id: f.person.person_id.clone(),
        engagement_id: f.person.engagement_id.clone(),
        slug: f.person.slug.clone(),
        document: f.document.id.clone(),
        title: f.document.title.clone(),
        kind: f.document.kind.as_str().to_owned(),
        issued_on: f.issued_on.to_owned(),
        retention: f.document.retention.clone(),
        template_sha256: f.template_hash.to_owned(),
        pdf_sha256: f.pdf_sha256.to_owned(),
        pdf_path: f.stored.to_owned(),
        texlive: typeset::TEXLIVE.to_owned(),
        engine: Some(f.engine.to_owned()),
        engine_version: Some(f.engine_version.to_owned()),
        prev_sha256: String::new(),
        entry_sha256: String::new(),
    }
    .seal(previous)
}

/// What the PDF says about itself, so a file out of the store is not anonymous.
fn document_metadata(
    document: &Document,
    serial: &Serial,
    who: &crate::domain::identity::Record,
    issued_on: &str,
) -> typeset::Metadata {
    typeset::Metadata {
        title: document.title.clone(),
        author: "Paridyum".to_owned(),
        subject: format!(
            "{serial} issued to {} under engagement {} on {issued_on}",
            who.person_id, who.engagement_id
        ),
        keywords: format!(
            "Paridyum; {}; {serial}; {}; {}",
            document.code, who.person_id, who.engagement_id
        ),
    }
}

/// Everything the person's current stage owes that is not already issued.
///
/// # Errors
/// Fails if the lifecycle declares a duplicate code or unknown stage.
pub fn outstanding(
    registry: &Registry,
    person: &Person,
    who: &crate::domain::identity::Record,
    issued: &[Entry],
) -> Result<Vec<String>> {
    let by_code = registry.lifecycle.by_code()?;
    let already: Vec<&str> = issued
        .iter()
        .filter(|e| e.person_id == who.person_id)
        .map(|e| e.code.as_str())
        .collect();
    let mut out = Vec::new();
    for code in registry.lifecycle.owed_upto(person.stage())? {
        let Some(doc) = by_code.get(code.as_str()) else {
            continue;
        };
        if doc.required_if.is_some_and(|c| !person.satisfies(c)) {
            continue;
        }
        if !already.contains(&code.as_str()) {
            out.push(code);
        }
    }
    Ok(out)
}

/// Fields a document needs that this record does not supply.
///
/// Renders the definitions the document WOULD get, scans the template for
/// commands it dereferences without a fallback, and reports the difference.
///
/// Without this the operator finds out from `Undefined control sequence` at
/// issuance time, which names a TeX command rather than a field and gives no
/// clue which record to edit. The class's `requires` list is not enough on its
/// own: it says what the class needs to be lawful, not what every template
/// happens to print.
///
/// # Errors
/// Fails if a template cannot be read.
pub fn unsupplied_fields(
    paths: &Paths,
    registry: &Registry,
    person: &Person,
    document: &Document,
) -> Result<Vec<String>> {
    let template = paths.root.join(&document.template);
    if !template.exists() {
        return Ok(Vec::new());
    }
    let body =
        fs::read_to_string(&template).with_context(|| format!("reading {}", template.display()))?;

    // The date is irrelevant to which commands get defined, so any will do.
    let defs = definitions(
        registry,
        Some(person),
        None,
        clock::parse_iso("2000-01-01")?,
    );
    Ok(crate::domain::template::required_commands(&body)
        .into_iter()
        .filter(|c| defs.get(c).is_none())
        .collect())
}

/// Every document owed at this stage that the record cannot yet render.
///
/// # Errors
/// Fails if the registry or a template cannot be read.
pub fn unrenderable(
    paths: &Paths,
    registry: &Registry,
    person: &Person,
    who: &crate::domain::identity::Record,
    register: &[Entry],
) -> Result<Vec<(String, Vec<String>)>> {
    let by_code = registry.lifecycle.by_code()?;
    let mut out = Vec::new();
    for code in outstanding(registry, person, who, register)? {
        let Some(doc) = by_code.get(code.as_str()) else {
            continue;
        };
        let missing = unsupplied_fields(paths, registry, person, doc)?;
        if !missing.is_empty() {
            out.push((code, missing));
        }
    }
    Ok(out)
}

/// Refuse to issue where the document would be defective. Never a warning.
pub fn refusals(registry: &Registry, person: &Person) -> Vec<String> {
    blockers(person, registry.forum())
        .into_iter()
        .map(|b| b.0)
        .collect()
}

/// Refusals that `--force` may not bypass.
///
/// `--force` exists because most refusals are judgement calls an operator may
/// legitimately overrule — a forum not yet chosen, a certificate area left thin —
/// and the documents are merely weaker for it.
///
/// A record still holding the template's placeholder is not that. Issuing from
/// one mints real serials, writes them to the register and seals them into the
/// chain, on documents naming nobody; and an issued document cannot be edited, so
/// the only correction is re-issuing under new serials. There is no situation in
/// which an operator means to do that, so the flag does not reach it.
#[must_use]
pub fn unforceable(person: &Person) -> Vec<String> {
    let unfilled = crate::domain::person::unfilled(person);
    if unfilled.is_empty() {
        return Vec::new();
    }
    vec![format!(
        "{} field(s) still hold the template's {} placeholder: {}",
        unfilled.len(),
        crate::domain::person::UNFILLED,
        unfilled.join(", ")
    )]
}
