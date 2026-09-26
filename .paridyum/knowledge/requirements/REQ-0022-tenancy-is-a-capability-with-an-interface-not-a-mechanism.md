---
id: REQ-0022
title: Tenancy is provable from outside, and a tenant can verify its own isolation without asking us
type: requirement
status: accepted
created: 2026-09-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, infra]
applies_to: ["*"]
product: [tenancy]
tags: [tenancy, isolation, rls, postgres, zitadel, adr-0024, adr-0038, adr-0045, adr-0049, c-0006, c-0008, req-0003, req-0013]
breaking: false
trigger: >
  Already fired. `tenancy` moved from `proposed` to `building` on 2026-09-07 by ADR-0068, and
  a capability at `building` owes a REQ-.
review_date: 2027-03-01
enforced_by: []
---

# REQ-0022 — Tenancy is a capability with an interface, not a mechanism with a column

## Status

**ACCEPTED**, 2026-09-07. Written because ADR-0068 promoted `tenancy` to `building` and the
promotion owes this artifact. `enforced_by` is empty and the emptiness is not the usual backlog
entry — most of the mechanical half is **already implemented** in `check-migrations.sh` and has
been reporting `n/a — no migrations found` since it was written. The checks are named in §5 with
what each one already does, so the gap between this document and the machinery is visible rather
than assumed.

## Context

`product-registry.yaml` recorded the risk before it was taken: tenancy is *"the capability whose
tenant-facing interface matters most and the one most likely to be built as an internal mechanism
first."*

The mechanism is well specified and mostly decided. ADR-0024 fixes two columns and RLS enabled
and forced. ADR-0038 fixes three accessors that raise rather than return `NULL`. ADR-0045 names
the two threats and the single enforcement point each has. None of that is in question here.

**What is missing is the half that makes it a capability rather than plumbing.** C-0008 forbids
building a runtime capability only Paridyum could consume, and ADR-0049 D3 requires a
tenant-facing consumer named before the thing is built. A `tenant_id` column satisfies neither: a
tenant cannot see it, cannot test it, and has no way to tell a working isolation boundary from
one that was never wired up. C-0006 non-commodity item 2 does not claim isolation — it claims
**provable** isolation, and the difference is the entire requirement.

The first consumer of this capability is Paridyum's own hiring store (ADR-0068), which is tenant
zero under ADR-0049 D2. That is the correct first workload and it is also the exact circumstance
in which the interface gets skipped, because tenant zero can always be told the answer out of
band.

## Requirement

**Tenancy shall be verifiable by the tenant, from outside the platform, without privileged
access and without asking an operator — and the platform shall fail closed rather than narrow
when it cannot determine who is asking.**

## Numbered items

1. **Two columns, declared and never inferred.** Every table declares exactly one of
   `platform`, `tenant` or `account` scope. `account_id` never appears without `tenant_id`,
   because account surrogates collide across tenants and a predicate on `account_id` alone
   crosses the outer silo while looking correctly scoped. ADR-0024 decision 1.

2. **RLS `ENABLE`d *and* `FORCE`d, verified against the live catalogue.** `ENABLE` alone exempts
   the table owner, and migrations run as the owner. The state that matters is
   `pg_class.relrowsecurity` and `relforcerowsecurity`, not what the migration files intended.
   ADR-0024 decision 2.

3. **The application role cannot bypass RLS.** Migrations and runtime connect as different
   roles. A single role that both owns the tables and serves requests makes items 1 and 2
   decorative.

4. **Every policy constrains both discriminators on an account-scoped table.** An outer-only
   policy isolates businesses from each other and nobody's customers, and every dashboard
   reports it as healthy. ADR-0024.

5. **Breadth is a positive claim, never an absence.** No policy widens access through
   `IS NULL`, a wildcard, or `coalesce()` on the account discriminator. Absence is what every
   dropped session field, failed lookup and middleware-ordering bug produces, and each of them
   would silently escalate an end user to reading the whole tenant. GLO-0005.

6. **The accessors raise; they never return `NULL`.** `paridyum.current_tenant()`,
   `current_account()` and `has_operator_scope()` raise `insufficient_privilege` on an unset
   setting. `has_operator_scope()` in particular does not default to `false` — a default of
   `false` converts a configuration failure into a silently narrower result set that nobody
   reports. ADR-0038 decisions 1 and 2.

7. **`SET LOCAL`, inside the transaction, from a signature-verified token and nothing else.**
   Connections are pooled and a session-scoped setting outlives the request that set it. A
   pooled connection carrying the previous request's `tenant_id` is a cross-tenant read that
   appears only under concurrency, which is to say only in production. ADR-0038 decisions 3
   and 4.

8. **A tenant can verify its own isolation.** This is the item the other seven exist to make
   true, and it is the one that will be argued as unnecessary. The tenant-facing interface
   shall let a tenant, using only its own credentials, obtain evidence that its data is
   separated: at minimum which tables hold its rows, that RLS is enabled and forced on each,
   and a machine-readable result it can retain. An answer given by an operator in a support
   ticket does not satisfy this, because it proves the operator's cooperation rather than the
   boundary.

9. **The interface is a `spec-` contract before it is an implementation.** ADR-0049's
   mechanical half. A capability whose interface is discovered by reading its server is one
   Paridyum could change without noticing whom it broke.

10. **Paridyum's own tenancy is granted no scope a customer could not also be granted.**
    ADR-0049 D2. Tenant zero uses the same interface, the same accessors and the same policies,
    and any place it does not is a place the interface is untested.

## Acceptance criteria

1. `check-migrations.sh` passes over the real migration set with no errors, and its output
   names a non-zero count of migrations rather than `n/a`.
2. A query issued without session context **raises**, and a test asserts the raise rather than
   asserting an empty result.
3. A test acting as tenant A cannot read tenant B's rows, and a second test acting as account
   A1 cannot read account A2's rows within tenant A. **Two tests, because item 4's failure mode
   passes the first one.**
4. `relrowsecurity` and `relforcerowsecurity` are both true for every non-platform table, read
   from the live catalogue.
5. The tenant-facing verification of item 8 returns a result to a caller holding only tenant
   credentials.

## Out of scope

- **Which database or cluster topology.** ADR-0024 decided it and RFC-0006 records the
  arithmetic. Virtual clusters and dedicated databases are the escalation path behind ADR-0024
  decision 6's funded trigger, re-entered when a contract pays and not before.
- **Per-tenant restore and export.** REQ-0013 owns it. Named here only because ADR-0024 cost 3
  identifies it as this model's weakest point, which is an argument for building it earlier
  than compliance alone would suggest.
- **What personal data may be stored in a tenant's rows.** REQ-0003, and its trigger has not
  fired: nothing stores a natural person yet.

## Progress against the acceptance criteria

Recorded so the gap is legible rather than implied. `platform-tenancy` holds the schema, the
runner and the verifiers; the two proofs run against the live database.

| Criterion | State |
|---|---|
| 1 · `check-migrations.sh` passes over the real set, non-zero count | **MET, 2026-09-12.** `--root platform-tenancy` inspects 4 migrations, 0 warnings. |
| 2 · A query with no session context **raises**, and a test asserts the raise | **MET.** `scripts/prove-isolation.py` proofs 4 and 5 assert `insufficient_privilege` on unset `tenant_id` and unset `operator_scope`. |
| 3 · Two denial tests — tenant A ≠ tenant B, and account A1 ≠ A2 within a tenant | **MET.** `prove-isolation.py` proof 1 (account boundary, both directions) and proof 2 (tenant boundary), exercised through `SET ROLE paridyum_app` with synthetic rows, rolled back. |
| 4 · `relrowsecurity` and `relforcerowsecurity` true for every non-platform table, from the live catalogue | **MET.** `scripts/verify-isolation.py` reads `pg_class`; passed live (3 tenanted tables). |
| 5 · The item-8 tenant-facing verification returns a result to a caller holding only tenant credentials | **OWED — and owed to us.** No tenant-facing interface yet; this is item 8, with item 9 (the `spec-` contract) its prerequisite. The behavioural proof above is operator-run, which cost 1 says does not satisfy item 8. Paridyum is the first tenant (C-0008, ADR-0049), so this is a capability *we* consume, not one waiting on a tenant to appear. |

The numbered items map onto this as: items 1–7 and 10 are discharged by the migrations and the
two verifiers (declared `enforced_by` checks await tool-forge, so they bind by the proofs and by
review); items 8 and 9 are the remaining work. They are tenant-facing, and because Paridyum is the
first tenant that work is for us to consume now — not deferred to a tenant that does not exist.
Cost 1 below argues the same: item 8 is a numbered item *precisely* so it is never perpetually
deferred.

The connection role those proofs run through is decided by **ADR-0079** — a non-owner, RLS-bound
role whose credential is a SOPS-held Automation secret, co-gated into the deployment with ADR-0071.

## Costs accepted

- **Item 8 is real work for a tenant who does not exist**, and it is the first thing that will
  be cut. It is written as a numbered item rather than a nice-to-have precisely because
  C-0008's rule is unenforceable if the interface is always deferred to the release after the
  one being planned.
- **Two tests where teams write one.** Cost 3's second test looks redundant to anyone who has
  not read ADR-0024's account-scoped failure mode, and reviewers will ask why it is there.
- **The accessors raising makes some bugs louder and some pages 500 instead of empty.** That is
  the intent. An empty page is a bug report nobody files.
