CREATE TABLE sump_events
(
  id INTEGER PRIMARY KEY NOT NULL,
  kind TEXT NOT NULL,
  info TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX idx_sump_events_on_created_at ON sump_events (created_at);
CREATE INDEX idx_sump_events_on_kind_created_at ON sump_events (kind, created_at);
