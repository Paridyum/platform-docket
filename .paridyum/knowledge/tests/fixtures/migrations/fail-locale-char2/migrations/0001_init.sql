-- STD-0010 §1: char(2) cannot hold `en-IN`, so the region subtag is discarded on write and no
-- later migration recovers which of en-IN, en-US and en-GB each row meant.
-- paridyum:scope platform
-- paridyum:plane control
CREATE TABLE locales (
    id          uuid PRIMARY KEY,  -- internal
    locale      char(2) NOT NULL,  -- internal
    created_at  timestamptz NOT NULL  -- internal
);
