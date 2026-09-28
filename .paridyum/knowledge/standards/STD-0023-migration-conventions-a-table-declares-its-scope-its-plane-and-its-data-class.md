---
id: STD-0023
title: Migration conventions — a table declares its scope, its plane and its data class
type: standard
status: accepted
created: 2026-09-28
locked_at: null
supersedes: [STD-0006]
superseded_by: null
scope: [org, platform, service]
applies_to: ["platform-*", "svc-*"]
product: [tenancy]
tags: [migrations, sql, tenancy, org, planes, region, postgres, rls, data-classification, adr-0115, rfc-0014, std-0006]
breaking: true
enforced_by:
  - check: db.tenant_scoped_tables_declare_tenancy
    severity: error
    rationale: >
      The scope directive this standard fixes is the input to that check. Without a ratified syntax
      the check enforces a convention that lives only in a script. Carried from STD-0006, widened to
      four categories.
  - check: db.tables_declare_their_plane
    severity: error
    rationale: >
      The plane directive was implemented in check-migrations.sh before any standard ratified it
      (RFC-0014 D3). This ratifies it, so the rule is written where a migration author looks.
  - check: db.no_personal_data_in_the_control_plane
    severity: error
    rationale: >
      Rule 5's control-plane clause. The class comment and the plane directive are both in the
      migration, so a pii, confidential or unclassified column in a control-plane table is decidable
      by reading it (ADR-0117 D3).
  - check: data.regulated_class_requires_control_evidence
    severity: error
    rationale: >
      The data-class directive: the classification is where liability attaches, so its syntax cannot
      be an implementation detail. Carried from STD-0006 unchanged.
---

# STD-0023 — Migration conventions

## Status

**ACCEPTED** 2026-09-28. **Supersedes STD-0006** (`locked`), restated here in full. Locked in a
later change, once the `org` rules below have been watched failing against planted defects.

**What changed from STD-0006:**

1. `-- paridyum:scope` takes a fourth value, **`org`** (ADR-0115 D1).
2. `-- paridyum:plane` is ratified. It was implemented in `check-migrations.sh` on RFC-0014's
   authority and used by every migration since, with no standard behind it.
3. `-- paridyum:rescope` is ratified. The gate has parsed it since the docket retrofit, and six
   applied migrations carry it.

Nothing legal under STD-0006 becomes illegal, with two exceptions it did not foresee: a platform table
carrying `org_id` or `account_id` (rule 1), and an unclassified column in a control-plane table
(rule 5).

## The rules

### 1. Scope and plane, declared immediately above `CREATE TABLE`

```sql
-- paridyum:scope platform | org | tenant | account     -- whose the rows are   (ADR-0115 D1)
-- paridyum:plane control  | regional                   -- which database        (ADR-0115 D10)
CREATE TABLE …
```

Exactly one of each per table, on the lines directly above the statement, in either order.

| Scope | Columns required | Columns forbidden | Meaning |
| --- | --- | --- | --- |
| `platform` | — | `org_id`, `tenant_id`, `account_id` | Paridyum's own data, and the registries that define orgs and tenants |
| `org` | `org_id` | `tenant_id`, `account_id` | One org's own data (GLO-0012) |
| `tenant` | `tenant_id` | `org_id`, `account_id` | One business's data (GLO-0002) |
| `account` | `tenant_id` **and** `account_id` | `org_id` | One silo inside one business (GLO-0003) |

| Plane | Meaning | Allowed scopes |
| --- | --- | --- |
| `control` | the one control-plane database, in the control region | `platform` only |
| `regional` | each region's data-plane database | any |

**There is no default for either.** A table with no directive is an error, never a `platform` or a
`control` table. ADR-0115 says neither can be inferred, and defaulting silently would be inference
with extra steps.

**A platform table references an org or a tenant by a named key**, never by a discriminator's name:
`tenants.owner_org_id`, not `tenants.org_id`. A reader, a policy author and this gate must all be
able to tell a scope from a relationship at a glance.

### 2. Row-level security on every org-, tenant- and account-scoped table

```sql
ALTER TABLE plans ENABLE ROW LEVEL SECURITY;
ALTER TABLE plans FORCE  ROW LEVEL SECURITY;
```

**Both statements.** `ENABLE` alone exempts the table owner, migrations run as the owner, and the
result is a table with RLS enabled, policies defined, and no isolation for the role the application
connects as.

**A predicate per level:**

```sql
-- org-scoped
CREATE POLICY org_members_org ON org_members
    USING (org_id = paridyum.current_org());

-- tenant-scoped
CREATE POLICY plans_tenant ON plans
    USING (tenant_id = paridyum.current_tenant());

-- account-scoped: an outer-only policy isolates businesses from each other and nobody's customers
CREATE POLICY invoices_scoped ON invoices
    USING (tenant_id = paridyum.current_tenant()
           AND (paridyum.has_operator_scope() OR account_id = paridyum.current_account()));
```

`paridyum.current_org()`, `paridyum.current_tenant()` and `paridyum.current_account()` raise when
their setting is unset and never return NULL (ADR-0038). A session in one context therefore cannot
read another context's rows by accident: the predicate raises rather than matching.

### 3. Operator scope is a positive claim, never an absence

The account-scoped clause above is the shape. **Forbidden:** widening access with `IS NULL`, a
wildcard, or `coalesce` on any discriminator.

```sql
-- FORBIDDEN
USING (tenant_id = paridyum.current_tenant()
       AND (paridyum.current_account() IS NULL OR account_id = paridyum.current_account()));
```

That reads as "an operator has no account filter", and it is the GLO-0005 hazard: absence is what
every dropped session field, uninitialised variable, null lookup and middleware ordering bug
produces.

### 4. Re-declaring a table's scope

An applied migration is immutable (REQ-0008), so a retrofit cannot edit the directive above the
original `CREATE TABLE`. It re-declares instead, naming the table:

```sql
-- paridyum:rescope docket_persons account
```

**The last declaration wins, in migration order**, as the schema itself does. A rescope of a table no
migration creates is an error. The rescoped category's column and predicate rules apply from that
migration on.

### 5. Data classification

A column carries its class as a trailing comment, and regulated classes carry the directive:

```sql
    legal_name   text NOT NULL,     -- pii
    institution  text NOT NULL,     -- confidential
    degree       text,              -- internal
    -- paridyum:class phi
    diagnosis    text,
```

- Classes: `pii`, `confidential`, `internal`, `public` (REQ-0003 item 1).
- Regulated directives: `phi`, `pci`, `financial-record`. A migration carrying any of them requires a
  control framework declared in `manifest.yaml`, and without one the gate fails — ANTI-0007.
- **Every column of a `control`-plane table carries its class, and none is `pii` or `confidential`**
  (ADR-0117 D3, `db.no_personal_data_in_the_control_plane`). An unclassified column there is the
  absence-as-permission shape GLO-0005 forbids, one level over: the gate cannot tell it from a
  residency breach.

Keyed on the **classification** and never on the column name. A `diagnosis` column is obvious and a
`notes` column holding the same text is not, so a name heuristic would produce false confidence in
exactly the case that matters.

### 6. Naming defers to C-0005

Plural snake_case tables, `id` primary keys, `<singular>_id` foreign keys, and `created_at` /
`updated_at` / `deleted_at` typed `timestamptz`. Not restated here; C-0005 owns it. Rule 1's
reserved discriminator names are the one exception to `<singular>_id`: a platform table's key to an
org is `owner_org_id` (or another named role), never `org_id`.

## Costs accepted

1. **A comment carries meaning, so a formatter can strip it.** The failure mode of losing a directive
   is an error, not a wrong answer: the gate reports the table undeclared. That is the mitigation.
2. **The database cannot enforce the directives.** They are invisible to Postgres. A
   `paridyum.table_scope` catalogue table would make them constrainable data, and needs a database in
   the pre-commit gate, which does not exist. Worth revisiting once a database runs in CI.
3. **A sidecar file was rejected**: drift between two files is a failure mode the comment does not
   have.
4. **Four scopes and two planes are more to get right than three scopes were.** Every combination the
   tables above forbid is decidable by reading one migration, which is why each is an error and not a
   review note.

## Related

- **ADR-0115** — the decision this gives a syntax to · **STD-0006** — superseded; restated here
- **RFC-0014** D3 — the plane directive's origin · **ADR-0117** D3 — no person in the control plane
- **GLO-0002**, **GLO-0003**, **GLO-0004**, **GLO-0005**, **GLO-0012** — the vocabulary these encode
- **ANTI-0007** — why the data-class directive exists · **REQ-0003** — classification at creation
- **C-0005** — naming · **REQ-0008** — migrations are immutable once applied
