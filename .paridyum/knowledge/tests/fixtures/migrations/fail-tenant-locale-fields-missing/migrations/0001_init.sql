-- db.tenant_declares_its_locale_fields — a tenant registry with no declared locales (ADR-0122 D4).
-- Every region rule holds and every control-plane column is classified; only the tenant's
-- content locales and defaults are missing, so each tenant would inherit one nobody chose.

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
