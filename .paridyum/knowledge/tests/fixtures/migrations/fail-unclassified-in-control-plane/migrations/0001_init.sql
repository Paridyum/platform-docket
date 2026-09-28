-- db.no_personal_data_in_the_control_plane — an unclassified column in the control plane.
-- Absence is not permission (GLO-0005, one level over): the gate cannot tell an unclassified
-- column from a residency breach, so it refuses to guess.
-- paridyum:scope platform
-- paridyum:plane control
CREATE TABLE orgs (
    id          uuid PRIMARY KEY,         -- internal
    region      text NOT NULL,            -- internal
    name        text NOT NULL,
    created_at  timestamptz NOT NULL      -- internal
);
