#!/usr/bin/env bash
#
# Migration gate — the ADR-0024 nested-tenancy checks.
#
# ADR-0024's failure mode is silent and unrecoverable at two boundaries, and until this
# script existed its five guards were "declared": named in frontmatter, enforced by nobody.
# That is the gap ADR-0002 exists to close and the one README.md calls the first engineering
# task.
#
# NO MIGRATION EXISTS YET, and that is why this script reports `n/a` rather than success when
# it finds nothing to inspect. A gate that prints "passed" over an empty input set is the
# defect class the ADR-0014 audit found four separate times. `n/a` is not a pass.
#
#   ./check-migrations.sh                 inspect every */migrations/*.sql under the repo root
#   ./check-migrations.sh --root DIR      inspect DIR instead (used by the fixture suite)
#   ./check-migrations.sh --expect-fail   exit 0 only if at least one error was found
#
# THE DECLARATION SYNTAX BELOW IS NOT YET RATIFIED. ADR-0024 decision 1 requires each table to
# declare one of three categories and deliberately does not fix the syntax. This script chooses
# a line comment immediately above the CREATE TABLE:
#
#   -- paridyum:scope platform | tenant | account
#   -- paridyum:plane control | regional
#
# That convention binds every future migration, so it belongs in a standard rather than in a
# script. Recorded as an open question in the check registry; if it changes, only the parser
# here and the fixtures change with it.
#
# THE SECOND DIRECTIVE IS NEWER AND EVEN LESS RATIFIED. RFC-0014 decision 3 adds it: once
# tenancy spans more than one database, `scope` says WHOSE a row is and `plane` says WHERE the
# table lives. Neither answers the other, and a pii column landing in the control plane instead
# of a region is a residency breach that every existing check reports as green.

set -uo pipefail

# ADR-0006 names macOS a supported workstation and macOS ships bash 3.2, which has neither
# `mapfile` nor `declare -A`. This script uses both. Without this guard it dies with
# `unbound variable` and no usable diagnosis -- which reads as a broken repository rather
# than a missing dependency, and is exactly the "green check that proves nothing" failure
# inverted: a red one that explains nothing. Found 2026-08-08 by running the gate under
# /bin/bash rather than the Homebrew bash that happened to be on PATH.
if [[ -z "${BASH_VERSINFO:-}" ]] || (( BASH_VERSINFO[0] < 4 )); then
  printf 'PARIDYUM GATE CANNOT RUN: needs bash 4+, found %s\n' "${BASH_VERSION:-unknown}" >&2
  printf '  macOS ships bash 3.2 (ADR-0006 supports macOS). Install a modern one:\n' >&2
  printf '    brew install bash   # then re-run, or invoke with: $(brew --prefix)/bin/bash %s\n' "$0" >&2
  printf '  Reported as a failure, not skipped: a gate that cannot run has not passed.\n' >&2
  exit 1
fi


ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
SCAN="$ROOT"
EXPECT_FAIL=false
while [[ $# -gt 0 ]]; do
  case "$1" in
    --root) SCAN="$2"; shift 2 ;;
    --expect-fail) EXPECT_FAIL=true; shift ;;
    *) shift ;;
  esac
done

if [[ -t 1 ]]; then
  RED=$'\033[31m'; GRN=$'\033[32m'; YLW=$'\033[33m'; RST=$'\033[0m'
else
  RED=""; GRN=""; YLW=""; RST=""
fi
ERRORS=0; WARNS=0
err() { printf '%s✘%s %s\n' "$RED" "$RST" "$1"; ERRORS=$((ERRORS+1)); }
wrn() { printf '%s!%s %s\n'  "$YLW" "$RST" "$1"; WARNS=$((WARNS+1)); }

# tests/fixtures/ is excluded, and the exclusion is load-bearing rather than tidiness: the
# fixture suite contains DELIBERATELY BROKEN migrations, so a default scan from the repository
# root found them and failed — in CI and in the pre-commit hook, on every commit. Caught by
# running the gate rather than by reading it, which is the whole argument for test-checks.sh.
# `--root` deliberately does NOT apply the exclusion, because that is how the suite reaches them.
PRUNE=()
[[ "$SCAN" == "$ROOT" ]] && PRUNE=(-not -path '*/tests/fixtures/*')
mapfile -t MIGRATIONS < <(find "$SCAN" -path '*/migrations/*.sql' -type f -not -path '*/.git/*' "${PRUNE[@]}" 2>/dev/null | sort)

if [[ ${#MIGRATIONS[@]} -eq 0 ]]; then
  printf '%sn/a%s no migrations found under %s — five ADR-0024 checks have no input, so they are\n' "$YLW" "$RST" "${SCAN#"$ROOT"/}"
  printf '    neither passing nor failing. This is deliberately not reported as success.\n'
  exit 0
fi

printf 'Inspecting %d migration(s)\n\n' "${#MIGRATIONS[@]}"

# Strip comments except our own directives, and fold to one statement per line so the
# per-table checks below do not have to be multi-line parsers. A real parser is tool-forge's
# job; this catches the case that actually happens, which is a forgotten column.
normalise() {
  sed -E 's/[[:space:]]+$//' "$1"
}

declare -A TBL_SCOPE=() TBL_PLANE=() TBL_FILE=() TBL_COLS=() TBL_COLTYPE=()
declare -A RLS_ENABLED=() RLS_FORCED=() POLICY_BODY=()

for f in "${MIGRATIONS[@]}"; do
  rel="${f#"$SCAN"/}"
  pending_scope=""
  pending_plane=""
  cur=""
  while IFS= read -r line; do
    # our directives
    if [[ "$line" =~ ^[[:space:]]*--[[:space:]]*paridyum:scope[[:space:]]+([a-z]+) ]]; then
      pending_scope="${BASH_REMATCH[1]}"; continue
    fi
    if [[ "$line" =~ ^[[:space:]]*--[[:space:]]*paridyum:plane[[:space:]]+([a-z]+) ]]; then
      pending_plane="${BASH_REMATCH[1]}"; continue
    fi
    # CREATE TABLE
    if [[ "$line" =~ ^[[:space:]]*CREATE[[:space:]]+TABLE[[:space:]]+(IF[[:space:]]+NOT[[:space:]]+EXISTS[[:space:]]+)?([a-zA-Z_][a-zA-Z0-9_]*) ]]; then
      cur="${BASH_REMATCH[2]}"
      TBL_FILE[$cur]="$rel"
      TBL_SCOPE[$cur]="${pending_scope:-<undeclared>}"
      TBL_PLANE[$cur]="${pending_plane:-<undeclared>}"
      TBL_COLS[$cur]=""
      pending_scope=""
      pending_plane=""
      continue
    fi
    # end of a CREATE TABLE body
    if [[ -n "$cur" && "$line" =~ ^[[:space:]]*\)\;?[[:space:]]*$ ]]; then cur=""; continue; fi
    # column inside the current table. The type is captured as well as the name, because the
    # STD-0010 checks below are all about the TYPE a column was given — a locale in char(2), an
    # instant in a naive timestamp, an amount in double precision. The original parser kept
    # names only, which was sufficient for ADR-0024 and is not for these.
    if [[ -n "$cur" && "$line" =~ ^[[:space:]]+([a-zA-Z_][a-zA-Z0-9_]*)[[:space:]]+([^,]*) ]]; then
      col="${BASH_REMATCH[1]}"
      case "$col" in
        PRIMARY|FOREIGN|UNIQUE|CONSTRAINT|CHECK|EXCLUDE|LIKE) ;;
        *)
          TBL_COLS[$cur]="${TBL_COLS[$cur]} $col"
          # Lowercased, whitespace-collapsed, trailing comma stripped. Constraint keywords
          # stay in the string: `text NOT NULL` matching on `text` is what the checks want,
          # and stripping them would need a type grammar this parser deliberately does not have.
          rest="$(tr '[:upper:]' '[:lower:]' <<<"${BASH_REMATCH[2]}" | tr -s '[:space:]' ' ' | sed -E 's/,[[:space:]]*$//; s/[[:space:]]+$//')"
          TBL_COLTYPE["$cur.$col"]="$rest"
          ;;
      esac
      continue
    fi
    # RLS statements
    if [[ "$line" =~ ALTER[[:space:]]+TABLE[[:space:]]+([a-zA-Z_][a-zA-Z0-9_]*)[[:space:]]+ENABLE[[:space:]]+ROW[[:space:]]+LEVEL[[:space:]]+SECURITY ]]; then
      RLS_ENABLED[${BASH_REMATCH[1]}]=1; continue
    fi
    if [[ "$line" =~ ALTER[[:space:]]+TABLE[[:space:]]+([a-zA-Z_][a-zA-Z0-9_]*)[[:space:]]+FORCE[[:space:]]+ROW[[:space:]]+LEVEL[[:space:]]+SECURITY ]]; then
      RLS_FORCED[${BASH_REMATCH[1]}]=1; continue
    fi
    if [[ "$line" =~ CREATE[[:space:]]+POLICY[[:space:]]+[a-zA-Z_][a-zA-Z0-9_]*[[:space:]]+ON[[:space:]]+([a-zA-Z_][a-zA-Z0-9_]*) ]]; then
      POLICY_BODY[${BASH_REMATCH[1]}]="${POLICY_BODY[${BASH_REMATCH[1]}]:-} $line"; continue
    fi
    # A policy clause continuing onto its own line is NOT handled here. An earlier version tried
    # and could not: this pass is line-oriented with no memory of the last policy target, so the
    # branch that lived here referenced a variable nothing ever assigned and silently did nothing
    # for the life of the script. The second pass below carries that memory and does the work.
  done < <(normalise "$f")
done

# Track the most recent policy target across lines. Done in a second pass because the
# first pass is line-oriented and a policy body routinely spans several lines.
for f in "${MIGRATIONS[@]}"; do
  last=""
  while IFS= read -r line; do
    if [[ "$line" =~ CREATE[[:space:]]+POLICY[[:space:]]+[a-zA-Z_][a-zA-Z0-9_]*[[:space:]]+ON[[:space:]]+([a-zA-Z_][a-zA-Z0-9_]*) ]]; then
      last="${BASH_REMATCH[1]}"
    elif [[ -n "$last" ]]; then
      POLICY_BODY[$last]="${POLICY_BODY[$last]:-} $line"
      [[ "$line" =~ \;[[:space:]]*$ ]] && last=""
    fi
  done < <(normalise "$f")
done

# --- Re-declaration, for a retrofit the original CREATE TABLE cannot express -------------
#
# ADR-0024 decision 9 anticipates retrofitting an inner discriminator onto tables that already
# exist. The parser above reads `-- paridyum:scope` from the comment directly above a CREATE
# TABLE, and that comment lives in a migration which has already been applied — `migrate.py`
# records the sha256 of every applied `up` and refuses an edited one, because a migration is
# immutable once applied. So a retrofit could add the column and could NOT restate the category,
# and the table would fail `db.tenant_scoped_tables_declare_tenancy` for carrying an account_id
# its own declaration forbids.
#
# The directive below is the way out, and it is deliberately verbose — it names the table, so it
# reads as a correction to a specific earlier declaration rather than as a second opinion:
#
#   -- paridyum:rescope docket_persons account
#
# LAST DECLARATION WINS, in migration order, which is the same rule the schema itself follows.
# A rescope for a table that was never created is an error rather than a no-op: it is either a
# typo or a reference to a table someone removed, and both are worth reporting.
declare -A RESCOPE=()
for f in "${MIGRATIONS[@]}"; do
  rel="${f#"$SCAN"/}"
  while IFS= read -r line; do
    if [[ "$line" =~ ^[[:space:]]*--[[:space:]]*paridyum:rescope[[:space:]]+([a-zA-Z_][a-zA-Z0-9_]*)[[:space:]]+([a-z]+) ]]; then
      rt="${BASH_REMATCH[1]}"; rs="${BASH_REMATCH[2]}"
      case "$rs" in
        platform|tenant|account) ;;
        *) err "$rel: rescope of '$rt' names '$rs', which is not one of platform|tenant|account (ADR-0024)"; continue ;;
      esac
      [[ -n "${TBL_FILE[$rt]:-}" ]] \
        || { err "$rel: rescopes '$rt', which no migration creates (ADR-0024)"; continue; }
      RESCOPE[$rt]="$rs"
    fi
  done < <(normalise "$f")
done
for t in "${!RESCOPE[@]}"; do TBL_SCOPE[$t]="${RESCOPE[$t]}"; done

has_col() { [[ " ${TBL_COLS[$1]} " == *" $2 "* ]]; }

# A column added by a later ALTER is a column the table has. The parser above only sees columns
# inside a CREATE TABLE body, which was sufficient while every column arrived with its table —
# and is not, the moment a discriminator is retrofitted (ADR-0024 D9).
for f in "${MIGRATIONS[@]}"; do
  while IFS= read -r line; do
    if [[ "$line" =~ ALTER[[:space:]]+TABLE[[:space:]]+([a-zA-Z_][a-zA-Z0-9_]*)[[:space:]]+ADD[[:space:]]+COLUMN[[:space:]]+(IF[[:space:]]+NOT[[:space:]]+EXISTS[[:space:]]+)?([a-zA-Z_][a-zA-Z0-9_]*) ]]; then
      at="${BASH_REMATCH[1]}"; ac="${BASH_REMATCH[3]}"
      [[ -n "${TBL_FILE[$at]:-}" ]] && TBL_COLS[$at]="${TBL_COLS[$at]:-} $ac"
    fi
  done < <(normalise "$f")
done

# --- db.tenant_scoped_tables_declare_tenancy --------------------------------
# ADR-0024 decision 1. Exactly one of three categories, declared and never inferred:
# `invoices` is account-scoped, the `plans` a product offers is tenant-scoped, `regions` is
# platform-owned, and no naming rule separates them.
for t in "${!TBL_FILE[@]}"; do
  scope="${TBL_SCOPE[$t]}"
  case "$scope" in
    platform)
      has_col "$t" tenant_id && err "${TBL_FILE[$t]}: table '$t' is declared platform-owned but carries tenant_id — the declaration and the schema disagree (ADR-0024)"
      ;;
    tenant)
      has_col "$t" tenant_id || err "${TBL_FILE[$t]}: table '$t' is declared tenant-scoped but has no tenant_id column (ADR-0024)"
      has_col "$t" account_id && err "${TBL_FILE[$t]}: table '$t' is declared tenant-scoped but carries account_id — declare it account-scoped or drop the column (ADR-0024)"
      ;;
    account)
      has_col "$t" tenant_id || err "${TBL_FILE[$t]}: table '$t' is declared account-scoped but has no tenant_id column — account_id never appears alone, because account surrogates collide across tenants (ADR-0024)"
      has_col "$t" account_id || err "${TBL_FILE[$t]}: table '$t' is declared account-scoped but has no account_id column (ADR-0024)"
      ;;
    *)
      err "${TBL_FILE[$t]}: table '$t' declares no tenancy category — add '-- paridyum:scope platform|tenant|account' above the CREATE TABLE. The default is not inferred, because a schema cannot say whether a table belongs to the platform, a business, or one of that business's customers (ADR-0024)"
      ;;
  esac
done

# --- db.tables_declare_their_plane ------------------------------------------
# RFC-0014 decision 3. `scope` says whose a row is; `plane` says which database the table is
# in. Once residency puts tenants in per-region databases and everything global in one, those
# are independent questions and neither is inferable from the schema — the same argument
# ADR-0024 makes for its three categories, one level out.
for t in "${!TBL_FILE[@]}"; do
  case "${TBL_PLANE[$t]}" in
    control|regional) ;;
    *)
      err "${TBL_FILE[$t]}: table '$t' declares no plane — add '-- paridyum:plane control|regional' above the CREATE TABLE. Not inferred: a schema cannot say whether a table is one global row set or one per region, and the wrong answer puts data in a jurisdiction it may not be in (RFC-0014)"
      ;;
  esac
done

# --- db.tenant_scoped_tables_are_regional -----------------------------------
# Keyed on the DECLARED category, never on the presence of a tenant_id column. A
# platform-owned table may legitimately carry a tenant id as a key without being tenant-scoped
# — the domain-binding table is exactly that, and ADR-0024 already treats `regions` this way.
# So this fires on the declaration, which is the thing the author asserted.
for t in "${!TBL_FILE[@]}"; do
  if [[ "${TBL_PLANE[$t]}" == "control" ]]; then
    case "${TBL_SCOPE[$t]}" in
      tenant|account)
        err "${TBL_FILE[$t]}: table '$t' is declared ${TBL_SCOPE[$t]}-scoped AND control-plane — a tenant's rows live in that tenant's region and nowhere else. One of the two declarations is wrong (RFC-0014, ADR-0041)"
        ;;
    esac
  fi
done

# --- db.account_scoped_tables_carry_tenant_id -------------------------------
# The schema half. The query half needs a typed query builder or SQL parsing and is
# tool-forge's; that split is recorded in the registry rather than implied by silence.
for t in "${!TBL_FILE[@]}"; do
  if has_col "$t" account_id && ! has_col "$t" tenant_id; then
    err "${TBL_FILE[$t]}: table '$t' has account_id without tenant_id — a predicate on account_id alone crosses the OUTER silo while looking like a correctly scoped inner query (ADR-0024)"
  fi
done

# --- db.row_level_security_enabled_on_tenant_tables -------------------------
# ENABLE without FORCE exempts the table owner and migrations run as the owner, so a table
# can have RLS enabled, policies defined, and no isolation for the role the application
# connects as. Every dashboard reports healthy.
for t in "${!TBL_FILE[@]}"; do
  [[ "${TBL_SCOPE[$t]}" == "platform" ]] && continue
  has_col "$t" tenant_id || continue
  [[ -n "${RLS_ENABLED[$t]:-}" ]] || err "${TBL_FILE[$t]}: table '$t' carries tenant_id but never ENABLEs ROW LEVEL SECURITY (ADR-0024)"
  [[ -n "${RLS_FORCED[$t]:-}" ]] || err "${TBL_FILE[$t]}: table '$t' does not FORCE ROW LEVEL SECURITY — ENABLE alone exempts the table owner, and migrations run as the owner (ADR-0024)"
  body="${POLICY_BODY[$t]:-}"
  [[ -n "$body" ]] || { err "${TBL_FILE[$t]}: table '$t' has RLS but no policy — RLS with no policy denies everything, which fails closed and will read as a bug rather than a boundary (ADR-0024)"; continue; }
  grep -qE 'tenant_id[[:space:]]*=' <<<"$body" \
    || err "${TBL_FILE[$t]}: no policy on '$t' constrains tenant_id (ADR-0024)"
  if has_col "$t" account_id; then
    grep -qE 'account_id[[:space:]]*=' <<<"$body" \
      || err "${TBL_FILE[$t]}: '$t' is account-scoped but no policy constrains account_id — an outer-only policy isolates businesses from each other and nobody's customers (ADR-0024)"
  fi
done

# --- db.operator_scope_is_an_explicit_grant ---------------------------------
# ADR-0024 decision 4 and GLO-0005, and the load-bearing check of the set. Breadth expressed
# as the ABSENCE of a restriction is granted by every bug that loses a value — the query
# still runs, the rows still return, and nothing errors.
for t in "${!TBL_FILE[@]}"; do
  has_col "$t" account_id || continue
  body="${POLICY_BODY[$t]:-}"
  [[ -n "$body" ]] || continue
  if grep -qiE 'account_id[[:space:]]+IS[[:space:]]+NULL|IS[[:space:]]+NULL[[:space:]]*(OR|\))|account_id[[:space:]]*=[[:space:]]*'"'"'\*'"'"'|coalesce\(' <<<"$body"; then
    err "${TBL_FILE[$t]}: a policy on '$t' widens access using a NULL, wildcard or coalesce on the account discriminator. Operator scope must be an explicit positive claim, never an absence — absence is what every dropped session field, null lookup and middleware ordering bug produces, and each of them would silently escalate an end user to read the whole tenant (GLO-0005, ADR-0024)"
  fi
  grep -qE 'operator_scope|is_operator|has_operator' <<<"$body" \
    || wrn "${TBL_FILE[$t]}: no policy on '$t' names an operator-scope predicate. If the tenant's own console never reads across accounts this is correct; if it does, the grant is being expressed somewhere this check cannot see (GLO-0005)"
done

# --- db.tables_are_plural_snake_case ----------------------------------------
# C-0005. Pluralisation cannot be decided lexically — `data`, `metadata`, `series` have no
# distinct plural — so the check tests the decidable half and allowlists the rest.
PLURAL_OK="data metadata series audit_log_data"
for t in "${!TBL_FILE[@]}"; do
  [[ "$t" =~ ^[a-z][a-z0-9_]*$ ]] \
    || { err "${TBL_FILE[$t]}: table '$t' is not lowercase snake_case (C-0005)"; continue; }
  if [[ ! "$t" =~ s$ ]] && [[ " $PLURAL_OK " != *" $t "* ]]; then
    wrn "${TBL_FILE[$t]}: table '$t' does not look plural. Add it to this check's allowlist if it is a noun with no distinct plural (C-0005)"
  fi
done

# --- db.region_is_chosen_at_provisioning_and_immutable ----------------------
# ADR-0041 decision 3: region is contractual and immutable. ADR-0039 step 1 has said "insert
# tenant row (with `region`)" since 2026-08-08 against a column that did not exist, which is
# the gap this closes.
#
# DECLARED 2026-09-15 AND IMPLEMENTED 2026-09-20. In between it was owned by this script and
# enforced by nothing — the T-2.7 defect, where `owner` is the only signal the registry has for
# implemented-versus-owed and every value other than `tool-forge` counts as built. It was read
# as passing for five days because nothing printed anything.
#
# SCANS THE WHOLE CORPUS, not one file, because the column arrives by ALTER in a later migration
# than the CREATE TABLE. A per-file check would demand the column in 0001 and fail forever.
#
# SKIPS ENTIRELY WHERE THERE IS NO `tenants` TABLE. The fixture schemas have none, and a check
# that fired on their absence would be asserting that every migration set in the estate must
# define the platform's tenant table.
if [[ -n "${TBL_FILE[tenants]:-}" ]]; then
  CORPUS="$(cat "${MIGRATIONS[@]}" 2>/dev/null)"
  # Strip line comments before matching. Without this, the prose above a constraint — which in
  # 0011 discusses DEFAULT at length precisely to say there isn't one — matches the patterns
  # below and the check reports a defect it invented from documentation.
  CORPUS_SQL="$(sed 's/--.*$//' <<<"$CORPUS")"
  # Flattened to one line for the patterns that span several. `CREATE TRIGGER x` and its
  # `BEFORE UPDATE ON tenants` are conventionally on separate lines, and grep is line-oriented —
  # so the trigger clause matched nothing and this check reported a missing trigger that was
  # twelve lines above it. Found by the check failing against the migration that satisfies it,
  # which is the only reason it was found at all.
  CORPUS_FLAT="$(tr '\n' ' ' <<<"$CORPUS_SQL" | tr -s ' ')"

  if ! grep -qiE 'ALTER[[:space:]]+TABLE[[:space:]]+tenants[[:space:]]+ADD[[:space:]]+COLUMN[[:space:]]+region\b' <<<"$CORPUS_FLAT" \
     && ! grep -qiE '^[[:space:]]*region[[:space:]]+text' <<<"$CORPUS_SQL"; then
    err "no migration gives 'tenants' a region column — ADR-0039 step 1 provisions a tenant WITH a region and ADR-0041 D3 calls it contractual, so a tenant currently acquires a jurisdiction by having none recorded (RFC-0045 slice 2)"
  else
    grep -qiE 'ALTER[[:space:]]+TABLE[[:space:]]+tenants[[:space:]]+ALTER[[:space:]]+COLUMN[[:space:]]+region[[:space:]]+SET[[:space:]]+NOT[[:space:]]+NULL' <<<"$CORPUS_FLAT" \
      || grep -qiE '^[[:space:]]*region[[:space:]]+text[[:space:]]+NOT[[:space:]]+NULL' <<<"$CORPUS_SQL" \
      || err "'tenants.region' is nullable — a tenant with no region has no jurisdiction, and nothing downstream can tell that from a tenant whose region is merely unread (ADR-0041 D3)"

    if grep -qiE 'ALTER[[:space:]]+TABLE[[:space:]]+tenants[[:space:]]+ALTER[[:space:]]+COLUMN[[:space:]]+region[[:space:]]+SET[[:space:]]+DEFAULT' <<<"$CORPUS_FLAT" \
       || grep -qiE '^[[:space:]]*region[[:space:]]+text[^,]*DEFAULT' <<<"$CORPUS_SQL"; then
      err "'tenants.region' carries a DEFAULT — provisioning that forgets to choose a region then succeeds, and the tenant acquires a jurisdiction by accident. Refused rather than defaulted (ADR-0041 D3)"
    fi

    # The constraint is a PROJECTION of region-registry.yaml, and this is what keeps the two
    # honest. C-0007's rule is that the list is data; a CHECK listing region codes is a second
    # copy of that data, legitimate only while something compares them.
    constraint="$(grep -oiE 'CHECK[[:space:]]*\([[:space:]]*region[[:space:]]+IN[[:space:]]*\([^)]*\)' <<<"$CORPUS_FLAT" | head -1)"
    if [[ -z "$constraint" ]]; then
      err "'tenants.region' is not constrained to a live region — any string is accepted, so a typo is a jurisdiction (ADR-0041 D3, region-registry.yaml)"
    elif [[ -f "$ROOT/region-registry.yaml" ]]; then
      live="$(python3 -c "
import yaml,sys
d=yaml.safe_load(open('$ROOT/region-registry.yaml')) or {}
print(' '.join(sorted(r['code'] for r in d.get('regions',[]) if r.get('status')=='live')))" 2>/dev/null)"
      declared="$(grep -oE "'[^']+'" <<<"$constraint" | tr -d "'" | sort | tr '\n' ' ')"
      if [[ -n "$live" ]] && [[ "$(tr -s ' ' <<<"$live ")" != "$(tr -s ' ' <<<"$declared")" ]]; then
        err "'tenants.region' permits [${declared% }] but region-registry.yaml declares live regions [$live] — the constraint and the registry disagree, and the registry is the source (ADR-0035, C-0007)"
      fi
    fi

    grep -qiE 'CREATE[[:space:]]+TRIGGER[[:space:]]+[a-z_]+[[:space:]]+BEFORE[[:space:]]+UPDATE[[:space:]]+ON[[:space:]]+tenants' <<<"$CORPUS_FLAT" \
      || err "no trigger refuses an UPDATE to 'tenants.region' — moving a tenant by UPDATE changes the contract while every row stays where it was, so the record names one jurisdiction and the bytes are in another (ADR-0041 D3)"
  fi
fi

# === STD-0010 — locale-neutral storage ======================================
# Four checks, added 2026-08-08. They share the tenancy gate rather than getting a script of
# their own because they read the same migrations, and a second scanner over one input set is
# how the two drift. All four are TYPE checks, which is why the parser above now records types.

col_type() { printf '%s' "${TBL_COLTYPE["$1.$2"]:-}"; }

# Monetary columns are identified by NAME, which is the same compromise every check in this
# script makes: `amount` is unmistakable and a monetary column called `x` is unreachable.
# A bare `total` that counts things is the known false positive; rename it or add a suffix.
is_money_col() {
  case "$1" in
    *_minor|amount|price|cost|fee|total|subtotal|balance) return 0 ;;
    *_amount|*_price|*_cost|*_fee|*_total|*_subtotal|*_balance) return 0 ;;
  esac
  return 1
}

# --- db.timestamps_are_timestamptz ------------------------------------------
# C-0005 has required this since it was written and nothing enforced it. Bare `timestamp` is
# the dangerous spelling: it looks like it carries a zone, does not, and is what most ORMs and
# most examples emit. `date` is deliberately not rejected — a calendar date with no instant is
# legitimate and no check can tell a birth date from a misuse.
for t in "${!TBL_FILE[@]}"; do
  for col in ${TBL_COLS[$t]}; do
    [[ "$col" == *_at ]] || continue
    ty="$(col_type "$t" "$col")"
    [[ "$ty" =~ ^timestamp ]] || continue
    [[ "$ty" =~ ^timestamptz ]] && continue
    [[ "$ty" =~ with[[:space:]]+time[[:space:]]+zone ]] && continue
    err "${TBL_FILE[$t]}: '$t.$col' is '$ty' — a wall-clock reading whose instant nobody recorded, so every later conversion invents one. Use timestamptz (C-0005, STD-0010 §2)"
  done
done

# --- db.locale_columns_are_bcp47_text ---------------------------------------
# char(2) is where "locale is not language" stops being advice and becomes unrecoverable: the
# column physically cannot hold `en-IN`, so the region is discarded at write time and nothing
# later recovers which of en-IN, en-US and en-GB each row meant. sr-Latn and zh-Hant-HK are
# longer still. 35 is the longest realistic BCP 47 tag; wider fixed widths are not the defect.
for t in "${!TBL_FILE[@]}"; do
  for col in ${TBL_COLS[$t]}; do
    case "$col" in
      locale|language|lang|*_locale|*_language) ;;
      *) continue ;;
    esac
    ty="$(col_type "$t" "$col")"
    if [[ "$ty" =~ ^(character[[:space:]]+varying|varchar|character|char)([[:space:]]*\(([0-9]+)\))? ]]; then
      n="${BASH_REMATCH[3]:-1}"
      (( n < 35 )) && err "${TBL_FILE[$t]}: '$t.$col' is '$ty' — a BCP 47 tag does not fit, so the region subtag is truncated on write and cannot be recovered. Use text (GLO-0006, STD-0010 §1)"
    fi
  done
done

# --- db.money_columns_declare_currency / db.money_amounts_are_integral -------
# ADR-0033. Both checks run in one pass because they inspect the same columns. This
# repository's own `pass` fixture violated the first of them on the day it was declared —
# `invoices.amount bigint`, no currency anywhere on the table — which is the argument for the
# check rather than a coincidence: the fixture is what a new service is pattern matched against.
for t in "${!TBL_FILE[@]}"; do
  money_cols=""
  for col in ${TBL_COLS[$t]}; do is_money_col "$col" && money_cols="$money_cols $col"; done
  [[ -n "$money_cols" ]] || continue

  has_currency=false
  for col in ${TBL_COLS[$t]}; do
    case "$col" in currency|*_currency) has_currency=true ;; esac
  done
  $has_currency || err "${TBL_FILE[$t]}: table '$t' holds monetary column(s)$money_cols and declares no currency column. A bare amount stops being recoverable the moment the table holds two currencies, because nothing in the data says which is which (ADR-0033)"

  for col in $money_cols; do
    ty="$(col_type "$t" "$col")"
    if [[ "$ty" =~ ^(real|double|float|numeric|decimal|money)([[:space:]]|\(|$) ]]; then
      err "${TBL_FILE[$t]}: '$t.$col' is '$ty' — a stored amount must be an integer count of minor units. Binary floating point cannot represent 0.10, and numeric says nothing about scale (ADR-0033)"
    fi
    [[ "$col" == *_minor ]] || err "${TBL_FILE[$t]}: '$t.$col' has no _minor suffix. The name is the only place the scale is visible to every consumer, and the difference between minor units and a decimal is a factor of one hundred applied to an invoice (ADR-0033)"
  done
done

# --- db.no_non_unicode_encoding ---------------------------------------------
# The narrowest check here and the one whose failure is least recoverable — changing a
# database's encoding is a dump and restore of production. It can only see an encoding a
# migration STATES, and the common case is a database created by infrastructure rather than by
# SQL, so a green result is NOT evidence the database is UTF-8. STD-0010 cost 2 records that.
for f in "${MIGRATIONS[@]}"; do
  rel="${f#"$SCAN"/}"
  if grep -qiE "encoding[[:space:]]*=?[[:space:]]*'?(sql_ascii|latin[0-9]+|win[0-9]+|iso_8859)" "$f"; then
    err "$rel declares a non-Unicode encoding. SQL_ASCII is not 'ASCII only' — it is no encoding at all, storing bytes uninterpreted, so it accepts every insert and corrupts on the first comparison (STD-0010 §3)"
  fi
  while IFS= read -r line; do
    grep -qiE "utf-?8" <<<"$line" || err "$rel declares a collation or ctype that is not UTF-8: ${line# } (STD-0010 §3)"
  done < <(grep -oiE "lc_(collate|ctype)[[:space:]]*=[[:space:]]*'[^']*'" "$f" 2>/dev/null)
done

# --- data.regulated_class_requires_control_evidence -------------------------
# ANTI-0007. The one mechanically decidable part: a migration DECLARING a regulated column
# while no control framework exists. Keyed on the classification rather than on column names,
# because a `diagnosis` column is obvious and a `notes` column holding the same text is not.
for f in "${MIGRATIONS[@]}"; do
  rel="${f#"$SCAN"/}"
  if grep -qiE 'paridyum:class[[:space:]]+(phi|pci|financial-record)' "$f"; then
    manifest="$(dirname "$(dirname "$f")")/manifest.yaml"
    if [[ ! -f "$manifest" ]] || ! grep -qE '^[[:space:]]*compliance:' "$manifest"; then
      err "$rel classifies a column as regulated (phi/pci/financial-record) and no control framework is declared in manifest.yaml. ANTI-0007: the controls are the organisation, not the checklist, and there is not one yet. This is the only anti-pattern here whose violation can end the company in a single incident."
    fi
  fi
done

printf '\n'
if [[ "$EXPECT_FAIL" == true ]]; then
  [[ $ERRORS -gt 0 ]] && { printf '%s✔%s expected failure produced %d error(s)\n' "$GRN" "$RST" "$ERRORS"; exit 0; }
  printf '%s✘%s expected at least one error and got none — the planted defect was NOT caught\n' "$RED" "$RST"; exit 1
fi
if [[ $ERRORS -gt 0 ]]; then
  printf '%s%d error(s)%s, %d warning(s)\n' "$RED" "$ERRORS" "$RST" "$WARNS"; exit 1
fi
printf '%sMigration check passed%s (%d warning(s))\n' "$GRN" "$RST" "$WARNS"
