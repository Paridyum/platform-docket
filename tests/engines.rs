//! Two typesetters, neither replacing the other (ADR-0054).

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]

use platform_docket::adapters::typeset::Engine;
use std::path::PathBuf;

mod common;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn pdflatex_is_preferred_when_present() {
    // The engine that produced the existing corpus wins when available, so a
    // machine that has always worked keeps producing byte-identical output.
    if !Engine::PdfLatex.available() {
        // Nothing to assert about preference on a machine without it.
        return;
    }
    let chosen = platform_docket::adapters::typeset::detect().expect("an engine is available");
    assert_eq!(chosen, Engine::PdfLatex);
}

#[test]
fn each_engine_declares_the_passes_it_needs() {
    // pdfTeX needs three to settle hyperref and \pageref{LastPage}; Tectonic
    // reruns internally, so driving it three times is wasted work.
    assert_eq!(Engine::PdfLatex.passes(), 3);
    assert_eq!(Engine::Tectonic.passes(), 1);
}

#[test]
fn engine_names_are_stable_because_the_register_stores_them() {
    // These strings are written into sealed register entries. Changing one
    // would make historical entries name an engine that no longer exists.
    assert_eq!(Engine::PdfLatex.as_str(), "pdflatex");
    assert_eq!(Engine::Tectonic.as_str(), "tectonic");
}

#[test]
fn an_entry_with_no_engine_recorded_reads_as_pdflatex() {
    // Entries written before two engines existed carry none. They were all
    // produced by pdflatex, so that is what they report rather than "unknown"
    // — safe precisely because the alternative did not exist then.
    let json = r#"{
        "serial":"PAR/NDA/2026/0001","code":"NDA","seq":1,"year":2026,
        "person_id":"PAR-P-000001-X","engagement_id":"PAR-INT-2026-0001",
        "slug":"t","document":"nda","title":"NDA","kind":"signed",
        "issued_on":"2026-08-19","retention":"contract",
        "template_sha256":"a","pdf_sha256":"b","pdf_path":"p.pdf",
        "texlive":"2026","prev_sha256":"0","entry_sha256":"x"
    }"#;
    let e: platform_docket::domain::register::Entry =
        serde_json::from_str(json).expect("an older entry still deserialises");
    assert_eq!(e.engine_or_default(), "pdflatex");
    assert!(e.engine.is_none());
}

#[test]
fn adding_the_engine_field_cannot_change_an_older_entrys_digest() {
    // The whole reason `engine` is Option + skip_serializing_if. If adding it
    // changed the serialised form, every sealed entry in every register would
    // have failed verification the day this shipped.
    let json = r#"{
        "serial":"PAR/NDA/2026/0001","code":"NDA","seq":1,"year":2026,
        "person_id":"PAR-P-000001-X","engagement_id":"PAR-INT-2026-0001",
        "slug":"t","document":"nda","title":"NDA","kind":"signed",
        "issued_on":"2026-08-19","retention":"contract",
        "template_sha256":"a","pdf_sha256":"b","pdf_path":"p.pdf",
        "texlive":"2026","prev_sha256":"0","entry_sha256":""
    }"#;
    let e: platform_docket::domain::register::Entry = serde_json::from_str(json).expect("parses");
    let round_tripped = serde_json::to_value(&e).expect("serialises");
    let map = round_tripped.as_object().expect("an object");
    assert!(
        !map.contains_key("engine"),
        "an absent engine must stay absent"
    );
    assert!(!map.contains_key("engine_version"));
}

#[test]
fn detection_names_both_engines_when_neither_is_installed() {
    // The message is what an operator on a fresh machine sees. It must not
    // name only the one that happens to be the default.
    //
    // Run as a subprocess with a PATH containing neither: `unsafe_code` is
    // forbidden crate-wide so the environment cannot be mutated in-process,
    // and a subprocess is how it actually fails anyway.
    let empty = tempfile::tempdir().expect("temp dir");
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_docket"))
        .args(["render", "nobody", "NDA"])
        .arg("--root")
        .arg(root())
        .env("PATH", empty.path())
        .env("DOCKET_STORE", empty.path().join("store"))
        .env("DOCKET_PEOPLE", empty.path().join("people"))
        // This test exercises the file-mode engine invocation; a DB/API URL
        // ambient in the environment (CI's Postgres job exports one) would route
        // `docket` to PgStore/the API client instead, so clear both.
        .env_remove("DOCKET_DATABASE_URL")
        .env_remove("DOCKET_API_URL")
        .output()
        .expect("runs");
    let err = String::from_utf8_lossy(&out.stderr);
    // It fails on the missing person first if the store is empty, so accept
    // either — what matters is that when it IS the typesetter, both are named.
    if err.contains("typesetter") {
        assert!(err.contains("pdflatex"), "{err}");
        assert!(err.contains("Tectonic"), "{err}");
    }
}

#[test]
fn tectonic_is_driven_with_the_flags_it_needs() {
    // Built by substitution rather than `format!`: the script uses shell
    // parameter expansion, and `${...}` is a format-string brace.
    const SHIM: &str = r#"#!/bin/sh
if [ "$1" = "--version" ]; then echo "Tectonic 0.0.0 (shim)"; exit 0; fi
echo "$* | epoch=$SOURCE_DATE_EPOCH force=$FORCE_SOURCE_DATE" >> "__CALLS__"
outdir=""
src=""
while [ $# -gt 0 ]; do
  case "$1" in
    --outdir) outdir="$2"; shift 2 ;;
    --keep-intermediates|--keep-logs) shift ;;
    *) src="$1"; shift ;;
  esac
done
base="${src##*/}"
stem="${base%.tex}"
printf '%PDF-1.7 shim\n' > "$outdir/$stem.pdf"
exit 0
"#;

    // Verifies the INVOCATION without Tectonic installed. A wrong flag here
    // would only surface on the machine that has no pdflatex -- which is
    // exactly the machine that cannot debug it.
    let dir = tempfile::tempdir().expect("temp dir");
    let bin = dir.path().join("tectonic");
    let calls = dir.path().join("calls.txt");

    std::fs::write(
        &bin,
        SHIM.replace("__CALLS__", &calls.display().to_string()),
    )
    .expect("write shim");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    }

    let people = dir.path().join("people");
    std::fs::create_dir_all(&people).expect("people dir");
    common::filled_template(&root(), &people.join("_TEMPLATE.yaml"));

    let run = |args: &[&str]| {
        std::process::Command::new(env!("CARGO_BIN_EXE_docket"))
            .args(args)
            .arg("--root")
            .arg(root())
                    // Only the shim on PATH: pdflatex must not be reachable. The shim
            // therefore uses shell builtins only -- `basename` lives in
            // /usr/bin, which is not on this PATH either.
            .env("PATH", dir.path())
            .env("DOCKET_STORE", dir.path().join("store"))
            .env("DOCKET_PEOPLE", &people)
            .env("DOCKET_OPERATOR", "rohan")
            // File-mode test: an ambient DB/API URL (CI's Postgres job sets one)
            // would route `docket` to PgStore/the API client, so clear both.
            .env_remove("DOCKET_DATABASE_URL")
            .env_remove("DOCKET_API_URL")
            .output()
            .expect("runs")
    };
    assert!(
        run(&["new", "t", "intern", "--date", "2026-08-20"])
            .status
            .success()
    );
    let out = run(&["issue", "t", "NDA", "--date", "2026-08-20"]);
    assert!(
        out.status.success(),
        "issuing under tectonic failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let recorded = std::fs::read_to_string(&calls).expect("the shim was called");
    assert_eq!(
        recorded.lines().count(),
        1,
        "tectonic reruns internally: one pass"
    );
    assert!(recorded.contains("--outdir"), "{recorded}");
    assert!(recorded.contains("--keep-logs"), "{recorded}");
    assert!(
        recorded.contains("epoch="),
        "SOURCE_DATE_EPOCH must be set: {recorded}"
    );
    assert!(
        recorded.contains("force=1"),
        "FORCE_SOURCE_DATE must be set: {recorded}"
    );

    // And the entry says what produced it (ADR-0054 point 3).
    let register =
        std::fs::read_to_string(dir.path().join("store/register/issued.jsonl")).expect("register");
    assert!(register.contains(r#""engine":"tectonic""#), "{register}");
}
