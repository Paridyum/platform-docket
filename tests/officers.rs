//! Officers: a different class, a different document set, and nobody signing their own.
//!
//! RFC-0042. The three things worth a test are the three that produce a WRONG DOCUMENT
//! rather than an error: an internship document issued to a director, a stage gate that
//! can never open because it waits on a document the class never receives, and a
//! signature block in which the subject signs for Paridyum.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::PathBuf;

use tool_docket::adapters::blobs::FileBlobs;
use tool_docket::adapters::clock::parse_iso;
use tool_docket::adapters::registry::Registry;
use tool_docket::adapters::store::{self, FileStore};
use tool_docket::application::{issue, personnel};
use tool_docket::config::Paths;
use tool_docket::domain::person;

mod common;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn registry() -> Registry {
    Registry::load(&Paths::resolve(&root())).expect("registry loads")
}

fn d(iso: &str) -> time::Date {
    parse_iso(iso).expect("date")
}

struct Sandbox {
    _dir: tempfile::TempDir,
    paths: Paths,
}

impl Sandbox {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("temp dir");
        let people = dir.path().join("people");
        std::fs::create_dir_all(&people).expect("people dir");
        common::filled_template(&root(), &people.join("_TEMPLATE.yaml"));
        common::filled_officer_template(&root(), &people.join("_TEMPLATE-officer.yaml"));
        let paths = Paths {
            root: root(),
            people,
            store: dir.path().join("store"),
        };
        Self { _dir: dir, paths }
    }

    fn officer(&self, slug: &str) -> person::Person {
        let reg = registry();
        personnel::create(
            &FileStore::new(&self.paths),
            &self.paths,
            &reg,
            slug,
            "officer",
            "PAR",
            d("2026-09-06"),
            "rohan",
        )
        .expect("creates an officer");
        store::person(&self.paths, slug).expect("record")
    }
}

#[test]
fn an_officer_owes_the_officer_set_and_not_the_internship_set() {
    let owed = registry()
        .lifecycle
        .owed_upto_for("preboarding", Some("officer"))
        .expect("stage is declared");

    assert!(
        owed.contains(&"NDO".to_owned()),
        "officer NDA is owed: {owed:?}"
    );
    assert!(
        owed.contains(&"IPO".to_owned()),
        "officer IPA is owed: {owed:?}"
    );

    // The failure this catches is a document that SAYS "Intern" being issued to a
    // director. A false document with a serial and a register entry is worse than none.
    for internship_only in [
        "OFR", "AGR", "IPA", "NDA", "GRD", "INS", "ONB", "EMC", "ISP", "COC", "DPN", "GRV",
    ] {
        assert!(
            !owed.contains(&internship_only.to_owned()),
            "{internship_only} is an internship document and must not be owed to an officer"
        );
    }
}

#[test]
fn an_intern_owes_exactly_what_they_did_before() {
    // The class filter must be invisible to the class that existed before it. Every
    // record this system has ever held is an intern, so a narrowing bug here is a
    // document silently never issued to a real person.
    let owed = registry()
        .lifecycle
        .owed_upto_for("preboarding", Some("intern"))
        .expect("stage is declared");

    for code in [
        "OFR", "GRD", "INS", "AGR", "NDA", "IPA", "ISP", "PSH", "COC", "GRV", "DPN",
    ] {
        assert!(
            owed.contains(&code.to_owned()),
            "{code} is still owed to an intern"
        );
    }
    assert!(
        !owed.contains(&"NDO".to_owned()),
        "the officer NDA is not an intern's"
    );
    assert!(
        !owed.contains(&"IPO".to_owned()),
        "the officer IPA is not an intern's"
    );
}

#[test]
fn issuing_an_internship_document_to_an_officer_is_refused() {
    let sb = Sandbox::new();
    let person = sb.officer("test-officer");
    let reg = registry();
    let by_code = reg.lifecycle.by_code().expect("codes");

    let err = issue::one(
        &FileStore::new(&sb.paths),
        &FileBlobs::new(&sb.paths),
        &sb.paths,
        &reg,
        &person,
        by_code.get("CRT").expect("declared"),
        d("2026-09-06"),
        "rohan",
    )
    .expect_err("a Certificate of Internship is not issued to a director");

    assert!(
        err.to_string().contains("class"),
        "the refusal names the class: {err}"
    );
}

#[test]
fn the_subject_is_substituted_out_rather_than_signing_their_own_document() {
    if skip_without_typesetter("the_subject_is_substituted_out") {
        return;
    }
    // RFC-0042 decision 4. Officer documents route to the Director, so the Director's
    // OWN route to himself. A document signed by its own subject is evidence of
    // nothing, so the routed signatory is replaced by the first active signatory in
    // registry order who is not the subject -- deterministically, and recorded on the
    // register entry, because after substitution it is no longer derivable from
    // lifecycle.yaml.
    let sb = Sandbox::new();
    let reg = registry();
    let director = reg
        .active_signatories()
        .find(|s| s.id == "director")
        .expect("a director is declared")
        .legal_name
        .clone();

    personnel::create(
        &FileStore::new(&sb.paths),
        &sb.paths,
        &reg,
        "self-signer",
        "officer",
        "PAR",
        d("2026-09-06"),
        "rohan",
    )
    .expect("creates");
    let path = sb.paths.person("self-signer");
    let body = std::fs::read_to_string(&path).expect("record");
    std::fs::write(
        &path,
        body.replace(
            "legal_name: Test Officer",
            &format!("legal_name: {director}"),
        ),
    )
    .expect("rewrite");

    let person = store::person(&sb.paths, "self-signer").expect("record");
    let by_code = reg.lifecycle.by_code().expect("codes");
    let entry = issue::one(
        &FileStore::new(&sb.paths),
        &FileBlobs::new(&sb.paths),
        &sb.paths,
        &reg,
        &person,
        by_code.get("NDO").expect("declared"),
        d("2026-09-06"),
        "rohan",
    )
    .expect("issues, with the Director substituted out");

    assert!(
        !entry.signed_by.contains(&"director".to_owned()),
        "the subject must not be the signatory: {:?}",
        entry.signed_by
    );
    assert!(
        !entry.signed_by.is_empty(),
        "somebody signed it: {:?}",
        entry.signed_by
    );

    // ASSERTED ON THE PAPER, not only the register. The register can be right while the
    // rendered document names the wrong person, and the rendered document is what
    // somebody signs.
    let Some(text) = rendered_text(&sb.paths, "self-signer") else {
        eprintln!("SKIP the rendered-text assertion: pdftotext is not installed");
        return;
    };
    assert!(
        !text.contains(&director),
        "the Director must not appear as signatory on his own document:\n{text}"
    );
}

#[test]
fn the_officer_gate_waits_only_on_officer_documents() {
    // An unfiltered gate would hold an officer at preboarding forever, waiting on the
    // internship agreement and the intern IP assignment -- neither of which anything
    // will ever issue to them.
    let sb = Sandbox::new();
    sb.officer("gated-officer");
    let reg = registry();
    let record =
        personnel::identify(&FileStore::new(&sb.paths), "gated-officer").expect("identified");

    let gates = personnel::unmet_gates(
        &FileStore::new(&sb.paths),
        &reg,
        &record,
        "preboarding",
        "officer",
    )
    .expect("gates computed");
    let codes: Vec<&str> = gates.iter().map(|g| g.code.as_str()).collect();

    for code in ["NDO", "IPO", "COO", "ISO"] {
        assert!(codes.contains(&code), "{code} gates an officer: {codes:?}");
    }
    for internship_only in ["AGR", "IPA", "NDA", "COC", "GRV", "ISP", "DPN"] {
        assert!(
            !codes.contains(&internship_only),
            "{internship_only} must not gate an officer: {codes:?}"
        );
    }
}

/// Whether a typesetter is installed. Same pattern, and the same reason, as `signing.rs`:
/// these render real PDFs because the guarantee is about output bytes, and a CI runner has
/// no TeX Live. Skipping is recorded rather than hidden.
fn skip_without_typesetter(test: &str) -> bool {
    if tool_docket::adapters::typeset::detect().is_ok() {
        return false;
    }
    eprintln!("SKIP {test}: no typesetter installed (see DECISION-DEBT D-35)");
    true
}

#[test]
fn the_officer_documents_typeset_and_name_the_director() {
    if skip_without_typesetter("the_officer_documents_typeset_and_name_the_director") {
        return;
    }
    // NDO and IPO are new templates that nothing had ever rendered. A LaTeX template that
    // does not compile is a document that cannot be issued, and the failure appears at the
    // moment an operator is trying to onboard somebody.
    let sb = Sandbox::new();
    let person = sb.officer("render-officer");
    let reg = registry();
    let by_code = reg.lifecycle.by_code().expect("codes");

    for code in ["NDO", "IPO", "COO", "ISO", "DPO"] {
        let doc = by_code.get(code).expect("declared");
        let issued = issue::one(
            &FileStore::new(&sb.paths),
            &FileBlobs::new(&sb.paths),
            &sb.paths,
            &reg,
            &person,
            doc,
            d("2026-09-06"),
            "rohan",
        );
        assert!(issued.is_ok(), "{code} renders: {:?}", issued.err());
        assert_eq!(issued.expect("checked on the line above").code, code);
    }

    let pdfs = store::register(&sb.paths).expect("register");
    assert_eq!(pdfs.len(), 5, "every officer document is registered");

    // THE SIGNATURE BLOCK RESOLVES FROM `signed_by_class`, NOT FROM THE TEMPLATE, and the
    // assertion is made against the RENDERED TEXT because that is what a reader signs. If
    // it regressed to the CEO/CTO pair, an officer document would be executed by two people
    // who are not the counterparty this arrangement names — and it would look correct in
    // the registry while being wrong on paper.
    let Some(text) = rendered_text(&sb.paths, "render-officer") else {
        eprintln!("SKIP the rendered-text assertion: pdftotext is not installed");
        return;
    };
    assert!(
        text.contains("PRIYANSHU SHARMA"),
        "the Director signs an officer document; rendered text was:\n{text}"
    );
    assert!(
        !text.contains("RUHANI KHURANA") && !text.contains("ROHAN BATRA"),
        "the CEO/CTO pair must not appear on an officer document:\n{text}"
    );
    assert!(
        text.contains("Officer"),
        "the counterparty is labelled Officer"
    );
}

/// The text of every PDF issued to a person, concatenated. `None` when `pdftotext` is
/// absent, so the caller can skip rather than fail on a machine that lacks it.
fn rendered_text(paths: &Paths, slug: &str) -> Option<String> {
    let mut out = String::new();
    let dir = paths.store.join("documents");
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "pdf") {
            let result = std::process::Command::new("pdftotext")
                .arg(&path)
                .arg("-")
                .output()
                .ok()?;
            out.push_str(&String::from_utf8_lossy(&result.stdout));
        }
    }
    let _ = slug;
    (!out.is_empty()).then_some(out)
}

#[test]
fn an_officer_is_not_blocked_by_the_intern_preconditions() {
    // `blockers` refused an officer for `education.currently_enrolled is false` and
    // `engagement.end_date is unset`. Both are the INTERN class's preconditions and say
    // so in their own message text — they exist because an unpaid student placement is
    // not employment. `person-classes.yaml` forbids `education` for an officer and does
    // not require an end date, so those refusals were unsatisfiable: the operator could
    // not fix them without breaking a different rule. Found by running the bootstrap.
    let officer = person::Person::new(
        "t".to_owned(),
        serde_yaml::from_str(
            "engagement: {class: officer, stage: preboarding, start_date: 2026-09-06}\n\
             identity: {legal_name: Test Officer, date_of_birth: 1995-04-12}\n",
        )
        .expect("parses"),
    );
    let blocked = person::blockers(&officer, Some("Dehradun"));
    assert!(
        blocked.is_empty(),
        "an officer record is issuable without education or an end date: {blocked:?}"
    );
}

#[test]
fn an_intern_is_still_blocked_by_them() {
    // The other half. Scoping the checks to the intern class must not have removed them
    // from the class they were written for — the unpaid basis genuinely does rest on
    // enrolment, and an unpaid internship genuinely must be time-bound.
    let intern = person::Person::new(
        "t".to_owned(),
        serde_yaml::from_str(
            "engagement: {class: intern, stage: preboarding, start_date: 2026-09-01}\n\
             identity: {legal_name: Test Person, date_of_birth: 2005-04-12}\n\
             education: {currently_enrolled: false}\n",
        )
        .expect("parses"),
    );
    let blocked: Vec<String> = person::blockers(&intern, Some("Dehradun"))
        .iter()
        .map(ToString::to_string)
        .collect();
    assert!(
        blocked.iter().any(|b| b.contains("currently_enrolled")),
        "enrolment still gates an intern: {blocked:?}"
    );
    assert!(
        blocked.iter().any(|b| b.contains("end_date")),
        "an internship is still time-bound: {blocked:?}"
    );
}

#[test]
fn the_published_team_joins_the_registries_rather_than_restating_them() {
    // `team.yaml` carries the public summary and nothing else. Name and designation come
    // from `signatories.yaml`, so the office someone holds cannot differ between the
    // document they sign and the page that names them.
    let reg = registry();
    let team = tool_docket::application::team::published(&reg).expect("team resolves");

    assert_eq!(team.len(), 4, "four officers are published");
    assert_eq!(team[0].short, "CEO", "declared order is honoured");
    assert_eq!(team[3].short, "Director");

    for member in &team {
        let sig = reg
            .signatories
            .iter()
            .find(|s| s.id == member.id)
            .expect("published members exist as signatories");
        assert_eq!(member.name, sig.legal_name, "the name is not restated");
        assert_eq!(member.designation, sig.designation);
        for locale in ["en", "ar"] {
            let summary = member.summary.get(locale);
            assert!(summary.is_some(), "{} has no {locale} summary", member.id);
            let summary = summary.expect("checked on the line above");
            assert!(!summary.trim().is_empty());
            assert!(
                !summary.ends_with('\n'),
                "folded scalars are trimmed, or they render as stray whitespace"
            );
        }
    }
}

#[test]
fn a_registry_edit_makes_the_officers_page_export_stale() {
    // D-48. The export is a file committed in another repository, so nothing over there can
    // tell that signatories.yaml moved and nobody re-ran `docket team`. This is the half that
    // is local, and it was the half actually going wrong.
    use tool_docket::application::team;

    let sb = Sandbox::new();
    let reg = registry();

    // NO RECEIPT IS SILENT. The receipt lives in the gitignored store, so warning here would
    // fire on every fresh checkout and every CI run forever — the always-present warning
    // lifecycle.yaml records as the reason a genuine one gets scrolled past. The failure this
    // detects is "edited and not re-exported", which needs a prior export to exist at all.
    assert_eq!(
        team::export_staleness(&sb.paths).expect("staleness computes"),
        None,
        "a checkout that has never exported is not evidence the published page is wrong"
    );

    team::record_export(&sb.paths, &reg, "2026-09-07").expect("records the export");
    assert_eq!(
        team::export_staleness(&sb.paths).expect("staleness computes"),
        None,
        "current immediately after an export"
    );

    // A receipt whose digest no longer matches the sources IS reported. Written directly
    // rather than by editing a registry: the sandbox shares the real repository root, so a
    // test that mutated `registry/team.yaml` would be editing the live one.
    let receipt = sb.paths.store.join("exports").join("team.json");
    std::fs::write(
        &receipt,
        r#"{"source_digest":"0000000000000000000000000000000000000000000000000000000000000000","exported_on":"2026-09-01","members":4}"#,
    )
    .expect("receipt is writable");

    let stale = team::export_staleness(&sb.paths).expect("staleness computes");
    let stale = stale.expect("a mismatched digest is reported");
    assert!(
        stale.contains("2026-09-01"),
        "it names when the export was taken: {stale}"
    );
    assert!(
        stale.contains("docket team"),
        "it says how to fix it: {stale}"
    );

    // The digest covers BOTH sources, because the export is a join: signatories.yaml supplies
    // name and designation, team.yaml the summary and the order. A digest of one alone would
    // miss half the ways the export goes stale.
    let digest = team::source_digest(&sb.paths).expect("digest computes");
    assert_eq!(digest.len(), 64, "sha256, hex");
    assert_ne!(digest, "0".repeat(64));
}

// ── ADR-0077: two companies, one archive.

#[test]
fn the_ceo_and_director_being_one_person_falls_through_to_the_cto() {
    // The operator's decision of 2026-09-09, arrived at by DATA rather than by a
    // branch: Priyanshu holds both `ceo` and `director`, so for his own documents
    // both entries are the subject and the substitution reaches `cto`.
    let reg = registry();
    let doc = reg
        .lifecycle
        .documents
        .iter()
        .find(|d| d.code == "NDO")
        .expect("NDO");
    let priyanshu = person::Person::new(
        "priyanshu-sharma".to_owned(),
        serde_yaml::from_str(
            "engagement: {class: officer}\nidentity: {legal_name: PRIYANSHU SHARMA}\n",
        )
        .expect("yaml"),
    );

    let signers = issue::resolve_signatories(&reg, &priyanshu, doc, "PAR");
    assert_eq!(signers, vec!["cto".to_owned()], "{signers:?}");
}

#[test]
fn a_cyber_document_is_never_countersigned_by_a_paridyum_only_officer() {
    // Aditya holds no office at PARIDYUM CYBER. A signature from him on a Cyber
    // document is one nobody could account for.
    let reg = registry();
    let cyber: Vec<&str> = reg
        .active_signatories_of("PCY")
        .map(|s| s.legal_name.as_str())
        .collect();
    assert!(
        !cyber.contains(&"ADITYA SHRIVASTAVA"),
        "a Paridyum-only officer is a Cyber signatory: {cyber:?}"
    );
    assert!(cyber.contains(&"RUHANI KHURANA"), "{cyber:?}");
    assert!(
        cyber.first() == Some(&"PRIYANSHU SHARMA"),
        "Paridyum's Director drives Cyber on paper, so he is the first fallback: {cyber:?}"
    );
}

#[test]
fn one_person_holds_office_in_two_companies_on_one_identifier() {
    // The thing that could not be represented before ADR-0077 D3: Rohan is CTO of
    // both companies at the same time, and is ONE person with ONE archive.
    let dir = tempfile::tempdir().expect("tempdir");
    let people = dir.path().join("people");
    std::fs::create_dir_all(&people).expect("people");
    common::filled_officer_template(&root(), &people.join("_TEMPLATE-officer.yaml"));
    let paths = Paths {
        root: root(),
        people,
        store: dir.path().join("store"),
    };
    let reg = registry();

    let first = personnel::create(
        &FileStore::new(&paths),
        &paths,
        &reg,
        "rohan",
        "officer",
        "PAR",
        d("2026-10-01"),
        "rohan",
    )
    .expect("paridyum engagement");
    let second = personnel::reengage(
        &FileStore::new(&paths),
        &reg,
        "rohan",
        "officer",
        "PCY",
        d("2026-10-01"),
        "rohan",
    )
    .expect("a concurrent engagement at another company is allowed");

    assert_eq!(
        first.person_id, second.person_id,
        "a second company allocated a second identity"
    );
    assert_eq!(first.engagement_id.as_str(), "PAR-OFC-2026-0001");
    assert_eq!(
        second.engagement_id.as_str(),
        "PCY-OFC-2026-0001",
        "the second company restarted its own engagement sequence, as it should"
    );
}

#[test]
fn a_second_engagement_at_the_same_company_is_still_refused() {
    // The protection the original refusal was actually providing, kept.
    let dir = tempfile::tempdir().expect("tempdir");
    let people = dir.path().join("people");
    std::fs::create_dir_all(&people).expect("people");
    common::filled_officer_template(&root(), &people.join("_TEMPLATE-officer.yaml"));
    let paths = Paths {
        root: root(),
        people,
        store: dir.path().join("store"),
    };
    let reg = registry();

    personnel::create(
        &FileStore::new(&paths),
        &paths,
        &reg,
        "rohan",
        "officer",
        "PAR",
        d("2026-10-01"),
        "rohan",
    )
    .expect("creates");
    let err = personnel::reengage(
        &FileStore::new(&paths),
        &reg,
        "rohan",
        "officer",
        "PAR",
        d("2026-10-01"),
        "rohan",
    )
    .expect_err("a second engagement at the SAME company must be refused");
    let text = format!("{err:#}");
    assert!(
        text.contains("already holds an open engagement at PAR"),
        "{text}"
    );
    assert!(text.contains("DIFFERENT company is allowed"), "{text}");
}

// ── ADR-0077 + the operator's rule of 2026-09-09: PARIDYUM owns every other
// company. Checked rather than commented, because decision 4 resolves signing
// authority through the relationship.

/// Load the real registry with `org.yaml` swapped for `yaml`, and run `check`.
fn findings_with_org(yaml: &str) -> Vec<String> {
    let dir = tempfile::tempdir().expect("tempdir");
    let reg_dir = dir.path().join("registry");
    std::fs::create_dir_all(&reg_dir).expect("registry dir");
    for entry in std::fs::read_dir(root().join("registry")).expect("read registry") {
        let src = entry.expect("entry").path();
        let name = src.file_name().expect("name").to_owned();
        if name == "org.yaml" {
            std::fs::write(reg_dir.join(&name), yaml).expect("org");
        } else {
            std::fs::copy(&src, reg_dir.join(&name)).expect("copy");
        }
    }
    for shared in ["templates", "latex", "people", "manifest.yaml"] {
        let link = dir.path().join(shared);
        std::os::unix::fs::symlink(root().join(shared), &link).ok();
    }
    let paths = Paths {
        root: dir.path().to_path_buf(),
        people: dir.path().join("people"),
        store: dir.path().join("store"),
    };
    let reg = Registry::load(&paths).expect("registry loads");
    let mut items = Vec::new();
    tool_docket::application::check::organisation_findings(&mut items, &reg);
    items
}

const TWO_ROOTS: &str = r#"
version: 2
companies:
  - code: PAR
    default: true
    name: PARIDYUM
    jurisdiction: {forum: Dehradun}
    reference: {format: "PAR/{kind}/{year}/{seq:04d}"}
  - code: PCY
    name: PARIDYUM CYBER
    jurisdiction: {forum: Dehradun}
    reference: {format: "PCY/{kind}/{year}/{seq:04d}"}
"#;

#[test]
fn a_company_that_nobody_owns_is_blocking() {
    let found = findings_with_org(TWO_ROOTS).join("\n");
    assert!(
        found.contains("2 companies declare no owner"),
        "PARIDYUM owns every other company, and an unowned second one passed: {found}"
    );
}

#[test]
fn ownership_that_loops_is_blocking() {
    let yaml = TWO_ROOTS.replace(
        "    default: true\n    name: PARIDYUM",
        "    default: true\n    owned_by: PCY\n    name: PARIDYUM",
    );
    let yaml = yaml.replace(
        "    name: PARIDYUM CYBER",
        "    owned_by: PAR\n    name: PARIDYUM CYBER",
    );
    let found = findings_with_org(&yaml).join("\n");
    assert!(found.contains("loops back on itself"), "{found}");
}

#[test]
fn an_owner_that_is_not_a_declared_company_is_blocking() {
    let yaml = TWO_ROOTS.replace(
        "    name: PARIDYUM CYBER",
        "    owned_by: PXX\n    name: PARIDYUM CYBER",
    );
    let found = findings_with_org(&yaml).join("\n");
    assert!(found.contains("not a declared company"), "{found}");
}

#[test]
fn two_companies_sharing_a_serial_format_is_blocking() {
    // The same serial issued to two people at two companies, and the serial is
    // what a third party verifies a document by.
    let yaml = TWO_ROOTS
        .replace(
            "    name: PARIDYUM CYBER",
            "    owned_by: PAR\n    name: PARIDYUM CYBER",
        )
        .replace(
            r#""PCY/{kind}/{year}/{seq:04d}""#,
            r#""PAR/{kind}/{year}/{seq:04d}""#,
        );
    let found = findings_with_org(&yaml).join("\n");
    assert!(found.contains("share the serial format"), "{found}");
}

#[test]
fn the_real_registry_satisfies_the_ownership_rule() {
    let yaml = std::fs::read_to_string(root().join("registry/org.yaml")).expect("org.yaml");
    let found = findings_with_org(&yaml).join("\n");
    assert!(
        !found.contains("owner") && !found.contains("loops") && !found.contains("serial format"),
        "the shipped registry breaks its own ownership rule: {found}"
    );
}

#[test]
fn a_person_with_two_open_engagements_is_refused_rather_than_resolved_by_recency() {
    // THE DEFECT THIS TEST EXISTS FOR, and it was live for an hour. `identify`
    // took the LAST ledger row for a slug, which is right while a person holds
    // one engagement at a time and silently wrong the moment they hold two:
    // every per-person command resolved Rohan to Paridyum Cyber and made his
    // Paridyum office unreachable. `issue` reads the company off the engagement,
    // so his Paridyum NDA would have been issued on Cyber's letterhead under a
    // PCY/ serial -- and an issued document is corrected only by reissuing.
    let dir = tempfile::tempdir().expect("tempdir");
    let people = dir.path().join("people");
    std::fs::create_dir_all(&people).expect("people");
    common::filled_officer_template(&root(), &people.join("_TEMPLATE-officer.yaml"));
    let paths = Paths {
        root: root(),
        people,
        store: dir.path().join("store"),
    };
    let reg = registry();

    personnel::create(
        &FileStore::new(&paths),
        &paths,
        &reg,
        "rohan",
        "officer",
        "PAR",
        d("2026-10-01"),
        "rohan",
    )
    .expect("creates");
    personnel::reengage(
        &FileStore::new(&paths),
        &reg,
        "rohan",
        "officer",
        "PCY",
        d("2026-10-01"),
        "rohan",
    )
    .expect("second company");

    let err = personnel::identify(&FileStore::new(&paths), "rohan")
        .expect_err("ambiguity must be refused");
    let text = format!("{err:#}");
    assert!(text.contains("holds 2 open engagements"), "{text}");
    assert!(text.contains("PAR-OFC-2026-0001"), "{text}");
    assert!(text.contains("PCY-OFC-2026-0001"), "{text}");

    // Named, it resolves to exactly what was asked for -- both ways round.
    let par = personnel::identify_at(&FileStore::new(&paths), "rohan", Some("PAR")).expect("PAR");
    assert_eq!(par.engagement_id.as_str(), "PAR-OFC-2026-0001");
    let pcy = personnel::identify_at(&FileStore::new(&paths), "rohan", Some("PCY")).expect("PCY");
    // First engagement at PCY in this store: the sequence is per company.
    assert_eq!(pcy.engagement_id.as_str(), "PCY-OFC-2026-0001");

    // A company they hold no office at is an error, not the other engagement.
    let wrong = personnel::identify_at(&FileStore::new(&paths), "rohan", Some("PXX"))
        .expect_err("must not fall back to an engagement elsewhere");
    assert!(format!("{wrong:#}").contains("holds no engagement at PXX"));

    // And one engagement each still needs no flag.
    personnel::create(
        &FileStore::new(&paths),
        &paths,
        &reg,
        "aditya",
        "officer",
        "PAR",
        d("2026-10-01"),
        "rohan",
    )
    .expect("creates");
    let solo = personnel::identify(&FileStore::new(&paths), "aditya").expect("unambiguous");
    assert_eq!(solo.engagement_id.as_str(), "PAR-OFC-2026-0002");

    // `engagements` is the view that shows both.
    let held = personnel::engagements(&FileStore::new(&paths), "rohan").expect("engagements");
    assert_eq!(held.len(), 2, "{held:?}");
}

// ── Removal. RFC-0042 §5: ceasing is `active: false`, never deletion -- so the
// register keeps saying who was an officer in 2026, and stops letting them sign
// in 2027.

/// The real registry with some signatories forced inactive.
fn registry_without(ceased: &[&str]) -> Registry {
    let dir = tempfile::tempdir().expect("tempdir");
    let reg_dir = dir.path().join("registry");
    std::fs::create_dir_all(&reg_dir).expect("registry dir");
    for entry in std::fs::read_dir(root().join("registry")).expect("read registry") {
        let src = entry.expect("entry").path();
        let name = src.file_name().expect("name").to_owned();
        std::fs::copy(&src, reg_dir.join(&name)).expect("copy");
    }
    let path = reg_dir.join("signatories.yaml");
    let mut text = std::fs::read_to_string(&path).expect("signatories");
    for id in ceased {
        let marker = format!("- id: {id}\n");
        let at = text.find(&marker).expect("signatory id");
        let rest = &text[at..];
        let offset = rest.find("    active: true").expect("active flag");
        text.replace_range(at + offset..at + offset + 16, "    active: false");
    }
    std::fs::write(&path, text).expect("write");
    for shared in ["templates", "latex", "people", "manifest.yaml"] {
        std::os::unix::fs::symlink(root().join(shared), dir.path().join(shared)).ok();
    }
    let paths = Paths {
        root: dir.path().to_path_buf(),
        people: dir.path().join("people"),
        store: dir.path().join("store"),
    };
    let reg = Registry::load(&paths).expect("registry loads");
    // The temp directory drops here, deliberately. `Registry::load` parses every
    // file into owned structs and holds no path afterwards, so nothing needs the
    // directory to outlive this function -- and `mem::forget` on a `Drop` type is
    // denied by STD-0003 for exactly the reason it was reached for here.
    drop(dir);
    reg
}

fn officer_named(name: &str) -> person::Person {
    person::Person::new(
        "x".to_owned(),
        serde_yaml::from_str(&format!(
            "engagement: {{class: officer}}\nidentity: {{legal_name: {name}}}\n"
        ))
        .expect("yaml"),
    )
}

fn nda_officer(reg: &Registry) -> &tool_docket::domain::lifecycle::Document {
    reg.lifecycle
        .documents
        .iter()
        .find(|d| d.code == "NDO")
        .expect("NDO")
}

#[test]
fn a_ceased_officer_stops_signing_new_documents() {
    // THE DEFECT THIS TEST EXISTS FOR, found by simulating a removal rather than
    // by reading the code. Officer documents route to `director`, and the
    // substitution only ever asked "is this the subject" -- so marking Priyanshu
    // `active: false` left him named as the signatory on every document issued
    // afterwards, and `docket check` reported nothing.
    let reg = registry_without(&["ceo", "director"]);
    let signers = issue::resolve_signatories(
        &reg,
        &officer_named("ADITYA SHRIVASTAVA"),
        nda_officer(&reg),
        "PAR",
    );
    assert!(
        !signers.contains(&"director".to_owned()),
        "a ceased officer is still signing: {signers:?}"
    );
    assert_eq!(signers, vec!["cto".to_owned()], "{signers:?}");
}

#[test]
fn removing_every_signatory_of_a_company_yields_no_signatory_rather_than_a_stale_one() {
    // Cyber's chain is two deep: Priyanshu for the holding company, then Ruhani.
    // Removing both must leave the document unsignable -- which `refuse_if_misrouted`
    // turns into a refusal -- never a blank or a stale name.
    let reg = registry_without(&["cyber-holding", "cyber-ceo", "cyber-cto"]);
    let signers = issue::resolve_signatories(
        &reg,
        &officer_named("RUHANI KHURANA"),
        nda_officer(&reg),
        "PCY",
    );
    assert!(
        signers.is_empty(),
        "a company with no active signatory produced one: {signers:?}"
    );
}

#[test]
fn a_ceased_officer_is_still_in_the_register() {
    // RFC-0042 §5: never deletion. A register that cannot say who was an officer
    // in 2026 cannot verify a document issued in 2026.
    let reg = registry_without(&["ceo", "director"]);
    assert!(
        reg.signatories.iter().any(|s| s.id == "director"),
        "the ceased officer was removed from the register rather than deactivated"
    );
    assert!(
        !reg.active_signatories_of("PAR").any(|s| s.id == "director"),
        "the ceased officer is still active"
    );
}

#[test]
fn a_company_whose_signatories_are_all_one_person_is_blocking() {
    // ADR-0087 D4 and the corrected D-59. The failure this prevents appears at
    // issuance, on the officer hardest to replace: `docket issue` refuses a
    // document whose subject and signatory are the same person, so a company
    // with one human among its signatories cannot issue that human anything.
    //
    // Counted by PERSON, not by entry -- Priyanshu holds both `ceo` and
    // `director` at Paridyum, which is two rows and one person. A row count
    // would call that covered.
    let reg = registry_without(&["cyber-holding", "cyber-cto"]);
    let mut items = Vec::new();
    tool_docket::application::check::organisation_findings(&mut items, &reg);
    let text = items.join("\n");
    assert!(
        text.contains("only one person among its active signatories"),
        "a one-person company was not reported: {text}"
    );
    assert!(text.contains("RUHANI KHURANA"), "{text}");
}

#[test]
fn removing_one_of_cybers_three_leaves_it_able_to_issue() {
    // The measurement that corrected D-59: the chain is two deep and survives
    // losing Priyanshu. Ruhani's documents fall to Rohan and his to Ruhani.
    let reg = registry_without(&["cyber-holding"]);
    let mut items = Vec::new();
    tool_docket::application::check::organisation_findings(&mut items, &reg);
    assert!(
        !items.iter().any(|f| f.contains("only one person")),
        "{items:?}"
    );

    let doc = nda_officer(&reg);
    let ruhani = issue::resolve_signatories(&reg, &officer_named("RUHANI KHURANA"), doc, "PCY");
    let rohan = issue::resolve_signatories(&reg, &officer_named("ROHAN BATRA"), doc, "PCY");
    assert_eq!(ruhani, vec!["cyber-cto".to_owned()]);
    assert_eq!(rohan, vec!["cyber-ceo".to_owned()]);
}

#[test]
fn the_shipped_registry_can_countersign_every_company() {
    let reg = registry();
    let mut items = Vec::new();
    tool_docket::application::check::organisation_findings(&mut items, &reg);
    assert!(
        !items.iter().any(|f| f.contains("active signatories")),
        "{items:?}"
    );
}
