# Requirements

Stopwatch function. Start and stop a stopwatch.
    Show in the status bar. System tray on windows
Time tracking
    store stop watch sessions and show totals across catagories
    by day
    (later) by user specified labels
Persist data across sessions

# Pages

## Main Page
- shows the stopwatch currently going
- allows stopping, starting
- Shows the total amount so far in the current day
    - allows setting a goal and shows a progress bar to that goal

## Report page
- Shows aggregate totals of sessions across categories
- By date, with options for lookback period
    - by week
    - by month
    - shows an average per day in the lookback period

# Architecture:

Rust is source of truth
JS is display

Rust handles:
    - tracking intervals



## Main Page
- shows the stopwatch currently going
    calculate from latest interval start time.
- allows stopping, starting
    send an event down to rust to register changes to intervals
- Shows the total amount so far in the current day
    get a filtered amount if intervals and add up
    - allows setting a goal and shows a progress bar to that goal
        

## Report page
- Shows aggregate totals of sessions across categories
- By date, with options for lookback period
    - by week
    - by month
    - shows an average per day in the lookback period


## API

getCurrentInterval
    returns the start of the current interval or Null if none is currently running

beginInterval
    begins an interval

endInterval
    ends an interval

getIntervals
    allows adding filters. 
    - Date
    - (later) custom labels
    composes the filters and gives a list of resulting intervals in chronological order

## Persistence

SQLite via `rusqlite` (`bundled` feature). Single file at `app_data_dir()/time-tracker.db`. One `Mutex<Connection>` held in Tauri managed state.

Schema:
- `intervals(id INTEGER PK, start_ms INTEGER NOT NULL, end_ms INTEGER NULL)` — `end_ms IS NULL` = currently running
- Timestamps as unix millis; index on `start_ms`

`CREATE TABLE IF NOT EXISTS` runs at startup. SQLite's ACID guarantees protect against crash mid-write.

## System Tray

Launches normally with the window visible. Closing the window hides it (`prevent_close` + `hide()`); the timer keeps running in Rust. Quit only via the tray menu.

Tray menu: Show/Hide window, Start/Stop timer, Quit. While an interval is active, a background task updates the tray title (macOS/Linux) or tooltip (Windows) every second with elapsed time.

Requires the `tray-icon` Cargo feature and tray permissions in `capabilities/default.json`.