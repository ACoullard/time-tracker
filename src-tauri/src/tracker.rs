use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

pub const SCHEMA: &str = include_str!("../schema.sql");

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct Interval {
    pub id: i64,
    pub start_ms: i64,
    pub end_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct RangeTotal {
    pub total_ms: i64,
    pub most_recent: Option<Interval>,
}

#[derive(Debug)]
pub enum DbError {
    AlreadyRunning,
    NoneRunning,
    Sqlite(rusqlite::Error),
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyRunning => write!(f, "an interval is already running"),
            Self::NoneRunning => write!(f, "no interval is running"),
            Self::Sqlite(e) => write!(f, "sqlite error: {e}"),
        }
    }
}

impl std::error::Error for DbError {}

impl From<rusqlite::Error> for DbError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sqlite(e)
    }
}

pub fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(SCHEMA)
}

pub fn get_current_interval(conn: &Connection) -> rusqlite::Result<Option<i64>> {
    conn.query_row(
        "SELECT start_ms FROM intervals WHERE end_ms IS NULL ORDER BY start_ms DESC LIMIT 1",
        [],
        |row| row.get(0),
    )
    .optional()
}

pub fn begin_interval(conn: &Connection, now_ms: i64) -> Result<i64, DbError> {
    if get_current_interval(conn)?.is_some() {
        return Err(DbError::AlreadyRunning);
    }
    conn.execute(
        "INSERT INTO intervals (start_ms) VALUES (?1)",
        params![now_ms],
    )?;
    Ok(now_ms)
}

pub fn end_interval(conn: &Connection, now_ms: i64) -> Result<(), DbError> {
    let rows = conn.execute(
        "UPDATE intervals SET end_ms = ?1 WHERE end_ms IS NULL",
        params![now_ms],
    )?;
    if rows == 0 {
        return Err(DbError::NoneRunning);
    }
    Ok(())
}

pub fn get_time_range_total(
    conn: &Connection,
    from_ms: i64,
    to_ms: i64,
) -> rusqlite::Result<RangeTotal> {
    let mut stmt = conn.prepare(
        "SELECT id, start_ms, end_ms FROM intervals
         WHERE start_ms < ?2 AND (end_ms IS NULL OR end_ms > ?1)",
    )?;
    let rows = stmt.query_map(params![from_ms, to_ms], |row| {
        Ok(Interval {
            id: row.get(0)?,
            start_ms: row.get(1)?,
            end_ms: row.get(2)?,
        })
    })?;

    let mut total_ms: i64 = 0;
    let mut most_recent: Option<Interval> = None;
    for row in rows {
        let interval = row?;
        if let Some(end) = interval.end_ms {
            total_ms += (end.min(to_ms) - interval.start_ms.max(from_ms)).max(0);
        }
        if most_recent
            .as_ref()
            .map_or(true, |m| interval.start_ms > m.start_ms)
        {
            most_recent = Some(interval);
        }
    }
    Ok(RangeTotal {
        total_ms,
        most_recent,
    })
}

pub fn get_most_recent_interval(conn: &Connection) -> rusqlite::Result<Option<Interval>> {
    conn.query_row(
        "SELECT id, start_ms, end_ms FROM intervals ORDER BY start_ms DESC LIMIT 1",
        [],
        |row| {
            Ok(Interval {
                id: row.get(0)?,
                start_ms: row.get(1)?,
                end_ms: row.get(2)?,
            })
        },
    )
    .optional()
}

pub fn get_intervals(
    conn: &Connection,
    from_ms: i64,
    to_ms: i64,
) -> rusqlite::Result<Vec<Interval>> {
    let mut stmt = conn.prepare(
        "SELECT id, start_ms, end_ms FROM intervals
         WHERE start_ms < ?2 AND (end_ms IS NULL OR end_ms > ?1)
         ORDER BY start_ms ASC",
    )?;
    let rows = stmt.query_map(params![from_ms, to_ms], |row| {
        Ok(Interval {
            id: row.get(0)?,
            start_ms: row.get(1)?,
            end_ms: row.get(2)?,
        })
    })?;
    rows.collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn current_is_none_when_idle() {
        let conn = setup();
        assert_eq!(get_current_interval(&conn).unwrap(), None);
    }

    #[test]
    fn begin_creates_running_interval() {
        let conn = setup();
        begin_interval(&conn, 1000).unwrap();
        assert_eq!(get_current_interval(&conn).unwrap(), Some(1000));
    }

    #[test]
    fn begin_when_running_errors() {
        let conn = setup();
        begin_interval(&conn, 1000).unwrap();
        assert!(matches!(
            begin_interval(&conn, 2000),
            Err(DbError::AlreadyRunning)
        ));
    }

    #[test]
    fn end_closes_running_interval() {
        let conn = setup();
        begin_interval(&conn, 1000).unwrap();
        end_interval(&conn, 5000).unwrap();
        assert_eq!(get_current_interval(&conn).unwrap(), None);
        let intervals = get_intervals(&conn, 0, 10_000).unwrap();
        assert_eq!(intervals.len(), 1);
        assert_eq!(intervals[0].end_ms, Some(5000));
    }

    #[test]
    fn end_when_idle_errors() {
        let conn = setup();
        assert!(matches!(end_interval(&conn, 1000), Err(DbError::NoneRunning)));
    }

    #[test]
    fn get_intervals_filters_by_overlap() {
        let conn = setup();
        begin_interval(&conn, 1000).unwrap();
        end_interval(&conn, 2000).unwrap();
        begin_interval(&conn, 5000).unwrap();
        end_interval(&conn, 7000).unwrap();
        begin_interval(&conn, 8000).unwrap();

        let result = get_intervals(&conn, 4000, 9000).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].start_ms, 5000);
        assert_eq!(result[1].start_ms, 8000);
        assert_eq!(result[1].end_ms, None);
    }

    #[test]
    fn get_intervals_excludes_non_overlapping() {
        let conn = setup();
        begin_interval(&conn, 1000).unwrap();
        end_interval(&conn, 2000).unwrap();
        let result = get_intervals(&conn, 5000, 6000).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn range_total_empty() {
        let conn = setup();
        let r = get_time_range_total(&conn, 0, 10_000).unwrap();
        assert_eq!(r.total_ms, 0);
        assert_eq!(r.most_recent, None);
    }

    #[test]
    fn range_total_closed_inside_window() {
        let conn = setup();
        begin_interval(&conn, 2000).unwrap();
        end_interval(&conn, 5000).unwrap();
        let r = get_time_range_total(&conn, 0, 10_000).unwrap();
        assert_eq!(r.total_ms, 3000);
        let m = r.most_recent.unwrap();
        assert_eq!(m.start_ms, 2000);
        assert_eq!(m.end_ms, Some(5000));
    }

    #[test]
    fn range_total_clipped_left() {
        let conn = setup();
        begin_interval(&conn, 500).unwrap();
        end_interval(&conn, 3000).unwrap();
        let r = get_time_range_total(&conn, 1000, 10_000).unwrap();
        assert_eq!(r.total_ms, 2000);
        let m = r.most_recent.unwrap();
        assert_eq!(m.start_ms, 500);
        assert_eq!(m.end_ms, Some(3000));
    }

    #[test]
    fn range_total_clipped_right() {
        let conn = setup();
        begin_interval(&conn, 8000).unwrap();
        end_interval(&conn, 12_000).unwrap();
        let r = get_time_range_total(&conn, 0, 10_000).unwrap();
        assert_eq!(r.total_ms, 2000);
        let m = r.most_recent.unwrap();
        assert_eq!(m.start_ms, 8000);
        assert_eq!(m.end_ms, Some(12_000));
    }

    #[test]
    fn range_total_closed_spans_window() {
        let conn = setup();
        begin_interval(&conn, 500).unwrap();
        end_interval(&conn, 20_000).unwrap();
        let r = get_time_range_total(&conn, 1000, 10_000).unwrap();
        assert_eq!(r.total_ms, 9000);
        let m = r.most_recent.unwrap();
        assert_eq!(m.start_ms, 500);
        assert_eq!(m.end_ms, Some(20_000));
    }

    #[test]
    fn range_total_non_overlapping_excluded() {
        let conn = setup();
        begin_interval(&conn, 1000).unwrap();
        end_interval(&conn, 2000).unwrap();
        let r = get_time_range_total(&conn, 5000, 10_000).unwrap();
        assert_eq!(r.total_ms, 0);
        assert_eq!(r.most_recent, None);
    }

    #[test]
    fn range_total_running_inside_window() {
        let conn = setup();
        begin_interval(&conn, 3000).unwrap();
        let r = get_time_range_total(&conn, 0, 10_000).unwrap();
        assert_eq!(r.total_ms, 0);
        let m = r.most_recent.unwrap();
        assert_eq!(m.start_ms, 3000);
        assert_eq!(m.end_ms, None);
    }

    #[test]
    fn range_total_running_clipped_left() {
        let conn = setup();
        begin_interval(&conn, 500).unwrap();
        let r = get_time_range_total(&conn, 1000, 10_000).unwrap();
        assert_eq!(r.total_ms, 0);
        let m = r.most_recent.unwrap();
        assert_eq!(m.start_ms, 500);
        assert_eq!(m.end_ms, None);
    }

    #[test]
    fn range_total_mix_closed_and_running() {
        let conn = setup();
        begin_interval(&conn, 1000).unwrap();
        end_interval(&conn, 2500).unwrap();
        begin_interval(&conn, 4000).unwrap();
        end_interval(&conn, 5000).unwrap();
        begin_interval(&conn, 7000).unwrap();
        let r = get_time_range_total(&conn, 0, 10_000).unwrap();
        assert_eq!(r.total_ms, 2500);
        let m = r.most_recent.unwrap();
        assert_eq!(m.start_ms, 7000);
        assert_eq!(m.end_ms, None);
    }
}
