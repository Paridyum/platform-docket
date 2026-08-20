//! Validating the registries, the templates and the register.

use crate::adapters::{registry::Registry, store};
use crate::config::Paths;
use crate::domain::chain;
use crate::domain::lifecycle::Document;
use crate::domain::personnel::validate;
use crate::domain::register::{sha256, verify_chain};
use anyhow::{Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;

/// A problem found by `check`.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    /// Whether this must block, or merely be reported.
    pub blocking: bool,
    /// What is wrong, in one line.
    pub message: String,
}

/// Everything `check` looked at, and everything it found.
#[derive(Debug)]
pub struct Findings {
    /// Problems, blocking first.
    pub items: Vec<Finding>,
    /// Documents declared.
    pub documents: usize,
    /// Stages declared.
    pub stages: usize,
    /// Entries in the register.
    pub entries: usize,
}

impl Findings {
    /// Problems that must block.
    #[must_use]
    pub fn errors(&self) -> usize {
        self.items.iter().filter(|f| f.blocking).count()
    }

    /// Problems that are reported but do not block.
    #[must_use]
    pub fn warnings(&self) -> usize {
        self.items.len() - self.errors()
    }
}

/// Facts the registry owns, which no template may restate.
///
/// Read from the registry rather than hard-coded, so the check cannot drift
/// away from the values it is protecting.
fn owned_facts(registry: &Registry) -> Vec<(String, String)> {
    let mut facts = Vec::new();
    if let Some(org) = registry.org_node() {
        for (field, owner) in [
            ("email", "org.email"),
            ("web", "org.web"),
            ("address", "org.address"),
        ] {
            if let Some(v) = org.get(field).and_then(|v| v.as_str())
                && v.len() > 6
            {
                facts.push((v.to_owned(), owner.to_owned()));
            }
        }
    }
    for s in &registry.signatories {
        facts.push((s.legal_name.clone(), format!("signatories.{}", s.id)));
    }
    facts
}

/// Record a blocking finding.
fn err(items: &mut Vec<Finding>, message: String) {
    items.push(Finding {
        blocking: true,
        message,
    });
}

/// Record a non-blocking finding.
fn warn(items: &mut Vec<Finding>, message: String) {
    items.push(Finding {
        blocking: false,
        message,
    });
}

/// Run every check.
///
/// # Errors
/// Fails only when a file cannot be read; a check that FINDS a problem
/// reports it in `Findings` rather than returning an error, so one bad
/// document does not hide the rest.
pub fn run(paths: &Paths, registry: &Registry) -> Result<Findings> {
    let mut items: Vec<Finding> = Vec::new();
    let by_code = registry.lifecycle.by_code()?;

    documents(&mut items, paths, registry, &by_code);
    stages(&mut items, registry, &by_code);
    organisation(&mut items, registry);
    templates(&mut items, paths, registry)?;
    gitignore(&mut items, paths);
    catalogue(&mut items, registry);
    communications(&mut items, registry)?;
    people(&mut items, paths, registry)?;
    let entries = register(&mut items, paths)?;
    archive_index(&mut items, paths, registry)?;

    items.sort();
    items.reverse();
    Ok(Findings {
        items,
        documents: by_code.len(),
        stages: registry.lifecycle.stages.len(),
        entries,
    })
}

/// Each declared document has a template, a known retention class and known
/// signatories.
fn documents(
    items: &mut Vec<Finding>,
    paths: &Paths,
    registry: &Registry,
    by_code: &BTreeMap<&str, &Document>,
) {
    let retention: BTreeSet<&str> = registry
        .retention
        .iter()
        .map(|r| r.class.as_str())
        .collect();
    let signatories: BTreeSet<&str> = registry.signatories.iter().map(|s| s.id.as_str()).collect();

    for (code, doc) in by_code {
        if !paths.root.join(&doc.template).exists() {
            err(items, format!("{code}: template missing: {}", doc.template));
        }
        if !retention.contains(doc.retention.as_str()) {
            err(
                items,
                format!(
                    "{code}: retention class {:?} is not declared in retention.yaml",
                    doc.retention
                ),
            );
        }
        for s in &doc.signed_by {
            if !signatories.contains(s.as_str()) {
                err(
                    items,
                    format!("{code}: signatory {s:?} is not in signatories.yaml"),
                );
            }
        }
    }
}

/// Every stage owes documents that exist, and every document is owed by some
/// stage -- an unowed document can never be issued by `onboard`.
fn stages(items: &mut Vec<Finding>, registry: &Registry, by_code: &BTreeMap<&str, &Document>) {
    let mut owed: BTreeSet<&str> = BTreeSet::new();
    for stage in &registry.lifecycle.stages {
        for code in &stage.owes {
            if !by_code.contains_key(code.as_str()) {
                err(
                    items,
                    format!(
                        "stage {}: owes {code}, which is not a declared document",
                        stage.name
                    ),
                );
            }
            owed.insert(code.as_str());
        }
    }
    for code in by_code.keys() {
        if !owed.contains(code) {
            warn(items, format!("{code}: declared but owed by no stage"));
        }
    }
}

/// The organisation's own facts are complete and internally consistent.
fn organisation(items: &mut Vec<Finding>, registry: &Registry) {
    if registry.forum().is_none_or(str::is_empty) {
        err(items, "org.jurisdiction.forum is unset".to_owned());
    }
    if registry.serial_format().is_none() {
        err(items, "org.reference.format is unset".to_owned());
    }
    if let Some(org) = registry.org_node()
        && org.get("cin").is_some_and(|v| !v.is_null())
        && org
            .get("legal_status")
            .and_then(|v| v.as_str())
            .is_some_and(|s| s.contains("Process"))
    {
        err(
            items,
            "org.cin is set but legal_status still says \"Incorporation in Process\"".to_owned(),
        );
    }
}

/// No template restates a fact the registry owns.
fn templates(items: &mut Vec<Finding>, paths: &Paths, registry: &Registry) -> Result<()> {
    let facts = owned_facts(registry);
    let dir = paths.root.join("templates");
    if !dir.exists() {
        return Ok(());
    }
    for entry in walk(&dir)? {
        let body =
            fs::read_to_string(&entry).with_context(|| format!("reading {}", entry.display()))?;
        let shown = entry
            .strip_prefix(&paths.root)
            .unwrap_or(&entry)
            .display()
            .to_string();
        for (line_no, line) in body.lines().enumerate() {
            if line.trim_start().starts_with('%') {
                continue;
            }
            for (literal, owner) in &facts {
                if line.contains(literal.as_str()) {
                    err(
                        items,
                        format!(
                            "{shown}:{}: hard-codes {literal:?}, which {owner} owns",
                            line_no + 1
                        ),
                    );
                }
            }
        }
    }
    Ok(())
}

/// Personal data is excluded from git. Git history cannot be erased, so a
/// person record committed once can never be removed on request.
fn gitignore(items: &mut Vec<Finding>, paths: &Paths) {
    let ignored = fs::read_to_string(paths.root.join(".gitignore")).unwrap_or_default();
    for dir in ["people/", "store/"] {
        if !ignored.contains(dir) {
            err(
                items,
                format!(".gitignore does not exclude {dir}: personal data must never enter git"),
            );
        }
    }
}

/// The chain is unbroken and every issued PDF still matches its hash.
fn register(items: &mut Vec<Finding>, paths: &Paths) -> Result<usize> {
    let entries = store::register(paths)?;
    for fault in verify_chain(&entries)? {
        err(items, fault.to_string());
    }
    for e in &entries {
        let pdf = paths.store.join(&e.pdf_path);
        if pdf.exists() {
            let bytes = fs::read(&pdf).with_context(|| format!("reading {}", pdf.display()))?;
            if sha256(&bytes) != e.pdf_sha256 {
                err(
                    items,
                    format!("{}: stored PDF does not match its recorded hash", e.serial),
                );
            }
        } else {
            warn(
                items,
                format!("{}: issued PDF is missing from the store", e.serial),
            );
        }
    }
    Ok(entries.len())
}

/// The person-class and field registries are internally consistent.
fn catalogue(items: &mut Vec<Finding>, registry: &Registry) {
    let by_path = match registry.catalogue.fields_by_path() {
        Ok(m) => m,
        Err(e) => {
            err(items, e.to_string());
            return;
        }
    };
    let groups: BTreeSet<&str> = registry.catalogue.groups().into_iter().collect();
    let mut prefixes: BTreeSet<&str> = BTreeSet::new();

    for class in &registry.catalogue.classes {
        if !prefixes.insert(class.prefix.as_str()) {
            err(
                items,
                format!(
                    "person class {}: prefix {:?} is used by another class; \
                 identifiers would collide",
                    class.class, class.prefix
                ),
            );
        }
        for g in &class.permits {
            if !groups.contains(g.as_str()) {
                err(
                    items,
                    format!(
                        "person class {}: permits group {g:?}, which no field belongs to",
                        class.class
                    ),
                );
            }
        }
        for path in &class.requires {
            if !by_path.contains_key(path.as_str()) {
                err(
                    items,
                    format!(
                        "person class {}: requires {path:?}, which fields.yaml does not declare",
                        class.class
                    ),
                );
            } else if let Some(f) = by_path.get(path.as_str())
                && !class.permits.contains(&f.group)
            {
                err(
                    items,
                    format!(
                        "person class {}: requires {path:?} but does not permit its group {:?}",
                        class.class, f.group
                    ),
                );
            }
        }
        for f in &class.forbids {
            if !groups.contains(f.as_str()) && !by_path.contains_key(f.as_str()) {
                err(
                    items,
                    format!(
                        "person class {}: forbids {f:?}, which is neither a group nor a field",
                        class.class
                    ),
                );
            }
            if class.permits.contains(f) {
                err(
                    items,
                    format!(
                        "person class {}: both permits and forbids {f:?}",
                        class.class
                    ),
                );
            }
        }
    }

    for f in &registry.catalogue.fields {
        if f.purpose.trim().is_empty() {
            err(
                items,
                format!(
                    "field {}: has no stated purpose, so it should not be collected",
                    f.path
                ),
            );
        }
    }
}

/// Every message names a stage and documents that exist.
///
/// A message carrying a document code the lifecycle does not declare would
/// tell someone to attach a file that cannot be produced -- which is exactly
/// the drift a wiki page describing the same thing would have.
fn communications(items: &mut Vec<Finding>, registry: &Registry) -> Result<()> {
    let by_code = registry.lifecycle.by_code()?;
    let stages: BTreeSet<&str> = registry
        .lifecycle
        .stages
        .iter()
        .map(|s| s.name.as_str())
        .collect();
    let mut ids: BTreeSet<&str> = BTreeSet::new();

    for m in &registry.messages {
        if !ids.insert(m.id.as_str()) {
            err(items, format!("message {:?} is declared twice", m.id));
        }
        if !stages.contains(m.stage.as_str()) {
            err(
                items,
                format!(
                    "message {}: stage {:?} is not declared in lifecycle.yaml",
                    m.id, m.stage
                ),
            );
        }
        for code in &m.attach {
            if !by_code.contains_key(code.as_str()) {
                err(
                    items,
                    format!(
                        "message {}: attaches {code}, which is not a declared document",
                        m.id
                    ),
                );
            }
        }
        if m.subject.trim().is_empty() || m.body.trim().is_empty() {
            err(
                items,
                format!("message {}: has an empty subject or body", m.id),
            );
        }
    }

    // Every stage that owes a document should have a message that carries one,
    // or nobody is told the document exists.
    for stage in &registry.lifecycle.stages {
        if stage.owes.is_empty() {
            continue;
        }
        if !registry.messages.iter().any(|m| m.stage == stage.name) {
            warn(
                items,
                format!(
                    "stage {} owes {} document(s) and no message carries them",
                    stage.name,
                    stage.owes.len()
                ),
            );
        }
    }
    Ok(())
}

/// Every person record matches what its class declares, and every record on
/// disk has an identifier.
fn people(items: &mut Vec<Finding>, paths: &Paths, registry: &Registry) -> Result<()> {
    let ledger = store::ledger(paths)?;
    for fault in chain::verify(&ledger)? {
        err(items, format!("identity ledger {fault}"));
    }
    let log = store::events(paths)?;
    for fault in chain::verify(&log)? {
        err(items, format!("event log {fault}"));
    }

    signatures(items, paths)?;

    for slug in store::slugs(paths)? {
        let Ok(record) = crate::application::personnel::identify(paths, &slug) else {
            err(
                items,
                format!(
                    "people/{slug}.yaml has no identifier in the ledger; \
                 it was created by hand rather than by `docket new`"
                ),
            );
            continue;
        };
        let Ok(class) = crate::application::personnel::class_of(registry, &record) else {
            err(
                items,
                format!(
                    "{slug}: ledger names an undeclared class {:?}",
                    record.class
                ),
            );
            continue;
        };
        let person = store::person(paths, &slug)?;
        for v in validate(&registry.catalogue, class, &person)? {
            err(items, format!("{slug}: {v}"));
        }

        // `permits` is class-level and cannot express "only when a child".
        // Guardian data on an adult's record is over-collection just as
        // surely as a PAN on an intern's, so it is checked here.
        let child = person
            .string("engagement.start_date")
            .and_then(|d| crate::adapters::clock::parse_iso(d).ok())
            .and_then(|d| person.is_child_on(d));
        match child {
            Some(false) if person.has_guardian() => err(
                items,
                format!(
                    "{slug}: a guardian is recorded but this person is eighteen or older \
                 on their start date; guardian data is collected only where DPDP s9 \
                 requires it"
                ),
            ),
            Some(true) if !person.has_guardian() => err(
                items,
                format!(
                    "{slug}: is under eighteen on their start date and no guardian is \
                 recorded; DPDP s9 requires verifiable guardian consent"
                ),
            ),
            _ => {}
        }
    }
    Ok(())
}

/// The generated index is current, and the payload manifest still matches.
///
/// ADR-0040's `docs.system_view_is_current` pattern: the failure mode of
/// generated output is staleness, not incorrectness, so the only claim worth
/// making about it is that regenerating produces the same bytes.
fn archive_index(items: &mut Vec<Finding>, paths: &Paths, registry: &Registry) -> Result<()> {
    let dir = paths.store.join("index");
    let roster = dir.join("roster.csv");
    if !paths.register().exists() {
        return Ok(()); // nothing issued yet; an index would be empty
    }
    if !roster.exists() {
        warn(
            items,
            "store/index is absent; run `docket index`".to_owned(),
        );
        return Ok(());
    }
    let before = fs::read_to_string(&roster).unwrap_or_default();
    let docs_before = fs::read_to_string(dir.join("documents.csv")).unwrap_or_default();
    crate::application::archive::index(paths, registry)?;
    let after = fs::read_to_string(&roster).unwrap_or_default();
    let docs_after = fs::read_to_string(dir.join("documents.csv")).unwrap_or_default();
    if before != after || docs_before != docs_after {
        err(
            items,
            "store/index was stale and has been regenerated; \
                    it is generated output and must never be hand-edited"
                .to_owned(),
        );
    }
    for problem in crate::application::archive::verify_manifest(paths)? {
        err(items, format!("manifest: {problem}"));
    }
    Ok(())
}

/// The signature ledger is intact, references real issuances, and its scans
/// are still where they were filed.
fn signatures(items: &mut Vec<Finding>, paths: &Paths) -> Result<()> {
    let signatures = store::signatures(paths)?;
    for fault in chain::verify(&signatures)? {
        err(items, format!("signature ledger {fault}"));
    }
    let register = store::register(paths)?;
    for sig in &signatures {
        match register.iter().find(|e| e.serial == sig.serial) {
            None => err(
                items,
                format!(
                    "signature ledger: {} is recorded as executed but was never issued",
                    sig.serial
                ),
            ),
            Some(e) => {
                if sig.signed_on < e.issued_on {
                    err(
                        items,
                        format!(
                            "signature ledger: {} is recorded as signed on {} but issued on {}",
                            sig.serial, sig.signed_on, e.issued_on
                        ),
                    );
                }
                // A scan recorded but absent means the evidence is gone.
                if let Some(rel) = &sig.scan_path {
                    let p = paths.store.join(rel);
                    if !p.exists() {
                        err(
                            items,
                            format!(
                                "signature ledger: {}'s scan is recorded at {rel} and is missing",
                                sig.serial
                            ),
                        );
                    } else if let Some(expected) = &sig.scan_sha256 {
                        let bytes =
                            fs::read(&p).with_context(|| format!("reading {}", p.display()))?;
                        if &sha256(&bytes) != expected {
                            err(
                                items,
                                format!(
                                    "signature ledger: {}'s scan does not match its recorded hash",
                                    sig.serial
                                ),
                            );
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Every `.tex` file beneath a directory, sorted for stable output.
fn walk(dir: &std::path::Path) -> Result<Vec<std::path::PathBuf>> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in fs::read_dir(&d).with_context(|| format!("reading {}", d.display()))? {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "tex") {
                out.push(path);
            }
        }
    }
    out.sort();
    Ok(out)
}
