---
id: ADR-0091
title: A single-org instance brands its default; a multi-tenant instance keeps a vendor-stripped default
type: adr
status: accepted
created: 2026-09-13
locked_at: null
supersedes: null
superseded_by: null
scope: [platform, infra, org]
applies_to: ["*"]
product: [tenancy]
tags: [branding, identity, zitadel, label-policy, control-plane, adr-0084, adr-0090, rfc-0012, std-0015, white-label]
breaking: false
enforced_by: []
---

# ADR-0091 — A single-org instance brands its default; a multi-tenant instance keeps a vendor-stripped default

> **Superseded in part, 2026-09-28 — ADR-0113, ADR-0119.** The single-org instance this ADR brands is the **staff
> instance**. Each region's **portal instance** holds one directory, which mirrors no org and no
> tenant (ADR-0113 D2), and each tenant signs in on its own domain (ADR-0119 D2).

## Status

**Accepted**, 2026-09-13. **Refines ADR-0084** for the two-instance world that ADR-0090 / RFC-0012 D2
made real. ADR-0084 was written when there was one Zitadel instance shared by all tenants; it did not
contemplate the **control-plane instance** (`id.paridyum.com`), which is single-org (staff only). Not
`locked`: it graduates when a check asserts each instance default matches the rule below.

## Context

ADR-0084 D1 says branding is set on the **organization, never the instance default** — because in a
multi-tenant instance the default login screen is shown before any org resolves, and branding it with
one tenant's identity would show that brand to every other tenant. So the tenant instance default
stays generic.

Two facts change the picture for the control plane:

1. **The login's first screen renders the instance default, not an org.** At the `loginname` step no
   user has resolved an org yet, so Zitadel paints the *instance* default label policy. Branding only
   the org leaves the entry screen on the vendor's default — Zitadel purple `#5469d4` and the "Powered
   by ZITADEL" watermark. Observed directly: the `paridyum-internal` org was branded, yet
   `id.paridyum.com/ui/v2/login` still showed Zitadel until the instance default was set.
2. **The control-plane instance has exactly one org** (`paridyum-internal`, staff). ADR-0084's reason
   for keeping the default generic — *don't show tenant A tenant B's brand* — cannot occur where
   there is only ever one org. And ADR-0084 D1's own stated intent is that **"none is shown the
   vendor's"**, which on a single-org entry screen is only achievable by branding the default.

## Decision

**1. A single-org instance brands its instance default.** The control-plane instance (`id.paridyum.com`,
one staff org) sets its instance-default label policy to Paridyum's STD-0015 brand — the same palette
its org carries — so the entry screen is Paridyum, not Zitadel. This is applied via the admin API by
`provision-control.py brand` (from the `paridyum-system` key, ADR-0090 / the SystemAPIUsers
membership), alongside the org-level policy.

**2. A multi-tenant instance keeps its default generic — but vendor-stripped, not vendor-default.**
The tenant instance (`auth.paridyum.com`) default must still carry no tenant's brand (ADR-0084 D1
holds). But "generic" means **neutral and free of the vendor**, honouring ADR-0084 D1's "none is
shown the vendor's": the watermark disabled and Zitadel's logo and purple removed for a neutral
ground — never a specific tenant's (or Paridyum's own tenant-org) brand. Tenant orgs still brand
themselves; the entry screen simply stops advertising the engine.

**3. The rule is a property of the instance's tenancy, not a per-instance opinion.** One org →
brand the default. More than one org → vendor-stripped-neutral default. This is what a check should
assert when this ADR graduates.

## Consequences

- The control-plane staff login is fully Paridyum (colours, dark theme, logo, no watermark, and the
  init/verify/reset email texts rewritten off the stock "created in ZITADEL" wording). ADR-0089's
  "Zitadel is the engine, not the face" is now true on the plane staff actually see.
- The tenant entry screen is a **follow-up**: it is still on Zitadel's literal default and needs the
  neutral vendor-strip of decision 2 (watermark off, neutral colours, no Zitadel logo), applied with
  the tenant `iam-admin` PAT via the admin API. Tracked, not done here.
- ADR-0084 is unchanged for org-level branding, which remains the rule for every tenant.

## Related

- **ADR-0084** — branding is per-org; this refines it for the instance default across the two planes.
- **ADR-0090** — the per-instance login deployment whose entry screen this brands.
- **RFC-0012 D1/D2** — white-label identity; the staff/tenant instance split.
- **ADR-0089** — Zitadel is the estate engine, not the face.
- **STD-0015** — the one mark and palette applied here.
