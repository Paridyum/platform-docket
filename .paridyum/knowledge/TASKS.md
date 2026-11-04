# Paridyum — Tasks

> **Superseded in part, 2026-09-28 — ADR-0113 to ADR-0122.** Where a task names the party above a tenant the
> **operator** — "the operator → tenant → account hierarchy", `OperatorMember` — read **org** and
> **org member** (GLO-0012, ADR-0114). Where a task puts the control plane inside tenant zero, read
> ADR-0118: the portal at paridyum.in is the platform's own surface, and tenant zero is Paridyum's
> tenant only. Signup, `max_tenants` and `<tenant>.paridyum.com` are replaced by RFC-0056, ADR-0114
> D5 and ADR-0121 (see PLAN-TENANT-FLOW). The estate's databases and identity are rebuilt from empty
> once these land; a task that names a live id or row describes the estate before that.

Companion to [`ROADMAP.md`](ROADMAP.md). Written 2026-08-15.

Every task states **what**, **why it blocks**, and **done when** — because a task without a
falsifiable completion condition is the same failure ADR-0018 names for a trigger nobody can
evaluate. Where a task produces a knowledge artifact, the artifact is the deliverable and this line
disappears when it exists.

**Legend:** `⛔` blocks the next phase · `→` produces a knowledge artifact · `⏱` expires if delayed

---

## Phase 0 — Commercial and legal ground

*Blocks the first runtime commit. Nothing in Phase 3 may start before this closes.*

### ~~T-0.1 Decide where this roadmap lives~~ — **done 2026-08-18**
Both files moved to `knowledge/` root as prose beside `DECISION-DEBT.md`. They now have history,
backup and the ADR-0025 checkpoint hook.

**The task's own objection was wrong, and that is why it sat open.** It recorded that placing them
in `knowledge/` "puts them under the gate, which is an argument both for and against." It does
not: `check-knowledge.sh` scans only the directories `artifact-registry.yaml` names, and
`README.md`, `DECISION-DEBT.md`, `TESTING.md` and `SECURITY.md` have always lived here ungated.
Verified by re-running the gate across the move and observing the artifact count unchanged. A
trade-off that does not exist made the cheapest option look expensive for three days — worth recording, because the same
mistaken objection is available to any future task about where prose lives.

### ~~T-0.2 Move RFC-0013 from `proposed` to `accepted` or `rejected`~~ — **done 2026-08-18**
**ACCEPTED.** All five decisions adopted; D5 stays deferred on its own trigger. RFC-0013's Status
section now carries the D1–D4 owner table.

**D3 was already satisfied for signing** — every commit in this repository carries a good
signature, verified 2026-08-18. What remains owed is D1's thesis (T-0.3), D2's harness (T-0.4),
D4's assignment wording (T-0.8), and D3's conception log.

**One finding recorded at acceptance:** all four owners are the same person, and D3 and D4 both
exist to survive a second contributor arriving. Neither is exercised until one does — which is also
D5's trigger — so the day the system is first tested is the day it is first relied on, with no
rehearsal.

### T-0.3 ⏱ Write the D1 technical-effect thesis
One page: the mechanism, the technical effect it produces, the metric demonstrating the effect, the
baseline it improves on. RFC-0013 states the diagnostic plainly — *"if that page cannot be written,
the finding is about the product rather than the paperwork, and it is better to know before
building."* Section 3(k) of the Indian Patents Act grants on measurable technical effect.
**Done when:** the page exists and names a metric with a number on it.

### T-0.4 ⏱ Land the D2 baseline benchmark harness
Latency, memory, throughput, density — captured **before** the optimised path exists. A claim of
improvement requires a measured predecessor, and once the naive implementation is gone it cannot be
recovered. RFC-0013 costs this at one day.
**Done when:** the harness runs and stores a baseline; and it runs against something, even a
deliberately naive implementation.

### ~~T-0.5 Agree BET-0004's clause-1 measurement convention~~ — **done 2026-08-15**
Recorded in BET-0004's revision. BET-0005 makes it cheap: Paridyum builds and operates product one
itself, so engineer-days from scoping to first production tenant are directly observed rather than
reconstructed. Actually recording it from product one is still on you.

### ~~T-0.6 Decide vertical one~~ — **done 2026-08-23**
BET-0004 *"does not settle the vertical order."* **BET-0005 supplies the criterion that was
missing:** not *which vertical sounds attractive* but **which one can Paridyum plausibly operate as
a real business, in India, without touching regulated data.** ANTI-0007 and C-0006 anti-goal 3 are
hard filters and remove more candidates than they appear to — anything payments-adjacent, health, or
regulated-financial is out.
**Answered: `hosting`.** Promoted to `stage: building` in `product-registry.yaml` on
2026-08-23 by operator decision, which is anti-goal 6's gate passed deliberately rather than
by drift. The reasoning that selected it is in that entry and in MKT-0001…0003, which are
external and dated as C-0006 requires.

**The consequence is recorded rather than left to be rediscovered:** `veer-entitlements` is
now blocked until hosting has a paying customer who RENEWED, per C-0009 obligation IV's own
terms. Both registry entries say so.

**Still owed by the promotion:** `catalogue` at `building` owes a REQ- and the
`product-contract` slot. MKT- is already discharged.

### T-0.7 → Set the floor and the royalty
BET-0007 fixes the shape — flat floor per running business **plus** a commission on the tenant's
transactions — and fixes neither number. The floor must be argued against fully-loaded idle-tenant
cost (schema, DNS, certificate, backups, monitoring share); the royalty must clear the aggregator's
own take without making the total unattractive against running the business independently.
**Done when:** both numbers exist with reasoning, and the floor survives a check against BET-0007
trigger clause 1.

### T-0.10 ⛔⏱ Research the payment path and write it up with citations
BET-0007 cost 1 commits to a royalty on tenant transactions and **deliberately states no figures**,
because RFC-0020 requires external facts cited and dated and RFC-0013 fixes the jurisdiction as
India. Split settlement through a licensed aggregator is the assumed path — the aggregator holds the
licence, performs sub-merchant KYC, and card data never reaches Paridyum. Direct authorisation
carries net-worth and escrow obligations a pre-incorporation company cannot meet.
**Two things to settle beyond the licence question:** how sub-merchant KYC fits a self-serve signup
(BET-0007 cost 2 — a tenant will have software in minutes and be unable to collect for days), and
confirmation that no catalogue product ever needs to store a card number.
**Done when:** an RFC or ADR states the path with dated citations, and BET-0007 trigger clause 3 can
be evaluated.

### T-0.11 Accept or reject RFC-0022 — **accepted 2026-09-27**, D5 published as partial (§D5a); **D2a's check still undeclared**
RFC-0022 designs the entire defence for BET-0005 cost 1. **Open question 1 was answered on
2026-08-15** — C-0006 non-commodity item 4 is narrowed: aggregate data may improve the runtime and
set product defaults, and may never inform where Paridyum competes. Four boundary options
(vertical, territorial, sunset, retail-price handicap) were considered and rejected as
physical-franchise mechanics that do not transfer to a white-label digital business.
**Question 2 answered 2026-09-27 by §D5a** (partial — the paths to a tenant's data measured, one logged,
pgaudit configured for three more, and a path 8 opened by the scoping that makes pgaudit work). **Still open:** questions 3–5.
**Blocks:** the first franchised tenant — D1, D2 and D5 are worth a fraction if made after the fact.
**Done when:** RFC-0022 is `accepted` or `rejected`, and D2a's check is declared in
`check-registry.yaml` under ADR-0014. **The first half is met; the second is not** — RFC-0022 still
carries `enforced_by: []` and no D2a check exists, so this task stays open for it.

### ~~T-0.13 Answer RFC-0025 open question 1 — build or assemble the fleet manager~~ — **decided 2026-08-18**
**Build it, in Rust.** The operator was shown the scope cost — a fleet manager with an API, quotas,
metering and a customer model is a product built before the product — and a cheaper manual-first
path was declined. ADR-0005's lock is satisfied rather than worked around, and REQ-0012's
exitability question does not arise for code we own.

**The cost is recorded, not resolved.** RFC-0025 accepted cost 1 carries it, and BET-0009 trigger
clause 1 measures it: if Phase 3a slips more than six weeks with the fleet manager a contributing
cause, the bet and the RFC both die regardless of how good the fleet manager is by then.

**Still open and now urgent:** which repository and which class it lives in. `infra` is where
ADR-0037 puts provisioning; a Rust service with a customer-facing API looks like `service`. ADR-0042
fixes the first wave at five repositories and this is not one of them.

### ~~T-0.14 Answer RFC-0025 open question 2 — can the fleet manager stay off every request path~~ — **closed 2026-08-18**
**Yes, and it is now verified rather than argued.** RFC-0025 decision 10 enumerates the paths.

Five are off-path under decisions already taken — RFC-0016 gives the rented edge TLS termination,
`Host` resolution and per-hostname certificates; decision 6 put quota into Kubernetes as static
configuration rather than a live entitlement call. The management interface is deliberately on-path
and harmless: the fleet manager being down stops deployment, not serving. Metering is constrained
to pull-based by decision 10, which was the one real gap.

**Row E was not designed.** Quota moved into Kubernetes because Proxmox enforces hard limits on
storage only — a cost reason — and the side effect was that entitlement stopped being a live
request-path call. Recorded in the RFC so that reintroducing a live lookup later is understood as
reopening this question rather than as a refactor.

**TST-0062 keeps it closed.** Stop the fleet manager, assert a hosted workload still serves. Prose
decays on the next feature; the test does not. Deferred on its own trigger — stage B1, because
before that there is nothing to keep serving.

**The certificate path is forbidden rather than tested**, because a renewal brokered through the
fleet manager fails silently and surfaces weeks later, outside any test window.

### ~~T-0.16 Decide the isolation and quota boundary~~ — **decided 2026-08-18**
**Namespaces with Kata for tier 1; tier 2 planned and not built.** RFC-0025 decision 8.

Cluster-per-customer was rejected on density: three control-plane nodes per customer, with etcd
wanting roughly four times its database size in RAM, is a customer's control plane consuming the
margin before an application runs — BET-0009 trigger 3 firing by construction.

**The operator's example settled it.** WordPress and a catalogue of containerised software, in the
Elestio shape, means the customer deploys an application and never runs `kubectl` — so an API
server per customer buys a capability nobody asked for. Tier 2 (vcluster, real Kubernetes) is
designed for and shipped on a trigger; vcluster is Apache 2.0, so it stays available rather than
becoming a purchase.

### ~~T-0.17 Decide the fleet manager's repository and class~~ — **decided 2026-08-18**
**`svc-fleet`, class `service`, one repository.** RFC-0025 decision 9. C-0007's derivations fix the
rest: crate `paridyum-fleet`, binary `fleet`.

**Not `infra`** — that class is `deployable: false` and `ci: false`, because ADR-0037 corrected it
on the grounds that *"an infra repository DEFINES the cluster; it does not run inside it."* The
fleet manager runs. Putting it there would also inherit the no-CI-standard gap `class-registry.yaml`
records against `infra`.

**Not `platform`** — that is *"platform services every product depends on."* Paridyum being
provisioned by the fleet manager is a build-time dependency, not a runtime one every product carries.

**ADR-0042 is not engaged.** It bounds the first wave of *platform contexts* at five; `svc-fleet` is
a different class on a different track and adds nothing to that wave. The cost it does add — one
more repository to maintain — belongs to BET-0009 trigger clause 1.

### ~~T-0.18 Register the hosting product under ADR-0050~~ — **done 2026-08-18**
Registered as `hosting` — *Managed application hosting* — `kind: catalogue`, **`stage: proposed`**.

**The dossier obligation is met, which is what made it registerable.** `catalogue` at `proposed`
owes MKT- artifacts naming the evidence that selected the market, and C-0006 requires that evidence
not be tenant-derived. MKT-0001, MKT-0002 and MKT-0003 are all external, dated, and tenant-derived
from nothing, because there are no tenants.

⚠️ **`proposed` and not `building`, deliberately.** Entry to `building` is the gate C-0006 anti-goal
6 describes, and `product-registry.yaml`'s own footer warns that *"the registry is not neutral about
breadth. It counts."*

**Promoting it probably answers T-0.6.** The phase 0 exit gate reads "vertical one chosen" and
treats it as open. If hosting becomes a catalogue product under construction it is either vertical
one, or it competes with whatever vertical one turns out to be — and anti-goal 6 forbids the second
until the first has a renewed paying customer. **That is a strategy decision and registering the
entry did not take it.**

### T-0.15 ⛔ Accept or reject RFC-0025 — **now unblocked**
T-0.13, T-0.14, T-0.16, T-0.17 and T-0.18 are all closed. **Every question this RFC was blocked on
has an answer**, and it carries ten decisions, a verified invariant (TST-0062) and its costs.

Until it resolves, BET-0009 is accepted and the way it gets built is not — which is the least
useful state for it to sit in, because work can legitimately start on a bet and cannot legitimately
start on a `proposed` design.
**Done when:** RFC-0025 is `accepted` or `rejected`.

### T-0.8 Schedule the incorporation → DPIIT → file sequence
RFC-0013 D4 fixes the order: build and measure → incorporate → obtain DPIIT recognition → file.
DPIIT requires incorporation and unlocks the ₹1,600 fee tier plus the 80% rebate; filing before it
is a straight loss. Incorporation documents must carry a **present** assignment — *hereby assigns*,
not *agrees to assign*. Do not attempt the PCT 90% natural-person reduction; the 2017 Understanding
forecloses it where an obligation to assign exists.
**Done when:** dates are on a calendar and the assignment clause language is drafted.

### ~~T-0.12 Close the workstation gaps~~ — **done 2026-08-15**
`bootstrap.sh --check` reports **0 failures, 1 warning**, and the warning is the GitHub Free plan
limitation (ADR-0004), not a defect. Signing verified end-to-end: three commits landed with `%G? =
G` through the real pre-commit path. `check-repo.sh` reports *"hooks reachable via: plugin"* — the
2026-08-07 audit finding is closed and ADR-0007 layer 1 is live.

**Unverifiable from here:** whether the key is registered on GitHub as an *SSH signing key*
(separate entry from auth). `gh` exposes no read path for it. Commits show "Unverified" on GitHub
until it is, even though they are correctly signed locally.

Original findings, kept because T-2.9 and T-2.10 came out of closing them:

### ~~T-0.12 detail~~ ⛔⏱ Close the workstation gaps — signing first
`scripts/bootstrap.sh --check` on 2026-08-15 found, on the macOS host:

| | |
| --- | --- |
| ✘ `rustup` missing | hard blocker on all Rust work |
| ✘ `terraform`, `just` missing | blocks the substrate work below |
| ! `commit.gpgsign`, `gpg.format`, `user.signingkey` all unset; no ed25519 key | **RFC-0013 D3** |
| ! `gh` token missing `workflow` scope | fails at push time with an error that does not mention scopes |
| ! plugin marketplace never added | ADR-0007 layer 1 has still never fired — the 2026-08-07 audit finding, unchanged |

**Signing first, and it is the only item here that expires.** RFC-0013 D3 requires signed commits
from the *first* commit; git history cannot be retroactively signed into contemporaneous evidence,
and India has no trade secret statute to fall back on. `bootstrap.sh` deliberately will not fix it
— *"clobbering identity or signing config is destructive and unrecoverable from here. Reported
only."*
**Done when:** `bootstrap.sh --check` reports zero failures, and a signed commit exists.

### T-0.9 Clear the four unrecorded-decision items in `DECISION-DEBT.md`
Including session `6a39cd18`'s twelve decision-shaped statements with no artifact, and the standing
question of whether ADR-0032 / ADR-0034 / STD-0010 should now lock.
**Done when:** each item is ticked or converted to an artifact.

---

## Phase 1 — Close the decisions that block repository #1

### T-1.1 ⛔ Answer RFC-0009 Q5 — is `lib-tenancy` one library or one per stack?
Rust services need ADR-0024's two-column nested silo with forced RLS; a Next.js interface needs the
same guarantee in TypeScript. `class-registry.yaml` permits `library` on both stacks, so it is
allowed — but two implementations of one mechanism is ANTI-0004's version-skew risk landing on the
security boundary.
**Blocks:** `lib-tenancy`, which blocks every `platform-` service.
**Done when:** RFC-0009 carries the answer, or a new ADR does.
**Partially unblocked 2026-09-24 — the naming half only.** ADR-0107 decided how one capability is
named when it exists in two stacks (`lib-auth-rust` / `lib-auth-typescript`, artifact name drops the
segment), because `lib-auth` hit it first. **The substance of Q5 is untouched:** whether the
isolation mechanism should exist twice at all, and what to do about the skew risk when it does, is
still this task. A naming precedent is not an answer to a security question.

### T-1.2 Answer RFC-0009 Q2 — does `platform-audit` start as a table or a context?
§7 says table. Open question 2 disagrees with §7: *"a table that every context writes to is
ANTI-0001 the moment there are two writers, so the trigger has to fire early — possibly
immediately, which would make the answer 'context'."* Two first-wave services both write audit
events, so plan for the trigger firing inside Phase 3.
**Done when:** decided, with the second-writer trigger written down either way.

### T-1.3 Answer RFC-0009 Q4 — where does an AccountMember authenticate?
Identity owns the entity; the login page belongs to the product. Narrowed by RFC-0012's revision:
an AccountMember authenticates against their tenant's Zitadel **organization**, so the seam is
between a product and a shared per-region issuer. Still unspecified.
**Done when:** the seam is named in RFC-0012 or a successor.

### T-1.4 ⛔ Move RFC-0009 to `accepted` and clear its `DECISION-DEBT` entry
Q1 and Q3 — the two the RFC named as blocking — were answered on 2026-08-08 by RFC-0012 and
ADR-0039 respectively. The reasoning has moved; the status has not.
**Done when:** `status: accepted`, and the entry disappears from the regenerated debt list.

### T-1.5 ⛔ Decide RFC-0018 — is the control plane one repository?
Determines whether `infra-` sprawls before there is anything to run.
**Done when:** accepted or rejected.

### T-1.6 ⛔ Decide the region model — RFC-0014 and RFC-0017
RFC-0012 places one shared Zitadel instance **per region**, so residency is an input to identity
rather than a later concern. RFC-0017 adds that operator personal data is regional.
**Done when:** both are accepted or rejected, and `region-registry.yaml` reflects the outcome.

### T-1.7 ⛔ Move REQ-0001 (runtime secrets) from `deferred` to `accepted`
`DECISION-DEBT.md` records ADR-0045 risk R3 as *expiring at the first deployment*: the Zitadel
masterkey, database credentials and service-account keys all exist before REQ-0001 is discharged,
and GitHub Free has no org-level secrets for private repositories — *"so there is no
secret-management story at all."*
**Blocks:** the first deploy in Phase 3, not the first commit.
**Done when:** REQ-0001 is `accepted` with a named mechanism, or R3's continuation is justified in
writing.

### T-1.8 Decide the deployment-substrate RFCs — RFC-0016, RFC-0019, RFC-0021
The edge is rented · a datacentre is one Proxmox cluster · device hostnames. These block deploying,
not building, so they may run concurrently with Phase 2.
**Done when:** all three are accepted or rejected.

---

## Phase 2 — Make the harness enforce before it guards real code

### T-2.1 ⛔ Watch every check in `rust-ci.yml` fail
Eight checks — `rustfmt.check_passes`, `clippy.clean_with_all_targets_all_features`,
`docs.doc_tests_pass`, `rustdoc.builds_without_warnings`, `test.coverage_meets_threshold` and both
`pr.*` — are implemented in `templates/infra-ci/rust-ci.yml` and **have never executed**, because
the reusable workflow is only invoked by Rust repositories and none exist beyond `tool-forge`.
Plant the defect each claims to reject; confirm red. Follow `scripts/test-checks.sh`, which already
encodes this convention.
**Why first:** the 2026-08-06 audit found checks reporting success while enforcing nothing.
Discovering these eight are broken while simultaneously debugging a new service is the expensive
ordering.
**Done when:** each of the eight has been observed failing on a planted defect.

### T-2.2 ⛔ Distribute `forge`
Its five implemented checks run against `tool-forge` only — the binary is not distributed and no
other repository's pipeline invokes it. Distribution, not implementation, is the gap.
**Done when:** `forge check` and `forge knowledge sync` run in a repository that is not
`tool-forge`.

### T-2.3 ⛔ Resolve the four locked-but-unenforced artifacts
ADR-0020, ANTI-0002, ANTI-0003, **STD-0009**. ADR-0002's rule is that `locked` requires a live
check; the gate calls these *"the backlog"*. Either implement the check or downgrade to `accepted`
with the reason recorded — both are legitimate, silence is not.
**Priority:** STD-0009 first, and it is no longer merely the most expensive — under BET-0005 it is
**existential.** It governs tenant-isolation testing, where the denial is the assertion, and
RFC-0022 D1 makes the test *"Paridyum's own tenant cannot read another tenant's rows"* the single
most valuable one in the system, because it is the one a prospective franchisee would ask for.
RFC-0022 R2 names the risk directly: D1 implemented and never tested.
**Done when:** `check-knowledge.sh` reports zero locked artifacts enforced by nothing.

### T-2.4 Implement only the checks that bind first-wave repos
Of the 71 awaiting `tool-forge`, build the subset a `platform-` service actually triggers. Leave
the rest declared — ADR-0014's declared-versus-implemented split exists precisely so this is a
legitimate state rather than debt. Resist `test.*` (12) and `cargo.*` (8) beyond need.
**Done when:** repo #4 scaffolds and passes its own gate, and the unbuilt count is reported rather
than reduced for its own sake.

### T-2.5 Verify the PreToolUse guard is actually installed
The 2026-08-07 audit found the plugin declared and enabled while the marketplace had never been
added — layer 1 had never fired, `SessionStart` context had never been injected, and the ADR-0025
checkpoint had never run automatically, *all while the configuration read as complete*. Confirm
before Phase 3, when the guard starts protecting code rather than prose.
**Done when:** `scripts/check-repo.sh | grep -i marketplace` is silent in every repo.

### T-2.9 `bootstrap.sh` exits 0 when a `cargo install` fails
Found 2026-08-15, provisioning this workstation. `sqlx-cli` failed to install — it required a newer
rustc than the active toolchain — and the script **printed the error and exited 0**. A provisioning
script that reports success while leaving a required tool absent is the same shape as a check that
passes while enforcing nothing, and `workstation.doctor_passes` is declared against this script's
output.
**Done when:** a failed `cargo install` sets a failure and is reflected in the exit code, and the
defect has been planted and watched to fail.

### T-2.10 A stale pinned rustup default silently served an 11-month-old compiler
Found the same session. `rustup toolchain list` showed both `stable` and a pinned
`1.90.0-aarch64-apple-darwin (active, default)`; the pinned one won, giving rustc 1.90.0 in August
2026, which is what broke T-2.9's install. `bootstrap.sh` checks that rustup *exists* and never
that the active toolchain is current — and ADR-0005's whole reason for mandating rustup over a
distro package is that a distro package "silently disagrees with CI about the compiler version." A
stale pin does exactly that, through rustup.
⚠️ **This task was written wrong on first draft and is corrected here.** It said the check should
warn when the active toolchain "is not `stable`". **STD-0003 requires the opposite** — pinned
exactly, *never* `stable`, because a floating channel means the compiler changes under you and CI
disagrees with your machine the week a release lands. The defect is not "not stable", it is
**"disagrees with the pin"**.
**Done when:** `bootstrap.sh --check` reports the active toolchain and warns when it disagrees with
the `rust-toolchain.toml` of the repository being checked, and when a globally pinned default
shadows one.

### T-2.13 ⛔ `sed` bypasses the PreToolUse guard entirely
Found 2026-08-17. A `Write` to `templates/rust-service/rust-toolchain.toml` was **denied** by the
C-0004 guard. A `sed -i` against the **same file, same session, seconds earlier** went through
untouched.

The guard intercepts tool calls; a shell command that edits a file is not one. So CLAUDE.md's
claim — *"when the plugin is installed, the tool call fails"* — and ADR-0007's argument that layer
1 *"cannot be reasoned around"* are both weaker than written: any agent with Bash can edit a locked
artifact, a vendored snapshot, or generated output by not using the edit tools. **This is the one
enforcement layer that does not depend on the GitHub plan, and it has a hole the width of
`sed -i`.**
**Done when:** the guard covers file-mutating Bash invocations, or ADR-0007 and CLAUDE.md are
corrected to state the limit plainly so nobody relies on a guarantee that is not there.

### T-2.14 A hand-written file can accidentally become "generated"
Found the same session, and it is how T-2.13 surfaced. C-0004 classifies generated output by
marker text in the first twenty lines — `@generated`, `DO NOT EDIT`, `Code generated by`. A comment
was added to a **hand-written template** reading *"Do not edit it here alone"*, and the file
immediately became unwritable through the tools: the guard was correct, the marker was false.

Cheap to hit and confusing when it happens, because the denial message asserts the file is
generated and names a generator that does not exist. Worth either narrowing the match (anchored,
case-sensitive, own-line) or making the denial say *which line matched*.
**Done when:** the match is narrowed, or the denial quotes the triggering line.

### ~~T-2.12 → No policy says when the Rust pin is advanced~~ — **closed 2026-08-20 by ADR-0053**
The pin moved four times across 2026-08-15/16 — 1.90.0 → 1.97.1 → 1.94.0 → 1.97.1 — and each move
was a judgement made from scratch. Rust ships every six weeks, so this recurs indefinitely.

**STD-0003 settles that the pin is exact and never a channel. It says nothing about when it
advances**, and a pin with no advancement rule produces exactly the failure that started this: a
compiler nearly a year old, discovered only because `sqlx-cli` refused to install against it.

What a decision would need to fix: the cadence or trigger for a bump; who verifies; the
verification required before one lands (`fmt`, `clippy --all-targets --all-features`, tests, and
`cargo deny` — done by hand each time so far); and that a dependency raising its floor is itself a
trigger. Pairs with T-2.11, which moves the number out of a locked standard so a bump does not
require superseding one.
**Done when:** an ADR states when the pin advances, or a note records why it stays ad hoc.

**ADR-0053 states it.** The operator's direction was to stop hard-pinning and stay on latest; the
conflict with locked STD-0003 turned out to be apparent rather than real, because STD-0003's erratum
says what it decides is *"the shape, not the number"* — an exact version, never a channel — and says
nothing about the number's age. Exactness is reproducibility; currency is age; the opposite of
latest is stale, not pinned. So the pin stays exact and advances on upstream's release, patch/minor
and toolchain automerge on green, majors never do, an advisory bypasses the cadence, and a
dependency raising its floor is its own trigger.

**The part worth reading is decision 7.** Staleness fails the gate independently of Renovate,
because ADR-0010 and ADR-0021 both assert Renovate keeps pins current and **no Renovate
configuration has ever existed in this organisation** — so both sentences were true of nobody for
the whole period they were written. This task existed because of what that cost (T-2.10), and a
policy enforced only by the same absent robot would have closed it on paper.

⛔ **Still owed, and nothing in ADR-0053 decisions 2-6 operates without it: Renovate itself.**

### T-2.11 STD-0003 embeds a Rust version, and STD-0003 is locked — **labelled 2026-08-16, not fixed**
An erratum was appended under ADR-0031 (`9bfb5ad`), verified by the gate as *"errata appended, body
byte-identical"*. It states the current pin, points at `templates/rust-service/rust-toolchain.toml`
as authoritative, and says outright that it does not fix the defect.

**It cannot.** ADR-0031 decision 5 forbids an erratum changing what an artifact decides, and the
body stays byte-identical — so the example still reads `channel = "1.90.0"` and a reader who skims
still copies it. Only a reader who reaches the end of the file is warned. **Supersession is still
required**, and the erratum bought time rather than closing this.

Original entry follows.

### T-2.11 detail — STD-0003 embeds a Rust version, and STD-0003 is locked
Found 2026-08-15 bumping the pins to 1.97.1. STD-0003's example reads `channel = "1.90.0"`, so a
reader copying the standard gets a compiler nearly a year old — and the standard is `locked`, so
the number cannot be corrected in place. Neither legal path fits: supersession is heavy for a
version bump, and ADR-0031's errata are for a *factual error*, which this is not — it was true when
written and merely dates.

This is the hand-maintained-number failure that README.md has already been caught by three times,
one layer down. **The structural fix is C-0007's rule** — *the grammar is constitutional, the list
is data*: the standard should state "pinned exactly, never `stable`" and name
`templates/rust-service/rust-toolchain.toml` as the current pin, rather than embedding a number
that can only be corrected by superseding a standard.
**Done when:** STD-0003 is superseded to reference the template instead of a literal version, or a
note records why the stale example is acceptable.

### T-2.15 The research-capture hook does not fire from a parent working directory
Found 2026-08-19, during the Phase 2b decision session. Substantial external research was done —
vendor documentation, licence terms, driver requirements — and **`research/` gained no line for it.**
The session ran with a working directory above the repository, and RFC-0027's capture hook is
declared in the repository's own `.claude/settings.json`.

This is the same shape as T-2.5 and T-2.13, and it is the third time: **a mechanism reads as
installed while never having fired.** The existing log's single line is itself evidence of the
inverse failure — `scripts/test-checks.sh` wrote a fixture URL into the real log because
`PARIDYUM_KNOWLEDGE_ROOT` was set, and RFC-0027 decision 6 voided rather than deleted it. So the hook
has been observed writing to the wrong place and not writing at all, and neither was noticed at the
time.

**Why it matters more than a missing log line.** The gate's own check is *"every page fetched is
cited in some `sources` block"* — it verifies fetches against citations, and it cannot verify a fetch
it never recorded. It currently reports zero reconciled sessions, which reads as clean. The
citations in ADR-0052 and in RFC-0016 decision 4 are therefore **agent-asserted rather than
independently captured**, which is precisely the distinction RFC-0027 exists to make. An agent
writing the log afterwards from memory is the forgery the mechanism forbids, so this cannot be
backfilled — only fixed forward.
**Done when:** a fetch from a session whose working directory is above the repository is recorded, or
the gate reports plainly that capture is not running rather than reporting zero reconciled sessions.

### T-2.16 Four cargo binaries are unpinned, and one sets the estate's compiler floor
Found 2026-09-05 while seeding `tool-registry.yaml` under ADR-0066. `RUN-0001` installs
`sqlx-cli`, `cargo-deny`, `cargo-audit` and `cargo-chef` with a bare `cargo install` and no
version, so four entries carry `ref: latest-unpinned` — a value chosen to fail
`knowledge.declared_tool_install_is_pinned` rather than to satisfy it, because writing a number
there would invent a decision nobody made.

**`sqlx-cli` is the one that matters.** Its 0.9.0 minimum is the floor that forced the Rust
toolchain pin and therefore created `toolchain-registry.yaml` at all — so the estate's most
consequential version constraint is set by a binary nothing pins, installed from whatever crates.io
had on the day.

That check is deliberately left `owner: tool-forge` for now: implementing it turns the gate red on
four pre-existing defects that this change did not create and has no mandate to decide. Picking
versions for four binaries is four decisions.
**Done when:** each of the four has a `--locked` install at an argued version, and
`knowledge.declared_tool_install_is_pinned` is implemented and defect-tested.

### T-2.17 `toolchain-registry.yaml` and `tool-registry.yaml` answer overlapping questions
ADR-0066 decision 8. Seven tools — `sops`, `age`, `talosctl`, `helm`, `kubectl`, `cilium`,
`ansible-lint` — have their versions parked under a `tooling:` key inside whichever stack needed
them first, which is why `sops` and `age` are filed under `ansible` although ADR-0057 makes them
estate-wide, and why `cilium` (a Helm chart version) is filed as a tool at all.

`tool-registry.yaml` covers all of them with `pin_home: toolchain-registry` and no version, so
coverage is complete and no fact has two homes. What remains is that a reader asking "what version
of sops do we run" must know the answer is in the file NOT named after tools. Migrating the pins
was rejected in the same change: changing behaviour while moving it is the failure
`artifact-registry.yaml` records about ADR-0010's `ci: false`, and it would make a registry's first
change also its first regression.
**Done when:** a decision exists either way, recorded rather than drifted into.

### T-2.18 `pin_home: os` is an admission, not a pin — and most `critical` tools carry it
Written as "nineteen of thirty-four" and corrected the same day against the file, which is the
fourth time a hand-counted figure in this repository has been wrong and the reason it is now
written as a shape. Count it with:

```sh
python3 -c "import yaml;print([t['id'] for t in yaml.safe_load(open('tool-registry.yaml'))['tools'] if t.get('pin_home')=='os' and t['tier']=='critical'])"
```

Every entry with `pin_home: os` says its version belongs to the OS package manager. `brew install
jq` on two workstations on two days installs two versions and nothing compares them — and the
majority of the `critical` tier is in that state, including `python3`, whose
`scripts/lib/validate-schema.py` is the mechanism behind every `*_registry_valid` check, and
`claude-code`, which self-updates and is pinned by nothing at all.

This is not a defect introduced by the registry; it is the pre-existing state, made countable for
the first time. Recorded here rather than resolved because the fix is either a lockfile-shaped
workstation manifest or an accepted risk, and neither has been argued.
**Done when:** the `critical`-tier entries carrying `pin_home: os` either gain a real pin or an
argued acceptance.

### T-2.19 The rendered diagram fetches a font, and we delete the line by hand
`scripts/render-diagram.sh` exists to strip three `<link>` elements archify's template writes for
JetBrains Mono. The diagram works offline either way; what the strip buys is that opening a
committed governance diagram does not tell a third party someone read it (ADR-0065 decision 7).

The cost is the typeface — diagrams render in system monospace — and a small permanent maintenance
burden anchored on two hostnames that moves whenever upstream's template does.
**Fix shape:** embed the font as a data URI, or upstream a flag. MIT permits the fork and the
change is four lines in `assets/template.html`. Not done here because a forked renderer is a second
thing to pin.
**Done when:** the strip is unnecessary, or the fork is adopted and pinned in its place.

### T-2.20 Nothing generates an architecture diagram, and hand-authoring one is forbidden
ADR-0065 decision 2 makes architecture-kind IR generated output from the set of manifests, and
`docs.architecture_ir_is_hand_authored` rejects one written by hand. The generator is `forge`,
which does not exist — so the rule currently forbids more than it enables, and the C4 Level 1 and
Level 2 views ADR-0040 made *possible* on 2026-08-09 are still not *produced*.
**Done when:** `forge` renders the system-context and container views from `manifest.yaml` into
archify architecture IR, and `docs.system_view_is_current` compares them.

### T-2.7 `check-registry.yaml` cannot express "owed by a component that is not tool-forge"
Found 2026-08-15 while trying to declare STD-0011's check. `owner` is the **only** signal the gate
uses for implemented-versus-owed: `tool-forge` means owed, and **every other value counts as
implemented**. So a check owed by `paridyum-platform`, `check-repo.sh` or any other component
cannot be declared without immediately being counted as built — a check reporting success while
enforcing nothing, which is the exact 2026-08-06 audit finding the registry was hardened against.

The workaround taken was to not declare the check at all and record the intent in STD-0011's prose,
which is honest but means the wanted check is invisible to `--audit`.
**Fix shape:** an explicit `implemented: true|false` field, with `owner` reduced to attribution.
That is an ADR-0014 change.
**Done when:** a check owed by a non-`tool-forge` component can be declared and counted correctly,
and STD-0011's `knowledge.session_discoveries_are_captured` is declared against it.

### T-2.8 Extend `capture-open-loops` to discovery-shaped statements
STD-0011's mechanism. ADR-0011's hook already detects decision-shaped statements; this widens the
phrase set to *"it turns out"*, *"the actual path is"*, *"that fails because"* and surfaces them in
`DECISION-DEBT.md` when a session changed no artifact. **`warn` severity, permanently** — an
error-severity heuristic over natural language gets routed around within a week.
**Blocked by:** T-2.7, which is what lets it be declared honestly.

### T-2.6 Fix the three standing gate warnings
STD-0005 cites keepachangelog with no declared source (RFC-0020's exact case) · 48 hardcoded
registry-computed numbers across 23 files, sitting **at** the baseline of 48 · `patterns/` declared
as a type with no directory, so nothing of that type is scanned.
**Done when:** the gate runs clean, or each warning has a written reason to persist.

---

## Phase 2b — The substrate

*Designed in RFC-0023. There is nowhere to deploy anything until this exists, so it sits between
the harness and the first wave. `sitar-as-in-01-ar` — Arch on metal, Proxmox as a guest, one Talos
cluster named `staging` with production topology.*

### ~~T-2b.1 Verify nested KVM~~ — **done 2026-08-15, enabled**
RFC-0023 R1 retired for availability. Decisions 2 and 3 stand.
**Still open:** whether L2 nesting is *fast enough*, which is only measurable once Talos nodes run
(T-2b.5). Confirmed available ≠ confirmed usable.

### T-2b.2 ⛔ Accept or reject RFC-0023 — and RFC-0016 with it
RFC-0023 decision 5 depends on the rented edge for the inbound path, and RFC-0016 is `proposed`.
The substrate has no static address and needs none only if that RFC resolves.
**Done when:** both are `accepted` or `rejected`.

**Unblocked 2026-08-19 — every open question either RFC was blocked on is now answered, and the
status transition itself is untaken.** RFC-0023 open questions 1 and 2 closed (decision 7, and
ADR-0052 decision 3); RFC-0016 open question 1 closed (decision 4 — Cloudflare). What remains is a
judgement rather than research, and it is the operator's: accepting an RFC is not an agent's act.

**One collision was found and resolved before it could bite.** ADR-0021 is `locked` and deploys to
an environment named `dev`; RFC-0023 decision 3 builds one cluster named `staging` and argues
explicitly against `dev`. Neither artifact mentioned the other, and the glossary defined no term
`environment`, which is why. **GLO-0011 now does**, separating the deploy target from the approval
gate in front of it — and on that reading ADR-0021 is *under-satisfied rather than contradicted*, so
no locked artifact needs superseding. The registry fix GLO-0011 describes is owed on a trigger, not
now.

### T-2b.3 Answer RFC-0023 open question 1 — which CSI driver — **chosen 2026-08-19, not yet run**
The largest remaining divergence risk and the one that most often forces manifests to fork.
Local-path is trivial here and wrong on metal; Ceph is right on metal and heavy here. Decision 4
requires **one answer for both**.
**Blocks:** nothing until a workload needs persistence. Blocks everything after that.
**Done when:** one driver is chosen and runs on the staging cluster.

**Longhorn**, recorded as RFC-0023 decision 7. `numberOfReplicas` is the variable that satisfies
decision 4 — two replicas here, three on metal — which is what decided it over the alternatives
rather than any judgement that it is the better product. Rook-Ceph lost on Sidero's own
documentation (slow on small clusters, dependent on CPU parallelisation — the two things nested L2
degrades most); Piraeus/LINSTOR is the runner-up and lost on operational familiarity alone.

### ~~T-2b.3 closed 2026-09-21 — it runs~~

`longhorn-system` has been `Active` 15d with `csi-attacher`, `csi-provisioner` and `csi-resizer`
all 2/2. The done-when was *"one driver is chosen and runs on the staging cluster"* — both halves
are now met, and the paragraph below was written while only the first was.

**Original entry follows.**

**The half of this task that is still open is the half that matters.** A driver is chosen; nothing
has run it. RFC-0023 decision 7 cost 2 and risk R5 are the reason to expect the running to be
interesting: Longhorn replicates synchronously over a cluster network sitting under two levels of
nested virtualisation.

**Two things this answer changed elsewhere**, both cheap now and expensive after five nodes exist:

- **T-2b.4 must give each node a second virtual disk.** Every replicated driver in the candidate set
  requires storage separate from the Talos install disk. This is the strongest argument against the
  legitimate option of deferring the driver choice — deferring it silently changes the *next* task.
- **The decision 3 memory table was revised.** Longhorn wants 8 GB per storage node and the table
  gave workers 6 GB. It was affordable only because the original table accounted for 48 GB of 64 and
  nobody had noticed the missing 16.

### T-2b.4 Provision the Proxmox guest with OpenTofu
`bpg/proxmox` is the maintained provider; `telmate` is the one most tutorials use and is less
actively maintained. Endpoint is a variable — metal and guest Proxmox expose the same API, which is
what makes the transition a variable rather than a migration.
**Done when:** the guest is created and destroyed by OpenTofu, not by hand.

⚠️ **This task said "Terraform" until 2026-08-19 and nothing had decided that.** RFC-0019 left the
choice open — *"Terraform or OpenTofu"* — and the task quietly picked one. **ADR-0052 decides
OpenTofu**, on the same REQ-0012 exitability test that made RFC-0025 reject Sidero Omni for being
BUSL: Terraform has shipped under BUSL since v1.6, and applying that test to Omni but not to
Terraform is drift. ADR-0020's locked `terraform/` directory and `terraform.fmt_and_validate` check
are satisfied unchanged — the identifier is vocabulary, and `tofu fmt`/`tofu validate` are the two
operations it names.

**Two additions to this task's scope**, both from decisions taken the same day:

- **A second virtual disk per Talos node**, sized as a variable — T-2b.3's finding. Without it no
  candidate CSI driver works at all.
- **State encryption is in scope here, not later.** ADR-0052 decision 3 answers RFC-0023 open
  question 2 with OpenTofu's native client-side encryption and a passphrase key provider, so state is
  encrypted before it touches any backend. The passphrase becomes a single point of failure that
  T-2b.6 owns.

### T-2b.5 Talos `staging` cluster — 3 control plane @ 4 GB, 2 workers @ 8 GB
Machine configs as code. Production topology deliberately: three control-plane nodes on one
physical host buy **no availability**, only correct quorum behaviour and manifests that match
metal. Never describe it as HA.
**Done when:** `kubectl get nodes` shows five, and a control-plane node can be drained and rejoined.

### T-2b.6 ⛔ Build the real secrets mechanism — RFC-0023 decision 6
REQ-0001 has **not** fired (this substrate serves no customer, so ADR-0045 R3 stands), but a
development environment with a different secrets story than production fails at exactly the
transition RFC-0023 exists to make seamless. SOPS/age over encrypted files in git is a legitimate
answer and costs about a day. Terraform state contains these secrets — open question 2.
**Done when:** no credential exists in plaintext on disk, and the mechanism is the one metal will
use.

### T-2b.7 Tunnel to the rented edge
Origin dials out; no inbound ports, no dynamic DNS, no static IP. TLS terminates at the edge per
RFC-0016 D1.
**Done when:** a hostname resolves through the edge to a service on the staging cluster.

**The vendor is Cloudflare** as of 2026-08-19 — RFC-0016 decision 4. The shape is the vendor's own
documented one rather than something assembled here: a Cloudflare Tunnel from the origin, with the
Cloudflare for SaaS **fallback origin** as a CNAME to the tunnel's public hostname, TLS terminating
at the edge.

**Selection is not approval, and REQ-0012 is `deferred`** — so the approval is held up by compliance
rather than by machinery. Owed before this carries anything real: the DPA signed rather than merely
published, the sub-processor list read against RFC-0015 decision 3, and an egress allowlist for
`cloudflared`.

⛔ **One input could reverse the vendor choice.** The three controls that answered the residency
criterion — Customer Metadata Boundary, Regional Services, Geo Key Manager — are a **paid add-on
whose price is uncited**, and RFC-0016 decision 4 cost 1 refuses to write a figure without a dated
source. Get the number before building on it: against stage B1's budget it is plausibly the largest
line, and if it is unaffordable the residency criterion becomes *unmet* rather than met.

### ~~T-2b.6 closed 2026-09-21 — SOPS is the mechanism and nothing is plaintext~~
`.sops.yaml` in all four infra repositories; seven of the eight files under
`infra-kubernetes/secrets/` carry a `sops:` block. The eighth is
`postgres-backup-creds.sops.example.yaml`, a template whose values are the literal `REPLACE` —
read to confirm, not assumed from the filename. **And its real counterpart does not exist**, which
is the T-G.7 evidence recorded below.

### ~~T-2b.7 and T-2b.8 closed 2026-09-21 — the whole chain, verified~~
Four hostnames answer from the public internet through the rented edge to a pod:
`paridyum.com`, `careers.paridyum.com`, `admin.paridyum.com` (307 → 200) and `id.paridyum.com`
(302 → 200), each with `x-edge-cache: HIT`. That is T-2b.7's *"a hostname resolves through the
edge to a service on the staging cluster"* and T-2b.8's *"a request from the public internet
reaches a pod and returns"*, together, four times over.

### T-2b.8 Deploy something trivial end to end
The smallest possible service, through git → build → Terraform → cluster → edge. This is what
proves the chain, and it is also the first thing the `post-deploy` tier has ever had to run against.
**Done when:** a request from the public internet reaches a pod and returns.

---

## Phase 3 — The first wave

*Order is fixed by ADR-0039 (the saga lives in tenancy and reads identity) and RFC-0012 (identity
proxies Zitadel, one shared instance per region).*

**BET-0005 splits this phase.** 3a builds enough platform for exactly one tenant (T-3.1 → T-3.5);
3b runs product one as Paridyum's own business and is where revenue starts (T-3.6); 3c makes it
franchisable (T-3.7, T-3.8); 3d signs the first franchisee (T-3.9). **First revenue no longer
depends on BET-0006 being right**, which is the largest risk reduction in the plan.

### T-3.1 ⛔ `platform-tenancy` — **first**, per ADR-0042 decision 2
Owns the operator → tenant → account hierarchy and ADR-0039's asynchronous compensating saga —
the ANTI-0005 gap RFC-0009 §6 recorded as unsatisfied. **It is built first because every other
context is a dependency of that saga rather than a peer of it.** Two names split deliberately:
OperatorMember and AccountMember are different populations that must never mix.
**Done when:** a half-created tenant compensates correctly — test the failure path, not the happy
one.

### T-3.2 `platform-identity` — proxy and provision Zitadel
Per RFC-0012: Zitadel keeps minting, validation never routes through the proxy, organization per
tenant in one shared instance per region, capability matrix as data. ADR-0042 kept it separate from
tenancy because *"their availability profiles differ by an order of magnitude: login must be
near-perfect, provisioning may be down for an hour."*
**Done when:** a tenant organization is provisioned through the proxy and a session validates
without touching it.

### T-3.3 `platform-routing` — resolve a request to a tenant and a region
Route, DomainBinding. ADR-0042 consequence 2 calls this *"the least justified of the five"* — it
could plausibly start as a module inside tenancy, and exists separately because ADR-0041 makes
region a contractual fact and *"a contractual fact with no owner ends up owned by whoever touched
it last."*
**Done when:** a custom domain resolves to the right tenant in the right region.

### T-3.4 `spec-product` — product contracts
ADR-0042 decision 4: each catalogue product declares its Zitadel Project, role vocabulary, audience
and tenancy scope. RFC-0012 requires the declaration and ADR-0039's saga reads it; today it is
prose. `class: spec` is *"interface contracts… no executable source"*.
**Done when:** the contract is machine-readable, versioned, and the saga reads it.

### T-3.5 Answer RFC-0009 Q5 in code — `lib-tenancy` or per-stack
ADR-0024's two nested silos, `tenant_id` + `account_id`, RLS forced; operator scope **a grant,
never an absence** (GLO-0005). **Note `lib-tenancy` is not one of ADR-0042's five** — the ADR is
silent on it, and T-1.1 is where the one-library-or-two question gets settled. Whatever the answer,
the mechanism lands here and the ANTI-0004 skew risk lands with it.
**Done when:** the isolation denial is asserted as a test per STD-0009, and T-2.3's check watches
it.
**Not answered by ADR-0107, deliberately.** `platform-tenancy` copied `platform-identity`'s token
verifier rather than inventing a shared crate, precisely so this task would not be decided as a side
effect; ADR-0107 then extracted `lib-auth-rust` on ADR-0092 D3's own trigger and answered the naming
question it ran into. **The tenancy mechanism is still undecided** — one library or one per stack,
and the ANTI-0004 skew risk on the RLS boundary — and it stays undecided here rather than being
inferred from what `auth` did.

### T-3.6 ⛔ Product one, **operated by Paridyum** — Phase 3b
Named by T-0.6. This is BET-0005's core move and the point at which the company first has income.
Build the vertical, run it as a real business on your own runtime as a tenant, acquire real end
customers. The domain depth C-0006 anti-goal 1 now demands is acquired here rather than asserted,
and BET-0004 clause 1's engineer-day figure is recorded here for the first time.
**Note what this does not need:** entitlement, billing or metering. One tenant has no plan
distinction to enforce.
**Done when:** Paridyum's own tenant has paying end customers, and the engineer-day figure is
written down.

### T-3.7 ⛔ RFC-0022 D1 and D5 — before any franchisee exists
**D1:** Paridyum's own tenant is provisioned through the same saga, stored in the same silos, and
granted no operator scope. **D5:** a tenant-visible operator access log — if a Paridyum operator
touches a tenant's data, that tenant sees it, by default and not on request.
**Why the ordering is not negotiable:** a data-use commitment made before any tenant data exists is
credible; the same words after three years of unrestricted access are a policy change nobody
believes. An access log running since the first row is evidence; one switched on after a dispute is
an admission.
**Done when:** D1's denial is asserted as a test per STD-0009 and has been watched failing, and D5
logs something a tenant can read.

### T-3.8 `platform-entitlement` — what a tenant is permitted to do — Phase 3c
PlatformPlan, Subscription, Entitlement. Its trigger — *the first plan distinction that must be
enforced rather than described* — fires when a second tenant exists, i.e. at the first franchisee.
Note BET-0007: quota enforcement is a **correctness** feature, not a billing one, and it must exist
before the price does.
**Done when:** one plan distinction is enforced rather than described.

### T-3.9 First franchised tenant — Phase 3d
The first customer of BET-0006. REQ-0013's trigger (*the first tenant-authored row*) fires here, and
RFC-0022 D4 promotes export from a deferred requirement to a launch commitment — under a landlord
who competes with you, the credible ability to leave is what makes staying rational.
**Done when:** a franchised tenant is isolated from Paridyum's own tenant at both silos, with the
denial asserted and the access log visible to them.

### T-3.10 Expect `platform-audit` to fire during this phase
Its trigger is the second writer, and T-1.2 predicts that arrives inside the first wave. RFC-0022 D5
depends on it. Plan the table so that promoting it to a context is not a migration.

### T-3.11 Settle where `platform-deployment` went
RFC-0009 §7's original first wave included it. ADR-0042 fixes the wave at five and **does not
mention it at all**, while narrowing routing from *"hostname to release"* (Domain, Certificate,
RouteBinding) to *"a request to a tenant and a region"* (Route, DomainBinding). So Release,
Certificate and the act of running an artifact currently have no owner.
**Done when:** an ADR says who owns them, or ADR-0042 is superseded to say why nobody needs to.

### T-3.12 → Revisit ADR-0042 decision 3 — billing's deferral reason has changed
ADR-0042 defers `platform-billing` because *"BET-0004 makes per-product pricing tempting and
BET-0002 does not authorise it, so building billing before the pricing model is decided would
encode a guess."* **The pricing model is now decided** — BET-0007, floor plus royalty. The stated
reason for the deferral no longer holds, though RFC-0009's own trigger (*the first operator who
must be charged*) does and still gates it.
Related: ADR-0042 consequence 4 — *"deferring billing means entitlement has no revenue signal"* —
which bites once tenants are on different floors.
**Done when:** ADR-0042 is amended or a note records that the trigger, not the reason, is what
holds billing back.

---

## Sprint Genesis — from nothing deployed to tenant zero running its products

*Opened 2026-09-15. Resumable by name: **Genesis**.*

**The arc, in one sentence:** the estate governs no running software; Genesis ends when Paridyum —
as tenant zero, holding no scope a customer could not hold — runs its own landing, careers, docket
and control plane as deployed products, each one a thing any other tenant could also deploy.

**Its artifacts.** Genesis implements RFC-0045 (the isolation envelope), ADR-0103 (containment as an
entitlement attribute), RFC-0046 (BYOK, build-gated), RFC-0048 (public surfaces are tenant-served)
and RFC-0049 (a tenant is a set of releases). RFC-0047 (renting the platform) is designed and gated
and is **not** in Genesis. All are `proposed`; none is accepted; per ADR-0013 §4 each lands in its own
commit before the work it justifies.

**Ordering principle: irreversibility, not architecture.** What gets permanently worse with time goes
first, regardless of how small it is. Two items are lossy — an unwritten audit history cannot be
reconstructed, and a region never recorded cannot be recovered — and they are both cheap today.

> ### ⛔ READ THIS BEFORE TRUSTING ANY GREEN CHECK — CI IS OFF, AND THE CAUSE IS NOW KNOWN
>
> **Diagnosed 2026-09-26. The organisation's included GitHub Actions allowance is exhausted.**
> `gh api "/orgs/Paridyum/settings/billing/usage?year=2026&month=9"` reports `Actions Linux` at
> exactly **`2000.0` minutes** with `netAmount: 0.0` — the whole GitHub Free allowance consumed,
> nothing billed past it. `2000.0` to one decimal is a ceiling, not a coincidence.
>
> **The last day with recorded usage is 2026-09-20** (641 minutes). So CI stopped four days before
> anyone noticed, and the "12:34 UTC on 2026-09-24" recorded elsewhere is when a person read a run
> page. Every run since fails in ~4–16 seconds with **zero steps and no logs** — jobs are created
> from the reusable workflow and then never start.
>
> **What this means for you, right now:**
>
> - **No pull request merged since 2026-09-20 has been gate-validated.** Run the gates locally and say
>   so: `check-knowledge.sh`, `check-repo.sh --root "$PWD"`, `check-migrations.sh --root "$PWD"`,
>   plus the stack's own `cargo test` / `npm run test`.
> - **`check-repo.sh` and `check-migrations.sh` need `--root "$PWD"`.** Run from a repository without
>   it, they resolve their own vendored `.paridyum/knowledge` directory and report a cheerful `n/a`.
>   That is how eleven real errors sat unseen in `platform-tenancy`.
> - **A red X on a PR is not a finding about your change.** Read the run: zero steps means the runner
>   never started.
>
> **Three ways out, and only the operator can pick.** (a) It resets on **2026-10-01** — free, and
> leaves the estate as fragile. (b) Attach a payment method and set a spending limit — money.
> (c) **RFC-0053** (self-hosted runners consume no included minutes), which is also T-G.21 below.
> The evidence and the full argument are in RFC-0053's Problem section.

---

### Wave 0 — decisions, before code

*Five answers only a human can give. Three of them block waves.*

#### ~~T-G.0 Authorise the sixth bounded context~~ — **closed 2026-09-20, TASKS.md updated 2026-09-24**
ADR-0101 (accepted 2026-09-20) authorised `platform-deployment` and supersedes ADR-0042. This line
sat stale for four days — the ADR itself names *"T-G.0 — the Genesis task this closes"* — because
nothing had walked TASKS.md back to close it. Waves 2, 3 and 4 are unblocked on this front.

#### ~~T-G.0b Is there a second failure domain?~~ — **decided 2026-09-24: declined, for now**
ADR-0105. No second hypervisor is bought or rented ahead of a funded need; the trigger is a
specific commercial one (a C2/C3 sale, a franchisee's recovery-objective contract), not a
calendar. Every attestation (T-G.18) must report RPO/RTO against `sitar` as the sole failure
domain — no field may read as though a warm second site exists. RFC-0045 slice 7, RFC-0046, and
RFC-0047 stay undeliverable, which is now a decision rather than an unanswered question.

#### ~~T-G.0c Flat or nested — where does a partner's tenant live?~~ — **answered 2026-09-24: nested**
RFC-0047 open question 9, answered in the RFC rather than left for a successor document. Researched
against three external references before deciding (RFC-0020): no mature multi-tenant platform found
(Salesforce Hyperforce, Auth0) offers per-sub-tenant region pinning inside one shared database — it
requires either a dedicated deployment per customer (Auth0 Private Cloud) or a geo-distributed SQL
engine built for row-level placement (CockroachDB), neither of which matches ADR-0023/ADR-0064's
plain-Postgres, one-hypervisor design. The region-per-account idea raised in this session (tenant
stays put, an individual account's data is placed near its own end-customer) is therefore resolved
as **a variant of nested, not a third architecture**: a partner's stamp chooses its own region at
its own provisioning (extending RFC-0047 §Proposal P3), rather than a `region` column added to
`accounts` in a shared database. Still gated exactly as before — on ADR-0105's declined second
failure domain and ADR-0024 D9 — so this closes the *design* question without changing when it is
buildable.

#### ~~T-G.0d docket's account silo — retrofit or accept?~~ — **retrofitted 2026-09-24**
ADR-0104 (accepted 2026-09-20, supersedes ADR-0102) decided the general question — a tenant may run
more than one instance of a product, and `account_id` is the discriminator between instances, not
only between "distinct client organisations" as ADR-0102's narrower first draft had it. The
`accounts` table and the five-table retrofit that decision described were still unwritten;
migration `0011_accounts_and_the_docket_instance_retrofit.sql` does it now, while every table is
still empty (verified against the schema before writing it). ~~RLS stays tenant + operator-scope,
unchanged, per ADR-0104 decision 7's accepted cost 4 (per-instance operator grants are a separate,
undecided question).~~

> **⛔ THE STRUCK SENTENCE WAS WRONG, AND IT WAS WRONG IN THE MIGRATION TOO. Corrected 2026-09-24.**
>
> **Cost 4 does not license it.** ADR-0104 accepted cost 4 leaves *per-instance operator grants*
> undecided — which operator may reach which instance, the `operator_grants` (0010) question. It
> says nothing about whether the policy carries the discriminator. **Accepted cost 1 governs that**
> and says the opposite: *"Every docket query and policy gains an inner predicate, permanently…
> That is the price of the boundary being real rather than notional."*
>
> **What the estate actually had, for four days:** `account_id` as a NOT NULL column on all five
> docket tables and an inner boundary enforced in **no** layer — not RLS (dropped), not the
> application (`account_id` appears in none of tool-docket's 74 Rust sources), not lib-auth
> (`assertAccount` returns early for operator sessions, by design). Two dockets under one tenant
> would have shared every person record. GLO-0004 rates that crossing catastrophic.
>
> **How it was lost:** `HANDOFF-2026-09-21` §1 resolved a two-files-both-numbered-0011 collision by
> keeping this file and deleting `0013_an_instance_is_an_account.sql`. The deleted file carried the
> inner predicate and the five `paridyum:rescope` directives; the survivor carried neither. The
> collision everyone was looking at was the numbering.
>
> **`check-migrations.sh` catches it and was not re-run** — 10 errors against the working tree,
> clean at `HEAD`. Fixed in place (the migrations are `pending`, never applied), gate now green.
> See `platform-tenancy/LOGBOOK.md` 2026-09-24 and `HANDOFF-2026-09-24.md`.
>
> The file is now numbered `0012_accounts_and_the_docket_instance_retrofit.sql`.

#### ~~T-G.0e Confirm ADR-0103's §Status reading~~ — **confirmed 2026-09-24**
ADR-0103 moved to `accepted`. The operator confirmed the reading: naming a containment rung
publicly, with no price and no self-serve path, is not "publishing a tier" under ADR-0024 D8.
ADR-0024 stands unsuperseded.

---

### Wave 1 — stop the loss

*Nothing here deploys anything. Everything here gets worse if deferred.*

#### ~~T-G.1 A cache key carries the tenant~~ — **done 2026-09-27, merged as web-hiring#5**

> **2026-09-27.** Routes moved to `/[lang]/tenants/[tenant]/careers`; the proxy rewrites the public
> path to carry the tenant its own resolver returns. The work arrived with its internal-namespace
> refusal disabled (`if (false && …)`) and an arbitrary tenant id in the URL rendering, because Next
> 16.2.12 runs the proxy a second time on its own rewrite. The control is now segment equality
> against the resolver, repeated in every page. Six review rounds, each measured on a production
> build; they also found that **production had shown "no open roles" regardless of what was
> published** (the chart never set the tenant), and that the branch had silently lost ISR until
> `dynamic = "error"`. **Not yet deployed** — needs a new image.
`careers/page.tsx` sets `revalidate = 60`. Next.js ISR keys on pathname, not `Host`, so the commit
that makes the app multi-tenant serves one tenant's postings to another — with RLS forced, the policy
correct, and the query never running. ANTI-0009 has existed as prose with no owner since it was
written; `web.cache_key_carries_tenancy` is now declared.
**The only latent defect in Genesis.** Everything else is a missing feature.
**Done when:** the resolved tenant is in the route, not a `cacheTag`, and the check fails a build that
omits it. Implements RFC-0048 slice 1.

#### T-G.2 One correlation id, one append-only audit stream ⏱ — **built 2026-09-26; blocked on applying 0010**
ADR-0041 D3: a shared correlation id must exist *"from the first request"* because it spans Zitadel's
event log and our own records, and *"retrofitting one across two systems means reconstructing history
that was never written."* Every request written so far has none.
**Permanently lossy. Each day of traffic is history that cannot be recovered.**
**Done when:** a fourth `SET LOCAL` carries the correlation id, `audit_events` exists append-only by
the trigger pattern 0006 and 0007 already use, and grants record who granted, to whom, when and how
they end. Implements RFC-0045 slice 1.

> **Checked 2026-09-24: the actual blocker is that RFC-0045 is `status: proposed`, not accepted.**
> §D gives `audit_events`'s exact column list (`correlation_id`, `actor`, `action`, `object`,
> `tenant_id`, `operator_scope`, `actor_region`, `at`) precisely enough to draft as a migration
> today — but building a migration against a still-`proposed` RFC is implementing a design before
> it is decided (CLAUDE.md precedence: `proposed` is "direction, not yet binding"). The `grants`
> table §D3 describes is *not* column-specified at all ("who granted it, to whom, when, and how it
> ends" is a sentence, not a schema) — that half cannot be drafted without inventing architecture
> RFC-0045 itself hasn't fixed. **Next action is getting RFC-0045 accepted**, not writing SQL
> against it; once accepted, `audit_events` is a same-shaped migration to 0011/0013 and genuinely
> small, and `grants`'s shape becomes the RFC's to fix during that acceptance, not this task's to
> guess.
>
> **✔ THE APP LAYER IS DONE 2026-09-26, and the schema half turned out to be written already.**
> RFC-0045 is `accepted`. Migration **0010** already carries `audit_events` with §D2's exact column
> list *and* `operator_grants` with §D3's who/to-whom/when/how-it-ends — so the half this note called
> unguessable had been written and was merely unapplied, having merged *after* 0011–0013 were applied.
>
> The fourth `SET LOCAL` now exists in all three places in the estate that open a Postgres
> transaction:
>
> | Where | What carries it |
> | --- | --- |
> | `platform-tenancy` | `src/correlation.rs` + `Db::begin()`; an axum extractor, infallible by design |
> | `@paridyum/auth` (`lib-auth-typescript` **0.2.0**) | `correlationIdFrom(headers)` + `withSessionContext`; `correlationId` is a **required** field |
> | `tool-docket` | `src/adapters/correlation.rs` + `enter()`; threaded from the edge, never minted inside |
>
> **The contract, in one line each, because these are the parts that were easy to get wrong:**
> ADR-0044 D1 wants the id generated once at the edge and *propagated unchanged*, so only a
> `traceparent`'s **trace id** is taken — never its parent id, which changes per hop — and minting is
> a fallback that **reports itself**. A caller-supplied `X-Correlation-Id` is validated to a narrow
> shape and **discarded rather than truncated** when it fails, because it lands in
> `audit_events.correlation_id` and in every log line, and a truncated id joins to nothing while
> looking like it should. For docket the CLI *is* the edge, so it mints **once per invocation**:
> per-transaction minting would give one `docket issue` four unrelated ids.
>
> **✔ AND THE STREAM HAS A WRITER, 2026-09-26.** `platform-tenancy/src/audit.rs` records
> `GET /v1/releases`: the allowed read **inside the same transaction as the read**, so the record and
> the access commit together; the **denial in its own transaction**, so it survives the thing that did
> not happen. A failed audit write fails the request — an unrecorded access is the defect the stream
> prevents — except for a denial, where the 403 still returns and the failure is logged, because
> turning a boundary decision into a 502 would hide it.
>
> **The probe found the control ADR-0044 D4 had already named and 0010 did not have.**
> `scripts/probe-audit-stream.py` asserts UPDATE and DELETE are refused; both were **accepted** on the
> first run, because the `FOR SELECT`/`FOR INSERT` policies mean an `UPDATE` matches no rows, changes
> nothing **and returns success** — the row trigger never fires. A caller rewriting the audit trail got
> `UPDATE 0` and no error, which is indistinguishable from a rewrite that worked. D4 says the primary
> control is the **grant**, not the trigger; 0010 now carries all three.
>
> **0010 was also four columns short of ADR-0044 D6's stated minimum** — the account, the source, the
> client, and **the outcome including denials** — because it transcribed RFC-0045 §D2 literally and
> §D2's list is the shorter of the two. Corrected in place, legally: REQ-0008 binds an *applied*
> migration and 0010 is `pending`.
>
> **What is still owed:**
>
> 1. **`0010` is unapplied**, and a failed audit write fails the request — so platform-tenancy must
>    not be deployed against the current database before it lands. An ordering, not a caveat.
> 1b. **And the volume is sized for a database with no audit stream.** ADR-0110 decision 7, measured
>    2026-09-27: **362 bytes per row** including all three indexes, against a **4 GiB** data volume —
>    ~11.9 million rows would fill it with nothing else in the database, and the stream is **never
>    pruned** (RFC-0045 §D4 keeps it longer than the data it describes). At 100,000 requests a day that
>    is three million rows in a month. Resize before a surface serves real traffic; the added latency
>    (~2 ms, an upper bound measured through `psql`) is not the constraint.
> 2. **docket writes no audit event** (tool-docket `DECISION-DEBT.md` D-72). Its four sealed chains are
>    what a document *is*, not who touched it: a read of a person record leaves no trace and a refused
>    write leaves less. **Blocked on `ADR-0110` (`proposed` 2026-09-26)**, and blocked on a decision
>    rather than on effort: docket is a CLI *and* a served API, so `docket check` audited per query
>    writes tens of rows where the same work served over HTTP writes one. ADR-0044 D6 fixes what a record
>    contains and is silent on how many one action produces. ADR-0110 proposes *one event per operation a
>    caller requested* — which is what `platform-tenancy` already does, so the ADR transcribes a
>    precedent rather than imposing a rule.
> 3. **Two classes of event cannot be recorded at all**, because `audit_events.tenant_id` is NOT NULL:
>    a refusal before a tenant resolves, and a control-plane read of the registry, which belongs to no
>    single tenancy. Both stay log lines. A placeholder tenant would attribute an action to a tenancy
>    that had nothing to do with it.
> 4. **`web-hiring`'s own copy of the session helper still sets three values** (recorded in
>    `lib-auth-typescript`'s `DECISION-DEBT.md`), so a request entering there carries no id.
>
> **One open question this surfaced, and it is RFC-0022's:** `GET /v1/tenants/{id}` is about exactly one
> tenancy, so it *could* be recorded in that tenant's stream — and RFC-0022 D5 commits to exactly that
> (*"if a Paridyum operator touches a tenant's data, that tenant sees it, by default and not on
> request"*). RFC-0022 was `proposed` when this was written (accepted 2026-09-27), so implementing it would have settled in code what the RFC exists to
> settle. Not done.
>
> **The parsing rules now exist in three implementations**, priced and recorded in tool-docket's
> `DECISION-DEBT.md` D-71 with the trigger that extracts them — a fourth Rust copy, or any change
> that must be made twice. `paridyum-auth` was rejected as the home because it is the estate's *auth*
> crate and a correlation id is observability.

#### T-G.3 `region` on `tenants`, immutable ⏱
ADR-0039 step 1 already says *"insert tenant row (with `region`)"* and ADR-0041 D3 calls it
contractual and immutable. No column exists. Trivial against one live region; a migration against
live rows once there are two.
**Done when:** the column is `NOT NULL` with no `DEFAULT`, constrained to a `status: live` region in
`region-registry.yaml`, refused rather than defaulted at provisioning, and an `UPDATE` trigger
refuses a move. `db.region_is_chosen_at_provisioning_and_immutable` passes. Implements RFC-0045 slice 2.

> **Checked 2026-09-24: written, not applied.** `platform-tenancy/migrations/0011_region_is_chosen_
> at_provisioning_and_never_moves.sql` exists, is fully argued (backfills the one live `tenants` row
> to `in`, adds `NOT NULL` with no `DEFAULT`), and `scripts/migrate.py status` reports it — and
> 0012, 0013 behind it — `pending`, not applied. Applying pending migrations to the live database is
> DDL against a populated table and was correctly refused by this session's own permission
> classifier rather than pushed through unattended. **Next action is `python3 scripts/migrate.py
> apply` from `platform-tenancy`, run by a human present for it** (or with the classifier's
> permission extended for that one command) — not a redesign; the migration is already written and
> reviewed.
>
> **✔ APPLIED 2026-09-24 11:29 — and the warning below arrived too late. T-G.3 IS DONE; read the
> rest, because two things it predicted came true.**
>
> `migrate.py apply` ran at 11:29 and applied 0011, 0012 and 0013 from uncommitted working-tree
> files. `tenants.region` exists; this task is closed. Two consequences:
>
> 1. **docket's write path is broken right now.** 0012 added `account_id uuid NOT NULL` to the five
>    person-bearing tables and `tool-docket/src/adapters/pg.rs` inserts `(id, tenant_id, record)`
>    and sets only `paridyum.tenant_id` + `paridyum.operator_scope`. Reads serve; writes fail. Not
>    production, and docket can be redeployed. **T-G.19** is now the repair, not the precondition.
> 2. **What was applied is not what was reviewed.** The review finished at ~17:15 and found the
>    inner `account_id` predicate missing from every policy 0012/0013 touched. Editing them would
>    have broken REQ-0008 immutability — and did cause checksum drift that blocked `migrate.py`
>    entirely, `status` included. 0012/0013 are now back to **exactly the bytes applied**, and the
>    corrections live in **`0014_the_inner_boundary_the_retrofit_left_out.sql`**, not yet applied.
>
> **✔ AND THE BOUNDARY 0014 CREATES IS NOW PROVEN, 2026-09-26 — locally, not on the cluster.**
> `prove-isolation.py`'s proofs **6 and 7** had reported SKIPPED since the day they were written, because
> they are guarded on 0014 being applied. So the one boundary 0014 exists to create was the one boundary
> nothing had exercised.
>
> `platform-tenancy/scripts/probe-isolation-locally.sh` replays the whole stream into a scratch database
> — where 0014, 0015 and 0016 *are* present — and runs the same seven proofs. **All seven pass, zero
> skipped, for the first time anywhere.** The docket account boundary and the account-bounded release
> history are proven to work rather than written and hoped for.
>
> **It proves the policies, not the cluster, and says so on SUCCESS**: what it rules out is a policy that
> was wrong, and what it cannot rule out is one that was never applied. **Applying 0010, 0014, 0015 and
> 0016 is still the step that makes the live boundary real.**
>
> **⛔ ORDER, measured 2026-09-27/28: the pending batch must not be applied until docket is redeployed.**
> The docket RUNNING on the cluster is the image of 2026-09-15 (Helm revision 3). It predates the
> account work: it sets two session values, so its writes have failed since 0012 (applied 2026-09-24),
> and — measured on a replay as `paridyum_app` — **0014 alone makes its reads raise**
> `paridyum.account_id is not set`. 0015 would then break the next version too: its `accounts_read`
> makes a three-value `enter()` raise on every call. `migrate.py apply` takes no target, so none of
> 0010, 0014, 0015, 0016 can go ahead of the others.
>
> **The order, each step proven:** (1) insert Paridyum's `product='docket', slug='default'` account row
> — with an explicit v7 id, because `accounts.id` has no default until 0014 (ADR-0078's own exception);
> (2) deploy tool-docket `main` (#26 sets all four session values) — proven against the live schema
> 0001–0009 + 0011–0013 by tool-docket#27: store 8/8, served 1/1; (3) apply the batch, minutes after
> (2): until 0010 lands, the new image's audit writes fail and are logged (no regression — the running
> image writes none). Step (2) needs an image, which needs CI minutes or a `write:packages` credential.
>
> **Verify before the next `apply`:** `SELECT seq, sha256 FROM paridyum._migrations WHERE seq IN
> ('0012','0013')` should read `0012` = `04c4b4f9c15be4adf318f497cfcf5e091c635938897691c6fff3fc142bc00786`,
> `0013` = `40ca59e5b3fa3055257b2a2df77f237f953ccc04f21f607e4b312f9480eeeb85`. 0013's bytes were
> recovered byte-exact from git's index; **0012's were reconstructed and are corroborated but not
> proven** — it was untracked, so it is in no commit. If that checksum disagrees, stop.
>
> See `HANDOFF-2026-09-24.md` §0.

#### ~~T-G.4 Clear the red gate~~ — **done 2026-09-16**

`scripts/check-knowledge.sh` reports **zero errors, 41 warnings**. The three errors this task named
— one ADR's `supersedes` typed as a string, and two ADR numbers reserved by RFC-0044 and cited by
it and ADR-0100 without existing — are all closed.

**The line above was stale when it was read, and the task says why.** It ends *"run the gate for
the current list rather than trusting this line (ADR-0035)"*, which is what happened on 2026-09-16:
the gate was re-run, found clean, and the task was still open. A task whose own text warns against
trusting its text is a task that needs its **done-when** executed periodically rather than its
**why** re-read — this one sat closed-in-fact for an unknown period. Original entry follows.

**Original:** *"Three errors stand on every run…"* — **Done when:** `check-knowledge.sh` reports
zero errors.

#### ~~T-G.5 Repair the `operator` ambiguity~~ — **done 2026-09-26** → `ADR-0109` + four corrections
GLO-0001 says an operator *"is our customer"* and owns 1..N tenants; GLO-0004's tree and BET-0005's
table both put **Paridyum** at that level. Harmless while one party provisions everything, and every
design above inherits it. `operator_id` is declared in the glossary and present in no migration.
**Done when:** one reading is chosen, the losing artifacts are corrected or superseded, and the column
either appears in a migration or stops being claimed. → produces a glossary correction.

> **Answered 2026-09-26 by ADR-0109 (`proposed`).** The two readings are GLO-0001's commercial
> party (spans *tenants*) and GLO-0005's runtime grant (spans *accounts within one tenant*), and
> BET-0005 uses both eleven lines apart. The runtime reading wins because it is implemented in
> **twelve applied — therefore immutable — migrations**, in `lib-auth`'s published API and in
> tool-docket, while `operator_id` appears in **no migration at all**. The commercial party is
> deliberately left unnamed: it is ADR-0024 D9's deferred level, and D9's own rule is *"Not 'we can
> imagine one.'"* Closes when ADR-0109 is accepted and GLO-0001/GLO-0004/BET-0002/BET-0005 carry
> the correction.
>
> **✔ CLOSED 2026-09-26.** ADR-0109 is `accepted`, and the corrections landed:
>
> - **GLO-0001** now defines the runtime grant, and its title changed with it. It carries a dated
>   *Correction* section rather than being superseded (D2): what was wrong was the word, not the
>   structural claim that a paying party may hold several tenants. `operator_id` is **removed** — it
>   was declared there and exists in no migration and no code, which is ADR-0014's defect class
>   applied to a column. `enforced_by` now names `db.operator_scope_is_an_explicit_grant`, so the
>   definition has a check behind it instead of being prose.
> - **GLO-0004's tree drew four levels while the entry's own title says two.** The drawing was the
>   odd one out: `operator` is not a level, it is a grant held *over* a tenant, so it now sits beside
>   the tree rather than above `tenant`, and Paridyum is labelled the platform. Three sentences that
>   said "the operator" meaning the tenant were corrected with it.
> - **BET-0005 used both readings eleven lines apart** — `| Operator | Paridyum |` in its table, and
>   "operator scope is a grant" in its costs. The row is now **Platform**, and the
>   `operator-as-tenant` tag became `paridyum-as-tenant`: a tag is a search term, and one carrying
>   the ambiguity is how the ambiguity keeps being found and copied.
> - **BET-0002 needed no change, and that is a finding rather than an omission.** D5 lists it, and
>   the word `operator` does not appear in it at all. Its pricing sentence — "per running business" —
>   is exactly the commercial fact D2 preserves, already expressed without the term. Recorded so
>   nobody reopens this task hunting for the edit.
>
> **Still deliberately unnamed:** the paying party above a tenant. ADR-0024 D9 admits it only on *"a
> real third level with a paying customer behind it — **not 'we can imagine one'**"*, and D9 is
> `locked`. Documents that need the concept write the phrase out.
>
> **One leftover, outside D5's list:** RFC-0022's title — *"the operator competes with its tenants"* —
> still uses the old sense. It is `proposed`, so it is noted in BET-0005's Related section rather than
> edited in passing.

#### ~~T-G.6 Declare `product:` on the web manifests~~ — **done 2026-09-27**: `web-hiring` declares `product: [hiring]` (web-hiring#6); RFC-0048 open question 2 marked answered
`web-console` declares `product: [identity]`; `web-hiring` and `web-paridyum` declare none, and under
ADR-0050 an absent value means *every* product.
**Done when:** both manifests declare a product and `knowledge.product_slugs_resolve` passes.
Implements RFC-0048 slice 2.

> **Checked 2026-09-26.** `web-paridyum` **declares `product: [paridyum]`** and did when this task was
> last read — so half of it has been done for some time and the line above is out of date, which is
> the ADR-0035 failure this file keeps rediscovering: a count in prose is a snapshot.
>
> **`web-hiring` is the remaining half, and the reason its manifest gives no longer holds.** The
> comment there says *"no such slug is registered in product-registry.yaml… and registering one is a
> commercial decision the operator owns"*. **`hiring` is registered** — `product-registry.yaml` line
> 235, `kind: capability`, `stage: building`, added 2026-09-20 for `seed-releases.py`. So
> `knowledge.product_slugs_resolve` would pass on `product: [hiring]` today, and the decision the
> comment defers has already been taken.
>
> **Not edited here, deliberately:** `web-hiring` has substantial uncommitted work in flight (T-G.1
> and T-G.12 — the cache key, and routes moving to `/tenants/[tenant]/careers`), and that repository's
> checkpoint hook commits everything with `git add -A`. A one-line manifest edit from a second session
> would land inside someone else's pull request. **Whoever holds web-hiring: this is two lines —
> `product: [hiring]`, and delete the comment that says you cannot.**

#### ~~T-G.7 Off-host backup with a stated RPO, and one rehearsed restore~~ — **done 2026-09-24**

> **All four done-when items landed 2026-09-24.** The restore rehearsal — the item every reading
> below calls "not yet run" — succeeded on the third attempt, once storage capacity (item 3 below)
> and a `serverName` bug were both fixed.
>
> **The `serverName` bug, found after the storage fix:** the first working attempt still failed
> with CNPG's `"no target backup found"`. Root cause: `externalCluster.barmanObjectStore` without
> an explicit `serverName` defaults it to the `externalCluster`'s own `.name`
> (`paridyum-backup-source`), so CNPG looked for backups under
> `s3://postgres-backups/paridyum-postgres/paridyum-backup-source/` — which doesn't exist. The real
> path, confirmed by listing the bucket directly with a throwaway `rclone lsf` Job, is
> `.../paridyum-postgres/paridyum/...` — `paridyum`, the *original* cluster's name.
> `postgres/backup.yaml` never hits this because it never sets an `externalCluster` at all.
> `postgres/restore-rehearsal.yaml` now sets `serverName: paridyum` explicitly, with the failure
> mode recorded in the file so it isn't rediscovered.
>
> **Verified, not just green.** After recreating with both fixes, the rehearsal cluster reported
> `Cluster in healthy state` in under 2 minutes, and querying it directly confirmed real, correct
> data: all 20 tables present, `tenants` holding its one real row —
> `id=68445ee9-175a-58a9-834e-a2f0c9c9b124, slug=paridyum, created_at=2026-09-12 07:37:53+00` —
> matching production exactly. Recovered to the latest available point (no `recoveryTarget` set),
> confirming PITR via continuous WAL archiving works, not only the nightly base backup.
>
> **The RPO the attestation can cite:** sub-minute, bounded by WAL-archiving lag rather than the
> nightly base backup, *within the single failure domain* — `sitar` is still the only place any of
> this runs (ADR-0064, ADR-0105), so this restore proves the backup is real and restorable, not
> that jurisdiction-level loss has a recovery path. RFC-0015's revision and ADR-0105 already say
> that plainly; this rehearsal doesn't change it.
>
> **Torn down cleanly, and the cleanup itself is now proof the fix works.** `longhorn-scratch`'s
> `reclaimPolicy: Delete` meant `make postgres-restore-rehearsal-cleanup` alone left **zero**
> orphaned PVs or Longhorn volumes — no second `kubectl delete volumes.longhorn.io` needed, unlike
> every attempt before the storage fix. Production (`postgres/paridyum-1`, `paridyum-1-wal`)
> confirmed `Bound`/healthy and `Cluster in healthy state` throughout.
>
> Everything below is the history of how this closed — kept rather than deleted, because most of
> it (the storage diagnosis, the two bugs fixed) is exactly the kind of thing STD-0011 says to
> keep.
>
> **1. The in-cluster credential exists and the cluster backs up for real.**
> `garage-consumer-secrets` minted `postgres-backup-creds` in `postgres`, `postgres-backup-enable`
> applied `postgres/backup.yaml`, and the live `Cluster` confirms it:
> `firstRecoverabilityPoint: "2026-09-14T02:00:13Z"`, `lastSuccessfulBackup: "2026-09-23T21:32:38Z"`.
> Closes the 2026-09-21 reading, which said this credential had never been created.
>
> **2. The off-host-emulated sync runs end to end.** `storage/garage/backup-sync-offhost-emulated.yaml`
> (CronJob, 02:30 UTC) copies `postgres-backups` to a second Garage bucket under a separate
> credential — rehearsing the off-host *process*, not yet a real off-host *destination* (ADR-0105
> still declines a second failure domain; the failure domain here is unchanged from decision 1's).
> Verified: a manual run transferred 329.835 MiB / 2445 objects, `Complete`, 25.3s. Two bugs fixed
> to get there — the source-namespace credential didn't exist in `storage` (secrets are
> namespace-scoped; `make postgres-backup-sync-secret` mirrors it) and
> `RCLONE_CONFIG_{SRC,DST}_REGION` was unset, so rclone signed as `us-east-1` against Garage's
> `s3_region = "garage"` and got `AuthorizationHeaderMalformed`. Both fixed in the tracked manifest
> and Makefile.
>
> **3. Storage capacity was blocking the restore rehearsal; found, diagnosed, and fixed.**
> `postgres/restore-rehearsal.yaml` + `make postgres-restore-rehearsal-run`/`-cleanup` stand up a
> disposable `Cluster`, bootstrapped via CNPG `recovery` against the real `postgres-backups`
> bucket — read-only, `paridyum` in `postgres` never touched. Two attempts both failed on a
> Longhorn fault, not a backup-content problem: `longhorn-manager` logs named it exactly —
> *"insufficient storage… does not have enough storage available"* — and
> `kubectl -n longhorn-system get nodes.longhorn.io` showed each worker's ~12.8GB disk already
> ~100–170% `storageScheduled`. Root cause, found in `infra-terraform/tofu/variables.tf`'s own
> sizing comment: `data_disk_gb` defaults to 12 and the comment names its own trigger — *"the first
> workload that needs more than 12 GiB replicated"* — which this was. Two layers of orphaned
> storage were cleaned along the way, both requiring `kubectl delete volumes.longhorn.io` in
> addition to `kubectl delete pv` (deleting a PV under `reclaimPolicy: Retain` does not delete the
> backing Longhorn `Volume` or its replicas — a two-step cleanup, easy to do only half of): two
> pre-existing orphans (10d old, predate this session) and three left over from the two failed
> attempts. After full cleanup, `storageScheduled` settled at **12.9GB on both workers — matching
> the legitimate footprint (`paridyum` 4+2Gi, `garage-0` 5+1Gi ≈ 12Gi) almost exactly.** No more
> debris exists anywhere; the disk is honestly ~100% committed by real data, zero headroom for
> anything new. Production (`postgres/paridyum-1`, `paridyum-1-wal`) stayed `Bound`/healthy
> throughout every step.
>
> **Fixed properly rather than waiting on the Proxmox disk resize:** `longhorn/storageclass-scratch.yaml`
> (new) declares `longhorn-scratch` — `numberOfReplicas: "1"` (half the footprint of the estate
> default's 2) and `reclaimPolicy: Delete` (so a normal `kubectl delete` is enough cleanup, no
> second orphan-hunt). `postgres/restore-rehearsal.yaml` now requests it instead of the default
> `longhorn` class; `postgres-restore-rehearsal-run` applies it automatically. Both manifests
> dry-run clean. **This does not fix the underlying capacity gap** — `data_disk_gb` still needs
> raising in `infra-terraform/tofu/variables.tf`, gated on checking the Proxmox `local-lvm` pool's
> actual free space first (the same file's sizing comment explains why that check isn't optional:
> at the last count 5 nodes × current defaults already approached the documented 118GiB pool
> budget) — it only unblocks *disposable* work like this rehearsal without waiting for that.
>
> **Next action, historical:** `make postgres-restore-rehearsal-run`, watched by a human — this is
> what was run, plus the `serverName` fix above, to close the task.
>
> *(2026-09-21 reading, superseded above: backups ran but to the same failure domain, the off-host
> credential had never been created, and `kubectl -n postgres get backup` returned no `Backup`
> resources despite a reported recent one — unchased, not evidence of failure, a gap in what could
> be verified from outside.)*
`0001_tenancy_foundation.sql` states in its own header that no real applicant may be inserted until
ADR-0064 D3's gate passes: an off-host destination **and** a rehearsed restore. ADR-0094 supplied the
endpoint and left the copy manual, so the estate's recovery point is a human's memory.
**The largest task in Genesis and the one everything waits on.** It unblocks Paridyum's own hiring,
`paridyum` → `stage: operated`, and — through that — every later stage in the product registry.
**Done when:** the off-host copy is scheduled with a stated RPO, one restore has been performed and
recorded, and the RPO is a number the attestation can cite. Implements RFC-0045 slice 4.

---

### Wave 2 — make the estate visible

*Requires T-G.0. The first wave that produces the picture the control plane was always missing.*

#### T-G.8 `Release` as a schema and a read model
Record what is already deployed before deploying anything. Validates the entity shape against reality
before a saga is written against it.
**Done when:** every running deployment in the estate appears as a `Release` row with tenant, product,
version, placement and state. Implements RFC-0049 slice 1.

> **Checked 2026-09-24: the schema half is written, nothing is populated yet.**
> `platform-tenancy/migrations/0013_release_is_a_product_running_for_a_tenant.sql` defines
> `releases` and `release_events` exactly to RFC-0049 §A's tuple, and is `pending` in
> `scripts/migrate.py status` — along with 0012, which it depends on for `accounts`. The
> done-when needs more than the migration: the nine live Helm releases (PLAN-THREE-SURFACES.md
> F0b) still need to be recorded as rows once the tables exist. Applying 0012+0013 unblocks this
> task's remainder; it does not finish it.
>
> **Updated 2026-09-24 — 0012+0013 are applied (11:29); `0014` is not.** The tables exist, so the
> nine Helm releases can be recorded as soon as `0014` lands and an `accounts` row exists — the
> seed creates the `default` one itself. Separately, `scripts/seed-releases.py` was broken against its own schema
> in three ways and is fixed: it inserted `name` where 0012 says `display_name`, omitted an `id`
> that had lost its `DEFAULT uuidv7()`, and set two session values after a comment explaining at
> length why three were required. `release_events` also became account-scoped in the same pass —
> it was tenant-readable and leaked every sibling instance's deploy history by join, while passing
> the gate, because the account check only interrogates tables that already carry the column.

#### T-G.9 `/dashboard` renders the estate
Tenants → Releases → version, placement, state, health. `/dashboard` stays a BFF and owns nothing
(ADR-0100 D1); it gains the half it never had.
**Done when:** Paridyum appears in `/dashboard` as one tenant among N, listing its landing, careers, docket,
admin and `app`. Implements RFC-0049 slice 2.

#### T-G.10 The placement resolver, inert — **⛔ blocked on T-G.12 (T-G.1 done 2026-09-27), not on a decision**
`poolFor(tenantId)` replaces the module-level `pool()`. At C0 it returns the one pool that exists, so
the change is behaviourally inert — and it is the seam every later containment rung needs.
**Done when:** no request path constructs a pool directly, and an unresolvable tenant raises rather
than falling back to a default. Implements RFC-0045 slice 3.

> **Checked 2026-09-26: RFC-0045 is now `accepted`, so the artifact blocker is gone — and there is
> exactly one place in the estate that constructs a pool.** `grep -rl 'new Pool'` across every web,
> console and library `src/` returns one file: **`web-hiring/src/lib/db.ts`**. Every other surface
> reads through a BFF or through `@paridyum/auth`'s session helper.
>
> That makes this task small — and it lands in the middle of another session's work. `web-hiring`
> currently has nine modified files including `src/lib/auth/context.ts`, which holds a **second**
> copy of the session helper and its own `pool()` call, plus an untracked `src/lib/tenancy/`. T-G.1
> and T-G.12 are being done there now.
>
> **Deliberately not started.** Two sessions editing `context.ts` is precisely what cost an afternoon
> on 2026-09-24, and that repository's checkpoint hook commits everything with `git add -A`.
>
> **For whoever holds web-hiring**, since you are already in these files: `poolFor(tenantId)` replaces
> `pool()` in `db.ts` and in `auth/context.ts`; at C0 it returns the one pool, so the change is inert;
> and **an unresolvable tenant must raise rather than fall back** — a default pool is how a request
> for an unknown tenant reads another tenant's data (RFC-0045 §A, ADR-0038's fail-closed rule applied
> to placement).

#### ~~T-G.11 The landing content schema~~ — **done 2026-09-26** → `RFC-0048 §E` + `0016` + a seed
RFC-0048 open question 1. `job_postings` had a natural schema because a posting is a known shape; a
landing page's sections are not, and **an over-general schema is how a product becomes a page-builder
by accident** — which is the C-0006 line the corrected RFC-0048 §B depends on staying the right side
of.
**Done when:** a migration defines tenant-scoped, localized (ADR-0096) landing content with a closed
set of section types, and Paridyum's own copy is seeded as rows. → produces a migration.

> **⛔ Checked 2026-09-26: BLOCKED on RFC-0048 acceptance, and nobody had recorded it.** This task
> implements RFC-0048, which is `status: proposed` — and it must *answer* that RFC's own open
> question 1 to do so. **This is the identical blocker T-G.2 carries against RFC-0045**, and T-G.2
> already states the reasoning: building a migration against a still-`proposed` RFC is implementing
> a design before it is decided, and the open question is the RFC's to fix during acceptance rather
> than this task's to guess in SQL.
>
> **The guessing part is now removed.** RFC-0048 §E (added 2026-09-26) closes open question 1 with
> four section types — `statement`, `list`, `link`, `contact` — each with fixed fields, ordered by
> an integer and nothing else, localized per ADR-0096 using the `is_localized_text` /
> `is_localized_list` functions 0007 already defines. The types are derived from what
> `web-paridyum` actually renders, not from what a tenant might want, and rule 4 is what keeps the
> set closed: a type is added by a migration AND a component in the same change, never by a
> settings field.
>
> **Next action is getting RFC-0048 accepted**, not writing SQL against it. Once accepted the
> migration is small and its shape is fixed by §E. Same ordering T-G.2 needs and for the same
> reason.
>
> **✔ CLOSED 2026-09-26.** RFC-0048 is `accepted`; migration `0016` transcribes §E, and
> `platform-tenancy/scripts/seed-landing.py` is the seed. Three things are worth carrying forward:
>
> 1. **The seed is a script, not an INSERT in the migration**, and the reason generalises: a
>    migration is immutable once applied (REQ-0008) and landing copy is **editable content**, so the
>    first correction to a headline would need a second migration to change a string. 0011 backfills
>    because a region is contractual; a headline is neither.
> 2. **Only the PUBLISHED copy is seeded.** `messages/` also holds `hero`, `hosting`, `isolation`,
>    `stage`, `audience`, `ours`, `commitments` and `proof`, and §E's table names several of them —
>    but that column shows the *schema* is adequate for those shapes, not that they should be rows.
>    ADR-0086 made the landing a coming-soon page and STD-0021 governs what it may claim; the
>    hosting offer was pulled deliberately. **A row is not inert:** once §B's renderer reads this
>    table, every row it finds is on the front page, so importing withdrawn copy would republish it
>    as a side effect of running a seed.
> 3. **`0016` has been executed, against a throwaway local Postgres.**
>    `scripts/probe-landing-schema.py` applies it and the seed's own INSERTs and asserts §E's four
>    rules and the RLS boundary — 15 cases, each in its own transaction, **writing as a
>    non-superuser** because `FORCE ROW LEVEL SECURITY` binds a table's owner and nothing binds a
>    superuser. That is not a substitute for `prove-isolation.py`; it means the schema does what its
>    comments claim before anyone applies it to a database holding a tenant.
>
> **Remaining, and it is not this task's:** RFC-0048 §B's renderer — `web-paridyum` reading
> `landing_sections` instead of `messages/`. `0016` is unapplied, so the rows do not exist yet either.

---

### Wave 3 — the first real deploy

#### T-G.12 Careers resolves its tenant from the request
`paridyumTenantId()` is deleted. Public surfaces resolve the tenant from the hostname; an unresolvable
host 404s rather than falling back. The ADR-0099 move, applied to the public half.
**Done when:** two tenants' careers pages are served from one deployment, each showing only its own
postings, and the config path is gone rather than retained as a fallback. RFC-0048 slices 3–4.

#### T-G.13 The deploy saga, one product end to end
`requested → deploying → active`, compensating to the previous Release and never to a deletion.
Careers first because it is the simplest.
**Done when:** a careers Release can be deployed, rolled back, and shown failing honestly in `/dashboard`.
Implements RFC-0049 slice 3.

#### T-G.14 The tenant-scoped extractor — **two of three directions built 2026-09-26; restore is named**
One walk of the declared scopes, pointed three ways: export (Art. 20), restore (from a PITR clone),
delete (Art. 17, with a certificate written to a stream that outlives the data).
**Done when:** a tenant's data can be exported, restored and erased without a superuser and without
hand-written SQL. Implements RFC-0045 slice 5.

> **✔ `platform-tenancy/scripts/extract.py`, 2026-09-26.** Export and erasure, with **no superuser and
> no hand-written SQL**: it reads as `paridyum_app` under the tenant's RLS context, so the archive cannot
> contain another tenant's rows **even if the walk is wrong** — the database refuses them rather than a
> `WHERE` clause in Python.
>
> The walk is derived from the `-- paridyum:scope` declarations, which is §F's "correct by construction":
> 25 exported tables today, one excluded, and a table added tomorrow is included tomorrow.
>
> **The finding came before the extractor worked: all 16 migrations replay in order on an empty
> database**, with only `uuidv7()` shimmed. Nothing had checked that outside the cluster, and it is what
> a restore rehearsal depends on (ADR-0071, T-G.7). It also catches a migration referencing a table
> created two files *later* — which works on the live database and fails every replay.
>
> Three properties worth carrying forward:
>
> 1. **An undeclared table refuses the export.** An omission makes an Art. 20 response *incomplete while
>    looking complete*, and the requester cannot tell. A pending migration refuses it too.
> 2. **`audit_events` is exported and never erased** (ADR-0044 D5 and RFC-0045 §D4). The first draft of
>    the walk returned one list, printed "audit_events is not deleted", and would have deleted it.
> 3. **The erasure certificate is written in the same transaction and before the deletes** — one row per
>    table, because D6 wants *what it was done to*. Written after, a crash between the two leaves the
>    data gone and no record of it going.
>
> **Restore is the remaining direction and is not half-built.** It is the same walk against a PITR clone;
> `--via local --database <clone>` is the connection, and the clone is `infra-kubernetes`'s
> `postgres/restore-rehearsal.yaml`, rehearsed 2026-09-24. What is missing is the step that reads from it
> — deferred because per-tenant restore has no requester, and writing it now means writing
> conflict-resolution rules (what happens to rows that changed after the recovery point) against no case.
> Recorded in that repository's `DECISION-DEBT.md` with the trigger.
>
> **Offboarding** — §F's fourth direction — is export, a retention hold, then erasure. That is a
> procedure over these two and it is in `SOP.md`, not a flag.

---

### Wave 4 — self-hosting and evidence

#### ~~T-G.23 Three infra repositories still have no CI~~ — **done 2026-09-27**: all 20 repositories have CI and 0 gate errors
**Found 2026-09-27 while closing the fourth.** `infra-kubernetes` had no `.github/workflows/` at all —
the repository defining Cilium, Longhorn, the Postgres cluster, its backups, Garage and Zitadel — and it
is now fixed. **`infra-edge`, `infra-substrate` and `infra-terraform` are the same**: no CI, no
`.claude/settings.json`, and two of the three with no `ARCHITECTURE.md`.
**Done when:** each of the three passes `check-repo.sh --root .` and calls it from a workflow of its own.

> **`class: infra` is `ci: false`, and that is not an exemption.** ADR-0021 does not oblige these
> repositories to call the shared workflow; `infra-ci`'s own `ci.yml` settled what that means — *"about
> not being obliged to, not about being exempt from checking itself"* — and `infra-kubernetes`'s new
> workflow is the shape to copy: manifest parsing, the knowledge gate, and `check-repo.sh --root .`
>
> **⛔ THE PATTERN IS WORTH MORE THAN THE TASK. Four repositories in two days reported a passing gate
> while carrying real errors, every one because `check-repo.sh` was run without `--root`** — which makes
> it resolve its own vendored `.paridyum/knowledge` directory and report a cheerful `n/a`. The count so
> far: `platform-tenancy` (11 errors), `lib-auth-typescript` (8), `infra-kubernetes` (3), and
> `infra-terraform` still unmeasured.
>
> **✔ THE CLASS IS CLOSED, 2026-09-27 — and that was the real fix.** `check-repo.sh` and
> `check-migrations.sh` now **refuse** (exit 2) when their own location is a `.paridyum/knowledge` path and
> `--root` was not given, naming the command that works. `n/a` is indistinguishable from a pass to everyone
> who has ever read one, which is the defect ADR-0014's audit found four separate times and which
> `check-migrations.sh`'s own header names — about its inputs, where nobody applied it to its invocation.
>
> **The guard then found a live bug**, which is the part to remember: `rust-ci.yml` called
> `check-migrations.sh` with no `--root`, so for any repository with migrations it scanned this snapshot's
> deliberately-failing fixtures and **could never have passed**. Both templates had drifted the same way.
> Fixed in the same change, because a guard landing without its call sites fixed breaks CI the moment the
> allowance resets.
>
> **This task is now only the three workflows**, and they are worth doing because the repositories are
> unmeasured rather than because the gate is blind.

#### T-G.22 The access log covers more than one path ⏱
**RFC-0022 §D5a, added 2026-09-27, measured six paths to a tenant's data and found one logged.** D5
promises a tenant sees operator access; a promise about access paths is only as strong as its weakest,
and today the application's is covered and five are not.
**Lossy in the same way T-G.2 is:** an access that happened without a record cannot acquire one.
**Done when:** `pgaudit.*` parameters are **applied** (they are written), the apiserver's existing audit
records are shipped somewhere a tenant can be shown rather than rotating on a control-plane node, and
RFC-0022 §D5a's table is updated with what is then true. → produces an RFC-0022 erratum.

> **✔ THREE OF THE FIVE ARE CONFIGURED, 2026-09-27 — and not applied.**
> `infra-kubernetes/postgres/cluster.yaml` and `postgres/restore-rehearsal.yaml` carry `pgaudit.*`
> parameters, which closes the superuser-`psql` path, the DSN-from-outside path, and the clone. It is a
> **control** rather than a log because pgaudit's settings *"may be modified only by a superuser"* — so
> `paridyum_app` cannot switch off its own auditing.
>
> **`pgaudit.log_parameter` is OFF, against CloudNativePG's own example**, because a bound parameter *is
> the data*: `WHERE email = $1` with parameters logged writes an email address into the container log,
> turning stdout into an unclassified personal-data store. `log_relation: on` answers *which rows* without
> copying them.
>
> ⚠ **Applying it restarts Postgres** — `shared_preload_libraries` is not reloadable and `instances: 1`
> means no replica. A short outage, alone and in a window; `infra-kubernetes/SOP.md` has the commands and
> the verification, including that if `SHOW shared_preload_libraries` lacks `pgaudit` the parameters are
> **inert** and nothing else will say so.
>
> **And it does not finish those paths.** The records go to stdout as container logs — rotated locally,
> shipped nowhere. They move from *unrecorded* to *recorded where only an operator can read them*. D5's
> bar is a tenant reading it.
>
> **✘ THE "NO AUDIT POLICY" CLAIM WAS WRONG, corrected the same day.** It was "verified" by a grep ending
> `| grep -v '.paridyum'` — and `.` matches any character, so every path under
> `/Users/rohan/Developer/paridyum/` was filtered out. The command returned nothing and the nothing was
> read as an answer. **Talos v1.13.7 ships kube-apiserver audit logging on by default** —
> `cluster.apiServer.auditPolicy`, `level: Metadata`, writing to `/var/log/audit/kube/*.log` on the
> control-plane nodes — so `kubectl exec` into the database pod **is** recorded. What is missing on that
> path is **reach**, not recording: nothing ships those logs, nothing makes them tamper-evident, and no
> tenant can read them. RFC-0022 §D5a carries the erratum.
>
> **Path 4 — the backup bucket — is partial at best.** Path 6 — raw volume access — is **not closable by
> logging**, and §D5a's first, stronger wording ("not closable at all") was overconfident and is now
> marked provisional: three avenues are named there — memory-encrypted execution with attestation, keys
> the tenant holds, and tamper-evident shipping that changes *detectability* rather than access — none
> researched, each with a trigger to argue it properly. The table is not expected to become all ticks, and
> the conclusion is dated so it is revisited rather than inherited.
>
> ⚠ **And nothing may describe the access log as complete until this closes** (ADR-0041 D4). The partial
> version demonstrates well, which is exactly why it is the capability most likely to be over-claimed.

#### T-G.21 Somebody watches the meter ⏱
**Diagnosed 2026-09-26, and it is the cheap half of RFC-0053.** The org's included 2,000 Actions
minutes a month ran out on **2026-09-20** and the estate found out on **2026-09-24**, from a person
reading a run page. ADR-0002 calls the CI gate the lock; a lock that opens when an allowance lapses is
a lock with an undocumented key, and nothing in the estate read the key.
**This is not the runner work.** Self-hosting (RFC-0053 §§B–H, which has no task number yet) answers
"CI has one path". This
answers "nobody watches the meter", it is one API call, and it would have turned a four-day silent
outage into a note on the Monday.
**Done when:** a scheduled check reads `GET /orgs/{org}/settings/billing/usage`, warns at 80% of the
included allowance and fails at 100%, and `ci.included_minutes_are_watched` is declared in
`check-registry.yaml` with an owner. Implements RFC-0053 slice 1b.

> **The successor endpoint is the finding.** The one RFC-0053 recorded as returning `410 This endpoint
> has been moved` — `/orgs/{org}/settings/billing/actions` — has a replacement that answers with no
> extra token scope: `/orgs/{org}/settings/billing/usage`, optionally `?year=&month=`. Every row
> carries `quantity`, `grossAmount`, `discountAmount` and `netAmount`; the allowance is spent when
> `discountAmount` stops matching `grossAmount`.
>
> **⚠ And the two views of that endpoint disagree.** Called with no `?year=&month=` it returns month
> rollup rows; called with them, daily rows. For September 2026 the rollup showed one
> `Actions storage` row with `discountAmount: 0` and a non-zero `netAmount` — **which is what led to a
> claim that the storage allowance was also spent, and that claim is withdrawn** (RFC-0053's erratum
> carries the withdrawal). The filtered daily rows for the same month sum to `discount == gross`
> exactly, for every SKU. Trust the filtered view: a per-day series that sums to a stated total is
> auditable, and a rollup that contradicts it is not. The minutes finding is unaffected.

> **✔ BUILT 2026-09-26.** `infra-ci/scripts/check-included-minutes.sh` and
> `.github/workflows/watch-included-minutes.yml`. Exit codes are distinct — 0 under threshold, 1
> warning, 2 spent, 3 cannot read — because "spent" and "cannot tell" are different incidents.
>
> **The script is the mechanism and the workflow is a convenience**, because the watcher cannot run on
> the runners it reports on: when the allowance is gone, it will not start either. Run it by hand with
> `gh` whenever runs fail in a few seconds with zero steps — `infra-ci/SOP.md` Task 5.
>
> **What remains for the done-when:** `PARIDYUM_BILLING_TOKEN` (a PAT with `read:org` on an account
> that administers the organisation — `GITHUB_TOKEN` cannot read org billing), and one run that
> completes. Both are operator actions. The check id is declared in `check-registry.yaml`.

#### T-G.15 Provisioning requests the entitled Releases
ADR-0039's saga currently ends at `active` with nothing running. A tenant should become `active` when
its products are running, not when its row is written.
**Done when:** a newly provisioned tenant reaches `active` with its entitled products deployed.
Implements RFC-0049 slice 4.

#### T-G.16 Upgrade, rollback, channel
Version per Release, channel per tenant. Solves RFC-0047's version-skew problem once, for everyone,
rather than inventing it later for partners.
**Done when:** a product can be upgraded and rolled back per tenant, and skew is visible in `/dashboard`.

#### T-G.17 `app` registers its own Release
Genesis versus steady state (RFC-0049 §D): `/dashboard` cannot bring itself into existence, and can manage
its own existence thereafter. ADR-0100 D4 narrows from *the provisioner is not provisioned* to *the
provisioner's first deployment is not provisioned*.
**The mitigations are the task, not the feature.** Rollback must not depend on the new version
working; the genesis path must stay runnable by a human; and a path nobody uses must be exercised on
a schedule or it will not work when it is needed.
**Done when:** `/dashboard` upgrades itself, rolls itself back, and the genesis path has been rehearsed and
scheduled. Implements RFC-0049 slice 6.

#### T-G.18 The isolation attestation
Every field produced by a check rather than typed by a human; no framework claimed as satisfied
(ADR-0041 D4); a field that cannot be produced is absent or `unknown`, never defaulted.
**Done when:** a tenant can obtain a dated, signed statement of its placement, containment, proof
time, recovery point and last privileged access — without asking an operator. Fills REQ-0022's empty
`enforced_by`. Implements RFC-0045 slice 7.

#### T-G.19 docket learns which instance it is serving — **code done (D-70, tool-docket#26); waits on a deploy, the account row, then the migrations**

> **2026-09-28.** The heading below is historical. `tool-docket/src/adapters/pg.rs` sets all FOUR
> session values (tool-docket#26) and every insert carries `account_id`. What remains, in the order
> T-G.3's ⛔ note proves: provision the tenant's docket account row FIRST (the new `enter()` refuses a
> tenant without one, and the live `accounts` table is empty); deploy docket's new image; apply the
> pending migrations minutes later; then `prove-isolation.py` against the cluster.
**⛔ docket's write path is broken until this lands — 0012 applied at 11:29 on 2026-09-24.**
`tool-docket/src/adapters/pg.rs` has no concept of an account: `enter()` sets two session values and
the inserts name no `account_id`, while the column is now NOT NULL in the live database. Reads
serve; writes fail. Not production, and docket can be redeployed once this is in.

It also needs to **create** the account it resolves: `accounts` holds 0 rows, so the
`product='docket', slug='default'` row does not exist yet.

The honest shape is ADR-0104 D5 path routing — `<tenant-host>/<product>/<slug>` — which this
repository does not have. The permitted interim is to resolve the tenant's
`product='docket', slug='default'` account row and carry it explicitly (ADR-0104 cost 2: a default
instance is still an account, never a null). **It must fail loudly when that row is absent** — never
a nil uuid, an `unwrap_or_default`, or a coalesce, which is precisely the widening-by-absence
GLO-0005 and `db.operator_scope_is_an_explicit_grant` exist to forbid.

**Done when:** `enter()` sets all three session values, the five inserts carry `account_id`, and
`prove-isolation.py` proves the docket account boundary behaviourally rather than by configuration.
Full statement of the problem: `tool-docket/DECISION-DEBT.md` D-70.

#### ~~T-G.20 Decide how a non-operator session reads its own account~~ — **done 2026-09-26** → `ADR-0108`
`releases_read` lets a tenant read its own Release without operator scope (ADR-0100 D3 needs that:
`<host>/dashboard` is a signed-in tenant surface). But `accounts` carries exactly one policy,
`accounts_operator`, which is `FOR ALL` and requires `has_operator_scope()` — and `product` lives
**only** on `accounts`, deliberately, because RFC-0015 forbids the second copy.

So a dashboard session can read its Release row and cannot name the product it is an instance of.
Either `accounts` gains a SELECT policy for the owning account, or those sessions carry operator
scope and ADR-0100 D3 means something narrower than it reads. **This was left undecided rather than
answered in a migration** — the estate's rule is that the artifact precedes the code, and this is a
decision about who may read what.

**Done when:** an ADR or an amendment to ADR-0100 settles it, and 0012's `accounts` policies match.
**Blocks:** T-G.9, which cannot render a product name without it.

> **✔ CLOSED 2026-09-26. ADR-0108 D2 chose the first option**, and migration `0015` transcribes it:
> an `accounts_read` policy, `FOR SELECT`, `tenant_id = current_tenant() AND (id = current_account()
> OR has_operator_scope())`. So a tenant session reads **its own** account row and no sibling's, and
> an operator still reads every one of its tenant's.
>
> Worth being precise about what this fixed, because it was not a leak: `accounts` being
> operator-only made a feature **impossible** rather than unsafe. A non-operator could read its own
> `releases` row and could not name the product, because `product` lives only on `accounts`
> (RFC-0015 forbids the second copy). The alternative — giving dashboard sessions operator scope —
> would have made ADR-0100 D3 mean something narrower than it reads, and would have widened a grant
> to serve a display field.
>
> `0015` is written and **not applied**, like `0010`, `0014` and `0016`.

---

### What Genesis deliberately excludes

- **RFC-0047 (renting the platform)** — designed, gated, and blocked on T-G.0c and ADR-0024 D9.
- **RFC-0046 (BYOK)** — designed, build-gated on explicit direction, and blocked on T-G.0b.
- **Quotas** (RFC-0045 slice 8) — real and unowned, but they degrade rather than leak, so they sit
  behind everything that leaks or loses.
- **`paridyum` → `stage: operated`** — a consequence of T-G.7 and the first real workload, not a task.
  The registry's own note is the authority on when it becomes honest.

---

## Standing constraints — check before any commercial call

From C-0006, which `CLAUDE.md` routes to for *"a pricing, market or ICP call"*:

1. **No franchising a vertical Paridyum has not operated** (rewritten 2026-08-15; the original
   forbade a catalogue outright). Breadth without coherence is still forbidden outright.
2. No infrastructure resold at markup as a revenue line — a royalty on a business we supplied is
   not a markup on rails (BET-0007).
3. **No regulated data before controls exist** — the only one that can end the company in a single
   incident (ANTI-0007).
4. Switching cost is not the retention strategy.
5. No enterprise before self-serve works.
6. No second vertical before the first has a renewed paying customer.

And the procedure when something is genuinely open: *"write a BET-, not an opinion in a chat log."*
Check whether an existing bet's `trigger` has fired first — BET-0006 and BET-0007 come up for review
**2026-12-01**, BET-0004 and BET-0005 on **2027-02-01**.

**And one lesson from the 2026-08-15 review, worth carrying:** C-0006 anti-goal 1 and its framings
table both ended "See BET-0003." When BET-0003 was withdrawn, both were left standing on a
withdrawn artifact — and BET-0004 recorded that nothing was contradicted, because re-reading C-0006
found only a pointer back to the bet just withdrawn. **An artifact that outsources its argument to a
lower-precedence one inherits that artifact's mortality.** When you supersede a bet, grep for its id.
