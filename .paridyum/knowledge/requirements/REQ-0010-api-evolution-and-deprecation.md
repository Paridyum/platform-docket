---
id: REQ-0010
title: A published API evolves compatibly and is retired on an announced schedule
type: requirement
status: deferred
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, spec]
applies_to: ["*"]
tags: [api, versioning, deprecation, idempotency, contracts]
breaking: false
trigger: >
  The first API is consumed by something the organisation does not deploy — another
  team's service, a browser, a mobile app, or an external client.
review_date: 2027-02-01
enforced_by: []
---

# REQ-0010 — A published API evolves compatibly and is retired on an announced schedule

## Status

**DEFERRED** as of 2026-08-07. No API exists and nothing consumes one.

The trigger is the first consumer you do not control. Until then an API is an internal
function call with extra steps and can change freely; after, every change is a change to
somebody else's system.

## Requirement

**A published API shall have a machine-readable contract, a compatibility policy enforced
in CI, safe retry semantics, and a deprecation path with an announced end date.**

## What already exists

C-0002 settles **shape**: plural noun resources, never verbs, `kebab-case` path segments,
`snake_case` JSON fields, root versioning `/v1/`, and a reserved unversioned set
(`/health`, `/ready`, `/metrics`, `/docs`). Two `openapi.*` checks are declared.

STD-0002 settles **provenance**: `API.md` is generated from `specs/openapi.yaml` and
hand-editing it is a CI failure — which is what stops documentation drifting from the
contract.

`manifest.yaml` records `exposes.http.openapi`, so the spec's location is already
discoverable.

The gap is **evolution**: nothing says what may change, how a consumer is warned, or when
the old version dies.

## Numbered items

1. **The spec is the source of truth**, and it is written before the handler. An OpenAPI
   document generated from an implementation documents what was built rather than what was
   agreed.
2. **Breaking-change detection in CI.** Diff the spec against the previous release and fail
   on an incompatible change — removed field, narrowed type, new required parameter,
   changed enum semantics. Without this, "we don't break APIs" is a policy nobody can
   verify.
3. **Additive-only within a major version.** New fields optional, new enum values tolerated
   by clients. Clients must ignore unknown fields, and that expectation must be stated,
   because a strict client turns every additive change into a breaking one.
4. **Deprecation has a schedule.** `Deprecation` and `Sunset` headers, an announced removal
   date, and no-new-consumers enforcement in between. A deprecation with no end date is a
   permanent second version to maintain.
5. **Idempotency keys on unsafe operations.** Any `POST` that moves money, creates a
   resource, or triggers a side effect needs a client-supplied idempotency key. Retries are
   not optional — networks retry whether or not the API is ready for it — so this is what
   stands between a timeout and a double charge.
6. **Timeouts, request size limits and pagination** stated and enforced server-side. An
   unbounded list endpoint is an outage waiting for a large tenant.
7. **RFC 3339 timestamps in UTC**, always, in every field. C-0002 mandates it in the
   database; the API layer is where it usually gets re-formatted into something local and
   ambiguous.
8. **Partial-failure semantics defined** for batch and bulk endpoints — all-or-nothing, or
   per-item results. Undefined is the worst option and the default.
9. **Internal versus public tiers.** An internal API may evolve faster; a public one may
   not. The tier must be declared, because the cost of the policy differs by an order of
   magnitude.
10. **Error format is uniform** across every service — a stable machine-readable code, a
    human message, and no internal detail leaked.

## Why this is not startable today

Every item governs a contract with a consumer, and there is neither. Item 2 needs a
previous release to diff against. Items 4 and 9 need someone to notify.

Item 1 is the exception and is worth acting on the moment the first service is designed,
because spec-first is a habit rather than a control — and it is much easier to start than
to retrofit.

## Related

- C-0002 — API shape, already settled
- STD-0002 — `API.md` generated from the spec, not hand-written
- REQ-0005 — the same problem for events; compatibility modes and registries
- ANTI-0003 — an API is the alternative to reaching into another service's schema
- REQ-0004 — risk tiering, which item 9 depends on
