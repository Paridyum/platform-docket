//! ADR-0072: what a node holds, and whether a cold spare can bring it back.
//!
//! The suite the operator's question actually needs answered. ADR-0071 §3:
//! "a backup that has never been restored is a belief, not a control" -- so
//! these tests restore, on a second store, from the bundle and the key alone.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]

use std::path::PathBuf;

use platform_docket::adapters::clock::parse_iso;
use platform_docket::adapters::store::FileStore;
use platform_docket::adapters::{crypt, registry::Registry};
use platform_docket::application::{node, personnel, vault};
use platform_docket::config::Paths;

mod common;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn d(iso: &str) -> time::Date {
    parse_iso(iso).expect("date")
}

/// Printed rather than silent: a green run must not be mistaken for coverage.
fn skip(test: &str) -> bool {
    if crypt::detect().is_err() {
        eprintln!("SKIP {test}: age is not installed");
        return true;
    }
    false
}

/// A node: its own store and its own estate key, never the operator's.
struct Node {
    /// Read by `new_with_own_key`, and holds the temp directory open besides.
    dir: tempfile::TempDir,
    paths: Paths,
    estate: PathBuf,
}

impl Node {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("temp dir");
        let people = dir.path().join("people");
        std::fs::create_dir_all(&people).expect("people dir");
        common::filled_officer_template(&root(), &people.join("_TEMPLATE-officer.yaml"));
        Self {
            estate: PathBuf::new(),
            paths: Paths {
                root: root(),
                people,
                store: dir.path().join("store"),
            },
            dir,
        }
    }

    /// A node holding the SAME estate key as another. That sharing is the whole
    /// design: RUN-0004 keeps one key in a password manager and installs it per
    /// machine, which is why a spare can open a bundle it never encrypted.
    fn with_key(mut self, key: &std::path::Path) -> Self {
        self.estate = key.to_path_buf();
        self
    }

    fn new_with_own_key() -> Self {
        let node = Self::new();
        let key = node.dir.path().join("keys.txt");
        std::fs::write(&key, crypt::generate_identity().expect("age-keygen")).expect("key");
        node.with_key(&key)
    }

    fn add(&self, slug: &str) {
        let reg = Registry::load(&Paths::resolve(&root())).expect("registry");
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
        .expect("creates");
    }
}

#[test]
fn an_empty_store_puts_nothing_at_risk() {
    let node = Node::new();
    assert!(node::classify(&node.paths).expect("classify").is_empty());
    assert!(
        node::unreplicated(&node.paths)
            .expect("unreplicated")
            .is_empty()
    );
}

#[test]
fn a_register_with_no_bundle_behind_it_is_reported_as_lost() {
    let node = Node::new();
    node.add("vivaswan");
    let lost = node::unreplicated(&node.paths).expect("unreplicated");
    assert!(
        lost.iter().any(|n| n == "register"),
        "the register is unreplicated and was not reported: {lost:?}"
    );
}

#[test]
fn a_backup_clears_the_register_from_the_lost_list() {
    if skip("a_backup_clears_the_register_from_the_lost_list") {
        return;
    }
    let node = Node::new_with_own_key();
    node.add("vivaswan");
    vault::back_up_register(&node.paths, &node.estate).expect("backup");
    assert!(
        node::unreplicated(&node.paths)
            .expect("unreplicated")
            .is_empty(),
        "a backed-up register is still being reported as lost"
    );
}

#[test]
fn a_directory_nobody_classified_is_reported_unclassified() {
    // The failure `vault sync` structurally cannot report: a path in neither
    // the upload list nor the prohibition. ADR-0072 decision 5.
    let node = Node::new();
    let stray = node.paths.store.join("invoices");
    std::fs::create_dir_all(&stray).expect("stray dir");
    std::fs::write(stray.join("a.pdf"), b"x").expect("stray file");

    let items = node::classify(&node.paths).expect("classify");
    let found = items
        .iter()
        .find(|i| i.name == "invoices")
        .expect("the stray directory was not listed at all");
    assert_eq!(found.disposition, node::Disposition::Unclassified);
    assert!(
        node::unreplicated(&node.paths)
            .expect("unreplicated")
            .iter()
            .any(|n| n == "invoices"),
        "an unclassified path must count as at risk until someone decides"
    );
}

#[test]
fn keys_are_never_replicated_and_the_vault_always_is() {
    let node = Node::new();
    for name in ["keys", "index", "exports"] {
        std::fs::create_dir_all(node.paths.store.join(name)).expect("dir");
    }
    std::fs::create_dir_all(node.paths.store.join("vault")).expect("vault");
    let items = node::classify(&node.paths).expect("classify");
    let of = |n: &str| {
        items
            .iter()
            .find(|i| i.name == n)
            .map(|i| i.disposition)
            .expect("classified")
    };
    assert_eq!(of("keys"), node::Disposition::Never);
    assert_eq!(of("index"), node::Disposition::Never);
    assert_eq!(of("exports"), node::Disposition::Never);
    assert_eq!(of("vault"), node::Disposition::Replicate);
}

#[test]
fn a_cold_spare_rebuilds_the_register_from_the_bundle_and_the_key_alone() {
    if skip("a_cold_spare_rebuilds_the_register_from_the_bundle_and_the_key_alone") {
        return;
    }
    let lost = Node::new_with_own_key();
    for slug in ["vivaswan", "priyanshu", "aditya"] {
        lost.add(slug);
    }
    vault::back_up_register(&lost.paths, &lost.estate).expect("backup");

    // What the off-host store holds. Copied out, because the point is that the
    // spare needs nothing else from the lost machine.
    let offhost = tempfile::tempdir().expect("offhost");
    let bundle = offhost.path().join("_register");
    std::fs::create_dir_all(&bundle).expect("bundle dir");
    for entry in std::fs::read_dir(node::bundle_path(&lost.paths)).expect("bundle") {
        let src = entry.expect("entry").path();
        assert_eq!(
            src.extension().and_then(|e| e.to_str()),
            Some("age"),
            "ADR-0071 §1: only ciphertext leaves the machine -- {} did",
            src.display()
        );
        std::fs::copy(&src, bundle.join(src.file_name().expect("name"))).expect("copy");
    }

    let spare = Node::new().with_key(&lost.estate);
    let restored = node::restore(&bundle, &spare.estate, &spare.paths).expect("restore");
    assert!(!restored.is_empty());
    assert!(
        node::verify_restored(&FileStore::new(&spare.paths))
            .expect("verify")
            .is_empty(),
        "the restored chains do not verify"
    );

    let ledger = platform_docket::adapters::store::ledger(&spare.paths).expect("ledger");
    let slugs: Vec<&str> = ledger.iter().map(|r| r.slug.as_str()).collect();
    for slug in ["vivaswan", "priyanshu", "aditya"] {
        assert!(slugs.contains(&slug), "{slug} did not survive: {slugs:?}");
    }
}

#[test]
fn a_restored_spare_continues_the_sequence_rather_than_restarting_it() {
    if skip("a_restored_spare_continues_the_sequence_rather_than_restarting_it") {
        return;
    }
    // The failure this guards is the one ADR-0051 forbids outright: a spare
    // that restarted at 1 would reissue identifiers already on documents.
    let lost = Node::new_with_own_key();
    for slug in ["a", "b", "c"] {
        lost.add(slug);
    }
    vault::back_up_register(&lost.paths, &lost.estate).expect("backup");

    let spare = Node::new().with_key(&lost.estate);
    node::restore(&node::bundle_path(&lost.paths), &spare.estate, &spare.paths).expect("restore");
    spare.add("d");

    let ledger = platform_docket::adapters::store::ledger(&spare.paths).expect("ledger");
    let d_row = ledger.iter().find(|r| r.slug == "d").expect("d");
    assert_eq!(
        d_row.person_id.as_str(),
        "PAR-P-000004-4",
        "the sequence restarted instead of continuing"
    );
}

#[test]
fn a_restore_onto_a_live_store_is_refused_rather_than_merged() {
    if skip("a_restore_onto_a_live_store_is_refused_rather_than_merged") {
        return;
    }
    // THE DEFECT THIS TEST EXISTS FOR. The first guard tested `register()` --
    // `register/issued.jsonl` -- which does not exist until something has been
    // issued. A store holding three people and no documents therefore looked
    // empty, and a second restore overwrote people.jsonl and silently deleted
    // a person allocated after the backup.
    let lost = Node::new_with_own_key();
    lost.add("a");
    vault::back_up_register(&lost.paths, &lost.estate).expect("backup");

    let spare = Node::new().with_key(&lost.estate);
    node::restore(&node::bundle_path(&lost.paths), &spare.estate, &spare.paths).expect("restore");
    // Allocated on the spare AFTER the backup: this is what an overwrite eats.
    spare.add("later");

    let err = node::restore(&node::bundle_path(&lost.paths), &spare.estate, &spare.paths)
        .expect_err("a second restore must be refused");
    let text = format!("{err:#}");
    assert!(text.contains("does not merge"), "{text}");
    assert!(text.contains("ADR-0051"), "{text}");

    let ledger = platform_docket::adapters::store::ledger(&spare.paths).expect("ledger");
    assert!(
        ledger.iter().any(|r| r.slug == "later"),
        "the refused restore still destroyed a record"
    );
}

#[test]
fn a_missing_bundle_says_what_to_run_rather_than_failing_obscurely() {
    if skip("a_missing_bundle_says_what_to_run_rather_than_failing_obscurely") {
        return;
    }
    let spare = Node::new_with_own_key();
    let err = node::restore(
        &spare.paths.store.join("nothing-here"),
        &spare.estate,
        &spare.paths,
    )
    .expect_err("no bundle");
    assert!(format!("{err:#}").contains("docket vault backup"));
}

#[test]
fn plaintext_in_the_bundle_is_refused() {
    if skip("plaintext_in_the_bundle_is_refused") {
        return;
    }
    // ADR-0071 §1: the `.age` extension is the contract. A file without it is
    // either a plaintext leak or of unknown provenance; both beat restoring.
    let spare = Node::new_with_own_key();
    let bundle = spare.paths.store.join("bundle");
    std::fs::create_dir_all(&bundle).expect("bundle");
    std::fs::write(bundle.join("register-people.jsonl"), b"{}").expect("plaintext");
    let err = node::restore(&bundle, &spare.estate, &spare.paths).expect_err("refused");
    assert!(format!("{err:#}").contains("ADR-0071"));
}

#[test]
fn a_wrong_key_cannot_open_the_bundle() {
    if skip("a_wrong_key_cannot_open_the_bundle") {
        return;
    }
    // The property the whole design rests on: the off-host store holds bytes
    // that are useless without the key from the password manager.
    let lost = Node::new_with_own_key();
    lost.add("a");
    vault::back_up_register(&lost.paths, &lost.estate).expect("backup");

    let stranger = Node::new_with_own_key();
    let err = node::restore(
        &node::bundle_path(&lost.paths),
        &stranger.estate,
        &stranger.paths,
    )
    .expect_err("a different key must not open it");
    assert!(format!("{err:#}").contains("decrypting"));
}

// ── docket store push: the guard D-51 said belonged "in whatever performs the
// upload". Every refusal below is a real upload accident, not a hypothetical.

#[test]
fn push_copies_only_the_vault_and_is_idempotent() {
    if skip("push_copies_only_the_vault_and_is_idempotent") {
        return;
    }
    let node = Node::new_with_own_key();
    node.add("vivaswan");
    vault::back_up_register(&node.paths, &node.estate).expect("backup");
    let dest = tempfile::tempdir().expect("dest");

    let first = node::push(&node.paths, dest.path()).expect("push");
    assert!(!first.is_empty());
    assert!(
        first.iter().all(|p| !p.already_current),
        "a first push should copy everything"
    );
    assert!(
        first.iter().all(|p| p.name.starts_with("vault/")),
        "something outside the vault was uploaded: {:?}",
        first.iter().map(|p| &p.name).collect::<Vec<_>>()
    );

    let second = node::push(&node.paths, dest.path()).expect("second push");
    assert!(
        second.iter().all(|p| p.already_current),
        "an unchanged store re-uploaded everything"
    );
}

#[test]
fn push_refuses_a_destination_inside_the_store() {
    if skip("push_refuses_a_destination_inside_the_store") {
        return;
    }
    let node = Node::new_with_own_key();
    node.add("a");
    vault::back_up_register(&node.paths, &node.estate).expect("backup");
    let err = node::push(&node.paths, &node.paths.store.join("uploads")).expect_err("refused");
    assert!(format!("{err:#}").contains("inside the store"));
}

#[test]
fn push_refuses_a_relative_destination_inside_the_store() {
    // THE DEFECT THIS TEST EXISTS FOR. The guard compared the destination as
    // given against an absolute store path, so a RELATIVE `store/uploads` did
    // not match, the refusal passed, and push wrote into the directory it
    // exists to protect. Found by running it, not by reading it.
    if skip("push_refuses_a_relative_destination_inside_the_store") {
        return;
    }
    let node = Node::new_with_own_key();
    node.add("a");
    vault::back_up_register(&node.paths, &node.estate).expect("backup");

    // A relative path that resolves inside this node's store.
    let relative = pathdiff_from_cwd(&node.paths.store.join("uploads"));
    let err = node::push(&node.paths, &relative).expect_err("a relative path must not slip past");
    assert!(format!("{err:#}").contains("inside the store"), "{err:#}");
}

/// A path expressed relative to the working directory, so the test exercises the
/// absolutising step rather than the absolute comparison.
fn pathdiff_from_cwd(target: &std::path::Path) -> PathBuf {
    let cwd = std::env::current_dir().expect("cwd");
    let mut up = PathBuf::new();
    for _ in cwd.components() {
        up.push("..");
    }
    up.join(target.strip_prefix("/").unwrap_or(target))
}

#[test]
fn push_refuses_when_a_never_list_directory_is_already_at_the_destination() {
    // The accident that matters: a `keys/` uploaded once, by hand, months ago.
    // The only check that inspects what is ALREADY there rather than what is
    // being sent -- and the only one that can catch an erasure that was never
    // real.
    if skip("push_refuses_when_a_never_list_directory_is_already_at_the_destination") {
        return;
    }
    let node = Node::new_with_own_key();
    node.add("a");
    vault::back_up_register(&node.paths, &node.estate).expect("backup");
    let dest = tempfile::tempdir().expect("dest");
    std::fs::create_dir_all(dest.path().join("keys")).expect("stray keys");

    let err = node::push(&node.paths, dest.path()).expect_err("refused");
    let text = format!("{err:#}");
    assert!(text.contains("must never be uploaded"), "{text}");
    assert!(text.contains("ADR-0069"), "{text}");
    assert!(text.contains("every retained version"), "{text}");
}

#[test]
fn push_refuses_an_unclassified_path() {
    if skip("push_refuses_an_unclassified_path") {
        return;
    }
    let node = Node::new_with_own_key();
    node.add("a");
    vault::back_up_register(&node.paths, &node.estate).expect("backup");
    std::fs::create_dir_all(node.paths.store.join("invoices")).expect("stray");
    std::fs::write(node.paths.store.join("invoices/a.pdf"), b"x").expect("stray file");

    let err = node::push(&node.paths, tempfile::tempdir().expect("d").path()).expect_err("refused");
    assert!(format!("{err:#}").contains("nobody has said whether invoices"));
}

#[test]
fn push_refuses_while_the_register_has_no_bundle() {
    if skip("push_refuses_while_the_register_has_no_bundle") {
        return;
    }
    // Uploading here would publish a half archive and report success.
    let node = Node::new_with_own_key();
    node.add("a");
    std::fs::create_dir_all(node.paths.store.join("vault")).expect("vault");
    let err = node::push(&node.paths, tempfile::tempdir().expect("d").path()).expect_err("refused");
    assert!(
        format!("{err:#}").contains("would be left behind"),
        "{err:#}"
    );
}

#[test]
fn push_refuses_plaintext_and_uploads_nothing() {
    if skip("push_refuses_plaintext_and_uploads_nothing") {
        return;
    }
    let node = Node::new_with_own_key();
    node.add("a");
    vault::back_up_register(&node.paths, &node.estate).expect("backup");
    std::fs::write(node.paths.store.join("vault/oops.jsonl"), b"leak").expect("plaintext");
    let dest = tempfile::tempdir().expect("dest");

    let err = node::push(&node.paths, dest.path()).expect_err("refused");
    assert!(format!("{err:#}").contains("ADR-0071 §1"), "{err:#}");
    // "Nothing was uploaded" must be true, not merely claimed.
    assert!(
        std::fs::read_dir(dest.path())
            .expect("dest readable")
            .next()
            .is_none(),
        "the destination was written to despite the refusal"
    );
}
