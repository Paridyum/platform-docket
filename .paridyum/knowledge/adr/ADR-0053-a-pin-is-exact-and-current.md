---
id: ADR-0053
title: A pin is exact and current, and a stale pin fails the gate rather than waiting for a robot
type: adr
status: accepted
created: 2026-08-20
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [dependencies, versions, pinning, renovate, toolchain, supply-chain, std-0003, adr-0005, adr-0048]
breaking: false
enforced_by:
  - check: deps.pins_are_not_stale
    severity: warn
    rationale: >
      Currency asserted by a robot nobody installed is not currency. This check observes the
      age of the pin directly, so the policy survives the automation failing or never being
      built — which is the state it was written in.
---

# ADR-0053 — A pin is exact and current

## Status

**ACCEPTED** as of 2026-08-20. Closes T-2.12.

## Context

**The operator's direction is to stop hard-pinning and stay on latest versions.** Read literally
that contradicts the locked artifacts governing pins, and read carefully it contradicts none of them.

STD-0003 is `locked` and requires an exact version in `rust-toolchain.toml`, never
`channel = "stable"`. Its erratum states the intent precisely: *"The **shape**, not the number: an
exact version, with the four components listed, and never a channel."* It says nothing whatever
about how old that number may be — and it notes in passing that the pin *"will move again roughly
every six weeks."* ADR-0005 is `locked` and names `floating stable` as the rejected option.

**So the apparent conflict dissolves once the axes are separated.** Exactness is about
reproducibility: does this commit build the same way tomorrow. Currency is about age: is the pinned
version the newest one. They are independent, and the opposite of *latest* is not *pinned* — it is
**stale**.

**Staleness is the failure this organisation has actually suffered.** T-2.10: a globally pinned
rustup default served a compiler eleven months old in August 2026, and it surfaced only because
`cargo install sqlx-cli` refused to build against it. T-2.12 records that the pin then moved
repeatedly inside a single week, each move a judgement made from scratch, with no rule saying when
one was due. Rust ships every six weeks, so this recurs indefinitely.

**And the mechanism that was supposed to prevent it does not exist.** ADR-0010 states *"Renovate
handles it"*; ADR-0021 states *"Renovate keeps the pins current."* There is no Renovate
configuration in any repository in this organisation and there never has been. Both sentences were
true of nobody for the entire period they were written down.

## Decision

**1. The pin stays exact. What changes is how fast it moves.** STD-0003 and ADR-0005 are unchanged
and neither is superseded. Every pinned artefact names an exact version; a floating channel stays
forbidden. A floating channel does not deliver currency, it delivers an **unknown** version, and it
destroys the ability to reproduce any past build in exchange.

**2. An upstream release is the trigger, not a calendar.** Upstream does not ship on our schedule,
and a fixed cadence drifts from the thing it tracks. The update opens as a pull request when the
release lands.

**3. A bump is verified before it lands, by the checks T-2.12 named**: `fmt`, `clippy` with
`--all-targets --all-features`, the test suite, and `cargo deny`. No bump merges red. This is the
part that was being done by hand each time.

**4. The gate decides, and a red pull request is the notification.** Patch, minor and toolchain
bumps automerge when the whole of decision 3 passes. When it does not pass, nothing merges and the
failing check *is* the message that the version is not drop-in — which check failed, and on what.
Renovate carries upstream's own release notes into the pull request body, so the human arriving at a
red bump reads the maintainer's account of the change rather than reconstructing it.

**Major bumps never automerge, however green.** ADR-0005 settles that: a major `axum` or `sqlx`
upgrade is *"an org-wide coordinated event."* The toolchain is in the automerging set deliberately —
a compiler bump introducing new lints fails CI, which is exactly the wanted signal, and a red pull
request already stops it.

**4a. Nothing automerges until it has been published for fourteen days.** Renovate's
`minimumReleaseAge`, set org-wide. Security-driven updates are exempt and keep decision 6's
immediate path.

**This is a separate control from decision 3 because it defends against a different thing, and the
distinction is the reason both exist.** A green build proves the new version is **compatible** with
our code. It proves nothing about whether the new version is **trustworthy** — a compromised release
compiles, passes the suite, and satisfies `cargo deny`, which only knows about advisories that have
already been filed. Compatibility is what tests can establish; trustworthiness is what elapsed time
and other people's incident reports establish. The cooldown buys the second at the cost of being a
fortnight behind on things that are not urgent, which is a price worth paying because nothing here
needs a crate the week it ships.

**5. A dependency raising its own floor is itself a trigger.** T-2.12 names this case, and it is how
T-2.10 was discovered rather than declared.

**6. A security advisory bypasses the cadence entirely.** RustSec via `cargo-audit` is not a
currency question and does not wait for the ordinary flow.

**7. Staleness fails the gate, independently of the update robot.** `deps.pins_are_not_stale`
observes the age of the pin as it finds it, whatever did or did not open a pull request. `warn` at
the inner bound and `error` past the outer one. **The bounds are registry data**, per ADR-0048: they
track upstream's release cadence, which is a cadence we do not set, so embedding them in this
artifact would rebuild the defect T-2.11 is still open about.

**8. Mutable tags stay forbidden, and for a different reason than staleness.** ADR-0021 requires
third-party GitHub Actions pinned to a commit SHA because *"a mutable tag on a third-party action is
remote code execution with extra steps."* Container images are pinned by digest on the same
argument. `:latest` is not a currency policy; it is an unpinned supply chain, and this decision does
not license it anywhere.

## Consequences

**Accepted costs**

1. **A bump pull request to review on upstream's cadence, per stack, indefinitely.** That is the
   price of currency and it does not amortise. Automerge is what keeps it from becoming a tax, and
   automerge is itself cost 2.
2. **Automerging patch and minor still means a dependency changes without a human reading its
   diff** — decision 4a narrows this rather than closing it. What remains after the cooldown: a
   compromise nobody detects within the window lands automatically, and so does a behavioural change
   that compiles and passes a suite whose coverage did not happen to reach it. The residue is
   accepted knowingly, because the alternative is a human reading the diff of a dependency they did
   not write and cannot meaningfully audit, which buys the appearance of control rather than control.
   Revisit the first time `cargo deny` or the cooldown catches something real, and treat that as
   evidence rather than as the system working.
3. **Deliberate staleness on every non-urgent update, for the length of the cooldown.** This is in
   direct tension with the currency decision 7 enforces, and the tension is real rather than
   resolved: the staleness bounds must be set wide enough to accommodate the cooldown, or the gate
   will fail on pins that are behaving exactly as designed.
4. **Decisions 2, 4 and 4a describe a machine nobody has built.** Renovate does not exist. Until it
   does, this ADR is enforced by compliance rather than machinery, which is the state most of this
   repository's locks are already in (ADR-0014).
5. **Decision 7 duplicates what the robot is supposed to do.** Deliberate, and the reason is in the
   rejected alternatives.

**Gained**

- T-2.12 closes with a rule rather than a note explaining why there is no rule.
- An eleven-month-old compiler becomes something the gate reports rather than something a failed
  `cargo install` reveals.
- The operator's stated intent — always on latest — becomes the enforced default instead of an
  aspiration that depends on someone remembering.

**Rejected alternatives**

- **Genuinely floating: `channel = "stable"`, ranges with no committed lockfile.** This is what the
  instruction says if read literally. Rejected because it contradicts locked STD-0003 and locked
  ADR-0005, would require superseding both, and — the substantive reason — it does not deliver what
  was asked for. It delivers an unknown version per machine per day, re-creating precisely the
  failure STD-0003 exists to prevent: CI disagreeing with your machine the week a release lands,
  with no way to reproduce what shipped.
- **Trust Renovate alone, and skip decision 7.** The obvious shape, and the one already tried
  without anyone noticing. ADR-0010 and ADR-0021 both asserted Renovate kept pins current while no
  configuration existed. An advancement policy enforced only by automation degrades silently to
  *"whatever we last merged"* the moment the automation stops, and nothing reports the difference.
  The check is what makes this decision falsifiable.
- **A fixed calendar cadence** — bump everything monthly. Rejected on ADR-0048's test: the thing
  being tracked changes on a cadence we do not control, so a schedule of ours drifts from it in both
  directions, bumping when nothing shipped and waiting when something did.
- **Leave it ad hoc**, which T-2.12 offered as a legitimate outcome. Rejected on the record: the
  ad-hoc period produced a compiler nearly a year old and a run of bumps in one week, each argued
  from scratch.

## Owed

- **Renovate configuration**, in every repository or once at the organisation level. Named by two
  locked ADRs and absent from all of them. Nothing in decisions 2 through 6 operates until it exists.
- **The staleness bounds as registry data**, per decision 7.
- **`deps.pins_are_not_stale` implementation** — declared with `owner: tool-forge`, which under
  ADR-0014 means owed rather than built.

## Related

- STD-0003 · ADR-0005 — the locked exactness rule this decision leaves untouched, and its erratum
- T-2.12 — the open task this closes · T-2.10 — the incident that motivated it · T-2.11 — the
  embedded-version defect decision 7 avoids repeating
- ADR-0048 — a value on a cadence we do not control is registry data
- ADR-0021 — SHA-pinned actions, and why decision 8 is a supply-chain rule rather than a freshness one
- ADR-0010 — the other artifact asserting Renovate keeps pins current
- ADR-0014 — declared versus implemented, which is the honest status of most of this
