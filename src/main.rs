//! Command-line entry point for issuing and verifying Paridyum documents.

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use tool_docket::adapters::clock::{self, Clock, SystemClock};
use tool_docket::adapters::http;
use tool_docket::adapters::{registry::Registry, store, typeset};
use tool_docket::application::{
    archive, browse, check, comms, enrol, issue, next, pack, personnel, roster, sign, status,
    verify,
};
use tool_docket::config::Paths;
use tool_docket::domain::address;
use tool_docket::domain::event::Kind;
use tool_docket::domain::identity::Standing;
use tool_docket::domain::person;
use tool_docket::domain::signature::Method;

/// Paridyum's internship document system.
#[derive(Parser)]
#[command(name = "docket", version, about, long_about = None)]
struct Cli {
    /// Repository root. Defaults to the directory holding this binary's crate.
    #[arg(long, global = true)]
    root: Option<PathBuf>,
    /// Who is running this. Recorded against everything it writes.
    ///
    /// Falls back to `DOCKET_OPERATOR`. Required for any command that writes:
    /// a log that cannot say who did something is not an audit trail.
    #[arg(long, global = true)]
    by: Option<String>,
    /// Browse everyone in a full-screen, read-only view.
    ///
    /// The CLI is the default. This is an alternate way in for looking at the
    /// whole roster at once, which is the one thing a command line does badly.
    /// It writes nothing.
    #[arg(long, global = true)]
    tui: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// The next thing to do for a person.
    Next {
        /// Person slug.
        slug: String,
    },
    /// Show what a person has been issued and what is still owed.
    Status {
        /// Person slug, matching a file in `people/`.
        slug: String,
    },
    /// Issue everything the person's current stage owes.
    Onboard {
        /// Person slug.
        slug: String,
        /// Issue date, ISO. Defaults to today.
        #[arg(long)]
        date: Option<String>,
        /// Issue despite refusals. The documents will be defective.
        #[arg(long)]
        force: bool,
    },
    /// Issue one document.
    Issue {
        /// Person slug.
        slug: String,
        /// Document code, e.g. NDA.
        code: String,
        /// Issue date, ISO. Defaults to today.
        #[arg(long)]
        date: Option<String>,
        /// Issue despite refusals. The documents will be defective.
        #[arg(long)]
        force: bool,
    },
    /// Render a draft: watermarked, no serial, not registered.
    Render {
        /// Person slug.
        slug: String,
        /// Document code.
        code: String,
    },
    /// Check a PDF against the register.
    Verify {
        /// Serial, e.g. PAR/NDA/2026/0001.
        serial: String,
        /// File to check. Defaults to the stored copy.
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Validate registries, templates, records and every chain.
    Check,
    /// Serve the operator API over loopback, for console-docket.
    ///
    /// Binds 127.0.0.1 only and refuses anything else. There is no login: SSH
    /// keys are the authentication and the operating system is the identity, which
    /// holds only while the socket cannot be reached from another machine. Reach it
    /// with `ssh -L 8080:127.0.0.1:8080 <host>`.
    Serve {
        /// Port to listen on.
        #[arg(long, default_value_t = 8080)]
        port: u16,
    },
    /// Record that an issued document came back executed.
    Sign {
        /// Serial, e.g. PAR/NDA/2026/0001.
        serial: String,
        /// How it was signed: wet-ink | aadhaar-esign | electronic.
        #[arg(long, default_value = "wet-ink")]
        method: String,
        /// Date the last party signed, ISO. Defaults to today.
        #[arg(long)]
        on: Option<String>,
        /// Date Paridyum received it, ISO. Defaults to the signing date.
        #[arg(long)]
        received: Option<String>,
        /// The returned scan. Required for wet-ink, which leaves no other trail.
        #[arg(long)]
        scan: Option<PathBuf>,
    },
    /// Move a person to the next lifecycle stage.
    Advance {
        /// Person slug.
        slug: String,
        /// Advance despite an unmet signature gate. Says what it is skipping.
        #[arg(long)]
        force: bool,
        /// Date of the move, ISO. Defaults to today.
        #[arg(long)]
        date: Option<String>,
    },
    /// Assemble a person's documents into a folder to hand over.
    Pack {
        /// Person slug.
        slug: String,
        /// Where to write it. Defaults to store/packs.
        #[arg(long)]
        out: Option<PathBuf>,
        /// Date on the cover sheet, ISO. Defaults to today.
        #[arg(long)]
        date: Option<String>,
    },
    /// Show the messages to send, filled in, with their attachments.
    Emails {
        /// Person slug.
        slug: String,
        /// Only this message, by id.
        #[arg(long)]
        only: Option<String>,
    },
    /// Regenerate the archive index and payload manifest.
    Index,
    /// Print the published team as JSON, for the public web surface.
    Team,
    /// Show the two addresses a person should be provisioned.
    Addresses {
        /// Person slug.
        slug: String,
        /// Disambiguate a collision. The NEW address takes the suffix; an
        /// address already in use is never renamed.
        #[arg(long)]
        suffix: Option<u32>,
    },
    /// Report documents past their retention period.
    Retention {
        /// Evaluate as at this date, ISO. Defaults to today.
        #[arg(long)]
        date: Option<String>,
    },
    /// Open a record for a new person and allocate their identifier.
    New {
        /// Slug to file the record under: lowercase, hyphenated.
        slug: String,
        /// Person class, e.g. intern.
        class: String,
        /// Date of record creation, ISO. Defaults to today.
        #[arg(long)]
        date: Option<String>,
        /// Ask the questions instead of copying the template.
        ///
        /// The questions are generated from registry/fields.yaml, so they
        /// cannot drift from the schema and each one shows the field's stated
        /// purpose. Without this flag the template is copied, as before.
        #[arg(short, long)]
        interactive: bool,
    },
    /// Open a new engagement for a person who already has an identifier.
    Reengage {
        /// Person slug.
        slug: String,
        /// Person class of the new engagement, e.g. employee.
        class: String,
        /// Date the engagement opens, ISO. Defaults to today.
        #[arg(long)]
        date: Option<String>,
    },
    /// Show everything held about one person.
    Record {
        /// Person slug.
        slug: String,
    },
    /// List everyone Paridyum has engaged.
    Roster {
        /// Show only people with this standing.
        #[arg(long)]
        standing: Option<String>,
    },
    /// Record something that happened to a person's engagement.
    Event {
        /// Person slug.
        slug: String,
        /// Event kind, e.g. `access_granted`.
        kind: String,
        /// What it concerned: a system, a document, a piece of equipment.
        #[arg(long)]
        detail: Option<String>,
        /// Date it happened, ISO. Defaults to today.
        #[arg(long)]
        date: Option<String>,
    },
    /// Mark a person separated. Documents and obligations survive.
    Separate {
        /// Person slug.
        slug: String,
        /// Last working date, ISO. Defaults to today.
        #[arg(long)]
        date: Option<String>,
    },
    /// Erase a person's data, keeping their identifier as a tombstone.
    Erase {
        /// Person slug.
        slug: String,
        /// Confirms the deletion is intended. Required.
        #[arg(long)]
        confirm: bool,
        /// Date of erasure, ISO. Defaults to today.
        #[arg(long)]
        date: Option<String>,
    },
}

/// Exit codes are contractual: 0 clean, 1 outstanding or integrity failure,
/// 2 refused or misused. This is what makes the tool usable from CI.
fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    let root = match cli.root {
        Some(r) => r,
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR")),
    };
    let paths = Paths::resolve(&root);
    let registry = Registry::load(&paths)
        .context("loading registry/ (is --root pointing at the repository?)")?;

    // Resolved once. A write command without an operator is refused rather
    // than defaulted, because a default is exactly the thing that made every
    // event in the log say "docket".
    let operator = cli
        .by
        .filter(|v| !v.trim().is_empty())
        // An EMPTY variable is unset, not an operator called "". Exporting a
        // blank is how a shell says "no value", and treating it as a name
        // turned every read command into an error.
        .or_else(|| {
            std::env::var("DOCKET_OPERATOR")
                .ok()
                .filter(|v| !v.trim().is_empty())
        })
        .map(|id| registry.operator(&id).map(|o| o.id.clone()))
        .transpose()?;

    // `--tui` wins, with or without a subcommand. It writes nothing, so it
    // needs no operator.
    if cli.tui {
        let stdin = std::io::stdin();
        let mut input = stdin.lock();
        let mut out = std::io::stdout();
        browse::run(&paths, &registry, &mut out, &mut input)?;
        return Ok(ExitCode::SUCCESS);
    }
    let Some(command) = cli.command else {
        // Printing help beats inventing a default action.
        <Cli as clap::CommandFactory>::command().print_help()?;
        println!();
        return Ok(ExitCode::from(2));
    };
    dispatch(command, &paths, &registry, operator.as_deref())
}

/// Route one command. Split from `run` so the routing table can grow without
/// the entry point becoming the longest function in the crate.
fn dispatch(
    command: Command,
    paths: &Paths,
    registry: &Registry,
    operator: Option<&str>,
) -> Result<ExitCode> {
    match command {
        Command::Next { slug } => cmd_next(paths, registry, &slug),
        Command::Status { slug } => cmd_status(paths, registry, &slug),
        Command::Onboard { slug, date, force } => {
            cmd_onboard(paths, registry, &slug, date.as_deref(), force, operator)
        }
        Command::Issue {
            slug,
            code,
            date,
            force,
        } => cmd_issue(
            paths,
            registry,
            &slug,
            &code,
            date.as_deref(),
            force,
            operator,
        ),
        Command::Render { slug, code } => cmd_render(paths, registry, &slug, &code),
        Command::Verify { serial, file } => cmd_verify(paths, &serial, file.as_deref()),
        Command::Check => cmd_check(paths, registry),
        Command::Sign {
            serial,
            method,
            on,
            received,
            scan,
        } => cmd_sign(
            paths,
            registry,
            &serial,
            &method,
            on.as_deref(),
            received.as_deref(),
            scan.as_deref(),
            operator,
        ),
        Command::Advance { slug, force, date } => {
            cmd_advance(paths, registry, &slug, force, date.as_deref(), operator)
        }
        Command::Pack { slug, out, date } => cmd_pack(
            paths,
            registry,
            &slug,
            out.as_deref(),
            date.as_deref(),
            operator,
        ),
        Command::Emails { slug, only } => cmd_emails(paths, registry, &slug, only.as_deref()),
        Command::Index => cmd_index(paths, registry),
        Command::Team => cmd_team(registry),
        Command::Addresses { slug, suffix } => cmd_addresses(paths, &slug, suffix),
        Command::Retention { date } => cmd_retention(paths, registry, date.as_deref()),
        Command::New {
            slug,
            class,
            date,
            interactive,
        } => cmd_new(
            interactive,
            paths,
            registry,
            &slug,
            &class,
            date.as_deref(),
            operator,
        ),
        Command::Reengage { slug, class, date } => {
            cmd_reengage(paths, registry, &slug, &class, date.as_deref(), operator)
        }
        Command::Record { slug } => cmd_record(paths, registry, &slug),
        Command::Serve { port } => cmd_serve(paths, registry, port),
        Command::Roster { standing } => cmd_roster(paths, registry, standing.as_deref()),
        Command::Event {
            slug,
            kind,
            detail,
            date,
        } => cmd_event(paths, &slug, &kind, detail.as_deref(), date.as_deref()),
        Command::Separate { slug, date } => cmd_separate(paths, &slug, date.as_deref(), operator),
        Command::Erase {
            slug,
            confirm,
            date,
        } => cmd_erase(paths, &slug, confirm, date.as_deref(), operator),
    }
}

/// The operator, or a refusal explaining how to supply one.
///
/// Deliberately not defaulted. A default is exactly what made every event in
/// the log say "docket", and an audit trail that cannot name a person is not
/// one.
fn writer(operator: Option<&str>) -> Result<&str> {
    operator.context(
        "this command writes to the record, so it needs an operator: \
         pass --by <id>, or set DOCKET_OPERATOR. `docket check` lists who is declared",
    )
}

fn resolve_date(explicit: Option<&str>) -> Result<time::Date> {
    match explicit {
        Some(d) => clock::parse_iso(d),
        None => SystemClock.today(),
    }
}

fn cmd_next(paths: &Paths, registry: &Registry, slug: &str) -> Result<ExitCode> {
    let steps = next::steps(paths, registry, slug)?;
    let done = steps.len() == 1 && steps.first().is_some_and(|s| s.command.is_none());
    for (i, step) in steps.iter().enumerate() {
        println!("{}. {}", i + 1, step.what);
        println!("   why: {}", step.why);
        if let Some(cmd) = &step.command {
            println!("   run: {cmd}");
        }
        println!();
    }
    // Exit 0 only when there is nothing to do, so this can drive a loop.
    Ok(if done {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn cmd_status(paths: &Paths, registry: &Registry, slug: &str) -> Result<ExitCode> {
    let person = store::person(paths, slug)?;
    let who = personnel::identify(paths, slug)?;
    let register = store::register(paths)?;
    let signatures = store::signatures(paths)?;
    let report = status::report(registry, &person, &who, &register, &signatures)?;

    println!("{}  ({})", report.name, report.slug);
    println!("person     {}", who.person_id);
    println!("engagement {}  class {}", who.engagement_id, who.class);
    println!("stage: {}\n", report.stage);
    for line in &report.lines {
        let (mark, note) = match &line.state {
            status::State::Signed(serial) => ("x", serial.as_str()),
            status::State::Issued(serial) => ("~", serial.as_str()),
            status::State::Outstanding => (" ", "NOT ISSUED"),
            status::State::NotApplicable => ("-", "not applicable"),
        };
        println!("  [{mark}] {}  {:<52} {note}", line.code, line.title);
    }
    println!(
        "\n{}/{} issued and {} executed, at stage \"{}\"",
        report.issued(),
        report.owed(),
        report.signed(),
        report.stage
    );

    let outstanding = report.outstanding();
    let awaiting = report.awaiting_signature();
    if !outstanding.is_empty() {
        println!("not issued: {}", outstanding.join(" "));
    }
    if !awaiting.is_empty() {
        // Issuing is not finishing. `[~]` marks a document that went out and
        // has not come back.
        println!("awaiting the executed copy back: {}", awaiting.join(" "));
    }
    if outstanding.is_empty() && awaiting.is_empty() {
        println!("nothing outstanding");
        return Ok(ExitCode::SUCCESS);
    }
    Ok(ExitCode::from(1))
}

/// Report refusals that `--force` cannot bypass, and say so.
fn report_unforceable(refusals: &[String], slug: &str) {
    eprintln!("refusing to issue for {slug}:");
    for r in refusals {
        eprintln!("  - {r}");
    }
    eprintln!(
        "\n--force does not apply here. Fill the record in first: issuing from an \
         unedited template writes real serials into the register, sealed into the \
         chain, on documents that name nobody -- and an issued document can only be \
         corrected by re-issuing under a new serial."
    );
}

fn report_refusals(refusals: &[String], slug: &str) {
    eprintln!("refusing to issue for {slug}:");
    for r in refusals {
        eprintln!("  - {r}");
    }
    eprintln!(
        "\nfix the record, or pass --force to issue anyway \
         (the documents will be defective)."
    );
}

fn cmd_onboard(
    paths: &Paths,
    registry: &Registry,
    slug: &str,
    date: Option<&str>,
    force: bool,
    operator: Option<&str>,
) -> Result<ExitCode> {
    let by = writer(operator)?;
    let person = store::person(paths, slug)?;
    // Checked before `force` is even looked at.
    let unforceable = issue::unforceable(&person);
    if !unforceable.is_empty() {
        report_unforceable(&unforceable, slug);
        return Ok(ExitCode::from(2));
    }
    let refusals = issue::refusals(registry, &person);
    if !refusals.is_empty() && !force {
        report_refusals(&refusals, slug);
        return Ok(ExitCode::from(2));
    }

    let who = personnel::identify(paths, slug)?;
    let register = store::register(paths)?;
    let todo = issue::outstanding(registry, &person, &who, &register)?;
    if todo.is_empty() {
        println!(
            "{slug}: nothing owed at stage \"{}\" that is not already issued",
            person.stage()
        );
        return Ok(ExitCode::SUCCESS);
    }

    let when = resolve_date(date)?;
    let by_code = registry.lifecycle.by_code()?;
    println!(
        "{slug} -- stage \"{}\" -- issuing {}: {}\n",
        person.stage(),
        todo.len(),
        todo.join(" ")
    );
    for code in &todo {
        let Some(doc) = by_code.get(code.as_str()) else {
            bail!("{code}: declared by a stage but not by documents");
        };
        let entry = issue::one(paths, registry, &person, doc, when, by)?;
        println!("  {code}  {:<22} {}", entry.serial, entry.pdf_path);
    }
    println!(
        "\n{} issued. Register: {}",
        todo.len(),
        paths.register().display()
    );
    Ok(ExitCode::SUCCESS)
}

fn cmd_issue(
    paths: &Paths,
    registry: &Registry,
    slug: &str,
    code: &str,
    date: Option<&str>,
    force: bool,
    operator: Option<&str>,
) -> Result<ExitCode> {
    let by = writer(operator)?;
    let person = store::person(paths, slug)?;
    let by_code = registry.lifecycle.by_code()?;
    let Some(doc) = by_code.get(code) else {
        let known: Vec<&str> = by_code.keys().copied().collect();
        bail!("unknown document code {code:?}; known: {}", known.join(" "));
    };
    // Same gate as `onboard`, and for the same reason: one document issued from
    // an unedited template is one real serial spent on a document naming nobody.
    let unforceable = issue::unforceable(&person);
    if !unforceable.is_empty() {
        report_unforceable(&unforceable, slug);
        return Ok(ExitCode::from(2));
    }
    let refusals = issue::refusals(registry, &person);
    if !refusals.is_empty() && !force {
        report_refusals(&refusals, slug);
        return Ok(ExitCode::from(2));
    }
    let entry = issue::one(paths, registry, &person, doc, resolve_date(date)?, by)?;
    println!("{code}  {}  {}", entry.serial, entry.pdf_path);
    Ok(ExitCode::SUCCESS)
}

fn cmd_render(paths: &Paths, registry: &Registry, slug: &str, code: &str) -> Result<ExitCode> {
    let person = store::person(paths, slug)?;
    let by_code = registry.lifecycle.by_code()?;
    let Some(doc) = by_code.get(code) else {
        bail!("unknown document code {code:?}");
    };
    let when = SystemClock.today()?;
    let defs = issue::definitions(registry, Some(&person), None, when);
    let out = paths
        .root
        .join("build")
        .join(format!("draft-{slug}-{code}"));
    let pdf = typeset::render(
        paths,
        &typeset::Job {
            template: &paths.root.join(&doc.template),
            out_dir: &out,
            definitions: &defs,
            date: when,
            draft: true,
            metadata: typeset::Metadata {
                title: format!("DRAFT -- {}", doc.title),
                author: "Paridyum".to_owned(),
                subject: "Draft. Not issued and not registered.".to_owned(),
                keywords: "Paridyum; draft; not issued".to_owned(),
            },
        },
    )?;
    println!("draft (unregistered, watermarked): {}", pdf.display());
    Ok(ExitCode::SUCCESS)
}

fn cmd_verify(paths: &Paths, serial: &str, file: Option<&Path>) -> Result<ExitCode> {
    let register = store::register(paths)?;
    let outcome = verify::verify(paths, &register, serial, file)?;
    println!("serial     {}", outcome.serial);
    println!("document   {}", outcome.title);
    println!("issued_on  {}", outcome.issued_on);
    println!(
        "code       {}   <- must match the code printed on the document",
        outcome.code
    );
    println!("file       {}", outcome.path.display());
    println!("expected   {}", outcome.expected);
    println!("actual     {}", outcome.actual);
    if outcome.matches() {
        println!("\nMATCH: this file is the document that was issued.");
        return Ok(ExitCode::SUCCESS);
    }
    eprintln!("\nMISMATCH: this file is NOT the document recorded under that serial.");
    Ok(ExitCode::from(1))
}

fn cmd_sign(
    paths: &Paths,
    registry: &Registry,
    serial: &str,
    method: &str,
    on: Option<&str>,
    received: Option<&str>,
    scan: Option<&Path>,
    operator: Option<&str>,
) -> Result<ExitCode> {
    let by = writer(operator)?;
    let signed_on = resolve_date(on)?;
    let received_on = match received {
        Some(d) => clock::parse_iso(d)?,
        None => signed_on,
    };
    let s = sign::record(
        paths,
        registry,
        serial,
        Method::parse(method)?,
        signed_on,
        received_on,
        scan,
        by,
    )?;
    println!("{}  {}", s.record.serial, s.title);
    println!(
        "signed on   {}  by {}",
        s.record.signed_on,
        s.record.method.as_str()
    );
    println!("received on {}", s.record.received_on);
    match (&s.record.scan_path, &s.record.scan_sha256) {
        (Some(p), Some(h)) => println!("scan        {p}\n            sha256 {h}"),
        _ => println!("scan        none held"),
    }
    println!("person      {}", s.person_id);
    Ok(ExitCode::SUCCESS)
}

fn cmd_advance(
    paths: &Paths,
    registry: &Registry,
    slug: &str,
    force: bool,
    date: Option<&str>,
    operator: Option<&str>,
) -> Result<ExitCode> {
    let by = writer(operator)?;
    let (from, to, gates) =
        personnel::advance(paths, registry, slug, resolve_date(date)?, force, by)?;
    if gates.is_empty() {
        println!("{slug}: {from} -> {to}");
        if force {
            println!("(--force: the signature gate was not checked)");
        }
        return Ok(ExitCode::SUCCESS);
    }
    eprintln!(
        "refusing to move {slug} out of \"{from}\": {} document(s) are not \
               recorded as executed.",
        gates.len()
    );
    for g in &gates {
        eprintln!("  - {g}");
    }
    eprintln!(
        "\nIssuing a document is not the same as getting it back. Record each with\n\
         `docket sign <serial>`, or pass --force to advance anyway."
    );
    Ok(ExitCode::from(2))
}

fn cmd_pack(
    paths: &Paths,
    registry: &Registry,
    slug: &str,
    out: Option<&Path>,
    date: Option<&str>,
    operator: Option<&str>,
) -> Result<ExitCode> {
    let by = writer(operator)?;
    let target = out.map_or_else(|| pack::default_out(paths), Path::to_path_buf);
    let p = pack::assemble(paths, registry, slug, &target, resolve_date(date)?, by)?;
    println!("{}", p.root.display());
    let mut folder = String::new();
    for f in &p.files {
        let dir = f.relative.rsplit_once('/').map_or("", |(d, _)| d);
        if dir != folder {
            folder.clear();
            folder.push_str(dir);
            if !folder.is_empty() {
                println!("  {folder}/");
            }
        }
        let name = f
            .relative
            .rsplit_once('/')
            .map_or(f.relative.as_str(), |(_, n)| n);
        let indent = if folder.is_empty() { "  " } else { "    " };
        println!("{indent}{name:<52} {}", f.serial);
    }
    println!("  MANIFEST-sha256.txt");
    println!(
        "\n{} documents. Verify the folder with: cd {} && shasum -a 256 -c MANIFEST-sha256.txt",
        p.files.len(),
        p.root.display()
    );
    println!("The store stays authoritative; this is a copy you can hand over or delete.");
    Ok(ExitCode::SUCCESS)
}

fn cmd_emails(
    paths: &Paths,
    registry: &Registry,
    slug: &str,
    only: Option<&str>,
) -> Result<ExitCode> {
    let all = comms::due(paths, registry, slug)?;
    let wanted: Vec<&comms::Rendered> = all
        .iter()
        .filter(|m| only.is_none_or(|o| m.id == o))
        .collect();
    if wanted.is_empty() {
        if let Some(o) = only {
            bail!("no message {o:?} is due for {slug}");
        }
        println!("{slug}: no messages due at this stage");
        return Ok(ExitCode::SUCCESS);
    }

    let mut incomplete = false;
    for m in &wanted {
        println!("{}", "=".repeat(78));
        println!("[{}]  stage {}  ->  {}", m.id, m.stage, m.to.as_str());
        println!("WHEN     {}", m.when);
        println!(
            "TO       {}",
            m.address.as_deref().unwrap_or("(no address recorded)")
        );
        println!("SUBJECT  {}", m.subject);
        if m.attachments.is_empty() {
            println!("ATTACH   (none)");
        } else {
            for (code, title, serial) in &m.attachments {
                println!("ATTACH   {code}  {title:<48} {serial}");
            }
        }
        println!("{}", "-".repeat(78));
        println!("{}", m.body.trim_end());
        if !m.missing.is_empty() {
            incomplete = true;
            eprintln!(
                "\n!! {} names {} document(s) that are not issued: {}",
                m.id,
                m.missing.len(),
                m.missing.join(" ")
            );
        }
    }
    println!("{}", "=".repeat(78));
    println!(
        "\n{} message(s). Paste into your mail client and attach the files from \
         `docket pack {slug}`.",
        wanted.len()
    );
    println!(
        "This tool does not send mail: sending is an outward-facing action with its \
              own credentials and audit trail."
    );
    Ok(if incomplete {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

/// Show the two addresses a person should be provisioned.
///
/// DERIVED AND PRINTED, NOT WRITTEN. It does not edit the record, because provisioning is a
/// thing an operator does in Google's console and the record should say what was actually
/// created rather than what was intended. Paste the two lines back once the accounts exist.
fn cmd_addresses(paths: &Paths, slug: &str, suffix: Option<u32>) -> Result<ExitCode> {
    let person = store::person(paths, slug)?;
    let record = personnel::identify(paths, slug)?;
    let Some(name) = person.string("identity.legal_name") else {
        bail!("{slug}: identity.legal_name is unset; an address is derived from it");
    };
    if name.contains(person::UNFILLED) {
        bail!(
            "{slug}: identity.legal_name is still the template placeholder. An address may \
             never be renamed once issued, so it is not derived from a name nobody has checked."
        );
    }

    let a = address::derive(name, &record.person_id, suffix);
    println!("{slug}  {}  {name}", record.person_id.as_str());
    println!();
    println!("  published  {}", a.published);
    println!("             what they give out, what goes on a document, what a commit is");
    println!("             authored from. Create as an alias delivering to the mailbox below.");
    println!();
    println!("  mailbox    {}", a.mailbox);
    println!("             where mail lands. NOT given out. Create this Gmail account first.");
    println!();
    println!("Then record both in people/{slug}.yaml:");
    println!("  access.company_email: {}", a.published);
    println!("  access.mailbox_email: {}", a.mailbox);
    Ok(ExitCode::SUCCESS)
}

/// Print the published team as JSON.
///
/// STDOUT, so it is redirected into the web repository and committed there. The two
/// repositories have no shared build (ADR-0001) and `people/` is gitignored, so a public
/// page cannot read this at build time without its content depending on which machine
/// built it. Exporting makes the published set a reviewed diff instead.
fn cmd_team(registry: &Registry) -> Result<ExitCode> {
    print!("{}", tool_docket::application::team::export_json(registry)?);
    Ok(ExitCode::SUCCESS)
}

fn cmd_index(paths: &Paths, registry: &Registry) -> Result<ExitCode> {
    let w = archive::index(paths, registry)?;
    for f in &w.files {
        println!("  {}", paths.store.join(f).display());
    }
    println!("\n{} people, {} documents indexed.", w.people, w.documents);
    println!("The index is generated output: regenerate it, never edit it.");
    Ok(ExitCode::SUCCESS)
}

fn cmd_retention(paths: &Paths, registry: &Registry, date: Option<&str>) -> Result<ExitCode> {
    let (due, unevaluated) = archive::retention_due(paths, registry, resolve_date(date)?)?;
    for u in &unevaluated {
        println!("unevaluated  {u}");
    }
    if due.is_empty() {
        println!("\nnothing past its retention period");
        return Ok(ExitCode::SUCCESS);
    }
    for d in &due {
        println!(
            "OVERDUE  {}  {}  class {} keeps it {} -- {} {} ({} days ago)",
            d.serial, d.person_id, d.class, d.keep_for, d.basis, d.since, d.days
        );
    }
    println!(
        "\n{} document(s) past retention. Deleting them is a decision, not \
              something this tool does for you.",
        due.len()
    );
    Ok(ExitCode::from(1))
}

fn cmd_check(paths: &Paths, registry: &Registry) -> Result<ExitCode> {
    let findings = check::run(paths, registry)?;
    for f in &findings.items {
        if f.blocking {
            eprintln!("ERROR  {}", f.message);
        } else {
            println!("warn   {}", f.message);
        }
    }
    println!(
        "\n{} documents, {} stages, {} register entries, {} errors, {} warnings",
        findings.documents,
        findings.stages,
        findings.entries,
        findings.errors(),
        findings.warnings()
    );
    Ok(if findings.errors() == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn cmd_new(
    interactive: bool,
    paths: &Paths,
    registry: &Registry,
    slug: &str,
    class: &str,
    date: Option<&str>,
    operator: Option<&str>,
) -> Result<ExitCode> {
    let by = writer(operator)?;

    // Interactive writes the record from the answers; the default copies the
    // template. Both paths stay (ADR-0054): the template is what a non-tty
    // caller and every existing script uses.
    if interactive {
        let stdin = std::io::stdin();
        let mut input = stdin.lock();
        let mut out = std::io::stdout();
        let answers = enrol::interview(registry, class, &mut out, &mut input)?;
        let body = enrol::to_yaml(registry, class, &answers);
        enrol::write_record(paths, slug, &body)?;
    }

    let record = personnel::create(paths, registry, slug, class, resolve_date(date)?, by)?;
    println!(
        "{}  {}  class {}",
        record.person_id, record.slug, record.class
    );
    println!("record: {}", paths.person(slug).display());
    println!("\nFill it in, then: docket check && docket onboard {slug}");
    Ok(ExitCode::SUCCESS)
}

fn cmd_reengage(
    paths: &Paths,
    registry: &Registry,
    slug: &str,
    class: &str,
    date: Option<&str>,
    operator: Option<&str>,
) -> Result<ExitCode> {
    let by = writer(operator)?;
    let r = personnel::reengage(paths, registry, slug, class, resolve_date(date)?, by)?;
    println!(
        "person     {}  (unchanged -- the archive stays whole)",
        r.person_id
    );
    println!("engagement {}  class {}", r.engagement_id, r.class);
    println!("\nDocuments owed by the new class: docket status {slug}");
    Ok(ExitCode::SUCCESS)
}

fn cmd_record(paths: &Paths, registry: &Registry, slug: &str) -> Result<ExitCode> {
    let d = personnel::dossier(paths, registry, slug)?;
    println!("{}  ({})", d.identity.person_id, d.identity.slug);
    println!("engagement {}", d.identity.engagement_id);
    println!("class      {}", d.identity.class);
    println!(
        "standing   {}  since {}",
        d.identity.standing.as_str(),
        d.identity.standing_changed_on
    );
    println!("created    {}", d.identity.created_on);

    println!("\ndocuments ({})", d.documents.len());
    for (code, serial, on) in &d.documents {
        println!("  {code}  {serial:<22} {on}");
    }

    println!("\naccess still open ({})", d.open_access.len());
    for a in &d.open_access {
        println!("  {a}");
    }

    println!("\nhistory ({})", d.history.len());
    for (at, kind, detail) in &d.history {
        println!("  {at}  {kind:<20} {}", detail.as_deref().unwrap_or(""));
    }

    if d.violations.is_empty() {
        println!(
            "\nrecord matches what class \"{}\" declares",
            d.identity.class
        );
        return Ok(ExitCode::SUCCESS);
    }
    eprintln!("\n{} problems with the record:", d.violations.len());
    for v in &d.violations {
        eprintln!("  {v}");
    }
    Ok(ExitCode::from(1))
}

fn cmd_roster(paths: &Paths, registry: &Registry, standing: Option<&str>) -> Result<ExitCode> {
    let all = roster::build(paths, registry)?;
    let lines: Vec<&roster::Line> = all
        .iter()
        .filter(|l| standing.is_none_or(|s| l.standing.as_str() == s))
        .collect();

    println!(
        "{:<16} {:<18} {:<10} {:<10} {:<20} {:>4}",
        "PERSON", "ENGAGEMENT", "CLASS", "STANDING", "NAME", "DOCS"
    );
    for l in &lines {
        println!(
            "{:<16} {:<18} {:<10} {:<10} {:<20} {:>4}",
            l.person_id,
            l.engagement_id,
            l.class,
            l.standing.as_str(),
            l.name.as_deref().unwrap_or("--"),
            l.documents
        );
    }
    let (active, separated, erased) = roster::tally(&all);
    println!(
        "\n{} people: {active} active, {separated} separated, {erased} erased",
        all.len()
    );
    Ok(ExitCode::SUCCESS)
}

fn cmd_event(
    paths: &Paths,
    slug: &str,
    kind: &str,
    detail: Option<&str>,
    date: Option<&str>,
) -> Result<ExitCode> {
    let record = personnel::identify(paths, slug)?;
    if record.standing == Standing::Erased {
        bail!("{slug} is erased; no further events may be recorded against them");
    }
    let event = personnel::record_event(
        paths,
        &record.person_id,
        Kind::parse(kind)?,
        detail,
        "docket",
        resolve_date(date)?,
    )?;
    println!(
        "{}  {}  {}  {}",
        event.at,
        record.person_id,
        event.kind.as_str(),
        event.detail.as_deref().unwrap_or("")
    );
    Ok(ExitCode::SUCCESS)
}

fn cmd_separate(
    paths: &Paths,
    slug: &str,
    date: Option<&str>,
    operator: Option<&str>,
) -> Result<ExitCode> {
    let by = writer(operator)?;
    let record = personnel::separate(paths, slug, resolve_date(date)?, by)?;
    println!(
        "{} separated on {}",
        record.person_id, record.standing_changed_on
    );
    println!("Documents and NDA obligations survive. Offboarding documents are still owed.");
    Ok(ExitCode::SUCCESS)
}

fn cmd_erase(
    paths: &Paths,
    slug: &str,
    confirm: bool,
    date: Option<&str>,
    operator: Option<&str>,
) -> Result<ExitCode> {
    let by = writer(operator)?;
    if !confirm {
        eprintln!(
            "erasing {slug} deletes their record and the detail of their history.\n\
             The identifier, class and dates survive so issued serials still resolve.\n\
             This cannot be undone. Re-run with --confirm."
        );
        return Ok(ExitCode::from(2));
    }
    // Read the folder link BEFORE erasing: the record is about to be deleted.
    let folder = store::person(paths, slug)
        .ok()
        .and_then(|p| p.string("access.cloud_folder").map(ToOwned::to_owned));
    let record = personnel::erase(paths, slug, resolve_date(date)?, by)?;
    println!(
        "{} erased on {}",
        record.person_id, record.standing_changed_on
    );
    println!("Deleted:  the person record, and the detail of their event history.");
    println!("Retained: the identifier, class and dates, so issued serials still resolve.");
    println!("\nWHAT THIS DID NOT REACH -- each of these needs a person:");
    for item in archive::unreachable_copies(folder.as_deref(), record.person_id.as_str()) {
        println!("  - {item}");
    }
    Ok(ExitCode::SUCCESS)
}

/// Serve the operator API over loopback.
///
/// Read-only so far: every route calls an `application::` function the CLI also
/// calls, and none of them writes. Writes arrive with operator resolution, which
/// the design sequences before the first one — a write that cannot name who made
/// it is not auditable, and this process has no way to name one yet.
fn cmd_serve(paths: &Paths, registry: &Registry, port: u16) -> Result<ExitCode> {
    let clock = SystemClock;
    let ctx = http::router::Context {
        paths,
        registry,
        clock: &clock,
    };
    http::server::serve(&ctx, port)?;
    Ok(ExitCode::SUCCESS)
}
