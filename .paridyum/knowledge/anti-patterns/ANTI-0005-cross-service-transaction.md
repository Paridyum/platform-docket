---
id: ANTI-0005
title: A cross-service transaction with no compensating path
type: anti-pattern
status: accepted
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [platform, service]
applies_to: ["platform-*", "svc-*"]
tags: [consistency, saga, failure-handling, judgement]
breaking: false
verified_by:
  - TST-0014
enforced_by: []
---

# ANTI-0005 — A cross-service transaction with no compensating path

## Status

**ACCEPTED**, not `locked`. No check can tell a sequence of calls that happens to span
services from one that requires atomicity across them — that is a statement about intent,
not about code. Upheld by review. `enforced_by` is empty rather than aspirational
(ADR-0002).

## The shape

```rust
// svc-orders
charge_payment(order).await?;      // svc-billing
reserve_stock(order).await?;       // svc-inventory   <-- fails here
send_confirmation(order).await?;   // never runs
```

The customer's card is charged. No stock is reserved. No email is sent. There is no
transaction to roll back, because the two writes happened in two different databases owned
by two different services.

This is not exotic. It is the default shape of the first multi-service feature anyone writes,
because it is what the single-service version looked like and it compiled.

## Why it is forbidden

**`?` is not a rollback.** In a single service, a failed transaction leaves nothing behind.
Across services, each completed call is permanent. The failure does not undo the work — it
abandons it, halfway, with money already moved.

- **The system is now inconsistent and nothing knows.** No error surfaces to the customer
  beyond a generic failure, no alert fires, and the charge sits there until someone
  complains.
- **Retrying makes it worse.** A naive retry charges twice. Making the retry safe requires
  idempotency the original design never considered.
- **It is discovered in production, by a user, about money.** These bugs are found by
  support tickets, not tests, because the failure needs a partial outage to reproduce.

The prohibition is **not** on sequences of calls that span services — that is most of what a
service does. It is on sequences where a **partial completion leaves the system in a state
nobody designed**, with no path back.

## What to do instead

**Name the pattern and write down the compensation.** Any of these is acceptable; choosing
none is not.

**Saga with explicit compensation.** Each step has an inverse, and failure runs the inverses
in reverse:

```
charge_payment      compensate: refund_payment
reserve_stock       compensate: release_stock
```

**Reorder so the irreversible step is last.** Frequently the whole problem dissolves. Reserve
stock first — reversible, cheap — and charge last. A failed charge releases a reservation;
a failed reservation has cost nothing.

**Make it eventually consistent via events.** `OrderPlaced` is published; billing and
inventory each react and publish their own outcome; a failure publishes
`StockReservationFailed`, which billing subscribes to and refunds. Nobody blocks on anybody.

**Accept the inconsistency, deliberately and in writing.** Sometimes the right answer is
that a rare partial failure gets reconciled by a nightly job or a human. That is a legitimate
engineering decision — *when it is a decision*. The anti-pattern is arriving there by
accident.

## What review must produce

For any change where two services both write, the RFC or ADR states:

1. Which step is the irreversible one, and why it is ordered where it is.
2. What happens if each step fails — specifically, not "returns an error".
3. Whether retries are safe, and what makes them idempotent if so.
4. Who notices when compensation itself fails, because it will.

If those four cannot be answered, the design is not finished. That is the actual gate here,
and it lives in ADR-0013's stage 3 review rather than in CI.

## Related

- ADR-0013 — the design review where these four questions get answered
- ANTI-0001 — why the two writes are in two databases in the first place
- ANTI-0002 — events as the decoupling tool, also the tool here
- ADR-0003 — Kubernetes; a pod dying mid-sequence is routine, not exceptional
