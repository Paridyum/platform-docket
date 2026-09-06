#!/usr/bin/env bash
#
# Open a record for each of Paridyum's four officers, then issue what each owes.
#
# ## What this does and deliberately does not do
#
# It runs in TWO PASSES, and the pause between them is the point.
#
# Pass one creates four records from `people/_TEMPLATE-officer.yaml` and stops. It
# does NOT invent a date of birth, a phone number or an address. The template ships
# `FILL-ME` placeholders because it used to ship realistic example values, and two
# freshly enrolled people displayed as the same person with nothing flagging it — a
# plausible placeholder cannot be told apart from real data by anything except
# somebody noticing. A bootstrap that filled them in to look complete would be that
# defect, deliberately.
#
# Pass two — after a human has filled the records in — runs `docket check` and then
# issues. `docket issue` refuses a record still holding a placeholder, so pass two
# cannot silently succeed on an unfilled record.
#
# ## Who signs what
#
# Officer documents route to the Director (RFC-0042 decision 4). The Director's OWN
# route to himself, so `docket` substitutes the first active signatory who is not the
# subject — in practice the CEO — and records who actually signed on the register
# entry. Nothing here needs to know that; it is stated so the output is not a
# surprise.
#
# ## Usage
#
#   export DOCKET_OPERATOR=rohan
#   scripts/bootstrap-officers.sh          # pass one: create records, then stop
#   $EDITOR people/*.yaml                  # fill every FILL-ME
#   scripts/bootstrap-officers.sh --issue  # pass two: check, then issue
set -euo pipefail

cd "$(dirname "$0")/.."

if [[ -z "${DOCKET_OPERATOR:-}" ]]; then
  echo "DOCKET_OPERATOR is unset. Every write records who made it; there is no default." >&2
  echo "  export DOCKET_OPERATOR=rohan" >&2
  exit 2
fi

# slug:office. The offices are the ones signatories.yaml declares; `docket team`
# joins on the same ids, so the page and the documents cannot disagree.
OFFICERS=(
  "ruhani-khurana:Chief Executive Officer"
  "rohan-batra:Chief Technology Officer"
  "aditya-shrivastava:Chief Operations Officer"
  "priyanshu-sharma:Director"
)

docket() { cargo run --quiet -- "$@"; }

if [[ "${1:-}" != "--issue" ]]; then
  echo "pass one: opening records"
  for entry in "${OFFICERS[@]}"; do
    slug="${entry%%:*}"
    office="${entry#*:}"
    if [[ -f "people/${slug}.yaml" ]]; then
      echo "  ${slug}  already exists, left alone"
      continue
    fi
    docket new "${slug}" officer
    echo "  ${slug}  created — office: ${office}"
  done

  cat <<'NEXT'

pass one done. Nothing has been issued and nothing is complete.

Now fill in every FILL-ME in people/*.yaml by hand. This script will not do it:
a date of birth or an address invented to make a record look finished is the one
defect people/_TEMPLATE.yaml exists to prevent.

  `role` is the office held, matching signatories.yaml
  `start_date` is the date the appointment took effect
  no `end_date` — an office is held until it is vacated

Then:

  scripts/bootstrap-officers.sh --issue
NEXT
  exit 0
fi

echo "pass two: checking records before issuing"
if ! docket check; then
  echo >&2
  echo "check failed. Nothing has been issued. Fix the records and run again." >&2
  exit 1
fi

echo
echo "pass two: issuing"
for entry in "${OFFICERS[@]}"; do
  slug="${entry%%:*}"
  echo "  ${slug}"
  docket onboard "${slug}"
done

echo
echo "issued. What each officer owes and holds:"
for entry in "${OFFICERS[@]}"; do
  docket status "${entry%%:*}"
done

cat <<'AFTER'

Next, in order:

  1. docket pack <slug>            one folder per officer, to hand over
  2. get them signed, then:
     docket sign <serial> --method wet-ink --on <date> --scan <file>
  3. docket team > ../../web/web-paridyum/team.json
     then commit that file in web-paridyum — the public page is a reviewed diff,
     not a build-time read of this repository.

Still outstanding, and neither is a docket problem:
  - Aditya Shrivastava's appointing instrument. signatories.yaml carries
    `appointed_on: null` and `instrument: null`, and C-0004 wants a human and a
    dated resolution. Check the legal-name spelling against it.
  - Counsel on RFC-0042 §3 at incorporation.
AFTER
