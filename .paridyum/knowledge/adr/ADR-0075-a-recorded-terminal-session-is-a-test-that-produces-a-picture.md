---
id: ADR-0075
title: A recorded terminal session is a test that produces a picture, and it is committed
type: adr
status: accepted
created: 2026-09-09
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, tool, console]
applies_to: ["*"]
tags: [documentation, vhs, terminal, demo, std-0002, std-0020, adr-0065, adr-0005, tool-registry, determinism, ci]
breaking: false
enforced_by: []
sources:
  - url: https://github.com/charmbracelet/vhs
    title: VHS — your CLI home video recorder
    retrieved: 2026-09-09
    supports: >
      VHS executes a declarative `.tape` script against a virtual terminal and emits GIF, MP4,
      WebM or a PNG frame sequence. Tapes carry `Type`, `Enter`, `Sleep` and `Wait`, the last
      matching on a regex, which is what makes a tape assert rather than merely wait. It is MIT
      licensed, latest release v0.11.0 (2026-03-10), and requires `ttyd` and `ffmpeg` on `PATH`.
      Those four facts are what decisions 1, 3 and 5 rest on.
  - url: https://api.github.com/repos/charmbracelet/vhs
    title: "charmbracelet/vhs — repository metadata"
    retrieved: 2026-09-08
    supports: >
      The project's licence, activity and release state at the time of adoption, read as the
      exitability check REQ-0012 asks for before depending on a third-party tool. Distinct from the
      repository page already cited: that documents what it does, this documents whether it is
      alive.
---

# ADR-0075 — A recorded terminal session is a test that produces a picture

## Status

**Accepted**, 2026-09-09. Not `locked`: decision 2's rule — a tape's commands appear in the
document beside it — is a review judgement, and decision 5's CI requirement cannot be asserted
until a repository actually carries a tape. It graduates when `check-repo.sh` can fail a
repository whose committed cast is older than the tape that produced it.

## Context

STD-0002 already contains the strongest documentation rule this estate has, and it is stated
twice: *"Every command in a README is verified in CI"*, and of doc examples, *"a doc example that
stops compiling fails CI, so the documentation cannot drift from the API. Prose can lie; a doc
test cannot."*

That rule has a hole, and `tool-docket` is standing in it. Its documentation set is now
`README.md`, `RUNNING.md`, `HIRING-PLAYBOOK.md`, `HR-CHECKLIST.md` and an 837-line `SOP.md`, and
the thing all five describe is a **sequence**: run `docket next`, read what it printed, run what
it told you, run it again. A per-command check proves each command exists. It proves nothing
about the loop, about the refusals that are the tool's actual safety properties — `docket
advance` refusing an unsigned NDA, the lock naming its holder — or about what the operator sees.

The estate's honest position today is that those are verified by a person, occasionally, and
`SOP.md` carries pasted output that was true when someone pasted it. **Pasted output is prose
wearing a monospace font,** and it drifts exactly like prose: `scripts/serve.sh` exists because a
stale binary served a wire contract the console had moved past, and nobody noticed until an
operator was shown a redacted React error.

Against adopting anything: rendering already took ADR-0065, RFC-0040 and STD-0020 between them,
ADR-0020 does not ask for this, and the estate has one tool per job by preference. A GIF in a README is also the
canonical example of documentation that looks like effort and communicates nothing — a
seven-second loop of text too small to read, restarting before it is understood.

The counter is that VHS's input is not a recording. It is a script, and a script that runs is a
test. That is the distinction this ADR turns on, and it is why the decision is about tapes
rather than about GIFs.

## Decision

### 1. The tape is the artifact. The cast is a build output that happens to be committed

A `.tape` file is source: reviewed, diffed, and run in CI. The GIF, MP4 or frame sequence it
produces is committed **only** so a reader on GitHub sees something without a toolchain, in the
same relationship ADR-0065 gave Archify's rendered HTML.

A cast regenerated without its tape changing is noise in the diff. A cast whose tape has changed
and which was not regenerated is the drift this ADR exists to prevent, and it is the check named
in the Status section.

### 2. A tape asserts, or it is a screensaver

Every tape uses `Wait` on a regex matching text the command must produce. A tape built only from
`Type`, `Enter` and `Sleep` proves the binary did not hang; it does not prove it did the right
thing, and it will keep passing after the output becomes wrong.

**Record the refusals, not only the happy path.** `docket advance` refusing an unsigned
agreement, and the lock printing `held by tabla-01 pid 4192, rohan, 14:02`, are the two most
valuable recordings in this estate, because they are the behaviours a reader does not believe
until they see them and the ones a regression would silently remove.

**The commands in a tape appear in the prose beside it**, copy-pasteable. The recording is never
the only place a command exists — a reader who cannot copy from a GIF must not be forced to.

### 3. GIF for the reader, and the frame sequence is the reason this is affordable

Commit the GIF; it renders on GitHub with no player and no toolchain. MP4 and WebM are permitted
where length makes a GIF absurd, but a recording long enough to need a codec is usually a
document that should have been written.

`Output frames/` is where the leverage is: a PNG sequence is diffable per frame, so a failing
recording can be inspected rather than re-watched. Use it when a tape fails and you are trying to
see where.

### 4. Where a recording belongs, and where it does not

**Belongs:** `tool-` and `console-` repositories, in `README.md` for the one-command first run,
and in `RUNNING.md` or an SOP for a sequence whose ordering or refusal is the point.

**Does not belong:** `ARCHITECTURE.md` — that is STD-0020's territory and a recording of a
running program says nothing about how it is built. Not in `lib-`, which has no terminal. Not in
`web-`, where the artifact is a page and Playwright already takes screenshots. And never as the
sole statement of a command.

**One tape per document, as a ceiling.** A page with four recordings is a page nobody scrolls.

### 5. It runs in CI on the schedule of the thing it records, and it never runs in a delivered path

VHS is `tier: supporting` and its network posture is `none`: it drives a local terminal and
writes a local file. It is a documentation-build dependency, not a runtime one, and no delivered
artifact may depend on it.

**Its two dependencies are declared rather than assumed.** VHS needs `ttyd` and `ffmpeg` on
`PATH`. Both enter `tool-registry.yaml` in their own right with `pin_home: os`, because the
registry's rule is that every pin has one home and a transitive binary that nothing declares is
exactly the dependency that breaks a build on a machine that is not the author's.

### 6. A tape that records real data is a defect

Tapes run against fixtures. `tool-docket`'s recordings use `fixtures/`, never `people/` or
`store/`, which are gitignored for the reason that decides this: they hold personal data, and a
committed GIF is a publication.

**This is not a convention, it is the same rule as ANTI-0007 arriving through a new door.** A
frame containing a real date of birth is a disclosure that survives in git history after the file
is deleted, and no retention clock in ADR-0073 can reach it.

## Consequences

**Accepted costs.**

- **Three pins, not one.** VHS, `ttyd` and `ffmpeg`, and the latter two are `pin_home: os` —
  which is the weakest pin the registry has, the same one `python3` carries with its
  RETROSPECTIVE note. A documentation build that depends on three unpinned-by-us binaries will
  break on someone's machine.
- **Determinism is asserted, not proven.** VHS's documentation does not address frame-level
  reproducibility, and timing-based rendering plausibly differs across machines. If casts churn
  in diffs without behaviour changing, the answer is to commit them less often — not to stop
  regenerating them.
- **A committed binary in every repository that adopts this.** GIFs are large, they change
  wholesale, and git stores each version forever.
- **Another thing that can fail CI for a reason unrelated to the code**, on a project where the
  gates are already the slow part.
- **The temptation this creates is worse than the cost.** A recording is more fun to make than a
  paragraph is to write, and decision 4's ceiling exists because that pressure is real.

**Gained.**

- STD-0002's rule extends from commands to sequences: the loop, the ordering and the refusals
  become things CI can fail on rather than things a person confirms occasionally.
- `SOP.md`'s pasted output stops being the estate's largest body of unverified prose.
- The refusals — the lock, the stage gate — become demonstrable, which is what makes a reader
  trust that the safety properties are real rather than described.

**Rejected alternatives.**

- **Asciinema.** The closer comparison, and it loses on the axis this ADR turns on: its input is
  a captured session, so what is reviewed is a recording of what someone did. A `.tape` is a
  script a reviewer reads in a diff. It also needs a player, where a GIF needs a browser.
- **A shell script producing a text transcript, diffed in CI.** Cheaper, no new pins, and it
  genuinely covers decision 2's assertion half — this is a real alternative, not a strawman. It
  loses only the reader-facing half, and if the pins in decision 5 prove painful this is the
  fallback rather than abandoning the rule.
- **Hand-pasted output in SOPs, the status quo.** Rejected in the Context: it is the drift.
- **Screen-recording by hand.** Not reviewable, not reproducible, not diffable, and it embeds
  whatever was on the operator's screen — which is decision 6's disclosure, with a webcam-era
  failure mode attached.
- **Nothing at all.** Defensible, and the honest reason it loses is `tool-docket` specifically:
  five documents describing one loop, with the safety properties stated in prose. That is the
  case the estate has, and it is the case this addresses.
