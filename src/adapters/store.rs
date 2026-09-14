//! The records store: person records, the register, and issued PDFs.
//!
//! The [`Store`] trait is docket's **record port** (ADR-0093 port design §4): the four sealed chains
//! and the mutable person document, behind one synchronous interface with two implementations —
//! [`FileStore`] here (JSONL/YAML on disk, the CLI's break-glass and offline path) and `PgStore` in
//! `adapters::pg` (Postgres, the served backend). Only records are on the port; blobs (issued PDFs,
//! packs, scans, generated indexes) stay on [`Paths`] and `fs::` for now — the blob store is a later,
//! separate question (ADR-0068 §6). Person WRITES are text (`&str`): the file store writes the YAML
//! verbatim so a hand-edited record keeps its shape, and the Postgres store parses the same text into
//! the JSONB document.

use crate::adapters::registry::read_yaml;
use crate::config::Paths;
use crate::domain::chain::{Chained, seal};
use crate::domain::event::Event;
use crate::domain::identity::Record as IdentityRecord;
use crate::domain::person::Person;
use crate::domain::register::Entry;
use crate::domain::signature::Record as Signature;
use anyhow::{Context, Result, bail};
use std::fs;
use std::io::Write as _;

/// docket's record store: the four sealed chains and the person document (ADR-0093).
///
/// Append methods take an UNSEALED record, seal it onto the current end of its chain, and return the
/// sealed record — so sealing lives behind the port, where the Postgres implementation can make the
/// read-seal-append one atomic, fork-proof transaction. The file implementation is single-writer and
/// seals the same way. Reads return the whole chain in order (the caller verifies).
///
/// `Sync` is a supertrait so a `&dyn Store` can be shared across the served backend's per-request
/// threads (ADR-0093 §decision-2). Both implementations are already `Sync` (a `PgPool` and a
/// `&Paths` both are), so it costs nothing and it is the concurrency the database now provides.
pub trait Store: Sync {
    /// Read one person's record. Fails if it is absent.
    ///
    /// # Errors
    /// Fails if the record is absent or does not parse.
    fn person(&self, slug: &str) -> Result<Person>;
    /// Whether a person record exists, without reading it.
    ///
    /// # Errors
    /// Fails only if existence cannot be determined.
    fn person_exists(&self, slug: &str) -> Result<bool>;
    /// Every person slug that has a record, sorted.
    ///
    /// # Errors
    /// Fails if the set of records cannot be listed.
    fn slugs(&self) -> Result<Vec<String>>;
    /// Create a new person record from a YAML body. Fails if one already exists.
    ///
    /// # Errors
    /// Fails if a record already exists, or the write is refused.
    fn create_person(&self, slug: &str, body: &str) -> Result<()>;
    /// Replace an existing person record with a YAML body (the mutable edit path).
    ///
    /// # Errors
    /// Fails if the write is refused.
    fn rewrite_person(&self, slug: &str, body: &str) -> Result<()>;

    /// The identity ledger, in order.
    ///
    /// # Errors
    /// Fails if the ledger cannot be read or a line does not parse.
    fn ledger(&self) -> Result<Vec<IdentityRecord>>;
    /// Seal an identity record onto the ledger and persist it.
    ///
    /// # Errors
    /// Fails if the record cannot be sealed or persisted.
    fn append_identity(&self, record: IdentityRecord) -> Result<IdentityRecord>;

    /// The register (issued documents), in issue order.
    ///
    /// # Errors
    /// Fails if the register cannot be read or a line does not parse.
    fn register(&self) -> Result<Vec<Entry>>;
    /// Seal a register entry onto the register and persist it.
    ///
    /// # Errors
    /// Fails if the entry cannot be sealed or persisted.
    fn append_register(&self, entry: Entry) -> Result<Entry>;

    /// The personnel event log, in order.
    ///
    /// # Errors
    /// Fails if the log cannot be read or a line does not parse.
    fn events(&self) -> Result<Vec<Event>>;
    /// Seal a personnel event onto the log and persist it.
    ///
    /// # Errors
    /// Fails if the event cannot be sealed or persisted.
    fn append_event(&self, event: Event) -> Result<Event>;

    /// The signature ledger, in order.
    ///
    /// # Errors
    /// Fails if the ledger cannot be read or a line does not parse.
    fn signatures(&self) -> Result<Vec<Signature>>;
    /// Seal a signature record onto the ledger and persist it.
    ///
    /// # Errors
    /// Fails if the record cannot be sealed or persisted.
    fn append_signature(&self, record: Signature) -> Result<Signature>;

    /// Rewrite the event log to `events`, re-sealing the chain from a new genesis.
    ///
    /// This is the single writer that does not append: `docket erase` rewrites the event log to its
    /// skeletal events and re-seals it, because an append-only log that cannot forget cannot honour a
    /// DPDP erasure (CLAUDE.md, design §6). The file store rewrites the JSONL and re-seals; the
    /// Postgres store must DELETE chain rows, which the append-only trigger forbids without a
    /// platform-tenancy migration, so it refuses with a clear error (tracked as debt).
    ///
    /// # Errors
    /// Fails if the log cannot be re-sealed or rewritten, or (on Postgres) if erasing is not yet
    /// supported by the store.
    fn reseal_events(&self, events: Vec<Event>) -> Result<()>;
}

/// The filesystem record store: JSONL chains under `store/` and YAML records under `people/`.
///
/// This is the store docket has always used, now behind the [`Store`] port. It delegates to the
/// module's free functions so its behaviour is exactly what it was before the port existed (the
/// existing suite is the proof). The CLI keeps it as a break-glass and offline path (ADR-0093 §3).
pub struct FileStore<'a> {
    paths: &'a Paths,
}

impl<'a> FileStore<'a> {
    /// A file store rooted at `paths`.
    #[must_use]
    pub fn new(paths: &'a Paths) -> Self {
        Self { paths }
    }
}

impl Store for FileStore<'_> {
    fn person(&self, slug: &str) -> Result<Person> {
        person(self.paths, slug)
    }

    fn person_exists(&self, slug: &str) -> Result<bool> {
        Ok(self.paths.person(slug).exists())
    }

    fn slugs(&self) -> Result<Vec<String>> {
        slugs(self.paths)
    }

    fn create_person(&self, slug: &str, body: &str) -> Result<()> {
        let path = self.paths.person(slug);
        if path.exists() {
            bail!("a record already exists at {}", path.display());
        }
        fs::create_dir_all(&self.paths.people)
            .with_context(|| format!("creating {}", self.paths.people.display()))?;
        fs::write(&path, body).with_context(|| format!("writing {}", path.display()))?;
        Ok(())
    }

    fn rewrite_person(&self, slug: &str, body: &str) -> Result<()> {
        let path = self.paths.person(slug);
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        }
        fs::write(&path, body).with_context(|| format!("rewriting {}", path.display()))?;
        Ok(())
    }

    fn ledger(&self) -> Result<Vec<IdentityRecord>> {
        ledger(self.paths)
    }

    fn append_identity(&self, record: IdentityRecord) -> Result<IdentityRecord> {
        append_chain(&self.paths.ledger(), record)
    }

    fn register(&self) -> Result<Vec<Entry>> {
        register(self.paths)
    }

    fn append_register(&self, entry: Entry) -> Result<Entry> {
        // The register's seal lived in the caller (issue.rs) and the append was of a pre-sealed
        // entry; behind the port the seal moves here so both stores do read-seal-append the same way.
        let existing = register(self.paths)?;
        let sealed = seal(entry, existing.last())?;
        append(self.paths, &sealed)?;
        Ok(sealed)
    }

    fn events(&self) -> Result<Vec<Event>> {
        events(self.paths)
    }

    fn append_event(&self, event: Event) -> Result<Event> {
        append_chain(&self.paths.events(), event)
    }

    fn signatures(&self) -> Result<Vec<Signature>> {
        signatures(self.paths)
    }

    fn append_signature(&self, record: Signature) -> Result<Signature> {
        append_chain(&self.paths.signatures(), record)
    }

    fn reseal_events(&self, events: Vec<Event>) -> Result<()> {
        // The erase special case (design §6): rewrite the whole event log and re-seal it from a new
        // genesis. This is the one path that does not append; it is written here behind the port so
        // the application layer never rewrites a chain file directly.
        let mut sealed: Vec<Event> = Vec::new();
        for e in events {
            let next = seal(e, sealed.last())?;
            sealed.push(next);
        }
        let mut body = String::new();
        for e in &sealed {
            body.push_str(&serde_json::to_string(e).context("serialising an event")?);
            body.push('\n');
        }
        let path = self.paths.events();
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        }
        fs::write(&path, body).with_context(|| format!("rewriting {}", path.display()))?;
        Ok(())
    }
}

/// Read one person record.
///
/// # Errors
/// Fails if the record is absent or does not parse.
pub fn person(paths: &Paths, slug: &str) -> Result<Person> {
    let path = paths.person(slug);
    if !path.exists() {
        bail!(
            "no person record: {}\n       create it from {}",
            path.display(),
            paths.people.join("_TEMPLATE.yaml").display()
        );
    }
    Ok(Person::new(slug.to_owned(), read_yaml(&path)?))
}

/// Read the whole register, in issue order.
///
/// A malformed line is an error rather than a skip. Silently ignoring one
/// would break the hash chain in a way that looks like tampering later.
///
/// # Errors
/// Fails if the register cannot be read, or if any line is
/// malformed -- skipping a bad line would break the chain in a way that
/// looks like tampering later.
pub fn register(paths: &Paths) -> Result<Vec<Entry>> {
    let path = paths.register();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let entry: Entry = serde_json::from_str(line)
            .with_context(|| format!("{}: line {} is not a valid entry", path.display(), i + 1))?;
        out.push(entry);
    }
    Ok(out)
}

/// Append one sealed entry.
///
/// Opened in append mode and never rewritten, so an existing record cannot
/// be replaced by a bug in this function.
///
/// # Errors
/// Fails if the register cannot be created or appended to.
pub fn append(paths: &Paths, entry: &Entry) -> Result<()> {
    let path = paths.register();
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    let line = serde_json::to_string(entry).context("serialising a register entry")?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("opening {} for append", path.display()))?;
    writeln!(file, "{line}").with_context(|| format!("appending to {}", path.display()))?;
    Ok(())
}

// `keep_pdf` moved to the blob port (`adapters::blobs::FileBlobs`): storing the issued PDF is a blob
// operation, and behind the port the issuer writes the bytes with `Blobs::put(key, bytes)` where the
// key is the register's `pdf_path` (design §11).

/// Read a hash-chained JSONL file, in order.
///
/// A malformed line is an error rather than a skip. Silently ignoring one
/// would break the chain in a way that looks like tampering later.
///
/// # Errors
/// Fails if the file cannot be read or any line does not parse.
pub fn read_chain<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        out.push(serde_json::from_str(line).with_context(|| {
            format!("{}: line {} is not a valid record", path.display(), i + 1)
        })?);
    }
    Ok(out)
}

/// Append one record to a chained file, sealing it onto the end.
///
/// Opened in append mode and never rewritten, so an existing record cannot
/// be replaced by a bug in this function.
///
/// # Errors
/// Fails if the file cannot be created, sealed onto, or appended to.
pub fn append_chain<T>(path: &std::path::Path, record: T) -> Result<T>
where
    T: Chained + serde::de::DeserializeOwned + serde::Serialize,
{
    let existing: Vec<T> = read_chain(path)?;
    let sealed = seal(record, existing.last())?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    let line = serde_json::to_string(&sealed).context("serialising a record")?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("opening {} for append", path.display()))?;
    writeln!(file, "{line}").with_context(|| format!("appending to {}", path.display()))?;
    Ok(sealed)
}

/// The identity ledger: which identifiers have been issued.
///
/// # Errors
/// Fails if the ledger cannot be read or a line does not parse.
pub fn ledger(paths: &Paths) -> Result<Vec<IdentityRecord>> {
    read_chain(&paths.ledger())
}

/// The signature ledger.
///
/// # Errors
/// Fails if the ledger cannot be read or a line does not parse.
pub fn signatures(paths: &Paths) -> Result<Vec<crate::domain::signature::Record>> {
    read_chain(&paths.signatures())
}

/// The personnel event log.
///
/// # Errors
/// Fails if the log cannot be read or a line does not parse.
pub fn events(paths: &Paths) -> Result<Vec<Event>> {
    read_chain(&paths.events())
}

/// Every person slug with a record on disk, sorted.
///
/// # Errors
/// Fails if the people directory cannot be listed.
pub fn slugs(paths: &Paths) -> Result<Vec<String>> {
    if !paths.people.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&paths.people)
        .with_context(|| format!("reading {}", paths.people.display()))?
    {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "yaml")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            && !stem.starts_with('_')
        {
            out.push(stem.to_owned());
        }
    }
    out.sort();
    Ok(out)
}

#[cfg(test)]
// The crate denies expect/unwrap; an assertion IS the point of a test, so they are exempted here as
// in every other test module.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::{FileStore, Store};
    use crate::config::Paths;
    use crate::domain::event::{Event, Kind};
    use crate::domain::identity::{EngagementId, PersonId, Record, Standing};
    use crate::domain::register::{Entry, verify_chain};
    use crate::domain::signature::{Method, Record as Signature};

    fn store_dir() -> (tempfile::TempDir, Paths) {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = Paths::resolve(dir.path());
        (dir, paths)
    }

    fn entry(seq: u32) -> Entry {
        Entry {
            serial: format!("PAR/NDA/2026/{seq:04}"),
            code: "NDA".to_owned(),
            seq,
            year: 2026,
            person_id: PersonId::new(1).expect("id"),
            engagement_id: EngagementId::new("PAR", "INT", 2026, 1).expect("id"),
            slug: "asha-verma".to_owned(),
            document: "nda".to_owned(),
            title: "Non-Disclosure Agreement".to_owned(),
            kind: "signed".to_owned(),
            issued_on: "2026-08-19".to_owned(),
            retention: "contract".to_owned(),
            template_sha256: "a".repeat(64),
            pdf_sha256: "b".repeat(64),
            pdf_path: format!("issued/asha-verma/{seq}.pdf"),
            texlive: "2026".to_owned(),
            engine: None,
            engine_version: None,
            signed_by: vec!["cto".to_owned()],
            prev_sha256: String::new(),
            entry_sha256: String::new(),
        }
    }

    fn identity(pseq: u32, standing: Standing) -> Record {
        Record {
            person_id: PersonId::new(pseq).expect("id"),
            engagement_id: EngagementId::new("PAR", "INT", 2026, 1).expect("id"),
            slug: "asha-verma".to_owned(),
            class: "intern".to_owned(),
            created_on: "2026-08-19".to_owned(),
            standing,
            standing_changed_on: "2026-08-19".to_owned(),
            prev_sha256: String::new(),
            entry_sha256: String::new(),
        }
    }

    fn evt(kind: Kind) -> Event {
        Event {
            person_id: PersonId::new(1).expect("id"),
            at: "2026-08-19".to_owned(),
            kind,
            detail: None,
            recorded_by: "cto".to_owned(),
            prev_sha256: String::new(),
            entry_sha256: String::new(),
        }
    }

    fn sig(serial: &str) -> Signature {
        Signature {
            serial: serial.to_owned(),
            signed_on: "2026-09-01".to_owned(),
            method: Method::WetInk,
            scan_sha256: None,
            scan_path: None,
            received_on: "2026-09-02".to_owned(),
            recorded_by: "cto".to_owned(),
            prev_sha256: String::new(),
            entry_sha256: String::new(),
        }
    }

    #[test]
    fn the_register_seals_and_reads_back_through_the_port() {
        let (_dir, paths) = store_dir();
        let store = FileStore::new(&paths);
        for i in 1..=3 {
            let sealed = store.append_register(entry(i)).expect("append");
            assert!(!sealed.entry_sha256.is_empty(), "the port sealed it");
        }
        let read = store.register().expect("read");
        assert_eq!(read.len(), 3);
        assert!(
            verify_chain(&read).expect("verifiable").is_empty(),
            "the register verifies through the port"
        );
        assert_eq!(read[1].prev_sha256, read[0].entry_sha256, "linked in order");
        assert_eq!(
            read[2].prev_sha256, read[1].entry_sha256,
            "and on down the chain"
        );
    }

    #[test]
    fn every_chain_seals_through_the_port() {
        let (_dir, paths) = store_dir();
        let store = FileStore::new(&paths);

        store
            .append_identity(identity(1, Standing::Active))
            .expect("identity");
        store.append_event(evt(Kind::Created)).expect("event");
        store
            .append_signature(sig("PAR/NDA/2026/0001"))
            .expect("signature");

        assert_eq!(store.ledger().expect("ledger").len(), 1);
        assert_eq!(store.events().expect("events").len(), 1);
        let sigs = store.signatures().expect("signatures");
        assert_eq!(sigs.len(), 1);
        assert!(!sigs[0].entry_sha256.is_empty(), "sealed by the port");
    }

    #[test]
    fn a_person_is_created_read_and_listed() {
        let (_dir, paths) = store_dir();
        let store = FileStore::new(&paths);

        assert!(!store.person_exists("asha-verma").expect("exists"));
        store
            .create_person("asha-verma", "legal_name: Asha Verma\nclass: intern\n")
            .expect("create");
        assert!(store.person_exists("asha-verma").expect("exists"));
        assert_eq!(
            store
                .person("asha-verma")
                .expect("read")
                .string("legal_name"),
            Some("Asha Verma")
        );
        assert_eq!(store.slugs().expect("slugs"), vec!["asha-verma".to_owned()]);

        // create is create-only; a second create is refused, a rewrite replaces.
        assert!(store.create_person("asha-verma", "x: 1\n").is_err());
        store
            .rewrite_person("asha-verma", "legal_name: Asha K Verma\nclass: intern\n")
            .expect("rewrite");
        assert_eq!(
            store
                .person("asha-verma")
                .expect("read")
                .string("legal_name"),
            Some("Asha K Verma")
        );
    }
}
