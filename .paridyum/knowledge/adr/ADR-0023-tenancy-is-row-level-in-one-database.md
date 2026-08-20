---
id: ADR-0023
title: Tenancy is row-level in one shared database, and isolation escalates only when funded
type: adr
status: superseded
created: 2026-08-07
locked_at: 2026-08-07
supersedes: null
superseded_by: ADR-0024
scope: [org, platform, service, infra]
applies_to: ["*"]
tags: [tenancy, isolation, postgres, rls, density, margin, compliance]
breaking: true
enforced_by:
  - check: db.tenant_scoped_tables_declare_tenancy
    severity: error
    rationale: >
      One forgotten tenant_id in a shared database is a permanent cross-tenant leak that
      no single-tenant test can detect and no later migration can undo.
  - check: db.row_level_security_enabled_on_tenant_tables
    severity: error
    rationale: >
      ENABLE without FORCE exempts the table owner, and migrations run as the owner — so
      isolation can be absent while every dashboard reports healthy.
  - check: db.no_tenant_id_from_client_input
    severity: error
    rationale: >
      RLS is enforced against a session variable. If a client can set that variable, the
      policy is applied correctly to the wrong tenant.
---

# ADR-0023 — Tenancy is row-level in one shared database

## Status

**LOCKED** as of 2026-08-07, and named checks exist for all three of its mechanical claims —
declared, not yet implemented, since `tool-forge` does not exist (ADR-0014's `declared` state).

Locking a decision before any code exists has precedent: ADR-0003 fixed Kubernetes as the
deployment target with zero services. The justification is the same and stronger here. This
decision appears in the first migration ever written, and a tenancy model is the single
hardest thing to change after data exists — every alternative requires migrating live customer
rows across an isolation boundary. Deciding it late does not keep options open; it just means
the first service picks by accident.

## Context

RFC-0006 compared five options. Two constraints eliminated three of them before isolation
strength was considered:

- **BET-0001** places the customer at a low flat price in large numbers, mostly idle.
- **BET-0002** charges per running business, so density is margin directly.

A database or control plane per tenant is negative gross margin on the first tenant at that
price point, and no volume improves it. Schema-per-tenant survives longer and fails on
REQ-0008: migrations become O(tenants), and that ceiling arrives exactly when the company is
succeeding.

## Decision

**One Postgres database, one schema, row-level security. Every tenant-scoped table carries
`tenant_id`. The tenant identifier comes from the verified session and never from client
input. One deployment serves all tenants; no per-tenant infrastructure is provisioned.**

1. **`tenant_id` on every tenant-scoped table.** A table without one must appear on the
   migration's explicit platform-owned allowlist. The default is tenant-scoped; the exception
   is declared and visible in review. An allowlist rather than an inference because the two
   cases are indistinguishable from the schema alone — `plans` is platform-owned and
   `invoices` is tenant-scoped, and nothing about either name says which.

2. **RLS `ENABLE`d and `FORCE`d, verified against the live catalogue.** `ENABLE` alone exempts
   the table owner and migrations run as the owner, which produces a table with RLS enabled,
   policies defined, and no isolation for the role the application connects as. Verified by
   querying `pg_class.relrowsecurity` and `relforcerowsecurity` rather than by parsing
   migrations, because the state that exists matters and the state the files intended does not.

3. **The application connects as a role that cannot bypass RLS.** Not the owner. Migrations
   and runtime use different roles.

4. **The tenant identifier is set from the verified session context only.** Never from a
   request body, query string, path segment or client-supplied header. The session context type
   should be the only thing capable of producing a tenant id, so the type system rejects the
   alternative and the check defends against the type being bypassed.

5. **No per-tenant infrastructure.** No database, schema, namespace, cluster or deployment per
   tenant. A custom domain is a routing entry and a certificate, not a workload.

6. **Escalation is deferred behind a funded trigger, not published as a tier.** Dedicated
   database (RFC-0006 option C) and virtual cluster (option D) are the escalation path. Neither
   is built, offered, or listed until a signed contract pays for it. Publishing an isolation
   tier is a commitment to operate it, and operating it for nobody is the twelve months
   RFC-0006 warns about. C-0006 anti-goal 5 is the same boundary from the commercial side.

## Costs accepted

1. **The failure mode is silent, and it is unrecoverable.** Every rejected option fails
   loudly; this one does not. A missing `tenant_id`, or one query run as a bypassing role,
   leaks data with no error anywhere, cannot be detected by any test that acts as a single
   tenant, and cannot be undone once real rows have crossed — REQ-0003's argument about
   personal data in year-old backups, in a different domain. The three checks are not
   defence-in-depth around a safe design; they are the reason this design is acceptable at
   all, and until `tool-forge` implements them **this ADR is enforced by compliance rather
   than by machinery**, exactly as CLAUDE.md describes for every other lock today.

2. **Noisy neighbours are real and unmitigated.** Shared connection pool, shared buffer cache,
   shared autovacuum. One heavy tenant degrades every other tenant, and the affected customers
   have no visibility into why. Quota and rate limiting become platform correctness features
   rather than billing features — BET-0002 cost 4 reaches the same conclusion from pricing.

3. **Per-tenant point-in-time restore is genuinely hard.** A customer will delete something
   important. Row-level extraction from a cluster-wide backup is the only answer available
   here, and it is the weakest point of the decision. It shares machinery with REQ-0013's
   export path, which is an argument for building that earlier than a compliance-only reading
   would suggest.

4. **"We share a database with other customers" is a sentence we will have to say.** Some
   deals will be lost on it, and the honest response is the checks and the escalation path
   rather than ambiguity about the architecture. C-0006's not-commodity list claims *provable*
   isolation, and a proof that cannot be stated plainly is not one.

5. **The escalation path is unbuilt, so the first customer who needs it waits.** Deliberate.
   The alternative is building it for nobody.

6. **This ADR is locked before a single migration exists**, so its first real contact with
   code may reveal that a rule is impractical. The remedy is supersession, which is expensive
   by design. Accepted because the alternative — deciding tenancy implicitly in the first
   service — is worse and less reversible.

## Alternatives rejected

See RFC-0006 for the full comparison. Briefly: **schema-per-tenant** on REQ-0008's no-downtime
migration requirement; **database-per-tenant** and **virtual clusters** on cost at rest against
BET-0001's price point. All three are re-entered as the escalation path in decision 6, when
funded.

## Consequences

- The first migration written in this organisation must establish the `tenant_id` convention,
  the platform-owned allowlist, and the two-role split. It is not a later refactor.
- Three checks declared: `db.tenant_scoped_tables_declare_tenancy`,
  `db.row_level_security_enabled_on_tenant_tables`, `db.no_tenant_id_from_client_input`. All
  three await `tool-forge`, and per README.md's most useful lesson none should be believed
  until it has been watched failing against a planted defect.
- REQ-0008 gains a tenancy dimension: a migration is now also judged on whether it preserves
  these invariants.
- ANTI-0007 exists because this decision makes a shared database the default, and a shared
  database is not an acceptable home for regulated data at any density.

## Related

- RFC-0006 — the comparison this resolves
- BET-0001, BET-0002 — the constraints that eliminated three options
- REQ-0008 — schema and migration governance, which this constrains
- REQ-0003 — the unrecoverability argument in its original form
- REQ-0013 — export, sharing machinery with per-tenant restore
- ANTI-0007 — the boundary this decision makes necessary
- C-0006 — anti-goal 5, the commercial form of decision 6
- ADR-0003, ADR-0020 — the deployment target this runs on
