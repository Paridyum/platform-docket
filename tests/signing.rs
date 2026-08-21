//! Signature recording and the stage gate it exists to enforce.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::PathBuf;

use tool_docket::adapters::clock::parse_iso;
use tool_docket::adapters::{registry::Registry, store};
use tool_docket::application::{check, issue, personnel, sign, status};
use tool_docket::config::Paths;
use tool_docket::domain::signature::Method;

mod common;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

struct Sandbox {
    dir: tempfile::TempDir,
    paths: Paths,
}

impl Sandbox {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("temp dir");
        let people = dir.path().join("people");
        std::fs::create_dir_all(&people).expect("people dir");
        common::filled_template(&root(), &people.join("_TEMPLATE.yaml"));
        let paths = Paths {
            root: root(),
            people,
            store: dir.path().join("store"),
        };
        Self { dir, paths }
    }

    /// A person with ONE document issued: the NDA.
    ///
    /// Every issuance runs pdflatex three times, so issuing documents that no
    /// assertion touches is pure wall-clock. Tests that need the whole gate
    /// ask for all four explicitly.
    fn onboarded(&self, slug: &str) -> tool_docket::domain::identity::Record {
        self.with_docs(slug, &["NDA"])
    }

    /// A person with every document the preboarding gate requires.
    fn gated(&self, slug: &str) -> tool_docket::domain::identity::Record {
        self.with_docs(slug, &["NDA", "ISP", "IPA", "AGR", "COC", "GRV"])
    }

    /// A person with exactly the named documents issued.
    fn with_docs(&self, slug: &str, codes: &[&str]) -> tool_docket::domain::identity::Record {
        let reg = registry();
        let r = personnel::create(&self.paths, &reg, slug, "intern", d("2026-08-19"), "rohan")
            .expect("creates");
        let person = store::person(&self.paths, slug).expect("record");
        let by_code = reg.lifecycle.by_code().expect("codes");
        for code in codes {
            let doc = by_code.get(*code).expect("declared");
            issue::one(&self.paths, &reg, &person, doc, d("2026-08-19"), "rohan").expect("issues");
        }
        r
    }

    fn scan(&self, name: &str) -> PathBuf {
        let p = self.dir.path().join(name);
        std::fs::write(&p, b"a scanned signed copy").expect("write scan");
        p
    }

    fn serial_of(&self, code: &str) -> String {
        store::register(&self.paths)
            .expect("register")
            .into_iter()
            .find(|e| e.code == code)
            .map(|e| e.serial)
            .expect("issued")
    }
}

fn registry() -> Registry {
    Registry::load(&Paths::resolve(&root())).expect("registry loads")
}

fn d(iso: &str) -> time::Date {
    parse_iso(iso).expect("date")
}

fn sign_it(sb: &Sandbox, code: &str, method: Method, on: &str) -> anyhow::Result<sign::Signed> {
    let serial = sb.serial_of(code);
    let scan = if method.needs_scan() {
        Some(sb.scan(&format!("{code}.pdf")))
    } else {
        None
    };
    sign::record(
        &sb.paths,
        &registry(),
        &serial,
        method,
        d(on),
        d(on),
        scan.as_deref(),
        "test",
    )
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
fn recording_a_signature_does_not_touch_the_issuance_register() {
    if skip_without_typesetter("recording_a_signature_does_not_touch_the_issuance_register") {
        return;
    }
    // The register is append-only so that what was issued cannot be rewritten
    // once someone has relied on it. A signature is a separate later fact.
    let sb = Sandbox::new();
    sb.onboarded("t");
    let before = std::fs::read(sb.paths.register()).expect("register");
    sign_it(&sb, "NDA", Method::WetInk, "2026-08-25").expect("signs");
    let after = std::fs::read(sb.paths.register()).expect("register");
    assert_eq!(before, after, "issuance entries must be untouched");
    assert_eq!(store::signatures(&sb.paths).expect("sigs").len(), 1);
}

#[test]
fn wet_ink_without_a_scan_is_refused() {
    if skip_without_typesetter("wet_ink_without_a_scan_is_refused") {
        return;
    }
    // Wet ink leaves no trail of its own, so the scan IS the evidence.
    let sb = Sandbox::new();
    sb.onboarded("t");
    let serial = sb.serial_of("NDA");
    let err = sign::record(
        &sb.paths,
        &registry(),
        &serial,
        Method::WetInk,
        d("2026-08-25"),
        d("2026-08-25"),
        None,
        "test",
    )
    .err()
    .map(|e| e.to_string())
    .unwrap_or_default();
    assert!(err.contains("--scan"), "{err}");
}

#[test]
fn esign_needs_no_scan_because_it_carries_its_own_trail() {
    if skip_without_typesetter("esign_needs_no_scan_because_it_carries_its_own_trail") {
        return;
    }
    let sb = Sandbox::new();
    sb.onboarded("t");
    assert!(sign_it(&sb, "NDA", Method::AadhaarEsign, "2026-08-25").is_ok());
}

#[test]
fn a_document_cannot_be_signed_before_it_was_issued() {
    if skip_without_typesetter("a_document_cannot_be_signed_before_it_was_issued") {
        return;
    }
    let sb = Sandbox::new();
    sb.onboarded("t");
    let serial = sb.serial_of("NDA");
    let scan = sb.scan("s.pdf");
    let err = sign::record(
        &sb.paths,
        &registry(),
        &serial,
        Method::WetInk,
        d("2020-01-01"),
        d("2020-01-01"),
        Some(&scan),
        "test",
    )
    .err()
    .map(|e| e.to_string())
    .unwrap_or_default();
    assert!(err.contains("cannot be signed before it exists"), "{err}");
}

#[test]
fn the_received_date_cannot_precede_the_signing_date() {
    if skip_without_typesetter("the_received_date_cannot_precede_the_signing_date") {
        return;
    }
    let sb = Sandbox::new();
    sb.onboarded("t");
    let serial = sb.serial_of("NDA");
    let scan = sb.scan("s.pdf");
    assert!(
        sign::record(
            &sb.paths,
            &registry(),
            &serial,
            Method::WetInk,
            d("2026-08-25"),
            d("2026-08-20"),
            Some(&scan),
            "test",
        )
        .is_err()
    );
}

#[test]
fn a_signature_is_recorded_once() {
    if skip_without_typesetter("a_signature_is_recorded_once") {
        return;
    }
    let sb = Sandbox::new();
    sb.onboarded("t");
    sign_it(&sb, "NDA", Method::AadhaarEsign, "2026-08-25").expect("first");
    let err = sign_it(&sb, "NDA", Method::AadhaarEsign, "2026-08-26")
        .err()
        .map(|e| e.to_string())
        .unwrap_or_default();
    assert!(err.contains("already recorded"), "{err}");
}

#[test]
fn signing_an_unissued_serial_is_refused() {
    if skip_without_typesetter("signing_an_unissued_serial_is_refused") {
        return;
    }
    let sb = Sandbox::new();
    sb.onboarded("t");
    assert!(
        sign::record(
            &sb.paths,
            &registry(),
            "PAR/NDA/1999/0001",
            Method::AadhaarEsign,
            d("2026-08-25"),
            d("2026-08-25"),
            None,
            "test",
        )
        .is_err()
    );
}

#[test]
fn the_scan_is_copied_into_the_store_and_hashed() {
    if skip_without_typesetter("the_scan_is_copied_into_the_store_and_hashed") {
        return;
    }
    // Holding only a path leaves the evidence outside the thing that claims it
    // exists; holding only a hash leaves nothing to look at.
    let sb = Sandbox::new();
    sb.onboarded("t");
    let s = sign_it(&sb, "NDA", Method::WetInk, "2026-08-25").expect("signs");
    let rel = s.record.scan_path.as_deref().expect("a path is recorded");
    let stored = sb.paths.store.join(rel);
    assert!(stored.exists(), "the scan must be inside the store");
    let bytes = std::fs::read(&stored).expect("read");
    assert_eq!(
        s.record.scan_sha256.as_deref(),
        Some(tool_docket::domain::chain::sha256(&bytes).as_str())
    );
}

#[test]
fn status_distinguishes_issued_from_executed() {
    if skip_without_typesetter("status_distinguishes_issued_from_executed") {
        return;
    }
    let sb = Sandbox::new();
    let who = sb.onboarded("t");
    let reg = registry();
    let person = store::person(&sb.paths, "t").expect("record");

    let before = status::report(
        &reg,
        &person,
        &who,
        &store::register(&sb.paths).expect("reg"),
        &store::signatures(&sb.paths).expect("sigs"),
    )
    .expect("report");
    assert_eq!(before.signed(), 0, "nothing has come back yet");
    assert!(!before.awaiting_signature().is_empty());

    sign_it(&sb, "NDA", Method::AadhaarEsign, "2026-08-25").expect("signs");
    let after = status::report(
        &reg,
        &person,
        &who,
        &store::register(&sb.paths).expect("reg"),
        &store::signatures(&sb.paths).expect("sigs"),
    )
    .expect("report");
    assert_eq!(after.signed(), 1);
    assert!(!after.awaiting_signature().contains(&"NDA"));
}

#[test]
fn the_stage_gate_refuses_while_agreements_are_unexecuted() {
    if skip_without_typesetter("the_stage_gate_refuses_while_agreements_are_unexecuted") {
        return;
    }
    // lifecycle.yaml stated "no access until the NDA and policy are signed"
    // as prose and nothing checked it. This is the check.
    let sb = Sandbox::new();
    sb.gated("t");
    let (from, to, gates) =
        personnel::advance(&sb.paths, &registry(), "t", d("2026-09-01"), false, "rohan")
            .expect("evaluates");
    assert_eq!(from, "preboarding");
    assert_eq!(to, "active");
    assert!(!gates.is_empty(), "the gate must hold");
    let codes: Vec<&str> = gates.iter().map(|g| g.code.as_str()).collect();
    for required in ["NDA", "ISP", "IPA", "AGR", "COC", "GRV"] {
        assert!(codes.contains(&required), "{required} must gate: {codes:?}");
    }
    // And the record must not have moved.
    let person = store::person(&sb.paths, "t").expect("record");
    assert_eq!(person.stage(), "preboarding");
}

#[test]
fn the_gate_opens_once_every_required_document_is_executed() {
    if skip_without_typesetter("the_gate_opens_once_every_required_document_is_executed") {
        return;
    }
    let sb = Sandbox::new();
    sb.gated("t");
    for code in ["NDA", "ISP", "IPA", "AGR", "COC", "GRV"] {
        sign_it(&sb, code, Method::AadhaarEsign, "2026-08-25").expect("signs");
    }
    let (from, to, gates) =
        personnel::advance(&sb.paths, &registry(), "t", d("2026-09-01"), false, "rohan")
            .expect("advances");
    assert!(gates.is_empty(), "{gates:?}");
    assert_eq!((from.as_str(), to.as_str()), ("preboarding", "active"));
    let person = store::person(&sb.paths, "t").expect("record");
    assert_eq!(person.stage(), "active", "the record must be updated");
}

#[test]
fn force_advances_past_an_unmet_gate() {
    if skip_without_typesetter("force_advances_past_an_unmet_gate") {
        return;
    }
    // Escape hatch, because refusing forever is not a workflow. It says so.
    let sb = Sandbox::new();
    sb.onboarded("t");
    let (_, _, gates) =
        personnel::advance(&sb.paths, &registry(), "t", d("2026-09-01"), true, "rohan")
            .expect("advances");
    assert!(gates.is_empty());
    assert_eq!(
        store::person(&sb.paths, "t").expect("record").stage(),
        "active"
    );
}

#[test]
fn issuance_is_recorded_in_the_event_log() {
    if skip_without_typesetter("issuance_is_recorded_in_the_event_log") {
        return;
    }
    // The log carried access and separation but never the documents.
    let sb = Sandbox::new();
    let who = sb.onboarded("t");
    let log = store::events(&sb.paths).expect("events");
    let issued = log
        .iter()
        .filter(|e| e.person_id == who.person_id)
        .filter(|e| e.kind == tool_docket::domain::event::Kind::DocumentIssued)
        .count();
    assert_eq!(
        issued, 1,
        "one event per issued document, and the fixture issues one"
    );
}

#[test]
fn signing_is_recorded_in_the_event_log() {
    if skip_without_typesetter("signing_is_recorded_in_the_event_log") {
        return;
    }
    let sb = Sandbox::new();
    let who = sb.onboarded("t");
    sign_it(&sb, "NDA", Method::AadhaarEsign, "2026-08-25").expect("signs");
    let log = store::events(&sb.paths).expect("events");
    assert!(
        log.iter()
            .filter(|e| e.person_id == who.person_id)
            .any(|e| e.kind == tool_docket::domain::event::Kind::DocumentSigned),
        "signing must appear in the history"
    );
}

#[test]
fn check_detects_a_tampered_signature_ledger() {
    if skip_without_typesetter("check_detects_a_tampered_signature_ledger") {
        return;
    }
    let sb = Sandbox::new();
    sb.onboarded("t");
    sign_it(&sb, "NDA", Method::AadhaarEsign, "2026-08-25").expect("signs");

    let path = sb.paths.signatures();
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(&path, text.replace("2026-08-25", "2026-08-01")).expect("write");

    let findings = check::run(&sb.paths, &registry()).expect("check runs");
    assert!(
        findings
            .items
            .iter()
            .any(|f| f.blocking && f.message.contains("signature ledger")),
        "{:#?}",
        findings.items
    );
}

#[test]
fn check_detects_a_missing_scan() {
    if skip_without_typesetter("check_detects_a_missing_scan") {
        return;
    }
    // The scan is the evidence for wet ink. Losing it must be visible.
    let sb = Sandbox::new();
    sb.onboarded("t");
    let s = sign_it(&sb, "NDA", Method::WetInk, "2026-08-25").expect("signs");
    let rel = s.record.scan_path.as_deref().expect("path");
    std::fs::remove_file(sb.paths.store.join(rel)).expect("remove");

    let findings = check::run(&sb.paths, &registry()).expect("check runs");
    assert!(
        findings
            .items
            .iter()
            .any(|f| f.blocking && f.message.contains("scan")),
        "{:#?}",
        findings.items
    );
}
