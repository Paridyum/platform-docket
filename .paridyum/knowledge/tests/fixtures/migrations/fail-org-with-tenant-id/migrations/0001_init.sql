-- db.tenant_scoped_tables_declare_tenancy — an org-scoped table carrying tenant_id (ADR-0115 D2).
-- A fact relating an org to a tenant is platform-owned registry data (tenants.owner_org_id); an
-- org's own rows name no tenant.
-- paridyum:scope org
-- paridyum:plane regional
CREATE TABLE org_members (
    id          uuid PRIMARY KEY,
    org_id      uuid NOT NULL,
    tenant_id   uuid NOT NULL,
    subject     text NOT NULL,
    created_at  timestamptz NOT NULL
);
ALTER TABLE org_members ENABLE ROW LEVEL SECURITY;
ALTER TABLE org_members FORCE ROW LEVEL SECURITY;
CREATE POLICY org_members_org ON org_members
    USING (org_id = paridyum.current_org());
