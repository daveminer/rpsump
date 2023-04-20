CREATE TABLE sump_event
(
  id INTEGER PRIMARY KEY NOT NULL,
  kind TEXT NOT NULL,
  info TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX idx_sump_event_on_created_at ON sump_event (created_at);
CREATE INDEX idx_sump_event_on_kind_created_at ON sump_event (kind, created_at);
