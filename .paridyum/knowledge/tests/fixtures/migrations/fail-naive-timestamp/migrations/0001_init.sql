-- C-0005 and STD-0010 §2: bare `timestamp` looks like it carries a zone and does not. It is
-- the spelling most ORMs and most examples emit.
-- paridyum:scope platform
-- paridyum:plane control
CREATE TABLE regions (
    id          uuid PRIMARY KEY,  -- internal
    code        text NOT NULL,  -- internal
    created_at  timestamp NOT NULL  -- internal
);
