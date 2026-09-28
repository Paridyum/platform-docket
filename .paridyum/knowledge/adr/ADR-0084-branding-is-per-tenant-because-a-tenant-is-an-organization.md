---
id: ADR-0084
title: Branding is per tenant, because a tenant is an organization
type: adr
status: accepted
created: 2026-09-12
locked_at: null
supersedes: null
superseded_by: null
scope: [platform, service, org]
applies_to: ["*"]
product: [tenancy]
tags: [branding, identity, zitadel, label-policy, tenancy, std-0015, adr-0047, adr-0049, adr-0070]
breaking: false
enforced_by: []
---

# ADR-0084 — Branding is per tenant, because a tenant is an organization

> **Superseded in part, 2026-09-28 — ADR-0113.** "A tenant is an organization" here means **the tenant's Zitadel
> organization** (ADR-0047 D2), not an Org (GLO-0012). Branding stays per tenant, configured at the
> portal (ADR-0119 D2).

## Status

**Accepted**, 2026-09-12. Paridyum is the first tenant and the first to be branded (C-0008,
ADR-0049). Not `locked`: it graduates when a check asserts every provisioned org carries an
org-level label policy rather than inheriting the instance default.

## Context

A tenant is a Zitadel organization (ADR-0047), and Zitadel resolves login-UI and email branding
**per organization** — a label policy (logos, the four theme colours light and dark, font, a
watermark toggle) and per-org message texts. Left alone, an org inherits the instance default,
which is Zitadel's own purple-on-grey with a "Powered by Zitadel" watermark — what the operator's
own first login screen showed. That is wrong twice: it is not Paridyum's identity, and in a
multi-tenant estate it would show one tenant the vendor's brand instead of their own.

Because branding is an org property and every tenant is an org, per-tenant branding is not extra
machinery — it is the natural grain of the identity model. The only decision is that we *use* it,
always at the org level, rather than ever styling the instance default and letting tenants inherit
it.

## Decision

1. **Branding is set on the organization, never on the instance default.** Every provisioned tenant
   gets its own label policy and message texts. The instance default stays generic and unbranded;
   no tenant is ever shown another tenant's brand, and none is shown the vendor's. This is the
   ADR-0049 D2 rule applied to pixels: Paridyum's branding is an org policy exactly as a customer's
   would be, with no privileged instance-level shortcut.

2. **Paridyum's branding is STD-0015, and only STD-0015.** One mark, one palette: signal `#35ecde`,
   bone `#f9f5ec`, obsidian `#000000`. The login runs the **dark** theme — signal on obsidian clears
   contrast (14.2:1), which is the only ground on which STD-0015 permits the accent to carry type, so
   the primary/link colour may be `#35ecde` there; on the light theme the primary is the darkened
   `#0a7a72` (STD-0015's `--color-signal-ink`), never the raw accent. The logo is the byte-identical
   asset from `lib-ui/brand`, not a re-export (STD-0015's logo rule). The watermark is disabled.

3. **Branding is declared per org and reconciled, like everything else in the org registry.** A
   `branding` block per organization in `organizations.yaml` (theme, colours by token, logo, and any
   message-text overrides), applied idempotently by a provisioning tool. A tenant's look is reviewable
   as data, not clicked into a console (the argument `applications.yaml` and `organizations.yaml`
   already won).

4. **Message text is branding too.** The stock init email reads "This user was created in Zitadel";
   the per-org message texts replace the vendor's wording with the tenant's. Owned by the same
   per-org declaration, applied where the API supports it.

## Consequences

- `configure-branding.py` (`make zitadel-branding`) reads the per-org `branding` block, creates or
  updates the org's label policy, uploads the logo, sets `disableWatermark`, and activates it —
  idempotent, the pattern of `configure-smtp.py` and `provision-org.py`.
- Paridyum's org is branded now: dark theme, the STD-0015 palette, the `lib-ui/brand` mark, no
  watermark. Its login and its emails stop looking like stock Zitadel.
- A logo asset that Zitadel will not accept as SVG is rasterised from the same source rather than
  redrawn, preserving STD-0015's byte-identity intent as closely as the target allows.
- Every future tenant gets its own branding by adding a block, never by touching another tenant's or
  the instance default.

## Related

- **STD-0015** — the one mark and one palette this applies, and the accent-is-never-type rule the
  dark ground satisfies.
- **ADR-0047 / ADR-0049** — tenant = org, and Paridyum brands itself through the same per-org path a
  customer would.
- **ADR-0070 / ADR-0082** — the identity surfaces (login UI, init email) this brands.
