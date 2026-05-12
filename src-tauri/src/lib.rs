mod tracker;

use std::sync::Mutex;

use rusqlite::Connection;
use tauri::{Manager, State};

use tracker::{init_schema, Interval};

pub struct AppState {
    db: Mutex<Connection>,
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_millis() as i64
}

#[tauri::command]
fn get_current_interval(state: State<AppState>) -> Result<Option<i64>, String> {
    let conn = state.db.lock().unwrap();
    tracker::get_current_interval(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn begin_interval(state: State<AppState>) -> Result<i64, String> {
    let conn = state.db.lock().unwrap();
    tracker::begin_interval(&conn, now_ms()).map_err(|e| e.to_string())
}

#[tauri::command]
fn end_interval(state: State<AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    tracker::end_interval(&conn, now_ms()).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_intervals(
    state: State<AppState>,
    from_ms: i64,
    to_ms: i64,
) -> Result<Vec<Interval>, String> {
    let conn = state.db.lock().unwrap();
    tracker::get_intervals(&conn, from_ms, to_ms).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let conn = Connection::open(dir.join("time-tracker.db"))?;
            init_schema(&conn)?;
            app.manage(AppState {
                db: Mutex::new(conn),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_current_interval,
            begin_interval,
            end_interval,
            get_intervals,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
