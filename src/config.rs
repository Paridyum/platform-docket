//! Filesystem locations the tool reads and writes.

use std::path::{Path, PathBuf};

/// Where the repository, the person records and the register live.
///
/// The store is deliberately separable from the repository: personal data
/// must never enter git, because git history cannot be erased and DPDP
/// erasure rights would then be unhonourable.
#[derive(Debug, Clone)]
pub struct Paths {
    /// Repository root, holding `registry/`, `templates/` and `latex/`.
    pub root: PathBuf,
    /// Person records. Gitignored.
    pub people: PathBuf,
    /// Register and issued PDFs. Gitignored.
    pub store: PathBuf,
}

impl Paths {
    /// Resolve from a root, honouring `DOCKET_STORE` and `DOCKET_PEOPLE`.
    #[must_use]
    pub fn resolve(root: &Path) -> Self {
        let env_path =
            |key: &str, default: PathBuf| std::env::var_os(key).map_or(default, PathBuf::from);
        Self {
            root: root.to_path_buf(),
            people: env_path("DOCKET_PEOPLE", root.join("people")),
            store: env_path("DOCKET_STORE", root.join("store")),
        }
    }

    /// The append-only register file.
    #[must_use]
    pub fn register(&self) -> PathBuf {
        self.store.join("register").join("issued.jsonl")
    }

    /// Directory holding one person's issued PDFs.
    ///
    /// Named by the PERSON identifier (ADR-0051), never the slug: this
    /// directory is what gets mirrored to a cloud folder, and a folder named
    /// after a person discloses who Paridyum engages from the listing alone.
    #[must_use]
    pub fn issued_dir(&self, person_id: &str) -> PathBuf {
        self.store.join("issued").join(person_id)
    }

    /// The identity ledger: which identifiers have been issued.
    #[must_use]
    pub fn ledger(&self) -> PathBuf {
        self.store.join("register").join("people.jsonl")
    }

    /// The signature ledger: which documents came back executed.
    #[must_use]
    pub fn signatures(&self) -> PathBuf {
        self.store.join("register").join("signatures.jsonl")
    }

    /// Where a returned scan is filed for one person.
    #[must_use]
    pub fn scans_dir(&self, person_id: &str) -> PathBuf {
        self.store.join("scans").join(person_id)
    }

    /// The personnel event log.
    #[must_use]
    pub fn events(&self) -> PathBuf {
        self.store.join("register").join("events.jsonl")
    }

    /// A person's record.
    #[must_use]
    pub fn person(&self, slug: &str) -> PathBuf {
        self.people.join(format!("{slug}.yaml"))
    }

    /// A registry file by bare name, e.g. `org`.
    #[must_use]
    pub fn registry(&self, name: &str) -> PathBuf {
        self.root.join("registry").join(format!("{name}.yaml"))
    }
}
