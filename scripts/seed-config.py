#!/usr/bin/env python3
"""Seed a tenant's docket configuration from the git registry into the per-tenant tables. ADR-0099.

ADR-0099 moves docket's configuration — companies, operators, signatories, field definitions, person
classes, the document lifecycle, onboarding messages and retention — out of git and into per-tenant
tables (platform-tenancy migration 0008). This is the one-time DATA move that makes Paridyum the
FIRST tenant's rows: it reads `registry/*.yaml` and writes each entry as a row for the given tenant,
under that tenant's operator RLS context. Idempotent (ON CONFLICT DO NOTHING), so re-running adds
only what is missing; it never edits or deletes a row.

It writes the record VERBATIM as the jsonb the tables store (the same shape `docket check` validates),
so the git registry and the seeded rows are the same document — the point of RFC-0039 D1 carried to
the config plane. It does NOT read git as a fallback afterwards; that cutover is the backend's
(config-from-DB), a separate reviewed change.

Connection: DOCKET_DATABASE_URL if set (local/throwaway, break-glass), else `kubectl exec` into the
primary pod exactly as scripts/migrate.py does — no credential leaves the cluster.

  DOCKET_DATABASE_URL=... python3 scripts/seed-config.py --tenant <uuid>          # dry run
  DOCKET_DATABASE_URL=... python3 scripts/seed-config.py --tenant <uuid> --apply  # write
"""
from __future__ import annotations
import argparse, json, os, subprocess, sys
from pathlib import Path

import yaml

# Paridyum's tenant id (ADR-0082 derivation of its Zitadel org). The default because Paridyum is the
# first tenant; any other tenant passes its own uuid.
PARIDYUM_TENANT = "68445ee9-175a-58a9-834e-a2f0c9c9b124"

REGISTRY = Path(__file__).resolve().parent.parent / "registry"

# (yaml file, the list key inside it, the target table). The record stored is each list entry.
SOURCES = [
    ("org.yaml", "companies", "docket_companies"),
    ("operators.yaml", "operators", "docket_operators"),
    ("signatories.yaml", "signatories", "docket_signatories"),
    ("fields.yaml", "fields", "docket_field_defs"),
    ("person-classes.yaml", "classes", "docket_person_classes"),
    ("lifecycle.yaml", "documents", "docket_lifecycle"),
    ("lifecycle.yaml", "stages", "docket_lifecycle_stages"),
    ("communications.yaml", "messages", "docket_communications"),
    ("retention.yaml", "classes", "docket_retention"),
]

# Cluster connection (mirrors scripts/migrate.py in platform-tenancy).
NS, POD, DB = "postgres", "paridyum-1", "paridyum"


def run_sql(sql: str) -> None:
    """Apply SQL, as the paridyum role, either over DOCKET_DATABASE_URL or via kubectl exec."""
    url = os.environ.get("DOCKET_DATABASE_URL")
    if url:
        cmd = ["psql", url, "-v", "ON_ERROR_STOP=1", "-q"]
    else:
        cmd = ["kubectl", "-n", NS, "exec", "-i", POD, "--", "psql", "-U", "postgres",
               "-d", DB, "-v", "ON_ERROR_STOP=1", "-q"]
    r = subprocess.run(cmd, input=sql, capture_output=True, text=True)
    if r.returncode != 0:
        raise SystemExit(f"✘ seed failed:\n{r.stderr.strip()}")


def entries(fname: str, key: str) -> list:
    doc = yaml.safe_load((REGISTRY / fname).read_text()) or {}
    items = doc.get(key) or []
    if not isinstance(items, list):
        raise SystemExit(f"✘ {fname}: '{key}' is not a list")
    return items


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--tenant", default=PARIDYUM_TENANT, help="the tenant uuid to seed (default: Paridyum)")
    ap.add_argument("--apply", action="store_true", help="write; without it, only reports counts")
    args = ap.parse_args()

    planned = []
    for fname, key, table in SOURCES:
        items = entries(fname, key)
        planned.append((table, items))
        print(f"  {fname:22} → {table:24} {len(items):3} record(s)")

    total = sum(len(i) for _, i in planned)
    if not args.apply:
        print(f"\n{total} record(s) would be seeded for tenant {args.tenant}. Re-run with --apply.")
        return 0

    # One transaction per table, under the tenant's operator RLS context. Dollar-quoting the jsonb
    # avoids escaping; ON CONFLICT DO NOTHING makes a re-run add only what is missing.
    for table, items in planned:
        if not items:
            continue
        lines = [
            "BEGIN;",
            f"SELECT set_config('paridyum.tenant_id', '{args.tenant}', true);",
            "SELECT set_config('paridyum.operator_scope', 'true', true);",
        ]
        for entry in items:
            rec = json.dumps(entry, ensure_ascii=False)
            lines.append(
                f"INSERT INTO {table} (tenant_id, record) "
                f"VALUES ('{args.tenant}', $rec${rec}$rec$::jsonb) ON CONFLICT DO NOTHING;"
            )
        lines.append("COMMIT;")
        run_sql("\n".join(lines))
        print(f"= seeded {table} ({len(items)})")

    print(f"✔ seeded {total} record(s) for tenant {args.tenant}.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
