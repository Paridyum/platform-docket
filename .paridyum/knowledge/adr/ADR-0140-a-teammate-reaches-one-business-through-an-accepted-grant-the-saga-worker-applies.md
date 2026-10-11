---
id: ADR-0140
title: A teammate reaches one business through an accepted grant the saga worker applies
type: adr
status: accepted
created: 2026-10-10
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform]
applies_to: ["platform-tenancy", "web-dashboard"]
product: [tenancy]
tags: [grants, operator-grants, org-model, zitadel, saga, revocation, rfc-0056, adr-0120, adr-0130]
breaking: false
enforced_by:
  - check: test.org_membership_confers_no_business_access
    severity: error
    rationale: Org membership alone must never reach a business; only an applied, unrevoked grant does (ADR-0120 D1).
sources:
  - url: https://zitadel.com/docs/guides/integrate/identity-providers/introduction
    title: "Identity providers — ZITADEL Docs"
    retrieved: 2026-09-28
    supports: >
      A user in an organization can carry a link to an external identity provider, which D2 uses to
      find the grantee's linked user by portal provider and subject.
  - url: https://zitadel.com/docs/guides/integrate/retrieve-user-roles
    title: "Retrieve User Roles in ZITADEL — ZITADEL Docs"
    retrieved: 2026-09-27
    supports: >
      Roles granted on a project appear in the user's token, which is how D2's `operator` grant takes
      effect and D3's removal ends it at the next token refresh.
---

# ADR-0140 — A teammate reaches one business through an accepted grant the saga worker applies

## Status

Accepted 2026-10-10 by the operator. It implements RFC-0056 U4 ("give a teammate access to one
business"), specifies ADR-0120 D3 (how a grant is applied), and extends ADR-0130 D5 (the owner is
provisioned already linked) to people who are not the owner.

## Context

ADR-0120 D1 makes every human reach into a tenant a per-tenant grant, and says nothing else confers
access — not org membership. Today the only grant that exists is the creating owner's: the saga's
`owner_access` step links the owner's user in the tenant's Zitadel organization and `first_grant`
writes the row (ADR-0130 D5, ADR-0118 D5). An org owner has no way to let a teammate into one
business, and ADR-0120 D3 says "the portal applies it" without saying how — while ADR-0128 D1 keeps the
portal out of every tenant, so the portal cannot be the thing that writes into a tenant's Zitadel
organization.

## Decision

### D1. Offer, then accept

- An org `owner` or `admin` **offers** an `operator_grants` row for a tenant the org owns, to a **live
  member** of that org. The row carries the role (`operator`), the grantee's issuer and subject, the
  grantor, and its state.
- The grantee **accepts** in the portal. Their verified address comes from **their own token's
  introspection** at acceptance, as migration 0025 does for the owner — never from the grantor.
- This acceptance is ADR-0120 D8's "consented at the grant": the linked user is created only from what
  the grantee consented to.

### D2. A durable job run by the saga worker applies it

The grant is applied by a durable job run by the saga worker (`tenancy-saga` / `tenancy-provisioner`,
ADR-0070 D3) — not by the portal. The job is idempotent and retried; a failure is recorded on the row.
It:

1. finds the user in the tenant's Zitadel organization **by IdP link** (the portal identity provider
   and the grantee's subject);
2. creates one **only if none exists**, under an id derived from tenant, issuer and subject (UUIDv5),
   so a retry never makes a second user;
3. grants `operator` on **each enabled product project that declares it**.

### D3. Revocation is write-once on the row

- A revocation is written once on the row and never cleared.
- The job removes that role's grants from the linked user.
- The linked user is **deactivated only when the person has no live grant left in that tenant**.

### D4. Revocation from both sides

A grant can be revoked from the portal by the org's `owner` or `admin` (the org granted it) and from the
tenant's own dashboard (ADR-0120 D4: revocation belongs to the tenant). Both write the same revocation
on the same row.

### D5. Schema: migration 0027

Migration 0027 extends `operator_grants` with `role`, `grantee_issuer`, a nullable `account_id`
(ADR-0120 D6), and the offer / accept / apply state.

**Vocabulary to reconcile:** the saga's `first_grant` writes `scope = 'owner'`, while ADR-0120 names the
role `operator`. The migration and the slice that follows it must settle one vocabulary — the owner's
grant is an `operator` grant whose origin is provisioning — and migrate existing rows to it.

### D6. Out of this decision's first slices

The reconciler (ADR-0120 D3) and the member-leaves fan-out (ADR-0120 D5) are follow-on slices. Until
they land, a leaver's grants are revoked by hand and drift is not alerted.

## Alternatives considered

- **The portal writes the Zitadel grant directly at acceptance.** Rejected: ADR-0128 D1 keeps the portal
  out of every tenant, and a synchronous remote call at click time has no retry and no durable failure.
- **Grant at offer, no acceptance.** Rejected: the grantee's address would come from the grantor, and
  ADR-0120 D8 requires the linked user to be consented at the grant.
- **Org membership implies access to every business.** Rejected by ADR-0120 D1; the enforcing test
  `test.org_membership_confers_no_business_access` holds the line.
- **Create a fresh user per grant.** Rejected: a person granted twice would hold two identities in one
  tenant; finding by IdP link and a derived id keep one.
- **Delete the linked user on any revocation.** Rejected: it would cut a person's other live grants in
  the same tenant; deactivation waits for the last one.

## Consequences

- Teammates reach exactly the businesses they were granted, and the record of who let them in is the
  row, applied by one worker with retries.
- A grant takes effect after the job runs, not at the click; the portal shows the row's state.
- The saga worker gains a second kind of work beside provisioning, with the Zitadel role it already
  holds for `owner_access`.
- The `scope`/`role` vocabulary must be reconciled before migration 0027 ships.
- Until the reconciler and fan-out slices land, drift and leavers are an operator duty.

## Related

- **RFC-0056** U4 · **ADR-0070** D3 · **ADR-0118** D5 · **ADR-0120** D1, D3, D4, D5, D6, D8 ·
  **ADR-0128** D1 · **ADR-0130** D5
