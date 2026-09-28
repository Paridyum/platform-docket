-- db.tenant_scoped_tables_are_regional — a tenant's rows declared into the control plane.
-- This is the silent residency breach RFC-0014 decision 3 exists to catch: every other check
-- in this gate passes on this file. The tenancy column is present, RLS is ENABLEd and FORCEd,
-- the policy names both predicates. Only the plane is wrong, and the data has left its region.

-- paridyum:scope account
-- paridyum:plane control
CREATE TABLE invoices (
    id          uuid PRIMARY KEY,  -- internal
    tenant_id   uuid NOT NULL,  -- internal
    account_id  uuid NOT NULL,  -- internal
    created_at  timestamptz NOT NULL  -- internal
);
ALTER TABLE invoices ENABLE ROW LEVEL SECURITY;
ALTER TABLE invoices FORCE ROW LEVEL SECURITY;
CREATE POLICY invoices_scoped ON invoices
    USING (tenant_id = paridyum.current_tenant()
           AND (paridyum.has_operator_scope() OR account_id = paridyum.current_account()));
