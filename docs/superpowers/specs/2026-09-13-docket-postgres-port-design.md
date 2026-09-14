# docket's Postgres port — the implementation design for ADR-0093

This is the implementation design for **ADR-0093 decisions 1–4** (docket becomes a
served, database-backed, multi-user staff platform). ADR-0093 is the *decision*;
this is *how*, sequenced, with the chain-preservation gate made explicit before a
single one of the fifteen application modules is touched. It supersedes the
persistence assumptions of the 2026-08-21 served-platform design (which described
a loopback JSON server over a **filesystem** store); the HTTP contract that design
established, and that `console-docket` speaks, is **kept unchanged**.

## 0. The honest starting point

An earlier autonomous run built three things and reported the backend "done." Two
are sound foundations; the third is a spike, and the report was wrong. Written
down so the next reader does not inherit the overstatement:

- **`adapters/pg.rs`** — a real, tested async Postgres store for the identity,
  register and event chains plus the mutable person. **Sound, but incomplete:** it
  is missing the **signature ledger**, which is a fourth `Chained` hash-chain
  (`domain::signature::Record`). Migration `0006` created three chain tables, not
  four.
- **`adapters/auth.rs`** — a real, tested ADR-0092 operator-token verifier
  (RS256, issuer/audience pinned, `operator` role required, subject extracted).
  **Sound, reused as-is by this design.** It is synchronous, which matters below.
- **`served.rs` + `bin/serve.rs`** — a minimal **axum** server exposing `GET
  /roster` and `POST /person` **straight against Postgres, bypassing the entire
  `application::` layer**. It serves two of ~twelve routes, carries **none** of the
  HR rules (no gates, no refusals, no document issuance, no chain sealing of the
  identity/register/event/signature ledgers on write), and cannot back the
  console. **This is the spike. It is removed by this design** (§7).

The real server — the one that serves the console's whole contract with every rule
intact — is `adapters/http/` over the **file store**. The port is: move that
server's persistence to Postgres, add token auth, and let more than one operator
write at once. That is the work ADR-0093 calls the single largest change in the
estate, and none of it is done.

## 1. What is being built

`tool-docket`'s system of record moves from YAML/JSONL files under `store/` and
`people/` to the shared Postgres database (platform-tenancy). The `application::`
layer and the `adapters/http/` contract are **preserved**; what changes underneath
them is where bytes live and how many people may write at once.

Four things move, and one deliberately does not:

| Moves to Postgres | How it is stored today | Table |
| --- | --- | --- |
| Identity ledger (`domain::identity::Record`) | `store/ledger.jsonl`, hash-chained | `docket_identities` |
| Register (`domain::register::Entry`) | `store/register.jsonl`, hash-chained | `docket_register` |
| Event log (`domain::event::Event`) | `store/events.jsonl`, hash-chained | `docket_events` |
| **Signature ledger (`domain::signature::Record`)** | `store/signatures.jsonl`, hash-chained | **`docket_signatures` (NEW — §3)** |
| Person records (`domain::person::Person`) | `people/<slug>.yaml`, mutable | `docket_persons` |

| Stays on the filesystem, for now | Why |
| --- | --- |
| Issued PDFs (`store/issued/<person_id>/…`) and scans | ADR-0093 §decision-5 / ADR-0068 §6 leave the **document/blob store** an open, separate question (S3 candidate). The **register chain** — which records that a document was issued, with its serial and digest — moves; the *bytes* of the PDF do not move in this change. A served instance and a break-glass CLI on different hosts will not share PDFs until the blob store is decided; the register (the legally-load-bearing record) is shared immediately. This is called out as a known limitation (§8), not hidden. |
| Generated indexes / `manifest-sha256.txt` | Regenerated, never authoritative (C-0004). Derived from the register, which moves. |

## 2. The core decision: the application layer stays synchronous

The fifteen `application::` modules and the whole `domain::` layer are synchronous
(`fn`, `?`, no `.await`), across **209** file-store call sites. `sqlx` (the locked
database library, ADR-0005) is async. Bridging that gap is the design's central
choice, and there are two honest paths:

**A — Port the whole layer to async.** Every `application::` and touched `domain::`
function becomes `async fn`; the router and handlers become async; the transport
becomes `axum`, reusing the spike's skeleton.

**B — Keep the layer synchronous; drive `sqlx` from a shared runtime.** Introduce a
**`Store` port** (a trait) with the ten-odd operations `adapters/store.rs` already
exposes. The Postgres implementation holds a `PgPool` and a handle to a
process-wide multi-threaded Tokio runtime, and each method is a synchronous
signature wrapping `runtime.block_on(async { … sqlx … })`. The `adapters/http/`
server becomes **thread-per-request** (ADR-0072's single-connection serialisation
is retired — concurrency is now the database, ADR-0093 §decision-2); each request
thread calls synchronous `application::` functions that call the synchronous
`Store`, which blocks on the runtime for its query. Nothing in `application::` or
`domain::` changes shape.

**This design chooses B.** Rationale:

- **It reuses the tested rules, unchanged.** Path A rewrites 209 call sites across
  fifteen modules whose correctness is the whole value of the tool — every gate,
  every refusal, every seal. A port that large, of exactly the code that must not
  regress, is where a subtle behaviour change hides. Path B changes the store
  implementation behind a trait and leaves the rules bit-for-bit.
- **It keeps the contract and the server the console already speaks.** The rich
  `adapters/http/router.rs` stays; only its `Context` gains a `Store` and its
  server loop gains a thread per request.
- **It matches the design lineage.** The 2026-08-21 design deliberately chose a
  blocking, thread-per-request shape. ADR-0093 changes the *store* and the
  *concurrency source*, not the argument for blocking request handling. `block_on`
  from a dedicated OS thread is the sanctioned bridge, not a workaround.
- **It stays inside the locked stack.** Still `sqlx`, still `rustls`; no sync
  Postgres crate, so no ADR-0005 deviation. `jsonwebtoken` (auth) is already sync.

Path A is **rejected** here and recorded as such: its only unique gain is a fully
async process, which a staff-scale internal tool serving a handful of concurrent
operators does not need, bought at the price of rewriting the code most dangerous
to rewrite.

## 3. Schema — migration 0006 gains the fourth chain

`platform-tenancy` migration `0006` currently creates `docket_persons`,
`docket_identities`, `docket_register`, `docket_events`. It **must gain
`docket_signatures`**, identical in shape to the other append-only chain tables:

- `chain_seq bigint GENERATED ALWAYS AS IDENTITY`, `record jsonb NOT NULL`,
  `tenant_id uuid NOT NULL` (the tenancy column, ADR-0023),
- generated key columns projected from the record for indexing/uniqueness (the
  serial the signature is against, and `entry_sha256`),
- the `paridyum.forbid_mutation()` append-only trigger,
- `ENABLE`/`FORCE ROW LEVEL SECURITY` and the tenant+operator policy, exactly as
  the other three (check-migrations.sh enforces this).

Because 0006 has not shipped to any environment holding real data (no real records
exist yet — the backup gate, ADR-0068 §5, has not been met), the table is **added
to 0006 in place** rather than in a follow-on migration. This is the one moment it
is correct to edit 0006 rather than append 0007.

`adapters/pg.rs` gains `append_signature` / `read_signatures`, exactly parallel to
the existing three chains, and the seal-round-trip test is added for
`signature::Record` (the fourth of four).

## 4. The `Store` port — records only; blobs stay on `Paths`

**Measured before writing this section**, because it changes the port's shape. The
application layer's filesystem coupling is not one thing; it is two, and only one
moves now:

| Concern | Reaches the disk via | Volume (grep) | This change |
| --- | --- | --- | --- |
| **Records** — the four chains + the person document | `store::` | ~55 `store::` calls | **move to the `Store` port → Postgres** |
| **Blobs & generated files** — issued PDFs, packs, scans, `store/index/`, `manifest-sha256.txt`, LaTeX templates, `.gitignore`/CSV reads | `fs::` + `paths.` directly | ~80 `fs::` calls, ~300 `paths.` uses (vault 22, check 12, node 12, pack 8 `fs::`) | **stay on `Paths`/`fs::`** (blob store is a later, separate S3 question — §1, §8) |

The consequence for the refactor, and the correction to the earlier phrasing that
the application functions simply "take `&dyn Store` where they take `&Paths`
today": most functions that touch records **also** touch blobs (issue a PDF,
assemble a pack, read a template), so they **gain a `Store` and keep their
`Paths`** for this phase. The cut is by *concern*, not by function — every
`store::foo(paths, …)` call becomes `store.foo(…)`; every `fs::`/`paths.` blob
call is left exactly as it is. A function that touches no records (much of
`vault.rs`, `pack.rs`, `team.rs`) is not touched at all.

This is honest about the interim (§8: a served instance shares the *record* that a
PDF was issued, not yet the PDF bytes) and it keeps the diff proportional to the
records surface (~55 sites) rather than the whole filesystem surface (~380).

The port trait, synchronous, is the record surface of `adapters/store.rs`:

```
person(slug) -> Option<Person>          upsert_person(&Person)
list_persons() -> Vec<Person>           slugs() -> Vec<String>
ledger() -> Vec<identity::Record>       append_identity(&Record)
register() -> Vec<register::Entry>      append_register(&Entry)
events() -> Vec<event::Event>           append_event(&Event)
signatures() -> Vec<signature::Record>  append_signature(&Record)
reseal_events(Vec<Event>)               // the erase special case, §6
```

`keep_pdf`/reading a PDF are **not** on the port — they are blob operations and
stay on `Paths`. (When the blob store is decided, ADR-0068 §6, they become a
second, separate port; this change does not pre-empt that.)

Two implementations:

- **`FileStore`** — the current `adapters/store.rs` bodies, moved behind the trait.
  Kept so the CLI's local/offline mode and the existing unit tests keep working
  during the port, and so a break-glass operator with no database still has a path.
- **`PgStore`** — wraps `pg.rs` (async) behind synchronous methods via the shared
  runtime (§2). Every method runs inside a transaction that first sets the tenant
  and operator scope (`enter`, already in `pg.rs`), so RLS applies to every read
  and write (ADR-0038, fail-closed).

`application::` functions take `&dyn Store` (or `&impl Store`) where they take
`&Paths` today. This is the bulk of the 209-site change, but it is **mechanical
and type-checked**: a call site that is not migrated does not compile.

## 5. The served backend and authentication

`adapters/http/` evolves; it is not replaced:

- **Transport → thread-per-request.** `server.rs` spawns a thread per accepted
  connection instead of serialising. The banner text about "one request at a time"
  and the store lock is removed; the loopback-only bind (`require_loopback`) is
  **relaxed to allow a configured bind address _only when a token verifier is
  present_** — the invariant from the 2026-08-21 design ("must refuse a routable
  interface until an authentication adapter is present") is honoured by making the
  routable bind *conditional on auth*, not by keeping it forbidden.
- **Auth → ADR-0092 tokens, replacing the header on the served path.**
  `adapters/http/operator.rs::resolve` today trusts `X-Docket-Operator`. On the
  served (routable, authenticated) backend, the operator is resolved instead from
  a verified Bearer token: `auth::OperatorVerifier::verify` → `registry::by_subject`
  → the operator id (the exact chain the spike's `authorize` middleware already
  proved end-to-end, lifted into the real server). The unverified header path is
  **removed from the served server**; it does not degrade to a claim.
- **The CLI is the break-glass path (ADR-0093 §decision-3, ADR-0054).** It writes
  directly through `PgStore` (or `FileStore` offline) with `--by` attribution — not
  over HTTP. It is not the primary interface and does not need the token flow.

The startup sequence (fetch JWKS once, build the verifier, open the pool, load the
registry) is exactly what `bin/serve.rs` already does; it is rebuilt against the
real `Context`/`Store` and the axum version is deleted.

## 6. The erase special case

`docket erase` is the one writer that does not append: it rewrites the event log to
its skeletal events and **re-seals the chain** (CLAUDE.md; an append-only log that
cannot forget cannot honour a DPDP erasure). Against an append-only Postgres table
with a mutation-forbidding trigger, "rewrite and re-seal" is **delete-all-then-
reinsert-the-skeleton within one transaction**, performed by a dedicated
`reseal_events` store operation that is the *only* path permitted to remove chain
rows — implemented as a `SECURITY DEFINER` function or an explicit,
narrowly-scoped exception to the append-only trigger, decided when erase is ported
(sequenced last, §7). It must leave the chain verifying from the new genesis.

## 7. Sequencing — reads before writes, seal-preservation as the gate

Each step compiles, tests green, and is useful or safe on its own.

1. **Schema + adapter completeness.** Add `docket_signatures` to migration 0006;
   add `append_signature`/`read_signatures` and the fourth seal-round-trip test to
   `pg.rs`. Re-validate 0006 against real Postgres (apply, generated columns,
   append-only trigger, RLS isolation) for all **four** chains.
2. **The `Store` port + `FileStore`.** Introduce the trait; move today's
   `store.rs` bodies behind it; add a `&dyn Store` to the functions that touch
   records and rewrite their `store::foo(paths, …)` calls as `store.foo(…)`,
   **leaving every `fs::`/`paths.` blob call exactly as it is** (§4 — records move,
   blobs stay). Functions touching no records are not touched. **No behaviour
   change** — the existing unit/integration suite is the proof, run against
   `FileStore`.
3. **`PgStore`.** Implement the trait over `pg.rs` + the shared runtime. Add the
   **chain-preservation gate**: an integration test that runs a representative
   sequence (enrol → onboard → sign → advance) through the *application layer* on
   `PgStore`, then reads every chain back and verifies each seal and every
   `prev`→`entry` link — all four chains. This test is the gate ADR-0093 §decision-1
   names ("the chain moves across with its guarantees or the move does not
   happen"). Reads-only parity first, then the write paths.
4. **Served backend.** Evolve `adapters/http/server.rs` to thread-per-request +
   conditional bind; resolve the operator from a verified token; delete `served.rs`
   and the axum `bin/serve.rs`; add the real `bin` against `Context<PgStore>`. The
   spike's happy-path e2e test is rewritten against the real router so its coverage
   is not lost.
5. **Erase.** Port `reseal_events` (§6) last, with a test that erase leaves the
   event chain verifying from the new genesis and the identifier surviving as a
   tombstone.
6. **console-docket.** OIDC (PKCE) against `id.paridyum.com`, session, forward the
   Bearer token to the served backend; `DOCKET_API` moves off loopback. (Its own
   design, TypeScript; unblocked once step 4 serves the real contract with auth.)
7. **Deploy + backup gate.** Helm chart + CI for the served backend;
   class-registry reclassification (ADR-0093 §decision-3); then ADR-0068 §5's
   off-host destination and **rehearsed** restore before the first *real* person
   record. Synthetic data flows through steps 1–6; real data waits for step 7.

## 8. Recorded consequences

- **Two stores exist during the port.** `FileStore` and `PgStore` both implement
  the port. That is deliberate (step 2 proves no-behaviour-change before Postgres
  enters) and the CLI keeps `FileStore` as an offline break-glass path. It is not
  permanent drift: `PgStore` is the system of record once step 4 ships.
- **PDFs are not shared yet.** The register chain (the record of issuance) is in
  Postgres; the PDF bytes remain on the issuing host's filesystem until the blob
  store is decided (ADR-0068 §6). A served instance can *say* a document was issued
  and show its serial and digest; it cannot yet hand back the *file* if it was
  rendered on another host. Called out, not hidden.
- **`block_on` on a dedicated runtime** is a real bridge with a real cost (a thread
  parked per in-flight query). At staff scale this is immaterial; if docket ever
  outgrows it, that is the trigger to revisit Path A, not before.
- **The backup gate is now the only thing between this and real HR use.** Every
  other blocker is code in this sequence; ADR-0068 §5 is process, and it binds.

## 9. Testing

| What | How |
| --- | --- |
| Fourth chain seals | `signature::Record` round-trips through JSONB preserving digest + links (parity with the other three) |
| No-behaviour-change at step 2 | the whole existing suite passes against `FileStore` behind the port |
| **Chain preservation on Postgres** | enrol→onboard→sign→advance through the application layer on `PgStore`, then every seal and link re-verified on read, all four chains (the ADR-0093 gate) |
| Tenant isolation | each docket table's RLS refuses a cross-tenant read/write (STD-0009), FORCE-RLS proven |
| Token auth on the served path | missing/invalid token → 401; verified subject bound to no operator → 403; valid → the operator id (lifted from the spike's proven matrix) |
| CLI/served parity | the same enrolment answers produce the same record whether written by CLI or over HTTP |
| Erase | the event chain verifies from the new genesis; the identifier tombstone survives |

## 10. Out of scope (at the time this was written)

The console's own OIDC design (step 6, its own document), the Helm chart and CI
(step 7), and any tenant-facing use (C-0008 makes that a product, not this
internal platform). The blob/document store was open here (ADR-0068 §6); it has
since been decided — see §11.

## 11. The blob port (added 2026-09-14 — ADR-0094 decided the store)

§4 moved *records* to Postgres and deliberately left *blobs* — issued PDFs, packs,
signed scans, the generated index/manifest — on `Paths`/`fs::`. That was correct
as an interim, but it has a sharp edge once docket is **served**: a served pod's
filesystem is ephemeral, so a document rendered by `onboard` over the API is lost
when the pod restarts. ADR-0094 chose the destination (Garage, in-cluster S3), so
the interim ends here.

**A second port, exactly parallel to the `Store` port.** A synchronous `Blobs`
trait — `put(key, bytes) -> ()`, `get(key) -> bytes`, `exists(key) -> bool`,
`delete(key)` (for erase) — with two implementations: `FileBlobs` (the current
`fs::` behaviour behind `Paths`, for the CLI's offline/break-glass path) and
`S3Blobs` (Garage, via the same `reqwest`/rustls already present, or an S3 client
that stays inside ADR-0005; driven by the shared runtime with `block_on` like
`PgStore`). Selected the same way the store is: an S3 endpoint configured (the
`docket-s3` secret: endpoint/bucket/key) → `S3Blobs`; else `FileBlobs`.

**What moves onto it, and what does not.** The record that a document was issued —
its serial, digest, path — stays in the register chain (Postgres) untouched; the
port only moves the **bytes**. `store::keep_pdf` and the read-back of an issued PDF
go through `Blobs`; `application::pack` assembles into it; `sign --scan` uploads a
scan through it; `application::vault` seals against it. The `pdf_path` recorded in
the register becomes the blob **key** (stable, opaque) rather than a filesystem
path — and because that key is inside an already-sealed chain entry, the key
scheme is frozen the first time it is written (a later reshuffle would orphan every
historical reference), exactly the discipline the register already imposes.

**Invariants.** A document's bytes verify against the register's stored
`pdf_sha256` after a round trip through the blob store (the blob analogue of the
seal-preservation gate — proven by test, not assumed). Erase deletes the blob as
well as destroying the key (ADR-0069), so `Blobs::delete` is real, not a tombstone.
Blob reads/writes are tenant-scoped by key prefix (the tenant/person-id path already
in `pdf_path`), so one tenant's bucket keys never collide with another's.

**Sequencing.** `keep_pdf` + issued-PDF read first (that is what makes served
`onboard` durable, the whole reason this is not deferred further); then pack, then
scans, then vault; erase's blob delete last, with the reseal (§6). Each step keeps
the CLI green on `FileBlobs` and adds `S3Blobs` behind the same trait.
