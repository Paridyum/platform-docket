#!/usr/bin/env bash
#
# Which declared test types actually RUN, and where. A report, not a gate.
#
# WHY THIS EXISTS. test-registry.yaml declares the test-type vocabulary (ADR-0029) and
# STD-0008 schedules those types into tiers. Neither answers the only question an operator
# asks about a testing taxonomy: of the types this estate says are REQUIRED, which ones
# execute somewhere, and where. Nothing could answer it before this script — the mapping
# from "TST-0003 integration" to "a job in some repository's pipeline" existed only in a
# person's head, so a type could sit `required` for months while no pipeline ran it and
# every gate stayed green. That is this estate's recurring defect in its purest form: a
# claim nothing compares against reality.
#
# WHY IT IS A REPORT AND NOT A CHECK. A check must be right. This cannot be: there is no
# declared mapping from a test type to the job that runs it (repository manifests carry no
# test-type field, and only two sites in the whole estate name a TST- id at all). So the
# honest form is a report that states, per row, WHAT it looked at and how strong that
# evidence is — and that says `cannot determine` rather than guessing. It deliberately does
# not add a check id: a check whose name promises more than its evidence supports is the
# defect this repository keeps finding, and turning this into one would commit it.
#
#   ./report-test-coverage.sh                 the whole registry
#   ./report-test-coverage.sh --only required  required types only
#   ./report-test-coverage.sh --tsv            one row per type, machine-readable
#   ./report-test-coverage.sh --estate DIR     scan a different estate root
#
# Exit 0 when the report was produced, whatever it says — a type not running is a finding,
# not a failure of this script. Exit 2 when the report could NOT be produced (no python3,
# no PyYAML, no registry), matching scripts/lib/validate-schema.py: a report that could not
# be built must never look like a clean one.

set -uo pipefail

# ADR-0006 names macOS a supported workstation and macOS ships bash 3.2. This wrapper stays
# inside bash 3.2's feature set on purpose so the diagnosis below is the only bash-version
# message anyone sees from this script; the work happens in python3.

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
LIB="$ROOT/scripts/lib/report-test-coverage.py"

if ! command -v python3 >/dev/null 2>&1; then
  printf 'report-test-coverage: python3 not found; cannot build the report\n' >&2
  printf '  Reported as an inability, not skipped: an empty report reads as "nothing runs".\n' >&2
  exit 2
fi

if [ ! -f "$LIB" ]; then
  printf 'report-test-coverage: missing %s\n' "$LIB" >&2
  exit 2
fi

# --root is passed rather than inferred inside the library so the library never has to guess
# where the knowledge repository is from its own __file__ — the same reason check-knowledge.sh
# resolves ROOT once and cds to it.
exec python3 "$LIB" --root "$ROOT" "$@"
