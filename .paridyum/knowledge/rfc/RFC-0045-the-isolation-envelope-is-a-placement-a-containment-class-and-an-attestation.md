---
id: RFC-0045
title: The isolation envelope is a placement, a containment class, and an attestation
type: rfc
status: proposed
created: 2026-09-15
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, infra, web, console]
applies_to: ["*"]
product: [tenancy]
tags: [tenancy, isolation, residency, region, compliance, soc2, gdpr, dpdp, backup, restore, quota, byok, attestation, onboarding, adr-0024, adr-0038, adr-0039, adr-0041, adr-0049, adr-0064, adr-0094, adr-0100, req-0013, req-0022]
breaking: false
sources:
  - url: https://learn.microsoft.com/en-us/azure/architecture/guide/multitenant/approaches/governance-compliance
    title: "Architectural Approaches for Governance and Compliance in Multitenant Solutions — Azure Architecture Center"
    retrieved: 2026-09-15
  - url: https://learn.microsoft.com/en-us/azure/architecture/guide/multitenant/approaches/storage-data
    title: "Architectural Approaches for Storage and Data in Multitenant Solutions — Azure Architecture Center"
    retrieved: 2026-09-15
---

# RFC-0045 — The isolation envelope is a placement, a containment class, and an attestation

## Problem

Paridyum's tenant **isolation mechanism** is decided and built. Its **isolation envelope** —
everything a tenant, an auditor or a regulator asks about *around* the mechanism — is decided in
fragments across the artifacts listed under §Constraints, and built nowhere.

Observably, today:

1. **Every tenant shares one failure domain.** ADR-0064 runs one Postgres instance on two workers
   that are virtual machines on one hypervisor. One machine stopping stops every tenant at once,
   and no tenant can be recovered independently of the others.

2. **Every tenant shares one backup timeline, whose recovery point is a human action.** ADR-0094
   places Garage — the WAL and backup endpoint — in the same cluster it backs up, and its own
   decision 4 states that this is not disaster recovery. The only off-cluster copy is ADR-0071's
   manual ciphertext dump. The estate cannot currently state an RPO, because the RPO is "whenever
   someone last ran it."

3. **No tenant can be restored, exported, or deleted on its own.** REQ-0013 requires portability;
   RFC-0006 named per-tenant restore as the chosen model's weakest point and predicted the request
   would arrive. Neither exists. Recovering one tenant's deleted rows means a human extracting from
   a whole-cluster backup as superuser, with no tooling and no record.

4. **`tenant.region` is contractual per ADR-0041 decision 3 and is enforced by nothing.** There is
   no `region` column, no placement rule, and no mechanism that would prevent a tenant whose
   contract says India from having its rows written in any other jurisdiction — a distinction that
   is currently moot because only one region exists, which is precisely why it is cheap to fix now
   and expensive after the second region is built.

5. **A tenant cannot verify its own isolation.** REQ-0022 requires exactly that — "verifiable by the
   tenant, from outside the platform, without privileged access and without asking an operator" —
   and its `enforced_by` is empty. `verify-isolation.py` and `prove-isolation.py` are real and good
   and both are operator-side. There is no artifact a tenant can obtain.

6. **Privileged access to tenant data is unrecorded.** `retention.py` and every backup path connect
   as superuser and bypass RLS by design and by necessity. Nobody can currently answer who read
   which tenant's rows, when, or why — the first question any access review asks.

7. **Noisy neighbours are unmitigated at both silos.** ADR-0024 cost 3 admits it and classes quota
   and rate limiting as *correctness* features rather than billing features. Neither exists, so one
   tenant's bulk import degrades every other tenant with no ceiling.

8. **There is no answer to "can we have our own database/keys?"** other than silence. ADR-0024
   decision 8 defers escalation behind a funded trigger, which is a commercial position, not a
   technical path — no design exists for what a funded escalation would actually deploy, so the
   first contract that pays for one starts from nothing under time pressure.

**Who experiences it, and how often.** Items 1–3 and 6–7 are experienced by every tenant
continuously and invisibly until an incident, at which point all of them are experienced at once.
Items 4, 5 and 8 are experienced at the sales and audit boundary — a security questionnaire, a DPA
negotiation, a SOC 2 Type II readiness assessment — which is to say at exactly the moment the
answer cannot be built quickly.

**What happens if nothing is done.** Personal data is currently held behind ADR-0064 decision 3's
gate (an off-host destination and a rehearsed restore), and `0001_tenancy_foundation.sql` states in
its own header that no real applicant may be inserted until that gate passes. The gate is therefore
already blocking the platform's first real workload. Beyond that: the second tenant makes item 7
visible, the first EU tenant makes item 4 unanswerable, the first enterprise questionnaire makes
items 5 and 8 deal-blocking, and the first deletion incident makes item 3 unrecoverable. None of
these are hypothetical failure modes of the design; four of them are named as accepted costs in the
ADRs that chose it, with no artifact owning their discharge.

**How we will know it is fixed.** A tenant can obtain, without asking us, a dated statement of
where its data resides, what shares its failure domain, when it was last provably isolated, and
what its recovery point is — and each claim in that statement is produced by a check rather than
written by a human.

## Constraints

Locked and accepted artifacts that bind this design. Several of them decide parts of it already,
and this RFC discharges rather than revisits them.

- **ADR-0024 (locked)** — two columns, RLS enabled and forced, operator scope as a positive grant,
  no per-tenant infrastructure, and **decision 8: escalation is deferred behind a funded trigger,
  not published as a tier.** Decision 8 is the sharpest constraint on this RFC and §Proposal B is
  written to respect it rather than route around it. Decision 9 defers a third tenancy level behind
  a named trigger; nothing here proposes one.
- **ADR-0023 (superseded)** — retained for RFC-0006's arithmetic, which this RFC reuses.
- **ADR-0038 (locked)** — three accessors that raise rather than return `NULL`, set per transaction.
  Nothing here changes the session contract; the whole ladder in §Proposal B is designed so it does
  not have to.
- **ADR-0039 (accepted)** — tenant provisioning is a compensating saga, and its step 1 already
  inserts the tenant row **"(with `region`)"**. The column this RFC asks for is the one that ADR
  already assumed.
- **ADR-0041 (accepted)** — compliance is sequenced; SOC 2 / GDPR / DPDP are phase 1, HIPAA and PCI
  are gated. **Decision 3 names five things as immediate design inputs**: a shared correlation id,
  `tenant.region` as contractual and immutable, a key-management policy from the first deployment,
  every grant attributable and revocable, and classification at the creating migration. Four of the
  five are undischarged. This RFC is largely their discharge, and should be read as such.
  **Decision 4 forbids claiming a framework before an audit says so**, which binds every word of
  the attestation in §Proposal E.
- **ADR-0049 (locked)** — nothing runs outside a tenant; Paridyum is a tenant and holds no scope a
  customer could not hold. This constrains the restore, export and break-glass paths in §Proposal F
  and §Proposal H, which are the paths most likely to want an exception.
- **ADR-0064 (accepted)** — one Postgres instance is the honest shape on one hypervisor, and its
  decision 3 gates personal data behind an off-host destination and a rehearsed restore.
- **ADR-0094 (accepted)** — Garage is the in-cluster S3 and WAL endpoint; its decision 4 keeps the
  off-host copy manual and names that as an accepted cost.
- **ADR-0071 (accepted)** — the off-host store only ever holds ciphertext.
- **ADR-0100 (accepted)** — the provisioning surface is tenant zero's control plane at
  `<platform-zone>/dashboard`, a BFF over `platform-tenancy`, owning no entity. Every operator-facing
  screen in §Appendix A lands there and nowhere else.
- **ADR-0082 / ADR-0047** — a tenant is a Zitadel organization and its database id is
  `uuidv5(ns, org_id)`, derived without a lookup. §Appendix A's journey starts here.
- **ADR-0084** — branding is per tenant. The custom-domain decision ADR-0100 anticipates is not yet
  written; §Appendix A step 10 assumes it and nothing here depends on its number.
- **REQ-0013 (accepted)** — tenant data portability. Its export machinery is the same machinery as
  per-tenant restore pointed the other way; RFC-0006 said so and this RFC builds them together.
- **REQ-0022 (accepted)** — tenancy is a capability with an interface. `enforced_by` is empty and
  §Proposal E is what fills it.
- **REQ-0003** — classification at the creating migration; already implemented.
- **REQ-0008** — migrations deploy without downtime. **This is the constraint that killed
  schema-per-tenant and database-per-tenant in RFC-0006, and §Proposal B lives or dies on the
  argument that a bounded N is not the same as an O(tenants) N.**
- **ANTI-0007** — regulated data does not get a shared-database exception.
- **ANTI-0001** — one owner per fact; `platform-tenancy` owns tenancy.
- **GLO-0005** — a grant, never an absence.
- **C-0004** — irreversible and governance-altering operations require a human. Placement changes
  and containment escalations are both.
- **C-0006** — non-commodity item 2 claims *provable* isolation, not isolation. Anti-goal 5 forbids
  selling to enterprises before self-serve works.
- **BET-0001 / BET-0002** — low flat price, mostly-idle tenants, density is margin. The arithmetic
  that eliminated per-tenant infrastructure *as a default* and does not eliminate it *as a funded
  exception*.

**Where this RFC comes closest to contradicting a locked artifact**, stated plainly rather than
buried: ADR-0024 decision 8 says escalation is "deferred behind a funded trigger, **not published as
a tier**." §Proposal B defines a ladder of containment classes. A ladder is not a published tier
list if and only if rungs above the default are quoted against a contract rather than offered in a
signup flow, and §Proposal B constrains itself accordingly — the ladder is an internal engineering
vocabulary and a contractual instrument, not a pricing page. If the intent is ever to *publish*
rungs as self-serve options, that is a supersession of ADR-0024 decision 8 and needs C-0004's human
decision, not this RFC. That question is left open in §Open questions rather than answered here.

## Proposal

The design rests on one reframing, from which everything else follows.

> **Isolation is not one property. It is a *placement* (where the data is, which is contractual and
> jurisdictional), a *containment class* (what shares its failure domain, which is funded and
> operational), and an *attestation* (what the tenant can verify, which is continuous and
> tenant-facing). Today the estate has a strong mechanism and no vocabulary for any of the three.**

Azure's guidance separates the same axes — data isolation, resource isolation and governance
boundaries — and the antipattern it names first is assuming data isolation implies the others. The
current estate makes exactly that assumption implicitly: RLS is excellent, and the ADRs' own
accepted costs are the places where it was never claimed to be enough.

### A. The one primitive to build first: placement is resolved, never assumed

**Build a tenant→connection resolver now, at the shared class, where it is a no-op.**

Every request path today reaches `pool()` — a single module-level pool pointed at one DSN — and
then sets three session variables. The proposal is to insert one resolution step:

```
session (verified) ──▶ tenantId ──▶ placement resolver ──▶ pool for (region, containment)
                                                        └─▶ withSessionContext(tenant, account, operator)
```

The resolver takes a `tenantId` and returns the pool its rows live in. At the shared class it
returns the one pool that exists, so the change is behaviourally inert on day one. That is the
point: **this is the only change in the whole RFC that must happen before rows accumulate**, because
it is the seam that makes every other rung reachable without rewriting application code.

Three properties make it worth building while it does nothing:

1. **RLS stays on at every rung.** A dedicated database still carries `tenant_id`, still forces RLS,
   still runs the same migrations, still sets the same three session variables. Containment is
   defence *added*, never defence *substituted*. This is what keeps one codebase serving every rung
   and is the difference between a ladder and a fork.
2. **The resolver is the only place placement is known.** No request path, no query, and no
   application module learns which class a tenant is in. ANTI-0001: placement is one fact with one
   owner, and `platform-tenancy` owns it.
3. **A resolver that cannot resolve fails closed**, in the same shape as ADR-0038's accessors — an
   unknown or unplaced tenant raises, never falls back to a default pool. A fallback here would be
   the ADR-0024 decision 4 hazard at the infrastructure layer: breadth granted by a lost value.

Concretely: `placements` is a new platform-owned table (`-- paridyum:scope platform`, control
plane) keyed by `tenant_id`, carrying `region`, `containment_class`, `dsn_secret_ref`,
`placed_at`, `placed_by` and `contract_ref`. `libs/lib-tenancy` gains `resolvePlacement(tenantId)`
and every app's `pool()` is replaced by `poolFor(tenantId)`.

### B. The containment ladder — bounded N, quoted not published

A ladder of containment classes. Each rung names what stops being shared, what it costs, and the trigger that admits a
tenant to it.

| Class | What is dedicated | What is still shared | Trigger |
|---|---|---|---|
| **C0 — shared** (default, today) | nothing | cluster, database, schema, pool, cache, WAL, backup timeline, keys | none; every tenant starts here |
| **C1 — dedicated database** | database, credentials, backup timeline, restore, per-tenant PITR | Postgres cluster, host, storage, keys, compute | signed contract, or a regulated class under ANTI-0007 |
| **C2 — dedicated cluster** | CNPG `Cluster`, storage, WAL stream, encryption key (BYOK possible), failure domain | Kubernetes cluster, region, network, edge | signed contract naming a failure-domain or key-custody requirement |
| **C3 — dedicated stamp** | everything in C2 plus its own Zitadel instance, namespace and regional placement | the control plane's knowledge of it, and nothing else | signed contract naming a sovereignty or personnel-restriction requirement |

**C1 is the genuinely new rung, and it is the one that needs defending**, because RFC-0006 rejected
"one database per tenant" outright.

RFC-0006 option C evaluated database-per-tenant **as the default for every tenant**, and killed it
on two grounds: negative gross margin on the first tenant at BET-0001's price, and migration fan-out
worse than schema-per-tenant. Both objections are objections to **N growing with success**. Neither
survives when N is bounded by signed contracts:

- **Margin.** A C1 tenant is priced to cover a dedicated database. The BET-0001 arithmetic applies
  to tenants paying BET-0001's price, which C1 tenants by definition are not. BET-0002 charges per
  running business; a dedicated database is a running business with a different floor.
- **REQ-0008, which is the real objection.** A migration must deploy without downtime. Against
  10,000 schemas that is impossible. Against a handful of databases it is a fan-out loop with a
  ceiling that the *contract count* sets, not the customer count — and if that ceiling is ever
  approached, it is approached with revenue attached and is visible years ahead in the contract
  pipeline. **This is the load-bearing claim of §Proposal B and the one most likely to be wrong**;
  it is listed again in §Open questions with the number that would falsify it.

C1 also does most of the compliance work for the least money: it delivers per-tenant restore, a
per-tenant recovery point, an independent backup timeline, and a credential boundary — which is the
majority of what a security questionnaire actually asks — without a second cluster.

**Respecting ADR-0024 decision 8.** None of C1–C3 appears in a signup flow, a pricing page, a public
capability matrix or a README. They are an engineering vocabulary and a contractual instrument.
Entry to any of them requires a signed contract and a C-0004 human decision recorded against the
tenant. The estate does not *offer* them; it is able to *quote* them, which is what decision 8's
"funded trigger" means operationally and what it never spelled out.

**What is deliberately absent from the ladder.** No rung is schema-per-tenant. REQ-0008's argument
against it is unchanged and it buys nothing C1 does not buy better — a dedicated database gives a
real credential and backup boundary where a schema gives a naming convention.

### C. Placement is a region, contractual and immutable — ADR-0041 D3 discharged

`tenant.region` moves from prose to schema and to enforcement.

1. **A `region` column on `tenants`**, `NOT NULL`, validated against `region-registry.yaml`'s
   `regions[].code` where `status: live`. A jurisdiction that is declared but not built is not a
   placement; the registry's own comment already draws that line.
2. **Region is chosen at provisioning and is immutable thereafter.** ADR-0041 decision 3 says a
   tenant does not move regions without a migration and a contract amendment. Enforced as a trigger
   that refuses `UPDATE` on the column, so a move is necessarily a deliberate, privileged,
   recorded migration rather than an `UPDATE` somebody ran.
3. **The saga carries it.** ADR-0039 step 1 already says "insert tenant row (with `region`)". Step 2
   creates the Zitadel org "in the regional instance" — already region-aware. This adds nothing to
   the saga's shape; it makes the field it already names real.
4. **Placement is refused, never defaulted.** A provisioning request that names no region is
   rejected. `suggest_for` in the registry orders what an operator is *shown*; it never picks.
   RFC-0015 decision 5 already says an operator in either country chooses their own region.
5. **Cross-region staff access is a control that must be evidenced** (ADR-0041 D3 again). Any
   operator session whose own region differs from the tenant's region is recorded in the audit
   stream in §Proposal D with that fact as an explicit field, not inferred later from IP addresses.

One region is live (`in`, which is also `control: true`). Every rule above is therefore currently
satisfied trivially and costs almost nothing to add — which is the entire argument for adding it
now rather than when the second region makes each rule a migration against live rows.

### D. One correlation id, one audit stream, append-only — ADR-0041 D3 discharged

ADR-0041 decision 3 states that a shared correlation id must exist **from the first request**,
because it spans Zitadel's event log and our own records and "retrofitting one across two systems
means reconstructing history that was never written." Every request written so far has been written
without one.

1. **A correlation id is minted at the edge**, carried through the session context, and set as a
   fourth `SET LOCAL` alongside the existing three. It is not an accessor with a policy behind it,
   so it does not change ADR-0038's contract; it is available to triggers and to the audit stream.
2. **An `audit_events` table**, `-- paridyum:scope tenant`, append-only by the same trigger pattern
   `0006` and `0007` already use, carrying `correlation_id`, `actor`, `action`, `object`,
   `tenant_id`, `operator_scope`, `actor_region`, `at`. Exportable per tenant — which ADR-0041
   requires and which §Proposal F's export path delivers for free.
3. **Grants are attributable and revocable.** ADR-0041 D3's fourth input. Operator scope is already
   a positive claim; this records **who granted it, to whom, when, and how it ends** as rows rather
   than as a Zitadel role's implicit present tense. An access review is then a query.
4. **Retention on the audit stream is stated, and longer than the data it describes.** A deletion
   under GDPR Art. 17 removes personal data and must not remove the record that a deletion occurred.

### E. The attestation — REQ-0022's empty `enforced_by`, filled

The thing that makes this "Azure level" rather than "well built": **a tenant can verify its own
isolation without asking us.**

A signed, dated **isolation attestation**, retrievable by any tenant for its own tenancy from its
own console, machine-readable, every field produced by a check rather than typed by a human:

| Field | Produced by |
|---|---|
| `tenant_id`, `region`, `jurisdiction`, `regime` | `placements` + `region-registry.yaml` |
| `containment_class` | `placements` |
| `shares_database_with` | count of other tenants at the same placement — a number, not a claim |
| `rls_forced`, `app_role_no_bypass`, `app_role_not_owner` | `verify-isolation.py` against the live catalogue |
| `isolation_proven_at` | last successful `prove-isolation.py` run — behavioural, through the app role |
| `recovery_point_objective`, `last_off_host_backup_at` | the backup path in §Proposal F |
| `last_privileged_access` | the break-glass log in §Proposal H |
| `data_classes_held` | `paridyum:class` annotations, per REQ-0003 |
| `subprocessors` | the estate's own declared list |
| `attested_at`, `signature` | the generator, signed |

Two rules keep it honest. **It states no framework as satisfied** — ADR-0041 decision 4 and
ADR-0028's outward-mapping direction both forbid it; the attestation says what is *true and
checked*, and a framework mapping is a separate document that asserts contribution, never
attainment. And **a field it cannot produce is absent or explicitly `unknown`, never defaulted** —
the same fail-closed discipline ADR-0038 applies to session values, applied to evidence.

This also satisfies ADR-0049 decision 3 in the direction that matters: the tenancy capability
finally has a tenant-facing consumer, which REQ-0022 said a `tenant_id` column could never be.

### F. Per-tenant restore and export are the same machinery, pointed two ways

RFC-0006 observed that REQ-0013's export path and per-tenant restore share machinery and that this
is an argument for building export earlier than a pure compliance reading suggests. This RFC takes
that argument.

One tenant-scoped extractor, walking the declared scopes (`platform` / `tenant` / `account`) that
`check-migrations.sh` already validates, so it is correct by construction and stays correct as
tables are added:

- **Export** — a tenant requests its data; the extractor reads under that tenant's RLS context, as
  the app role, and produces a ciphertext archive. GDPR Art. 20, REQ-0013.
- **Restore** — the same extractor reads from a PITR clone rather than the live database, and the
  rows are written back under the same tenant's context. No superuser, no hand-written SQL.
- **Deletion** — the same walk, as deletes, producing a certificate of what was removed, written to
  the audit stream that outlives it. GDPR Art. 17.
- **Offboarding** — export, then retention hold for the contractual period, then deletion. The Azure
  guidance names this sequence; the estate currently has none of the three steps.

**And the prerequisite that is already a named gate.** ADR-0064 decision 3 and ADR-0063 §1 both hold
personal data behind an off-host destination and a rehearsed restore, and
`0001_tenancy_foundation.sql` says so in its header. ADR-0094 supplied the endpoint and left the
off-host copy manual. This RFC proposes making it **continuous and scheduled with a stated RPO**,
because an RPO that is a human's memory cannot appear in §Proposal E's attestation, and a restore
that has never been rehearsed is a claim rather than a control — which is REQ-0003 item 4's own
argument about retention, applied to backup.

### G. Quotas are correctness, not billing

ADR-0024 classes quota and rate limiting among its accepted costs as platform correctness features
rather than billing features, and BET-0002 reaches the same conclusion from pricing. Neither has an owner. Proposed: per-tenant connection
ceilings, statement timeouts, and request rate limits, enforced at the pool and the edge, defaulted
for C0 and raised by contract — so the ladder's rungs differ in *ceiling* as well as in containment,
and a C0 tenant's bulk import cannot consume the pool that every other C0 tenant shares.

### H. Privileged access is logged, bounded and visible to the tenant

Superuser paths exist and must — `retention.py` expires rows across every tenant, and backups read
everything. Azure's "your access to tenants' data" section is the control an auditor asks for first,
and SOC 2 CC6.1 wants it named.

Every RLS-bypassing session is **requested, reasoned, time-bounded, logged to the append-only
stream, and surfaced to the affected tenant** in §Proposal E's `last_privileged_access`. A tenant
seeing when we touched its data is uncomfortable and correct; it is also the single strongest thing
a small platform can offer against the assumption that a shared database means unlogged staff
access.

## Alternatives considered

### 1. Do nothing structural — keep the envelope implicit and handle each ask as it arrives

**Rejected.** This is the current state and it is already failing: four of the eight problems in
§Problem are named as *accepted costs* in the ADRs that chose the model, with no artifact owning
their discharge, and one of them (ADR-0064 D3's gate) is actively blocking the platform's first real
workload. "Handle it when asked" also fails at the worst moment by construction — every one of these
asks arrives during a sale, an incident or an audit, which are the three times nothing can be built
carefully. ADR-0041 decision 3's whole argument is that these five inputs are far cheaper now than
retrofitted, and this alternative is the decision to retrofit them.

### 2. Abandon RLS and go to database-per-tenant for everyone

**Rejected, and it is the alternative the operator raised directly.** RFC-0006's arithmetic is
unchanged and is not a matter of preference: at BET-0001's flat low price with mostly-idle tenants,
a dedicated database per tenant is negative gross margin on the *first* tenant and no volume
improves it. REQ-0008 independently kills it — migrations become O(tenants) and that ceiling arrives
exactly when the company is working. It would also discard a genuinely strong, database-enforced,
fail-closed mechanism that is already built, tested behaviourally through the app role, and
enforced by four declared checks, in exchange for a mechanism whose isolation is easier to *explain*
but no stronger against the threat that actually matters here — application-layer confusion about
which tenant is asking, which a separate DSN does not fix on its own. §Proposal B keeps this option
where it belongs: a funded rung, entered by contract, for the tenants who pay for it.

### 3. Adopt the deployment-stamp / cell architecture now

The Azure guidance's recommended shape for tenant-specific compliance requirements, and the correct
long-run answer. **Rejected as a starting point, not on merit.** A stamp is a unit of deployment
that presumes deployment automation this estate does not yet have — ADR-0064 runs one Postgres
instance on one hypervisor, and a second stamp on the same hypervisor is two names for one failure
domain, which is worse than one honest name. RFC-0006's own warning against building the escalation
path before anyone needs it ("how twelve months disappear into a control plane with no customers")
applies directly. C3 in §Proposal B *is* a stamp; the difference is that it is reached by a contract
rather than adopted as the default topology, and the resolver in §Proposal A is what makes reaching
it a routing change rather than a rewrite.

### 4. Buy managed infrastructure (Azure, AWS, a managed Postgres) and inherit its compliance posture

**Rejected, and it contradicts a settled direction rather than merely losing on cost.** RFC-0016
rents the edge and keeps the rest; RFC-0017 and ADR-0094 keep personal data and its bytes on
governed infrastructure; C-0006's non-commodity list is what would be given away. It would also not
solve §Problem's items 3, 5, 6 and 7 — per-tenant restore, tenant-verifiable attestation, privileged
access logging and quotas are all *application-layer* properties that no managed database supplies.
It would solve items 1 and 2 (failure domain and backup) genuinely and immediately, which is worth
recording honestly: this alternative buys the two hardest problems and none of the rest.

### 5. Publish the ladder as self-serve isolation tiers

**Rejected here, and it is not this RFC's decision to make.** It contradicts ADR-0024 decision 8 in
terms, and C-0006 anti-goal 5 forbids selling to enterprises before self-serve works. Publishing a
tier is a commitment to operate it on demand; quoting one is a commitment to operate it for a named
contract. §Proposal B takes the second. Raised again in §Open questions because the operator's
stated goal — reaching Azure-level governance — is in tension with it, and the tension should be
resolved by a human, not absorbed by an RFC.

## Consequences

**What this makes worse.**

1. **The ladder creates a second class of tenant, and the estate must keep both correct forever.**
   Every migration must apply to C0 and to every C1+ database; every check must run against all of
   them; every incident has two shapes. This is exactly the fan-out cost RFC-0006 used to reject
   database-per-tenant, taken deliberately at a bounded N — and a bound that is *believed* rather
   than *enforced* is the way this becomes the thing it rejected. §Open questions names the ceiling.

2. **The attestation is a promise that is expensive to keep and catastrophic to break.** A dated,
   signed, machine-readable statement of isolation is a document a tenant will hand to their own
   auditor. Every field becomes a thing that must never be wrong, and a field that is stale is worse
   than a field that was never offered. It also means a failing check is now visible to tenants
   before we have finished reading it.

3. **Privileged-access logging will be read, and the reading will be uncomfortable.** Surfacing
   `last_privileged_access` to tenants means routine maintenance becomes a visible event that
   generates questions. The alternative is not answering the question at all, but the cost is real
   and falls on a small team.

4. **Region immutability will block something legitimate.** A tenant that picked wrong, or grew into
   a different jurisdiction, cannot be fixed with an `UPDATE`. That is the intent, and it will still
   feel wrong the first time it costs a migration and a contract amendment for an obvious clerical
   error.

5. **The resolver adds a lookup to every request path.** Cached and cheap, but it is a new thing
   between a session and a query, and a new thing that can fail. It fails closed, which converts a
   subtle wrong-tenant risk into a loud availability risk — the right trade, and still a trade.

6. **Quotas will throttle a paying tenant at some point**, probably during their most important
   week, and the ceiling will have been set by a default nobody revisited.

7. **This RFC is large and touches many artifacts.** It discharges parts of ADR-0041, ADR-0064,
   ADR-0094, REQ-0013 and REQ-0022 and proposes new schema, a new library seam, new checks and a new
   tenant-facing surface. Sliced badly it becomes a six-month project with nothing shippable, which
   is why §Rollout orders it by what is irreversible rather than by what is architecturally tidy.

**What it does not make worse.** No rung changes ADR-0038's session contract, and RLS remains forced
at every rung. Application code learns nothing about placement. That is the design's main claim to
being affordable.

## Rollout

Ordered by **irreversibility**, not by architecture — what must land before rows accumulate goes
first, regardless of how small it is.

| Slice | What lands | Why this position | Reversible? |
|---|---|---|---|
| **1** | Correlation id through the session context; `audit_events` append-only; grant attribution | ADR-0041 D3: cannot be retrofitted across two systems. History not written now is gone | additive |
| **2** | `region` column, `NOT NULL`, registry-validated, immutable by trigger; saga carries it; placement refused not defaulted | Trivial at one region, a migration against live rows at two | hard after rows |
| **3** | `placements` table + `resolvePlacement` + `poolFor(tenantId)`; a no-op at C0 | The seam every later rung needs; inert now, a rewrite later | yes, inert |
| **4** | Continuous off-host backup with a stated RPO; one rehearsed restore, recorded | **Unblocks ADR-0064 D3's gate and the first real applicant row** | n/a |
| **5** | Tenant-scoped extractor: export, restore, delete, certificate | REQ-0013 + per-tenant restore, one machinery | yes |
| **6** | Break-glass: requested, reasoned, time-bounded, logged | Needs slice 1's stream | yes |
| **7** | Attestation generator and tenant-facing surface | Needs 1–6 to have anything true to say | yes |
| **8** | Quotas and rate limits, defaulted at C0 | Independent; ordered last because it degrades rather than leaks | yes |
| **9** | C1 reference implementation, exercised with Paridyum as the tenant (ADR-0049 D2) | Proves the ladder before a contract depends on it | yes |

**Slices 1, 2 and 4 are the ones with a deadline.** 1 and 2 because they are cheap today and
expensive after rows; 4 because it is already a written gate blocking the platform's first workload.
Slices 3 and 5–9 can be sequenced against demand.

**Backwards compatibility.** Slices 1–3 are additive; no existing query, policy or session path
changes behaviour. Slice 3's resolver returns the single existing pool until a C1 tenant exists.

**How it is undone.** Slices 1, 5, 6, 7 and 8 are droppable. Slice 3 collapses back to a direct pool.
Slice 2's `region` column can be made nullable but its values cannot be recovered if never written —
which is the argument for its position. Slice 9 is a deployment, removable with its tenant.

**Graduation.** Per §7 of the change lifecycle: the containment ladder, placement immutability and
the attestation all **bind future work** and are mechanically checkable, so they graduate to a
`locked` ADR with `enforced_by` once slices 1–3 are real. Proposed check ids, for `check-registry.yaml`
under ADR-0014's `declared` state: `db.tenant_row_declares_a_live_region`,
`db.region_is_immutable_after_placement`, `domain.placement_is_resolved_never_defaulted`,
`domain.privileged_session_is_logged_and_bounded`, `domain.attestation_claims_no_framework`.
The per-tenant restore, export and quota work is an RFC that finishes when it is built, and stays
an RFC.

## Open questions

1. **What is the C1 ceiling, as a number?** §Proposal B's whole defence is that a contract-bounded N
   is not REQ-0008's O(tenants) N. That defence is only honest with a stated number at which the
   fan-out is re-examined — and it should be stated before the first C1 tenant, not after. A
   candidate is "the migration fan-out must complete inside one deployment window", which makes the
   ceiling a measurement rather than an opinion.

2. ~~**Does the ladder ever get published?**~~ **ANSWERED 2026-09-15 by ADR-0103.** A rung's
   existence and name may be published with no price and no self-serve path; provisioning stays
   gated on a signed contract and a C-0004 human approval, which keeps ADR-0024 decision 8 intact.
   ADR-0103 §Status records the reading this rests on and the condition under which it is void.
   Containment is an attribute an entitlement grants, never a property of a plan name.

3. **What is the actual target RPO, and does the hypervisor constraint permit it?** ADR-0064 is
   explicit that `sitar` is the failure domain and that nothing arranged inside the cluster changes
   that. An RPO better than the off-host copy's cadence may be unachievable until there is a second
   hypervisor, in which case §Proposal E's attestation must say a number that is worse than we would
   like — which is the correct outcome and worth deciding before the field is designed.

4. ~~**BYOK at C2 — is it a real capability or a statement of intent?**~~ **ANSWERED 2026-09-15.**
   In scope, designed in **RFC-0046**, and **build-gated** on explicit human direction. Until it is
   built and rehearsed, ADR-0103 decision 7 and ADR-0041 decision 4 forbid any artifact listing it as
   available. RFC-0046 §F is the rehearsal that gates naming it.

5. **Does `audit_events` belong in the same database as the data it audits?** A shared database means
   an audit stream inside the blast radius it exists to record. Separating it is more correct and
   contradicts §Proposal A's "one placement per tenant" simplicity.

6. **Who operates a C3 stamp?** ADR-0064 is honest that one hypervisor is the shape; a dedicated
   stamp presumes a second failure domain that does not exist. C3 may be unreachable until the
   hardware question is answered, and the ladder should probably say so rather than list a rung
   nothing can currently deliver.

7. **Does the attestation need to be independently verifiable?** Signed by us is a start; a
   third-party-anchored or transparency-log-style artifact is stronger and considerably more work. A
   tenant asking "why should we believe your signature about your own isolation" is a fair question
   with no answer in this draft.

---

## Appendix A — the new-tenant journey, end to end

Normative for §Proposal's surfaces; written here because the journey is what makes the design
concrete, and because most of the gaps in §Problem are invisible until the journey is traced.
Operator-facing screens live at `<platform-zone>/dashboard` per ADR-0100 and nowhere else.

### A.1 The happy path

| # | Step | Owner | Notes |
|---|---|---|---|
| 1 | **Intake** — legal name, jurisdiction of establishment, contact, intended data classes | `/dashboard` | Data classes drive ANTI-0007's gate at step 3; asking later means asking after placement |
| 2 | **Placement decision** — region chosen explicitly; `suggest_for` orders the list and never picks | operator + customer | ADR-0041 D3: contractual. RFC-0015 D5: the operator chooses |
| 3 | **Containment decision** — C0 unless a contract says otherwise | operator | ANTI-0007: a regulated class forces C1+ **or refuses the tenant**. A C1+ choice is a C-0004 human decision recorded against the tenant |
| 4 | **`POST /tenants`** → `202`, `requested` → `provisioning` | `platform-tenancy` | ADR-0039. `/dashboard` is a BFF and writes nothing itself (ADR-0100 D1) |
| 5 | **Tenant row** with `region`, `containment_class`, `contract_ref` | saga step 1 | Id is `uuidv5(ns, org_id)` — derived, not minted (ADR-0082) |
| 6 | **Zitadel org** in the regional instance | saga step 2 | ADR-0047: one org per tenant |
| 7 | **Placement row + credentials** | saga step 1b *(new)* | C0 points at the shared pool; C1+ provisions a database and a secret |
| 8 | **Branding and policy** | saga step 3 | ADR-0084 |
| 9 | **Entitlements** — catalogue grants, plus quota ceilings for the class | saga step 4 | §Proposal G |
| 10 | **Custom domain + certificate** *(premium)* | saga step 5 | The step that fails in ways retry does not fix (ADR-0039 step 5) |
| 11 | **Billing customer** | saga step 6 | Deactivated, never deleted, on compensation |
| 12 | **First attestation generated** *(new)* | §Proposal E | A tenant that has never been attested is not `active` |
| 13 | **`active`**, first operator invited | saga step 7 | |

### A.2 The cases that must be handled, and what each one does

Grouped by where they bite. Every one of these is a case the current estate has no answer for.

**Placement and residency**

| Case | Behaviour |
|---|---|
| No region named | **Refuse.** Never default (§Proposal C.4) |
| Region declared in the registry but not `status: live` | **Refuse.** A jurisdiction is not a region |
| Tenant asks for a region that would split its data | **Refuse.** One placement per tenant; §Open question 5 is the only exception under discussion |
| Tenant later requires a different jurisdiction | Contract amendment + migration + new attestation. Never an `UPDATE` (§Proposal C.2) |
| Tenant's users are in a country with no region | Placement is legal, not latency. `suggest_for` orders; the customer chooses |
| Does the plan decide the region? | **No.** Region is orthogonal to containment and to plan — every tenant chooses, at every class (ADR-0103 D4) |
| Staff in region X need access to a tenant in region Y | Permitted, **recorded with `actor_region` as an explicit field** (§Proposal C.5) |

**Containment and escalation**

| Case | Behaviour |
|---|---|
| Tenant asks "can we have our own database?" | Quotable at C1 against a contract. Not offered in a signup flow (ADR-0024 D8) |
| Tenant asks for BYOK | C2. Honestly: §Open question 4 — not designed yet, and should be said rather than promised |
| Tenant asks for a dedicated region/stamp | C3, and §Open question 6 — may be undeliverable on one hypervisor. Say so |
| Tenant intends to hold PHI / card data | **ANTI-0007 refuses**, regardless of class, until the control framework is declared and real. ADR-0041 phase 2 |
| Tenant outgrows C0 | C0→C1 is an export, a restore into the new database, a placement update, and a re-attestation. Downtime-bounded, rehearsed before it is sold |
| Tenant downgrades C1→C0 | Supported and discouraged; the same machinery reversed. It re-enters a shared failure domain, which the attestation will say |

**Identity and access**

| Case | Behaviour |
|---|---|
| First operator's email already exists in another tenant's org | Fine — orgs are separate; the subject resolves per org (ADR-0047) |
| Tenant wants its own IdP / SSO | Zitadel org-level federation. Does not change placement or containment |
| An operator leaves | Grant revoked, revocation recorded (§Proposal D.3). An access review is a query |
| Tenant asks who at Paridyum can see its data | Answered by §Proposal E's `last_privileged_access` and §Proposal H's log — not by an assurance |

**Failure, exit and edge cases**

| Case | Behaviour |
|---|---|
| Saga fails at any step | Compensate per ADR-0039's table. Never hard-delete; `failed`, not gone |
| Certificate issuance stalls (step 10) | The expected `compensation_failed`; reconciler retries. Tenant is `active` without a custom domain |
| Tenant deletes something important | Per-tenant restore from PITR via the extractor (§Proposal F). At C0 this is a clone-and-extract; at C1 it is a database-level PITR |
| Tenant requests its data (GDPR Art. 20) | Export, ciphertext archive, logged (§Proposal F) |
| Tenant requests erasure (Art. 17) | Extractor deletes, emits a certificate, writes it to an audit stream that outlives the data (§Proposal D.4) |
| Tenant offboards | Export → contractual retention hold → deletion → certificate. The sequence, not just the last step |
| Tenant disputes an invoice / goes dormant | Suspension is an entitlement state, never a placement change and never a deletion |
| Tenant's bulk import degrades others | Quota ceiling refuses at the pool, loudly, before it becomes everyone's incident (§Proposal G) |
| Whole-cluster loss | Restore from the off-host copy; the RPO is the number in the attestation, which is why §Open question 3 must be answered before that field ships |
| Tenant asks for an audit of where its data lives | The attestation *is* the answer, dated and machine-readable (§Proposal E) |
| Paridyum itself | Tenant zero, same journey, same silos, no scope a customer could not hold (ADR-0049 D2) |

### A.3 What the tenant sees, and when

1. **At signup** — the region they chose, in writing, with its jurisdiction and the regime that
   placement addresses. Never a framework claimed as satisfied (ADR-0041 D4).
2. **At activation** — their first attestation. A tenant that has never been attested is not
   `active`; that ordering is deliberate and is what stops the attestation becoming optional.
3. **Continuously** — a console page rendering the current attestation, with `isolation_proven_at`
   and `last_privileged_access` visible without asking anyone.
4. **On request** — export, deletion certificate, audit stream extract, all self-service.
5. **On incident** — which containment class they are in, therefore who else was affected. A C0
   tenant is told it shares a failure domain, because it does, and because the attestation already
   said so before the incident.

## Related

- ADR-0024 — the mechanism, and decision 8's funded trigger this RFC operationalises without publishing
- ADR-0038 — the session contract, unchanged at every rung
- ADR-0039 — the saga whose `region` field this makes real
- ADR-0041 — the five design inputs, four of which this discharges
- ADR-0049 — nothing runs outside a tenant, including restore, export and break-glass
- ADR-0064, ADR-0094, ADR-0071 — the failure domain and the backup posture this RFC's slice 4 confronts
- ADR-0100 — where every operator screen in Appendix A lands
- REQ-0013 — export, built as one machinery with restore
- REQ-0022 — the empty `enforced_by` that §Proposal E fills
- RFC-0006, RFC-0007 — the option comparisons this RFC reuses rather than reopens
- ANTI-0007 — the gate that refuses a regulated tenant at Appendix A step 3
- ADR-0103 — containment as an entitlement attribute; answers open questions 2 and 4's publication half
- RFC-0046 — the BYOK design, build-gated
- C-0006 — *provable* isolation, which is the word this whole RFC is about
