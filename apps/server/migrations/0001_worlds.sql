CREATE TABLE worlds (
    id UUID PRIMARY KEY,
    seed BIGINT NOT NULL,
    ruleset_version INTEGER NOT NULL,
    tick BIGINT NOT NULL DEFAULT 0,
    snapshot JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE world_events (
    world_id UUID NOT NULL REFERENCES worlds(id) ON DELETE CASCADE,
    sequence BIGINT NOT NULL,
    tick BIGINT NOT NULL,
    event JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (world_id, sequence)
);

CREATE INDEX world_events_tick_idx ON world_events(world_id, tick, sequence);
