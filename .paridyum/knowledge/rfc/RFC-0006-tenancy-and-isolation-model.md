---
id: RFC-0006
title: How a tenant is isolated, and when that isolation escalates
type: rfc
status: accepted
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
resolved_by: ADR-0023
scope: [org, platform, service, infra]
applies_to: ["*"]
tags: [tenancy, isolation, postgres, rls, kubernetes, density, compliance]
breaking: true
enforced_by: []
---

# RFC-0006 — How a tenant is isolated, and when that isolation escalates

## Status

**ACCEPTED**, resolved by ADR-0023 — **which has since been superseded by ADR-0024. Read
ADR-0024 for the decision in force.**

`resolved_by` still names ADR-0023 and deliberately so: this RFC genuinely was closed by that
decision, and repointing it would erase the record of what was decided when. What changed is the
model, not this comparison. ADR-0024 made tenancy *nested* — Paridyum's tenants are themselves
multi-tenant applications — and RFC-0007 carries that comparison. Everything below still holds
for the OUTER silo, and the arithmetic that eliminated database-per-tenant and virtual clusters
is reused by ADR-0024 unchanged.

## The question

Every business running on Paridyum is a tenant. Nothing yet says what separates one tenant's
data from another's, and no service can be written without that answer — it appears in the
first migration, the first query, the connection pool, the deployment topology and the
pricing model.

The decision is genuinely open. ADR-0003 and ADR-0020 fix Kubernetes as the deployment target
but say nothing about tenancy, and neither does REQ-0008, which governs how schemas change
rather than what they contain.

Two constraints arrive from outside this question and narrow it before any option is
compared:

- **BET-0001** puts the customer at a low price point in large numbers, most of them idle.
  Per-tenant infrastructure is unaffordable at that ACV — cost 3 of that bet spells out why.
- **BET-0002** charges a flat price per business. Density is therefore margin, directly. Every
  improvement is ours to keep and every regression is ours to absorb.

Together they mean the answer is not chosen on isolation strength alone. It is chosen on
isolation strength *per unit cost at rest*, which is a different question and eliminates
options that would otherwise be obvious.

## Options

### A. One database, one schema, row-level security, `tenant_id` on every tenant-scoped table

Postgres `ROW LEVEL SECURITY` with policies keyed to a session variable set per request.
Every tenant-scoped table carries `tenant_id`.

- **Cost at rest per tenant:** a set of rows. Effectively zero.
- **Isolation:** enforced by the database, not the application, provided RLS is both `ENABLE`d
  and `FORCE`d and the session variable cannot be influenced by the client.
- **Failure mode:** one forgotten `tenant_id`, or one query run as a role that bypasses RLS,
  is a cross-tenant leak. Permanent, silent, and invisible to any test that queries as a
  single tenant.
- **Noisy neighbours:** shared connection pool, shared buffer cache, shared vacuum. One heavy
  tenant degrades everyone.
- **Per-tenant operations:** restoring one tenant to a point in time means extracting rows
  from a full-cluster backup. Unpleasant, and it will be needed.

### B. One database, one schema per tenant

- **Cost at rest:** small but not zero — catalogue bloat, and connection-pool pressure because
  pools are per-search-path in practice.
- **Isolation:** stronger by accident than by design. A missing qualification hits the wrong
  schema rather than the wrong tenant's rows, which is a better failure but still a failure.
- **Breaks at scale:** thousands of schemas make migrations O(tenants). REQ-0008 requires
  migrations be deployable without downtime; ten thousand sequential `ALTER TABLE`s are not.
- Per-tenant restore is genuinely easier, which is its real advantage.

### C. One database per tenant

- **Isolation:** strong and easy to explain, which has real commercial value.
- **Cost at rest:** a running Postgres per tenant, or a managed instance per tenant. At
  BET-0001's price point this is negative margin on the first tenant and does not improve.
- Migration fan-out worse than B.

### D. Virtual clusters per tenant (vCluster or equivalent)

Each tenant gets an apparent Kubernetes API server inside the host cluster.

- **Isolation:** hard, including the control plane. This is what a regulated customer will ask
  for by name.
- **Cost:** a control plane per tenant, plus the operational burden of N clusters. This is a
  platform team's steady state, not a pre-v0.1 starting point.
- **Relevant, but not now.** It is the correct answer to a question nobody has asked yet.

### E. Hybrid — A by default, escalating to C or D on a stated trigger

The shape most platforms converge on. The question is not whether to end up here but **what
is built first**, since building the escalation path before anyone needs it is how twelve
months disappear into a control plane with no customers.

## Discussion

**Options C and D are eliminated by arithmetic, not by preference.** At a flat low monthly
price with most tenants idle, a per-tenant database or control plane is negative gross margin
per tenant from the first one. No volume fixes it. They are re-entered only when a customer's
contract pays for them, and at that point they are funded rather than speculative.

**B is the interesting near-miss.** It buys a better failure mode and much easier per-tenant
restore, for a migration cost that grows linearly with success. REQ-0008's no-downtime
requirement is the deciding constraint: a migration that is O(tenants) is a migration that
eventually cannot be deployed at all, and that ceiling arrives precisely when the company is
working.

**A's danger is real and is not the same class as the others.** The other options fail loudly.
A fails silently: a forgotten `tenant_id`, or a query issued by a role that owns the table and
therefore bypasses non-forced RLS, leaks data with no error anywhere. It cannot be caught by
tests that act as one tenant, and once real rows have crossed the boundary it is unrecoverable
— the same shape as REQ-0003's observation that personal data in year-old backups cannot be
retroactively un-logged.

That argues for a check rather than for a different option, because the checks are
mechanically decidable and cheap:

- every `CREATE TABLE` either declares `tenant_id` or is on an explicit platform-owned
  allowlist;
- RLS is both `ENABLE`d **and** `FORCE`d, verified against the live catalogue rather than the
  migration text;
- the tenant identifier reaching a query originates from the verified session, never from
  client input.

`ENABLE` without `FORCE` deserves emphasis, because it is the trap: it exempts the table
owner, migrations usually run as the owner, and the result is a table with RLS enabled,
policies defined, and no isolation whatsoever for the role the application connects as. Every
dashboard reports healthy.

**On per-tenant restore.** A's weakest point, and it is a real operational gap rather than a
theoretical one — a customer will delete something important. Row-level extraction from a
cluster backup is the answer, and REQ-0013's export path is the same machinery pointed the
other way, which is an argument for building that earlier than a pure compliance reading
would suggest.

## Recommendation

**Option A now, option E's escalation path deferred behind a revenue trigger rather than
offered as a tier.** Publishing an isolation tier is a commitment to operate it; deferring it
behind a signed contract means the first customer who needs it also funds it. C-0006 anti-goal
5 already forbids selling to enterprises before self-serve works, which is the same boundary
seen from the commercial side.

Resolved by ADR-0023.

## Related

- ADR-0023 — the decision
- BET-0001, BET-0002 — the two constraints that eliminated C and D
- REQ-0008 — the no-downtime migration requirement that eliminated B
- REQ-0003 — the unrecoverability argument, in its original form
- REQ-0013 — export, which shares machinery with per-tenant restore
- ANTI-0007 — why regulated data does not get a shared-database exception
