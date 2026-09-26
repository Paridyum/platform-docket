---
id: ADR-0106
title: The YouTube estate is three sub-brands split by viewer intent, and Explained-Technology launches first
type: adr
status: accepted
created: 2026-09-24
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [brand, media, youtube, go-to-market, editorial, std-0015, std-0021, bet-0005, bet-0006]
breaking: false
enforced_by: []
---

# ADR-0106 — The YouTube estate is three sub-brands split by viewer intent

## Status

**ACCEPTED** as of 2026-09-24 by the operator, who supplied the architecture and declared it
final. This artifact records that decision; it does not relitigate it. The questions the
architecture leaves open are named in §*What this does not decide* rather than answered
silently, per CLAUDE.md's rule against inventing an answer and encoding it.

## Context

Paridyum has had no media, content or distribution artifact of any kind. `product-registry.yaml`
declares a `go-to-market` slot on the product contract and that slot has been empty since it was
written. A search of the estate on 2026-09-24 found zero references to YouTube anywhere in the
knowledge base, the repositories, or the operator's memory. This is the first artifact in that
space, which is why it is an ADR rather than an amendment to one.

**Three facts shaped how it is written.**

1. **"Franchise" is a taken word and it does not mean this.** BET-0005 spends a section on where
   the franchise analogy *stops*, and its answer is that tenants run under their own brand — the
   opposite of renting the franchisor's. A family of channels that all carry the Paridyum name is
   therefore the one thing the word cannot describe here. This artifact says **estate** and
   **sub-brand**; the source document's "franchise" is recorded as intent, not adopted as
   vocabulary.
2. **STD-0021 binds every public surface, not just `paridyum.com`.** Paridyum has no customers.
   A channel may not imply operating scale, adoption, or a track record it has not earned.
3. **STD-0015 owns the mark.** `ui.mark_is_not_redrawn` is an error-severity check. Channel
   avatars, banners and lower-thirds are brand assets and are bound by it — including
   `ui.brand_assets_carry_no_ground`, which a 1920x1080 video frame makes very easy to violate.

## Decision

### 1. The estate is three sub-brands, divided by editorial intent

| Sub-brand | Core question | Purpose |
| --- | --- | --- |
| **Paridyum Explained** | Why and how does this work? | Understanding |
| **Paridyum Academy** | How do I learn and do this? | Education |
| **Paridyum Labs** | What happens when we build or test this? | Experimentation |

**The division is intent, not subject.** This is the load-bearing clause. A subject belongs to
no sub-brand exclusively, and the same subject appearing in all three is correct rather than
duplicative: *How Linux Works* (Explained), *Learn Linux System Administration* (Academy),
*I Built a Linux Server* (Labs). **Topic decides what is discussed; editorial purpose decides
where it belongs.**

### 2. Explained is a family of verticals

Technology, Science, Law, Civilization, Human. These are editorial areas, not rigid boundaries,
and the list is open. Academy's areas are Programming, Linux, Cloud, Cybersecurity, Engineering;
Labs' are Projects, Experiments, Hardware, Reviews.

### 3. Technology is a vertical under Explained, not a competing umbrella

There is no "Paridyum Technology". The first channel is **Paridyum Explained: Technology**,
positioned as *the systems, technologies and ideas powering the digital world*.

### 4. The estate launches one channel at a time

Phase 1 Explained-Technology, phase 2 Academy, phase 3 Labs. Other Explained verticals develop
independently as capacity allows. Initial strategy optimises for a recognisable editorial
standard, **not** upload volume.

### 5. A new channel requires five conditions, all of them

A sufficiently large content opportunity; a distinct audience intent; a repeatable production
model; enough content to sustain the channel; and a stated reason the content cannot stay on an
existing channel. **Topic existence is not a reason.** This exists to stop the estate fanning
out into fifteen starved channels.

### 6. The editorial boundary is a refusal list

Explained-Technology may not become a technology-news or consumer-tech channel. Excluded: daily
news, product announcements, spec-sheet comparisons, clickbait, rumours and leaks, reaction
videos, generic listicles, and imitation of established reviewers.

Priorities, in order: understanding over trending, engineering over consumerism, depth over
speed, context over hype, original explanation over aggregation.

**The per-video test:** *does this help the viewer understand something they previously did not
understand?* A product review passes only when the review is meaningful technical analysis.

### 7. Voice is per sub-brand, standard is shared

Explained is curious, investigative, documentary, research-driven — *"I understand this now."*
Academy is clear, structured, practical, progressive — *"I can do this now."* Labs is
experimental, hands-on, transparent, results-driven — *"I want to see what happens."*

## Consequences

- **Each channel is a product** and therefore inherits the estate's product rules: a logbook
  recording how it was built (RFC-0033) and an SOP for its production pipeline before that
  pipeline is rebuilt from memory (STD-0013).
- **STD-0021 applies to every frame.** No manufactured credibility, no implied scale.
- **STD-0015 applies to every asset.** Avatars and banners are brand assets under an
  error-severity check, not design work done in a thumbnail editor.
- **Decision 5 is the artifact's real teeth.** Without it, three sub-brands times five verticals
  is a fifteen-channel commitment nobody has the production capacity to honour.

## What this does not decide

Named rather than invented, and each one blocks something:

1. **Why the estate exists.** Marketing for the platform, a revenue product in its own right, or
   the operator's own media — the answer changes the success measure, and nothing here states it.
2. **Who the viewer is, against BET-0006.** The accepted wager is that the wedge is the operator,
   not the builder. Explained-Technology's stated coverage — compilers, kernels, GPUs, git
   internals — is builder-facing. That is not necessarily wrong, but it is an audience the
   current bet says is *not* the customer, and the divergence is undeclared.
3. **Whether a vertical is a channel or a playlist.** Decision 3 makes Technology a vertical;
   decision 4 launches it as a channel named `Explained: Technology`. Whether Science later
   becomes a sibling channel or a second playlist on one `Paridyum Explained` channel is open,
   and the handle chosen at launch will quietly decide it if nobody decides it deliberately.
4. **Cadence.** "Not upload volume" is a direction, not a number, and is unenforceable as written.
5. **Phase ordering against cost of production.** Labs is phase 3 and is the only sub-brand whose
   raw material the estate already generates as a byproduct — substrate builds, Proxmox,
   Terraform, Zitadel, the logbooks. Explained is phase 1 and is the most expensive to produce
   and the most contested. The ordering may still be right; it has not been argued.
6. **Any relationship to the `go-to-market` slot** in `product-registry.yaml`, which remains empty.
