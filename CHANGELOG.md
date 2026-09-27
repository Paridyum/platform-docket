# Changelog — tool-docket

Format: [Keep a Changelog 2.0.0](https://keepachangelog.com/en/2.0.0/) ·
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- **A correlation id reaches every database transaction** (T-G.2, ADR-0041 D3 / ADR-0044 D1 /
  RFC-0045 §D1). `enter()` now sets a fourth session value, `paridyum.correlation_id`, alongside
  tenant, operator scope and account. `src/adapters/correlation.rs` reads a W3C `traceparent`'s
  **trace id** — never its parent id, which changes per hop — or an `X-Correlation-Id` matching a
  narrow shape, and mints one only when nothing usable arrived, reporting which of the three happened.

  **Two edges, both correct.** The served dispatcher (ADR-0099) reads the request's headers, so a
  client's trace survives. The CLI mints **once per invocation**: `docket issue` *is* the edge, and
  minting per transaction would give one command four unrelated ids — the per-hop minting ADR-0044 D1
  forbids, in the place it would be least visible.

  The id is a **parameter**, never resolved inside `enter()`, which is what makes that property
  structural rather than a convention. `TenantStores::build` takes it; `PgStore` holds it for the life
  of the one request or invocation it serves.

  Three implementations of this contract now exist in the estate. That is priced and recorded, with
  the trigger that turns it into a shared crate, in DECISION-DEBT D-71.

### Changed

- **The operator token verifier now calls `paridyum-auth` (`lib-auth-rust`), pinned to a tag
  (ADR-0107).** `src/adapters/auth.rs` was the estate's second in-crate RSA/JWKS verifier and
  ADR-0092 §3 recorded it as debt *"until a third consumer"*; `platform-tenancy` became the third, so
  the debt is paid. What stays here is docket's own policy, which a shared library must not hold
  (ANTI-0004): the single-plane issuer and `paridyum-operations` audience pins, the `operator` role
  requirement, and the `resourceowner` → tenant uuid derivation (ADR-0082/ADR-0099). Behaviour is
  unchanged, including the lenient `Bearer ` prefix this adapter has always accepted — the shared
  `strip_bearer` is strict, and tightening this caller while moving its code would have been a
  behaviour change disguised as a refactor. `OperatorVerifier::new` keeps its signature, so no call
  site in `serve.rs`, `router.rs`, `server.rs` or `operator.rs` changed.

### Fixed

- **The audit writer set a nil-uuid account and `operator_scope = true` that nothing read.**
  `audit_events_append` (tenancy 0010) checks the tenant only, so the session now carries the
  tenant and the correlation id and nothing else. A sentinel standing in for an absent account is
  the widening-by-absence GLO-0005 forbids: with it, a read in that transaction would have seen
  every account's rows; without it, a read cannot widen (ADR-0038). The row's `operator_scope` is
  read off the verified token (`VerifiedOperator::operator_scope`) and carried in `audit::Context`
  instead of a literal. **Proved here** by `the_audit_sink_writes_under_the_append_policy_alone`,
  run by `scripts/probe-audit-writer.sh` against every tenancy migration as an RLS-bound role — and
  watched failing with the nil-uuid sentinel, and with a literal `bind(true)`, each planted back in.

- **First green CI on `main` after the go-live merges.** Bumped `rustls` 0.23.44 → 0.23.45 to close
  `RUSTSEC-2026-0285` (TLS 1.3 handshake messages accepted at the wrong encryption level) on docket's
  live TLS path, and cleared a `.gitleaksignore` self-finding by allowlisting the documented
  placeholder value `unused` in `.gitleaks.toml` (ADR-0017-compliant value allowlist). Postgres CI
  still needs a `contents:read` token (DECISION-DEBT D-69).

- **`engines` test is isolated from an ambient store URL.** With the Postgres CI job exporting
  `DOCKET_DATABASE_URL`, the file-mode engine test's `docket` subprocess inherited it and routed to
  PgStore instead of the file store it set up, so `docket new` failed. The test now `env_remove`s
  `DOCKET_DATABASE_URL` and `DOCKET_API_URL`, exercising file mode as intended regardless of the
  ambient environment.

### Added

- **The deployment surface for the served backend.** ADR-0093 §3. A Helm chart (`helm/` —
  Deployment + Service, `/healthz` liveness/readiness probes, resource limits, digest-only image,
  SOPS→k8s-Secret wiring for the database URL), a multi-stage `Dockerfile` for the `serve` binary
  (rustls not openssl, the locked toolchain), and `publish.yml` publishing the image by digest to
  `ghcr.io/paridyum/tool-docket`. `API.md` and `RUNBOOK.md` for the served API and its operation.
  Reclassified `class: tool → platform` (ADR-0093 §3); the repository-name tension (still
  `tool-docket`) is tracked as DECISION-DEBT D-60.

- **A cold spare, so losing a laptop stops losing the estate.** ADR-0072.
  `docket store status | restore`.

  The question asked was how to run on more than one machine. Measuring first moved it: RUN-0004
  already keeps the one age key in a password manager, so a lost laptop loses no key — what lived
  on exactly one machine was the **store**. And multi-*writer* turned out to be wrong rather than
  hard, because a person identifier is allocated from the local ledger, so two machines allocate
  the same identifier to two different people and locked ADR-0051 forbids exactly that.

  `store status` classifies every top-level path in the store as replicate, via-bundle, never, or
  **UNCLASSIFIED**, and exits non-zero when anything is unaccounted for or unreplicated. This is
  `vault sync` inverted: naming an upload list and a prohibition answers "may I upload this?" and
  cannot answer "is anything missing?", so a directory added next year fell outside both.

  `store restore` rebuilds a store on a second machine from the register bundle and the estate
  key alone, then verifies every chain before reporting success. It refuses to restore onto a
  store that already holds any of the target files.

- **CI renders.** A `render` job installs TeX Live and runs the four suites that produce PDFs,
  plus a step that fails if any of them *skips* — a skip prints to stderr and would otherwise
  leave the job green while testing nothing. D-30, D-35.

  **TeX Live rather than Tectonic, and that ordering is a finding.** Tectonic was tried first for
  the obvious reason — a 10MB binary against a multi-hundred-MB apt install — and cannot render
  these documents: `paridyum.cls` uses `pdfx` for PDF/A, which fails under XeTeX with
  "CreationDate is not properly supported". More decisively, pdflatex is what every developer
  machine runs and what produced every document this estate has issued.

- **`scripts/ci-parity.sh`** runs the suite with `PATH` stripped to the Rust toolchain. Both
  times a test has passed locally and failed on a runner, it was found by pushing and reading a
  log; that now takes seconds.

- **The Director's public summary named the wrong countersignatory, in both languages.** It read
  *"the CEO countersigns his"* — and Priyanshu holds the CEO office too, so `docket issue` finds both
  his entries are the subject and falls through to the **CTO**. The prose contradicted the
  `documents_signed_by` the same file generates. Corrected in `en` and `ar`, and it now says *why* the
  fallthrough happens rather than only naming the officer, because the surprising part is that one
  person holding two offices changes who signs.

- **The served backend writes one audit event per request** (ADR-0110, ADR-0044 D6, closing most of
  D-72). `adapters/audit.rs` and the dispatcher: the operation name comes from a **closed vocabulary**
  rather than the request path — a path echoed into the column would make every caller-supplied segment
  part of a contract queries are written against — and the outcome comes from the response status. **401
  and 403 are the only denials**; a 400, 404 or 409 is an allowed operation with a wrong request, because
  recording typos as refusals makes "what was refused" unanswerable.

  **One event, not one per query** — `docket check` reads four chains, a person store and a registry, and
  nobody asked for a `SELECT` on `docket_signatures`. The writer is called from the dispatcher, where the
  operation is known, rather than from the store, which only sees queries.

  **A failed audit write does not change the response**, which is the opposite of `platform-tenancy`'s
  choice and deliberate: there the row is written inside the read's transaction and nothing has been
  served yet; here the work is done and the response is in hand, so a 500 would report a failure that did
  not happen. Asserted by comparing a served response against a failing sink rather than against a
  literal status.

  **Two limits are asserted as tests rather than left to be discovered:** a rejected token writes nothing
  (no tenant resolved, and `tenant_id` is `NOT NULL`), and `/api/health` writes nothing (answered before
  the gate, and a row per probe would be the estate's highest-volume entry saying nothing about anyone's
  data). `PARIDYUM_REGION` is now required by `serve.rs`, refused rather than defaulted.

- **PARIDYUM MEDIA can sign.** RFC-0052 created PMD as an entity and deliberately gave it no
  signatory, so `docket check` refused the whole registry — *"PMD has no active signatory, so nothing
  can be issued for it"* (ADR-0077 D4). Appointed on operator direction 2026-09-26: **PRIYANSHU
  SHARMA** signing for the holding company, exactly as at Cyber, because ADR-0077 D4 resolves signing
  through ownership and PMD is `owned_by: PAR`.

  **A second entry was required rather than chosen.** `countersignatories` counts distinct *people*,
  because `docket issue` refuses a document whose subject and signatory are the same person — so with
  Priyanshu alone, PMD could issue nothing to Priyanshu. **ROHAN BATRA** is the second; he is already
  CTO at PAR and PCY, so this invents no person and no office. **No CEO entry**, because nobody holds
  a media office yet and inventing a title to fill a column is the fiction this register keeps out.

  Nothing had been issued under `PMD/...` — the register holds 0 entries — so this precedes the first
  document rather than repairing one. **`docket check` now reports 0 errors**, for the first time.

- **The four `pre_existing_ip` placeholders are resolved as `None`**, which is what
  `people/_TEMPLATE.yaml` and `_TEMPLATE-officer.yaml` already ship: the four real records were filled
  from an older template and kept `FILL-ME`. Operator direction 2026-09-26 that there is no
  pre-existing IP. (`people/` is gitignored, so only this note is in the repository.)

- **`docket check` asserts that every class template is admitted back by `.gitignore`.** D-49.
  Checked against the file text rather than `git check-ignore`, which reports an already-tracked
  file as not ignored whether or not the negation exists — the difference that made an earlier
  mutation test pass while proving nothing.

- **`docket serve` states its one-request-at-a-time limit on startup**, where an operator is
  standing when a screen appears to hang. D-41.

- **`docket check` refuses a company that cannot countersign its own officers.** ADR-0087.
  Fewer than two *people* among a company's active signatories means that person's own documents
  can never be issued — `docket issue` refuses a document whose subject and signatory are the
  same person, and there is nobody to substitute. Counted by person rather than by entry, because
  Priyanshu holds both `ceo` and `director` at Paridyum: two rows, one human.

- **`docket check` enforces the group's ownership rule.** ADR-0077 decision 5.

  Exactly one company declares no owner; every other reaches it through `owned_by`, no chain
  loops, and no owner names a company that is not declared. Stated by the operator as a general
  rule — Paridyum owns every other company — so it is an invariant rather than a comment.

  Not bookkeeping: decision 4 resolves who may execute a subsidiary's documents *through* that
  relationship, so a company whose ownership does not reach the root has no answer to who may sign
  for it, and would discover that at its first issuance. `check` now also validates **every**
  company's jurisdiction, serial format and signatories rather than only the default one, and
  refuses two companies sharing a serial format — which would issue one serial to two people.

- **Two companies, one archive.** ADR-0077. `--company` on `docket new` and `docket reengage`.

  Paridyum is no longer the only entity. `registry/org.yaml` becomes `companies:`, each with its
  own letterhead, jurisdiction, POSH routing and serial format; `signatories.yaml` gains a
  `company` and its order is now the fallback order for countersignatures.

  **The person identifier stays group-wide and the engagement identifier carries the company.**
  Rohan is `PAR-P-000001-X` once, holding `PAR-OFC-2026-0001` and `PCY-OFC-2026-0002`. ADR-0051
  already labelled that first segment "organisation" and already said one person may hold many
  engagements, so nothing locked needed superseding — one store per company was rejected precisely
  because it would have allocated him two identities and split his archive.

  `reengage` was refusing any second engagement while one was open. That refusal was protecting
  something real — two open internships at one company — and was also, accidentally, asserting
  that a person works for one entity. Narrowed to one open engagement **per company**.

  **The CEO and Director being one person is now data, not a special case.** Priyanshu holds both
  `ceo` and `director` entries, so for his own documents both are the subject and the existing
  substitution falls through to `cto` — which is the operator's decision, arrived at without a
  branch in code.

- **`docket store push` — the upload guard, in the place D-51 said it belonged.** RUN-0007.

  D-51 held that the check could not live in docket, because docket cannot see what Sync received.
  Right about the network, wrong about the boundary: the thing performing the upload is a person
  copying files into a watched folder, and filling that folder is an operation docket can guard.

  Copies `store/vault/` and nothing else, skips what is already current, and refuses five ways —
  a destination inside the store, a never-list directory **already at the destination**, an
  unclassified path, a file that is not `.age`, a register with no bundle behind it. The second is
  the one that matters: every other refusal inspects what is being sent, and only that one can
  catch a `store/keys/` uploaded by hand months ago, after which no erasure is real.

  `push` fills the folder; the vendor's client does the network leg. The output says so, because
  "uploaded" otherwise reads as "arrived".

- **A cross-machine restore rehearsal, in containers.** `rehearsal/rehearse.sh`, `rehearsal/Dockerfile`.

  Four containers on a private network with no shared filesystem between the nodes. Node A creates
  people, backs up, pushes the bundle to an off-host container, and is then **deleted along with
  its volume**. Node B, which has never held the store, pulls the bundle, restores, verifies every
  chain, and continues the sequence at `PAR-P-000004-4`. Thirteen assertions, four of them
  negative: a stranger's key cannot open the bundle, a second node on a shared store is refused by
  name, no identifier is allocated twice, and the API answers on its own loopback while being
  unreachable from another container.

  **Not a deployment artifact.** docket is an operator CLI on a workstation; the image exists so
  that a restore can be proven across a real machine boundary, which two temporary directories on
  one host cannot do however carefully the test is written.

- **An exclusive store lock, because the concurrency hazard was already live.** ADR-0072 D2.

  The loopback server's accept loop is serial precisely because *"this process is the only writer
  to four append-only hash chains"*. That covered the server and not the CLI, and `append_chain`
  is read, seal-onto-last, append with nothing in between. Ten concurrent `docket new` runs
  against one store **duplicated the person identifier in five of five pairs** and forked the
  chain in four; with the lock, five refusals and five clean entries. Taken by every writing
  command, `create_new` rather than `flock` because `unsafe_code` is forbidden and this needs no
  libc. The refusal names the holder's host, pid and operator, so a stale lock is a decision
  rather than a guess.

### Fixed

- **A ceased officer went on signing new documents.** Routing names an *office*; only the register
  knows who holds it. `resolve_signatories` substituted only when the routed signatory was the
  *subject* of the document, never when the office was vacant — so marking an officer
  `active: false` (RFC-0042 §5's way of recording that they ceased) left their name printed on
  every document issued afterwards, and `docket check` reported nothing because `signed_by` was
  validated against every signatory rather than the active ones. Found by simulating a removal.

- **`ceased_on` did not exist.** RFC-0042 §5 requires ceasing to be `active: false` *plus* a date;
  the struct had no such field, so half its rule was unimplementable. Added, with a check that
  refuses `active: false` without it — a register that cannot say *when* an officer ceased cannot
  say whether they held office on the date a given document was issued.


- **CI was red for five days and nothing said so.** One test —
  `the_keystore_is_never_in_what_gets_uploaded` — failed on every runner and passed on every
  developer machine, because `Sandbox::new` called `age-keygen` unconditionally for a test that
  reads two paths and needs no cryptography. The guard belongs on the resource the test uses, not
  on the fixture that builds it. D-45, the second occurrence.

- **Tectonic could never have rendered a Paridyum document.** `TEXINPUTS` was set for both
  engines and Tectonic does not read it, so every render failed with `File 'paridyum.cls' not
  found`. The invocation now passes `-Z search-path`, the engine's documented equivalent. This
  had been broken since the engine was added and no machine on the estate had Tectonic installed
  to find out (D-30). It is still not usable — see D-57.


- **Every per-person command silently targeted the wrong company.** `personnel::identify` took the
  last ledger row for a slug, which is right while a person holds one engagement at a time and
  wrong the moment concurrent engagements landed an hour earlier. Rohan resolved to Paridyum
  Cyber; his Paridyum office was unreachable by `status`, `next`, `record`, `pack` and `onboard` —
  and because `issue` reads the company off the engagement, **his Paridyum NDA would have been
  issued on Cyber's letterhead under a `PCY/` serial.**

  Ambiguity is now refused rather than resolved by recency, naming both engagements. A global
  `--company` selects one; a company the person holds no office at is an error rather than a
  silent fallback. `docket record` lists every engagement instead of the latest.


- **The roster hid half an office.** `identity::current` kept the last ledger row per person,
  which is right for a sequential change (an intern who becomes an employee shows as an employee)
  and wrong for a concurrent one: it showed Rohan at Paridyum Cyber and silently omitted that he
  is also CTO of Paridyum. Now keyed on person **and** company. The summary counts people and
  engagements separately for the same reason — four officers were reading as five.


- **Two refusals in `store push` that lied, both caught by tests rather than review.** The
  plaintext check bailed inside the copy loop while reporting "Nothing was uploaded" — false for
  every file already written; every file is now validated before any file is copied. And the
  inside-the-store guard used `starts_with`, which is purely lexical, so a relative
  `store/uploads` walked past it and wrote into the directory it was protecting; a `..`-laden path
  defeated the first fix too. Destinations are normalised lexically before comparison.


- **A duplicated person identifier could pass `docket check` in silence.**
  `docket.identifiers_are_never_reused` was declared against ADR-0051 on 2026-08-19 and never
  implemented. Detection was an accident of the torn chain that usually accompanies a race: one
  of the five racing pairs appended *cleanly*, producing the same identifier twice with correct
  links, and check passed it. Now asserted directly.

  Reuse is one identifier across two **slugs**, not one identifier on two rows — the first
  version counted rows, called an erasure's appended row a duplicate, and failed
  `every_chain_still_verifies_after_an_erasure`.

- **`server.rs` documented itself as "one thread per connection"** while its body is deliberately
  one connection at a time. A stale line inviting someone to restore the threading the comment
  below it argues against.

- **The vault: one `age` key per person, so erasure has a mechanism.** ADR-0069.
  `docket vault seal | status | sync | tombstones`.

  **Permanence and erasure cannot both hold for the same bytes.** A key that always opens a
  document makes it uneraseable; no such key makes loss permanent. Sync.com keeps file versions,
  so deleting a file there is not erasure — and this estate already gitignores `people/` rather
  than accept an unerasable record. So the split is by whether erasure applies: the **register**
  is encrypted to the estate key and never erasable, the **vault** to a per-person key and
  erasable by destroying it.

  **What makes the split work is that `verify` compares a PRESENTED file.** An erased person can
  still hand over their own certificate in 2036 and have it verified against a surviving register
  entry. Erasure removes Paridyum's copy, not their ability to prove what they were issued —
  covered by a test that erases somebody and then verifies their kept copy.

  Encrypting to a person's key **and** an estate key was rejected as escrow in disguise:
  destroying the personal key would shred nothing. `crypt::encrypt` therefore takes one
  recipient, so adding a second is a change to the function where the argument against it is
  written down.

- **`docket erase` now destroys the vault, the plaintext and the key**, and writes a tombstone —
  identifier, date, operator, counts, and no name. Previously it deleted the record and **left
  every issued PDF in place**, which made the erasure decorative.

  The tombstone is written *before* the key is destroyed, so a crash between them leaves a
  tombstone with a surviving key — visible and recoverable — rather than a destroyed key nothing
  recorded.

- **`docket vault backup`** — the register, ledger, event log, signatures and tombstones,
  encrypted to the estate key into the uploaded path.

  **`seal` covered issued PDFs and left the more important half unprotected.** The register is
  what makes a document verifiable decades later, and none of it was in any uploaded path.
  Encrypted to the estate key rather than a person's because it is the non-erasable half — which
  is also what makes it safe in a versioned store, where there is no erasure promise for a
  retained version to break.

  The bundle names its five files individually rather than sweeping `store/`. A glob would have
  taken `keys/`, and a key inside a versioned store cannot be destroyed — after which no erasure
  this system performs is real. A test asserts nothing under `keys/` reaches the uploaded path.

  Recovery is rehearsed by a test rather than asserted: decrypt with nothing but the estate key
  and confirm the serial and hash come back, which is exactly what `verify` needs.

- **Signed scans are sealed and erased with everything else.** They were in neither the vault
  nor the register bundle, so they were in nothing that ever left the laptop — and `erase`
  deleted the issued PDFs while leaving them, which erased the copies the person already holds
  and kept the one they do not.

  **They are the irreplaceable half.** An issued document was handed over in a pack, so losing
  Paridyum's copy is recoverable from the counterparty. `sign.rs` says why a scan is not:
  *"wet ink leaves no trail of its own, so the scan IS the evidence."*

- **`docket vault escrow <path>`** — every per-person key, encrypted to the estate key, for
  offline media.

  **Crypto-shredding needs destroyable key material, so a backup of it weakens erasure by exactly
  as much as it improves durability.** No scheme gets both. Without an escrow, erasure is
  provable and a lost laptop loses every vault; with one, the laptop is survivable and erasure
  becomes *asserted* — docket destroys its copy and cannot reach the shelf. The command prints
  that trade before the operator files the medium.

  It refuses to write inside the store, which is what gets uploaded. And `docket check` warns
  after any shredding that an earlier escrow still holds the destroyed key — the only warning in
  the system about something docket cannot observe.

- **`docket check` warns about anything issued and not sealed**, and about a register backup that
  is missing or older than the register it covers.

- **`template_sha256` now covers every file a template includes, not just the top one.** It
  feeds `verification_code`, which is printed on the document, so it is the estate's claim about
  which text was issued. The moment a template gains an `\input`, a hash of the top-level file
  stops being that claim: two documents whose shared clauses differ would record the same hash.

  Done **before** any clause was extracted, and while the register held **zero entries** — after
  the first issuance there would have been two eras of hash in one register with no way to tell
  them apart. Nine unit tests, including that a changed clause moves the hash, that *moving* a
  clause moves it too, and that a cycle is refused rather than followed.

- **`templates/clauses/governing-law.tex`** — one home for a clause that existed five times, two
  copies differing only in where the line wrapped.

- **`tests/repository.rs`** — facts about the checkout that every other suite passes through,
  because a test running here has the working tree in front of it: that every class template
  reaches a fresh clone (D-49), that no real person record is tracked, that every declared
  template resolves its includes, and that an adapted template is reviewed when its source
  changes (D-46).

- **Export receipts.** `docket team` records the digest of what it exported from;
  `docket check` warns when the registries have moved since (D-48). A missing receipt is
  **silent** rather than a warning — it lives in the gitignored store, so warning would fire on
  every fresh checkout forever, which `lifecycle.yaml` already records as how a genuine warning
  gets scrolled past.

- **`docket addresses <slug>`, and `access.mailbox_email`.** ADR-0067 — nothing in this estate
  said what a person's address looked like, and `C-0007` governs repositories, crates, API paths,
  tables, env vars and images but not mailboxes.

  Two addresses, derived from the legal name and the person identifier rather than typed, so they
  cannot disagree and neither is invented at the keyboard:

      published  firstname.p000147c7@paridyum.com          the identity
      mailbox    firstname.p000147c7.paridyum@gmail.com    where mail lands

  **Anti-collision by default: the address is derived from the person identifier, so it cannot
  clash.** ADR-0051 already guarantees that identifier is unique across Paridyum forever and is
  never reissued, so an address inherits both properties instead of enforcing them.

  That deletes a subsystem rather than adding one. A `firstname.lastname` scheme cannot issue the
  next address without a register of every address ever issued — including for people who left,
  and for people whose records were erased, since reuse delivers one person's password resets to
  another. There is nothing to register here, and nobody's address is worse because they joined
  second.

  It also makes an address self-validating: a mistyped identifier fails its MOD 11-2 check
  character rather than bouncing silently. The cost, stated in the ADR rather than hidden, is
  that the address is not guessable and is harder to read aloud — which is survivable because
  `hello@`, `legal@` and `security@` are what inbound strangers actually use.

  **`intern.paridyum.com` was proposed and rejected.** It is ADR-0051's locked mistake in a worse
  place: class is mutable for a person, and the predictable case — an intern is hired — leaves an
  address that either lies or must be renamed. A rename is not available, because most copies of
  an address are in other people's address books, on signed documents, and in commit authorship
  that C-0003 requires be attributable. The real argument for a subdomain is sending reputation,
  and the standard answer to that is a subdomain for *bulk and automated* mail — about what sends,
  not who sends it.

  The mailbox carries the **person identifier** rather than a second hand-kept counter. It is
  already unique across Paridyum forever, already refuses reissue, and carries a check character
  against transcription error. A second sequence would be a second thing that must never be
  reused, which ADR-0051 names as "the one failure that silently reattaches one person's
  documents to another."

  `access.company_email` now means the published address specifically, and
  `access.mailbox_email` is separate — because **severing an alias is provable and recovering a
  consumer mailbox is a request.** The record must not report the second as the first. That
  distinction is a direct cost of the operator's decision to use free Gmail long-term rather than
  Workspace, and ADR-0067 lists what else that costs: no admin console, no enforced 2FA, no audit,
  no legal hold, no recovery.

- **`COO`, `ISO` and `DPO` — the Code of Conduct, Acceptable Use Policy and Data Protection
  Notice, as officer documents.** Closes the half of D-46 that mattered most: an officer holds
  strictly more access than any intern will and had agreed to none of it.

  **Not word-swaps of the intern policies.** Several sections are substantively different and
  two are inverted, so a find-and-replace would have produced documents that are wrong about
  the one person they most need to be right about:

  - *Speaking publicly* — an intern may not speak on Paridyum's behalf. An officer may, and the
    useful rule is about care: claims must be true when made, not merely intended.
  - *Access* — an intern may access only what an assignment requires. An officer's access is
    administrative, so least privilege becomes a discipline they impose on themselves rather
    than a boundary somebody else drew. Being able to reach something is not a reason to.
  - *Security testing* — an officer may authorise it, so the rule is that authorisation is
    recorded and scoped beforehand, and that nobody authorises their own testing of a system
    they alone administer without telling another officer.
  - *Incident reporting* — `org.yaml` routes incidents to the CTO. An incident involving the CTO
    cannot be reported to the CTO, so the officer policy names another officer as the route.
    This is the one route in the policy that could otherwise point at its own subject.
  - *Breach* — an office is vacated or removed by the directors, not terminated. There is no
    Internship Agreement to point at, and the Grievance Policy is an intern document, so the
    route for a concern about an officer is another officer.

  **The Data Protection Notice differs in substance, not only wording.** It lists what the
  officer class actually permits — no education, no statutory identifiers, no emergency
  contact — because a notice that overstates what is held is a defect in the notice. It also
  states the two facts that are *published* rather than merely held: name and office, on the
  public site.

  Wording is a draft for review by qualified counsel, as the intern policies already say, and
  more so here: the duties an officer owes are not written down anywhere in this estate yet.

- **`docket team`, and `registry/team.yaml`** — what Paridyum publishes about its officers.

  Three files, three purposes: `signatories.yaml` is authority, `operators.yaml` is access,
  `team.yaml` is disclosure. A name and a designation are NOT restated in the third — they live
  in the first and `docket team` joins them, because the office someone holds must not be able
  to differ between the document they sign and the page that names them. Same rule `org.yaml`
  states for the office address, for the same reason.

  Nothing in it derives from `people/`. That directory holds personal data under a stated
  purpose per field and is gitignored so DPDP erasure stays honourable; a home address must not
  become publishable by living in the same system as a job title. What is published is a
  decision taken in a reviewed diff.

  It emits JSON on stdout for `web-paridyum` to commit rather than being read across
  repositories: there is no shared build (ADR-0001), and a public page reading a gitignored
  directory at build time would render differently depending on which machine built it.

  The export carries `documents_signed_by` — who countersigns each officer — computed with the
  **same function issuance uses**, so the public page and the paper cannot disagree.

- **`scripts/bootstrap-officers.sh`** — two passes, and the pause between them is the point.

  Pass one opens four records and stops. It does NOT invent a date of birth or an address: the
  template ships `FILL-ME` because it used to ship realistic values and two freshly enrolled
  people displayed as the same person with nothing flagging it. A bootstrap that filled them in
  to look complete would be that defect on purpose. Pass two checks, then issues — and `docket
  issue` refuses a record still holding a placeholder, so pass two cannot quietly succeed on an
  unfilled one.

### Changed

- **A document routed to its own subject is now SUBSTITUTED rather than refused.**

  Officer documents route to the Director (RFC-0042 decision 4), so the Director's own routed to
  himself and issuance refused — correct, and unhelpful, since his documents still had to exist.
  The routed signatory is now replaced by the first active signatory in registry order who is
  not the subject.

  **Registry order rather than anything cleverer**, because the substitute must be the same
  person every time: who signed a legal document must not depend on hash order or on how many
  documents came before it. Refusal survives for the one case no ordering fixes — every
  eligible signatory being the subject.

  Verified end to end: the Director's own NDA renders with **RUHANI KHURANA, Chief Executive
  Officer (CEO)** in the Paridyum signature block and PRIYANSHU SHARMA countersigning as
  Officer. Asserted against `pdftotext` output, not the register, because the register can be
  right while the paper is wrong and the paper is what somebody signs.

- **`Entry.signed_by`** records who actually signed, since after substitution it is no longer
  derivable from `lifecycle.yaml`. `skip_serializing_if` keeps existing entries byte-identical,
  and therefore their sealed digests.

### Fixed

- **`people/_TEMPLATE-officer.yaml` was gitignored, and would not have reached a clone.**
  `people/*` is ignored so personal data never enters history, with one negation per template.
  The new one had none, so `docket new <slug> officer` would have fallen back to the intern
  template on any fresh checkout — producing a record carrying `education`, which the officer
  class forbids, and which `docket check` rejects as over-collection the instant it is written.
  A glob negation was rejected: this directory is the one place where a false negative is
  personal data in git history. Recorded as D-49.

- **Two intern preconditions were refusing issuance for every class.** `blockers` reported
  `education.currently_enrolled is false` and `engagement.end_date is unset` for an officer.
  Both are the intern class's and say so in their own message text — they exist because an
  unpaid student placement is not employment. `person-classes.yaml` forbids `education` for an
  officer and requires no end date, so **the refusals were unsatisfiable**: the operator could
  not clear them without breaking a different rule.

  Found by running the bootstrap, which is the argument for running it rather than reasoning
  about it. The DPDP s9 child check deliberately stays unscoped: it is about processing a
  child's data and says nothing about internships.

- **An `officer` class, and documents scoped to the class they belong to.** RFC-0042.

  Paridyum's officers were covered by nothing. CEO, CTO and Director existed in
  `operators.yaml` and `signatories.yaml` — files that say who may run the tool and who may
  sign for Paridyum — and nowhere else. No person record, no NDA, no IP assignment, while
  every intern will sign all three. The people who write most of the work product were the
  unlogged exception.

  A fourth officer is being appointed, and there was no defined way to do it.

  **`lifecycle.yaml` now declares `classes:` on every document**, written out explicitly
  rather than defaulted. Before this the set was implicitly "all fourteen", which is correct
  only while every person is an intern. `docket issue` refuses a document whose class does
  not match: a Certificate of Internship issued to a director is a document that is FALSE,
  and a false document carrying a serial and a register entry is worse than none.

  **The stage gate is narrowed to the class too**, and it had to be. `preboarding` requires
  the internship agreement and the intern IP assignment executed; an officer receives
  neither, so an unfiltered gate would have refused to advance them forever, waiting on a
  document nothing would ever produce.

  **Nobody signs their own document.** `signed_by_class` routes an officer's documents to
  the Director, and issuance is refused where subject and signatory resolve to the same
  person — a CEO's NDA signed by that CEO is evidence of nothing. Compared on legal name,
  because that is the string a reader of the PDF sees and the two registries share no key.
  Asserted against the RENDERED PDF TEXT rather than the register, because the register can
  be right while the paper is wrong.

  **`\paridyumsignblock`** resolves the signature block from the document instead of the
  template naming the CEO/CTO pair in twenty-one places. One template now serves both, and
  `signed_by` in the registry is the single statement of who signs.

- **`NDO` and `IPO` — officer NDA and IP assignment, as their own templates.**

  An earlier draft of RFC-0042 said `NDA`, `COC`, `ISP` and `DPN` applied to an officer
  "unchanged". **They do not, and implementation is what found it.** `nda.tex` says "Intern"
  sixteen times and "internship" six; `infosec-acceptable-use.tex` says them twenty-eight
  and eleven; `code-of-conduct.tex` states "This internship is unpaid and part-time", which
  is false of a director. The party term is a macro away, but "the internship" is the
  ARRANGEMENT, not the person — generalising it would edit the operative text of documents
  interns have already signed, for the convenience of a different class of person.

  So officers get their own two, and the intern set stays `classes: [intern]`. The cost is
  two copies of confidentiality prose that will drift, and nothing checks that they agree.
  It is the smaller cost. **`COC`, `ISP` and `DPN` for officers are not written**, and their
  absence is recorded rather than papered over with a claim that they applied.

  The assignee is PARIDYUM, not an individual — operator decision, recorded in RFC-0042 §3,
  with §8's future-entity clause carrying it into the company on registration, the same
  clause the Internship Agreement already relies on. Worth confirming with counsel at
  incorporation; it is an operator decision, not a legal opinion.

- **`people/_TEMPLATE-officer.yaml`**, and `docket new` picks a class-specific template
  where one exists.

  `_TEMPLATE.yaml` is the intern record: education, a guardian block, certificate content.
  Copying it for an officer produces a record `docket check` rejects as over-collection the
  moment it is written, which reads as a product defect rather than as the registry working.

- **`engagement.job_id`** — optional, declared in `fields.yaml`, naming the published posting
  an engagement answered. RFC-0041, implementing ADR-0063 decision 4.

  It makes "who did we hire for this posting" a question with an answer. Optional and staying
  optional: a direct approach, a returning intern or a founder has no posting to name, and a
  required field would be satisfied with a fiction. Nothing resolves it — the repositories
  have no shared build, and a document issued today must render identically in 2036 when the
  posting is gone. A typo in it is caught by nothing until both live in one store, which is
  stated in RFC-0041 rather than hidden.

- **Appointment provenance on a signatory** — `appointed_on`, `instrument`, `note`.

  Adding a name to `signatories.yaml` decides who may bind Paridyum: governance-altering
  under C-0004, needing a human and a dated instrument. The evidence now travels with the
  entry instead of in someone's memory. This file has recorded a name in error once already.

- **ADITYA SHRIVASTAVA recorded as Chief Operations Officer**, in both registries.
  **`appointed_on` and `instrument` are `null` and the entry says so.** The operator
  confirmed the appointment; the instrument has not been produced, and the legal-name
  spelling has not been checked against one.

- **`scripts/gates.sh`** — fmt, clippy, the test suite and `docket check`, each exit code
  reported separately, nothing piped.

  Not a convenience. Running the four by hand is how a failing gate gets recorded as
  passing: `cargo fmt --check 2>&1 | head -20` followed by reading `$?` returns the exit
  code of `head`, so a visible diff reads as clean. That happened twice in one day, both
  times by someone who had already written the rule against it down. A lesson in a document
  does not install itself. Confirmed to fail on the exact defect that slipped through, and
  to pass once it was fixed.

- **`/api/health` carries a wire contract version, and `scripts/serve.sh` cannot start a
  stale binary.** Two halves of one fix.

  `contract` moves only when the shape of a response changes in a way a client would read
  wrong — unlike `version`, which moves whenever anything ships. So a console can decide,
  from one cheap request that needs no store, whether it is talking to a build it
  understands, instead of printing a confident guess in prose.

  `scripts/serve.sh` builds before it serves, so there is no ordering left to get wrong, and
  it refuses an occupied port rather than spending a minute building and then dying on a
  bind error. That second check is not hypothetical: while it was being tested, a `docket
  serve` from an earlier session was still holding 8080, answered a readiness probe, and was
  briefly mistaken for the new server.

  Both exist because their absence had a cost. A binary from before a shape change served
  the old shape indefinitely; `console-docket` refused every response, correctly, and the
  operator saw a React error code and a digest. The instruction on offer was "remember to
  rebuild", and a sentence telling somebody to remember is not a fix.

- **A document can declare itself `on_request`**, meaning it is issued when somebody asks
  rather than owed by a lifecycle stage. `docket check` no longer warns about one, and now
  **refuses** a document that is both `on_request` and owed by a stage.

  PCK, LTR and PUB were each already carrying a `note` saying the warning was intended — so
  `check` reported three warnings on every clean run, for years of runs to come. A warning
  list that is never empty is one an operator learns to scroll past, which is exactly how the
  next genuine orphan would have hidden among the three known ones. It reports 0 errors and
  **0 warnings** now.

  The new refusal is a real control, not tidiness: PUB is consent to use a person's name or
  photograph, which the DPDP s7 employment legitimate use does not cover. A consent a stage
  *requires* is not freely given, so a stage quietly coming to owe it is the mistake worth
  blocking.

- **`fields.yaml` declares `choices` and `default` per field**, and both interfaces
  offer them. A field whose declared set is exactly `true` and `false` is a flag, which
  is how the record decides to write a bare boolean rather than the string `"true"`.

  These two facts already existed — as `is_flag` and `default_for`, two hardcoded
  `match` arms in `application::enrol`. They were a second statement of the registry's
  vocabulary and only the CLI could read them: `docket new --interactive` offered a
  default and a true/false answer, while the console's enrolment wizard, generated from
  the same catalogue, rendered a free-text box for the same field. Both matches are gone
  and both interfaces read the registry, so they cannot drift again.

  `GET /api/form/{class}` sends both. It stays silent about types it could only infer —
  `engagement.start_date` looks like a date and the API says nothing, because whether
  "on joining" is a valid answer belongs here and not to a browser.

- **`docket check` refuses two ways a declared set goes quietly wrong.** A `default`
  outside its own `choices` is blocking — the interview writes it on a blank answer, so
  every record opened that way would be wrong from the first field. And
  `engagement.stage` offering a stage `lifecycle.yaml` does not declare is blocking,
  because a record opened at it would sit at a stage no gate can ever match. The reverse
  — a declared stage nobody offers — is a warning, since not being able to open a record
  at `alumni` is a reasonable thing to decide on purpose.

### Added

- **Write routes on `docket serve`,** so the console can do the job rather than
  only describe it: `POST /api/person` (enrol), `/onboard`, `/sign`, `/advance`,
  plus `GET /api/form/{class}` for the enrolment questions.

  **Every write resolves an operator or is refused with 403.** An
  `X-Docket-Operator` header, checked against `registry/operators.yaml`. This is
  attribution exactly as strong as the CLI's `--by` — a claimed identifier that
  must name a real operator, proven by SSH rather than by us — and it is refused
  rather than defaulted, because a default is what made every event in the log say
  "docket". It doubles as the CSRF defence: a cross-site form cannot set a custom
  header, so the token the original design specified is unnecessary.

  **A refusal is 409, not 400 and not 500.** 400 means the request was wrong, 500
  means we are broken, 409 means it was understood and the domain declined. Only
  the third is information an operator can act on, and flattening them is how a UI
  comes to render a refusal as an error and invite a retry. Refusals carry the same
  `blockers` shape the person page renders, so a refused write and a held person
  read identically.

  **The routes do not expose `--force`.** `onboard` cannot be forced at all from
  here; §8 forbids an interface that offers a way past a refusal, and a console
  button that issues defective documents is worse than no button. `advance` accepts
  `force` because the CLI has it and the boundary mirrors the CLI, but it defaults
  off, so a console that does not send it can never skip a gate.

  `sign` checks the serial was issued to the person in the URL. Without that a
  request could file a signature against anybody's document through anybody's URL,
  and the signature ledger would be correct while the audit trail lied.

- **`docket serve` — the operator API `console-docket` consumes.** Read-only:
  `/api/health`, `/api/roster` (with `?standing=`), `/api/person/{slug}`,
  `/api/check`, `/api/retention`, `/api/emails/{slug}`. Every route calls the
  same `application::` function the CLI calls, so the console and the command line
  cannot disagree about what a person owes, and **no route writes anything** —
  asserted by a test over every route rather than argued from the absence of write
  handlers.

  **Binds `127.0.0.1` and refuses anything else**, as a function with tests rather
  than a default. There is no login: SSH keys are the authentication and the
  operating system is the identity (ADR-0047 gives this tool no identity system of
  its own, so it must not be reachable without one). Verified live that the
  listening socket is `127.0.0.1` only and that the LAN address refuses the
  connection — the property `next dev` in `console-docket` fails.

  The HTTP implementation is ours, on `std::net`, adding **no dependency**. The
  obvious crate for the job last published in 2022, and the needed surface is one
  client over loopback with no TLS and no keep-alive. It is strict by choice:
  chunked encoding, absolute-form targets, HTTP/2 and unparsable `Content-Length`
  are refused rather than guessed at, with bounds on the request line, header
  count and body size, and a read/write timeout so a silent client cannot hold the
  accept loop.

  Connections are handled **one at a time, deliberately**. This process is the only
  writer to four append-only hash chains, and that is what makes them safe without
  file locking; a thread per connection would make two concurrent writes possible
  and a torn chain the failure.

  Wire types live in `adapters::http::dto` rather than `#[derive(Serialize)]` on
  the `application::` types, so renaming an internal field cannot silently rename a
  field in a published API.

- `docket` CLI in Rust: `status`, `onboard`, `issue`, `render`, `verify`, `check`.
- `latex/paridyum.cls`, the single typesetting authority. Absorbs the
  letterhead, boxes, footer, page numbering and signature blocks that were
  duplicated across eleven separate `.tex` files.
- Fourteen document templates. Ten transcribed from the `.docx` originals,
  which are the source of truth for content.
- Four documents that did not exist: POSH policy, Data Protection Notice,
  Internship Offer, and an intern acknowledgement on the offboarding
  checklist.
- `registry/` — `org`, `signatories`, `lifecycle`, `retention`. One home per
  fact; `check` fails any template that restates one.
- Tamper-evident register: hash-chained JSONL, outside git so that erasure
  remains possible.
- Personnel record keeping: `docket new`, `record`, `roster`, `event`,
  `separate`, `erase`. Permanent identifiers (`PAR-INT-2026-0001`), never
  reused, including after erasure.
- `registry/person-classes.yaml` and `registry/fields.yaml`: 3 person
  classes, 54 declared fields, each with a stated purpose and a store.
- Class-gated collection. An unpaid student intern is not an employee, so
  no statute requires their PAN, Aadhaar, UAN or bank account; the `intern`
  class forbids them and `check` refuses a record that holds one.
- Hash-chained identity ledger and personnel event log, alongside the
  document register. The chain logic is one implementation shared by all
  three.
- DPDP erasure that actually deletes: the record file and the person's event
  detail go; the identifier, class and dates survive as a tombstone so
  issued serials still resolve.
- 96 tests.
- Vector logo, built from source at `latex/assets/paridyum-logo.tex`.
- `identity.date_of_birth` required for interns, a `guardian` field group,
  and `GRD` (Parent / Guardian Consent) owed whenever the person is under
  eighteen on their start date — DPDP s2(f) and s9.
- Verification code in the footer of every issued document, derived from
  serial + recipient + issue date + template hash. `docket verify`
  recomputes it. A lookup check, not a signature.
- The offboarding checklist now names the systems the event log records as
  still granted, and lists the documents on record with their serials. Both
  derived, so the paper cannot disagree with the register.
- **Two identifiers (ADR-0051).** `PAR-P-000001-X` is opaque, permanent and
  names the archive folder; `PAR-INT-2026-0001` carries class and year and
  names the engagement. Class is mutable for a person and immutable for an
  engagement, which is the whole distinction.
- ISO 7064 MOD 11-2 check character on the person identifier, because it is
  typed into folder names and read aloud. Catches every single-digit error
  and every adjacent transposition; both properties are swept in tests.
- `docket reengage` — an intern who is hired keeps their person identifier
  and gets a new engagement identifier, so the archive stays whole.
- The register keys on `person_id` + `engagement_id`. A slug is a filename
  and nothing keys off it.
- Issued PDFs are stored under the person identifier, not the slug: a folder
  named after a person discloses who Paridyum engages from the listing alone.
- **PDF/A-2b (ISO 19005-2)** via `pdfx`, with the sRGB OutputIntent the
  standard requires. Output is still byte-identical.
- **PDF metadata** — `/Title`, `/Author`, `/Subject`, `/Keywords` written from
  a generated `.xmpdata`. A file pulled out of the store is no longer
  anonymous.
- `docket index` — generated `roster.csv`, `documents.csv`, `README.txt` and a
  BagIt-style `manifest-sha256.txt`. The archive verifies with plain
  `shasum -a 256 -c`, with no Paridyum tooling. Generated output under C-0004;
  `docket check` fails if it is stale (ADR-0040's pattern).
- `docket retention` — reports documents past their period, and reports classes
  whose prose period it cannot parse rather than assuming them compliant.
- `docket erase` now names what it could NOT reach: the cloud folder, backups,
  mail, printouts. A tool that reports a complete erasure it did not perform
  is worse than one that states its boundary.
- `access.cloud_folder` field, so erasure can name the remote folder.
- `SOP.md` (STD-0013), six tasks with real pasted output.
- `provides` block in `manifest.yaml` — required by `manifest.schema.json` and
  absent before.
- `docket sign <serial>` — records that a document came back executed, in its
  own hash-chained ledger. Never by mutating the issuance entry: the register
  is append-only so what was issued cannot be rewritten, and a signature is a
  later fact.
- Refuses wet-ink with no scan (the scan is the only evidence that method
  leaves), a signing date before issuance, a received date before signing, a
  second signature for the same serial, and an unissued serial.
- Scans are copied into the store and hashed. `docket check` reports a scan
  that has gone missing or no longer matches its hash.
- `docket advance <slug>` — moves a person to the next stage and **refuses**
  while the stage's `requires_signed` documents are merely issued. That gate
  was prose in `lifecycle.yaml` and nothing checked it, which is how access
  gets granted before the NDA comes back. `--force` overrides and says so.
- `docket status` now distinguishes issued (`[~]`) from executed (`[x]`) and
  lists what is awaiting return. "Nothing outstanding" previously meant
  *issued*, which is not the question onboarding turns on.
- Signature state is a column in `documents.csv`.
- `tests/signing.rs`, 16 cases.
- `docket pack <slug>` — assembles a person's documents into a folder grouped
  by **what they must do with each**: sign and return, complete and return, for
  your records, optional. Derived from `lifecycle.yaml`, so a new document lands
  in the right folder without editing a list. Ships a `MANIFEST-sha256.txt` that
  verifies with plain `shasum -a 256 -c`, and is a copy — the store stays
  authoritative.
- **PCK** Read Me First — a cover sheet generated from the register, so it
  cannot describe a pack it was not built from. Says explicitly not to sign
  anything unread, and lists where to take each kind of question including the
  one that must not go through the supervisor.
- **INS** Institution Confirmation, **PUB** Publicity Consent (owed by no stage,
  deliberately — a consent a stage requires is not freely given), **COC** Code of
  Conduct, **GRV** Grievance Policy, **EXF** Exit Feedback.
- COC and GRV join the preboarding signature gate.
- `retention.yaml` classes now declare `from:` — the epoch a period is counted
  from.
- `.github/workflows/ci.yml` calling `infra-ci`'s reusable workflow, and the
  vendored `.paridyum/knowledge/` snapshot ADR-0001 requires and the workflow
  reads. `check-repo.sh` now passes.
- `CLAUDE.md`, `.claude/settings.json`, `.gitattributes`, and the `Not tested`
  / `Quarantine` / `Evidence` sections STD-0008 requires in `TESTING.md` — all
  four were missing and all four were caught by the repository check.
- `registry/communications.yaml` — the seven messages Paridyum sends, each
  naming its documents by code, so a message cannot reference a document the
  lifecycle does not have. `docket check` verifies it.
- `docket emails <slug>` — renders the messages due at a person's stage with
  placeholders filled and attachments resolved to serials. Reports an
  attachment that has not been issued and exits 1. Does not send mail.
- `HR-CHECKLIST.md` — one page, 34 checkboxes, every step a command.
- **Operator identity.** `--by <id>` or `DOCKET_OPERATOR`, validated against
  the new `registry/operators.yaml`. Every write records who made it; a write
  without an operator is refused rather than defaulted. Before this, every
  event in the log said `recorded_by: "docket"`, so the trail could say access
  was granted and never by whom — and that is not fixable in retrospect.
- **Tectonic as a second typesetter (ADR-0054).** `pdflatex` is preferred when
  present; Tectonic is used when it is not. Neither replaces the other. Each
  register entry now records `engine` and `engine_version`.
- `docket next <slug>` — the single next thing to do and the command that does
  it, ordered so that following it from the top requires no knowledge of the
  lifecycle. Exit 0 only when nothing remains.
- `tests/engines.rs` — verifies the Tectonic invocation with a shim, since
  Tectonic is not installed here: a wrong flag would otherwise surface only on
  the machine that has no pdflatex, which is the machine least able to debug it.
- `docket new --interactive` — asks the questions instead of handing over a
  YAML file. Prompts are generated from `registry/fields.yaml`: only what the
  class permits, only what belongs in the personnel store, the stated purpose
  as help, required first with an offer to stop there. Writes a
  self-documenting record that passes `check` unedited.
- `adapters/console` — prompt and confirm, with the closed-input case handled
  so a required question cannot spin forever.
- `docket --tui` — full-screen, read-only: roster, then a person's documents
  with signature state and their `next` steps. **No new dependency.** Reading a
  keypress needs raw mode, which needs `unsafe`, which is `forbid` here, so
  line-based navigation is the only permitted shape — and it avoids fourteen
  transitive crates. Read-only by construction: no input reaches a write path.
- `adapters/screen` — clear, heading, and character-counted fit/pad so a
  non-ASCII name is not cut mid-codepoint or thrown out of column.
- `issue::unrenderable` — reports fields a document prints that the record does
  not supply, as record paths rather than TeX commands, before issuing is
  attempted. Surfaced in `docket next`.

### Changed

- Person records now use the field paths declared in `fields.yaml`
  (`identity.legal_name`, `engagement.role`) rather than the ad-hoc shape
  the first version grew. Templates and the record template were migrated
  together.

### Fixed

- **An unedited record was issuable, and looked filled in.** `docket new` copied a
  template carrying realistic example values, so two freshly enrolled people
  displayed as the same person and `docket check` reported nothing — every required
  field was present. `docket onboard` would have issued twelve documents naming
  somebody else, each with a real serial sealed into the chain, correctable only by
  re-issuing under new serials.

  The template now ships `FILL-ME` wherever a person's own details go;
  `domain::person::unfilled` finds it at any depth, including inside sequences;
  `check` reports each as blocking; and `issue` refuses. **`--force` does not reach
  it** — the flag is for judgement calls, and nobody means to issue documents
  naming nobody. Values that are not personal data keep ordinary defaults, and the
  template says which is which so nobody has to guess.

- **`enrol::to_yaml` silently discarded answers to top-level fields.** Any field
  whose path has no dot — `pre_existing_ip` is the one that exists — was dropped by
  a `split_once('.')` with a bare `continue`. An operator answered the question in
  `docket new --interactive`, the answer went nowhere, and the only symptom arrived
  much later as `IPA needs: dkPPreExistingIp` from pdflatex: a macro name for a
  field they had in fact supplied. So the single field on the IP assignment
  agreement stating what an intern already owns was unanswerable through the
  interview.

- **A record generated by `docket new --interactive` could not be advanced.**
  `personnel::advance` searched for the literal substring `stage: {from}`, and
  `enrol::to_yaml` writes the stage quoted — `stage: "preboarding"` — so the search
  missed and the first `advance` failed with "could not find `stage: offered`",
  which was true and useless. A record with no stage line at all, which is what
  skipping the question produces, failed the same way.

  Now rewritten line by line, handling plain, quoted and trailing-comment forms and
  writing the line when it is absent. Fixing only the absent case would have been
  worse than fixing neither: inserting into a record that already had a quoted
  stage left **two** `stage:` keys in one mapping, and the record then meant
  whichever the parser kept. That is not hypothetical — it is how the bug was
  found, with the signing suite failing on `duplicate entry with key "stage"`.
  The trailing comment is preserved, because it lists the valid stages.

- **Onboarding an incomplete record reached pdflatex and returned a 500.** The API
  now runs `issue::unrenderable` before rendering anything and refuses with 409
  naming what is missing, rather than surfacing `Undefined control sequence` as a
  server error. Found by enrolling through the API with a partial answer set, which
  is exactly what a half-completed form sends.

- **Issuance was never recorded as an event.** `Kind::DocumentIssued` existed
  and was emitted nowhere, so a person's history had a hole where their
  documents should be.
- **`Entry.signed` was dead weight.** Declared `Option<Signature>` and never
  populated, so the register implied it tracked signature and did not. Removed;
  proven first that a `skip_serializing_if = "None"` field cannot change an
  existing entry's digest.
- **The Internship Offer was treated as needing no signature back**, because
  the rule keyed only on `kind: issued`. It carries an acceptance block the
  person signs. `needs_countersignature` now also honours `countersigned_by`.
- `templates/rust-service/manifest.yaml` in the knowledge repository was
  missing the required `provides` block — so the knowledge gate was failing and
  every repository scaffolded from it inherited an invalid manifest.
- `tests/signing.rs` took 51s because each case issued a nine-document pack.
  The default fixture now issues one document; the two gate cases ask for four.
  10s.
- The class's `draft` option leaked to `graphicx` as a global option, so a
  draft rendered a box containing the filename instead of the logo. Loaded
  with `[final]`.
- Pronouns recorded in a person record were silently ignored: the record
  nests them under `identity.pronouns`, generating `\dkPIdentityPronouns*`,
  while templates read `\dkPPronouns*`. The neutral default always won, so a
  record saying `she/her` printed "their". Aliased before defaulting.
- `identity.date_of_birth` rendered as ISO while every other date was long
  form, so a legal document carried two date formats.
- The offboarding checklist asked for a last working date the record already
  held, and broke lines so checkboxes were orphaned from their labels.
- Non-deterministic output: the PDF trailer `/ID` is derived from the wall
  clock by pdfTeX. `SOURCE_DATE_EPOCH` does not cover it; `\pdftrailerid`
  does.
- `SOURCE_DATE_EPOCH` now uses UTC midnight rather than local midnight, so
  output no longer depends on the operator's timezone. The archived Python
  implementation had this bug.
- Multi-page documents had no letterhead and no page numbering after page 1,
  so a page could be inserted or removed unnoticed. Every page now carries
  `Page X of Y` and the reference number.
- The `Page X of Y` line rendered at 845.8pt on an 845.0pt page and was
  silently dropped. The footer block is now anchored to grow upward.
- `\titleformat` targeted the numbered `\section`, which no document uses.
- **Retention measured from the wrong date.** Every period is stated relative to
  the END of the engagement — "8 years after the internship ends", "30 days of
  the last working day" — and all of them were measured from the document's
  ISSUE date. Wrong by the length of the engagement, and for the emergency
  contact actively harmful: 30 days after issue falls mid-internship, so the
  tool would have advised deleting a contact that was still needed.
- `tool-forge/manifest.yaml` declared `lifecycle: alpha`, which is not in the
  schema's enum, and had no `provides` block. Both required.
- **A wizard record could not issue.** Answering only the eight fields the
  `intern` class requires left the Internship Offer failing with `Undefined
  control sequence`, naming a TeX command rather than a field. The two lists
  are both correct and are not the same list — `requires` is about lawfulness,
  a template's needs are whatever it prints — so they are now checked
  separately. See DECISION-DEBT D-34.
- **The storage split was declared and unenforced.** `person-classes.yaml`
  declares four stores and the reason for each, yet third-party emergency
  contact details could sit in the personnel record and `check` passed with
  zero errors. `validate` now reports a field declared to live elsewhere.
  Forbidden is still checked first: for an intern's PAN, "no statute requires
  this" is the reason worth telling someone, not "wrong file".
- The interview asked date of birth before name, because paths sorted
  alphabetically. Small, and the kind of wrongness that makes a tool feel like
  a form.
- `personnel::create` refused to run after `--interactive` had written the
  record. The ledger decides whether a person is new, not the file.

Supersedes the Python implementation, archived at
`archived/docket-python-2026-08-19/`. Output verified byte-identical.
