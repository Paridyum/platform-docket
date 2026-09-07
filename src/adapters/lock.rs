//! An exclusive lock on the store, so that exactly one process writes it.
//!
//! ADR-0072 decision 2. The invariant is not new: the loopback server's accept
//! loop is serial precisely because "this process is the only writer to four
//! append-only hash chains, and that is what makes them safe without file
//! locking". That reasoning was sound and covered only the server. A CLI write
//! landing while the server holds the store is the same failure through a
//! different door, and it reproduces on the first attempt -- ten concurrent
//! `docket new` runs duplicated the person identifier in five of five pairs.

use crate::config::Paths;
use anyhow::{Context, Result, bail};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// Where the lock lives. Inside the store, because the store is what it guards:
/// two roots pointing at one store must contend, and two stores must not.
const LOCK_FILE: &str = ".writing.lock";

/// Held for as long as a writing command runs; releases on drop.
///
/// Drop rather than an explicit `release`, because every writing command has
/// error paths that return early and one of them would eventually forget.
#[derive(Debug)]
pub struct Guard {
    path: PathBuf,
}

impl Drop for Guard {
    fn drop(&mut self) {
        // A failure to remove leaves a stale lock, which is recoverable and
        // reported by the next writer. Panicking here would turn a cleanup
        // problem into a lost command, and `panic` is denied besides.
        let _ = fs::remove_file(&self.path);
    }
}

/// Take the store lock, or fail naming who holds it.
///
/// `create_new` is `O_EXCL`: the check and the creation are one atomic
/// operation, so two processes cannot both believe they succeeded.
///
/// **Why not `flock`.** STD-0003 sets `unsafe_code = "forbid"` and this crate
/// has no libc dependency. Atomic exclusive create is in `std`, needs neither,
/// and behaves identically on macOS and Linux. The cost is that it does not
/// survive a kill cleanly, which is why the error below is written to be acted
/// on rather than merely obeyed.
///
/// # Errors
/// Fails if the store cannot be created, or if another process holds the lock.
pub fn acquire(paths: &Paths, operator: Option<&str>) -> Result<Guard> {
    let path = paths.store.join(LOCK_FILE);
    fs::create_dir_all(&paths.store)
        .with_context(|| format!("creating {}", paths.store.display()))?;

    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(mut file) => {
            use std::io::Write as _;
            // Written after the lock is won, not before: the holder record is
            // for a human reading a stale lock, and a lock that recorded its
            // holder first would have to be created non-exclusively.
            let _ = write!(file, "{}", describe(operator));
            Ok(Guard { path })
        }
        Err(e) if e.kind() == ErrorKind::AlreadyExists => bail!(contention(&path)),
        Err(e) => Err(e).with_context(|| format!("creating {}", path.display())),
    }
}

/// The holder record: three lines, plain text, meant to be read with `cat`.
fn describe(operator: Option<&str>) -> String {
    format!(
        "host  {}\npid   {}\nby    {}\n",
        hostname(),
        std::process::id(),
        operator.unwrap_or("unknown"),
    )
}

/// The refusal, written so that the reader can tell a live holder from a corpse.
///
/// `another docket is writing` is an error someone escapes by deleting the file.
/// Naming the holder and the path is what makes the deletion a decision.
fn contention(path: &Path) -> String {
    let held = fs::read_to_string(path).unwrap_or_default();
    let who = held.trim();
    let detail = if who.is_empty() {
        "  (the lock file is empty; it was probably left by a killed process)".to_owned()
    } else {
        who.lines()
            .map(|l| format!("  {l}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    format!(
        "another docket holds the store, so this write is refused.\n{detail}\n\n\
         ADR-0072: exactly one process writes the store. Two writers duplicate person \
         identifiers, which locked ADR-0051 forbids.\n\n\
         If that process is still running, wait for it -- or, if it is the server, use it \
         instead of the CLI.\n\
         If it is gone, the lock is stale and clearing it is safe:\n  rm {}",
        path.display()
    )
}

/// The machine's name, for the holder record.
///
/// Best effort by design. `HOSTNAME` is exported by some shells and not others,
/// and the estate has no node registry yet (ADR-0072 decision 4). A missing
/// hostname degrades the message; it must never fail the lock.
fn hostname() -> String {
    if let Ok(h) = std::env::var("HOSTNAME")
        && !h.trim().is_empty()
    {
        return h.trim().to_owned();
    }
    std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown-host".to_owned())
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic)]
mod tests {
    use super::{LOCK_FILE, acquire};
    use crate::config::Paths;

    fn paths(dir: &std::path::Path) -> Paths {
        Paths::resolve(dir)
    }

    #[test]
    fn the_first_writer_takes_the_lock() {
        let dir = tempfile::tempdir().expect("tempdir");
        let p = paths(dir.path());
        let _guard = acquire(&p, Some("rohan")).expect("first acquire");
        assert!(p.store.join(LOCK_FILE).exists());
    }

    #[test]
    fn the_second_writer_is_refused() {
        let dir = tempfile::tempdir().expect("tempdir");
        let p = paths(dir.path());
        let _guard = acquire(&p, Some("rohan")).expect("first acquire");
        let err = acquire(&p, Some("aditya")).expect_err("second must be refused");
        let text = format!("{err:#}");
        assert!(text.contains("another docket holds the store"), "{text}");
    }

    #[test]
    fn the_refusal_names_the_holder_and_the_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        let p = paths(dir.path());
        let _guard = acquire(&p, Some("rohan")).expect("first acquire");
        let text = format!("{:#}", acquire(&p, None).expect_err("refused"));
        assert!(text.contains("by    rohan"), "holder not named: {text}");
        assert!(text.contains(LOCK_FILE), "path not named: {text}");
        assert!(text.contains("ADR-0051"), "reason not given: {text}");
    }

    #[test]
    fn dropping_the_guard_releases_the_lock() {
        let dir = tempfile::tempdir().expect("tempdir");
        let p = paths(dir.path());
        {
            let _guard = acquire(&p, Some("rohan")).expect("first acquire");
        }
        assert!(!p.store.join(LOCK_FILE).exists(), "lock outlived its guard");
        acquire(&p, Some("aditya")).expect("a released lock is retakeable");
    }

    #[test]
    fn an_empty_stale_lock_still_produces_an_actionable_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let p = paths(dir.path());
        std::fs::create_dir_all(&p.store).expect("store");
        std::fs::write(p.store.join(LOCK_FILE), "").expect("stale lock");
        let text = format!("{:#}", acquire(&p, Some("rohan")).expect_err("refused"));
        assert!(text.contains("killed process"), "{text}");
        assert!(text.contains("rm "), "no remedy offered: {text}");
    }
}
