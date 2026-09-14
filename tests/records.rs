//! End-to-end personnel record keeping against a throwaway store.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::PathBuf;

use tool_docket::adapters::blobs::FileBlobs;
use tool_docket::adapters::clock::parse_iso;
use tool_docket::adapters::registry::Registry;
use tool_docket::adapters::store::{self, FileStore};
use tool_docket::application::{check, personnel, roster};
use tool_docket::config::Paths;
use tool_docket::domain::event::Kind;
use tool_docket::domain::identity::Standing;
use tool_docket::domain::personnel::Violation;

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
    tool_docket::adapters::typeset::detect().is_ok()
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
    let defs = tool_docket::application::issue::definitions(&reg, Some(&person), None, date());
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
    let defs = tool_docket::application::issue::definitions(&reg, Some(&person), None, date());
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
    let mut defs = tool_docket::application::issue::definitions(&reg, Some(&person), None, date());
    tool_docket::application::issue::add_history(&mut defs, &FileStore::new(&sb.paths), &person)
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
    let mut defs = tool_docket::application::issue::definitions(&reg, Some(&person), None, date());
    tool_docket::application::issue::add_history(&mut defs, &FileStore::new(&sb.paths), &person)
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
    let owed = tool_docket::application::issue::outstanding(&reg, &person, &who, &[])
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
                tool_docket::adapters::registry::Epoch::Never,
                "{}",
                c.class
            );
        } else if stated.contains("ends") || stated.contains("working day") {
            assert_eq!(
                c.from,
                tool_docket::adapters::registry::Epoch::EngagementEnd,
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
    use tool_docket::application::pack::Action;

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
        tool_docket::application::issue::one(
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
        tool_docket::application::pack::grouped(&FileStore::new(&sb.paths), &reg, "t", None)
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
    tool_docket::application::issue::one(
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
    let p = tool_docket::application::pack::assemble(
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
        assert_eq!(tool_docket::domain::chain::sha256(&bytes), hash, "{rel}");
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
    let err = tool_docket::application::pack::assemble(
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
    tool_docket::application::issue::one(
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
    let first = tool_docket::application::pack::assemble(
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

    let again = tool_docket::application::pack::assemble(
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
    let due = tool_docket::application::comms::due(&FileStore::new(&sb.paths), &reg, "t", None)
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
    let due = tool_docket::application::comms::due(&FileStore::new(&sb.paths), &reg, "t", None)
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
        tool_docket::application::comms::due(&FileStore::new(&sb.paths), &reg, "adult", None)
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
        tool_docket::application::comms::due(&FileStore::new(&sb.paths), &reg, "child", None)
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
    let due = tool_docket::application::comms::due(&FileStore::new(&sb.paths), &reg, "t", None)
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

    let first = tool_docket::application::next::steps(
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

    let steps = tool_docket::application::next::steps(
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
    let steps = tool_docket::application::next::steps(
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
    let steps = tool_docket::application::next::steps(
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
