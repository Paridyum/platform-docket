# Agent entrypoint — platform-docket

Issues Paridyum's internship and employment documents, keeps the personnel
records they are issued against, and archives both.

**This file is a router, not a manual** (ADR-0026). The rules that bind you are
in `.paridyum/knowledge/`, which is a read-only vendored snapshot — never edit
it. Everything below is either specific to this repository or destructive if
you get it wrong.

## Read these before changing anything here

| If you are about to | Read |
|---|---|
| add or change a document | `registry/lifecycle.yaml`, then `templates/` |
| add a field to a person record | `registry/fields.yaml` — a field with no stated purpose must not exist |
| change what a class may collect | `registry/person-classes.yaml` and ADR-0051 |
| touch identifiers | **ADR-0051 is locked.** Person identifiers are opaque; engagement identifiers carry class and year |
| change layout | `latex/paridyum.cls` — the only place layout lives |
| change how anything is operated | `SOP.md` |

## The rules this repository adds

**Nothing in `templates/` or `latex/` may restate a fact the registry owns.**
The office address, the officers' names, the email — one home each, in
`registry/`. `docket check` fails the build if a template hard-codes one.

**Personal data never enters git.** `people/` and `store/` are gitignored and
must stay that way. Git history cannot be erased, which would make DPDP erasure
rights unhonourable — and `docket erase` has to be able to actually delete.

**The register, the identity ledger, the event log and the signature ledger are
append-only and hash-chained.** Do not add a code path that edits an existing
entry. The one exception is `docket erase`, which rewrites the event log to its
skeletal events and re-seals the chain, because an append-only log that cannot
forget cannot honour an erasure request.

**A signature is a separate record, never a field on the issuance entry.**
Recording it by mutation would invalidate that entry's digest and every digest
after it.

**Generated output is regenerated, never hand-edited** (C-0004). That is
`store/index/`, `store/manifest-sha256.txt`, and the `dkdata.tex` written during
a render.

## Before you claim it works

    scripts/gates.sh

Runs fmt, clippy, the test suite and `docket check`, and reports each exit code
separately. Use it rather than the four commands by hand.

**The reason is not convenience.** Running them by hand is how a failing gate gets
recorded as passing: `cargo fmt --check 2>&1 | head -20` followed by reading `$?`
returns the exit code of `head`, so a visible diff reads as clean. That happened
twice in one day, both times by someone who had written the rule against it down.
Nothing in `gates.sh` is piped, so there is no exit code to lose.

`docket check` is the one that catches registry and record problems; the Rust
gate alone will not. It exits 0 on warnings, so `gates.sh` reports the warning
count separately — a registry that has started warning is worth looking at even
though nothing is broken yet.

## What this repository does not do

It does not sign anything, grant access, delete on a retention deadline, reach
a cloud folder, or make a document legally sufficient. `SOP.md` states each
boundary. Do not add a feature that implies otherwise.
