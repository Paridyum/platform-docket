-- db.tables_declare_their_plane — the tenancy category is declared and the plane is not.
-- RFC-0014 decision 3: `scope` says whose a row is, `plane` says which database holds it.
-- Neither answers the other, so declaring one is not declaring both.

-- paridyum:scope platform
-- paridyum:plane control
CREATE TABLE regions (
    id          uuid PRIMARY KEY,
    code        text NOT NULL,
    created_at  timestamptz NOT NULL
);

-- paridyum:scope tenant
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
