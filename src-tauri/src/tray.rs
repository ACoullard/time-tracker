use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuEvent, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Wry,
};

pub struct TrayState {
    tray: tauri::tray::TrayIcon<Wry>,
    toggle_item: MenuItem<Wry>,
    running_start_ms: Mutex<Option<i64>>,
    last_duration_ms: Mutex<Option<i64>>,
}

struct TrayStatus {
    running: bool,
    elapsed_ms: i64,
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_millis() as i64
}

fn format_hms(ms: i64) -> String {
    let total = (ms / 1000).max(0);
    let h = total / 3600;
    let m = (total / 60) % 60;
    let s = total % 60;
    if h > 0 {
        format!("{:02}:{:02}:{:02}", h, m, s)
    } else {
        format!("{:02}:{:02}", m, s)
    }
}

pub fn setup(
    app: &tauri::App<Wry>,
    initial_running: Option<i64>,
    initial_last_ms: Option<i64>,
) -> tauri::Result<()> {
    let handle = app.handle().clone();

    let toggle = MenuItem::with_id(
        app,
        "tray:toggle",
        if initial_running.is_some() { "Stop" } else { "Start" },
        true,
        None::<&str>,
    )?;
    let show_hide =
        MenuItem::with_id(app, "tray:show_hide", "Show / Hide Window", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "tray:quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&toggle, &show_hide, &quit])?;

    let icon = app
        .default_window_icon()
        .expect("bundle should provide a default window icon")
        .clone();

    let builder = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(handle_menu)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = toggle_window(tray.app_handle());
            }
        });

    #[cfg(target_os = "macos")]
    let builder = builder.icon_as_template(true);

    let tray = builder.build(app)?;

    app.manage(TrayState {
        tray,
        toggle_item: toggle,
        running_start_ms: Mutex::new(initial_running),
        last_duration_ms: Mutex::new(initial_last_ms),
    });

    refresh(&handle);

    let tick_handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let state = tick_handle.state::<TrayState>();
            let running = state.running_start_ms.lock().unwrap().is_some();
            drop(state);
            if running {
                refresh(&tick_handle);
            }
        }
    });

    Ok(())
}

pub fn on_started(app: &AppHandle<Wry>, start_ms: i64) {
    let state = app.state::<TrayState>();
    *state.running_start_ms.lock().unwrap() = Some(start_ms);
    let _ = state.toggle_item.set_text("Stop");
    drop(state);
    refresh(app);
}

pub fn on_stopped(app: &AppHandle<Wry>, duration_ms: i64) {
    let state = app.state::<TrayState>();
    *state.running_start_ms.lock().unwrap() = None;
    *state.last_duration_ms.lock().unwrap() = Some(duration_ms);
    let _ = state.toggle_item.set_text("Start");
    drop(state);
    refresh(app);
}

fn refresh(app: &AppHandle<Wry>) {
    let state = app.state::<TrayState>();
    let start = *state.running_start_ms.lock().unwrap();
    let last = *state.last_duration_ms.lock().unwrap();
    let status = match start {
        Some(s) => TrayStatus { running: true, elapsed_ms: now_ms() - s },
        None => TrayStatus { running: false, elapsed_ms: last.unwrap_or(0) },
    };
    apply(&state, &status);
}

#[cfg(target_os = "macos")]
fn apply(state: &TrayState, status: &TrayStatus) {
    let elapsed = format_hms(status.elapsed_ms);
    if status.running {
        let _ = state.tray.set_title(Some(&elapsed));
        let _ = state.tray.set_tooltip(Some(&format!("Running — {elapsed}")));
    } else {
        let _ = state.tray.set_title(None::<&str>);
        let tip = if status.elapsed_ms > 0 {
            format!("Paused — last {elapsed}")
        } else {
            "Paused".to_string()
        };
        let _ = state.tray.set_tooltip(Some(&tip));
    }
}

#[cfg(target_os = "windows")]
fn apply(state: &TrayState, status: &TrayStatus) {
    let elapsed = format_hms(status.elapsed_ms);
    let tip = if status.running {
        format!("Running — {elapsed}")
    } else if status.elapsed_ms > 0 {
        format!("Paused — last {elapsed}")
    } else {
        "Paused".to_string()
    };
    let _ = state.tray.set_tooltip(Some(&tip));
}

#[cfg(target_os = "linux")]
fn apply(state: &TrayState, status: &TrayStatus) {
    let elapsed = format_hms(status.elapsed_ms);
    let tip = if status.running {
        format!("Running — {elapsed}")
    } else if status.elapsed_ms > 0 {
        format!("Paused — last {elapsed}")
    } else {
        "Paused".to_string()
    };
    let _ = state.tray.set_tooltip(Some(&tip));
    let _ = state
        .tray
        .set_title(if status.running { Some(elapsed.as_str()) } else { None });
}

fn handle_menu(app: &AppHandle<Wry>, event: MenuEvent) {
    match event.id().as_ref() {
        "tray:toggle" => {
            let running = app
                .state::<TrayState>()
                .running_start_ms
                .lock()
                .unwrap()
                .is_some();
            if running {
                let _ = crate::do_end(app);
            } else {
                let _ = crate::do_begin(app);
            }
        }
        "tray:show_hide" => {
            let _ = toggle_window(app);
        }
        "tray:quit" => app.exit(0),
        _ => {}
    }
}

fn toggle_window(app: &AppHandle<Wry>) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible()? {
            window.hide()?;
        } else {
            window.show()?;
            window.set_focus()?;
        }
    }
    Ok(())
}
