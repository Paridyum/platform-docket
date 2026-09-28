-- STD-0010 §3: SQL_ASCII is not "ASCII only", it is no encoding at all. Every insert succeeds
-- and the corruption appears at the first non-trivial comparison. Remediation is a dump and
-- restore of production, which is why this is checked before a database exists.
CREATE DATABASE paridyum ENCODING 'SQL_ASCII' LC_COLLATE = 'C';

-- paridyum:scope platform
-- paridyum:plane control
CREATE TABLE regions (
    id          uuid PRIMARY KEY,  -- internal
    code        text NOT NULL,  -- internal
    created_at  timestamptz NOT NULL  -- internal
);
