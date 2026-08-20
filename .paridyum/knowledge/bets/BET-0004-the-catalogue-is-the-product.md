---
id: BET-0004
title: The product is a catalogue of vertical applications on one shared multi-tenant runtime
type: bet
status: accepted
created: 2026-08-08
locked_at: null
supersedes: [BET-0003]
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [market, product, runtime, catalogue, multi-tenancy]
breaking: false
trigger: >
  RATIFIED 2026-08-08. Any ONE of the three kills the bet; they are ordered by how early each
  can fire, and each tests a different one of the three claims below.
  (1) ECONOMICS — the engineering cost of the fifth catalogue product exceeds half the cost of
  the first. The wager is that a shared runtime makes each product cheaper; if product five is
  not dramatically cheaper than product one, the leverage is not there and the catalogue is
  simply serial bespoke work with a shared login.
  (2) FRAGMENTATION — more than one in four shipped products acquires tenant-specific branches
  or forks. A catalogue that fragments per customer is bespoke work wearing a catalogue's name,
  and it revives the per-vertical linear cost BET-0003 was written to avoid.
  (3) DEMAND — more than one in three of the first sixty customers signs only after Paridyum
  agrees to build a product that is not in the catalogue. At that rate the catalogue does not
  match the market and the product is really custom development.
review_date: 2027-02-01
enforced_by: []
---

# BET-0004 — The catalogue is the product

## The wager

**Paridyum builds and operates a catalogue of named vertical applications on one shared
multi-tenant runtime. A customer selects products; every customer who selects a product gets the
same product, and ships of that product reach all of them.**

The runtime is still the thing that makes many products cheap to operate. What changed is what
sits on top of it: a catalogue we build, rather than applications a model generates per customer.

## Why this supersedes BET-0003 — and how

**BET-0003 was not falsified. It was withdrawn.** Its trigger was *"more than one in three
applications that reach production needed hand-written code outside the spec contract, measured
over the first hundred"* — and **zero** applications have reached production. No evidence
touched it.

It was withdrawn on 2026-08-08 by a direct statement of commercial intent, during an
architecture conversation about identity, in answer to a question posed in two forms:

> *"When you sign a new customer, are you handing them software you already built, or generating
> software for them?"* — **"I am company A."**

Recording the distinction matters. A bet killed by its trigger is evidence about the world. A bet
withdrawn by decision is evidence about intent, and the two justify very different amounts of
confidence in what replaces them. This one is intent.

**What that means for the reader:** BET-0003's reasoning about generation economics was never
tested and may still be right. If the catalogue proves slow to extend — one vertical at a time,
linear cost per product, which is exactly the failure BET-0003 was written to avoid — the
generated-layer wager is available again, undefeated.

## How the three trigger clauses were chosen

Clause 1 is the one to watch, because it fires **earliest** and tests the **load-bearing** claim.
Product five is reachable; sixty customers is not, and by the time clause 3 could fire the
strategy is largely spent. Clause 1 is also measured from Paridyum's own engineering records
rather than from sales outcomes, so it is not confounded by who was sold to.

**Clause 1 needs a measurement convention agreed before it is needed, or it becomes arguable.**
"Engineering cost" must mean one thing — engineer-days from scoping to first production tenant,
excluding runtime work that benefits every product — and it must be recorded per product from
product one, not reconstructed at product five. Without that, the clause is unfalsifiable in
practice while looking rigorous, which is the failure ADR-0018 names for a trigger nobody can
evaluate.

Clause 2 catches the slow version of the same failure and is visible in the codebase rather than
in a spreadsheet. Its weakness is that a disciplined team can suppress the signal by refusing
forks, at which point the cost reappears as slower delivery — which clause 1 would catch.

## What is being bet

1. **A catalogue can be built fast enough** that customers find what they need in it. This is the
   whole wager, and it is the thing BET-0003 doubted.
2. **Shared products beat bespoke ones commercially** — one Invoicing improved for everyone
   rather than a hundred variants improved for one.
3. **The multi-tenant runtime still carries most of the leverage.** Products share tenancy,
   identity, billing, entitlement and isolation, so product number ten costs far less than
   product number one.

## What this does not decide

**It does not forbid generation.** A generated application remains expressible: RFC-0012's
topology puts a tenant's own applications alongside granted catalogue products in the same
organization, deliberately. If generation returns as a feature rather than as the product, no
architecture changes.

**It does not settle the vertical order**, or how many products constitute a viable catalogue.

## Consequences already visible

- **`platform-generation` and `platform-spec`** — two of RFC-0009's six first-wave contexts —
  were justified by BET-0003. **Re-justified the same day** (RFC-0009 Revision 2): both survive
  with changed roles and changed classes. `platform-spec` becomes a `spec-` repository holding
  *product* contracts rather than *business* specifications — the declaration RFC-0012 requires
  of every catalogue product and ADR-0039's saga reads. `platform-generation` becomes a `tool-`
  repository and leaves the first wave: it is now the factory Paridyum builds its catalogue with
  rather than the product itself, and building it before three products exist would be designing
  a generator against a catalogue that does not exist. **First wave: six contexts become five.**
- **C-0006's market posture** was written against a runtime-as-product framing and should be
  re-read, though nothing in it is obviously contradicted.
- **RFC-0012's identity topology assumed BET-0003** in its first draft — tenants running only
  generated applications is what removed the need for shared products inside tenant instances.
  Withdrawing the bet is what moved the design to organization-per-tenant, where Project Grants
  serve a catalogue natively.

## Revision — 2026-08-15

**Two corrections and one addition. The wager itself is unchanged.**

**Correction 1 — the C-0006 claim above is wrong.** The consequences section states that C-0006
*"should be re-read, though nothing in it is obviously contradicted."* It was contradicted, in
the plainest possible way: anti-goal 1 forbade *"a catalogue of hand-authored vertical apps"*,
and this bet wagers on exactly that. C-0006 is a `constitution/` artifact and outranks every bet,
so for a week a rank-6 artifact contradicted a rank-1 one with a note saying it didn't.

Worth recording how it happened. Anti-goal 1 ended *"See BET-0003"* and the framings table said
*"the shape BET-0003 bets against"* — **C-0006 had delegated the argument entirely and never
stated its own reason.** So re-reading C-0006 for a contradiction found only a pointer to a bet
this one had just withdrawn, which reads as agreement. C-0006 was amended on 2026-08-15.

**Correction 2 — clause 1's measurement convention was never agreed.** This bet says it *"needs a
measurement convention agreed before it is needed, or it becomes arguable."* It was not, and the
clause has been unfalsifiable-in-practice since. BET-0005 makes it cheap to fix: Paridyum builds
and operates product one itself, so engineer-days from scoping to first production tenant are
directly observed rather than reconstructed.

**Addition — BET-0005 states the structure this bet left implicit.** *"A customer selects
products"* never said who logs in. The answer is both: Paridyum operates catalogue products as a
tenant on its own runtime and sells to end customers directly, **and** franchises the same
products to tenants who run them as their businesses. That resolves what "product one" means —
one Paridyum operates — and supplies the answer to the question this bet could not answer, which
is who wants to launch an invoicing business.

## Related

- BET-0005 — Paridyum operates what it franchises; the structure this bet left implicit
- BET-0003 — superseded, withdrawn rather than falsified; its economics argument stands untested
- BET-0001 — the wedge; unchanged, and a catalogue may reach it differently
- BET-0002 — price per running business; a catalogue makes per-product pricing tempting and this
  bet does not authorise it
- C-0006 — market posture, written before this and worth re-reading
- RFC-0009 — the decomposition whose generation contexts depend on this
- RFC-0012 — the identity topology that changed when this changed
