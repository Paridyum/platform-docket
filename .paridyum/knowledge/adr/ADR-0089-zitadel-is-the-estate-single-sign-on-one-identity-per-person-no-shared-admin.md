---
id: ADR-0089
title: Zitadel is the estate's single sign-on; one identity per person, role-scoped, never a shared admin
type: adr
status: accepted
created: 2026-09-13
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, infra, service]
applies_to: ["*"]
tags: [identity, sso, zitadel, oidc, saml, federation, admin, mfa, std-0017, rfc-0012, adr-0047, adr-0086, adr-0070, c-0003]
breaking: false
enforced_by: []
---

# ADR-0089 — Zitadel is the estate's single sign-on; one identity per person, never a shared admin

> **Superseded in part, 2026-09-28 — ADR-0113, ADR-0119.** The staff instance holds **staff only** (RFC-0017,
> ADR-0119 D1). Org members hold a portal credential in their org's region, and staff reach an org or
> a tenant **only by a grant** it can see and revoke (ADR-0120). Estate SSO is unchanged.

## Status

**Accepted**, 2026-09-13. Answers an operator question — *"wire Zitadel everywhere to log in, and use
one admin-level account for everything?"* — by naming the mechanism that makes the good half of that
true and the reserved-name/shared-credential half false. Applies STD-0017, RFC-0012 and ADR-0047;
adds the estate-SSO decision none of them states outright.

## Context

The instinct behind "one login for everything" is correct: a person should not hold a dozen
passwords, and offboarding should be one action, not twelve. But two of its literal forms are
already forbidden:

- **`admin@…` / one `admin` account** — STD-0017 reserves `admin`, `root` and `user` (*"the first
  three strings any credential-stuffing list tries"*) and requires role accounts to be
  service-qualified.
- **A shared login** — STD-0017 rule 4: *"a login two people share is not an account, it is a
  secret."*

The mechanism that delivers the intent without either is **single sign-on**: one *identity* per
person, trusted by every system, rather than one *credential* pasted into each. Zitadel is already
the estate's identity engine (ADR-0047, RFC-0012); this makes it the estate's SSO for staff and
operators, and says what federates to it.

## Decision

**1. One human identity per person, in the control-plane instance.** Each operator/staff member has
exactly one Zitadel identity, in the `id.paridyum.com` control-plane instance (ADR-0086 / the
two-plane split). That identity is their single sign-on. **No login is shared** (STD-0017 rule 4),
and **no account is named `admin`/`root`/`user`** (STD-0017). This is separate from tenant
identities, which live in the tenant plane — an operator's SSO identity is not tenant zero's.

**2. Every system that speaks OIDC or SAML federates its login to that instance.** The target set,
and where each stands:

| System | Mechanism | State |
| --- | --- | --- |
| `web-admin` (tenant admin) | OIDC | done |
| `console-*` operator tools | OIDC device flow (ADR-0070 D5) | contracted in `applications.yaml` |
| the Kubernetes API | OIDC (`kube-apiserver` → Zitadel) | aspirational — replaces static kubeconfig |
| observability (Grafana, …) | OIDC | when it exists |
| tenant-facing products | OIDC per product project (ADR-0070) | the tenant plane, not staff SSO |

**3. A system that cannot federate gets a registered role account, never a shared one.** Some
consoles only do SSO on a paid tier, and GitHub org SAML needs Enterprise (ADR-0004 keeps the org on
Free) — so **GitHub stays each person's own GitHub identity** (STD-0017 rule 1's exception, the name
is the truth) with MFA, and any other non-federatable service gets a service-qualified role account
in STD-0017's register, MFA'd, one person to a login. These are the documented exceptions, counted,
not a second habit.

**4. Authorization is per system; one identity is not omnipotence.** SSO answers *who*; each system
still answers *may they*. Role grants are per system and per project (RFC-0012 D7 — identity carries
grants, never adjudicates them), so a single identity holds different powers in different places and
is `operator` nowhere it was not granted (GLO-0005, ADR-0049 D2). **MFA is enforced at the IdP**, so
it is proven once and every federated system inherits it.

## Consequences

- **Offboarding is one action** — disabling the Zitadel identity ends access to every federated
  system at once, which is the whole reason to centralise. The registered exceptions (GitHub, paid
  consoles) are the short list still severed by hand, and being a list is the point.
- **Zitadel is now the login availability of everything** (RFC-0012 cost 4 restated one layer out):
  an outage stops new logins across the estate, not just one app. It is off the token-validation
  path (ADR-0047 D4), so existing sessions survive — but this is the service that must not fail, and
  the control-plane instance's own resilience is now an estate concern.
- **A compromised SSO identity reaches everything it was granted** — which is why decision 4's MFA,
  per-system role-scoping, and the control-plane instance's isolation from the tenant plane are not
  optional. One identity is a smaller attack surface than a dozen reused passwords, but a
  higher-value one.
- **No `admin` god-account exists to find** — a credential-stuffing list's first three guesses hit
  nothing, and there is no shared secret to leak or to fail to rotate when a person leaves.

## Rejected alternatives

- **One shared `admin@paridyum.com` for everything** — the literal reading of the question. Rejected
  on STD-0017 rules 1 and 4 and the reserved-name rule: it is unattributable (C-0003), unrotatable
  without locking everyone out, and the single most-guessed credential in existence.
- **A separate identity per system, no SSO** — no shared secret, but it recreates the dozen-passwords
  problem and, worse, the dozen-deprovisions problem: offboarding becomes a checklist that is one
  forgotten row away from a live account nobody owns. SSO exists to make that one row.

## Related

- **STD-0017** — role never person, reserved names, one person one account; the rules this satisfies.
- **RFC-0012, ADR-0047** — Zitadel as the identity engine; D7's authorization-federates, reused here.
- **ADR-0086** — the control-plane instance (`id.paridyum.com`) this SSO identity lives in.
- **ADR-0070** — one OIDC client per surface; the device flow consoles use.
- **ADR-0004** — GitHub Free, why org SAML SSO is not available and GitHub is a registered exception.
- **C-0003** — every action carries a real, attributable identity; a shared account cannot.
