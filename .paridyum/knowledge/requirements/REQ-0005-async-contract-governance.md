---
id: REQ-0005
title: An event is a versioned contract with compatibility rules and a replay path
type: requirement
status: deferred
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, spec]
applies_to: ["*"]
tags: [events, contracts, schema-registry, idempotency, async]
breaking: false
trigger: >
  A second service subscribes to an event published by a first — the moment an event
  stops being an internal detail and becomes a contract between two deployables.
review_date: 2027-02-01
enforced_by: []
---

# REQ-0005 — An event is a versioned contract with compatibility rules and a replay path

## Status

**DEFERRED** as of 2026-08-07. No service publishes an event and none subscribes.

The trigger is precise on purpose. A single service publishing to itself needs none of
this. The **second subscriber** is the moment the event shape stops being changeable at
will, and it is the last cheap moment to put the rules in place.

## Requirement

**An event published as a contract shall have a registered schema, a stated compatibility
mode, a version and deprecation path, and defined delivery semantics — and a consumer
shall be able to prove it still works against the producer's current schema without
running the producer.**

## Numbered items

1. **Schema registry.** Every published event has a machine-readable schema — Protobuf,
   Avro or JSON Schema — with a registered version. `manifest.yaml`'s `exposes.events`
   currently names events but does not describe their shape, which is enough to detect a
   dead subscription and not enough to detect a broken one.
2. **Compatibility mode.** Declared per event stream: backward, forward or full. Breaking
   changes detected mechanically in CI, the way `buf breaking` does for Protobuf. A
   compatibility mode nobody checks is a preference.
3. **Envelope standard.** Every event carries id, type, version, timestamp, correlation id
   and causation id. The correlation id is what makes a cross-service failure debuggable
   at all, and it must be propagated, not regenerated.
4. **Delivery semantics stated.** At-least-once is the realistic default, which makes
   **idempotent consumers mandatory rather than optional**. A consumer that is not
   idempotent is a bug waiting for a redelivery, and redelivery is normal.
5. **Ordering guarantees.** Stated explicitly, including that there usually are none across
   partitions. Most ordering bugs come from assuming a guarantee nobody promised.
6. **Dead-letter queue and replay.** Where a poison message goes, who is alerted, and how a
   fixed consumer replays. Without a replay path, a bad deploy silently drops events and
   nobody knows what was lost.
7. **Consumer-driven contract tests.** A consumer's expectations are asserted against the
   producer's schema in CI. This is what lets a producer change safely, and it is the
   difference between an event contract and an event convention.
8. **Deprecation path.** Announcement window, migration deadline, no-new-consumers lint,
   then removal. Same shape as API deprecation, and equally necessary.
9. **Cron, queue and webhook governance.** Retries, backoff, poison-message handling. A
   cron job reaching into another service is ANTI-0003 wearing a schedule.

## What already exists

- C-0002 fixes event **naming**: `<Subject><PastTenseVerb>`, PascalCase, never imperative.
- `manifest.yaml` declares `exposes.events` and `depends.events`, so a subscription to an
  event nobody publishes is already detectable — the *dead* case.
- ANTI-0002 establishes events as the decoupling tool and excludes event edges from cycle
  detection, which is what makes item 4 load-bearing: decoupling via events is only safe if
  consumers tolerate redelivery.

The gap is everything about **shape** rather than **name**.

## Related

- C-0002 — event naming
- ANTI-0002 — events as the fix for synchronous cycles
- ANTI-0005 — compensating paths; events are how most sagas are carried
- STD-0002 — `manifest.yaml`, `exposes.events` / `depends.events`
- RFC-0002 — decomposition, where event contracts are first designed
