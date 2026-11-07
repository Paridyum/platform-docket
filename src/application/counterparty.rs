//! Enrolling and listing the organisations documents are issued to (RFC-0136).
//!
//! FILE STORE ONLY, for now. The records live under `store/counterparties/`; the Postgres store has
//! no table for them until a platform-tenancy migration adds one, so the CLI refuses these
//! commands there rather than writing to a disk the served backend never reads (DECISION-DEBT).

use crate::adapters::{clock, store};
use crate::config::Paths;
use crate::domain::counterparty::{Allocation, Counterparty, next_id};
use anyhow::{Context, Result, bail};
use std::fs;
use std::io::Write as _;

/// The retention class in `registry/retention.yaml` that governs a counterparty record.
///
/// Named here so `check` can refuse a registry that does not declare it (REQ-0003: every record
/// is retained on a stated schedule, and a schedule nobody wrote down is not one).
pub const RETENTION_CLASS: &str = "counterparty";

/// The three facts an operator supplies.
#[derive(Debug, Clone, Copy)]
pub struct Facts<'a> {
    /// The organisation's legal name.
    pub legal_name: &'a str,
    /// Its address.
    pub address: &'a str,
    /// The office that receives documents.
    pub office: &'a str,
}

/// Record a new organisation counterparty and allocate its identifier.
///
/// ALLOCATED FROM THE LEDGER, NEVER FROM THE FILES. `store/register/counterparties.jsonl` is
/// append-only, so an identifier or slug once allocated -- or once named by a register entry -- is
/// never allocated again, even after its record file is deleted (ADR-0051). The allocation is
/// sealed before the record is written: a crash between them leaves an identifier allocated and
/// unused, which is harmless, rather than a record no ledger accounts for. The CLI holds the
/// store's write lock around this, which is what makes read-then-append one step.
///
/// # Errors
/// Fails if the slug is malformed or ever allocated, a fact is empty, or a write fails.
pub fn enrol(
    paths: &Paths,
    slug: &str,
    facts: Facts<'_>,
    date: time::Date,
    by: &str,
) -> Result<Counterparty> {
    check_slug(slug)?;
    let ledger: Vec<Allocation> = store::read_chain(&paths.counterparty_ledger())?;
    let register = store::register(paths)?;
    let named = register.iter().filter(|e| e.counterparty_id.is_some());
    if paths.counterparty(slug).exists()
        || ledger.iter().any(|a| a.slug == slug)
        || named.clone().any(|e| e.slug == slug)
    {
        bail!("slug {slug:?} is already allocated, and a slug is never reused (ADR-0051)");
    }
    let on_disk = list(paths)?;
    let used = ledger
        .iter()
        .map(|a| &a.id)
        .chain(named.filter_map(|e| e.counterparty_id.as_ref()))
        .chain(on_disk.iter().map(|c| &c.id));
    let record = Counterparty {
        slug: slug.to_owned(),
        version: 1,
        id: next_id(used)?,
        legal_name: facts.legal_name.trim().to_owned(),
        address: facts.address.trim().to_owned(),
        office: facts.office.trim().to_owned(),
    };
    if let Some(problem) = record.problems().into_iter().next() {
        bail!("{problem}; nothing was recorded");
    }
    let body = record.to_yaml()?;
    store::append_chain(
        &paths.counterparty_ledger(),
        Allocation {
            id: record.id.clone(),
            slug: slug.to_owned(),
            allocated_on: clock::to_iso(date)?,
            by: by.to_owned(),
            prev_sha256: String::new(),
            entry_sha256: String::new(),
        },
    )?;
    write_new(&paths.counterparty(slug), &body)?;
    Ok(record)
}

/// Write a record whole or not at all: to a temporary file beside it, then renamed into place,
/// so a crash mid-write cannot leave a truncated record that every later command trips on.
fn write_new(path: &std::path::Path, body: &str) -> Result<()> {
    let dir = path.parent().context("a record path has a directory")?;
    fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    let temp = path.with_extension("yaml.tmp");
    let mut file =
        fs::File::create(&temp).with_context(|| format!("writing {}", temp.display()))?;
    file.write_all(body.as_bytes())
        .and_then(|()| file.sync_all())
        .with_context(|| format!("writing {}", temp.display()))?;
    fs::rename(&temp, path).with_context(|| format!("moving {} into place", temp.display()))
}

/// The slug rule, the same as a person's.
fn check_slug(slug: &str) -> Result<()> {
    if slug.is_empty()
        || !slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        bail!("slug {slug:?} must be lowercase letters, digits and hyphens");
    }
    Ok(())
}

/// Read one counterparty's record.
///
/// # Errors
/// Fails if the slug is malformed, there is no record by it, or it does not parse.
pub fn read(paths: &Paths, slug: &str) -> Result<Counterparty> {
    check_slug(slug)?;
    let path = paths.counterparty(slug);
    if !path.exists() {
        bail!("no counterparty {slug:?}; record one with `docket counterparty new {slug}`");
    }
    let body = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    Counterparty::parse(slug, &body)
}

/// A record that may be issued against: one the ledger allocated, under that slug.
///
/// A record made by hand is reported by `check`; this stops it reaching the register too.
///
/// # Errors
/// Fails as `read` does, or when no allocation made this record.
pub fn issuable(paths: &Paths, slug: &str) -> Result<Counterparty> {
    let record = read(paths, slug)?;
    if !ledger(paths)?
        .iter()
        .any(|a| a.id == record.id && a.slug == record.slug)
    {
        bail!(
            "{slug}: {} is in no allocation; it was made by hand",
            record.id
        );
    }
    Ok(record)
}

/// Every allocation ever made, in order.
///
/// # Errors
/// Fails if the ledger cannot be read.
pub fn ledger(paths: &Paths) -> Result<Vec<Allocation>> {
    store::read_chain(&paths.counterparty_ledger())
}

/// Every recorded counterparty, by slug.
///
/// # Errors
/// Fails if the directory cannot be listed or any record does not parse. One unreadable record
/// fails the whole list on purpose: allocation reads this, and skipping a record it cannot read
/// could hand its identifier out a second time.
pub fn list(paths: &Paths) -> Result<Vec<Counterparty>> {
    let dir = paths.counterparties();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut slugs = Vec::new();
    for entry in fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "yaml")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
        {
            slugs.push(stem.to_owned());
        }
    }
    slugs.sort();
    slugs.iter().map(|s| read(paths, s)).collect()
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn sandbox() -> (tempfile::TempDir, Paths) {
        let dir = tempfile::tempdir().expect("temp dir");
        let paths = Paths {
            root: std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")),
            people: dir.path().join("people"),
            store: dir.path().join("store"),
        };
        (dir, paths)
    }

    fn day() -> time::Date {
        clock::parse_iso("2026-11-07").expect("date")
    }

    const FACTS: Facts<'static> = Facts {
        legal_name: "Example University",
        address: "1 College Road, Dehradun",
        office: "The Registrar",
    };

    #[test]
    fn enrolling_allocates_in_sequence_and_refuses_a_taken_slug_or_an_empty_fact() {
        let (_dir, paths) = sandbox();
        let a = enrol(&paths, "example-university", FACTS, day(), "rohan").expect("first");
        let b = enrol(&paths, "example-incubator", FACTS, day(), "rohan").expect("second");
        assert_eq!((a.id.seq(), b.id.seq()), (Some(1), Some(2)));
        assert!(enrol(&paths, "example-university", FACTS, day(), "rohan").is_err());
        let blank = Facts {
            office: " ",
            ..FACTS
        };
        assert!(enrol(&paths, "another", blank, day(), "rohan").is_err());
        assert!(enrol(&paths, "Bad Slug", FACTS, day(), "rohan").is_err());
        assert_eq!(
            list(&paths).expect("list"),
            [b, a],
            "by slug, and nothing else recorded"
        );
    }

    #[test]
    fn a_deleted_records_identifier_and_slug_are_never_allocated_again() {
        // Review M2: allocation from the files on disk handed a deleted organisation's identifier
        // to the next one, and every serial issued to the first then resolved to the second.
        let (_dir, paths) = sandbox();
        let a = enrol(&paths, "example-university", FACTS, day(), "rohan").expect("first");
        fs::remove_file(paths.counterparty("example-university")).expect("deleted by hand");
        let b = enrol(&paths, "example-incubator", FACTS, day(), "rohan").expect("second");
        assert_ne!(a.id, b.id);
        assert_eq!(b.id.seq(), Some(2));
        let again = enrol(&paths, "example-university", FACTS, day(), "rohan");
        assert!(format!("{:#}", again.expect_err("slug reserved")).contains("never reused"));
        let held: Vec<_> = ledger(&paths)
            .expect("ledger")
            .into_iter()
            .map(|l| l.id)
            .collect();
        assert_eq!(held, [a.id, b.id]);
    }

    #[test]
    fn a_record_no_allocation_made_is_not_issuable() {
        let (_dir, paths) = sandbox();
        let a = enrol(&paths, "example-university", FACTS, day(), "rohan").expect("first");
        assert_eq!(
            issuable(&paths, "example-university").expect("allocated"),
            a
        );
        let body = fs::read_to_string(paths.counterparty("example-university")).expect("body");
        fs::write(paths.counterparty("hand-made"), body).expect("copied by hand");
        assert!(issuable(&paths, "hand-made").is_err());
    }
}
