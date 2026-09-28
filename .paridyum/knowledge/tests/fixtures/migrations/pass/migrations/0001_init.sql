-- paridyum:scope platform
-- paridyum:plane control
CREATE TABLE regions (
    id          uuid PRIMARY KEY,         -- internal
    code        text NOT NULL,            -- public
    created_at  timestamptz NOT NULL      -- internal
);

-- ADR-0114 D1: the org registry is personless control-plane data; the org's people are org-scoped
-- rows in its region, isolated from every other org by their own predicate (ADR-0115 D3).
-- paridyum:scope platform
-- paridyum:plane control
CREATE TABLE orgs (
    id          uuid PRIMARY KEY,         -- internal
    region      text NOT NULL,            -- internal
    created_at  timestamptz NOT NULL      -- internal
);

-- paridyum:scope org
-- paridyum:plane regional
CREATE TABLE org_members (
    id          uuid PRIMARY KEY,
    org_id      uuid NOT NULL,
    subject     text NOT NULL,            -- internal
    role        text NOT NULL,
    created_at  timestamptz NOT NULL
);
ALTER TABLE org_members ENABLE ROW LEVEL SECURITY;
ALTER TABLE org_members FORCE ROW LEVEL SECURITY;
CREATE POLICY org_members_org ON org_members
    USING (org_id = paridyum.current_org());

-- paridyum:scope tenant
-- paridyum:plane regional
CREATE TABLE plans (
    id          uuid PRIMARY KEY,
    tenant_id   uuid NOT NULL,
    name        text NOT NULL,
    created_at  timestamptz NOT NULL
);
ALTER TABLE plans ENABLE ROW LEVEL SECURITY;
ALTER TABLE plans FORCE ROW LEVEL SECURITY;
CREATE POLICY plans_tenant ON plans
    USING (tenant_id = paridyum.current_tenant());

-- GLO-0006's four fields, each its own column and none derived from another. `text` rather
-- than a fixed width for the three BCP 47 / IANA values; char(3) for currency only because
-- ISO 4217 fixes the width. STD-0010 §1.
-- paridyum:scope tenant
-- paridyum:plane regional
CREATE TABLE tenant_settings (
    id                uuid PRIMARY KEY,
    tenant_id         uuid NOT NULL,
    language          text NOT NULL,
    locale            text NOT NULL,
    timezone          text NOT NULL,
    currency          char(3) NOT NULL,
    created_at        timestamptz NOT NULL
);
-- A jsonb key that happens to be a language subtag is not a locale set: `config ? 'id'` must pass
-- db.localized_content_matches_its_tenant_locales, which reports two or more distinct codes only.
ALTER TABLE tenant_settings ADD COLUMN config jsonb NOT NULL CHECK (config ? 'id');
ALTER TABLE tenant_settings ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_settings FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_settings_tenant ON tenant_settings
    USING (tenant_id = paridyum.current_tenant());

-- ADR-0033: an integer count of minor units, and the currency beside it. This table read
-- `amount bigint` with no currency anywhere until 2026-08-08, which is what prompted the
-- check — the fixture is what a new service gets pattern matched against.
-- paridyum:scope account
-- paridyum:plane regional
CREATE TABLE invoices (
    id            uuid PRIMARY KEY,
    tenant_id     uuid NOT NULL,
    account_id    uuid NOT NULL,
    amount_minor  bigint NOT NULL,
    currency      char(3) NOT NULL,
    created_at    timestamptz NOT NULL
);
ALTER TABLE invoices ENABLE ROW LEVEL SECURITY;
ALTER TABLE invoices FORCE ROW LEVEL SECURITY;
CREATE POLICY invoices_scoped ON invoices
    USING (tenant_id = paridyum.current_tenant()
           AND (paridyum.has_operator_scope() OR account_id = paridyum.current_account()));
