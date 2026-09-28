---
id: ADR-0050
title: A product is a fourth governance axis, and the dossier is registry data
type: adr
status: accepted
created: 2026-08-17
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [governance, product, artifact-model, market, registry, strategy]
breaking: true
enforced_by:
  - check: knowledge.product_registry_valid
    severity: error
    rationale: Moving the product list out of prose into data makes it easier to change, which is both the intent and the risk. The four prior registries each needed this and the argument has not weakened.
  - check: knowledge.product_slugs_resolve
    severity: error
    rationale: A typo'd slug produces an artifact that validates, binds nothing, and reads as though it governs the product whose name it nearly spells. Same guarantee an unknown check id gets under ADR-0014.
  - check: knowledge.market_artifacts_cite_sources
    severity: error
    rationale: The requirement that makes MKT- an evidence type rather than a bet with a different prefix. It cannot live in the schema — validate-schema.py implements no conditionals and silently ignores them.
  - check: product.stage_dossier_complete
    severity: error
    rationale: Without it the dossier's requires lists are prose in a YAML file, which is the state this repository's founding argument is about.
  - check: product.sequencing_respects_second_vertical_rule
    severity: warn
    rationale: Declared at warn rather than error deliberately. It reads registry state and the state can be legitimately transient during a supersession; more importantly, an error-severity check would invite reading it as enforcement of C-0006 anti-goal 6 itself, which it is not.
---

# ADR-0050 — A product is a fourth governance axis, and the dossier is registry data

## Status

**`accepted`** as of 2026-08-17. Not `locked`, and the reason is ADR-0002's rather than
modesty: every check above is **declared and unimplemented**. A decision becomes lockable
once something can mechanically reject its violation, and today nothing does.

The path to `locked` is the one ADR-0022 took and it is worth naming so it is not forgotten:
that ADR held at `accepted` until the registry had been exercised once — `glossary` was added by
editing two files and nothing else, and section 8c then failed on all three
hardcoded-but-checked sites, which is the silent failure the registry existed to convert into a
loud one, caught on its first unassisted run. **The equivalent condition here is the first
catalogue product reaching `proposed` with an `MKT-` behind it, and the dossier check failing
when that artifact is removed.** Until that has been watched, this stays `accepted`.

## Context

RFC-0024 has the full argument. The short form is three facts that were all true on the day this
was written and could not all be true together.

C-0008 states that every runtime capability Paridyum builds is a product a tenant can buy.
BET-0004 and BET-0005 make the catalogue the product and commit Paridyum to operating what it
franchises. And **the governance model had no way to address a product at all** — `scope`
selects repository classes, `applies_to` narrows to repository names, and there was no third
field. The company was constitutionally a set of products, governed by a system that could only
name repositories.

The concrete cost was a live constitutional requirement that nothing could satisfy. C-0006
requires that every vertical Paridyum enters be recorded in an artifact naming the evidence that
selected it, and no artifact type could hold that: a `BET-`'s load-bearing field is what would
*kill* it, and this is about what *selected* something. The ROADMAP's phase 3 exit gate is
"vertical one chosen"; when it was met, the evidence had nowhere to go.

## Decision

**Four parts.**

### 1. `product` is a fourth axis, orthogonal to `scope` and `applies_to`

A `product:` array in frontmatter, holding slugs registered in `product-registry.yaml`.

Resolution mirrors ADR-0015 exactly, so there is one rule to learn rather than two:

- An artifact naming no product **binds every product**, as an artifact scoped `org` binds every
  repository and a classless repository is governed rather than exempt.
- An artifact naming products binds those, narrowing within whatever `scope` already selected.
- `forge context <repo>` is **unchanged**. `forge context --product <slug>` is new.

**A product is never a `scope` value.** One product spans several repositories and one
repository serves several products; they are independent axes and folding them together is the
special-casing artifact-registry.yaml's header describes.

### 2. `MKT-` is a new artifact type, and `sources` is what defines it

Declared in `artifact-registry.yaml`: `lockable: false`, `requires_trigger: never`,
`requires_sources: always`, `immutable_when_locked: true`, directory `markets/`.

**`requires_sources` is a fourth axis on the artifact-type registry, added by this decision.**
The registry previously carried `lockable`, `deferrable` and `requires_trigger`, and the first
draft of this ADR hardcoded the sources rule as `type == market` inside the gate. That is
the precise shape `requires_trigger` already exists to replace: it absorbed `type ==
requirement` from `check-knowledge.sh` when ADR-0018's rule and ADR-0022's rule turned out to be
one axis at two settings. A fourth type-specific branch in the gate would have re-introduced,
for evidence, the defect the third axis had just removed for triggers.

ADR-0022 is locked and enumerates the axes that existed when it was written. **Nothing here
contradicts it** — the axes are
registry data under ADR-0022's own decision, and adding one is the operation that ADR makes
cheap. This is recorded because the alternative was a structural change to the substrate of a
locked decision appearing in no artifact at all, which is the drift ADR-0013 exists to prevent.

An `MKT-` must carry at least one `sources` entry. That is the line between this type and
`BET-`: an observation can be checked against the world today, a wager can only be judged later.
Both are needed and neither substitutes for the other.

### 3. The dossier is registry data, not artifact types

`product-registry.yaml` carries `stages`, a `dossier` block naming what each `kind` owes at each
stage, and the product list. **Adding a required document kind is one line in `dossier`.**

`artifacts` names artifact-type prefixes the gate can resolve and count. `slots` names
obligations with no type yet — `go-to-market`, `sales-playbook`, `support-sla` — which warn
rather than fail, because a check that failed on a slot would be asserting knowledge it does not
have.

### 4. Paridyum is registered as product zero

`product: paridyum`, `kind: platform`, `stage: building`.

C-0008 says Paridyum is tenant zero and never a privileged one. This is the same principle in
the governance layer: the operator appears in the registry of the things it sells, carries the
dossier its kind owes, and is addressable by the same selector as anything else.

The alternative — an unregistered operator governing a registry of other people's products —
is the asymmetry C-0008 forbids in the runtime, and it would have been harder to see here,
because nothing looks more natural than the platform being the exception.

## Consequences

**Accepted costs**

1. **A fourth axis is a fourth thing to get wrong, and the failure is quiet.** `scope`,
   `applies_to` and now `product` all narrow, and an artifact that names a product when it meant
   to name a repository class binds a smaller set than its author intended while validating
   cleanly. `knowledge.product_slugs_resolve` catches an unregistered slug; it cannot catch a
   registered one used by mistake.

2. **The backlog grows by five.** Five checks declared, none implemented, in a repository where
   roughly half of what is declared already awaits `tool-forge`. Stated plainly because ADR-0014
   requires it of itself, and because the honest reading is that this decision is enforced by
   compliance today and not by machinery.

3. **A product registry makes registering products cheap, and cheap is the wrong direction.**
   C-0006 anti-goal 6 forbids a second vertical before the first has a renewed paying customer,
   and this change supplies the paperwork for violating it faster than anyone could previously
   have produced it. `product.sequencing_respects_second_vertical_rule` is the mitigation and it
   is partial by construction.

4. **`slots` is a category that will be argued about.** It is an obligation with no artifact
   type, checked at `warn`, and it will be tempting either to promote them all to types (the
   twenty-empty-directories option RFC-0024 rejected) or to stop reading the warnings. The
   intended resolution is that a slot warning for long enough is evidence it should become a
   type — but nothing measures "long enough".

5. **Four capability products are registered at `proposed` with nothing built.** `identity`,
   `tenancy`, `entitlement` and `routing` exist in the registry and nowhere else. This
   reproduces what `pattern` already does in `artifact-registry.yaml` — recording the space as
   it is rather than only the populated parts — but a registry whose entries are mostly
   aspirational is one people learn to skim.

6. **`MKT-` will be used to launder opinions.** The type requires a source, not a good one, and
   a determined author can cite a decoration. The check verifies that evidence was offered and
   can never verify that it was any good.

**Gained**

- C-0006's evidence obligation becomes satisfiable. It was written, live, and impossible to
  comply with.
- The company's own structure becomes expressible in its governance model. A set of products
  governed by a system that could only name repositories was a mismatch that would have been
  paid for later, in the form of product knowledge accumulating wherever it could — chat logs,
  a `docs/` folder, or one person's memory.
- One clause of C-0006 becomes mechanically observable for the first time, with the scope of
  that claim stated rather than implied.
- The ~150-item startup documentation taxonomy that prompted this becomes registry data. Most of
  it already had a home; what did not is four dossier slots and one type.
- Paridyum is governed by the product model it governs products with, for the same reason
  ADR-0015 put the knowledge base under its own constitution.

**Rejected alternatives**

Each is argued in full in RFC-0024; the reason each lost, in one line:

- *Add `product` to the `scope` enum.* Makes `scope` a union of two incompatible selectors, and
  `forge context <repo>` stops being resolvable — a real defect, not an inelegance.
- *A repository per product.* Breaks C-0001 principle 4 the way RFC-0005 already described, and
  makes the anti-goal 6 question — how many verticals are in flight — require every checkout.
- *A `products/<slug>/` subtree of free-form documents.* The `docs/` directory this repository
  was built to not be. `glossary/` sat in exactly that state and the cost is on record.
- *An artifact type per document family.* Twenty types, nineteen empty on creation, and it puts
  the taxonomy — the most volatile list here — into the type system, which is the defect the
  four prior registries each exist to fix.
- *A `repos:` key in the product registry.* Rejected as a second home for a fact
  (RFC-0015) and as the hand-authored cross-repo view ADR-0040 forbids. Repositories declare
  their product in `manifest.yaml`; the mapping is read in one direction.

## Known limitation

**Nothing checks that a registered product has any referent.** `identity` and three siblings are
registered with no repository declaring them and no artifact naming them, which is a legitimate
state — the registry records the product space as it is. The reverse check, that every
registered product is reachable from something, is not added because with five legitimate
exceptions it would fire immediately, and a check whose first act is to flag an intentional
decision teaches people to disable checks.

This is the same limitation ADR-0015 records about scope values with no referent, and it is
recorded here in the same words so the two are recognisably one problem.

## Related

- RFC-0024 — the question, the options, and the taxonomy triage
- C-0008, ADR-0049 — every runtime capability is a product; Paridyum is tenant zero
- C-0006 — the evidence obligation this makes satisfiable, and anti-goal 6
- ADR-0015 — the resolution rule this mirrors, and the known-limitation wording it reuses
- ADR-0022 — the artifact-type registry this adds a type through, and the path to `locked`
- ADR-0014, C-0005 — the other two instances of list-in-code
- RFC-0005 — commercial strategy as knowledge; the argument against a sibling repository
- RFC-0015, ADR-0040 — why the registry does not list repositories
- BET-0004, BET-0005 — the catalogue, and operating what is franchised

## Amendment — 2026-09-27

**The default for a new product's `kind` is `catalogue`, with `capability` as an argued
exception — added by ADR-0112.** This decision did not change: it says which axis a product is
declared on and what each kind owes. ADR-0112 supplies the default this one left open, closes the
exception to two grounds (a runtime primitive, or no end-customer business possible), and requires
every `capability` entry to name its ground — checked by
`product.capability_names_its_exception`. Read the two together; a future reader of this ADR alone
would otherwise reconstruct the per-case judgement that produced the drift ADR-0112 exists to end.
