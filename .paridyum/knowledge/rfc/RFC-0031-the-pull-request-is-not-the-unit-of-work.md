---
id: RFC-0031
title: The pull request is not the unit of work — stacks, a work-in-progress ceiling, and automatic draining
type: rfc
status: proposed
created: 2026-08-19
locked_at: null
supersedes: null
superseded_by: null
resolved_by: null
scope: [org]
applies_to: ["*"]
tags: [pull-requests, stacked-prs, review, wip, merge-queue, change-lifecycle, std-0007, adr-0012, adr-0013, req-0011, github]
breaking: true
enforced_by: []
sources:
  - url: https://github.blog/changelog/2026-07-30-stacked-pull-requests-are-now-in-public-preview/
    title: "GitHub Changelog — Stacked pull requests are now in public preview"
    retrieved: 2026-08-19
    supports: >
      Stacked pull requests entered public preview on 2026-07-30, free on every repository,
      installed with `gh extension install github/gh-stack`. Merging a layer lands every
      unmerged layer below it. Merge queue support was "rolling out over the coming weeks"
      as of that date.
  - url: https://docs.github.com/en/pull-requests/reference/stacked-pull-requests
    title: "GitHub Docs — Stacked pull requests (reference)"
    retrieved: 2026-08-19
    supports: >
      Cross-fork stacks are NOT supported — all branches must be in the same repository.
      Branch protections, required reviews, status checks and CODEOWNERS are evaluated
      against the stack's base branch. A merge group may exceed its configured maximum by
      50% to keep a stack intact, and a stack larger than that is split across consecutive
      groups. If a pull request is ejected from the queue, every pull request above it in
      the stack is ejected too. Linear history across the stack is mandatory. Not supported
      in GitHub Desktop.
  - url: https://docs.github.com/en/pull-requests/reference/stacked-prs-cli-commands
    title: "GitHub Docs — Stacked pull requests CLI commands"
    retrieved: 2026-08-19
    supports: >
      `gh stack init | checkout | modify | rebase | merge`. Requires gh 2.0 or later.
      `gh stack merge` defers to the merge queue when the base branch has one, ignoring
      merge-method flags with a warning.
  - url: https://docs.github.com/en/pull-requests/reference/stacked-pull-requests-apis-and-webhooks
    title: "GitHub Docs — Stacked pull requests APIs and webhooks"
    retrieved: 2026-08-19
    supports: >
      A REST Stacks API lists, reads, creates, extends and dissolves stacks; a `stack` object
      on pull request resources carries stack number, size, position and base. GraphQL exposes
      `stack` and `stackEntry` on `PullRequest`, read-only. Webhook `pull_request` payloads
      carry a `stack` property. This is what makes automated enforcement possible at all.
  - url: https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue
    title: "GitHub Docs — Managing a merge queue"
    retrieved: 2026-08-19
    supports: >
      Merge queues are available in public repositories owned by an organisation, or in PRIVATE
      repositories owned by organisations using GitHub Enterprise Cloud. Not Free, and NOT Team.
      ADR-0004's stated upgrade path is Free to Team, which therefore does not unlock this.
  - url: https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/incorporating-changes-from-a-pull-request/automatically-merging-a-pull-request
    title: "GitHub Docs — Automatically merging a pull request"
    retrieved: 2026-08-19
    supports: >
      "The option to enable auto-merge is shown only on pull requests that cannot be merged
      immediately." Auto-merge therefore requires branch protection to create something to wait
      for. On a plan with no branch protection every pull request is immediately mergeable and
      the option never appears.
---

# RFC-0031 — The pull request is not the unit of work

## Amendment — 2026-08-19, before acceptance

**The first draft of this RFC proposed an enforcement mechanism that cannot be built on this
organisation's GitHub plan.** Found by `knowledge-audit` within the hour, against ADR-0004, which
is `locked` and states plainly that Free *"cannot block a merge on this plan"* and that protected
branches and required reviews are Team-plan features for private repositories.

Every mechanism the first draft named — a required status check, GitHub's auto-merge, and the
merge queue — depends on branch protection. None of the three is available. The merge queue is
worse still: it is private-repo-only on **Enterprise Cloud** (source 5), so ADR-0004's own
Free-to-Team upgrade path would not unlock it either.

**How it was missed is worth recording, because the failure is general.** The `design-change`
procedure's step 2 is "retrieve before designing" and calls it non-negotiable. It was performed —
`grep -rl "pull request\|pull_request" constitution/ adr/ standards/ anti-patterns/` — and it did
not match ADR-0004, which says *"PR"* and *"status check"* throughout and never spells the phrase
out. **A retrieval grep that searches for one spelling of a concept will miss the artifact that
uses another, and it fails silently, returning a clean-looking list.** The lesson belongs in
STD-0011: retrieve on the *mechanism* a design depends on — here, "status check", "branch
protection", "merge" — not only on its subject matter.

**What changed:** §Proposal B and C are rewritten from gates to actors. §Proposal D gains the plan
consequence it hid. Consequences and Open questions are revised. §Problem, §Alternatives and
§Proposal A are unchanged — the problem and the reasoning were not what was wrong.

## Problem

**STD-0007 predicted this and accepted it without a mitigation.** Its Costs accepted §2, written
2026-08-07 and locked the same day:

> *"Small PRs mean more PRs. More CI runs, more review context-switching, more merge ordering to
> think about. This is a real cost paid to avoid a worse one."*

That cost is now the binding constraint on the work about to start. Observably:

- STD-0007 caps a reviewable slice at **400 changed lines**, error severity. `tool-bench` alone —
  three tenancy arms, a driver, a workload, provenance capture, an isolation suite — decomposes
  into roughly six to nine slices under that budget. Phase 3a's four `platform-` repositories are
  larger.
- **Nothing anywhere caps how many slices may be in flight at once.** There is no artifact in
  `knowledge/` that limits open pull requests, expires an abandoned one, or says when a branch
  stops being work-in-progress and becomes debris. The grep returns empty.
- **Nothing merges anything automatically.** Every landing is a manual act, so the open count only
  falls when a human remembers to make it fall.
- The repository currently operates by committing to `main`, which is why this has not yet hurt.
  STD-0007 Costs accepted §5 records that its own check *"is unenforced for work committed straight
  to `main`, which is how this repository currently operates."* The first repository that adopts
  pull requests properly is the first one exposed to this.

**What happens if nothing is done:** the number of open pull requests grows monotonically, because
opening one is cheap and closing one is not, and nothing in the system applies back-pressure. The
end state is a queue nobody reads, which is the same failure ADR-0018 names for triggerless
requirements and ADR-0035 names for hand-maintained counts — a status that cannot be trusted
poisons the ones that can.

**How we will know it is fixed:** the count of open pull requests has a ceiling that is enforced
rather than intended, and the count falls without anyone deciding to make it fall.

## Constraints

Locked artifacts that bind this design. None is negotiable here; where one is contradicted, this
RFC proposes a supersession rather than an exception.

| Artifact | State | What it binds |
| --- | --- | --- |
| **ADR-0013** | `locked` | Stage 4: *"Implement — small PRs referencing the RFC — each independently reviewable."* Untouched by this RFC |
| **STD-0007** | `locked` | The 400-line budget, the slice document, `Depends on:` ordering, the `Review-budget-exception:` hatch. Untouched — this RFC sits beside it, in exactly the relationship STD-0007 itself has to ADR-0013 |
| **ADR-0012** | `locked` | §4: contribution happens **from a fork**. **Directly contradicted.** See §Proposal D |
| **ADR-0004** | `locked` | The org is on **GitHub Free**. Private repositories get no branch protection, no required status checks and no merge queue. *"CI on every PR and push to `main`. Cannot block a merge on this plan."* **The binding constraint on this entire design** |
| **ADR-0002** | `locked` | The CI gate is the lock. A rule here that cannot be mechanically evaluated must not be written as though it can |
| **ADR-0014** | `locked` | A name in `enforced_by` is a commitment, not a control. `declared` is a legitimate state |
| **REQ-0011** | `deferred` | *"A change is small enough to review, and reviewed by someone who did not write it."* Trigger: a second person or agent identity. **Not yet fired** |
| **ADR-0025** | `locked` | Session work is checkpointed automatically, so commit count carries no information — every rule here measures branches and pull requests, never commits |
| **C-0004** | — | Generated output is regenerated, never hand-edited. Slice documents, and therefore stack definitions, are generated |

**ADR-0004 determines what enforcement can mean here, and it is the constraint this RFC's first
draft violated.** On GitHub Free with private repositories there is no such thing as a blocking
check. A workflow may go red, and the merge proceeds anyway. ADR-0004 accepts this openly — *"the
lock is advisory-plus-friction, not a hard block"* — and names the compensating control: a sweep
that catches violations after the fact, which it promotes from nice-to-have to a v0.2 requirement
*"because on this plan it is the only backstop."*

**So a rule enforced here is a rule something acts on, never one something blocks.** That is not a
weaker design. A gate refuses an action; an actor performs one, and closing a pull request that
breaches the ceiling both enforces the rule and reclaims the slot, which no gate does.

**REQ-0011 being `deferred` is the single most load-bearing constraint in this document, and it is
easy to miss.** It says plainly: *"Review rules with one person are theatre. Written now, they
would be waived on day one."* With one contributor, a pull request is **not** a request for
review. Nobody is waiting to approve it. Its entire job is to run the checks that only fire on
`pull_request` — the set is whatever `rust-ci.yml` conditions on that event, not a number worth
copying here — and to leave a signed, dated record.

A framework that models a human review queue would therefore be building for a team that does not
exist — and would be waived on day one, exactly as REQ-0011 predicts. **This design assumes no
human approver, and states what changes when one appears.**

## Proposal

Four parts. A, B and C are additive. D is a supersession and is the expensive one.

### A. The unit of work is a stack, not a pull request

GitHub shipped stacked pull requests to public preview on 2026-07-30 (source 1). A stack is an
ordered chain of pull requests, each based on the one below it; each layer is reviewed against its
own diff, and merging a layer lands every unmerged layer beneath it in one operation.

**STD-0007 already defines stacks without using the word.** Its slicing rule 5:

> *"Ordering is declared, not discovered. `Depends on:` names the slices that must merge first. A
> dependency found during review is a decomposition error."*

A slice document's `Depends on:` graph, restricted to a single requirement, **is** a stack
definition. The `slice-requirement` procedure already emits ordered slices, one per pull request.
Adopting stacks therefore adds no planning step — it gives an existing declaration a mechanical
representation, and `gh stack` maintains the rebases that declaration implies.

**What this is not.** Stacking is a *review-granularity* mechanism. It does nothing about the
number of open pull requests — a six-slice change becomes six open pull requests rather than one.
On the problem this RFC states, **stacking alone makes the number worse.** It is proposed because
it makes B and C coherent, not because it solves the stated problem. Saying otherwise would be the
kind of claim this repository catches in its own artifacts.

### B. A work-in-progress ceiling, enforced

**At most 2 open stacks per contributor, at most 5 layers per stack.** A hard ceiling of 10 open
pull requests per contributor, org-wide.

This is the part that actually answers the problem. The numbers are defensible rather than derived,
and are stated in the standard that follows this RFC (§Rollout) so they can be changed in one line
— the treatment STD-0007 gave its own 400, and for the same stated reason.

- **5 layers** because a stack deeper than that cannot be rebased without the lower layers having
  moved under review, and because 5 × 400 lines is already 2,000 lines of work in flight.
- **2 stacks** because one is a policy against ever pausing work, and three is not a limit.

**Enforcement is an actor, not a gate, because ADR-0004 leaves no gate available.** A workflow on
`pull_request.opened` reads the stack from the REST Stacks API (source 4), and when the ceiling is
already full it **closes the new pull request** with a comment naming the rule and the stacks
currently holding the slots.

Closing rather than blocking is what the plan permits, and it is also the better mechanism. A
required check refuses a merge and leaves the pull request open, which on this problem is
precisely the wrong outcome — the thing being limited is the open count, so an enforcement action
that adds to it enforces nothing. Closing reclaims the slot in the same act that enforces the rule.

**This is the only rule that acts destructively.** Everything else comments or warns. The
asymmetry justifies it: a false positive costs one reopened pull request, a false negative costs
the entire problem this document exists to fix. The branch is never touched, so nothing is lost.

### C. The queue drains without a human

A ceiling with no drain is a stop-work order. Three mechanisms, in order of importance:

1. **A merge actor lands the bottom layer.** GitHub's own auto-merge is unavailable — it is shown
   *"only on pull requests that cannot be merged immediately"* (source 6), and with no branch
   protection every pull request here is immediately mergeable, so the option never appears. The
   merge queue is unavailable too: private repositories need Enterprise Cloud (source 5).

   So a scheduled workflow does it. It lists open stacks, finds each bottom layer whose checks have
   concluded green, and merges it with `gh pr merge`; the layers above rebase and retarget as GitHub
   handles stack merges (source 1). **No human action is required to reduce the open count**, which
   is the property that matters, and it is obtained without either unavailable feature.

   **The merge actor is the one piece of automation that must be conservative.** It merges only on
   *concluded and green*, never on *not yet failed*, because on this plan nothing else stands
   between it and `main`.
2. **Expiry.** A layer with no push for **7 days**, or a stack whose bottom layer has not merged in
   **14**, is closed automatically. The branch is retained — closing a pull request is reversible,
   and the rule is about the queue rather than the work.
3. **Stacks are opened only against a slice document.** `pr.slice_cites_requirement` exists today
   at `warn` severity. This RFC does not raise it. Raising it is a separate, smaller change and
   bundling it here would hide it.

**The merge queue's eviction behaviour does not bite today, and will the day a queue exists.**
Documented rather than guessed (source 2): a stack enters the queue together, a merge group may
exceed its configured maximum by 50% to keep a stack intact, and **if any pull request is ejected
from the queue, every pull request above it is ejected too** — one flaky check on layer 1 evicts
layers 2 through 5. With no queue on this plan the hazard is dormant, and it is recorded here
because the 5-layer cap is sized for it: the cap should not be raised later on the grounds that
nothing has gone wrong, when the mechanism that would make it go wrong is simply absent.

### D. ADR-0012 §4 is contradicted and must be superseded

**GitHub's documentation is unambiguous** (source 2): *"Stacked pull requests require all branches
to be in the same repository. Cross-fork stacks are not supported."*

ADR-0012 §4 is equally unambiguous, and it is `locked`:

> *"contributor forks svc-x → pushes to their fork → opens PR to Paridyum/svc-x"*

**These cannot both hold.** A contributor working from a fork, as ADR-0012 requires, cannot open a
stack, which this RFC makes the unit of work.

This is invisible today: the founder has write access, and is the only contributor. It becomes
visible on the day a second contributor arrives — **which is also REQ-0011's trigger and RFC-0013
D5's trigger.** That is the day ADR-0012 exists for, and discovering the contradiction then means
resolving it while also onboarding a person.

**Proposed: a superseding ADR, written now, while the estate is empty.** This is RFC-0013's own
argument reused, and it held there for the same reason — *"every mechanism proposed below is nearly
free while the estate is empty and either expensive or impossible once it is not."*

The superseding ADR must preserve what ADR-0012 was actually protecting, which is not forks. Forks
were the *mechanism*; the protected property is that **an untrusted contributor cannot push to a
Paridyum repository.** A branch-protection model preserves that property while permitting stacks:

- Base permission `None` and required 2FA — **unchanged**, ADR-0012 §1 and §2 carry over verbatim.
- A **trusted** contributor gets write, and may push only to branches they own, with `main`
  protected and every gate required. They can stack.
- An **untrusted** contributor keeps read plus fork, contributes flat pull requests under
  STD-0007, and cannot stack. This is a real capability difference and must be stated as one.
- The trust boundary is a decision with a named owner, not a default.

**The supersession costs money, and the first draft hid it.** ADR-0004's first upgrade trigger is
*"a second person gains write access to any repository"* — which is exactly what §D grants. So
superseding ADR-0012 fires ADR-0004's trigger and mandates the move to GitHub Team. That is a
defensible thing to spend on: Team restores branch protection and required checks, which would let
the actors of §B and §C be reinforced by real gates rather than replacing them. But it is a cost
this RFC creates, and it must be visible in the decision rather than discovered at the invoice.

**It does not buy a merge queue.** Merge queues on private repositories require Enterprise Cloud
(source 5), which is a much larger step. The merge actor of §C.1 is therefore the design for the
foreseeable term, not a stopgap until a plan change.

**The superseding ADR is not written in this RFC.** It is a separate artifact requiring its own
review, and drafting a supersession of a locked ADR inside an RFC about pull request throughput
would be a consequential decision taken in the wrong place — the failure ADR-0013 exists to
prevent.

## Alternatives considered

**1. Do nothing; keep flat pull requests under STD-0007.** Costs nothing, adds no dependency on a
three-week-old preview API, and creates no conflict with ADR-0012. Rejected because it leaves the
stated problem entirely unaddressed: STD-0007's accepted cost stays accepted, and the open count
still has no ceiling and no drain. It is worth recording that **this alternative is strictly better
than adopting A without B and C** — stacking alone increases the open count, so a partial adoption
of this RFC is worse than rejecting it.

**2. WIP ceiling and auto-merge only; no stacking.** Solves the stated problem — the ceiling and
the drain are what do the work — with no preview-API dependency and no ADR-0012 conflict. This is
the strongest rejected alternative and it nearly won. Rejected because dependent slices are the
common case under STD-0007's `Depends on:`, and without stacks each dependent slice must either
wait for its predecessor to merge (serialising the work the ceiling was meant to let flow) or be
opened against a non-`main` base and manually retargeted on every merge. The manual retarget is
exactly the labour `gh stack rebase` removes. **If the preview API proves unstable, this is the
fallback**, and it is a fallback rather than a rewrite because B and C do not depend on A.

**3. A third-party stacking tool — Graphite, `git-spice`, `spr`, `ghstack`.** More mature than a
three-week-old preview. Rejected on REQ-0012's exitability test, which already killed Sidero Omni
for Track B: a commercial stacking tool becomes a dependency on the critical path of every change
in the company, and the free tiers are the ones that move. GitHub's native implementation is on the
platform the repositories already require, and its failure mode is degrading to flat pull requests
rather than losing the tooling.

**4. Trunk-based development; no pull requests at all.** Which is what this repository does today.
Rejected because it makes six of `rust-ci.yml`'s paths permanently unreachable — `change lifecycle`,
`regression test on a fix` and patch coverage are all `github.event_name == 'pull_request'` — and a
gate that cannot run is the false confidence ADR-0002 exists to prevent. It also produces no
reviewable record on the day REQ-0011 fires.

## Consequences

What this makes worse, stated rather than implied:

1. **A dependency on a public-preview API, three weeks old, explicitly "subject to change."** The
   Stacks API is what makes B enforceable at all. **Mitigation, and it is a requirement rather than
   a suggestion: an actor that cannot read the stack must do nothing and say so.** Under the
   actor design the failure mode is benign — an unreadable stack means no pull request is closed
   and no pull request is merged, which degrades to the status quo rather than to a lockout. This
   is strictly safer than the first draft's required check, whose failure would have blocked
   opening pull requests at all.
2. **Two actors with write access, running unattended, on a plan with no branch protection.** The
   ceiling actor closes pull requests and the merge actor writes to `main`. Nothing stands behind
   them. This is a genuinely new risk that the first draft did not carry, and it is the price of
   enforcement on this plan: ADR-0004 accepted advisory CI, and this RFC responds by granting
   automation the authority CI does not have. The mitigations are that closing is reversible, that
   the merge actor requires concluded-green rather than not-yet-red, and that both are auditable
   in the Actions log — but the risk does not disappear, it is accepted.
3. **A capability difference between trusted and untrusted contributors** (§Proposal D). Untrusted
   contributors get a worse workflow. That is a real cost of the supersession and must not be
   described as a detail.
4. **The ceiling will block real work at some point,** and the moment it does will be the moment
   the work is most urgent. This is STD-0007's Costs accepted §1 in a new place: the pressure to
   waive the rule is highest exactly when it is doing its job.
5. **More automation to maintain,** in a company whose entire premise is that governance must be
   mechanically enforced rather than remembered. Every scheduled job here is a thing that can break
   silently. The expiry job in particular closes things, and a bug in it closes work.
6. **No GitHub Desktop support** (source 2). Irrelevant today; recorded because it is a constraint
   on tooling choice rather than a preference.

## Rollout

1. **This RFC accepted** — reviewed by a human, `status: accepted`, committed on its own.
2. **ADR-00NN supersedes ADR-0012** — the access model, per §Proposal D. Own artifact, own review,
   **and a plan change**: it fires ADR-0004's first upgrade trigger. **Blocks everything below it**,
   because the ceiling actor needs to know who may stack.

   **With one contributor this step can wait.** Nothing in §A, §B or §C needs it — the founder
   already has write access and can stack today. Deferring it keeps the org on Free and keeps this
   RFC free of a spending decision, at the cost of leaving the ADR-0012 contradiction standing as
   known debt rather than resolved. That trade is stated here and not taken; it belongs to whoever
   accepts this RFC.
3. **STD-00NN** — the numbers and the checks, standing beside STD-0007 in the same relationship
   STD-0007 has to ADR-0013: the ADR states the rule, the standard fixes the thresholds and names
   the checks. The ceiling, the expiry windows and the auto-merge requirement live there so they
   can be changed in one line.
4. **Check ids declared in `check-registry.yaml`** before any of them is implemented — ADR-0014's
   bargain. `declared` is legitimate; a locked artifact naming a check that does not exist is a
   commitment, and the backlog already carries five.
5. **First adopter is `tool-bench`** (see `docs/superpowers/specs/2026-08-19-tool-bench-design.md`).
   It is the first repository to use pull requests properly, its slices are already ordered, and it
   is small enough that a broken ceiling check is obvious rather than mysterious.

**How this is undone.** A, B and C are independent, and undoing any of them is deleting a workflow
file. C's expiry and merge actors are scheduled workflows. B's ceiling actor runs on
`pull_request.opened`. A is abandoned by running `gh stack dissolve` and continuing with flat pull
requests — which is alternative 2, and is why alternative 2 is the documented fallback rather than
a different design. **Nothing here is a platform setting**, which is a side benefit of having no
platform settings available.

**Backwards compatibility:** none needed. There are no open pull requests anywhere in the estate.
Every mechanism here is nearly free now and expensive once there are, which is the same argument
RFC-0013 won on and the reason this is being written before the first repository needs it rather
than after.

## Open questions

1. **Does the ceiling count per contributor or per organisation?** Written per contributor above,
   which with one contributor is the same number. They diverge on the day REQ-0011 fires, and
   per-organisation is the reading that actually bounds review load. **Not decided.**
2. **What is the trust boundary in the superseding ADR, concretely?** "Trusted contributor" is an
   adjective, and an adjective in a locked artifact is the defect STD-0007 was written to fix.
   Whoever writes ADR-00NN owes a criterion, not a word.
3. **What guards the merge actor?** It has write access to `main` on a plan where nothing else
   does, and it runs unattended. Requiring concluded-green checks is necessary and probably not
   sufficient. A dry-run period where it comments what it *would* merge is the obvious first
   answer, and how long that period lasts is undecided.
4. **Should `pr.slice_cites_requirement` rise from `warn` to `error` for stacked work?** Deliberately
   excluded from this RFC (§Proposal C.3) to keep it visible as its own decision rather than
   smuggled in as a consequence of a larger one.
5. **What closes a stack whose requirement is withdrawn?** Expiry catches abandonment by silence.
   Nothing catches a stack that is still being pushed to in service of a requirement that no longer
   exists.

## Related

- **STD-0007** — the accepted cost this RFC exists to pay down; untouched, and this stands beside it
- **ADR-0013** — stage 4, unchanged; this quantifies throughput as STD-0007 quantified size
- **ADR-0012** — contradicted by §Proposal D; supersession proposed, not performed
- **REQ-0011** — `deferred`, and the reason this design assumes no human approver
- **ADR-0002, ADR-0014** — why the ceiling is a declared check before it is an implemented one
- **ADR-0004** — the plan constraint that determines what enforcement can mean here; the artifact
  the first draft of this RFC contradicted, and whose upgrade trigger §Proposal D fires
- **STD-0011** — where the retrieval-grep lesson in the amendment note belongs
- **REQ-0012** — the exitability test that rejected a third-party stacking tool
- **RFC-0013** — the "nearly free while the estate is empty" argument, reused here
- **STD-0008** — test tiers and quarantine, now load-bearing because a flake evicts a whole stack
