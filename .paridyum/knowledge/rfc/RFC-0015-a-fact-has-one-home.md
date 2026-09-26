---
id: RFC-0015
title: A fact has one home, and nothing crosses a region boundary
type: rfc
status: proposed
created: 2026-08-11
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, infra]
applies_to: ["*"]
tags: [region, residency, replication, disaster-recovery, entitlement, placement, gdpr, adr-0041, adr-0047]
breaking: false
enforced_by: []
sources:
  - url: https://www.datacenterdynamics.com/en/news/aws-unable-to-restore-access-to-data-centers-hit-by-iran-strikes/
    title: "Datacentre Dynamics — AWS 'unable to restore access' to data centers hit by Iran strikes"
    retrieved: 2026-09-24
    supports: >
      AWS declared data hosted exclusively in one damaged UAE availability zone permanently
      unrecoverable, and damage in Bahrain spanned multiple zones — physical destruction exceeded
      the level the region's multi-zone design was built to withstand. The failure a sealed
      off-jurisdiction copy addresses is real at building scale, not hypothetical.
  - url: https://www.chosun.com/english/industry-en/2026/09/16/OQE6F6MFLNASVAG2YRPVOUABAA/
    title: "Chosun — AWS permanent data loss in the Middle East after Iran attacks"
    retrieved: 2026-09-24
    supports: >
      Data that had not been copied out of the region was completely lost; the mitigation was
      migration to another region. The copy that survived was the copy outside the damaged
      facility — the class of copy decision 3's revision distinguishes.
  - url: https://www.edps.europa.eu/sites/default/files/publication/edpb_recommendations_on_supplementary_measures_-_dpo_meeting_11_dec_2020_en.pdf
    title: "EDPB Recommendations 01/2020 on measures that supplement transfer tools"
    retrieved: 2026-09-24
    supports: >
      Encryption can be an effective supplementary measure for a third-country transfer — the
      recommendations' hosting scenario is storage abroad under keys retained by the exporter — but
      it supplements the transfer rules rather than exempting a transfer from them, because the
      exporter keeps the keys. The legal reading the revision's conditions implement. The condition
      is specifically that keys are held by the exporter, or an entity in the EEA or an adequate
      third country — not merely "not the destination" — which is a narrower bar than ADR-0071
      states and is the reason this RFC's key-custody gap (see the 2026-09-24 review note below)
      is a real one rather than a formality.
  - url: https://www.dpdpa.com/dpdparules/rule15.html
    title: "DPDP Rules 2025, Rule 15 — transfer of personal data outside India"
    retrieved: 2026-09-24
    supports: >
      Rule 15, finalised with the Rules' notification on 2025-11-13, is a permissive default: a
      Data Fiduciary may transfer personal data outside India, restricted only by a Central
      Government general or special order naming a country or sector. No such order exists yet —
      no blacklist, no mandated encryption, no required DPA. This RFC's four sealed-copy conditions
      are therefore stricter than current Indian law requires; MKT-0003's "negative list, not yet
      published" framing is confirmed current, not stale.
---

# RFC-0015 — A fact has one home

## Status

**PROPOSED** as of 2026-08-11. Feedback deadline **2026-09-10**. **Revised 2026-09-24** — decision 3,
and the revision is part of what review now reads; the status stays `proposed` until a human accepts
it.

**Second of four.** RFC-0014 defines the structure — what a region is, what the planes are, how a
table declares its plane. This RFC decides how those planes behave: what may cross between them,
what redundancy means, how a tenant is stopped, and how its region is chosen. RFC-0016 decides the
edge. Depends on RFC-0014 and is not readable without it.

## The question

RFC-0014 establishes that global facts and regional facts live in different databases. It does not
say what happens when a region needs a global fact, and that is the decision with the widest blast
radius in the whole model.

Answer it one way — copy global facts into every region — and every request stays local, at the cost
of copies that go stale, copies a deletion request must reach, and a rule with an exception in it.
Answer it the other way — regions ask, and nothing is copied — and deletion, revocation and
staleness stop being problems at all, at the cost of putting the control plane on the path of some
requests.

ADR-0047 already answered a narrower version of this question for the staff and operator directory
and chose the second: *"it does not replicate, because N regional copies of a staff directory means
N joiner processes and N places to forget to deprovision."* This RFC generalises that answer and
states what it costs.

Three further questions follow from it and are settled here because each is cheap now and structural
later: how many regions actually run, what redundancy means when it may not cross a border, and who
chooses where a tenant lives.

## Decision 1 — one fact, one home; nothing is replicated

**Regional data stays in its region. Global data lives in the control plane and is read from
anywhere. Nothing is copied into a region.**

This is the same rule ADR-0047 decision 1 already applies to the staff and operator directory —
*"it does not replicate, because N regional copies of a staff directory means N joiner processes
and N places to forget to deprovision."* An earlier draft of this RFC carved out an exception for a
read-only entitlement projection and then had to spend a paragraph arguing why that exception was
safe. The argument was sound and the exception was still wrong: a rule with one defended exception
is a rule that will acquire a second. REQ-0003's Related section makes the general form of the
point — *"projections mean copies, which is what makes deletion hard."*

`platform-entitlement` therefore answers from the control plane, and a region asks when it needs an
answer.

**What this removes.** No sync, no staleness, no propagation bound, no revocation lag, and no risk
that personal data enters a projection by a later widening of what the projection carries. A
cancellation takes effect the moment it is made, everywhere. Deletion has one place to reach.

**What this costs, and the shape it takes.** The control plane is now reachable on the request path.
The cost is bounded by how often a request needs a global fact, which is less often than it first
appears:

| Action | Needs the control plane? |
| --- | --- |
| Log in | No — regional Zitadel (ADR-0047) |
| Read or write business data | No — regional database |
| Anything gated by a plan or quota | **Yes** |
| Sign up, provision, change plan | **Yes** — and the target region too; see below |

So this is not "every request crosses a region", it is "**gated actions cross a region**", and a
user attributes that latency to the action rather than to the product.

**Behaviour when the control plane is unreachable is decided here, not discovered later.** Degrade
by action class:

1. **Existing work continues.** Login and business data are regional and unaffected. This is the
   overwhelming majority of traffic.
2. **Gated actions fail closed** — `503`, retryable, never a guess. Guessing *permitted* is a
   security hole; guessing *denied* breaks paying customers. GLO-0005's rule that absence must never
   produce breadth is the same argument one level up.
3. **Provisioning and plan changes pause.** ADR-0039's saga cannot run.

   **Provisioning is the one operation that spans both planes, and it is stated here because it is
   easy to place wrongly.** ADR-0039 decision 1 says *"`platform-tenancy` owns the saga"* — that is
   a statement about which context owns it, not about which plane it runs in. The saga reads global
   facts (which operator, which subscription, which region was contracted for) and writes regional
   ones (the tenant row, the Zitadel organization in that region, per ADR-0047). It therefore needs
   **both** the control plane and the target region available, and it pauses if either is down — not
   only the control plane. No other operation has this property.

A control-plane outage therefore means everyone already working carries on and nothing new starts.
That is an acceptable failure shape, and it is a decision rather than an accident.

**The consequence is an availability obligation.** The control region needs the highest availability
target in the estate, because it is the only component whose loss is felt in every region. REQ-0004
§10 owns SLOs; this decision is what makes the control region's number different in kind from every
other number.

**The edge routing table is the one global fact that lives outside the control plane**, and it is
config rather than data: generated, versioned, republished rather than synced, carrying no personal
data, and required for the edge to function at all (RFC-0016 decision 1). It is named here rather than left
to be discovered, because it is the single place this rule bends.

## Decision 2 — one region live, the model built for many

Everything above is designed for N regions. **Exactly one region runs.**

This is C-0006 anti-goal 6 applied literally — *"Design for breadth, sell to one segment"* — and
BET-0001's beachhead section applied to the geography axis: *"a runtime contract fitted to the needs
of one segment makes market number two a rewrite rather than a configuration."*

The archive is the argument against the alternative. Eight regions were declared, six were probably
never exercised locally, dev ran two, and every one of the eight pointed at the same endpoint.
Building the second region before a customer needs it reproduces that outcome with better prose.

**Trigger for a second region: a signed contract whose residency requirement the control region cannot
satisfy.** Not a conversation, not a plausible prospect — the same bar ADR-0024 decision 8 sets for
its deferred isolation tiers and C-0006 anti-goal 5 sets for enterprise isolation generally.

## Decision 3 — redundancy is in-region, across datacentres

**Redundancy is achieved across datacentres inside a region. Nothing crosses a region boundary —
not a standby, not a replica, not a backup.**

RFC-0014's separation is what makes this cheap rather than a compromise. A region with more than
one datacentre gets a synchronous standby, a near-zero recovery point and a recovery time measured
in minutes, **without a single byte crossing a border.** Availability and residency are not in
tension; they only appeared to be while region and datacentre were the same word.

**No existing artifact mentions datacentres at all**, so the three readings below are stated as
compatibility arguments rather than as citations. Each is a reading of an artifact that is silent on
the question, which is exactly the condition under which this RFC should say so rather than claim
support it does not have:

- **One Postgres per region** (ADR-0047 decision 1) is unaffected. A primary with a synchronous
  standby in a second datacentre of the same region is one database, not two.
- **One Zitadel deployment per region** (ADR-0047 decision 1) spans datacentres within its region.
- **The masterkey is immutable per region** (ADR-0045). It is shared across that region's
  datacentres, so losing a datacentre does not touch ADR-0045's unrecoverability risk at all — that
  risk is a region-level property and stays exactly where ADR-0045 put it.

**The cross-region alternatives remain rejected.** A warm standby in a paired region and backup
replication to a second region are each a **permanent cross-border transfer of personal data**,
requiring disclosure in every customer contract and in the DPA, and a lawful pairing for each region
as a standing obligation.

**Accepted risk — jurisdiction-level loss has no recovery path, by choice.** Two datacentres in a
region cover the failures that actually happen: a machine, a rack, a power event, a building. They do
not cover the loss of the jurisdiction itself — a country-wide event, a provider exit, a regulatory
seizure — because both datacentres and every backup are inside it. The only mitigation is a copy
somewhere else, which is the cross-border transfer this decision exists to refuse, and which would
weaken the guarantee that is the product (C-0006's *"provable isolation"*).

**So the risk is accepted rather than mitigated, and recorded here so it is not later discovered as
an oversight.** This follows ADR-0045's practice of naming accepted risks explicitly — its R3 does
the same for a masterkey that exists before secret management does. Whether it needs a review date
belongs with the ADR this RFC resolves into.

**The residual case is a region with one datacentre**, which will happen in exactly the
jurisdictions where residency matters most. There, and only there, the original trade returns:
restore-from-backup with real downtime, or a cross-border pairing that must be disclosed. Because
datacentre count is a per-region property in the registry (RFC-0014 decision 1), this is answered per region
rather than by a platform-wide policy that would be wrong somewhere.

**The control region's numbers are different in kind.** Decision 1 makes every region depend on it for
gated actions and provisioning, so its recovery objectives are not one row in a table — they are the
estate's worst case. REQ-0004 item 12 requires an RPO and RTO stated and *"restores drilled"*; an
unverified backup is not a backup. Those figures are not chosen here.

### Sealed copies are not replication — REVISED 2026-09-24

**This decision was written on 2026-08-11, before ADR-0071 (2026-09-07) chose an off-host store
that sits in Canada.** The estate therefore already sends an encrypted copy out of the jurisdiction
while the paragraphs above, read literally, say no backup may cross. Two accepted positions, three
weeks apart, never reconciled — and a reader of this section alone would conclude the Sync copy is
a violation. It is not, and this revision states the distinction the original draft did not have.

**What is refused is a copy with operational consequences.** D1's objection is not geography for
its own sake. A copy that serves a request, resolves an entitlement, holds a session or is kept in
sync must stay consistent, must go stale correctly, and must be reachable by a deletion and by a
suspension — D4's rule, that anything of a tenant's outside its region *"reintroduces exactly the
coordination problem this design does not have."* Those are the costs this decision refuses to pay
across a boundary.

**A sealed copy has none of those properties.** It is written once, never read by a request, never
reconciled with anything, and reached by erasure only as destruction. It is a backup in the sense
ADR-0064 D3 requires, not a replica in the sense this decision refuses.

**The rule. A copy that is restore-only, encrypted before it leaves, and on no request path MAY
cross a region boundary. A copy that serves a request, resolves an entitlement, holds a session or
is kept in sync MAY NOT.** The first requires all four conditions:

1. **Ciphertext before egress** — encrypted while still in-region, to a key the destination does
   not hold (ADR-0071 decision 1).
2. **Opaque paths** — ids, never names (ADR-0071 decision 4): a file name is metadata a
   zero-knowledge store does not necessarily encrypt.
3. **No automated restore path** — restoring is a deliberate, recorded operator action; the copy
   never mounts into a running region.
4. **Named, not hidden** — the copy appears in the DPA, in the attestation RFC-0045 §E generates,
   and in the honest RPO/RTO ADR-0105 decision 3 requires. The disclosure the refused alternative
   demanded is now the condition of the permitted one, rather than a reason to keep no copy at all.

**What this does not change.** No standby, no replica, no second live region; nothing a tenant can
reach lives outside its region (D4 stands untouched); the control plane does not move; and ADR-0105
is unaffected — a sealed copy is not a second failure domain, because there is still nowhere to
run. Jurisdiction-level loss changes from *no copy survives* to *a cold copy survives, and recovery
is restore-elsewhere under a contract amendment*. Availability is still lost with the jurisdiction,
and this revision does not present a restore as a failover; ADR-0105's honest-number rule is what
keeps the two apart in every attestation. At a rung where the key is the tenant's (RFC-0046), a
sealed copy is ciphertext under their DEK and *"no longer proof of recoverability"* — the
attestation says which rung it describes.

**The alternative this revision had to beat was keeping the rule and moving the copy in-region.**
It loses on facts already recorded: MinIO on `sitar` is on the source's own hypervisor, which
ADR-0064 D3 names as not a backup and ADR-0071 rejected for that reason, and the cloud-S3 option
was rejected at this stage on cost and control with its revisit trigger written into ADR-0071
decision 2. The live choice was therefore this copy or none — and none fails ADR-0064 D3's gate,
the gate the first real applicant waits behind.

**The evidence that makes this the weakest paragraph of the original decision is dated.** In March
2026 strikes destroyed AWS facilities in the UAE and Bahrain at a scale AWS's own multi-zone design
did not withstand, and in September AWS declared data permanently unrecoverable where no copy
existed outside the damaged facility; its mitigation was migration to another region. That is the
failure a sealed copy addresses, and it is cited in this artifact's `sources` rather than carried
as a memory. It does not make the estate ready to recover — ADR-0105 and T-G.7 own that — but it
means *"both datacentres and every backup are inside it"* is a sentence this RFC may no longer say
without the copy this revision permits.

**A tenant's contract may refuse even this.** The permission is the estate's default, not a term
imposed on every tenant: where a regime or a contract forbids it, the attestation records the
posture instead. A permission, not an obligation.

**Reviewed 2026-09-24 — three gaps found, none blocking, none closed by this artifact:**

1. **Key custody jurisdiction is unstated.** RUN-0004 places the single age private key in "the
   operator's password manager" and names neither the product nor its jurisdiction. EDPB's own
   condition (see `sources`) is that a key must be held by the exporter or an entity in the EEA
   or an adequate country — a narrower bar than "not held by Sync.com." Not load-bearing under
   DPDP today (Rule 15 is unrestricted, see `sources`), and becomes load-bearing the day RFC-0017
   is accepted and an EU-resident operator exists. Nothing here decides it; RUN-0004 is where it
   would be recorded.
2. **Sync.com's own file-version retention is not addressed for the database dump.** ADR-0069
   states it for the document vault — *"Sync.com keeps file versions. Deleting a file there is not
   erasure"* — but ADR-0071 decision 5 only states that erased rows persist in older dumps until
   *our* retention schedule ages them out, and is silent on whether Sync's version history extends
   a superseded dump's life past that schedule. This revision's "reached by erasure only as
   destruction" assumes the ADR-0071 leg without the ADR-0069 caveat carried over.
3. **Operator personal data rides the same path, unnamed.** Until RFC-0017 is accepted, operator
   PII lives in the control-plane database ADR-0071 dumps nightly, so it already crosses under
   this revision's four conditions today — the text above speaks only of tenant data. Not a
   contradiction; an omission worth one sentence.

Not fixed here because none require a decision this RFC's scope owns — 1 is RUN-0004's fact to
record, 2 is ADR-0071's gap to close, 3 is a wording gap either artifact could close on its next
revision.

## Decision 4 — stopping a tenant is a single-region action

**A tenant is suspended in its own region, by that region, and the effect is immediate.**

This follows from RFC-0014 decision 1 and is stated because the alternative is easy to assume and wrong. A
tenant's business data, its Zitadel organization and its users' sessions all live in one region
(ADR-0047 decision 1). There is nowhere else holding anything that would need to be told. There is
no propagation, no coordination, no window during which the tenant is stopped in one place and
running in another — because the tenant does not exist in another place.

Two corollaries:

- **Suspending an operator is one action per region in which it holds a tenant**, applied to each of
  its tenants. Not a global operation, and today not a plural one either, since decision 2 runs one
  region.
- **The edge stops nothing.** It routes. Withdrawing a routing entry prevents traffic *arriving*; it
  does not suspend anything, and treating it as the mechanism would put a security decision in a
  configuration artefact sitting in the class ADR-0037 leaves ungoverned (consequence 3). Open
  question 2 is only about whether to shed the traffic as well, never about where suspension happens.

This is worth recording as a decision because it is a property that a later change could silently
destroy. Any proposal that puts part of a tenant outside its region — a cache, a projection, a
cross-region index — reintroduces exactly the coordination problem this design does not have, and
decision 1's reversal conditions are where that debt would be taken on.

## Decision 5 — how a tenant's region is chosen

Nothing decides this today. ADR-0039 step 1 reads *"insert tenant row (with `region`)"* — the region
arrives as an input and no artifact says where that input comes from. Under CLAUDE.md's rule that is
not a small omission; it is an instruction to guess, at the one moment the answer becomes permanent.

**The operator chooses, explicitly, from the regions that exist.** Not the platform, not a rule, not
an algorithm reading a signal. Region is a term of the contract the operator signs (ADR-0041), and a
contractual term is agreed rather than assigned.

This is a smaller decision than the one an earlier draft made, and deliberately so. Placement-by-rule
requires the platform to know something it does not have: whether this business has a residency
obligation, where its users will actually be, and which of those two wins. All three are facts the
operator holds and we do not. **Asking is not a worse answer than inferring; it is the only correct
one available.**

Guidance is offered at the point of choice and is advice, not a decision: *if law requires a
jurisdiction, choose it; otherwise choose where most of your users will be.* The distinction matters
because guidance that hardens into a default becomes the inference this decision refuses.

**Placement is never inferred from the address a signup arrives from.** A founder registering a
US company from a hotel in Berlin would be placed in Frankfurt, permanently, on the strength of one
night's travel. GLO-0006 refuses the same inference for locale, and its worked example is this exact
person: *"`en` / `en-IN` / `America/New_York` / `USD` — the same person, on assignment, billed in
dollars."* Where someone physically is says nothing reliable about where they belong. The argument is
stronger for region than for locale, because a wrong locale is a setting the user changes and a wrong
region is ADR-0041's contract amendment plus a data migration.

**The choice is made once, explicitly, and is an input to the saga rather than a default inside it.**
ADR-0041 makes region immutable, so provisioning must fail closed on an absent region rather than
apply a fallback. A default here is a permanent decision nobody made — GLO-0005's absence hazard,
one level above the database.

**The archive is prior art and a warning.** It carried a country-to-region map, and mapped twenty
countries out of roughly two hundred; everything unmapped *"falls to whatever default placement
`pd-core` applies."* Under this decision the map survives only as **the ordering of a list** — which
region to show first — and never as a placement. A suggestion the operator can see and override is
useful; the same data applied silently is the fallback that made the archive's model unsound.

**The choice is recorded with its reason**, because ADR-0041 makes cross-region staff access a
control that must be evidenced and REQ-0003 item 8 will ask which transfers were permitted. A region
recorded without why it was chosen answers neither question a year later.


## Consequences

**Accepted costs**

1. **The control region is a hard dependency for gated actions and provisioning everywhere.** Not for
   serving existing work — decision 1's degradation rules keep login and business data regional —
   but a control-plane outage pauses plan-gated actions, signup and provisioning in every region at
   once. This is the price of refusing replication, taken deliberately, and it is why the home
   region carries the estate's strictest availability target.

2. **Gated actions are slower than regional ones**, permanently, by roughly the round trip from the
   region to the control region. This is the cost decision 1 accepts in exchange for having no copies
   to go stale.

3. **Jurisdiction-level loss has no recovery path.** Stated as an accepted risk in decision 3 rather
   than mitigated, because every mitigation is a cross-border transfer. **Revised 2026-09-24:** a
   sealed copy may cross (decision 3 as revised), so the loss is no longer of every copy. What
   remains accepted is narrower and is stated rather than implied: recovery is a cold restore under
   a contract amendment, never a failover, and it depends on a copy that is configured, rehearsed
   and named — which for the database leg is not true today (T-G.7).

4. **Every region multiplies a real operational surface**: a Postgres, a Zitadel with its own
   immutable masterkey whose loss is unrecoverable (ADR-0045), a cluster, a release path, a backup
   drill — and a datacentre count that decides whether decision 3 gives it fast recovery or a
   restore. Decision 2 caps this at one for now; it does not reduce the per-region cost.

5. **Asking the operator to choose a region is a question at signup**, in a product whose wedge is a
   solo founder who may not know the answer (BET-0001). Decision 5 accepts that friction rather than
   guessing, because the guess is permanent.

6. **STD-0009's isolation matrix gains an axis.** ISO-005 requires every access path tested, not only
   the request path. Cross-region staff access is a control ADR-0041 says *"must be evidenced"*, and
   a region asking the control plane for an entitlement answer is a path no existing test covers.

**Gained**

- Tenant data remains shared-nothing across regions. No replication, no conflict resolution, no
  distributed transaction — the hardest class of problem in geo-distributed systems is absent by
  construction rather than solved.
- Deletion, revocation and staleness stop being three problems and become none, because there is
  nowhere for a stale copy to exist.
- Suspension is immediate and local, with no coordination window in which a tenant is stopped in one
  place and running in another.
- Availability and residency stop being in tension, once RFC-0014 separates region from datacentre.

## Alternatives considered

- **Replicate a read-only entitlement projection into each region**, stripped to non-personal facts,
  so no request ever leaves its region. Decision 1 in an earlier draft, recorded because the reasoning
  that killed it generalises: it buys latency and costs a propagation bound, a revocation lag, a
  second place a deletion request must reach, and a rule with an exception in it. A second exception
  after ADR-0047's staff directory would have made "nothing replicates" a preference, not a rule.

- **Regions ask the control plane and cache the answer.** The middle position: no durable copy, but a
  short-lived one. Rejected as the worst of both — it reintroduces staleness and a revocation window
  while keeping the dependency, and a cache with a long enough TTL to be useful is a projection with
  worse bookkeeping. Listed as decision 1's first reversal step rather than as a design.

- **A pair of regions replicating to each other**, so each covers the other's failure. Rejected twice
  over: it is a permanent cross-border transfer (decision 3), and RFC-0014's separation of region
  from datacentre means the availability it buys is available inside a region anyway.

- **A control plane per continent**, so each continent's operators are held locally. The intuitive
  answer to RFC-0017's problem, and it fails on ADR-0047's own argument: N control planes means N
  staff directories, N joiner processes, N places to forget a deprovision — the exact cost ADR-0047
  refused. It would also make an operator with businesses on two continents unrepresentable, and
  multiply decision 3's jurisdiction-loss risk by the number of continents rather than containing it.
  RFC-0017 gets the same residency outcome by moving the personal data down instead.

- **Place a tenant by rule** — residency requirement by law, otherwise nearest its users. This was
  decision 5 in an earlier draft. Rejected because the platform does not hold the facts the rule
  needs: whether this business has a residency obligation, where its users will actually be, and
  which wins. Asking is not a worse answer than inferring; it is the only correct one available.

- **Warm standby in a paired region.** Fastest recovery from losing a jurisdiction, and a permanent
  cross-border transfer requiring disclosure in every customer contract and a lawful pairing for each
  region as a standing obligation.

## Open questions

1. **When a tenant is suspended, is its edge routing entry withdrawn as well?** Suspension itself is
   settled and is not the question — see decision 4. What is open is whether traffic should still be
   allowed to *arrive*. Leaving the entry in place lets the region answer with a meaningful
   "suspended" response, which is what a customer in a billing dispute should see. Withdrawing it
   sheds the traffic before it reaches the region, which is what an abuse flood or a takedown order
   may require. These want opposite answers, so the trigger for each needs stating rather than a
   single default.

2. **Does the jurisdiction-loss risk in decision 3 carry a review date?** ADR-0045 records accepted
   risks without one; REQ- artifacts require a trigger. An accepted risk that nothing revisits is
   indistinguishable from one nobody noticed.

3. **May the region picker order its list by the signup address?** As a suggestion the operator
   overrides it is harmless; it is also the one place the inference decision 5 refuses could
   legitimately return, so it should be decided rather than left to whoever builds the form.

4. **Should BET-0001 gain a fifth assumption?** Its beachhead section lists four the contract may not
   bake in; *"that a business's data may live wherever we put it"* is the same shape as the other
   four — cheap to avoid now, structural to remove later. This is a bet edit, not part of this RFC,
   and it is recorded here so it is not lost.

## Reversal conditions

- **Decision 1 reverses** if measurement shows gated actions are frequent enough that the round trip
  to the control region dominates a user-visible workflow. The reversal is a cache, then a projection,
  in that order — and taking either step means accepting the staleness, deletion and revocation
  costs this decision exists to avoid, so it needs evidence rather than an intuition about latency.
- **Decision 2 reverses** on its stated trigger: a signed contract whose residency requirement the
  control region cannot satisfy.
- **Decision 3 reverses for a specific region** when that region has one datacentre and a customer
  contract requires a recovery time the restore path cannot meet. That is a per-region reversal, not
  a policy change, and it converts a cross-border pairing from avoidable to disclosable.
- **Decision 4 reverses the moment anything of a tenant's lives outside its region.** A cache, a
  projection or a cross-region index reintroduces exactly the coordination problem this design does
  not have, which is why decision 1 and decision 4 stand or fall together.

## Related

- RFC-0014 — the structure this RFC's behaviour rests on; not readable without it
- RFC-0016 — the edge, including the one global fact that lives outside the control plane
- ADR-0047 — the no-replication precedent decision 1 generalises, and the control region it establishes
- ADR-0041 — `tenant.region` as contractual and immutable, and cross-region staff access as evidenced
- ADR-0039 — the provisioning saga, and the one operation that spans both planes
- ADR-0045 — the per-region masterkey, and the practice of naming an accepted risk
- ADR-0024 — the operator-scope grant whose absence hazard decision 1's fail-closed rule reuses
- C-0006 — anti-goal 5 on enterprise isolation, anti-goal 6 on breadth, and the commodity distinction
- BET-0001 — the beachhead section, the solo founder decision 5 asks a question of, and open question 4
- REQ-0003 — item 8 owns residency; its Related section warns that projections mean copies
- REQ-0004 — item 10 owns SLOs and item 12 the RPO/RTO decision 3 makes urgent
- STD-0009 — the isolation matrix that gains a region axis
- GLO-0005 — absence must never produce breadth, which decision 1's fail-closed rule applies upward
- GLO-0006 — the worked example decision 5 borrows for why placement is never inferred
- ADR-0071 — the off-host store the 2026-09-24 revision reconciles with; its copy is the sealed copy this RFC now names
- ADR-0094 — Garage is in-cluster, so it does not survive the failure the sealed copy covers
- ADR-0105 — a sealed copy is not a second failure domain, and every attestation states the honest RPO
- RFC-0046 — at a tenant-held-key rung, a sealed copy is no longer proof of recoverability
- MKT-0003 — no blanket localisation in India; the boundary this decision defends is a product promise
- T-G.7 — the task that makes the database leg of the sealed copy real; the copy this RFC permits is not yet configured
- `archived/paridyum-docs/REGION-MODEL.md` — the country map, and the fallback that made it unsound
