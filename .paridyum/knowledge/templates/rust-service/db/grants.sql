-- Per-service database privileges (ANTI-0001, ANTI-0003).
--
-- THIS FILE IS THE ENFORCEMENT. manifest.yaml's `owns.tables` is a declaration of
-- intent; nothing about a YAML file stops code writing to a table it did not declare.
-- These grants do, at the database, at connect time, with no reliance on review.
--
-- The rule is Martin Thompson's Single Writer Principle: data is owned by one execution
-- context. Two services writing one table means every consistency and locking problem
-- that principle exists to avoid, now spread across two deployables with independent
-- release cadences — and it is effectively permanent once shipped.
--
-- Industry does not enforce this with metadata. It enforces it with exactly this: a
-- distinct database user per service, granted write access only on what it owns.
--
-- KEEP THIS IN STEP WITH manifest.yaml. `domain.entity_has_single_owner` verifies the
-- two agree. If they drift, the declaration is fiction and the grants are the truth.

-- ---------------------------------------------------------------------------
-- 1. The service's own role. One role per service, never shared, never a superuser.
-- ---------------------------------------------------------------------------
-- Password comes from the secret store at deploy time, never from this file and never
-- from the repository (C-0004, REQ-0001). The placeholder is substituted by the
-- deployment, and `repo.secret_scan_clean` will fail the build if a real one lands here.
CREATE ROLE svc_billing LOGIN PASSWORD :'svc_billing_password';

-- ---------------------------------------------------------------------------
-- 2. Baseline: no implicit access to anything.
-- ---------------------------------------------------------------------------
-- Postgres grants CREATE and USAGE on `public` to every role by default, which quietly
-- undermines the whole arrangement. Revoke it explicitly.
REVOKE ALL ON SCHEMA public FROM PUBLIC;
REVOKE ALL ON ALL TABLES IN SCHEMA public FROM PUBLIC;

GRANT USAGE ON SCHEMA public TO svc_billing;

-- ---------------------------------------------------------------------------
-- 3. Write access ONLY on the tables this service declares it owns.
-- ---------------------------------------------------------------------------
-- One line per entry in manifest.yaml `owns.tables`. Adding a table here without adding
-- it there is the drift the check catches.
GRANT SELECT, INSERT, UPDATE, DELETE ON invoices      TO svc_billing;
GRANT SELECT, INSERT, UPDATE, DELETE ON invoice_lines TO svc_billing;

GRANT USAGE, SELECT ON SEQUENCE invoices_id_seq      TO svc_billing;
GRANT USAGE, SELECT ON SEQUENCE invoice_lines_id_seq TO svc_billing;

-- ---------------------------------------------------------------------------
-- 4. Deliberately absent: any grant on a table this service does not own.
-- ---------------------------------------------------------------------------
-- If this service needs another's data, the answer is that service's API, or subscribing
-- to its events and keeping your own projection (ANTI-0003). It is NOT a SELECT here.
--
-- A read looks harmless and is not: it is a dependency no manifest declares, so the
-- dependency graph the org reasons about stops matching the one that exists. The owner
-- then cannot refactor a schema they believe is private, and the breakage lands in
-- production at a time chosen by someone else's deploy.
--
-- The one legitimate exception is a read replica the owner PUBLISHES as a contract,
-- declared in their manifest under `exposes.database`. That is an interface that happens
-- to speak SQL, versioned and owned like any other. The distinction is agreement: a
-- published replica carries an obligation the owner accepted. A reach into a schema
-- imposes one nobody did.

-- ---------------------------------------------------------------------------
-- 5. Migrations run as a different role.
-- ---------------------------------------------------------------------------
-- The service role must not hold DDL. Otherwise an ORM's "helpful" auto-migration, or a
-- compromised service, can reshape the schema at runtime. Migrations are a deploy-time
-- step with its own credential and its own audit trail (ADR-0010).
CREATE ROLE svc_billing_migrator LOGIN PASSWORD :'svc_billing_migrator_password';
GRANT USAGE, CREATE ON SCHEMA public TO svc_billing_migrator;
GRANT ALL ON ALL TABLES IN SCHEMA public TO svc_billing_migrator;
