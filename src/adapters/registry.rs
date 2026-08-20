//! Reading the registry files that hold Paridyum's facts.

use crate::config::Paths;
use crate::domain::lifecycle::Lifecycle;
use crate::domain::personnel::{Catalogue, ClassFile, FieldFile};
use anyhow::{Context, Result};
use serde::Deserialize;
use serde_yaml::Value;
use std::fs;
use std::path::Path;

/// A signatory who may sign on behalf of Paridyum.
#[derive(Debug, Clone, Deserialize)]
pub struct Signatory {
    /// Key used to name the generated commands, e.g. `ceo`.
    pub id: String,
    /// Name as it is printed on the document.
    pub legal_name: String,
    /// Title as it is printed beneath the name.
    pub designation: String,
    /// Short form, where a document has no room for the full designation.
    #[serde(default)]
    pub short: Option<String>,
    /// Whether this person may currently sign.
    #[serde(default = "yes")]
    pub active: bool,
}

const fn yes() -> bool {
    true
}

#[derive(Debug, Deserialize)]
struct Signatories {
    signatories: Vec<Signatory>,
}

/// Someone permitted to operate this tool.
///
/// Separate from a signatory: signing a document and running the tool that
/// issues it are different acts, done today by the same people and not
/// necessarily tomorrow.
#[derive(Debug, Clone, Deserialize)]
pub struct Operator {
    /// Short id passed to `--by`.
    pub id: String,
    /// Name recorded in the log.
    pub name: String,
    /// Their role, for the report.
    pub role: String,
    /// Whether they may currently operate.
    #[serde(default = "yes")]
    pub active: bool,
}

#[derive(Debug, Deserialize)]
struct Operators {
    operators: Vec<Operator>,
}

/// One retention class and the reason it is that long.
#[derive(Debug, Clone, Deserialize)]
pub struct RetentionClass {
    /// Name referenced by a document's `retention` field.
    pub class: String,
    /// Which date the period is measured from.
    #[serde(default)]
    pub from: Epoch,
    /// How long, in prose.
    pub keep_for: String,
    /// Why. A period without a stated basis is a guess that hardens into policy.
    pub basis: String,
}

/// The date a retention period is counted from.
///
/// Named explicitly because every period in `retention.yaml` is stated
/// relative to the END of the engagement, and measuring from issuance instead
/// is wrong by the whole length of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Epoch {
    /// The engagement's end date, or the separation date if earlier.
    #[default]
    EngagementEnd,
    /// The date the document was issued.
    Issue,
    /// No deadline.
    Never,
}

#[derive(Debug, Deserialize)]
struct Retention {
    classes: Vec<RetentionClass>,
}

/// Everything the registry declares.
#[derive(Debug)]
pub struct Registry {
    /// `org.yaml`, unmodelled: templates address fields the code has no rule about.
    pub org: Value,
    /// Who may sign.
    pub signatories: Vec<Signatory>,
    /// Documents and stages.
    pub lifecycle: Lifecycle,
    /// Retention classes.
    pub retention: Vec<RetentionClass>,
    /// Person classes and the field catalogue.
    pub catalogue: Catalogue,
    /// What Paridyum sends, when, and what goes with it.
    pub messages: Vec<crate::application::comms::Message>,
    /// Who may operate this tool.
    pub operators: Vec<Operator>,
}

impl Registry {
    /// Load every registry file.
    ///
    /// # Errors
    /// Fails if any registry file is missing or malformed.
    pub fn load(paths: &Paths) -> Result<Self> {
        let org: Value = read_yaml(&paths.registry("org"))?;
        let signatories: Signatories = read_yaml(&paths.registry("signatories"))?;
        let lifecycle: Lifecycle = read_yaml(&paths.registry("lifecycle"))?;
        let retention: Retention = read_yaml(&paths.registry("retention"))?;
        let classes: ClassFile = read_yaml(&paths.registry("person-classes"))?;
        let fields: FieldFile = read_yaml(&paths.registry("fields"))?;
        let messages: crate::application::comms::MessageFile =
            read_yaml(&paths.registry("communications"))?;
        let operators: Operators = read_yaml(&paths.registry("operators"))?;
        Ok(Self {
            org,
            signatories: signatories.signatories,
            lifecycle,
            retention: retention.classes,
            catalogue: Catalogue {
                classes: classes.classes,
                fields: fields.fields,
            },
            messages: messages.messages,
            operators: operators.operators,
        })
    }

    /// The `org` mapping, which every template's letterhead reads.
    #[must_use]
    pub fn org_node(&self) -> Option<&Value> {
        self.org.get("org")
    }

    /// The governing-law forum, or `None` when unset.
    ///
    /// Unset is not a formatting problem: it produces a jurisdiction clause
    /// naming no court, so issuance refuses on it.
    #[must_use]
    pub fn forum(&self) -> Option<&str> {
        self.org_node()?.get("jurisdiction")?.get("forum")?.as_str()
    }

    /// The serial format string.
    #[must_use]
    pub fn serial_format(&self) -> Option<&str> {
        self.org_node()?.get("reference")?.get("format")?.as_str()
    }

    /// Resolve an operator id, naming the valid ones when it is unknown.
    ///
    /// # Errors
    /// Fails when the id is unknown or the operator is not active. An
    /// unrecognised operator is refused rather than recorded as a free string:
    /// a log that accepts any name cannot be relied on to say who did anything.
    pub fn operator(&self, id: &str) -> Result<&Operator> {
        if let Some(o) = self.operators.iter().find(|o| o.id == id && o.active) {
            return Ok(o);
        }
        let known: Vec<&str> = self
            .operators
            .iter()
            .filter(|o| o.active)
            .map(|o| o.id.as_str())
            .collect();
        anyhow::bail!(
            "unknown operator {id:?}; declared: {}. Pass --by <id>, or set DOCKET_OPERATOR",
            known.join(" ")
        )
    }

    /// Signatories who may currently sign.
    pub fn active_signatories(&self) -> impl Iterator<Item = &Signatory> {
        self.signatories.iter().filter(|s| s.active)
    }
}

/// Parse a YAML file, naming the file when it fails.
///
/// # Errors
/// Fails if the file cannot be read or does not parse.
pub fn read_yaml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_yaml::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}
