//! The encrypted archive: one key per person, so erasure has a mechanism.

use crate::adapters::store::Store;
use crate::adapters::{clock, crypt};
use crate::config::Paths;
use crate::domain::identity::PersonId;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

/// Where a person's ciphertext lives. Named by the opaque identifier (ADR-0051), because this
/// directory becomes a cloud folder and a listing named after people discloses who Paridyum
/// engages to anyone who can read it.
fn vault_dir(paths: &Paths, person: &PersonId) -> PathBuf {
    paths.store.join("vault").join(person.as_str())
}

/// Where a person's key lives.
///
/// **NEVER SYNCED AND NEVER IN GIT.** Both retain what you delete from them, so a key inside
/// either cannot be destroyed — and destroying it is the entire erasure mechanism (ADR-0069 D3).
/// `store/` is gitignored, and only `store/vault/` is uploaded.
fn key_path(paths: &Paths, person: &PersonId) -> PathBuf {
    paths
        .store
        .join("keys")
        .join(format!("{}.age", person.as_str()))
}

/// What a shredding leaves behind.
///
/// NOT PERSONAL DATA, deliberately: an identifier, a date, an operator and counts. No name, no
/// address, no document titles beyond their codes. A tombstone is how the archive says "erased
/// lawfully on this date" rather than "appears to be missing" — a distinction that matters most
/// to whoever reads it when nobody involved is still here.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tombstone {
    /// Whose vault was destroyed. The permanent identifier survives; the data does not.
    pub person_id: String,
    /// ISO date of the shredding.
    pub shredded_on: String,
    /// Operator who did it. Every write records who made it.
    pub by: String,
    /// How many encrypted objects were destroyed.
    pub objects: usize,
    /// Whether the key itself was found and destroyed. `false` means the vault was already
    /// unreadable — worth recording, because it is a different fact from having shredded it.
    pub key_destroyed: bool,
}

/// Read the tombstones. Append-only, and part of the register rather than the vault: it
/// survives precisely because it is not personal data.
///
/// # Errors
/// Fails if the file exists and cannot be parsed.
pub fn tombstones(paths: &Paths) -> Result<Vec<Tombstone>> {
    let path = tombstone_path(paths);
    let Ok(body) = fs::read_to_string(&path) else {
        return Ok(Vec::new());
    };
    body.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).with_context(|| format!("parsing {}", path.display())))
        .collect()
}

fn tombstone_path(paths: &Paths) -> PathBuf {
    paths.store.join("register").join("tombstones.jsonl")
}

/// Load a person's key, creating one if they have none.
///
/// The key is stored `age`-encrypted to the estate recipient. That is protection at rest and
/// **not** a second way into the data: the estate key opens the key FILE, and shredding deletes
/// that file. Encrypting the documents themselves to both keys was rejected in ADR-0069 as
/// escrow in disguise.
///
/// The estate identity is PASSED IN rather than read from the environment here. It is resolved
/// once, by the caller that knows it needs one, which keeps this testable without a test having
/// to mutate the process environment — and makes the dependency visible in the signature.
///
/// # Errors
/// Fails if `age` is unavailable or the key cannot be written.
fn person_identity(paths: &Paths, person: &PersonId, estate: &Path) -> Result<String> {
    let path = key_path(paths, person);

    if path.exists() {
        let sealed = fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
        let plain = crypt::decrypt_with_file(estate, &sealed)?;
        return String::from_utf8(plain).context("the stored key is not text");
    }

    let identity = crypt::generate_identity()?;
    let recipient = crypt::recipient_of(estate)?;
    let sealed = crypt::encrypt(&recipient, identity.as_bytes())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    fs::write(&path, sealed).with_context(|| format!("writing {}", path.display()))?;
    Ok(identity)
}

/// The recipient line inside an identity file.
fn recipient_in(identity: &str) -> Result<String> {
    identity
        .lines()
        .find_map(|l| l.strip_prefix("# public key: "))
        .map(str::to_owned)
        .context("the identity carries no public key line")
}

/// What a seal did.
#[derive(Debug, Default)]
pub struct Sealed {
    /// Objects written this run.
    pub written: Vec<String>,
    /// Objects already present and left alone.
    pub skipped: usize,
}

/// Encrypt everything issued to a person into their vault.
///
/// IDEMPOTENT: an object already sealed is left alone rather than re-encrypted. `age` output is
/// not deterministic — it carries a fresh ephemeral key each time — so re-encrypting would
/// rewrite every file on every run and make a sync diff meaningless.
///
/// # Errors
/// Fails if the register cannot be read, `age` is unavailable, or a document is missing.
pub fn seal(
    store: &dyn Store,
    paths: &Paths,
    slug: &str,
    company: Option<&str>,
    estate: &Path,
) -> Result<Sealed> {
    crypt::detect()?;
    let who = crate::application::personnel::identify_at(store, slug, company)?;
    let identity = person_identity(paths, &who.person_id, estate)?;
    let recipient = recipient_in(&identity)?;
    let dir = vault_dir(paths, &who.person_id);
    fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;

    let mut out = Sealed::default();
    for entry in store.register()? {
        if entry.person_id != who.person_id {
            continue;
        }
        let source = paths.store.join(&entry.pdf_path);
        if !source.exists() {
            bail!(
                "{} is registered but its file is missing at {}",
                entry.serial,
                source.display()
            );
        }
        seal_one(&source, &dir, &recipient, &mut out)?;
    }

    // THE SIGNED SCANS TOO, and they are the ones that matter most. `sign.rs` says it: "wet ink
    // leaves no trail of its own, so the scan IS the evidence." An issued PDF was handed to the
    // person in their pack, so losing Paridyum's copy is recoverable from them. A returned scan
    // is the opposite — they signed it and gave it back, and Paridyum holds the only copy.
    //
    // They were not sealed at all until this: not in the vault, not in the register bundle, and
    // therefore in nothing that ever left the laptop.
    let scans = paths.scans_dir(who.person_id.as_str());
    if let Ok(entries) = fs::read_dir(&scans) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                seal_one(&path, &dir, &recipient, &mut out)?;
            }
        }
    }
    Ok(out)
}

/// Encrypt one file into a person's vault, skipping it if already there.
fn seal_one(source: &Path, dir: &Path, recipient: &str, out: &mut Sealed) -> Result<()> {
    let Some(name) = source.file_name().and_then(|n| n.to_str()) else {
        return Ok(());
    };
    let target = dir.join(format!("{name}.age"));
    if target.exists() {
        out.skipped += 1;
        return Ok(());
    }
    let plain = fs::read(source).with_context(|| format!("reading {}", source.display()))?;
    let sealed = crypt::encrypt(recipient, &plain)?;
    fs::write(&target, sealed).with_context(|| format!("writing {}", target.display()))?;
    out.written.push(format!("{name}.age"));
    Ok(())
}

/// Decrypt one object out of a person's vault.
///
/// # Errors
/// Fails if the key is gone — which is what a shredded vault looks like, and the message says so
/// rather than reporting a decryption failure.
pub fn open(
    store: &dyn Store,
    paths: &Paths,
    slug: &str,
    company: Option<&str>,
    name: &str,
    estate: &Path,
) -> Result<Vec<u8>> {
    crypt::detect()?;
    let who = crate::application::personnel::identify_at(store, slug, company)?;
    if !key_path(paths, &who.person_id).exists() {
        bail!(
            "{slug} has no vault key. If they were erased this is expected and permanent: the \
             key was destroyed, which is what erasure means here. `docket verify` still works \
             against a copy they present."
        );
    }
    let identity = person_identity(paths, &who.person_id, estate)?;
    let object = vault_dir(paths, &who.person_id).join(name);
    let sealed = fs::read(&object).with_context(|| format!("reading {}", object.display()))?;
    crypt::decrypt(&identity, &sealed)
}

/// Destroy a person's key and vault, and record that it happened.
///
/// **THIS IS THE ERASURE MECHANISM.** Deleting files from a versioned cloud store is not
/// erasure, because the versions remain; destroying the only key that opens them is. The
/// register is untouched, so what was issued stays provable and a document the person still
/// holds still verifies.
///
/// # Errors
/// Fails if the tombstone cannot be written. **The tombstone is written before the key is
/// destroyed**, so a crash between them leaves a tombstone with a surviving key — recoverable
/// and visible — rather than a destroyed key nothing recorded.
pub fn shred(paths: &Paths, person: &PersonId, date: time::Date, by: &str) -> Result<Tombstone> {
    let dir = vault_dir(paths, person);
    let objects = fs::read_dir(&dir).map_or(0, |entries| entries.flatten().count());
    let key = key_path(paths, person);

    let stone = Tombstone {
        person_id: person.as_str().to_owned(),
        shredded_on: clock::to_iso(date)?,
        by: by.to_owned(),
        objects,
        key_destroyed: key.exists(),
    };
    append_tombstone(paths, &stone)?;

    if key.exists() {
        fs::remove_file(&key).with_context(|| format!("destroying {}", key.display()))?;
    }
    if dir.exists() {
        fs::remove_dir_all(&dir).with_context(|| format!("removing {}", dir.display()))?;
    }
    Ok(stone)
}

fn append_tombstone(paths: &Paths, stone: &Tombstone) -> Result<()> {
    let path = tombstone_path(paths);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    let mut body = fs::read_to_string(&path).unwrap_or_default();
    body.push_str(&serde_json::to_string(stone)?);
    body.push('\n');
    fs::write(&path, body).with_context(|| format!("appending to {}", path.display()))
}

/// The register bundle: the half that must never be lost.
///
/// **`seal` covers issued PDFs and nothing else, which left the more important half unprotected.**
/// The register is what makes a document verifiable in 2036 — it holds the serial, the hash and
/// the verification code — and the ledger, event log and tombstones are the rest of the account
/// of what happened. None of them were in the uploaded path.
///
/// Encrypted to the ESTATE key rather than a person's, because these are the non-erasable half
/// (ADR-0069 D1). That is also what makes them safe in a versioned store: there is no erasure
/// promise here to be broken by a retained old version.
///
/// Re-encrypted every run rather than skipped when present, unlike `seal`. The register changes
/// whenever anything is issued, so "already there" is not a reason to leave it — and it is one
/// small file, so rewriting it costs a sync diff of one object.
const REGISTER_BUNDLE: &str = "_register";

/// Files the bundle carries, in a fixed order so the archive is reproducible.
///
/// Named individually rather than "everything but the vault": a new directory under `store/`
/// should have to be considered before it is archived, not swept in by a glob that nobody
/// revisits. `keys/` is the one that must never be here, and a glob would have taken it.
const BUNDLED: &[&str] = &[
    "register/issued.jsonl",
    "register/people.jsonl",
    "register/signatures.jsonl",
    "register/events.jsonl",
    "register/tombstones.jsonl",
];

/// Encrypt the register and its companions into the uploaded path.
///
/// # Errors
/// Fails if `age` is unavailable or the estate identity is missing.
pub fn back_up_register(paths: &Paths, estate: &Path) -> Result<Vec<String>> {
    crypt::detect()?;
    let recipient = crypt::recipient_of(estate)?;
    let dir = paths.store.join("vault").join(REGISTER_BUNDLE);
    fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;

    let mut written = Vec::new();
    for name in BUNDLED {
        let source = paths.store.join(name);
        if !source.exists() {
            continue;
        }
        let plain = fs::read(&source).with_context(|| format!("reading {}", source.display()))?;
        let sealed = crypt::encrypt(&recipient, &plain)?;
        let flat = name.replace('/', "-");
        let target = dir.join(format!("{flat}.age"));
        fs::write(&target, sealed).with_context(|| format!("writing {}", target.display()))?;
        written.push(flat);
    }

    if written.is_empty() {
        bail!("nothing to back up: the store holds no register yet");
    }
    Ok(written)
}

/// Whether the register bundle is missing or older than the register it covers.
///
/// Infallible on purpose: this runs inside `docket check`, and a check that cannot answer
/// should say "no finding" rather than abort the run. An unreadable timestamp is treated as
/// not-stale for the same reason — reporting a problem you cannot actually see is worse than
/// staying quiet, because it trains an operator to disbelieve the warning.
#[must_use]
pub fn register_backup_stale(paths: &Paths) -> Option<String> {
    let register = paths.store.join("register/issued.jsonl");
    if !register.exists() {
        return None;
    }
    let bundle = paths
        .store
        .join("vault")
        .join(REGISTER_BUNDLE)
        .join("register-issued.jsonl.age");
    if !bundle.exists() {
        return Some(
            "the register has never been backed up; it is the half that makes every document \
             verifiable and it exists only here. Run `docket vault backup`."
                .to_owned(),
        );
    }
    // Modification time, not content: the bundle is ciphertext and cannot be compared to the
    // plaintext without decrypting it, and a check should not need the key to run.
    let newer = |a: &Path, b: &Path| -> bool {
        match (
            a.metadata().and_then(|m| m.modified()),
            b.metadata().and_then(|m| m.modified()),
        ) {
            (Ok(x), Ok(y)) => x > y,
            _ => false,
        }
    };
    if newer(&register, &bundle) {
        return Some(
            "the register has changed since it was last backed up. Run `docket vault backup`."
                .to_owned(),
        );
    }
    None
}

/// Export every per-person key, encrypted to the estate key, for offline media.
///
/// **THIS WEAKENS SHREDDING, AND THAT IS THE WHOLE POINT OF MAKING IT EXPLICIT.** Crypto-
/// shredding needs key material that can genuinely be destroyed. Any copy of that material
/// improves durability by exactly as much as it damages the erasure promise. There is no scheme
/// that gets both, and a design that claims to has quietly picked one.
///
/// So the trade is a command an operator runs deliberately, not a default:
///
/// - **Without an escrow**, erasure is provable and losing `store/keys/` loses every vault.
/// - **With one**, the vault survives a lost laptop and erasure becomes *asserted* — docket
///   destroys its own copy of the key and cannot reach the one on the shelf.
///
/// The same distinction `access.company_email` and `access.mailbox_email` already draw: severing
/// what you control is provable, and asking somebody to destroy what they hold is not.
///
/// **The destination is named by the operator and is never inside the store.** Writing it into
/// `store/vault/` would put every key into the versioned cloud folder, after which no erasure
/// this system performs is real.
///
/// # Errors
/// Fails if `age` is unavailable, the destination is inside the store, or nothing is written.
pub fn escrow(paths: &Paths, estate: &Path, to: &Path) -> Result<usize> {
    crypt::detect()?;
    if to.starts_with(&paths.store) {
        bail!(
            "{} is inside the store, and the store is what gets uploaded. An escrow there would \
             put every per-person key into a versioned cloud folder, which retains what you \
             delete from it — after which no erasure this system performs is real. Write it to \
             offline media you physically control.",
            to.display()
        );
    }

    let keys = paths.store.join("keys");
    let mut bundle = String::new();
    let mut count = 0;
    let mut names: Vec<_> = fs::read_dir(&keys)
        .map(|entries| entries.flatten().map(|e| e.path()).collect::<Vec<_>>())
        .unwrap_or_default();
    names.sort();

    for path in names {
        let Some(person) = path.file_stem().and_then(|n| n.to_str()) else {
            continue;
        };
        let sealed = fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
        let identity = crypt::decrypt_with_file(estate, &sealed)?;
        // One record per person, delimited so a partial read is detectable rather than being
        // silently interpreted as a shorter escrow.
        writeln!(
            bundle,
            "# person: {person}\n{}\n# end: {person}",
            String::from_utf8_lossy(&identity).trim()
        )?;
        count += 1;
    }

    if count == 0 {
        bail!("no per-person keys exist yet; there is nothing to escrow");
    }

    let recipient = crypt::recipient_of(estate)?;
    let out = crypt::encrypt(&recipient, bundle.as_bytes())?;
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    fs::write(to, out).with_context(|| format!("writing {}", to.display()))?;
    Ok(count)
}

/// Whether a shredding has happened since an escrow was taken.
///
/// **THE ONE CASE WORTH SHOUTING ABOUT.** An escrow written before a shredding still holds the
/// key docket destroyed, so the erasure is incomplete in a way nothing else reports — the
/// operator has to destroy that medium, or re-export and destroy the old one.
///
/// Takes the escrow's own modification time rather than reading it: the file is on offline media
/// by design and usually absent, and a check must not need it mounted to run.
#[must_use]
pub fn escrow_outstanding(paths: &Paths, escrow: Option<&Path>) -> Option<String> {
    let stones = tombstones(paths).ok()?;
    let last = stones.last()?;
    let Some(escrow) = escrow else {
        return Some(format!(
            "a shredding was recorded on {} — if an escrow copy of the keys exists on offline \
             media, it still holds the destroyed key and the erasure is not complete until that \
             medium is destroyed or re-exported.",
            last.shredded_on
        ));
    };
    let taken = escrow.metadata().and_then(|m| m.modified()).ok()?;
    let shredded = paths
        .store
        .join("register")
        .join("tombstones.jsonl")
        .metadata()
        .and_then(|m| m.modified())
        .ok()?;
    (shredded > taken).then(|| {
        format!(
            "{} was written before the shredding recorded on {}, so it still holds a key that \
             was destroyed here. Re-export it and destroy the old medium.",
            escrow.display(),
            last.shredded_on
        )
    })
}

/// What `sync` should upload, and what it must never upload.
///
/// Returned rather than performed: docket does not talk to Sync.com. The vendor's client is
/// Windows and Mac only (RFC-0038), the upload is an operation with a runbook, and a tool that
/// shelled out to it would be pretending to own something it does not.
#[must_use]
pub fn syncable(paths: &Paths) -> (PathBuf, Vec<PathBuf>) {
    (
        paths.store.join("vault"),
        vec![
            paths.store.join("keys"),
            paths.store.join("issued"),
            paths.store.join("scans"),
            paths.store.join("index"),
            paths.people.clone(),
        ],
    )
}

/// Whether anything issued is not yet sealed.
///
/// # Errors
/// Fails if the register cannot be read.
pub fn unsealed(store: &dyn Store, paths: &Paths) -> Result<Vec<String>> {
    let mut out = Vec::new();

    // SHREDDED IS NOT THE SAME AS NEVER SEALED, and keying on the key file cannot tell them
    // apart: neither has one. Reporting a shredded person's documents as unsealed would ask an
    // operator to re-seal what erasure destroyed; treating a never-sealed person as shredded
    // reports "everything is sealed" when nothing is. The tombstone is what distinguishes them,
    // and it is the record that erasure actually happened.
    let shredded: std::collections::BTreeSet<String> = tombstones(paths)?
        .into_iter()
        .map(|t| t.person_id)
        .collect();

    // Scans first, because they are the irreplaceable half: an issued PDF was handed over in a
    // pack, a returned scan exists nowhere else.
    let register = store.register()?;
    for sig in store.signatures()? {
        let Some(rel) = sig.scan_path.as_ref() else {
            continue;
        };
        let Some(name) = Path::new(rel).file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Some(person) = register
            .iter()
            .find(|e| e.serial == sig.serial)
            .map(|e| e.person_id.clone())
        else {
            continue;
        };
        if shredded.contains(person.as_str()) {
            continue;
        }
        if !vault_dir(paths, &person)
            .join(format!("{name}.age"))
            .exists()
        {
            out.push(format!("{} (signed scan)", sig.serial));
        }
    }

    for entry in register {
        let dir = vault_dir(paths, &entry.person_id);
        let Some(name) = Path::new(&entry.pdf_path)
            .file_name()
            .and_then(|n| n.to_str())
        else {
            continue;
        };
        if shredded.contains(entry.person_id.as_str()) {
            continue;
        }
        if !dir.join(format!("{name}.age")).exists() {
            out.push(entry.serial);
        }
    }
    Ok(out)
}
