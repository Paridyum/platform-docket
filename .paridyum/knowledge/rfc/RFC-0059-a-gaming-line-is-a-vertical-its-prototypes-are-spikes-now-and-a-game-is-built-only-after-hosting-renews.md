---
id: RFC-0059
title: A gaming line is a vertical; its prototypes are spikes now, and a game is built only after hosting renews
type: rfc
status: accepted
created: 2026-10-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [games, gaming, vertical, catalogue, prototype, spike, anti-goal-6, c-0006, c-0010, adr-0112, adr-0077, rfc-0052, rfc-0013, ip, experiments]
breaking: false
resolved_by: null
verified_by: []
enforced_by: []
---

# RFC-0059 — A gaming line is a vertical; its prototypes are spikes now, and a game is built only after hosting renews

## Status

**ACCEPTED** 2026-10-07, by the operator under ADR-0087's personal authority. Proposed the same day.

Offered three positions on C-0006 anti-goal 6 — keep the gate, amend C-0006, or leave the question
open — the operator answered in their own words:

> *"allow for prototyping"*

**That answer is recorded as D3, and nothing wider.** Prototyping needs no amendment: under C-0010 a
prototype is a spike, and anti-goal 6 gates building, not research (D2). It is **not** recorded as an
amendment of C-0006. A pilot that could ship before hosting renews would need one, and that is a
separate, deliberate act which this answer does not take.

| | Decision | State on acceptance |
| --- | --- | --- |
| D1 | A gaming line is a vertical, counted once as `games` | **Live** as posture; registration waits for the first MKT- |
| D2 | Anti-goal 6 gates building a game, and nothing before it | **Live.** No `game-*` repository holds production code until hosting renews |
| D3 | Prototyping is permitted now, and a prototype is a spike | **Live.** In `~/Developer/experiments/`, deleted once its finding is written |
| D4 | A pilot is a vertical slice, and a vertical slice is `building` | **Waiting** on D2's gate |
| D5 | The line earns its own company on a named trigger | **Not fired.** Nothing has been signed for a game |
| D6 | Until then, a game's IP follows RFC-0013 D4 | **Live** from the first spike |

ADR-0133, accepted with this RFC, carries the repository half: the `game-` class and its
engine-agnostic naming.

## Problem

**The operator, 2026-10-07:**

> *"we are planning to pilot or prototype a few idea[s] that may or may not hit prod … th[e] naming
> shou[l]d be engine agnostic as we might use ue5 for one game bevy for other etc … this might come
> under o[u]r gaming vertical"*

Nothing in the estate says where a game goes, and three things already decided meet the request as
worded:

- **"Vertical" means two things here.** In C-0006 and `product-registry.yaml` it is a market served
  by a `catalogue` product, and anti-goal 6 gates the second one. In the company registry (ADR-0077)
  a separate line of business gets its own company — `PCY` for cyber security, `PMD` for media. "Our
  gaming vertical" could mean either, and the two carry different consequences.
- **"Prototype" has one legal meaning under C-0010: a spike whose code is deleted.** "May or may not
  hit prod" describes exactly what C-0010 forbids if it means rough code that ships when it works out.
- **No repository class fits a game.** ADR-0133 adds one.

## What is already decided

Retrieved rather than reasoned about, per CLAUDE.md.

- **C-0006 anti-goal 6:** *"We will not add a second vertical before the first one has a paying
  customer who renewed."* Its reason: *"Breadth is the most attractive way to avoid finding out
  whether anything works."* It constrains sequencing, not ambition.
- **`product-registry.yaml` makes the gate mechanical.** Entry to `building` from `proposed` is
  anti-goal 6, checked as registry state by `product.sequencing_respects_second_vertical_rule`.
  `hosting` is vertical one, at `building` since 2026-08-23. `veer-entitlements` and `reel` wait at
  `proposed` on hosting's first renewal — the covenant product included, which C-0009 obligation IV
  says grants itself no exception.
- **ADR-0112:** a product is `catalogue` unless it is a runtime primitive or no end-customer business
  is possible. A game has end customers.
- **C-0006, non-commodity item 4:** every vertical Paridyum enters is recorded in an artifact naming
  the evidence that selected it, and that evidence may not be tenant-derived. The `catalogue` dossier
  owes an MKT- at `proposed`, and the registry's footer records that no catalogue product was
  registered before its MKT- existed.
- **C-0010:** there is no prototype tier. *"A spike is legal, and its rule is deletion … its output
  is an artifact and its code is thrown away — never merged, never promoted, never 'cleaned up
  later'. A spike that lands is a prototype that was renamed."* And: *"Scope may be cut. Finish may
  not."*
- **The gate does not cover research.** reel's registry entry, on RFC-0054: *"WHAT IS NOT GATED: the
  runbook, the evidence and the design … the product build waits on the gate."*
- **RUN-0001 already has a home for throwaway code:** `experiments/` — *"throwaway; never deployed,
  never depended on."*
- **RFC-0052's test for a company** is *"distinctness of counterparty, liability, revenue contract,
  or regulatory surface"*, not distinctness of brand. Its D3 creates a company on a named trigger
  rather than in advance.
- **RFC-0013 D4:** work created before incorporation belongs to its author personally and does not
  transfer by forming a company; the incorporation documents must carry a **present** assignment.
  On this RFC's date `PAR`, `PCY` and `PMD` all read `Incorporation in Process`, `cin: null`.

## Decisions

### D1 — A gaming line is a vertical, and it is counted where verticals are counted

**In C-0006's sense games are a market Paridyum would enter, so the line is a vertical** — whichever
company eventually holds it (D5). A company boundary changes who signs and who is liable. It does not
change what the attention anti-goal 6 protects is spent on.

**It registers once, as `games`, `kind: catalogue`, when its first MKT- exists** — not in this RFC.
Neither ADR-0112 exception applies: a game is not a runtime primitive, and its players are end
customers. Titles are offerings inside the one product, as WordPress and similar software are
offerings inside `hosting`; registering each title would count one market several times.

### D2 — Anti-goal 6 gates building a game, and nothing before it

**No title is built as a product until hosting has a paying customer who renewed**, or until C-0006
is amended deliberately. Concretely: no `game-*` repository receives production code, and `games`
does not move past `proposed`.

Gaming joins the catalogue candidates already waiting behind hosting; it does not jump the queue.
The registry counts them, and the registry's footer says that counting is its purpose.

### D3 — Prototyping is permitted now, and a game prototype is a spike

The operator's answer. **A find-the-fun build, an engine bake-off, a rendering or networking probe, a
control-feel test — each is a spike under C-0010**, and none is gated by anti-goal 6, for the reason
RFC-0054 gave reel: evidence and design are not the build.

What makes it a spike rather than a prototype:

1. **It states its question before its first line of code** — one sentence, in a `SPIKE.md` at its
   root, with the date it ends.
2. **It lives in `~/Developer/experiments/<YYYY-MM-DD>-<question>/`**, never in a `game-*`
   repository. If it is a git repository it carries `.paridyum-no-checkpoint` and no remote, so
   neither the checkpoint hook nor anyone else pushes it to the organisation.
3. **It ends in a dated finding in `knowledge/`** — a note under `docs/`, or evidence the line's MKT-
   cites — written before its code is deleted. C-0010 cost 3 is the test: a stretch of prototyping
   that produces no artifact is read as stalling, not as research.
4. **Its code is then deleted**, with everything built to stand in for the real thing: placeholder
   art, grey-box levels, temporary audio. A finished creative work made along the way — a concept
   piece, a track — is not a placeholder. It is IP under D6, and survives as such.
5. **It is never promoted.** A spike that becomes the game is a prototype that was renamed.

### D4 — A pilot is a vertical slice, and a vertical slice is `building`

**"May or may not hit prod" is answered by retirement, not by building worse.** A pilot is the games
industry's vertical slice — a small part of a title finished to the standard it would ship at — which
is C-0010's *scope may be cut, finish may not* in that industry's own words. It starts from an empty
`game-<codename>` repository (ADR-0133) and inherits the spikes' findings, never their code. A slice
that is stopped is `retired`, and its repository archived; the record stays.

A pilot is building, so it waits on D2's gate.

### D5 — The line earns its own company on a named trigger

**By RFC-0052's test, games are a distinct line of business** — different counterparties (store and
platform holders, engine licensors), a different liability surface (consumer refunds, age ratings),
different revenue contracts — and would hold their own company under `PAR`, as `PCY` and `PMD` do.

**Not yet.** The company is created when the first of these happens:

1. a contract with a store or platform holder;
2. an engine licence that takes a royalty or a fee;
3. a contributor engaged on a title for pay;
4. a title entering `building`.

Until then nothing is signed for a game, so there is nothing for an entity to sign. Creation follows
RFC-0052 D1: an `org.yaml` entry in `tool-docket`, `owned_by: PAR`, deliberately red until an officer
is appointed. Its code and name are chosen then, not here.

### D6 — Until that company exists, a game's IP follows RFC-0013 D4

**Everything a spike produces — code before it is deleted, designs, art, music, names — belongs to
its author personally until a present assignment moves it.** RFC-0013 D4 applies unchanged, and the
incorporation documents name game work explicitly rather than leaning on a general clause.

- **A contributor signs an IP assignment before contributing**, on docket's `IPA` document — RFC-0052
  D5's rule for a freelance editor, applied to an artist, a composer or a programmer.
- **Accounts are the business's from the first minute**: a store developer account, a domain, a
  title's social handle. RFC-0052 D5's Brand Account rule applies, and RFC-0058 (proposed) states the
  same for store accounts.
- **A title's public name is a brand** — ADR-0087: *Paridyum is the main brand* — chosen when it is
  announced. Repositories carry a codename (ADR-0133), so renaming a title renames nothing.

## Alternatives considered

**Amend C-0006 now, so pilots can ship before hosting renews.** Not taken: the operator asked for
prototyping, and prototyping needs no amendment. It stays available, and C-0006 names the route — an
anti-goal that keeps obstructing good work is amended deliberately, not drifted around.

**Treat a separate company as an exemption from anti-goal 6.** Rejected (D1). It would satisfy the
letter of a rule about verticals while spending exactly the attention the rule protects. The covenant
product, whose reason for existing is constitutional, was given no exception (C-0009 obligation IV);
a gaming line has a weaker claim to one.

**Register each title as its own catalogue product.** Rejected (D1): one market counted several times,
and a registry entry per idea before any MKT- exists.

**A `spike-` repository class, so prototypes live in the organisation.** Rejected. A class whose
repositories must never ship is a prototype tier with a prefix, and every repository in the
organisation is production under C-0010. A prototype that genuinely needs a remote is open question 1.

**Games inside `PMD`.** Rejected by RFC-0052's own test: a sponsor contract and a platform's
monetisation terms are not a store agreement or an engine licence, and publishing liability is not
consumer-refund liability.

**Outside the estate — `personal/`, or a separate GitHub organisation.** Rejected. The operator calls
it *our* gaming vertical, ADR-0004 puts every repository in the `Paridyum` organisation, and outside
the estate none of these rules would bind the work — which is the reason to keep it in.

## Consequences

**Accepted costs**

1. **No game ships until hosting renews, and nobody can say when that will be.** What the prototypes
   teach ages in the meantime.
2. **Good prototype code is thrown away**, and the slice rebuilds it. Studios discard prototypes
   routinely, which softens this; it does not remove it.
3. **A spike lives on one machine with no remote.** A lost disk loses the prototype — by then its
   finding should already be written. A prototype that needs two people has no home yet.
4. **Breadth becomes visible in the registry** the day `games` registers: one more catalogue
   candidate behind the same renewal. That is the registry working, and it is uncomfortable.
5. **Nothing mechanical stops a spike growing into a stealth pilot** — C-0010 cost 4. D3's test is
   the only one: a stated question, a dated finding, then deletion.

**Gained**

- Prototyping starts today, without amending the constitution.
- Ideas are measured before one is chosen, in the order C-0006 asks for.
- A title that ships is built finished from its first commit, and its IP is assigned before it is
  worth anything.

## Rollout

1. This RFC, accepted, with **ADR-0133** — the `game-` class, its naming, and `games/` in RUN-0001.
2. **The checkpoint guard measures the staged size** (ADR-0133 decision 9), so a game repository's
   Git LFS assets are committed rather than left out.
3. Per idea: a spike with a stated question, ending in a finding.
4. The first MKT- for the gaming market, and `games` registers at `proposed`.
5. When the gate opens, or C-0006 is amended: the first title's engine ADR, then its repository.

## Open questions

1. **Where does a prototype live that needs a second person, or a backup?** `experiments/` is one
   machine. Reopened by the first spike that needs either.
2. **Does a title use the runtime** — player identity, entitlement — or ship standalone? Decided per
   title when it enters `building`, not here.

## Related

- C-0006 — anti-goal 6, non-commodity item 4, and the amendment route
- C-0010 — the spike rule, *scope may be cut, finish may not*, costs 3 and 4
- C-0009 — obligation IV: no product exempts itself from anti-goal 6
- ADR-0112 — catalogue by default
- ADR-0077 · ADR-0087 — the company registry, the brand rule, governance authority
- RFC-0052 — the company test, its trigger shape (D3), contributor assignments (D5)
- RFC-0013 — D4, present assignment before incorporation
- RFC-0054 — research is not the build
- RFC-0058 — store accounts belong to the business
- ADR-0133 — the repository half of this decision
- RUN-0001 — `experiments/`
