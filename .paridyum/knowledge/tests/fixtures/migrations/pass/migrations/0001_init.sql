-- paridyum:scope platform
-- paridyum:plane control
CREATE TABLE regions (
    id          uuid PRIMARY KEY,
    code        text NOT NULL,
    created_at  timestamptz NOT NULL
);

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
