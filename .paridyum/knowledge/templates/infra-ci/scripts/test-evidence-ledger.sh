#!/usr/bin/env bash
#
# Proves append-evidence.py and verify-evidence.py by breaking them.
#
# Same rule as scripts/test-checks.sh and test-junit-to-envelope.sh: "treat a green
# check as unproven until you have watched it fail". Here it is the whole product.
# RFC-0034 §4 asks for a ledger in which "a rewritten history is detectable rather than
# merely discouraged" — and a chain that has only ever been seen to verify is
# indistinguishable from one that always returns 0. So every fault verify-evidence.py
# can report is planted, in a real ledger built by the real appender from envelopes the
# real converter produced, and watched.
#
# The cases that carry the most weight are the three tampering ones, because they are
# the claim RFC-0034 makes on the ledger's behalf:
#
#   * a line edited in place            -> "contents altered after sealing"
#   * a line deleted from the middle    -> "does not link to the entry before it"
#   * a line edited AND re-sealed       -> the NEXT entry stops linking
#
# The third is the one that shows the chain is doing work rather than the digest alone:
# fixing an entry's own hash does not fix the entry that points at it.
#
#   ./test-evidence-ledger.sh        run every case
#   ./test-evidence-ledger.sh -v     show each case's output

set -uo pipefail
HERE="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
APPEND="$HERE/append-evidence.py"
VERIFY="$HERE/verify-evidence.py"
CONV="$HERE/junit-to-envelope.py"
VERBOSE=false
if [[ "${1:-}" == "-v" ]]; then VERBOSE=true; fi

if [[ -t 1 ]]; then RED=$'\033[31m'; GRN=$'\033[32m'; RST=$'\033[0m'; else RED=""; GRN=""; RST=""; fi
PASS=0; FAIL=0
ok()  { printf '%s✔%s %s\n' "$GRN" "$RST" "$1"; PASS=$((PASS+1)); }
bad() {
  printf '%s✘%s %s\n' "$RED" "$RST" "$1"; FAIL=$((FAIL+1))
  if [[ -n "${2:-}" ]]; then printf '%s\n' "$2" | sed 's/^/      /' | head -12; fi
}

# Provenance and identity are read from the environment when no flag gives them, so a
# caller's exported GITHUB_* would silently change what these cases are testing.
unset GITHUB_REPOSITORY GITHUB_SHA GITHUB_REF GITHUB_RUN_ID GITHUB_RUN_ATTEMPT \
      GITHUB_SERVER_URL GITHUB_ACTIONS PARIDYUM_OPERATOR 2>/dev/null

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
cd "$WORK" || exit 1

GENESIS="0000000000000000000000000000000000000000000000000000000000000000"

# ── assertions ───────────────────────────────────────────────────────────────
# expect_ok <label> <cmd...>
expect_ok() {
  local label="$1"; shift
  local out rc
  out="$("$@" 2>&1)"; rc=$?
  if [[ "$VERBOSE" == true ]]; then printf '%s\n' "$out"; fi
  if [[ $rc -eq 0 ]]; then ok "$label"; else bad "$label — expected exit 0, got $rc" "$out"; fi
}
# expect_refused <label> <pattern> <cmd...> — must exit 1 AND say why
expect_refused() {
  local label="$1" pat="$2"; shift 2
  local out rc
  out="$("$@" 2>&1)"; rc=$?
  if [[ "$VERBOSE" == true ]]; then printf '%s\n' "$out"; fi
  if [[ $rc -ne 1 ]]; then
    bad "$label — the planted defect was NOT refused (exit $rc)" "$out"
  elif ! grep -qF "$pat" <<<"$out"; then
    bad "$label — refused, but nothing in the message matched '$pat'" "$out"
  else ok "$label"; fi
}
# expect_exit <label> <code> <cmd...>
expect_exit() {
  local label="$1" want="$2"; shift 2
  local out rc
  out="$("$@" 2>&1)"; rc=$?
  if [[ "$VERBOSE" == true ]]; then printf '%s\n' "$out"; fi
  if [[ $rc -eq $want ]]; then ok "$label"; else bad "$label — expected exit $want, got $rc" "$out"; fi
}
# expect_warns <label> <pattern> <cmd...> — exit 0 and warn
expect_warns() {
  local label="$1" pat="$2"; shift 2
  local out rc
  out="$("$@" 2>&1)"; rc=$?
  if [[ "$VERBOSE" == true ]]; then printf '%s\n' "$out"; fi
  if [[ $rc -ne 0 ]]; then bad "$label — expected exit 0, got $rc" "$out"
  elif ! grep -qF "$pat" <<<"$out"; then bad "$label — no warning matched '$pat'" "$out"
  else ok "$label"; fi
}
# expect_lines <label> <file> <n>
expect_lines() {
  local got
  if [[ ! -f "$2" ]]; then got=missing; else got="$(awk 'END{print NR}' "$2")"; fi
  if [[ "$got" == "$3" ]]; then ok "$1"; else bad "$1 — $2 has $got line(s), expected $3"; fi
}
# expect_no_file <label> <path>
expect_no_file() {
  if [[ -e "$2" ]]; then bad "$1 — $2 exists"; else ok "$1"; fi
}
# field <file> <lineno> <dotted path>
field() {
  python3 - "$1" "$2" "$3" <<'PY'
import json, sys
line = open(sys.argv[1], encoding="utf-8").read().splitlines()[int(sys.argv[2]) - 1]
doc = json.loads(line)
for key in sys.argv[3].split("."):
    doc = doc[int(key)] if key.isdigit() else doc[key]
print(json.dumps(doc, sort_keys=True))
PY
}
# expect_field <label> <file> <lineno> <path> <expected>
expect_field() {
  local got; got="$(field "$2" "$3" "$4" 2>&1)"
  if [[ "$got" == "$5" ]]; then ok "$1"; else bad "$1 — line $3 $4 is $got, expected $5"; fi
}
# expect_fault <label> <pattern> <target...> — verify must exit 1 and name the fault
expect_fault() {
  local label="$1" pat="$2"; shift 2
  local out rc
  out="$(python3 "$VERIFY" "$@" 2>&1)"; rc=$?
  if [[ "$VERBOSE" == true ]]; then printf '%s\n' "$out"; fi
  if [[ $rc -ne 1 ]]; then
    bad "$label — verify did NOT report a fault (exit $rc)" "$out"
  elif ! grep -qF "$pat" <<<"$out"; then
    bad "$label — a fault was reported but none matched '$pat'" "$out"
  else ok "$label"; fi
}
# expect_clean <label> <target...>
expect_clean() {
  local label="$1"; shift
  expect_ok "$label" python3 "$VERIFY" "$@"
}

# ── helpers ──────────────────────────────────────────────────────────────────
# junit <file> <timestamp> <name> — one passing case, so every envelope differs in
# content (different artifact path and digest) while staying trivially valid.
junit() {
  cat > "$1" <<X
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="$3" tests="1" failures="0" errors="0" time="0.5">
  <testsuite name="$3" timestamp="$2" tests="1" time="0.5">
    <testcase classname="$3" name="one" time="0.5"/>
  </testsuite>
</testsuites>
X
}

ID=(--repository Paridyum/console-docket
    --commit 97f05b0e1c2d3f4a5b6c7d8e9f0a1b2c3d4e5f60
    --ref refs/heads/main)
L=(--tier pr --stack typescript --implements TST-0001 --provenance local --submitted-by rohan)

# envelope <out> <junit> — build a real envelope with the real converter, so nothing
# here tests the ledger against a shape the producer would never emit.
envelope() { python3 "$CONV" "${ID[@]}" "${L[@]}" --out "$1" "$2" >/dev/null 2>&1; }

# variant <in> <out> <op...> — `path=json` sets, `-path` deletes. Used only for the
# envelopes the converter CANNOT produce: it computes totals from one parse and derives
# run/submitted_by from provenance, so the cross-field defects are unreachable through
# its CLI and have to be built here.
cat > variant.py <<'PY'
import json, sys
doc = json.load(open(sys.argv[1], encoding="utf-8"))
for op in sys.argv[3:]:
    if op.startswith("-"):
        path = op[1:].split("."); node = doc
        for k in path[:-1]: node = node[k]
        node.pop(path[-1], None)
    else:
        raw, _, value = op.partition("=")
        path = raw.split("."); node = doc
        for k in path[:-1]: node = node[k]
        node[path[-1]] = json.loads(value)
open(sys.argv[2], "w", encoding="utf-8").write(json.dumps(doc, indent=2) + "\n")
PY
variant() { python3 variant.py "$@"; }

# edit_line <file> <lineno> <op...> — the same, applied to a ledger LINE in place.
cat > editline.py <<'PY'
import json, sys
path, lineno = sys.argv[1], int(sys.argv[2])
lines = open(path, encoding="utf-8").read().splitlines()
doc = json.loads(lines[lineno - 1])
reseal = False
for op in sys.argv[3:]:
    if op == "--reseal":
        reseal = True
        continue
    raw, _, value = op.partition("=")
    keys = raw.split("."); node = doc
    for k in keys[:-1]: node = node[k]
    node[keys[-1]] = json.loads(value)
if reseal:
    # Re-seal exactly as the documented rule says: SHA-256 over canonical JSON of the
    # entry with `entry_sha256` removed. A forger who knows the rule can do this, which
    # is the point of the case that uses it: it still does not fix the NEXT entry.
    import hashlib
    body = {k: v for k, v in doc.items() if k != "entry_sha256"}
    doc["entry_sha256"] = hashlib.sha256(
        json.dumps(body, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
        .encode("utf-8")).hexdigest()
lines[lineno - 1] = json.dumps(doc, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
open(path, "w", encoding="utf-8").write("\n".join(lines) + "\n")
PY
edit_line() { python3 editline.py "$@"; }

printf '\n── a first append is genesis ──────────────────────────────────────\n'
junit j1.xml 2026-08-03T09:00:00Z alpha
envelope e1.json j1.xml
expect_ok    "the first envelope appends" python3 "$APPEND" --ledger-dir led e1.json
expect_lines "  the month file has one line" led/2026-08.jsonl 1
expect_field "  the ledger line declares its own schema" led/2026-08.jsonl 1 schema '"paridyum.evidence-ledger/1"'
expect_field "  prev_sha256 is GENESIS, written rather than omitted" led/2026-08.jsonl 1 prev_sha256 "\"$GENESIS\""
expect_field "  the envelope is carried whole, not summarised" led/2026-08.jsonl 1 envelope.repository '"Paridyum/console-docket"'
expect_field "  and its provenance survives (STD-0014 rule 3)" led/2026-08.jsonl 1 envelope.provenance '"local"'
expect_clean "  the chain verifies" led

# The digest is only useful if someone else can compute it. This reimplements the rule
# from append-evidence.py's header — sorted keys, compact separators, UTF-8, the
# entry_sha256 key REMOVED — without importing either script, and compares.
cat > recompute.py <<'PY'
"""An independent implementation of the documented digest, from the header alone."""
import hashlib, json, sys
bad = 0
for lineno, line in enumerate(open(sys.argv[1], encoding="utf-8"), 1):
    entry = json.loads(line)
    body = {k: v for k, v in entry.items() if k != "entry_sha256"}
    want = hashlib.sha256(json.dumps(
        body, sort_keys=True, separators=(",", ":"), ensure_ascii=False
    ).encode("utf-8")).hexdigest()
    if want != entry["entry_sha256"]:
        print(f"line {lineno}: independent digest {want} != stored {entry['entry_sha256']}")
        bad += 1
    env = hashlib.sha256(json.dumps(
        entry["envelope"], sort_keys=True, separators=(",", ":"), ensure_ascii=False
    ).encode("utf-8")).hexdigest()
    if env != entry["envelope_sha256"]:
        print(f"line {lineno}: independent envelope digest {env} != stored")
        bad += 1
sys.exit(1 if bad else 0)
PY
expect_ok "the digest definition in the header reproduces the stored digests" \
  python3 recompute.py led/2026-08.jsonl

printf '\n── a second append links to the first ─────────────────────────────\n'
junit j2.xml 2026-08-04T09:00:00Z beta
envelope e2.json j2.xml
expect_ok    "the second envelope appends" python3 "$APPEND" --ledger-dir led e2.json
expect_lines "  the month file has two lines" led/2026-08.jsonl 2
HEAD1="$(field led/2026-08.jsonl 1 entry_sha256)"
expect_field "  entry 2 points back at entry 1's digest" led/2026-08.jsonl 2 prev_sha256 "$HEAD1"
expect_clean "  the chain still verifies" led
expect_ok    "  and the independent digest still agrees" python3 recompute.py led/2026-08.jsonl

printf '\n── appending the same envelope twice is a no-op ───────────────────\n'
expect_ok    "re-appending an envelope already present exits 0" python3 "$APPEND" --ledger-dir led e1.json
expect_lines "  and writes no second entry" led/2026-08.jsonl 2
expect_warns "  it says where the envelope already is" "already in" \
  python3 "$APPEND" --ledger-dir led e1.json
# Identity is the envelope's own bytes, so a byte-identical copy under another filename
# is the same evidence. This is the case the composite key (repository + run + attempt +
# artifact path) could not decide for a `local` envelope, which has no run at all.
cp e1.json e1-copy.json
expect_ok    "a byte-identical copy under a different name is the same evidence" \
  python3 "$APPEND" --ledger-dir led e1-copy.json
expect_lines "  still two entries" led/2026-08.jsonl 2

printf '\n── a different envelope for the same job is kept, and announced ───\n'
# An append-only ledger does not replace a claim. Both entries stay; the operator is
# told that one job has now reported two different results.
variant e1.json e1-other.json 'duration_ms=999' 'totals={"tests":1,"passed":0,"failed":1,"skipped":0,"errored":0}' 'outcome="failed"'
expect_warns "the same job reporting different numbers warns rather than replacing" \
  "reported two different results" python3 "$APPEND" --ledger-dir led e1-other.json
expect_lines "  and both claims are on disk" led/2026-08.jsonl 3
expect_clean "  the chain absorbs it" led

printf '\n── refusals: an envelope that is not valid never reaches the ledger ─\n'
BEFORE="$(awk 'END{print NR}' led/2026-08.jsonl)"
variant e2.json bad-commit.json 'commit="97f05b0"'
expect_refused "an abbreviated commit is refused by the schema" \
  "does not match" python3 "$APPEND" --ledger-dir led bad-commit.json
variant e2.json bad-tier.json 'tier="weekly"'
expect_refused "a tier outside STD-0008 is refused by the schema" \
  "is not one of" python3 "$APPEND" --ledger-dir led bad-tier.json
variant e2.json bad-extra.json 'nickname="flaky one"'
expect_refused "an unknown field is refused (additionalProperties: false)" \
  "unexpected property" python3 "$APPEND" --ledger-dir led bad-extra.json
variant e2.json bad-month.json 'started_at="2026-99-04T09:00:00Z"'
expect_refused "a started_at that matches the pattern but is not a date is refused" \
  "is not a real RFC 3339 UTC instant" python3 "$APPEND" --ledger-dir led bad-month.json
expect_no_file "  and no ledger/2026-99.jsonl was created" led/2026-99.jsonl
printf 'not json at all' > junk.json
expect_refused "a file that is not JSON is refused" \
  "not parseable JSON" python3 "$APPEND" --ledger-dir led junk.json
expect_lines "nothing invalid reached the ledger" led/2026-08.jsonl "$BEFORE"
expect_clean "  and the chain is untouched" led

printf '\n── refusals: the three rules the schema cannot enforce (STD-0014) ──\n'
# Each of these passes the vendored validator — it announces that it does not implement
# allOf/if/then — and is caught by the hand-written check the appender borrows from
# junit-to-envelope.py, so producer and consumer cannot drift apart.
variant e2.json no-submitter.json '-submitted_by'
expect_refused "a local envelope with no submitted_by is refused" \
  "an anonymous claim about a test run" python3 "$APPEND" --ledger-dir led no-submitter.json
variant e2.json local-with-run.json 'run={"id":"1","attempt":1,"url":"https://github.com/x"}'
expect_refused "a local envelope carrying a run is refused (forbidden when local)" \
  "a local envelope has no inspectable run" python3 "$APPEND" --ledger-dir led local-with-run.json
variant e2.json ci-no-run.json 'provenance="ci"' '-submitted_by'
expect_refused "a ci envelope with no run is refused" \
  "the run cannot be traced to its logs" python3 "$APPEND" --ledger-dir led ci-no-run.json
variant e2.json bad-totals.json 'totals={"tests":1,"passed":2,"failed":0,"skipped":0,"errored":0}'
expect_refused "totals that do not sum to tests are refused" \
  "but tests = 1" python3 "$APPEND" --ledger-dir led bad-totals.json
variant e2.json silent-skip.json 'outcome="skipped"' 'skipped_reason=null'
expect_refused "a skipped outcome with no reason is refused" \
  "indistinguishable from a pass downstream" python3 "$APPEND" --ledger-dir led silent-skip.json
expect_lines "still nothing invalid on disk" led/2026-08.jsonl "$BEFORE"

printf '\n── the month comes from started_at, never from the clock ──────────\n'
junit j-jul.xml 2026-07-28T11:30:00Z july
envelope e-jul.json j-jul.xml
expect_ok    "an envelope from a previous month appends" python3 "$APPEND" --ledger-dir led e-jul.json
expect_lines "  it lands in that month's file" led/2026-07.jsonl 1
expect_lines "  and not in the current month's" led/2026-08.jsonl "$BEFORE"
expect_field "  the July file starts its own chain at GENESIS" led/2026-07.jsonl 1 prev_sha256 "\"$GENESIS\""
expect_field "  and holds the July run" led/2026-07.jsonl 1 envelope.started_at '"2026-07-28T11:30:00Z"'
expect_clean "  both month files verify" led

printf '\n── --dry-run writes nothing ───────────────────────────────────────\n'
junit j-dry.xml 2026-08-09T09:00:00Z dry
envelope e-dry.json j-dry.xml
expect_ok    "a dry run reports what it would do" python3 "$APPEND" --ledger-dir led --dry-run e-dry.json
expect_lines "  and appends nothing" led/2026-08.jsonl "$BEFORE"

printf '\n── TAMPERING: a line edited in place is detected ───────────────────\n'
# The claim RFC-0034 §4 makes: "a rewritten history is detectable rather than merely
# discouraged". Worked on a real ledger, one entry at a time, restoring between cases so
# each fault is reported in isolation rather than in a cascade.
cp -R led led-good
edit_line led/2026-08.jsonl 2 'envelope.outcome="failed"'
expect_fault "flipping an outcome from passed to failed is caught" \
  "contents altered after sealing" led
expect_fault "  the fault names the entry, not just the file" "entry 2" led
expect_fault "  and says what to do about it" "a correction is a new entry, never an edit" led
rm -rf led && cp -R led-good led
expect_clean "restoring the line makes it verify again" led

# One byte, in the middle of a digest, leaving the JSON valid. Nothing about the run
# changed and it is still caught, because the digest covers the whole entry.
edit_line led/2026-08.jsonl 1 'envelope.commit="97f05b0e1c2d3f4a5b6c7d8e9f0a1b2c3d4e5f61"'
expect_fault "a single character changed in a commit sha is caught" \
  "contents altered after sealing" led
rm -rf led && cp -R led-good led

# A raw byte mutation that breaks the JSON — the crudest edit, and it must not read as
# "verified" merely because the line could not be parsed.
python3 - <<'PY'
from pathlib import Path
p = Path("led/2026-08.jsonl")
lines = p.read_text(encoding="utf-8").splitlines()
lines[1] = lines[1][:20] + "\x00" + lines[1][21:]
p.write_text("\n".join(lines) + "\n", encoding="utf-8")
PY
expect_fault "a corrupted byte that breaks the JSON is reported, not skipped" \
  "unreadable line" led
rm -rf led && cp -R led-good led

printf '\n── TAMPERING: a deleted middle line is detected ────────────────────\n'
sed -i.bak '2d' led/2026-08.jsonl && rm -f led/2026-08.jsonl.bak
expect_fault "deleting the middle entry breaks the link of the one after it" \
  "does not link to the entry before it" led
expect_fault "  and the message says an entry was deleted, inserted or reordered" \
  "deleted, inserted or reordered" led
rm -rf led && cp -R led-good led

# Blanking a line rather than removing it: the line count is unchanged, which is what an
# editor's "delete the contents" leaves behind.
python3 - <<'PY'
from pathlib import Path
p = Path("led/2026-08.jsonl")
lines = p.read_text(encoding="utf-8").splitlines()
lines[1] = ""
p.write_text("\n".join(lines) + "\n", encoding="utf-8")
PY
expect_fault "emptying a line without removing it is caught" "empty line" led
rm -rf led && cp -R led-good led

printf '\n── TAMPERING: re-sealing the edited entry does not save the forger ─\n'
# The case that proves the CHAIN is load-bearing and not just the per-entry digest. The
# entry's own hash is recomputed correctly by the documented rule, so "contents altered"
# no longer fires — and the entry after it still points at the old digest.
edit_line led/2026-08.jsonl 2 'envelope.outcome="failed"' --reseal
expect_fault "editing an entry and correctly re-sealing it breaks the NEXT entry" \
  "does not link to the entry before it" led
expect_fault "  reported at entry 3, the first one that can no longer reach back" "entry 3" led
rm -rf led && cp -R led-good led

printf '\n── TAMPERING: an entry moved into another month is detected ────────\n'
mkdir -p moved
cp led/2026-07.jsonl moved/2026-06.jsonl
expect_fault "a line filed under a month that is not its own is caught" \
  "filed under the wrong month" moved
expect_fault "  the message names both months" "but this file is 2026-06" moved

printf '\n── TAMPERING: a hand-inserted duplicate is detected ────────────────\n'
mkdir -p dup
head -2 led/2026-08.jsonl > dup/2026-08.jsonl
head -1 led/2026-08.jsonl >> dup/2026-08.jsonl
expect_fault "the same envelope appearing twice in one file is caught" "duplicate envelope" dup

printf '\n── TAMPERING: an envelope_sha256 that lies about its envelope ──────\n'
# The idempotency key is checked rather than trusted: a wrong one would let the same
# envelope be appended again as if it were new.
mkdir -p liar
cp led/2026-08.jsonl liar/2026-08.jsonl
edit_line liar/2026-08.jsonl 1 "envelope_sha256=\"$GENESIS\"" --reseal
expect_fault "an envelope_sha256 that does not match its envelope is caught" \
  "does not match its envelope" liar

printf '\n── the appender refuses to extend a broken chain ───────────────────\n'
rm -rf led && cp -R led-good led
edit_line led/2026-08.jsonl 2 'envelope.outcome="failed"'
BROKEN="$(awk 'END{print NR}' led/2026-08.jsonl)"
expect_refused "appending onto a broken chain is refused" \
  "Appending would seal a rewritten history" python3 "$APPEND" --ledger-dir led e-dry.json
expect_lines "  and the broken file did not grow" led/2026-08.jsonl "$BROKEN"
# A fault in August must not stop July's evidence getting in: the chain is per file.
junit j-jul2.xml 2026-07-29T11:30:00Z july2
envelope e-jul2.json j-jul2.xml
expect_ok    "a healthy month still accepts appends while another month is broken" \
  python3 "$APPEND" --ledger-dir led e-jul2.json
expect_lines "  July grew" led/2026-07.jsonl 2
rm -rf led && cp -R led-good led

printf '\n── verify never says "intact" about something it did not read ──────\n'
mkdir -p empty
expect_exit "an empty directory is exit 2, not a clean bill of health" 2 python3 "$VERIFY" empty
expect_exit "a path that does not exist is exit 2" 2 python3 "$VERIFY" nowhere
mkdir -p stray && cp led/2026-08.jsonl stray/backup.jsonl
expect_fault "a .jsonl in a ledger directory that is not a month file is a fault" \
  "not a ledger file" stray

printf '\n── the appender refuses to run without the checks it claims ────────\n'
# Refusal 1 depends on the cross-field rules imported from junit-to-envelope.py. If that
# file is not there the honest answer is exit 2, never a weaker check that still exits 0.
expect_exit "a missing converter is exit 2, not a silent downgrade" 2 \
  python3 "$APPEND" --ledger-dir led --converter /nonexistent/junit-to-envelope.py e-dry.json
expect_exit "a missing schema is exit 2" 2 \
  python3 "$APPEND" --ledger-dir led --schema /nonexistent/schema.json e-dry.json
expect_lines "  and neither wrote anything" led/2026-08.jsonl "$BEFORE"

printf '\n── an envelope on stdin, the way the scheduled pull delivers it ────\n'
# collect-evidence.py pulls envelopes from the GitHub API and pipes them in; making it
# write a temporary file first would put unvalidated evidence on disk to validate it.
junit j-pipe.xml 2026-08-14T09:00:00Z pipe
envelope e-pipe.json j-pipe.xml
PIPE_BEFORE="$(awk 'END{print NR}' led/2026-08.jsonl)"
OUT="$(python3 "$APPEND" --ledger led - < e-pipe.json 2>&1)"; RC=$?
if [[ "$VERBOSE" == true ]]; then printf '%s\n' "$OUT"; fi
if [[ $RC -eq 0 ]]; then ok "an envelope read from stdin appends (--ledger, as the collector spells it)"
else bad "stdin append failed (exit $RC)" "$OUT"; fi
expect_lines "  the ledger grew by one" led/2026-08.jsonl "$((PIPE_BEFORE + 1))"
if grep -qF "<stdin>" <<<"$OUT"; then ok "  and it is named <stdin> rather than a path that does not exist"
else bad "  the stdin source was not named" "$OUT"; fi
expect_clean "  the chain still verifies" led
# Nothing is written until everything is read: a missing file is a usage error, and
# discovering it after three lines are on disk leaves a batch half-done.
BATCH_BEFORE="$(awk 'END{print NR}' led/2026-08.jsonl)"
junit j-b1.xml 2026-08-15T09:00:00Z batch1
envelope e-b1.json j-b1.xml
expect_exit "a batch naming a file that does not exist writes nothing at all" 2 \
  python3 "$APPEND" --ledger-dir led e-b1.json /nonexistent/envelope.json
expect_lines "  the ledger did not grow" led/2026-08.jsonl "$BATCH_BEFORE"

printf '\n── the reuse is live, not decorative ───────────────────────────────\n'
# STD-0014 quotes the validator's own admission that it does not enforce allOf. If it
# ever stops saying so, the reason the cross-field checks are hand-written is stale and
# the next reader should be told rather than left to assume.
NOTE_OUT="$(python3 "$APPEND" --ledger-dir led --dry-run e-dry.json 2>&1)"
if grep -qF 'unsupported keyword, not enforced — <root>: allOf' <<<"$NOTE_OUT"; then
  ok "the appender relays the validator's allOf gap (STD-0014 quotes this)"
else
  bad "the appender no longer relays the allOf gap — STD-0014's reasoning needs rechecking" "$NOTE_OUT"
fi
if grep -q 'cross_field_errors' "$APPEND"; then
  ok "the cross-field rules are borrowed from junit-to-envelope.py, not copied"
else
  bad "append-evidence.py no longer reuses cross_field_errors — the two can now drift"
fi

printf '\n── a real CI envelope, and a real run of the whole thing ───────────\n'
junit j-ci.xml 2026-08-11T09:00:00Z ci
# A subshell so the exported GITHUB_* reach both commands and leave with them: the
# converter reads the environment to decide provenance, which is the shape the reusable
# workflows will call this pair in.
ci_append() (
  export GITHUB_REPOSITORY=Paridyum/console-docket
  export GITHUB_SHA=97f05b0e1c2d3f4a5b6c7d8e9f0a1b2c3d4e5f60
  export GITHUB_REF=refs/pull/12/merge
  export GITHUB_RUN_ID=32492213670
  export GITHUB_RUN_ATTEMPT=2
  export GITHUB_SERVER_URL=https://github.com
  python3 "$CONV" --tier pr --stack typescript --implements TST-0001 \
    --out e-ci.json j-ci.xml >/dev/null 2>&1 || return 1
  python3 "$APPEND" --ledger-dir led e-ci.json
)
expect_ok "a ci envelope built from the GitHub Actions environment appends" ci_append
CI_LINE="$(awk 'END{print NR}' led/2026-08.jsonl)"
expect_field "  the run identity is carried into the ledger" led/2026-08.jsonl "$CI_LINE" envelope.run.id '"32492213670"'
expect_field "  and its provenance is ci, distinguishable from the local entries" \
  led/2026-08.jsonl "$CI_LINE" envelope.provenance '"ci"'
expect_field "  it links to the local entry before it, both kinds in one chain" \
  led/2026-08.jsonl "$CI_LINE" prev_sha256 "$(field led/2026-08.jsonl $((CI_LINE - 1)) entry_sha256)"
expect_clean "the finished ledger verifies end to end" led
expect_ok    "and every digest in it is reproducible from the documented rule" \
  python3 recompute.py led/2026-08.jsonl

printf '\n───────────────────────────────────────────────────────────────────\n'
printf '%d passed, %d failed\n' "$PASS" "$FAIL"
if [[ $FAIL -ne 0 ]]; then exit 1; fi
