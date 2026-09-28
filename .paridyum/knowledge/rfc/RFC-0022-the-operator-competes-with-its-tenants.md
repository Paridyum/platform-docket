---
id: RFC-0022
title: What must be true for a founder to build on a platform whose operator competes with them
type: rfc
status: accepted
created: 2026-08-15
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service]
applies_to: ["*"]
tags: [conflict-of-interest, isolation, data-use, trust, self-preferencing, export, audit]
breaking: true
enforced_by: []
sources:
  - url: https://cloudnative-pg.io/documentation/1.22/logging/
    title: CloudNativePG — Logging (PGAudit support)
    retrieved: 2026-09-27
    supports: >
      §D5a path 2. PGAudit is enabled by adding `pgaudit.*` parameters to the cluster's
      `postgresql` section; the operator manages `shared_preload_libraries` and the
      CREATE/DROP EXTENSION in every database, and the audit records are routed to stdout
      as JSON under `.record.audit`. This is what makes closing the direct-psql path a
      configuration change rather than a project.
  - url: https://github.com/pgaudit/pgaudit
    title: pgaudit — README
    retrieved: 2026-09-27
    supports: >
      §D5a paths 2 and 3. The statement classes are READ, WRITE, FUNCTION, ROLE, DDL, MISC,
      MISC_SET and ALL, combined as a comma-separated list with `-` to exclude. And the
      property that makes this a control rather than a log: "Settings may be modified only by
      a superuser. Allowing normal users to change their settings would defeat the point of an
      audit log." So the audited application role cannot switch off its own auditing.
  - url: https://docs.siderolabs.com/talos/v1.13/configure-your-talos-cluster/system-configuration/patching
    title: Talos Linux — Configuration Patches
    retrieved: 2026-09-27
    supports: >
      §D5a's erratum. `cluster.apiServer.auditPolicy` exists in the Talos machine config and is
      REPLACED on merge when patched rather than merged — so a patch must carry a complete policy,
      not an addition to the default one.
  - url: https://garagehq.deuxfleurs.fr/documentation/reference-manual/features/
    title: Garage — List of features
    retrieved: 2026-09-27
    supports: >
      §D5a path 4. The feature list documents Prometheus metrics and OpenTelemetry traces
      and does NOT document S3 server access logging or a per-object audit trail. Stated as
      "not documented" rather than "not possible": absence from a feature page is not proof
      of absence from the product.
---

# RFC-0022 — The operator competes with its tenants

> **Superseded in part, 2026-09-28 — ADR-0113, ADR-0116, ADR-0120.** In this RFC **"the operator" is Paridyum** —
> the platform — not GLO-0001's operator. D5's tenant-visible access is built as ADR-0120's grants,
> and staff act in ADR-0116's platform context, which the database confines to the personless
> skeleton.

## Status

**ACCEPTED** as of 2026-09-27 by the operator. Proposed 2026-08-15, opened by BET-0005, which commits
Paridyum to operating the same catalogue products it franchises.

**Accepted with D5 labelled PARTIAL, which the RFC's own open question 2 demanded.** That question asked
whether every access path can be logged or only application-mediated ones, and said *"a partial one must
be described as partial."* It is now answered from measurement rather than estimate — §D5a is the answer
— and D5 is **partial today**: the application path is logged and five others are not. Publishing D5 as
an unqualified guarantee would have been claiming a capability before it is real, which ADR-0041 D4
forbids.

**What acceptance releases:** D1 through D4 become binding rather than proposed, and D5 becomes
publishable **in its partial form** with §D5a as the statement of what is and is not covered. What it
does not release is any page saying "we log every access", because that is not true and §D5a says why.

D1 is already in force (ADR-0049 D2, and tenant zero holds no grant a customer could not hold). D4's
export exists as of 2026-09-26 (`platform-tenancy/scripts/extract.py`, RFC-0045 §F).

**Four boundary options were considered and rejected the same day**, all for one reason: they are
physical-franchise mechanics that do not transfer to a digital business where **tenants run under
their own brand**.

| Rejected | Why |
| --- | --- |
| Vertical exclusivity — never franchise what we operate | Contradicts BET-0005 outright; nothing proven could ever be franchised |
| Territorial exclusivity | Solves scarcity of location. There is no location, and software territories leak |
| Sunset / hand-over — operate, then exit | Rejected by decision: Paridyum does not intend to exit its own operations |
| Retail price handicap — always price above franchisees | Wrong axis (the conflict is not price), unenforceable across many tenants at many prices, arguably worse than neutral since premium positioning is an advantage, and it edges toward a vertical-restraint question that would need CCI research before it could be written |

**One bound was accepted**, and it is the only one: **nothing tenant-derived informs where
Paridyum competes.** C-0006 non-commodity item 4 was narrowed to say so on the same day.

**White-label removes an exposure that would otherwise dominate this RFC.** `paridyum.com` sells
to prospective *tenants*; a tenant's end customers arrive through the tenant's own domain and need
never learn Paridyum exists. There is no shared shelf, so the platform-ranking self-preferencing
problem that defines app stores and marketplaces **does not arise here**. That is worth stating
because it is the first thing a reader familiar with the Amazon or Apple cases will look for.

## The question

**A founder is deciding whether to run their business on infrastructure operated by a company
that runs the same business. What has to be true for that to be a rational decision?**

This is not a reputational question to be managed with messaging. BET-0005 cost 1 names the
failure — adverse selection on exactly the founders worth having — and with territory,
distribution and exit all ruled out, **the technical guarantee and the data-use commitment are the
entire defence.** This RFC is where that defence is designed, and it must exist before the first
franchised tenant, because a commitment made after the fact is worth a fraction of one made
before.

**The conflict is informational, and once the franchise machinery is stripped away that is all it
is.** Paridyum sees every tenant's customer count, revenue, churn, pricing, feature usage and
growth curve. Tenants see none of ours, and their end customers do not know we exist. We then
compete in the same markets holding that view. Exposure 6 below is therefore not one item on a
list — it is the main event, and the rest of this RFC exists to make it credible that we do not
act on it.

## Why this is urgent precisely because nothing exists

The same argument RFC-0013 makes about intellectual property applies here. The commitments below
are cheap while there are no tenants and no operating history, and they are expensive or
impossible later:

- A data-use commitment made before any tenant data exists is credible. The same words published
  after three years of unrestricted access are a policy change nobody believes.
- An access log that has run since the first row is evidence. One switched on after a dispute is
  an admission.
- Building Paridyum's own tenant through the same code path as everyone else's costs nothing on
  day one and is a migration later.

## What is already decided elsewhere

| Already decided | Where |
| --- | --- |
| Paridyum operates what it franchises; no commercial boundary | BET-0005 |
| Two nested silos, `tenant_id` + `account_id`, RLS forced | ADR-0024 |
| Operator scope is **a grant, never an absence** | GLO-0005 |
| Switching cost is not the retention strategy; export is a sales asset | C-0006 anti-goal 4 |
| Export exists at both levels | REQ-0013 |
| Provable isolation is ours to build; *provable* is the product | C-0006 non-commodity 2 |
| The denial is the assertion | STD-0009 |
| An agent's authority is scoped, its model pinned, its actions auditable | REQ-0007 |

## The threat model, stated as a tenant would state it

Six distinct exposures. Only the first is the one people think of, and it is not the worst.

1. **Direct read of tenant data.** ADR-0024 puts every tenant in one database behind forced RLS.
   An operator with database credentials can bypass RLS. This is the obvious exposure and the
   most tractable.

2. **Aggregate and derived data — and this one contradicts a live artifact.** C-0006's
   non-commodity list, item 4, states as an *asset*:

   > **Aggregate operational data.** What a business looks like when it is working, across
   > thousands of them. Benchmarks, defaults, and eventually models. Available to nobody who does
   > not host the runtime.

   Written under BET-0001 and BET-0003, that was aggregation across customers who were not
   competitors. **Under BET-0005 it is competitive intelligence gathered from businesses Paridyum
   competes with.**

   **Resolved 2026-08-15** — item 4 was narrowed rather than kept or dropped: aggregate data may
   improve the runtime and set product defaults, and **may never inform where Paridyum competes.**
   D2 carries the mechanism. This was the sharpest unresolved conflict in the strategy and it is
   now a decision rather than a caveat.

3. **Roadmap self-preferencing.** A catalogue feature that reaches Paridyum's own tenant first
   gives it a head start in a market it shares with franchisees. Invisible from outside and easy
   to do accidentally, since Paridyum's tenant is the natural place to test.

4. **Pricing self-preferencing.** Paridyum's own tenant pays neither floor nor royalty (BET-0007).
   Its unit economics are therefore structurally better than any franchisee's in the same
   vertical, which is a permanent competitive advantage rather than an accounting artefact.

5. **Support and operational priority.** An incident affecting Paridyum's own revenue and one
   affecting a franchisee's compete for the same on-call engineer.

6. **Catalogue selection informed by tenant performance.** Paridyum can observe which franchised
   verticals perform, then enter them. With no boundary (BET-0005), nothing prevents this — and
   it is the exposure most likely to be discovered by a franchisee after the fact and experienced
   as betrayal.

## Proposal

### D1 — Paridyum's own tenant is a tenant, with no privileges

It is provisioned through the same saga, stored in the same silos, reached through the same code
path, and granted no operator scope. GLO-0005 already makes scope a grant; this says the grant is
never made to Paridyum's operating tenant.

**This is testable, and STD-0009 already specifies how**: the denial is the assertion. A test that
Paridyum's own tenant cannot read another tenant's rows is the single most valuable test in the
system, because it is the one a prospective franchisee would ask for.

### D2 — A published data-use commitment, written before there is data to use

Naming what Paridyum may and may not derive from tenant data. The proposed shape:

- **No per-tenant data leaves the tenant boundary** for any purpose other than operating the
  platform for that tenant.
- **Aggregate statistics only above a stated k-anonymity threshold**, and never in a vertical
  where Paridyum operates a competing tenant.
- **Tenant performance may never inform where Paridyum competes** — not which vertical to enter,
  not which segment to target, not when. This is exposure 6, which is the main event.

**C-0006 non-commodity item 4 was narrowed to match on 2026-08-15**, so the artifacts now agree.
Aggregate data may improve the runtime and set product defaults; it may not select a market.

### D2a — the part of D2 that cannot be checked, made evidentiary instead

**A decision about which market to enter is not a code path.** No check sees it, and pretending
otherwise would be the false confidence ADR-0002 exists to prevent. Data *access* is testable
(D1, D5); *acting on what you know* is not.

The proposal: **every vertical Paridyum enters is recorded in an artifact naming the evidence that
selected it, and that evidence may not be tenant-derived.** A check verifies the artifact exists
and that its cited sources are external — public market data, Paridyum's own operating experience,
customer conversations outside the tenant base. It cannot verify that the *real* reason matches
the stated one; it can make an unstated reason conspicuous, and it makes the commitment auditable
by the counterparty rather than only by us.

This is prose-with-frontmatter that a gate can read, which is this repository's founding argument
applied to the one commitment that resists direct testing. **Its limit must be stated wherever it
is claimed:** it raises the cost of quietly breaking the promise; it does not make breaking it
impossible.

### D3 — Parity commitments on roadmap and price

- **No catalogue capability reaches Paridyum's own tenant before franchised tenants**, beyond a
  stated and bounded canary window.
- **Paridyum's own tenant is billed internally at the same floor and royalty as any franchisee.**
  The money is circular, but the accounting is not: it keeps unit economics comparable, and it is
  what makes BET-0005 trigger clause 3 measurable rather than arguable.

### D4 — Export moves from a deferred requirement to a launch commitment

C-0006 anti-goal 4 already says a tested export path is a sales asset rather than a concession.
Under this RFC it becomes the guarantee of last resort: **the credible ability to leave is what
makes staying rational when the landlord competes with you.** REQ-0013 is `deferred` on the
trigger *"the first tenant-authored row"* — which now fires at the first franchised tenant, and
should be treated as firing then rather than discovered later.

### D5 — A tenant-visible operator access log

If a Paridyum operator accesses a tenant's data, **that tenant can see it** — not on request, by
default. This is the strongest single trust mechanism available and the cheapest to build early,
since `platform-audit` already exists as a planned context and RFC-0009 open question 2 expects
its trigger to fire during the first wave anyway.

Self-attestation is worth very little. A log the counterparty reads is worth a great deal.

### D5a — the part of D5 that is not covered, measured rather than estimated

**Added 2026-09-27, answering open question 2.** D5 says a tenant sees operator access to its data. That
is a promise about access paths, so it is only as strong as the weakest path — and there are six in the
table below, a seventh that is not ours, and an eighth opened on the same day (see its note).
Measured against this estate as it stands:

| # | Path to a tenant's data | Logged where the tenant can see it | Closable how |
| --- | --- | --- | --- |
| 1 | **The application** — `platform-tenancy`, `tool-docket`, the web surfaces | **Yes.** `audit_events`, one row per operation (ADR-0110), readable by the tenant's own administrator | — |
| 2 | **`psql` as the superuser inside the pod** — `migrate.py`, `retention.py`, the seeds, the isolation proofs, any ad-hoc `kubectl exec` | **No.** These bypass the application entirely | **PGAudit.** CloudNativePG enables it by adding `pgaudit.*` parameters and manages the library and the extension itself; records reach stdout as JSON under `.record.audit` |
| 3 | **`paridyum_app`'s DSN used from outside the application** | **No.** Same bypass, with RLS still in force | PGAudit, as path 2 |
| 4 | **Backups** — the WAL archive and base backups in Garage | **No.** Whoever can read the bucket can read every tenant's rows | Partially. Garage documents metrics and traces, not S3 access logging; a logging proxy in front of it, or credential issuance that is itself recorded |
| 5 | **A restore clone** — `postgres/restore-rehearsal.yaml` | **No.** A clone is a full copy of every tenant's data | Policy: a clone is a database and takes the same `pgaudit.*` parameters |
| 6 | **Raw volume access** — the Longhorn volume, or the node's disk | **No**, and no log can cover it | Not closable by logging. Encryption changes the exposure and not the operator's position: the running database holds the key |

**Path 7 is not ours.** Zitadel holds the authentication events, and the correlation id is the join
(ADR-0044 D1). A tenant seeing both halves needs that id present in each, which is why it was built
before this.

> ### Path 8, added the same day — opened by the scoping that makes paths 2, 3 and 5 work
>
> Before pgaudit was switched on, the question nobody had asked: `pgaudit.log` applies to **every
> role**, Zitadel's service role included, on the same cluster. Measured over 60 s of catalog
> statistics: the `zitadel` database at ~8 transactions/s and ~2.5 rows written/s at idle, `paridyum`
> at ~0. With `log_relation` on, auditing Zitadel's own polling is hundreds of thousands to millions of
> lines a day into a container log kubelet rotates by size — so a path-2 `psql` session would be
> rotated away within minutes, and the control would have defeated itself while looking healthy.
>
> So `infra-kubernetes` `make postgres-audit-scope` sets **Zitadel's own role, in its own database**,
> to `ddl, role`. Every other role — the superuser included — keeps `read, write, ddl, role`, so a human
> reading or changing the `zitadel` database is recorded **unless that human turns it off first**:
> pgaudit's settings are superuser-only, which binds `paridyum_app` and `zitadel` and not the
> superuser. A superuser can `SET pgaudit.log = 'none'`, and that `SET` is not itself recorded
> (`misc_set` is excluded, for the noise reason in `cluster.yaml`). So path 2 is recorded against a
> careless operator, not a deliberate one — the same position as path 6, one level up. What still
> records the deliberate case is the kube-apiserver's audit log (Talos default, `level: Metadata`,
> the erratum below): the `kubectl exec` into the database pod, who and when, though not what was read.
>
> **That opens path 8:** a holder of the Zitadel database credential can read and change identity
> data in the `zitadel` database unaudited while connected as that role. Identity data is a tenant's
> users' personal data, so this is a path to a tenant's data, not an internal one. It is a
> secret-access path — the credential lives in a SOPS-encrypted Kubernetes secret — and it is closable
> the same way path 6's neighbours are not: **by keeping the records long enough that the exemption is
> unnecessary.** The root cause is that audit records live only in a size-rotated container log
> (`infra-kubernetes` DECISION-DEBT: raise kubelet's log limits in the Talos config, ship
> `.record.audit` off the node — the "reach" half of T-G.22). When that lands, the exemption should go.

> ### ✘ ERRATUM, same day — the paragraph below was wrong, and its "verified" was worthless
>
> It read: *"the kube-apiserver has **no audit policy configured** in this estate — verified 2026-09-27,
> nothing in `infra-terraform` or `infra-kubernetes` sets `--audit-log-*` or an audit policy file."*
>
> **The verification was a grep that filtered out its own evidence.** It ended
> `| grep -v '.paridyum'`, and `.` matches any character — so every path under
> `/Users/rohan/Developer/paridyum/` was excluded, which is every path there is. The command returned
> nothing and the nothing was read as an answer.
>
> **The truth is better than the claim.** Talos v1.13.7 ships kube-apiserver audit logging **on by
> default**: `cluster.apiServer.auditPolicy` with `rules: [- level: Metadata]`, present in
> `infra-terraform/talos/rendered/controlplane.yaml`, writing to `/var/log/audit/kube/*.log` on the
> control-plane nodes. So **`kubectl exec` into the database pod IS recorded** — who, which pod, when.
>
> What it is not is **shipped** or **visible to a tenant**. So this does not close paths 2 or 5; it
> changes what is missing from them, from *"nobody knows it happened"* to *"the operator's own node knows,
> and rotates it in 30 days"*. Recorded as an erratum rather than an edit (ADR-0031) because the mistake
> is the useful part: it is the fourth time in two days that a check reported nothing because the check
> was wrong, and the first where the author of the check was the one relying on it.
>
> One consequence worth carrying: a Talos patch **replaces** `auditPolicy` rather than merging into it, so
> narrowing or widening the policy means writing a complete one, and a partial patch silently discards
> Talos's default.

**What remains on the apiserver path, then, is not recording but reach.** The records exist on the node.
Nothing ships them, nothing makes them tamper-evident, and no tenant can read them — which is D5's actual
bar, and the reason this path stays in the table as uncovered rather than being ticked.

**The honest conclusion, and it is not "we will close them all".** Paths 2, 3 and 5 are closable by
configuration — **and were configured on 2026-09-27**: `pgaudit.*` parameters on the cluster and on the
restore clone (`infra-kubernetes/postgres/`), pending an apply that restarts the database. Path 4 is
partially closable.

**Path 6 is not closable *by logging*, and the stronger claim this section made first — "not closable at
all" — was overconfident.** What there is evidence for is narrower: no log can cover raw volume access,
because the party being logged controls the logger. Whether the *exposure* can be closed by other means is
a question this RFC has not researched, and three avenues are named here precisely so the conclusion is
revisited rather than inherited:

1. **Memory-encrypted execution with attestation.** If the database runs where the host cannot read its
   memory, and a tenant can verify that from outside, raw disk access stops yielding plaintext and the
   operator's position changes rather than being described differently. Unresearched here: what it costs,
   what it requires of the substrate, and whether the attestation is something a tenant can actually
   check rather than be told about.
2. **Keys the tenant holds rather than us.** RFC-0046's territory. It does not help while the running
   database holds the key in memory — which is exactly why it pairs with (1) rather than substituting
   for it.
3. **Tamper-evident shipping, which changes detectability rather than access.** The estate already has
   this pattern: docket's four sealed chains are hash-chained and append-only. Applied to an access log
   shipped somewhere the operator cannot rewrite, concealment becomes detectable even where access stays
   possible — which is a weaker guarantee than prevention and a much stronger one than a promise.

**So this section's verdict is provisional and dated.** It is what could be established on 2026-09-27 with
the estate as it stands, and the trigger to reopen it is the first of: a tenant asking for more than
partial, a substrate change that makes (1) available, or RFC-0046 moving from designed to built. An
architecture conclusion that nobody is expected to revisit is the shape of thing ADR-0011 exists to
prevent.

So the strongest statement this estate can truthfully make to a tenant is not *"we log every access"*.
It is:

> These paths are logged and you can read the log. These are gated by a procedure that records its own
> use and requires a human approval. This one remains, and it remains because we run the hardware —
> here is what that means.

That is a weaker sentence and a more credible one, and it is the same argument D2a already makes about
the commitments that cannot be checked: **an unfalsifiable guarantee is worth less than a bounded one**.
A tenant sophisticated enough to ask about self-preferencing is sophisticated enough to notice which
kind they were handed.

**What this does not permit.** No page, capability matrix or sales conversation may describe the access
log as complete until paths 2, 3 and 5 are closed, path 4 is stated, and path 8 is closed or stated. ADR-0041 D4 forbids claiming a
capability before it is real, and this is the capability most likely to be over-claimed, because the
partial version demonstrates well.

### D6 — The separate legal entity stays deferred, with a trigger

BET-0005 rejected it as premature and named the reconsideration point. Restated here so it is
attached to the conflict rather than only to the bet: **reconsider when BET-0005 trigger clause 2
approaches** — one in four declining founders citing the competing operation.

## Alternatives considered

**Draw a commercial boundary after all.** The standard franchise answer, and it dissolves most of
this RFC. Ruled out by BET-0005 on 2026-08-15. Recorded here because if that decision is revisited,
D2 through D5 shrink considerably, and this is the cheapest available fix to exposure 6.

**Say nothing and rely on isolation being good.** The default path, and the reason it fails is that
a guarantee nobody has published is a guarantee nobody can rely on. C-0006 already makes this
argument about `locked`: an unenforced promise is a suggestion.

**Contractual commitment only, no technical mechanism.** Cheaper and faster. Rejected on the same
ground ANTI-0007 rejects a disclaimer: terms allocate liability between parties, they do not
prevent the access, and they are not what a technically sophisticated founder is asking about.

**Technical mechanism only, no published commitment.** Insufficient in the other direction. D1
prevents the access that goes through the application; it does nothing about exposures 2, 3, 4 and
6, which are all decisions rather than reads.

## Open questions

1. ~~**Does C-0006 non-commodity item 4 survive?**~~ **Answered 2026-08-15: narrowed.** Aggregate
   data may improve the runtime and set product defaults; it may never inform where Paridyum
   competes. A real asset given up, and given up because the alternative is a platform whose
   customers are rationally afraid of it. Kept in this list, struck through, because the reasoning
   matters more than the outcome and a deleted question reads as one that was never asked.
2. ~~**Is a tenant-visible operator access log achievable under ADR-0024?**~~ **Answered 2026-09-27:
   partial, and §D5a is the answer.** One of the six paths in its table is logged today — the application's, through
   `audit_events`. Three more are closable by configuration (PGAudit, which CloudNativePG supports
   natively), one partially (the backup bucket), and **one not at all**: raw volume access, because the
   party being logged controls the logger. So D5 is published as partial, with the paths named. Kept in
   this list, struck through, because the shape of the answer matters more than the verdict — the
   question was whether to describe it as a guarantee, and the answer is no.
3. **Does the commitment need to be contractual to be worth anything?** A published policy can be
   changed unilaterally. A term in the tenant agreement cannot. The second is much more credible
   and much harder to reverse, which is the point.
4. **Who audits it?** Self-attestation against one's own commitment is close to worthless. A third
   party costs money that does not exist yet. REQ-0012 requires vendor controls to *produce
   evidence*; refusing our own tenants what we demand of vendors is C-0006 anti-goal 4's
   incoherence argument in a new place.
5. **Does D3's roadmap parity survive contact with reality?** Paridyum's own tenant is the natural
   place to test a change before exposing franchisees to it. That is good engineering and a head
   start at the same time, and the canary window is where the tension lives.

## Risks

- **R1 — The commitments are made and then quietly eroded.** The failure mode of every neutrality
  promise. D5's log is the only proposal here that resists it, because it is evidence rather than
  assertion.
- **R2 — D1 is implemented and never tested.** The 2026-08-06 audit found checks reporting success
  while enforcing nothing — ask `scripts/check-knowledge.sh` for the current split rather than
  trusting a figure written here. This is the check that most needs to be watched failing.
- **R3 — The conflict is disclosed well and founders decline anyway.** Honest disclosure does not
  make the conflict acceptable to everyone, and BET-0005 trigger clause 2 exists because that is a
  real outcome rather than a communications failure.

## Related

- BET-0005 — the structure that opens this, and the decision that removed the commercial fix
- BET-0006, BET-0007 — the segment that bears the conflict and the royalty it pays
- C-0006 — non-commodity items 2 and 4, anti-goal 4; item 4 is in direct conflict with D2
- ADR-0024, GLO-0004, GLO-0005 — shared database, nested silos, scope as a grant
- STD-0009 — how D1 is tested; the denial is the assertion
- REQ-0013 — export, which D4 promotes to a launch commitment
- REQ-0007, REQ-0012 — scoped authority and evidence-producing controls
- ANTI-0007 — why a disclaimer is not a mechanism
- RFC-0009 — `platform-audit`, which D5 depends on
