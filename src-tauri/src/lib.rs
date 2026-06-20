mod tracker;
mod tray;

use std::sync::Mutex;

use chrono::{Duration, Local, NaiveDate, TimeZone};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State, Wry};
use tauri_specta::Event;

use tracker::{init_schema, DailyGoal, DailyTotal, Interval, RangeTotal, TimerState};

pub struct AppState {
    db: Mutex<Connection>,
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_millis() as i64
}

#[cfg(target_os = "windows")]
fn get_idle_duration_ms() -> u64 {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
    use windows_sys::Win32::System::SystemInformation::GetTickCount;
    unsafe {
        let mut info = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };
        if GetLastInputInfo(&mut info) == 0 {
            return 0;
        }
        // wrapping_sub handles the ~49.7-day u32 rollover correctly
        GetTickCount().wrapping_sub(info.dwTime) as u64
    }
}

#[cfg(target_os = "macos")]
fn get_idle_duration_ms() -> u64 {
    // CGEventSourceSecondsSinceLastEventType ships with macOS — no extra crate needed.
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventSourceSecondsSinceLastEventType(state_id: u32, event_type: u32) -> f64;
    }
    const COMBINED_SESSION_STATE: u32 = 1;
    const ANY_INPUT_EVENT: u32 = 0xFFFF_FFFF;
    let secs = unsafe { CGEventSourceSecondsSinceLastEventType(COMBINED_SESSION_STATE, ANY_INPUT_EVENT) };
    (secs * 1000.0) as u64
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn get_idle_duration_ms() -> u64 {
    0
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

#[derive(Clone, Default, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub struct IntervalChanged {}

#[derive(Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
#[serde(rename_all = "camelCase")]
pub struct PopupShow {
    pub title: String,
    pub message: String,
}

#[derive(Clone, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
#[serde(rename_all = "camelCase")]
pub struct IdleDetected {
    pub idle_since_ms: i64,
}

pub fn do_begin(app: &AppHandle<Wry>) -> CmdResult<i64> {
    let state = app.state::<AppState>();
    let conn = state.db.lock().unwrap();
    let start = tracker::begin_interval(&conn, now_ms())?;
    drop(conn);
    tray::on_started(app, start);
    let _ = IntervalChanged {}.emit(app);
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
    let _ = IntervalChanged {}.emit(app);
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
fn end_interval_at(app: AppHandle<Wry>, end_ms: i64) -> CmdResult<()> {
    let state = app.state::<AppState>();
    let conn = state.db.lock().unwrap();
    let TimerState::Running { start_ms } = tracker::timer_state(&conn)? else {
        return Err(CmdError::from("no interval running"));
    };
    tracker::end_interval(&conn, end_ms)?;
    drop(conn);
    let duration = end_ms - start_ms;
    tray::on_stopped(&app, duration);
    let _ = IntervalChanged {}.emit(&app);
    Ok(())
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
    drop(conn);
    let _ = IntervalChanged {}.emit(&app);
    Ok(())
}

#[tauri::command]
#[specta::specta]
fn delete_interval(app: AppHandle<Wry>, id: i64) -> CmdResult<()> {
    let state = app.state::<AppState>();
    let conn = state.db.lock().unwrap();
    let before = tracker::timer_state(&conn)?;
    tracker::delete_interval(&conn, id)?;
    let after = tracker::timer_state(&conn)?;
    drop(conn);
    if matches!(before, TimerState::Running { .. }) && !matches!(after, TimerState::Running { .. }) {
        tray::on_empty(&app);
    }
    let _ = IntervalChanged {}.emit(&app);
    Ok(())
}

#[tauri::command]
#[specta::specta]
fn set_daily_goal(state: State<AppState>, goal_ms: i64) -> CmdResult<()> {
    let day = Local::now().format("%Y-%m-%d").to_string();
    let conn = state.db.lock().unwrap();
    Ok(tracker::set_daily_goal(&conn, &day, goal_ms)?)
}

fn build_day_list(from_day: &str, to_day: &str) -> CmdResult<Vec<(String, i64, i64)>> {
    let from = NaiveDate::parse_from_str(from_day, "%Y-%m-%d")
        .map_err(|e| CmdError::from(e.to_string()))?;
    let to = NaiveDate::parse_from_str(to_day, "%Y-%m-%d")
        .map_err(|e| CmdError::from(e.to_string()))?;

    let mut days = Vec::new();
    let mut cur = from;
    while cur <= to {
        let key = cur.format("%Y-%m-%d").to_string();
        let next = cur + Duration::days(1);
        let start_ms = Local
            .from_local_datetime(&cur.and_hms_opt(0, 0, 0).unwrap())
            .earliest()
            .ok_or_else(|| CmdError::from("invalid local datetime"))?
            .timestamp_millis();
        let end_ms = Local
            .from_local_datetime(&next.and_hms_opt(0, 0, 0).unwrap())
            .earliest()
            .ok_or_else(|| CmdError::from("invalid local datetime"))?
            .timestamp_millis();
        days.push((key, start_ms, end_ms));
        cur = next;
    }
    Ok(days)
}

#[tauri::command]
#[specta::specta]
fn get_daily_totals(state: State<AppState>, from_day: String, to_day: String) -> CmdResult<Vec<DailyTotal>> {
    let days = build_day_list(&from_day, &to_day)?;
    let conn = state.db.lock().unwrap();
    Ok(tracker::get_daily_totals(&conn, &days)?)
}

#[tauri::command]
#[specta::specta]
fn get_daily_goals_for_range(state: State<AppState>, from_day: String, to_day: String) -> CmdResult<Vec<DailyGoal>> {
    let days = build_day_list(&from_day, &to_day)?;
    let day_keys: Vec<String> = days.into_iter().map(|(k, _, _)| k).collect();
    let conn = state.db.lock().unwrap();
    Ok(tracker::get_applicable_goals(&conn, &day_keys)?)
}

#[tauri::command]
#[specta::specta]
fn get_streak(state: State<AppState>, as_of_day: String) -> CmdResult<u32> {
    let conn = state.db.lock().unwrap();
    let mut total_streak = 0u32;
    let mut chunk_end = NaiveDate::parse_from_str(&as_of_day, "%Y-%m-%d")
        .map_err(|e| CmdError::from(e.to_string()))?;

    for _ in 0..1200 {
        let chunk_start = chunk_end - Duration::days(30);
        let days = build_day_list(
            &chunk_start.format("%Y-%m-%d").to_string(),
            &chunk_end.format("%Y-%m-%d").to_string(),
        )?;
        let chunk_streak = tracker::get_streak(&conn, &days)?;
        total_streak += chunk_streak;
        if chunk_streak < days.len() as u32 {
            break;
        }
        chunk_end = chunk_start - Duration::days(1);
    }

    Ok(total_streak)
}

#[tauri::command]
#[specta::specta]
async fn show_system_popup(app: AppHandle<Wry>, title: String, message: String) -> CmdResult<()> {
    if let Some(window) = app.get_webview_window("popup") {
        let _ = PopupShow { title, message }.emit(&app);
        window.show().map_err(|e| CmdError(e.to_string()))?;
        window.set_focus().map_err(|e| CmdError(e.to_string()))?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri_specta::Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![
            get_timer_state,
            begin_interval,
            end_interval,
            end_interval_at,
            get_intervals,
            get_range_total,
            get_current_goal,
            set_daily_goal,
            update_interval,
            delete_interval,
            show_system_popup,
            get_daily_totals,
            get_daily_goals_for_range,
            get_streak,
        ])
        .events(tauri_specta::collect_events![IntervalChanged, PopupShow, IdleDetected]);

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

            let idle_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                use tauri_plugin_store::StoreExt;
                let mut last_fired_idle_since: Option<i64> = None;

                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(30)).await;

                    let threshold_ms: u64 = idle_handle
                        .store("settings.json")
                        .ok()
                        .and_then(|s| s.get("idleThresholdMs"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(5 * 60 * 1000);

                    let state = idle_handle.state::<AppState>();
                    let is_running = {
                        let conn = state.db.lock().unwrap();
                        matches!(
                            tracker::timer_state(&conn).unwrap_or(TimerState::Empty),
                            TimerState::Running { .. }
                        )
                    };
                    if !is_running {
                        last_fired_idle_since = None;
                        continue;
                    }

                    let idle_ms = get_idle_duration_ms();
                    if idle_ms < threshold_ms {
                        last_fired_idle_since = None;
                        continue;
                    }

                    let idle_since_ms = now_ms() - idle_ms as i64;
                    let already_sent = last_fired_idle_since
                        .map(|prev| (idle_since_ms - prev).abs() < 60_000)
                        .unwrap_or(false);
                    if already_sent {
                        continue;
                    }

                    last_fired_idle_since = Some(idle_since_ms);

                    if let Some(popup) = idle_handle.get_webview_window("popup") {
                        let _ = popup.show();
                        let _ = popup.set_focus();
                    }
                    let _ = IdleDetected { idle_since_ms }.emit(&idle_handle);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
