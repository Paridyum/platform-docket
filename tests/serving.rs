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
        std::fs::copy(
            root().join("people").join("_TEMPLATE.yaml"),
            people.join("_TEMPLATE.yaml"),
        )
        .expect("template");
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
