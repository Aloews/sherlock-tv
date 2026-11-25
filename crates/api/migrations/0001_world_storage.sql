CREATE TABLE worlds (
    id BIGSERIAL PRIMARY KEY,
    seed BIGINT NOT NULL,
    ruleset_version INTEGER NOT NULL CHECK (ruleset_version > 0),
    tick BIGINT NOT NULL DEFAULT 0 CHECK (tick >= 0),
    current_snapshot JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE world_snapshots (
    world_id BIGINT NOT NULL REFERENCES worlds(id) ON DELETE CASCADE,
    tick BIGINT NOT NULL CHECK (tick >= 0),
    ruleset_version INTEGER NOT NULL CHECK (ruleset_version > 0),
    state JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (world_id, tick)
);

CREATE TABLE world_events (
    world_id BIGINT NOT NULL REFERENCES worlds(id) ON DELETE CASCADE,
    sequence BIGINT NOT NULL CHECK (sequence >= 0),
    tick BIGINT NOT NULL CHECK (tick >= 0),
    event_type TEXT NOT NULL,
    event JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (world_id, sequence)
);

CREATE INDEX world_events_tick_idx ON world_events (world_id, tick, sequence);
