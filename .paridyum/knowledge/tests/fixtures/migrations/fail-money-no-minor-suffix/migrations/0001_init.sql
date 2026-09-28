-- ADR-0033 §2: the _minor suffix is the whole mitigation. A column named `amount` holding minor
-- units is indistinguishable from one holding a decimal, and the difference is a factor of one
-- hundred applied to somebody's invoice.
-- paridyum:scope platform
-- paridyum:plane control
CREATE TABLE prices (
    id          uuid PRIMARY KEY,  -- internal
    amount      bigint NOT NULL,  -- internal
    currency    char(3) NOT NULL,  -- internal
    created_at  timestamptz NOT NULL  -- internal
);
