//! End-to-end personnel record keeping against a throwaway store.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::PathBuf;

use platform_docket::adapters::blobs::FileBlobs;
use platform_docket::adapters::clock::parse_iso;
use platform_docket::adapters::registry::Registry;
use platform_docket::adapters::store::{self, FileStore};
use platform_docket::application::{check, personnel, roster};
use platform_docket::config::Paths;
use platform_docket::domain::event::Kind;
use platform_docket::domain::identity::Standing;
use platform_docket::domain::personnel::Violation;

mod common;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

struct Sandbox {
    /// Held for its lifetime: dropping it deletes the store.
    dir: tempfile::TempDir,
    paths: Paths,
}

impl Sandbox {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("temp dir");
        let people = dir.path().join("people");
        std::fs::create_dir_all(&people).expect("people dir");
        // `docket new` copies the checked-in template, so it must be present.
        common::filled_template(&root(), &people.join("_TEMPLATE.yaml"));
        let paths = Paths {
            root: root(),
            people,
            store: dir.path().join("store"),
        };
        Self { dir, paths }
    }

    fn edit(&self, slug: &str, edits: &[(&str, &str)]) {
        let path = self.paths.person(slug);
        let mut body = std::fs::read_to_string(&path).expect("record");
        for (from, to) in edits {
            body = body.replace(from, to);
        }
        std::fs::write(&path, body).expect("write record");
    }
}

fn registry() -> Registry {
    Registry::load(&Paths::resolve(&root())).expect("registry loads")
}

fn date() -> time::Date {
    parse_iso("2026-08-19").expect("fixed date")
}

/// Whether a typesetter is installed.
///
/// The tests below render real PDFs, because the guarantees they make are about
/// output bytes and a mocked typesetter cannot make them. A GitHub runner has
/// neither TeX Live nor Tectonic, so they SKIP there rather than fail — the
/// same pattern `pdflatex_is_preferred_when_present` already uses.
///
/// Skipping is not free: it means CI does not exercise rendering. That is
/// recorded as decision debt rather than hidden, and the fix is to install a
/// typesetter in CI, not to weaken the test.
fn typesetter_available() -> bool {
    platform_docket::adapters::typeset::detect().is_ok()
}

/// Print once why a test did nothing, so a green run is not mistaken for
/// coverage.
fn skip_without_typesetter(test: &str) -> bool {
    if typesetter_available() {
        return false;
    }
    eprintln!("SKIP {test}: no typesetter installed (see DECISION-DEBT D-35)");
    true
}

#[test]
fn creating_a_person_allocates_an_identifier_and_a_record() {
    let sb = Sandbox::new();
    let r = personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &registry(),
        "asha-verma",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    assert_eq!(
        r.person_id.as_str(),
        "PAR-P-000001-X",
        "opaque, no class, no year"
    );
    assert_eq!(
        r.engagement_id.as_str(),
        "PAR-INT-2026-0001",
        "class and year here"
    );
    assert_eq!(r.standing, Standing::Active);
    assert!(
        sb.paths.person("asha-verma").exists(),
        "a record file is written"
    );
}

#[test]
fn person_sequences_are_global_and_engagement_sequences_are_scoped() {
    let sb = Sandbox::new();
    let reg = registry();
    let a = personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "a",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    let b = personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "b",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    let c = personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "c",
        "employee",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");

    // A person identifier counts PEOPLE and carries no class or year.
    assert_eq!(a.person_id.as_str(), "PAR-P-000001-X");
    assert_eq!(b.person_id.as_str(), "PAR-P-000002-8");
    assert_eq!(c.person_id.as_str(), "PAR-P-000003-6");

    // An engagement identifier counts ENGAGEMENTS within a class and year,
    // which is the fact that cannot go stale (ADR-0051).
    assert_eq!(a.engagement_id.as_str(), "PAR-INT-2026-0001");
    assert_eq!(b.engagement_id.as_str(), "PAR-INT-2026-0002");
    assert_eq!(c.engagement_id.as_str(), "PAR-EMP-2026-0001");
}

#[test]
fn an_unknown_class_names_the_ones_that_exist() {
    let sb = Sandbox::new();
    let err = personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &registry(),
        "a",
        "freelancer",
        "PAR",
        date(),
        "rohan",
    )
    .err()
    .map(|e| e.to_string())
    .unwrap_or_default();
    assert!(
        err.contains("intern"),
        "should list declared classes: {err}"
    );
}

#[test]
fn a_duplicate_slug_is_refused() {
    let sb = Sandbox::new();
    let reg = registry();
    personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "a",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    assert!(
        personnel::create(
            &FileStore::new(&sb.paths),
            &sb.paths,
            &reg,
            "a",
            "intern",
            "PAR",
            date(),
            "rohan"
        )
        .is_err()
    );
}

#[test]
fn collecting_a_statutory_identifier_for_an_unpaid_intern_is_refused() {
    // The finding this whole mechanism exists for. An unpaid student intern
    // is not an employee; no statute requires their PAN; collecting it is
    // processing personal data for which no purpose exists.
    let sb = Sandbox::new();
    let reg = registry();
    personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "t",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    sb.edit(
        "t",
        &[("version: 1", "version: 1\nstatutory:\n  pan: ABCDE1234F")],
    );

    let d = personnel::dossier(&FileStore::new(&sb.paths), &reg, "t", None).expect("dossier");
    assert!(
        d.violations
            .iter()
            .any(|v| matches!(v, Violation::Forbidden { path, .. }
            if path == "statutory.pan")),
        "expected statutory.pan to be forbidden, got {:?}",
        d.violations
    );
}

#[test]
fn an_employee_may_hold_statutory_identifiers_but_not_in_the_person_record() {
    let sb = Sandbox::new();
    let reg = registry();
    personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "e",
        "employee",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    sb.edit(
        "e",
        &[
            ("class: intern", "class: employee"),
            (
                "version: 1",
                "version: 1\nstatutory:\n  pan: ABCDE1234F\ncompensation:\n  basis: salary",
            ),
        ],
    );
    // The class PERMITS them, unlike an intern's — no `Forbidden` violation.
    // They still belong in the `statutory` store rather than in the person
    // record, so `WrongStore` is expected and is the correct outcome.
    let d = personnel::dossier(&FileStore::new(&sb.paths), &reg, "e", None).expect("dossier");
    assert!(
        d.violations
            .iter()
            .all(|v| !matches!(v, Violation::Forbidden { .. })),
        "an employee is permitted these: {:?}",
        d.violations
    );
    assert!(
        d.violations
            .iter()
            .all(|v| matches!(v, Violation::WrongStore { .. })),
        "and the only complaint is which file they are in: {:?}",
        d.violations
    );
}

#[test]
fn access_still_open_is_derived_from_recorded_events() {
    let sb = Sandbox::new();
    let reg = registry();
    let r = personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "t",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    for (kind, detail) in [
        (Kind::AccessGranted, "github"),
        (Kind::AccessGranted, "vpn"),
        (Kind::AccessRevoked, "github"),
    ] {
        personnel::record_event(
            &FileStore::new(&sb.paths),
            &r.person_id,
            kind,
            Some(detail),
            "test",
            date(),
        )
        .expect("records");
    }
    let d = personnel::dossier(&FileStore::new(&sb.paths), &reg, "t", None).expect("dossier");
    assert_eq!(d.open_access, ["vpn"]);
}

#[test]
fn separation_keeps_the_record_and_its_documents() {
    let sb = Sandbox::new();
    let reg = registry();
    personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "t",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    let r =
        personnel::separate(&FileStore::new(&sb.paths), "t", date(), "rohan").expect("separates");
    assert_eq!(r.standing, Standing::Separated);
    assert!(sb.paths.person("t").exists(), "separation is not erasure");
    assert!(
        personnel::separate(&FileStore::new(&sb.paths), "t", date(), "rohan").is_err(),
        "not twice"
    );
}

#[test]
fn erasure_deletes_the_record_but_keeps_the_identifier() {
    // DPDP erasure has to actually delete something, and the register still
    // has to resolve the serials issued under that identifier.
    let sb = Sandbox::new();
    let reg = registry();
    let created = personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "t",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    personnel::record_event(
        &FileStore::new(&sb.paths),
        &created.person_id,
        Kind::AccessGranted,
        Some("github"),
        "test",
        date(),
    )
    .expect("records");

    let erased = personnel::erase(
        &FileStore::new(&sb.paths),
        &FileBlobs::new(&sb.paths),
        &sb.paths,
        "t",
        date(),
        "rohan",
    )
    .expect("erases");
    assert_eq!(erased.standing, Standing::Erased);
    assert_eq!(
        erased.person_id, created.person_id,
        "the identifier survives"
    );
    assert!(!sb.paths.person("t").exists(), "the record file is gone");

    let log = store::events(&sb.paths).expect("events");
    let theirs: Vec<_> = log
        .iter()
        .filter(|e| e.person_id == created.person_id)
        .collect();
    assert!(
        theirs.iter().all(|e| e.detail.is_none()),
        "detail must not survive erasure: {theirs:?}"
    );
    assert!(
        theirs.iter().all(|e| e.kind.survives_erasure()),
        "only skeletal events survive: {theirs:?}"
    );
}

#[test]
fn an_erased_identifier_is_never_reissued() {
    let sb = Sandbox::new();
    let reg = registry();
    personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "a",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    personnel::erase(
        &FileStore::new(&sb.paths),
        &FileBlobs::new(&sb.paths),
        &sb.paths,
        "a",
        date(),
        "rohan",
    )
    .expect("erases");
    let next = personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "b",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    assert_eq!(
        next.person_id.as_str(),
        "PAR-P-000002-8",
        "the erased number is not reused"
    );
}

#[test]
fn every_chain_still_verifies_after_an_erasure() {
    // Erasure rewrites the event log, which is the one place this system
    // does not append. The chain must be re-sealed, not merely truncated.
    let sb = Sandbox::new();
    let reg = registry();
    let a = personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "a",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "b",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    personnel::record_event(
        &FileStore::new(&sb.paths),
        &a.person_id,
        Kind::AccessGranted,
        Some("x"),
        "t",
        date(),
    )
    .expect("records");
    personnel::erase(
        &FileStore::new(&sb.paths),
        &FileBlobs::new(&sb.paths),
        &sb.paths,
        "a",
        date(),
        "rohan",
    )
    .expect("erases");

    let findings = check::run(
        &FileStore::new(&sb.paths),
        &FileBlobs::new(&sb.paths),
        &sb.paths,
        &reg,
    )
    .expect("check runs");
    let blocking: Vec<&str> = findings
        .items
        .iter()
        .filter(|f| f.blocking)
        .map(|f| f.message.as_str())
        .collect();
    assert!(blocking.is_empty(), "{blocking:#?}");
}

#[test]
fn an_erased_person_stays_on_the_roster_as_a_tombstone() {
    // A roster that lost people when their data was erased would not be a
    // register of who was engaged.
    let sb = Sandbox::new();
    let reg = registry();
    personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "a",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    personnel::erase(
        &FileStore::new(&sb.paths),
        &FileBlobs::new(&sb.paths),
        &sb.paths,
        "a",
        date(),
        "rohan",
    )
    .expect("erases");

    let lines = roster::build(&FileStore::new(&sb.paths), &reg).expect("roster");
    assert_eq!(lines.len(), 1);
    let line = lines.first().expect("one line");
    assert_eq!(line.standing, Standing::Erased);
    assert_eq!(line.person_id, "PAR-P-000001-X");
    assert!(line.name.is_none(), "no personal data on a tombstone");
    assert_eq!(roster::tally(&lines), (0, 0, 1));
    let _ = sb.dir.path();
}

#[test]
fn a_record_created_by_hand_is_reported_rather_than_trusted() {
    // A file dropped into people/ has no identifier, so nothing ties its
    // documents to a person in the ledger.
    let sb = Sandbox::new();
    let reg = registry();
    std::fs::copy(
        root().join("people").join("_TEMPLATE.yaml"),
        sb.paths.person("smuggled"),
    )
    .expect("copy");
    let findings = check::run(
        &FileStore::new(&sb.paths),
        &FileBlobs::new(&sb.paths),
        &sb.paths,
        &reg,
    )
    .expect("check runs");
    assert!(
        findings
            .items
            .iter()
            .any(|f| f.message.contains("has no identifier")),
        "{:#?}",
        findings.items
    );
}

#[test]
fn recorded_pronouns_reach_the_document() {
    // Regression: the record nests pronouns under `identity.pronouns`, which
    // flattens to \dkPIdentityPronouns*, while templates read the shorter
    // \dkPPronouns*. Without the alias the neutral default silently won and
    // a person was misgendered on a certificate they keep.
    let sb = Sandbox::new();
    let reg = registry();
    personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "p",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    sb.edit(
        "p",
        &[
            ("    subject: they", "    subject: she"),
            ("    object: them", "    object: her"),
            ("    possessive: their", "    possessive: her"),
        ],
    );
    let person = store::person(&sb.paths, "p").expect("record");
    let defs = platform_docket::application::issue::definitions(&reg, Some(&person), None, date());
    assert_eq!(defs.get("dkPPronounsSubject"), Some("she"));
    assert_eq!(defs.get("dkPPronounsObject"), Some("her"));
    assert_eq!(defs.get("dkPPronounsPossessive"), Some("her"));
}

#[test]
fn pronouns_default_to_they_them_when_not_recorded() {
    let sb = Sandbox::new();
    let reg = registry();
    personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "p",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    sb.edit(
        "p",
        &[(
            "  pronouns:\n    subject: they\n    object: them\n    possessive: their",
            "",
        )],
    );
    let person = store::person(&sb.paths, "p").expect("record");
    let defs = platform_docket::application::issue::definitions(&reg, Some(&person), None, date());
    assert_eq!(
        defs.get("dkPPronounsObject"),
        Some("them"),
        "never inferred from a name"
    );
}

#[test]
fn the_offboarding_checklist_names_access_still_open() {
    // The log knew which systems were granted while the checklist printed a
    // generic list. Deriving it means the paper cannot disagree with the log.
    let sb = Sandbox::new();
    let reg = registry();
    let r = personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "t",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    for (kind, detail) in [
        (Kind::AccessGranted, "github"),
        (Kind::AccessGranted, "vpn"),
        (Kind::AccessRevoked, "vpn"),
    ] {
        personnel::record_event(
            &FileStore::new(&sb.paths),
            &r.person_id,
            kind,
            Some(detail),
            "test",
            date(),
        )
        .expect("records");
    }
    let person = store::person(&sb.paths, "t").expect("record");
    let mut defs =
        platform_docket::application::issue::definitions(&reg, Some(&person), None, date());
    platform_docket::application::issue::add_history(
        &mut defs,
        &FileStore::new(&sb.paths),
        &person,
    )
    .expect("history");

    assert_eq!(defs.get("dkPAccessOpenCount"), Some("1"));
    let items = defs.get("dkPAccessOpenItems").unwrap_or_default();
    assert!(items.contains("github"), "{items}");
    assert!(
        !items.contains("vpn"),
        "revoked access must not be listed: {items}"
    );
}

#[test]
fn a_person_with_no_recorded_history_still_renders() {
    // A record with no events must not fail the build: \begin{itemize} with
    // no \item is a LaTeX error, so the empty case needs a body.
    let sb = Sandbox::new();
    let reg = registry();
    personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "t",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    let person = store::person(&sb.paths, "t").expect("record");
    let mut defs =
        platform_docket::application::issue::definitions(&reg, Some(&person), None, date());
    platform_docket::application::issue::add_history(
        &mut defs,
        &FileStore::new(&sb.paths),
        &person,
    )
    .expect("history");
    assert_eq!(defs.get("dkPAccessOpenCount"), Some("0"));
    assert!(
        defs.get("dkPAccessOpenItems")
            .is_some_and(|i| i.contains("\\item"))
    );
    assert!(
        defs.get("dkPDocsIssuedItems")
            .is_some_and(|i| i.contains("\\item"))
    );
}

#[test]
fn consent_is_owed_by_no_stage() {
    // A consent a lifecycle stage REQUIRES is not freely given, and a consent
    // that is not freely given is not consent. PUB must be offerable and never
    // demanded, so no stage may owe it.
    let reg = registry();
    let owed: Vec<&str> = reg
        .lifecycle
        .stages
        .iter()
        .flat_map(|s| s.owes.iter().map(String::as_str))
        .collect();
    assert!(
        !owed.contains(&"PUB"),
        "publicity consent must not be compulsory"
    );
}

#[test]
fn the_institution_letter_is_owed_only_when_the_institution_asks() {
    let sb = Sandbox::new();
    let reg = registry();
    personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "no-noc",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    sb.edit(
        "no-noc",
        &[(
            "institution_requires_noc: true",
            "institution_requires_noc: false",
        )],
    );
    let who = personnel::identify(&FileStore::new(&sb.paths), "no-noc").expect("identified");
    let person = store::person(&sb.paths, "no-noc").expect("record");
    let owed = platform_docket::application::issue::outstanding(&reg, &person, &who, &[])
        .expect("outstanding");
    assert!(!owed.contains(&"INS".to_owned()), "{owed:?}");
}

#[test]
fn the_conduct_and_grievance_policies_gate_the_move_to_active() {
    // They are acknowledged, not merely published: a route to complain that
    // the person never saw is not a route.
    let reg = registry();
    let stage = reg.lifecycle.stage("preboarding").expect("declared");
    for code in ["COC", "GRV"] {
        assert!(
            stage.requires_signed.iter().any(|c| c == code),
            "{code} must gate leaving preboarding: {:?}",
            stage.requires_signed
        );
    }
}

#[test]
fn every_retention_class_names_the_date_it_counts_from() {
    // Measuring a period stated as "after the internship ends" from the ISSUE
    // date is wrong by the whole length of the engagement, and for the
    // emergency contact it fires while the person is still working.
    for c in &registry().retention {
        let stated = c.keep_for.to_lowercase();
        if stated.contains("indefinite") {
            assert_eq!(
                c.from,
                platform_docket::adapters::registry::Epoch::Never,
                "{}",
                c.class
            );
        } else if stated.contains("ends") || stated.contains("working day") {
            assert_eq!(
                c.from,
                platform_docket::adapters::registry::Epoch::EngagementEnd,
                "{} states an end-relative period",
                c.class
            );
        }
    }
}

#[test]
fn the_checked_in_catalogue_is_internally_consistent() {
    // Empty store: repository validity must not depend on gitignored records.
    let sb = Sandbox::new();
    let paths = sb.paths.clone();
    let findings = check::run(
        &FileStore::new(&paths),
        &FileBlobs::new(&paths),
        &paths,
        &registry(),
    )
    .expect("check runs");
    let blocking: Vec<&str> = findings
        .items
        .iter()
        .filter(|f| f.blocking)
        .map(|f| f.message.as_str())
        .collect();
    assert!(blocking.is_empty(), "{blocking:#?}");
}

#[test]
fn every_declared_field_states_a_purpose() {
    // A field whose purpose cannot be written down should not be collected.
    for f in &registry().catalogue.fields {
        assert!(!f.purpose.trim().is_empty(), "{} has no purpose", f.path);
        assert!(
            f.purpose.len() > 15,
            "{}: purpose is too thin: {:?}",
            f.path,
            f.purpose
        );
    }
}

#[test]
fn the_pack_groups_documents_by_what_must_be_done_with_them() {
    use platform_docket::application::pack::Action;

    if skip_without_typesetter("the_pack_groups_documents_by_what_must_be_done_with_them") {
        return;
    }

    // "Which of these must I sign and send back" is the only question someone
    // opening the folder actually has, so that is what the folders answer.
    let sb = Sandbox::new();
    let reg = registry();
    personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "t",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    let person = store::person(&sb.paths, "t").expect("record");
    let by_code = reg.lifecycle.by_code().expect("codes");
    for code in ["NDA", "ONB", "INS", "PUB"] {
        let doc = by_code.get(code).expect("declared");
        platform_docket::application::issue::one(
            &FileStore::new(&sb.paths),
            &FileBlobs::new(&sb.paths),
            &sb.paths,
            &reg,
            &person,
            doc,
            date(),
            "rohan",
        )
        .expect("issues");
    }

    let grouped =
        platform_docket::application::pack::grouped(&FileStore::new(&sb.paths), &reg, "t", None)
            .expect("grouped");
    let find = |a: Action| -> Vec<String> {
        grouped
            .iter()
            .find(|(x, _)| *x == a)
            .map(|(_, v)| v.iter().map(|e| e.code.clone()).collect())
            .unwrap_or_default()
    };
    assert_eq!(find(Action::SignAndReturn), ["NDA"]);
    assert_eq!(find(Action::CompleteAndReturn), ["ONB"]);
    assert_eq!(find(Action::ForRecords), ["INS"]);
    assert_eq!(
        find(Action::Optional),
        ["PUB"],
        "no stage owes it, so it is optional"
    );
}

#[test]
fn the_pack_verifies_on_its_own_and_leaves_the_store_untouched() {
    if skip_without_typesetter("the_pack_verifies_on_its_own_and_leaves_the_store_untouched") {
        return;
    }
    let sb = Sandbox::new();
    let reg = registry();
    personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "t",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    let person = store::person(&sb.paths, "t").expect("record");
    let doc = reg
        .lifecycle
        .by_code()
        .expect("codes")
        .get("NDA")
        .copied()
        .expect("declared");
    platform_docket::application::issue::one(
        &FileStore::new(&sb.paths),
        &FileBlobs::new(&sb.paths),
        &sb.paths,
        &reg,
        &person,
        doc,
        date(),
        "rohan",
    )
    .expect("issues");

    let before = std::fs::read(sb.paths.register()).expect("register");
    let out = sb.dir.path().join("out");
    let p = platform_docket::application::pack::assemble(
        &FileStore::new(&sb.paths),
        &FileBlobs::new(&sb.paths),
        &sb.paths,
        &reg,
        "t",
        None,
        &out,
        date(),
        "rohan",
    )
    .expect("assembles");

    // Every listed hash matches the file it names.
    let manifest = std::fs::read_to_string(p.root.join("MANIFEST-sha256.txt")).expect("manifest");
    assert!(!manifest.is_empty());
    for line in manifest.lines() {
        let (hash, rel) = line.split_once("  ").expect("manifest line");
        let bytes = std::fs::read(p.root.join(rel)).expect("packed file");
        assert_eq!(
            platform_docket::domain::chain::sha256(&bytes),
            hash,
            "{rel}"
        );
    }
    assert!(
        p.root.join("00_Read_Me_First.pdf").exists(),
        "the cover sheet sorts first"
    );

    // Issuing the cover sheet appends; the earlier entries are untouched.
    let after = std::fs::read(sb.paths.register()).expect("register");
    assert!(after.starts_with(&before), "the register is append-only");
}

#[test]
fn packing_a_person_with_nothing_issued_is_refused() {
    if skip_without_typesetter("packing_a_person_with_nothing_issued_is_refused") {
        return;
    }
    let sb = Sandbox::new();
    let reg = registry();
    personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "t",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    let out = sb.dir.path().join("out");
    let err = platform_docket::application::pack::assemble(
        &FileStore::new(&sb.paths),
        &FileBlobs::new(&sb.paths),
        &sb.paths,
        &reg,
        "t",
        None,
        &out,
        date(),
        "rohan",
    )
    .err()
    .map(|e| e.to_string())
    .unwrap_or_default();
    assert!(err.contains("no issued documents"), "{err}");
}

#[test]
fn rebuilding_a_pack_replaces_it_rather_than_accumulating() {
    if skip_without_typesetter("rebuilding_a_pack_replaces_it_rather_than_accumulating") {
        return;
    }
    let sb = Sandbox::new();
    let reg = registry();
    personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "t",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    let person = store::person(&sb.paths, "t").expect("record");
    let doc = reg
        .lifecycle
        .by_code()
        .expect("codes")
        .get("NDA")
        .copied()
        .expect("declared");
    platform_docket::application::issue::one(
        &FileStore::new(&sb.paths),
        &FileBlobs::new(&sb.paths),
        &sb.paths,
        &reg,
        &person,
        doc,
        date(),
        "rohan",
    )
    .expect("issues");

    let out = sb.dir.path().join("out");
    let first = platform_docket::application::pack::assemble(
        &FileStore::new(&sb.paths),
        &FileBlobs::new(&sb.paths),
        &sb.paths,
        &reg,
        "t",
        None,
        &out,
        date(),
        "rohan",
    )
    .expect("assembles");
    let stray = first.root.join("01_Sign_and_return").join("99_stale.pdf");
    std::fs::write(&stray, b"left over from a previous pack").expect("write");

    let again = platform_docket::application::pack::assemble(
        &FileStore::new(&sb.paths),
        &FileBlobs::new(&sb.paths),
        &sb.paths,
        &reg,
        "t",
        None,
        &out,
        date(),
        "rohan",
    )
    .expect("reassembles");
    assert!(!stray.exists(), "a stale file must not survive a rebuild");
    assert!(again.root.join("MANIFEST-sha256.txt").exists());
}

#[test]
fn every_message_attaches_documents_that_exist() {
    // A message naming a document the lifecycle does not declare would tell
    // someone to attach a file that cannot be produced — the exact drift a
    // wiki page describing the same thing would have.
    let reg = registry();
    let by_code = reg.lifecycle.by_code().expect("codes");
    let stages: Vec<&str> = reg
        .lifecycle
        .stages
        .iter()
        .map(|s| s.name.as_str())
        .collect();
    for m in &reg.messages {
        assert!(
            stages.contains(&m.stage.as_str()),
            "{}: stage {:?}",
            m.id,
            m.stage
        );
        for code in &m.attach {
            assert!(
                by_code.contains_key(code.as_str()),
                "{}: attaches {code}",
                m.id
            );
        }
        assert!(!m.subject.trim().is_empty(), "{}: empty subject", m.id);
        assert!(!m.body.trim().is_empty(), "{}: empty body", m.id);
    }
}

#[test]
fn a_message_reports_attachments_that_have_not_been_issued() {
    // Better than silently listing a file that is not there: the operator is
    // about to attach it to a real email.
    let sb = Sandbox::new();
    let reg = registry();
    personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "t",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    let due = platform_docket::application::comms::due(&FileStore::new(&sb.paths), &reg, "t", None)
        .expect("due");
    let offer = due
        .iter()
        .find(|m| m.id == "offer")
        .expect("offer is due at `offered`");
    assert_eq!(offer.missing, ["OFR"], "nothing has been issued yet");
}

#[test]
fn messages_are_addressed_from_the_record() {
    let sb = Sandbox::new();
    let reg = registry();
    personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "t",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    let due = platform_docket::application::comms::due(&FileStore::new(&sb.paths), &reg, "t", None)
        .expect("due");
    let offer = due.iter().find(|m| m.id == "offer").expect("offer");
    assert_eq!(offer.address.as_deref(), Some("test.person@example.com"));
    assert!(
        offer.subject.contains("Software Engineering Intern"),
        "{}",
        offer.subject
    );
    assert!(offer.body.contains("Test"), "{}", offer.body);
    assert!(
        !offer.body.contains('{'),
        "no placeholder may survive: {}",
        offer.body
    );
}

#[test]
fn the_guardian_message_is_due_only_for_a_child() {
    let sb = Sandbox::new();
    let reg = registry();
    personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "adult",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    let adult =
        platform_docket::application::comms::due(&FileStore::new(&sb.paths), &reg, "adult", None)
            .expect("due");
    assert!(
        !adult.iter().any(|m| m.id == "guardian"),
        "no guardian, no message"
    );

    personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "child",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    sb.edit(
        "child",
        &[
            ("date_of_birth: 2005-04-12", "date_of_birth: 2009-04-12"),
            (
                "# guardian:\n#   name: ...",
                "guardian:\n  name: Sunita Verma\n  email: s@example.com\n# ...",
            ),
        ],
    );
    let child =
        platform_docket::application::comms::due(&FileStore::new(&sb.paths), &reg, "child", None)
            .expect("due");
    let g = child
        .iter()
        .find(|m| m.id == "guardian")
        .expect("guardian message is due");
    assert_eq!(
        g.address.as_deref(),
        Some("s@example.com"),
        "addressed to the guardian"
    );
}

#[test]
fn messages_are_cumulative_across_stages() {
    // Reaching `active` does not mean the preboarding messages were sent.
    let sb = Sandbox::new();
    let reg = registry();
    personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "t",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    sb.edit("t", &[("stage: preboarding", "stage: active")]);
    let due = platform_docket::application::comms::due(&FileStore::new(&sb.paths), &reg, "t", None)
        .expect("due");
    let ids: Vec<&str> = due.iter().map(|m| m.id.as_str()).collect();
    assert!(
        ids.contains(&"offer"),
        "an earlier stage's message is still listed: {ids:?}"
    );
    assert!(ids.contains(&"access"));
    assert!(
        !ids.contains(&"completion"),
        "a later stage's is not: {ids:?}"
    );
}

#[test]
fn next_walks_a_person_through_the_workflow_in_order() {
    // The ordering IS the workflow: someone following it from the top never
    // has to know the lifecycle.
    let sb = Sandbox::new();
    let reg = registry();
    personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "t",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");

    let first = platform_docket::application::next::steps(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "t",
        None,
    )
    .expect("steps");
    assert!(
        first.first().is_some_and(|s| s.what.contains("Issue")),
        "nothing is issued yet, so issuing is first: {:?}",
        first.first().map(|s| &s.what)
    );
}

#[test]
fn next_refuses_to_look_past_a_record_that_cannot_be_issued_against() {
    // A record that would produce a defective document blocks everything, so
    // the list stops at the fix rather than burying it under later steps.
    let sb = Sandbox::new();
    let reg = registry();
    personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "t",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    sb.edit(
        "t",
        &[("currently_enrolled: true", "currently_enrolled: false")],
    );

    let steps = platform_docket::application::next::steps(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "t",
        None,
    )
    .expect("steps");
    assert!(
        steps.iter().all(|s| s.what.starts_with("Fix the record")),
        "{steps:?}"
    );
    assert!(steps.iter().any(|s| s.what.contains("unpaid")), "{steps:?}");
}

#[test]
fn next_does_not_suggest_advancing_out_of_an_ungated_stage() {
    // `active` declares no signature gate. Suggesting the move from it would
    // tell an operator to offboard someone on their first day.
    let reg = registry();
    let active = reg.lifecycle.stage("active").expect("declared");
    assert!(
        active.requires_signed.is_empty(),
        "this test is about an ungated stage; if `active` gains a gate, revisit it"
    );

    let sb = Sandbox::new();
    personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "t",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    sb.edit("t", &[("stage: preboarding", "stage: active")]);
    let steps = platform_docket::application::next::steps(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "t",
        None,
    )
    .expect("steps");
    assert!(
        !steps.iter().any(|s| s.what.contains("next stage")),
        "must not suggest advancing from an ungated stage: {steps:?}"
    );
}

#[test]
fn next_says_nothing_remains_for_an_erased_person() {
    let sb = Sandbox::new();
    let reg = registry();
    personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "t",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    personnel::erase(
        &FileStore::new(&sb.paths),
        &FileBlobs::new(&sb.paths),
        &sb.paths,
        "t",
        date(),
        "rohan",
    )
    .expect("erases");
    let steps = platform_docket::application::next::steps(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "t",
        None,
    )
    .expect("steps");
    assert_eq!(steps.len(), 1);
    assert!(
        steps.first().is_some_and(|s| s.command.is_none()),
        "there is no command for an erased person"
    );
}

#[test]
fn a_counterparty_record_holding_a_contact_person_is_reported_by_check() {
    // RFC-0136: an organisation counterparty holds its legal name, address and receiving office,
    // and nothing else. A contact's email typed into the record by hand is over-collection, and
    // `check` must say so rather than the record quietly carrying it.
    let sb = Sandbox::new();
    let paths = sb.paths.clone();
    let facts = platform_docket::application::counterparty::Facts {
        legal_name: "Example University",
        address: "1 College Road, Dehradun",
        office: "The Registrar",
    };
    platform_docket::application::counterparty::enrol(
        &paths,
        "example-university",
        facts,
        parse_iso("2026-11-07").expect("date"),
        "rohan",
    )
    .expect("enrolled");
    let run = || {
        check::run(
            &FileStore::new(&paths),
            &FileBlobs::new(&paths),
            &paths,
            &registry(),
        )
        .expect("check runs")
    };
    assert!(
        !run()
            .items
            .iter()
            .any(|f| f.blocking && f.message.contains("counterparty")),
        "a well-formed record is clean"
    );

    let file = paths.counterparty("example-university");
    let body = std::fs::read_to_string(&file).expect("record");
    std::fs::write(&file, format!("{body}contact_email: someone@example.edu\n")).expect("edit");
    assert!(
        run()
            .items
            .iter()
            .any(|f| f.blocking && f.message.contains("holds only")),
        "the hand-added contact is reported"
    );
}

/// A document declared for an organisation, built as the registry would declare one (RFC-0136).
fn organisation_document(extra: &str) -> platform_docket::domain::lifecycle::Document {
    serde_yaml::from_str(&format!(
        "code: TST\nid: test-collaboration\ntitle: Test Letter\n\
         template: templates/letter/business-communication.tex\nkind: issued\n\
         recipient: organisation\nsigned_by: [director]\nretention: permanent\n\
         on_request: true\n{extra}"
    ))
    .expect("declaration parses")
}

fn example_counterparty(paths: &Paths) -> platform_docket::domain::counterparty::Counterparty {
    let facts = platform_docket::application::counterparty::Facts {
        legal_name: "Example University",
        address: "1 College Road, Dehradun",
        office: "The Registrar",
    };
    platform_docket::application::counterparty::enrol(
        paths,
        "example-university",
        facts,
        parse_iso("2026-11-07").expect("date"),
        "rohan",
    )
    .expect("enrolled")
}

#[test]
fn an_organisations_document_is_never_issued_to_a_person_nor_the_reverse() {
    // Refused before a serial is allocated: the register is append-only, so a document issued to
    // the wrong kind of recipient could never be taken back.
    use platform_docket::adapters::store::Store as _;
    use platform_docket::application::issue;
    let sb = Sandbox::new();
    let store = FileStore::new(&sb.paths);
    let blobs = FileBlobs::new(&sb.paths);
    let reg = registry();
    personnel::create(
        &store,
        &sb.paths,
        &reg,
        "asha-verma",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    let person = store.person("asha-verma").expect("record");
    let to_person = issue::one(
        &store,
        &blobs,
        &sb.paths,
        &reg,
        &person,
        &organisation_document(""),
        date(),
        "rohan",
    );
    assert!(format!("{:#}", to_person.expect_err("refused")).contains("organisation"));

    let org = example_counterparty(&sb.paths);
    let nda = reg
        .lifecycle
        .documents
        .iter()
        .find(|d| d.code == "NDA")
        .expect("NDA")
        .clone();
    let to_org = issue::to_counterparty(
        &store,
        &blobs,
        &sb.paths,
        &reg,
        &org,
        &nda,
        "PAR",
        date(),
        None,
    );
    assert!(format!("{:#}", to_org.expect_err("refused")).contains("issued to a person"));
    assert!(
        store.register().expect("register").is_empty(),
        "nothing was recorded"
    );
}

#[test]
fn an_organisations_document_is_signed_by_office_and_a_vacant_one_falls_through() {
    // Signing resolves through signatories.yaml exactly as for a person (ADR-0077 D4, ADR-0087):
    // the Director signs, and if that office is vacant the next active signatory does.
    use platform_docket::application::issue::counterparty_signatories;
    let doc = organisation_document("");
    let mut reg = registry();
    assert_eq!(counterparty_signatories(&reg, &doc, "PAR"), ["director"]);
    for s in &mut reg.signatories {
        if s.id == "director" {
            s.active = false;
        }
    }
    assert_eq!(counterparty_signatories(&reg, &doc, "PAR"), ["ceo"]);
}

#[test]
fn check_refuses_an_organisations_document_shaped_for_a_person() {
    let sb = Sandbox::new();
    let mut reg = registry();
    reg.lifecycle
        .documents
        .push(organisation_document("classes: [intern]\n"));
    let findings = check::run(
        &FileStore::new(&sb.paths),
        &FileBlobs::new(&sb.paths),
        &sb.paths,
        &reg,
    )
    .expect("check runs");
    assert!(
        findings
            .items
            .iter()
            .any(|f| f.blocking && f.message.contains("TST: issued to an organisation")),
        "{:?}",
        findings.items
    );
}

/// Every command `template` dereferences without a `\dkfill` fallback that an organisation
/// counterparty's issuance would leave undefined. Needs no typesetter.
fn undefined_for_a_counterparty(code: &str) -> Vec<String> {
    use platform_docket::application::issue;
    use platform_docket::domain::template::closure;
    let reg = registry();
    let doc = reg
        .lifecycle
        .documents
        .iter()
        .find(|d| d.code == code)
        .expect("declared");
    let body = closure(&root(), &root().join(&doc.template))
        .expect("template files")
        .into_iter()
        .map(|(_, bytes)| String::from_utf8_lossy(&bytes).into_owned())
        .collect::<String>()
        .lines()
        // A comment is not rendered; `\%` is a literal percent sign and is.
        .map(|l| l.replace("\\%", "").split('%').next().unwrap_or("").to_owned())
        .collect::<Vec<_>>()
        .join("\n");
    let sb = Sandbox::new();
    let mut defs = issue::definitions_for(&reg, None, None, date(), Some(doc));
    issue::counterparty_definitions(&mut defs, &example_counterparty(&sb.paths));
    defs.merge(&issue::negotiated_terms(doc, full_terms(doc).as_deref()).expect("terms"));
    // Every `\dk...` command, not only the `\dkP` ones `required_commands` scans for: this
    // template's data is the organisation's `\dkC...` and the company's `\dkOrg...`.
    let mut found: Vec<String> = body
        .split("\\dk")
        .skip(1)
        .map(|rest| {
            let name: String = rest
                .chars()
                .take_while(char::is_ascii_alphanumeric)
                .collect();
            format!("dk{name}")
        })
        // `\dkfill` is NOT exempt: a registered document must not be a blank form (review M5).
        // Only `\dkblank` is, for what is signed and dated by hand.
        .filter(|c| c != "dkblank")
        .filter(|c| defs.get(c).is_none())
        .collect();
    found.sort();
    found.dedup();
    found
}

#[test]
fn the_collaboration_letter_needs_nothing_an_organisation_does_not_supply() {
    // A field the counterparty cannot supply would stop pdflatex at issuance -- or, worse, tempt
    // someone to add a contact person's name to the record to make it render.
    assert!(
        undefined_for_a_counterparty("ICL").is_empty(),
        "{:?}",
        undefined_for_a_counterparty("ICL")
    );
}

#[test]
fn a_collaboration_letter_issued_to_an_organisation_registers_and_verifies() {
    use platform_docket::adapters::store::Store as _;
    use platform_docket::application::{issue, verify};
    if skip_without_typesetter(
        "a_collaboration_letter_issued_to_an_organisation_registers_and_verifies",
    ) {
        return;
    }
    let sb = Sandbox::new();
    let store = FileStore::new(&sb.paths);
    let blobs = FileBlobs::new(&sb.paths);
    let reg = registry();
    let org = example_counterparty(&sb.paths);
    let icl = reg
        .lifecycle
        .documents
        .iter()
        .find(|d| d.code == "ICL")
        .expect("ICL");
    let e = issue::to_counterparty(
        &store,
        &blobs,
        &sb.paths,
        &reg,
        &org,
        icl,
        "PAR",
        date(),
        None,
    )
    .expect("issued");
    assert_eq!(e.serial, "PAR/ICL/2026/0001");
    assert_eq!(e.counterparty_id.as_ref(), Some(&org.id));
    assert!(e.person_id.is_none() && e.engagement_id.is_none());
    assert_eq!(e.signed_by, ["director"]);
    let checked = verify::verify(
        &blobs,
        &store.register().expect("register"),
        &e.serial,
        None,
    )
    .expect("verifies");
    assert!(checked.matches());
}

#[test]
fn the_mou_is_refused_while_the_company_has_no_cin_and_only_for_that() {
    // RFC-0136 D7: a binding agreement waits for incorporation. org.yaml's `cin` is null today,
    // so the MOU is refused before a serial exists; with a CIN recorded the guard steps aside.
    use platform_docket::adapters::store::Store as _;
    use platform_docket::application::issue;
    let sb = Sandbox::new();
    let store = FileStore::new(&sb.paths);
    let blobs = FileBlobs::new(&sb.paths);
    let mut reg = registry();
    let org = example_counterparty(&sb.paths);
    let mou = reg
        .lifecycle
        .documents
        .iter()
        .find(|d| d.code == "MOU")
        .expect("MOU")
        .clone();
    let refused = issue::to_counterparty(
        &store,
        &blobs,
        &sb.paths,
        &reg,
        &org,
        &mou,
        "PAR",
        date(),
        None,
    );
    let why = format!("{:#}", refused.expect_err("refused before incorporation"));
    assert!(
        why.contains("not incorporated") && why.contains("cin"),
        "{why}"
    );
    assert!(
        store.register().expect("register").is_empty(),
        "no serial was spent"
    );

    let par = reg
        .org
        .get_mut("companies")
        .and_then(|c| c.as_sequence_mut())
        .and_then(|c| c.first_mut())
        .expect("PAR");
    par["cin"] = serde_yaml::Value::from("U00000UT2026PTC000000");
    // Past the guard: without a typesetter it now fails at rendering, and with one the template
    // still stops on the unrecorded registered office -- either way not on the guard.
    if let Err(e) = issue::to_counterparty(
        &store,
        &blobs,
        &sb.paths,
        &reg,
        &org,
        &mou,
        "PAR",
        date(),
        None,
    ) {
        assert!(!format!("{e:#}").contains("not incorporated"), "{e:#}");
    }
}

/// A terms file giving every term the document declares, or `None` where it declares none.
fn full_terms(doc: &platform_docket::domain::lifecycle::Document) -> Option<String> {
    (!doc.terms.is_empty()).then(|| {
        doc.terms.iter().fold(String::new(), |mut out, t| {
            out.push_str(&t.key);
            out.push_str(": Agreed ");
            out.push_str(&t.key);
            out.push('\n');
            out
        })
    })
}

/// Record PAR as incorporated on a registry copy: a CIN, and optionally a registered office.
fn incorporate(reg: &mut Registry, office: Option<&str>) {
    let par = reg
        .org
        .get_mut("companies")
        .and_then(|c| c.as_sequence_mut())
        .and_then(|c| c.first_mut())
        .and_then(|c| c.as_mapping_mut())
        .expect("PAR");
    par.insert("cin".into(), "U00000UT2026PTC000000".into());
    if let Some(office) = office {
        par.insert("registered_office".into(), office.into());
    }
}

fn mou() -> platform_docket::domain::lifecycle::Document {
    registry()
        .lifecycle
        .documents
        .into_iter()
        .find(|d| d.code == "MOU")
        .expect("MOU")
}

#[test]
fn the_mou_is_refused_without_its_terms_or_with_a_term_it_does_not_declare() {
    // Review M5: its negotiated terms are part of the registered document, so it is refused
    // without them -- before a serial -- even once incorporation has been recorded.
    use platform_docket::adapters::store::Store as _;
    use platform_docket::application::issue;
    let sb = Sandbox::new();
    let (store, blobs) = (FileStore::new(&sb.paths), FileBlobs::new(&sb.paths));
    let mut reg = registry();
    incorporate(&mut reg, None);
    let org = example_counterparty(&sb.paths);
    let doc = mou();
    let issue = |terms: Option<&str>| {
        issue::to_counterparty(
            &store,
            &blobs,
            &sb.paths,
            &reg,
            &org,
            &doc,
            "PAR",
            date(),
            terms,
        )
    };
    let why = format!("{:#}", issue(None).expect_err("refused"));
    assert!(
        why.contains("--terms") && why.contains("term_months"),
        "{why}"
    );
    let extra = format!(
        "{}contact_name: A Person\n",
        full_terms(&doc).expect("declared")
    );
    let why = format!("{:#}", issue(Some(&extra)).expect_err("refused"));
    assert!(why.contains("contact_name"), "{why}");
    assert!(
        store.register().expect("register").is_empty(),
        "no serial was spent"
    );
}

#[test]
fn with_its_cin_registered_office_and_terms_the_mou_renders_and_verifies() {
    use platform_docket::adapters::store::Store as _;
    use platform_docket::application::{issue, verify};
    // Review m7: shows the MOU template compiles once incorporation is recorded. Rendering, so it
    // SKIPS without a typesetter; CI's render job has TeX Live and runs it.
    if skip_without_typesetter(
        "with_its_cin_registered_office_and_terms_the_mou_renders_and_verifies",
    ) {
        return;
    }
    let sb = Sandbox::new();
    let (store, blobs) = (FileStore::new(&sb.paths), FileBlobs::new(&sb.paths));
    let mut reg = registry();
    incorporate(&mut reg, Some("1 Test Road, Dehradun"));
    let org = example_counterparty(&sb.paths);
    let doc = mou();
    let terms = full_terms(&doc);
    let e = issue::to_counterparty(
        &store,
        &blobs,
        &sb.paths,
        &reg,
        &org,
        &doc,
        "PAR",
        date(),
        terms.as_deref(),
    )
    .expect("issued");
    assert_eq!(e.counterparty_id.as_ref(), Some(&org.id));
    let checked = verify::verify(
        &blobs,
        &store.register().expect("register"),
        &e.serial,
        None,
    )
    .expect("verifies");
    assert!(checked.matches());
}

#[test]
fn a_kind_requiring_incorporation_is_refused_on_the_person_path_too() {
    use platform_docket::application::issue;
    let sb = Sandbox::new();
    let store = FileStore::new(&sb.paths);
    let reg = registry();
    personnel::create(
        &store,
        &sb.paths,
        &reg,
        "asha-verma",
        "intern",
        "PAR",
        date(),
        "rohan",
    )
    .expect("creates");
    let person = store::person(&sb.paths, "asha-verma").expect("record");
    let doc: platform_docket::domain::lifecycle::Document = serde_yaml::from_str(
        "code: TPA\nid: test-person-agreement\ntitle: Test\nclasses: [intern]\n\
         template: templates/agreement/nda.tex\nkind: signed\nsigned_by: [ceo]\n\
         retention: contract\nrequires_incorporation: true\n",
    )
    .expect("declaration");
    let refused = issue::one(
        &store,
        &FileBlobs::new(&sb.paths),
        &sb.paths,
        &reg,
        &person,
        &doc,
        date(),
        "rohan",
    );
    assert!(format!("{:#}", refused.expect_err("refused")).contains("not incorporated"));
}

#[test]
fn no_value_in_an_organisation_document_swallows_the_space_after_it() {
    // Re-review r3: a bare `\dkTTermMonths months` is a control word, and TeX eats the space
    // after it -- the sealed MOU printed "12months". Every command followed by a word needs `{}`.
    for code in ["ICL", "MOU"] {
        let doc = registry()
            .lifecycle
            .documents
            .into_iter()
            .find(|d| d.code == code)
            .expect("declared");
        let body = std::fs::read_to_string(root().join(&doc.template)).expect("template");
        for line in body.lines().map(|l| l.split('%').next().unwrap_or("")) {
            for rest in line.split("\\dk").skip(1) {
                let name: String = rest
                    .chars()
                    .take_while(char::is_ascii_alphanumeric)
                    .collect();
                let after = rest.get(name.len()..).unwrap_or("");
                let glued = after.starts_with(' ')
                    && after.chars().nth(1).is_some_and(char::is_alphanumeric);
                assert!(
                    !glued,
                    "{code}: \\dk{name} is followed by a word; write \\dk{name}{{}}"
                );
            }
        }
    }
}

#[test]
fn the_mou_needs_only_what_incorporation_will_record() {
    // The second guard: the template reads the CIN and registered office bare, so it cannot render
    // until org.yaml holds both. Everything else it needs, the counterparty and its declared terms
    // supply -- and it has no `\dkfill` left to print as a blank.
    assert_eq!(
        undefined_for_a_counterparty("MOU"),
        ["dkOrgCin", "dkOrgRegisteredOffice"]
    );
}

#[test]
fn check_reports_an_entry_naming_nobody_and_one_naming_an_unrecorded_organisation() {
    // Both are refused by the store on write; these arrive by hand edit or a restore.
    use platform_docket::domain::counterparty::CounterpartyId;
    use platform_docket::domain::register::Entry;
    let golden = include_str!("fixtures/register-v1.jsonl");
    let template: Entry =
        serde_json::from_str(golden.lines().next().expect("line")).expect("an entry");
    let run = |paths: &Paths| {
        check::run(
            &FileStore::new(paths),
            &FileBlobs::new(paths),
            paths,
            &registry(),
        )
        .expect("check runs and reports, rather than aborting")
    };
    let write = |paths: &Paths, e: Entry| {
        let sealed = e.seal(None).expect("seal");
        store::append(paths, &sealed).expect("raw append");
    };

    let sb = Sandbox::new();
    let mut org = template.clone();
    org.person_id = None;
    org.engagement_id = None;
    org.counterparty_id = Some(CounterpartyId::new(7).expect("id"));
    write(&sb.paths, org.clone());
    assert!(
        run(&sb.paths)
            .items
            .iter()
            .any(|f| f.blocking && f.message.contains("which no counterparty record holds"))
    );

    let sb = Sandbox::new();
    org.counterparty_id = None;
    write(&sb.paths, org);
    assert!(
        run(&sb.paths)
            .items
            .iter()
            .any(|f| f.blocking && f.message.contains("names no recipient"))
    );
}

#[test]
fn an_organisations_document_carries_the_issuing_companys_letterhead() {
    // Review m4: `--company PCY` chose PCY's signatory but printed the default company as the
    // party. And an undeclared company is named as such, not as a missing signatory.
    use platform_docket::application::issue;
    let sb = Sandbox::new();
    let reg = registry();
    let org = example_counterparty(&sb.paths);
    let doc = organisation_document("");
    let defs = issue::organisation_definitions(&reg, &org, &doc, "PCY", None, date(), &[])
        .expect("PCY is declared");
    assert_eq!(defs.get("dkOrgName"), Some("PARIDYUM CYBER"));
    assert_eq!(defs.get("dkCLegalName"), Some("Example University"));
    assert!(issue::organisation_definitions(&reg, &org, &doc, "ZZZ", None, date(), &[]).is_err());

    let store = FileStore::new(&sb.paths);
    let blobs = FileBlobs::new(&sb.paths);
    let refused = issue::to_counterparty(
        &store,
        &blobs,
        &sb.paths,
        &reg,
        &org,
        &doc,
        "ZZZ",
        date(),
        None,
    );
    assert!(format!("{:#}", refused.expect_err("refused")).contains("unknown company"));
}

#[test]
fn check_reports_a_deleted_counterparty_record_and_one_made_by_hand() {
    // The allocation ledger is the source; a record it did not allocate, or an allocation whose
    // record is gone, is reported rather than trusted.
    let sb = Sandbox::new();
    let paths = sb.paths.clone();
    let org = example_counterparty(&paths);
    let found = |paths: &Paths, what: &str| {
        check::run(
            &FileStore::new(paths),
            &FileBlobs::new(paths),
            paths,
            &registry(),
        )
        .expect("check runs")
        .items
        .iter()
        .any(|f| f.blocking && f.message.contains(what))
    };
    let body = std::fs::read_to_string(paths.counterparty(&org.slug)).expect("record");
    std::fs::write(paths.counterparty("hand-made"), &body).expect("copied by hand");
    assert!(found(&paths, "made by hand"));
    std::fs::remove_file(paths.counterparty("hand-made")).expect("tidy");
    std::fs::remove_file(paths.counterparty(&org.slug)).expect("deleted");
    assert!(found(&paths, "its record is gone"));
}
