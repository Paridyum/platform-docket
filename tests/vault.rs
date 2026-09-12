//! The encrypted archive, and the erasure it exists to make honest.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::PathBuf;

use tool_docket::adapters::clock::parse_iso;
use tool_docket::adapters::{crypt, registry::Registry, store};
use tool_docket::application::{issue, personnel, sign, vault, verify};
use tool_docket::config::Paths;
use tool_docket::domain::signature::Method;

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

/// The vault needs `age` and a typesetter: it encrypts real PDFs, and a mocked one would not
/// test the thing that matters. Skipped rather than failed where either is absent, the same
/// pattern the issuance suite uses — and printed, so a green run is not mistaken for coverage.
fn skip(test: &str) -> bool {
    if crypt::detect().is_err() {
        eprintln!("SKIP {test}: age is not installed");
        return true;
    }
    if tool_docket::adapters::typeset::detect().is_err() {
        eprintln!("SKIP {test}: no typesetter installed (see DECISION-DEBT D-35)");
        return true;
    }
    false
}

struct Sandbox {
    _dir: tempfile::TempDir,
    paths: Paths,
    /// This sandbox's own estate key, so a test never touches the operator's.
    estate: PathBuf,
}

impl Sandbox {
    /// A store with its OWN estate key, so a test never touches the operator's.
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("temp dir");
        let people = dir.path().join("people");
        std::fs::create_dir_all(&people).expect("people dir");
        common::filled_officer_template(&root(), &people.join("_TEMPLATE-officer.yaml"));

        // GENERATED ONLY WHERE `age` EXISTS, and this is D-45's lesson arriving a
        // second time. Every test that USES the key already calls `skip` first;
        // this constructor demanded `age-keygen` unconditionally, so a test that
        // reads nothing but paths -- `the_keystore_is_never_in_what_gets_uploaded`
        // -- failed on a runner with no typesetter or age, while passing on every
        // developer machine. The guard belongs on the resource the test actually
        // uses, not on the fixture that happens to build it.
        let keys = dir.path().join("estate");
        std::fs::create_dir_all(&keys).expect("estate dir");
        let key_file = keys.join("keys.txt");
        if crypt::detect().is_ok() {
            let identity = crypt::generate_identity().expect("age-keygen runs");
            std::fs::write(&key_file, identity).expect("estate key written");
        }

        Self {
            estate: key_file,
            paths: Paths {
                root: root(),
                people,
                store: dir.path().join("store"),
            },
            _dir: dir,
        }
    }

    /// An officer with one document issued.
    fn with_document(&self, slug: &str) -> tool_docket::domain::identity::Record {
        let reg = registry();
        let record = personnel::create(
            &self.paths,
            &reg,
            slug,
            "officer",
            "PAR",
            d("2026-09-06"),
            "rohan",
        )
        .expect("creates");
        let person = store::person(&self.paths, slug).expect("record");
        let doc = reg
            .lifecycle
            .by_code()
            .expect("codes")
            .get("NDO")
            .copied()
            .expect("NDO is declared");
        issue::one(&self.paths, &reg, &person, doc, d("2026-09-06"), "rohan").expect("issues");
        record
    }
}

#[test]
fn sealing_is_idempotent() {
    if skip("sealing_is_idempotent") {
        return;
    }
    // `age` output carries a fresh ephemeral key each run, so re-encrypting would rewrite every
    // object every time and make a sync diff meaningless — every file would look changed.
    let sb = Sandbox::new();
    sb.with_document("seal-twice");

    let first = vault::seal(&sb.paths, "seal-twice", None, &sb.estate).expect("seals");
    assert_eq!(first.written.len(), 1);
    assert_eq!(first.skipped, 0);

    let second = vault::seal(&sb.paths, "seal-twice", None, &sb.estate).expect("seals again");
    assert!(second.written.is_empty(), "nothing re-encrypted");
    assert_eq!(second.skipped, 1);
}

#[test]
fn a_sealed_document_is_ciphertext_and_opens_back_to_the_original() {
    if skip("a_sealed_document_is_ciphertext_and_opens_back_to_the_original") {
        return;
    }
    let sb = Sandbox::new();
    let who = sb.with_document("round-trip");
    vault::seal(&sb.paths, "round-trip", None, &sb.estate).expect("seals");

    let entry = store::register(&sb.paths)
        .expect("register")
        .into_iter()
        .find(|e| e.person_id == who.person_id)
        .expect("one entry");
    let name = std::path::Path::new(&entry.pdf_path)
        .file_name()
        .and_then(|n| n.to_str())
        .expect("a filename")
        .to_owned();

    let sealed = std::fs::read(
        sb.paths
            .store
            .join("vault")
            .join(who.person_id.as_str())
            .join(format!("{name}.age")),
    )
    .expect("the sealed object exists");
    assert!(
        sealed.starts_with(b"age-encryption.org/v1"),
        "it is an age file, not a copy"
    );
    assert!(
        !sealed.starts_with(b"%PDF"),
        "the plaintext is not sitting there"
    );

    let opened = vault::open(
        &sb.paths,
        "round-trip",
        None,
        &format!("{name}.age"),
        &sb.estate,
    )
    .expect("opens");
    let original = std::fs::read(sb.paths.store.join(&entry.pdf_path)).expect("plaintext");
    assert_eq!(opened, original, "what comes back is what went in");
}

#[test]
fn erasure_destroys_the_vault_and_leaves_a_tombstone() {
    if skip("erasure_destroys_the_vault_and_leaves_a_tombstone") {
        return;
    }
    // THE POINT OF THE WHOLE DESIGN. Deleting files from a versioned cloud store is not
    // erasure, because the versions remain. Destroying the only key that opens them is.
    let sb = Sandbox::new();
    let who = sb.with_document("to-erase");
    vault::seal(&sb.paths, "to-erase", None, &sb.estate).expect("seals");

    let key = sb
        .paths
        .store
        .join("keys")
        .join(format!("{}.age", who.person_id.as_str()));
    let vault_dir = sb.paths.store.join("vault").join(who.person_id.as_str());
    let issued = sb.paths.store.join("issued").join(who.person_id.as_str());
    assert!(key.exists() && vault_dir.exists() && issued.exists());

    personnel::erase(&sb.paths, "to-erase", d("2026-09-07"), "rohan").expect("erases");

    assert!(!key.exists(), "the key is destroyed — this is the erasure");
    assert!(!vault_dir.exists(), "the ciphertext goes too");
    assert!(
        !issued.exists(),
        "and the plaintext copies, or the shredding would be decorative"
    );
    assert!(!sb.paths.person("to-erase").exists(), "the record is gone");

    let stones = vault::tombstones(&sb.paths).expect("tombstones readable");
    let stone = stones.last().expect("one was written");
    assert_eq!(stone.person_id, who.person_id.as_str());
    assert_eq!(stone.by, "rohan");
    assert!(stone.key_destroyed);
    assert_eq!(stone.objects, 1);
    // A tombstone is not personal data: an identifier, a date, an operator and counts.
    let json = serde_json::to_string(stone).expect("serialises");
    assert!(!json.contains("Test Officer"), "no name: {json}");
}

#[test]
fn an_erased_persons_document_still_verifies_from_their_own_copy() {
    if skip("an_erased_persons_document_still_verifies_from_their_own_copy") {
        return;
    }
    // THE PROPERTY THAT MAKES THE SPLIT WORK. `verify` compares a PRESENTED file against the
    // hash recorded at issuance, so erasing Paridyum's copy does not remove the person's
    // ability to prove what they were issued. Without this, permanence and erasure really
    // would be irreconcilable and one of them would have had to give.
    let sb = Sandbox::new();
    let who = sb.with_document("still-provable");
    let entry = store::register(&sb.paths)
        .expect("register")
        .into_iter()
        .find(|e| e.person_id == who.person_id)
        .expect("one entry");

    // Their copy, kept somewhere docket will never reach.
    let theirs = sb
        .paths
        .store
        .parent()
        .expect("a parent")
        .join("their-copy.pdf");
    std::fs::copy(sb.paths.store.join(&entry.pdf_path), &theirs).expect("they keep a copy");

    vault::seal(&sb.paths, "still-provable", None, &sb.estate).expect("seals");
    personnel::erase(&sb.paths, "still-provable", d("2026-09-07"), "rohan").expect("erases");

    let register = store::register(&sb.paths).expect("the register survives erasure");
    let outcome = verify::verify(&sb.paths, &register, &entry.serial, Some(&theirs))
        .expect("verification runs");
    assert!(
        outcome.matches(),
        "an erased person can still prove what they were issued"
    );
}

#[test]
fn opening_a_shredded_vault_says_why_rather_than_failing_to_decrypt() {
    if skip("opening_a_shredded_vault_says_why_rather_than_failing_to_decrypt") {
        return;
    }
    // A raw decryption error here would read as corruption. It is not: it is the system having
    // worked, and the message has to say so or somebody will go looking for a backup.
    let sb = Sandbox::new();
    sb.with_document("shredded");
    vault::seal(&sb.paths, "shredded", None, &sb.estate).expect("seals");
    personnel::erase(&sb.paths, "shredded", d("2026-09-07"), "rohan").expect("erases");

    let err = vault::open(&sb.paths, "shredded", None, "anything.age", &sb.estate)
        .expect_err("cannot open");
    let message = err.to_string();
    assert!(message.contains("permanent"), "{message}");
    assert!(
        message.contains("verify"),
        "it names what still works: {message}"
    );
}

#[test]
fn the_keystore_is_never_in_what_gets_uploaded() {
    // Not a behaviour test — a statement about the sync layout, asserted so that adding a path
    // to the upload set has to come past it. A key inside a versioned store cannot be
    // destroyed, and destroying it is the erasure mechanism (ADR-0069 D3).
    let sb = Sandbox::new();
    let (upload, never) = vault::syncable(&sb.paths);

    assert!(upload.ends_with("vault"), "only the ciphertext is uploaded");
    for forbidden in ["keys", "issued", "index"] {
        assert!(
            never.iter().any(|p| p.ends_with(forbidden)),
            "{forbidden} is named as never-upload"
        );
    }
    assert!(
        never.iter().any(|p| p == &sb.paths.people),
        "the person records are named as never-upload"
    );
    assert!(
        !never.contains(&upload),
        "the upload path is not also in the forbidden list"
    );
}

#[test]
fn the_register_backup_restores_verifiability_from_the_estate_key_alone() {
    if skip("the_register_backup_restores_verifiability_from_the_estate_key_alone") {
        return;
    }
    // `seal` covers issued PDFs and left the more important half unprotected: the register is
    // what makes a document verifiable in 2036, and it was in no uploaded path at all.
    //
    // This rehearses the recovery that D-50 says must be rehearsed — decrypt with nothing but
    // the escrowed estate key, and check that what comes back is enough to verify a document
    // somebody presents.
    let sb = Sandbox::new();
    let who = sb.with_document("restorable");
    let expected = store::register(&sb.paths)
        .expect("register")
        .into_iter()
        .find(|e| e.person_id == who.person_id)
        .expect("one entry");

    let written = vault::back_up_register(&sb.paths, &sb.estate).expect("backs up");
    assert!(
        written.iter().any(|n| n == "register-issued.jsonl"),
        "the register itself is in the bundle: {written:?}"
    );

    let bundle = sb
        .paths
        .store
        .join("vault")
        .join("_register")
        .join("register-issued.jsonl.age");
    let sealed = std::fs::read(&bundle).expect("the bundle exists");
    assert!(
        sealed.starts_with(b"age-encryption.org/v1"),
        "ciphertext, not a copy"
    );

    // The estate key, and nothing else. No person key, no store, no docket state.
    let identity = std::fs::read_to_string(&sb.estate).expect("estate key readable");
    let plain = crypt::decrypt(&identity, &sealed).expect("the estate key opens it");
    let first = String::from_utf8(plain).expect("text");
    let recovered: serde_json::Value =
        serde_json::from_str(first.lines().next().expect("one entry")).expect("json");

    assert_eq!(recovered["serial"], expected.serial.as_str());
    assert_eq!(recovered["pdf_sha256"], expected.pdf_sha256.as_str());
    // Serial plus hash is exactly what `verify` compares a presented file against, so this is
    // the whole of what recovery has to return.
}

#[test]
fn the_backup_never_carries_the_keystore() {
    if skip("the_backup_never_carries_the_keystore") {
        return;
    }
    // The bundle names its contents individually rather than sweeping `store/`. A glob would
    // have taken `keys/` — and a key inside a versioned store cannot be destroyed, after which
    // no erasure this system performs is real.
    let sb = Sandbox::new();
    sb.with_document("no-keys");
    vault::seal(&sb.paths, "no-keys", None, &sb.estate).expect("seals");
    vault::back_up_register(&sb.paths, &sb.estate).expect("backs up");

    let uploaded = sb.paths.store.join("vault");
    let mut found = Vec::new();
    let mut stack = vec![uploaded];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("readable").flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                found.push(path.to_string_lossy().into_owned());
            }
        }
    }
    assert!(!found.is_empty(), "something was uploaded");
    for path in &found {
        assert!(
            !path.contains("/keys/"),
            "a key reached the uploaded path: {path}"
        );
    }
}

#[test]
fn a_signed_scan_is_sealed_and_erased_with_everything_else() {
    if skip("a_signed_scan_is_sealed_and_erased_with_everything_else") {
        return;
    }
    // THE IRREPLACEABLE HALF. `sign.rs`: "wet ink leaves no trail of its own, so the scan IS the
    // evidence." An issued PDF was handed to the person in their pack, so losing Paridyum's copy
    // is recoverable from them; a returned scan exists nowhere else.
    //
    // It was in neither the vault nor the register bundle, and `erase` deleted the issued PDFs
    // while leaving it — erasing the copies the person already had and keeping the one they did
    // not.
    let sb = Sandbox::new();
    let who = sb.with_document("with-scan");
    let entry = store::register(&sb.paths)
        .expect("register")
        .into_iter()
        .find(|e| e.person_id == who.person_id)
        .expect("one entry");

    let scan = sb.paths.store.parent().expect("a parent").join("scan.pdf");
    std::fs::write(&scan, b"a scanned signed copy").expect("scan written");
    sign::record(
        &sb.paths,
        &registry(),
        &entry.serial,
        Method::WetInk,
        d("2026-09-07"),
        d("2026-09-07"),
        Some(&scan),
        "rohan",
    )
    .expect("records the signature");

    let scans = sb.paths.scans_dir(who.person_id.as_str());
    assert!(scans.exists(), "the scan was filed");

    // Reported as needing sealing, and named as a scan so it is not mistaken for the PDF.
    let pending = vault::unsealed(&sb.paths).expect("status computes");
    assert!(
        pending.iter().any(|p| p.contains("signed scan")),
        "the scan is reported unsealed: {pending:?}"
    );

    vault::seal(&sb.paths, "with-scan", None, &sb.estate).expect("seals");
    assert!(
        vault::unsealed(&sb.paths).expect("status").is_empty(),
        "sealing covers the scan as well as the document"
    );

    personnel::erase(&sb.paths, "with-scan", d("2026-09-07"), "rohan").expect("erases");
    assert!(
        !scans.exists(),
        "the scan is destroyed with everything else, or the erasure keeps the one copy the \
         person does not have"
    );
}

#[test]
fn never_sealed_is_not_mistaken_for_shredded() {
    if skip("never_sealed_is_not_mistaken_for_shredded") {
        return;
    }
    // Both states have no key file, so keying on that reported "every issued document is sealed"
    // when nothing was. The tombstone is what distinguishes them — it is the record that erasure
    // actually happened.
    let sb = Sandbox::new();
    sb.with_document("not-yet");

    let before = vault::unsealed(&sb.paths).expect("status computes");
    assert_eq!(
        before.len(),
        1,
        "a never-sealed document is reported: {before:?}"
    );

    vault::seal(&sb.paths, "not-yet", None, &sb.estate).expect("seals");
    assert!(vault::unsealed(&sb.paths).expect("status").is_empty());

    personnel::erase(&sb.paths, "not-yet", d("2026-09-07"), "rohan").expect("erases");
    assert!(
        vault::unsealed(&sb.paths).expect("status").is_empty(),
        "a shredded document is not reported: re-sealing what erasure destroyed is not a task"
    );
}

#[test]
fn an_escrow_inside_the_store_is_refused() {
    if skip("an_escrow_inside_the_store_is_refused") {
        return;
    }
    // `store/vault/` is what gets uploaded. Every per-person key in a versioned cloud folder
    // means no erasure this system performs is real — and it is the one mistake that would be
    // completely silent.
    let sb = Sandbox::new();
    sb.with_document("escrowed");
    vault::seal(&sb.paths, "escrowed", None, &sb.estate).expect("seals");

    let inside = sb.paths.store.join("vault").join("escrow.age");
    let err = vault::escrow(&sb.paths, &sb.estate, &inside).expect_err("refused");
    assert!(err.to_string().contains("uploaded"), "{err}");
    assert!(!inside.exists(), "and nothing was written");

    let outside = sb
        .paths
        .store
        .parent()
        .expect("a parent")
        .join("offline.age");
    let count = vault::escrow(&sb.paths, &sb.estate, &outside).expect("written outside");
    assert_eq!(count, 1);
    let bytes = std::fs::read(&outside).expect("exists");
    assert!(bytes.starts_with(b"age-encryption.org/v1"), "ciphertext");
}

#[test]
fn a_shredding_after_an_escrow_is_reported_as_incomplete() {
    if skip("a_shredding_after_an_escrow_is_reported_as_incomplete") {
        return;
    }
    // The honest half of the escrow trade: with a copy on a shelf, `docket erase` destroys
    // docket's key and cannot reach that one, so erasure becomes ASSERTED rather than proven.
    // Nothing else in the system reports that, which is why this warning exists.
    let sb = Sandbox::new();
    assert_eq!(
        vault::escrow_outstanding(&sb.paths, None),
        None,
        "quiet until a shredding has happened"
    );

    sb.with_document("shred-after-escrow");
    vault::seal(&sb.paths, "shred-after-escrow", None, &sb.estate).expect("seals");
    personnel::erase(&sb.paths, "shred-after-escrow", d("2026-09-07"), "rohan").expect("erases");

    let warning = vault::escrow_outstanding(&sb.paths, None).expect("reported");
    assert!(warning.contains("not complete"), "{warning}");
    assert!(
        warning.contains("destroyed or re-exported"),
        "it says what to do: {warning}"
    );
}
