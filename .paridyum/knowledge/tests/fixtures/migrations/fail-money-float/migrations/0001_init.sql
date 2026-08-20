-- ADR-0033: binary floating point cannot represent 0.10, so a total accumulated here is wrong
-- by an amount that grows with the number of line items, silently.
-- paridyum:scope platform
-- paridyum:plane control
CREATE TABLE prices (
    id            uuid PRIMARY KEY,
    amount_minor  double precision NOT NULL,
    currency      char(3) NOT NULL,
    created_at    timestamptz NOT NULL
);
