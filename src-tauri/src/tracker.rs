use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

pub const SCHEMA: &str = include_str!("../schema.sql");

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct Interval {
    pub id: i64,
    pub start_ms: i64,
    pub end_ms: Option<i64>,
}

impl Interval {
    pub fn is_running(&self) -> bool {
        self.end_ms.is_none()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(tag = "state")]
pub enum TimerState {
    Empty,
    Running { start_ms: i64 },
    Paused { last_duration_ms: i64 },
}

pub fn timer_state(conn: &Connection) -> rusqlite::Result<TimerState> {
    Ok(match get_most_recent_interval(conn)? {
        None => TimerState::Empty,
        Some(i) if i.is_running() => TimerState::Running { start_ms: i.start_ms },
        Some(i) => TimerState::Paused {
            last_duration_ms: i.end_ms.unwrap() - i.start_ms,
        },
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct DailyGoal {
    pub day: String,
    pub goal_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct DailyTotal {
    pub day: String,
    pub total_ms: i64,
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
    NotFound,
    Sqlite(rusqlite::Error),
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyRunning => write!(f, "an interval is already running"),
            Self::NoneRunning => write!(f, "no interval is running"),
            Self::NotFound => write!(f, "interval not found"),
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

pub fn begin_interval(conn: &Connection, now_ms: i64) -> Result<i64, DbError> {
    if matches!(timer_state(conn)?, TimerState::Running { .. }) {
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

pub fn update_interval(
    conn: &Connection,
    id: i64,
    start_ms: i64,
    end_ms: Option<i64>,
) -> Result<(), DbError> {
    let rows = conn.execute(
        "UPDATE intervals SET start_ms = ?1, end_ms = ?2 WHERE id = ?3",
        params![start_ms, end_ms, id],
    )?;
    if rows == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

pub fn delete_interval(conn: &Connection, id: i64) -> Result<(), DbError> {
    let rows = conn.execute("DELETE FROM intervals WHERE id = ?1", params![id])?;
    if rows == 0 {
        return Err(DbError::NotFound);
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
            .is_none_or(|m| interval.start_ms > m.start_ms)
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

pub fn get_current_goal(conn: &Connection) -> rusqlite::Result<Option<DailyGoal>> {
    conn.query_row(
        "SELECT day, goal_ms FROM daily_goals ORDER BY day DESC LIMIT 1",
        [],
        |row| Ok(DailyGoal { day: row.get(0)?, goal_ms: row.get(1)? }),
    )
    .optional()
}

pub fn set_daily_goal(conn: &Connection, day: &str, goal_ms: i64) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO daily_goals (day, goal_ms) VALUES (?1, ?2)
         ON CONFLICT(day) DO UPDATE SET goal_ms = excluded.goal_ms",
        params![day, goal_ms],
    )?;
    Ok(())
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

// Closed (start, end) intervals overlapping `[from_ms, to_ms)`, ordered by start
// ascending. Running intervals are excluded.
fn closed_intervals_in_range(
    conn: &Connection,
    from_ms: i64,
    to_ms: i64,
) -> rusqlite::Result<Vec<(i64, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT start_ms, end_ms FROM intervals
         WHERE start_ms < ?2 AND end_ms IS NOT NULL AND end_ms > ?1
         ORDER BY start_ms ASC",
    )?;
    let rows = stmt
        .query_map(params![from_ms, to_ms], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

// Every goal dated on or before `last_day`, ordered by day ascending.
fn goals_through(conn: &Connection, last_day: &str) -> rusqlite::Result<Vec<DailyGoal>> {
    let mut stmt =
        conn.prepare("SELECT day, goal_ms FROM daily_goals WHERE day <= ?1 ORDER BY day ASC")?;
    let rows = stmt
        .query_map(params![last_day], |row| {
            Ok(DailyGoal { day: row.get(0)?, goal_ms: row.get(1)? })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

pub fn get_daily_totals(conn: &Connection, days: &[(String, i64, i64)]) -> rusqlite::Result<Vec<DailyTotal>> {
    if days.is_empty() {
        return Ok(vec![]);
    }
    let first_start = days[0].1;
    let last_end = days[days.len() - 1].2;
    let closed = closed_intervals_in_range(conn, first_start, last_end)?;

    let mut result = Vec::with_capacity(days.len());
    for (key, day_start, day_end) in days {
        let mut total_ms: i64 = 0;
        for &(start, end) in &closed {
            if start >= *day_end || end <= *day_start {
                continue;
            }
            total_ms += (end.min(*day_end) - start.max(*day_start)).max(0);
        }
        result.push(DailyTotal { day: key.clone(), total_ms });
    }
    Ok(result)
}

pub fn get_applicable_goals(conn: &Connection, day_keys: &[String]) -> rusqlite::Result<Vec<DailyGoal>> {
    if day_keys.is_empty() {
        return Ok(vec![]);
    }
    let last_key = &day_keys[day_keys.len() - 1];
    let all_goals = goals_through(conn, last_key)?;

    let mut result = Vec::with_capacity(day_keys.len());
    let mut goal_ptr = 0usize;
    let mut current_goal_ms: i64 = 0;
    for key in day_keys {
        while goal_ptr < all_goals.len() && all_goals[goal_ptr].day.as_str() <= key.as_str() {
            current_goal_ms = all_goals[goal_ptr].goal_ms;
            goal_ptr += 1;
        }
        result.push(DailyGoal { day: key.clone(), goal_ms: current_goal_ms });
    }
    Ok(result)
}

// Counts consecutive days ending at the last entry in `days` where total_ms >= goal_ms.
// Days with no goal set (goal_ms = 0) break the streak.
pub fn get_streak(conn: &Connection, days: &[(String, i64, i64)]) -> rusqlite::Result<u32> {
    if days.is_empty() {
        return Ok(0);
    }
    let range_start = days[0].1;
    let range_end = days[days.len() - 1].2;
    let intervals = closed_intervals_in_range(conn, range_start, range_end)?;

    let last_key = days[days.len() - 1].0.as_str();
    let goals = goals_through(conn, last_key)?;

    // All three pointers start past the end and only ever walk left.
    let mut hi = intervals.len(); // count of intervals starting before the day's end
    let mut lo = intervals.len(); // first interval ending after the day's start
    let mut gi = goals.len(); // count of goals dated on or before the day

    let mut streak = 0u32;
    for (key, day_start, day_end) in days.iter().rev() {
        // Borders of the day's interval window: [lo, hi).
        while hi > 0 && intervals[hi - 1].0 >= *day_end {
            hi -= 1;
        }
        while lo > 0 && intervals[lo - 1].1 > *day_start {
            lo -= 1;
        }
        // Most recent goal set on or before this day; absent goal => 0.
        while gi > 0 && goals[gi - 1].day.as_str() > key.as_str() {
            gi -= 1;
        }
        let goal_ms = if gi > 0 { goals[gi - 1].goal_ms } else { 0 };
        if goal_ms == 0 {
            break;
        }

        let mut total_ms: i64 = 0;
        for &(start, end) in &intervals[lo..hi] {
            total_ms += (end.min(*day_end) - start.max(*day_start)).max(0);
        }
        if total_ms < goal_ms {
            break;
        }
        streak += 1;
    }
    Ok(streak)
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
    fn most_recent_is_none_when_idle() {
        let conn = setup();
        assert_eq!(get_most_recent_interval(&conn).unwrap(), None);
    }

    #[test]
    fn begin_creates_running_interval() {
        let conn = setup();
        begin_interval(&conn, 1000).unwrap();
        let i = get_most_recent_interval(&conn).unwrap().unwrap();
        assert!(i.is_running());
        assert_eq!(i.start_ms, 1000);
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
        let i = get_most_recent_interval(&conn).unwrap().unwrap();
        assert!(!i.is_running());
        assert_eq!(i.end_ms, Some(5000));
        let intervals = get_intervals(&conn, 0, 10_000).unwrap();
        assert_eq!(intervals.len(), 1);
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
    fn update_interval_changes_times() {
        let conn = setup();
        begin_interval(&conn, 1000).unwrap();
        end_interval(&conn, 5000).unwrap();
        let original = get_most_recent_interval(&conn).unwrap().unwrap();
        update_interval(&conn, original.id, 2000, Some(6000)).unwrap();
        let updated = get_most_recent_interval(&conn).unwrap().unwrap();
        assert_eq!(updated.start_ms, 2000);
        assert_eq!(updated.end_ms, Some(6000));
    }

    #[test]
    fn update_interval_not_found_errors() {
        let conn = setup();
        assert!(matches!(
            update_interval(&conn, 999, 1000, Some(2000)),
            Err(DbError::NotFound)
        ));
    }

    #[test]
    fn goal_none_on_empty_table() {
        let conn = setup();
        assert_eq!(get_current_goal(&conn).unwrap(), None);
    }

    #[test]
    fn goal_insert_and_retrieve() {
        let conn = setup();
        set_daily_goal(&conn, "2024-01-15", 3_600_000).unwrap();
        let g = get_current_goal(&conn).unwrap().unwrap();
        assert_eq!(g.day, "2024-01-15");
        assert_eq!(g.goal_ms, 3_600_000);
    }

    #[test]
    fn goal_same_day_overwrites() {
        let conn = setup();
        set_daily_goal(&conn, "2024-01-15", 3_600_000).unwrap();
        set_daily_goal(&conn, "2024-01-15", 7_200_000).unwrap();
        let g = get_current_goal(&conn).unwrap().unwrap();
        assert_eq!(g.goal_ms, 7_200_000);
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM daily_goals", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn goal_different_days_returns_most_recent() {
        let conn = setup();
        set_daily_goal(&conn, "2024-01-14", 3_600_000).unwrap();
        set_daily_goal(&conn, "2024-01-15", 5_400_000).unwrap();
        let g = get_current_goal(&conn).unwrap().unwrap();
        assert_eq!(g.day, "2024-01-15");
        assert_eq!(g.goal_ms, 5_400_000);
    }

    // Builds a contiguous ascending day list of `n` days, each `len` ms wide,
    // starting at `start`. Keys are synthetic but sort chronologically.
    fn day_list(start: i64, len: i64, n: i64) -> Vec<(String, i64, i64)> {
        (0..n)
            .map(|i| {
                let s = start + i * len;
                (format!("2024-01-{:02}", i + 1), s, s + len)
            })
            .collect()
    }

    #[test]
    fn streak_empty_days_is_zero() {
        let conn = setup();
        assert_eq!(get_streak(&conn, &[]).unwrap(), 0);
    }

    #[test]
    fn streak_no_goal_is_zero() {
        let conn = setup();
        begin_interval(&conn, 0).unwrap();
        end_interval(&conn, 100).unwrap();
        let days = day_list(0, 100, 3);
        assert_eq!(get_streak(&conn, &days).unwrap(), 0);
    }

    #[test]
    fn streak_counts_trailing_run() {
        let conn = setup();
        // Days: [0,100) [100,200) [200,300). Goal 50ms from day 1 (key 2024-01-01).
        // Fill day 0 and day 2 fully, leave day 1 empty.
        begin_interval(&conn, 0).unwrap();
        end_interval(&conn, 100).unwrap();
        begin_interval(&conn, 200).unwrap();
        end_interval(&conn, 300).unwrap();
        set_daily_goal(&conn, "2024-01-01", 50).unwrap();
        let days = day_list(0, 100, 3);
        // Last day meets goal, prior day (empty) breaks it.
        assert_eq!(get_streak(&conn, &days).unwrap(), 1);
    }

    #[test]
    fn streak_full_run() {
        let conn = setup();
        begin_interval(&conn, 0).unwrap();
        end_interval(&conn, 300).unwrap();
        set_daily_goal(&conn, "2024-01-01", 50).unwrap();
        let days = day_list(0, 100, 3);
        assert_eq!(get_streak(&conn, &days).unwrap(), 3);
    }

    #[test]
    fn streak_clips_border_intervals() {
        let conn = setup();
        // One interval straddling the boundary between day 0 and day 1.
        // Day 0 gets [80,100)=20ms, day 1 gets [100,150)=50ms.
        begin_interval(&conn, 80).unwrap();
        end_interval(&conn, 150).unwrap();
        set_daily_goal(&conn, "2024-01-01", 40).unwrap();
        let days = day_list(0, 100, 2);
        // Day 1 has 50 >= 40 (meets), day 0 has 20 < 40 (breaks).
        assert_eq!(get_streak(&conn, &days).unwrap(), 1);
    }

    #[test]
    fn streak_sums_multiple_intervals_per_day() {
        let conn = setup();
        // Day 0: two intervals totaling 60ms.
        begin_interval(&conn, 0).unwrap();
        end_interval(&conn, 30).unwrap();
        begin_interval(&conn, 40).unwrap();
        end_interval(&conn, 70).unwrap();
        set_daily_goal(&conn, "2024-01-01", 60).unwrap();
        let days = day_list(0, 100, 1);
        assert_eq!(get_streak(&conn, &days).unwrap(), 1);
    }

    #[test]
    fn streak_goal_change_applies_per_day() {
        let conn = setup();
        // Days 0..3 each filled with 100ms.
        begin_interval(&conn, 0).unwrap();
        end_interval(&conn, 300).unwrap();
        set_daily_goal(&conn, "2024-01-01", 50).unwrap(); // days 1..2 use 50
        set_daily_goal(&conn, "2024-01-03", 150).unwrap(); // day 3 needs 150 -> fails
        let days = day_list(0, 100, 3);
        // Last day fails its raised goal immediately.
        assert_eq!(get_streak(&conn, &days).unwrap(), 0);
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
