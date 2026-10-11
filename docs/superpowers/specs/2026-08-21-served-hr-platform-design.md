# A served HR platform for `platform-docket`

**Status:** design, approved 2026-08-21. **Amended 2026-08-21** — see §0.
**Supersedes:** nothing. **Depends on:** ADR-0047, ADR-0054, ANTI-0004, C-0007.

## 0. Amendment — the interface moved out, so this serves JSON

This was designed as a server-rendered HTML interface inside `tool-docket`, on the
reasoning in §2 that a separate repository could only reach the domain by
extracting it into a `lib-`, which is ANTI-0004.

**§2 stands. §5, §6 and §7 do not.** After this was approved, the operator
interface became its own repository — `console-docket`, TypeScript and Next.js —
under ADR-0055, which added a `console` class for exactly this: an interface for
Paridyum's own operators, `deployable: false`, running where the operator is.

That does not reopen ANTI-0004, because nothing was extracted. No domain rule
moved. `console-docket` holds no rules at all; it asks this process. The boundary
is a wire, not a library, and a wire cannot be imported by a third consumer and
mistaken for a utility.

What changes here:

- **`serve` exposes JSON, not HTML.** Routes gain an `/api/` prefix. §7's mapping
  of route to application function is otherwise unchanged, which is the point: the
  same tested functions, one serialisation layer thinner.
- **§5's technology choice is superseded.** No `tiny_http`. Its last release was
  2022-10-06, and an unmaintained HTTP server is a poor dependency for a process
  that handles personal data. The needed surface is small enough to own: loopback
  only, one client, no TLS, no keep-alive requirement. `serde_json` is already a
  dependency, so payloads cost nothing new. This is §5's own argument against
  `axum` — dependency weight is a present cost — carried one step further, and it
  matches the zero-dependency TUI already in this crate.
- **`domain::escape::html` is not needed.** JSON is escaped by `serde_json`,
  which is a tested implementation of a specification rather than our own. The
  HTML escaping problem moves to `console-docket`, where React escapes by
  default. §9's first test row is therefore dropped.
- **CSRF changes shape.** There is no browser form posting to this process; the
  console is the only client. §5's CSRF token is replaced by an origin-independent
  scheme in §5a: writes require a header no cross-site form can set.

Everything in §4 (loopback only, SSH as authentication) and §8 (behaviour that
must not be softened) is unaffected and still binding.

## 1. What is being built

A graphical interface to `docket`, run as one served instance that several HR
people reach from their own machines. Production-ready, local, no cluster.

Started as `docket serve --port 8080`.

## 2. Why it is not a new repository

The interface needs the domain: identifiers, the lifecycle, four hash chains,
class-gated collection, retention epochs, the signature gate. A separate
repository can only reach those by extracting them into a `lib-`.

That is **ANTI-0004, business logic in a shared library.** Its `applies_to` is
`platform-*`, `svc-*`, `lib-*`, so a `tool-` repository is outside the letter of
it — and its argument lands squarely: *"`is_eligible_for_refund()` is a domain
rule wearing a utility's clothes, and no static analysis distinguishes them."*
Every rule in `docket`'s domain is that kind of rule. Duplicating them instead
is worse: two copies of the retention epoch is how one of them becomes wrong.

The layering already anticipated this. `src/adapters/` is *"everything that
touches the filesystem, the clock or an external process"*, and an HTTP server
is one more. `domain/` and `application/` do not change.

**Consequence: no new class, no `class-registry.yaml` edit, no ADR.** There is a
genuine gap in the class registry — `web` is *"served from the cluster"* with a
TypeScript stack, `tool` is *"developer-facing"* — and it stays open, because
nothing new is being classed. Recording it is enough; see §11.

## 3. Naming

    docket serve [--port 8080] [--open]

One binary, one repository. C-0007's rule — the binary name is the bare
capability — is unaffected: the binary is still `docket`. `serve` because it
names what the process does and reads correctly beside `check`, `issue`, `pack`.
Not `ui` or `gui`, which name a thing rather than an action.

## 4. Access, and why it is not blocked

One served instance needs authentication, and **ADR-0047 binds every
repository** (`org` scope, `applies_to: ["*"]`), so a bespoke login is not
available and Zitadel does not exist yet.

**The server binds `127.0.0.1` only. Remote access is an SSH port-forward.**

    ssh -L 8080:127.0.0.1:8080 hr-workstation

- SSH keys are the authentication; the operating system is the identity.
- **The application has no login of its own, so ADR-0047 does not engage.**
  There is no identity system here to be compliant or non-compliant.
- Personal data is encrypted in transit, by SSH rather than by us.
- `--by <operator>` stops being an unverified claim. The SSH user is verified,
  which is stronger than the CLI manages today.
- One process is the only writer. That is what makes the hash chains safe
  without file locking.

ADR-0047 point 1 already describes *"a separate `paridyum` instance in the home
region [that] holds staff and operators"* — precisely these users. When
`platform-identity` exists, OIDC becomes a **second** authentication adapter
beside loopback-only, added rather than swapped (ADR-0054). The interim is the
first implementation of a boundary designed for two, not a placeholder.

**Refusing to bind a routable interface** is part of the design, not a default.
If `--host` is ever added it must refuse anything but a loopback address until
an authentication adapter is present.

## 5. Technology

**`tiny_http`, blocking, with server-rendered HTML. No async runtime, no
JavaScript framework, no build step.**

ADR-0046 does not bind: its `applies_to` is `web-*`, `mobile-*`, `lib-*`, so
TypeScript and Next.js are not required here.

`axum` was rejected. It pulls `tokio`, `hyper` and `tower`; for three
concurrent users a thread-per-request blocking server is the more appropriate
shape rather than a compromise, and the machine this runs on has under 3 GB
free, which makes a large dependency tree a present cost rather than a
theoretical one.

Two things to build rather than depend on:

- **`domain::escape::html`**, beside the existing `domain::escape::tex`. Personal data goes into
  HTML; every interpolation is escaped at the boundary, the same rule the LaTeX
  side already follows for the same reason.
- **A CSRF token** on every form that writes. Loopback plus SSH lowers the risk
  and does not remove it: a page open in the operator's browser is reachable by
  script from any other page in that browser.

No client-side JavaScript is required for any screen. If a screen ever seems to
need it, that is a signal the screen is doing too much.

## 6. Directory structure

    src/adapters/http/
    ├── mod.rs        //! Serving the operator interface over loopback.
    ├── server.rs     listener, loopback-only bind, graceful shutdown
    ├── router.rs     method + path -> handler; no logic
    ├── request.rs    form parsing, CSRF check, operator resolution
    ├── response.rs   status, headers, redirect-after-POST
    └── page/
        ├── mod.rs
        ├── layout.rs one shell: navigation, flash messages, footer
        ├── roster.rs GET /
        ├── person.rs GET /person/{slug}
        ├── enrol.rs  GET + POST /new
        ├── actions.rs POST onboard | sign | advance | pack
        ├── files.rs  GET /document/{serial}, GET /pack/{slug}/{path}
        └── review.rs GET /check, /retention, /emails/{slug}

Every module states its single responsibility in a `//!` line (STD-0004). One
file per screen, because a screen is a responsibility and a single `pages.rs`
would be the file nobody can navigate.

`src/domain/escape.rs` gains `html()`. Nothing else outside `adapters/http/`
changes.

## 7. Screens, and what each one calls

| Route | Application function |
| --- | --- |
| `GET /` | `application::roster::build` |
| `GET /person/{slug}` | `application::personnel::dossier`, `application::next::steps` |
| `GET /new` | `domain::personnel::questions` |
| `POST /new` | `application::enrol::to_yaml`, `application::personnel::create` |
| `POST /person/{slug}/onboard` | `application::issue::outstanding`, `application::issue::one` |
| `POST /person/{slug}/sign` | `application::sign::record` |
| `POST /person/{slug}/advance` | `application::personnel::advance` |
| `GET /person/{slug}/pack` | `application::pack::assemble` |
| `GET /document/{serial}` | `adapters::store::register`, then the file |
| `GET /check` | `application::check::run` |
| `GET /retention` | `application::archive::retention_due` |
| `GET /emails/{slug}` | `application::comms::due` |

**Every route is an existing tested function.** The adapter adds parsing,
escaping and layout, and no rules.

The enrolment form is generated from **`domain::personnel::questions`** — the same
function that drives the CLI interview. The form and the prompts therefore
cannot diverge: a field added to `registry/fields.yaml` appears in both, with
its stated purpose as help text, and a field the class forbids appears in
neither.

## 8. Behaviour that must not be softened

- **Refusals stay refusals.** `application::issue::refusals` and `application::personnel::unmet_gates`
  render as a blocked action with the reason, never as a warning past which a
  button still works. The signature gate exists to stop access being granted
  before the NDA is back; a UI that offers an override defeats it.
- **`docket next` is the landing content** of a person's page. The ordering is
  the workflow, and it is the same ordering the CLI shows.
- **Every write records the operator** (`--by`, here the SSH user). A write with
  no operator is refused, as in the CLI.
- **Nothing writes to `registry/`.** Those are repository files, changed through
  review, not through a form.
- **No erasure through the UI.** `docket erase` prints what it cannot reach —
  cloud folders, backups, mail, printouts — and that list needs a person to read
  and act on. A button implying erasure was complete would be worse than no
  button.
- **No sending mail** (D-27). `/emails/{slug}` renders text to copy.

## 9. Testing

The router and pages are an adapter, so the rules are already covered. What is
new and needs its own tests:

| What | How |
| --- | --- |
| `domain::escape::html` | unit, including `<`, `>`, `&`, `"`, `'`, and a name containing a tag |
| CSRF | a POST without a token is refused; with a stale token is refused |
| loopback refusal | a non-loopback bind address is refused at startup |
| routing | method and path map to the intended handler; unknown paths 404 |
| form to record | `POST /new` produces the same YAML as the CLI interview for the same answers |
| refusals render | a record with an unmet gate shows the reason and no working button |
| no operator | a POST with no resolvable operator is refused |

The last three are the ones worth having: they are where a UI would silently
diverge from the CLI.

Rendering tests skip where no typesetter is installed (D-35), and that applies
here too: any test that issues a document through a route inherits it.

## 10. Sequencing

1. `domain::escape::html`, with tests. Nothing renders until escaping exists.
2. `server.rs` + `router.rs` + `layout.rs`, serving `GET /` from
   `application::roster::build`. Read-only, one screen, end to end.
3. `GET /person/{slug}` — dossier plus `application::next::steps`. Still read-only.
4. `GET /check`, `/retention`, `/emails/{slug}`. Still read-only.
5. CSRF and operator resolution. **Before the first write.**
6. `POST` actions: onboard, sign, advance, pack.
7. `GET /new` and `POST /new`, last, because it is the most complex form and
   benefits from everything above being settled.

Steps 1–4 are useful on their own and write nothing, so the platform is
deliverable in a state that cannot damage a record.

## 11. Recorded consequences

- **The class gap stays open.** `web` is cluster-served TypeScript, `tool` is
  developer-facing Rust, and an internal operator application is neither. Not
  closed here because nothing new is being classed; it becomes real the first
  time an operator interface needs its own repository.
- **Backup becomes everyone's dependency.** One machine holds the store. That
  is already true of the CLI; a served platform makes it a shared risk.
- **SSH access is all-or-nothing.** Anyone who can tunnel has full write
  access. Per-operator authorisation waits for `platform-identity`.
- **Disk.** Under 3 GB free on the machine this was designed on, and this adds
  a dependency tree plus rendered PDFs.

## 12. Out of scope

Editing registry files. Deleting documents. Sending mail. Erasure. Any
tenant-facing use — this is Tenant Zero internal tooling, and the moment tenant
traffic could reach it, C-0008 makes it a product and this design does not
apply.
