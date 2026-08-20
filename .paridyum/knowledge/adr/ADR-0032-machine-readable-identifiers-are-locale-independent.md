---
id: ADR-0032
title: Machine-readable identifiers are locale-independent; rendered text is presentation
type: adr
status: accepted
created: 2026-08-08
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, library, spec, sdk, web, mobile]
applies_to: ["*"]
tags: [i18n, l10n, api, events, errors, contracts, observability]
breaking: false
verified_by:
  - TST-0059
enforced_by:
  - check: openapi.enum_values_are_identifiers
    severity: error
    rationale: >
      An enum is the case where the collapse is invisible in review. `"Pending"` and
      `"pending"` differ by one character and by whether every consumer in every language
      must now special-case a display string, and no reviewer catches it twice.
  - check: openapi.error_responses_declare_a_code
    severity: error
    rationale: >
      An error with a message and no code forces the client to parse English to decide
      behaviour. That is the specific defect this ADR exists to prevent, and it is the one
      that is cheapest to reject at the schema and most expensive to remove after SDKs ship.
  - check: domain.events_carry_no_rendered_text
    severity: error
    rationale: >
      An event is durable. A rendered sentence in an event body is wrong in every language
      the consumer did not have when it was published, and unlike an API response it cannot
      be re-rendered later — the parameters that would have allowed it were discarded at
      publish time.
---

# ADR-0032 — Machine-readable identifiers are locale-independent

## Status

**ACCEPTED** as of 2026-08-08, with three declared checks.

Created `accepted` rather than `locked`, per the `new-decision` skill §5b: the ADR-0025
checkpoint hook commits mid-authoring, and ADR-0029 was frozen before its author had finished
counting. It locks in a separate commit once the checks it names have been reviewed.

## Context

Paridyum has no user-facing surface yet (REQ-0009 records why), and that is exactly the moment
this decision is cheap. The pressure to decide now is not the frontend — it is that every
interface a service publishes outlives the service, and three of them already have a governing
artifact: OpenAPI documents under `spec-` (class-registry.yaml, RFC-0004), events under REQ-0005,
and generated SDKs under `sdk-` where C-0004 forbids hand-editing the output. Two of those three
are deferred until their trigger fires, so the governance is assigned rather than active — an
argument for deciding this now, not against.

The failure this prevents is not "the product is only in English". It is subtler and it is
structural: **English leaks out of the presentation layer and becomes load-bearing data.** Once
a client branches on the text of an error message, or a dashboard aggregates on a rendered
sentence, or an event consumer parses a name out of `"Rohan created a task"`, the English string
is a contract. Translating the product then breaks the contract, so the product does not get
translated — or it does, and the parsers break in production for the locale nobody tested.

REQ-0009 covers the presentation half and is deferred until a framework exists. It cannot cover
this half, because this half is not about screens. `svc-*` and `platform-*` are `stacks: [rust]`
and will publish contracts long before the first component is written.

There is a real competing pressure, and pretending otherwise would make this a rationalisation.
Returning a code and no message pushes work onto every consumer, including consumers we do not
own and internal tools where nobody will ever build a message catalogue. A pure-code API is
genuinely worse to debug from a terminal. The decision below is therefore not "never send
English" — it is about which of the two is *authoritative*.

## Decision

**1. Every value a machine branches on is an ASCII identifier chosen by us, never a phrase in
any natural language.** This covers enum values, status values, error codes, event type names,
metric names, log event names, feature flags, permission strings and configuration keys.

```
pending          not  "Pending"    nor  "Ausstehend"
ORDER_CREATED    not  "Order created successfully!"
TodoAssigned     not  "Rohan assigned you a task"
auth.login.success  not  a counter named from the success banner
```

Those shapes are fixed elsewhere and this ADR does not re-decide them. Event type names in
particular are **C-0005's**: `<Subject><PastTenseVerb>` in PascalCase, enforced on every declared
event by `manifest.schema.json`'s `^[A-Z][A-Za-z0-9]*$`. A dotted lowercase event name is drift,
and RFC-0010 already lists it as such.

**2. An error carries a code and machine-readable parameters. A message is optional and never
authoritative.**

```json
{
  "error": {
    "code": "PASSWORD_TOO_SHORT",
    "params": { "minimum": 8 }
  }
}
```

A human-readable `message` MAY accompany it for operator debugging. It is presentation data: no
client may branch on it, no test may assert on its text, and it may change without a version
bump. The parameters are what makes the client's own rendering possible, and omitting them is
the same defect as omitting the code — `PASSWORD_TOO_SHORT` with no `minimum` cannot be rendered
in any language without the client hardcoding the number we already knew.

**3. An event carries its type and its parameters, never a rendered sentence.**

```json
{ "event_type": "TodoAssigned", "actor_id": "usr_789", "todo_id": "todo_456" }
```

**4. Schema identifiers are English and stay English.** Table names, column names, API field
names, crate names and event type names are not translated for anybody, ever. C-0005 already
fixes the shape of each; this states the part C-0005 did not have to: a Spanish-speaking consumer gets
`user_name`, not `nombre_usuario`. Localising an identifier does not help a translator and
destroys every consumer.

**5. Logs and metrics are locale-neutral.** A structured event with a stable name, never a
sentence. A metric whose name is derived from user-facing text becomes unqueryable the moment
that text is translated, and the failure is silent — the old series simply stops receiving
points.

**6. User-generated content is exempt and is not translated.** A task description a user typed
is data, stored and returned exactly as authored. Running it through a message catalogue is a
category error; it is neither a key nor a translatable string.

## Consequences

**Accepted costs**

1. **Every consumer needs a catalogue, including internal ones.** A CLI, an admin tool and a
   support script now each need a way to turn `AUTHORIZATION_DENIED` into a sentence, and the
   first few will hardcode an English map. That is acceptable — a hardcoded map in a consumer is
   local and removable; a rendered string in a contract is neither.

2. **Debugging by eye gets worse before it gets better.** A response body of codes reads less
   well in a terminal than a sentence. The optional non-authoritative `message` is the
   mitigation and it is deliberately weak, because a message that clients are allowed to depend
   on is not optional in practice.

3. **A code is a permanent identifier.** `PASSWORD_TOO_SHORT` cannot be renamed once published,
   for the same reason an artifact id cannot. We are trading a translation problem for a naming
   problem, and naming problems are the ones this organisation already has machinery for
   (C-0005).

4. **The parameters are a second contract.** `params` is now part of the API surface, versioned
   under REQ-0010 like everything else. An error whose parameters change is a breaking change,
   which is stricter than most organisations treat error bodies.

**Gained**

- The API is programmatically consumable, which was going to be required anyway for `sdk-`.
- Adding a locale becomes a presentation change with no backend work. REQ-0009's argument — that
  retrofitting is a rewrite of the view layer — stops being true of the core.
- Logs and metrics stay queryable across a translation, which is the failure mode that is
  hardest to detect after the fact because nothing errors.

**Rejected alternatives**

- **Backend renders with the user's locale, using `Accept-Language`.** Rejected as the default.
  It makes the response depend on a request header, so the same resource has different bodies
  for different callers — which breaks caching, makes contract tests locale-dependent, and puts
  a message catalogue in every Rust service. ADR-0034 keeps `Accept-Language` as a hint for the
  cases where the backend genuinely does render, which is the next alternative.

- **No human-readable message anywhere.** Rejected as overreach. Server-rendered documents,
  emails and notifications are generated by the backend with no client to render them, and
  pretending otherwise would force a real case to violate a locked rule on day one. STD-0010 §7
  names those boundaries explicitly rather than leaving them to be argued case by case.

- **A `governance/internationalization/` tree with its own `I18N-nnn` control ids.** Rejected on
  the same ground ADR-0022 was written on: a rule outside the artifact registry has no schema, no
  directory the gate scans, and no path into `enforced_by`. It would be unenforced prose in a
  repository whose founding argument is that unenforced prose is a suggestion.

## Related

- GLO-0006 — the four fields, and why they are four
- ADR-0033 — the monetary case, which needed its own decision
- ADR-0034 — how a locale is chosen when the backend does render
- ANTI-0008 — the shape this forbids, stated as a shape
- STD-0010 — the concrete envelopes and column conventions
- REQ-0005, REQ-0010 — event and API evolution, which now govern `code` and `params`
- REQ-0009 — the presentation half, deferred
- C-0005 — identifier naming, including PascalCase event types. This extends it and must never
  contradict it; a dotted lowercase event name is drift, not an amendment (RFC-0010)
