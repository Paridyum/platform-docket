---
id: ADR-0109
title: Operator means breadth within one tenant, and the commercial party above a tenant is not modelled
type: adr
status: superseded
created: 2026-09-26
locked_at: null
supersedes: null
superseded_by: ADR-0114
scope: [org, platform, service, web, infra]
applies_to: ["*"]
product: [tenancy]
tags: [tenancy, vocabulary, operator, operator-scope, glossary, adr-0024, adr-0038, glo-0001, glo-0004, glo-0005, bet-0002, bet-0005]
breaking: false
enforced_by: []
---

# ADR-0109 — Operator means breadth within one tenant, and the commercial party above a tenant is not modelled

## Status

**ACCEPTED** as of 2026-09-26 by the operator. Proposed 2026-09-26. Answers T-G.5, which asks that one reading be chosen, the losing
artifacts corrected, and `operator_id` either put in a migration or stop being claimed.

**What acceptance releases:** T-G.5's corrections to GLO-0001, GLO-0004, BET-0002 and BET-0005, and
the removal of `operator_id` from a glossary definition no migration backs.

## Context

**The estate uses `operator` at two different levels, and one document uses both.**

**Reading A — the commercial party.** GLO-0001: *"An operator is the party that holds a Paridyum
subscription and runs one or more tenants. The operator is our customer. Nobody else in the
hierarchy is."* Column: `operator_id`. One operator owns 1..N tenants. GLO-0004's tree draws it
above `tenant`, and BET-0002 prices per running business precisely so one operator may hold several.

**Reading B — the runtime grant.** GLO-0005: *"Operator scope is a session that may read every
account **within one tenant**."* This is `paridyum.operator_scope`, `paridyum.has_operator_scope()`,
docket's operator token, and the role a tenant's own console runs as.

**They are not the same level.** Reading A's operator spans *tenants*; reading B's spans *accounts
inside one tenant*. Every policy in the estate is `tenant_id = paridyum.current_tenant() AND …`, so
nothing the code calls "operator" can cross a tenant boundary — which is exactly what reading A's
operator is defined by.

**BET-0005 uses both, eleven lines apart** — *"Paridyum, as a paying operator of its own runtime"*
(reading A) and *"GLO-0005's rule that operator scope is a grant, never an absence"* (reading B).
That is not a drafting slip; it is what an unresolved term does to every document that inherits it.

**The asymmetry that decides this.** Reading B is implemented in **twelve migrations, all applied
and therefore immutable** (REQ-0008), in `lib-auth`'s published `SessionContext.operatorScope`, in
tool-docket's adapter, and in ADR-0038's accessors. Reading A is implemented **nowhere**:
`operator_id` appears in no migration. Verified 2026-09-26.

## Decision

**1. `operator` means reading B: a principal with breadth within one tenant.**

Chosen because it is the reading that is already load-bearing in artifacts that cannot be changed.
Renaming it would mean editing applied migrations — which REQ-0008 forbids and `migrate.py`
refuses — plus a breaking change to a published library API and to every ADR that cites operator
scope. Reading A costs nothing to correct because it is a declaration and not an implementation.

This is the same principle the estate applies to schemas, applied to vocabulary: **when two
readings disagree, the one already written into something immutable wins, and the one that is only
claimed gets corrected.**

**2. GLO-0001 is corrected, not superseded.** Its structural claim — that a paying party may hold
several tenants — is a commercial fact and stays true. What is corrected is the word: that party is
not what this system calls an operator.

**3. The commercial party above a tenant is not modelled, and is deliberately not named here.**

It is ADR-0024 decision 9's deferred third level, and D9's own rule governs: *"Trigger: a real third
level with a paying customer behind it — a reseller or white-label partner above the operator, or
sub-accounts within an account. **Not 'we can imagine one.'**"* ADR-0024 is `locked`.

Naming it today would be naming a rung that is not operable — the discipline ADR-0103 decision 3
states for containment classes, and the reason is identical: a name in the vocabulary is a
commitment that something exists to be named. **It gets a term when it is funded, from whoever
funds it.**

**4. `operator_id` stops being claimed.** GLO-0001 declares a column that no migration has and no
code reads. A column named in the glossary and absent from the schema is the same defect class as a
check named in `enforced_by` that resolves to nothing (ADR-0014) — it reads as a fact and is a plan.
It is removed from the definition and returns when a migration introduces it.

**5. The artifacts that inherit the ambiguity are corrected**: GLO-0001's definition and column
line; GLO-0004's tree, whose top level is Paridyum-the-platform and whose `operator` row is
reading B; BET-0005's two uses; and BET-0002's pricing sentence, which keeps its commercial meaning
and loses the schema implication.

## Consequences

**Accepted costs**

1. **GLO-0001's title stops matching its content.** "Operator" as a glossary entry now defines the
   runtime grant, and the entry's original subject has no term. That is the honest state and it is
   uncomfortable on purpose — an empty slot invites someone to fill it when the trigger fires.
2. **The commercial hierarchy is unsayable in one word** until D9's trigger. Documents that need it
   must write the phrase out, which is a small tax that keeps the imagined level visible as
   imagined.
3. **BET-0002's pricing model is now expressed in a term the schema does not carry.** Correct —
   billing is not modelled yet either, and pretending the vocabulary is ready would hide that.

**Gained**

- **One word, one level.** Every migration, every policy, every ADR that says operator means the
  same thing, and the thing it means is the thing the code enforces.
- **ADR-0108 stops being ambiguous.** Its decision 1 — `/dashboard` requires operator scope — reads
  as "a tenant's own administrator", which is what it meant and what GLO-0004 point 1 describes.
- **The third level stays visibly deferred** rather than half-existing in a glossary.

## Alternatives considered

- **Keep reading A and rename the runtime concept** (`operator_scope` → `tenant_admin_scope`).
  The vocabulary would be cleaner and the cost is disqualifying: twelve applied migrations are
  immutable, `lib-auth` publishes the name, and `paridyum.operator_scope` is a session variable in
  a live database. A rename that cannot reach the thing it renames is a second name, not a rename.
- **Invent a term now for the commercial party** — `subscriber`, `licensee`, `customer`. Rejected
  by decision 3. `subscriber` is additionally already spoken for in unrelated technical senses
  (event subscriber, tracing subscriber), and `customer` is the word GLO-0003 used for an account
  before ADR-0104 widened it — both would import a fresh collision to fix an old one.
- **Qualify both** — "commercial operator" and "runtime operator". Rejected: a term that needs an
  adjective every time is a term that failed, and the adjective will be dropped in the document
  that matters.

## Related

- **T-G.5** — the task this answers
- **GLO-0001**, **GLO-0004**, **GLO-0005** — the three the correction touches
- **ADR-0024** — `locked`; D9 is decision 3's authority
- **ADR-0038** — the accessors that implement reading B
- **ADR-0103** D3 — a named rung is an operable rung; decision 3 applies it to vocabulary
- **ADR-0108** — whose decision 1 this disambiguates
- **BET-0002**, **BET-0005** — the commercial artifacts corrected
