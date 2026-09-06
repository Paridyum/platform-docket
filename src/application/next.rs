//! The single next thing to do for one person.

use crate::adapters::{registry::Registry, store};
use crate::config::Paths;
use crate::domain::identity::Standing;
use anyhow::Result;

/// One thing to do, and the command that does it.
#[derive(Debug)]
pub struct Step {
    /// What it is, in a sentence.
    pub what: String,
    /// Why it is next.
    pub why: String,
    /// The command to run, or `None` where a person must act outside the tool.
    pub command: Option<String>,
}

/// Everything outstanding for a person, most urgent first.
///
/// The ordering IS the workflow. Someone following it from the top never has
/// to know the lifecycle: each step becomes available only once the one above
/// it is done, which is the same ordering `docket advance` enforces and the
/// same one `HR-CHECKLIST.md` writes out longhand.
///
/// # Errors
/// Fails if the person is unknown or the store cannot be read.
pub fn steps(paths: &Paths, registry: &Registry, slug: &str) -> Result<Vec<Step>> {
    let who = crate::application::personnel::identify(paths, slug)?;
    let mut out = Vec::new();

    if who.standing == Standing::Erased {
        out.push(Step {
            what: "Nothing. This person's data was erased.".to_owned(),
            why: format!("erased on {}", who.standing_changed_on),
            command: None,
        });
        return Ok(out);
    }

    let person = store::person(paths, slug)?;
    let register = store::register(paths)?;

    // 1. A record that cannot lawfully be issued against blocks everything.
    let refusals = crate::application::issue::refusals(registry, &person);
    for r in &refusals {
        out.push(Step {
            what: format!("Fix the record: {r}"),
            why: "issuing is refused until this is corrected".to_owned(),
            command: Some(format!("$EDITOR {}", paths.person(slug).display())),
        });
    }
    if !refusals.is_empty() {
        return Ok(out);
    }

    workflow_steps(paths, registry, slug, &who, &person, &register, &mut out)?;
    if out.is_empty() {
        out.push(Step {
            what: "Nothing.".to_owned(),
            why: format!(
                "everything stage \"{}\" owes is issued, executed and sent",
                person.stage()
            ),
            command: None,
        });
    }
    Ok(out)
}

/// The steps that follow once the record itself is sound.
fn workflow_steps(
    paths: &Paths,
    registry: &Registry,
    slug: &str,
    who: &crate::domain::identity::Record,
    person: &crate::domain::person::Person,
    register: &[crate::domain::register::Entry],
    out: &mut Vec<Step>,
) -> Result<()> {
    // 2. Documents that cannot render because the record is missing a field
    //    they print. Before the issue step, because issuing would fail.
    let unrenderable =
        crate::application::issue::unrenderable(paths, registry, person, who, register)?;
    if !unrenderable.is_empty() {
        for (code, missing) in &unrenderable {
            // Already record paths: `issue::unsupplied_fields` maps the
            // commands back itself, so every caller gets the readable form and
            // not just this one (D-44).
            out.push(Step {
                what: format!(
                    "Fill in {} for {code}: {}",
                    missing.len(),
                    missing.join(", ")
                ),
                why: format!(
                    "{code} prints these and the record does not have them, so \
                     issuing it would fail"
                ),
                command: Some(format!("$EDITOR {}", paths.person(slug).display())),
            });
        }
        return Ok(());
    }

    // 3. Documents the stage owes and has not issued.
    let owed = crate::application::issue::outstanding(registry, person, who, register)?;
    if !owed.is_empty() {
        out.push(Step {
            what: format!("Issue {} document(s): {}", owed.len(), owed.join(" ")),
            why: format!("owed at stage \"{}\"", person.stage()),
            command: Some(format!("docket onboard {slug}")),
        });
        return Ok(());
    }

    // 4. A pack to hand over, if anything has been issued since the last one.
    let packed = paths.store.join("packs").join(who.person_id.as_str());
    if !packed.exists() {
        out.push(Step {
            what: "Build the folder to hand over".to_owned(),
            why: "everything owed is issued and no pack has been built".to_owned(),
            command: Some(format!("docket pack {slug}")),
        });
    }

    // 5. Documents issued and not returned executed.
    let awaiting = crate::application::sign::outstanding(paths, registry, &who.person_id)?;
    if !awaiting.is_empty() {
        let codes: Vec<&str> = awaiting.iter().map(|e| e.code.as_str()).collect();
        out.push(Step {
            what: format!(
                "Chase {} signed copies: {}",
                awaiting.len(),
                codes.join(" ")
            ),
            why: "issued, not yet returned executed".to_owned(),
            command: Some(format!(
                "docket sign {} --method wet-ink --on <date> --scan <file>",
                awaiting.first().map_or("<serial>", |e| e.serial.as_str())
            )),
        });
    }

    // 6. Advancing is suggested ONLY where the stage has a signature gate and
    //    that gate is now open.
    //
    //    An empty gate means "nothing blocks advancing", which is not the same
    //    as "you should advance". `active` declares no gate, and suggesting the
    //    move from it would tell you to offboard someone on their first day.
    //    Leaving a stage with no gate is a human decision about the engagement,
    //    not something derivable from the record.
    let current = registry.lifecycle.stage(person.stage());
    let gated = current.is_some_and(|s| !s.requires_signed.is_empty());
    if gated {
        let unmet = crate::application::personnel::unmet_gates(
            paths,
            registry,
            who,
            person.stage(),
            person.class(),
        )?;
        if unmet.is_empty() && registry.lifecycle.next_after(person.stage())?.is_some() {
            out.push(Step {
                what: "Move to the next stage".to_owned(),
                why: "every document this stage requires executed is executed".to_owned(),
                command: Some(format!("docket advance {slug}")),
            });
        }
    }

    // 7. Messages that carry documents now issued.
    if let Ok(messages) = crate::application::comms::due(paths, registry, slug) {
        let ready: Vec<&str> = messages
            .iter()
            .filter(|m| m.missing.is_empty())
            .map(|m| m.id.as_str())
            .collect();
        if !ready.is_empty() {
            // NOTHING RECORDS WHAT HAS ALREADY BEEN SENT, because this tool
            // does not send mail (D-27) and a "sent" flag it could not verify
            // would be worse than none. So this lists what is ready, not what
            // is outstanding, and says which it is.
            out.push(Step {
                what: format!("Ready to send: {}", ready.join(" ")),
                why: "every attachment these carry is issued. Nothing records \
                      what has already gone out, so check before resending"
                    .to_owned(),
                command: Some(format!("docket emails {slug}")),
            });
        }
    }

    // 8. Housekeeping, last, because it is never urgent and always outstanding.
    let indexed = paths.store.join("index").join("roster.csv");
    let latest_issue = register.iter().map(|e| e.issued_on.as_str()).max();
    if latest_issue.is_some() && !indexed.exists() {
        out.push(Step {
            what: "Refresh the archive index".to_owned(),
            why: "documents have been issued and the index has never been built".to_owned(),
            command: Some("docket index".to_owned()),
        });
    }

    Ok(())
}
