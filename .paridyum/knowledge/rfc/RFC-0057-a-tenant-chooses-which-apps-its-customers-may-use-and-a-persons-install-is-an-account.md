---
id: RFC-0057
title: A tenant chooses which apps its customers may use, and a person's install is an account
type: rfc
status: proposed
created: 2026-09-28
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, web]
applies_to: ["*"]
product: [paridyum, tenancy, entitlement]
tags: [apps, catalogue, apps-as-a-service, install, account, instance, entitlement, franchise, customer, b2c, b2b, adr-0104, adr-0112, bet-0005, c-0006, c-0008, adr-0116, glo-0003]
breaking: false
enforced_by: []
---

# RFC-0057 — A tenant chooses which apps its customers may use, and a person's install is an account

## Status

**PROPOSED** 2026-09-28, with the org model (ADR-0113 to ADR-0122). Acceptance is the operator's act.

## Problem

**The operator named this "the missing piece"** (2026-09-27):

> *"the business owner can just allow what apps as a service to show as a runnable app in tenants
> actual instance for their customers to install and run — like paridyum.com is going to have all
> the apps enabled so when someone creates an account on paridyum.com its identity and everything
> business owner specific should be governed by this portal."*

On 2026-09-28 the operator described each business as having *"an ecosystem of consumer end
products."*

**No artifact covers any of this.** Searches of `adr/` and `rfc/` return nothing on installable app
catalogues, customer installs, or owner-curated enablement. The Release model (RFC-0049) says what a
tenant *runs*. Nothing says what a tenant's *customers* may use, who installs it, or what an
install is.

**The obvious answer is half wrong.** "A customer installs an app, and the install is an account"
is right for a *person*. For a *company* — a customer with its own staff and records — it is the
trap the 2026-09-27 brief documents (F-A to F-C): for docket the container is final at the first
record, and the obligation is the company's.

## Constraints

- **ADR-0104.** A product instance is an account, addressed as `/app/<product>/<account>/…` on the
  tenant's host. Accounts are free to create (ADR-0115 D7).
- **GLO-0003.** Every user has exactly one personal account, and an account has no relationship with
  Paridyum.
- **ADR-0112.** A product is a catalogue product by default. A catalogue product claims a market and
  its stages gate it.
- **BET-0005 and C-0006 anti-goal 1.** *"We will not franchise a vertical we have not operated."* For
  another tenant to offer a catalogue product to its own customers *is* franchising it, so the
  product must be at `franchised`.
- **C-0006, framing table.** No marketplace: *"a marketplace is a venue for other people's
  software."* **C-0006 anti-goal 6:** no second vertical before the first has a renewed customer.
- **C-0008 and ADR-0116 D2.** Paridyum's tenant having every app enabled must be configuration any
  tenant could have on the same terms — never a privilege.
- **ADR-0120.** A company is reached through grants into its own tenant.
- **ADR-0122 D3.** An app is offered in a language only if it ships that language.
- **ADR-0101 D3 and T-3.8.** "What may this tenant offer" is an entitlement question, and its
  trigger fires here.

## Proposal

### A. An owner configures three things per tenant, at the portal

| # | Setting | What it is | Where it lands |
| --- | --- | --- | --- |
| 1 | **Own apps** | the products the tenant runs for **itself** — docket for its staff, careers, its landing | Releases in the tenant (RFC-0049) |
| 2 | **Customer apps** | the products the tenant's **customers** may use — the catalogue shown on its host | the tenant's entitlement and configuration |
| 3 | **Who may install** | self-install by any signed-in customer, or assignment by the tenant's staff | per customer app |

### B. A person's install is an account

- A customer signed in on the tenant's host installs an enabled app, and gets **an account — an
  instance — of that product in the tenant**, bound to them through `account_members`
  (ADR-0120 D6). This is ADR-0104's model, unchanged.
- **The account boundary separates one customer from another.** The tenant's own staff read across
  accounts only under operator scope (GLO-0005). The tenant, as the customer's vendor, discloses
  that in its terms.
- **Addressing** is ADR-0104's: `/app/<product>/<account>/…` on the tenant's host. A customer with
  one instance of a product never types its segment, because the bare product path redirects.

### C. A company is never installed; it is set up

- **An app is marked person-scale or business-scale in its product contract.**
- **A person-scale app installs as an account** (B).
- **A business-scale app** — one whose customer is a company with its own staff and records, docket
  being the case in point — is offered to a company customer as *"set up your own business"*:
  - the company creates its own org and tenant at paridyum.in (RFC-0056 U1–U3), or its
    provider creates the tenant for it and transfers it (U6, U10);
  - it grants the provider access (ADR-0120).

  This is rung 3, and the reason is the brief's F-A to F-C: a sealed-chain container is final, the
  company is the fiduciary, and an account cannot carry a company's identity.
- **The classification is judgement, recorded in the product's contract** (the `product-contract`
  slot in `product-registry.yaml`). No gate can decide what a customer is.

### D. Which products may appear in a tenant's customer catalogue

| Tenant | May offer | Why |
| --- | --- | --- |
| **Paridyum's tenant** | every product at `operated` or later | BET-0005: Paridyum operates its catalogue before franchising it |
| **any other tenant** | capability products for its own use (A1); catalogue products to its customers **only at `franchised`** | C-0006 anti-goal 1; ADR-0112 |
| **anyone** | **never** another party's software | C-0006: no marketplace. The day a tenant could list someone else's app, this RFC is void |

**"Paridyum's tenant has every app enabled" is D's first row applied to Paridyum.** Any tenant
reaches the same set as each product reaches `franchised`. It is configuration, not a privilege.

### E. Entitlement decides what a tenant may offer

- **"May this tenant offer product X to its customers" is `platform-entitlement`'s question**, and
  T-3.8's trigger — *"the first plan distinction that must be enforced"* — fires with this RFC.
- **It is a gated action** (RFC-0015 D1): answered from the control plane, and refused closed when the
  control plane is unreachable.

### F. Languages

- **A customer app appears in a language only if it ships that language** (ADR-0122 D3).
- **A tenant's catalogue page shows only the apps available in the visitor's resolved content
  locale**, with ADR-0034 D7's deterministic fallback.

### G. What customers pay for is the tenant's own commerce

A tenant charging its customers for apps is *"a tenant's invoice to its accounts (a catalogue
product)"* (ADR-0042 D6). It is gated on the payment path (T-0.10), and BET-0007's royalty applies at
`franchised`. **It is not designed here.**

## Alternatives considered

- **One shared instance of each app for all a tenant's customers, separated by rows.** Rejected: an
  instance is the isolation unit (ADR-0104), and separating customers inside one instance
  reintroduces the inner boundary without its RLS predicate.
- **Every customer as a tenant, even a person.** Rejected: a person needs no tenant, accounts are
  free (ADR-0115 D7), and a tenant per consumer would multiply Zitadel organizations and regional
  rows for no isolation gain.
- **Every customer as an account, even a company.** Rejected: brief F-A to F-C. This is the trap,
  not a simplification.
- **A marketplace of third-party apps.** Rejected: C-0006, until that artifact changes.

## Consequences

- **`platform-entitlement` moves onto the portal's launch path**, because the customer catalogue is
  its first real distinction.
- **Every product contract gains a scale field** — person or business — which becomes a
  `product-contract` slot requirement.
- **`account_members` (ADR-0120 D6) becomes the binding for customer installs** as well as for
  per-instance staff grants: one mechanism, two populations.
- **For most tenants the day-one customer catalogue is nearly empty.** Only capability products
  are offerable before any catalogue product reaches `franchised`, and that is anti-goal 1 working
  as intended. Paridyum's tenant is the showcase because it operates the products.

## Rollout

| Slice | What lands | Depends on |
| --- | --- | --- |
| 1 | Own apps (A1) through the portal, over Releases | RFC-0049; ADR-0118 |
| 2 | The scale field in product contracts; docket and hiring classified | ADR-0112's re-review for each |
| 3 | Customer apps (A2) plus entitlement (E), Paridyum's tenant first | `platform-entitlement`; T-3.8 |
| 4 | Self-install (A3, B) on Paridyum's tenant | `account_members`; ADR-0120 D6 |
| 5 | Other tenants' customer catalogues | the first product at `franchised` |

## Open questions

1. **One account per product per customer**, or a customer profile spanning a tenant's apps?
   GLO-0003's personal account is the natural spine; decide with the first two-app customer.
2. **The default for A3** — self-install or staff-assigned?
3. **Which existing products are person-scale.** Hosting and reel have end customers who may be
   people or small companies; decide per product in its contract, not here.
4. **How the catalogue presents a business-scale app to a company** without implying the company
   becomes the tenant's account.

## Related

- **ADR-0104** · **ADR-0115** D7 · **GLO-0003** · **GLO-0005** · **RFC-0049**
- **ADR-0112** · **BET-0005** · **C-0006** (anti-goals 1 and 6; the framing table) · **C-0008** ·
  **ADR-0116** D2
- **ADR-0101** D3 · **ADR-0042** D6 · **BET-0007** · **RFC-0015** D1
- **ADR-0120** (D6 — `account_members`) · **ADR-0122** (D3) · **RFC-0056** ·
  **RFC-0058**
- The 2026-09-27 review brief (F-A to F-C) and the 2026-09-28 research record, in `docs/`
