---
id: ADR-0092
title: An operator authenticates with a Zitadel token, bound to a docket operator by opaque subject
type: adr
status: accepted
created: 2026-09-13
locked_at: null
supersedes: null
superseded_by: null
scope: [platform, tool, console, org]
applies_to: ["*"]
tags: [identity, zitadel, oidc, operator, docket, tool-docket, console-docket, jwks, adr-0047, adr-0054, adr-0055, adr-0070, adr-0072, rfc-0012, d-38]
breaking: false
enforced_by: []
---

# ADR-0092 — An operator authenticates with a Zitadel token, bound to a docket operator by opaque subject

> **Dated note, 2026-09-28 — the ids below belong to a control plane that no longer exists.** The
> `id.paridyum.com` instance, its owner user (`390506086174098663`), the `paridyum-operations` project
> (`390606436373103770`) and the `console-docket` client (`390606437346247834`) were destroyed with the
> estate's rebuild from empty (ADR-0118 D3). The decision stands, and the rebuild is its first test:
> because the binding is by opaque subject, the rebuilt owner is a **new subject**, and
> `tool-docket`'s `registry/operators.yaml` must be re-bound to it before docket admits anyone. The
> rebuilt ids are recorded where they are used — infra-kubernetes `zitadel/provision-control.py` and
> `GO-LIVE.md` — not here.

## Status

**Accepted**, 2026-09-13. Resolves tool-docket's **D-38** ("the HR platform has no per-operator
authorisation … anyone who can tunnel in has full write access … waits for platform-identity"). Not
`locked`: the binding and role gate are mechanically checkable, but no check asserts them yet; it
graduates when `check-registry.yaml` carries one. Depends on ADR-0090 (the control-plane login) and
the operations project re-homed to `id.paridyum.com` (RFC-0012 D2).

## Context

Two identity records name the same person and are not connected:

- **Zitadel**, control plane (`id.paridyum.com`): user `rohan`, opaque subject `390506086174098663`,
  granted the `operator` role in `paridyum-operations` (ADR-0070 — a project is a role vocabulary).
- **docket** (`registry/operators.yaml`): `id: rohan, name: ROHAN BATRA`, which every write records
  for the audit trail.

Today the docket operator is a **claim, not a proof**. `tool-docket`'s HTTP adapter takes an
`x-docket-operator` header checked against the registry — attribution "exactly as strong as the
CLI's `--by`" (its own words) — and the real security is that the server binds loopback and is
reached over `ssh -L` (ADR-0072): the OS is the identity. Anyone who can tunnel in can claim to be
any operator, and there are no roles. That was tolerable with one operator on one machine; it is
D-38, left open by design until an identity system existed. `platform-identity` and the control-plane
instance now exist, so it can close.

The constraints that shape the answer, each retrieved rather than assumed:

- **RFC-0012 D1**: products validate tokens **locally against JWKS**; nothing routes validation
  through a proxy. So the check belongs in the product, `tool-docket`, not in the console.
- **ADR-0055 §6**: a console holds no business rules; an authorization decision is a rule.
- **ADR-0054**: capability is **added, not swapped** — OIDC is added *beside* the loopback/SSH model
  (ADR-0055 §5 says exactly this), not in place of it.
- **ADR-0051**'s instinct: bind on an identifier that cannot silently become wrong. A loginname is
  mutable; the subject is opaque and permanent.
- **ADR-0047 D5 / ADR-0070 D4**: audience is opt-in and client-requested, so a correctly-signed
  token minted for *another* application is one the issuer would otherwise accept — the audience
  must be checked, not just the signature.

## Decision

**1. An operator authenticates with a Zitadel access token from the control-plane instance, and the
`operator` role gates the write.** No token, or a token without `operator` in `paridyum-operations`'s
roles, is refused. The role is the authorization vocabulary (ADR-0070); the first slice treats
`operator` as the full access the SSH tunnel already granted — now *authenticated and attributed to a
verified person* — with finer-grained roles a later ADR-0054 addition, not a prerequisite.

**2. The Zitadel identity binds to the docket operator by the opaque `sub`.** `operators.yaml` gains
an optional `subject:` per operator (the Zitadel user id). `tool-docket` resolves the operator by
matching the token's `sub` to `operators[].subject`, and records the docket `id` on the write exactly
as before. Displayed to a human it reads `rohan@paridyum` → **ROHAN BATRA**; underneath it is
`390506086174098663` → `rohan`, so a Zitadel rename cannot break the link or reattach the ledger to
the wrong person. An operator with no `subject` keeps working over the CLI/`--by` path (below) but
cannot authenticate over OIDC until bound — the binding is a deliberate act, not a fallback.

**3. Validation lives in `tool-docket`, on the HTTP adapter.** It verifies the bearer token against
`id.paridyum.com`'s JWKS, pins the issuer, and **requires the audience to contain the
`paridyum-operations` project** (ADR-0047 D5 / ADR-0070 D4 — defeating a valid token minted for a
different app), then requires the `operator` role and resolves `sub`→operator. `console-docket`
forwards the token and holds no part of this decision (ADR-0055 §6, RFC-0012 D1). The verifier is a
focused module in `tool-docket`; there is no shared Rust auth crate today (`lib-auth-typescript` is
TypeScript, platform-identity's verifier is in-crate), so this is the second in-crate Rust verifier —
extracting a `lib-auth` Rust sibling waits for the third consumer, and is recorded as debt rather
than built speculatively (the ADR-0055 "no template yet" pattern).

> **The trigger fired on 2026-09-24 and the debt is closed.** `platform-tenancy` copied
> `platform-identity`'s verifier and became the third consumer, so `lib-auth-rust` (crate
> `paridyum-auth`) was extracted under ADR-0107 and this module now consumes it: the JWKS, issuer
> and audience mechanism moved, and the `operator` role requirement and the org→tenant derivation
> stayed here, because those are docket's policy (ANTI-0004). The sibling is named
> `lib-auth-rust` rather than `lib-auth` for the reason ADR-0107 gives.

**4. OIDC is added beside loopback+SSH, not in place of it (ADR-0054).** `tool-docket` keeps binding
loopback and keeps the `ssh -L` access model (ADR-0072) as defence in depth; the token becomes the
authenticated identity on the **HTTP/console path**, replacing the claimed `x-docket-operator` header
there. The **CLI `--by` path stays** for on-machine administration, where SSH is the authentication
and there is no browser to run an OIDC flow. Two entry points, two honest models — not a regression,
the same shape ADR-0055 §5 named.

**5. `console-docket` is the OIDC-PKCE native client.** It uses the control-plane client
`390606437346247834` (native, PKCE, loopback `127.0.0.1:8471`, re-homed to `id.paridyum.com`),
requests the `paridyum-operations` audience and roles scopes, holds the token server-side (Next.js,
never in the browser), and forwards it as `Authorization: Bearer` to `tool-docket`. It uses
`lib-auth-typescript` (`@paridyum/auth`) for the flow and session. The loopback callback reaches the operator's own
browser through the same `ssh -L` forward that reaches docket (ADR-0072); the redirect URI is the
operator's localhost, which is what a native OIDC client is for.

## Consequences

**Gained.** D-38 closes: a docket write is attributable to a *proven* person, not a claimed string;
a departed operator loses access when their Zitadel account is disabled, with no registry edit; and
the `operator` role gives docket a place to grow finer authorization without another auth mechanism.

**Accepted costs.**
- **A second in-crate Rust JWKS verifier** until a `lib-auth` Rust sibling is worth extracting; the
  duplication is the RFC-0012 D1 "validate locally" cost, bounded and recorded. **Paid off
  2026-09-24 by ADR-0107** — `lib-auth-rust` exists and this crate consumes it.
- **The loopback OIDC callback over `ssh -L`** is fiddlier than a served web app's; it is inherent to
  ADR-0072's one-node/tunnel model and ADR-0055's console class, not introduced here.
- **`operators.yaml` now carries a Zitadel id**, coupling the registry to the instance. It is an
  opaque id, not a secret, and the coupling is the point of the binding.

**Rejected alternatives.**
- **Validate in `console-docket`, trust it over loopback** — violates ADR-0055 §6 (a rule in the
  console) and RFC-0012 D1 (the product must validate), and would leave `tool-docket` trusting any
  process on loopback, which is exactly D-38.
- **Bind on `preferred_username`** — simpler and matches the mental model, but a Zitadel rename then
  silently breaks the link or, worse, rebinds the ledger; ADR-0051's whole reason.
- **Replace loopback/SSH with OIDC** — violates ADR-0054; loses the defence-in-depth of an
  unreachable-without-a-key socket, and strands the CLI path that has no browser.

## Related

- **D-38** (tool-docket DECISION-DEBT) — the open item this resolves.
- **RFC-0012 D1/D2/D5**, **ADR-0047** — validate locally, the two planes, the capability matrix.
- **ADR-0054**, **ADR-0055**, **ADR-0072** — added-not-swapped, the console class, one-node docket.
- **ADR-0070**, **ADR-0051** — a project is a role vocabulary; identifiers that cannot go stale.
- **ADR-0090**, **ADR-0089** — the control-plane login this authenticates against; Zitadel is the SSO.
