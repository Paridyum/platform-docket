---
id: RFC-0058
title: A tenant publishes its apps under its own store accounts, and Paridyum is a grantee there
type: rfc
status: proposed
created: 2026-09-28
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, mobile, infra]
applies_to: ["*"]
product: [paridyum]
tags: [mobile, app-store, play-store, app-store-connect, white-label, publishing, signing, secrets, tenant-supplied-credentials, grants, localisation, apple-4-2-6, adr-0070, adr-0084, req-0001, req-0014, adr-0120]
breaking: false
enforced_by: []
sources:
  - url: https://developer.apple.com/app-store/review/guidelines/
    title: "App Review Guidelines — Apple Developer (guideline 4.2.6)"
    retrieved: 2026-09-28
  - url: https://developers.google.com/android-publisher/getting_started
    title: "Getting Started — Google Play Developer API"
    retrieved: 2026-09-28
---

# RFC-0058 — A tenant publishes its apps under its own store accounts, and Paridyum is a grantee there

## Status

**PROPOSED** 2026-09-28, with the org model (ADR-0113 to ADR-0122). Acceptance is the operator's act.

## Problem

**The operator, 2026-09-28:**

> *"this model help[s] us wire up Playstore and other management for pushing Paridyum apps under
> its Playstore account which model solves the requirement of other tenant can do the same."*

**A tenant's consumer apps need to reach phones**, and nothing in the estate says how.

- The `mobile` repository class exists, but no mobile product does.
- No artifact says whose store account an app is published under, who signs it, where the store
  credentials live, or how its listings follow a tenant's languages.

**What forces the shape is outside our control — the stores decide who may publish.** Apple's App
Review Guidelines, 4.2.6:

> *"Apps created from a commercialized template or app generation service will be rejected unless
> they are submitted directly by the provider of the app's content. These services should not submit
> apps on behalf of their clients and should offer tools that let their clients create customized,
> innovative apps that provide unique customer experiences. Another acceptable option for template
> providers is to create a single binary to host all client content in an aggregated or 'picker'
> model…"*

## Constraints

- **Apple 4.2.6, above.** A tenant's app is submitted by that tenant, not by Paridyum on its
  behalf.
- **Google Play Developer API.** An account owner grants a service account access: *"Go to the Users
  & Permissions page on the Google Play Console. Click Invite new users. Put an email address for
  your service account in the email address field and grant the necessary rights."*
- **ADR-0120 D7.** Access Paridyum is given to a tenant's external accounts is a grant:
  recorded, visible, revocable.
- **ADR-0070.** One OIDC application per deployable surface per environment. A tenant's mobile app
  is its own client in the tenant's Zitadel organization.
- **ADR-0084.** Branding is per tenant.
- **ADR-0122.** Store listings exist per content locale.
- **REQ-0001 (accepted).** Runtime secrets live outside repositories and outside GitHub.
- **ADR-0057.** SOPS and age for secrets at rest.
- **REQ-0014 (deferred).** A released artifact states what is in it and who built it. Store builds
  are released artifacts.
- **C-0006 anti-goal 3 and ANTI-0007.** No regulated data before controls. A store's own data-safety
  declarations must be honest (STD-0021).
- **BET-0005.** A tenant's end customers need never learn Paridyum exists, so white-label is the
  default.

## Proposal

### A. Each tenant owns its store accounts

- **The developer accounts are held under the legal identity of the business the tenant is** — a Google Play
  developer account, an Apple Developer Program team.
- **Paridyum never owns a tenant's store account**, including Paridyum's tenant, whose apps
  are published under Paridyum's tenant's accounts exactly as any tenant's are. That is the
  operator's requirement, satisfied by construction.

### B. Paridyum is a grantee inside those accounts, never a holder of their passwords

- **Google Play.** The tenant invites the service account Paridyum uses for it, under Users &
  Permissions, with release permissions only and an expiry where the store allows one.
- **App Store Connect.** The tenant either invites Paridyum's publishing identity as a team member
  with a release-scoped role, or creates an API key and supplies it (D).
- **Each is an outward grant** (ADR-0120 D7): recorded, shown on the tenant's console, and
  revocable by the tenant at the store. Paridyum's reconciler notices a revoked grant and marks
  publishing blocked rather than retrying.

### C. Builds are per tenant, from one product

- **A tenant's app is built from the product's single source**, with the tenant's own values:
  - bundle identifier;
  - name, icon and colours (ADR-0084);
  - OIDC client, in the tenant's Zitadel organization (ADR-0070);
  - content locales (ADR-0122).
- **It is signed with the tenant's key material** (D).
- **It records its provenance** — source revision, builder, inputs (REQ-0014's shape, whose trigger
  this fires).
- **No tenant-specific code branch exists.** A fork per tenant is BET-0004 trigger clause 2's
  fragmentation.

### D. Tenant-supplied credentials are a new class of secret, and get their own ADR first

Some store paths require material only the tenant can create, such as an App Store Connect API key
or an upload key where the store requires one. That material:

- is supplied by the tenant at the portal, **write-only** — never displayed again;
- is held encrypted, per tenant, **in the tenant's region**;
- is used only by the publishing path;
- is rotated on the tenant's request, and destroyed when the tenant revokes the grant or leaves.

**No secret class like this exists today.** REQ-0001 covers the estate's own runtime secrets.
ADR-0057 covers encryption at rest for repository-held secrets, and RFC-0046 covers tenant-held
encryption keys. **The design of this class is an ADR that must precede any code**, including where
the key that encrypts it lives.

### E. Listings follow the tenant's languages

- Store listing text and screenshots exist for every content locale the tenant activates.
- **Activating a content locale** (ADR-0122 D3) **includes the listing for every app the
  tenant publishes**, or the gate refuses.

### F. The tenant is the publisher of record

- The support contact, the privacy-policy link — per jurisdiction and per language — and the store's
  data-safety declarations are **the tenant's**.
- **Paridyum supplies drafts from the product's data map** — the column classifications REQ-0003
  requires — so the declarations can be true. It never submits a declaration the data map does not
  support (STD-0021).

### G. The picker model is Paridyum's own option, not a white-label one

Apple's 4.2.6 permits one binary hosting many clients' content. That suits a **Paridyum-branded**
consumer app — a product Paridyum operates under its own name. It does not suit a tenant that wants
its customers never to learn Paridyum exists (BET-0005), so it is **not the default for tenants**.
It remains available to Paridyum's own products.

## Alternatives considered

- **Paridyum publishes every tenant's app under Paridyum's accounts.** Rejected: Apple 4.2.6
  rejects it, and it breaks white-labelling and makes Paridyum the publisher of record for other
  tenants' customers' data.
- **Tenants publish entirely by hand.** It needs no secrets and no grants. **Supported as the
  fallback**, but it gives up per-release automation, and every tenant carries the store's
  toolchain.
- **A third-party white-label app-building vendor.** A real option. It would be a tool adoption
  (ADR-0066) and must pass REQ-0012's exitability test first.

## Consequences

- **The first mobile product triggers REQ-0014** (provenance) and the tenant-supplied-credentials
  ADR (D).
- **A per-tenant build pipeline**: bundle identifiers, signing and listings per tenant. Its cost
  grows with tenants × apps × locales, which is the argument for automating it once rather than
  per tenant.
- **Store review is outside our control.** A rejection is the tenant's, as the publisher of record.
  The portal must present it as such, with the store's words, never paraphrased into a promise.

## Rollout

| Slice | What lands | Depends on |
| --- | --- | --- |
| 0 | This RFC accepted | — |
| 1 | The tenant-supplied-credentials ADR (D) | — |
| 2 | Paridyum's tenant publishes its first app, through its own accounts, by grant | the first mobile product; slice 1 |
| 3 | Listings per content locale in the activation gate | ADR-0122 |
| 4 | Any tenant, through the portal | slice 2 rehearsed |

## Open questions

1. **Which store facts to verify and cite before slice 2.** Account-holder requirements, whether
   Google Play organisation accounts need extra verification, and the permission names for
   release-only access. This RFC deliberately states none of them from memory (RFC-0020).
2. **The picker model for Paridyum's own consumer products** — per product, when each exists.
3. **Where the credential class's encryption key lives** (D). Per region? Per tenant? That is the
   ADR's question.

## Related

- **ADR-0120** D7 (outward grants) · **ADR-0122** D3 (listings in the gate) ·
  **ADR-0119** (a tenant's apps sign in against the tenant's organization)
- **ADR-0070** · **ADR-0084** · **ADR-0111** (a Native client and the device flow, for app sign-in)
- **REQ-0001** · **ADR-0057** · **RFC-0046** · **REQ-0014** · **REQ-0003** · **STD-0021** ·
  **ANTI-0007** · **C-0006** anti-goal 3
- **BET-0005** (white-label) · **BET-0004** clause 2 (fragmentation) · **ADR-0066** and **REQ-0012**
  (adopting a vendor)
