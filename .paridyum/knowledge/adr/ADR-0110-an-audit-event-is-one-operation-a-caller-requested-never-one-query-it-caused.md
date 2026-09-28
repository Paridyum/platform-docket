---
id: ADR-0110
title: An audit event is one operation a caller requested, never one query it caused
type: adr
status: accepted
created: 2026-09-26
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, web, tool]
applies_to: ["*"]
product: [tenancy]
tags: [audit, observability, adr-0044, rfc-0045, rfc-0022, tenancy, correlation-id, granularity]
breaking: false
enforced_by: []
---

# ADR-0110 — An audit event is one operation a caller requested, never one query it caused

## Status

**ACCEPTED** as of 2026-09-27 by the operator. Proposed 2026-09-26. Unblocks `tool-docket`'s audit
writer (its `DECISION-DEBT.md` D-72), and **describes what `platform-tenancy` already does** — which is
the argument for deciding it now rather than after a second implementation disagrees with the first.

**What acceptance releases:** docket writing audit events at all. It wrote none, and the reason was not
effort: the unit was undecided, and a CLI that audited per query would write forty rows for one
`docket check` while a served request wrote one for the same work.

**Decision 5 was reviewed before acceptance and kept**, against a specific proposal to narrow it to
reads of person-bearing tables only. Decision 7 is what the review added instead: the cost is now a
measured number rather than an intuition. The reasoning for keeping it is in decision 5 and is worth
reading before anyone proposes the narrowing again — it turns on a distinction the estate cannot
currently draw.

## Context

ADR-0044 decision 6 fixes **what** an audit record contains — who, what, when, where from, the
correlation id, and the outcome including denials. It is silent on **how many records one action
produces**, and that silence is load-bearing in a way that only became visible once something wrote to
the stream.

`platform-tenancy` now writes an audit event for `GET /v1/releases`: one row for the read, one for a
refusal. That read is a single SQL statement, so the question did not arise. **`tool-docket` is where it
arises**, and it arises twice over:

- **It is a CLI and a served API from one codebase.** `docket check` opens several transactions and
  reads four sealed chains, a person store and a registry. `docket issue` reads, seals and appends.
  Audited per store call, one command produces tens of rows; audited per command, one.
- **A store call is not a thing a caller asked for.** Nobody requests a `SELECT` on
  `docket_signatures`. They request a document. An audit stream whose rows are query-shaped records
  what the implementation did, and an implementation detail that changes — a query merged, a cache
  added — changes the audit trail for an action nobody changed.

There is a real pull in the other direction, and it is not weak: **the finest granularity is the safest
for an investigation.** If a row exists per table touched, then "was this person's record read on the
14th" is answerable exactly. Per operation, it is answerable only as far as the operation's name
describes what it touched.

## Decision

**1. One audit event per operation a caller requested.** An HTTP request is one operation. A CLI
invocation is one operation. A saga step is one operation. The queries each causes are not operations.

**2. `object` names what the operation was about, not the tables it read.** `releases`, `person:
asha-verma`, `document:PAR/CRT/2026/0007`. It is the noun the caller would use, because the person
reading the trail during an incident is looking for the noun they were told about.

**3. Where one operation legitimately touches several silos, it is several events — one per silo.**
This is the exception that keeps decision 1 from hiding a boundary crossing. An operation spanning two
accounts is two events, because a single row saying "both" is exactly the record that cannot answer
GLO-0004's question. `extract.py`'s erasure already does this, writing one certificate row per table
for the same reason: a single row saying "everything" cannot be checked against a count.

**4. The correlation id is what reassembles them.** Several events sharing one id are one action's
footprint (ADR-0044 D1). That is what makes decision 3 cheap and what makes per-operation granularity
recoverable: an investigator groups by correlation id and sees the whole action, whatever it spanned.

**5. Reads are audited, not only writes.** ADR-0044 D6 wants what was *attempted*; a read of a person
record is the access a data subject would ask about. This is stated because "audit the mutations" is the
industry default and it is the wrong default for a store holding personal data.

**The narrowing that was proposed and rejected, because the reason is the load-bearing part.** The
obvious economy is to audit reads only of person-bearing tables and leave operational reads —
`releases`, `landing_sections` — unrecorded. It halves the volume and it cannot be implemented
correctly, for a reason that is about the model rather than about effort:

**RFC-0022 D5 is a promise about *Paridyum's* access to a tenant's data, and this estate cannot
currently tell Paridyum's access apart from a tenant's own.** Since ADR-0109, `operator` means *the
tenant's own administrator*; there is no marker on a session that says "this principal is the platform
rather than the customer". So "audit only the accesses D5 is about" is not a filter that can be written.
Until that distinction exists, the only implementation of D5 that is not a guess is **audit everything
the application serves** — and then D5's own scope is a question about coverage (RFC-0022 §D5a) rather
than about which reads were skipped.

The secondary reason is smaller and still real: an operational read is how a tenant's estate is
enumerated. *Which products does this business run, at which versions* is competitively meaningful
under BET-0005, where the platform operates products alongside the tenants it sells to.

**6. A CLI invocation records that it was a CLI.** The `actor` is the operating human or role, and the
operation name carries the command. A trail that cannot distinguish `docket issue` run from a laptop
from the same operation served over HTTP cannot answer the question RFC-0022 D5 exists for.

## Consequences

**Accepted costs**

1. **Table-level questions become operation-level questions.** "Was `docket_signatures` read" is no
   longer directly answerable; "was this document issued, and by whom" is. That is the trade, and it is
   accepted because the second question is the one anybody actually asks, and the first is answerable
   from the operation's definition.
2. **An operation's name becomes a contract.** Renaming `release.list` breaks a query someone wrote
   against the trail. Operation names therefore belong to the same discipline as an error `code`: a
   closed vocabulary per service, changed deliberately.
3. **A badly scoped operation hides work.** If someone writes an operation called `docket.sync` that
   touches everything, the trail says `docket.sync` and nothing more. Decision 3 limits the damage at
   silo boundaries and does not eliminate it.
4. **Write amplification is bounded but not zero, and decision 7 replaces this paragraph's hand-waving
   with a measurement.** One row per request, on a read path, is still a write on every read.

**Gained**

- **docket can write audit events**, which is the immediate point.
- **Two implementations agree.** `platform-tenancy`'s per-request rows already satisfy this, so the
  decision is a transcription of a precedent rather than a rule imposed on one repository.
- **The trail survives refactoring.** Merging two queries, adding a cache, or moving a read behind a
  port leaves the audit stream unchanged — because it records what was asked for, not how it was served.

**7. The stream's cost is a measured number, and the volume it lands on is sized for a database that
does not have one.**

Measured 2026-09-27 against PostgreSQL 16 with `audit_events` exactly as migration 0010 declares it,
its three indexes, and realistic values (a 32-character trace id, a Zitadel subject, an IPv4 source, a
full Chrome user agent):

| | |
| --- | --- |
| Total per row, table + all three indexes | **362 bytes** |
| 100,000 rows | 35 MB |
| The cluster's data volume today (`postgres/cluster.yaml`) | **4 GiB** |
| Rows that would fill it, with nothing else in the database | **~11.9 million** |
| Added latency per transaction, measured as the difference with and without the insert | **~2 ms** on a laptop through `psql`, which includes process startup and is therefore an upper bound |

**The latency is not the constraint. The volume is, and it is monotonic**: RFC-0045 §D item 4 keeps this
stream *longer* than the data it describes, so it is never pruned. At 100,000 requests a day the stream
alone reaches three million rows in a month.

So: **before any surface serves real traffic against this database, the volume is resized and the
stream's growth is measured rather than assumed.** That is an ordering, not a caveat — and it is the
honest form of accepted cost 4, which previously said "at the estate's current volume this is not a
concern" without knowing what the volume was. `ci`-style monitoring is not the answer here; a stated
figure and a resize are.

Partitioning by month is the next step after that and is deliberately not decided now: it is a
schema change with no requester, and the first real traffic will say whether it is needed.

## Alternatives considered

### 1. One event per table touched

The safest for an investigator, and rejected on two grounds. It records the implementation rather than
the request, so the trail changes when nothing a caller did changed. And it makes a CLI invocation cost
tens of appends to an append-only table, which turns a read-only command into a write-heavy one — for a
tool whose sealed chains are *already* the record of what it produced.

### 2. Audit only mutations

The industry default, and wrong here. ADR-0044 D6 asks for what was attempted, and a read of a person
record is precisely the access a data subject would ask about. RFC-0022 D5's promise — a tenant sees
operator access to its data — is mostly about reads, because reads are what an operator does most.

### 3. Leave it undecided and let each repository choose

What the estate has today, and it produced the state this ADR exists to end: one repository writing
per-request rows and another writing none, with no statement about which is right. Two implementations
that disagree about what one row *means* make the stream unqueryable across the estate — and an audit
trail that cannot be queried across services cannot answer a regulator's question, which spans them.

## Rollout

1. **Accept or reject.** Until then docket writes nothing, which is the status quo and is lossy.
2. **docket writes one event per command and per served request**, with the operation vocabulary in its
   own module the way its error codes are.
3. **The vocabulary is declared, not invented per call site** — a constant per operation, so the closed
   set is greppable.
4. **A check:** `db.audit_events_are_operation_shaped` — every distinct `action` in the stream resolves
   to a declared operation name. Declared with this ADR and unimplemented until there are rows to check.

## Open questions

1. **What is the operation for a saga that spans services?** ADR-0039's provisioning saga calls identity
   and deployment. One event per service with a shared correlation id is the obvious reading of decision
   4, and "obvious" is not the same as decided.
2. **Does a failed operation that touched two silos write two events or one?** Decision 3 says one per
   silo; a failure may not have reached the second. Recording an event for a silo nothing touched is a
   record of something that did not happen.
3. **How long is the vocabulary allowed to get?** A closed set per service is a discipline while it is
   twenty names and a formality at two hundred.

## Related

- **ADR-0044** — `accepted`; fixes what an audit record contains, and is silent on how many
- **RFC-0045 §D** — the stream, the grants, and the retention that outlives the data
- **RFC-0022 D5** — `proposed`; a tenant sees operator access to its data, which is mostly reads
- **GLO-0004** — the two boundaries decision 3 refuses to blur
- **ADR-0038** — the session context an event is written under
- **tool-docket `DECISION-DEBT.md` D-72** — the writer this unblocks
