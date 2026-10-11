//! End-to-end issuance against a throwaway store.
//!
//! Every case here is either a defect that reached a built PDF during
//! development, or a guarantee the register makes to someone presenting a
//! document years later.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::process::Command;

use platform_docket::adapters::blobs::FileBlobs;
use platform_docket::adapters::clock::parse_iso;
use platform_docket::adapters::registry::Registry;
use platform_docket::adapters::store::{self, FileStore};
use platform_docket::application::{check, issue, status, verify};
use platform_docket::config::Paths;
use platform_docket::domain::person::Person;

mod common;

const ISSUE_DATE: &str = "2026-08-19";

/// Repository root, from the crate manifest rather than the cwd.
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// A store and people directory that exist only for one test.
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
        // `personnel::create` copies the checked-in template.
        common::filled_template(&root(), &people.join("_TEMPLATE.yaml"));
        let paths = Paths {
            root: root(),
            people,
            store: dir.path().join("store"),
        };
        Self { dir, paths }
    }

    /// Write a person record derived from the checked-in template.
    /// Create a person the way an operator would, then apply edits.
    ///
    /// Goes through `personnel::create` rather than writing the file
    /// directly, so the identity ledger has an entry for them. A record with
    /// no identifier is a finding, not a valid fixture.
    fn person(&self, slug: &str, edits: &[(&str, &str)]) -> Person {
        platform_docket::application::personnel::create(
            &FileStore::new(&self.paths),
            &self.paths,
            &registry(),
            slug,
            "intern",
            "PAR",
            date(),
            "rohan",
        )
        .expect("creates");
        if !edits.is_empty() {
            let path = self.paths.person(slug);
            let mut body = std::fs::read_to_string(&path).expect("record");
            for (from, to) in edits {
                body = body.replace(from, to);
            }
            std::fs::write(&path, body).expect("write record");
        }
        store::person(&self.paths, slug).expect("read back")
    }
}

/// The ledger record for a slug, which the register now keys on.
fn who(sb: &Sandbox, slug: &str) -> platform_docket::domain::identity::Record {
    platform_docket::application::personnel::identify(&FileStore::new(&sb.paths), slug)
        .expect("identified")
}

fn registry() -> Registry {
    Registry::load(&Paths::resolve(&root())).expect("registry loads")
}

fn date() -> time::Date {
    parse_iso(ISSUE_DATE).expect("fixed date")
}

/// Issue one document by code.
fn issue_one(
    sb: &Sandbox,
    person: &Person,
    code: &str,
) -> platform_docket::domain::register::Entry {
    let reg = registry();
    let by_code = reg.lifecycle.by_code().expect("codes");
    let doc = by_code.get(code).expect("known code");
    issue::one(
        &FileStore::new(&sb.paths),
        &FileBlobs::new(&sb.paths),
        &sb.paths,
        &reg,
        person,
        doc,
        date(),
        "rohan",
    )
    .expect("issues")
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
fn the_checked_in_registry_is_valid() {
    // Guards the registry itself: a template renamed without updating
    // lifecycle.yaml would otherwise only surface at issuance.
    //
    // Runs against an EMPTY store and people directory. Person records are
    // gitignored, so whether the repository is valid must not depend on
    // whichever records happen to be on this machine.
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
    assert!(
        blocking.is_empty(),
        "registry has blocking findings: {blocking:#?}"
    );
}

#[test]
fn onboarding_issues_the_whole_preboarding_pack() {
    if skip_without_typesetter("onboarding_issues_the_whole_preboarding_pack") {
        return;
    }
    let sb = Sandbox::new();
    let person = sb.person("t", &[]);
    let reg = registry();

    let todo = issue::outstanding(&reg, &person, &who(&sb, "t"), &[]).expect("outstanding");
    // Grows as documents are added; assert the composition, not the count.
    for required in [
        "OFR", "AGR", "NDA", "IPA", "ISP", "PSH", "DPN", "COC", "GRV", "ONB", "EMC",
    ] {
        assert!(
            todo.contains(&required.to_owned()),
            "{required} must be owed: {todo:?}"
        );
    }

    let by_code = reg.lifecycle.by_code().expect("codes");
    for code in &todo {
        let doc = by_code.get(code.as_str()).expect("declared");
        issue::one(
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

    let register = store::register(&sb.paths).expect("register");
    assert_eq!(
        register.len(),
        todo.len(),
        "one register entry per issuance"
    );

    let report = status::report(&reg, &person, &who(&sb, "t"), &register, &[]).expect("report");
    assert_eq!(report.issued(), report.owed());
    assert!(report.outstanding().is_empty());
}

#[test]
fn re_issuing_the_same_stage_produces_nothing() {
    if skip_without_typesetter("re_issuing_the_same_stage_produces_nothing") {
        return;
    }
    let sb = Sandbox::new();
    let person = sb.person("t", &[]);
    let reg = registry();
    issue_one(&sb, &person, "NDA");
    let register = store::register(&sb.paths).expect("register");
    let todo = issue::outstanding(&reg, &person, &who(&sb, "t"), &register).expect("outstanding");
    assert!(
        !todo.contains(&"NDA".to_owned()),
        "NDA must not be issued twice"
    );
}

#[test]
fn a_second_person_gets_the_next_sequence_number() {
    if skip_without_typesetter("a_second_person_gets_the_next_sequence_number") {
        return;
    }
    let sb = Sandbox::new();
    let a = sb.person("a", &[]);
    let b = sb.person(
        "b",
        &[("legal_name: Asha Verma", "legal_name: Second Person")],
    );
    assert_eq!(issue_one(&sb, &a, "NDA").serial, "PAR/NDA/2026/0001");
    assert_eq!(issue_one(&sb, &b, "NDA").serial, "PAR/NDA/2026/0002");
}

#[test]
fn each_document_code_has_its_own_sequence() {
    if skip_without_typesetter("each_document_code_has_its_own_sequence") {
        return;
    }
    let sb = Sandbox::new();
    let p = sb.person("t", &[]);
    assert_eq!(issue_one(&sb, &p, "NDA").serial, "PAR/NDA/2026/0001");
    assert_eq!(issue_one(&sb, &p, "IPA").serial, "PAR/IPA/2026/0001");
}

#[test]
fn verify_matches_the_issued_document_and_rejects_a_modified_one() {
    if skip_without_typesetter("verify_matches_the_issued_document_and_rejects_a_modified_one") {
        return;
    }
    let sb = Sandbox::new();
    let p = sb.person("t", &[]);
    let entry = issue_one(&sb, &p, "NDA");
    let register = store::register(&sb.paths).expect("register");

    let ok = verify::verify(&FileBlobs::new(&sb.paths), &register, &entry.serial, None)
        .expect("verifies");
    assert!(ok.matches(), "the stored document must verify");

    let forged = sb.dir.path().join("forged.pdf");
    let mut bytes = std::fs::read(sb.paths.store.join(&entry.pdf_path)).expect("read pdf");
    bytes.push(b'x');
    std::fs::write(&forged, bytes).expect("write forged");
    let bad = verify::verify(
        &FileBlobs::new(&sb.paths),
        &register,
        &entry.serial,
        Some(&forged),
    )
    .expect("verifies");
    assert!(!bad.matches(), "a modified PDF must not verify");
}

#[test]
fn an_unknown_serial_is_an_error_not_a_pass() {
    let sb = Sandbox::new();
    let register = store::register(&sb.paths).expect("register");
    assert!(
        verify::verify(
            &FileBlobs::new(&sb.paths),
            &register,
            "PAR/NDA/1999/0001",
            None
        )
        .is_err()
    );
}

#[test]
fn issuing_the_same_document_twice_is_byte_identical() {
    if skip_without_typesetter("issuing_the_same_document_twice_is_byte_identical") {
        return;
    }
    // The register records a hash. If rendering were not reproducible, that
    // hash would be meaningless the moment anyone re-ran a build.
    let first = Sandbox::new();
    let a = issue_one(&first, &first.person("t", &[]), "NDA");
    let second = Sandbox::new();
    let b = issue_one(&second, &second.person("t", &[]), "NDA");
    assert_eq!(a.pdf_sha256, b.pdf_sha256);
}

#[test]
fn issuance_is_refused_when_the_unpaid_basis_is_not_evidenced() {
    if skip_without_typesetter("issuance_is_refused_when_the_unpaid_basis_is_not_evidenced") {
        return;
    }
    let sb = Sandbox::new();
    let p = sb.person(
        "t",
        &[("currently_enrolled: true", "currently_enrolled: false")],
    );
    let refusals = issue::refusals(&registry(), &p);
    assert!(
        refusals.iter().any(|r| r.contains("unpaid")),
        "expected an unpaid-basis refusal, got {refusals:?}"
    );
}

#[test]
fn a_conditional_document_is_skipped_until_it_applies() {
    if skip_without_typesetter("a_conditional_document_is_skipped_until_it_applies") {
        return;
    }
    let sb = Sandbox::new();
    let reg = registry();

    let without = sb.person("a", &[]);
    let owed = issue::outstanding(&reg, &without, &who(&sb, "a"), &[]).expect("outstanding");
    assert!(
        !owed.contains(&"EQP".to_owned()),
        "no equipment issued, so no EQP"
    );

    let with = sb.person(
        "b",
        &[("equipment:\n  issued: false", "equipment:\n  issued: true")],
    );
    let report = status::report(&reg, &with, &who(&sb, "b"), &[], &[]).expect("report");
    // EQP is owed at `active`; at `preboarding` it is not yet in scope.
    assert!(report.lines.iter().all(|l| l.code != "EQP"));
}

#[test]
fn the_binary_reports_meaningful_exit_codes() {
    if skip_without_typesetter("the_binary_reports_meaningful_exit_codes") {
        return;
    }
    // Exit codes are contractual, because CI reads them.
    let sb = Sandbox::new();
    sb.person("t", &[]);
    let bin = env!("CARGO_BIN_EXE_docket");

    let run = |args: &[&str]| -> i32 {
        let out = Command::new(bin)
            .args(args)
            .arg("--root")
            .arg(root())
            .env("DOCKET_STORE", &sb.paths.store)
            .env("DOCKET_PEOPLE", &sb.paths.people)
            // File-mode test: an ambient DB/API URL (CI's Postgres job exports one) would route the
            // binary to break-glass/remote mode and change these exit codes. Clear both.
            .env_remove("DOCKET_DATABASE_URL")
            .env_remove("DOCKET_API_URL")
            .output()
            .expect("runs");
        // Surfaced on failure: an exit code alone does not say what broke.
        if !out.stderr.is_empty() {
            eprintln!("[{args:?}] {}", String::from_utf8_lossy(&out.stderr));
        }
        out.status.code().unwrap_or(-1)
    };

    assert_eq!(run(&["check"]), 0, "a clean tree checks out");
    assert_eq!(run(&["status", "t"]), 1, "outstanding documents report 1");
    assert_eq!(run(&["status", "nobody"]), 2, "an unknown person is misuse");
    assert_eq!(run(&["issue", "t", "NOPE"]), 2, "an unknown code is misuse");
}

#[test]
fn no_template_restates_a_fact_the_registry_owns() {
    // The whole point of the registry: the office address appeared verbatim
    // in eleven files before this system existed.
    let sb = Sandbox::new();
    let paths = sb.paths.clone();
    let findings = check::run(
        &FileStore::new(&paths),
        &FileBlobs::new(&paths),
        &paths,
        &registry(),
    )
    .expect("check runs");
    let hardcoded: Vec<&str> = findings
        .items
        .iter()
        .filter(|f| f.message.contains("hard-codes"))
        .map(|f| f.message.as_str())
        .collect();
    assert!(hardcoded.is_empty(), "{hardcoded:#?}");
}

/// Templates referenced by the registry all exist on disk.
#[test]
fn every_declared_template_exists() {
    let reg = registry();
    for doc in &reg.lifecycle.documents {
        let path: &Path = &root().join(&doc.template);
        assert!(path.exists(), "{}: missing {}", doc.code, doc.template);
    }
}
