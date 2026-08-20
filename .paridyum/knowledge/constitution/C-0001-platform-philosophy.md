---
id: C-0001
title: Platform philosophy and engineering principles
type: constitution
status: accepted
created: 2026-08-06
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [charter, principles, philosophy]
breaking: false
enforced_by: []
---

# Constitution C-0001 — Platform philosophy

## Status

**`accepted`, deliberately not `locked`.**

Per ADR-0002, `locked` is reserved for what a check can enforce. Nothing here can be
mechanically verified — these are the principles that *generate* the enforceable rules,
not rules themselves. Marking this `locked` would make the word mean two different
things. It is no less binding for being unenforced; it is simply binding on judgement
rather than on CI.

This is the document to re-read when a decision feels genuinely open.

## Vision

To become the operating system for businesses.

## Mission

Enable anyone to launch production-grade software businesses in minutes.

## Principles

### 1. Platform before product

The first artifact is not a service. It is the thing that makes every future service
consistent. A capability built once in the platform is worth more than the same
capability built well three times in three services.

### 2. Automate everything that will happen twice

The second time a task is performed by hand is the signal to automate it. Not the
fifth. Manual steps are where drift enters, and drift is unrecoverable at scale.

### 3. Documentation is code

Knowledge is versioned, reviewed, and — wherever possible — executable. A decision that
exists only in someone's head, or only in a chat log, does not exist. See ADR-0002.

### 4. No tribal knowledge

Every rule must be discoverable from a single repository checkout, offline, by someone
or something that has never spoken to anyone. If onboarding requires a conversation,
the knowledge base has a gap. This principle is why ADR-0001 vendors knowledge
redundantly rather than centralising it.

### 5. Never infer, always retrieve

Applies to humans and agents equally. When a question has a settled answer, find it.
Inventing an answer that happens to differ from the settled one is more expensive than
asking, because it ships and then has to be undone.

### 6. Every service is replaceable

No service is special. Identical layout, identical CI, identical observability,
identical deployment shape. A service that needs an exception is a signal that the
platform is missing a capability — fix the platform, not the service.

### 7. Make the right thing the easy thing

Standards enforced only by discipline decay. Standards enforced by scaffolding and CI
hold. If the compliant path is harder than the non-compliant one, the tooling is wrong
and no amount of documentation will save it.

### 8. Decide once, in writing, with the cost stated

An ADR that lists only benefits is incomplete and probably wrong. The value of the
record is in the rejected alternatives and the accepted costs — those are what
future-you needs in order to judge whether the decision still holds.

## How to use this document

When facing a decision with no locked artifact covering it:

1. Check whether a principle here already answers it. Usually one does.
2. If two principles conflict, the lower-numbered one wins.
3. Write the outcome as an ADR. If it can be enforced, lock it.

If a principle here is repeatedly obstructing good work, that is worth knowing — amend
it deliberately rather than accumulating quiet exceptions.
