---
id: ADR-0072
title: Exactly one node writes the docket store, and every other node is a cold spare
type: adr
status: superseded
created: 2026-09-07
locked_at: null
supersedes: null
superseded_by: ADR-0093
scope: [org, tool]
applies_to: ["tool-docket"]
tags: [availability, durability, concurrency, locking, register, hash-chain, adr-0051, adr-0057, adr-0069, adr-0071, rfc-0021, tool-docket]
breaking: false
enforced_by:
  - check: docket.identifiers_are_never_reused
    severity: error
    rationale: >
      Declared 2026-08-19 against ADR-0051 and never implemented. The reproduction in the
      context below shows a duplicated person identifier passing `docket check` in silence,
      so the registry has been asserting a property nothing verified. Decision 3 implements
      the check that already existed on paper.
  - check: docket.store_has_one_writer
    severity: error
    rationale: >
      The invariant the HTTP server's serial accept loop was already relying on, moved from
      one adapter to the store itself so that a CLI write cannot bypass it.
---

# ADR-0072 — One node writes, the others are cold spares

## Status

**Accepted**, 2026-09-07. Not `locked`: two of the five decisions below are newly enforced in
code and have not yet survived a real incident, and decision 4 is a procedure rather than a
property.

## Context

The operator asked how to run docket on more than one machine so that losing a laptop does not
lose the estate. Measuring the answer first produced a different question.

**The key was never the exposure.** RUN-0004 already puts the one age private key in a password
manager and installs it per machine, so a lost laptop loses no key. What lives on exactly one
laptop and nowhere else is the **store**: the four append-only hash chains, the issued documents,
the signed scans, and the per-person vault keys. `docket vault backup` produces an encrypted
register bundle; nothing said where it goes, and no restore had been rehearsed.

**Multi-writer is not hard, it is wrong.** A person identifier is allocated from the local
ledger — `PersonId::new(identity::next_person_seq(&ledger))`. Two machines holding the same
ledger allocate the same identifier to two different human beings, which breaks **locked
ADR-0051** at its root, and both identifiers carry a valid check character. Making identifiers
collision-free means making them random rather than sequential, and ADR-0051 is locked on
sequential. So the sequence cannot absorb concurrency; the concurrency has to go.

**The hazard is not hypothetical and it is not remote.** It exists today, on one machine. The
HTTP server is deliberately serial — *"this process is the only writer to four append-only hash
chains, and that is what makes them safe without file locking"* — and binding is loopback-only,
so many operators reach one process over `ssh -L`. Docket is therefore already **multi-user by
being single-node**. But `append_chain` is read → seal-onto-last → append with no lock, and
nothing stops a CLI write while the server holds the store. Ten concurrent `docket new` runs
against one store, five pairs:

| Outcome | Result |
| --- | --- |
| Duplicate person identifiers | **5 of 5 pairs** |
| Forked chain links (two entries claiming one parent) | 4 of 5 pairs |
| Duplicates `docket check` reported | **0** |

The window is not narrow; it is the common case. And the last row is the finding that matters:
`check` reported the *forks*, because `verify_chain` looks at links. One pair appended cleanly —
sequential writes where both had already read a stale ledger — producing **the same identifier
twice with a correct chain, which `docket check` passed in silence.** Detection of a duplicated
identity is currently a side effect of a torn chain rather than an assertion about identity.

## Decision

### 1. Exactly one node is authoritative; the rest are cold spares

A **cold spare** holds a restorable copy of the store and does not run docket against it. It is
redundancy of *bytes*, not of *operation*. Promotion is deliberate (decision 4).

This is not a compromise reached for lack of a better design — it is the only shape consistent
with what is already built. The serial accept loop, the loopback bind and the SSH-forward access
model all assume one writing process, and they were each argued for on their own merits before
this question was asked. A second writing node would invalidate all three.

### 2. A command that writes takes an exclusive lock on the store

The lock is a file created with `O_EXCL` semantics (`create_new`), holding the hostname, process
id, operator and time of the holder. It is taken by every writing command — CLI and server alike —
and released on exit.

**`O_EXCL` creation rather than `flock`**, because STD-0003 sets `unsafe_code = "forbid"` and the
crate has no libc dependency; atomic exclusive create is in `std`, needs no unsafe and no new
dependency, and works identically on macOS and Linux. The accepted cost is stated in the
consequences: it does not survive a crash cleanly, so a stale lock needs clearing by hand.

**The lock names its holder** so that the refusal is legible. `another docket is writing` is an
error someone will delete the lock file to escape; `held by tabla-01 pid 4192, rohan, 14:02` is
one they can act on.

### 3. The ledger's identifiers are asserted unique, not inferred from the chain

`docket check` compares every identifier in the ledger against every other and errors on a
repeat. This is **belt and braces on purpose**: the lock prevents the duplicate, and the check
finds it if the lock was ever bypassed, stale-cleared wrongly, or two stores were merged by hand.

The reproduction above is the whole argument. A property this important must be *asserted*, not
inherited from the failure mode that happened to accompany it.

### 4. A spare is promoted by a human, and there is no election

Promotion is: confirm the old node is not writing, restore the store, move the authority marker,
and **repoint the operators' `ssh -L` forwards.** Because remote users arrive over a forward
rather than through installed software, the spare needs no user provisioning at all — which makes
promotion far cheaper than it would be for a conventional multi-node service.

Automatic failover is refused. Under C-0004 this is an irreversible, governance-altering
operation; and there is nothing to elect *with* — no quorum, no second writer, and a split brain
here mints duplicate human identities.

### 5. What replicates, and the one thing that does not

| Layer | Erasable? | Replication |
| --- | --- | --- |
| Estate age key | never | password manager, every node (RUN-0004) |
| Register chains, events, signatures, tombstones | **never**, by ADR-0069 | replicate freely; a versioned copy is safe and wanted |
| Vault documents and signed scans | shreddable | replicate as ciphertext |
| `store/keys/` per-person keys | the thing that makes erasure work | **never to a versioned store** |

Replication is safe at all only because ADR-0069 already made erasure **key-destruction rather
than file-deletion**. A synced copy of ciphertext does not un-erase anybody. A synced copy of a
per-person key does, which is why the last row has no cell to fill.

That last row is the same wall ADR-0069 D6 hit, one level down: a key you can restore is a key
you cannot prove you destroyed. **Multi-node sharpens that trade; it does not soften it**, and
this decision does not pretend to close it.

### 6. A restore is rehearsed on the spare before real data exists

ADR-0071 §3 already states the rule and it is adopted here rather than restated: *"A backup that
has never been restored is a belief, not a control."* The rehearsal must run on a *different*
machine, from the register bundle and the estate key alone, because a restore proven only on the
machine that made the backup proves the cryptography and not the recovery.

## Consequences

**Accepted costs**

- **No concurrent operation, by design.** Two operators cannot each run a writing docket. They
  share one over SSH forwards, which is what the server was built for.
- **A stale lock needs a human.** If docket is killed, the next write refuses until the lock is
  cleared. The lock names its holder so that decision is informed; an automatic timeout was
  rejected because a timeout that is wrong once mints a duplicate identity.
- **Promotion is manual, so there is a window.** Between a laptop loss and a promotion, nothing
  can be issued. Acceptable: docket issues documents on human timescales.
- **The spare's copy is as fresh as the last backup**, not as fresh as the last write. The
  recovery-point objective is the backup cadence, and it is the operator's to set.
- **Per-person keys remain the unbacked half.** Unchanged by this decision, and named again here
  so that the cold spare is not mistaken for having solved it.

**Gained**

- The failure the operator asked about — a lost laptop — stops losing the register, the issued
  documents and the signed scans.
- A concurrency defect that reproduces on the first try, on the machine in use today, is closed
  before the first real document exists.
- A duplicated human identity becomes a detectable condition rather than a silent one.
- The single-writer invariant stops being a property of one adapter's accept loop and becomes a
  property of the store, which is where it was always being relied upon.
