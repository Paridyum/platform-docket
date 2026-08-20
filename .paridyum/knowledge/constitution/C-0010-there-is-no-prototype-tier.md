---
id: C-0010
title: Everything built is the production system, and there is no prototype tier
type: constitution
status: accepted
created: 2026-08-19
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [charter, quality, prototype, environments, design-before-code, rfc-0023, c-0008]
breaking: false
enforced_by: []
---

# Constitution C-0010 — There is no prototype tier

## Status

**`accepted`, deliberately not `locked`,** for the reason C-0001 gives and ADR-0002 enforces:
this states a principle, and nothing can check a principle. `enforced_by` is empty because
naming a proxy check here would claim that finish is mechanically verifiable, which is the
false confidence ADR-0002 exists to prevent.

**C-0001 outranks this document**, and this specifies rather than amends it. Principle 7 —
*make the right thing the easy thing* — observes that standards enforced only by discipline
decay. This document is exactly such a standard, and it says so rather than pretending
otherwise. Its mechanically enforceable fragments already live elsewhere and are listed under
*What already carries part of this*.

## The principle

> **Everything Paridyum builds is the production system. `dev` names where something runs,
> never how well it is built.**

Two halves, and both are load-bearing:

**There is no prototype tier.** Nothing is written on the understanding that it will be
replaced by the real version later. The thing being written *is* the real version — the first
one, at whatever scope was chosen, finished to the standard it will be operated at.

**The cost is paid before the code, not after it.** Weeks of research and a written decision
are cheaper than the wrong abstraction shipped, because the decision costs a document and the
abstraction costs every service built on top of it. Retrieval and design are not the preamble
to the work; below a certain confidence they *are* the work, and producing no code that week
is the correct outcome.

## Why this is a principle and not a work ethic

**"Temporary" is a claim about the future that nothing in the present tests.** Provisional code
is not replaced on a schedule; it is replaced when it fails, which is after something depends on
it. By the time anyone looks, the placeholder is load-bearing and the rewrite is a migration.
The moment to refuse it is the only moment it is cheap — before it is written.

**It is what makes C-0008 achievable rather than aspirational.** Tenant zero forbids a
privileged path, and C-0008 names its origin precisely: *"no path that exists because it was
convenient during development."* C-0008 forbids the artifact. This forbids the reasoning that
produces it, which is where it is actually stopped, because the operator bypass never looks like
an exception at the time — it looks like the obvious way to get the first thing working.

**It is the premise RFC-0023 decision 4 is the infrastructure half of.** *"Environment is a
variable, never a branch"* is this principle applied to configuration, and RFC-0023 records the
cost of the general case honestly in its risk R2: nobody decides to fork, someone adds one
conditional under deadline. The same erosion applies to everything else built here, and this
document exists so the general form is written down rather than inferred from one RFC about a
Proxmox host.

**And it is the only reason the design lifecycle survives contact with a deadline.** ADR-0013
computes significance from the diff so it cannot be self-declared away, and ADR-0019 puts
decomposition before implementation. Both are affordable only if not-writing-code is an
acceptable state to be in for as long as it takes. That permission is granted here, explicitly,
so that it is not re-litigated under pressure.

## What this forbids

- A `TODO`, a stub, or a `todo!()` standing in for a decision that has not been made. If the
  decision is genuinely open, the output is an RFC, not a placeholder in a merged file.
- A mock, a fixture, or a hardcoded value on a delivered path, awaiting the real one.
- A code path conditional on the environment it is running in — RFC-0023 decision 4, generalised
  past infrastructure. A difference that cannot be expressed as a variable is a design
  difference, and it is recorded as one.
- Deferring the cross-cutting concerns — tenancy, identity, audit, observability, migrations —
  on the reasoning that the feature is what matters and those can be added afterwards. They
  cannot; adding them afterwards is the rewrite (C-0008, ADR-0049, ADR-0044).
- A test skipped, ignored, or narrowed to make a gate pass, rather than because the artifact it
  is linked to says so.
- Writing code because writing it is faster than finding the decision that already governs it.
  That is CLAUDE.md's one rule — *never infer architecture, retrieve it* — and it is a shortcut
  in this document's sense, not merely a process lapse.

## The boundary, which is not a loophole

**Scope may be cut. Finish may not.** Building one vertical slice instead of five is a
legitimate and expected decision — STD-0007 is built around it. Building five slices to a
standard that will not survive a customer is not the same decision, and confusing the two is how
this principle gets read as a mandate to gold-plate. It is not. It says: choose less, then
finish what you chose.

**A spike is legal, and its rule is deletion.** Measuring whether nested KVM performs, or which
CSI driver behaves on both cluster sizes, is research, and research runs code. What makes it a
spike rather than a prototype is that its output is an artifact and its code is thrown away —
never merged, never promoted, never "cleaned up later". A spike that lands is a prototype
that was renamed.

**`deferred`, with a trigger, is the sanctioned way to not do something.** ADR-0018 exists for
exactly this, and REQ-0001 is the standing example: a real gap, recorded, with the condition
that reopens it. Deferring loudly is the opposite of a shortcut. Deferring silently, inside a
file, is the thing this forbids.

**This repository's own artifacts iterate by supersession, not by revision.** A decision that
turns out wrong is superseded and both records survive (CLAUDE.md, ADR-0031). That is the
mechanism for being wrong here, and it is not an exception to this principle — it is what this
principle looks like applied to knowledge rather than code.

## What already carries part of this

Named so that the empty `enforced_by` above is understood as honest rather than unattempted.
None of these enforces the principle; each rejects one shape of its violation:

- `test.no_ignored_tests_without_reason` — a skipped test must name why.
- `test.coverage_meets_threshold` · `test.mutation_score_meets_threshold` — assertions that
  execute and assertions that matter, respectively.
- `test.testing_md_declares_untested_surface` — untested surface is declared, not discovered.
- `test.bugfix_carries_a_regression_test` — a fix is not finished at the symptom.
- `agent.generated_code_immutable` · `agent.service_creation_escalates` — C-0004, live in-session.
- RFC-0023 decision 4 — the configuration half, on the substrate.

The residue is judgement, and it is most of it.

## Costs accepted

C-0001 principle 8: *an artifact that lists only benefits is incomplete and probably wrong.*

1. **Everything is slower, and the slowness is front-loaded onto the period with no revenue.**
   This compounds with C-0008 cost 1 rather than overlapping it: C-0008 says tenancy is built
   before invoicing, this says both are built finished. Neither cost pays for the other.

2. **The probability of building the wrong thing *well* goes up.** A cheap probe is the standard
   defence against that, and this document restricts probes to spikes that get deleted — so the
   cheapest form of learning, shipping something rough and watching it fail, is unavailable.
   That is a real loss and there is no mitigation for it beyond the research this principle
   spends its time on instead. The exchange is deliberate; it is not free.

3. **It is indistinguishable from stalling, from outside and sometimes from inside.** "Thinking
   for weeks" and "avoiding the hard part for weeks" produce the same observable output, which
   is none. The only available test is whether the time produced a dated artifact — an RFC, an
   ADR, a market finding — and a stretch of research that produces no artifact should be read as
   the second thing, not the first.

4. **It cannot be checked, and a determined author can satisfy every gate and still ship
   something shaped around one demo.** ADR-0002's residue, stated here rather than papered over
   by declaring a check that would only catch the careless.

5. **It is hardest to hold exactly when it matters most.** A solo operator under a deadline has
   nobody to refuse the shortcut on their behalf, and this document is what is left standing in
   for that person. That is a weak mechanism and it is named as one.

## Related

- C-0001 — principles 1, 7 and 8, which this specifies rather than amends
- **C-0008 · ADR-0049** — tenant zero, whose "no path convenient during development" this is the
  general form of
- **RFC-0023** — decision 4, *environment is a variable, never a branch*; the infrastructure half
- ADR-0013 · ADR-0019 — the design lifecycle this document buys the time for
- ADR-0018 · REQ-0001 — `deferred` with a trigger, the sanctioned way not to build something
- STD-0007 — vertical slices, which is how scope is cut without cutting finish
- STD-0008 · `test-registry.yaml` — the tiers and gates that carry the checkable fragments
- ADR-0002 — why this is `accepted` and its `enforced_by` is empty
