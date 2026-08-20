---
id: ANTI-0008
title: Rendered text in a machine contract — a sentence where an identifier belongs
type: anti-pattern
status: accepted
created: 2026-08-08
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, library, spec, sdk, web, mobile]
applies_to: ["*"]
tags: [i18n, l10n, api, events, contracts, observability, anti-pattern]
breaking: false
enforced_by: []
---

# ANTI-0008 — Rendered text in a machine contract

## The shape

A value that a machine reads, whose content is a sentence written for a person.

```json
{ "status": "Pending" }
{ "error": "You don't have permission to access this resource" }
{ "message": "Rohan created a task" }
{ "created_at": "08/08/2026 11:58 AM" }
{ "price": "₹1,000.00" }
```

Each one is a presentation decision that has escaped into data. The API, the event, the log line
and the metric are all machine contracts, and none of them is a screen.

## Why it is forbidden

Not because the product is monolingual. Because **the English becomes load-bearing.**

The sequence is always the same and it does not require anyone to be careless:

1. A response carries a readable message, because a readable message is helpful.
2. A client branches on it — a retry rule keyed on `"rate limit"`, a support tool that greps for
   `"permission"`, a dashboard grouping by `status`.
3. The string is now a contract nobody wrote down and nothing versions.
4. Translating the product breaks the contract. So it does not get translated, or it does and
   the parser silently stops matching for the locale nobody tested.

Step 4 is the cost, and it arrives years after step 1, charged to someone who did not make the
decision. That gap is the reason this is an anti-pattern rather than a preference.

The event case is worse than the API case, and worth separating. An API response can be
re-rendered on the next call. **An event is durable**: `{"message": "Rohan created a task"}` was
published with the parameters discarded, so no consumer can ever produce that sentence in another
language, and no amount of later work recovers `actor_id`. ADR-0032 decision 3 exists for this
asymmetry.

## The correct shape

An identifier the machine reads, plus the parameters a renderer needs.

```json
{ "status": "pending" }
{ "error": { "code": "AUTHORIZATION_DENIED", "params": {} } }
{ "event_type": "TodoCreated", "actor_id": "usr_789", "todo_id": "todo_456" }
{ "created_at": "2026-08-08T06:28:00Z" }
{ "amount_minor": 100000, "currency": "INR" }
```

## Where it is not this anti-pattern

Three exemptions, stated because a rule with no boundary gets applied where it does not belong and
then gets ignored where it does.

**A non-authoritative `message` beside a code.** ADR-0032 decision 2 permits it for operator
debugging. It stops being an exemption the moment anything branches on it.

**Server-rendered output with no client.** An email, an SMS, an invoice PDF, a CSV export. These
*are* the presentation layer; they are rendered by a backend because there is nowhere else to
render them. ADR-0034 decides which locale they use.

**User-generated content.** A task description a user typed is data. It is stored and returned as
authored, is never a translation key, and is never passed through a message catalogue.

## Why this is `accepted` rather than `locked`

Same reason ANTI-0004 gives. The boundary between "an identifier that happens to be an English
word" and "a sentence" is a judgement no check makes reliably: `pending` is an identifier,
`Pending` is a rendered string, and `not_found` and `"Not found"` differ by punctuation and
intent. ADR-0032 carries three checks that catch the decidable subset — enum casing, an error
schema with a message and no code, an event schema with a prose field. Locking this artifact
against those would claim the general rule is enforced when only its easiest cases are.

## Related

- ADR-0032 — the decision this is the shape of
- ADR-0033 — the monetary instance, which needed its own decision
- ADR-0034 — the exemption for server-rendered output
- GLO-0006 — why a formatted date or amount is a rendering, not a value
- ANTI-0004 — the precedent for an anti-pattern that stays `accepted`
- STD-0010 — the envelopes that make the correct shape the default
