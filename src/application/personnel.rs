//! Creating, describing and retiring a person's record.

use crate::adapters::{clock, registry::Registry, store};
use crate::config::Paths;
use crate::domain::chain;
use crate::domain::event::{self, Event, Kind};
use crate::domain::identity::{self, EngagementId, PersonId, Record, Standing};
use crate::domain::personnel::{Class, Violation, validate};
use anyhow::{Context, Result, bail};
use std::fs;

/// Allocate an identifier and open a record for a new person.
///
/// The record file is written from `people/_TEMPLATE.yaml` so the shape a
/// person is described in comes from one place, not from this function.
///
/// # Errors
/// Fails if the class is unknown, the slug is taken, the template is
/// missing, or the ledger cannot be written.
pub fn create(
    paths: &Paths,
    registry: &Registry,
    slug: &str,
    class_name: &str,
    date: time::Date,
    by: &str,
) -> Result<Record> {
    let Some(class) = registry.catalogue.class(class_name) else {
        let known: Vec<&str> = registry
            .catalogue
            .classes
            .iter()
            .map(|c| c.class.as_str())
            .collect();
        bail!(
            "unknown person class {class_name:?}; declared: {}",
            known.join(" ")
        );
    };
    if !slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        bail!("slug {slug:?} must be lowercase letters, digits and hyphens");
    }

    let ledger = store::ledger(paths)?;
    if identity::slug_taken(&ledger, slug) {
        bail!("slug {slug:?} is already allocated");
    }
    // A record may already be on disk because `--interactive` just wrote it
    // from the answers. That is the one legitimate case, and the LEDGER is
    // what decides whether this person is new -- `slug_taken` above has
    // already said they are not.
    let record_exists = paths.person(slug).exists();

    let year = date.year();
    let person_id = PersonId::new(identity::next_person_seq(&ledger))?;
    let engagement_id = EngagementId::new(
        &class.prefix,
        year,
        identity::next_engagement_seq(&ledger, &class.prefix, year),
    )?;
    let created_on = clock::to_iso(date)?;

    if !record_exists {
        let template = paths.people.join("_TEMPLATE.yaml");
        let body = fs::read_to_string(&template)
            .with_context(|| format!("reading {}", template.display()))?;
        fs::create_dir_all(&paths.people)
            .with_context(|| format!("creating {}", paths.people.display()))?;
        fs::write(paths.person(slug), body)
            .with_context(|| format!("writing {}", paths.person(slug).display()))?;
    }

    let record = store::append_chain(
        &paths.ledger(),
        Record {
            person_id: person_id.clone(),
            engagement_id,
            slug: slug.to_owned(),
            class: class.class.clone(),
            created_on: created_on.clone(),
            standing: Standing::Active,
            standing_changed_on: created_on.clone(),
            prev_sha256: String::new(),
            entry_sha256: String::new(),
        },
    )?;

    record_event(
        paths,
        &person_id,
        Kind::Created,
        Some(&format!("class {class_name}")),
        by,
        date,
    )?;
    Ok(record)
}

/// Open a new engagement for a person who already has an identifier.
///
/// The case ADR-0051 exists for: an intern who is hired. The person
/// identifier is REUSED so the archive stays whole, and a new engagement
/// identifier records that this stint is a different one under a different
/// class.
///
/// # Errors
/// Fails if the person is unknown, erased, or still in an open engagement --
/// re-engaging someone who was never separated would silently overwrite
/// which engagement their documents belong to.
pub fn reengage(
    paths: &Paths,
    registry: &Registry,
    slug: &str,
    class_name: &str,
    date: time::Date,
    by: &str,
) -> Result<Record> {
    let current = identify(paths, slug)?;
    match current.standing {
        Standing::Active => bail!(
            "{slug} is still in an open engagement ({}); separate them first",
            current.engagement_id
        ),
        Standing::Erased => bail!("{slug} is erased; their data was deleted on request"),
        Standing::Separated => {}
    }
    let Some(class) = registry.catalogue.class(class_name) else {
        let known: Vec<&str> = registry
            .catalogue
            .classes
            .iter()
            .map(|c| c.class.as_str())
            .collect();
        bail!(
            "unknown person class {class_name:?}; declared: {}",
            known.join(" ")
        );
    };

    let ledger = store::ledger(paths)?;
    let year = date.year();
    let engagement_id = EngagementId::new(
        &class.prefix,
        year,
        identity::next_engagement_seq(&ledger, &class.prefix, year),
    )?;
    let on = clock::to_iso(date)?;

    let sealed = store::append_chain(
        &paths.ledger(),
        Record {
            person_id: current.person_id.clone(),
            engagement_id,
            slug: slug.to_owned(),
            class: class.class.clone(),
            created_on: current.created_on.clone(),
            standing: Standing::Active,
            standing_changed_on: on,
            prev_sha256: String::new(),
            entry_sha256: String::new(),
        },
    )?;
    record_event(
        paths,
        &sealed.person_id,
        Kind::EngagementChanged,
        Some(&format!(
            "re-engaged as {class_name}: {}",
            sealed.engagement_id
        )),
        by,
        date,
    )?;
    Ok(sealed)
}

/// Append one event to the personnel log.
///
/// # Errors
/// Fails if the log cannot be written.
pub fn record_event(
    paths: &Paths,
    person_id: &PersonId,
    kind: Kind,
    detail: Option<&str>,
    recorded_by: &str,
    date: time::Date,
) -> Result<Event> {
    store::append_chain(
        &paths.events(),
        Event {
            person_id: person_id.clone(),
            at: clock::to_iso(date)?,
            kind,
            detail: detail.map(ToOwned::to_owned),
            recorded_by: recorded_by.to_owned(),
            prev_sha256: String::new(),
            entry_sha256: String::new(),
        },
    )
}

/// The ledger record for a slug.
///
/// # Errors
/// Fails if the ledger cannot be read or the slug is not in it.
pub fn identify(paths: &Paths, slug: &str) -> Result<Record> {
    let ledger = store::ledger(paths)?;
    // The ledger appends on every standing change, so the LAST entry for a
    // slug is the current one.
    ledger
        .into_iter()
        .rfind(|r| r.slug == slug)
        .with_context(|| format!("{slug} has no identifier; run `docket new {slug} <class>`"))
}

/// A person's class, from the ledger.
///
/// # Errors
/// Fails if the ledger has no entry, or names a class the registry lacks.
pub fn class_of<'a>(registry: &'a Registry, record: &Record) -> Result<&'a Class> {
    registry.catalogue.class(&record.class).with_context(|| {
        format!(
            "ledger names class {:?}, which is not declared",
            record.class
        )
    })
}

/// Everything known about one person, gathered for display.
#[derive(Debug)]
pub struct Dossier {
    /// Ledger record: identifier, class, standing.
    pub identity: Record,
    /// Ways the record does not match what its class declares.
    pub violations: Vec<Violation>,
    /// Documents issued, newest last.
    pub documents: Vec<(String, String, String)>,
    /// Systems granted and not revoked.
    pub open_access: Vec<String>,
    /// Event history, oldest first.
    pub history: Vec<(String, String, Option<String>)>,
}

/// Gather a person's whole record.
///
/// # Errors
/// Fails if the ledger, register, log or record cannot be read.
pub fn dossier(paths: &Paths, registry: &Registry, slug: &str) -> Result<Dossier> {
    let identity = identify(paths, slug)?;
    let class = class_of(registry, &identity)?;

    let violations = if identity.standing == Standing::Erased {
        Vec::new()
    } else {
        validate(&registry.catalogue, class, &store::person(paths, slug)?)?
    };

    let documents = store::register(paths)?
        .into_iter()
        .filter(|e| e.person_id == identity.person_id)
        .map(|e| (e.code, e.serial, e.issued_on))
        .collect();

    let log = store::events(paths)?;
    let open_access = event::open_access(&log, &identity.person_id);
    let history = event::for_person(&log, &identity.person_id)
        .into_iter()
        .map(|e| (e.at.clone(), e.kind.as_str().to_owned(), e.detail.clone()))
        .collect();

    Ok(Dossier {
        identity,
        violations,
        documents,
        open_access,
        history,
    })
}

/// Why a person may not leave their current stage yet.
#[derive(Debug, PartialEq, Eq)]
pub struct Gate {
    /// Document code the stage requires executed.
    pub code: String,
    /// Document title, for the message.
    pub title: String,
    /// Why it is not satisfied.
    pub reason: String,
}

impl std::fmt::Display for Gate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({}): {}", self.code, self.title, self.reason)
    }
}

/// Move a person to the next stage of their engagement.
///
/// Refuses while the current stage's `requires_signed` documents are not
/// recorded as executed. That gate was prose in `lifecycle.yaml` and nothing
/// checked it, which is exactly how access gets granted before the NDA comes
/// back.
///
/// Writes the new stage into the person record, because the record is where
/// the stage lives; the move itself is recorded as an event.
///
/// # Errors
/// Fails if the person is unknown, already at the last stage, the gate is not
/// satisfied, or the record cannot be written.
pub fn advance(
    paths: &Paths,
    registry: &Registry,
    slug: &str,
    date: time::Date,
    force: bool,
    by: &str,
) -> Result<(String, String, Vec<Gate>)> {
    let record = identify(paths, slug)?;
    if record.standing != Standing::Active {
        bail!(
            "{slug} is {}; only an active engagement advances",
            record.standing.as_str()
        );
    }
    let person = store::person(paths, slug)?;
    let from = person.stage().to_owned();
    let Some(next) = registry.lifecycle.next_after(&from)? else {
        bail!("{slug} is at {from:?}, the last stage; there is nowhere to advance to");
    };
    let to = next.name.clone();

    let gates = unmet_gates(paths, registry, &record, &from)?;
    if !gates.is_empty() && !force {
        return Ok((from, to, gates));
    }

    // The stage lives in the record, so the record is what changes.
    let path = paths.person(slug);
    let body = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let updated = rewrite_stage(&body, &from, &to).with_context(|| {
        format!(
            "{}: there is no `stage:` line to update and no `engagement:` block to \
             write one into",
            path.display()
        )
    })?;
    fs::write(&path, updated).with_context(|| format!("writing {}", path.display()))?;

    record_event(
        paths,
        &record.person_id,
        Kind::StageChanged,
        Some(&format!("{from} -> {to}")),
        by,
        date,
    )?;
    Ok((from, to, Vec::new()))
}

/// Move the record's `stage:` to `to`, or write the line if it is absent.
///
/// Text editing rather than a YAML round-trip, deliberately: re-serialising would
/// drop every comment in the file, and those comments are each field's stated
/// purpose under the DPDP Act. Losing them to change one word is a poor trade.
///
/// It replaces the line rather than the substring `stage: {from}`, because that
/// substring is not what a record necessarily contains. Two shapes both occur and
/// neither was handled:
///
/// - `stage: "preboarding"` — quoted, which is what `enrol::to_yaml` writes for an
///   answered stage. The substring search missed it, so a record created by
///   `docket new --interactive` could not be advanced at all.
/// - no `stage:` line — which reads as the first stage, because `Person::stage`
///   defaults. Produced whenever the stage question is skipped.
///
/// Handling only the second would have been worse than handling neither: an
/// insert on a record that already had a quoted stage would leave TWO `stage:`
/// keys in one mapping, and the record would then mean whichever one the parser
/// happened to keep.
///
/// Returns `None` only when there is neither a `stage:` line nor an `engagement:`
/// block to put one in.
fn rewrite_stage(body: &str, from: &str, to: &str) -> Option<String> {
    use std::fmt::Write as _;

    let mut out = String::with_capacity(body.len() + 32);
    let mut done = false;

    for line in body.lines() {
        if let (false, Some((indent, comment))) = (done, stage_line(line, from)) {
            let _ = writeln!(out, "{indent}stage: {to}{comment}");
            done = true;
            continue;
        }
        let _ = writeln!(out, "{line}");
    }
    if done {
        return Some(out);
    }

    // No stage line anywhere. Write one under `engagement:`.
    let mut out = String::with_capacity(body.len() + 32);
    for line in body.lines() {
        let _ = writeln!(out, "{line}");
        if !done && line.trim_end() == "engagement:" {
            let _ = writeln!(out, "  stage: {to}");
            done = true;
        }
    }
    done.then_some(out)
}

/// A `stage:` line naming `from`: its indentation, and any trailing comment.
///
/// Three shapes all occur in real records and all must match, because a miss here
/// means the caller writes a SECOND `stage:` key into the same mapping and the
/// record then means whichever one the parser keeps:
///
/// - `stage: offered` — plain.
/// - `stage: "offered"` — quoted, as `enrol::to_yaml` writes it.
/// - `stage: offered   # offered | preboarding | ...` — with the template's
///   trailing comment listing the valid stages. This one is why the original
///   substring search worked at all, and why matching the whole value naively
///   broke it: the comment is part of the line and not part of the value.
///
/// The comment is returned so it can be written back. Dropping it would quietly
/// delete the list of valid stages from every record that gets advanced.
fn stage_line(line: &str, from: &str) -> Option<(String, String)> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix("stage:")?;

    // Split the value from a trailing comment before comparing anything.
    let (value, comment) = match rest.find('#') {
        Some(at) => (
            rest.get(..at).unwrap_or_default(),
            rest.get(at..).unwrap_or_default(),
        ),
        None => (rest, ""),
    };
    let value = value.trim();

    // Strip one layer of matching quotes. Anything else is left alone rather than
    // guessed at.
    let bare = value
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
        .unwrap_or(value);

    if bare != from {
        return None;
    }
    let indent = line.get(..line.len() - trimmed.len())?.to_owned();
    // Two spaces before a preserved comment, which is how the template spaces it.
    let comment = if comment.is_empty() {
        String::new()
    } else {
        format!("   {comment}")
    };
    Some((indent, comment))
}

/// Documents the current stage requires executed that are not.
///
/// # Errors
/// Fails if the registry or store cannot be read.
pub fn unmet_gates(
    paths: &Paths,
    registry: &Registry,
    record: &Record,
    stage: &str,
) -> Result<Vec<Gate>> {
    let Some(stage) = registry.lifecycle.stage(stage) else {
        return Ok(Vec::new());
    };
    let by_code = registry.lifecycle.by_code()?;
    let register = store::register(paths)?;
    let signatures = store::signatures(paths)?;
    let mut out = Vec::new();

    for code in &stage.requires_signed {
        let title = by_code
            .get(code.as_str())
            .map_or_else(|| code.clone(), |d| d.title.clone());
        match register
            .iter()
            .find(|e| e.person_id == record.person_id && &e.code == code)
        {
            None => out.push(Gate {
                code: code.clone(),
                title,
                reason: "not issued".to_owned(),
            }),
            Some(e) if !crate::domain::signature::is_signed(&signatures, &e.serial) => {
                out.push(Gate {
                    code: code.clone(),
                    title,
                    reason: format!("issued as {} but not recorded as executed", e.serial),
                });
            }
            Some(_) => {}
        }
    }
    Ok(out)
}

/// Mark a person separated. Their documents and obligations survive.
///
/// # Errors
/// Fails if the person is unknown or the ledger cannot be written.
pub fn separate(paths: &Paths, slug: &str, date: time::Date, by: &str) -> Result<Record> {
    let current = identify(paths, slug)?;
    if current.standing != Standing::Active {
        bail!("{slug} is already {}", current.standing.as_str());
    }
    let updated = Record {
        standing: Standing::Separated,
        standing_changed_on: clock::to_iso(date)?,
        prev_sha256: String::new(),
        entry_sha256: String::new(),
        ..current
    };
    let sealed = store::append_chain(&paths.ledger(), updated)?;
    record_event(paths, &sealed.person_id, Kind::Separated, None, by, date)?;
    Ok(sealed)
}

/// Erase a person's personal data, keeping the identifier as a tombstone.
///
/// This is DPDP erasure made actually possible. The record file is deleted
/// and the event log is rewritten to its skeletal events. What survives is
/// the identifier, the class, and the dates -- which name no one, and which
/// the document register still refers to.
///
/// Rewriting the log is the one place this system does not append. It is
/// necessary: an append-only log that cannot forget is a log that cannot
/// honour an erasure request, and the chain is re-sealed afterwards so it
/// remains verifiable.
///
/// # Errors
/// Fails if the person is unknown, or the store cannot be rewritten.
pub fn erase(paths: &Paths, slug: &str, date: time::Date, by: &str) -> Result<Record> {
    let current = identify(paths, slug)?;
    if current.standing == Standing::Erased {
        bail!("{slug} is already erased");
    }
    let person_id = current.person_id.clone();

    let record_file = paths.person(slug);
    if record_file.exists() {
        fs::remove_file(&record_file)
            .with_context(|| format!("deleting {}", record_file.display()))?;
    }

    let kept: Vec<Event> = store::events(paths)?
        .into_iter()
        .filter(|e| e.person_id != person_id || e.kind.survives_erasure())
        .map(|e| Event {
            detail: if e.person_id == person_id {
                None
            } else {
                e.detail
            },
            prev_sha256: String::new(),
            entry_sha256: String::new(),
            ..e
        })
        .collect();
    reseal_events(paths, kept)?;

    let updated = Record {
        slug: slug.to_owned(),
        standing: Standing::Erased,
        standing_changed_on: clock::to_iso(date)?,
        prev_sha256: String::new(),
        entry_sha256: String::new(),
        ..current
    };
    let sealed = store::append_chain(&paths.ledger(), updated)?;
    record_event(paths, &person_id, Kind::Erased, None, by, date)?;
    Ok(sealed)
}

/// Rewrite the event log, re-sealing the chain over what remains.
fn reseal_events(paths: &Paths, events: Vec<Event>) -> Result<()> {
    let mut sealed: Vec<Event> = Vec::new();
    for e in events {
        let next = chain::seal(e, sealed.last())?;
        sealed.push(next);
    }
    let mut body = String::new();
    for e in &sealed {
        body.push_str(&serde_json::to_string(e)?);
        body.push('\n');
    }
    let path = paths.events();
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    fs::write(&path, body).with_context(|| format!("rewriting {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
// The lint configuration denies expect/unwrap for the crate; an assertion IS the
// point of a test.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod stage_tests {
    use super::rewrite_stage;

    #[test]
    fn an_unquoted_stage_is_moved_in_place() {
        let body = "version: 1\nengagement:\n  class: intern\n  stage: offered\n";
        let out = rewrite_stage(body, "offered", "preboarding").expect("rewritten");
        assert!(out.contains("  stage: preboarding"), "{out}");
        assert!(!out.contains("offered"), "{out}");
    }

    #[test]
    fn a_quoted_stage_is_moved_too() {
        // What `enrol::to_yaml` writes. A substring search for `stage: offered`
        // missed this, so a record from `docket new --interactive` could not be
        // advanced at all.
        let body = "engagement:\n  stage: \"offered\"\n";
        let out = rewrite_stage(body, "offered", "preboarding").expect("rewritten");
        assert!(out.contains("stage: preboarding"), "{out}");
    }

    #[test]
    fn an_absent_stage_is_written_under_engagement() {
        let body = "version: 1\nengagement:\n  class: intern\n";
        let out = rewrite_stage(body, "offered", "preboarding").expect("written");
        assert!(out.contains("  stage: preboarding"), "{out}");
    }

    #[test]
    fn a_record_never_ends_up_with_two_stage_keys() {
        // The hazard in fixing only the absent case: inserting into a record that
        // already has a quoted stage leaves two keys in one mapping, and the
        // record then means whichever the parser keeps.
        for body in [
            "engagement:\n  stage: offered\n",
            "engagement:\n  stage: \"offered\"\n",
            "engagement:\n  class: intern\n",
        ] {
            let out = rewrite_stage(body, "offered", "preboarding").expect("rewritten");
            assert_eq!(
                out.lines()
                    .filter(|l| l.trim_start().starts_with("stage:"))
                    .count(),
                1,
                "{out}"
            );
            let parsed: serde_yaml::Value = serde_yaml::from_str(&out).expect("valid yaml");
            assert_eq!(
                parsed
                    .get("engagement")
                    .and_then(|e| e.get("stage"))
                    .and_then(|v| v.as_str()),
                Some("preboarding"),
                "{out}"
            );
        }
    }

    #[test]
    fn indentation_is_preserved_rather_than_normalised() {
        let body = "engagement:\n    stage: offered\n";
        let out = rewrite_stage(body, "offered", "active").expect("rewritten");
        assert!(out.contains("    stage: active"), "{out}");
    }

    #[test]
    fn a_record_with_neither_a_stage_nor_an_engagement_block_is_refused() {
        assert!(rewrite_stage("version: 1\n", "offered", "preboarding").is_none());
    }

    #[test]
    fn a_trailing_comment_on_the_stage_line_is_matched_and_kept() {
        // The template's own shape. Matching the whole value naively missed it, so
        // a second `stage:` key was inserted and the record became ambiguous --
        // caught by the signing suite failing with `duplicate entry with key
        // "stage"`, not by this file.
        let body = "engagement:\n  stage: preboarding                     # offered | preboarding | active\n";
        let out = rewrite_stage(body, "preboarding", "active").expect("rewritten");
        assert_eq!(
            out.lines()
                .filter(|l| l.trim_start().starts_with("stage:"))
                .count(),
            1,
            "{out}"
        );
        assert!(out.contains("stage: active"), "{out}");
        // The comment lists the valid stages. Dropping it would quietly delete
        // that from every record that ever gets advanced.
        assert!(out.contains("# offered | preboarding | active"), "{out}");
    }

    #[test]
    fn comments_survive_the_rewrite() {
        // The whole reason this is text editing rather than a YAML round-trip:
        // every comment is a field's stated purpose under the DPDP Act.
        let body =
            "engagement:\n  # Drives which documents are owed right now.\n  stage: offered\n";
        let out = rewrite_stage(body, "offered", "preboarding").expect("rewritten");
        assert!(out.contains("# Drives which documents are owed"), "{out}");
    }
}
