-- db.tenant_scoped_tables_declare_tenancy — a platform table carrying a discriminator name.
-- STD-0023 rule 1: a platform table references an org by a named key (owner_org_id), never by
-- org_id, so that a scope and a relationship cannot be mistaken for each other.
-- paridyum:scope platform
-- paridyum:plane regional
CREATE TABLE plan_catalogue (
    id          uuid PRIMARY KEY,
    org_id      uuid NOT NULL,
    created_at  timestamptz NOT NULL
);
