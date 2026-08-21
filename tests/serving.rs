//! The operator API, against a throwaway store.
//!
//! These exercise the routing and the wire shapes. The rules underneath are
//! already covered by the other suites, and duplicating those assertions here
//! would create a second statement of what the rules are — so what is tested is
//! only what the adapter adds: which handler runs, what status comes back, and
//! whether the payload says the same thing the CLI would.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::PathBuf;

use tool_docket::adapters::clock::{FixedClock, parse_iso};
use tool_docket::adapters::http::request;
use tool_docket::adapters::http::router::{self, Context};
use tool_docket::adapters::registry::Registry;
use tool_docket::application::{personnel, roster};
use tool_docket::config::Paths;

mod common;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

struct Sandbox {
    /// Held for its lifetime: dropping it deletes the store.
    _dir: tempfile::TempDir,
    paths: Paths,
}

impl Sandbox {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("temp dir");
        let people = dir.path().join("people");
        std::fs::create_dir_all(&people).expect("people dir");
        common::filled_template(&root(), &people.join("_TEMPLATE.yaml"));
        let store = dir.path().join("store");
        Self {
            _dir: dir,
            paths: Paths {
                root: root(),
                people,
                store,
            },
        }
    }
}

fn registry() -> Registry {
    Registry::load(&Paths::resolve(&root())).expect("registry loads")
}

fn date() -> time::Date {
    parse_iso("2026-08-21").expect("fixed date")
}

/// Route one request, given as the raw HTTP text a client would send.
///
/// Deliberately goes through the real parser rather than constructing a
/// `Request` by hand: a routing test that skips parsing cannot catch a path that
/// parses differently from how it is written.
fn call(sandbox: &Sandbox, registry: &Registry, raw: &str) -> (u16, serde_json::Value) {
    let clock = FixedClock(date());
    let ctx = Context {
        paths: &sandbox.paths,
        registry,
        clock: &clock,
    };
    let req = request::parse(raw.as_bytes()).expect("request parses");
    let response = router::route(&ctx, &req);
    let body = if response.body.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_slice(&response.body).expect("response body is json")
    };
    (response.status, body)
}

fn get(sandbox: &Sandbox, registry: &Registry, path: &str) -> (u16, serde_json::Value) {
    call(sandbox, registry, &format!("GET {path} HTTP/1.1\r\n\r\n"))
}

/// A POST carrying an operator header, as the console sends it.
fn post(
    sandbox: &Sandbox,
    registry: &Registry,
    path: &str,
    json_body: &str,
) -> (u16, serde_json::Value) {
    call(
        sandbox,
        registry,
        &format!(
            "POST {path} HTTP/1.1\r\nX-Docket-Operator: ruhani\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{json_body}",
            json_body.len()
        ),
    )
}

/// A POST with no operator header, as a forged cross-site request would be.
fn post_anonymous(
    sandbox: &Sandbox,
    registry: &Registry,
    path: &str,
    json_body: &str,
) -> (u16, serde_json::Value) {
    call(
        sandbox,
        registry,
        &format!(
            "POST {path} HTTP/1.1\r\nContent-Length: {}\r\n\r\n{json_body}",
            json_body.len()
        ),
    )
}

/// Every byte under the store, so a test can prove nothing changed.
fn store_bytes(sandbox: &Sandbox) -> Vec<(PathBuf, Vec<u8>)> {
    fingerprint(&sandbox.paths.store)
}

/// Enrol one intern, the way the CLI does.
fn enrol(sandbox: &Sandbox, registry: &Registry, slug: &str) {
    personnel::create(&sandbox.paths, registry, slug, "intern", date(), "ruhani").expect("enrols");
}

#[test]
fn health_answers_without_a_store() {
    let sandbox = Sandbox::new();
    let (status, body) = get(&sandbox, &registry(), "/api/health");
    assert_eq!(status, 200);
    assert_eq!(body["ok"], true);
    // The version is the crate's, so a console can tell which docket it reached.
    assert!(body["version"].is_string());
}

#[test]
fn the_roster_says_what_the_cli_says() {
    let sandbox = Sandbox::new();
    let reg = registry();
    enrol(&sandbox, &reg, "asha");
    enrol(&sandbox, &reg, "vikram");

    // The comparison that matters. If these ever disagree, HR is looking at two
    // different organisations depending on which interface they opened.
    let lines = roster::build(&sandbox.paths, &reg).expect("roster");
    let (active, separated, erased) = roster::tally(&lines);

    let (status, body) = get(&sandbox, &reg, "/api/roster");
    assert_eq!(status, 200);
    assert_eq!(body["people"].as_array().expect("array").len(), lines.len());
    assert_eq!(body["active"], active);
    assert_eq!(body["separated"], separated);
    assert_eq!(body["erased"], erased);
}

#[test]
fn a_roster_filter_narrows_the_rows_and_leaves_the_tallies_alone() {
    let sandbox = Sandbox::new();
    let reg = registry();
    enrol(&sandbox, &reg, "asha");

    let (_, all) = get(&sandbox, &reg, "/api/roster");
    let (status, filtered) = get(&sandbox, &reg, "/api/roster?standing=separated");
    assert_eq!(status, 200);

    // No one is separated, so no rows come back...
    assert_eq!(filtered["people"].as_array().expect("array").len(), 0);
    // ...but the counts still describe the whole organisation. A filtered page
    // reporting "0 active" would misstate the size of the company.
    assert_eq!(filtered["active"], all["active"]);
    assert_ne!(all["active"], 0);
}

#[test]
fn a_person_page_carries_the_workflow_in_order() {
    let sandbox = Sandbox::new();
    let reg = registry();
    enrol(&sandbox, &reg, "asha");

    let (status, body) = get(&sandbox, &reg, "/api/person/asha");
    assert_eq!(status, 200);
    assert_eq!(body["slug"], "asha");
    assert_eq!(body["class"], "intern");
    assert_eq!(body["erased"], false);

    // `docket next` is the landing content of a person's page (§8), and the
    // ordering IS the workflow. A freshly enrolled intern owes something.
    let steps = body["steps"].as_array().expect("steps");
    assert!(!steps.is_empty(), "a new intern has nothing to do?");
    assert!(steps[0]["what"].is_string());
}

#[test]
fn a_person_page_matches_dossier_and_next_rather_than_recomputing_them() {
    let sandbox = Sandbox::new();
    let reg = registry();
    enrol(&sandbox, &reg, "asha");

    let dossier = personnel::dossier(&sandbox.paths, &reg, "asha").expect("dossier");
    let (_, body) = get(&sandbox, &reg, "/api/person/asha");

    assert_eq!(body["person_id"], dossier.identity.person_id.to_string());
    assert_eq!(
        body["engagement_id"],
        dossier.identity.engagement_id.to_string()
    );
    assert_eq!(
        body["documents"].as_array().expect("documents").len(),
        dossier.documents.len()
    );
    assert_eq!(
        body["violations"].as_array().expect("violations").len(),
        dossier.violations.len()
    );
}

#[test]
fn an_unknown_person_is_404_not_500() {
    let sandbox = Sandbox::new();
    let reg = registry();
    enrol(&sandbox, &reg, "asha");

    let (status, body) = get(&sandbox, &reg, "/api/person/nobody");
    // The distinction is the point: 500 would send an operator looking for a
    // broken server when they have merely mistyped a name.
    assert_eq!(status, 404);
    assert!(
        body["error"].as_str().expect("error").contains("nobody"),
        "{body}"
    );
}

#[test]
fn check_reports_the_same_findings_the_cli_would() {
    let sandbox = Sandbox::new();
    let reg = registry();
    enrol(&sandbox, &reg, "asha");

    let expected = tool_docket::application::check::run(&sandbox.paths, &reg).expect("check");
    let (status, body) = get(&sandbox, &reg, "/api/check");
    assert_eq!(status, 200);
    assert_eq!(body["errors"], expected.errors());
    assert_eq!(body["documents"], expected.documents);
    assert_eq!(body["stages"], expected.stages);
    assert_eq!(
        body["findings"].as_array().expect("findings").len(),
        expected.items.len()
    );
    // `blocking` travels as a boolean, so a console cannot mistake a blocking
    // finding for advice by comparing against the wrong severity spelling.
    for f in body["findings"].as_array().expect("findings") {
        assert!(f["blocking"].is_boolean(), "{f}");
    }
}

#[test]
fn retention_reports_what_it_could_not_judge_as_well_as_what_is_overdue() {
    let sandbox = Sandbox::new();
    let reg = registry();
    enrol(&sandbox, &reg, "asha");

    let (status, body) = get(&sandbox, &reg, "/api/retention");
    assert_eq!(status, 200);
    // Both keys must always be present. A console that only reads `overdue`
    // would silently treat an unparsable retention period as compliant, which is
    // how over-retention becomes invisible.
    assert!(body["overdue"].is_array(), "{body}");
    assert!(body["unevaluated"].is_array(), "{body}");
}

#[test]
fn emails_render_and_are_never_described_as_sent() {
    let sandbox = Sandbox::new();
    let reg = registry();
    enrol(&sandbox, &reg, "asha");

    let (status, body) = get(&sandbox, &reg, "/api/emails/asha");
    assert_eq!(status, 200);
    let messages = body.as_array().expect("array of messages");
    // Whatever is due, every message carries its text to copy and the list of
    // attachments it names but that have not been issued. D-27: this tool does
    // not send mail, so `missing` is what stops one going out incomplete.
    for m in messages {
        assert!(m["subject"].is_string(), "{m}");
        assert!(m["body"].is_string(), "{m}");
        assert!(m["missing"].is_array(), "{m}");
        assert!(
            m.get("sent").is_none(),
            "nothing here may claim to send: {m}"
        );
    }
}

#[test]
fn emails_for_an_unknown_person_are_404() {
    let sandbox = Sandbox::new();
    let (status, _) = get(&sandbox, &registry(), "/api/emails/nobody");
    assert_eq!(status, 404);
}

#[test]
fn a_write_method_on_a_read_route_is_405_and_writes_nothing() {
    let sandbox = Sandbox::new();
    let reg = registry();
    enrol(&sandbox, &reg, "asha");
    let before = std::fs::read_to_string(sandbox.paths.ledger()).expect("ledger");

    let (status, body) = call(
        &sandbox,
        &reg,
        "POST /api/person/asha HTTP/1.1\r\nContent-Length: 0\r\n\r\n",
    );
    assert_eq!(status, 405);
    assert!(body["error"].as_str().expect("error").contains("POST"));

    // The ledger is a hash chain. A route that half-handled a POST would be the
    // worst possible defect here, so this asserts the bytes are untouched rather
    // than trusting that no handler exists.
    let after = std::fs::read_to_string(sandbox.paths.ledger()).expect("ledger");
    assert_eq!(before, after);
}

#[test]
fn an_unknown_path_is_404() {
    let sandbox = Sandbox::new();
    let (status, _) = get(&sandbox, &registry(), "/api/nonsense");
    assert_eq!(status, 404);
    let (status, _) = get(&sandbox, &registry(), "/");
    assert_eq!(status, 404);
}

#[test]
fn no_route_writes_anything_at_all() {
    let sandbox = Sandbox::new();
    let reg = registry();
    enrol(&sandbox, &reg, "asha");

    // The guarantee §10 sequences the read-only phase for: the platform is
    // deliverable in a state that cannot damage a record. Asserted over every
    // route rather than argued from the absence of write handlers.
    let store_before = fingerprint(&sandbox.paths.store);
    for path in [
        "/api/health",
        "/api/roster",
        "/api/roster?standing=active",
        "/api/person/asha",
        "/api/check",
        "/api/retention",
        "/api/emails/asha",
    ] {
        let (status, _) = get(&sandbox, &reg, path);
        assert_eq!(status, 200, "{path}");
    }
    assert_eq!(
        store_before,
        fingerprint(&sandbox.paths.store),
        "a read route changed the store"
    );
}

/// Every file under a directory, with its bytes, sorted.
fn fingerprint(dir: &std::path::Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(next) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&next) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Ok(bytes) = std::fs::read(&path) {
                out.push((path, bytes));
            }
        }
    }
    out.sort();
    out
}

// ---------------------------------------------------------------------------
// Writes
// ---------------------------------------------------------------------------

#[test]
fn a_write_without_an_operator_is_refused_and_changes_nothing() {
    let sandbox = Sandbox::new();
    let reg = registry();
    enrol(&sandbox, &reg, "asha");
    let before = store_bytes(&sandbox);

    let (status, body) = post_anonymous(&sandbox, &reg, "/api/person/asha/advance", "{}");

    // 403, not 500. The distinction sends an operator to different places: fix
    // the request, versus the tool is broken. An authorisation failure reported as
    // 500 reads as an outage.
    assert_eq!(status, 403, "{body}");
    assert!(
        body["error"]
            .as_str()
            .unwrap_or_default()
            .contains("operator"),
        "{body}"
    );
    assert_eq!(
        before,
        store_bytes(&sandbox),
        "an anonymous write touched the store"
    );
}

#[test]
fn an_operator_the_registry_does_not_declare_cannot_write() {
    let sandbox = Sandbox::new();
    let reg = registry();
    enrol(&sandbox, &reg, "asha");
    let before = store_bytes(&sandbox);

    let (status, body) = call(
        &sandbox,
        &reg,
        "POST /api/person/asha/advance HTTP/1.1\r\nX-Docket-Operator: nobody\r\nContent-Length: 2\r\n\r\n{}",
    );
    assert_eq!(status, 403, "{body}");
    // The message must name the real operators, so the fix is obvious.
    assert!(
        body["error"].as_str().unwrap_or_default().contains("known"),
        "{body}"
    );
    assert_eq!(before, store_bytes(&sandbox));
}

#[test]
fn onboarding_an_unfilled_record_is_refused_rather_than_issuing_documents() {
    let sandbox = Sandbox::new();
    let reg = registry();
    // The sandbox template is filled (see tests/common), so blank one field back
    // out to reproduce the real hazard: a record that looks complete and is not.
    enrol(&sandbox, &reg, "asha");
    let path = sandbox.paths.person("asha");
    let body = std::fs::read_to_string(&path).expect("record");
    std::fs::write(
        &path,
        body.replace("legal_name: Test Person", "legal_name: FILL-ME"),
    )
    .expect("write");

    let before = store_bytes(&sandbox);
    let (status, payload) = post(&sandbox, &reg, "/api/person/asha/onboard", "{}");

    // 409: understood, and declined. Not 400 (the request was fine) and not 500.
    assert_eq!(status, 409, "{payload}");
    assert!(
        payload["refusals"]
            .as_array()
            .expect("refusals")
            .iter()
            .any(|r| r.as_str().unwrap_or_default().contains("FILL-ME")),
        "{payload}"
    );
    // The whole point: no serial was spent.
    assert_eq!(
        before,
        store_bytes(&sandbox),
        "a refused onboard issued something"
    );
}

#[test]
fn advancing_past_an_unmet_gate_is_refused_and_the_stage_does_not_move() {
    let sandbox = Sandbox::new();
    let reg = registry();
    enrol(&sandbox, &reg, "asha");

    let stage_before = tool_docket::adapters::store::person(&sandbox.paths, "asha")
        .expect("record")
        .stage()
        .to_owned();

    let (status, payload) = post(&sandbox, &reg, "/api/person/asha/advance", "{}");
    assert_eq!(status, 409, "{payload}");

    let blockers = payload["blockers"].as_array().expect("blockers");
    assert!(!blockers.is_empty(), "{payload}");
    // The shape the person page already renders, so a refused write and a held
    // person read identically in the console.
    assert!(blockers[0]["code"].is_string(), "{payload}");
    assert!(blockers[0]["reason"].is_string(), "{payload}");

    let stage_after = tool_docket::adapters::store::person(&sandbox.paths, "asha")
        .expect("record")
        .stage()
        .to_owned();
    assert_eq!(
        stage_before, stage_after,
        "a refused advance moved the stage"
    );
}

#[test]
fn the_advance_route_does_not_expose_force() {
    let sandbox = Sandbox::new();
    let reg = registry();
    enrol(&sandbox, &reg, "asha");

    // `force` IS accepted by the API — the CLI has it and the boundary mirrors the
    // CLI. What must be true is that it is opt-in and absent by default, so a
    // console that does not send it can never skip a gate.
    let (default_status, _) = post(&sandbox, &reg, "/api/person/asha/advance", "{}");
    assert_eq!(default_status, 409, "the default must refuse");

    let (forced_status, _) = post(
        &sandbox,
        &reg,
        "/api/person/asha/advance",
        "{\"force\":true}",
    );
    assert_eq!(
        forced_status, 200,
        "force is still available below the screens"
    );
}

#[test]
fn signing_a_document_belonging_to_someone_else_is_refused() {
    let sandbox = Sandbox::new();
    let reg = registry();
    enrol(&sandbox, &reg, "asha");
    enrol(&sandbox, &reg, "vikram");

    // No documents exist, so any serial is somebody else's by definition. The
    // assertion is that the route checks ownership at all: without it, a request
    // could file a signature against any document through any person's URL, and
    // the signature ledger would be correct while the audit trail lied.
    let (status, body) = post(
        &sandbox,
        &reg,
        "/api/person/vikram/sign",
        "{\"serial\":\"PAR/NDA/2026/0001\",\"method\":\"wet-ink\",\"signed_on\":\"2026-08-21\"}",
    );
    assert_ne!(status, 200);
    assert!(
        body["error"]
            .as_str()
            .unwrap_or_default()
            .contains("was not issued to vikram"),
        "{body}"
    );
}

#[test]
fn the_enrolment_form_is_generated_from_the_registry() {
    let sandbox = Sandbox::new();
    let reg = registry();

    let (status, body) = get(&sandbox, &reg, "/api/form/intern");
    assert_eq!(status, 200);

    let questions = body["questions"].as_array().expect("questions");
    assert!(!questions.is_empty());

    // Every question carries its declared purpose. If a field's purpose cannot be
    // written down the field should not exist, so there is never a reason for the
    // form to omit it.
    for q in questions {
        assert!(q["purpose"].as_str().is_some_and(|p| !p.is_empty()), "{q}");
        assert!(q["required"].is_boolean(), "{q}");
    }

    // A field the class forbids appears in neither the form nor the CLI interview.
    // An intern has no PAN: collecting one is processing data for which no purpose
    // exists, which is what DPDP purpose limitation forbids.
    let paths: Vec<&str> = questions
        .iter()
        .filter_map(|q| q["path"].as_str())
        .collect();
    // Exact paths, not substrings. A `contains("pan")` here passed for the wrong
    // reason and then failed for the wrong reason: `access.company_email` contains
    // "pan", so the assertion was never about a PAN at all.
    for forbidden in [
        "statutory.pan",
        "statutory.aadhaar",
        "statutory.uan",
        "statutory.esic",
        "statutory.bank_account",
        "compensation.salary",
    ] {
        assert!(
            !paths.contains(&forbidden),
            "an intern's form offers {forbidden}, which no statute requires for an \
             unpaid placement: {paths:?}"
        );
    }

    // And it matches what the CLI would ask, rather than being a second list.
    let class = reg.catalogue.class("intern").expect("intern class");
    let expected = tool_docket::domain::personnel::questions(&reg.catalogue, class);
    assert_eq!(questions.len(), expected.len());
}

#[test]
fn an_unknown_class_has_no_form() {
    let sandbox = Sandbox::new();
    let (status, _) = get(&sandbox, &registry(), "/api/form/wizard");
    assert_eq!(status, 404);
}

#[test]
fn enrolling_through_the_api_produces_the_same_record_the_cli_would() {
    let sandbox = Sandbox::new();
    let reg = registry();

    let answers = serde_json::json!({
        "slug": "priya",
        "class": "intern",
        "answers": {
            "identity.legal_name": "Priya Nair",
            "engagement.role": "Software Engineering Intern",
            "engagement.start_date": "2026-09-01",
        }
    });
    let (status, payload) = post(&sandbox, &reg, "/api/person", &answers.to_string());
    assert_eq!(status, 200, "{payload}");

    // The record exists, is filed under the slug, and carries the answers.
    let person = tool_docket::adapters::store::person(&sandbox.paths, "priya").expect("record");
    assert_eq!(person.string("identity.legal_name"), Some("Priya Nair"));

    // And the identifier was allocated, so the ledger and the file agree. A
    // record written without one would be a file nothing points at.
    let record = personnel::identify(&sandbox.paths, "priya").expect("identified");
    assert_eq!(record.slug, "priya");
    assert_eq!(record.class, "intern");

    // Byte-for-byte what the CLI's interview would have written for the same
    // answers, which is the property that keeps the form and the prompts from
    // drifting apart.
    let mut expected_answers = std::collections::BTreeMap::new();
    expected_answers.insert("identity.legal_name".to_owned(), "Priya Nair".to_owned());
    expected_answers.insert(
        "engagement.role".to_owned(),
        "Software Engineering Intern".to_owned(),
    );
    expected_answers.insert("engagement.start_date".to_owned(), "2026-09-01".to_owned());
    let expected = tool_docket::application::enrol::to_yaml(&reg, "intern", &expected_answers);
    let actual = std::fs::read_to_string(sandbox.paths.person("priya")).expect("read");
    assert_eq!(actual, expected);
}

#[test]
fn a_post_with_no_body_where_one_is_required_is_refused() {
    let sandbox = Sandbox::new();
    let reg = registry();
    let (status, body) = post(&sandbox, &reg, "/api/person", "");
    assert_ne!(status, 200);
    assert!(
        body["error"]
            .as_str()
            .unwrap_or_default()
            .contains("JSON body"),
        "{body}"
    );
}

/// Whether a typesetter is installed.
///
/// Issuing renders real PDFs, and a GitHub runner has neither TeX Live nor
/// Tectonic — so this test SKIPS there rather than failing, the same pattern the
/// issuance and engines suites already use. Skipping is not free: it means CI does
/// not exercise issuance through the API either, which is `tool-docket` D-35.
fn typesetter_available() -> bool {
    tool_docket::adapters::typeset::detect().is_ok()
}

#[test]
fn onboarding_a_filled_record_issues_and_records_the_operator() {
    if !typesetter_available() {
        // Printed, so a green run is never mistaken for coverage.
        eprintln!(
            "SKIP onboarding_a_filled_record_issues_and_records_the_operator: \
             no typesetter installed (D-35)"
        );
        return;
    }

    let sandbox = Sandbox::new();
    let reg = registry();
    enrol(&sandbox, &reg, "asha");

    let (status, payload) = post(&sandbox, &reg, "/api/person/asha/onboard", "{}");
    // Checked on the typesetter rather than on the status. An earlier version
    // treated any 409 as "skip", which meant a genuine refusal — a gate unmet, a
    // record unfilled — would have been silently accepted as a pass. It also did
    // not cover the case that actually broke CI: no typesetter is a 500, not a 409.
    assert_eq!(status, 200, "{payload}");

    let issued = payload["issued"].as_array().expect("issued");
    assert!(!issued.is_empty(), "{payload}");

    let register = tool_docket::adapters::store::register(&sandbox.paths).expect("register");
    assert!(
        !register.is_empty(),
        "documents were reported issued but the register is empty"
    );

    // Every event this write caused names the operator who caused it.
    //
    // Asserted against the EVENT log rather than the register, because that is
    // where authorship lives — the register describes the document and the log
    // describes the act. A write with no author is what `--by` was introduced to
    // stop and the API must not reintroduce it.
    let events = tool_docket::adapters::store::events(&sandbox.paths).expect("events");
    let issued_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.kind, tool_docket::domain::event::Kind::DocumentIssued))
        .collect();
    assert!(!issued_events.is_empty(), "issuing recorded no event");
    for e in issued_events {
        assert_eq!(e.recorded_by, "ruhani", "{e:?}");
    }
}

#[test]
fn the_operators_route_lists_only_those_who_may_act() {
    let sandbox = Sandbox::new();
    let reg = registry();
    let (status, body) = get(&sandbox, &reg, "/api/operators");
    assert_eq!(status, 200);

    let listed = body.as_array().expect("array");
    assert!(!listed.is_empty(), "nobody may operate this tool?");

    let ids: Vec<&str> = listed.iter().filter_map(|o| o["id"].as_str()).collect();
    // An inactive operator is not offered. A name that cannot be used is a name
    // somebody picks and is then refused for, which reads as a broken tool.
    for o in &reg.operators {
        assert_eq!(
            ids.contains(&o.id.as_str()),
            o.active,
            "{} is active={} but listed={}",
            o.id,
            o.active,
            ids.contains(&o.id.as_str())
        );
    }
    // Every id offered must actually be accepted by a write, or the chooser is
    // offering something that fails.
    for id in &ids {
        assert!(reg.operator(id).is_ok(), "{id} is offered but not accepted");
    }
}
