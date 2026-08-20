---
id: ANTI-0004
title: Business logic in a shared library
type: anti-pattern
status: accepted
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [platform, service, library]
applies_to: ["platform-*", "svc-*", "lib-*"]
tags: [coupling, shared-code, domain-logic, judgement]
breaking: false
enforced_by: []
---

# ANTI-0004 — Business logic in a shared library

## Status

**ACCEPTED**, not `locked`, and deliberately so.

Whether a function is "business logic" or "a utility" is a judgement no check can make.
`is_valid_email()` is a utility. `is_eligible_for_refund()` is a domain rule wearing a
utility's clothes, and no static analysis distinguishes them. ADR-0002 reserves `locked`
for what something can mechanically reject; this is upheld by review, and `enforced_by` is
empty because pretending otherwise would be the false confidence ADR-0002 exists to
prevent.

`module.no_grabbag_names` (STD-0004) catches the crudest symptom — a crate or module called
`utils`, `common`, `helpers`, `shared`, `misc` — which is where this usually lands. That is
a proxy, not the rule.

## The shape

```
lib-core
  └── refund.rs   fn is_eligible_for_refund(invoice, customer) -> bool

svc-billing   ──depends──▶ lib-core
svc-support   ──depends──▶ lib-core
```

Two services needed the same rule, so it moved somewhere both could reach. Every step was
reasonable: duplication is bad, the rule is genuinely shared, and a library is what
libraries are for.

## Why it is forbidden

**A shared library containing a domain rule is a service boundary that was avoided rather
than drawn.**

- **Nobody owns the rule.** `lib-core` has an `owner` in its manifest, but the *refund
  policy* is owned by whoever understands refunds — and that team now changes behaviour in
  two services by editing a third repository.
- **Changing it requires a coordinated release.** Bump `lib-core`, release it, bump both
  consumers, release both. That is a synchronised deploy across three repositories to change
  one business rule, which is the cost ANTI-0002 warns about arriving by a different route.
- **Version skew becomes behaviour skew.** `svc-billing` on `lib-core` 1.2 and `svc-support`
  on 1.3 now *disagree about who gets a refund*. Two answers to one business question, and
  no error anywhere — the worst possible failure, because it is invisible and it is about
  money.
- **The library accretes.** Once one domain rule lives there, the next has a precedent. This
  is how `lib-core` becomes a distributed monolith's shared kernel, depended on by
  everything and ownable by no one.

## What to do instead

**Whoever owns the decision owns the code that makes it. Others ask.**

```
svc-support  ──GET /v1/invoices/{id}/refund-eligibility──▶  svc-billing
```

One answer, one owner, one deploy to change it.

**Duplication is often the cheaper mistake.** Two services each having their own thirty-line
eligibility check is worse-looking and better-behaved than one shared library, because each
can change independently and neither can silently disagree with the other. The instinct that
duplication must always be removed is the instinct that produces this anti-pattern.

**What legitimately belongs in a shared library:** things with no domain opinion at all —
serialisation helpers, a tracing subscriber setup, an error type, a retry wrapper, generated
API clients. The test: *if the business changed its mind, would this file need editing?* If
yes, it is a domain rule and it belongs to a service.

## How to spot it in review

Ask of anything in a `lib-*` crate:

1. Does it encode a decision the business could change? → domain rule.
2. Does its name contain a domain noun — `invoice`, `refund`, `customer`, `pricing`? →
   almost certainly a domain rule.
3. Would two services on different versions of it behave differently in a way a user could
   observe? → domain rule, and the dangerous kind.
4. Is it in a module called `utils`, `common`, `helpers` or `shared`? → it was not
   classified at all; classify it now.

## Related

- STD-0004 — code organisation by responsibility; `module.no_grabbag_names` is the proxy check
- C-0002 — `lib-*` publishes library targets only; no `lib-common`, no `lib-utils`
- ANTI-0002 — the coordinated release this produces, arriving by a different route
- ANTI-0006 — the other shape a missing boundary takes
