# Paridyum — Tasks

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

### T-0.6 ⛔ → Decide vertical one
BET-0004 *"does not settle the vertical order."* **BET-0005 supplies the criterion that was
missing:** not *which vertical sounds attractive* but **which one can Paridyum plausibly operate as
a real business, in India, without touching regulated data.** ANTI-0007 and C-0006 anti-goal 3 are
hard filters and remove more candidates than they appear to — anything payments-adjacent, health, or
regulated-financial is out.
**Blocks:** Phase 3b, which is now where first revenue comes from.
**Done when:** a BET- or ADR names vertical one and the reasoning that selected it.

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

### T-0.11 ⛔ Accept or reject RFC-0022
RFC-0022 designs the entire defence for BET-0005 cost 1. **Open question 1 was answered on
2026-08-15** — C-0006 non-commodity item 4 is narrowed: aggregate data may improve the runtime and
set product defaults, and may never inform where Paridyum competes. Four boundary options
(vertical, territorial, sunset, retail-price handicap) were considered and rejected as
physical-franchise mechanics that do not transfer to a white-label digital business.
**Still open:** questions 2–5, of which 2 (can operator access actually be logged under ADR-0024's
shared database) decides whether D5 is a guarantee or a partial one — and a partial one must be
described as partial.
**Blocks:** the first franchised tenant — D1, D2 and D5 are worth a fraction if made after the fact.
**Done when:** RFC-0022 is `accepted` or `rejected`, and D2a's check is declared in
`check-registry.yaml` under ADR-0014.

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

### T-2.12 → No policy says when the Rust pin is advanced
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
