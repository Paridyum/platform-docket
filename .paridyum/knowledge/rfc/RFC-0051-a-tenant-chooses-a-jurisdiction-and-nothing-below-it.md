---
id: RFC-0051
title: A tenant chooses a jurisdiction, and nothing below it
type: rfc
status: proposed
created: 2026-09-15
locked_at: null
supersedes: null
superseded_by: null
scope: [org, web, platform, infra]
applies_to: ["*"]
product: [tenancy]
tags: [region, residency, placement, jurisdiction, datacentre, sizing, onboarding, dashboard, c-0006, adr-0041, adr-0103, rfc-0015, rfc-0045, rfc-0050]
breaking: false
---

# RFC-0051 — A tenant chooses a jurisdiction, and nothing below it

## Problem

The create-tenant screen (RFC-0050 slice 5) has to ask where a tenant's data will live, and
nothing decides **what it may ask**.

The pressure is to build the selector every cloud console has: a region dropdown, then an
availability zone, then an instance size. Each of those is a separate decision, each is
separately governed, and **the estate's answer is different for all three** — but the answer has
never been written down in one place, so the screen would be built by copying a mental model of
AWS.

Observably:

1. **`region` is contractual and immutable and has no interface.** ADR-0041 decision 3 fixes it;
   ADR-0103 decision 4 says every tenant chooses at every containment class, refused rather than
   defaulted; RFC-0045 §C makes it a column with a trigger. Nothing renders it.

2. **One region is built and fourteen jurisdictions are declared.** `region-registry.yaml` has
   `in` at `status: live` and fourteen legal regimes named — GDPR, DPDP, PDPA, APPI, LGPD and the
   rest. A selector showing one option is a form pretending to offer a choice; a selector showing
   fifteen is a lie about what can be delivered. Neither is acceptable and the registry already
   draws the line this screen needs: *"a region is a jurisdiction that has been built; a
   jurisdiction is a region that has not."*

3. **Two datacentres exist inside the one region, and nothing says whether a tenant picks one.**
   `in-mum-1` and `in-hyd-1` are both `status: live`.

4. **Nothing says whether a tenant sizes anything.** The drafts RFC-0050 audits carry no sizing
   control, but every console a person has used does, and the absence in a mockup is not a
   decision.

**What happens if nothing is done.** The screen ships with the cloud-console shape — region, zone,
size — and two of those three violate governance that already exists, one of them a constitutional
anti-goal. It is far cheaper to refuse a control than to remove one people are using.

**How we will know it is fixed.** A person creating a tenant chooses a jurisdiction, is told what
regime it answers, cannot choose anything below it, and can see which jurisdictions exist but are
not yet built.

## Constraints

- **C-0006 anti-goal 2 (constitution)** — *"We will not sell infrastructure by the resource… A
  price that names a vCPU, a gigabyte, a transfer volume or an hour is refused."* Narrowed
  2026-08-18 to the **unit of sale** rather than the asset. This decides §Proposal C outright.
- **ADR-0041 decision 3** — `tenant.region` is contractual and immutable; multi-region is
  residency-driven, not a latency optimisation.
- **ADR-0103 decision 4** — region is orthogonal to containment and to plan; every tenant chooses;
  refused, never defaulted; immutable after placement. Decision 2 — no self-serve path to a
  containment rung above C0.
- **RFC-0015 decision 5** — an operator in either country chooses their own region.
- **RFC-0015 decision 3 / `region-registry.yaml`** — two datacentres per region is what makes a
  synchronous standby and a near-zero recovery point work *"without a byte crossing a border"*.
  One would return that region to restore-from-backup.
- **`region-registry.yaml`** — jurisdictions are legal regimes; regions are jurisdictions that have
  been built; `suggest_for` orders what is shown and never picks; `residency.satisfies` records
  which regime a placement **addresses**, never one it claims to meet.
- **ADR-0041 decision 4 / ADR-0028** — no framework stated as satisfied before an audit says so;
  controls map outward.
- **STD-0021** — no credibility that has not been earned. A region list implying global coverage is
  exactly that.
- **BET-0009** — compute may be sold as a running workload, on a separate data plane, under four
  triggers. Not engaged by this screen and named so it is not confused with it.
- **RFC-0045 §C, §E** — placement; and the attestation, which restates the chosen jurisdiction back
  to the tenant as evidence.
- **ADR-0039** — the saga; placement is fixed at step 1 and the screen feeds it.

## Proposal

### A. Two axes: jurisdiction constrains, latency orders — REVISED 2026-09-15

An earlier draft of this section treated latency as a distraction and gave it one line. That was
too thin, and the registry already disagrees with it.

**The two are not in competition; they answer different questions.**

| Axis | Question | Governs |
| --- | --- | --- |
| **Jurisdiction** | *Where is this business legally allowed to keep its data?* | a **constraint** — if a regime is required, only that answer is valid |
| **Latency** | *Where are the people who will use it?* | a **preference** — it orders what is offered and never overrides the constraint |

**ADR-0041 decision 3 is about why regions are BUILT, not about what a tenant may weigh.** Its
words — *"multi-region is residency-driven, not a latency optimisation"* — settle that we do not
open a region to shave milliseconds. They do not say a tenant choosing between two lawful
jurisdictions must ignore where its users are. Most tenants have no residency requirement at all,
and for them proximity is the entire basis of a rational choice.

**The mechanism already exists and is already latency.** `region-registry.yaml`'s `suggest_for`
carries, in its own note, that UAE and Indonesia are *"deliberately routed to India for latency
rather than politics"* — and that it is **"Ordering only. Never a placement."** This RFC makes that
field visible rather than inventing a second one.

**How the control presents both:**

1. **Order by proximity**, using `suggest_for` against the visitor's country. Nothing is
   preselected (ADR-0103 D4 — refused, never defaulted), so ordering is the whole of the latency
   affordance and it cannot silently become a default.
2. **Label by regime.** Every option reads as its legal boundary — **"India · DPDP"** — because
   that is what the choice binds. Geography is a subtitle, never the headline: a cloud console
   leads with geography because *its* choice is about latency and can be changed later; this one
   is immutable and contractual.
3. **Say why an option is first.** "Closest to your users" is an honest, checkable statement about
   ordering. It is not a latency figure — we have no measurement, and a number on this screen
   would be the manufactured precision STD-0021 refuses.
4. **Filter by regime when a tenant has one.** A tenant required to hold data under GDPR narrows to
   that jurisdiction and latency stops mattering, which is the correct precedence: the constraint
   eliminates, the preference orders what survives.

**The rule, stated once so a future reader does not have to re-derive it: latency may reorder the
list and may never shorten it.** A tenant that wants a lawful-but-distant jurisdiction gets it.

### A2. Everything not operational is marked coming soon — and the label needs a fact behind it

**Operator direction, 2026-09-15: every non-operational jurisdiction is shown as "coming soon".**
What follows is how to do that without the label becoming a promise nobody made.

**The registry's own model says there is nothing to mark.** `region-registry.yaml` is explicit:
*"A region is a jurisdiction that has been built; a jurisdiction is a region that has not."* And
the schema backs it — a **datacentre** may carry `status: planned`, and a **region** may not: its
enum is `live`, `degraded`, `disabled`, `retired`. A planned region is a contradiction in the model,
because the thing that makes it a region is having been built.

So the thirteen are not regions awaiting launch. They are **jurisdictions with no region**, and
"coming soon" is a claim laid on top of that fact rather than read from it.

**That claim needs somewhere to be true or false, and STD-0021 is why.** "Coming soon" says we
intend to build it. Nothing in the estate currently says we intend to build thirteen jurisdictions,
and a screen that implies a roadmap the company has not committed to is manufacturing credibility —
the same failure the landing page standard exists to prevent, arriving through a form control.

**Proposed: `availability` becomes a field on a jurisdiction**, so the label is registry data:

| `availability` | On screen | Meaning |
| --- | --- | --- |
| *(derived — a live region exists)* | selectable | operational today |
| `coming_soon` | **"Coming soon"**, disabled | a build is intended; the claim is recorded and can be wrong in an artifact rather than only in a UI |
| `on_request` | **"On request"**, routes to a conversation | no build is planned; a contract could start one |

This is a small addition to `region-registry.schema.json` and it keeps RFC-0014 decision 1's *"one
status authority"* intact — it adds a field to the jurisdiction, and does **not** add a second
status to the region, which is exactly the archive's failure that decision was written against.

**Until the operator sets those values, the honest default is `on_request` for all thirteen** —
and the difference between the two labels is a commercial decision, not an engineering one. Which
jurisdictions are genuinely queued is §Open questions 5.

### B. The unit of choice is a jurisdiction, not a region

### C. A datacentre is never offered

`in-mum-1` and `in-hyd-1` are **a pair, not options**. RFC-0015 decision 3 and the registry's own
note: two datacentres are what make a synchronous standby and a near-zero recovery point possible
without a byte crossing a border. Choosing one is opting out of the standby — it returns that
tenant to restore-from-backup while appearing to be a refinement.

The registry makes the deeper point itself, and it is the reason this must be an explicit refusal
rather than an omission: under the naming scheme these would be `ap-south-1` and `ap-south-2`,
*"which AWS calls two regions. Same buildings, and a name that would have argued for splitting
India's data across two residency boundaries for no legal reason whatsoever."* A zone selector
does not merely add a control; it teaches a false model of what a boundary is.

Datacentre count is **per-region data**, visible in an attestation as a property of the placement
the tenant chose — never as an input to it.

### D. Nothing about servers is offered — and this one is constitutional

No vCPU, no memory, no instance size, no node count, no storage quota as a purchasable dial.

**C-0006 anti-goal 2** forbids the **unit of sale** naming a resource: *"A price that names a vCPU,
a gigabyte, a transfer volume or an hour is refused."* A sizing control is that unit of sale
rendered as a form. The anti-goal's own reasoning applies directly to the screen — it *"makes the
customer's bill scale with our inefficiency, invites line-item comparison against a cloud
provider's published rates, and charges most at the moment a customer is succeeding."*

**What replaces it is a quota, and a quota is not a product.** RFC-0045 §Proposal G and ADR-0024's
accepted costs both class quota and rate limiting as **correctness** features rather than billing
features. A tenant has ceilings; it does not buy them on this screen, and a ceiling raised by
contract is an entitlement change, not a slider.

**The narrow permitted exception, named so it is not mistaken for this.** BET-0009 permits selling
compute when the unit is a *running workload*, on a data plane physically separate from the
platform's, under four triggers. That is a different product with a different buyer. Nothing in the
tenant-creation flow touches it, and if a sizing control ever appears it belongs there and not
here.

### E. Containment is not on this screen either

ADR-0103 decision 2: no self-serve path to a rung above C0. A new tenant is C0 and the screen says
nothing about it. Escalation is a contract and a recorded human approval (`domain.no_self_serve_path_to_a_dedicated_rung`).

What the screen *may* do is what §Proposal A does for unbuilt jurisdictions — make it visible that
something exists and is reached by asking. Whether it does is a commercial call (ADR-0103 §Status,
open question 2 there), not this RFC's.

### F. What the tenant is told, and where it is verifiable

At the moment of choosing: the jurisdiction, the regime it **addresses**, and that the choice is
immutable.

**Never that a framework is satisfied.** `residency.satisfies: [DPDP]` in the registry carries its
own warning — *"a region never claims a framework as satisfied before an audit says so; this field
records which regime the placement ADDRESSES."* The screen copies that distinction exactly, or it
manufactures the credibility STD-0021 forbids and ADR-0041 decision 4 prohibits by name.

The choice then reappears in RFC-0045 §Proposal E's attestation as a checked field rather than a
remembered promise — which is what makes stating it on a form honest in the first place.

## Alternatives considered

1. **The cloud-console shape: region → zone → size.** Familiar to every operator, and every part of
   it is available to build. **Rejected on three separate governed grounds** — a zone selector
   breaks the standby pair (§B), a size selector is C-0006 anti-goal 2's unit of sale (§C), and
   presenting residency as geography teaches a model where the boundary is latency rather than law.
   Familiarity is a real cost to give up and it is the cheapest thing here to give up.

2. **Show only the one live jurisdiction; hide the rest.** Simplest, and defensible: a form with one
   option is at least not a false promise. **Rejected** — it makes the roadmap invisible to the one
   audience that would tell us which jurisdiction to build next, and it gives a prospective EU
   tenant no route to ask. The disabled-with-a-route pattern says the same true thing and leaves a
   door.

3. **Let an operator assign the region rather than the tenant choosing.** Simpler to operate and
   arguably safer. **Rejected**: RFC-0015 decision 5 and ADR-0103 decision 4 both put the choice
   with the party whose legal obligation it is. An operator-assigned jurisdiction is us deciding
   someone else's compliance position for them.

4. **Preselect the suggested jurisdiction to reduce friction.** The standard conversion move.
   **Rejected**: ADR-0103 decision 4 requires refusal over defaulting, and this is precisely the
   ADR-0024 decision 4 hazard — *breadth granted by a lost value* — in a form control. A default
   accepted without being read is not a choice, and this choice is contractual.

## Consequences

**What this makes worse.**

1. **The screen looks sparse next to any cloud console**, and a technical buyer may read the absence
   of controls as an absence of capability. That is the same trade STD-0021 took for the landing
   page and C-0006 anti-goal 2 took for pricing.
2. **Thirteen disabled options is a visible admission of how small the footprint is.** Honest, and
   uncomfortable in a sales conversation. The alternative is discovering it during onboarding.
3. **No sizing control means no self-serve answer to "we need more headroom"** — it becomes a
   conversation. Correct under §C and slower.
4. **The jurisdiction-not-geography framing will be argued about.** "Asia Pacific (Mumbai)" is what
   people recognise. §A answers it once and it will come back.
5. **A tenant choosing wrong is expensive to fix**, deliberately. §A surfaces that before the
   choice; it does not make it cheaper.

## Rollout

| Slice | What lands |
| --- | --- |
| **1** | Read `region-registry.yaml` at build time — live regions and declared jurisdictions, with regime and `suggest_for` |
| **2** | The jurisdiction control: ordered, nothing preselected, unbuilt shown-and-disabled, immutability stated before the choice |
| **3** | The choice submits to ADR-0039 step 1's `region`; refuses without it |
| **4** | The jurisdiction and its regime appear in the attestation (RFC-0045 §E) as checked fields |

Slice 1 is a registry read and is independent of the rest. Slice 3 depends on `platform-tenancy`
being reachable.

**This adds no new artifact obligations**: the registry is the source, and RFC-0051 decides only
what a screen may ask of it.

## Open questions

1. **What does "ask about an unbuilt jurisdiction" actually do?** A contact route, a waitlist that
   records demand, or nothing but an address. The second is the most useful — it would tell us
   which region to build next from evidence rather than instinct — and a waitlist is a promise with
   a shape, so it needs deciding rather than defaulting.
2. **Does the disabled list show all fourteen, or a curated few?** Fourteen is the honest registry
   and a long control. Curating means choosing which regimes to advertise, which is a commercial
   decision wearing a UI one.
3. **Is latency ever mentioned?** `suggest_for` exists *for* latency and the registry is explicit
   that it is ordering only. Saying so makes the ordering legible; saying it badly reintroduces the
   geography model §A rejects.
4. **Where does a tenant see its datacentre pair?** §B says it is a property of the placement, not
   an input. The attestation is the obvious home, and whether a tenant cares is unproven.

## Related

- **C-0006** anti-goal 2 — the unit of sale, which decides §C
- **ADR-0041** decision 3 (contractual, immutable) and decision 4 (nothing claimed before audit)
- **ADR-0103** decision 4 (region ⊥ containment ⊥ plan; refused, never defaulted) and decision 2
- **RFC-0015** decisions 3 and 5 — the standby pair, and who chooses
- **`region-registry.yaml`** — the source, and the jurisdiction/region distinction this screen renders
- **RFC-0045** §C placement, §E the attestation that verifies the choice
- **RFC-0050** — the create-tenant screen this decides the content of
- **BET-0009** — where a sizing control would legitimately live, and why it is not here
- **STD-0021** — why an unbuilt jurisdiction is shown as unbuilt
