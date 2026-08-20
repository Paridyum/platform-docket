---
name: decompose-spec
description: Decompose a specification, SRS, or large feature request into requirements, entities and bounded contexts BEFORE any code or service is designed. Use when handed a document describing many things a system must do, or when a request implies more than one service. Argument is the spec or a path to it.
---

# Decompose a specification

A large specification is not a task. It is a **research input**. The failure this exists
to prevent is not bad code — it is a plausible implementation of the wrong decomposition,
produced faster than anyone can review it, after which the code has momentum and rejecting
it costs more than accepting it.

**Do not write code during this skill. Do not create a repository, a crate, or a
`manifest.yaml`.** The output is understanding and artifacts, and the next step after it is
a human decision.

Input: `$ARGUMENTS`

---

## 0. Check what already exists — first, always

Most of what a spec asks for is often already decided here. Building it again is the most
expensive possible outcome, because you get two answers to one question.

```sh
ls adr/ constitution/ standards/ requirements/ anti-patterns/ rfc/
grep -ril "<the domain noun>" adr/ constitution/ standards/ requirements/
scripts/check-knowledge.sh          # what is actually enforced today
```

Read `anti-patterns/` in full before proposing any boundary. Six shapes are forbidden and
three of them are about exactly the decomposition you are being asked to make.

State plainly what already covers part of the spec. Do not restate it as new.

---

## 1. Extract atomic requirements

One sentence, one requirement. Split every "and".

> Bad: *Users can upload invoices and receive notifications.*
> Good: *Users can upload invoices.* / *An uploaded invoice notifies the uploader.*

Keep them **technology-neutral**. "The system shall support uploads larger than 20GB" is a
requirement; "implement multipart upload using S3" is already a design and has smuggled a
decision past review.

Do not invent requirements the spec does not state. A gap is a question for the author, and
listing it as an open question is more useful than filling it in.

---

## 2. Discover entities and actors

List every noun that has identity and a lifecycle, and every actor that acts on one.

Then ask the question that does most of the work: **are two of these nouns actually the
same thing?** `User`, `Customer`, `Employee` and `Vendor` are usually four entities in four
contexts, not one `User` with four roles. Collapsing them early is how a god service starts
(ANTI-0006).

For each entity, one line: what it is, who creates it, who changes it, who reads it.

---

## 3. Group into candidate bounded contexts

Cluster requirements by the entity they change, not by the layer they live in. A context is
a set of requirements that change the same entities for the same reason.

For each candidate context state:

- **Owns** — the entities it is sole writer of
- **Publishes** — events others may react to, `<Subject><PastTenseVerb>` per C-0002
- **Consumes** — events and APIs it depends on
- **One sentence of responsibility, with no "and"** — if you need "and", it is two contexts
  (STD-0004's test, applied a level up)

Under ADR-0001 each context is one repository, so this step is deciding how many
repositories exist. Treat it accordingly.

---

## 4. Assign single ownership

Every entity gets **exactly one** owning context. Write the matrix; do not leave it
implicit.

| Entity | Owner | Everyone else |
| --- | --- | --- |
| `invoices` | billing | reads via API or projection |

Two owners is ANTI-0001 and is the one boundary mistake that is both mechanically
detectable and effectively permanent once shipped. If two contexts both seem to own
something, one of these is true and you must say which:

- one genuinely owns it and the other needs a read contract
- they own **different** things that share a name — split the name
- there is a **missing third context** that owns it and both should call

---

## 5. Turn every "X does Y to Z" into an event where it can be

The naive reading of *"Invoice sends emails"* is a `send_email()` in the invoice service.
That is wrong: it makes invoicing depend on notification, and the second subscriber will
require changing invoicing.

```
Invoice ──publishes──▶ InvoiceApproved ──▶ Notification (subscribes, sends email)
```

Ask of every cross-context interaction: **does the caller need an answer to finish its own
job?** If no, it is an event. Most are events.

---

## 6. Check the graph before proposing it

- **No synchronous cycles.** ANTI-0002. Event loops are fine and are the fix.
- **No shared writes.** ANTI-0001.
- **No direct reads of another context's tables.** ANTI-0003.
- **No shared library holding a domain rule.** ANTI-0004. Duplication is often cheaper.
- **Every cross-context write sequence has a named compensating path.** ANTI-0005.
- **No context whose responsibility needs an "and".** ANTI-0006.

If the decomposition violates one of these, it is wrong — change the decomposition, not the
rule.

---

## 7. Write the artifacts

**One `REQ-` per bounded context**, not one per requirement, with the atomic requirements
numbered inside it so they can be cited as `REQ-NNNN §14`.
(Written `NNNN` deliberately: a concrete id in an example is indistinguishable from a
real citation, and `repo.no_dangling_decision_references` fails on it.)

A hundred requirements as a hundred files means a hundred statuses nobody maintains; within
months most are stale, and a status you cannot trust poisons the ones you can. One file per
context keeps the addressable reference and drops the part that rots. See ADR-0018.

Set `status: deferred` with a **trigger** for anything real but not being built now. A
deferred requirement with no trigger is a `TODO` with frontmatter and the gate rejects it.

Then, and only then:

- An **RFC** proposing the decomposition, with the rejected alternatives — including the
  boundaries you considered and dropped, which is the section future-you will read.
- **ADRs** for decisions the RFC settles, once a human accepts it (ADR-0013 stage 3).

---

## 8. Report, then stop

Produce a summary the human can act on:

```
Found        N requirements, N entities, N actors, N external systems
Contexts     N candidates, one repository each under ADR-0001
Already      <what the knowledge base already decides — do not rebuild>
Conflicts    <anything violating an ANTI->
Unknowns     <what the spec does not say>
```

**Then stop and ask.** Do not proceed to implementation. The decomposition is the decision;
the code is a consequence of it, and ADR-0013 requires a human between the two.

If the answer is "just build it", the honest reply is that the wrong boundary costs more
than the delay — and that the delay is hours while the boundary is years.
