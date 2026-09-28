-- db.no_personal_data_in_the_control_plane — a pii column in the control plane (ADR-0117 D3).
-- RFC-0014 cost 2: "a residency breach that every check reports as green." Every other rule holds.
-- paridyum:scope platform
-- paridyum:plane control
CREATE TABLE orgs (
    id            uuid PRIMARY KEY,         -- internal
    region        text NOT NULL,            -- internal
    contact_email text NOT NULL,            -- pii
    created_at    timestamptz NOT NULL      -- internal
);
