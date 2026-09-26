-- paridyum:scope platform
-- paridyum:plane control
CREATE TABLE tenants (
    id          uuid PRIMARY KEY,
    slug        text NOT NULL,
    created_at  timestamptz NOT NULL
);

ALTER TABLE tenants ADD COLUMN region text;
ALTER TABLE tenants ALTER COLUMN region SET NOT NULL;
ALTER TABLE tenants ADD CONSTRAINT tenants_region_is_live CHECK (region IN ('in','eu'));
CREATE TRIGGER tenants_region_immutable
  BEFORE UPDATE ON tenants
  FOR EACH ROW
  WHEN (NEW.region IS DISTINCT FROM OLD.region)
  EXECUTE FUNCTION paridyum.forbid_region_move();
