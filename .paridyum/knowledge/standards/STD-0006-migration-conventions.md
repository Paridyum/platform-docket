---
id: STD-0006
title: Migration conventions — how a table declares its tenancy and its data class
type: standard
status: locked
created: 2026-08-07
locked_at: 2026-08-07
supersedes: null
superseded_by: null
scope: [org, platform, service]
applies_to: ["platform-*", "svc-*"]
tags: [migrations, sql, tenancy, postgres, rls, data-classification]
breaking: false
enforced_by:
  - check: db.tenant_scoped_tables_declare_tenancy
    severity: error
    rationale: >
      The directive this standard fixes is the input to that check. Without a ratified
      syntax the check enforces a convention that lives only in a script.
  - check: data.regulated_class_requires_control_evidence
    severity: error
    rationale: >
      Same argument for the data-class directive: the classification is where liability
      attaches, so its syntax cannot be an implementation detail.
---

# STD-0006 — Migration conventions

## Status

**LOCKED** as of 2026-08-07, with two implemented checks.

## Why this exists

ADR-0024 decision 1 requires every table to declare one of three tenancy categories and
**deliberately does not fix the syntax.** That was the right scope for an ADR and it left a gap:
`check-migrations.sh` had to choose a syntax in order to check anything, and a convention that
exists only inside a gate script is a rule nobody agreed to that nonetheless binds every
migration anyone writes. C-0001 principle 5 — *never infer, always retrieve* — cannot be
followed against a convention that was never written down.

This ratifies the choice the implementation made. Nothing here is new behaviour; the value is
that it is now a rule rather than a side effect.

## The rules

### 1. Tenancy category, declared immediately above `CREATE TABLE`

```sql
-- paridyum:scope platform | tenant | account
```

Exactly one per table, on the line directly above the statement.

| Category | Columns required | Meaning |
| -------- | ---------------- | ------- |
| `platform` | neither | Paridyum's own data. Shared by every tenant. |
| `tenant` | `tenant_id` | Belongs to one business (GLO-0002). |
| `account` | `tenant_id` **and** `account_id` | Belongs to one customer of one business (GLO-0003). |

**There is no default.** A table with no directive is an error, not a `platform` table. ADR-0024
says the category cannot be inferred, and defaulting silently would be inference with extra
steps — `invoices` is account-scoped, the `plans` a product offers is tenant-scoped, `regions`
is platform-owned, and no naming rule separates them.

`account_id` never appears without `tenant_id`. Account surrogate values are unique within a
tenant and collide across tenants, so a predicate on `account_id` alone crosses the **outer**
silo while looking like a correctly scoped inner query.

### 2. Row-level security, on every table that carries `tenant_id`

```sql
ALTER TABLE plans ENABLE ROW LEVEL SECURITY;
ALTER TABLE plans FORCE  ROW LEVEL SECURITY;
```

**Both statements.** `ENABLE` alone exempts the table owner, migrations run as the owner, and
the result is a table with RLS enabled, policies defined, and no isolation whatsoever for the
role the application connects as. Every dashboard reports healthy.

A policy per level. On an account-scoped table, an outer-only policy isolates businesses from
each other and nobody's customers:

```sql
CREATE POLICY invoices_scoped ON invoices
    USING (tenant_id = paridyum.current_tenant()
           AND (paridyum.has_operator_scope() OR account_id = paridyum.current_account()));
```

### 3. Operator scope is a positive claim, never an absence

The clause above is the shape. **Forbidden:** widening access with `IS NULL`, a wildcard, or
`coalesce` on the account discriminator.

```sql
-- FORBIDDEN
USING (tenant_id = paridyum.current_tenant()
       AND (paridyum.current_account() IS NULL OR account_id = paridyum.current_account()));
```

That reads as "an operator has no account filter", and it is the GLO-0005 hazard: absence is
what every dropped session field, uninitialised variable, null lookup and middleware ordering
bug produces. Each of them would silently promote an ordinary end user to read the entire
tenant, and the query would still run and still return rows.

### 4. Data classification, where the field is regulated

```sql
    -- paridyum:class phi
    diagnosis   text,
```

Values: `phi`, `pci`, `financial-record`. A migration carrying any of them requires a control
framework declared in `manifest.yaml`, and without one the gate fails — ANTI-0007.

Keyed on the **classification** and never on the column name, deliberately. A `diagnosis`
column is obvious and a `notes` column holding the same text is not, so a name heuristic would
produce false confidence in exactly the case that matters.

### 5. Naming defers to C-0005

Plural snake_case tables, `id` primary keys, `<singular>_id` foreign keys, and `created_at` /
`updated_at` / `deleted_at` typed `timestamptz`. Not restated here; C-0005 owns it.

## Costs accepted

1. **A comment carries meaning, so a formatter can strip it.** The chosen syntax is a SQL
   comment, which means a reformatter that drops comments silently removes the declaration —
   and the gate then reports the table as undeclared, which is loud rather than dangerous. That
   is the mitigation: the failure mode of losing the directive is an error, not a wrong answer.

2. **The database cannot enforce it.** The directive is invisible to Postgres, so nothing stops
   a table being created outside a migration without one. ADR-0024 already prefers live-catalogue
   verification for RLS (`pg_class.relrowsecurity`) over parsing migration text, and the same
   argument applies here — a `paridyum.table_scope` catalogue table would make the declaration
   data the database itself could constrain. **Rejected for now** because it needs a running
   database to check, which the pre-commit gate does not have, and a rule enforced only in CI is
   weaker than one enforced before the commit. Worth revisiting once a database exists in CI.

3. **A sidecar file was rejected**, though it is more structured and unstrippable: it puts the
   declaration in a different file from the table it describes, and drift between the two is a
   new failure mode the comment simply does not have.

## Related

- ADR-0024 — the decision this gives a syntax to; decision 1 deliberately left it open
- GLO-0002, GLO-0003, GLO-0004, GLO-0005 — the vocabulary these directives encode
- ANTI-0007 — why the data-class directive exists
- C-0005 — table, column and key naming
- REQ-0003 — classification at the migration that creates the column
- REQ-0008 — migrations are reversible and deployable without downtime
