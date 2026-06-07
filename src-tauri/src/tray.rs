use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuEvent, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Wry,
};

use crate::tracker::TimerState;

pub struct TrayState {
    tray: tauri::tray::TrayIcon<Wry>,
    toggle_item: MenuItem<Wry>,
    timer: Mutex<TimerState>,
    icon_running: tauri::image::Image<'static>,
    icon_paused: tauri::image::Image<'static>,
}

fn load_icon(bytes: &'static [u8]) -> tauri::image::Image<'static> {
    tauri::image::Image::from_bytes(bytes).expect("valid PNG icon")
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

const PNG_RUNNING: &[u8] = include_bytes!("../icons/stopwatch-running.png");
const PNG_PAUSED: &[u8] = include_bytes!("../icons/stopwatch-paused.png");

pub fn setup(app: &tauri::App<Wry>, initial: TimerState) -> tauri::Result<()> {
    let handle = app.handle().clone();
    let running = matches!(initial, TimerState::Running { .. });

    let icon_running = load_icon(PNG_RUNNING);
    let icon_paused = load_icon(PNG_PAUSED);

    let toggle = MenuItem::with_id(
        app,
        "tray:toggle",
        if running { "Stop" } else { "Start" },
        true,
        None::<&str>,
    )?;
    let show_hide =
        MenuItem::with_id(app, "tray:show_hide", "Show / Hide Window", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "tray:quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&toggle, &show_hide, &quit])?;

    let initial_icon = if running {
        icon_running.clone()
    } else {
        icon_paused.clone()
    };

    let builder = TrayIconBuilder::new()
        .icon(initial_icon)
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
        timer: Mutex::new(initial),
        icon_running,
        icon_paused,
    });

    refresh(&handle);

    let tick_handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let state = tick_handle.state::<TrayState>();
            let running = matches!(*state.timer.lock().unwrap(), TimerState::Running { .. });
            if running {
                refresh(&tick_handle);
            }
        }
    });

    Ok(())
}

pub fn on_started(app: &AppHandle<Wry>, start_ms: i64) {
    let state = app.state::<TrayState>();
    *state.timer.lock().unwrap() = TimerState::Running { start_ms };
    let _ = state.toggle_item.set_text("Stop");
    refresh(app);
}

pub fn on_stopped(app: &AppHandle<Wry>, duration_ms: i64) {
    let state = app.state::<TrayState>();
    *state.timer.lock().unwrap() = TimerState::Paused {
        last_duration_ms: duration_ms,
    };
    let _ = state.toggle_item.set_text("Start");
    refresh(app);
}

fn refresh(app: &AppHandle<Wry>) {
    let state = app.state::<TrayState>();
    let timer = *state.timer.lock().unwrap();
    let icon = match timer {
        TimerState::Running { .. } => state.icon_running.clone(),
        _ => state.icon_paused.clone(),
    };
    let _ = state.tray.set_icon(Some(icon));
    apply(&state, timer);
}

#[cfg(target_os = "macos")]
fn apply(state: &TrayState, timer: TimerState) {
    match timer {
        TimerState::Running { start_ms } => {
            let elapsed = format_hms(now_ms() - start_ms);
            let _ = state.tray.set_title(Some(&elapsed));
            let _ = state.tray.set_tooltip(Some(&format!("Running — {elapsed}")));
        }
        TimerState::Paused { last_duration_ms } => {
            let _ = state.tray.set_title(None::<&str>);
            let _ = state.tray.set_tooltip(Some(&format!(
                "Paused — last {}",
                format_hms(last_duration_ms)
            )));
        }
        TimerState::Empty => {
            let _ = state.tray.set_title(None::<&str>);
            let _ = state.tray.set_tooltip(Some("Paused"));
        }
    }
}

#[cfg(target_os = "windows")]
fn apply(state: &TrayState, timer: TimerState) {
    let tip = match timer {
        TimerState::Running { start_ms } => format!("Running — {}", format_hms(now_ms() - start_ms)),
        TimerState::Paused { last_duration_ms } => {
            format!("Paused — last {}", format_hms(last_duration_ms))
        }
        TimerState::Empty => "Paused".to_string(),
    };
    let _ = state.tray.set_tooltip(Some(&tip));
}

#[cfg(target_os = "linux")]
fn apply(state: &TrayState, timer: TimerState) {
    match timer {
        TimerState::Running { start_ms } => {
            let elapsed = format_hms(now_ms() - start_ms);
            let _ = state.tray.set_tooltip(Some(&format!("Running — {elapsed}")));
            let _ = state.tray.set_title(Some(elapsed.as_str()));
        }
        TimerState::Paused { last_duration_ms } => {
            let _ = state.tray.set_tooltip(Some(&format!(
                "Paused — last {}",
                format_hms(last_duration_ms)
            )));
            let _ = state.tray.set_title(None::<&str>);
        }
        TimerState::Empty => {
            let _ = state.tray.set_tooltip(Some("Paused"));
            let _ = state.tray.set_title(None::<&str>);
        }
    }
}

fn handle_menu(app: &AppHandle<Wry>, event: MenuEvent) {
    match event.id().as_ref() {
        "tray:toggle" => {
            let running = matches!(
                *app.state::<TrayState>().timer.lock().unwrap(),
                TimerState::Running { .. }
            );
            if running {
                let _ = crate::do_end(app);
            } else {
                let _ = crate::do_begin(app);
            }
        }
        "tray:show_hide" => {
            let _ = toggle_window(app);
        }
        "tray:quit" => {
            for (_, window) in app.webview_windows() {
                let _ = window.destroy();
            }
            app.exit(0);
        }
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
