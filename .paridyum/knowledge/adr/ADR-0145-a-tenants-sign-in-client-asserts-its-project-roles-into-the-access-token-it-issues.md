---
id: ADR-0145
title: A tenant's sign-in client asserts its project roles into the access token it issues
type: adr
status: accepted
created: 2026-10-11
locked_at: null
supersedes: null
superseded_by: null
scope: [platform, service, infra, spec]
applies_to: ["*"]
product: [paridyum, tenancy, identity, docket]
tags: [identity, zitadel, oidc, roles, access-token, client, provisioning, saga, adr-0070, adr-0092, adr-0126, adr-0130, rfc-0012]
breaking: false
enforced_by: []
---

# ADR-0145 — A tenant's sign-in client asserts its roles into the access token

## Status

**ACCEPTED** 2026-10-11 by the operator. **It amends ADR-0126 D4**: the PKCE client the saga
creates per enabled product carries the role-assertion flags of decision 1 below. It records,
rather than changes, what ADR-0130 D5 and ADR-0092 D1 already require.

## Context

ADR-0130 D5 provisions a tenant's owner *already linked*, so that *"the role is in the token from
the first sign-in"*. ADR-0092 D1 has a resource server — `tool-docket` — admit an operator on the
`operator` role it reads from the **locally-verified access token**, and the console reads the
same claim (ADR-0070 D4). Both take the role being in the token as a premise.

**Nothing had made that premise true.** In Zitadel, a project's `projectRoleAssertion` asserts a
project's roles into **userinfo** only. The roles reach the *access* and *id* tokens only when the
**application** sets `accessTokenRoleAssertion` and `idTokenRoleAssertion`. The registry's
`provision.py` set both, so `web-hiring` and `web-console` worked. But `platform-deployment`'s
`ensure_client` — the creator of each **per-tenant** client the saga makes (ADR-0126 D4) — set
neither.

Measured live 2026-10-11, on tenant-a3: a synthetic owner held `operator` on the tenant's docket
project (`projectRoleAssertion: true`), the project was in the token's `aud`, and the roles scope
was requested — and the access token carried **no roles claim at all**, while the same token's
userinfo carried `operator` correctly. Docket answered 401 *"the token does not carry the
'operator' role"*. Every saga-created client in the estate was made the same way: **26 of 29**
lacked the flag. The owner's role was `projectRoleAssertion`-only, into a claim nothing reads.

## Decision

**1. The creator sets the flags on the client it makes.**

The saga's identity step (ADR-0126 D4) creates each per-tenant client with both
`accessTokenRoleAssertion: true` and `idTokenRoleAssertion: true`. `platform-deployment`'s
`ensure_client` sets them in the config it writes, and rewrites a client found predating the
contract, so a retry heals it with no second provisioning.

**2. A client that predates the flag is healed, not recreated.**

An existing client without the flag is updated in place with that one flag on, changing nothing
else. `zitadel/repair-role-assertion.py` (`make zitadel-tenant-clients-repair`, infra-kubernetes)
does this for the clients that already existed; it is idempotent and applied 2026-10-11 to all 29.

**3. Nothing that is asserted only into userinfo is read for authorisation.**

A resource server or a surface takes a principal's project roles from the verified **access
token** (ADR-0070 D4, ADR-0092 D1). The contract that puts them there — the application's
`accessTokenRoleAssertion` — is a property of the client, and every client made for a sign-in
carries it.

## Consequences

**Accepted costs** — one more property on every tenant client, and a coupled one: turn roles off
on a client and its resource server stops authorising, with the failure (401, "the token does not
carry the role") naming neither the flag nor the client. Writing it into an artifact is now an
obligation for any future client creator, of which there are several (the saga, `provision.py`,
the control plane).

**Gained** — the premise ADR-0130 D5 and ADR-0092 D1 rest on is true and named. The owner's role
is in the token from the first sign-in; the console renders the roster; docket admits the operator
— verified end to end on the live estate the day this was accepted.

**Rejected alternatives** — *read userinfo for roles*: an extra round trip on every request, and
it contradicts ADR-0092 D1's locally-verified-token requirement. *Have each resource server
depend on the project flag alone*: it is the application's flag, not the project's, that writes
roles into the access token; the project flag was already true and the claim was still absent.
*Leave it carried by code and the repair script*: the contract is invisible to the next client
creator, which is exactly how these 26 arose.
