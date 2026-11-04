---
id: ADR-0138
title: Registration opens behind a rented challenge, our rate limits, and a published abuse contact
type: adr
status: accepted
created: 2026-10-09
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform]
applies_to: ["*"]
product: [tenancy, identity, routing]
tags: [registration, abuse, rate-limit, bot-challenge, cloudflare, email-routing, takedown, rfc-0056, rfc-0072, adr-0061, adr-0129]
breaking: false
enforced_by: []
---

# ADR-0138 — Registration opens behind a rented challenge, our rate limits, and a published abuse contact

## Status

Accepted, 2026-10-09, by the operator, who chose the challenge, the abuse inbox and the limits below.
It is the record RFC-0072 §5 and RFC-0056 D4 require before self-registration at `in.paridyum.in` opens
(slice S7.R).

## Context

The portal is live on staging (RFC-0072 S7.0–S7.8), and the saga has made tenants from nothing
(X2, P5). Registration is still closed twice: by the portal instance's login policy
(`allowRegister: false`, configure-portal.py) and by the edge (`/ui/v2/login/register` answers 404).

RFC-0072 §5 opens it only when every RFC-0056 D control exists. Their state on 2026-10-09:

| RFC-0056 D | Control | State |
| --- | --- | --- |
| 1 | An address is proven before anything else exists | Exists: Zitadel's email code, and tenancy refuses every write from an unverified address |
| 2 | `max_tenants` per org | Exists (ADR-0114 D5) |
| 3 | Rate limits at the edge | **Missing** |
| 4 | A bot challenge at the edge, in its own record | **Missing**, and this is that record |
| 5 | An abuse contact for the zone, a takedown runbook, a way to stop a tenant | **Missing** |
| F | The personal-data gate | Open under ADR-0129, reviewed 2026-10-29 |

Two facts shape the choices:
- **Zitadel's login UI is not ours.** It ships with Zitadel (v4.15.3). Anything that must sit inside the
  signup form means forking and maintaining it.
- **None of our domains receives mail.** The routing table has no MX record, and `mail.paridyum.in`
  only sends (RUN-0008).

## Decision

### D1. The bot challenge is the rented tier's: a Cloudflare Managed Challenge on the signup paths

A Cloudflare WAF custom rule on the `paridyum.in` zone issues a **Managed Challenge** when the host is
`in.paridyum.in` and the path starts with `/ui/v2/login/register`. Managed Challenge is Turnstile run by
Cloudflare: it is invisible to most people and interactive only when the signals are poor. The login
UI's form submits post to the page's own URL, so the rule covers the page and its submissions alike.

- **Why the rented tier:** ADR-0061 gives absorbing abuse to the rented tier, and Cloudflare already is
  that tier. Nothing in the login, the portal or any service changes.
- **Exit (REQ-0012):** the rule is one expression, and no code depends on it. Leaving means deleting the
  rule and putting a challenge in our tier: a proof-of-work page in nginx in front of the same paths.
  Leaving loses nothing but the challenge's signal quality.
- **Not a tool registry entry.** ADR-0066's registry holds programs the estate installs. This is a
  feature of a service we already rent, recorded here.

### D2. Cloudflare zone configuration is code, applied the way DNS is

The rule (D1) and the email routing (D4) live in infra-edge as declared data, applied by a plan/apply
tool with the same contract as `dns/apply.py` (ADR-0061): plan by default, `--apply` explicit, the
token read from SOPS and never printed. Nothing is configured by hand in the dashboard **except** the
abuse inbox's destination address (D4), which Cloudflare requires its owner to verify.

The scoped token owed in infra-edge's DECISION-DEBT gains two permissions: Zone WAF edit and Email
Routing edit, on the zones they apply to.

### D3. Rate limits: a burst at our edge, and the exact hourly count where we own the code

**At our edge (edge-in-01), per visitor IP, POST only.** The IP is `CF-Connecting-IP`, which is
trustworthy because the node admits HTTPS only from Cloudflare (ADR-0061). GETs are not counted, so
reading a page never spends the allowance. nginx's slowest rate is one request a minute, so the edge
stops floods. It does not count hours.

| What | Paths | Edge allowance | Over it |
| --- | --- | --- | --- |
| Registration | `in.paridyum.in/ui/v2/login/register*` | a burst of 5, then 1 per minute | 429 |
| Address-proof attempts | `in.paridyum.in/ui/v2/login/verify*` | a burst of 10, then 1 per minute | 429 |
| Tenant creation | `paridyum.in/api/orgs/*/tenants` | a burst of 5, then 1 per minute | 429 |

The login paths were measured against the running login on 2026-10-09: both answer 200, and an
unknown path under `/ui/v2/login/` answers 404.

**The exact hourly count is the application's, where we own it.** platform-tenancy refuses a sixth
tenant request **by the same person within an hour**, as `429 rate_limited` with `Retry-After`. The
count runs over `tenant_provisionings` by `requested_by_issuer`, `requested_by_subject` and
`created_at`, and needs no new column. Registration has no hourly count of ours, because the form is
Zitadel's. Its volume is held by the challenge (D1) and the edge burst.

The operator chose the starting values on 2026-10-09: 5 an hour, 10 code attempts per 10 minutes. The
edge values are configuration, and tenancy's limit is a configuration value too.

**Per-address-domain limits** (RFC-0056 D3's other half) belong on the same tenancy count, using
`requested_by_email`'s domain, and are recorded as owed.

### D4. Abuse reports go to `abuse@` on both zones, through Cloudflare Email Routing

`abuse@paridyum.xyz` and `abuse@paridyum.in` forward, through Cloudflare Email Routing, to an inbox the
operator names. The MX and SPF records it needs are code in `dns/routing-table.yaml`. The destination
address is personal data. It is verified in the Cloudflare dashboard and **never written to a
repository**.

The contact is published:
- on every portal page, and on the app zone's 404 page;
- in `/.well-known/security.txt` on `paridyum.in` and `paridyum.xyz` (RFC 9116), as the contact for
  abuse as well as security.

### D5. Takedown is a runbook over the controls that exist, and a suspend state is owed

There is no tenant `suspended` state today (orgs have one, tenants do not). A takedown runbook,
RUN-0009, stops a tenant with what exists:
1. remove its host bindings in routing, as the saga's compensation does; the host stops resolving at
   once, and the edge cache is purged;
2. deactivate its Zitadel organization, so nobody signs in to it;
3. record the report, the action and the time; tell the reporter and the org's owner.

Reinstating reverses steps 1 and 2. A proper `suspended` state on tenants (RFC-0015 D4's single-region
stop), which the saga and every app honour, is recorded as owed. The runbook is what bridges the gap
until then.

### D6. The order of opening

Registration opens only after every item below is true and checked on the live system:

1. D3's limits answer 429 past their threshold, at the edge and in tenancy (measured with a synthetic
   client);
2. D1's challenge is served on the signup path, and not on sign-in;
3. D4's addresses deliver to the operator's inbox, and the contact is published;
4. RUN-0009 exists, and its steps 1 and 2 have been rehearsed on a synthetic tenant;
5. ADR-0129 is still in force (review 2026-10-29).

**The edge path opens first, while the policy still refuses.** A path the edge answers with 404 is
never limited or challenged, so items 1 and 2 cannot be measured on the signup path while it is closed
there. So the order is:
1. remove the edge's `closed` register path, while the portal instance's login policy still has
   `allowRegister: false` and refuses every registration;
2. measure items 1 and 2 on the real path;
3. set `allowRegister: true` (configure-portal.py, planned first).

Either closure alone keeps registration shut, so closing again is either step.

## Alternatives considered

- **A Turnstile widget inside the signup form.** Stronger binding, because the token is checked
  server-side by the form's own handler. **Rejected:** the form is Zitadel's login, so this means forking
  and maintaining it.
- **hCaptcha.** Independent of Cloudflare. **Rejected:** the same forking problem, and a second vendor.
- **Cloudflare rate limiting rules instead of nginx.** **Rejected:** the free plan allows one rule with a
  fixed window, and D3 needs three. Rate limits are control, which ADR-0061 gives to our tier.
- **An exact hourly limiter beside nginx** (a small service holding windows per IP). **Rejected by the
  operator on 2026-10-09:** a new component to run and adopt. The burst at the edge with the exact count
  in tenancy covers the same abuse.
- **Invite-only launch.** RFC-0056 considered it and the operator chose public self-service.
- **Open now and add the controls after.** **Rejected by the operator on 2026-10-09:** the signup sends
  real mail from our domain, and every control is cheaper before abuse than after.

## Consequences

**Accepted costs.**
- The challenge depends on Cloudflare's judgement. A person on a poor-reputation network may be asked to
  solve it, and this is felt most on mobile networks in region `in`.
- An IP limit punishes shared addresses (carrier NAT, offices). The starting values are generous, and a
  real complaint is the signal to raise them.
- Abuse mail depends on a forwarding service we rent. If it fails, reports bounce, and the published
  contact is wrong until it is fixed.

**Owed (recorded in DECISION-DEBT where they land):**
- per-address-domain limits on tenant creation (tenancy);
- a tenant `suspended` state honoured by the saga and every app;
- deleting unproven registrations after `unverified_expiry` (RFC-0056 E);
- the scoped Cloudflare token, with D2's two added permissions.

## References

- RFC-0072 §5 and S7.R; RFC-0056 D and F.
- ADR-0061 — the rented tier and ours; DNS as code.
- ADR-0066, REQ-0012 — adoption and exit.
- ADR-0129 — the personal-data gate.
- ADR-0114 D5 — `max_tenants`.

## Addendum — 2026-10-09: Cloudflare owns Email Routing's MX and DKIM records

D4 said the MX and SPF records Email Routing needs are code in `dns/routing-table.yaml`. Applied, that
held only for SPF. Enabling Email Routing made Cloudflare rewrite the zone's MX and DKIM records
read-only (`meta.email_routing`) and drop infra-edge's marker. So:
- **MX and DKIM are Cloudflare's**, created and locked when `cloudflare/apply.py` enables Email Routing.
  They are not in the routing table, and `dns/apply.py` leaves them alone as records it did not make.
- **SPF stays ours**, in the routing table with the marker.
- `cf-plan` reports any record Email Routing says is missing, so a zone whose records drift is still
  caught by code.

Nothing else in D4 changes. The destination address stays out of every repository.
