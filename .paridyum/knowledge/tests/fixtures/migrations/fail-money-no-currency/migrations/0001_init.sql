-- ADR-0033: a monetary column with no currency beside it. Recoverable only until the table
-- holds rows in two currencies, at which point nothing in the data says which is which.
-- paridyum:scope platform
-- paridyum:plane control
CREATE TABLE prices (
    id            uuid PRIMARY KEY,  -- internal
    amount_minor  bigint NOT NULL,  -- internal
    created_at    timestamptz NOT NULL  -- internal
);
