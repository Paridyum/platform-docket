---
id: RFC-0020
title: An external fact is cited and dated, because it changes somewhere else
type: rfc
status: proposed
created: 2026-08-12
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [governance, provenance, sources, staleness, adr-0035, adr-0011]
breaking: false
enforced_by:
  - check: knowledge.external_url_is_a_declared_source
    severity: warn
    rationale: >
      A URL in prose with no date reads as settled and cannot be reviewed; a locked artifact
      cannot be corrected in place, so this reports rather than blocks.
  - check: knowledge.sources_are_reviewed
    severity: warn
    args: { max_age_days: 365 }
    rationale: >
      A stale source is not a defect — the artifact was right when written. This is the
      review list, not a build break.
---

# RFC-0020 — An external fact is cited and dated

## Status

**PROPOSED** as of 2026-08-12. Feedback deadline **2026-09-11**.

Governance of the governance, so it sits inside REQ-0006's scope.

## The question

Two decisions written this month rest on facts this repository cannot re-derive.

RFC-0014 places Finland, Sweden, Poland and Spain in **one** jurisdiction, and the entire
fourteen-jurisdiction namespace follows from that. The reason is GDPR Art. 1(3). RFC-0019 declares
that a Proxmox cluster never spans datacentres, and every other decision in it follows. The reason
is corosync's 5 ms latency requirement.

Neither reason is in this repository. Both were read on a web page, on a date, and **both can change
without anything here noticing.** A regulation is amended. A vendor revises a documented limit. The
artifact keeps asserting what was true when it was written, in the confident present tense, and the
decision resting on it stays green.

ADR-0035 has already had this argument in a narrower form:

> **Counts in prose are snapshots, not claims.**

Its subject was registry counts, and its remedy was to stop stating them, because the registry can
always be asked. **An external fact is the same shape and strictly worse**, because there is nothing
here to ask. It cannot be re-derived, only re-read — and nobody re-reads a sentence that reads as
settled.

## Decision 1 — an external fact is declared in `sources`, with the date it was read

**An artifact whose decision rests on a fact this repository does not control declares that fact in
a `sources` block**, added to `knowledge.schema.json`:

```yaml
sources:
  - url: https://pve.proxmox.com/wiki/Cluster_Manager
    title: Proxmox VE — Cluster Manager
    retrieved: 2026-08-12
    supports: >
      Corosync requires under 5 ms between all nodes. Decision 1 rests entirely on this.
```

`url`, `title` and `retrieved` are required; `supports` is optional and worth writing whenever an
artifact cites more than one source.

**`retrieved` is the load-bearing field.** It converts an assertion into a dated observation, which
is the only honest form for a fact that can change without telling you. A link with no date says
*this is true*; a link with a date says *this was true on the twelfth of August, and you should look
again*.

**`supports` is what makes a later review cheap.** Six months on, the question is never "are these
links still good" — it is "which decision moves if this one changed". A source that names the claim
it backs answers that in a line. One that does not sends the reviewer back through the whole
argument.

## Decision 2 — what counts, and what does not

**External** — declare it: a law or regulation; a vendor's documented behaviour, limit or pricing;
a standard's contents; a published benchmark; any capability of a product we do not build.

**Not external** — do not declare it: anything in this repository; anything in a repository we own;
a value in a registry here, which ADR-0035 already governs and which is re-derivable on demand; and
reasoning, which is argued rather than cited.

The line is **re-derivability**, not formality. If someone can answer the question by running a
command in this estate, it is internal and needs no source. If the only way to check is to go and
read someone else's page again, it is external and carries a date.

**A dead link still beats no link.** URLs rot, and the objection is real. But a rotted URL still
names what to search for and when it was read, which is strictly more than an unattributed
assertion gives a reviewer.

## Decision 3 — three mechanisms, none of them new machinery

1. **Completeness is the schema's job.** `sources` entries require `url`, `title` and `retrieved`,
   so `knowledge.frontmatter_valid` already rejects an undated source. Verified by planting one.
2. **`knowledge.external_url_is_a_declared_source`** — a URL in the body that the frontmatter does
   not declare. Warns rather than errors, because a locked artifact cannot be corrected in place.
   Code spans and fenced blocks are excluded: RFC-0012 carries a URL *template* in an example, and
   an example is not a citation.
3. **`knowledge.sources_are_reviewed`** — a `retrieved` date older than the budget, defaulting to
   365 days and overridable per ADR-0036. Warns, and produces the review list this RFC exists for.

**Both new checks warn rather than fail**, deliberately. A stale source is not a defect — the
artifact was right when written. It is a review item, which is exactly what ADR-0011's ledger is for
and exactly the posture ADR-0035 took toward the counts it found.

## Consequences

**Accepted costs**

1. **`STD-0005` cannot be fixed.** It cites Keep a Changelog and SemVer in its body with no
   provenance, and it is `locked` — so the warning fires on it and can only be cleared by a
   supersession nobody needs. That is the rule working, and it is also a permanent warning. Recorded
   rather than special-cased, because an exception list is how a check becomes decoration.

2. **Judgement decides what is external**, and decision 2's line will be argued about. A rule that
   needs judgement cannot be fully checked, and the checks here catch the mechanical half only — a
   bare URL, and an old date. An artifact that states a regulatory position with no link at all
   passes every check in this RFC.

3. **Warnings accumulate.** At a 365-day budget, everything written this month reports in August
   2027. That is the intent, and it is also how a warning stream becomes background noise — ADR-0035
   hit exactly this and answered it with an aggregate plus a baseline. If the count here grows past
   a handful, the same treatment applies.

4. **One more thing to write.** A `sources` block on a dense RFC is thirty lines of frontmatter.

**Gained**

- The two RFCs written this month stop being assertions and become dated observations, which is what
  they always were.
- A review has a starting point that is not "re-read everything": the sources with the oldest dates,
  and the claims they name.
- The provenance of a decision survives the person who made it, which is the same reason ADR-0011
  exists.

## Alternatives considered

- **Cite in prose, as footnotes.** Familiar, and rejected because prose is not checkable and has no
  date field. It is also how STD-0005's two citations came to have no provenance.

- **A separate `sources/` directory of citation artifacts.** More structure, deduplicates a source
  used twice. Rejected on the same argument RFC-0005 used against a separate strategy repository: a
  fact that lives away from the decision it supports is a fact nobody reads while making the
  decision.

- **Require a `review_date` per source rather than an age budget.** More precise, and it asks the
  author to predict how fast a law changes. The budget makes the default correct and lets ADR-0036's
  argument override it where a source is known to move faster.

- **Error rather than warn.** Rejected on decision 3's argument: a stale source is not a defect, and
  a check that fails the build for something nobody did wrong gets suppressed within a month.

## Open questions

1. **Does the 365-day budget belong per artifact type?** A regulation and a vendor's pricing page
   decay at different rates. ADR-0036 says a check's parameters come from args rather than prose, so
   the mechanism exists; whether the granularity is worth it is unproven.

2. **Should `sources` be required for any artifact type?** Today it is optional everywhere. A `bet`
   makes a market claim and a `requirement` often cites a regime, so either could plausibly require
   at least one. Requiring it before there is a habit of writing it would produce empty blocks.

3. **Does this reach beyond `knowledge/`?** A service's `manifest.yaml` may assert a vendor
   capability with the same decay. Out of scope here, and worth asking once one exists.

## Reversal conditions

- **Reverses if the warnings are routinely ignored**, which would mean the review this RFC promises
  is not happening and the block is documentation rather than a control. ADR-0035's baseline check
  is the precedent for catching that: measure whether the count moves.
- **Decision 3's warn-not-error reverses** if a stale source is ever found to have carried a decision
  that was wrong for a year. That would make it a defect after all.

## Related

- ADR-0035 — counts in prose are snapshots not claims; this is the same argument for facts held elsewhere
- ADR-0011 — the anti-forgetting ledger, and the review-list posture decision 3 adopts
- ADR-0036 — check parameters come from args, which is how the age budget is set
- ADR-0031 — errata are additive and dated, the other place this repository dates a correction
- REQ-0006 — governance of the governance, whose scope this sits inside
- REQ-0012 — vendor governance; a vendor's documented behaviour is the commonest external fact
- RFC-0014, RFC-0019 — what prompted this, and the first artifacts to carry `sources`
- RFC-0005 — why a fact lives beside the decision it supports rather than in its own repository
- STD-0005 — the locked artifact whose two uncited references consequence 1 records
