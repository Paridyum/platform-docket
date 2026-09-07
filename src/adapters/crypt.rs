//! Encrypting and decrypting with `age`, by invoking it.

use anyhow::{Context, Result, bail};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Invoked rather than linked, and that is a deliberate trade.
///
/// A Rust `age` crate would avoid a process and a `PATH` dependency. `age` the BINARY is what
/// ADR-0057 adopted, what `tool-registry.yaml` pins at `tier: critical`, and what RUN-0004
/// documents an operator using by hand. Linking a second implementation would mean the estate
/// had two things that must agree about its most important key, and the one an operator reaches
/// for in an emergency is the binary.
const AGE: &str = "age";
const AGE_KEYGEN: &str = "age-keygen";

/// Whether `age` is usable here.
///
/// # Errors
/// Fails with what to install when either binary is absent.
pub fn detect() -> Result<()> {
    for bin in [AGE, AGE_KEYGEN] {
        Command::new(bin)
            .arg("--version")
            .output()
            .with_context(|| format!("{bin} is not on PATH; the vault needs it (ADR-0057)"))?;
    }
    Ok(())
}

/// The estate identity file, per RUN-0004.
///
/// `SOPS_AGE_KEY_FILE` first because that is what sops itself honours, so an operator who has
/// moved their key has moved it once. The default is RUN-0004's stated location.
///
/// # Errors
/// Fails when no identity is present, naming the runbook rather than the concept — somebody
/// hitting this is trying to do something else and does not want a lecture on key management.
pub fn estate_identity() -> Result<PathBuf> {
    if let Ok(explicit) = std::env::var("SOPS_AGE_KEY_FILE") {
        let path = PathBuf::from(explicit);
        if path.exists() {
            return Ok(path);
        }
        bail!(
            "SOPS_AGE_KEY_FILE points at {}, which does not exist",
            path.display()
        );
    }
    let home = std::env::var("HOME").context("HOME is unset; the estate identity has no home")?;
    let path = PathBuf::from(home).join(".config/sops/age/keys.txt");
    if path.exists() {
        return Ok(path);
    }
    bail!(
        "no age identity at {} and SOPS_AGE_KEY_FILE is unset. RUN-0004 has the procedure: one \
         key, in the password manager, at that path with mode 600.",
        path.display()
    )
}

/// The public recipient an identity file decrypts for.
///
/// # Errors
/// Fails if the identity cannot be read or holds no key.
pub fn recipient_of(identity: &Path) -> Result<String> {
    let out = Command::new(AGE_KEYGEN)
        .arg("-y")
        .arg(identity)
        .output()
        .with_context(|| format!("deriving a recipient from {}", identity.display()))?;
    if !out.status.success() {
        bail!(
            "age-keygen could not read {}: {}",
            identity.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    let recipient = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    if recipient.is_empty() {
        bail!("{} holds no age identity", identity.display());
    }
    Ok(recipient)
}

/// A fresh identity, as `age-keygen` writes it.
///
/// # Errors
/// Fails if `age-keygen` cannot be run.
pub fn generate_identity() -> Result<String> {
    let out = Command::new(AGE_KEYGEN)
        .output()
        .context("generating an age identity")?;
    if !out.status.success() {
        bail!(
            "age-keygen failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Encrypt to exactly one recipient.
///
/// ONE RECIPIENT, and the single-ness is the security property rather than an implementation
/// detail. ADR-0069 rejected encrypting to a person's key AND an estate key: a second recipient
/// that can always decrypt makes the data uneraseable, so destroying the personal key would
/// shred nothing. The signature takes one recipient so that adding a second is a change to this
/// function, where the argument against it is written down.
///
/// # Errors
/// Fails if `age` cannot be run or refuses the recipient.
pub fn encrypt(recipient: &str, plaintext: &[u8]) -> Result<Vec<u8>> {
    let mut child = Command::new(AGE)
        .args(["--encrypt", "--recipient", recipient])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("running age --encrypt")?;
    child
        .stdin
        .take()
        .context("age took no stdin")?
        .write_all(plaintext)
        .context("writing plaintext to age")?;
    let out = child.wait_with_output().context("waiting for age")?;
    if !out.status.success() {
        bail!(
            "age --encrypt failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(out.stdout)
}

/// Decrypt with an identity held in memory.
///
/// The identity is written to a temporary file because `age` takes `-i <path>` and not a
/// stream. Created with `tempfile`, which opens it 0600 and unlinks it on drop — the key is
/// never on disk under a predictable name, and never survives a crash.
///
/// # Errors
/// Fails if `age` cannot be run, or if the identity does not open the ciphertext.
pub fn decrypt(identity: &str, ciphertext: &[u8]) -> Result<Vec<u8>> {
    let mut key = tempfile::NamedTempFile::new().context("staging an identity for age")?;
    key.write_all(identity.as_bytes())
        .context("writing the identity")?;
    key.flush().context("flushing the identity")?;

    let mut child = Command::new(AGE)
        .arg("--decrypt")
        .arg("-i")
        .arg(key.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("running age --decrypt")?;
    child
        .stdin
        .take()
        .context("age took no stdin")?
        .write_all(ciphertext)
        .context("writing ciphertext to age")?;
    let out = child.wait_with_output().context("waiting for age")?;
    if !out.status.success() {
        bail!(
            "age --decrypt failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(out.stdout)
}

/// Decrypt with an identity in a file, for the estate key which never leaves its own file.
///
/// # Errors
/// As `decrypt`.
pub fn decrypt_with_file(identity: &Path, ciphertext: &[u8]) -> Result<Vec<u8>> {
    let mut child = Command::new(AGE)
        .arg("--decrypt")
        .arg("-i")
        .arg(identity)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("running age --decrypt")?;
    child
        .stdin
        .take()
        .context("age took no stdin")?
        .write_all(ciphertext)
        .context("writing ciphertext to age")?;
    let out = child.wait_with_output().context("waiting for age")?;
    if !out.status.success() {
        bail!(
            "age --decrypt failed with {}: {}",
            identity.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(out.stdout)
}
