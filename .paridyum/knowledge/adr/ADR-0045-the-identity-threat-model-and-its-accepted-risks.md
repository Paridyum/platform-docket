---
id: ADR-0045
title: The identity platform's threat model — what is defended, and the four risks accepted
type: adr
status: accepted
created: 2026-08-08
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service]
applies_to: ["platform-*", "svc-*"]
tags: [security, threat-model, identity, tenancy, zitadel, req-0015, compliance]
breaking: false
verified_by:
  - TST-0026
  - TST-0027
enforced_by:
  - check: db.session_context_accessors_fail_closed
    severity: error
    rationale: >
      Threat T1 — cross-tenant read — has exactly one enforcement point once tenants are Zitadel
      organizations rather than instances, and this is it. A session accessor that returns NULL
      instead of raising converts the highest-severity threat in this model into a silent one.
  - check: db.row_level_security_enabled_on_tenant_tables
    severity: error
    rationale: >
      T2 — cross-account read within one tenant — is defended by RLS alone. Zitadel has no
      isolation level below the organization, so a table with a policy that omits `account_id`
      is the whole of that defence, absent.
---

# ADR-0045 — The identity platform's threat model

## Status

**ACCEPTED** as of 2026-08-08. Partially discharges REQ-0015, whose trigger fired the same day.

**This is a first pass over one subsystem, not the organisation's threat model.** It covers the
identity platform because that is what has a design. It says nothing about the catalogue
products, the generated layer, or the supply chain.

## Context

REQ-0015's trigger is *"the first component that accepts input from outside the trust boundary is
**designed**"* — deliberately written to fire at design rather than deployment. RFC-0012 designs
exactly that: a public login surface, a proxy that terminates it, and a token that every product
trusts.

Two things make this urgent rather than procedural. ADR-0041 put SOC 2, GDPR and DPDP in phase 1,
and every one of them asks for this document. And RFC-0012's move from instance-per-tenant to
organization-per-tenant **changed where the isolation boundaries are enforced** — which is a
change to the threat model, made for commercial reasons, and it should not stand unexamined.

## The assets, in severity order

1. **Tenant data.** A cross-tenant read is *"catastrophic, reportable"* (GLO-0004) — one business
   reading a competitor's records, with Paridyum's own regulatory exposure attached.
2. **Account data within a tenant.** *"Catastrophic for that business, and it is their
   reputation, not ours."*
3. **Credentials and sessions**, held by Zitadel, shared per region.
4. **The Zitadel System API credential**, which creates and destroys organizations.
5. **The regional masterkey**, immutable once initialised.

## Threats, and what defends each

**T1 — cross-tenant read.** *Two controls.* A Zitadel organization boundary, and ADR-0024's RLS
constraining `tenant_id` with `FORCE`. The tenant value comes from the signed organization claim
and is set with `SET LOCAL` per transaction, so a pooled connection cannot carry the previous
request's tenant. `db.no_tenant_id_from_client_input` rejects the shape where it comes from
anywhere else.

**T2 — cross-account read inside one tenant.** *One control: RLS.* Zitadel has no isolation level
below the organization — *"ZITADEL does not enforce a native hierarchy or inheritance for users
or organizations"* — so every account in a tenant lives in one org and the inner silo is enforced
entirely by policy predicates on `account_id`. **This is the largest single change introduced by
organization-per-tenant** and it is accepted risk R1 below.

**T3 — privilege escalation via operator scope.** Defended by GLO-0005's rule that scope is a
positive grant, `db.operator_scope_is_an_explicit_grant` rejecting `IS NULL`/`coalesce`/wildcard
widening, and ADR-0038's accessors raising rather than returning NULL. The hazard is not an
attacker; it is a dropped session field producing breadth.

**T4 — token confusion between products.** A token minted for product A accepted by product B.
Defended by per-project audiences and the rule that every resource server validates `iss` **and**
`aud`, not signature alone.

**T5 — compromise of the System API credential.** The highest-consequence credential: it can
destroy every tenant in a region. Held only by `platform-identity`, never delegated to a tenant,
and inside REQ-0001's scope — which is `deferred`, and is the reason R3 below is accepted rather
than mitigated.

**T6 — a malicious or buggy Zitadel Action.** Actions run custom code inside the token flow, in a
process shared by every tenant in the region, and consume that region's action-seconds. Never
delegated to tenants (RFC-0012's capability matrix), and Paridyum-authored Actions are code that
needs review like any other.

**T7 — noisy neighbour / denial of service.** One tenant's login storm degrades a region.
Organization-per-tenant loses Zitadel's per-instance quota, so this moves to rate limiting at
`platform-identity`. Accepted risk R2.

**T8 — audit gap.** An action nobody can attribute. Defended by ADR-0044: correlation ids
propagated from the edge, append-only records, denials recorded as well as successes.

**T9 — masterkey loss or compromise.** Immutable once initialised, per region. Loss means the
region's encrypted data is unrecoverable; compromise means it is readable. No rotation path
exists in the product, which is a property of Zitadel rather than a choice.

## Accepted risks

**R1 — the inner silo has one enforcement layer.** Cross-account isolation depends on Postgres
RLS alone, because Zitadel offers no second level. *Accepted* on two grounds: GLO-0004 rates T1
above T2, and T1 is where Paridyum's own reportable exposure sits, so the doubled control is on
the correct boundary. RLS with `FORCE` is not a weak control — six checks in
`check-migrations.sh` and STD-0009's denial-asserting suites enforce it — but it is *one*
control, and the honest statement is that application data is now more strictly isolated than
identity data.
**Partially mitigated 2026-08-08.** ADR-0038 decision 5 adds a second, independent control: the
repository layer asserts every returned row's `account_id` against the session context and raises
on a mismatch. The two fail independently — RLS through a policy omitting `account_id`, a missing
`FORCE`, or a `BYPASSRLS` role; the assertion only if `lib-auth-typescript` itself is wrong. That does not
restore the *isolation* level Zitadel would have provided, and it does close the case where a
single policy is written wrong.
**Revisit if** a tenant's contract or auditor requires a dedicated user store, at which point
that tenant needs its own virtual instance.

**R2 — no per-tenant quota.** Rate limiting at the proxy is a weaker instrument than a Zitadel
quota, because the proxy sees logins and not per-API-call usage. *Accepted* as the price of the
catalogue topology.

**R3 — runtime secrets have no home.** The masterkey, database credentials and service-account
keys all exist before REQ-0001 is discharged, and REQ-0001's own context notes GitHub Free has no
org-level secrets for private repositories. *Accepted only until the first deployment.* This is
the risk most likely to be realised and the least defensible to leave standing, so it is the one
clause here whose **expiry is recorded rather than intended**: DECISION-DEBT.md carries it, and
ADR-0011's Stop hook resurfaces it at the start of every session until it is cleared. An accepted
risk with no expiry becomes a permanent one, and the ledger is the only mechanism here that
refuses to forget.

**R4 — all tenants in a region share one Postgres.** A database compromise is regional, not
per-tenant. *Accepted*: it is inherent to the deployment constraint, and the alternative is one
Zitadel per tenant, which was rejected on operational grounds.

## Consequences

**Accepted costs**

1. **This model is a snapshot and will be wrong.** It describes a system with no code. Every
   threat here is inferred from a design, and the first real deployment will surface at least one
   nobody listed.
2. **R3 is a live risk carrying an "accepted" label**, which is a real hazard in itself: an
   accepted risk with no expiry becomes a permanent one, and this one has a natural deadline
   nobody has written down.
3. **Nothing here is enforced by machinery.** The two checks named above defend T1 and T2; T4
   through T9 rest on compliance with prose and on checks owned by `tool-forge`.
4. **The scope is one subsystem**, and stating that plainly makes the gap visible rather than
   letting this stand in for the organisation's threat model.

**Gained**

- REQ-0015's trigger is discharged for the subsystem that fired it, while the design can still
  change cheaply.
- The isolation change RFC-0012 made for commercial reasons is now examined and recorded rather
  than assumed benign.
- ADR-0041 phase 1 has the artifact every one of its frameworks asks for first.

**Rejected alternatives**

- **Defer until a service exists.** REQ-0015 was written to fire at design precisely because a
  model produced after deployment documents what was built rather than shaping it.
- **A full STRIDE pass over the whole platform.** More rigorous and mostly speculative — four of
  the five contexts have no code. Rigour applied to imagined systems produces confident documents
  about nothing.
- **Fold this into REQ-0015.** A requirement states what must be true; a threat model records
  decisions about accepted risk. They have different lifecycles, and R1–R4 are decisions.

## Related

- REQ-0015 — the requirement this partially discharges
- RFC-0012 — the design examined here; its cost 1 is R1 stated from the other side
- ADR-0038 — the fail-closed accessors defending T1 and T3
- ADR-0024, STD-0009 — the RLS model and the tests that prove denial
- GLO-0004, GLO-0005 — the severity ordering and the absence hazard
- ADR-0041 — the compliance sequence requiring this
- ADR-0044 — the audit trail defending T8
- REQ-0001 — runtime secrets; R3 is its unpaid balance
