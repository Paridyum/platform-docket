---
id: ADR-0044
title: Every request carries a correlation id, and the audit record is append-only and per-tenant
type: adr
status: accepted
created: 2026-08-08
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, library, web, mobile]
applies_to: ["*"]
tags: [audit, observability, correlation, compliance, soc2, gdpr, adr-0041]
breaking: false
enforced_by:
  - check: telemetry.correlation_id_is_propagated
    severity: error
    rationale: >
      A correlation id that is regenerated at a boundary rather than propagated fails exactly
      here and nowhere else: every service works, every log line has an id, and no trace spans
      two of them. It cannot be added retroactively to requests that already happened, which is
      what makes it an error rather than a warning.
  - check: db.timestamps_are_timestamptz
    severity: error
    rationale: >
      An audit record whose instant nobody recorded cannot answer when access occurred, which
      is the first question every framework in ADR-0041 phase 1 asks.
---

# ADR-0044 — Correlation ids and append-only audit

## Status

**ACCEPTED** as of 2026-08-08. Created `accepted` per `new-decision` §5b.

## Context

ADR-0041 put SOC 2, GDPR and DPDP in phase 1 and named five things that become design inputs
immediately *"because none can be retrofitted."* This is the first of them, and it is the only
one that must exist **in the first request the first service handles** — the rest can be added to
a running system.

The specific difficulty is that the audit trail spans two systems that will never share a
database. **Zitadel** holds authentication events — who logged in, from where, with which factor,
which IdP. **Paridyum's own services** hold authorization and data events — who read which
tenant's records, who granted operator scope, who changed a policy.

A regulator's question is never confined to one of them. *"Show me everything that account did on
the 14th"* crosses both, and correlating them after the fact means joining on timestamp and user
id across two clocks — which is guesswork presented as evidence.

There is no version of this that can be added later. A correlation id must be present in the
request that generated the record; a request that has already been served cannot acquire one.

## Decision

**1. Every request carries a correlation id, generated at the outermost boundary and propagated
unchanged.** A W3C `traceparent` header, or `X-Correlation-Id` where a client cannot emit one.
Generated once, at the edge — never regenerated at a service boundary, which is the failure mode
that produces a system where every service works and nothing can be traced across two.

**2. The id appears in every log line, every audit record, and every event.** Not only on error
paths. The record needed during an investigation is the one written when nothing was wrong.

**3. `platform-identity` writes the correlation id into Zitadel where Zitadel accepts it**, and
records the mapping where it does not. This is the join that makes the two audit trails one, and
it is the reason the id must be established before the first login rather than before the first
audit.

**4. The audit record is append-only.** No `UPDATE`, no `DELETE`, enforced by grants rather than
by convention — the application role holds `INSERT` and `SELECT` on audit tables and nothing
else. An audit trail the application can rewrite is not evidence.

**5. Audit records are tenant-scoped and exportable per tenant.** `tenant_id` and, where it
applies, `account_id`, under ADR-0024's rules like any other table. A tenant must be able to take
its audit trail with it — REQ-0013's requirement, and GDPR/DPDP portability applied to the log.

**6. What is recorded, at minimum:** who (subject, tenant, account), what (action, resource),
when (`timestamptz`, UTC), where from (source address, client), the correlation id, and the
outcome — **including denials.** A trail recording only successful access answers "what happened"
and not "what was attempted", and the second is the security question.

**7. Retention is stated per record class before the first record is written.** An audit trail
with no retention policy is either deleted by someone who assumed it was logs, or kept forever
and becomes a liability under the same regulations that required it.

**8. Personal data in audit records is classified at the migration that creates the column**, per
REQ-0003 and STD-0006. An audit trail is a personal-data store; a deletion request reaches it.

## Consequences

**Accepted costs**

1. **Append-only means audit tables only grow**, and the cost is real: partitioning, archival to
   cold storage, and a restore path that can bring a partition back to answer a question about a
   period nobody expected to be asked about.

2. **Correlation propagation is a discipline every service must keep**, and one service that
   regenerates the id breaks tracing for everything downstream of it, silently. The check
   catches it in CI; nothing catches it in a service nobody has written yet.

3. **Recording denials multiplies volume.** A scanner probing endpoints generates far more audit
   records than a user working, and the useful signal is the rarer one. This is the correct
   trade and it makes capacity planning harder.

4. **The Zitadel side is not ours to shape.** Its event log records what it records; where it
   will not carry our correlation id we hold the mapping, and a mapping is a second thing that
   can be wrong. Decision 3 states the intent and cannot state the mechanism until the first
   integration exists.

5. **This ADR mandates work with no customer.** Nobody has asked for an audit export. ADR-0041's
   argument is the whole justification, and it is an argument about the future.

**Gained**

- One question — *"show me everything this account did"* — becomes a query rather than a
  reconstruction, across both systems.
- The un-retrofittable half of ADR-0041 phase 1 is settled while there is no data.
- Denials being recorded makes STD-0009's discipline visible in production, not only in tests:
  the denial is the assertion there, and the evidence here.

**Rejected alternatives**

- **Application logs as the audit trail.** Cheapest, and logs are mutable, unretained, and shaped
  for debugging rather than for evidence. The first regulator question distinguishes them.

- **Correlation ids on error paths only.** Half the cost and useless: an investigation is about a
  successful request that should not have succeeded.

- **Rely on Zitadel's audit log alone.** Complete for authentication and blind to everything
  after it — every authorization decision and every data access happens in our services.

- **Defer until a customer or auditor asks.** The rest of ADR-0041 phase 1 can be added to a
  running system. This cannot: a request that has already been served cannot acquire an id, and
  the gap is exactly the period nobody can produce evidence for.

## Related

- ADR-0041 — the sequencing that makes this phase 1, and names it un-retrofittable
- REQ-0003 — personal data classification, which decision 8 depends on
- REQ-0013 — tenant data portability, which decision 5 extends to the audit trail
- REQ-0015 — the threat model this feeds
- ADR-0024, STD-0006 — tenancy and migration conventions the audit tables follow
- ADR-0038 — the session context that establishes who the subject is
- RFC-0012 — the two-system split decision 3 exists to bridge
- STD-0010 §8 — logs and metrics are locale-neutral and structured
