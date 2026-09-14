//! A read-only view of everyone, and of one person at a time.

use crate::adapters::store::Store;
use crate::adapters::{registry::Registry, screen};
use crate::config::Paths;
use crate::domain::identity::Standing;
use anyhow::Result;
use std::io::{BufRead, Write};

/// What the browser is showing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    /// Everyone, numbered.
    Roster,
    /// One person, by slug.
    Person(String),
}

/// What a keystroke meant.
#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    /// Redraw whatever is showing.
    Stay,
    /// Show this view next.
    Go(View),
    /// Leave.
    Quit,
    /// Say this, then redraw.
    Complain(String),
}

/// Interpret one line of input against what is currently showing.
///
/// Pure, so the whole navigation is testable without a terminal. The browser
/// is READ-ONLY by design: there is no command here that writes, so a view
/// left half-finished cannot corrupt a record. Writing is `docket new -i`,
/// `sign`, `advance` -- each of which says what it did.
#[must_use]
pub fn interpret(view: &View, line: &str, slugs: &[String]) -> Action {
    let input = line.trim();
    match input.to_lowercase().as_str() {
        "q" | "quit" | "exit" => return Action::Quit,
        "r" | "" => return Action::Stay,
        "b" | "back" => {
            return match view {
                View::Person(_) => Action::Go(View::Roster),
                View::Roster => Action::Stay,
            };
        }
        _ => {}
    }

    // A number selects from the roster, wherever you are.
    if let Ok(n) = input.parse::<usize>() {
        return match n.checked_sub(1).and_then(|i| slugs.get(i)) {
            Some(slug) => Action::Go(View::Person(slug.clone())),
            None => Action::Complain(format!(
                "there is no {n}; the roster has {} {}",
                slugs.len(),
                if slugs.len() == 1 { "entry" } else { "entries" }
            )),
        };
    }

    // Otherwise treat it as a slug, so a name typed from memory works.
    if slugs.iter().any(|s| s == input) {
        return Action::Go(View::Person(input.to_owned()));
    }
    Action::Complain(format!(
        "no entry {input:?}. Type a number, a slug, b, r, or q"
    ))
}

/// Draw the roster.
///
/// # Errors
/// Fails if the store cannot be read or the terminal cannot be written.
pub fn draw_roster(
    out: &mut impl Write,
    store: &dyn Store,
    registry: &Registry,
) -> Result<Vec<String>> {
    let lines = crate::application::roster::build(store, registry)?;
    screen::clear(out)?;
    screen::heading(out, "Everyone Paridyum has engaged")?;
    writeln!(
        out,
        "\n  {} {} {} {} {} {}",
        screen::pad("#", 3),
        screen::pad("PERSON", 17),
        screen::pad("CLASS", 11),
        screen::pad("STANDING", 11),
        screen::pad("NAME", 22),
        screen::pad("DOCS", 5)
    )?;

    let mut slugs = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        writeln!(
            out,
            "  {} {} {} {} {} {}",
            screen::pad(&(i + 1).to_string(), 3),
            screen::pad(&l.person_id, 17),
            screen::pad(&l.class, 11),
            screen::pad(l.standing.as_str(), 11),
            screen::pad(l.name.as_deref().unwrap_or("--"), 22),
            screen::pad(&l.documents.to_string(), 5)
        )?;
        slugs.push(l.slug.clone());
    }
    if lines.is_empty() {
        writeln!(
            out,
            "\n  nobody yet. `docket new <slug> intern -i` opens a record."
        )?;
    }

    let (active, separated, erased) = crate::application::roster::tally(&lines);
    writeln!(
        out,
        "\n{} {}: {active} active, {separated} separated, {erased} erased",
        lines.len(),
        if lines.len() == 1 { "person" } else { "people" }
    )?;
    Ok(slugs)
}

/// Draw one person: where they stand, and what to do next.
///
/// # Errors
/// Fails if the store cannot be read or the terminal cannot be written.
pub fn draw_person(
    out: &mut impl Write,
    store: &dyn Store,
    paths: &Paths,
    registry: &Registry,
    slug: &str,
    company: Option<&str>,
) -> Result<()> {
    screen::clear(out)?;
    let who = crate::application::personnel::identify_at(store, slug, company)?;
    screen::heading(out, &format!("{}  ({slug})", who.person_id))?;
    writeln!(
        out,
        "engagement {}   class {}   standing {} since {}",
        who.engagement_id,
        who.class,
        who.standing.as_str(),
        who.standing_changed_on
    )?;

    if who.standing == Standing::Erased {
        writeln!(
            out,
            "\nThis person's data was erased. The identifier remains so that"
        )?;
        writeln!(out, "serials issued to them still resolve.")?;
        return Ok(());
    }

    let signatures = store.signatures()?;
    let register = store.register()?;
    let mine: Vec<_> = register
        .iter()
        .filter(|e| e.person_id == who.person_id)
        .collect();

    writeln!(out, "\nDOCUMENTS ({})", mine.len())?;
    for e in &mine {
        let state = if crate::domain::signature::is_signed(&signatures, &e.serial) {
            "executed"
        } else {
            "issued"
        };
        writeln!(
            out,
            "  {} {} {} {}",
            screen::pad(&e.code, 5),
            screen::pad(&e.serial, 22),
            screen::pad(state, 10),
            screen::fit(&e.title, 44)
        )?;
    }

    writeln!(out, "\nNEXT")?;
    for (i, step) in crate::application::next::steps(store, paths, registry, slug, company)?
        .iter()
        .enumerate()
    {
        writeln!(out, "  {}. {}", i + 1, step.what)?;
        if let Some(cmd) = &step.command {
            writeln!(out, "     {cmd}")?;
        }
    }
    Ok(())
}

/// Run the browser until the operator leaves.
///
/// # Errors
/// Fails if the store cannot be read or the terminal cannot be written.
pub fn run(
    store: &dyn Store,
    paths: &Paths,
    registry: &Registry,
    out: &mut impl Write,
    input: &mut impl BufRead,
) -> Result<()> {
    let mut view = View::Roster;
    let mut message: Option<String> = None;

    loop {
        let slugs = match &view {
            View::Roster => draw_roster(out, store, registry)?,
            View::Person(slug) => {
                draw_person(out, store, paths, registry, slug, None)?;
                // Keep the roster's ordering available so a number still works.
                crate::application::roster::build(store, registry)?
                    .into_iter()
                    .map(|l| l.slug)
                    .collect()
            }
        };

        if let Some(m) = message.take() {
            writeln!(out, "\n  {m}")?;
        }
        write!(
            out,
            "\n[number] open   [b] back   [r] refresh   [q] quit\n> "
        )?;
        out.flush()?;

        let mut line = String::new();
        // End of input leaves, rather than spinning on a closed pipe.
        if input.read_line(&mut line)? == 0 {
            writeln!(out)?;
            return Ok(());
        }
        match interpret(&view, &line, &slugs) {
            Action::Quit => return Ok(()),
            Action::Stay => {}
            Action::Go(next) => view = next,
            Action::Complain(m) => message = Some(m),
        }
    }
}

#[cfg(test)]
// A test that cannot assert an expected value is not a test. The lint stays
// denied for the crate; only assertions inside test modules are exempt.
#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic
)]
mod tests {
    use super::{Action, View, interpret};

    fn slugs() -> Vec<String> {
        vec!["asha-verma".to_owned(), "priya-nair".to_owned()]
    }

    #[test]
    fn a_number_opens_that_row() {
        assert_eq!(
            interpret(&View::Roster, "2", &slugs()),
            Action::Go(View::Person("priya-nair".to_owned()))
        );
    }

    #[test]
    fn numbering_is_one_based_because_the_screen_is() {
        assert_eq!(
            interpret(&View::Roster, "1", &slugs()),
            Action::Go(View::Person("asha-verma".to_owned()))
        );
        assert!(matches!(
            interpret(&View::Roster, "0", &slugs()),
            Action::Complain(_)
        ));
    }

    #[test]
    fn a_number_off_the_end_says_how_many_there_are() {
        let Action::Complain(m) = interpret(&View::Roster, "9", &slugs()) else {
            panic!("expected a complaint");
        };
        assert!(m.contains("2 entries"), "{m}");
    }

    #[test]
    fn the_count_is_not_pluralised_wrongly_for_one() {
        let one = vec!["asha-verma".to_owned()];
        let Action::Complain(m) = interpret(&View::Roster, "5", &one) else {
            panic!("expected a complaint");
        };
        assert!(m.contains("1 entry"), "{m}");
    }

    #[test]
    fn a_slug_typed_from_memory_works() {
        assert_eq!(
            interpret(&View::Roster, " priya-nair ", &slugs()),
            Action::Go(View::Person("priya-nair".to_owned()))
        );
    }

    #[test]
    fn back_returns_to_the_roster_and_does_nothing_there() {
        assert_eq!(
            interpret(&View::Person("asha-verma".to_owned()), "b", &slugs()),
            Action::Go(View::Roster)
        );
        assert_eq!(interpret(&View::Roster, "b", &slugs()), Action::Stay);
    }

    #[test]
    fn blank_and_r_both_redraw() {
        assert_eq!(interpret(&View::Roster, "", &slugs()), Action::Stay);
        assert_eq!(interpret(&View::Roster, "  \n", &slugs()), Action::Stay);
        assert_eq!(interpret(&View::Roster, "r", &slugs()), Action::Stay);
    }

    #[test]
    fn quitting_is_spelled_several_ways() {
        for word in ["q", "Q", "quit", "exit"] {
            assert_eq!(
                interpret(&View::Roster, word, &slugs()),
                Action::Quit,
                "{word}"
            );
        }
    }

    #[test]
    fn nonsense_says_what_the_keys_are() {
        let Action::Complain(m) = interpret(&View::Roster, "banana", &slugs()) else {
            panic!("expected a complaint");
        };
        assert!(m.contains("number"), "{m}");
        assert!(m.contains('q'), "{m}");
    }

    #[test]
    fn nothing_in_the_browser_writes() {
        // The point of it being read-only: no input can reach a write path, so
        // a half-drawn view cannot leave a record in a bad state.
        for word in ["q", "b", "r", "1", "asha-verma", "banana", "0"] {
            let action = interpret(&View::Roster, word, &slugs());
            assert!(
                matches!(
                    action,
                    Action::Quit | Action::Stay | Action::Go(_) | Action::Complain(_)
                ),
                "{word} produced something other than navigation"
            );
        }
    }
}
