-- db.no_org_discriminator_on_business_data — a tenant's rows naming their owner org.
-- This is ADR-0115 D9's deferred third silo arriving by accident: the column is not org_id, so the
-- scope check is satisfied, and it still makes business data know its org.
-- paridyum:scope tenant
-- paridyum:plane regional
CREATE TABLE plans (
    id            uuid PRIMARY KEY,
    tenant_id     uuid NOT NULL,
    owner_org_id  uuid NOT NULL,
    created_at    timestamptz NOT NULL
);
ALTER TABLE plans ENABLE ROW LEVEL SECURITY;
ALTER TABLE plans FORCE ROW LEVEL SECURITY;
CREATE POLICY plans_tenant ON plans
    USING (tenant_id = paridyum.current_tenant());
