-- db.localized_content_matches_its_tenant_locales — one tenant's languages written as everyone's.
-- The shape migrations 0007 and 0016 had: a helper that constraints call, with the locale set
-- written into it (ADR-0122 D4).
CREATE OR REPLACE FUNCTION paridyum.is_localized_text(v jsonb) RETURNS boolean
  LANGUAGE sql IMMUTABLE AS $$
    SELECT jsonb_typeof(v) = 'object' AND v ? 'en' AND v ? 'ar'
  $$;

-- paridyum:scope tenant
-- paridyum:plane regional
CREATE TABLE postings (
    id          uuid PRIMARY KEY,
    tenant_id   uuid NOT NULL,
    title       jsonb NOT NULL,
    created_at  timestamptz NOT NULL,
    CONSTRAINT postings_title_l10n CHECK (paridyum.is_localized_text(title))
);
ALTER TABLE postings ENABLE ROW LEVEL SECURITY;
ALTER TABLE postings FORCE ROW LEVEL SECURITY;
CREATE POLICY postings_tenant ON postings
    USING (tenant_id = paridyum.current_tenant());
