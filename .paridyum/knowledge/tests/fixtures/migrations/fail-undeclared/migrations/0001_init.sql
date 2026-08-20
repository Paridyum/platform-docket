-- paridyum:scope platform
-- paridyum:plane control
CREATE TABLE regions (
    id          uuid PRIMARY KEY,
    code        text NOT NULL,
    created_at  timestamptz NOT NULL
);

-- The plane is declared and the SCOPE deliberately is not, so this fixture fails for exactly
-- one reason. Without the plane line it would also trip db.tables_declare_their_plane, and a
-- fixture caught for two reasons no longer proves the one it is named after.
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

-- paridyum:scope account
-- paridyum:plane regional
CREATE TABLE invoices (
    id          uuid PRIMARY KEY,
    tenant_id   uuid NOT NULL,
    account_id  uuid NOT NULL,
    amount      bigint NOT NULL,
    created_at  timestamptz NOT NULL
);
ALTER TABLE invoices ENABLE ROW LEVEL SECURITY;
ALTER TABLE invoices FORCE ROW LEVEL SECURITY;
CREATE POLICY invoices_scoped ON invoices
    USING (tenant_id = paridyum.current_tenant()
           AND (paridyum.has_operator_scope() OR account_id = paridyum.current_account()));
