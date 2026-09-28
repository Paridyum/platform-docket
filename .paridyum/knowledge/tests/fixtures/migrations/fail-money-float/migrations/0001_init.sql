-- ADR-0033: binary floating point cannot represent 0.10, so a total accumulated here is wrong
-- by an amount that grows with the number of line items, silently.
-- paridyum:scope platform
-- paridyum:plane control
CREATE TABLE prices (
    id            uuid PRIMARY KEY,  -- internal
    amount_minor  double precision NOT NULL,  -- internal
    currency      char(3) NOT NULL,  -- internal
    created_at    timestamptz NOT NULL  -- internal
);
