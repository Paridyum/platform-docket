//! Reading the registry files that hold Paridyum's facts.

use crate::config::Paths;
use crate::domain::lifecycle::Lifecycle;
use crate::domain::personnel::{Catalogue, ClassFile, FieldFile};
use anyhow::{Context, Result};
use serde::Deserialize;
use serde_yaml::Value;
use std::collections::BTreeMap;
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
    /// ISO date the officer ceased to hold this office, where they have.
    ///
    /// RFC-0042 §5: **ceasing is `active: false` plus this date, never deletion.**
    /// Documents already issued name their signatory, and a register that cannot
    /// say who was an officer in 2026 cannot verify a document issued in 2026.
    /// The RFC asked for this field and the struct did not have it, so half of
    /// its rule was unimplementable until 2026-09-12.
    #[serde(default)]
    pub ceased_on: Option<String>,
    /// Which company this office is held at. Defaults to `PAR` (ADR-0077).
    ///
    /// Defaulted rather than required so that an entry written before the
    /// second company existed still parses and still means what it meant.
    #[serde(default)]
    pub company: Option<String>,
    /// Whether this person may currently sign.
    ///
    /// CEASING IS `false`, NEVER DELETION (RFC-0042 decision 5). Documents already
    /// issued name their signatory, and a register that cannot say who was an officer
    /// in 2026 cannot verify a document issued in 2026.
    #[serde(default = "yes")]
    pub active: bool,
    /// The date the appointment took effect.
    #[serde(default)]
    pub appointed_on: Option<String>,
    /// The instrument the appointment rests on -- a resolution or written consent.
    ///
    /// Adding a name to this file decides who may bind Paridyum, which is
    /// governance-altering under C-0004 and needs a human and a dated instrument. The
    /// field exists so the evidence travels with the entry rather than in someone's
    /// memory; this file has recorded a name in error once already.
    #[serde(default)]
    pub instrument: Option<String>,
    /// Freeform note. Where an entry needs explaining, it explains itself.
    #[serde(default)]
    pub note: Option<String>,
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
    /// The Zitadel control-plane subject (opaque, permanent user id) this operator authenticates as
    /// through the served console (ADR-0092). The binding is on the subject, not the mutable
    /// loginname, so a rename cannot reattribute the ledger. Absent for a CLI-only operator, who is
    /// attributed by `--by` over a trusted connection but cannot sign in over OIDC until bound.
    #[serde(default)]
    pub subject: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Operators {
    operators: Vec<Operator>,
}

/// What Paridyum publishes about one officer.
///
/// THE PUBLIC HALF, AND ONLY THAT. A name and a designation are not restated here --
/// they live in `signatories.yaml`, keyed by `id`, so the office someone holds cannot
/// differ between the document they sign and the page that names them.
///
/// Nothing here derives from `people/`. That directory holds personal data under a
/// stated purpose per field and is gitignored so DPDP erasure stays honourable; what is
/// published is a decision taken in a reviewed diff, not a projection of the record.
#[derive(Debug, Clone, Deserialize)]
pub struct TeamMember {
    /// Signatory id this entry describes.
    pub id: String,
    /// Whether it may appear on the public site.
    ///
    /// Withholding somebody from the page keeps their entry, so removing a person from
    /// the website is not the same act as removing them from the register.
    #[serde(default)]
    pub public: bool,
    /// Display order on the page. Lower first.
    #[serde(default)]
    pub order: u32,
    /// One paragraph per locale. Locale-keyed rather than a single string, because the
    /// public surface is bilingual and a missing translation renders as `undefined`.
    #[serde(default)]
    pub summary: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct Team {
    team: Vec<TeamMember>,
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
    /// What may be published about each officer.
    pub team: Vec<TeamMember>,
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
        let team: Team = read_yaml(&paths.registry("team"))?;
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
            team: team.team,
        })
    }

    /// The `org` mapping, which every template's letterhead reads.
    #[must_use]
    pub fn org_node(&self) -> Option<&Value> {
        self.company(None)
    }

    /// The facts of one company, or of the default company when `code` is `None`.
    ///
    /// ADR-0077 decision 1: a company is a registry entry. Every document resolves
    /// its letterhead, jurisdiction, POSH routing and serial format through here,
    /// so a document issued for Paridyum Cyber cannot accidentally carry Paridyum's
    /// footer -- there is no second copy of those facts to fall out of step.
    #[must_use]
    pub fn company(&self, code: Option<&str>) -> Option<&Value> {
        let companies = self.org.get("companies")?.as_sequence()?;
        match code {
            Some(wanted) => companies
                .iter()
                .find(|c| c.get("code").and_then(Value::as_str) == Some(wanted)),
            // `default: true`, falling back to the first entry. A registry with no
            // default is a misconfiguration, not a reason to issue nothing.
            None => companies
                .iter()
                .find(|c| c.get("default").and_then(Value::as_bool) == Some(true))
                .or_else(|| companies.first()),
        }
    }

    /// The code of the default company.
    #[must_use]
    pub fn default_company(&self) -> Option<&str> {
        self.company(None)?.get("code")?.as_str()
    }

    /// Every declared company code, in registry order.
    #[must_use]
    pub fn company_codes(&self) -> Vec<&str> {
        self.org
            .get("companies")
            .and_then(Value::as_sequence)
            .map(|cs| {
                cs.iter()
                    .filter_map(|c| c.get("code").and_then(Value::as_str))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Signatories of one company, in registry order.
    ///
    /// Order is the fallback order (ADR-0077 decision 4). `issue` walks it and
    /// takes the first who is not the subject of the document, which is what
    /// makes Priyanshu holding both CEO and Director a data question rather than
    /// a special case in code.
    #[must_use]
    pub fn signatories_of(&self, company: &str) -> Vec<&Signatory> {
        self.signatories
            .iter()
            .filter(|s| s.company.as_deref().unwrap_or("PAR") == company)
            .collect()
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

    /// Resolve the operator a Zitadel token's `sub` is bound to (ADR-0092). Active operators only,
    /// matched on the opaque subject. An unbound or inactive subject is refused rather than
    /// defaulted — a token that names nobody in the registry may not write.
    ///
    /// # Errors
    /// Fails when no active operator is bound to the subject.
    pub fn by_subject(&self, subject: &str) -> Result<&Operator> {
        self.operators
            .iter()
            .find(|o| o.active && o.subject.as_deref() == Some(subject))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "no active operator is bound to subject {subject:?}; bind one with a \
                     `subject:` in registry/operators.yaml (ADR-0092)"
                )
            })
    }

    /// Signatories who may currently sign.
    pub fn active_signatories(&self) -> impl Iterator<Item = &Signatory> {
        self.signatories.iter().filter(|s| s.active)
    }

    /// Active signatories of one company, in registry order.
    ///
    /// ADR-0077 decision 4: a document is executed by a signatory of the company
    /// the ENGAGEMENT belongs to. Without this filter a Paridyum Cyber document
    /// could be countersigned by a Paridyum officer who holds no office there,
    /// which is a signature nobody can account for.
    ///
    /// Registry order is the fallback order. `issue` walks it and takes the first
    /// who is not the document's subject, which is what makes one person holding
    /// both CEO and Director a data question rather than a special case in code.
    pub fn active_signatories_of<'a>(
        &'a self,
        company: &'a str,
    ) -> impl Iterator<Item = &'a Signatory> {
        self.active_signatories()
            .filter(move |s| s.company.as_deref().unwrap_or("PAR") == company)
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
