---
id: ADR-0144
title: The console calls the tenant a tenant on its own screens
type: adr
status: accepted
created: 2026-10-11
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, web]
applies_to: ["*"]
product: [paridyum, docket]
tags: [vocabulary, tenant, business, console, adr-0113, adr-0143, glo-0002, glo-0012]
breaking: false
enforced_by: []
---

# ADR-0144 — The console calls the tenant a tenant on its own screens

## Status

**ACCEPTED** 2026-10-11 by the operator. **It amends ADR-0113 D1**: on the tenant console's
screens the word is *tenant*, not *business*. Every other part of D1 stands — including that
governance and code say *tenant* and that nothing calls an org a business.

## Context

ADR-0113 D1 says: *"Business" is the tenant's name on screen and in conversation, and means
exactly a tenant.* It was written when the only screens that named a tenant were the platform's
own — the portal, the landing, careers — where the reader is an **org owner** being shown the
businesses they own. For that reader "business" is the outside-in word, and D1 chose it
deliberately.

The console did not exist yet. ADR-0143 (accepted the day before this amendment) gives each
tenant its own administration surface at `<host>/dashboard`, addressed to the **tenant's own
operators**, signed in through the tenant's own client, reading the tenant's own roster. There
the outside-in word is the wrong one: the reader is inside the thing the word names. The
surface's own name, its hosts, its role grants and its tokens all say *tenant* (GLO-0002,
GLO-0012, ADR-0130 D5) — a console screen that said "your business" would be the one place in
the tenant's estate translating the vocabulary its own machinery speaks.

The operator asked the question directly: *can we name it tenant only there?*

## Decision

On the console's screens — ADR-0143's surface, at a tenant's `<host>/dashboard` — the tenant is
named **tenant**.

ADR-0113 D1 continues to name the tenant a **business** on every screen where the **org** is the
reader: the portal (paridyum.in), the landing, careers — and in conversation. Nothing calls an
org a business (unchanged). The glossary is unchanged: a tenant is still a business (GLO-0002);
only the console's screen word changes.

## Consequences

**Accepted costs** — one thing is named by two words on two surfaces: an org owner reads
"business" in the portal and "tenant" in the console. The seam is the portal's link into each
business's console, whose label can carry the tenant's own display name — a name, and so neither
word.

**Gained** — the console speaks the vocabulary its readers' own estate uses: roles, tokens,
hosts and the surface's own name all say tenant. Nothing on a console screen needs translating
against governance, and the people who live in the console are named from inside it.

**Rejected alternatives** — *rename everywhere to "tenant"*: discards the portal's owner-facing
word that ADR-0113 D1 chose deliberately for a person who owns several businesses. *Keep
"business" in the console too*: one word everywhere, but it makes the console address the people
it serves in the platform's outside-in voice. *A glossary entry for a "console tenant"*: a
second term for one thing, which is what GLO-0002 exists to prevent.
