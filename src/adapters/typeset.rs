//! Producing a PDF from a template and a set of definitions.

use crate::adapters::clock;
use crate::config::Paths;
use crate::domain::definitions::Definitions;
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The TeX Live release the `pdflatex` path is reproducible against.
///
/// Kept as the default for entries that record no engine, so a register
/// written before engine detection existed still reads correctly.
pub const TEXLIVE: &str = "2026";

/// A typesetter this tool knows how to drive.
///
/// TWO ENGINES, NOT A REPLACEMENT (ADR-0054). `pdflatex` was here first, every
/// existing register entry was produced by it, and every machine in the estate
/// already has TeX Live. Tectonic is added for machines that do not — a single
/// binary that fetches what it needs, instead of a multi-gigabyte install.
/// Swapping would have invalidated the reproducibility claim on every document
/// already issued, to spare an install that only a new operator experiences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Engine {
    /// TeX Live's `pdflatex`. Preferred when present.
    PdfLatex,
    /// Tectonic. Used when `pdflatex` is absent.
    Tectonic,
}

impl Engine {
    /// The executable name.
    #[must_use]
    pub fn program(self) -> &'static str {
        match self {
            Self::PdfLatex => "pdflatex",
            Self::Tectonic => "tectonic",
        }
    }

    /// The name recorded on a register entry.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PdfLatex => "pdflatex",
            Self::Tectonic => "tectonic",
        }
    }

    /// How many passes it needs to settle cross-references.
    ///
    /// pdfTeX needs three: hyperref writes its outline on the first and
    /// `lastpage` resolves `Page X of Y` on the second. Tectonic reruns
    /// internally until stable, so driving it three times is wasted work.
    #[must_use]
    pub fn passes(self) -> u8 {
        match self {
            Self::PdfLatex => 3,
            Self::Tectonic => 1,
        }
    }

    /// Whether this engine is on the PATH.
    #[must_use]
    pub fn available(self) -> bool {
        Command::new(self.program())
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    }

    /// Its reported version, for the register.
    #[must_use]
    pub fn version(self) -> String {
        let out = Command::new(self.program()).arg("--version").output();
        out.ok()
            .and_then(|o| {
                String::from_utf8(o.stdout)
                    .ok()
                    .and_then(|t| t.lines().next().map(str::trim).map(ToOwned::to_owned))
            })
            .unwrap_or_else(|| "unknown".to_owned())
    }
}

/// Pick an engine: `pdflatex` if present, otherwise Tectonic.
///
/// Preference order is deliberate and is the ADR-0054 point in miniature. The
/// engine that produced the existing corpus wins when it is available, so a
/// machine that has always worked keeps producing byte-identical output; the
/// alternative serves only machines that would otherwise produce nothing.
///
/// # Errors
/// Fails when neither is installed, naming both rather than only the default.
pub fn detect() -> Result<Engine> {
    for engine in [Engine::PdfLatex, Engine::Tectonic] {
        if engine.available() {
            return Ok(engine);
        }
    }
    bail!(
        "no typesetter found. Install TeX Live (which provides `pdflatex`), or \
         Tectonic, which is a single binary and fetches what it needs:\n  \
         https://tectonic-typesetting.github.io"
    )
}

/// Three passes: hyperref writes its outline file on the first, `lastpage`
/// resolves `Page X of Y` on the second, and the third settles both.
const PASSES: u8 = 3;

/// What to render, and whether it counts as issued.
#[derive(Debug)]
pub struct Job<'a> {
    /// Template to typeset.
    pub template: &'a Path,
    /// Directory for the intermediate files and the output PDF.
    pub out_dir: &'a Path,
    /// Values the template reads.
    pub definitions: &'a Definitions,
    /// Date the document bears, which also pins `SOURCE_DATE_EPOCH`.
    pub date: time::Date,
    /// A draft is watermarked, carries no serial, and is never registered.
    pub draft: bool,
    /// XMP metadata for the archival record. Empty for a draft.
    pub metadata: Metadata,
}

/// What the PDF says about itself.
///
/// Written to `<jobname>.xmpdata`, which pdfx reads. Without it the file has
/// an empty `/Title`, `/Author` and `/Subject`, so a PDF pulled out of the
/// store -- emailed, downloaded from a cloud folder -- is anonymous and can
/// only be identified by reading its body.
#[derive(Debug, Default, Clone)]
pub struct Metadata {
    /// Document title.
    pub title: String,
    /// Who issued it.
    pub author: String,
    /// Serial, recipient identifier and engagement.
    pub subject: String,
    /// Semicolon-separated keywords.
    pub keywords: String,
}

impl Metadata {
    /// Render the `.xmpdata` file pdfx consumes.
    ///
    /// Braces and backslashes are stripped rather than escaped: this file is
    /// read as LaTeX by pdfx, and a stray brace from a person's name would
    /// break the build at issuance time for no benefit.
    #[must_use]
    pub fn to_xmpdata(&self) -> String {
        let clean = |v: &str| {
            v.chars()
                .filter(|c| !matches!(c, '{' | '}' | '\\' | '%' | '#' | '$'))
                .collect::<String>()
        };
        format!(
            "% GENERATED by docket -- do not edit.\n\
             \\Title{{{}}}\n\\Author{{{}}}\n\\Subject{{{}}}\n\\Keywords{{{}}}\n\
             \\Publisher{{{}}}\n",
            clean(&self.title),
            clean(&self.author),
            clean(&self.subject),
            clean(&self.keywords),
            clean(&self.author),
        )
    }
}

/// Render one document, returning the path to the PDF.
///
/// # Errors
/// Fails if `pdflatex` is absent, any pass fails, or no PDF appears.
pub fn render(paths: &Paths, job: &Job<'_>) -> Result<PathBuf> {
    fs::create_dir_all(job.out_dir)
        .with_context(|| format!("creating {}", job.out_dir.display()))?;

    let stem = job
        .template
        .file_stem()
        .and_then(|s| s.to_str())
        .context("template path has no usable file name")?;

    fs::write(job.out_dir.join("dkdata.tex"), job.definitions.to_tex())
        .context("writing the generated definitions")?;

    // pdfx reads <jobname>.xmpdata from the output directory.
    fs::write(
        job.out_dir.join(format!("{stem}.xmpdata")),
        job.metadata.to_xmpdata(),
    )
    .context("writing the XMP metadata")?;

    let source = if job.draft {
        let body = fs::read_to_string(job.template)
            .with_context(|| format!("reading {}", job.template.display()))?;
        let drafted = body.replace(
            r"\documentclass{paridyum}",
            r"\documentclass[draft]{paridyum}",
        );
        if drafted == body {
            bail!(
                "{}: expected a `\\documentclass{{paridyum}}` line to switch into draft mode",
                job.template.display()
            );
        }
        let path = job.out_dir.join(format!("{stem}.tex"));
        fs::write(&path, drafted).context("writing the draft source")?;
        path
    } else {
        job.template.to_path_buf()
    };

    // TEXINPUTS must find the generated data first, then the class, then assets.
    // The trailing colon preserves the default search path.
    let texinputs = format!(
        "{}:{}:{}:",
        job.out_dir.display(),
        paths.root.join("latex").display(),
        paths.root.join("latex").join("assets").display()
    );

    let engine = detect()?;
    for pass in 1..=PASSES {
        let mut cmd = Command::new(engine.program());
        cmd.current_dir(&paths.root)
            .env("TEXINPUTS", &texinputs)
            // Pins every date the engine would otherwise take from the wall
            // clock. Both engines honour it.
            .env("SOURCE_DATE_EPOCH", clock::epoch_seconds(job.date).to_string())
            .env("FORCE_SOURCE_DATE", "1");

        match engine {
            Engine::PdfLatex => {
                cmd.arg("-interaction=nonstopmode")
                    .arg("-halt-on-error")
                    .arg(format!("-output-directory={}", job.out_dir.display()))
                    .arg(&source);
            }
            Engine::Tectonic => {
                // Tectonic resolves cross-references itself, so the loop below
                // runs it once. `--keep-intermediates` so a failure leaves the
                // same .log an operator would look for after pdflatex.
                cmd.arg("--outdir")
                    .arg(job.out_dir)
                    .arg("--keep-intermediates")
                    .arg("--keep-logs")
                    .arg(&source);
            }
        }

        let output = cmd.output().with_context(|| {
            format!(
                "running {} (it reported as available a moment ago)",
                engine.program()
            )
        })?;

        if !output.status.success() {
            let log = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let errors: Vec<&str> = log
                .lines()
                .chain(stderr.lines())
                .filter(|l| l.starts_with('!') || l.contains("error:"))
                .take(4)
                .collect();
            let detail = if errors.is_empty() {
                format!("see {}", job.out_dir.join(format!("{stem}.log")).display())
            } else {
                errors.join("\n       ")
            };
            bail!(
                "{stem}: {} failed on pass {pass} of {}\n       {detail}",
                engine.program(),
                engine.passes()
            );
        }

        // Tectonic needs one pass; pdfTeX needs three to settle hyperref and
        // \pageref{LastPage}.
        if pass >= engine.passes() {
            break;
        }
    }

    let pdf = job.out_dir.join(format!("{stem}.pdf"));
    if !pdf.exists() {
        bail!(
            "{stem}: {} reported success but produced no PDF",
            engine.program()
        );
    }
    Ok(pdf)
}
