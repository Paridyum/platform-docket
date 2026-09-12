//! Validating the registries, the templates and the register.

use crate::adapters::{registry::Registry, store};
use crate::config::Paths;
use crate::domain::chain;
use crate::domain::identity::Record as IdentityRecord;
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

/// Whether the officers page still matches the registries it was exported from.
///
/// A WARNING, NOT AN ERROR. A stale export is something to go and fix, not a reason to refuse
/// every other operation — and the state it most often reports is "never exported", which is
/// legitimate for a checkout that has not published a public site.
///
/// It cannot see `web-paridyum`; there is no shared build (ADR-0001). What it detects is the
/// half that is local and was actually going wrong: registries edited after the last export.
fn team_export(items: &mut Vec<Finding>, paths: &Paths) -> Result<()> {
    if let Some(reason) = crate::application::team::export_staleness(paths)? {
        warn(items, reason);
    }
    Ok(())
}

/// Whether anything issued is sitting outside the vault.
///
/// A WARNING. An unsealed document is not wrong — it is a document that exists only on one
/// laptop, which is a thing to fix rather than a reason to refuse the next operation. A store
/// with nothing issued says nothing at all, so this stays quiet until there is something to be
/// quiet about.
fn vault(items: &mut Vec<Finding>, paths: &Paths) -> Result<()> {
    if let Some(reason) = crate::application::vault::register_backup_stale(paths) {
        warn(items, reason);
    }
    // The escrow gap: an escrow taken before a shredding still holds the destroyed key, so the
    // erasure is incomplete in a way nothing else reports. Passed `None` because the medium is
    // offline by design and usually not mounted — the warning is about the operator's shelf,
    // which no check can see.
    if let Some(reason) = crate::application::vault::escrow_outstanding(paths, None) {
        warn(items, reason);
    }
    let pending = crate::application::vault::unsealed(paths)?;
    if !pending.is_empty() {
        warn(
            items,
            format!(
                "{} document(s) are issued but not in the vault, so they exist only here: {}. \
                 Run `docket vault seal <slug>`.",
                pending.len(),
                pending.join(" ")
            ),
        );
    }
    Ok(())
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
    gitignore(&mut items, paths, registry);
    catalogue(&mut items, registry);
    communications(&mut items, registry)?;
    people(&mut items, paths, registry)?;
    let entries = register(&mut items, paths)?;
    archive_index(&mut items, paths, registry)?;
    team_export(&mut items, paths)?;
    vault(&mut items, paths)?;

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
    stage_findings(items, &registry.lifecycle, by_code);
}

/// The same, over just the lifecycle it reads.
///
/// Split out so it can be tested. Building a whole `Registry` needs all seven
/// files, and a check nobody can exercise cheaply is a check that goes untested.
fn stage_findings(
    items: &mut Vec<Finding>,
    lifecycle: &crate::domain::lifecycle::Lifecycle,
    by_code: &BTreeMap<&str, &Document>,
) {
    let mut owed: BTreeSet<&str> = BTreeSet::new();
    for stage in &lifecycle.stages {
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
    for (code, doc) in by_code {
        // Declared on-request: not an orphan, and saying so every run would teach
        // an operator to scroll past the list that the next real orphan appears in.
        if doc.on_request {
            if owed.contains(code) {
                err(
                    items,
                    format!(
                        "{code}: declared on_request but a stage also owes it — \
                         it cannot be both offered and required"
                    ),
                );
            }
            continue;
        }
        if !owed.contains(code) {
            warn(items, format!("{code}: declared but owed by no stage"));
        }
    }
}

/// The organisation's own facts are complete and internally consistent.
/// Test seam: `check::run` is the front door, and a test that had to build a
/// whole store to exercise one registry rule would be testing the store.
pub fn organisation_findings(items: &mut Vec<String>, registry: &Registry) {
    let mut found = Vec::new();
    organisation(&mut found, registry);
    items.extend(found.into_iter().map(|f| f.message));
}

fn organisation(items: &mut Vec<Finding>, registry: &Registry) {
    let codes = registry.company_codes();
    if codes.is_empty() {
        err(items, "registry/org.yaml declares no companies".to_owned());
        return;
    }

    // EVERY company, not just the default one. A subsidiary with no jurisdiction
    // produces a governing-law clause naming no court, and the failure would
    // appear only on that company's first document.
    let mut formats: BTreeMap<&str, &str> = BTreeMap::new();
    for code in &codes {
        let Some(company) = registry.company(Some(code)) else {
            continue;
        };
        let field = |path: &[&str]| -> Option<&serde_yaml::Value> {
            let mut node = company;
            for key in path {
                node = node.get(key)?;
            }
            Some(node)
        };

        if field(&["jurisdiction", "forum"])
            .and_then(serde_yaml::Value::as_str)
            .is_none_or(str::is_empty)
        {
            err(items, format!("{code}: jurisdiction.forum is unset"));
        }
        match field(&["reference", "format"]).and_then(serde_yaml::Value::as_str) {
            None => err(items, format!("{code}: reference.format is unset")),
            Some(format) => {
                // TWO COMPANIES SHARING A SERIAL FORMAT COLLIDE. Each keeps its
                // own per-kind counter, so `PAR/NDA/2026/0001` would be issued
                // twice to different people at different companies -- and the
                // serial is what a third party verifies a document by.
                if let Some(other) = formats.insert(format, code) {
                    err(
                        items,
                        format!(
                            "{code} and {other} share the serial format {format:?}; \
                             each company needs its own, or two documents get one serial"
                        ),
                    );
                }
            }
        }
        if field(&["cin"]).is_some_and(|v| !v.is_null())
            && field(&["legal_status"])
                .and_then(serde_yaml::Value::as_str)
                .is_some_and(|v| v.contains("Process"))
        {
            err(
                items,
                format!(
                    "{code}: cin is set but legal_status still says \"Incorporation in Process\""
                ),
            );
        }
        countersignatories(items, registry, code);
    }

    ownership(items, registry, &codes);
    ceased_officers(items, registry);
}

/// An officer who has ceased says when (RFC-0042 §5).
///
/// `active: false` alone records that somebody stopped holding office and not
/// when, which is the half of the question a document issued in between turns
/// on. The register's job is to say who was an officer on a given date; without
/// the date it can only say who is one now.
fn ceased_officers(items: &mut Vec<Finding>, registry: &Registry) {
    for s in &registry.signatories {
        if !s.active && s.ceased_on.is_none() {
            err(
                items,
                format!(
                    "signatories.{}: active is false with no ceased_on. RFC-0042 §5 wants both: \
                     a register that cannot say WHEN an officer ceased cannot say whether they \
                     held office on the date a given document was issued",
                    s.id
                ),
            );
        }
        if s.active && s.ceased_on.is_some() {
            err(
                items,
                format!(
                    "signatories.{}: ceased_on is set while active is still true",
                    s.id
                ),
            );
        }
    }
}

/// A company can countersign its own officers' documents.
///
/// `docket issue` refuses a document whose subject and signatory are the same
/// person, so a company whose active signatories are all ONE PERSON cannot issue
/// that person anything -- and the failure appears at issuance, on the officer
/// who is hardest to replace.
///
/// **Counted by person, not by entry**, and that is the whole reason this is not
/// just "the list is non-empty". Priyanshu holds both `ceo` and `director` at
/// Paridyum: two entries, one human, and for his own documents both are the
/// subject. A count of rows would call that covered.
///
/// D-59 asserted that removing him would leave Ruhani's Cyber documents
/// unsignable. Measured, that was wrong -- the chain reaches Rohan. This check
/// is the property that claim was reaching for, asserted rather than assumed.
fn countersignatories(items: &mut Vec<Finding>, registry: &Registry, company: &str) {
    let people: BTreeSet<&str> = registry
        .active_signatories_of(company)
        .map(|s| s.legal_name.trim())
        .collect();

    match people.len() {
        0 => err(
            items,
            format!(
                "{company} has no active signatory, so nothing can be issued for it \
                 (ADR-0077 decision 4)"
            ),
        ),
        1 => err(
            items,
            format!(
                "{company} has only one person among its active signatories ({}). Their own \
                 documents cannot be issued: `docket issue` refuses a document whose subject \
                 and signatory are the same person, and there is nobody to substitute",
                people.iter().copied().collect::<Vec<_>>().join(", ")
            ),
        ),
        _ => {}
    }
}

/// PARIDYUM owns every other company, directly or through another it owns.
///
/// The operator stated this as a general rule rather than a fact about one
/// subsidiary, so it is checked rather than written in a comment: exactly one
/// company has no owner, every other reaches it by following `owned_by`, and no
/// chain loops.
///
/// It is not bookkeeping. ADR-0077 decision 4 resolves who may execute a
/// subsidiary's documents THROUGH this relationship -- Paridyum's Director signs
/// for the holding company -- so a company whose ownership does not reach the
/// root has no answer to the question of who may sign for it.
fn ownership(items: &mut Vec<Finding>, registry: &Registry, codes: &[&str]) {
    let owner_of = |code: &str| -> Option<String> {
        registry
            .company(Some(code))?
            .get("owned_by")?
            .as_str()
            .map(str::to_owned)
    };

    let roots: Vec<&&str> = codes.iter().filter(|c| owner_of(c).is_none()).collect();
    match roots.len() {
        1 => {}
        0 => err(
            items,
            "every company declares an owner, so none is the holding company; \
             exactly one must have no `owned_by`"
                .to_owned(),
        ),
        _ => err(
            items,
            format!(
                "{} companies declare no owner ({}); PARIDYUM owns every other company, \
                 so exactly one is the holding company",
                roots.len(),
                roots.iter().map(|c| **c).collect::<Vec<_>>().join(", ")
            ),
        ),
    }

    for code in codes {
        let mut seen = vec![(*code).to_owned()];
        let mut at = (*code).to_owned();
        while let Some(owner) = owner_of(&at) {
            if !codes.contains(&owner.as_str()) {
                err(
                    items,
                    format!("{at}: owned_by names {owner:?}, which is not a declared company"),
                );
                break;
            }
            if seen.contains(&owner) {
                err(
                    items,
                    format!(
                        "{code}: ownership loops back on itself through {owner} -- \
                         a company cannot own itself, however indirectly"
                    ),
                );
                break;
            }
            seen.push(owner.clone());
            at = owner;
        }
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
fn gitignore(items: &mut Vec<Finding>, paths: &Paths, registry: &Registry) {
    let ignored = fs::read_to_string(paths.root.join(".gitignore")).unwrap_or_default();
    for dir in ["people/", "store/"] {
        if !ignored.contains(dir) {
            err(
                items,
                format!(".gitignore does not exclude {dir}: personal data must never enter git"),
            );
        }
    }
    let classes: Vec<String> = registry
        .catalogue
        .classes
        .iter()
        .map(|c| c.class.clone())
        .collect();
    class_templates_are_not_ignored(items, &paths.people, &classes, &ignored);
}

/// Every class template on disk is admitted back by a negation (D-49).
///
/// `people/*` is ignored so that personal data never enters git, and each class
/// template needs its own `!people/_TEMPLATE-<class>.yaml` line to come back.
/// `_TEMPLATE-officer.yaml` was written and silently ignored: on a fresh clone
/// `docket new <slug> officer` falls back to `_TEMPLATE.yaml`, producing a
/// record carrying `education` -- a field the officer class forbids -- which
/// `check` then rejects as over-collection. The failure reads as a product
/// defect rather than as a missing file, and it was found by running
/// `git check-ignore` by hand while reviewing a diff, which is not a process.
///
/// **Asserted against the file rather than by shelling out to git**, because
/// the question is whether the NEGATION was written, and a `git check-ignore`
/// answers something subtly different: a file already tracked is reported as
/// not ignored whether or not the line exists. That difference already wasted
/// one mutation test, which passed while proving nothing.
///
/// A glob negation was considered and rejected when the line was first written:
/// `!people/_TEMPLATE*.yaml` would also admit anything a person happened to name
/// that way, and this directory is the one place in the repository where a false
/// negative is personal data in git history. Per-template lines, deliberately --
/// so this check exists to make the deliberate step a required one.
fn class_templates_are_not_ignored(
    items: &mut Vec<Finding>,
    people: &std::path::Path,
    classes: &[String],
    ignored: &str,
) {
    let negations: BTreeSet<&str> = ignored
        .lines()
        .map(str::trim)
        .filter_map(|l| l.strip_prefix('!'))
        .collect();

    for class in classes {
        let name = format!("_TEMPLATE-{class}.yaml");
        if !people.join(&name).exists() {
            // No class-specific template is legitimate: the class uses the
            // generic one. Only a template that EXISTS can be lost to the
            // ignore rule.
            continue;
        }
        if !negations.contains(format!("people/{name}").as_str()) {
            err(
                items,
                format!(
                    "people/{name} exists but .gitignore does not admit it. Add \
                     `!people/{name}` -- without it a fresh clone has no {class} template and \
                     `docket new <slug> {class}` silently falls back to the intern record"
                ),
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
/// Declared value sets, and the two ways one can quietly become wrong.
///
/// `fields.yaml` may declare `choices` for a field, and both interfaces offer
/// exactly that list — the CLI interview and the console's enrolment wizard. Two
/// things can go wrong with a list written in one file about facts that live in
/// another, and both produce a record that passes here and is refused later:
///
/// - **A default outside its own set.** The interview writes it when the operator
///   presses return, so every record made that way is wrong from the first field.
/// - **`engagement.stage` disagreeing with the lifecycle.** The stages are declared
///   in `lifecycle.yaml`, which is their home; the copy in `fields.yaml` exists so
///   the question can be offered as a list. A stage offered here that the lifecycle
///   does not know is a record that no stage gate will ever match.
fn declared_choices(items: &mut Vec<Finding>, registry: &Registry) {
    choice_findings(
        &registry.catalogue.fields,
        &registry.lifecycle.stages,
        items,
    );
}

/// The same, over just the two lists it reads.
///
/// Split out so it can be tested. Building a whole `Registry` needs all seven
/// files, and a check nobody can exercise cheaply is a check that goes untested —
/// which is the failure this file exists to catch, in this file.
fn choice_findings(
    fields: &[crate::domain::personnel::Field],
    stages: &[crate::domain::lifecycle::Stage],
    items: &mut Vec<Finding>,
) {
    for f in fields {
        if let Some(d) = &f.default
            && !f.choices.is_empty()
            && !f.choices.contains(d)
        {
            err(
                items,
                format!(
                    "fields.yaml: {} defaults to {d:?}, which is not one of its declared \
                     choices ({}) — the interview writes that default on a blank answer",
                    f.path,
                    f.choices.join(", ")
                ),
            );
        }
    }

    let Some(field) = fields.iter().find(|f| f.path == "engagement.stage") else {
        return;
    };
    if field.choices.is_empty() {
        return;
    }
    let declared: BTreeSet<&str> = stages.iter().map(|s| s.name.as_str()).collect();
    let offered: BTreeSet<&str> = field.choices.iter().map(String::as_str).collect();
    for extra in offered.difference(&declared) {
        err(
            items,
            format!(
                "fields.yaml: engagement.stage offers {extra:?}, which lifecycle.yaml \
                 does not declare — no stage gate would ever match it"
            ),
        );
    }
    for missing in declared.difference(&offered) {
        warn(
            items,
            format!(
                "fields.yaml: engagement.stage does not offer {missing:?}, which \
                 lifecycle.yaml declares — a record cannot be opened at that stage"
            ),
        );
    }
}

fn catalogue(items: &mut Vec<Finding>, registry: &Registry) {
    declared_choices(items, registry);

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
/// Every allocated identifier appears exactly once.
///
/// `docket.identifiers_are_never_reused` was declared against ADR-0051 on
/// 2026-08-19 and never implemented. Nothing asserted it: a reissued identifier
/// was found only when it happened to arrive with a torn chain, and two
/// concurrent writers that appended cleanly produced **the same person
/// identifier twice with correct links, which this check passed in silence**.
///
/// ADR-0072 decision 3 keeps this beside the lock deliberately. The lock is what
/// prevents the duplicate; this is what finds one that arrived anyway -- from a
/// stale lock cleared wrongly, two stores merged by hand, or an edited ledger.
/// A property this load-bearing is asserted, not inferred from the failure mode
/// that usually accompanies it.
///
/// **Reuse is one identifier across two SLUGS, not one identifier on two rows.**
/// The ledger is append-only and a standing change appends: an erasure adds a
/// second row for the same person, and the first version of this check called
/// that a duplicate and failed `every_chain_still_verifies_after_an_erasure`.
/// A slug is uniquely allocated and never released, so distinct slugs under one
/// identifier means two people -- which is the defect.
///
/// The limit, stated rather than hidden: two writers that raced on the *same*
/// slug would land one identifier on one slug and pass here. That case is what
/// the lock is for, and `personnel` refuses an already-allocated slug besides.
fn unique_identifiers(items: &mut Vec<Finding>, ledger: &[IdentityRecord]) {
    let mut people: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    let mut engagements: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for record in ledger {
        people
            .entry(record.person_id.as_str())
            .or_default()
            .insert(&record.slug);
        engagements
            .entry(record.engagement_id.as_str())
            .or_default()
            .insert(&record.slug);
    }

    for (id, slugs) in &people {
        if slugs.len() > 1 {
            err(
                items,
                format!(
                    "identity ledger: person identifier {id} is allocated to {} different \
                     people -- {}. ADR-0051 forbids reuse: a document issued to this identifier \
                     can no longer say which person it names",
                    slugs.len(),
                    slugs.iter().copied().collect::<Vec<_>>().join(", "),
                ),
            );
        }
    }
    for (id, slugs) in &engagements {
        if slugs.len() > 1 {
            err(
                items,
                format!(
                    "identity ledger: engagement identifier {id} is allocated to {} different \
                     people -- {}",
                    slugs.len(),
                    slugs.iter().copied().collect::<Vec<_>>().join(", "),
                ),
            );
        }
    }
}

fn people(items: &mut Vec<Finding>, paths: &Paths, registry: &Registry) -> Result<()> {
    let ledger = store::ledger(paths)?;
    for fault in chain::verify(&ledger)? {
        err(items, format!("identity ledger {fault}"));
    }
    unique_identifiers(items, &ledger);
    let log = store::events(paths)?;
    for fault in chain::verify(&log)? {
        err(items, format!("event log {fault}"));
    }

    signatures(items, paths)?;

    for slug in store::slugs(paths)? {
        // `engagements`, NOT `identify`. `identify` refuses a person holding two
        // open engagements (ADR-0077 D3) so that no command silently picks one --
        // and this `else` read that refusal as "no identifier in the ledger",
        // reporting a correctly registered officer as hand-created. The question
        // here is whether the record is registered AT ALL, which is what
        // `engagements` answers without having to choose between them.
        let held = crate::application::personnel::engagements(paths, &slug)?;
        let Some(record) = held.first().cloned() else {
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

#[cfg(test)]
// A test that cannot assert an expected value is not a test. The lint stays
// denied for the crate; only assertions inside test modules are exempt.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::choice_findings;
    use crate::domain::lifecycle::Stage;
    use crate::domain::personnel::{Field, FieldFile};

    fn fields(yaml: &str) -> Vec<Field> {
        let parsed: FieldFile = serde_yaml::from_str(yaml).expect("fields parse");
        parsed.fields
    }

    fn stages(names: &[&str]) -> Vec<Stage> {
        names
            .iter()
            .map(|n| {
                serde_yaml::from_str::<Stage>(&format!("stage: {n}\ndescription: d\nowes: []\n"))
                    .expect("stage parses")
            })
            .collect()
    }

    fn run(yaml: &str, declared: &[&str]) -> Vec<String> {
        let mut items = Vec::new();
        choice_findings(&fields(yaml), &stages(declared), &mut items);
        items.into_iter().map(|f| f.message).collect()
    }

    fn ledger_row(seq: u32, eng_seq: u32, slug: &str) -> super::IdentityRecord {
        super::IdentityRecord {
            person_id: crate::domain::identity::PersonId::new(seq).expect("person id"),
            engagement_id: crate::domain::identity::EngagementId::new("PAR", "OFC", 2026, eng_seq)
                .expect("engagement id"),
            slug: slug.to_owned(),
            class: "officer".to_owned(),
            created_on: "2026-09-07".to_owned(),
            standing: crate::domain::identity::Standing::Active,
            standing_changed_on: "2026-09-07".to_owned(),
            // Deliberately blank. These rows form NO valid chain, which is the
            // whole point: the check must find a duplicate without help from
            // the links, because the case it exists for is a duplicate that
            // arrived with the links intact.
            prev_sha256: String::new(),
            entry_sha256: String::new(),
        }
    }

    #[test]
    fn a_class_template_with_no_negation_is_blocking() {
        // D-49. The failure this prevents is silent: a fresh clone has no
        // officer template, `docket new` falls back to the intern record, and
        // the resulting over-collection error reads as a product defect.
        let dir = tempfile::tempdir().expect("tempdir");
        let people = dir.path().join("people");
        std::fs::create_dir_all(&people).expect("people");
        std::fs::write(people.join("_TEMPLATE-officer.yaml"), "version: 1\n").expect("template");

        let paths = crate::config::Paths {
            root: dir.path().to_path_buf(),
            people,
            store: dir.path().join("store"),
        };
        let classes = vec!["officer".to_owned()];

        let mut items = Vec::new();
        super::class_templates_are_not_ignored(&mut items, &paths.people, &classes, "people/*\n");
        assert_eq!(items.len(), 1, "{items:?}");
        assert!(items[0].message.contains("does not admit it"));

        // With the negation, clean.
        let mut ok = Vec::new();
        super::class_templates_are_not_ignored(
            &mut ok,
            &paths.people,
            &classes,
            "people/*\n!people/_TEMPLATE-officer.yaml\n",
        );
        assert!(ok.is_empty(), "{ok:?}");
    }

    #[test]
    fn a_class_with_no_template_of_its_own_is_not_a_finding() {
        // Using the generic record is legitimate; only a template that EXISTS
        // can be lost to the ignore rule.
        let dir = tempfile::tempdir().expect("tempdir");
        let people = dir.path().join("people");
        std::fs::create_dir_all(&people).expect("people");
        let paths = crate::config::Paths {
            root: dir.path().to_path_buf(),
            people,
            store: dir.path().join("store"),
        };
        let mut items = Vec::new();
        super::class_templates_are_not_ignored(
            &mut items,
            &paths.people,
            &["intern".to_owned()],
            "people/*\n",
        );
        assert!(items.is_empty(), "{items:?}");
    }

    #[test]
    fn a_person_identifier_allocated_twice_is_blocking() {
        let ledger = vec![ledger_row(5, 4, "race-a4"), ledger_row(5, 5, "race-b4")];
        let mut items = Vec::new();
        super::unique_identifiers(&mut items, &ledger);
        let text = items
            .iter()
            .map(|f| f.message.clone())
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(items.len(), 1, "{text}");
        assert!(
            text.contains("PAR-P-000005-2 is allocated to 2 different"),
            "{text}"
        );
        assert!(text.contains("race-a4, race-b4"), "{text}");
        assert!(text.contains("ADR-0051"), "{text}");
    }

    #[test]
    fn an_engagement_identifier_allocated_twice_is_blocking() {
        let ledger = vec![ledger_row(5, 4, "race-a4"), ledger_row(6, 4, "race-b4")];
        let mut items = Vec::new();
        super::unique_identifiers(&mut items, &ledger);
        assert_eq!(items.len(), 1);
        assert!(
            items[0]
                .message
                .contains("PAR-OFC-2026-0004 is allocated to 2 different")
        );
    }

    #[test]
    fn a_second_row_for_the_same_person_is_not_reuse() {
        // What an erasure or a standing change leaves behind. Caught the first
        // version of this check, which counted rows instead of people.
        let ledger = vec![ledger_row(5, 4, "a"), ledger_row(5, 4, "a")];
        let mut items = Vec::new();
        super::unique_identifiers(&mut items, &ledger);
        assert!(items.is_empty(), "a standing change is not a duplicate");
    }

    #[test]
    fn distinct_identifiers_are_clean() {
        let ledger = vec![ledger_row(5, 4, "a"), ledger_row(6, 5, "b")];
        let mut items = Vec::new();
        super::unique_identifiers(&mut items, &ledger);
        assert!(items.is_empty());
    }

    #[test]
    fn a_default_outside_its_own_choices_is_blocking() {
        // The interview writes the default when the operator presses return, so
        // this makes every record opened that way wrong from the first field.
        let found = run(
            "fields:\n  - {path: a.b, group: g, store: personnel, purpose: p, \
             choices: ['true', 'false'], default: 'maybe'}\n",
            &[],
        );
        assert_eq!(found.len(), 1, "expected exactly one finding: {found:?}");
        assert!(found[0].contains("a.b"), "{}", found[0]);
        assert!(found[0].contains("maybe"), "{}", found[0]);
    }

    #[test]
    fn a_default_with_no_declared_set_is_fine() {
        // Most fields are free text with a sensible starting value —
        // `identity.pronouns` defaults to they/them and accepts anything.
        let found = run(
            "fields:\n  - {path: a.b, group: g, store: personnel, purpose: p, default: 'x'}\n",
            &[],
        );
        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn a_stage_the_lifecycle_does_not_declare_is_blocking() {
        // Offering it opens records at a stage no gate will ever match, and the
        // person is stuck there with no way forward that any rule describes.
        let found = run(
            "fields:\n  - {path: engagement.stage, group: engagement, store: personnel, \
             purpose: p, choices: [offered, banished]}\n",
            &["offered", "active"],
        );
        // The missing `active` is a warning, so both are reported.
        assert!(
            found
                .iter()
                .any(|m| m.contains("banished") && m.contains("lifecycle.yaml")),
            "{found:?}"
        );
    }

    #[test]
    fn a_stage_the_lifecycle_declares_but_nobody_offers_is_only_a_warning() {
        // Not being able to OPEN a record at `alumni` is a reasonable thing to
        // decide on purpose, unlike offering one that does not exist.
        let mut items = Vec::new();
        choice_findings(
            &fields(
                "fields:\n  - {path: engagement.stage, group: engagement, store: personnel, \
                 purpose: p, choices: [offered]}\n",
            ),
            &stages(&["offered", "alumni"]),
            &mut items,
        );
        assert_eq!(items.len(), 1, "{items:?}");
        assert!(!items[0].blocking, "{items:?}");
    }

    #[test]
    fn the_two_lists_agreeing_reports_nothing() {
        let found = run(
            "fields:\n  - {path: engagement.stage, group: engagement, store: personnel, \
             purpose: p, choices: [offered, active], default: offered}\n",
            &["offered", "active"],
        );
        assert!(found.is_empty(), "{found:?}");
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod on_request_tests {
    use crate::domain::lifecycle::Lifecycle;

    /// The document/stage halves of `lifecycle.yaml`, parsed as the real thing.
    fn lifecycle(yaml: &str) -> Lifecycle {
        serde_yaml::from_str(yaml).expect("lifecycle parses")
    }

    const OWED: &str = "
documents:
  - code: NDA
    id: nda
    title: t
    template: x.tex
    kind: issued
    retention: personnel
stages:
  - stage: preboarding
    description: d
    owes: [NDA]
";

    const ORPHAN: &str = "
documents:
  - code: NDA
    id: nda
    title: t
    template: x.tex
    kind: issued
    retention: personnel
stages:
  - stage: preboarding
    description: d
    owes: []
";

    const ON_REQUEST: &str = "
documents:
  - code: PUB
    id: pub
    title: t
    template: x.tex
    kind: collected
    retention: personnel
    on_request: true
stages:
  - stage: preboarding
    description: d
    owes: []
";

    const BOTH: &str = "
documents:
  - code: PUB
    id: pub
    title: t
    template: x.tex
    kind: collected
    retention: personnel
    on_request: true
stages:
  - stage: preboarding
    description: d
    owes: [PUB]
";

    /// `stages()` needs a `by_code`; both come from the same parsed lifecycle.
    fn findings(yaml: &str) -> Vec<super::Finding> {
        let life = lifecycle(yaml);
        let by_code = life
            .documents
            .iter()
            .map(|d| (d.code.as_str(), d))
            .collect();
        let mut items = Vec::new();
        super::stage_findings(&mut items, &life, &by_code);
        items
    }

    #[test]
    fn a_document_no_stage_owes_is_still_reported() {
        // The warning has to keep working. Declaring the three intentional ones
        // was only worth doing if a real orphan still shows up.
        let found = findings(ORPHAN);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].message.contains("NDA"), "{found:?}");
        assert!(!found[0].blocking, "{found:?}");
    }

    #[test]
    fn a_document_declared_on_request_is_not_an_orphan() {
        // Previously this warned on every run, which made the warning list
        // permanently non-empty and therefore unread.
        assert!(
            findings(ON_REQUEST).is_empty(),
            "{:?}",
            findings(ON_REQUEST)
        );
    }

    #[test]
    fn a_document_owed_by_a_stage_is_not_reported() {
        assert!(findings(OWED).is_empty(), "{:?}", findings(OWED));
    }

    #[test]
    fn being_both_offered_and_required_is_blocking() {
        // The mistake that would quietly turn publicity consent into a demand.
        // A consent a stage requires is not freely given, so this is not a
        // tidiness warning — it is the control.
        let found = findings(BOTH);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].blocking, "{found:?}");
        assert!(found[0].message.contains("PUB"), "{found:?}");
    }
}
