# Multi-tenant docket — implementation design (ADR-0099)

Status: draft, stage-by-stage. Governs the work that turns docket from a served
single-tenant tool into a multi-tenant platform. The decision is ADR-0099; this
is how it is built, and in what order, without breaking Paridyum's live
deployment at any point.

## The shape of the change

The data plane is already multi-tenant (migrations 0006/0007: every row
`tenant_id`, FORCE RLS, `pg.rs` sets `paridyum.tenant_id` per transaction). Two
single-tenant fixpoints remain and one new plane must be built:

1. **Backend tenant binding** — `serve.rs` reads one `DOCKET_TENANT_ID`; it must
   derive the tenant per request from the operator token's `resourceowner`.
2. **Operator identity** — global `operators.yaml`; must become per-tenant.
3. **Config plane** — the whole `registry/` (companies, signatories, operators,
   fields, person-classes, lifecycle, communications, retention, templates) is
   git-declared Paridyum-only; it must become per-tenant data, seeded from git.

## Stages (each a reviewed PR; Paridyum keeps working throughout)

### Stage 1 — schema for the config plane (platform-tenancy)

New migrations, each tenant-scoped and `FORCE ROW LEVEL SECURITY`, policy
`current_tenant() AND has_operator_scope()` (config is operator-only, like the
docket chains), following the conventions `check-migrations.sh` enforces
(scope directive, FORCE RLS, policy, money as minor+currency, locale).

Table-by-table mapping from the git registry:

- `docket_companies` ← `org.yaml`. Columns: `code` (text, first serial segment),
  `name`, `tagline`, `address`, `email`, `web`, `legal_status`, `jurisdiction`,
  `serial_format`, `owned_by` (self-FK by code within tenant, nullable for the
  one holding company), `is_default` (exactly one true per tenant),
  `posh_routing` (jsonb). The ownership-reaches-root and unique-serial-format
  invariants ADR-0077 states move from `docket check` into CHECK/trigger +
  application validation.
- `docket_operators` ← `operators.yaml`. `op_id` (the short handle), `name`,
  `title`, `active`, `subject` (the Zitadel control-plane subject, ADR-0092).
  This is the tenant-scoped replacement for the global registry: an operator is
  a subject with the `operator` role in the tenant's org AND a row here.
- `docket_signatories` ← `signatories.yaml`. `person_ref`, `company_code`,
  `role`, `order`, `active`. The countersignature fallback order per company.
- `docket_field_defs` ← `fields.yaml`. Field key, label, class (data
  classification, REQ-0003), purpose, required, kind, choices (jsonb). A field
  with no stated purpose must not exist — the CHECK carries that.
- `docket_person_classes` ← `person-classes.yaml`. Class → the fields it may
  collect and the identifier scheme (ADR-0051).
- `docket_lifecycle` ← `lifecycle.yaml`. Document classes, their stages and
  transitions. The hardest table (see Stage 5 templates).
- `docket_retention` ← `retention.yaml`, `docket_comms` ← `communications.yaml`,
  `docket_team` ← `team.yaml` — smaller, same pattern.

Store each as verbatim jsonb where the shape is nested and reviewed (companies,
lifecycle, fields), with generated columns for the keys queried on — the same
RFC-0039 D1 reasoning migration 0006 used for the chains, so a config record
round-trips unchanged and an admin edit is diffable.

### Stage 2 — backend reads tenant-from-token and config-from-DB

- `auth.rs`: `VerifiedOperator { subject, resourceowner }`; derive the tenant
  uuid from `resourceowner` (ADR-0082, no DB lookup — deterministic from the
  signed claim). A token with no `resourceowner` is refused.
- `RouterContext`/request: carry the derived tenant; every store call uses it.
  `PgStore` stops holding a fixed tenant — the tenant is a per-request value.
- Config loading: a `TenantConfig` port that loads companies/operators/etc. for
  the request's tenant from the DB, replacing `Registry::load` from git. While a
  tenant has no rows, fall back to the git registry (so Paridyum keeps working
  before Stage 3 cuts it over). Cache per tenant with an invalidation hook.
- `serve.rs`: drop `DOCKET_TENANT_ID`; the chart drops the env.

### Stage 3 — seed Paridyum's config and cut over

A one-time, reviewed data migration: read the current git `registry/`, write it
as Paridyum's tenant rows (Paridyum is the first tenant). Verify docket serves
Paridyum identically from the DB config, then remove the git-registry fallback
for tenants that have rows. The git files remain as the onboarding seed and as
history until retired.

### Stage 4 — onboarding a tenant

Extend platform-tenancy's `provision-org` path: a new Zitadel org → its derived
tenant row (ADR-0082) → the `operator` role granted to its staff → a seeded
default docket config (a starter company, the standard field defs and classes,
the default lifecycle) the tenant then edits. Idempotent, like the rest of
provision.

### Stage 5 — the web-docket admin surface (ADR-0098)

web-docket (the served BFF console) gains the operator-only, tenant-scoped admin
area for the tenant's own companies, signatories, operators, field sets and
templates. Templates are reviewed content, so their editor is a review surface
(propose → diff → publish), not a free-text box — the postings model (ADR-0095)
is the precedent.

## Invariants that must survive the move

- **One person, one identifier, across companies** (ADR-0051) — person id stays
  group-wide within a tenant; company code prefixes only engagement/serials.
- **Append-only sealed chains** (ADR-0093) — unchanged; config is mutable, the
  chains are not, and they are already tenant-scoped.
- **Ownership reaches the root; serial formats are unique per tenant**
  (ADR-0077) — moved from `docket check` to DB constraints + app validation.
- **A field with no purpose must not exist; PII is classified** (REQ-0003) —
  carried by `docket_field_defs` CHECKs.
- **RLS is the isolation** — every config table FORCE-RLS, tenant+operator
  policy; a backend that derives the wrong tenant still cannot read another's
  rows.

## What does not change

The break-glass CLI stays single-tenant-explicit (`DOCKET_TENANT_ID` + direct
DB). The sealed chains, the blob store (Garage, tenant-prefixed keys), and the
document rendering pipeline are untouched — only where their *inputs* (the
config) come from changes.
