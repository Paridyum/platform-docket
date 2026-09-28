# Logbook — tool-docket

How this tool got the way it is, recorded as it was built rather than
reconstructed afterwards (RFC-0033). The `CHANGELOG.md` says what changed;
this says what was decided, what was wrong on the way, and what it cost.

## 2026-09-28 — the staging values carry the rebuilt estate's ids

**The estate was rebuilt from empty** (ADR-0113 to ADR-0123), so every Zitadel id and the tenant
uuid changed. The live release named the old ones: `DOCKET_OIDC_AUDIENCE=390606436373103770` and
`DOCKET_TENANT_ID=68445ee9-…`. Both had been typed at `helm upgrade` time, because
`values-staging.yaml` held `SET_FROM_…` placeholders. The upshot was that docket refused every token.

**Placeholders in a committed values file were the mistake.** They made the real deploy
unreproducible from the repository, and let a stale id outlive the estate it came from. The file
now holds the ids themselves. The release also carried `service.type=LoadBalancer`, another hand
`--set` with no address-registry entry and no edge route. Deploying from the committed values
returns it to `ClusterIP`. When docket.paridyum.com is routed (GO-LIVE), the route goes through the
edge like every other host, not through a bare LoadBalancer.

**Still open:** the chart does not wire `DOCKET_S3_*` from the existing `docket-s3` Secret, so
served issuance stores documents on the pod's filesystem, not in Garage (ADR-0094). This is
recorded in DECISION-DEBT, not fixed here.

## 2026-09-15 — onboarding a tenant with default config (ADR-0099 stage 4)

A tenant that `provision-org` has created (Zitadel org → derived tenant row → operator role grant)
is authenticated but cannot yet DO anything: docket has no configuration for it, and its granted
operator resolves to no `docket_operators` row. Stage 4 is the step that fills both, and the shape of
it is a split `seed-config.py` learned:

- **`--defaults` seeds only the generic document machinery** — fields, person classes, the document
  lifecycle (+ stages), onboarding messages, retention. This is the reusable docket model every
  tenant starts from, taken from the reference registry; it is NOT a tenant's companies, operators or
  signatories, which are that tenant's own and stay empty until it configures them. A full seed
  (no flag) still writes everything, which is how Paridyum — the first tenant, whose registry IS the
  reference — seeds itself.
- **`--operator "id|name|role|subject"` seeds the first operator's row**, because a granted operator
  who signs in still fails `operator::resolve` (ADR-0092 binds by subject to a `docket_operators`
  row) until that row exists. Its subject is the one `provision-org` granted the role to.

So onboarding is: `provision-org --apply` (org, tenant, role, grant), then `seed-config.py --tenant
<new> --defaults --operator "<id>|<name>|<role>|<subject>" --apply`. The tenant is then functional —
it can enrol people and issue the standard documents — and adds its own companies and signatories
next (Stage 5's admin surface). Proven on a non-superuser PG17: a fresh tenant seeded with
`--defaults` gets the 108 generic records and zero companies/operators; adding `--operator` binds its
first operator, and the subject resolves. Cross-tenant isolation holds throughout — Paridyum's rows
are untouched by seeding another tenant.

## 2026-09-15 — the served backend reads config from the tenant's rows (ADR-0099 stage 3, cutover)

The seed put each tenant's configuration in the 0008 tables; this makes the served backend READ it.
`Registry::from_db(read)` assembles the registry from the config rows — reusing the exact wrapper
types `load` deserialises from YAML, because 0008 stores each entry as the shape those types expect —
or returns `None` when a tenant has no rows, in which case the dispatcher falls back to the git
registry. That fallback is the one line that turns the git registry from the source of truth into a
seed: a seeded tenant is served from its own rows, an unseeded one (only Paridyum, until stage 3's
seed runs on the cluster) from git. The store factory (`PgTenantStores::build`) now returns the
config alongside the store and blobs, all built per request from the token's tenant.

**The gap the cutover surfaced, and the migration it needed.** `lifecycle.yaml` carries TWO
correlated lists — `documents` and `stages` — and 0008 stored only the documents (a stage has no
`code`, the key that table is built on). Rebuilding a `Lifecycle` needs both, so `from_db` failed on
Paridyum's own seeded config until platform-tenancy migration **0009** added `docket_lifecycle_stages`
and the seed learned to fill it. The lesson worth keeping: decomposing a config file per-record works
for independent entries (companies, operators, fields) and breaks for a file whose top-level lists
refer to each other — `lifecycle` needed two tables, not one.

Proven on a non-superuser PG17: `from_db` returns `None` for an unseeded tenant and assembles the
types for a seeded one (unit-tested with a fake reader; the DB reader covered by the factory test),
and Paridyum's 121 seeded rows (companies, operators, signatories, fields, classes, 26 documents, 5
stages, messages, retention) round-trip. `team` stays git/absent — it is public-page content, not
backend config, and 0008/0009 give it no table. Stages 4 (onboarding) and 5 (the web-docket console)
remain; this completes the backend half of the promotion.

## 2026-09-15 — seeding Paridyum's config into its tenant rows (ADR-0099 stage 3, the data move)

Migration 0008 gave docket's configuration per-tenant tables; this fills Paridyum's. `scripts/seed-
config.py` reads `registry/*.yaml` and writes each entry as a row for a tenant, under that tenant's
operator RLS context — Paridyum becoming the FIRST tenant's rows rather than a special case in code.

It writes the record VERBATIM as the jsonb the tables store, so the git registry and the seeded rows
are the same document (RFC-0039 D1 carried to config); the generated key columns (`code`, `subject`,
`class`, …) then extract from it, which is what the backend will look up by. Idempotent — one
transaction per table, `ON CONFLICT DO NOTHING` — so a re-run adds only what is missing and never
edits or deletes. Connection mirrors `platform-tenancy/scripts/migrate.py`: `DOCKET_DATABASE_URL` for
a local/break-glass run, else `kubectl exec` into the primary pod.

Proven against a PG17 whose app role is a non-superuser (RLS binds it): all 116 records seed
(companies 2, operators 4, signatories 7, fields 62, classes 4, lifecycle 26, messages 7, retention
4), the company codes (`PAR`, `PCY`) and rohan's operator subject (`390506086174098663`, the ADR-0092
bind key) resolve from the generated columns, and a second run leaves the counts unchanged.

This is the DATA half of stage 3. The CODE half — the served backend reading config from these rows
(`Registry::from_db`) with the git registry as fallback until a tenant is seeded — is the cutover
that follows, and is where the git registry stops being the source of truth.

## 2026-09-15 — the served backend derives its tenant from the token (ADR-0099 stage 2)

The single-tenant fixpoint was `serve.rs` reading one `DOCKET_TENANT_ID` and stamping it on every
request. It is gone: `auth.rs`'s `VerifiedOperator` now carries the tenant, derived from the token's
`resourceowner` org claim (ADR-0082, `domain::tenant`), and the store is built PER REQUEST for that
tenant. One deployment serves every tenant; RLS keeps them apart, as it already did for the data.

**The shape of the change is a store factory, not a param on every method.** `Store`/`Blobs` are
built once at startup in the single-tenant design; making them per-request without threading a
`tenant` through forty method signatures meant a `TenantStores` port that builds `(Store, Blobs)`
for a tenant, and a `Serving` context that is either `Fixed` (the loopback/break-glass backend and
the CLI, one tenant, no token) or `PerTenant` (the served backend). The server verifies the token at
the boundary, derives the tenant, builds the store for it, and only then dispatches — so a handler
never sees a store scoped to the wrong tenant.

**Two consequences worth stating.** Reads are now authenticated too: a read names no operator today,
but it names a *tenant*, and without a token there is no tenant — so every request but `/api/health`
(which names neither, and is the k8s probe) now carries a bearer token. And `docket login` requests
the `urn:zitadel:iam:user:resourceowner` scope so the token actually carries the org; without it the
backend refuses the request rather than guessing a tenant, which is the multi-tenant version of the
`DOCKET_TENANT_ID` guess this removes.

The break-glass CLI is untouched: it still names its one tenant explicitly (`DOCKET_TENANT_ID` +
direct DB), because a break-glass path that inferred the tenant would be less legible, not more.
Proven against a PG17 whose app role is a non-superuser (RLS binds it, as in CI): the full suite is
green, and a token with no `resourceowner` is refused. Config still loads from the git registry
(Paridyum's) as the fallback; moving it to the per-tenant tables (0008) is stage 3.

## 2026-09-14 (go-live) — the first green CI on main after the merges, and what the merges broke

Merging the stacked branches to `main` and moving `infra-ci@v1` finally let real CI run (before the
tag move it was `startup_failure`: the caller passed `postgres: true` to a `@v1` that predated the
input). It surfaced two failures that only a real run could:

- **`RUSTSEC-2026-0285`** — rustls 0.23.44 accepted TLS 1.3 handshake messages at the wrong
  encryption level. This is docket's actual TLS path (the API client, the S3 signer), not a dev-dep,
  so it is **fixed, not ignored**: `cargo update -p rustls` → 0.23.45, the patched release. `cargo
  deny check advisories` is clean.
- **Secret scan (`leaks found: 1`)** — ironically the `.gitleaksignore` comment itself: it spelled
  out the deleted spike's example DSN in full, so the `paridyum-db-url-with-password` rule fired on
  the comment on every commit that carried the file, and the served.rs fingerprint (pinned to a
  now-rewritten commit) no longer covered it after the squash. Fixed durably at the value layer: the
  captured "password" in every one of these findings is the placeholder word `unused`, so it joins
  the anchored placeholder alternation in `.gitleaks.toml` (beside `changeme`, `dummy`, …). That
  clears the finding across the whole `--history` scan with no per-commit fingerprint to chase — and
  respects ADR-0017 (no blanket path allowlist; this is a value allowlist for a documented
  non-secret). The comment is also reworded to stop spelling the DSN out.

Verified locally against a throwaway PG17 with platform-tenancy 0001–0007 applied: 261 lib + 5
served-API tests pass; `gitleaks` reports no leaks over full history; `cargo deny` clean. The one
remaining red is not a code defect: the cross-repo migration checkout needs a `contents:read` token
on platform-tenancy (D-69), which `read:packages`-only `PARIDYUM_PACKAGES_TOKEN` is not.



The `Store` port moved docket's *records* to Postgres and deliberately left the *bytes* — issued PDFs,
packs, signed scans, the plaintext a vault seals — on the filesystem. That was the right interim, but
it has a sharp edge now that docket is served: a served pod's filesystem is ephemeral, so a document
rendered by `onboard` over the API is lost when the pod restarts. ADR-0094 chose the destination
(Garage, in-cluster S3), so this entry ends the interim with a **second port, exactly parallel to the
record port**.

**The `Blobs` trait** (`adapters::blobs`): synchronous `put(key,bytes)`, `get(key)->bytes`,
`exists(key)->bool`, `delete(key)`, `Sync` supertrait so a `&dyn Blobs` is shared across the served
backend's per-request threads. Two impls. `FileBlobs` is the behaviour docket always had, now behind
the port: a key maps to `store/<key>`, so `store::keep_pdf`'s filesystem logic moved here (and was
deleted from `store.rs`) and every historical `pdf_path` still resolves — the CLI's offline/break-glass
path is unchanged, proven by the existing suite. `S3Blobs` is Garage, selected the same way the store
is: `DOCKET_S3_ENDPOINT` + `DOCKET_S3_BUCKET` present → `S3Blobs`, else `FileBlobs`. Keys are prefixed
with the tenant id before they reach the bucket (the tenant-scoping invariant), and that prefix is an
addressing detail of the S3 layer — it is NOT part of the `pdf_path` recorded in the register, so
already-sealed keys are untouched.

**The S3 client stays inside ADR-0005.** `rusty-s3` (BSD-2-Clause) is a pure `SigV4` signer with no HTTP
or TLS of its own — it produces a presigned URL, and the crate's existing `reqwest` (rustls) sends it.
No openssl, no second TLS stack. The signer is async only in that reqwest is, so — like `PgStore` —
`S3Blobs` drives each request to completion on the shared tokio runtime with `block_on`; the callers
are blocking threads (thread-per-request server, synchronous CLI), never inside the runtime, so this is
the sanctioned bridge, not a nested-runtime hazard. `cargo deny check licenses` passed with NO change to
`deny.toml`: rusty-s3 and its new transitive deps (jiff `Unlicense OR MIT`, instant-xml, md-5, zeroize,
hmac) all resolve to licences already on the allow list, unlike `webpki-roots` earlier which needed one
added. Recorded here rather than left implicit.

**Threaded in the order §11 sets, each step keeping the suite green.** `keep_pdf` + the issued-PDF
read-back first (that is what makes served `onboard` durable, the whole reason this is not deferred
again): the issuer computes the key `issued/<person_id>/<file>` and `Blobs::put`s the bytes; the key is
what the register records as `pdf_path`, frozen the first time it is written. Then `application::pack`
(assembles the pack by `Blobs::get`ting each entry's bytes rather than `fs::copy`ing a path). Then
`sign --scan` (the scan uploads via `Blobs::put` under `scans/<person_id>/…`, and that key is the
signature record's `scan_path`). Then `application::vault` seal (reads the plaintext to encrypt through
`Blobs::get`, finding scans through the signature ledger's `scan_path` because the port has no listing).
Then erase's blob `delete`, alongside the reseal (§6): it deletes each `pdf_path` and `scan_path` for
the person through the port — `Blobs::delete` is a real, idempotent delete, not a tombstone (ADR-0069) —
then tidies the now-empty local dirs. `verify` and `check` also read the stored bytes back through the
port, so the served backend's `verify`/`check` are durable too. The register chain is untouched
throughout: serial and digest stay in Postgres; only the bytes moved.

**Wired at both entry points.** `main::run` builds the blob store once (sharing the Pg runtime when
there is one, else building a runtime kept alive for the command) and threads `&dyn Blobs` through
`dispatch` → `route`/`route_person` → each handler; `bin/serve` and the loopback `docket serve` build it
into the `router::Context`, so the served backend runs on `S3Blobs` and the CLI on `FileBlobs` unless
S3 is configured.

**Invariants proven by test, not assumed.** `adapters::blobs` unit tests: a document's bytes verify
against the recorded SHA-256 after a round trip through **both** `FileBlobs` and `S3Blobs` (the blob
analogue of seal-preservation); `Blobs::delete` actually removes the object and is idempotent; and the
S3 object lands under the tenant-scoped path `/<bucket>/<tenant>/<key>`, proving prefix scoping. The
S3 test runs against a minimal in-memory S3 over a real `TcpListener` — no Garage required for
`cargo test`, exactly as instructed (Garage is in-cluster and not reachable from here).

**Deferred, deliberately.** Migrating already-issued filesystem PDFs into Garage is a one-time DATA
move, not a code change (D-65). Sealed vault `.age` objects and per-person keys stay on `Paths` for now
— the key must be locally destroyable for `shred` to be real erasure (D-66).

**Skills and tools used.** Reading ADR-0094, design §11 and the whole `store.rs`/`pg.rs` port pattern
before writing a line (governance precedes implementation); mirroring `PgStore`'s `block_on`-on-a-shared-
runtime bridge rather than inventing one; `cargo deny check licenses` before assuming the new dep was
free; the compiler as the guide through the caller signatures; an in-memory `TcpListener` mock so the
gate needs no Garage; `scripts/gates.sh` as the single un-piped gate.

## 2026-09-14 — the deployment surface for the served backend (ADR-0093 §3)

ADR-0093 makes docket a served, staff-authenticated platform and reclassifies it: the served
`serve` binary is platform-shaped, so `tool-docket` "stops being deployable: false" and now owes
ADR-0020's Helm chart, probes and resource limits. This entry adds that surface without touching the
Rust internals (a parallel worker owns the Postgres port):

- **Helm chart** (`helm/`), modelled on `platform-identity/helm` rather than invented: Deployment +
  Service, a digest-only image (the chart `fail`s without `image.digest`, ADR-0060), `/healthz`
  liveness AND readiness probes against the serve binary's unauthenticated route, resource
  requests/limits (ADR-0020), ClusterIP, and the SOPS→k8s-Secret wiring the estate uses — the DB
  URL is referenced from a Secret named `docket`, the chart creates none. Non-secret DOCKET_* config
  is inline in `values-staging.yaml`. `helm lint` clean; `helm template` renders and refuses without
  a digest as designed. Root filesystem is read-only with emptyDir mounts for `/tmp` and the blob
  store — ephemeral on purpose while only synthetic data is served (§5), durable object storage left
  as the open §6 decision.
- **Dockerfile** for the `serve` binary: multi-stage, `rust:1.97.1-slim-trixie` (the locked
  toolchain, STD-0003), rustls not openssl (ADR-0005 — sqlx and reqwest carry rustls features so the
  runtime needs only ca-certificates), `--locked`, non-root, PID-1 binary for clean SIGTERM. It
  bakes `registry/`, `templates/` and `latex/` read-only at DOCKET_ROOT; records and PDFs never
  enter a layer. The image build passes once the parallel refactor lands and `serve` compiles — the
  env-var interface it targets is stable.
- **CI** `publish.yml`, a thin caller of the estate's reusable `image-publish.yml@v1` (ADR-0021),
  publishing by digest to `ghcr.io/paridyum/tool-docket`. `ci.yml` is unchanged.
- **manifest.yaml** reclassified `class: tool → platform`, gaining `exposes.http` and the `serve`
  entrypoint. The class registry already carries `platform` as deployable, so no registry edit was
  needed; `platform` permits `rust`, so the class↔stack gate passes.

**The one real tension, flagged not hidden (D-60).** The repository is still named `tool-docket`,
whose `tool-` prefix maps to the `tool` class under C-0005/C-0007, while the manifest now says
`platform`. No shipped gate enforces prefix↔class agreement (`repo.name_matches_scheme` is
forge-owned and reads the name, not the manifest class), so nothing breaks today — but the honest
resolution is a repository rename to `platform-docket`, which is a separate breaking operation. Left
for a human decision rather than performed silently as a side effect of a chart.

**Skills and tools used.** Reading `platform-identity/helm`, the shared `rust-ci.yml` /
`image-publish.yml`, and `class-registry.schema.json` / `manifest.schema.json` before writing
anything, to match the established convention rather than invent one; `helm lint` and `helm template`
as the structural gate; `check-knowledge.sh` for the class↔stack and manifest checks.

## 2026-09-13 — the store starts moving to Postgres: the register adapter (ADR-0093, RFC-0039 D1)

ADR-0093 supersedes ADR-0072's one-node/filesystem store; docket becomes served + multi-user, so its
store is the database. First the invariant that had to hold: `domain::{register,identity,event}` now
each carry a test proving a sealed chain survives the JSON round trip — the record → JSON → record
path a JSONB column is — with digests and links intact, including the Standing and Kind enums. If that
had failed, the whole move would have been wrong (RFC-0039 D1).

Then the adapter. `adapters/pg.rs` is the async sqlx store (ADR-0005): `append_register` /
`read_register` for now. Each operation runs in a transaction that first sets the RLS session
variables (`paridyum.tenant_id`, `operator_scope`) with `set_config(..., local := true)`, because
migration 0006's policies gate on both and the accessors RAISE when unset (ADR-0038) — a forgetful
call is refused, not silently widened. Records are stored verbatim as JSONB (`sqlx::types::Json`) so
the seal is byte-preserved; `read_register` verifies the chain before returning and treats a fault as
an error, as the filesystem store did.

**Why async in a sync CLI.** sqlx is async and is the locked DB lib. The served backend will await
this adapter; the break-glass CLI will `block_on` it (ADR-0093 §3) — one implementation, two callers.
That is why the CLI still works "remotely": it becomes a Postgres client over the mesh, not an
SSH reader of a file node.

**Proven, not asserted.** `tests/pg_store.rs` runs the register through a real Postgres 17: seal
verifies after the DB round trip, records are byte-identical, and another tenant sees nothing (RLS).
It seeds a fresh tenant per run so it is re-runnable, and skips without `DOCKET_DATABASE_URL` so
`cargo test` needs no database (SOP has the one-liner). Supply chain: sqlx/tokio/rustls pulled
`webpki-roots` (CDLA-Permissive-2.0), allowed in deny.toml with a note, matching platform-identity.

The adapter is now COMPLETE for the store: the three chains share one generic `append_chain`/
`read_chain` (register/identity/event), and the mutable person record has `upsert_person`/
`read_person`/`list_persons` (the roster). A person is a `serde_yaml::Value` carried losslessly
through JSONB. Four integration tests pass against real Postgres — each chain keeps its seal (incl.
the Standing and Kind enums), the person upsert replaces rather than duplicates, and RLS isolates
every table by tenant.

The ADR-0092 auth core is next in and done: `adapters/auth.rs` verifies an operator's control-plane
token against the JWKS — RS256, issuer pinned to id.paridyum.com, audience pinned to the
paridyum-operations project (a token minted for another app is otherwise accepted, ADR-0047 D5),
`operator` role required — and returns the opaque subject, which `Registry::by_subject` binds to a
docket operator (ADR-0092 §2; `operators.yaml` now carries rohan's `subject`). Seven tests cover the
happy path and the full denial matrix (wrong role/issuer/audience, expired, unknown key), signing
with a keypair generated at test time so nothing is embedded to scan for. Second in-crate RSA
verifier in the estate; a shared Rust lib-auth waits for a third consumer (ADR-0092 §3).
**[2026-09-24: the third consumer arrived and this is now a call into `paridyum-auth` —
see the entry for that date.]**

Then the served backend itself: `src/served.rs` (the router, testable via `app`) and `src/bin/serve.rs`
(the deployable binary — fetch the control-plane JWKS once at startup, then serve). Every protected
route goes through one `authorize` middleware that requires an operator token, so a handler cannot
run unauthenticated by construction (STD-0004). Endpoints so far: `GET /healthz` (open), `GET /roster`
and `POST /person` (operator-gated) — list and create HR records, the multi-user surface ADR-0093 is
for. The CLI stays sync and unchanged; this is the everyday interface beside it (ADR-0093 §3).

Proven end to end: a wiring test suite (healthz open, protected routes 401 without/with a bad token)
and `tests/served_api.rs`, which signs a real operator token, `POST`s a person and `GET`s the roster
back through the router, the pg adapter, the database and RLS — the whole product path on the happy
line. What remains is **console-docket's OIDC login** in front of this API (the TypeScript half), a
Helm chart + CI to deploy it, and the backup gate before any real person record (ADR-0068 §5).

Started 2026-09-06, part-way through the tool's life. Everything before that
date lives in `CHANGELOG.md` and `DECISION-DEBT.md`, which were doing this
job less directly.

## 2026-09-06 — officers, and the four templates that could not be shared

The ask was job descriptions and job IDs for the openings, wired to docket.
Two gaps sat behind it, and the second was not the one anybody was looking for.

**The wiring did not exist.** ADR-0063 decision 4 names the seam — a person's
`role` should reference a posting — and explicitly does not perform it.
`engagement.role` was free prose whose stated purpose is what the agreement,
the certificate and the reference letter print. So `engagement.job_id` is a new
optional field beside it rather than a replacement: those are typeset documents
and must not depend on a JSON file in another repository resolving at render
time. RFC-0041 has the argument, including why the posting identifier had to be
frozen before anything could reference it.

**Then the officers.** CEO, CTO and Director existed in `operators.yaml` and
`signatories.yaml` and nowhere else — no person record, no NDA, no IP
assignment, while every intern will sign all three. The instinct behind the
question ("we will hire our existing through this pipeline?") was right about
the gap and wrong about the mechanism: an officer is appointed, not offered, and
running one through `docket onboard` would have issued an Internship Offer and
a Certificate of Internship to a director and had the CEO sign her own offer
letter.

**What implementation found that the design got wrong.** RFC-0042 first claimed
`NDA`, `COC`, `ISP` and `DPN` applied to an officer "unchanged". They do not.
`nda.tex` says "Intern" sixteen times and "internship" six;
`infosec-acceptable-use.tex` twenty-eight and eleven; `code-of-conduct.tex`
states "This internship is unpaid and part-time", which is false of a director.
The party term is one macro away — but "the internship" is the *arrangement*,
not the person, and generalising it would have edited the operative text of
documents interns have already been issued, for the convenience of a different
class of person. So officers got their own two documents and the RFC was
corrected rather than the code bent to match it. The cost is recorded as D-46:
confidentiality prose now exists twice and nothing checks that it agrees.

**Three things had to be class-aware, and the middle one is the one that would
have shipped broken.** Documents (`classes:`, so a false document is refused);
the **stage gate** (`preboarding` requires the internship agreement and intern
IP assignment executed, so an unfiltered gate would have held an officer
forever, waiting on a document nothing would ever produce); and the record
template (`_TEMPLATE.yaml` carries education and a guardian block, both of
which the officer class forbids, so copying it produces a record `docket check`
rejects the instant it is written — which reads as a product defect rather than
as the registry working).

**Nobody signs their own document.** `signed_by_class` routes officer documents
to the Director, and `docket issue` refuses where subject and signatory resolve
to the same person. Verified against the **rendered PDF text**, not the
register: the register can be right while the paper is wrong, and the paper is
what somebody signs. `\paridyumsignblock` was added so the block resolves from
the document rather than the template naming the CEO/CTO pair in twenty-one
places.

**On the IP question, the operator overruled the design twice and the second
version is simpler.** The first draft proposed a founders' agreement with an
assignment-on-incorporation undertaking, then assignment to the Director as
nominee and trustee — both answers to "you cannot assign to a company that does
not exist" (`org.yaml` has `cin: null`; Internship Agreement §13 records
Paridyum as operated by Priyanshu Sharma personally). The operator's decision:
write everything in the company's name and treat it as in effect from
signature, as it would be after incorporation. That is what every intern
document already does, so it adds no new exposure — it extends the existing one
to the founders' own work product. Copyright needs no filing to subsist, so
nothing is lost by not registering early. Recorded in RFC-0042 §3 as an
operator decision rather than a legal finding, and flagged for counsel at
incorporation.

**Left undone, deliberately.** `COC`, `ISP` and `DPN` do not exist as officer
documents (D-46) — an officer holds the most access in the estate and has
agreed to no acceptable-use policy. `engagement.job_id` is validated by nothing
(D-47). ADITYA SHRIVASTAVA is recorded as COO with `appointed_on` and
`instrument` both `null`, because the appointment was stated and the instrument
was not produced; C-0004 and RFC-0042 §5 both want one, and the legal-name
spelling has not been checked against it.

Verified with `scripts/gates.sh` — fmt, clippy, 313 tests, `docket check` —
all green, run through the script rather than by hand, because the changelog
entry for that script exists precisely because piping `cargo fmt --check` into
`head` returns the exit code of `head`.

Skills used: `paridyum-platform:design-change` for the change lifecycle.
RFC-0041 and RFC-0042 were written, reviewed and accepted before any code in
this repository was touched.

## 2026-09-06 — the bootstrap, and what running it found

Four officers to onboard, run by one operator. Written as `scripts/bootstrap-officers.sh` in two
passes: open the records, stop; then, once a human has filled them in, check and issue. It does
not invent personal details, for the reason `_TEMPLATE.yaml` carries placeholders at all.

**Running it in a sandbox found two defects that reading the code did not.**

`blockers` refused every officer for `education.currently_enrolled is false` and
`engagement.end_date is unset`. Both are the intern class's preconditions and say so in their own
message text; `person-classes.yaml` forbids `education` for an officer and requires no end date,
so the refusals were **unsatisfiable** — no edit clears them without breaking a different rule.
Scoped to the intern class, with the DPDP s9 child check deliberately left unscoped because it is
about processing a child's data rather than about internships.

And the refusal for a self-signed document, correct as a rule, left the Director with no
documents at all. Substitution replaced it: the first active signatory in registry order who is
not the subject. Registry order rather than anything cleverer, because the substitute has to be
the same person every time — who signed a legal document must not depend on hash order.

**The sandbox mattered for a second reason.** `docket new` allocates a person identifier that is
never reused (ADR-0051), so a test run against the real store would have burned four of them.
`DOCKET_PEOPLE` and `DOCKET_STORE` made a throwaway store possible; the real run is the
operator's.

Verified: all eight documents issued in the sandbox, the register recording `signed_by=['director']`
for three officers and `['ceo']` for the Director, and `pdftotext` confirming **RUHANI KHURANA,
Chief Executive Officer (CEO)** in the Paridyum block of the Director's own NDA with PRIYANSHU
SHARMA countersigning as Officer. Sandbox removed; nothing was written to the real store.

## 2026-09-06 — the three officer policies, and the two that were inverted

D-46's open half: `COC`, `ISP` and `DPN` did not exist for officers, so the four people with
the most access in the estate had agreed to no acceptable-use policy. Written as `COO`, `ISO`
and `DPO`.

**The interesting part is that a word-swap would have been wrong in both directions.** Going
through the intern text section by section, four differ in substance and two are inverted:

*Speaking publicly* — the intern code says they may not speak on Paridyum's behalf. An officer
may; that is part of the office. Copying the intern clause would have forbidden a director from
doing their job, so the officer version is about care instead: a claim must be true when made,
not merely intended, and the officers' names are on the public site, which is what makes it
matter more for them than for anyone else.

*Access* — the intern policy grants access to what an assignment requires. An officer's access
is administrative and broad by necessity, so the boundary cannot be drawn by somebody else. The
officer version makes least privilege a discipline they impose on themselves: **being able to
reach something is not a reason to.** Copying the intern clause would have described a
permission model the estate does not have.

Two routes could point at their own subject, which is the same defect the signature
substitution exists to prevent, in a different place. `org.yaml` routes security incidents to
the CTO — so an incident involving the CTO had nowhere to go. And the Grievance Policy is
`classes: [intern]`, so it cannot be the route for a concern about an officer. Both now name
another officer, and the ISO says why in one line.

*Breach* — an office is vacated or removed by the directors. It is not terminated, and there is
no Internship Agreement to point at, so the intern clause was simply describing a thing that
cannot happen.

**`DPO` differs in what it says is collected, not only in wording.** The intern notice lists
education details, emergency contact and a supervisor; the officer class forbids `education` and
`statutory` and does not permit `emergency_contact`. A data protection notice that overstates
what is held is a defect in the notice — its entire purpose is to be an accurate account of the
processing. So section 1 is derived from what the class actually permits, and it adds the one
thing an officer has that an intern does not: two facts about them are *published*, not merely
held.

Verified by rendering all five officer documents and reading the PDF text, not the source. The
remaining "intern" strings in the output are deliberate cross-references — "differs from the
copy given to an intern", "the Grievance Policy is an intern document" — plus one false positive
on "internal business information".

Left open and narrowed rather than closed: D-46 is no longer a coverage gap, it is a
duplication. Five officer documents now carry text adapted from five intern ones, and nothing
compares them. Diffing fails immediately because they are supposed to differ.

Gates green: fmt, clippy, tests, `docket check` — 26 documents, 0 errors.

## 2026-09-06 — addresses, and the subdomain that would have repeated a locked mistake

Four officers need email, interns follow, and nothing said what an address looks like. C-0007
governs repositories, crates, API paths, tables, environment variables and images — not
mailboxes. Three function addresses existed by custom: `legal@`, `hello@`, `security@`.

The shape proposed was `vivaswan.pdol001.paridyum@gmail.com`, with `intern.paridyum.com`
floated for the domain half.

**The subdomain is the part worth recording.** `intern.paridyum.com` is ADR-0051's locked
mistake in a worse place. That ADR rejected `PAR-INT-2026-0001` as a person identifier because
`INT` is a class and class is mutable for a person — "the predictable case is the one that
breaks it: an intern is hired." An address is a far worse place to make it than a folder name,
because most copies are not yours: they are in other people's address books, on signed
documents, in university records, and in commit authorship that C-0003 requires be real and
attributable. A folder can be renamed by one person. An address cannot.

The honest counter is sending reputation, and it has a standard answer that does not put a
mutable fact in someone's identity: isolate bulk and automated sending on its own subdomain.
That is about what sends, not who sends.

**`pdol001` could not be decoded and was replaced rather than guessed at.** It is not the
person identifier and not the engagement identifier. Whatever it meant, docket already mints
something with exactly the properties a Gmail disambiguator needs — unique across Paridyum
forever, never reissued, and carrying a check character. Adding a second hand-kept counter
would have been a second thing that must never be reused, which ADR-0051 singles out as the
failure that silently reattaches one person's records to another.

**The Gmail account name keeps the given name, and that is a deliberate departure from
ADR-0051's opacity rule.** That rule is argued from a directory listing disclosing who Paridyum
engages to anyone who can read it. A Gmail account name is not a listing anybody browses; what
it has to be is findable in an account switcher when an operator holds twelve of them.

**The operator chose free Gmail long-term rather than Workspace, and the cost lands on
something docket was built to do.** `access.company_email`'s stated purpose is "so offboarding
can prove it was removed", and a consumer account cannot be suspended, audited or recovered by
Paridyum. Rather than let the record imply a proof that does not exist, the field now means the
published alias specifically and `access.mailbox_email` is separate: **severing an alias is
provable; recovering a consumer mailbox is a request.** Splitting them is also the mitigation —
Paridyum can end someone's ability to act as Paridyum without needing access to their mailbox.

Derivation is a domain function with six tests rather than a documented convention, including
the two cases a convention gets wrong: a three-part name keeps every part, and a name with no
ASCII yields a visibly wrong local part instead of an invented romanisation — permanent, under
a rule that forbids renaming.

`gates.sh` earned itself again: `cargo build` was clean and clippy was not (`-D warnings`), so
building and eyeballing would have recorded a failing gate as green.

## 2026-09-07 — the collision form, and a check character that did not check

The operator proposed replacing the surname with the person identifier — `firstname.<id>` —
on the grounds that it would be shorter and easier to print.

**Measured, it is neither.** Of the four real officer names, three get *longer*:
`rohan.batra@` is 24 characters and `rohan.p000147c7@` is 28. And on printability the evidence
runs the other way and comes from ADR-0051 itself, which attached a check character to the
identifier because *"this string gets typed and read aloud"* and MOD 11-2 *"catches the error
class a hand-copied folder name actually produces."* The identifier needs error-detection
machinery because people mistype it. `batra` does not.

**But the instinct was right about the collision case, and the first design was wrong there.**
When two people collide they share the surname, so `rohan.batra.p000147c7@` carries a segment
that disambiguates nothing. Dropping it is strictly better.

That let a rule be deleted rather than added. The middle-name step is gone: it existed only to
push the collision back one place, it depended on the person having a middle name, and it gave a
third shape an outsider could not predict. **Two shapes now** — `firstname.lastname@` normally,
`firstname.<id>@` on collision — and the second matches the Gmail local part, so one derivation
serves both domains.

**The test caught a fabricated example.** `p000147c3` had been used throughout the ADR, the
changelog and the doc comments; the real check character for sequence 147 is `7`. A documented
identifier that fails its own check digit is precisely the string somebody copies. Corrected in
all five places, including `identity.rs`'s own doc comment, which had carried `PAR-P-000147-3`
since before this work — the module that implements the check character was illustrating it with
a value that fails it.

Verified with `scripts/gates.sh`: fmt, clippy, tests, `docket check` — all green.

## 2026-09-07 — anti-collision by default, which deleted a subsystem

The operator's call: make the collision-proof form the default rather than the fallback. Every
address is `firstname.<person id>`.

**It removed more than it added, which is why it is the right call and my two-shape design was
not.** I had said an address register was mandatory — an append-only list of every address ever
issued, never pruned, consulted before handing out the next one, retained for people who had
left and for people whose records had been erased, because reuse delivers one person's password
resets to another. That whole structure existed to detect a collision. Derive the address from
an identifier that ADR-0051 already guarantees is unique forever and never reissued, and the
collision cannot happen: there is nothing to detect and nothing to keep.

Two smaller things fell out of it. An address is now **self-validating** — a transposed digit
fails the MOD 11-2 check character rather than bouncing silently, so the check character
ADR-0051 added for folder names earns its keep on every address. And **nobody's address is worse
because they joined second**, which the fallback scheme could not say.

**The cost is real and is recorded rather than argued away.** The address is not guessable and is
harder to read aloud. What makes that survivable is that `hello@`, `legal@` and `security@`
already exist and are what an inbound stranger actually uses — a personal address is given out,
not deduced. I had weighted guessability too heavily by treating personal addresses as the
inbound path when the function addresses already are.

The measurement from the previous round still stands and is now moot: `firstname.<id>` is not
shorter than `firstname.lastname` — it is longer for three of the four officer names. It was
never chosen for length.

Verified with `scripts/gates.sh`: fmt, clippy, tests, `docket check` — all green.

## 2026-09-07 — D-46 researched, and the answer was not to rewrite

Asked to research whether the duplicated officer/intern templates could be rewritten from
scratch. **The research says no, and it contradicts what I recommended before doing it.**

I had measured 71% of lines identical across the pairs and concluded a shared-clause refactor
was worth a day. Broken into sections, the picture inverts: **5 of 51 sections are byte-identical
and 28 differ in substance.** The line-level figure was counting `\begin{itemize}` and shared
phrasing inside sections that genuinely differ. Two sections are inverted outright.

Even with a party-term macro, 45% would be shareable — and it would put `\dkPartyTerm` into prose
a person signs, so no reviewer could read the document without evaluating a substitution in their
head. For legal text that is a worse defect than duplication.

**Across the entire template set, only seven duplicate section copies existed.** Five were
`Governing Law`, appearing in five documents, two of which differed only in where the line
wrapped. That one is now `templates/clauses/governing-law.tex`. The remaining two were within a
single pair each and not worth the indirection.

**The order mattered more than the extraction.** `template_sha256` hashed one file, and it feeds
`verification_code`, which is *printed on the document*. Extracting a clause without changing
that would have left five documents recording a hash that no longer covered their own terms.
So the closure hashing went first, with tests, and only then did a clause move — and the register
held **zero entries** at the time, which is the only moment that change is free. After the first
issuance there would be two eras of hash in one register with no way to tell them apart.

Verified by rendering all 25 documents to text before and after and diffing: **identical, not one
word changed.** That is the check that matters for a refactor of legal text, and it is the reason
to render rather than reason.

What replaced the rewrite is an **adaptation pin** — `% adapted-from: <source>@<sha256>` in each
adapted template, recomputed by the suite. It does not compare content, because the two are
supposed to differ; it makes changing one half silently impossible. Proved by amending the intern
ISP and watching the suite name the officer file and the digest to paste.

D-49 and D-48 closed in the same pass. The D-49 guard needed a second attempt to verify: removing
the `.gitignore` negation did not fail the test, because the file was already *tracked* and
tracked files ignore `.gitignore`. The test was right and my mutation was invalid — proved
properly by adding a throwaway class with an untracked ignored template, which failed with the
fix in the message.

339 tests. `scripts/gates.sh` green.

## 2026-09-07 — the vault, and the property that made it possible

Permanence and erasure look irreconcilable, and for the same bytes they are: a key that always
opens a document makes it uneraseable, and no such key makes loss permanent. Sync keeps versions,
so deleting a file there is a gesture — and this estate gitignores `people/` precisely rather
than accept an unerasable record.

**What resolved it was reading `verify` before designing anything.** It compares a PRESENTED file
against the hash recorded at issuance; the stored copy is a convenience and the signature already
takes an optional path. So shredding the archive does not destroy verifiability — an erased
person can hand over their own certificate years later and have it check out against a register
entry that survives. Without that, one of the two requirements would have had to give.

**One recipient per object, and the single-ness is the security property.** Encrypting to a
person's key and an estate key would have been recoverable *and* granular, and it is escrow in
disguise: destroying the personal key shreds nothing while the estate key still opens the file.
`crypt::encrypt` takes one recipient so that adding a second has to be a change to that function,
where the argument against it is written down.

**Erase was worse than I thought before touching it.** It deleted the record and re-sealed the
event log, and left every issued PDF sitting in `store/issued/`. So the thing docket called
erasure left the person's documents on disk. It now destroys the key, the ciphertext, the
plaintext and the record, in that order, with the tombstone written *first* — a crash between
tombstone and deletion leaves a visible inconsistency rather than a silent one.

**A test wanted to set an environment variable and the crate forbids `unsafe`.** That turned out
to be the compiler pointing at a design flaw rather than an obstacle: `estate_identity()` was
being read deep inside the vault, where a test could only reach it by faking a process
environment. Passing the identity in from the one caller that knows it needs one made the
dependency visible in the signature and the tests trivial. The lint was right.

Verified end to end in a sandbox with its own estate key: seal, confirm the object is
`age-encryption.org/v1` and not `%PDF`, erase, then verify the person's kept copy — **MATCH**.
Six integration tests, skipping rather than failing where `age` or a typesetter is absent.

`docket vault sync` prints what to upload and what must never be, because docket does not talk to
Sync: the vendor's client is Windows and Mac only (RFC-0038), and a tool that shelled out to it
would be pretending to own something it does not.

## 2026-09-07 — the half the vault had left out

The operator asked about periodic encrypted bundles to Sync, and about backing textual data into
Bitwarden with the `bw` CLI. Answering the first found a hole in what had just been built.

**`vault seal` covered issued PDFs and nothing else.** The register, ledger, event log,
signatures and tombstones — the half that is *never* erased and that makes every document
verifiable decades later — were in no uploaded path at all. The vault was protecting the
erasable half and leaving the permanent half on one laptop. `docket vault backup` fixes it,
encrypted to the estate key rather than a person's, which is also what makes it safe in a
versioned store: there is no erasure promise there for a retained old version to break.

**A whole-store bundle was rejected**, which is the shape the question implied. You cannot erase
one person from a tarball that is already uploaded and versioned, and encrypting it to one key
makes everyone's data recoverable with that key forever — the escrow ADR-0069 turned down.

**Bitwarden for textual data was rejected on RUN-0004's own stated property:** *"There is exactly
one thing to lose and exactly one thing to back up."* Backing arbitrary data there turns one
thing to protect into many. Worse, the password manager holds the key — putting the data beside
it means one compromise gets both. And Bitwarden has a trash and sync history, so it is not a
medium you can reliably delete from, which is the same flaw that keeps keys out of Sync.

**The bundle lists its five files individually rather than globbing `store/`.** A glob would have
taken `keys/`, and a key in a versioned store cannot be destroyed — after which no erasure this
system performs is real. There is a test asserting nothing under `keys/` reaches the uploaded
path, because that is the mistake that would be silent.

Recovery is rehearsed rather than asserted: a test decrypts the bundle with nothing but the
estate key and checks the serial and hash come back. That is the whole of what `verify` needs, so
**the estate key alone restores verifiability for every document ever issued** — including for
people whose vaults have been shredded and who hold the only copy.

Still open and now sharper (D-50): the per-person keys have no backup anywhere. Losing the laptop
today would leave the register intact and every person's documents unopenable — you would know
exactly what was issued and be unable to produce any of it.

347 tests. Gates green.

## 2026-09-07 — D-50, and the thing that was actually unprotected

Asked to handle "the per-person keys have no backup". Two findings, and the first says I had
described the problem wrongly.

**The tension is structural, not a gap.** Crypto-shredding needs key material that can genuinely
be destroyed. Any copy of that material improves durability by exactly as much as it damages the
erasure promise. I went looking for a scheme that escapes it — deriving keys from the estate key
plus a salt, so nothing needs backing up — and it fails for the same reason in a different place:
the salt then has to be destroyable, so it cannot live anywhere that retains deletions either.
There is no design that gets both. What a design can do is make the choice explicit, which is
what `docket vault escrow` is, with both columns written into ADR-0069 D6.

**I also checked whether the registry already resolved it, and it half did — the other way.** The
`permanent` retention class says *"register metadata is retained even when the underlying personal
data is erased."* So I could not move permanent-class PDFs to the estate key to make them durable:
the registry authorises keeping the metadata, not the document. The split as built is right.

**The second finding is the one that mattered.** Chasing "what is actually irreplaceable" found
that **signed scans were protected by nothing at all.** `seal` walked the register, which lists
issued PDFs; scans live in `store/scans/` and were in neither the vault nor the register bundle,
so they were in nothing that ever left the laptop. And `erase` deleted the issued PDFs while
leaving them — **erasing the copies the person already holds and keeping the one they do not**,
which is the inverse of what erasure is for.

They are the half that cannot be recovered from the counterparty. A document was handed over in a
pack; a returned scan exists nowhere else, and `sign.rs` says why: *"wet ink leaves no trail of
its own, so the scan IS the evidence."*

**A bug the test found, not me.** `vault status` reported "every issued document is sealed" when
nothing was sealed. The guard that skips a shredded person's documents keyed on the key file being
absent — and a person who has never been sealed also has no key file, so the two states were
indistinguishable. The tombstone is the discriminator: it is the record that erasure actually
happened.

**And a process note worth keeping.** Three separate edits silently did nothing this session
because `cargo fmt` had rewrapped the text I was matching against. The last two carry an assert on
the match, which turned a silent no-op into an immediate failure. A replace without one is a
change you have to remember to verify.

351 tests. Gates green.

## 2026-09-07 — Multiple nodes, and the two questions hiding inside one

**The question was how to run docket on more than one machine so that losing a laptop does not
hurt. Measuring first turned it into a different question, twice.**

**The key was never the exposure.** RUN-0004 already keeps the one age key in a password manager
and installs it per machine, so a lost laptop loses no key. What lived on exactly one machine was
the *store* — four hash chains, the issued PDFs, the signed scans, the per-person keys.

**Then multi-writer turned out to be wrong rather than hard.** A person identifier comes from
`next_person_seq(&ledger)`, read locally. Two machines allocate the same identifier to two
different human beings, and ADR-0051 is locked on sequential identifiers, so the sequence cannot
absorb concurrency — the concurrency has to go. That is the whole argument for a cold spare, and
it is stronger than "a spare is simpler".

**And docket was already multi-user, which settled the shape.** The server binds loopback,
refuses a routable address, and runs one connection at a time because *"this process is the only
writer to four append-only hash chains"*. Many operators, one machine, one process, one writer.
The single-writer invariant a second node would have broken was already being relied upon and
already argued for on its own merits.

**The hazard was live, on the machine in use.** That serial loop covers the server and not the
CLI, and there was no file locking anywhere. Ten concurrent `docket new` runs, five pairs:
duplicate identifiers in **five of five**, forked chains in four. Not a narrow race — the common
case, because `append_chain` is read, seal-onto-last, append with nothing in between.

**The sharpest finding came from reading the failure rather than the summary.** `docket check`
reported the forks and *not* the duplicates. One pair had appended cleanly — two stale reads that
happened to serialise — producing **the same person identifier twice with a correct chain, which
check passed in silence.** `docket.identifiers_are_never_reused` had been declared against
ADR-0051 on 2026-08-19 and never implemented; detection was an accident of the torn chain that
usually accompanies a race. A property that load-bearing has to be asserted.

**Three defects of my own, each found by running the thing rather than reading it.**

The uniqueness check's first version counted *rows* and called an erasure's appended row a
duplicate — caught by `every_chain_still_verifies_after_an_erasure`, which is the test that
exists for exactly that append. Reuse is one identifier across two **slugs**.

The restore guard tested `into.register()` — `register/issued.jsonl` — which does not exist until
something has been issued. A store holding three people and no documents therefore looked empty,
so the second restore overwrote `people.jsonl` and **silently deleted a person allocated after
the backup**. Found by running the restore twice in the simulation, not by reading the guard. A
guard has to be about the files being written, not about a file that stands for them.

And I reported `exit=0` for a refusal that had actually exited 2, because `$?` after a pipe is
the exit code of `tail`. The same defect `CLAUDE.md` already warns about for `gates.sh`, made in
a shell one-liner minutes after reading it.

**What I did not build, and why it is debt rather than an omission.** ADR-0072 decides five
things; code enforces four. The node registry and the off-authority write refusal (D1, D4) are
prose, because which machine is authoritative is estate inventory and RFC-0021 is still
`proposed` — building against a proposed naming scheme would bind docket to a name that may
change. D-52 records it, including the consequence: nothing yet stops a second docket started
against a restored store on the spare.

**What the trade still is.** Replication is safe at all only because ADR-0069 made erasure
key-destruction rather than file-deletion, so ciphertext can be copied anywhere. The per-person
keys are the one row of the table with no cell to fill: a key you can restore is a key you cannot
prove you destroyed. Multi-node sharpens that; it does not soften it, and the ADR says so rather
than implying a spare closed it.

**Skills and tools used.** `age` and `age-keygen` for the real encryption path; `SOPS_AGE_KEY_FILE`
to rehearse with a throwaway key so the operator's own was never touched; `tar` into a scratch
root to build a disposable second machine; a Python `assert old in s` on every text replace, after
last session's silent no-ops; two deliberate mutations (`if false &&` on the merge guard,
`keys` reclassified `Replicate`) to confirm the new tests fail when the guards are removed.

371 tests. Gates green. `check-knowledge.sh` passes.

## 2026-09-07 — Two containers, and the one thing they could not prove

**The operator asked to make it production-ready by running two nodes connected over a network.
Half of that was the missing piece and half of it was the thing the morning's work had just ruled
out**, so it was worth separating rather than building.

Two docket binaries reachable over a network is multi-writer — the ADR-0051 collision closed
hours earlier — and `require_loopback` refuses a routable bind anyway, deliberately, because the
server has no authentication of its own. A spare that *runs* docket is not a spare.

**What was right is the isolation.** Two environments with separate filesystems, the bundle
crossing a network as bytes, is exactly the leg `tests/node.rs` cannot reach: those tests share a
kernel and a disk, so they prove the cryptography and not that a spare needs nothing from the
machine it is replacing. So the network carries the *bundle*, not docket traffic.

Node A creates three people, backs up, pushes the bundle to an off-host container, and is then
deleted along with its volume. Node B pulls it, restores, verifies, and allocates
`PAR-P-000004-4`. Thirteen assertions, four negative.

**Three failures on the way, and each one was informative.**

A glibc version error that read like a docket bug: `rust:1.97.1-slim` is trixie and the runtime
was bookworm. Both stages are now pinned, with the reason in the Dockerfile, because the symptom
names no cause.

**Two assertions were vacuous when first written, which is the failure mode a harness is most
prone to.** The plaintext check used `grep -v`, which is line-based, so it flagged every
encrypted file — the check that was meant to prove ciphertext had left the machine was instead
reporting a leak that did not exist. And the reachability check curled a container that was not
serving at all: the failure it recorded proved only that nothing was listening. Both are fixed,
and the ciphertext one was confirmed by injecting a plaintext file and watching it fail. A green
assertion nobody has made fail is not evidence.

**And one failure was the system working.** The reachability test first ran on the node sharing a
store with a running server, so `serve` was refused by the lock and the API never came up. I had
written a check that reported the lock as a defect. It now runs on its own store.

**What the containers cannot prove, stated in D-53 rather than implied away.** The Sync.com leg.
RFC-0038 records the vendor's client as Windows and Mac only, so it can never run in a Linux
container; the off-host container carries the same bytes over the same kind of hop and is a
stand-in, not the vendor. Exactly one manual rehearsal remains.

**Skills and tools used.** Docker with a private bridge network and per-node volumes so no two
nodes could see one filesystem; `age-keygen` inside the image for a real estate key and a real
wrong key; `tar` streamed through `docker exec` so the bundle moved without either node mounting
the other; `python3 -m http.server` as the off-host stand-in; a deliberate plaintext injection to
prove one assertion could fail.

371 tests plus 13 container assertions. Gates green.

---

## 2026-09-08 — the front half gets a playbook, and it is honest about being manual

`HR-CHECKLIST.md` begins at "1 · Offer". Everything before that — writing a posting, publishing
it, taking applications, deciding — was undocumented, and the CLI has no `post`, `apply` or
`shortlist` subcommand to document. ADR-0063 named this the front half of docket and left it for
Phase B. `HIRING-PLAYBOOK.md` covers it now.

**Capability was retrieved before it was written, per STD-0013.** `HR-CHECKLIST.md` (123 lines)
and `SOP.md` (837) already cover offer through alumni, so the playbook does not restate them: it
ends at section 5 by handing over to `HR-CHECKLIST.md` §1 at the exact line where docket takes
over. Two documents meeting at one seam, not two overlapping ones.

**The load-bearing decision was to describe the manual path rather than the designed one.**
`api/apply` is built and deliberately inert — gate 1 returns 503 with *"Until then, apply by
email"* unless `PARIDYUM_BACKUPS_CONFIRMED=1`, which asserts a rehearsed off-host restore
(ADR-0071 D3, ADR-0068 §5). So intake is a mailbox today, and sections 3 and 4 of the playbook
have no commands in them at all. Writing them as though the store existed would have produced a
checklist nobody could follow. The future state is confined to one closing section that says
exactly which sections change and which do not.

**Two joins between repositories are stated because neither is enforced.** `engagement.class` on
the posting must match the class passed to `docket new`, and `engagement.job_id` in the person
record must be the posting id — the only join between `web-paridyum/jobs/` and
`tool-docket/people/`. Both are checkboxes because nothing checks them; `jobs.ts` already carries
a hand-copied mirror of `ENGAGEMENT_CLASSES` and `DOCUMENT_PROVIDES` with a comment admitting a
copy goes stale silently.

**Skills and tools used.** `paridyum-platform:new-decision` for ADR-0074 and
`paridyum-platform:design-change` for RFC-0043 earlier in the session, which is where the
surface split this playbook assumes was decided; `check-knowledge.sh` after each; grep and sed
over `src/main.rs` to establish that no hiring subcommand exists rather than assuming it, and
over `api/apply/route.ts` to read the three gates in the order the code applies them.

No code changed. Documentation only.

**Later the same day — `RUNNING.md`, because the README answered the wrong question.**
`README.md` is 422 lines across 20 headings and had no install step, no first command and no
statement of which machine to run on. STD-0002 requires a fixed README shape with "Run locally"
and "Test"; this repository had neither, so the entry path was 400 lines of capability with no
door. The setup facts existed but were scattered across ADR-0072 (one authoritative node, the
`O_EXCL` lock), ADR-0068 D1 (loopback, SSH forwarding as the authentication), ADR-0071 (`.age`
before anything leaves), `scripts/serve.sh`'s header comment (build-first, refuse-occupied-port,
both written after real faults) and `config.rs` (`DOCKET_PEOPLE`, `DOCKET_STORE`). `RUNNING.md`
collects them; the README gains four lines and a pointer rather than a rewrite.

**The section that matters is §1, and it is first for that reason.** "Which machine" is the
question that goes wrong irreversibly — two docket instances fork a hash chain, and a forked
chain reads as tampering later. Everything else in the file is recoverable.

## 2026-09-09 — Four real people, and the guard D-51 had put in the wrong place

**Four officers are now recorded**, and the register stopped being a demonstration. That changed
which open debt was worth taking: not the node registry, but the one standing between four real
people and a single laptop.

**D-51 had reasoned itself into the wrong boundary, and its own words contained the correction.**
The entry said the check could not live in docket, because docket cannot see what Sync received —
true — and that what could exist is *"a guard on the other side, in whatever performs the
upload"*. The thing performing the upload is a person copying files into a watched folder. Filling
that folder is an operation docket can perform, and therefore one it can guard. The vendor's leg
was never the part that needed checking.

**The refusal worth building was the one that looks backwards.** Four of the five inspect what is
being sent. Only the never-list check inspects what is *already at the destination*, and it is the
only thing that can catch a `store/keys/` uploaded once, by hand, months ago — after which no
erasure the system performs is real and nothing would ever have said so.

**Two defects, both found by tests, both producing a refusal that lied.** The plaintext check
bailed inside the copy loop while printing "Nothing was uploaded", which was false for every file
the walk had already reached; a refusal that misreports what it did is worse than no refusal.
And the inside-the-store guard used `starts_with`, which is purely lexical — a relative
`store/uploads` walked straight past it and wrote into the directory it existed to protect. I
found that one by running the command against the real store and then having to delete what it
had created. The first fix absolutised the path and a `..`-laden path still defeated it, so
destinations are now normalised lexically.

**On the officer records.** A date of birth reading 27/01/2026 made the record structurally
impossible rather than merely wrong: DPDP s9 demanded a guardian and the officer class forbids
holding one, commented *"an officer is not a minor"*. Two rules disagreeing was the signal the
data was wrong. I asked rather than assuming 2006, because the year is printed on both documents.

**And the identifiers were reassigned, which is only possible because nothing has been issued.**
The request was to reorder four people across four identifiers, which locked ADR-0051 forbids as
reuse. Checking first showed what the prohibition is actually about: no document, no serial, no
signature, no tombstone, and every address field still null. ADR-0051's harm is a certificate that
stops naming the same person, and no certificate existed. So the ledger was reset and re-allocated
rather than edited — editing an append-only chain would have invalidated every digest after the
change — and the filled records survived untouched, because they are keyed by slug and a slug is a
filename, never an identity. That window is now closed: the first issuance ends it.

**`pre_existing_ip` is held as FILL-ME on all four deliberately**, which keeps `docket check` red.
That red is accurate — the records are incomplete — and it converts "remember not to issue" into
"cannot issue", with `--force` explicitly not applying.

**Skills and tools used.** `docket check` as the arbiter rather than my own reading of the
registry; deliberate mutations to confirm new tests fail; the real store as a test subject once,
which is how the relative-path defect surfaced and also why it had to be cleaned up by hand;
`check-knowledge.sh` to catch that citing a check in a new ADR left its `blocks:` list stale.

378 tests. fmt, clippy and test green; `check` red on four held records.

## 2026-09-09 (later) — A second company, and the decision that had already been taken

**Paridyum Cyber arrived, and the first useful thing was finding out how much of this was already
decided.** ADR-0051 labels the first segment of `PAR-P-000147-3` as *organisation* and states that
*"one person may hold many engagements; the archive stays whole across all of them."* Both
sentences were written for a single-company estate and both turned out to be the answer. Nothing
locked needed superseding, and the engagement identifier's shape did not change at all — only the
value of a segment that already existed.

**The obvious design was the wrong one, and its wrongness was checkable.** One docket store per
company is what anybody would reach for. It means two ledgers, so a person in both is allocated
two person identifiers — Rohan's archive splits down the middle, and the check that identifiers
are never reused cannot see across the boundary it would most need to. That is the exact harm
ADR-0051 exists to prevent, arrived at by convenience.

**The real work was not the registries.** `reengage` refused any second engagement while one was
open — *"separate them first"* — which reads as a rule about sequence and is actually an assertion
that a person works for one entity. That assertion was never decided and is now false. Narrowing it
to one open engagement **per company** keeps the protection it was actually providing (two open
internships at one company is a record nobody can reason about) and drops the part nobody had
chosen.

**Two collapses that resolved themselves into data.** Priyanshu holds CEO and Director at once, so
RFC-0042's rule — the CEO signs the Director's own documents — had nothing left to point at. Rather
than branch on it, he gets both entries in `signatories.yaml`; for his own documents both are the
subject, and the substitution that already existed falls through to the CTO. The operator's
decision and the code's behaviour are the same object. A test asserts the NDO resolves to `cto`.

**And the roster was quietly lying within minutes of the change working.** `identity::current`
keeps the last ledger row per person, which is correct for an intern who becomes an employee and
wrong for a man holding two offices: it showed Rohan at Paridyum Cyber and omitted Paridyum
entirely. Found by reading the roster after the ledger was already right — the ledger had five
correct rows and the view showed four. The summary then said "5 people" for four officers, which
is the same defect one line further down.

**On resetting the ledger a second time.** Ruhani's engagement had been opened at Paridyum before
she moved to Cyber, and an engagement's company is immutable by decision. Nothing is issued, no
address is provisioned, and every appointment date is still in the future, so the reset cost
nothing — and the person identifiers the operator chose were preserved exactly, which is why no
address changed. That window closes at the first issuance, and I said so rather than leaving it to
be discovered.

**What I did not do.** The officers page still names one company, and re-exporting now would
publish six officers of two companies as one flat list — worse than a stale page. D-55 holds it.
And a person record still has one `engagement:` block: Rohan being CTO of both is true of both, so
it works by luck, and D-54 names the schema choice rather than guessing at it.

**Skills and tools used.** `grep` against the locked ADR before designing anything, which is what
turned a supersede into a reading; `docket check` as the arbiter after every registry edit; four
new tests in `tests/officers.rs` including the two-companies-one-identifier case; and a careful
note that zsh does not word-split unquoted parameters, after a loop silently passed an empty
`--company` four times.

382 tests. fmt, clippy and test green; `check` red on four held records, plus the D-55 warning.

## 2026-09-09 (later still) — "Paridyum owns every other company"

**Said as an aside, and it is the sentence that decides who may sign.** ADR-0077 decision 4
resolves a subsidiary's countersignature through ownership — Paridyum's Director signs for the
holding company — so a company whose ownership does not reach the root has no answer to the
question of who may execute its documents. It would find that out at its first issuance, months
after somebody added the entry.

So it became an invariant rather than a comment: exactly one company declares no `owned_by`, every
other reaches it by following the chain, no chain loops, and no owner names a company that is not
declared. Four tests, each swapping `org.yaml` for a registry that breaks one rule.

**`owned_by` names a company rather than asserting `PAR` everywhere.** A company owned by Paridyum
Cyber is still owned by Paridyum, and a rule that demanded the root be named directly would be
wrong the first time the group is two levels deep.

**Widening the check found a second collision nobody had asked about.** `organisation` had only
ever validated the default company — so a subsidiary with no jurisdiction would have produced a
governing-law clause naming no court, visible only on that company's first document. Validating
every company made the shared-serial-format problem obvious: each company keeps its own per-kind
counter, so two companies with `PAR/{kind}/{year}/{seq}` issue `PAR/NDA/2026/0001` twice, to
different people. The serial is the string a third party verifies a document by. That check exists
now because the loop made it visible, not because anybody predicted it.

**A shell lesson, twice in one day.** Backticks inside a double-quoted `python -c` are executed by
zsh before python sees them, which silently ate a doc comment's `` `check::run` `` and left the
sentence starting "Test seam:  is the front door". Heredocs for anything containing backticks;
`python -c` only for the trivial case.

387 tests. fmt, clippy and test green; `check` red on the four held records.

## 2026-09-12 — Asked for a timeline, found a live correctness hole instead

**The question was whether docket should have a timeline or graph view. Checking before answering
is what made the answer easy, and it was not the answer the question expected.**

`docket record` already prints a dated history — the hash chain IS a timeline. ADR-0065 already
assigns structural diagrams to archify, with hand-authored `lifecycle` and `dataflow` IR explicitly
legal, so a renderer here would duplicate a tool pinned four weeks ago. And a graph of two
companies and four people is a screenshot, not a feature. The estate's own idiom applies: a planned
region is not a region, it is a sales conversation.

**But running the per-person commands to check what a view would even show surfaced this.**

    docket status rohan-batra
      engagement PCY-OFC-2026-0002    <- Paridyum Cyber

`identify` took the last ledger row for a slug. That is correct while a person holds one engagement
at a time — an intern who becomes an employee should resolve to the employee row — and it was
silently wrong from the moment concurrent engagements landed, an hour earlier, in code I had
written. Every per-person command reads through it, so Rohan's Paridyum office was unreachable, and
**`issue` derives the company from the engagement**: his Paridyum NDA would have carried Cyber's
letterhead, Cyber's jurisdiction and a `PCY/` serial. An issued document is corrected only by
reissuing under a new serial.

**I had already fixed this exact defect once, in `current()`, for the roster — and missed that
`identify` is a separate path doing the same thing.** One fix, one sibling left behind. The lesson
is not "be careful"; it is that a recency-based lookup is a *pattern*, and finding one instance
should have prompted a grep for the rest.

**Refusing beat resolving.** Ambiguity now errors and names both engagements rather than picking
one, because which company a command concerns decides the letterhead, the jurisdiction and the
serial. A global `--company` answers it — one flag, reused by `new` and `reengage` for allocation
and by everything else for disambiguation, rather than a per-command flag repeated eleven times.
A company the person holds no office at is an error, never a quiet fallback to the other.

**On the view question, the honest position is recorded rather than implied:** no graph, no general
timeline, with a named trigger — a person holding engagements at three or more companies, or the
group going two ownership levels deep. The one view I would still argue for is a forward
obligations report, because `retention` answers what is past due and nothing answers what falls due
next, and those are DPDP deadlines with legal consequence. Even that is a sorted list.

**Process note.** Threading one parameter through nineteen call sites with regexes double-patched
several and mis-inserted others into the middle of function bodies. Reading the compiler's error
list as a worklist rather than trusting the scripts is what kept it correct; three signatures had
to be repaired by hand after a script put `company: Option<&str>,` on the line after a closing
brace.

388 tests. fmt, clippy and test green; `check` red on the four held records.

## 2026-09-12 (later) — Working the debt list, and the first thing it found was a red CI

**Asked to implement the open engineering debt, the first useful act was `gh run list`.** CI had
been failing for five days — three consecutive red runs, including one from that morning — and
nothing in this checkout says so. `gates.sh` runs four local gates and reports them green, which
is exactly the trap D-28 had written down in 2026-08-20 and which nobody had come back to read:
*"do not treat a green local gate as evidence that CI passes."* The entry's status line was stale
and its warning was the live part.

**The failure was a fixture, not a test.** `the_keystore_is_never_in_what_gets_uploaded` reads two
paths and asserts about a layout, and it failed on every runner because `Sandbox::new` called
`age-keygen` unconditionally. Green everywhere `age` is installed, red everywhere it is not. That
is D-45's defect a second time, and the second occurrence is what made the general fix obvious:
`scripts/ci-parity.sh`, which runs the suite with `PATH` stripped to the Rust toolchain. Both
instances had been found by pushing and reading a log.

**Then D-30 closed by being proved true.** Tectonic had never been run against this repository —
no machine on the estate had it — so I installed it, and found two defects stacked.

`TEXINPUTS` was set for both engines and **Tectonic does not read it**. Every render failed with
`File 'paridyum.cls' not found`. The engine has `-Z search-path`, documented as "like TEXINPUTS",
and nothing had ever passed it. Fixed.

With that fixed, the real one: `paridyum.cls` uses `pdfx` for PDF/A, and under XeTeX it fails with
`CreationDate is not properly supported`. Seven of thirteen issuance tests pass; six cannot render
at all. **So the engine docket advertises as its supported alternative has never worked**, and
`detect` still offers it — which is now D-57 rather than a surprise for whoever first runs docket
on a machine without TeX Live.

**That finding chose the CI design.** I had written the render job around Tectonic for the obvious
reason: a 10MB checksummed binary against a multi-hundred-megabyte apt install. Having to throw it
away produced a better argument than the one I started with — **pdflatex is what every developer
machine runs and what has produced every document this estate has issued, so a CI job rendering
with a different engine would be verifying something nobody ships.** The registry entry says
`texlive`, not `tectonic`, and its exit path cites the experiment rather than asserting a
preference.

**A correction I nearly did not make.** Closing D-35 made a comment in `ci.yml` wrong: it promised
coverage would rise from 60 to 70 "when D-35 is closed and CI can render". D-35 is closed, CI does
render, and coverage cannot move — because **rendering and coverage are two different jobs**, and
only one of them is in docket's own workflow. The other belongs to the shared `infra-ci`
workflow in another repository. The comment is corrected in place so the next reader does not
repeat the inference.

**On D-49, a distinction worth keeping.** The check asserts against the `.gitignore` TEXT rather
than shelling out to `git check-ignore`, because those answer different questions: a file already
tracked is reported as not ignored whether or not the negation line exists. That difference had
already wasted one mutation test in an earlier session, which passed while proving nothing.

**Closed: D-28, D-30, D-35, D-45, D-49. Narrowed: D-36, D-41. Opened: D-57.**

**Skills and tools used.** `gh run list` / `gh run view --log-failed` to read a CI failure without
pushing; `gh api` to find a real pinned release rather than inventing a version; a stripped-`PATH`
harness to reproduce a runner locally; an actual Tectonic install to test a claim rather than
reason about it; and a mutation of `.gitignore` to prove the new check fails when it should.

389 tests. fmt, clippy and test green; `check` red on the four held records.

## 2026-09-13 — How a CEO is fired, and an entry I got wrong by not measuring it

**The question had no answer, and finding that out was most of the value.** RFC-0042 decided how an
officer is appointed — instrument, registry entry, human under C-0004 — and said nothing about
removal beyond the record's shape. The asymmetry is what made it a defect rather than a gap: an
appointment without an instrument is a `docket check` finding, and a removal without one was
indistinguishable from a correct removal.

**Simulating the firing found two live defects that reading would not have.** I set Priyanshu
`active: false` and resolved a document:

    resolve_signatories(...) -> ["director"]

He would still have been printed as the signatory on every document issued afterwards. Routing
names an OFFICE; only the register knows who holds it — and the substitution only ever asked "is
this the subject of the document", never "is this office vacant". `check` was silent because
`signed_by` is validated against every signatory rather than the active ones.

And `ceased_on` did not exist at all. RFC-0042 §5 says ceasing is `active: false` **plus** a date;
the struct had no such field, so half the rule had been unimplementable since August.

**Then the decision separated two things I could have conflated.** The operator's answer — Rohan
absolute, Priyanshu second — is about who may DECIDE. Rohan is `cto` in the signatory register and
third in Paridyum's fallback order. The tempting move was to reorder that list to match the
authority, and it would have been wrong: a document is executed by whoever holds the office it
routes to, because that is what lets the register say who held an office on a date. Reordering
would have made Rohan sign everything and changed every document Paridyum issues. ADR-0087 keeps
them apart and says why.

**And I had to retract D-59.** I had written — and put in a summary to the operator — that removing
Priyanshu would leave "nobody to countersign Ruhani's own documents". Measured:

    PCY, Priyanshu removed — ruhani: ["cyber-cto"]
    PCY, Priyanshu removed — rohan:  ["cyber-ceo"]

The chain is two deep and survives losing any one of three. I had reasoned from the registry's
ORDER — he is first in both lists — without following what the substitution does when the first
entry is vacant. That is a specific way to be wrong: reading a list's shape instead of the code
that reads it. The entry is kept with the correction rather than deleted, and the property it was
groping for is now a check — fewer than two PEOPLE among a company's active signatories is an
error, counted by person because Priyanshu is two rows and one human.

**A note on what I did not touch.** `check-knowledge.sh` reports three errors in `ADR-0086` and
`STD-0022`, both untracked files from another session and possibly mid-edit. Not mine to fix
silently.

**Skills and tools used.** Editing the real registry to simulate a removal and restoring it from a
backup; an existing test as the probe — `the_ceo_and_director_being_one_person_falls_through_to_
the_cto` failing with `["director"]` was the proof; a throwaway printing test to measure the Cyber
chain before writing a claim about it, which is what caught D-59; and `mem::forget` being denied by
STD-0003, which was right — the `Registry` is fully parsed and the temp directory can simply drop.

396 tests. fmt, clippy and test green; `check` red on the four held records.

## 2026-09-14 — an honest correction, a port design, and the fourth chain

**The correction.** An earlier autonomous run reported "the complete, e2e-tested backend is done."
Reading the real HTTP layer back showed that was overstated. The server the console actually speaks
to is `adapters/http/` — a `std::net` JSON API serving the whole contract (roster with tallies, a
person dossier with steps/blockers/documents/history, `/check`, `/retention`, `/emails`, `/form`, and
the onboard/sign/advance/enrol writes), every route calling a tested `application::` function over the
**file store**. What the run built instead — `served.rs` + `bin/serve.rs` — is a minimal **axum**
server exposing two routes straight against Postgres, bypassing the entire application layer and all
its rules. It cannot back the console. `pg.rs` and `auth.rs` are sound foundations; the served
backend is a spike. Wiring the console's OIDC on top of it would have been building on sand.

**The design.** ADR-0093's real work — the store move the ADR itself calls "the single largest change
in the estate" (~200 lines of `fs::` across ~10 call sites, 15 application modules, 209 file-store
sites) — is not done. Because governance precedes implementation, that is now designed before it is
built: `docs/superpowers/specs/2026-09-13-docket-postgres-port-design.md`. Its core decision is to
keep the application layer **synchronous** behind a `Store` port and drive `sqlx` from a shared
runtime via `block_on`, rather than rewrite the fifteen modules async — the code most dangerous to
rewrite is exactly the tested rules, and Path B leaves them bit-for-bit while reusing the rich
contract the console already speaks. The axum spike is removed by that design; the real server gains
thread-per-request and ADR-0092 token auth.

**Step 1, shipped and validated.** Writing that design caught a concrete gap: docket has **four**
sealed chains, and the Postgres foundation covered three. The **signature ledger** had no table in
migration 0006, no `append`/`read` in `pg.rs`, and no store at all. Closed: `docket_signatures` added
to 0006 (platform-tenancy); `append_signature`/`read_signatures` added to `pg.rs`; a JSON-round-trip
unit test added to `domain/signature.rs` (the fourth of four, holding the signature record — with its
kebab-case `Method` and `skip_serializing_if` scan fields — to the same seal-preservation bar); and a
signature-chain integration test added to `tests/pg_store.rs`.

Proven against a real Postgres 17 (migrations 0001–0006 applied as role `paridyum`): 5/5 integration
tests green — all four chains seal-intact through JSONB and RLS-isolated by tenant — plus 237 lib
tests, fmt and clippy clean, and `check-migrations.sh` passing.

**Skills and tools used.** Retrieving the real architecture before deciding (reading `adapters/http/`
and the 2026-08-21 served-platform design rather than inferring from the spike); a throwaway
`postgres:17` in Docker with the `uuidv7()` shim to validate the migration and the new chain
end-to-end; `check-migrations.sh` as the governed structural gate; `cargo clippy --all-targets` which
caught a `vec![]`-over-array on the new test.

## 2026-09-14 (later) — the port is threaded through, and the real server goes served (ADR-0093 steps 2, 4, 5)

The design's Path B, built. The `Store` port (four sealed chains + the person document, `FileStore`
and `PgStore` behind it) was already defined and tested; what remained was to make the application
layer and the served backend USE it. That is the ~200-line change ADR-0093 calls the largest in the
estate, done module by module against the compiler.

**Records through the port, blobs left on `Paths`.** Every `application::` function that reads or
writes a RECORD now takes a `store: &dyn Store` and calls `store.foo()` where it used to call
`store::foo(paths, …)`; every `fs::`/`paths.` BLOB call (issued PDFs, packs, scans, generated
indexes, templates) was left exactly as it was (design §4). The cut is by concern, not by function —
most writers gained a `Store` and kept their `Paths`. The record appends now go through
`store.append_*`, which **seals internally**: `issue.rs` no longer builds a pre-sealed entry and
inserts it (that was the fork the port exists to prevent) — it builds the entry unsealed and hands it
to `append_register`, where the Postgres store seals under a per-chain advisory lock in one
transaction and the file store seals single-writer. Person creation moved onto the port too:
`enrol::write_record` is now `store.create_person`, and `personnel::create` asks the STORE whether a
record exists (not the filesystem) and writes a default record THROUGH the port — so a served enrol
lands in `docket_persons`, not as a stray file on the server's disk. `erase`'s rewrite-and-reseal
became `store.reseal_events` (§6).

**The served backend, evolved not replaced.** `router::Context` gained `store: &dyn Store` and an
optional `verifier`; `Store: Sync` (a supertrait) and a `dyn Clock + Sync` let a `&Context` cross the
new thread-per-request server, which uses `std::thread::scope` so the borrowed context needs no `Arc`.
`operator::resolve` now branches on the verifier: with one present (the served backend) it resolves
the operator from a **verified Bearer token** — `OperatorVerifier::verify` → `registry::by_subject` →
the operator id, missing/invalid → 401, unbound subject → 403 (a new `NOT_AUTHENTICATED` marker gives
the router its 401); with none (loopback break-glass) it keeps the `X-Docket-Operator` attribution
header. The header path is not read at all when a verifier is present, so a served route cannot
degrade to an unverified claim (design §5). `require_loopback` became `permit_bind`: a routable bind
is allowed only when a verifier is present, refused otherwise.

**The spike, deleted.** `src/served.rs` and the axum `bin/serve.rs` are gone; `axum` (dep) and `tower`
(dev-dep) removed with them (nothing else used them). The new `bin/serve` (name kept — the deploy
chart depends on it) builds a multi-threaded tokio runtime it keeps alive, opens a `PgStore` on its
handle, fetches the JWKS once and builds the verifier, loads the registry, and serves the **real**
`adapters/http` router with a `Context` carrying `Some(verifier)` on `DOCKET_BIND` — same env vars as
before. `tests/served_api.rs` was rewritten to drive that real router (a `PgStore` + a test verifier,
a signed operator token, raw HTTP through `request::parse`): it proves missing token → 401, unbound
subject → 403, and a bound operator creating a person then seeing exactly them on the roster (RLS
scoped) — the spike's happy path, now over the whole rule stack.

**The CLI** constructs a `FileStore::new(&paths)` at each command and passes it to the threaded
application functions; every command works on the file store. The PgStore-CLI (a runtime + a
`PgStore` when `DOCKET_DATABASE_URL` is set) is tracked debt (D-61), as the design permits.

**What deferred.** `PgStore::reseal_events` refuses with a clear error — erasing on Postgres needs a
cross-repo platform-tenancy migration 0007 (an append-only delete exception), which must not be made
here (D-60); the file/CLI erase path keeps working through the port. `advance` still edits the person
body on disk to preserve its DPDP field-purpose comments, so the served stage-change waits on a
port operation for the raw body (D-62). The loopback `docket serve` is now thread-per-request over the
file store, dropping ADR-0072's serialisation for that break-glass path (D-63). PDF bytes still stay
host-local (design §8).

**Gates.** `scripts/gates.sh` all green — fmt, clippy (`--all-targets`, zero warnings; one justified
`too_many_arguments` allow on `sign::record` now that the store is a parameter), 239 lib + all
integration tests, and `docket check` (which needed a `default-run = "docket"` in Cargo.toml now that
two binaries ship). Against the throwaway Postgres 17 (migrations 0001–0006, localhost:5433):
`--test pg_store` 7/7 and `--test served_api` 1/1 green — the chain-preservation gate (all four chains
seal-intact through the application layer on `PgStore`) and the end-to-end auth+create+roster path.

**Skills and tools used.** Reading the port design and every touched module before editing (governance
precedes implementation); the compiler as the refactor's guide (a call site that is not migrated does
not compile — Path B's stated safety property, confirmed); a scripted mechanical transform for the
~150 test call sites (wrapping each `&paths` in `&FileStore::new(&paths)`), then `cargo fmt`;
`scripts/gates.sh` as the single un-piped gate (CLAUDE.md); the throwaway Postgres for the record
port's real proof.

## 2026-09-14 (later still) — the break-glass CLI reaches the same Postgres (ADR-0093 §3, D-61 resolved)

The previous entry left the CLI on `FileStore` and named the gap as D-61: a `PgStore` when
`DOCKET_DATABASE_URL` is set. This closes it. The CLI is now a break-glass client of the SAME database
the served backend serves, and defaults to local files when the variable is unset — offline break-glass,
unchanged.

**Store built once, threaded down.** The ~25 handlers each opened `let store = FileStore::new(paths)`;
that construction moved into `run()`, which builds the backend once and threads a `store: &dyn Store`
through `dispatch` → `route` → `route_person` → every handler. The handlers still take `&Paths` where
they touch BLOBs (issued PDFs, packs, scans) — records via the store, blobs via paths, the split the
app layer already draws. `--tui` browse and `docket serve` (loopback break-glass) both take the same
threaded store, so both run on either backend.

**Backend chosen by environment.** `open_backend`: `DOCKET_DATABASE_URL` set → build a multi-thread
tokio runtime kept alive for the process (as `bin/serve` does), open the pool on it, read the tenant
from `DOCKET_TENANT_ID` and construct `PgStore::new(pool, tenant, handle)`; unset → `FileStore`. A DB
URL without a valid `DOCKET_TENANT_ID` uuid is refused, not defaulted — every row is tenant-scoped by
the 0006 RLS policies, and a guessed tenant is how one operator's write lands against another's
records. One stderr line names the store each run, and for Postgres the host (credentials stripped)
and tenant, so an operator cannot mistake the cluster for local files.

**The write lock stayed on the file store.** `lock::acquire` is skipped when the backend is Postgres:
the database's advisory-locked, atomic seal-append is the concurrency control there (ADR-0093
§decision-2); a lockfile beside a database store would serialise nothing and leave a stray file.

**Two commands refuse on Postgres, clearly.** `advance` rewrites `people/<slug>.yaml` as text to keep
its DPDP purpose comments (D-62) and `erase` re-seals the event log by rewriting it (D-60, needs the
cross-repo migration 0007). `unavailable_on_pg` refuses both up front with the reason and the remedy,
so `advance` no longer fails deep in an `fs` read and `erase` no longer deletes host-local blobs before
its re-seal fails. `PgStore::reseal_events` still carries its own clear refusal as the backstop.

**Proven against the running throwaway Postgres (localhost:5433, migrations 0001–0006).** `--test
pg_store` 7/7. Then the CLI itself, tenant seeded the way the test seeds one: `docket roster` returns
cleanly with the banner; `docket new asha-verma intern` lands the person in `docket_persons` and its
sealed identity (`PAR-P-000001-X`) and `created` event in `docket_identities`/`docket_events` — read
back only after setting the RLS session variables, which is the isolation working — with NO file
written to `people/`; `roster` then shows the person; `advance` and `erase` refuse with their
D-62/D-60 messages. `scripts/gates.sh` green in file mode (fmt, clippy `--all-targets` zero warnings —
two justified `too_many_arguments` allows where the store parameter pushed a dispatch handler to nine,
tests, `docket check`).

**Skills and tools used.** Reading the whole `Store`/`PgStore`/`bin/serve` trio before touching a line
(governance precedes implementation); the compiler as the guide through the ~25 handler signatures;
`unavailable_on_pg` and the lock gate written as explicit, greppable functions in the style of the
existing `writes()`; `scripts/gates.sh` as the single un-piped gate; the throwaway Postgres and a
seeded tenant for the CLI's real end-to-end proof.

## 2026-09-14 (later still) — the CLI gains a third mode: an authenticated API client (ADR-0097)

**Why.** ADR-0093 §3 said the CLI is "a break-glass client of the same database", which quietly implied
everyone who uses the CLI holds the production database URL. ADR-0097 corrects that: most operators —
HR included — must use docket WITHOUT the database link. So the CLI grows a third mode, an authenticated
HTTP client of a remote docket's `/api/*`, reached with an operator token and no database secret. This
is the console's position (ADR-0092), now available from the command line.

**Mode precedence, chosen before any store opens (ADR-0097 §2).** `classify(api, database)` is a pure
function with its own tests: `DOCKET_API_URL` set → API client; else `DOCKET_DATABASE_URL` → the
break-glass `PgStore` (banner unchanged, still says the word "break-glass"); else the offline
`FileStore`. One stderr banner per run names the mode. **The crucial property:** API mode does NOT load
the `Registry` or open a store, and skips `--by`/operator resolution — the token is the attribution. So
`run()` handles `login`/`logout` and the whole API branch BEFORE `Registry::load`/`open_backend`.

**The device flow and the token cache (ADR-0097 §3, REQ-0001).** `docket login` runs the OIDC device
authorization grant against `id.paridyum.com`: `adapters::oidc` does one round-trip each in
`request_device_code` and `poll_once` (→ Pending/SlowDown/Denied/Expired/Approved), and the poll LOOP
lives in `main` using `thread::park_timeout` — never `thread::sleep`, which clippy.toml bans. The token
it returns is cached by `adapters::token` at `$XDG_CACHE_HOME/docket/token.json`, 0600, with a 30s
expiry skew and a `DOCKET_TOKEN` env override for service accounts. The token is never printed. The
scope carries `urn:zitadel:iam:org:project:id:{project}:aud` so the token is audienced for
paridyum-operations and passes the audience `auth.rs` pins.

**The API client (`adapters::api`).** `ApiClient` on `reqwest::blocking` (the new "blocking" feature,
rustls kept). It mirrors the wire types rather than sharing `http::dto` — the same deliberate
duplication ADR-0093 keeps between docket and console-docket, so a server field rename cannot silently
retrack a client. `Authorization: Bearer <token>`; 409 → a rendered `Refused` (exit 2), 401 → an error
hinting `docket login`, other non-2xx → the server's `{error}`. The `render_*` functions are pure
`-> String`, unit-tested against a local `TcpListener` mock — no live backend.

**Unavailable over the API, refused up front.** `unavailable_over_api` (pure, tested) names the
commands that need local blobs, the filesystem layout, or the estate key: issue, render, verify, pack,
index, team, vault*, store*, addresses, reengage, event, separate, erase, serve — plus `--tui` and
`sign --scan`, handled at their call sites. `sign` over the API is person-scoped, so it gained an
optional `--for <slug>`, required in API mode.

**Verification.** `scripts/gates.sh`: fmt ok, clippy `--all-targets` zero warnings, test ok (259 lib
tests including the new token/oidc/api suites, 5 main mode-precedence/refusal tests). `docket check`
FAILED only on pre-existing local record data (`pre_existing_ip` placeholders in gitignored `people/`),
which is a data state on this laptop, not a code regression — the three code gates are green.

**The Zitadel dependency, flagged.** API mode needs a device-flow PUBLIC client registered for the CLI
in paridyum-operations; its id is `DOCKET_OIDC_CLIENT_ID`, which has NO default and is refused if unset.
That client does not exist yet — creating it is the deployment gate for this path, recorded as D-65.

**Skills and tools used.** Reading `main.rs`/`http/*`/`bin/serve`/`auth.rs` before writing a line
(governance precedes implementation); reusing the prior run's finished `token.rs` after verifying it
against STD-0003; the compiler and clippy `-D warnings` as the guide; a local `TcpListener` as the mock
issuer/server so every test runs with no network; `park_timeout` for the poll wait per clippy.toml;
`scripts/gates.sh` as the single un-piped gate.

## 2026-09-24 — the verifier became a call into `paridyum-auth`, and the policy stayed here

**ADR-0092 §3's trigger fired elsewhere.** This adapter was the estate's second in-crate RSA/JWKS
verifier, recorded as debt *"until a third consumer"* rather than extracted speculatively.
`platform-tenancy` copied `platform-identity`'s verifier and became the third, so ADR-0107 extracted
`lib-auth-rust` and this file now calls it.

**The interesting part of the change is what did not move.** Three things in this file looked like
they belonged in a shared auth library and do not, by ANTI-0004's test — *if the business changed its
mind, would the library need editing?*

- **The `operator` role.** ADR-0092 D1 is docket's rule. A role name in the library would mean
  changing this tool's authorisation by editing a repository two platform services also depend on.
- **The pins.** One plane (`id.paridyum.com`) and the `paridyum-operations` audience. The library
  offers `Pins::single_plane`; *which* plane and *which* project are this tool's configuration, and
  they are the reason a tenant-plane token is refused here rather than accepted with less privilege.
- **`derive_tenant_id`.** ADR-0082's uuidv5 is the tenancy domain's, and it is computed identically
  in three languages; a copy in an auth library would be a fourth place for it to drift.

**The lenient `Bearer ` prefix stayed lenient.** The shared `strip_bearer` is strict, which is what
`platform-identity` always did. This adapter has always accepted a bare token, so it keeps its own
two-line strip with the reason written next to it. Moving code is not licence to change behaviour.

**`OperatorVerifier::new` kept its signature** — `(keys, issuer, audience)` — so `serve.rs`,
`router.rs`, `server.rs` and `operator.rs` are untouched. The type is now a newtype over
`paridyum_auth::JwksVerifier`.

**Tests kept, and why each one is still earning its place.** The role, `resourceowner`, and lenient
prefix cases test this tool's policy. The wrong-issuer, wrong-audience and expiry cases now overlap
with the library's own tests and were kept anyway: they assert **this tool's pins are wired
correctly**, which is a different claim from "the library rejects a bad audience", and a
misconfigured `Pins` would pass every library test.

**Verified:** `cargo test` — 276 unit + the adapter suites green. Two pre-existing failures in
`tests/issuance.rs` (`the_checked_in_registry_is_valid`, `the_binary_reports_meaningful_exit_codes`)
are unrelated to this change and were already red: *"PMD has no active signatory"*, which the
knowledge CHANGELOG records as deliberately red pending an appointment only Rohan can make.
`cargo clippy --all-targets --all-features` clean; `cargo fmt --all --check` clean. Compiled against
the library through a temporary **path** dependency; the committed `Cargo.toml` names the tag, which
does not exist until `Paridyum/lib-auth-rust` is pushed.

Skills used: none. Governance retrieved: ADR-0107, ADR-0092, ADR-0082, ADR-0099, ADR-0047, ADR-0070,
ADR-0053, ANTI-0004, STD-0022.

## 2026-09-26 — the fourth session value reaches docket, and where the edge is for a CLI

T-G.2's last leg. `platform-tenancy` and `@paridyum/auth` got the same contract earlier today; this
completes the three places in the estate that open a Postgres transaction.

**The decision that took the thinking: where is the edge for a tool that is also a CLI?** ADR-0044 D1
requires the id to be generated once at the outermost boundary and *propagated unchanged*. Two answers,
and both are right:

* **Served (ADR-0099)** — the HTTP dispatcher reads the request's headers, because a client's
  `traceparent` is a trace that must survive rather than be replaced.
* **CLI** — there is no inbound request, so `docket issue` *is* the edge. The id is minted **once per
  invocation** and every transaction in that invocation shares it.

Minting inside `enter()` would have been one line and is the thing this change is written not to do: a
single `docket issue` opens several transactions, so per-transaction minting turns one command into
four unrelated ids — the per-hop minting D1 forbids, in the place it would be least visible. So the id
is a **parameter** of `enter()`, threaded from the edge. That is why this change touches eighteen
signatures instead of one: the property is structural, not a convention somebody has to remember.

**`PgStore` holds it rather than the `Store` trait carrying it.** A `PgStore` is built for the life of
one request (`PgTenantStores::build`) or one CLI invocation, which makes the store the natural scope.
Putting it on the trait would have added a parameter to a port that the **file-backed** implementation
also satisfies — and that one has no sessions, no RLS and nothing to do with a correlation id.

**It goes in the same statement as tenant and operator scope**, which costs no extra round trip, and
before the account lookup because that lookup needs the other two set (`accounts` FORCEs RLS on
`current_tenant()` AND `has_operator_scope()`). The ordering was already load-bearing; this did not
change it.

**The duplication is priced, not hidden — D-71.** This is the *third* implementation of one contract
(`platform-tenancy/src/correlation.rs`, `@paridyum/auth`'s `correlation.ts`). It was not shared because
`paridyum-auth` is the estate's **auth** crate and a correlation id is observability: putting it there
because a crate happens to exist is ANTI-0004 pointed the other way, a library acquiring a concern it
has no business owning. A `lib-observability-rust` for eighty lines is a repository, a version, a pin
and a release path. **The trigger is written down so it fires:** a fourth Rust implementation, or any
change to the parsing rules that must be made twice. That is the shape ADR-0092 D3 used for the JWKS
verifier, which ADR-0107 extracted after the third copy — and this is the third copy, so the next
change to these rules is the extraction.

**D-72 is the other half, and it is honest about what is still missing.** Docket now carries an id that
makes each unit of work joinable to its request, and **writes nothing to join**. The four sealed chains
are what a document *is*, not who touched it: a read of a person record leaves no trace, and a refused
write leaves less. `audit_events` is in tenancy migration 0010, unapplied, and no docket code inserts a
row. The half that shipped is the half that cannot be added retroactively.

**Verified:** `cargo test` — **289 unit** (up from 280) plus 5 + 7 + 14 in the adapter suites, all
green; `clippy --all-targets --all-features` clean; `cargo fmt --check` clean. The nine new tests are
mostly refusals: `ff`, an uppercase trace id, an all-zero trace id, a three-field header, and a
fallback header carrying a space, a tab, a semicolon or a newline — each asserting the value is
**replaced and reported**, never truncated.

**Two failures in `tests/issuance.rs` are pre-existing and unrelated**, and the same two the ADR-0107
entry recorded: `the_checked_in_registry_is_valid` and `the_binary_reports_meaningful_exit_codes` fail
because **PMD has no active signatory** — deliberately red pending an officer appointment only the
operator can make (RFC-0042 §5, ADR-0087 D1) — and because four person records still hold the
template's `FILL-ME` placeholder in `pre_existing_ip`. Both are data, in `registry/` and `people/`.

Skills used: none. Governance retrieved: ADR-0044, ADR-0041 D3, ADR-0038, ADR-0078, ADR-0092 D3,
ADR-0099, ADR-0104, ADR-0107, ANTI-0004, RFC-0045 §D, GLO-0005, TASKS T-G.2, tenancy migration 0010.

## 2026-09-26 (later) — PMD can sign, and the published officers page named the wrong CEO

**PMD had no signatory and `docket check` refused the whole registry for it** — *"PMD has no active
signatory, so nothing can be issued for it"* (ADR-0077 D4). It had been red since RFC-0052 created the
entity, and two tests in `tests/issuance.rs` failed with it.

**I recommended against appointing anyone, and the file I was about to edit had already answered me.**
My argument was that PMD is `legal_status: Incorporation in Process` with `cin: null`, so naming
officers of a company that does not legally exist puts a fiction in the one register whose job is to
say who held office on a date. The Cyber block eleven lines above where I was writing says:

> This is an operator decision about who acts, not a legal opinion about who may bind an
> unincorporated company. RFC-0042 §3 takes the same posture.

So the question was settled on 2026-09-09 and my objection was to a claim the register does not make.
Worth recording because it is the third time today that reading further would have been cheaper than
reasoning from first principles.

**Appointed on operator direction: PRIYANSHU SHARMA for the holding company**, the same shape as
`cyber-holding`, because ADR-0077 D4 resolves signing through ownership and PMD is `owned_by: PAR`.
Two subsidiaries whose documents route differently for no stated reason is how a register stops being
predictable.

**The second entry was required rather than chosen**, and the reason is a property of the check:
`countersignatories` counts distinct **people**, not entries, because `docket issue` refuses a document
whose subject and signatory are the same person. With Priyanshu alone, PMD could issue nothing to
Priyanshu — the one officer hardest to route around. ROHAN BATRA is second; he is already CTO at PAR
and PCY. **No CEO entry:** nobody holds a media office yet, and Academy is expected to leave
(RFC-0052 D3), which is the moment to appoint to it.

**Then the remaining warning turned out to be worse than a stale file.** `docket check` said the
officers page had not been re-exported since 2026-09-07. Generating it showed why that matters:

    published:  ceo  RUHANI KHURANA
    generated:  ceo  PRIYANSHU SHARMA

**`paridyum.com`'s officers page has been naming Ruhani Khurana as Paridyum's CEO.** She is CEO of
**Paridyum Cyber** — a different company. A public page has been attributing the wrong company's office
to a named person, and the warning that was trying to say so called it a re-export. It is pre-existing
and unrelated to this change: the PAR entries did not move, the export was simply stale. Corrected in
`web-paridyum` in the same pass.

`documents_signed_by` for `director` moves from `CEO` to `CTO` with it — which is correct and follows
from Priyanshu holding both offices, so his own documents fall through to the CTO (the 2026-09-09
decision, arrived at by data rather than by a branch in code).

**Verified:** `docket check` — **0 errors**, 1 warning (which the `team.json` export clears), 26
documents, 5 stages, 0 register entries. The four `pre_existing_ip` placeholders are `None`, which is
what both templates already shipped.

Skills used: none. Governance retrieved: ADR-0077 D4/D5, ADR-0087 D1, RFC-0042 §3/§5, RFC-0052 D3,
C-0004.

## 2026-09-27 — docket writes one audit event per request, and two limits are tests rather than notes

ADR-0110 was accepted this morning, which is what this was waiting on. D-72 said docket writes no audit
event; the blocker was never effort, it was the **unit**.

**One event per operation, which is why the writer is not in the store.** `docket check` opens several
transactions and reads four sealed chains, a person store and a registry. Audited per store call that is
tens of rows for one command — and nobody asked for a `SELECT` on `docket_signatures`, they asked for a
document. So the call site is the **dispatcher**, where the operation is known, and `PgStore` — which only
ever sees queries — is untouched. That also keeps the `Store` trait, which `FileStore` satisfies too, free
of a parameter it has no use for.

**The operation name is a closed vocabulary, not the path.** `operation_of` maps `(method, segments)` onto
a fixed set and falls through to `request.unknown`. A path interpolated into `action` would make every
caller-supplied segment part of a contract that queries against the trail are written against (ADR-0110
accepted cost 2) — and would put caller-controlled text into a column read during an incident. There is a
test that a request for `/api/../../etc/passwd` records `request.unknown` and that neither column contains
`passwd`.

**The outcome comes from the response status, and the mapping needed an argument.** 401 and 403 are the
only denials. A 400, a 404 or a 409 is an **allowed** operation with a wrong request — recording typos as
refusals would fill a security review with typos and make *"what was refused"* stop being answerable. A
404 reading as `allowed` looks odd until you ask what the row is for: the operation was served, and "this
tenant asked for a person who does not exist" is a fact worth keeping rather than a refusal.

**A failed audit write does not change the response, and that is the opposite of what `platform-tenancy`
does.** There, the row is written inside the read's transaction: nothing has been served, so a failure
fails the request and an unrecorded access cannot happen. Here the work is finished and the response is in
hand — turning it into a 500 would report a failure that did not occur and leave the caller unable to tell
whether the operation took effect. So it logs at error level. **An unrecorded access is a gap in the log,
not a gap in the work**, and the two call sites now say so in each other's terms.

The test for it compares a served response against one served with a failing sink, rather than asserting
`200`. The first version asserted the literal and **failed on the first run** — `FakeStores` answers that
route with a 404 — which is the difference between testing the claim and testing the fixture.

**Two limits are asserted rather than written in a comment.** A rejected token writes **nothing**:
`audit_events.tenant_id` is `NOT NULL` and a token that failed verification yields no tenant to attribute
the attempt to. A placeholder tenant would attribute an attempt to a tenancy that had nothing to do with
it. And `/api/health` writes nothing — answered before the gate, and a row per probe would be the estate's
highest-volume audit entry while saying nothing about anyone's data.

**`audit` is a field on `Backend::PerTenant`, not an `Option` on `Serving`.** An `Option` would let the
served backend run with no sink and look configured. A field on the variant that has a database makes
"served but unaudited" a state that **does not typecheck** — the same reasoning as `Serving.verifier`
being `None` only on the backend that has no token.

**Three things recorded as debt rather than smoothed over.** Migration 0010 is unapplied, so every insert
fails until it lands and the logs say so once per request. **File mode is not audited and cannot be** — no
database, no tenant — and it is deliberately not solved with a local log file, which would be a second
audit store the tenant cannot read, and an audit trail only the auditee can read is what RFC-0022 D5
exists to replace. And a served event names **no account**, because the account is resolved inside the
store's transaction and the dispatcher never sees it; a row naming an account nobody looked up would be a
guess about which silo was touched.

**Verified:** `scripts/gates.sh` — all green. `cargo test --lib` **301 passing**, up from 289; the twelve
new ones are the status mapping, the vocabulary, the path-echo refusal, and the five dispatcher properties
above.

Skills used: none. Governance retrieved: ADR-0110, ADR-0044 D1/D6, ADR-0092 D1, ADR-0093 §3, ADR-0099,
ADR-0038, RFC-0022 §D5/§D5a, RFC-0014, GLO-0004.

## 2026-09-27 (later) — the audit session sets only what its policy reads, proved here

The audit writer I added this morning set the session account to the nil uuid and operator scope to
`true` "so nothing raises". Migration 0010's append policy reads the tenant only, and the sink only
inserts, so neither value was read — and a sentinel standing in for an absent account is what
GLO-0005 forbids: in that transaction, a read would have seen every account's rows.

**How it was proved, after review said the first proof was the wrong one.** The first proof was a
case added to `platform-tenancy`'s audit probe. `/code-review high` pointed out that it retyped
docket's SQL by hand, so it would go on passing after docket's code changed. The proof now lives
here: `the_audit_sink_writes_under_the_append_policy_alone` calls the real `Sink::record`, reads the
row back, and asserts that a read in the sink's own session does not widen. It needs a role subject
to RLS — a superuser bypasses it and proves nothing — so `scripts/probe-audit-writer.sh` replays all
sixteen tenancy migrations into a scratch database, creates the role BEFORE the replay (so each
guarded grant takes the live database's branch), seeds a tenant and runs it. It was then run with
the old sentinel planted back in, and failed — which is what makes the passing run evidence.

**Also from the review:** the row's `operator_scope` was a literal `true` in the sink, the same
stand-in shape the change removes; it now comes from `audit::Context`, set by the dispatcher, which
knows it because it holds a `VerifiedOperator`. The session SQL is one pair of constants instead of
two hand-kept strings. And "a later read raises" was an overclaim — RLS evaluates the predicate only
on rows a scan reaches — so the comment now says what holds: a read cannot widen.

**Why the dispatcher's silence mattered to the choice of proof.** A failed audit write is logged and
the response served unchanged (deliberately — the work already happened). That makes this the one
path where a regression is invisible to every other test, so it is the one path that most needed a
test of the real code against the real policy.

**Second review round, same day.** It found that the proof was weaker than its description: the
widening check opened its own transaction from a copied constant, so it did not test the session the
sink sets; and it recorded `operator_scope = true` and read back `WHERE operator_scope`, so it could
not tell a bound value from the literal it replaced. Now the sink and the test both go through one
`request_transaction`, the test records `false` and reads the column back, and `operator_scope` is
read off the verified token rather than written as a literal anywhere. Both regressions were planted
and watched failing. The harness also stopped altering the developer's `paridyum_app` role — it
connects as a throwaway login role that is a member of it — and the test refuses to write unless the
database it connected to is the named `*_probe` scratch database, because `audit_events` is
append-only and a stray row would be permanent. What the proof still cannot see is in D-73.

## 2026-09-27 (later still) — #25's premise was wrong, and the correction

Reviewing the companion change in `platform-tenancy` surfaced that repository's rule — *"Four `SET
LOCAL`s, always, even when one looks redundant"* — and its `db.rs`, which documents the nil account as
the value for a request that is not account-scoped and **"not a sentinel that widens anything."** That
contradicted the reason #25 gave for narrowing the audit session. Checked, and #25 was wrong:

- **The nil account widened nothing.** It matches no row, so access comes from operator scope alone.
  In the old session a read would have seen every account's audit rows — because the operator
  holds operator scope over the tenant, which is what that grant means, not because of the nil uuid.
- **Two values was the more fragile choice, in the one place fragility is invisible.** A future
  append policy that read the account would have raised on every audit write, and the dispatcher
  swallows those. Four values is the estate's rule precisely so that a policy change does not break
  a caller that set fewer.

So the session is four values again. What survives from #25 is the part that was right: operator
scope is read off the verified token rather than written as a literal, and the live test runs the
real writer against every migration — now asserting that a session WITHOUT operator scope reads
nothing, which is the nil account's documented property checked against the real policies. Planted
a literal `'true'` scope; the test failed with "saw rows it holds no grant for".

**Correction to the line above, from the third review round.** That message belonged to the version
of the test that existed when the plant was run, which read its "fails closed" check through a
different path from the sink. The review noticed the check bypassed `audit_transaction`, so a sink
session that ignored its scope would not have been caught. The check now reads through
`audit_transaction` itself; re-planted on the final code — the sink's session made to bind `true`
whatever it is passed — it fails with "the sink's scope-less session read rows it holds no grant
for". Both plants were re-run on the final code and the numbers above are theirs. An earlier draft
of this entry said "all ten fail" for the `enter()` plant; it was nine, and the draft was wrong
because it reported what was expected rather than what was counted.

**The lesson is about which claim to verify.** #25 verified that the narrower session *worked*, and
it did. It never verified the premise — that the old session widened — against the estate's own
statement of what the nil account means, which was one repository away.

**And the review of the correction found the outage.** Its first finding: `enter()` still read
`accounts` with three session values, under a comment that the lookup was legal with two — true when
written, false since tenancy 0015 added `accounts_read` with `id = current_account() OR
has_operator_scope()`. Measured on a replay of 0001–0016 as `paridyum_app`: the three-value lookup
raised `paridyum.account_id is not set` every time. 0015 is pending on the cluster, so **applying it
before this lands would have broken every docket store call** — and the migration window was the next
thing on the list. `set_session` is now the one place all four values are set, for `enter()` and the
sink alike.

Proving it needed the database suites, and they had rotted: `tests/pg_store.rs` and
`tests/served_api.rs` seeded tenants without `region` (NOT NULL since 0011) or a docket account
(required since 0012), and skipped without a word when no database was configured, so their rot was
invisible. Revived — seeding through an admin connection, with the deployment's grants mirrored from
tenancy's own probe — all ten database tests pass against the full stream. With the three-value
`enter()` planted back, **nine of the ten** fail — every test that goes through `enter()`; the audit
sink's does not call it. The factory test's premise (a tenant with no row builds an empty
store) was itself pre-0012; it now asserts the refusal ADR-0104 asks for.

Skills used: code-review (high, three rounds on #25/#26). Governance retrieved: platform-tenancy
AGENTS.md (four `SET LOCAL`s), platform-tenancy `db.rs` (`NO_ACCOUNT`), tenancy migrations 0010, 0012,
0015, ADR-0038, ADR-0044 D1/D4, ADR-0093 §3, ADR-0104 cost 2, GLO-0005.

## 2026-09-28 — docket proved against the schema it will be deployed into; one commit skipped review

Review of the migration order found the step nobody had tested: docket #26 had been proved against
the full stream (0001–0016), never against the schema it would actually meet — the cluster's, with
0010, 0014, 0015 and 0016 pending. Seeds named no `accounts.id` (it has no default before 0014) and the
harness assumed `audit_events` existed. Both fixed; against the live subset the store and served tests
pass, and the audit sink's test is skipped by name with the reason printed, because a missing table is
a fact about that schema, not a regression.

**A commit reached `main` without a pull request.** `git checkout main && git pull --ff-only && git
checkout -b test/…` — the pull failed ("cannot rebase onto multiple branches"), the `&&` chain stopped,
the branch was never created, and the commit and `git push origin HEAD` went to `main`. Found when
`gh pr create` refused a head equal to its base. Not reverted: force-pushing `main` is denied (C-0004)
and a revert-and-redo would add two commits for a test-only change whose gates were green. Reviewed
after the fact instead (`/code-review high` on the range), and this change carries its findings: the
deploy-window consequence stated, the ADR-0078 exception stated, the factory test's seed copy removed,
the harness gating correctly on a subset, and an empty migrations directory refused. From here the
branch is checked with `git branch --show-current` before every commit.

Skills used: code-review (high, after the fact). Governance retrieved: ADR-0078 D1/D2, C-0004,
tenancy migrations 0010/0012/0014, platform-tenancy AGENTS.md.


## 2026-09-28 — the seed's default tenant, after the rebuild from empty

The estate was rebuilt from empty on 2026-09-28 (infra-kubernetes#10). Paridyum's tenant is re-derived
from its new Zitadel organization — `392737556011156703` → `451ce2e0-b82f-59ad-ad54-ab02ad9811cb`, the
row `tenant-provision` wrote — so `seed-config.py`'s default named a tenant that no longer exists.
The derivation tests in `src/` keep `68445ee9-…`: they prove the function, not the estate.

**Still owed for docket to run again, and not done here:** the operator registry's `subject` is the
destroyed control plane's user, and `DOCKET_OIDC_AUDIENCE` its operations project — both wait on the
control-plane instance being recreated (infra-kubernetes#10). docket's configuration is not re-seeded
until then; the operator said docket may stay down while the estate is made right.

## 2026-09-28 (second entry) — re-bound to the rebuilt control plane

The control plane was rebuilt the same day through the first ADR-0123 window (infra-kubernetes#10),
so what the first entry listed as owed is done here: rohan's operator entry is bound to the new
subject `392752909697746075` — read from the rebuilt instance, where `provision-control.py` resolves
the owner by email — and `tests/served_api.rs`, which loads the real registry, changes with it. The
SOP's audience is the new `paridyum-operations` project `392754455717610538`.

**Not re-run:** `scripts/probe-audit-writer.sh`, the only runner of `served_api`, needs a full build of
this crate, and the workstation had 4.4 GiB free after the build directory was cleared. The change is
two literals that must agree, and they do; the probe is the check that proves it, and it is owed.
The derivation and verifier tests in `src/` keep their old ids — they are vectors, not the estate.
