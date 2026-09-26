---
id: ADR-0067
title: An address is derived from the person identifier so it cannot collide, on the root domain
type: adr
status: accepted
created: 2026-09-06
locked_at: null
supersedes: null
superseded_by: null
scope: [org, tool]
applies_to: ["*"]
tags: [naming, identifiers, email, accounts, offboarding, dpdp, personal-data, adr-0051, adr-0045, c-0003, c-0007, tool-docket]
breaking: false
enforced_by: []
---

# ADR-0067 — An address cannot collide, because it is derived rather than allocated

## Status

**Accepted**, 2026-09-06. Not `locked`: the rule is checkable in principle — an address is a
string in a person record and its shape can be asserted — but nothing checks it today, and
`locked` without `enforced_by` is the false confidence ADR-0002 exists to prevent. It graduates
when `docket check` validates `access.company_email`.

## Context

Four officers need addresses, interns follow, and nothing in this estate says what an address
looks like. **C-0007 governs repositories, crates, API paths, database tables, environment
variables, images and labels — and not mailboxes.** Three function addresses exist by custom
rather than by decision: `legal@`, `hello@` and `security@paridyum.com`.

The shape proposed was `vivaswan.pdol001.paridyum@gmail.com` — a given name, a code, the
organisation — with a matching address on `paridyum.com`, possibly under an `intern.` subdomain.

Two forces met, and they pull in opposite directions.

**A mailbox in a global namespace must be unique.** Gmail's namespace is every Gmail user on
earth, so `vivaswan@gmail.com` is long gone. Something must disambiguate, and whatever
disambiguates has to be unique across Paridyum forever, because **an address that is reused
delivers one person's mail — including their password resets — to another.** That is a worse
failure than the folder-name collision ADR-0051 was written about, and it is the same failure.

**An address is the most-copied string a person owns.** It is in other people's address books,
in git commit authorship — which C-0003 requires be real and attributable — on signed documents,
and in university records. It outlives the arrangement that produced it.

Two operator decisions were taken and are recorded as the constraints this is designed against:

1. **`paridyum.com`, and anti-collision by default** — an address that cannot clash rather than
   one that clashes rarely.
2. **Free consumer Gmail is the long-term mailbox**, not a bootstrap for Google Workspace.

The second is the expensive one and §"Accepted costs" says why.

## Decision

### 1. Every address is `firstname.<person id>`, on the root domain

    rohan.p000147c7@paridyum.com

**Derived, never allocated, and therefore incapable of colliding.** The person identifier is
unique across Paridyum forever and is never reissued (ADR-0051), so both properties an address
must have are inherited rather than enforced.

**This is what removes the address register.** A `firstname.lastname` scheme cannot hand out the
next address without knowing every address ever issued — including to people who have left, and
to people whose records were erased, because reuse delivers one person's password resets to
another. That register would have been a new permanent, append-only, never-pruned structure
whose only job was preventing a collision this scheme cannot have.

It also makes an address **self-validating**. A mistyped `rohan.batra@` goes nowhere silently; a
mistyped identifier fails the MOD 11-2 check character, and `docket` can say so without a
lookup. The check character ADR-0051 added for folder names now earns its keep on every address.

**Never under a class subdomain.** `intern.paridyum.com` was proposed and is rejected, because
it is ADR-0051's locked mistake in a worse place. That ADR rejected `PAR-INT-2026-0001` as a
person identifier on exactly this reasoning:

> The predictable case is the one that breaks it: an intern is hired. Either the folder stays
> labelled `INT` forever and lies, or the person gets a second identifier and the archive splits
> in two.

Class is mutable for a person. An address encoding it either becomes false or forces a rename,
and a rename is not available — see D3. A folder can at least be renamed by one person; an
address cannot, because most of the copies are in other people's hands.

The honest argument for a subdomain is **sending reputation**: mail from a subdomain that gets
flagged does not damage the root domain's deliverability. It is a real concern with a standard
answer that does not require putting a mutable fact in a person's identity — isolate *bulk and
automated* sending on its own subdomain, which is about what sends, not about who the sender is.

### 2. The mailbox behind it is named from the opaque person identifier

    vivaswan.p000147c7.paridyum@gmail.com
    │        │         │
    │        │         └── organisation, so the account is recognisable in a listing
    │        └──────────── ADR-0051's person identifier, lowercased, punctuation dropped
    └───────────────────── given name, so a human can find the account

The middle segment is `PAR-P-000147-7` rendered for a namespace that allows no hyphens. It is
**already guaranteed unique and never reissued**, it carries an ISO 7064 MOD 11-2 check
character against transcription error, and it encodes no fact that can go stale. Minting a
second sequence beside it — the `pdol001` in the original proposal — was rejected: two numbering
systems for one person means two things that must never be reused, and ADR-0051 names reuse as
"the one failure that silently reattaches one person's documents to another."

The given name stays, against the opacity ADR-0051 requires of a *folder* name, and the
difference is deliberate. That ADR's opacity argument rests on a directory listing disclosing
who Paridyum engages to anyone who can read it. A Gmail account name is not a listing anybody
browses; what it has to be is findable by the operator in an account switcher and a password
manager, at the moment they are trying to reach the right mailbox.

### 3. An address is never reused, and never renamed once issued

Not after separation. Not for a person with the same name. Not to correct a misspelling in one
already in use.

Reuse delivers the previous holder's mail to the new one, and the mail that matters is the
password reset. **Renaming is the same failure seen from the other side**: every copy held by
somebody else keeps pointing at an address that now belongs to nobody, or worse, to someone
else later.

Under decision 1 both rules hold **by construction rather than by discipline**, which is the
main thing that decision buys. There is no cascade to get wrong, no suffix to allocate, and no
second person whose address is worse because they joined later.

### 4. `paridyum.com` is the identity; the Gmail address is the mailbox

The Gmail address is where mail physically lands. **The `paridyum.com` address is what a person
gives out**, what appears on a document, what is published on the officers page, and what a
commit is authored from.

This is the one lever that survives decision 5's cost. A consumer Gmail account cannot be
suspended by Paridyum, but **the alias can be severed by Paridyum alone** — repointing or
retiring a person's `paridyum.com` address is an act fully within Paridyum's control, and it
ends the person's ability to act as Paridyum even where the mailbox behind it does not.

### 5. Both addresses are recorded, and offboarding says which of them it actually proved

`access.company_email` holds the published address. A second field holds the mailbox. Both are
recorded because both must be dealt with at offboarding, and the field exists — in its own
words — "so offboarding can prove it was removed."

**With a consumer account that proof is not available, and the record must not imply otherwise.**
Severing the alias is provable. Recovering the mailbox is a request, and the departing person
holds the recovery phone and address. Offboarding therefore records two different facts, and
does not report the second as the first.

## Consequences

**Accepted costs.**

- **Offboarding cannot prove the mailbox was removed**, only that the alias was severed. This
  weakens something `tool-docket` was specifically built to do, and it is a direct consequence of
  decision 5 above rather than an oversight. It is the reason §5 splits the record in two.
- **Confidential material will sit in accounts Paridyum does not administer.** No admin console,
  no enforced 2FA, no audit, no legal hold, no recovery. This touches the NDA every person signs
  and the DPDP position on data Paridyum is responsible for but cannot reach. It is an open risk,
  not a solved one.
- **A free account can be disabled by Google** for inactivity or a policy judgement, with no
  administrator to appeal through. The mailbox and its history are then gone.
- **Sending as `@paridyum.com` from a consumer account** needs an SMTP relay with SPF and DKIM
  configured per account, and deliverability will still be worse than a domain mailbox.
- **A misspelling in an issued address is permanent.** D3 has no escape hatch by design; the
  mitigation is that an address is issued from a record whose legal name was checked against a
  government ID.
- **An address is not guessable, and that is the real price.** A university, a candidate or a
  customer cannot reach somebody without being told the exact string. What makes this survivable
  is that the function addresses already exist — `hello@`, `legal@` and `security@` — and they
  are what inbound strangers actually use. A personal address is given out, not deduced.
- **It is harder to read aloud than a surname.** `rohan.p000147c7` over a phone is worse than
  `rohan.batra`, and no amount of design fixes that. The mitigation is that an error is
  *detectable* — the check character fails — rather than a silent bounce.
- **The person identifier is now published widely**, so headcount and joining order can be read
  off a sequence. It discloses nothing about the person, which is what ADR-0051 made it opaque
  for, but it is more visible than that ADR anticipated.
- **Two addresses per person to keep in step**, and a person who replies from the wrong one
  leaks the mailbox address into correspondence that should only ever have seen the alias.

**Gained.**

- An intern who becomes an employee keeps their address. Nothing in it goes stale.
- **No address can collide or be reused, by construction** — so no register of issued addresses
  is needed, and nobody's address depends on who joined first.
- A wrong address is detectable from the string alone, without a lookup.
- The unique part of a mailbox name is minted by something that already guarantees uniqueness
  and already refuses to reissue, rather than by a second counter kept by hand.
- Paridyum can end someone's ability to act as Paridyum without needing access to their mailbox.
- The scheme survives a later move to Workspace or to ADR-0045's identity provider unchanged: the
  published address is already the identity, and only what sits behind it would change.

**Rejected alternatives.**

- **`intern.paridyum.com` for interns.** Isolates sending reputation and signals the arrangement
  honestly. Rejected under ADR-0051: it encodes a class, class is mutable for a person, and the
  first intern who is hired forces either a lie or a rename. Reputation isolation belongs to a
  sending subdomain, which is about what sends rather than who sends it.
- **`firstname.lastname@paridyum.com`, with a fallback on collision.** The first design here,
  and it is what most companies use: reachable, readable, familiar. Rejected because every
  collision-handling scheme needs a register of every address ever issued in order to detect the
  collision — including for people erased years earlier — and because the second person with a
  given name gets a worse address for no reason they control. Deriving from an identifier that
  is already unique deletes both problems rather than managing them.
- **`firstname@paridyum.com`.** Shorter, and fine at four people. Rejected because the first
  duplicate given name forces an exception or a rename, and D3 forbids the rename — so the
  scheme would break on an event that is certain rather than unlikely.
- **`rbatra@paridyum.com`.** Compact, and collision-resistant only until it is not. Rejected
  for the same reason as the two above: it manages collisions rather than preventing them, and
  it is no more guessable than the identifier form while offering none of its guarantees.
- **A separate `pdol001`-style sequence.** What was proposed. Rejected: a second permanent
  never-reused counter beside one that already exists, kept by hand, for a person who already has
  an identifier. The reuse it invites is the failure ADR-0051 singles out.
- **Docket's person identifier alone as the mailbox name**, with no given name. Maximally opaque
  and consistent with ADR-0051's folder rule. Rejected because an operator holding twelve accounts
  needs to find the right one in a switcher, and ADR-0051's opacity is argued from a listing that
  discloses engagement — which a Gmail account name is not.
- **Google Workspace.** Solves every accepted cost above: admin control, provable removal, audit,
  retention, domain mailboxes without a relay. Not chosen — the operator's decision is free Gmail
  long-term, and the costs are recorded here rather than argued again.
