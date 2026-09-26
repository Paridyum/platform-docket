#!/usr/bin/env bash
#
# Proves junit-to-envelope.py by breaking it.
#
# Same rule as scripts/test-checks.sh: "treat a green check as unproven until you have
# watched it fail". This converter's whole value is its refusals — an envelope it emits
# is a claim about a test run, and a validator only ever seen to pass is
# indistinguishable from one that cannot fail. So every refusal in the header of
# junit-to-envelope.py is planted here and watched.
#
# The two cases that matter most are the cross-field rules STD-0014 §"Two rules the
# schema cannot enforce" names. They cannot be reached through the CLI, because the
# converter computes totals from one parse and derives `run`/`submitted_by` from
# provenance, so a valid input can never produce those defects. They are reached by
# corrupting the envelope between build and write — which also demonstrates the
# standard's actual claim: the schema validator passes the corrupted envelope and the
# hand-written check catches it.
#
#   ./test-junit-to-envelope.sh        run every case
#   ./test-junit-to-envelope.sh -v     show each case's output

set -uo pipefail
HERE="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd -- "$HERE/../../.." && pwd)"
CONV="$HERE/junit-to-envelope.py"
VERBOSE=false; [[ "${1:-}" == "-v" ]] && VERBOSE=true

if [[ -t 1 ]]; then RED=$'\033[31m'; GRN=$'\033[32m'; RST=$'\033[0m'; else RED=""; GRN=""; RST=""; fi
PASS=0; FAIL=0
ok()  { printf '%s✔%s %s\n' "$GRN" "$RST" "$1"; PASS=$((PASS+1)); }
bad() { printf '%s✘%s %s\n' "$RED" "$RST" "$1"; FAIL=$((FAIL+1)); [[ -n "${2:-}" ]] && printf '%s\n' "$2" | sed 's/^/      /' | head -6; }

# The environment a caller may have set decides provenance and identity. Every case
# states what it needs, so the suite must start from nothing inherited.
unset GITHUB_REPOSITORY GITHUB_SHA GITHUB_REF GITHUB_RUN_ID GITHUB_RUN_ATTEMPT \
      GITHUB_SERVER_URL GITHUB_ACTIONS PARIDYUM_OPERATOR 2>/dev/null

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
cd "$WORK" || exit 1

# Identity every case shares, so that what a case is actually testing is the only thing
# that varies. A real 40-char SHA: the schema rejects an abbreviation and one case proves it.
ID=(--repository Paridyum/console-docket
    --commit 97f05b0e1c2d3f4a5b6c7d8e9f0a1b2c3d4e5f60
    --ref refs/heads/main)

run() { python3 "$CONV" "$@" 2>&1; }

# expect_ok <label> <cmd...>
expect_ok() {
  local label="$1"; shift
  local out; out="$("$@" 2>&1)"; local rc=$?
  [[ "$VERBOSE" == true ]] && printf '%s\n' "$out"
  if [[ $rc -eq 0 ]]; then ok "$label"; else bad "$label — expected exit 0, got $rc" "$out"; fi
}
# expect_refused <label> <pattern> <cmd...> — must exit 1 AND say why
expect_refused() {
  local label="$1" pat="$2"; shift 2
  local out; out="$("$@" 2>&1)"; local rc=$?
  [[ "$VERBOSE" == true ]] && printf '%s\n' "$out"
  if [[ $rc -ne 1 ]]; then
    bad "$label — the planted defect was NOT refused (exit $rc)" "$out"
  elif ! grep -qF "$pat" <<<"$out"; then
    bad "$label — refused, but nothing in the message matched '$pat'" "$out"
  else ok "$label"; fi
}
# expect_warns <label> <pattern> <cmd...> — must exit 0 and warn
expect_warns() {
  local label="$1" pat="$2"; shift 2
  local out; out="$("$@" 2>&1)"; local rc=$?
  [[ "$VERBOSE" == true ]] && printf '%s\n' "$out"
  if [[ $rc -ne 0 ]]; then bad "$label — expected exit 0, got $rc" "$out"
  elif ! grep -qF "$pat" <<<"$out"; then bad "$label — no warning matched '$pat'" "$out"
  else ok "$label"; fi
}
# expect_field <label> <file> <jq-ish dotted path> <expected>
expect_field() {
  local label="$1" file="$2" path="$3" want="$4"
  local got
  got="$(python3 - "$file" "$path" <<'PY' 2>&1
import json, sys
doc = json.load(open(sys.argv[1]))
for key in sys.argv[2].split("."):
    doc = doc[int(key)] if key.isdigit() else doc[key]
print(json.dumps(doc, sort_keys=True))
PY
)"
  if [[ "$got" == "$want" ]]; then ok "$label"
  else bad "$label — $path is $got, expected $want"; fi
}
# expect_absent <label> <file>
expect_absent() {
  # An explicit if, not `A && bad || ok`: with that form, `bad` returning
  # non-zero also runs `ok`, so a failing case would report both (SC2015).
  if [[ -e "$2" ]]; then bad "$1 — $2 was written anyway"; else ok "$1"; fi
}

# ── fixtures ─────────────────────────────────────────────────────────────────
# Written by hand rather than generated, so each one isolates exactly one shape of
# JUnit the converter has to read: the distinction between <failure> and <error>, a
# <skipped> with and without a message, a timestamp with no zone, and no timing at all.
cat > pass.xml <<'X'
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="unit" tests="3" failures="0" errors="0" time="1.5">
  <testsuite name="a" timestamp="2026-08-21T14:27:12.500Z" tests="3" time="1.5">
    <testcase classname="a" name="one" time="0.5"/>
    <testcase classname="a" name="two" time="0.5"/>
    <testcase classname="a" name="three" time="0.5"/>
  </testsuite>
</testsuites>
X
cat > fail.xml <<'X'
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="unit" tests="3" time="2.0">
  <testsuite name="b" timestamp="2026-08-21T14:30:00Z" tests="3" time="2.0">
    <testcase classname="b" name="one" time="0.5"/>
    <testcase classname="b" name="two" time="0.5"><failure message="expected 1 to be 2"/></testcase>
    <testcase classname="b" name="three" time="1.0"/>
  </testsuite>
</testsuites>
X
cat > error.xml <<'X'
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="unit" tests="3" time="0.75">
  <testsuite name="c" timestamp="2026-08-21T14:31:00Z" tests="3" time="0.75">
    <testcase classname="c" name="one" time="0.25"/>
    <testcase classname="c" name="two" time="0.25"><error message="pdflatex not on PATH"/></testcase>
    <testcase classname="c" name="three" time="0.25"><failure message="expected 1 to be 2"/></testcase>
  </testsuite>
</testsuites>
X
cat > skip-reason.xml <<'X'
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="e2e" tests="2" time="0.01">
  <testsuite name="d" timestamp="2026-08-21T14:32:00Z" tests="2" time="0.01">
    <testcase classname="d" name="issues a document"><skipped message="no typesetter on this runner (tool-docket D-35)"/></testcase>
    <testcase classname="d" name="signs a document"><skipped message="no typesetter on this runner (tool-docket D-35)"/></testcase>
  </testsuite>
</testsuites>
X
cat > skip-silent.xml <<'X'
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="e2e" tests="2" time="0.01">
  <testsuite name="d" timestamp="2026-08-21T14:32:00Z" tests="2" time="0.01">
    <testcase classname="d" name="issues a document"><skipped/></testcase>
    <testcase classname="d" name="signs a document"><skipped/></testcase>
  </testsuite>
</testsuites>
X
cat > partial-skip.xml <<'X'
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="unit" tests="2" time="0.4">
  <testsuite name="p" timestamp="2026-08-21T14:33:00Z" tests="2" time="0.4">
    <testcase classname="p" name="one" time="0.2"/>
    <testcase classname="p" name="two" time="0.2"><skipped/></testcase>
  </testsuite>
</testsuites>
X
cat > naive.xml <<'X'
<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="n" timestamp="2026-08-21T14:34:00" tests="1" time="0.1">
  <testcase classname="n" name="one" time="0.1"/>
</testsuite>
X
cat > untimed.xml <<'X'
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="unit">
  <testsuite name="u" tests="1">
    <testcase classname="u" name="one"/>
  </testsuite>
</testsuites>
X
cat > miscounted.xml <<'X'
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="unit" tests="99" time="0.2">
  <testsuite name="m" timestamp="2026-08-21T14:35:00Z" tests="99" time="0.2">
    <testcase classname="m" name="one" time="0.2"/>
  </testsuite>
</testsuites>
X
cat > empty.xml <<'X'
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="unit" tests="0" time="0.0"><testsuite name="z" timestamp="2026-08-21T14:36:00Z" tests="0" time="0.0"/></testsuites>
X
printf 'not xml at all' > junk.xml

L=(--tier pr --stack typescript --implements TST-0001 --provenance local --submitted-by rohan)

printf '\n── outcomes: the four JUnit shapes ────────────────────────────────\n'
expect_ok "a passing suite converts" python3 "$CONV" "${ID[@]}" "${L[@]}" --out e-pass.json pass.xml
expect_field "  outcome passed"      e-pass.json outcome '"passed"'
expect_field "  totals computed from the parse, not the tests= attribute" e-pass.json totals \
  '{"errored": 0, "failed": 0, "passed": 3, "skipped": 0, "tests": 3}'

expect_ok "a failing suite converts" python3 "$CONV" "${ID[@]}" "${L[@]}" --out e-fail.json fail.xml
expect_field "  outcome failed"     e-fail.json outcome '"failed"'
expect_field "  one failure, no errors" e-fail.json totals \
  '{"errored": 0, "failed": 1, "passed": 2, "skipped": 0, "tests": 3}'

# The point of requirement 3: <error> and <failure> in the SAME file must stay two
# numbers, and the outcome must be the one that says the harness broke.
expect_ok "an errored suite converts" python3 "$CONV" "${ID[@]}" "${L[@]}" --out e-err.json error.xml
expect_field "  outcome errored, not failed"  e-err.json outcome '"errored"'
expect_field "  errored and failed both kept, not collapsed" e-err.json totals \
  '{"errored": 1, "failed": 1, "passed": 1, "skipped": 0, "tests": 3}'

expect_ok "a wholly skipped suite converts when the skips carry a reason" \
  python3 "$CONV" "${ID[@]}" "${L[@]}" --out e-skip.json skip-reason.xml
expect_field "  outcome skipped"    e-skip.json outcome '"skipped"'
expect_field "  the JUnit skip message is carried verbatim for a person" e-skip.json skipped_reason \
  '"no typesetter on this runner (tool-docket D-35)"'
expect_field "  a passing suite leaves skipped_reason null" e-pass.json skipped_reason 'null'

printf '\n── a skip with no reason is a defect (requirement 4) ───────────────\n'
expect_refused "a wholly skipped suite with no <skipped message> is refused" \
  "pass --skipped-reason" python3 "$CONV" "${ID[@]}" "${L[@]}" --out e-nos.json skip-silent.xml
expect_absent  "  and no envelope is written" e-nos.json
expect_ok "the same suite converts once a person states the reason" \
  python3 "$CONV" "${ID[@]}" "${L[@]}" --out e-nos2.json --skipped-reason "quarantined pending RFC-0034 step 4" skip-silent.xml
expect_warns "a reasonless skip inside a passing suite still warns" \
  "reads as a pass downstream" python3 "$CONV" "${ID[@]}" "${L[@]}" --out e-part.json partial-skip.xml
expect_field "  and the outcome stays passed" e-part.json outcome '"passed"'
expect_refused "a JUnit with no <testcase> at all is refused rather than reported as a pass" \
  "no <testcase> in any input" python3 "$CONV" "${ID[@]}" "${L[@]}" --out e-empty.json empty.xml

printf '\n── merging several JUnit files into one envelope ───────────────────\n'
expect_ok "three files merge" python3 "$CONV" "${ID[@]}" "${L[@]}" --out e-merge.json pass.xml fail.xml error.xml
expect_field "  totals are the sum across all three" e-merge.json totals \
  '{"errored": 1, "failed": 2, "passed": 6, "skipped": 0, "tests": 9}'
expect_field "  the worst outcome across the three wins" e-merge.json outcome '"errored"'
expect_field "  every input is referenced as an artifact" e-merge.json artifacts.2.kind '"junit"'
expect_field "  started_at is the earliest of them" e-merge.json started_at '"2026-08-21T14:27:12Z"'
expect_field "  duration is their sum in ms" e-merge.json duration_ms '4250'

printf '\n── the sha256 of each artifact is the real digest ──────────────────\n'
WANT="$( (shasum -a 256 pass.xml 2>/dev/null || sha256sum pass.xml) | cut -d' ' -f1)"
expect_field "artifacts[0].sha256 matches the file on disk" e-pass.json artifacts.0.sha256 "\"$WANT\""

printf '\n── started_at is RFC 3339 UTC Z, or refused (requirement 5) ────────\n'
expect_field "a fractional Z timestamp is truncated to whole seconds" e-pass.json started_at '"2026-08-21T14:27:12Z"'
expect_warns "a timestamp with no zone is announced, not silently accepted" \
  "states no time zone" python3 "$CONV" "${ID[@]}" "${L[@]}" --out e-naive.json naive.xml
expect_refused "a JUnit with no timestamp anywhere is refused rather than stamped with now()" \
  "an input this converter does not have" python3 "$CONV" "${ID[@]}" "${L[@]}" --out e-nots.json --duration-ms 10 untimed.xml
expect_absent "  and no envelope is written" e-nots.json
expect_warns "--started-at-from-mtime substitutes the file's mtime and says what it substituted" \
  "when the report was written rather than when the run began" \
  python3 "$CONV" "${ID[@]}" "${L[@]}" --out e-mtime.json --duration-ms 10 --started-at-from-mtime untimed.xml
expect_refused "a JUnit with no time attribute is refused rather than given duration 0" \
  "0 would be a claim that the run took no time" \
  python3 "$CONV" "${ID[@]}" "${L[@]}" --out e-nodur.json --started-at 2026-08-21T00:00:00Z untimed.xml

printf '\n── implements is a claim, never an omission (requirement 2) ────────\n'
expect_warns "no --implements writes [] and warns that nothing declared is covered" \
  "implements NO declared test type" \
  python3 "$CONV" "${ID[@]}" --tier pr --stack typescript --provenance local --submitted-by rohan --out e-noimp.json pass.xml
expect_field "  implements is [] and not absent" e-noimp.json implements '[]'
expect_ok "comma-separated and repeated --implements both work" \
  python3 "$CONV" "${ID[@]}" --tier pr --stack typescript --provenance local --submitted-by rohan \
  --implements TST-0001,TST-0002 --implements TST-0003 --out e-imp.json pass.xml
expect_field "  and are deduplicated in order" e-imp.json implements '["TST-0001", "TST-0002", "TST-0003"]'
expect_refused "a TST- id absent from test-registry.yaml is refused" \
  "names types absent from" python3 "$CONV" "${ID[@]}" --tier pr --stack typescript \
  --provenance local --submitted-by rohan --implements TST-9999 --out e-badimp.json pass.xml

printf '\n── provenance ─────────────────────────────────────────────────────\n'
expect_refused "provenance local with no operator is refused (STD-0014 §provenance)" \
  "an anonymous claim" python3 "$CONV" "${ID[@]}" --tier pr --stack typescript \
  --implements TST-0001 --provenance local --out e-anon.json pass.xml
expect_absent "  and no unattributed envelope is written" e-anon.json
expect_ok "PARIDYUM_OPERATOR attributes it when no flag does" \
  env PARIDYUM_OPERATOR=rohan python3 "$CONV" "${ID[@]}" --tier pr --stack typescript \
  --implements TST-0001 --provenance local --out e-envop.json pass.xml
expect_field "  submitted_by comes from the environment" e-envop.json submitted_by '"rohan"'
expect_refused "provenance ci with no run id is refused" \
  "GITHUB_RUN_ID is unset" python3 "$CONV" "${ID[@]}" --tier pr --stack typescript \
  --implements TST-0001 --provenance ci --out e-norun.json pass.xml

# Actions env only, no flags: the shape the reusable workflows will call it in.
expect_ok "the GitHub Actions environment alone is enough" \
  env GITHUB_REPOSITORY=Paridyum/console-docket \
      GITHUB_SHA=97f05b0e1c2d3f4a5b6c7d8e9f0a1b2c3d4e5f60 \
      GITHUB_REF=refs/pull/12/merge GITHUB_RUN_ID=32492213670 GITHUB_RUN_ATTEMPT=2 \
      GITHUB_SERVER_URL=https://github.com \
      python3 "$CONV" --tier pr --stack typescript --implements TST-0001 --out e-ci.json pass.xml
expect_field "  provenance defaults to ci when a run identifies itself" e-ci.json provenance '"ci"'
expect_field "  the run url is built from the server, repository, run and attempt" e-ci.json run \
  '{"attempt": 2, "id": "32492213670", "url": "https://github.com/Paridyum/console-docket/actions/runs/32492213670/attempts/2"}'
expect_field "  refs/pull/N/merge is kept distinguishable from a branch" e-ci.json ref '"refs/pull/12/merge"'
expect_field "  no submitted_by on a ci envelope" e-ci.json 'implements' '["TST-0001"]'
if grep -q submitted_by e-ci.json; then bad "a ci envelope must not carry submitted_by"; else ok "a ci envelope carries no submitted_by"; fi

printf '\n── the schema layer is live ────────────────────────────────────────\n'
expect_refused "an abbreviated commit is refused by the schema" \
  "does not match" python3 "$CONV" --repository Paridyum/console-docket --commit 97f05b0 \
  --ref refs/heads/main "${L[@]}" --out e-abbrev.json pass.xml
expect_refused "a tier outside STD-0008 is refused by the schema" \
  "is not one of" python3 "$CONV" "${ID[@]}" --tier weekly --stack typescript \
  --implements TST-0001 --provenance local --submitted-by rohan --out e-tier.json pass.xml
expect_refused "a stack outside class-registry.yaml is refused by the schema" \
  "is not one of" python3 "$CONV" "${ID[@]}" --tier pr --stack cobol \
  --implements TST-0001 --provenance local --submitted-by rohan --out e-stack.json pass.xml
expect_refused "an artifact that does not exist is refused rather than digested" \
  "does not exist, so it has no digest" python3 "$CONV" "${ID[@]}" "${L[@]}" \
  --artifact coverage:nope.info --out e-noart.json pass.xml
expect_refused "a file that is not JUnit is refused" \
  "not parseable XML" python3 "$CONV" "${ID[@]}" "${L[@]}" --out e-junk.json junk.xml
expect_refused "the same JUnit passed twice is refused, not counted twice" \
  "would be counted twice" python3 "$CONV" "${ID[@]}" "${L[@]}" --out e-dup.json pass.xml ./pass.xml
expect_refused "a non-numeric run attempt is refused" \
  "is not a number" python3 "$CONV" "${ID[@]}" --tier pr --stack typescript --implements TST-0001 \
  --provenance ci --run-id 1 --run-attempt latest --out e-att.json pass.xml

printf '\n── the file'"'"'s own tests= attribute is never trusted ──────────────\n'
expect_warns "a file whose tests= disagrees with its <testcase> count says so" \
  "declares tests=99 but carries 1" python3 "$CONV" "${ID[@]}" "${L[@]}" --out e-mis.json miscounted.xml
expect_field "  and the counted value is what is written" e-mis.json totals \
  '{"errored": 0, "failed": 0, "passed": 1, "skipped": 0, "tests": 1}'

printf '\n── the two rules the schema cannot enforce (STD-0014) ──────────────\n'
# Reached by corrupting the envelope between build and write, because no CLI input can
# produce them. Each case asserts three things: the vendored validator PASSES the
# corrupt envelope, the hand-written check catches it, and main() exits 1 with nothing
# on disk — i.e. the check is on the write path and not merely available.
cat > corrupt.py <<'PY'
"""Corrupt the envelope between build and write, then let the converter try to write it.

Proves the claim STD-0014 makes rather than restating it: for each of these two
defects, `validate-schema.py` against test-envelope.schema.json reports NOTHING, and
the converter refuses anyway.
"""
import importlib.util, json, sys
from pathlib import Path

conv, mutation, out = sys.argv[1], sys.argv[2], sys.argv[3]
spec = importlib.util.spec_from_file_location("conv", conv)
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)

MUTATIONS = {
    # Rule 1: JSON Schema has no arithmetic at any version.
    "totals": lambda e: e["totals"].__setitem__("passed", e["totals"]["passed"] + 1),
    # Rule 2: expressed in the schema with allOf/if/then, which the vendored validator
    # announces it does not implement.
    "drop-run": lambda e: e.pop("run", None),
    "drop-submitted-by": lambda e: e.pop("submitted_by", None),
}

real_build = mod.build
def build(args):
    env = real_build(args)
    MUTATIONS[mutation](env)
    schema = json.loads(Path(args.schema).read_text())
    validator = mod.load_validator(Path(args.validator))
    schema_errs = validator.validate(json.loads(json.dumps(env)), schema, "$")
    cross_errs = mod.cross_field_errors(json.loads(json.dumps(env)))
    print(f"SCHEMA_ERRORS={len(schema_errs)} CROSS_ERRORS={len(cross_errs)}")
    for e in cross_errs:
        print(f"CROSS: {e}")
    return env
mod.build = build

sys.exit(mod.main(sys.argv[4:] + ["--out", out]))
PY

# expect_corruption_refused <label> <mutation> <out> <converter args...>
expect_corruption_refused() {
  local label="$1" mut="$2" out="$3"; shift 3
  local o; o="$(python3 corrupt.py "$CONV" "$mut" "$out" "$@" 2>&1)"; local rc=$?
  [[ "$VERBOSE" == true ]] && printf '%s\n' "$o"
  if ! grep -q 'SCHEMA_ERRORS=0' <<<"$o"; then
    bad "$label — the schema was expected to PASS the corrupt envelope; it did not" "$o"
  elif grep -q 'CROSS_ERRORS=0' <<<"$o"; then
    bad "$label — the hand-written check did not catch it" "$o"
  elif [[ $rc -ne 1 ]]; then
    bad "$label — caught, but the converter still exited $rc" "$o"
  elif [[ -e "$out" ]]; then
    bad "$label — caught, but the envelope was written anyway" "$o"
  else ok "$label"; fi
}

expect_corruption_refused \
  "rule 1: totals not summing to tests — schema passes it, the converter refuses" \
  totals c-totals.json "${ID[@]}" "${L[@]}" pass.xml
expect_corruption_refused \
  "rule 2a: provenance ci with no run — schema passes it, the converter refuses" \
  drop-run c-run.json --repository Paridyum/console-docket \
  --commit 97f05b0e1c2d3f4a5b6c7d8e9f0a1b2c3d4e5f60 --ref refs/heads/main \
  --tier pr --stack typescript --implements TST-0001 --provenance ci --run-id 1 pass.xml
expect_corruption_refused \
  "rule 2b: provenance local with no submitted_by — schema passes it, the converter refuses" \
  drop-submitted-by c-sub.json "${ID[@]}" "${L[@]}" pass.xml

# The validator must keep SAYING it does not enforce allOf. If it ever stops, the
# reasoning above is stale and the next reader should be told, not left to assume.
# Captured rather than piped: `grep -q` exits on the first match, and under `pipefail`
# the SIGPIPE it delivers to python becomes the pipeline's status, so a piped form of
# this check fails on a match. Found here, which is the point of the suite.
NOTE_OUT="$(python3 "$CONV" "${ID[@]}" "${L[@]}" --out /dev/null pass.xml 2>&1)"
if grep -qF 'unsupported keyword, not enforced — <root>: allOf' <<<"$NOTE_OUT"; then
  ok "the validator still announces that it does not enforce allOf (STD-0014 quotes this)"
else
  bad "the validator no longer announces the allOf gap — STD-0014's reasoning needs rechecking" "$NOTE_OUT"
fi

printf '\n── real JUnit from this estate, not fixtures ───────────────────────\n'
# Requirement: at least one real file. Two are used, and they exercise opposite paths.
# A REAL Vitest file, COMMITTED — not produced by running a sibling repository.
#
# This read `CD=/Users/rohan/Developer/paridyum/consoles/console-docket` until 2026-09-20: an
# absolute path into one developer's home directory, in a blocking-tier suite inside a template
# every repository vendors. It passed on that one machine and failed everywhere else — including
# CI, where it had been red on knowledge#1 with the cause hidden by `expect_ok`'s `head -4`.
#
# The requirement above — "at least one real file" — is satisfied better by committing one: the
# file IS real output from console-docket's Vitest (158 cases, captured 2026-09-20), and now it
# is the same real file on every machine rather than whatever that machine happens to produce.
# VITEST_JUNIT still overrides, for anyone regenerating it deliberately.
VITEST="${VITEST_JUNIT:-$ROOT/tests/fixtures/junit/console-docket-vitest.xml}"
if [[ -f "$VITEST" ]]; then
  CASES="$(grep -c '<testcase' "$VITEST")"
  expect_ok "console-docket's real Vitest JUnit converts" \
    env -u GITHUB_RUN_ID python3 "$CONV" --tier pr --stack typescript --implements TST-0001 \
    --provenance local --submitted-by rohan --repository Paridyum/console-docket \
    --commit 97f05b0e1c2d3f4a5b6c7d8e9f0a1b2c3d4e5f60 --ref refs/heads/main \
    --out e-real-vitest.json "$VITEST"
  expect_field "  totals.tests equals the <testcase> count in the real file ($CASES)" \
    e-real-vitest.json totals.tests "$CASES"
else
  bad "the committed real Vitest fixture is missing: $VITEST — the real-file case did not run"
fi

# This repository's own gate suite writes JUnit (STD-0008), and it is the harder input:
# no timestamp and no time attribute anywhere, and real failures in it.
# COMMITTED, for the same reason as the Vitest file above. `$ROOT/junit.xml` is in knowledge's
# .gitignore (line 46), so it exists on a machine that has run the gate suite and NEVER in a
# fresh checkout — which is exactly where this test needs to pass. It was the second of the two
# cases failing CI while passing locally.
#
# It is still real output from this repository's own gate suite, with the property that makes it
# the harder input: 190 cases and no timestamp or time attribute anywhere.
#
# THIS CAPTURE HAS NO FAILURES IN IT, and the paragraph above the original line claimed it did —
# "and real failures in it". Checked rather than copied: `grep -c '<failure'` returns 0. A second
# fixture carrying real failures is worth adding when one is captured; asserting the property
# here without it would be the documentation-versus-reality gap this whole suite exists to close.
GATES="${GATES_JUNIT:-$ROOT/tests/fixtures/junit/knowledge-gate-suite.xml}"
if [[ -f "$GATES" ]]; then
  GCASES="$(grep -c '<testcase' "$GATES")"
  GFAILS="$(grep -c '<failure' "$GATES")"
  expect_ok "knowledge's own gate-suite JUnit converts, with the missing inputs supplied explicitly" \
    python3 "$CONV" --tier pr --stack none --implements TST-0001 --provenance local \
    --submitted-by rohan --repository Paridyum/knowledge \
    --commit 97f05b0e1c2d3f4a5b6c7d8e9f0a1b2c3d4e5f60 --ref refs/heads/main \
    --started-at-from-mtime --duration-ms 0 --out e-real-gates.json "$GATES"
  expect_field "  totals.tests equals its <testcase> count ($GCASES)" e-real-gates.json totals.tests "$GCASES"
  expect_field "  totals.failed equals its <failure> count ($GFAILS)" e-real-gates.json totals.failed "$GFAILS"
  # Derived from the file, not hardcoded.
  #
  # This asserted `"failed"` unconditionally, which made the case pass only while
  # this repository's own gate suite was RED — it failed the moment everything was
  # healthy, which is exactly backwards. It survived because junit.xml happened to
  # carry two real failures when the suite was written, and broke as soon as
  # test-checks.sh regenerated a clean one.
  #
  # The lesson generalises past this line: a fixture that is a live build artifact is
  # not a fixture. The counts above are read from the file for the same reason; the
  # outcome has to be read the same way.
  if [[ "$GFAILS" -gt 0 ]]; then GOUTCOME='"failed"'; else GOUTCOME='"passed"'; fi
  expect_field "  outcome matches its content ($GOUTCOME)" e-real-gates.json outcome "$GOUTCOME"
else
  bad "$ROOT/junit.xml is absent — the untimed real-file case did not run"
fi

printf '\n───────────────────────────────────────────────────────────────────\n'
printf '%d passed, %d failed\n' "$PASS" "$FAIL"
[[ $FAIL -eq 0 ]] || exit 1
