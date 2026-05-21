mod tracker;

use std::sync::Mutex;

use rusqlite::Connection;
use tauri::{Manager, State};

use tracker::{init_schema, RangeTotal, Interval};

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
#[specta::specta]
fn get_current_interval(state: State<AppState>) -> Result<Option<i64>, String> {
    let conn = state.db.lock().unwrap();
    tracker::get_current_interval(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn begin_interval(state: State<AppState>) -> Result<i64, String> {
    let conn = state.db.lock().unwrap();
    tracker::begin_interval(&conn, now_ms()).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn end_interval(state: State<AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    tracker::end_interval(&conn, now_ms()).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn get_intervals(
    state: State<AppState>,
    from_ms: i64,
    to_ms: i64,
) -> Result<Vec<Interval>, String> {
    let conn = state.db.lock().unwrap();
    tracker::get_intervals(&conn, from_ms, to_ms).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn get_range_total(
    state: State<AppState>,
    from_ms: i64,
    to_ms: i64,
) -> Result<RangeTotal, String> {
    let conn = state.db.lock().unwrap();
    tracker::get_time_range_total(&conn, from_ms, to_ms).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri_specta::Builder::<tauri::Wry>::new().commands(
        tauri_specta::collect_commands![
            get_current_interval,
            begin_interval,
            end_interval,
            get_intervals,
            get_range_total,
        ],
    );

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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
