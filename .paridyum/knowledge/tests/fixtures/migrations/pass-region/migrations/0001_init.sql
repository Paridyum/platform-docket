-- paridyum:scope platform
-- paridyum:plane control
CREATE TABLE tenants (
    id            uuid PRIMARY KEY,         -- internal
    owner_org_id  uuid NOT NULL,            -- internal
    slug          text NOT NULL,            -- public
    created_at    timestamptz NOT NULL      -- internal
);

ALTER TABLE tenants ADD COLUMN region text;   -- internal
UPDATE tenants SET region = 'in' WHERE region IS NULL;
ALTER TABLE tenants ALTER COLUMN region SET NOT NULL;
ALTER TABLE tenants ADD CONSTRAINT tenants_region_is_live CHECK (region IN ('in'));
CREATE TRIGGER tenants_region_immutable
  BEFORE UPDATE ON tenants
  FOR EACH ROW
  WHEN (NEW.region IS DISTINCT FROM OLD.region)
  EXECUTE FUNCTION paridyum.forbid_region_move();

-- ADR-0122 D4: a tenant declares its content locales and defaults in one tenant-scoped, regional
-- row, and nothing is defaulted — a default would be a declaration nobody made (ADR-0034 D2).
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
