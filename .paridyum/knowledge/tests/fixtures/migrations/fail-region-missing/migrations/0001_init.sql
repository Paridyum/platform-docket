-- paridyum:scope platform
-- paridyum:plane control
CREATE TABLE tenants (
    id          uuid PRIMARY KEY,
    slug        text NOT NULL,
    created_at  timestamptz NOT NULL
);
