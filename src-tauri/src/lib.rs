mod tracker;
mod tray;

use std::sync::Mutex;

use chrono::Local;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State, Wry};
use tauri_specta::Event;

use tracker::{init_schema, DailyGoal, Interval, RangeTotal, TimerState};

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
    let TimerState::Running { start_ms } = tracker::timer_state(&conn)? else {
        return Err(CmdError::from("no interval running"));
    };
    let end = now_ms();
    tracker::end_interval(&conn, end)?;
    drop(conn);
    let duration = end - start_ms;
    tray::on_stopped(app, duration);
    let _ = IntervalChanged {
        running_start_ms: None,
    }
    .emit(app);
    Ok(())
}

#[tauri::command]
#[specta::specta]
fn get_timer_state(state: State<AppState>) -> CmdResult<TimerState> {
    let conn = state.db.lock().unwrap();
    Ok(tracker::timer_state(&conn)?)
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

#[tauri::command]
#[specta::specta]
fn get_current_goal(state: State<AppState>) -> CmdResult<Option<DailyGoal>> {
    let conn = state.db.lock().unwrap();
    Ok(tracker::get_current_goal(&conn)?)
}

#[tauri::command]
#[specta::specta]
fn update_interval(app: AppHandle<Wry>, id: i64, start_ms: i64, end_ms: Option<i64>) -> CmdResult<()> {
    let state = app.state::<AppState>();
    let conn = state.db.lock().unwrap();
    tracker::update_interval(&conn, id, start_ms, end_ms)?;
    let running_start = match tracker::timer_state(&conn)? {
        TimerState::Running { start_ms: s } => Some(s),
        _ => None,
    };
    drop(conn);
    let _ = IntervalChanged { running_start_ms: running_start }.emit(&app);
    Ok(())
}

#[tauri::command]
#[specta::specta]
fn set_daily_goal(state: State<AppState>, goal_ms: i64) -> CmdResult<()> {
    let day = Local::now().format("%Y-%m-%d").to_string();
    let conn = state.db.lock().unwrap();
    Ok(tracker::set_daily_goal(&conn, &day, goal_ms)?)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri_specta::Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![
            get_timer_state,
            begin_interval,
            end_interval,
            get_intervals,
            get_range_total,
            get_current_goal,
            set_daily_goal,
            update_interval,
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
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(builder.invoke_handler())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(move |app| {
            builder.mount_events(app);

            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let conn = Connection::open(dir.join("time-tracker.db"))?;
            init_schema(&conn)?;

            let initial = tracker::timer_state(&conn).unwrap_or(TimerState::Empty);

            app.manage(AppState {
                db: Mutex::new(conn),
            });

            tray::setup(app, initial)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
