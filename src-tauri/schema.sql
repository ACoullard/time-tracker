-- Current database schema.
-- Applied via `CREATE ... IF NOT EXISTS`

CREATE TABLE IF NOT EXISTS intervals (
    id       INTEGER PRIMARY KEY,
    start_ms INTEGER NOT NULL,
    end_ms   INTEGER  -- NULL while the interval is running
);

CREATE INDEX IF NOT EXISTS idx_intervals_start ON intervals(start_ms);
