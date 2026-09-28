---
id: ADR-0034
title: Locale is resolved through the tenancy chain and is always an explicit declaration
type: adr
status: accepted
created: 2026-08-08
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, web, mobile, sdk]
applies_to: ["*"]
tags: [i18n, locale, tenancy, nested-tenancy, operator-scope, resolution]
breaking: false
verified_by:
  - TST-0060
enforced_by:
  - check: locale.no_ambient_locale_lookup
    severity: error
    rationale: >
      An ambient lookup — `LANG`, `LC_ALL`, a system-default locale constructor — is the
      GLO-0005 hazard in a new costume: the resolved value comes from the absence of a
      decision rather than from a claim, so it is whatever the container image happened to
      ship. It works on every developer machine and renders the wrong conventions in
      production, which is the ordering of symptoms that gets a defect shipped.
  - check: locale.formatting_names_an_explicit_locale
    severity: error
    rationale: >
      A formatting call with no locale argument silently adopts one, and this ADR's entire
      subject is that the adopted value is never the right one. Rejecting the call shape is
      the only point at which the mistake is visible, because the output is a plausible date
      either way.
---

# ADR-0034 — Locale is resolved, never inferred

> **Superseded in part, 2026-09-28 — ADR-0113, ADR-0122.** **D1's chain splits in two** (ADR-0122 D5): the portal
> resolves request → member → **org default** → platform, and a tenant resolves request → user →
> account → tenant → platform. The level this ADR calls the default of *"who pays us"* is the
> **org's** default: read only by the portal, and used once to **seed** a new tenant's declarations.
> No tenant request reads its org's settings.

## Status

**ACCEPTED** as of 2026-08-08, with two declared checks and one test type. Created `accepted`
per the `new-decision` skill §5b; it locks in a separate commit.

## Context

ADR-0032 keeps rendered text out of machine contracts, which removes the locale question from
most of the platform. It does not remove it everywhere: an email, an SMS, a push notification, an
invoice PDF and a CSV export are all produced by a backend with no client to render them. For
those, some service must pick a locale, and there is no obvious answer to pick.

There is no obvious answer because Paridyum is nested (GLO-0004). A conventional product has two
levels — application default and user preference — and the industry's stock advice assumes them.
Paridyum has an operator who pays us, a tenant that is a business, an account that is one of that
business's customers, and a user inside it. Every one of those can legitimately carry a locale,
and GLO-0002 already makes the middle levels load-bearing rather than theoretical: a tenant is
"the unit of isolation, of branding, of custom domain, and of billing", so its product is branded
and localised as *theirs*, not as ours. Not "white-label" — C-0006 lists that framing among the
ones that are actively wrong, because it describes a billing arrangement rather than a capability.

Left undecided, each service resolves it differently, and the divergence surfaces as a user who
gets the interface in one language and the receipt for the same action in another.

## Decision

**1. Resolution walks from most specific to least, and the first explicit declaration wins.**

```
request override        an explicit, audited, per-request parameter
      ↓
user preference         the persisted setting on the acting user
      ↓
account default         GLO-0003 — the tenant's customer
      ↓
tenant default          GLO-0002 — the business.  OUTER silo
      ↓
operator default        GLO-0001 — who pays us
      ↓
platform default        declared in configuration, never compiled in
```

Applied independently to each of GLO-0006's four fields. A user who has set a timezone and not a
language resolves timezone at level 2 and language at level 3 — or level 4, if the account has
not set one either. The fields do not travel together and a level does not "win" wholesale.

**2. The platform default is a declaration, not a fallback.** It is a configured value that a
deployment must state. It is not the process locale, not `LANG`, not `LC_ALL`, not the result of a
system-default constructor, and not a literal in a binary.

This is GLO-0005's rule transplanted, and the transplant is the load-bearing part of this ADR.
GLO-0005 says operator scope must be a positive claim because *breadth expressed as absence is
granted by every bug that loses a value*. Locale fails the same way and in the same direction: a
dropped session field, an uninitialised struct, a middleware ordering bug and an empty header all
produce "no locale", and if that resolves to whatever the container shipped, the code path that
handles the failure is the code path that is never tested. The output is a plausible date in the
wrong convention. Nothing errors.

**3. `Accept-Language` is a hint and never a preference.** It may seed resolution for a caller
with no persisted setting — an anonymous visitor, a first request before sign-in. It never
overrides a stored user preference and is never written back as one without an explicit act by
the user. It is a property of a browser profile, frequently a shared machine, and often a value
the person never chose.

**4. A locale in a token is a cache, not a source.** A `locale` claim in a JWT may be used for
rendering within the request. It is never authoritative for anything durable — a preference
change must take effect before the token expires, and a long-lived token would otherwise pin a
user to a setting they have already changed. Anything persisted, and anything sent
out-of-band later, reads the setting.

**5. A reader's locale renders; it never reinterprets stored data.** When a principal with
operator scope (GLO-0005) reads across accounts — which is the only breadth that exists, since
GLO-0005 is explicit that "there is no scope that spans tenants":

- **The reader's locale** decides language, date convention, number grouping and sort order.
- **The record's own fields** decide what the data *is*. An invoice's `currency` is ADR-0033 §5
  and does not change because a reader with a different default opened it. A stored instant is
  UTC and is converted for display, not re-anchored.

Stated because the collapse is tempting and one-directional: it is easy to write "render this
tenant's data in this tenant's locale" and thereby show an operator a console in a language they
do not read; it is equally easy to write "render everything in the reader's locale" and thereby
redenominate an invoice.

**6. Out-of-band messages resolve at send time against the recipient, not the actor.** An email
about a task assignment is rendered in the *recipient's* resolved locale and timezone. The actor
who triggered it is irrelevant, and the locale of the request that caused it is a hint about
nobody.

**7. Every tenant declares its supported set.** A tenant carries `supported_locales`, and
resolution never yields a value outside it — a user preference for an unsupported locale falls
through to the next level rather than rendering half-translated. Falling back is per-field and
deterministic: `de-AT` → `de` → the tenant default, never to an arbitrary sibling.

## Consequences

**Accepted costs**

1. **Six levels is more machinery than most products need**, and the middle three — account,
   tenant and operator — will hold no value for a long time. Load-bearing by design and
   unpopulated in practice until the first tenant configures its own branding; that is not a
   contradiction but it is worth stating plainly, because the cost today is a resolution function
   nobody can read at a glance and a test matrix with mostly-empty levels. Accepted because the
   alternative is discovering the levels one at a time, each as a schema change to a live tenant
   table.

2. **A preference change must invalidate rendering caches.** Decision 4 means the authoritative
   value is behind a read, so anything that caches a rendered artifact has to key on the resolved
   locale. Straightforward and easy to forget.

3. **`supported_locales` can strand a user.** A user whose language is genuinely unsupported gets
   the tenant default, which is correct and will still read as a bug to them. The alternative —
   rendering an unsupported locale with most keys missing — is worse and is what REQ-0009 §7 calls
   out.

4. **All three of the rules that matter most are unenforceable statically.** The order in decision
   1, the split in decision 5 and the recipient rule in decision 6 are runtime behaviour. TST-0060
   is the answer and it is `deferred`, so until a service exists these hold by compliance rather
   than by machinery — the state CLAUDE.md describes as normal here and asks to be treated as
   binding anyway.

**Gained**

- One resolution order across every service, so the interface and the receipt agree.
- A tenant-branded product works without a fork: its locale, brand and currency are
  configuration, which is what GLO-0002 already makes a tenant the unit of.
- The absence failure — the one that renders plausibly wrong output and never errors — is
  converted into a startup-time configuration error.

**Rejected alternatives**

- **Two levels: platform default and user preference.** The conventional model, and it cannot
  express a white-label tenant whose product is localised as theirs. Adding the middle levels
  later means a schema change to tables that by then hold every tenant.

- **Derive everything from one `region` field.** Rejected in GLO-0006 with the counterexamples:
  no two of the four fields covary reliably, and the derivation is right for the modal user and
  silently wrong for everyone else.

- **Trust `Accept-Language` as the preference.** Cheap, requires no storage, and wrong for shared
  machines, corporate images and anybody whose browser language is not their reading language. It
  also makes the response vary by header, which ADR-0032 already rejected for caching reasons.

- **Resolve once at the edge and pass the result down.** Attractive, and it breaks decision 6: a
  notification produced asynchronously has no request context, and the edge-resolved value belongs
  to the actor rather than the recipient. Passing the *recipient id* and resolving at send time is
  the shape that survives.

## Related

- GLO-0006 — the four fields this resolves
- GLO-0004 — the chain the levels come from
- GLO-0005 — operator scope, and the absence hazard decision 2 transplants
- ADR-0032 — why most of the platform never asks this question
- ADR-0033 — currency as a property of the record, which decision 5 depends on
- STD-0010 — where the resolved values are stored and how they are carried
- REQ-0009 — the presentation half, deferred
- GLO-0002 — a tenant is the unit of branding, which is what makes the middle levels real
