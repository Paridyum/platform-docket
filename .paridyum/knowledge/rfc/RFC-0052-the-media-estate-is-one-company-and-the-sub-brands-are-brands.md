---
id: RFC-0052
title: The media estate is one company, the sub-brands are brands, and a disclosure gate precedes the first video
type: rfc
status: accepted
created: 2026-09-24
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [media, brand, corporate-structure, adr-0077, adr-0087, rfc-0013, ip, disclosure, trademark, bet-0008, youtube]
breaking: false
resolved_by: null
verified_by: []
enforced_by: []
---

# RFC-0052 — The media estate is one company, and the sub-brands are brands

## Status

**ACCEPTED**, 2026-09-24, by the operator under ADR-0087's personal authority. Proposed the same
day. Six decisions, D1–D6, all adopted. D6 was the urgent one; the other five are merely
important, and the difference is that D6 protects something that cannot be recovered once it is
lost.

| | Decision | State on acceptance |
| --- | --- | --- |
| D1 | One company: `PMD`, owned by `PAR` | **Live.** `registry/org.yaml` entry added on acceptance |
| D2 | Explained, Academy and Labs are brands, not companies | **Live.** No entity, no serial format |
| D3 | A sub-brand earns a company on a named trigger | **Expected to fire.** The operator confirms Academy will sell courses and certificates |
| D4 | Media is go-to-market cost until D3 fires | **Live** as posture; `go-to-market` slot still empty |
| D5 | Present assignment, Brand Accounts, contributor IPAs | **Owed.** Brand Account applies from the first channel created |
| D6 | Disclosure gate, including D6a for talks | **Live now.** Binds before the first video and before the first CFP |

**What acceptance costs, stated plainly:** one registry entry, and a question asked before each
publish. D6a's protocol is four items. Nothing here requires incorporation, and nothing waits for
it.

**What acceptance does not give `PMD`:** a signatory — and `docket check` calls that an **error**,
not a tolerable gap. Verified on acceptance:

    ERROR  PMD has no active signatory, so nothing can be issued for it (ADR-0077 decision 4)

This RFC's first draft claimed the entity could sit signatory-less as "the correct state for a
company whose parent has no CIN." **The tool disagrees, and the tool is right.** ADR-0077 decision
4 resolves signing authority within the engagement's company; a company that cannot answer who
signs for it *"would fail only at its first issuance"*, which is precisely the late failure the
check exists to pull forward. Registering the entity is what makes the missing officer visible.

So the register is **deliberately red** from acceptance until an officer is appointed. That
appointment owes what RFC-0042 §5 requires — a dated instrument, a registry entry citing it, and a
human approving it under C-0004 — and is authorised by Rohan Batra under ADR-0087 decision 1. It is
not made here, because making it would be exactly the unilateral act RFC-0042, C-0004 and ADR-0087
exist between them to prevent.

The ownership invariant and the serial-format uniqueness check both **passed** on the same run:
`PMD` reaches the root through `owned_by: PAR`, and `PMD/{kind}/{year}/{seq:04d}` collides with
nothing.

## The question

ADR-0106 accepted a three-sub-brand YouTube estate — Explained, Academy, Labs — and said nothing
about who owns it, who it bills as, or what it may say. The operator asks the structural question
directly: **is this one Paridyum Media, or is each brand its own company?**

The question has a house answer already half-written, and a risk nobody has named.

## What is already decided, and what it settles

Retrieved rather than reasoned about, per CLAUDE.md.

**ADR-0077 makes a company registry data and fixes five invariants.** One docket, one register,
many companies. Exactly one company declares no `owned_by` — that is the holding company — and
every other reaches it by following the chain, **checked by `docket check`** because decision 4
resolves *who may sign a subsidiary's documents* through that relationship. Each company carries
its own serial format, because two sharing one issues the same serial to two people. A person's
identifier is group-wide; only the *engagement* identifier carries the company. Shareholding is
deliberately not recorded (D6), because docket has no question that needs it.

**The register currently holds two companies**, and neither is incorporated:

| Code | Name | Tagline | Owner | Status |
| --- | --- | --- | --- | --- |
| `PAR` | PARIDYUM | Technology & Software | — (root) | Incorporation in Process, `cin: null` |
| `PCY` | PARIDYUM CYBER | Cyber Security | `PAR` | Incorporation in Process, `cin: null` |

**ADR-0087 states the brand rule in one clause:** *Paridyum is the main brand.* Governance
authority is personal and ordered — Rohan, then Priyanshu — and is **not** signing authority,
which belongs to an office.

**PCY is the precedent, and reading it correctly is the whole answer.** Paridyum Cyber is not a
second company because cyber security is a different *brand*. It is a second company because it is
a different **line of business**: different customers, different engagements, its own CEO in Ruhani
Khurana, its own officer set, its own serial namespace. The brand followed the business. It did not
cause it.

## The proposal

### D1 — One company: PARIDYUM MEDIA, code `PMD`, owned by `PAR`

A single entity holds the entire media estate. It takes an `org.yaml` entry with its own address,
jurisdiction, legal status, POSH routing, verification contact and serial format
(`PMD/{kind}/{year}/{seq:04d}`), and `owned_by: PAR`, which is what gives it an answer to who may
sign for it.

**Why a company at all, rather than a division of `PAR`.** Three reasons, none of them branding:

1. **Different counterparties.** A sponsor contract, a platform's monetisation terms, a music
   licence and a freelance editor's agreement are not the counterparties `PAR` was shaped for, and
   they arrive with their own indemnities.
2. **Different liability surface.** Publishing carries defamation, copyright, advertising-disclosure
   and platform-takedown exposure that has nothing to do with running a multi-tenant runtime.
   Keeping it off the entity that will hold the patent and the customer contracts is the entire
   reason subsidiaries exist.
3. **The precedent already accepts the cost.** `PCY` established that a distinct line of business
   gets an entity here. Media is at least as distinct from software as cyber security is.

### D2 — Explained, Academy and Labs are brands inside `PMD`, not companies

They are not companies, they do not get `org.yaml` entries, and they do not get serial formats.

**The test for a company is not distinctness of brand. It is distinctness of counterparty,
liability, revenue contract, or regulatory surface.** Applied to the three sub-brands, all four
come back identical: one bank account, one GST registration, one sponsor pipeline, one set of
contributor IP assignments, one creator. Three entities would triple the compliance and audit
surface and buy exactly nothing — and under ADR-0077 each would need its own officer set and
fallback order, which currently means naming signatories for companies that sign nothing.

**ADR-0077 D5 makes the cost concrete rather than theoretical.** Every company needs a distinct
serial format and its own per-kind counter. Three media companies means three serial namespaces
for documents that are the same document.

**ADR-0087's "Paridyum is the main brand" already settles the naming.** These are endorsed
sub-brands — *Paridyum Explained*, *Paridyum Academy*, *Paridyum Labs* — never standalone marks.
ADR-0106's architecture is already correct on this point and needs no change.

### D3 — A sub-brand earns a company on a named trigger, and only then

So that D2 is a decision with an exit rather than a permanent ceiling. A sub-brand is promoted to
its own entity when **any one** of the following becomes true:

1. It contracts with **end customers in its own right** — paid courses, paid memberships, ticketed
   events. That is a consumer relationship with refunds, service-tax treatment and complaint
   handling, and it is a different business from advertising revenue.
2. It attracts a **regulatory surface the others do not have** — education, financial advice,
   anything ADR-0041's sequencing puts behind controls that do not exist.
3. It takes **outside capital or a partner** who needs to own a share of that thing and not the
   rest.
4. Its liability profile diverges sharply enough that a claim against it would threaten the others.

**Academy is the likeliest to fire this**, and (1) is the trigger it fires. Explained and Labs
monetise through advertising and sponsorship — `PMD` is the counterparty. An Academy that sells a
course is contracting with thousands of individuals, and that is a different company.

**Confirmed by the operator on 2026-09-24: Academy will sell courses and certificates.** That moves
trigger (1) from hypothetical to expected, and it adds a second surface trigger (2) did not
anticipate. **A certificate is an issued document**, and docket already knows that shape exactly —
`lifecycle.yaml`'s `issued` kind, a serial from the company's own format, a register entry, and a
verification contact a third party checks it against. A course certificate that is not issued that
way is a PDF with a name on it, and the difference is whether it means anything in five years.

Two consequences follow and neither is decided here. **A certificate issued under `PMD`'s serial
format is issued by `PMD`**, which is an argument for promoting Academy on incorporation rather
than later, since re-issuing certificates under a new entity is not a thing that can be done
cleanly. And **certification carries a representation about the holder's competence**, which is a
different claim from anything Explained or Labs makes and is the closest this estate comes to
STD-0021's honesty floor binding on a product rather than a page.

### D4 — The media estate is a cost of go-to-market until it is not, and BET-0008 is why

BET-0008 holds that the estate is cost of goods, not a revenue line. Media revenue would be the
first income Paridyum earns that is not catalogue revenue, and it would arrive **before** the
catalogue earns anything — which makes it the easiest possible thing to mistake for traction.

Proposed: media spend and media revenue are booked as **go-to-market cost and its offset**, not as
a revenue line, until D3 fires on a sub-brand. `product-registry.yaml`'s empty `go-to-market` slot
is where this belongs.

This is an accounting posture with a strategic purpose. It stops a channel that is doing well from
being read as a business that is doing well, which is exactly the misreading BET-0005's DRIFT
trigger was written to catch one level up.

### D5 — Channel ownership, and the assignment nobody has signed

**Nothing is incorporated.** `PAR` and `PCY` both read `legal_status: Incorporation in Process`
with `cin: null`. `PMD` cannot be incorporated ahead of its parent, so the media estate will be
built, branded and possibly launched by a natural person before any company exists to own it.

RFC-0013 D4 already decided the general answer and it applies unchanged here: work created before
incorporation belongs to its author personally and **does not transfer by operation of forming a
company**. Incorporation documents must carry a **present** assignment — *hereby assigns*, not
*agrees to assign*.

What the media estate adds is that the assets are unusually easy to strand:

- **A YouTube channel created on a personal Google account is personally held**, and channel
  ownership transfer is a platform operation with its own failure modes. Create every channel
  under a **Brand Account** from the first minute, so that ownership is a permission rather than an
  identity. This costs nothing today and is a migration later.
- **Handles, domains and marks** claimed personally are the same problem in a different registry.
- **Footage, scripts, music licences and thumbnails** are copyright works with an author, and a
  freelance editor owns their contribution absent a written assignment. §27 of the Contract Act
  makes non-competes largely unenforceable in India (RFC-0013), so the assignment and
  confidentiality clauses are the only terms doing real work — and docket already issues an `IPA`.
- **Contributor documents exist and should be used.** `lifecycle.yaml` already issues NDA, IPA and
  agreement documents. An editor is an engagement, not an informal arrangement.

### D6 — A disclosure gate precedes the first video, and this is the urgent one

**This is the finding this RFC exists for.**

RFC-0013 establishes, as accepted law-specific findings: a resident of India may not file abroad
first (§39, criminal, up to two years); §3(k) grants on measurable technical effect; and — the one
that matters here — **India's §31 grace period covers a paper read before a learned society and
essentially nothing else.** RFC-0013 states it plainly: *"For every disclosure channel Paridyum is
realistically going to use, the Indian grace period does not exist."* And: *"A public disclosure
cannot be withdrawn. There is no undo, in any jurisdiction, ever."*

**ADR-0106 has just commissioned a public disclosure channel and pointed it at infrastructure.**

The collision is specific, not general:

- RFC-0013 identifies **exactly one** patent candidate in the estate: the **nested tenancy
  isolation mechanism** — RFC-0007's inner silo, ADR-0024's decisions, the complete authorisation
  matrix across two nested discriminators, with density, query overhead and p99 as its measurable
  effect. C-0006 has already claimed it as non-commodity and BET-0004 stakes the company on it.
- RFC-0013 is equally plain that **BET-0004's catalogue is a business method** and unpatentable
  under §3(k)'s absolute bar. There is nothing to protect there.
- So the estate has **one** thing a video can destroy, and it is precisely the thing an
  infrastructure channel would find most interesting to explain.

**Labs is the sharpest edge, and it is worth being explicit about why.** Explained-Technology
explaining how Kubernetes works discloses nothing of Paridyum's. A Labs episode called *I Built My
Own Private Cloud*, filmed on `infra-substrate`, is a screen recording of the mechanism. ADR-0106
puts Labs in phase 3, which buys time — but the ideation attached to that ADR argued for pulling
Labs forward precisely because its raw material already exists, and **that argument is now
constrained by this one**.

**RFC-0013 D5 deferred the disclosure ledger behind a two-part trigger**: the first patent
candidate, or the first non-founder with access. A public channel is a third trigger that D5 did
not anticipate, and it is the only one of the three that fires **before** anyone would think to
look for it — because the first video is published by the founder, to no non-founder, with no
candidate yet docketed.

Proposed, and deliberately minimal so that it is actually used:

1. **One question before publish**, answered in the episode's record: *does this describe how
   Paridyum's runtime achieves isolation, density, or any effect C-0006 claims as non-commodity?*
   Yes or unsure stops the publish and escalates to Rohan under ADR-0087's personal authority.
2. **The bright line: other people's technology is free, ours is gated.** Explaining Kubernetes,
   Linux, DNS, GPUs, LLMs — unrestricted, and that is the whole of ADR-0106's launch slate.
   Explaining *our* mechanism — gated, whatever the sub-brand.
3. **Architecture decisions are publishable; implementations of the candidate are not.** The
   ideation behind ADR-0106 proposed a *"The Decision"* format built on the estate's own ADRs, and
   it is the strongest content idea available — so it needs its boundary drawn now rather than
   during production. An ADR that decides a *commercial or structural* question (ADR-0105's declined
   failure domain, ADR-0104's supersession) discloses nothing patentable. An ADR that specifies the
   *isolation mechanism* — ADR-0024, RFC-0007 — is the candidate itself.
4. **Filming a screen is publishing what is on it.** A terminal in the background of a Labs episode
   is a disclosure with no author and no review.
5. **This does not build RFC-0013 D5's full docket.** Five questions and an escalation path, not a
   registry with zero entries. D5's machinery still waits for its own trigger.

#### D6a — Conference and dev talks, which are a harder case than video

The operator asks, on 2026-09-24, whether a talk of the familiar kind — *how we run Kubernetes
clusters across regions* — is acceptable, states that no internal working is to be disclosed, and
asks for the line to be governed rather than judged each time. It is the right question, because a
talk is **more dangerous than a video in three specific ways** and less dangerous in none.

1. **Q&A is unscripted and unrecoverable.** A gate can review a script. Nothing reviews the answer
   to *"but how do you actually keep tenants apart?"*, asked by the one person in the room who
   wants to know, thirty seconds after the talk that made them want to.
2. **The organiser owns the recording.** It is published on their schedule, under their terms, and
   cannot be taken down on ours.
3. **The audience is selected for exactly the wrong thing.** A conference room is filled with the
   people most capable of reconstructing a mechanism from a partial description — which is the
   definition of an enabling disclosure.

**The line is not the topic. It is the layer.** This matters because "Kubernetes across regions"
sounds safe and can be made unsafe in one sentence.

| Tier | What it covers | Disposition |
| --- | --- | --- |
| **Free** | Off-the-shelf components and how they are operated: Kubernetes topology, multi-region layout, failover, DNS, Terraform, Proxmox, Zitadel as deployed. Anyone's stack. | **Unrestricted.** This is the talk the operator describes and it is fine. |
| **Gated** | Anything about how *tenants and accounts are separated inside* that infrastructure — the inner silo, the nested discriminator, the authorisation matrix. | **Stops. Escalates to Rohan.** This is RFC-0013's candidate. |
| **Never, before filing** | The **numbers**. Tenant and account density per node, query overhead of the nested discriminator, p99 under load. | **Refused outright until the Indian application is filed.** |

**The third row is the one that would be missed, and it is the sharpest trap in this document.**
RFC-0013 is explicit that §3(k) grants on *measurable* technical effect, that *"improves
performance" fails*, and that metrics are expected in the claim itself. Those four numbers are not
evidence supporting the invention — **they are the invention's technical effect**, which is to say
they are the claim. A slide reading *"we run N tenants per node"* discloses the patentable part
while saying nothing about the mechanism, and it will feel like the safest slide in the deck.

**The gate is temporal, not permanent, and that is the part worth holding onto.** RFC-0013 D4 fixes
the sequence: build and measure → incorporate → DPIIT → file. Once the Indian application secures a
priority date, most of the gated tier opens. **The rule is therefore "not yet", not "never"** — and
the fastest route to giving the talk in full is to finish T-0.8, not to argue about the line.

**One legitimate channel exists for the mechanism itself, and it is worth naming because it is the
only one.** RFC-0013 found that §31's grace period covers a paper read before a **learned society**
and published in that society's transactions. A developer conference is not one. An academic venue
plausibly is. So if the mechanism is ever to be described publicly before filing, the route is a
paper at a learned society with Form 31 filed under Rule 29A — not a keynote. That route is
narrow, slow, and real, and it is the only one that does not forfeit the claim.

**Protocol, kept to four items so it is used rather than admired:**

- **The deck is reviewed before it is submitted**, against the table above, not before the talk. A
  CFP abstract is already a commitment.
- **Two prepared deflections**, rehearsed, for the isolation question that will be asked: what the
  system guarantees, and that how it is achieved is not something being discussed today. Refusing
  cleanly is normal at this level and costs no credibility.
- **No live terminals, no unredacted dashboards, no screen-shares of the estate.** D6 item 4
  applies to a stage more than it applies to an edit suite, because nothing is cut afterwards.
- **A one-line record per talk** — venue, date, tier claimed, who cleared it. This is D5's
  disclosure ledger in the smallest form that would still answer an examiner, and writing it costs
  a minute now and cannot be reconstructed later.

## Costs, stated

- **A third entity is a third set of filings, a third audit, a third compliance calendar** — for a
  business with no revenue and one operator. D1 is a real cost and the liability argument is what
  pays for it. If the media estate never publishes, this cost was wasted.
- **D6 slows the most interesting content**, and it does so permanently rather than temporarily. The
  most valuable thing Paridyum could make — the estate explaining its own systems work — is the
  thing most likely to hit the gate. That tension does not resolve; it gets managed.
- **D2 will feel wrong the first time Academy outgrows the others**, which is why D3 exists.

## Alternatives considered

### A — One company per sub-brand: Paridyum Explained Ltd, Academy Ltd, Labs Ltd

**Rejected.** This is the operator's stated alternative and it is the one worth killing carefully,
because the instinct behind it is sound: the three sub-brands *are* genuinely distinct, and ADR-0106
spends its length establishing exactly how distinct.

They are distinct in **editorial intent**, which is what ADR-0106 decided and is the right axis for
brand architecture. It is not the axis a company is drawn on. A company is drawn on counterparties,
liability, revenue contracts and regulatory surface — and on all four the three sub-brands are the
same business filmed three ways. The structure would create three sets of officers under ADR-0077
D4 for three entities that sign nothing, three serial namespaces under D5 for identical documents,
and three compliance calendars for a group whose root company has no CIN yet.

The deeper objection: it would **encode the brand architecture in the corporate register**, where
changing it is a filing rather than an edit. ADR-0106 explicitly keeps its verticals open and
expects the list to grow. A structure that makes each growth a company formation is a structure that
will quietly discourage the growth, or be abandoned the first time it is inconvenient.

### B — No media company: the estate is a division of PARIDYUM

**Rejected**, and this is the closest call in the document. It is free, it is simple, and while
nothing is incorporated it is indistinguishable from D1 in practice.

It is refused on **liability**, not on tidiness. `PAR` is the entity that will hold the patent
filing RFC-0013 sequences, the catalogue contracts BET-0004 stakes the company on, and the tenant
relationships BET-0005 builds. Publishing exposure — defamation, copyright claims, advertising
disclosure, a takedown that becomes a dispute — has no business sitting on that entity. The whole
purpose of a subsidiary is to stop an unrelated claim reaching the thing that matters.

Its real advantage is preserved anyway: D1 decides *one company*, and explicitly does not decide
*when it incorporates*. Running as a division until incorporation is the operating reality this RFC
does not fight.

### C — A media company under PARIDYUM CYBER rather than under PARIDYUM

**Rejected**, and it is recorded because ADR-0077 D5 makes it legal: `owned_by` names a company
rather than asserting the root, so ownership may run through an intermediate and `docket check`
would follow the chain.

There is no reason to use that capability here. It would route media's signing authority through
Paridyum Cyber's officers under D4, which puts Ruhani Khurana in the path of media documents for no
stated reason, and it asserts a relationship between cyber security and publishing that does not
exist. Intermediate ownership should express something true; here it would express nothing.

### D — Publish under the `PARIDYUM` house brand with no sub-brands at all

**Rejected as already decided.** ADR-0106 settled the sub-brand architecture and this RFC does not
reopen it. Recorded only to be explicit that the corporate decision is not a back door to revisiting
the brand one: D2 deliberately places the sub-brands *inside* one company precisely so that ADR-0106
remains free to change without any corporate consequence.

### E — Defer the disclosure gate until RFC-0013 D5's trigger fires

**Rejected, and it is the alternative this RFC is most confident about.**

D5's trigger is the first patent candidate or the first non-founder with access. The argument for
waiting is consistency: D5 reasoned that a registry with zero entries governs nothing, and that is
correct reasoning that this RFC does not dispute.

It fails here for one asymmetry. **Every other mechanism D5 defers is recoverable when the trigger
fires; this one is not.** RFC-0013 says it in its own words — *a public disclosure cannot be
withdrawn, there is no undo, in any jurisdiction, ever*. A disclosure ledger built late can be
backfilled. A video published early cannot be unpublished in any sense the Patents Act recognises.

D6 is therefore deliberately *not* D5's machinery. It is five questions and an escalation path,
which is the smallest thing that addresses an irreversible risk, and it leaves D5's docket waiting
for D5's own trigger.

## What this does not decide

1. **Whether `PMD` incorporates at all, or waits.** D1 says one company; it does not say when. The
   sequence in RFC-0013 D4 is build → incorporate → DPIIT → file, and media has no filing.
2. **Trademark strategy.** `PARIDYUM` presumably files in classes 9 and 42 for software; media adds
   **class 41** (education and entertainment). Whether Explained, Academy and Labs are filed at all
   or ride the house mark is open, and public use on YouTube creates evidence of use and exposure at
   the same time. This needs its own artifact.
3. **Who runs it.** `PMD` needs at least one signatory under ADR-0077 D4. Every existing officer's
   appointment instrument is already outstanding (`appointed_on: null`, `instrument: null`), so a
   third company inherits that debt rather than creating it.
4. **Whether media revenue is ever a revenue line.** D4 sets the posture until D3 fires and no
   further.
5. **The registry change itself.** `org.yaml` gains an entry only on acceptance. Governance precedes
   implementation.
