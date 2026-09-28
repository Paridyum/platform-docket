-- db.row_level_security_enabled_on_tenant_tables — an org table whose policy names no org.
-- RLS is ENABLEd and FORCEd and a policy exists, so a dashboard reports it isolated. It isolates
-- nobody: every org's members are visible to every org.
-- paridyum:scope org
-- paridyum:plane regional
CREATE TABLE org_members (
    id          uuid PRIMARY KEY,
    org_id      uuid NOT NULL,
    role        text NOT NULL,
    created_at  timestamptz NOT NULL
);
ALTER TABLE org_members ENABLE ROW LEVEL SECURITY;
ALTER TABLE org_members FORCE ROW LEVEL SECURITY;
CREATE POLICY org_members_any ON org_members
    USING (role IS NOT NULL);
