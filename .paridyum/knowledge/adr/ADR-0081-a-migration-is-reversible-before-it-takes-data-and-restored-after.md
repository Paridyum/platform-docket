---
id: ADR-0081
title: A migration is reversible before it takes data, and restored after
type: adr
status: accepted
created: 2026-09-12
locked_at: null
supersedes: null
superseded_by: null
scope: [platform, service]
applies_to: ["platform-*", "svc-*"]
tags: [database, migrations, rollback, reversibility, req-0008, std-0006, adr-0071, adr-0024]
breaking: false
enforced_by: []
---

# ADR-0081 — A migration is reversible before it takes data, and restored after

## Status

**Accepted**, 2026-09-12. Closes REQ-0008 item 4 ("every migration has a tested rollback"). Not
`locked`: the round-trip test that proves a `down` is a runner command an operator invokes, not yet a
CI gate; it graduates when CI applies up→down→up on an ephemeral database for every migration.

## Context

REQ-0008 item 4 asks for a tested rollback, and its own text names the trap: "an untested
down-migration is a claim, not a control." Two honest positions fall out of that sentence and they
pull opposite ways.

- **Forward-only.** A down that drops or rewrites a populated table destroys rows. The only rollback
  that is safe once data exists is a restore from backup — which the estate already has, rehearsed,
  in ADR-0071 — followed by a forward fix. On this reading, down-migrations are the untested control
  item 4 warns against, and the requirement is satisfied by the restore.
- **Reversible.** Before a migration has taken any writes, its inverse is not destructive and *is*
  cheaply testable: reversing an additive migration on an empty schema returns the schema to where it
  was, provably, by round-tripping it. In that window a down is the fastest way to undo a bad deploy,
  and refusing to have one throws away a real, safe tool.

Both are right in their own window, and the window is the data. So the decision is neither
forward-only nor unconditionally reversible: it is **reversible until the first write, a restore
after** — the hybrid.

This suits where Paridyum is. The tables are empty behind the ADR-0071 gate, so today every migration
is in its reversible window, and the down is genuinely testable now. The migrations so far are all
additive (new tables, a default change), whose inverses are exact. The moment real applicant data
lands, the same down becomes destructive, and the rule flips to restore — automatically, enforced by
the runner rather than by an operator remembering which window they are in.

## Decision

1. **A migration file carries an `up` and, below a `-- migrate:down` line, a `down`.** The `up` is
   everything above the marker — the migration as it was already written — and the `down` is its
   inverse. A migration with no `down` marker is forward-only, and `status` says so.

2. **The checksum is taken over the `up` section only.** Adding or editing a `down` is not drift;
   editing an already-applied `up` still is (REQ-0008's immutability). This is what lets a `down` be
   added to a migration that is already applied — as all four existing ones were — without the runner
   refusing. The `up` section of a file whose `down` was appended is byte-identical to that file
   before the append, so no re-baseline is needed.

3. **`down` reverses the last-applied migration, and refuses if any table in `public` holds a row.**
   The runner counts rows across the tenanted schema before running a `down`; a single row aborts it
   with a message pointing at the restore path. This is the hybrid made mechanical: the operator does
   not decide which window they are in — the data does, and the refusal is the enforcement.

4. **Past the first write, recovery is ADR-0071's restore plus a forward fix, never a `down`.** A
   populated table is restored from the rehearsed off-host dump and then carried forward by a new
   migration. A destructive down against real rows is not offered, because a recovery tool that
   destroys data is the failure mode this whole decision is built around.

5. **A `down` is proven by round-tripping it.** Before a `down` is trusted it is exercised
   up→down→up on the empty database, and the schema after must equal the schema before. An untested
   down is exactly item 4's claim-not-a-control; the round-trip is what makes it a control.

## Consequences

- `scripts/migrate.py` gains `down`, up/down parsing, and the up-only checksum. `apply` runs only the
  `up`; `down` runs the `down` of the last-applied migration after the empty-table check, then removes
  its `_migrations` row.
- Migrations 0001–0004 each gained a `down`. 0004's round-trip was run live: `down` dropped the
  `uuidv7()` defaults and restored `resolve_account` to its pre-0004 form, `apply` put them back, and
  the schema matched at both ends. The guard was proven too: with one synthetic row present, `down`
  refused and the migration stayed applied.
- The `-- migrate:down` convention is this ADR's, not STD-0006's — STD-0006 is locked and fixes the
  *tenancy and class* directives, not the up/down split. If the syntax later earns a check it is
  ratified the way STD-0006 ratified its own directives: as a standard, once something enforces it.
- Forward-only remains legal for a migration whose inverse would be destructive even empty (a data
  backfill, a destructive transform); it declares no `down`, and `status` shows it as forward-only so
  the absence is visible rather than assumed.

## Related

- **REQ-0008** — item 4, the requirement this closes; the immutability rule the up-only checksum
  respects.
- **ADR-0071** — the rehearsed restore that is the recovery path past the first write.
- **STD-0006** — the migration standard whose directives this sits beside without amending.
- **ADR-0024** — the tenanted `public` schema whose emptiness the guard reads.
