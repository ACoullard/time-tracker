mod tracker;
mod tray;

use std::sync::Mutex;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State, Wry};
use tauri_specta::Event;

use tracker::{init_schema, Interval, RangeTotal};

pub struct AppState {
    db: Mutex<Connection>,
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_millis() as i64
}

#[derive(Debug, Serialize, specta::Type)]
#[serde(transparent)]
pub struct CmdError(String);

impl From<tracker::DbError> for CmdError {
    fn from(e: tracker::DbError) -> Self {
        Self(e.to_string())
    }
}

impl From<rusqlite::Error> for CmdError {
    fn from(e: rusqlite::Error) -> Self {
        Self(e.to_string())
    }
}

impl From<&str> for CmdError {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for CmdError {
    fn from(s: String) -> Self {
        Self(s)
    }
}

pub type CmdResult<T> = Result<T, CmdError>;

#[derive(Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub struct IntervalChanged {
    pub running_start_ms: Option<i64>,
}

pub fn do_begin(app: &AppHandle<Wry>) -> CmdResult<i64> {
    let state = app.state::<AppState>();
    let conn = state.db.lock().unwrap();
    let start = tracker::begin_interval(&conn, now_ms())?;
    drop(conn);
    tray::on_started(app, start);
    let _ = IntervalChanged {
        running_start_ms: Some(start),
    }
    .emit(app);
    Ok(start)
}

pub fn do_end(app: &AppHandle<Wry>) -> CmdResult<()> {
    let state = app.state::<AppState>();
    let conn = state.db.lock().unwrap();
    let Some(start) = tracker::get_current_interval(&conn)? else {
        return Err(CmdError::from("no interval running"));
    };
    let end = now_ms();
    tracker::end_interval(&conn, end)?;
    drop(conn);
    let duration = end - start;
    tray::on_stopped(app, duration);
    let _ = IntervalChanged {
        running_start_ms: None,
    }
    .emit(app);
    Ok(())
}

#[tauri::command]
#[specta::specta]
fn get_current_interval(state: State<AppState>) -> CmdResult<Option<i64>> {
    let conn = state.db.lock().unwrap();
    Ok(tracker::get_current_interval(&conn)?)
}

#[tauri::command]
#[specta::specta]
fn begin_interval(app: AppHandle<Wry>) -> CmdResult<i64> {
    do_begin(&app)
}

#[tauri::command]
#[specta::specta]
fn end_interval(app: AppHandle<Wry>) -> CmdResult<()> {
    do_end(&app)
}

#[tauri::command]
#[specta::specta]
fn get_intervals(state: State<AppState>, from_ms: i64, to_ms: i64) -> CmdResult<Vec<Interval>> {
    let conn = state.db.lock().unwrap();
    Ok(tracker::get_intervals(&conn, from_ms, to_ms)?)
}

#[tauri::command]
#[specta::specta]
fn get_range_total(state: State<AppState>, from_ms: i64, to_ms: i64) -> CmdResult<RangeTotal> {
    let conn = state.db.lock().unwrap();
    Ok(tracker::get_time_range_total(&conn, from_ms, to_ms)?)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri_specta::Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![
            get_current_interval,
            begin_interval,
            end_interval,
            get_intervals,
            get_range_total,
        ])
        .events(tauri_specta::collect_events![IntervalChanged]);

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default()
                .bigint(specta_typescript::BigIntExportBehavior::Number),
            "../src/lib/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);

            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let conn = Connection::open(dir.join("time-tracker.db"))?;
            init_schema(&conn)?;

            let initial_running = tracker::get_current_interval(&conn).ok().flatten();
            let initial_last_ms = if initial_running.is_none() {
                tracker::get_most_recent_interval(&conn)
                    .ok()
                    .flatten()
                    .and_then(|i| i.end_ms.map(|end| end - i.start_ms))
            } else {
                None
            };

            app.manage(AppState {
                db: Mutex::new(conn),
            });

            tray::setup(app, initial_running, initial_last_ms)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
