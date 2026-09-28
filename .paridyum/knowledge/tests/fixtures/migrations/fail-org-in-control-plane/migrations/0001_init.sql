-- db.org_scoped_tables_are_regional — an org's own rows declared into the control plane.
-- An org's rows are its people (RFC-0017), and they live in the org's region. Every column is
-- classified so that the personal-data check has nothing to say: only the plane is wrong.
-- paridyum:scope org
-- paridyum:plane control
CREATE TABLE org_members (
    id          uuid PRIMARY KEY,         -- internal
    org_id      uuid NOT NULL,            -- internal
    role        text NOT NULL,            -- internal
    created_at  timestamptz NOT NULL      -- internal
);
ALTER TABLE org_members ENABLE ROW LEVEL SECURITY;
ALTER TABLE org_members FORCE ROW LEVEL SECURITY;
CREATE POLICY org_members_org ON org_members
    USING (org_id = paridyum.current_org());
