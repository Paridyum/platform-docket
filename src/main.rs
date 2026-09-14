//! Command-line entry point for issuing and verifying Paridyum documents.

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use tool_docket::adapters::clock::{self, Clock, SystemClock};
use tool_docket::adapters::http;
use tool_docket::adapters::lock;
use tool_docket::adapters::pg::{self, PgStore};
use tool_docket::adapters::store::{FileStore, Store};
use tool_docket::adapters::{registry::Registry, typeset};
use tool_docket::application::{
    archive, browse, check, comms, enrol, issue, next, node, pack, personnel, roster, sign, status,
    vault, verify,
};
use tool_docket::config::Paths;
use tool_docket::domain::address;
use tool_docket::domain::event::Kind;
use tool_docket::domain::identity::Standing;
use tool_docket::domain::person;
use tool_docket::domain::signature::Method;
use uuid::Uuid;

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
    /// Which company a command concerns, e.g. PCY.
    ///
    /// For `new` and `reengage` this is the company the engagement is opened
    /// at. For everything else it disambiguates a person who holds more than
    /// one open engagement -- and it is only ever needed for such a person,
    /// because that is the only case docket refuses to guess at.
    #[arg(long, global = true)]
    company: Option<String>,
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
    /// Encrypt a person's documents into the vault, or report what is not sealed.
    Vault {
        #[command(subcommand)]
        what: VaultCmd,
    },
    /// What this node holds, and how a cold spare is brought up (ADR-0072).
    Store {
        #[command(subcommand)]
        what: StoreCmd,
    },
    /// Show the two addresses a person should be provisioned.
    Addresses {
        /// Person slug.
        slug: String,
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

    // Printing help must not require a store: a database-backed CLI would
    // otherwise open a connection just to say what the commands are. So the
    // no-command, no-tui case is answered before the backend is built.
    let command = cli.command;
    if !cli.tui && command.is_none() {
        <Cli as clap::CommandFactory>::command().print_help()?;
        println!();
        return Ok(ExitCode::from(2));
    }

    // The store the whole command shares, built ONCE here and threaded down as a
    // `&dyn Store` (ADR-0093 §3): a `PgStore` against the cluster database when
    // `DOCKET_DATABASE_URL` is set, the offline file store otherwise. It says on
    // stderr which one, so an operator knows whether they are hitting the cluster.
    let backend = open_backend(&paths)?;

    // `--tui` wins, with or without a subcommand. It writes nothing, so it needs
    // no operator, but it reads through the same store as everything else.
    if cli.tui {
        let stdin = std::io::stdin();
        let mut input = stdin.lock();
        let mut out = std::io::stdout();
        browse::run(backend.as_store(), &paths, &registry, &mut out, &mut input)?;
        return Ok(ExitCode::SUCCESS);
    }
    let Some(command) = command else {
        // Unreachable: the no-command case returned above. Kept so the value is
        // a `Command` rather than an `Option` from here on.
        return Ok(ExitCode::from(2));
    };
    dispatch(
        command,
        &backend,
        &paths,
        &registry,
        operator.as_deref(),
        cli.company.as_deref(),
    )
}

/// docket's record store for one command: the file store, or Postgres.
///
/// [`FileStore`] borrows the paths; [`PgStore`] owns its pool and a handle onto
/// the process's tokio runtime. That runtime must outlive the store that drives
/// sqlx on it, so it is kept here for the whole command and dropped only with the
/// backend — the same lifetime `bin/serve` gives it. Fields drop in declaration
/// order, so the store (pool + handle) drops before the runtime.
enum Backend<'a> {
    File(FileStore<'a>),
    Pg {
        store: PgStore,
        _runtime: tokio::runtime::Runtime,
    },
}

impl Backend<'_> {
    /// The store behind whichever backend this is, as the shared port type.
    fn as_store(&self) -> &dyn Store {
        match self {
            Backend::File(s) => s,
            Backend::Pg { store, .. } => store,
        }
    }

    /// Whether this is the Postgres backend — the file write lock and the two
    /// filesystem-bound commands turn on this.
    const fn is_pg(&self) -> bool {
        matches!(self, Backend::Pg { .. })
    }
}

/// Build the store the whole command shares, choosing the backend by environment.
///
/// `DOCKET_DATABASE_URL` set makes the CLI a break-glass client of the SAME
/// Postgres database the served backend uses (ADR-0093 §3): every record read and
/// write goes to the cluster, not local files, so it says so on stderr. Unset is
/// the default offline path — the file store under `store/`, exactly as before.
fn open_backend(paths: &Paths) -> Result<Backend<'_>> {
    let url = std::env::var("DOCKET_DATABASE_URL")
        .ok()
        .filter(|v| !v.trim().is_empty());
    let Some(url) = url else {
        eprintln!("docket: store = local files at {}", paths.store.display());
        return Ok(Backend::File(FileStore::new(paths)));
    };

    // A database URL without a tenant is refused rather than defaulted: every
    // row is tenant-scoped by the RLS policies (migration 0006), and guessing the
    // tenant is exactly how one operator's break-glass write would land against
    // another tenant's records.
    let tenant_raw = std::env::var("DOCKET_TENANT_ID")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .context(
            "DOCKET_DATABASE_URL is set, so this is a break-glass client of the cluster \
             database and must be told which tenant: set DOCKET_TENANT_ID to a uuid",
        )?;
    let tenant: Uuid = tenant_raw
        .parse()
        .with_context(|| format!("DOCKET_TENANT_ID={tenant_raw:?} is not a uuid"))?;

    // A multi-thread runtime the command owns for its whole life. `PgStore` drives
    // sqlx to completion on this runtime's handle from the calling (blocking)
    // thread, so the runtime must outlive every call — bin/serve builds it the
    // same way, for the same reason.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("building the tokio runtime")?;
    let pool = runtime
        .block_on(pg::pool(&url))
        .context("opening the docket database (is DOCKET_DATABASE_URL reachable?)")?;
    eprintln!(
        "docket: store = Postgres at {} for tenant {tenant}; break-glass client of the \
         cluster, NOT local files (ADR-0093 §3)",
        db_host(&url)
    );
    let store = PgStore::new(pool, tenant, runtime.handle().clone());
    Ok(Backend::Pg {
        store,
        _runtime: runtime,
    })
}

/// The `host[:port]/database` of a Postgres URL, with any credentials removed.
///
/// For the one-line stderr banner only, and it must never echo the password: it
/// takes the authority AFTER the last `@` (or the whole authority when there are
/// no credentials) and stops at the query string. An unparseable URL degrades to
/// a placeholder rather than risking printing it verbatim.
fn db_host(url: &str) -> String {
    let Some((_, after_scheme)) = url.split_once("://") else {
        return "(the configured URL)".to_owned();
    };
    let host_part = after_scheme
        .rsplit_once('@')
        .map_or(after_scheme, |(_, host)| host);
    host_part.split('?').next().unwrap_or(host_part).to_owned()
}

/// Commands the break-glass CLI cannot yet run against the Postgres store, each
/// with the reason it is refused CLEARLY here rather than failing obscurely later.
///
/// Both writers still assume the filesystem. `advance` changes a stage by
/// rewriting `people/<slug>.yaml` as TEXT, to keep each field's DPDP purpose
/// comment (D-62) — on Postgres the record is in `docket_persons`, not on disk,
/// so it would fail deep in an `fs` read. `erase` re-seals the event log by
/// rewriting it and deletes the person's local blobs; the Postgres re-seal needs
/// migration 0007's delete exception (D-60), and refusing up front also avoids
/// deleting host-local PDFs before that re-seal fails. An erasure against a
/// Postgres instance is honoured by running `erase` offline on the file store.
fn unavailable_on_pg(command: &Command) -> Option<&'static str> {
    match command {
        Command::Advance { .. } => Some(
            "advance changes a stage by rewriting people/<slug>.yaml as text, to preserve each \
             field's DPDP purpose comment (D-62); on Postgres the record lives in the database, \
             not on disk. Change the stage from the served backend, or run advance offline \
             against the file store (unset DOCKET_DATABASE_URL).",
        ),
        Command::Erase { .. } => Some(
            "erase re-seals the event log by rewriting it, which the Postgres append-only trigger \
             forbids without migration 0007 (D-60). Honour this erasure by running `docket erase` \
             offline against the file store (unset DOCKET_DATABASE_URL), which re-seals correctly.",
        ),
        _ => None,
    }
}

/// Whether a command writes to the store, and so must hold the store lock.
///
/// ADR-0072 decision 2. Written as an exhaustive match rather than a list of
/// names, so that adding a command does not compile until someone has said
/// which side of this line it falls on. A default of `false` here is a silent
/// unlocked writer, which is the exact defect this exists to prevent.
fn writes(command: &Command) -> bool {
    match command {
        // Reads. None of these open a file for writing inside `store/`.
        Command::Next { .. }
        | Command::Status { .. }
        | Command::Render { .. }
        | Command::Verify { .. }
        | Command::Check
        | Command::Emails { .. }
        | Command::Addresses { .. }
        | Command::Retention { .. }
        | Command::Record { .. }
        | Command::Roster { .. }
        | Command::Pack { .. } => false,

        // `store status` reads; `store restore` writes, and writes into a
        // DIFFERENT root than the one being locked. Classified by subcommand
        // rather than wholesale, because locking the source store to restore
        // into a spare would refuse the one operation a spare exists for.
        Command::Store { what } => matches!(what, StoreCmd::Restore { .. } | StoreCmd::Push { .. }),

        // Writers. `Team` and `Index` are here because both record a receipt
        // beside their output; `Vault` because seal, shred, backup and escrow
        // all write, and refusing `vault status` during a write costs nothing
        // next to classifying a subcommand wrongly.
        Command::Onboard { .. }
        | Command::Issue { .. }
        | Command::Sign { .. }
        | Command::Advance { .. }
        | Command::Index
        | Command::Team
        | Command::Vault { .. }
        | Command::New { .. }
        | Command::Reengage { .. }
        | Command::Event { .. }
        | Command::Separate { .. }
        | Command::Erase { .. }
        // `Serve` holds the lock for its whole lifetime, which is what makes a
        // CLI write during `serve` refuse instead of tearing a chain. It shares
        // this arm rather than getting its own: two arms with one body are one
        // arm, and clippy is right to say so.
        | Command::Serve { .. } => true,
    }
}

/// Route one command. Split from `run` so the routing table can grow without
/// the entry point becoming the longest function in the crate.
fn dispatch(
    command: Command,
    backend: &Backend<'_>,
    paths: &Paths,
    registry: &Registry,
    operator: Option<&str>,
    company: Option<&str>,
) -> Result<ExitCode> {
    // Refuse a command that cannot work against Postgres before it does anything,
    // rather than letting it fail obscurely (or, for erase, delete local blobs
    // first). The file store runs every command, so this gate is pg-only.
    if backend.is_pg()
        && let Some(reason) = unavailable_on_pg(&command)
    {
        bail!("{reason}");
    }

    // The file store's single-writer guard. NOT taken on Postgres: the database's
    // advisory-locked, atomic seal-append is the concurrency control there
    // (ADR-0093 §decision-2), and a lockfile beside a database store would serialise
    // nothing useful while leaving a stray file. Held for the whole command and
    // released on drop; bound to a name rather than `_`, which would drop it at once
    // and unlock the store before the first write.
    let _lock = if writes(&command) && !backend.is_pg() {
        Some(lock::acquire(paths, operator)?)
    } else {
        None
    };

    route(
        command,
        backend.as_store(),
        paths,
        registry,
        operator,
        company,
    )
}

/// The routing table itself, split from the locking so neither grows into the
/// other. `dispatch` decides whether a command may write; this decides where it
/// goes.
fn route(
    command: Command,
    store: &dyn Store,
    paths: &Paths,
    registry: &Registry,
    operator: Option<&str>,
    company: Option<&str>,
) -> Result<ExitCode> {
    match command {
        Command::Next { slug } => cmd_next(store, paths, registry, &slug, company),
        Command::Status { slug } => cmd_status(store, registry, &slug, company),
        Command::Onboard { slug, date, force } => cmd_onboard(
            store,
            paths,
            registry,
            &slug,
            company,
            date.as_deref(),
            force,
            operator,
        ),
        Command::Issue {
            slug,
            code,
            date,
            force,
        } => cmd_issue(
            store,
            paths,
            registry,
            &slug,
            &code,
            date.as_deref(),
            force,
            operator,
        ),
        Command::Render { slug, code } => cmd_render(store, paths, registry, &slug, &code),
        Command::Verify { serial, file } => cmd_verify(store, paths, &serial, file.as_deref()),
        Command::Check => cmd_check(store, paths, registry),
        Command::Sign {
            serial,
            method,
            on,
            received,
            scan,
        } => cmd_sign(
            store,
            paths,
            registry,
            &serial,
            &method,
            on.as_deref(),
            received.as_deref(),
            scan.as_deref(),
            operator,
        ),
        Command::Advance { slug, force, date } => cmd_advance(
            store,
            paths,
            registry,
            &slug,
            force,
            date.as_deref(),
            operator,
        ),
        Command::Pack { slug, out, date } => cmd_pack(
            store,
            paths,
            registry,
            &slug,
            company,
            out.as_deref(),
            date.as_deref(),
            operator,
        ),
        Command::Emails { slug, only } => {
            cmd_emails(store, registry, &slug, company, only.as_deref())
        }
        Command::Index => cmd_index(store, paths, registry),
        Command::Team => cmd_team(paths, registry),
        Command::Addresses { slug } => cmd_addresses(store, &slug, company),
        Command::Vault { what } => cmd_vault(store, paths, &what),
        Command::Store { what } => cmd_store(paths, &what),
        Command::Retention { date } => cmd_retention(store, registry, date.as_deref()),
        other => route_person(other, store, paths, registry, operator, company),
    }
}

/// The half of the routing table that concerns a person's record: opening one,
/// re-engaging, and the events that move them through their lifecycle.
///
/// Split from `route` because the table outgrew clippy's line limit, and the
/// seam is the one that was already there -- everything above reads or issues,
/// everything here changes who is on the register.
fn route_person(
    command: Command,
    store: &dyn Store,
    paths: &Paths,
    registry: &Registry,
    operator: Option<&str>,
    company: Option<&str>,
) -> Result<ExitCode> {
    match command {
        Command::New {
            slug,
            class,
            date,
            interactive,
        } => cmd_new(
            store,
            interactive,
            paths,
            registry,
            &slug,
            &class,
            company,
            date.as_deref(),
            operator,
        ),
        Command::Reengage { slug, class, date } => cmd_reengage(
            store,
            registry,
            &slug,
            &class,
            company,
            date.as_deref(),
            operator,
        ),
        Command::Record { slug } => cmd_record(store, registry, &slug, company),
        Command::Serve { port } => cmd_serve(store, paths, registry, port),
        Command::Roster { standing } => cmd_roster(store, registry, standing.as_deref()),
        Command::Event {
            slug,
            kind,
            detail,
            date,
        } => cmd_event(
            store,
            &slug,
            company,
            &kind,
            detail.as_deref(),
            date.as_deref(),
        ),
        Command::Separate { slug, date } => cmd_separate(store, &slug, date.as_deref(), operator),
        Command::Erase {
            slug,
            confirm,
            date,
        } => cmd_erase(store, paths, &slug, confirm, date.as_deref(), operator),
        // `route` handles every other variant and delegates here; a command
        // reaching this arm means the split above lost one.
        other => unreachable_command(&other),
    }
}

/// A command that `route` did not handle and `route_person` does not own.
///
/// The cost of splitting one exhaustive match in two: the compiler can no
/// longer prove the halves cover the enum between them. Refusing loudly beats
/// a silent success, and names the command so the gap is one line to close.
fn unreachable_command(_command: &Command) -> Result<ExitCode> {
    bail!("internal: a command reached no route arm; the routing table has a gap")
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

fn cmd_next(
    store: &dyn Store,
    paths: &Paths,
    registry: &Registry,
    slug: &str,
    company: Option<&str>,
) -> Result<ExitCode> {
    let steps = next::steps(store, paths, registry, slug, company)?;
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

fn cmd_status(
    store: &dyn Store,
    registry: &Registry,
    slug: &str,
    company: Option<&str>,
) -> Result<ExitCode> {
    let person = store.person(slug)?;
    let who = personnel::identify_at(store, slug, company)?;
    let register = store.register()?;
    let signatures = store.signatures()?;
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
    store: &dyn Store,
    paths: &Paths,
    registry: &Registry,
    slug: &str,
    company: Option<&str>,
    date: Option<&str>,
    force: bool,
    operator: Option<&str>,
) -> Result<ExitCode> {
    let by = writer(operator)?;
    let person = store.person(slug)?;
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

    let who = personnel::identify_at(store, slug, company)?;
    let register = store.register()?;
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
        let entry = issue::one(store, paths, registry, &person, doc, when, by)?;
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
    store: &dyn Store,
    paths: &Paths,
    registry: &Registry,
    slug: &str,
    code: &str,
    date: Option<&str>,
    force: bool,
    operator: Option<&str>,
) -> Result<ExitCode> {
    let by = writer(operator)?;
    let person = store.person(slug)?;
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
    let entry = issue::one(
        store,
        paths,
        registry,
        &person,
        doc,
        resolve_date(date)?,
        by,
    )?;
    println!("{code}  {}  {}", entry.serial, entry.pdf_path);
    Ok(ExitCode::SUCCESS)
}

fn cmd_render(
    store: &dyn Store,
    paths: &Paths,
    registry: &Registry,
    slug: &str,
    code: &str,
) -> Result<ExitCode> {
    let person = store.person(slug)?;
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

fn cmd_verify(
    store: &dyn Store,
    paths: &Paths,
    serial: &str,
    file: Option<&Path>,
) -> Result<ExitCode> {
    let register = store.register()?;
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

// A dispatch handler: its arguments are the shared store and paths plus the
// fields clap parsed from `Command::Sign`. Grouping them into a struct only to
// satisfy the count would hide the one-to-one correspondence with the command.
#[allow(clippy::too_many_arguments)]
fn cmd_sign(
    store: &dyn Store,
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
        store,
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
    store: &dyn Store,
    paths: &Paths,
    registry: &Registry,
    slug: &str,
    force: bool,
    date: Option<&str>,
    operator: Option<&str>,
) -> Result<ExitCode> {
    let by = writer(operator)?;
    let (from, to, gates) =
        personnel::advance(store, paths, registry, slug, resolve_date(date)?, force, by)?;
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
    store: &dyn Store,
    paths: &Paths,
    registry: &Registry,
    slug: &str,
    company: Option<&str>,
    out: Option<&Path>,
    date: Option<&str>,
    operator: Option<&str>,
) -> Result<ExitCode> {
    let by = writer(operator)?;
    let target = out.map_or_else(|| pack::default_out(paths), Path::to_path_buf);
    let p = pack::assemble(
        store,
        paths,
        registry,
        slug,
        company,
        &target,
        resolve_date(date)?,
        by,
    )?;
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
    store: &dyn Store,
    registry: &Registry,
    slug: &str,
    company: Option<&str>,
    only: Option<&str>,
) -> Result<ExitCode> {
    let all = comms::due(store, registry, slug, company)?;
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
fn cmd_addresses(store: &dyn Store, slug: &str, company: Option<&str>) -> Result<ExitCode> {
    let person = store.person(slug)?;
    let record = personnel::identify_at(store, slug, company)?;
    let Some(name) = person.string("identity.legal_name") else {
        bail!("{slug}: identity.legal_name is unset; an address is derived from it");
    };
    if name.contains(person::UNFILLED) {
        bail!(
            "{slug}: identity.legal_name is still the template placeholder. An address may \
             never be renamed once issued, so it is not derived from a name nobody has checked."
        );
    }

    let a = address::derive(name, &record.person_id);
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

/// What to do with the node's store (ADR-0072).
#[derive(clap::Subcommand)]
enum StoreCmd {
    /// Report what this node holds and what would be lost if it were lost.
    Status,
    /// Copy the uploadable half of the store into the synced folder, guarded.
    Push {
        /// The synced folder, e.g. ~/Sync/paridyum-docket. Must be outside the store.
        to: PathBuf,
    },
    /// Rebuild a store on a cold spare from a register bundle and the estate key.
    Restore {
        /// The `_register` bundle directory, as synced down from the off-host store.
        from: PathBuf,
        /// Root to restore into. Must not already hold a register.
        into: PathBuf,
        /// The estate age identity. Defaults to the one RUN-0004 installs.
        #[arg(long)]
        identity: Option<PathBuf>,
    },
}

/// What to do with the vault.
#[derive(clap::Subcommand)]
enum VaultCmd {
    /// Encrypt everything issued to a person that is not already sealed.
    Seal {
        /// Person slug.
        slug: String,
    },
    /// Report anything issued and not yet sealed.
    Status,
    /// Print what to upload, and what must never be uploaded.
    Sync,
    /// List the shreddings on record.
    Tombstones,
    /// Encrypt the register and its companions into the uploaded path.
    Backup,
    /// Export every per-person key for offline media. Weakens shredding; see the output.
    Escrow {
        /// Where to write it. Must be outside the store, which is what gets uploaded.
        to: PathBuf,
    },
}

/// `docket store` — what this node holds, and bringing up a cold spare.
fn cmd_store(paths: &Paths, what: &StoreCmd) -> Result<ExitCode> {
    match what {
        StoreCmd::Status => {
            let items = node::classify(paths)?;
            if items.is_empty() {
                println!("the store is empty; nothing is at risk on this machine yet");
                return Ok(ExitCode::SUCCESS);
            }
            for item in &items {
                println!(
                    "{:<12}  {:>9}  {}",
                    item.disposition.as_str(),
                    item.bytes,
                    item.name
                );
            }

            let unclassified: Vec<_> = items
                .iter()
                .filter(|i| i.disposition == node::Disposition::Unclassified)
                .map(|i| i.name.clone())
                .collect();
            let lost = node::unreplicated(paths)?;

            println!();
            if !unclassified.is_empty() {
                // Louder than the rest, because an unclassified path is the one
                // failure mode `vault sync` structurally cannot report: it is in
                // neither the upload list nor the prohibition.
                println!(
                    "UNCLASSIFIED: {}\n  Nobody has said whether these replicate. Add each to \
                     node::declared() -- ADR-0072 decision 5 -- before the next backup.",
                    unclassified.join(", ")
                );
            }
            if lost.is_empty() {
                println!("every path that must be replicated has a bundle behind it");
            } else {
                println!(
                    "LOST IF THIS MACHINE IS LOST: {}\n  Run `docket vault backup`, then copy \
                     {} to the off-host store (ADR-0071).",
                    lost.join(", "),
                    node::bundle_path(paths).display()
                );
                return Ok(ExitCode::from(1));
            }
            if !unclassified.is_empty() {
                return Ok(ExitCode::from(1));
            }
            Ok(ExitCode::SUCCESS)
        }
        StoreCmd::Push { to } => {
            let pushed = node::push(paths, to)?;
            let moved = pushed.iter().filter(|p| !p.already_current).count();
            for item in &pushed {
                if item.already_current {
                    println!("current   {}", item.name);
                } else {
                    println!("uploaded  {}", item.name);
                }
            }
            println!(
                "\n{moved} file(s) copied, {} already current, into {}",
                pushed.len() - moved,
                to.display()
            );
            // The vendor's client does the network leg; this only fills the folder
            // it watches. Said plainly, because "uploaded" above means "written to
            // disk" and somebody will otherwise read it as "arrived".
            println!("The Sync client uploads from there. Confirm it has finished before");
            println!("treating this as a backup -- ADR-0071 §3.");
            Ok(ExitCode::SUCCESS)
        }
        StoreCmd::Restore {
            from,
            into,
            identity,
        } => {
            let identity = match identity {
                Some(p) => p.clone(),
                None => tool_docket::adapters::crypt::estate_identity()?,
            };
            let target = Paths::resolve(into);
            let restored = node::restore(from, &identity, &target)?;
            for name in &restored {
                println!("restored  {name}");
            }
            // Verified rather than announced. ADR-0071 §3: a backup that has
            // never been restored is a belief, and a restore that was never
            // checked is the same belief one step later. The restored chains are
            // read back through the file store rooted at the restore target.
            let restored_store = FileStore::new(&target);
            let faults = node::verify_restored(&restored_store)?;
            if faults.is_empty() {
                println!(
                    "\n{} file(s) restored; every chain verifies",
                    restored.len()
                );
                Ok(ExitCode::SUCCESS)
            } else {
                for fault in &faults {
                    println!("FAULT  {fault}");
                }
                println!("\nthe restore completed and does NOT verify; do not promote this node");
                Ok(ExitCode::from(1))
            }
        }
    }
}

fn cmd_vault(store: &dyn Store, paths: &Paths, what: &VaultCmd) -> Result<ExitCode> {
    match what {
        VaultCmd::Seal { slug } => {
            // Resolved here, at the one place that knows a key is needed, rather than deep
            // in the vault where a test would have to fake an environment to reach it.
            let estate = tool_docket::adapters::crypt::estate_identity()?;
            // `vault seal` has no company of its own: it seals everything issued to a
            // PERSON, and a person is one archive across companies (ADR-0077 D2).
            let out = vault::seal(store, paths, slug, None, &estate)?;
            for name in &out.written {
                println!("  sealed   {name}");
            }
            println!(
                "{} sealed, {} already present",
                out.written.len(),
                out.skipped
            );
            Ok(ExitCode::SUCCESS)
        }
        VaultCmd::Status => {
            let pending = vault::unsealed(store, paths)?;
            if pending.is_empty() {
                println!("every issued document is sealed");
                return Ok(ExitCode::SUCCESS);
            }
            for serial in &pending {
                println!("  unsealed  {serial}");
            }
            println!(
                "\n{} document(s) not in the vault: run `docket vault seal <slug>`",
                pending.len()
            );
            Ok(ExitCode::FAILURE)
        }
        VaultCmd::Sync => {
            let (upload, never) = vault::syncable(paths);
            println!("upload:");
            println!("  {}", upload.display());
            println!("\nNEVER upload — a versioned store retains what you delete from it, so a");
            println!("key inside one cannot be destroyed, and destroying it is what erasure is:");
            for path in never {
                println!("  {}", path.display());
            }
            println!("\nThe generated index is not uploaded either: encrypting the documents and");
            println!("syncing a roster that maps identifier to name defeats the opaque naming.");
            Ok(ExitCode::SUCCESS)
        }
        VaultCmd::Backup => {
            let estate = tool_docket::adapters::crypt::estate_identity()?;
            for name in vault::back_up_register(paths, &estate)? {
                println!("  backed up  {name}");
            }
            println!("\nEncrypted to the estate key, not a person's: this is the half that is");
            println!("never erased, which is also what makes it safe in a versioned store.");
            Ok(ExitCode::SUCCESS)
        }
        VaultCmd::Escrow { to } => {
            let estate = tool_docket::adapters::crypt::estate_identity()?;
            let count = vault::escrow(paths, &estate, to)?;
            println!("{count} key(s) written to {}", to.display());
            println!();
            println!("READ THIS BEFORE FILING IT. Crypto-shredding needs key material that can");
            println!("genuinely be destroyed, so this copy improves durability by exactly as much");
            println!("as it weakens erasure. With it, a lost laptop no longer loses every vault.");
            println!("Also with it, `docket erase` destroys docket's copy of a key and cannot");
            println!("reach this one — erasure becomes ASSERTED rather than proven.");
            println!();
            println!("So: offline media you physically control, and after any erasure either");
            println!("destroy this medium or re-export and destroy the old one. `docket check`");
            println!("will remind you, because it is the one gap nothing else reports.");
            Ok(ExitCode::SUCCESS)
        }
        VaultCmd::Tombstones => {
            let stones = vault::tombstones(paths)?;
            if stones.is_empty() {
                println!("no shreddings on record");
                return Ok(ExitCode::SUCCESS);
            }
            for t in &stones {
                println!(
                    "  {}  {}  by {}  {} object(s)  key {}",
                    t.shredded_on,
                    t.person_id,
                    t.by,
                    t.objects,
                    if t.key_destroyed {
                        "destroyed"
                    } else {
                        "already absent"
                    }
                );
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

/// Print the published team as JSON.
///
/// STDOUT, so it is redirected into the web repository and committed there. The two
/// repositories have no shared build (ADR-0001) and `people/` is gitignored, so a public
/// page cannot read this at build time without its content depending on which machine
/// built it. Exporting makes the published set a reviewed diff instead.
fn cmd_team(paths: &Paths, registry: &Registry) -> Result<ExitCode> {
    print!("{}", tool_docket::application::team::export_json(registry)?);
    // The receipt is written AFTER the export is emitted, and records what it was derived
    // from. `docket check` compares it, so editing signatories.yaml and forgetting to
    // re-export becomes a warning here rather than a stale public page nobody notices (D-48).
    let today = clock::to_iso(SystemClock.today()?)?;
    tool_docket::application::team::record_export(paths, registry, &today)?;
    Ok(ExitCode::SUCCESS)
}

fn cmd_index(store: &dyn Store, paths: &Paths, registry: &Registry) -> Result<ExitCode> {
    let w = archive::index(store, paths, registry)?;
    for f in &w.files {
        println!("  {}", paths.store.join(f).display());
    }
    println!("\n{} people, {} documents indexed.", w.people, w.documents);
    println!("The index is generated output: regenerate it, never edit it.");
    Ok(ExitCode::SUCCESS)
}

fn cmd_retention(store: &dyn Store, registry: &Registry, date: Option<&str>) -> Result<ExitCode> {
    let (due, unevaluated) = archive::retention_due(store, registry, resolve_date(date)?)?;
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

fn cmd_check(store: &dyn Store, paths: &Paths, registry: &Registry) -> Result<ExitCode> {
    let findings = check::run(store, paths, registry)?;
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

/// Resolve a `--company` flag against the registry, or fall back to the default.
///
/// An unknown code is refused with the declared list rather than silently
/// creating a fifth company out of a typo -- the engagement identifier would be
/// permanent, and `PCU-OFC-2026-0001` is not correctable after issuance.
fn resolve_company(registry: &Registry, given: Option<&str>) -> Result<String> {
    let declared = registry.company_codes();
    match given {
        Some(code) => {
            if declared.contains(&code) {
                Ok(code.to_owned())
            } else {
                bail!(
                    "unknown company {code:?}; declared in registry/org.yaml: {}",
                    declared.join(", ")
                )
            }
        }
        None => registry
            .default_company()
            .map(str::to_owned)
            .context("registry/org.yaml declares no companies"),
    }
}

// A dispatch handler: the shared store and paths plus the fields clap parsed from
// `Command::New`. See `cmd_sign` for why the count is left as it is.
#[allow(clippy::too_many_arguments)]
fn cmd_new(
    store: &dyn Store,
    interactive: bool,
    paths: &Paths,
    registry: &Registry,
    slug: &str,
    class: &str,
    company: Option<&str>,
    date: Option<&str>,
    operator: Option<&str>,
) -> Result<ExitCode> {
    let by = writer(operator)?;
    let company = resolve_company(registry, company)?;

    // Interactive writes the record from the answers; the default copies the
    // template. Both paths stay (ADR-0054): the template is what a non-tty
    // caller and every existing script uses.
    if interactive {
        let stdin = std::io::stdin();
        let mut input = stdin.lock();
        let mut out = std::io::stdout();
        let answers = enrol::interview(registry, class, &mut out, &mut input)?;
        let body = enrol::to_yaml(registry, class, &answers);
        enrol::write_record(store, slug, &body)?;
    }

    let record = personnel::create(
        store,
        paths,
        registry,
        slug,
        class,
        &company,
        resolve_date(date)?,
        by,
    )?;
    println!(
        "{}  {}  class {}",
        record.person_id, record.slug, record.class
    );
    println!("record: {}", paths.person(slug).display());
    println!("\nFill it in, then: docket check && docket onboard {slug}");
    Ok(ExitCode::SUCCESS)
}

fn cmd_reengage(
    store: &dyn Store,
    registry: &Registry,
    slug: &str,
    class: &str,
    company: Option<&str>,
    date: Option<&str>,
    operator: Option<&str>,
) -> Result<ExitCode> {
    let by = writer(operator)?;
    let company = resolve_company(registry, company)?;
    let r = personnel::reengage(
        store,
        registry,
        slug,
        class,
        &company,
        resolve_date(date)?,
        by,
    )?;
    println!(
        "person     {}  (unchanged -- the archive stays whole)",
        r.person_id
    );
    println!("engagement {}  class {}", r.engagement_id, r.class);
    println!("\nDocuments owed by the new class: docket status {slug}");
    Ok(ExitCode::SUCCESS)
}

fn cmd_record(
    store: &dyn Store,
    registry: &Registry,
    slug: &str,
    company: Option<&str>,
) -> Result<ExitCode> {
    let d = personnel::dossier(store, registry, slug, company)?;
    println!("{}  ({})", d.identity.person_id, d.identity.slug);
    // EVERY engagement, not the latest. `record` is the answer to "what is held
    // about this person", and one line showing only their most recent company
    // was how Rohan's Paridyum office disappeared from his own record.
    let held = personnel::engagements(store, slug)?;
    if held.len() > 1 {
        println!("engagements ({})", held.len());
        for e in &held {
            println!(
                "  {:<20} {:<10} {}",
                e.engagement_id.as_str(),
                e.class,
                e.standing.as_str()
            );
        }
    } else {
        println!("engagement {}", d.identity.engagement_id);
    }
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

fn cmd_roster(store: &dyn Store, registry: &Registry, standing: Option<&str>) -> Result<ExitCode> {
    let all = roster::build(store, registry)?;
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
    // PEOPLE AND ENGAGEMENTS ARE COUNTED SEPARATELY (ADR-0077 D3). A row is an
    // engagement, and one person may hold two at once -- so `all.len()` is the
    // number of offices held, and calling it "people" made four officers read
    // as five the moment Rohan took a second one.
    let people: std::collections::BTreeSet<_> = all.iter().map(|l| l.person_id.clone()).collect();
    if people.len() == all.len() {
        println!(
            "\n{} people: {active} active, {separated} separated, {erased} erased",
            all.len()
        );
    } else {
        println!(
            "\n{} people holding {} engagements: {active} active, {separated} separated, \
             {erased} erased",
            people.len(),
            all.len()
        );
    }
    Ok(ExitCode::SUCCESS)
}

fn cmd_event(
    store: &dyn Store,
    slug: &str,
    company: Option<&str>,
    kind: &str,
    detail: Option<&str>,
    date: Option<&str>,
) -> Result<ExitCode> {
    let record = personnel::identify_at(store, slug, company)?;
    if record.standing == Standing::Erased {
        bail!("{slug} is erased; no further events may be recorded against them");
    }
    let event = personnel::record_event(
        store,
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
    store: &dyn Store,
    slug: &str,
    date: Option<&str>,
    operator: Option<&str>,
) -> Result<ExitCode> {
    let by = writer(operator)?;
    let record = personnel::separate(store, slug, resolve_date(date)?, by)?;
    println!(
        "{} separated on {}",
        record.person_id, record.standing_changed_on
    );
    println!("Documents and NDA obligations survive. Offboarding documents are still owed.");
    Ok(ExitCode::SUCCESS)
}

fn cmd_erase(
    store: &dyn Store,
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
    let folder = store
        .person(slug)
        .ok()
        .and_then(|p| p.string("access.cloud_folder").map(ToOwned::to_owned));
    let record = personnel::erase(store, paths, slug, resolve_date(date)?, by)?;
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

/// Serve the operator API over loopback (break-glass; the primary served backend is `bin/serve`).
///
/// No token verifier, so [`http::server::serve`] refuses anything but a loopback bind: SSH keys are
/// the authentication and the operating system is the identity (ADR-0054). The routable, token-
/// authenticated backend is `bin/serve` (ADR-0093). Writes are attributed from the `X-Docket-Operator`
/// header here because there is no verifier to bind a token to (design §5).
fn cmd_serve(store: &dyn Store, paths: &Paths, registry: &Registry, port: u16) -> Result<ExitCode> {
    let clock = SystemClock;
    let ctx = http::router::Context {
        store,
        paths,
        registry,
        verifier: None,
        clock: &clock,
    };
    http::server::serve(&ctx, SocketAddr::from(([127, 0, 0, 1], port)))?;
    Ok(ExitCode::SUCCESS)
}
