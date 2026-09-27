#!/usr/bin/env bash
#
# Run docket's audit writer against the real `audit_events` policies.
#
#   scripts/probe-audit-writer.sh            TENANCY_MIGRATIONS defaults to the sibling checkout
#
# WHY. The dispatcher deliberately swallows an audit-write failure (it logs and serves), so a writer
# that stopped satisfying the policies would lose every audit row while `cargo test` stays green —
# nothing else exercises the path. This replays platform-tenancy's migration stream into a scratch
# database, seeds one tenant, and runs `the_audit_sink_writes_under_the_append_policy_alone` as a
# LOGIN role subject to RLS. A superuser would bypass RLS and prove nothing.
#
# IT LEAVES THE MACHINE AS IT FOUND IT. The policies and the migrations' guarded grants name
# `paridyum_app`, so the test must act with that role's privileges — but a developer's local cluster
# may already use `paridyum_app` with its own password. So this never alters it: it creates a
# throwaway login role that is a MEMBER of `paridyum_app` (creating `paridyum_app` NOLOGIN only if it
# is absent, and dropping only what it created), and connects as that.
#
# IT PROVES THE POLICIES AS WRITTEN, NOT THE CLUSTER. Whether 0010 is applied there is `migrate.py
# status`'s question, not this script's.
set -uo pipefail

HERE="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
MIGRATIONS="${TENANCY_MIGRATIONS:-$HERE/../../../platform/platform-tenancy/migrations}"
DB="${PROBE_DB:-docket_audit_probe}"
LOGIN="docket_audit_probe_login"
PASS="probe-$(openssl rand -hex 12)"
TENANT=11111111-2222-3333-4444-555555555555

for tool in psql createdb dropdb openssl cargo python3; do
  command -v "$tool" >/dev/null 2>&1 || { printf '✘ %s is not on PATH\n' "$tool"; exit 3; }
done
[[ -d "$MIGRATIONS" ]] || { printf '✘ no migrations at %s (set TENANCY_MIGRATIONS)\n' "$MIGRATIONS"; exit 3; }
[[ "$DB" == *_probe ]] || { printf '✘ PROBE_DB must end in _probe (the test refuses anything else)\n'; exit 3; }

created_app=false
if ! psql -qAt -d postgres -c "SELECT 1 FROM pg_roles WHERE rolname = 'paridyum_app'" | grep -q 1; then
  psql -q -v ON_ERROR_STOP=1 -d postgres -c "CREATE ROLE paridyum_app NOLOGIN" || exit 3
  created_app=true
fi

cleanup() {
  dropdb --if-exists "$DB" >/dev/null 2>&1
  psql -q -d postgres -c "DROP ROLE IF EXISTS $LOGIN" >/dev/null 2>&1
  if [[ "$created_app" == true ]]; then psql -q -d postgres -c "DROP ROLE IF EXISTS paridyum_app" >/dev/null 2>&1; fi
}
trap cleanup EXIT

dropdb --if-exists "$DB" >/dev/null 2>&1
createdb "$DB" || { printf '✘ createdb %s failed\n' "$DB"; exit 3; }
psql -q -v ON_ERROR_STOP=1 -d postgres \
  -c "DROP ROLE IF EXISTS $LOGIN" \
  -c "CREATE ROLE $LOGIN LOGIN PASSWORD '$PASS' IN ROLE paridyum_app" \
  || { printf '✘ creating the probe login role failed\n'; exit 3; }
psql -q -v ON_ERROR_STOP=1 -d "$DB" \
  -c 'CREATE OR REPLACE FUNCTION uuidv7() RETURNS uuid LANGUAGE sql AS $$ SELECT gen_random_uuid() $$;' \
  || { printf '✘ the uuidv7 shim failed\n'; exit 3; }

# The role exists BEFORE the replay, as it does on the live database — so each migration's guarded
# grant block takes the branch the cluster takes, not the "role absent" branch.
applied=0
for f in "$MIGRATIONS"/*.sql; do
  if ! python3 -c 'import pathlib,sys; sys.stdout.write(pathlib.Path(sys.argv[1]).read_text().split("-- migrate:down")[0])' "$f" \
       | psql -q -v ON_ERROR_STOP=1 -d "$DB" >/dev/null; then
    printf '✘ %s failed to apply\n' "$(basename "$f")"; exit 1
  fi
  applied=$((applied+1))
done
printf '✔ %d tenancy migration(s) replayed into %s\n' "$applied" "$DB"

psql -q -v ON_ERROR_STOP=1 -d "$DB" \
  -c "INSERT INTO tenants (id, slug, name, zitadel_org_id, region) VALUES ('$TENANT','probe','Probe','1','in')" \
  -c "GRANT USAGE ON SCHEMA paridyum TO paridyum_app" \
  || { printf '✘ seeding the tenant failed\n'; exit 1; }

# The same server psql used: PGHOST/PGPORT when set, else the local socket's defaults.
export DOCKET_DATABASE_URL="postgres://$LOGIN:$PASS@${PGHOST:-localhost}:${PGPORT:-5432}/$DB"
export DOCKET_TEST_TENANT="$TENANT"
export DOCKET_AUDIT_PROBE_DB="$DB"
cd "$HERE/.." || exit 3
# Full output: on a failure the assertion message is the evidence, and `tail` would cut it.
cargo test -q --lib adapters::pg::tests::the_audit_sink_writes_under_the_append_policy_alone -- --exact --nocapture
