# Running docket — which machine, and how

`README.md` says what docket does. `HIRING-PLAYBOOK.md` and `HR-CHECKLIST.md` say
what to type. **This says where to type it**, which is the question neither
answers and the one that goes wrong first.

---

## 1 · Which machine

**Exactly one machine is authoritative.** Not one at a time by convention — one,
chosen deliberately, and the others hold restorable copies they do not run
against. That is ADR-0072, and it is not a limitation to route around:

- Writes take an exclusive lock on the store. A second docket refuses, by design.
- The API binds loopback and nothing else.
- The register is a hash chain. Two writers can fork it, and a forked chain
  reads as tampering later.

**A cold spare is redundancy of bytes, not of operation.** Restoring a copy onto
a second laptop and working there is how you get two chains. Promotion to
authoritative is a deliberate act — see ADR-0072 decision 4, not this file.

If you are not sure which machine is authoritative, **stop and find out.** It is
not a question to answer by trying.

## 2 · What must be installed

| | |
|-|-|
| Rust | 1.97 or later (`rust-toolchain.toml` pins it) |
| TeX Live | with `pdflatex` on `PATH` — this is not optional; it is how documents exist |
| Node + pnpm | only if you want the browser console (§5) |

Check the one that actually breaks:

```sh
pdflatex --version
```

No `pdflatex`, no PDFs. Every issue command will fail, and it will fail at the
end of the work rather than the start.

## 3 · Where the data lives

```
people/     person records          GITIGNORED
store/      register + issued PDFs  GITIGNORED
registry/   the facts               committed
```

**Both data directories are gitignored on purpose.** They are personal data;
they do not belong in a repository, and a `git push` is not a backup.

Override either with an environment variable, which is how you point at a store
that lives outside the checkout:

```sh
export DOCKET_PEOPLE=/path/to/people
export DOCKET_STORE=/path/to/store
```

Set these in your shell profile on the authoritative machine, or you will run a
command against the repository's empty defaults and wonder where everyone went.

## 4 · Build, and the first command

```sh
cargo build --release
./target/release/docket check
```

`docket check` is always the first command. It reads the registry and every
existing record, and exits 0 only when both are sound. **If it is not 0, fix
that before anything else** — every other command builds on what it validates.

Then, for anyone in flight:

```sh
docket next <slug>
```

Run it, do what it prints, run it again. Exit 0 means nothing is owed.

## 5 · The browser console

`console-docket` is the HR-facing interface. It does not read the store; it
talks to a local docket over loopback.

**On the authoritative machine:**

```sh
./scripts/serve.sh
```

**Use the script. Do not run `docket serve` by hand.** The script builds first,
every time, and refuses an occupied port. Both behaviours exist because of real
faults: a stale binary served last week's wire contract to a console that had
moved on, and the operator saw a redacted React error instead of a cause; and a
leftover server on 8080 answered probes for a build that had not started.

**Then, in `console-docket`:**

```sh
DOCKET_API=http://127.0.0.1:8080 pnpm install
DOCKET_API=http://127.0.0.1:8080 pnpm run build
DOCKET_API=http://127.0.0.1:8080 pnpm run start --hostname 127.0.0.1
```

Pick your operator at `/who` before writing anything. Every write is recorded
against it and nothing is written without one.

## 6 · Working from somewhere else

**The API has no login, and that is deliberate.** It binds `127.0.0.1` and
refuses any other address (ADR-0068 decision 1). There is no configuration flag
to expose it, because a boundary with one hole in it is a boundary someone has
to remember the shape of.

Remote access is SSH port forwarding, and only that:

```sh
ssh -L 8080:127.0.0.1:8080 <authoritative-host>
```

Then point the console at `http://127.0.0.1:8080` as usual. **SSH is the
authentication.** Do not put a reverse proxy in front of it, do not add a
tunnel, and do not open the port "just for the form endpoint".

## 7 · When it refuses

**`another docket is writing`** — with a hostname, pid, operator and time. That
is the lock working. Find the named process and let it finish.

If the holder is genuinely dead (a crash, a closed laptop), the lock file is
stale and needs clearing **by hand**. It is `O_EXCL`-created and does not survive
a crash cleanly; that cost is recorded in ADR-0072's consequences rather than
hidden. Clear it only once you are certain no docket is running — that certainty
is the whole safety property.

**A stage gate refuses** — `docket advance` will not move someone whose
agreements are not signed. That is not a bug and `--force` is not the normal
answer. Read the reasons it printed.

## 8 · Backups

The store is personal data. Off-host copies are **age-encrypted before they
leave the machine** and land as `*.age` (ADR-0071); the drive never holds
plaintext. Issued documents are the one exception, because a certificate is
meant to be shared and independently verified.

A file in a sensitive tree without `.age` is a plaintext leak. A file that will
not decrypt with an operator key is unrecoverable, which is worse.

---

## Four things not to get wrong

**Do not run docket on two machines.** Everything in §1. This is the failure
that cannot be undone by noticing it later.

**Do not edit anything under `store/`.** The register, the ledgers and the index
are generated or append-only. `docket check` will tell you something was edited;
it cannot tell you what it said before.

**Do not expose the API.** §6. SSH or nothing.

**Do not skip `cargo build` by running the old binary.** Use `scripts/serve.sh`,
which is the fix for a fault that has already happened once.
