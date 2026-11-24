CREATE TABLE worlds (
  id UUID PRIMARY KEY,
  tick BIGINT NOT NULL CHECK (tick >= 0),
  snapshot JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE world_events (
  world_id UUID NOT NULL REFERENCES worlds(id) ON DELETE CASCADE,
  tick BIGINT NOT NULL CHECK (tick >= 0),
  sequence BIGINT NOT NULL CHECK (sequence >= 0),
  event JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (world_id,tick,sequence)
);
CREATE INDEX world_events_world_tick_idx ON world_events(world_id,tick);
