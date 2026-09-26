#!/usr/bin/env python3
"""Walk a test-evidence ledger and report every way its chain is broken (RFC-0034 §4).

WHY THIS EXISTS
---------------
RFC-0034 §4 puts test evidence in `ledger/YYYY-MM.jsonl`, "append-only and hash-chained
in the manner `tool-docket` already uses for four ledgers: each entry carries
`prev_sha256` and `entry_sha256`, so a rewritten history is detectable rather than
merely discouraged."

"Detectable" is a claim about a program, and this is the program. Without it the two
digest fields are decoration: nothing would notice a line edited in place, a line
deleted from the middle, a line moved between months, or a whole tail re-sealed. A
ledger nobody verifies has exactly the trust properties of a text file.

It is also the module `append-evidence.py` imports, so that "is this chain intact"
has ONE implementation. Two would be two answers to the same question, and the one
that mattered would be whichever ran last.

WHAT IT REFUSES TO DO
---------------------
1. Depend on anything. Standard library only, no schema, no validator, no converter.
   Integrity is the property that must remain checkable when everything else is
   broken — a verifier that cannot run because PyYAML is missing verifies nothing.
   Envelope *validity* is the appender's job (STD-0014: a producer validates, and a
   consumer does the same); this checks that what was accepted is still what is there.
2. Stop at the first fault. `tool-docket`'s `chain::verify` reports every fault for a
   stated reason and the same reason holds here: one altered entry tells an operator
   nothing about whether the rest of the tail was touched.
3. Cascade. One cause produces one fault. An unreadable line makes the chain state
   unknown rather than making every later entry look broken, because a screen of
   faults with one cause is a screen nobody reads.
4. Exit 0 on a fault. This is the only thing standing between a rewritten history and
   a believed one.
5. Guess that a quiet month is a deleted month. See "what the chain does not cover".

THE DIGEST, EXACTLY
-------------------
The chain is per FILE. `entry_sha256` of an entry is:

    SHA-256( canonical_json( the entry object with `entry_sha256` removed ) )

where canonical_json is UTF-8 bytes of `json.dumps(value, sort_keys=True,
separators=(",", ":"), ensure_ascii=False)` — sorted keys, no whitespace, non-ASCII
written as itself rather than escaped. It covers `schema`, `prev_sha256`,
`appended_at`, `envelope_sha256` and the whole `envelope`, so nothing in the line
except the digest field is outside the digest.

`prev_sha256` of the first entry in a file is GENESIS, sixty-four `0`s — the same
constant and the same role as `tool-docket`'s `chain::GENESIS`, so that "first entry"
is a stated value rather than an absent field that a forger could also produce by
deleting one.

The digest is defined over the *value*, not over the bytes on the line: the entry is
parsed and re-canonicalised before hashing. That is deliberate, so a producer in
another language can agree with this one by canonicalising the same way rather than
by reproducing Python's writer. `tool-docket`'s Rust reaches the same bytes for the
same data — `serde_json::Value`'s map is a `BTreeMap`, so its keys are sorted, and
`to_string` is compact and does not escape non-ASCII.

WHAT THE CHAIN DOES NOT COVER, SAID OUT LOUD
--------------------------------------------
Each month file is an independent chain starting at GENESIS. It has to be: an
envelope is filed by its `started_at`, so one that ran last month arrives today and
belongs in last month's file — and a single chain across all months would require
inserting into the middle of it, which is a rewrite, which is the thing being
prevented.

The cost is that deleting an ENTIRE month file leaves nothing to detect it with. A
gap in the month sequence is reported as a note rather than a fault, because a month
in which no tests ran produces exactly the same gap and calling that tampering would
teach operators to ignore the message. Closing this properly needs something outside
the ledger — a signed root, or the file list committed to git — and that is a
decision RFC-0034 has not taken.

USAGE
-----
    verify-evidence.py ledger/                 every month file in a directory
    verify-evidence.py ledger/2026-08.jsonl    one file
    verify-evidence.py -q ledger/              faults and notes only, no per-file line

Exit: 0 no faults, 1 at least one fault (each printed), 2 nothing to verify.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path

# Sixty-four zeros. `tool-docket`'s chain.rs spells GENESIS the same way, and for the
# same reason: the first entry's `prev_sha256` is a value that was written, so a chain
# whose head is missing is distinguishable from one whose head was removed.
GENESIS = "0" * 64

# The wrapper's own version, on the same rule STD-0014 gives the envelope's: a reader
# that does not recognise the major refuses the entry rather than reading the fields it
# happens to know. The envelope carries its own `schema` independently — they version
# separately because the store and the format change for different reasons (STD-0014
# §"What this standard does not cover": the envelope is storage-agnostic on purpose).
LEDGER_SCHEMA = "paridyum.evidence-ledger/1"
LEDGER_MAJOR = LEDGER_SCHEMA.rsplit("/", 1)[1]

DIGEST_KEY = "entry_sha256"

MONTH_FILE = re.compile(r"^([0-9]{4})-(0[1-9]|1[0-2])\.jsonl$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")

if sys.stderr.isatty():
    RED, YLW, GRN, DIM, RST = "\033[31m", "\033[33m", "\033[32m", "\033[2m", "\033[0m"
else:
    RED = YLW = GRN = DIM = RST = ""


class Unusable(Exception):
    """There is nothing to verify, or it cannot be read at all. Exit 2."""


# ── the digest ───────────────────────────────────────────────────────────────
def canonical(value) -> bytes:
    """The one byte-form a value is hashed in. See "THE DIGEST, EXACTLY" above.

    Every digest in this file and in `append-evidence.py` goes through here, so the
    definition exists once. A second spelling of it — a `json.dumps` somewhere with
    different arguments — would silently produce a different chain.
    """
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=False
    ).encode("utf-8")


def digest(value) -> str:
    return hashlib.sha256(canonical(value)).hexdigest()


def entry_digest(entry: dict) -> str:
    """An entry's own digest: everything in it except the field that holds the digest.

    Excluding the field rather than blanking it, which is what `chain::digest_of` does
    in the Rust — a blanked field and an absent one are different objects and would
    hash differently, so the choice has to be made once and written down.
    """
    return digest({k: v for k, v in entry.items() if k != DIGEST_KEY})


def envelope_digest(envelope) -> str:
    """The identity of an envelope, and therefore the idempotency key.

    Lives here rather than in the appender because verification recomputes it too: an
    `envelope_sha256` that does not match its own envelope is an appender that wrote
    the wrong key, and the ledger would then accept a duplicate it thinks is new.
    """
    return digest(envelope)


# ── faults ───────────────────────────────────────────────────────────────────
class Fault:
    """One thing wrong, at one place, with what to do about it.

    `kind`/`index`/`identity` mirror `chain::Fault` in `tool-docket` so the two
    ledgers report the same shapes of damage in the same words. The extra fields are
    the ones a file-backed ledger has and an in-memory chain does not: which file,
    which line, and a remedy — a fault an operator cannot act on is a fault they will
    learn to scroll past.
    """

    def __init__(self, kind, path, line, index, identity, detail, remedy):
        self.kind = kind
        self.path = path
        self.line = line
        self.index = index
        self.identity = identity
        self.detail = detail
        self.remedy = remedy

    def render(self) -> str:
        where = f"line {self.line}"
        if self.index:
            where += f", entry {self.index}"
        head = f"{RED}✘{RST} {self.path}: {where} ({self.identity}): {self.kind}"
        return f"{head}\n      {self.detail}\n      {DIM}{self.remedy}{RST}"


class Note:
    """Something worth saying that is not evidence of tampering. Never fails a run."""

    def __init__(self, path, text):
        self.path = path
        self.text = text

    def render(self) -> str:
        return f"{YLW}!{RST} {self.path}: {self.text}"


def identity(envelope) -> str:
    """How an entry is named in a fault, so a person can find the run it describes.

    Deliberately not the line number, which changes when the file does; a repository,
    a commit and a run are what someone would search for.
    """
    if not isinstance(envelope, dict):
        return "no envelope"
    repo = envelope.get("repository") or "no repository"
    commit = str(envelope.get("commit") or "")[:12] or "no commit"
    run = envelope.get("run")
    if isinstance(run, dict):
        where = f"run {run.get('id')}/{run.get('attempt')}"
    else:
        where = f"local:{envelope.get('submitted_by') or 'anonymous'}"
    return f"{repo}@{commit} {where}"


# ── the walk ─────────────────────────────────────────────────────────────────
def _structure_errors(entry: dict) -> list[str]:
    """Shape problems that make the chain unreadable rather than broken."""
    errs = []
    schema = entry.get("schema")
    if schema != LEDGER_SCHEMA:
        major = str(schema).rsplit("/", 1)[-1] if schema else None
        errs.append(
            f"ledger schema is {schema!r}, not {LEDGER_SCHEMA!r}"
            + (
                f" — major {major} is not {LEDGER_MAJOR}, so the field names in this "
                "entry are not the ones being checked"
                if major != LEDGER_MAJOR
                else ""
            )
        )
    for key in ("prev_sha256", DIGEST_KEY, "envelope_sha256"):
        value = entry.get(key)
        if not isinstance(value, str) or not HEX64.match(value):
            errs.append(f"{key} is {value!r}, not 64 lowercase hex characters")
    if not isinstance(entry.get("appended_at"), str):
        errs.append(f"appended_at is {entry.get('appended_at')!r}, not a string")
    if not isinstance(entry.get("envelope"), dict):
        errs.append("envelope is missing or is not an object")
    return errs


def verify_file(path: Path) -> tuple[list, list[Fault], list[Note]]:
    """Walk one ledger file. Returns (entries, faults, notes).

    `entries` is returned so `append-evidence.py` gets the chain head, the idempotency
    index and the verdict from ONE pass — it must never append to a file it has not
    just verified, and re-reading the file to find that out would be a second pass
    that could disagree with the first.
    """
    faults: list[Fault] = []
    notes: list[Note] = []
    entries: list[dict] = []
    name = str(path)

    month = None
    if MONTH_FILE.match(path.name):
        month = path.name[: -len(".jsonl")]
    else:
        faults.append(Fault(
            "not a ledger file", name, 0, 0, path.name,
            "the name is not YYYY-MM.jsonl, so nothing says which month it holds",
            "a ledger directory holds month files only; move or delete anything else, "
            "because a copy of a month file beside the original is a second history",
        ))

    # `None` means "the chain state here is unknown" — set after an unreadable line so
    # the next entry is not reported as broken by a break we already reported. One
    # cause, one fault (refusal 3 in the header).
    expected_prev: str | None = GENESIS
    index = 0
    first_seen: dict[str, int] = {}
    last_appended = None
    last_started = None

    with path.open(encoding="utf-8") as fh:
        for lineno, raw in enumerate(fh, 1):
            text = raw.rstrip("\n")

            if not text.strip():
                faults.append(Fault(
                    "empty line", name, lineno, 0, "unknown",
                    "this line has no content",
                    "an append-only file never gains a blank line; an entry's contents "
                    "were removed in place. Restore this file from the last copy whose "
                    "verification passed",
                ))
                expected_prev = None
                continue

            try:
                entry = json.loads(text)
            except json.JSONDecodeError as e:
                faults.append(Fault(
                    "unreadable line", name, lineno, 0, "unknown",
                    f"not JSON — {e}",
                    "the line was edited or truncated. Restore this file from the last "
                    "copy whose verification passed",
                ))
                expected_prev = None
                continue

            if not isinstance(entry, dict):
                faults.append(Fault(
                    "unreadable line", name, lineno, 0, "unknown",
                    f"the line is a JSON {type(entry).__name__}, not an object",
                    "every ledger line is one object wrapping one envelope",
                ))
                expected_prev = None
                continue

            index += 1
            envelope = entry.get("envelope")
            who = identity(envelope)

            if problems := _structure_errors(entry):
                for p in problems:
                    faults.append(Fault(
                        "malformed entry", name, lineno, index, who, p,
                        "this entry was not written by append-evidence.py, or was "
                        "written by a version that spelled the chain differently",
                    ))
                expected_prev = None
                entries.append(entry)
                continue

            # ── the two checks that are the whole point ──
            # Order matters for the reader, not for detection: a rewritten entry is
            # reported at the entry that changed, and a removed one at the entry that
            # can no longer reach its predecessor.
            recomputed = entry_digest(entry)
            stored = entry[DIGEST_KEY]
            sealed = recomputed == stored
            if not sealed:
                faults.append(Fault(
                    "contents altered after sealing", name, lineno, index, who,
                    f"recomputed entry_sha256 is {recomputed}, the line claims {stored}",
                    "something in this line was changed after it was appended. The "
                    "ledger is append-only: a correction is a new entry, never an edit. "
                    "Restore this line from the last copy whose verification passed",
                ))

            prev = entry["prev_sha256"]
            if expected_prev is not None and prev != expected_prev:
                was = "GENESIS" if expected_prev == GENESIS else expected_prev
                faults.append(Fault(
                    "does not link to the entry before it", name, lineno, index, who,
                    f"prev_sha256 is {prev} but the entry before it seals as {was}",
                    "an entry was deleted, inserted or reordered here — the digest this "
                    "one points back to is not the one preceding it. Compare against the "
                    "last copy whose verification passed to find which",
                ))

            # ── ledger-specific: an entry filed under the wrong month ──
            # A month file is only meaningful if what is in it is what its name says.
            # This is also the shape a mover leaves behind: a line lifted from one file
            # into another keeps its own started_at.
            started = envelope.get("started_at") if isinstance(envelope, dict) else None
            if month and isinstance(started, str):
                if started[:7] != month:
                    faults.append(Fault(
                        "filed under the wrong month", name, lineno, index, who,
                        f"the envelope started at {started}, which is {started[:7]}, "
                        f"but this file is {month}",
                        f"an entry is filed by its envelope's started_at. Either this "
                        f"line was moved here, or it was appended by something that "
                        f"used the wall clock instead of the run's own start time",
                    ))
                elif last_started and started < last_started:
                    notes.append(Note(
                        name,
                        f"line {lineno}: started_at {started} precedes the entry before "
                        f"it ({last_started}); a late submission does this legitimately, "
                        "so it is reported and not treated as a fault",
                    ))
                last_started = started

            # ── the idempotency key, checked rather than trusted ──
            # Only meaningful on an entry whose own seal holds. When it does not, the
            # line has already been reported as altered and every sub-digest inside it
            # is wrong for that one reason — refusal 3, one cause, one fault. When the
            # seal DOES hold, a mismatch here is a different animal: an appender that
            # wrote the wrong key, or a forger who re-sealed a line without noticing
            # this field, and either lets a duplicate in as if it were new.
            claimed = entry["envelope_sha256"]
            actual = envelope_digest(envelope)
            if sealed and claimed != actual:
                faults.append(Fault(
                    "envelope_sha256 does not match its envelope", name, lineno, index, who,
                    f"the envelope digests to {actual}, the entry claims {claimed}",
                    "the appender's duplicate check reads this field, so a wrong value "
                    "lets the same envelope be appended twice as if it were new",
                ))
            if (earlier := first_seen.get(actual)) is not None:
                faults.append(Fault(
                    "duplicate envelope", name, lineno, index, who,
                    f"the identical envelope is already at line {earlier}",
                    "append-evidence.py refuses to append an envelope this file already "
                    "holds, so two copies mean a line was inserted by hand or two "
                    "appenders wrote at the same time",
                ))
            else:
                first_seen[actual] = lineno

            appended = entry["appended_at"]
            if last_appended and appended < last_appended:
                notes.append(Note(
                    name,
                    f"line {lineno}: appended_at {appended} precedes the entry before it "
                    f"({last_appended}); clocks on two submitting machines can differ, so "
                    "this is reported and not treated as a fault — a real reordering "
                    "shows up as a broken link above",
                ))
            last_appended = appended

            expected_prev = stored
            entries.append(entry)

    return entries, faults, notes


def month_gap_notes(paths: list[Path]) -> list[Note]:
    """Months with no file between the earliest and the latest.

    A note, never a fault. See "WHAT THE CHAIN DOES NOT COVER": a month in which
    nothing ran and a month whose file was deleted are indistinguishable from inside
    the ledger, and reporting the first as tampering would train people to ignore the
    second.
    """
    months = sorted(p.name[: -len(".jsonl")] for p in paths if MONTH_FILE.match(p.name))
    if len(months) < 2:
        return []
    ordinals = {}
    for m in months:
        y, mo = int(m[:4]), int(m[5:7])
        ordinals[y * 12 + (mo - 1)] = m
    lo, hi = min(ordinals), max(ordinals)
    missing = [
        f"{o // 12:04d}-{o % 12 + 1:02d}" for o in range(lo, hi + 1) if o not in ordinals
    ]
    if not missing:
        return []
    return [Note(
        "ledger",
        f"no file for {', '.join(missing)}, between {months[0]} and {months[-1]}. "
        "A month with no test runs looks exactly like a month whose file was deleted; "
        "the chain is per-file and cannot tell them apart (RFC-0034 §4 does not say "
        "which it should be)",
    )]


def collect(targets: list[Path]) -> list[Path]:
    files: list[Path] = []
    for t in targets:
        if t.is_dir():
            files.extend(sorted(t.glob("*.jsonl")))
        elif t.is_file():
            files.append(t)
        else:
            raise Unusable(f"{t} is neither a file nor a directory")
    return files


def main(argv=None) -> int:
    p = argparse.ArgumentParser(
        prog="verify-evidence.py",
        description="Verify the hash chain of a test evidence ledger (RFC-0034 §4).",
    )
    p.add_argument("target", nargs="+", type=Path,
                   help="a ledger directory, or one YYYY-MM.jsonl file; repeatable")
    p.add_argument("-q", "--quiet", action="store_true",
                   help="print faults and notes only, not a line per clean file")
    args = p.parse_args(argv)

    try:
        files = collect(args.target)
    except Unusable as e:
        print(f"verify-evidence: {e}", file=sys.stderr)
        return 2

    if not files:
        # Not exit 0. An empty result from a verifier reads as "verified", and the one
        # thing this must never do is say a ledger is intact when it did not read one.
        print(f"verify-evidence: no ledger file in {', '.join(str(t) for t in args.target)}; "
              "nothing was verified", file=sys.stderr)
        return 2

    all_faults: list[Fault] = []
    all_notes: list[Note] = month_gap_notes(files)
    total_entries = 0

    for f in files:
        try:
            entries, faults, notes = verify_file(f)
        except OSError as e:
            print(f"verify-evidence: cannot read {f} — {e}", file=sys.stderr)
            return 2
        total_entries += len(entries)
        all_faults.extend(faults)
        all_notes.extend(notes)
        if not args.quiet:
            mark = f"{GRN}✔{RST}" if not faults else f"{RED}✘{RST}"
            plural = "entry" if len(entries) == 1 else "entries"
            verdict = "chain intact" if not faults else f"{len(faults)} fault(s)"
            print(f"{mark} {f}: {len(entries)} {plural}, {verdict}")

    sys.stdout.flush()  # per-file lines are stdout, faults are stderr; keep the order
    for n in all_notes:
        print(n.render(), file=sys.stderr)
    for fault in all_faults:
        print(fault.render(), file=sys.stderr)

    # Counted, never asserted: ADR-0035's rule about numbers in prose is the same rule
    # about numbers in a summary line.
    plural = "entry" if total_entries == 1 else "entries"
    if all_faults:
        print(f"\n{RED}✘{RST} {len(all_faults)} fault(s) across {total_entries} "
              f"{plural} in {len(files)} file(s). This history has been rewritten; "
              "the entries above say where.", file=sys.stderr)
        return 1
    print(f"\n{GRN}✔{RST} {total_entries} {plural} in {len(files)} file(s), "
          f"every chain intact"
          + (f", {len(all_notes)} note(s)" if all_notes else ""))
    return 0


if __name__ == "__main__":
    sys.exit(main())
