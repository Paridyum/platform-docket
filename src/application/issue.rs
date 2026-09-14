//! Producing a document, allocating its serial and recording it.

use crate::adapters::store::{self, Store};
use crate::adapters::{clock, registry::Registry, typeset};
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
    definitions_for(registry, person, reference, date, None)
}

/// As `definitions`, resolving the signature block for one document.
///
/// WHO SIGNS DEPENDS ON WHO THE DOCUMENT IS ABOUT, which is why this cannot live in the
/// template. `\dkSignerOne*` and `\dkSignerTwo*` are emitted from the document's
/// `signed_by`, narrowed by `signed_by_class` — so `\paridyumsignblock` renders the
/// CEO/CTO pair for an intern and the Director alone for an officer, from one template.
#[must_use]
pub fn definitions_for(
    registry: &Registry,
    person: Option<&Person>,
    reference: Option<&Serial>,
    date: time::Date,
    document: Option<&Document>,
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

    if let Some(doc) = document {
        let class = person.map_or("intern", Person::class);
        let signers = doc.signatories_for(class);
        for (n, id) in ["One", "Two"].iter().zip(signers.iter()) {
            if let Some(s) = registry.active_signatories().find(|s| &s.id == id) {
                defs.set(format!("dkSigner{n}Name"), &s.legal_name);
                defs.set(format!("dkSigner{n}Designation"), &s.designation);
            }
        }
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
        defs.absorb(crate::domain::template::PERSON_PREFIX, &p.value);
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
pub fn add_history(defs: &mut Definitions, store: &dyn Store, person: &Person) -> Result<()> {
    let issued = store.register()?;
    let who = crate::application::personnel::identify(store, &person.slug).ok();
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
            let log = store.events()?;
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

/// Refuse a document that is going to the wrong person.
///
/// TWO REFUSALS, BOTH ABOUT A DOCUMENT THAT WOULD BE WRONG RATHER THAN ABSENT — which is
/// why they are here and not in `check`. A document reaches this function with a serial
/// about to be allocated and a register entry about to be sealed; refusing afterwards is
/// not possible, because the register is append-only by design.
///
/// # Errors
/// Fails if the document is not declared for this person's class, or if the person would
/// be signing it for Paridyum.
/// Who signs this document for Paridyum, with the subject substituted out.
///
/// AUTOMATIC SUBSTITUTION, DETERMINISTIC, AND RECORDED. Officer documents route to the
/// Director (RFC-0042 decision 4), and the Director's own therefore route to himself —
/// a document signed by its own subject is evidence of nothing. Rather than refuse it,
/// the routed signatory is replaced by **the first active signatory in registry order
/// who is not the subject**.
///
/// Registry order rather than any cleverness, because the substitute has to be the same
/// person every time this is asked: who signed a legal document must not depend on a
/// hash order or on how many documents came before it. And the result is written to the
/// register entry's `signed_by`, because after substitution the signatory is no longer
/// derivable from `lifecycle.yaml` and the paper is the only other place it appears.
///
/// Returns an empty list only where no eligible signatory exists at all, which the
/// caller refuses on — one signatory who is also the subject is a real dead end.
pub fn resolve_signatories(
    registry: &Registry,
    person: &Person,
    document: &Document,
    company: &str,
) -> Vec<String> {
    let routed = document.signatories_for(person.class());
    let subject = person
        .string("identity.legal_name")
        .map(|n| n.trim().to_owned());

    let is_subject = |id: &String| {
        subject.as_ref().is_some_and(|name| {
            registry
                .active_signatories_of(company)
                .any(|s| &s.id == id && s.legal_name.eq_ignore_ascii_case(name))
        })
    };

    // Whether this id still names someone who may sign.
    //
    // ROUTING NAMES AN OFFICE; ONLY THE REGISTER KNOWS WHO HOLDS IT. A document
    // routed to `director` was printed with the Director's name whatever the
    // register said, because the loop below only ever asked "is this the subject
    // of the document". So an officer marked `active: false` -- RFC-0042 §5's
    // way of recording that somebody ceased to hold office -- went on signing
    // new documents indefinitely, and `check` reported nothing, because
    // `signed_by` was validated against every signatory rather than the active
    // ones. Found by simulating a removal rather than by reading the code.
    let is_vacant = |id: &String| !registry.active_signatories_of(company).any(|s| &s.id == id);

    let mut out: Vec<String> = Vec::new();
    for id in routed {
        if is_subject(id) || is_vacant(id) {
            // The substitute must not itself be the subject, and must not already be
            // signing this document — otherwise a two-signatory document would collapse
            // to one name printed twice.
            //
            // A vacant office substitutes by the same path as a conflicted one:
            // the next active signatory of this company, in registry order. If
            // the list is exhausted nothing is pushed, and `refuse_if_misrouted`
            // is what turns a document with no signatory into a refusal rather
            // than a document with a blank signature block.
            if let Some(alt) = registry
                .active_signatories_of(company)
                .map(|s| s.id.clone())
                .find(|cand| !is_subject(cand) && !routed.contains(cand) && !out.contains(cand))
            {
                out.push(alt);
            }
        } else if !out.contains(id) {
            out.push(id.clone());
        }
    }
    out
}

/// Refuse a document that is going to the wrong person.
///
/// A DOCUMENT THAT DOES NOT APPLY TO THIS CLASS IS NOT ISSUED, EVER — and it is refused
/// here rather than in `check` because by this point a serial is about to be allocated
/// and a register entry about to be sealed, and the register is append-only by design.
/// Several templates say "Intern" in their body text; issuing a Certificate of
/// Internship to a director produces a document that is FALSE, and a false document
/// carrying a serial is worse than none.
///
/// # Errors
/// Fails if the document is not declared for this person's class, or if no signatory
/// other than the subject exists to sign it.
fn refuse_if_misrouted(person: &Person, document: &Document, signatories: &[String]) -> Result<()> {
    if !document.applies_to(person.class()) {
        bail!(
            "{}: not issued to a person of class {:?}; the registry declares it for {:?}",
            document.code,
            person.class(),
            document.classes
        );
    }

    // Substitution has already run. Reaching here empty means every eligible signatory
    // was the subject, which no ordering can fix.
    if signatories.is_empty() && !document.signatories_for(person.class()).is_empty() {
        bail!(
            "{}: no signatory other than the subject is available; \
             registry/signatories.yaml has nobody else active",
            document.code
        );
    }
    Ok(())
}

/// Issue one document: render it, keep the PDF, and seal a register entry.
///
/// # Errors
/// Fails if the template is missing, the serial format is unset,
/// typesetting fails, or the register cannot be written.
pub fn one(
    store: &dyn Store,
    paths: &Paths,
    registry: &Registry,
    person: &Person,
    document: &Document,
    date: time::Date,
    by: &str,
) -> Result<Entry> {
    one_with(store, paths, registry, person, document, date, None, by)
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
    store: &dyn Store,
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
    let who = crate::application::personnel::identify(store, &person.slug)?;

    // The company comes from the ENGAGEMENT, not from a flag and not from the
    // person record (ADR-0077 D2). A document belongs to the engagement it was
    // issued against, so the signatory has to be an officer of that company.
    let company = who
        .engagement_id
        .company()
        .unwrap_or_else(|| registry.default_company().unwrap_or("PAR"));
    let signatories = resolve_signatories(registry, person, document, company);
    refuse_if_misrouted(person, document, &signatories)?;

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
    let existing = store.register()?;
    let used: Vec<u32> = existing
        .iter()
        .filter(|e| e.code == document.code && e.year == year)
        .map(|e| e.seq)
        .collect();
    let seq = next_seq(&used);
    let serial = Serial::render(format, &document.code, year, seq)?;

    // Read the template first: its hash feeds the verification code, which
    // must be printed ON the document and therefore known before rendering.
    //
    // THE HASH COVERS EVERY FILE THE TEMPLATE IS MADE OF, not just the top one. Shared clauses
    // live in their own files and are pulled in with `\input`; a hash of the top-level file
    // alone would let two documents with different terms record the same hash, and the
    // verification code printed on them would then be a claim that is not true.
    let files = crate::domain::template::closure(&paths.root, &template)?;
    let template_hash = sha256(&crate::domain::template::hashable(&files));
    let issued_on = clock::to_iso(date)?;

    // Recorded on the entry: a durable artifact says what produced it.
    let engine = typeset::detect()?;
    let engine_version = engine.version();

    let work = tempfile::tempdir().context("creating a working directory")?;
    let mut defs = definitions_for(registry, Some(person), Some(&serial), date, Some(document));
    override_signers(&mut defs, registry, &signatories);
    defs.set(
        "dkDocVerify",
        &serial::verification_code(serial.as_str(), &person.slug, &issued_on, &template_hash),
    );
    add_history(&mut defs, store, person)?;
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

    // Build the entry UNSEALED and hand it to the port: the seal lives behind `append_register`,
    // where the Postgres store makes read-tail-seal-insert one atomic, fork-proof transaction and
    // the file store does the same single-writer (design §4). Sealing here and inserting separately
    // is exactly the fork the port exists to prevent.
    let entry = register_entry(&RegisterEntry {
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
        signed_by: &signatories,
    });

    let entry = store.append_register(entry)?;

    // The log carried access and separation but never issuance, so a person's
    // history was missing the documents themselves.
    crate::application::personnel::record_event(
        store,
        &who.person_id,
        crate::domain::event::Kind::DocumentIssued,
        Some(&format!("{} {serial}", document.code)),
        by,
        date,
    )?;
    Ok(entry)
}

/// Overwrite the signature block with the signatories that actually signed.
///
/// `definitions_for` resolves the ROUTED signatories from `lifecycle.yaml`; substitution
/// happens afterwards, so these are set last and later values win. Without this the
/// register would record the substitute and the paper would name the subject — the worse
/// of the two failures, because the paper is what somebody signs.
fn override_signers(defs: &mut Definitions, registry: &Registry, signatories: &[String]) {
    for (n, id) in ["One", "Two"].iter().zip(signatories.iter()) {
        if let Some(sig) = registry.active_signatories().find(|s| &s.id == id) {
            defs.set(format!("dkSigner{n}Name"), &sig.legal_name);
            defs.set(format!("dkSigner{n}Designation"), &sig.designation);
        }
    }
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
    /// Resolved AFTER substitution, so the entry records who actually signed.
    signed_by: &'a [String],
}

/// Build the register entry for one issuance, UNSEALED.
///
/// The seal is applied by the [`Store`] port's `append_register`, atomically against the committed
/// tail of the chain (design §4), so this returns the entry with empty `prev`/`entry` digests.
fn register_entry(f: &RegisterEntry<'_>) -> Entry {
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
        signed_by: f.signed_by.to_vec(),
        template_sha256: f.template_hash.to_owned(),
        pdf_sha256: f.pdf_sha256.to_owned(),
        pdf_path: f.stored.to_owned(),
        texlive: typeset::TEXLIVE.to_owned(),
        engine: Some(f.engine.to_owned()),
        engine_version: Some(f.engine_version.to_owned()),
        prev_sha256: String::new(),
        entry_sha256: String::new(),
    }
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
    for code in registry
        .lifecycle
        .owed_upto_for(person.stage(), Some(person.class()))?
    {
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

/// Fields a document needs that this record does not supply, as record paths.
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
/// **The names returned are record paths, not commands** (D-44). Reporting
/// `dkPPreExistingIp` moved the problem rather than solving it: the operator
/// still had to know that meant `pre_existing_ip` before they could edit
/// anything. `docket next` did this translation for itself while the API and
/// every other caller did not, so the mapping lives here now and there is one
/// of it. A command no declared field derives is named as itself and marked
/// unmapped, never dropped and never guessed at.
///
/// # Errors
/// Fails if a template cannot be read, or if two declared fields derive the
/// same command.
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
    // EVERY FILE, not just the top one: a command used only inside a shared clause is still a
    // field the record has to supply, and reading the top level alone would report a document
    // as complete when rendering it would leave a `\dkP...` undefined.
    let body = crate::domain::template::closure(&paths.root, &template)?
        .into_iter()
        .map(|(_, bytes)| String::from_utf8_lossy(&bytes).into_owned())
        .collect::<Vec<_>>()
        .join("\n");

    // The date is irrelevant to which commands get defined, so any will do.
    // THE DOCUMENT IS PASSED so the signature-block commands resolve here too;
    // without it `\dkSignerOneName` reads as a field the person record is missing,
    // and `docket check` reports every class-scoped document as incomplete.
    let defs = definitions_for(
        registry,
        Some(person),
        None,
        clock::parse_iso("2000-01-01")?,
        Some(document),
    );
    let by_command = crate::domain::template::paths_by_command(&registry.catalogue.fields)?;
    Ok(crate::domain::template::required_commands(&body)
        .into_iter()
        .filter(|c| defs.get(c).is_none())
        .map(|c| crate::domain::template::field_of(&c, &by_command))
        .collect())
}

/// Every document owed at this stage that the record cannot yet render.
///
/// Each entry pairs the document code with the record paths it is missing, in
/// the form `unsupplied_fields` reports them.
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
