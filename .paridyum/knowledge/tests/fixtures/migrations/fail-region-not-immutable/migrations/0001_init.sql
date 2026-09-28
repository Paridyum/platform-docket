-- paridyum:scope platform
-- paridyum:plane control
CREATE TABLE tenants (
    id          uuid PRIMARY KEY,  -- internal
    slug        text NOT NULL,  -- internal
    created_at  timestamptz NOT NULL  -- internal
);

ALTER TABLE tenants ADD COLUMN region text;  -- internal
ALTER TABLE tenants ALTER COLUMN region SET NOT NULL;
ALTER TABLE tenants ADD CONSTRAINT tenants_region_is_live CHECK (region IN ('in'));

-- ADR-0122 D4 — present so this fixture fails only for the defect it plants.
-- paridyum:scope tenant
-- paridyum:plane regional
CREATE TABLE tenant_settings (
    tenant_id          uuid PRIMARY KEY,
    supported_locales  text[] NOT NULL,
    content_locale     text NOT NULL,
    locale             text NOT NULL,
    timezone           text NOT NULL,
    currency           char(3) NOT NULL,
    created_at         timestamptz NOT NULL
);
ALTER TABLE tenant_settings ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_settings FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_settings_tenant ON tenant_settings
    USING (tenant_id = paridyum.current_tenant());
