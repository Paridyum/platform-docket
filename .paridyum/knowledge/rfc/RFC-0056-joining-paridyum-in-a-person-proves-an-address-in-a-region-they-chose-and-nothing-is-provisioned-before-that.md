---
id: RFC-0056
title: Joining paridyum.in — a person proves an address in a region they chose, and nothing is provisioned before that
type: rfc
status: accepted
created: 2026-09-28
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, web]
applies_to: ["*"]
product: [paridyum, tenancy, identity]
tags: [signup, self-serve, onboarding, verification, abuse, invitations, org, team, region, transfer, user-flows, plan-tenant-flow, adr-0100, adr-0039, adr-0071, adr-0064, req-0003, rfc-0051, rfc-0015]
breaking: false
enforced_by: []
---

# RFC-0056 — Joining paridyum.in: a person proves an address in a region they chose, and nothing is provisioned before that

## Status

**ACCEPTED** 2026-09-29 by the operator ("full portal, start now", and "accept all recommendations"
for the five open questions, each answered below). Proposed 2026-09-28, with the org model
(ADR-0113 to ADR-0122).
On these screens a tenant is called a *business* (ADR-0113 D1); in this RFC it is a tenant.

## Problem

**PLAN-TENANT-FLOW §2 records an artifact nobody has written:**

> *"the signup flow D5 authorises is 'a public route that starts `requested → provisioning`' in
> ADR-0100's own words, and no artifact designs it. G2 authorises it; something still has to specify
> the abuse controls, the verification step and what a half-finished signup leaves behind."*

**The operator asked for the flows themselves:** *"people join as individuals or as an org/team
(research all possible user flows and correct me here)."*

**The org model (ADR-0114, ADR-0117) changes what signup creates.** It creates:

- a person's credential, in the region they chose;
- a personal org;
- only later, and only on request, a tenant.

The tenant flow as written provisions a tenant at signup; this RFC separates the two.

**If nothing is decided, signup gets built from the plan's one sentence.** The plan's C1 — *"verified
email → account → the control plane"* — would then carry an unverified-address hole, an open
provisioning endpoint and no expiry for abandoned registrations.

## Constraints

**The org model**

- **ADR-0114:** personal and team orgs are one model, and membership authorises the portal only.
  `max_tenants` is per org and counted in the control plane.
- **ADR-0117:**
  - D1 — an org's region is chosen explicitly and is immutable;
  - D6 — a credential lives in one region, and sign-in starts with the region;
  - D7 — admitting-side records.
- **ADR-0119:** a portal instance per region, and a tenant's customers never sign up at
  paridyum.in.

**Region and placement**

- **RFC-0051:** the jurisdiction screen — labelled by regime, ordered by `suggest_for`, nothing
  preselected, never inferred.
- **RFC-0015 D5:** *"Placement is never inferred from the address a signup arrives from."*

**Provisioning**

- **ADR-0039:** the provisioning saga and its lifecycle. It runs **only** when a tenant is
  requested — never at signup.
- **ADR-0101 D3:** the `max_tenants` refusal belongs to the saga, not the surface.

**Personal data**

- **ADR-0071 §3 / ADR-0064 D3:** no real person is stored until a restore has been rehearsed (T-G.7).
  **A signup stores a real person's address.**
- **REQ-0003:** personal data is classified, minimised, held on a retention schedule and deletable.

**Copy, cost and locale**

- **STD-0021:** the marketing surface may not manufacture credibility. Signup copy promises only what
  exists.
- **BET-0006** cost 4 (self-serve removes the human who would catch a bad fit) and **C-0006**
  anti-goal 5 (no enterprise before self-serve works).
- **ADR-0034** D3: `Accept-Language` is a hint. **ADR-0122:** a member's preferences are the
  portal chain's second level.

## Proposal

### A. Every flow, and the correct shape of each

| # | Flow | Shape |
| --- | --- | --- |
| U1 | **An individual joins** | paridyum.in → **choose a region** (RFC-0051's screen) → email plus passkey or password at that region's portal instance → **prove the address** → a **personal org** is minted (ADR-0114 D2) → the portal home, with no tenant yet |
| U2 | **A team joins** | U1, then **create a team org** (name, handle). The creator is `owner`. Members are invited by address (C). A personal org can become a team by adding members instead (GLO-0012) |
| U3 | **Create the first tenant** | From an org: **create a tenant** — its own region (RFC-0051, independent of the org's), name, slug (its default host), content locales, locale, timezone, currency (ADR-0122 D2) → the saga runs (control plane + target region) → the default host serves → the owner's first grant is issued (ADR-0120 D2) → the owner enables apps (RFC-0057) → optionally adds a domain (ADR-0121 D4) |
| U4 | **Give a teammate access to a tenant** | An org `owner` or `admin` issues a per-tenant grant (ADR-0120 D2). **Org membership alone reaches no tenant** |
| U5 | **One person, several orgs** | An org switcher. One credential, in the person's region; an org in another region holds them as a linked user (ADR-0117 D7) |
| U6 | **A provider serves a client** | The client's tenant approves the provider's grant request. Or the provider's org creates the client's tenant, and pays for it, and transfers it later (U10) |
| U7 | **Paridyum support** | U6's request/approve, from the staff instance (ADR-0120 D2) |
| U8 | **More than three tenants** | `max_tenants` refuses in the saga, and the portal says why. More is a support conversation (PLAN-TENANT-FLOW D6) |
| U9 | **A tenant's customers** | They sign up on the **tenant's** host, in its tenant instance. Never at paridyum.in |
| U10 | **Transfer a tenant to another org** | The owning org offers; the receiving org's `owner` accepts; `tenants.owner_org_id` changes (ADR-0114 D4). Grants from the old org's members are listed for the new owner to keep or revoke, defaulting to **revoke** |
| U11 | **Leave or delete** | A member leaving an org revokes every grant that membership produced (ADR-0120 D5). An org is deleted only when it owns no tenant. A person's deletion removes their credential; admitting sides' audit history stays under those sides' retention (ADR-0117 D7) |
| U12 | **Move an org to another region** | Not self-serve. It is ADR-0041's contract amendment plus a migration of the org's people. The org's tenants do not move — they have their own regions |

**The correction to the operator's model, in one line:** a person joins, a personal org exists at
once, and a tenant is created deliberately afterwards, with its own region and languages. Signing
up never creates a tenant.

### B. Nothing is provisioned before the address is proven

The states a signup passes through:

| State | What exists | How long |
| --- | --- | --- |
| `registered` | a credential in the chosen region's portal instance, address unproven. No org, no row anywhere else | expires, and is deleted, after `unverified_expiry` (proposed 7 days) — REQ-0003 retention |
| `verified` | the credential, plus a personal org — its registry row and its org-scoped rows | kept; it costs nothing |
| `tenant_requested` → ADR-0039's lifecycle | a saga run | ADR-0039 |

**A `registered` person can do nothing except prove their address, or leave.** No invitation they
send is delivered, no org they name is created, and no handle is reserved.

### C. Invitations

- **An invitation names an address and a role**, and expires after `invite_expiry` (proposed 14
  days).
- **It is sent from the inviting org's region**, in the invitee's language if the inviter declared
  it, and otherwise in the org's default (ADR-0034 D6's recipient rule, with the best knowledge the
  sender has).
- **An invitee with no credential signs up through U1**, in a region *they* choose. An invitation
  never chooses a person's region for them.
- **An invitee in another region is admitted as a linked user** (ADR-0117 D7).

### D. Abuse controls, all failing closed

1. **An address is proven before anything else exists** (B).
2. **`max_tenants` per org** is the ceiling on provisioning (ADR-0114 D5), checked by the saga in
   the control plane.
3. **Rate limits at the edge** on registration, proof attempts and tenant creation — per source
   network and per address domain. **This is abuse control, not placement.** It decides whether a
   request is served, never where anything lives (RFC-0015 D5 is about placement).
4. **A bot challenge at the edge** on registration. The edge is rented (ADR-0061), and choosing a
   challenge vendor is a tool adoption that owes its own record (ADR-0066) and REQ-0012's
   exitability test.
5. **Default hosts are a public abuse surface.** A public-suffix app zone stops cookie attacks
   between tenants; it does not stop someone publishing a phishing page on
   `<slug>.<app zone>`. So:
   - an abuse contact is published for the zone;
   - a takedown runbook exists before the first non-Paridyum tenant is served;
   - a tenant may be suspended — RFC-0015 D4's single-region stop.
6. **No money moves at signup.** Collecting payment is KYC-gated and later (BET-0007 cost 2).
7. **Disposable-address policy** is the operator's choice (open question 3). The default is to allow
   them and rely on items 1–5, because blocking lists are guesses that exclude real people.

### E. What a half-finished signup leaves behind

| Abandoned at | What remains | Clean-up |
| --- | --- | --- |
| `registered` | an unproven credential | deleted at `unverified_expiry` |
| `verified`, no tenant | a personal org | none needed; it is the ordinary resting state |
| a saga that fails | a tenant in `failed`, visible to the org (ADR-0039 never hard-deletes) | retry, or retire it from the portal |
| `awaiting_domain` | an active tenant on its default host | nothing; the own-domain step never blocks activation (ADR-0121 D4) |
| an unaccepted invitation | a pending invitation | expires at `invite_expiry` |

### F. The personal-data gate comes first

**Signup stores real people's addresses**, so the public route opens only after one of:

- **T-G.7** — a rehearsed off-host restore (ADR-0071 §3, ADR-0064 D3);
- **a risk-acceptance ADR**, signed by the operator, with T-G.7 as its closing trigger — not yet
  written.

**Until then, signup is exercised against synthetic identities only.**

### G. Honest copy

- The region screen shows RFC-0051's `coming soon` and `on request` exactly as the registry records
  them.
- Nothing claims availability, languages or apps that do not exist on the day (STD-0021).
- The portal's own interface languages are the portal's software locales (ADR-0122 D1).

## Alternatives considered

- **Provision a tenant at signup, in one step.** It is the shortest path to "a running business"
  (C-0001's *"in minutes"*). **Rejected:**
  - the tenant's region and languages are permanent or gated choices that deserve their own screen;
  - an unverified signup must not reach the saga.

  U3 is one screen later, not one day.
- **Org-first signup** — every person creates or joins a team before anything else, as in Slack.
  **Rejected:** individuals are the first case (BET-0006's first customer), and a personal org gives them a
  team of one for free (the Vercel model).
- **Invite-only launch.** Safer, and it keeps a human in the loop for fit (BET-0006 cost 4). The
  operator chose public self-service; this RFC gates it on F instead.
- **Infer the region from the visitor's address** and preselect it. Refused by RFC-0015 D5 and
  RFC-0051.

## Consequences

- **The tenant flow's C1/C2 are replaced** by U1–U3 and D's controls. PLAN-TENANT-FLOW D5 and D6
  carry dated notes.
- **The portal gains two screens that did not exist:** the region choice and the tenant-creation
  form. The latter's five choices come from ADR-0122 D2.
- **Two expiry jobs** (unverified registrations, invitations) and a **takedown runbook** — operational
  work that exists on day one.
- **The rate limits and the bot challenge are edge configuration**, reviewable as data in the edge
  repository, not code in the portal.

## Rollout

| Slice | What lands | Depends on |
| --- | --- | --- |
| 0 | This RFC accepted; the org ADRs landed | ADR-0114, ADR-0117 |
| 1 | `in`'s portal instance; U1 against synthetic identities; B's states and expiry | ADR-0119, ADR-0117 D10 |
| 2 | Team orgs, invitations (U2, C) | 1 |
| 3 | Tenant creation (U3) through the saga, with `max_tenants` | the saga API; `platform-entitlement` |
| 4 | Edge rate limits and bot challenge (D3, D4); takedown runbook (D5) | the challenge vendor's adoption record |
| 5 | **The public route opens** | F — T-G.7, or the signed risk ADR |
| 6 | Transfer (U10), leave and delete (U11) | 3 |

## Open questions

1. **`unverified_expiry` and `invite_expiry`.** Proposed 7 and 14 days. They belong in `args`, not
   prose.
   **Answered 2026-09-29: 7 and 14 days, as proposed.**
2. **Passkey-first or password-first** at the portal instance. Both are Zitadel features; the
   choice is product and support policy.
   **Answered 2026-09-29: passkey-first, with a password as the fallback.**
3. **Disposable addresses.** Allow them (the default here) or refuse them.
   **Answered 2026-09-29: allowed; the §D abuse controls fail closed.**
4. **The bot-challenge vendor**, and its exitability.
   **Answered 2026-09-29: whichever vendor, behind an interface, with no account lock-in, so it can be exited.**
5. **The handle namespace.** Are org handles unique per region or globally? Globally needs a
   control-plane index of handles — acceptable only if a handle is never personal data. That is not
   true for a personal org named after its person, so the proposal is **unique per region**.
   **Answered 2026-09-29: unique per region, as proposed.**

## Related

- **PLAN-TENANT-FLOW** §2, D5, D6 · **ADR-0100** D5 · **ADR-0039** · **ADR-0101** D3
- **ADR-0114** · **ADR-0117** · **ADR-0119** · **ADR-0120** ·
  **ADR-0121** · **ADR-0122** · **RFC-0057**
- **RFC-0051** · **RFC-0015** D4, D5 · **ADR-0071** §3 · **ADR-0064** D3 · **REQ-0003** · **STD-0021**
- **ADR-0061** (the edge) · **ADR-0066** (a tool is registry data) · **REQ-0012** (exitability)
- **BET-0006** cost 4 · **C-0006** anti-goal 5 · **BET-0007** cost 2
