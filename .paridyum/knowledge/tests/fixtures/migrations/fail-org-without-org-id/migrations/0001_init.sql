-- db.tenant_scoped_tables_declare_tenancy — an org-scoped table with no org_id (ADR-0115 D1).
-- Every other rule holds: it is regional, RLS is ENABLEd and FORCEd, a policy exists. Without the
-- discriminator the policy cannot name one org, so the table belongs to every org at once.
-- paridyum:scope org
-- paridyum:plane regional
CREATE TABLE org_members (
    id          uuid PRIMARY KEY,
    subject     text NOT NULL,
    role        text NOT NULL,
    created_at  timestamptz NOT NULL
);
ALTER TABLE org_members ENABLE ROW LEVEL SECURITY;
ALTER TABLE org_members FORCE ROW LEVEL SECURITY;
CREATE POLICY org_members_all ON org_members
    USING (role IS NOT NULL);
