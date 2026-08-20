//! Asking an operator questions on a terminal.

use anyhow::{Context, Result};
use std::io::{BufRead, Write};

/// Ask one question, returning `None` when the answer is left blank.
///
/// # Errors
/// Fails if the terminal cannot be read or written.
pub fn ask(
    out: &mut impl Write,
    input: &mut impl BufRead,
    label: &str,
    help: &str,
    required: bool,
    default: Option<&str>,
) -> Result<Option<String>> {
    loop {
        writeln!(out, "\n  {help}")?;
        let marker = if required { " (required)" } else { "" };
        match default {
            Some(d) => write!(out, "  {label}{marker} [{d}]: ")?,
            None => write!(out, "  {label}{marker}: ")?,
        }
        out.flush()?;

        let mut line = String::new();
        let read = input.read_line(&mut line).context("reading the answer")?;
        // End of input: treat as accepting the default rather than looping
        // forever on a closed pipe.
        if read == 0 {
            return Ok(default.map(ToOwned::to_owned));
        }
        let answer = line.trim();

        if answer.is_empty() {
            if let Some(d) = default {
                return Ok(Some(d.to_owned()));
            }
            if !required {
                return Ok(None);
            }
            writeln!(out, "  -- this one is required")?;
            continue;
        }
        return Ok(Some(answer.to_owned()));
    }
}

/// Ask a yes/no question. Blank takes `default`.
///
/// # Errors
/// Fails if the terminal cannot be read or written.
pub fn confirm(
    out: &mut impl Write,
    input: &mut impl BufRead,
    question: &str,
    default: bool,
) -> Result<bool> {
    let hint = if default { "Y/n" } else { "y/N" };
    write!(out, "\n{question} [{hint}]: ")?;
    out.flush()?;
    let mut line = String::new();
    if input.read_line(&mut line).context("reading the answer")? == 0 {
        return Ok(default);
    }
    Ok(match line.trim().to_lowercase().as_str() {
        "y" | "yes" => true,
        "n" | "no" => false,
        _ => default,
    })
}

#[cfg(test)]
// A test that cannot assert an expected value is not a test. The lint stays
// denied for the crate; only assertions inside test modules are exempt.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::{ask, confirm};

    fn run(script: &str, required: bool, default: Option<&str>) -> (Option<String>, String) {
        let mut out = Vec::new();
        let mut input = script.as_bytes();
        let answer = ask(
            &mut out,
            &mut input,
            "Name",
            "who they are",
            required,
            default,
        )
        .expect("asks");
        (answer, String::from_utf8_lossy(&out).into_owned())
    }

    #[test]
    fn an_answer_is_trimmed() {
        assert_eq!(
            run("  Asha Verma  \n", true, None).0.as_deref(),
            Some("Asha Verma")
        );
    }

    #[test]
    fn a_blank_optional_answer_is_none() {
        assert_eq!(run("\n", false, None).0, None);
    }

    #[test]
    fn a_blank_takes_the_default() {
        assert_eq!(run("\n", true, Some("they")).0.as_deref(), Some("they"));
    }

    #[test]
    fn a_required_question_is_asked_again() {
        let (answer, shown) = run("\nAsha\n", true, None);
        assert_eq!(answer.as_deref(), Some("Asha"));
        assert!(shown.contains("this one is required"), "{shown}");
    }

    #[test]
    fn the_stated_purpose_is_shown_as_help() {
        // If the purpose cannot be written down the field should not exist, so
        // it is always worth putting in front of whoever is answering.
        assert!(run("x\n", true, None).1.contains("who they are"));
    }

    #[test]
    fn closed_input_does_not_loop_forever() {
        // A required question with no input would otherwise spin.
        assert_eq!(run("", true, None).0, None);
        assert_eq!(run("", true, Some("d")).0.as_deref(), Some("d"));
    }

    #[test]
    fn confirm_reads_yes_and_no_and_falls_back_to_the_default() {
        let mut out = Vec::new();
        for (script, default, expected) in [
            ("y\n", false, true),
            ("no\n", true, false),
            ("\n", true, true),
            ("banana\n", false, false),
            ("", true, true),
        ] {
            let mut input = script.as_bytes();
            assert_eq!(
                confirm(&mut out, &mut input, "ok?", default).expect("asks"),
                expected,
                "script {script:?}"
            );
        }
    }
}
