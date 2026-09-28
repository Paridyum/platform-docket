---
id: RFC-0055
title: Using, serving and operating are separate flows, and a client company is its own tenant
type: rfc
status: proposed
created: 2026-09-27
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, web, console]
applies_to: ["*"]
tags: [products, tenancy, consumption, franchise, reseller, accounts, business-store, docket, org, grants, rung-3, rfc-0044, rfc-0047, rfc-0049, adr-0104, adr-0112, adr-0114, adr-0115, adr-0116, adr-0120, bet-0005, c-0006, c-0008]
breaking: false
resolved_by: null
verified_by: []
enforced_by: []
sources:
  - url: https://learn.microsoft.com/en-us/partner-center/customers/gdap-introduction
    title: "Granular delegated admin privileges (GDAP) introduction — Microsoft Learn"
    retrieved: 2026-09-27
  - url: https://learn.microsoft.com/en-us/partner-center/customers/gdap-customer-terminate
    title: "Customer-Led Removal of a Granular Admin Relationship — Microsoft Learn"
    retrieved: 2026-09-27
  - url: https://quickbooks.intuit.com/learn-support/en-us/help-article/company-file/firms-accounting-quickbooks-online-accountant/L8hCIQ77L_US_en_US
    title: "Do your firm's accounting in QuickBooks Online Accountant — Intuit"
    retrieved: 2026-09-27
  - url: https://quickbooks.intuit.com/learn-support/en-us/help-article/manage-client/manage-wholesale-discount-plan-wholesale-discount/L13MoEwj6_US_en_US
    title: "Manage your ProAdvisor Preferred Pricing plan and ProAdvisor discount clients — Intuit"
    retrieved: 2026-09-27
  - url: https://help.gohighlevel.com/support/solutions/articles/155000007979-what-is-an-agency-sub-account-in-highlevel-
    title: "Agency Sub-Accounts in HighLevel Explained — HighLevel"
    retrieved: 2026-09-27
  - url: https://indiankanoon.org/doc/20425670/
    title: "Section 2, The Digital Personal Data Protection Act, 2023 — Indian Kanoon"
    retrieved: 2026-09-27
  - url: https://indiankanoon.org/doc/62814281/
    title: "Section 7, The Digital Personal Data Protection Act, 2023 — Indian Kanoon"
    retrieved: 2026-09-27
  - url: https://indiankanoon.org/doc/186118625/
    title: "Section 8, The Digital Personal Data Protection Act, 2023 — Indian Kanoon"
    retrieved: 2026-09-27
---

# RFC-0055 — Using, serving and operating are separate flows, and a client company is its own tenant

## Status

**PROPOSED, 2026-09-27. Revised 2026-09-28**, before it reached `main`. Opened on the operator's
distinction, in their words:

> *"the current flow is very janky as the tenant is getting connected and apps follow that tenant
> but if someone wants to run docket as a business and also use docket at the same time this flow
> is a bit not clear… docket being a simple app they can enable and consume under their tenant and
> a separate business store for enabling businesses."*

**What the revision changed, and why.** The first text (commit `c4ed749` on
`docs/tasks-match-2026-09-27`, and quoted in the 2026-09-27 review brief) had four verbs, a
three-rung ladder, and a client served as an account inside the provider's tenant. The review brief
checked that against the schema, the code and outside practice, and found:

- **For docket, an account can never become a tenant.** The four sealed chains refuse `UPDATE` and
  `DELETE` (`0006`), so a client's records written into a provider's tenant can never leave it.
- **The obligation belongs to the client.** Under DPDP the client company is the Data Fiduciary for its
  staff and the provider its processor (s.2, s.7(i), s.8(1), s.8(2), s.8(7)(b)).
- **An account cannot stand for a company.** It is one product instance, with no Zitadel organization
  and no branding of its own (`0012`; ADR-0047 D2).
- **"Serve as accounts — available today" was not true for docket.** docket served only `default`, and
  per-instance grants were undecided (ADR-0104 cost 4).

Outside practice converges on the rung the first text omitted: **the client in its own tenant, reached
through a grant it can see and revoke** — Microsoft GDAP; QuickBooks Online Accountant. The org model
(ADR-0113 to ADR-0122, accepted 2026-09-28) supplies the party that owns tenants, and ADR-0120 the
grant. **F1, F3 and D1 to D3 are replaced; F2, D4 and D5 stand, restated in the new terms.**

`enforced_by` is empty. The gates this depends on belong to ADR-0115, C-0006 and BET-0005, and none of
them moves here.

## The question

A tenant may want to **use** a product (docket for its own people) and **serve** or **operate** one
(serve others with it) at the same time. The model — *a tenant connects, and apps follow the tenant*
(RFC-0049) — describes use clearly and the rest poorly, and the artifacts share one verb while naming
different flows:

- **RFC-0044** — *"To sell docket — a tenant spins up under its own domain"* — is Paridyum's tenant
  selling to consuming tenants.
- **RFC-0047** — *"a partner is a tenant that deployed the `app` product"* — is renting the platform.
  Gated; design only.
- **BET-0005** — *"a founder running that product as their business"* — is the franchise.

## What is already decided, retrieved rather than invented

| Already decided | Where |
| --- | --- |
| Business data has two silos, `tenant` then `account`; **a third silo is deferred behind a named, unfired trigger**; the org is not it | **ADR-0115** D1, D9 |
| The customer of paridyum.in is an **org**, which owns tenants and holds none of their data | **ADR-0114** |
| A product instance **is an account**; `/app/<product>/<account>/…`; accounts are free to create | **ADR-0104** |
| Reaching into a tenant is a **per-tenant grant** the tenant sees and revokes — owners, providers and Paridyum alike | **ADR-0120** |
| A tenant is a set of **Releases**; apps follow the tenant | **RFC-0049** |
| Renting the platform is a stamp — **design only, not buildable** | **RFC-0047** |
| No marketplace: a marketplace is a venue for other people's software | **C-0006** framing table |
| **Catalogue is the default kind**; docket's and hiring's re-review is owed | **ADR-0112** |
| The only thing that creates tenants is the provisioning saga, run for an org; nothing runs outside an org or a tenant | **ADR-0039**, **ADR-0116** |

## Findings

### F1 — The jank is five verbs under one word

| Verb | What it actually is | Where it lives | Today |
| --- | --- | --- | --- |
| **Use** | A tenant enables a product for itself | the tenant's Releases (RFC-0049) | available |
| **Divide** | One company runs several instances of a product — divisions, jurisdictions | accounts (ADR-0104) | schema only; docket serves `default` |
| **Serve** | A provider works inside its clients' **own** tenants | a per-tenant grant (ADR-0120) | not built |
| **Sell the business** | The franchise: another business offers a product Paridyum operates | the product's `franchised` stage (BET-0005) | gated |
| **Rent the platform** | Own brand, own control plane, own tenants | a stamp (RFC-0047) | gated |

Nothing is wrong with RFC-0044, RFC-0047 or BET-0005 on their own. What was missing is the sentence
that says they are **different rows**.

### F2 — There is exactly one tenant-creator, and it is the platform

The ADR-0039 saga creates tenants, for the org that will own them (ADR-0118 D5). **A product never
creates tenants.** Docket, under any configuration, cannot.

So the operator's route 1 — *"deploy docket as a service and then create a tenant inside that"* — **is
not a product-level flow and this RFC refuses to grow one.** It is the platform-recursion shape, and it
exists only as RFC-0047, doubly gated.

### F3 — Five rungs, with different preconditions — not alternatives to choose between

1. **Use** — no gate.
2. **Divide** — per-instance grants (ADR-0120 D6).
3. **Serve** — the client's own org and tenant, plus a grant. Needs the org model, the `lib-auth` fix
   (ADR-0119 D5) and the grant mechanism.
4. **Sell the business** — ADR-0112's re-review of the product, `operated`, anti-goals 1 and 6, and
   BET-0005 clause 1.
5. **Rent the platform** — RFC-0047's gates, ADR-0115 D9, and ADR-0064.

**A separate company is never rung 2.**

## Decisions

### D1 — Using and operating are separate flows, on separate surfaces

- **Use** — enable a product under your tenant; apps follow the tenant. RFC-0049, unchanged. **The
  operator's "jank" is not this flow**; it is the absence of the others.
- **Operation** — enable a **business**. A different entitlement, granted against a different stage: a
  product at `franchised` in `product-registry.yaml`. **The business store is a section of the portal,
  over an org** (ADR-0114). Selling a business is a product reaching `franchised` — never a second
  kind of product, and never a new product.
- **The business store is not a marketplace** (C-0006). It lists **Paridyum's own operated playbooks**
  that have earned the stage, never third-party software.
- **Naming discipline.** The store is not named, scoped or built here (ADR-0103 D3): a name for
  something not operable is a commitment, and it gets its term when a product reaches `franchised`.

### D2 — A client company is its own tenant, reached by a grant

- **A provider serving clients** — an HR bureau, a CA firm, a PEO — keeps its own tenant for its own
  staff, and works inside each client's **own** tenant through a per-tenant grant the client approves,
  sees and revokes (ADR-0120).
- **A client's tenant may be created, and paid for, by the provider's org**, and transferred to the
  client's own org later; only `tenants.owner_org_id` changes (ADR-0114 D4).
- **Accounts remain right for divisions of one company** (ADR-0104). They are never right for a
  separate company, because for docket the container is final at the first record.
- **"Deploy the product, then create tenants inside it" is still refused at the product level** (F2).

### D3 — The dual role is one org

- **Using docket and running a docket business at the same time** is one org holding its own tenant
  (rung 1) and grants into its clients' tenants (rung 3).
- **Selling the business itself** (rung 4) is added once the product is franchised.
- **No second tenant and no invented account stands for the "business arm".**

### D4 — RFC-0044 is qualified, because its language was the jank

*"Docket as a service"* is **Paridyum's tenant operating the vertical for consuming tenants**. Nothing in
RFC-0044 authorises a tenant to run docket-as-a-service for others, and a qualification note on it
points here.

### D5 — What this refuses to promise

Serving clients (rung 3) waits on the grant mechanism, the org model's code and the `lib-auth` fix;
dividing (rung 2) waits on per-instance grants. Selling the business waits on the product reaching
`franchised` (for docket, ADR-0112's re-review first), anti-goal 1's operation and anti-goal 6's
renewal. Renting the platform waits on RFC-0047. Saying that plainly is the flow being clear; a
sign-up page that skips it would be the jank again.

## Costs, stated

- **The honest answer to "can I run docket as a business?" is "use it now; serving clients comes with
  grants; the rest is gated."** Less exciting than a storefront, and true.
- **A provider's clients each need a tenant**, and each tenant an owner — the provider's org at first,
  which is a transfer to plan for.
- **Two surfaces to eventually build** — enablement (apps under the tenant) and operation (business
  enablement against `franchised`) — and their difference must be legible in the portal.
- **RFC-0044 stops being quite so quotable.** The qualification makes it strictly smaller.

## Alternatives considered

1. **Route 1 as a product-level flow** — deploy a product as a service, create tenants inside.
   Rejected on F2: only the platform creates tenants.
2. **A client company as an account in the provider's tenant** (this RFC's first text). Rejected: for
   docket the container is final at the first record, the client is the fiduciary, and an account
   cannot carry a company's identity.
3. **A third tenancy column for providers.** ADR-0115 D9's one-way door, refused for the same reasons
   as RFC-0047's.
4. **Treat franchising as ordinary consumption.** It makes anti-goal 1 unenforceable.
5. **Build the business store now.** A storefront over an empty shelf (ADR-0008, ADR-0103 D3).

## Open questions

1. **The store's surface and name** — owed when a product reaches `franchised`.
2. **RFC-0047 open question 8** — whether a stamp-isolated partner engages ADR-0115 D9 — remains a
   human's reading.
3. **Docket's `franchised` candidacy** — depends on ADR-0112's owed re-review.

## What this does not decide

- The business store's design, repository, or class — nothing is built.
- Franchise mechanics — BET-0007 and RFC-0047 own those.
- Any movement of RFC-0047's gates or ADR-0115 D9.
- Docket's ADR-0112 re-review.

## Related

- **RFC-0044** — qualified by D4 · **RFC-0047** — route 1's real home · **RFC-0049** — the use flow
- **ADR-0114** (the org) · **ADR-0115** D9 · **ADR-0116** · **ADR-0120** (the grant) · **ADR-0104**
  (accounts are for divisions) · **ADR-0047** D2 (an account is not a Zitadel object)
- **BET-0005**, **C-0006** — operate-before-franchise, and the anti-goals · **ADR-0112** · **ADR-0039**
- The 2026-09-27 review brief and the 2026-09-28 research record, in `docs/`
