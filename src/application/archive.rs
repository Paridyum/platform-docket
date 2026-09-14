//! Packaging the store so it outlives the tool that wrote it.

use crate::adapters::registry::Epoch;
use crate::adapters::store::Store;
use crate::adapters::{clock, registry::Registry};
use crate::config::Paths;
use crate::domain::chain::sha256;
use crate::domain::identity::Standing;
use anyhow::{Context, Result};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// What `index` wrote.
#[derive(Debug)]
pub struct Written {
    /// Files created or replaced.
    pub files: Vec<String>,
    /// People in the roster.
    pub people: usize,
    /// Documents in the document index.
    pub documents: usize,
}

/// Escape one CSV field: quote it and double any internal quote.
///
/// Every field is quoted unconditionally. Deciding per-field whether quoting
/// is needed is where CSV writers acquire bugs, and a person's name is
/// exactly the field that will one day contain a comma.
fn field(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn row(cells: &[&str]) -> String {
    let mut line = String::new();
    for (i, c) in cells.iter().enumerate() {
        if i > 0 {
            line.push(',');
        }
        line.push_str(&field(c));
    }
    line.push('\n');
    line
}

/// Generate the roster and document indexes.
///
/// **Generated output (C-0004): regenerated, never hand-edited.** A
/// hand-maintained master list becomes a second home for facts that already
/// live in the register, and a second copy is what ADR-0040 exists to
/// prevent. `check` compares this against a fresh generation and fails when
/// they disagree, which is ADR-0040's `docs.system_view_is_current` pattern.
///
/// CSV because it must be readable without this binary. The register's
/// fixity is otherwise only reachable through Rust, which makes the archive
/// exactly as durable as this crate.
///
/// # Errors
/// Fails if the store cannot be read or the index cannot be written.
pub fn index(store: &dyn Store, paths: &Paths, registry: &Registry) -> Result<Written> {
    let dir = paths.store.join("index");
    fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;

    let ledger = store.ledger()?;
    let register = store.register()?;
    let signatures = store.signatures()?;
    let current = crate::domain::identity::current(&ledger);

    let files = [
        ("roster.csv", roster_csv(store, &current, &register)),
        ("documents.csv", documents_csv(&register, &signatures)),
        ("README.txt", readme(current.len(), register.len())),
    ];
    let mut written = Vec::new();
    for (name, body) in files {
        let path = dir.join(name);
        fs::write(&path, body).with_context(|| format!("writing {}", path.display()))?;
        written.push(format!("index/{name}"));
    }

    let mpath = paths.store.join("manifest-sha256.txt");
    fs::write(&mpath, payload_manifest(paths, &register)?)
        .with_context(|| format!("writing {}", mpath.display()))?;
    written.push("manifest-sha256.txt".to_owned());

    let _ = registry;
    Ok(Written {
        files: written,
        people: current.len(),
        documents: register.len(),
    })
}

/// One row per person, with their current engagement.
fn roster_csv(
    store: &dyn Store,
    current: &[&crate::domain::identity::Record],
    register: &[crate::domain::register::Entry],
) -> String {
    let mut out = row(&[
        "person_id",
        "engagement_id",
        "class",
        "standing",
        "standing_changed_on",
        "created_on",
        "slug",
        "legal_name",
        "role",
        "start_date",
        "end_date",
        "documents",
        "cloud_folder",
    ]);
    for r in current {
        let person = if r.standing == Standing::Erased {
            None
        } else {
            store.person(&r.slug).ok()
        };
        let get = |path: &str| -> String {
            person
                .as_ref()
                .and_then(|p| p.string(path))
                .unwrap_or_default()
                .to_owned()
        };
        let docs = register
            .iter()
            .filter(|e| e.person_id == r.person_id)
            .count();
        out.push_str(&row(&[
            r.person_id.as_str(),
            r.engagement_id.as_str(),
            &r.class,
            r.standing.as_str(),
            &r.standing_changed_on,
            &r.created_on,
            if r.standing == Standing::Erased {
                "(erased)"
            } else {
                &r.slug
            },
            &get("identity.legal_name"),
            &get("engagement.role"),
            &get("engagement.start_date"),
            &get("engagement.end_date"),
            &docs.to_string(),
            &get("access.cloud_folder"),
        ]));
    }
    out
}

/// One row per issued document.
fn documents_csv(
    register: &[crate::domain::register::Entry],
    signatures: &[crate::domain::signature::Record],
) -> String {
    let sig = |e: &crate::domain::register::Entry| {
        crate::domain::signature::for_serial(signatures, &e.serial)
    };
    let mut out = row(&[
        "serial",
        "code",
        "title",
        "kind",
        "person_id",
        "engagement_id",
        "issued_on",
        "retention",
        "pdf_path",
        "pdf_sha256",
        "texlive",
        "signed_on",
        "signed_method",
        "scan_sha256",
    ]);
    for e in register {
        out.push_str(&row(&[
            &e.serial,
            &e.code,
            &e.title,
            &e.kind,
            e.person_id.as_str(),
            e.engagement_id.as_str(),
            &e.issued_on,
            &e.retention,
            &e.pdf_path,
            &e.pdf_sha256,
            &e.texlive,
            sig(e).map_or("", |s| s.signed_on.as_str()),
            sig(e).map_or("", |s| s.method.as_str()),
            sig(e).and_then(|s| s.scan_sha256.as_deref()).unwrap_or(""),
        ]));
    }
    out
}

/// The finding aid: how to read this archive without any Paridyum tooling.
fn readme(people: usize, documents: usize) -> String {
    format!(
        "Paridyum document archive\n\
         =========================\n\n\
         GENERATED by `docket index`. Do not edit: it is regenerated from the\n\
         registers and `docket check` fails if it is stale.\n\n\
         roster.csv     one row per person, with their current engagement\n\
         documents.csv  one row per issued document\n\
         manifest-sha256.txt   SHA-256 of every issued PDF\n\n\
         To verify every document without any Paridyum tooling, from the store\n\
         root:\n\n\
         \x20 shasum -a 256 -c manifest-sha256.txt\n\n\
         Folders under issued/ are named by person identifier, which is opaque\n\
         and permanent (ADR-0051). roster.csv maps an identifier to a person.\n\
         An identifier ends in an ISO 7064 MOD 11-2 check character.\n\n\
         Documents were typeset with TeX Live {}. Output is PDF/A-2b\n\
         (ISO 19005-2), so the files render without that toolchain.\n\n\
         {people} people, {documents} documents.\n",
        crate::adapters::typeset::TEXLIVE,
    )
}

/// BagIt-style payload manifest, with paths relative to the store root so
/// they resolve wherever the store is mounted.
fn payload_manifest(paths: &Paths, register: &[crate::domain::register::Entry]) -> Result<String> {
    let mut out = String::new();
    for e in register {
        let p = paths.store.join(&e.pdf_path);
        if p.exists() {
            let bytes = fs::read(&p).with_context(|| format!("reading {}", p.display()))?;
            let _ = writeln!(out, "{}  {}", sha256(&bytes), e.pdf_path);
        }
    }
    Ok(out)
}

/// A document whose retention period has elapsed.
#[derive(Debug)]
pub struct Overdue {
    /// Serial of the document.
    pub serial: String,
    /// Person it was issued to.
    pub person_id: String,
    /// Retention class that governs it.
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

/// Documents past their retention period.
///
/// **Deliberately conservative.** `retention.yaml` states periods in prose
/// because the basis matters more than the arithmetic, so only the classes
/// with an unambiguous numeric period are evaluated here. A class this cannot
/// parse is reported as unevaluated rather than assumed compliant -- silently
/// treating "deleted within 30 days" as satisfied is how over-retention
/// becomes invisible.
///
/// # Errors
/// Fails if the store or registry cannot be read.
pub fn retention_due(
    store: &dyn Store,
    registry: &Registry,
    today: time::Date,
) -> Result<(Vec<Overdue>, Vec<String>)> {
    let register = store.register()?;
    let ledger = store.ledger()?;
    let mut due = Vec::new();
    let mut unevaluated: Vec<String> = Vec::new();

    for class in &registry.retention {
        if class.from != Epoch::Never && parse_period_days(&class.keep_for).is_none() {
            unevaluated.push(format!(
                "{} ({}) -- the period is prose this cannot turn into a number",
                class.class,
                class.keep_for.trim()
            ));
        }
    }

    for e in &register {
        let Some(class) = registry.retention.iter().find(|c| c.class == e.retention) else {
            continue;
        };
        if class.from == Epoch::Never {
            continue;
        }
        let Some(days) = parse_period_days(&class.keep_for) else {
            continue;
        };

        // The engagement has to have ENDED for the clock to start. While a
        // person is still engaged nothing they hold is over-retained.
        let (basis, since) = if class.from == Epoch::Issue {
            ("issued", e.issued_on.clone())
        } else if let Some(end) = engagement_end(&ledger, e) {
            ("engagement ended", end)
        } else {
            unevaluated.push(format!(
                "{}: {} is measured from the end of the engagement, which has not \
                 ended or is not recorded",
                e.serial, class.class
            ));
            continue;
        };

        let Ok(from) = clock::parse_iso(&since) else {
            continue;
        };
        let elapsed = (today - from).whole_days();
        if elapsed > days {
            due.push(Overdue {
                serial: e.serial.clone(),
                person_id: e.person_id.to_string(),
                class: class.class.clone(),
                keep_for: class.keep_for.trim().to_owned(),
                basis: basis.to_owned(),
                since,
                days: elapsed,
            });
        }
    }
    Ok((due, unevaluated))
}

/// When a person's engagement ended, if it has.
///
/// The separation date from the ledger where there is one, otherwise the end
/// date on the record. A person still engaged returns `None`: their documents
/// cannot be over-retained while the engagement they belong to is running.
fn engagement_end(
    ledger: &[crate::domain::identity::Record],
    entry: &crate::domain::register::Entry,
) -> Option<String> {
    // Separation is what ends an engagement. A person still active has an open
    // engagement, so nothing they hold can be over-retained yet -- returning
    // None here is what keeps the emergency contact from being flagged for
    // deletion in the middle of an internship.
    ledger
        .iter()
        .filter(|r| r.person_id == entry.person_id)
        .filter(|r| matches!(r.standing, Standing::Separated | Standing::Erased))
        .map(|r| r.standing_changed_on.clone())
        .next()
}

/// Days in a prose retention period, when it is unambiguous.
///
/// Recognises "N years" and "N days". Returns `None` for "indefinitely" and
/// for anything it does not understand, so an unparsed class is visible
/// rather than silently compliant.
#[must_use]
pub fn parse_period_days(text: &str) -> Option<i64> {
    let lower = text.to_lowercase();
    if lower.contains("indefinite") {
        return None;
    }
    let mut number: Option<i64> = None;
    for token in lower.split_whitespace() {
        if let Ok(n) = token.parse::<i64>() {
            number = Some(n);
        } else if number.is_some() {
            if token.starts_with("year") {
                return number.map(|n| n.saturating_mul(365));
            }
            if token.starts_with("day") {
                return number;
            }
            if token.starts_with("month") {
                return number.map(|n| n.saturating_mul(30));
            }
        }
    }
    None
}

/// Copies of a person's data that erasure cannot itself reach.
///
/// `docket erase` deletes the record and the event detail. It cannot reach a
/// cloud folder, a mail attachment, a backup, or a printout. Naming them is
/// the honest thing: a tool that reports a complete erasure it did not
/// perform is worse than one that states its boundary.
#[must_use]
pub fn unreachable_copies(cloud_folder: Option<&str>, person_id: &str) -> Vec<String> {
    let mut out = vec![
        format!(
            "issued PDFs under store/issued/{person_id}/ are retained \
             deliberately -- the register still refers to them; delete them \
             separately if the retention basis has also ended"
        ),
        "any backup or snapshot of the store taken before now".to_owned(),
        "documents sent by mail, and their attachments in sent folders".to_owned(),
        "printed or scanned copies held physically".to_owned(),
    ];
    match cloud_folder {
        Some(url) if !url.trim().is_empty() => out.insert(
            0,
            format!(
                "the remote folder {url} -- delete it, then empty the provider's \
                 trash and purge version history; a provider 'delete' typically \
                 retains the file for a further period"
            ),
        ),
        _ => out.insert(
            0,
            "no remote folder is recorded for this person; if one exists it must \
             be deleted by hand"
                .to_owned(),
        ),
    }
    out
}

/// Verify every issued PDF against the manifest, without the register.
///
/// # Errors
/// Fails if the manifest cannot be read.
pub fn verify_manifest(paths: &Paths) -> Result<Vec<String>> {
    let path = paths.store.join("manifest-sha256.txt");
    if !path.exists() {
        return Ok(vec![
            "no manifest-sha256.txt; run `docket index`".to_owned(),
        ]);
    }
    let text = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let mut problems = Vec::new();
    for line in text.lines() {
        let Some((hash, rel)) = line.split_once("  ") else {
            continue;
        };
        let p: &Path = &paths.store.join(rel);
        if !p.exists() {
            problems.push(format!("{rel}: listed in the manifest but missing"));
            continue;
        }
        let bytes = fs::read(p).with_context(|| format!("reading {}", p.display()))?;
        if sha256(&bytes) != hash {
            problems.push(format!("{rel}: does not match its manifest hash"));
        }
    }
    Ok(problems)
}

#[cfg(test)]
// A test that cannot assert an expected value is not a test. The lint stays
// denied for the crate; only assertions inside test modules are exempt.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::{field, parse_period_days, unreachable_copies};

    #[test]
    fn periods_in_prose_are_parsed_when_unambiguous() {
        assert_eq!(
            parse_period_days("8 years after the internship ends"),
            Some(2920)
        );
        assert_eq!(
            parse_period_days("deleted within 30 days of the last working day"),
            Some(30)
        );
        assert_eq!(
            parse_period_days("3 years after the internship ends"),
            Some(1095)
        );
    }

    #[test]
    fn an_indefinite_period_is_not_a_deadline() {
        assert_eq!(parse_period_days("indefinitely"), None);
    }

    #[test]
    fn an_unparseable_period_is_none_rather_than_a_guess() {
        // Reported as unevaluated. Guessing a number here would silently
        // license over-retention, which is the failure nobody notices.
        assert_eq!(parse_period_days("until the matter is closed"), None);
        assert_eq!(parse_period_days(""), None);
    }

    #[test]
    fn csv_fields_survive_a_comma_and_a_quote() {
        assert_eq!(field("Verma, Asha"), "\"Verma, Asha\"");
        assert_eq!(field("she said \"no\""), "\"she said \"\"no\"\"\"");
    }

    #[test]
    fn erasure_names_the_remote_folder_when_one_is_recorded() {
        let with = unreachable_copies(Some("https://drive.example/abc"), "PAR-P-000001-X");
        assert!(with.first().is_some_and(|s| s.contains("drive.example")));
        assert!(with.iter().any(|s| s.contains("version history")));
    }

    #[test]
    fn erasure_says_so_when_no_folder_is_recorded() {
        let without = unreachable_copies(None, "PAR-P-000001-X");
        assert!(
            without
                .first()
                .is_some_and(|s| s.contains("no remote folder is recorded"))
        );
        assert!(without.iter().any(|s| s.contains("backup")));
    }
}
