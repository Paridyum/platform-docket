---
id: ADR-0069
title: The document vault is encrypted per person and shreddable; the register is neither
type: adr
status: accepted
created: 2026-09-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org, tool]
applies_to: ["tool-docket"]
tags: [archival, encryption, age, dpdp, erasure, personal-data, retention, sync, adr-0051, adr-0057, req-0001, tool-docket]
breaking: false
enforced_by: []
---

# ADR-0069 — The vault is shreddable, and the register is not

## Status

**Accepted**, 2026-09-07. Not `locked`: the layout is checkable and nothing checks it yet.

## Context

Documents that must be producible in 2036 currently live in one directory on one laptop.
Certificates carry `retention: permanent` because *"a certificate must stay verifiable for as
long as anyone might present it"*; an IP assignment has no end at all.

ADR-0051 already anticipated the destination, naming folders by the opaque person identifier
because *"this directory becomes a cloud folder, and a folder named after a person discloses who
Paridyum engages to anyone who can read the listing."*

**Two requirements are in direct opposition and both are real.**

*Erasure.* The DPDP Act gives a person the right to erasure, and this estate gitignores
`people/` rather than accept an unerasable record — *"an unerasable record makes the erasure
right unhonourable."* **Sync.com keeps file versions. Deleting a file there is not erasure**, and
claiming otherwise would be the same dishonesty in a different system.

*Recoverability.* An archive nobody can open is not an archive, and the obvious answer is an
escrow key that always decrypts.

**They cannot both hold for the same bytes.** A key that always opens a document makes that
document uneraseable. No such key makes loss permanent. Any design claiming both has quietly
chosen one.

## Decision

### 1. Two classes, split by whether erasure applies

| | Encrypted to | Erasable | Holds |
| --- | --- | --- | --- |
| **Register** | the estate key | **no** | serials, hashes, dates, identifiers, slugs, tombstones |
| **Vault** | a per-person key | **yes, by destroying that key** | issued PDFs, signed scans, the person record |

Not a new split: it is ADR-0051's, where *"identifiers remain"* and personal data goes. The
register is the identifiers; the vault is the data.

### 2. Shredding does not destroy verifiability, and this is what makes the split work

**`docket verify` compares a PRESENTED file against the hash recorded at issuance.** It does not
need the stored copy — that is a convenience, and `verify` already takes an optional path.

So a person whose vault has been shredded can still present their own certificate in 2036 and
have it verified against a surviving register entry. Erasure removes Paridyum's copy; it does not
remove their ability to prove what they were issued.

### 3. Keys are never uploaded, and shredding is deletion rather than revocation

Vault objects are `age`-encrypted to exactly one recipient: that person's key.

The keystore never goes to Sync and never enters git — both retain what you delete, so a key
inside one cannot be destroyed. It is protected at rest by the estate key per RUN-0004, and
**shredding means deleting the key material, including that encrypted copy.** The estate key
protects a file that later gets deleted; it is not a second way into the data.

`age` because ADR-0057 chose it, RUN-0004 documents the handling, and `tool-registry.yaml`
carries it at `tier: critical`. No new dependency.

### 4. Ciphertext and identifiers go to Sync. The index does not

Folders are named by person identifier — already implemented in `store::keep_pdf`. Filenames keep
the code and serial (`NDO-PAR-NDO-2026-0001.pdf.age`), disclosing a document type and not a
person.

**The generated index is never uploaded.** Encrypting documents and syncing a roster that maps
identifier to name defeats the opaque naming entirely, and it is the easy mistake because the
index is the useful part.

Sync's own end-to-end encryption is a second layer, not the control being relied on.

### 5. Erasure writes a tombstone, and a tombstone is not personal data

Person identifier, date, operator, and counts of what was destroyed. No name, no address, no
titles beyond codes.

A tombstone is how the archive says *"erased lawfully on this date"* rather than *"appears to be
missing"* — which matters most to whoever reads it when nobody involved is still here.

### 6. Key durability is a trade the operator makes, not a problem the design solves

**Crypto-shredding needs key material that can genuinely be destroyed. Any copy of that material
improves durability by exactly as much as it damages the erasure promise.** There is no scheme
that gets both, and one claiming to has quietly picked a side.

So the choice is a command rather than a default — `docket vault escrow <path>`:

| | Erasure | A lost laptop |
| --- | --- | --- |
| **No escrow** | **provable** — docket held the only key and destroyed it | loses every vault |
| **Escrow** | **asserted** — docket destroyed its copy and cannot reach the shelf | recoverable |

The same distinction ADR-0067 draws between severing an alias, which Paridyum controls
outright, and recovering a consumer mailbox, which is a request. Provable and asserted are
different facts and the record must not report the second as the first.

**The escrow may not be written inside the store**, and `escrow` refuses to. The store is what
gets uploaded; every per-person key in a versioned cloud folder would mean no erasure this system
performs is real, and it is the one mistake that would be entirely silent.

**After any shredding, an escrow taken earlier still holds the destroyed key.** `docket check`
warns about exactly that, because nothing else in the system can see it: the erasure is not
complete until that medium is destroyed or re-exported.

### 7. What is irreplaceable is the signed scan, not the issued document

Worth stating because it changes what the archive is for. An issued PDF was handed to the person
in their pack, so losing Paridyum's copy is recoverable **from the counterparty**. A returned
scan is the opposite: they signed it and gave it back, and `sign.rs` says why it matters —
*"wet ink leaves no trail of its own, so the scan IS the evidence."*

So scans are sealed alongside issued documents, and erased alongside them. Deleting the PDFs
while leaving the scans would erase the copies the person already holds and keep the one they do
not, which is the inverse of what erasure is for.

## Consequences

**Accepted costs.**

- **Losing a person's key destroys their documents permanently, unless an escrow exists** — and
  an escrow makes erasure asserted rather than provable (D6). The trade cannot be avoided, only
  chosen; what this ADR provides is that choosing it is deliberate and that the weaker guarantee
  is recorded rather than assumed away.
- **An escrow is offline media and therefore manual**, and manual steps rot. `docket check`
  reminds after a shredding and can see nothing else about it.
- **A shredded person cannot be re-issued their documents.** Paridyum can prove what was issued
  and can no longer produce it.
- **A keystore that must not leak and must not be lost**, two requirements pulling apart.
  REQ-0001 is already open debt and this makes it heavier.
- **Restore is not a copy-back** — fetch, decrypt per person, verify against the register. It has
  to be rehearsed or it does not work.
- **The register still carries slugs**, which are name-derived and survive erasure. ADR-0051
  accepted this already, but it means erasure is not total and the tombstone must not imply it is.
- **Nothing enforces the layout yet.**

**Gained.**

- An erasure request has an answer that is true, in a store where deletion is not.
- A certificate stays verifiable after its holder is erased.
- One encryption tool, already adopted, registered and documented.
- A cloud listing discloses document types and opaque identifiers, and no names.

**Rejected alternatives.**

- **One escrowed key for everything.** Simplest and recoverable. Rejected: it makes erasure
  impossible while leaving the claim available, which is what gitignoring `people/` exists to
  avoid.
- **Encrypt to both a per-person key and an estate key.** Recoverable *and* granular, and it was
  tempting. Rejected as escrow in disguise — destroying the personal key shreds nothing while the
  estate key still opens the file.
- **Sync's end-to-end encryption alone.** Real and good. Rejected as the only control: it defends
  against Sync reading the data, not against a compromised Sync account, which is likelier — and
  it offers no per-person granularity, so erasure again has no mechanism.
- **Delete files for erasure, no encryption.** What most people do. Rejected: Sync keeps versions,
  so the delete is a gesture.
- **Laptop only.** No cloud exposure. Rejected: one disk is not an archive against a thirty-year
  horizon.
