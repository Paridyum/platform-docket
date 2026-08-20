---
id: RFC-0010
title: Four naming gaps close in one supersession — abbreviations, env vars, image names, labels
type: rfc
status: accepted
created: 2026-08-08
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [naming, constitution, abbreviations, configuration, containers, kubernetes, observability]
breaking: true
resolved_by: C-0007
verified_by: []
enforced_by: []
---

# RFC-0010 — Four naming gaps close in one supersession

## Status

**ACCEPTED** as of 2026-08-09, and **resolved by C-0007** the same day.

The feedback window was not waited out. That is a deviation from the stage-2 shape of ADR-0013
and it is recorded rather than glossed: the estate is still zero services, zero charts and zero
deployables, so the migration set for all four additions is empty and the cost of being wrong is
a third supersession rather than a rename of running software. The window exists to protect
something; here there is nothing yet to protect.

It superseded **C-0005 once**. Not ADR-0020, not REQ-0004, not any standard — see *The true
supersession set*, which is smaller than the first draft of this RFC assumed and smaller than
the proposal that prompted it, and which the supersession bore out exactly.

Everything below this line is the proposal as argued, left in its original tense. An RFC is the
record of why a decision was taken, not a description of the decision — C-0007 is that.

C-0005 is one day old and already needs extending. That is not an argument against it; it is
the mechanism working. C-0005 moved the class *list* out of the constitution precisely so the
*grammar* could be amended without the list coming with it. This is the first amendment to the
grammar, and it is deliberately batched — four gaps, one supersession, the same trade RFC-0004
made when it paid three at once.

## Problem

C-0005 covers repositories, crates, binaries, HTTP paths, database objects, events, Helm
releases, Kubernetes namespaces and environments. An external review of naming practice
proposed a 24-domain naming governance tree with its own control IDs, registries and
directory. Audited against this repository, **most of it already exists** — the glossary is
the terminology registry, `class-registry.yaml` is the class registry, STD-0004 plus
`module.no_grabbag_names` is the reserved-word list, and the supersession protocol is the
rename policy for artifacts. Three of its proposals directly contradict locked C-0005 rules
(`production` for `prod`, `user.created` for `UserCreated`, unprefixed repository names).

Four things survived the audit as genuine gaps. They are unrelated to each other except in
one respect that matters: **each is a name, and each currently has no home.** Left alone, the
first agent to need one will invent it, and the invention will be encoded in code before
anyone notices — which is the drift CLAUDE.md's one rule exists to prevent.

## The four gaps

### 1. The abbreviation rule binds one stack and is unusable even there

C-0005's final bullet under **Rust crates and binaries** reads:

> Modules and files: `snake_case`. Types: `PascalCase`. No abbreviations that are not already
> in the glossary.

Two defects, and the first is the one that matters.

**It binds only the `rust` stack.** Two lines later C-0005 states that *"these rules bind the
`rust` stack, not a list of prefixes"* — deliberately, so that adding `web` required no change.
The consequence is that the abbreviation rule inherits that binding. **HTTP paths, JSON fields,
database columns, event names, environment variables and every TypeScript identifier in every
`web-` and `mobile-` repository have no abbreviation rule at all.** The one place Paridyum
states a policy on abbreviation is the one place it was least likely to be a problem, because
Rust module names are also governed by STD-0004 and `module.no_grabbag_names`.

**And within `rust` it contradicts another section of the same artifact.** The Databases
section mandates primary key `id` and foreign keys `<singular>_id`. A Rust struct
deserialising that row has fields `id` and `tenant_id`; those are Rust identifiers, bound by
the rule, and `id` — an abbreviation of *identifier* — is not in the glossary. The glossary
holds five entries, four of them about tenancy. So C-0005 mandates a column name whose only
possible Rust representation its own rule prohibits.

The failure mode is not that agents will over-abbreviate. It is that the rule is unenforceable
as written, so nothing will ever be built to check it, so the actual outcome across five of six
stacks is no policy at all — and `usr_cfg_svc` and `user_configuration_service` are equally
legal.

OpenTelemetry's naming specification solves exactly this and solves it as a *rule* rather than
a list: widely recognised technical abbreviations are permitted (their examples: IP, DB, CPU,
HTTP, AWS); domain-specific abbreviations are permitted **only when qualified by a namespace**;
ambiguous abbreviations are prohibited. That formulation is portable here without modification,
and it is a better fit than the reviewed proposal's `abbreviations.yaml` for the same reason
C-0005 gave for moving the class list: **the grammar is constitutional, and a hand-maintained
approved-words list is a fifth thing to keep in sync.**

Crucially it must be lifted **out of the Rust section** and stated where the stack-binding
sentence does not reach it. This is the only one of the four additions that changes the meaning
of existing text rather than adding to it, and it is the reason a standard could not have
carried it — see Option C.

The glossary is not replaced by this. It stays the authority for *Paridyum-specific* terms —
`tenant`, `account`, `operator` — where the risk is ambiguity, not length. Its role narrows
from "the allowlist of permitted words", which it was never populated to be, to what it
actually is.

### 2. Environment variables are named nowhere

C-0005 names environments (`dev`, `staging`, `prod`). It does not name environment
*variables*, and nothing else does either — a grep across the repository returns no rule.

This is the gap with the shortest fuse. Every service, every stack, every CI workflow and
every Helm chart will set them, and the first one to do so sets the precedent. The observable
outcome elsewhere is four names for one value: `DB`, `DATABASE_URL`, `PG_URL`, `POSTGRES_URL`.

The answer is not contentious and does not need inventing. POSIX permits only letters, digits
and underscore in the portable set; SCREAMING_SNAKE_CASE is the near-universal convention, and
lowercase is conventionally reserved for application-local shell variables. A vendor prefix
avoids collisions with the environment the process inherits.

This one also has a real interaction with REQ-0001: **the name is where a secret announces
itself.** `PARIDYUM_BILLING_DATABASE_URL` and `PARIDYUM_BILLING_LOG_LEVEL` are the same shape,
and a secret-detection rule that keys on names needs the shape to be predictable before it can
key on anything. REQ-0001 is deferred; the naming rule is not blocked by it and should land
first, because retrofitting names onto deployed services is exactly the cost this repository
keeps paying to avoid.

### 3. Container image names are unowned — but the tag policy is not

The reviewed proposal treats "container image naming and tagging" as one gap. It is two, and
they belong in different places. Checked properly:

| Concern | Owned today? |
| --- | --- |
| Registry path — `ghcr.io/<org>/<what>` | **No.** Nothing names it |
| No `:latest` in a deployed workload | **Yes** — REQ-0004 §2, deferred |
| Base image is distroless or slim | **Yes** — ADR-0020, `docker.image_is_distroless_or_slim`, live |
| SBOM and build provenance | **Yes** — REQ-0014, deferred |
| Pinning by digest rather than tag | **No.** Nearest owner is REQ-0004 §2 |

So the naming gap is **only the first row**, and it is small: an image name should be derivable
from the repository name the same way the crate name and the Helm release name already are.
C-0005 already does this twice — `svc-billing` → crate `paridyum-billing`, `svc-billing` →
release `billing` — and an image is the third instance of a derivation rule that already
exists. Writing it down costs one paragraph.

Digest pinning is **not** proposed here. It is a supply-chain control, not a name, and putting
it in the naming constitution would be the two-sources-of-truth failure ADR-0002 exists to
prevent — REQ-0004 §2 already holds the adjacent rule. Flagged as an open question instead,
because the current guidance is unambiguous that tags are mutable per-repository references and
only digests are immutable, and REQ-0004 §2's `no :latest` stops short of saying so.

### 4. Two of six Kubernetes labels

C-0005 names `app.kubernetes.io/name` and `app.kubernetes.io/part-of: paridyum`. The
recommended set is six: `name`, `instance`, `version`, `component`, `part-of`, `managed-by`.

The four missing ones are not decoration. `managed-by` and `instance` are what distinguishes
two releases of one chart; `component` and `version` are what cost attribution, fleet queries
and incident tooling actually group by.

The cost of adding them later is asymmetric and worth stating precisely rather than
dramatically. Adding a label to `metadata.labels` and the pod template is a rolling update of
every workload — disruptive but routine. Adding one to `spec.selector` is not possible at all:
that field is immutable on an `apps/v1` `Deployment`, so it takes a delete and recreate. Which
half you land in depends on a choice nobody has made yet, and the cheap moment to make it is
before the first chart exists.

This is also the gap where C-0005's existing text is closest to already being right, which is
the argument for fixing it in the same supersession rather than on its own: it is two lines.

## Why one supersession rather than four changes

Each gap on its own is too small to justify superseding the most-cited artifact in the
repository. Together they are not. The reasoning is RFC-0004's, applied a second time: **a
supersession that is cheap to batch and expensive to repeat should be batched.** Four separate
amendments to C-0005 would be four full restatements of every naming rule it carries — repos,
crates, binaries, HTTP, databases, events, Kubernetes, environments and the `forge` binary note
— and four opportunities to drop one in transcription. C-0005 got that transcription right
once. Nothing checks it, so the fifth attempt is as unguarded as the first.

The counter-argument is real and belongs on the record: batching four unrelated changes means
accepting or rejecting them as one. If one is wrong, it arrives locked with three that are
right. That is mitigated by their independence — none of the four references another, so any
one can be dropped from the successor before it is written without disturbing the rest — and
it is the reason this is an RFC with a feedback deadline rather than a direct supersession.

## The true supersession set

| Artifact | Status | Needs superseding? |
| --- | --- | --- |
| C-0005 naming | locked | **Yes** — all four gaps are text inside it |
| ADR-0020 Kubernetes deployment target | locked | **No** — it owns base images and probes, not names |
| REQ-0004 runtime and operations | deferred | **No** — `no :latest` stays its; digest pinning is unowned, see open question 3 |
| REQ-0014 supply chain and provenance | deferred | **No** — SBOM and attestation are untouched |
| REQ-0001 runtime secret management | deferred | **No** — consumes the env var shape, does not define it |
| STD-0004 code organization | locked | **No** — grab-bag names are already covered there |
| The glossary | — | **No** — narrowed in role, not contradicted |

**One.** The reviewed proposal implied a new governance tree and a 24-entry control namespace;
the actual cost is one supersession of one artifact. That gap between apparent and real cost is
worth stating, because it is the same gap RFC-0004 found when its first draft guessed four
supersessions and named the wrong three.

## Found while auditing: a check's spec described a list its code does not have

Not part of this proposal, **already fixed**, and recorded here because this is where it was
found — the same disclosure RFC-0004 made when its supersession audit corrected its own first
draft.

`knowledge.no_duplicate_id_namespaces` declared in `check-registry.yaml`:

> Fails when an artifact id uses a prefix outside the set C, ADR, STD, PAT, ANTI, RUN, RFC, or
> when two artifacts share the same id.

Seven prefixes. `artifact-registry.yaml` declares **ten** — the set above plus `BET`, `GLO` and
`REQ`. Twenty-three artifacts carry one of the three missing prefixes, and the gate is green.

The first reading is that the implementation is broken. It is not, and the truth is more
useful: `scripts/check-knowledge.sh` **derives** the prefix alternation from
`artifact-registry.yaml` at startup and builds both its id pattern and its cross-reference
pattern from it. ADR-0022 made the check registry-driven and left no hardcoded list anywhere in
the code. **The prose in `check-registry.yaml` was the only copy of the enumeration that
survived, and it was the copy nothing validated.**

That inverts the usual lesson. The danger was not an unmaintained implementation; it was a
`spec` field restating an enumeration the implementation had already stopped owning. A reader
consulting `check-registry.yaml` — which CLAUDE.md sends them there to do — would have learned
that a third of this repository's artifacts are illegal ids.

The `spec` now describes the derivation rather than the result. **This is a fifth site where a
list was hand-maintained beside a mechanism that derives it**, and the reason no running count
appears in this RFC is that a hand-maintained tally of hand-maintained-list failures would be
the same defect once more.

## What the successor to C-0005 says

Every existing rule carried forward verbatim, as C-0005 did for C-0002. Four additions:

**Abbreviations.** Permitted when widely recognised across the industry, or when
namespace-qualified within a domain. Prohibited when ambiguous or invented to save characters.
The glossary remains the authority for Paridyum-specific terms; it is not, and never was, an
allowlist of English.

**Environment variables.** `SCREAMING_SNAKE_CASE`, prefixed `PARIDYUM_`, then the capability,
then the setting: `PARIDYUM_BILLING_DATABASE_URL`. Letters, digits and underscore only. No
service invents an unprefixed name for a value another service also has.

**Container images.** Derived from the repository name the way the crate and release names
already are: `svc-billing` → `ghcr.io/<org>/billing`. Tag and digest policy is REQ-0004's and
is not restated here.

**Kubernetes labels.** All six recommended labels, with the values derived rather than chosen:
`name` from the release name, `part-of: paridyum`, `component` from the class, `version` from
the chart's `appVersion`, `managed-by: helm`, `instance` from the release instance.

### What each rule binds

C-0005's existing sections bind unevenly on purpose — the Rust rules bind a stack, the API and
database rules bind everything. The four additions must each state which, because gap 1 is a
direct consequence of a rule inheriting a binding nobody chose for it.

| Addition | Binds | Why not everything |
| --- | --- | --- |
| Abbreviations | every stack, every class | the defect being fixed is precisely that it does not |
| Environment variables | classes with `deployable: true` | an SDK is a published artifact and a mobile app bakes config at build time; neither reads a process environment |
| Container image names | classes with `deployable: true` | nothing else produces an image |
| Kubernetes labels | classes with `deployable: true` | ADR-0020's own boundary — `mobile` ships to a store, not a cluster |

`deployable` is already a field in `class-registry.yaml`, so three of the four bind to registry
data rather than to a prefix glob — which is the trap RFC-0004 identified in ADR-0003's and
ADR-0010's `applies_to`, and the reason those needed superseding. **Written this way, none of
the four needs superseding again when a class is added.**

### Enforcement

C-0005 is `locked`, so its successor must be too, and ADR-0002 requires every locked rule to
name a check. Three of the four are mechanical; one is not, and saying so is the point of the
column.

| Rule | Proposed check | Owner | Severity |
| --- | --- | --- | --- |
| Env var shape and prefix | `repo.env_vars_match_scheme` | `tool-forge` | error |
| Image name derives from repo | `docker.image_name_matches_repo` | `tool-forge` | error |
| Six labels present on rendered workloads | `helm.declares_recommended_labels` | `tool-forge` | error |
| Abbreviations | — | — | **none** |

The abbreviation rule gets no check, deliberately. "Widely recognised" and "ambiguous" are
judgement, and a check that approximates them with a wordlist would be the false-confidence
failure ADR-0002 names — worse than no check, because it would look like enforcement. It is
carried as prose in a locked artifact and upheld at review, which CLAUDE.md's *honest current
state* section already describes as the normal condition for roughly half the declared checks.

`helm.declares_recommended_labels` must inspect the **rendered** manifest, for the reason
`helm.declares_probes` already records: a value set from `values.yaml` is invisible to a text
scan of the template.

### Verification — the second bridge, added after this RFC was first drafted

ADR-0030 landed while this was being written and is `locked`, `scope: [org]`. It adds
`verified_by`: not what rejects a violating diff, but what demonstrates the rule holds in
running software. Applying it here changes one row and, more usefully, forces an honest
statement about the other three.

| Rule | `enforced_by` | `verified_by` | Why |
| --- | --- | --- | --- |
| Kubernetes labels | `helm.declares_recommended_labels` | **TST-0043** `policy` | a label present in the chart and absent on the running object is a runtime failure the diff cannot see |
| Env var shape and prefix | `repo.env_vars_match_scheme` | — | static: the name is fully determined by the source tree |
| Image name derives from repo | `docker.image_name_matches_repo` | — | static: a derivation from the repository name |
| Abbreviations | — | — | **neither bridge** |

Only the label rule carries a genuine runtime obligation. The other two names are fully
determined by files a check can read, and naming a test type for them would be the failure
ADR-0030 §6 predicts — *"a rule that is satisfied by adding an irrelevant id"*.

**The abbreviation rule has neither bridge, and that is the most honest row in this RFC.**
Under ADR-0030 §1 an artifact with only `enforced_by` is *defended at the diff and unproven at
runtime*; one with neither is upheld by nothing but a reader. That is the correct outcome for a
judgement rule and it should be visible rather than smoothed over — it is the same disclosure
`check-knowledge.sh` section 4 makes when it prints the enforcement backlog.

If the successor names TST-0043, **`test-registry.yaml`'s TST-0043 entry must name the
successor in `governed_by`**, or `knowledge.verification_mapping_is_symmetric` warns. Its
`governed_by` is currently `[ADR-0003, ADR-0020, REQ-0004]`, and its `checks:` list would gain
`helm.declares_recommended_labels`. That is a registry edit in the same change, not a follow-up
— the symmetry check warns rather than fails, which is exactly the condition under which drift
decays quietly (ADR-0030 cost 2).

## What `breaking: true` actually costs

Nothing, today, and that is the entire argument for the timing rather than a footnote to it.

`breaking: true` is declared because the additions constrain names that repositories will
carry. But no repository carries them yet: no service is deployed, no chart is rendered, no
image is built, no process reads a `PARIDYUM_` variable, and `tool-forge` — the owner of all
three proposed checks — does not exist. The migration set is empty.

| Addition | Repositories that must change today |
| --- | --- |
| Abbreviations | none — and the rule is prose-enforced, so none ever "fails" |
| Environment variables | none — no deployable exists |
| Container image names | none — nothing is built or pushed |
| Kubernetes labels | none — `infra-kubernetes` does not exist |

The counterpart is that a rule with no subjects gets no feedback: nothing will exercise these
until the first service is deployed, and a rule that was wrong will stay undetected until then.
That is a real cost of writing rules ahead of the estate, and it is the one C-0005's own
rationale accepts when it argues the cheapest moment is before the stack arrives.

## Options

### A. One supersession of C-0005 covering all four

As proposed.

**Cost.** A second supersession of the naming constitution in two days. Restating every rule
is mechanical but not free, and the risk is losing a detail in transcription — the same risk
C-0005 carried, which is why it opened by stating that every rule was carried forward
unchanged.

### B. Four separate amendments, as each gap becomes urgent

**Rejected.** Four full restatements, four transcription risks, and the predictable outcome
that the second one never happens — the exact prediction C-0005's own rationale makes about
supersession-per-change, and the one ADR-0009 records as having come true.

### C. A new naming-extensions standard, leaving C-0005 untouched

Superficially attractive: no supersession, and standards are cheaper to change.

**Rejected.** Naming would then have two homes at two precedence levels, and a standard cannot
extend a constitution — under CLAUDE.md's ordering the constitution wins, so the standard would
be legal only where C-0005 is silent, and a reader would have to know which of the two a given
name falls under before they could look it up. The abbreviation gap is *inside* C-0005's text
and could not be fixed from a standard at all.

### D. The reviewed proposal — a `governance/naming/` tree with NAM-001..024

**Rejected outright**, on four counts. It duplicates the glossary, `class-registry.yaml`,
STD-0004 and the supersession protocol. Three of its rules contradict locked C-0005
(`production` for `prod`, `user.created` for `UserCreated`, unprefixed repository names) and
would have to be adopted by superseding it anyway. Its registries for products, projects, teams
and DNS describe an organisation that does not exist — ADR-0009 records what happens when the
ceremony outruns the estate.

And its `NAM-001..024` control IDs are a second identifier namespace for a concern
`check-registry.yaml` already owns. Half of that is already caught:
`knowledge.no_duplicate_id_namespaces` rejects any *artifact* id outside the registered prefix
set, so a `NAM-0001.md` fails immediately. The other half is not — `NAM-` used as a control id
*inside* a YAML file is invisible to every check in the repository, which is precisely how a
parallel namespace would establish itself. The defence against it is that check ids are already
registry data under ADR-0014, not that a gate would stop it.

## Recommendation

**Option A, written as the last amendment to the naming grammar for as long as possible.**

The test for the successor is the one RFC-0004 set for C-0002's: *does the next gap of this
shape need another supersession?* For three of the four the answer is yes, and that is
acceptable — a name added to the constitution is a rule, and rules there should cost something.
For the fourth it is not: **the Kubernetes label set is a list, and lists in this repository go
in registries.** If a seventh label is ever wanted, the successor should already have delegated
the set the way C-0005 delegated classes, rather than being superseded a third time to add a
row. That is the one design decision in this RFC that is not obvious, and it is open below.

Sequence:

1. **Supersede C-0005** with all four additions and every existing rule carried forward.
2. **Declare the three checks** in `check-registry.yaml`, owned by `tool-forge`, which does not
   exist — so they join the declared-not-implemented backlog `scripts/check-knowledge.sh`
   already reports, honestly and visibly.
3. **Update TST-0043 in `test-registry.yaml`** in the same change — `governed_by` gains the
   successor, `checks:` gains `helm.declares_recommended_labels`. Deferring this leaves a
   warning that nothing forces anyone to clear.
4. **Do not touch REQ-0001, REQ-0004 or REQ-0014.** Their triggers govern when their items
   land; this RFC only ensures that when they fire, the names they operate on already exist.

## Open questions

1. **Should the six labels live in a registry rather than in the constitution?** The
   recommendation argues yes on precedent. Against: six is the whole recommended set and there
   is no seventh — a registry for a closed list is the vestigial-concept failure C-0005 warns
   about when it explains why `knowledge` has no class.
2. **What is `<org>` in the image path?** `ghcr.io/paridyum/billing` presumes the GitHub
   organisation name is the registry namespace. ADR-0004 decided the org on the free plan;
   whether the registry namespace is allowed to be the same string is undecided, and the
   rename behaviour argues it should not be. On an organisation rename GitHub *does* transfer
   packages and container images to the new namespace — but says plainly that doing so **may
   break projects that depend on them**, because nothing rewrites the references held by
   consumers. Old profile URLs 404 and team `@mention`s do not redirect either. So coupling
   the image namespace to the org name makes a rename a breaking change for every consumer of
   every image, which is a durable identity choice rather than a formatting one.
3. **Does digest pinning belong in REQ-0004 §2 or REQ-0014?** §2 already says `no :latest`,
   which is the weaker half of the same rule. REQ-0014 owns provenance, and a digest *is* the
   provenance handle. One of them should claim it before the first deployment, not after.
4. **Is `PARIDYUM_` the right prefix, or should it be `PDY_`?** The full word is unambiguous
   and long; `PDY_` is the fallback binary name C-0005 already contemplates for `forge`. Being
   consistent about which short form exists, if any, is cheaper now than after both are in use.
5. **Which namespace does the environment-variable check belong in?** `repo.` is described in
   `check-registry.yaml` as repository *shape*, and an environment variable is not shape — it
   appears in a Helm `values.yaml`, a `Dockerfile` and a CI workflow, three places three
   different owners already check. The candidates are `helm.`, a new `config.` namespace, or
   splitting it across owners. Under ADR-0014 a new namespace is a registry entry, so this is
   cheap to get right and annoying to change once ids are cited in a locked artifact.
6. **Is one check enough for a rule that spans three file types?** A single
   `repo.env_vars_match_scheme` implies one implementation reading Helm values, Dockerfiles and
   workflows. Three narrower checks would each be honest about what they cover and would fail
   more usefully. The precedent cuts both ways: `module.no_grabbag_names` deliberately spans
   repository, crate and module names in one check, and says so.

## Related

- C-0005 — the artifact to be superseded; all four gaps are text inside it
- C-0002 — its predecessor, superseded one day earlier, for the same reason
- RFC-0004 — the precedent for batching supersessions, and for auditing the true set
- ADR-0002 — why a locked rule with no check is a problem, and why a bad check is worse
- ADR-0014 — check ids are registry data; the reason a new check namespace is cheap, and the
  decision the stale prefix list above fails to live up to
- ADR-0022 — artifact types are registry data; declares the ten prefixes the stale spec omitted
  three of
- ADR-0029 — test types are registry data; the source of the TST- id the label rule names
- ADR-0030 — the second bridge; why only one of these four rules carries a verification method,
  and why the abbreviation rule carries neither
- ADR-0020 — owns base images and probes; deliberately untouched
- REQ-0001 — consumes the environment variable shape this defines
- REQ-0004 §2 — owns `no :latest`; the nearest home for digest pinning
- REQ-0014 — owns SBOM and provenance; untouched
- STD-0004 — already owns grab-bag names, which the reviewed proposal duplicated
- ADR-0009 — the estate that arrived without a class because the ceremony was too expensive
