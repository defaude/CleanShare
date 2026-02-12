use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager, WindowEvent};
use tauri_plugin_notification::NotificationExt;

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ClipboardCleanedEvent {
    id: u64,
    original_text: String,
    cleaned_text: String,
    urls_modified: usize,
    params_removed: usize,
}

struct MonitorControl {
    enabled: AtomicBool,
    epoch: AtomicU64,
    cleaned_counter: AtomicU64,
    latest_cleaned: Mutex<Option<ClipboardCleanedEvent>>,
}

#[derive(Clone)]
struct MonitorState(Arc<MonitorControl>);

const MAIN_WINDOW_LABEL: &str = "main";
const TRAY_SHOW_TOGGLE_ID: &str = "toggle-window";
const TRAY_QUIT_ID: &str = "quit";
const TRAY_ICON_MONO_BYTES: &[u8] = include_bytes!("../../../../CleanShare_logo_mono.png");

impl Default for MonitorState {
    fn default() -> Self {
        Self(Arc::new(MonitorControl {
            enabled: AtomicBool::new(true),
            epoch: AtomicU64::new(0),
            cleaned_counter: AtomicU64::new(0),
            latest_cleaned: Mutex::new(None),
        }))
    }
}

impl MonitorState {
    fn is_enabled(&self) -> bool {
        self.0.enabled.load(Ordering::Relaxed)
    }

    fn set_enabled(&self, enabled: bool) {
        self.0.enabled.store(enabled, Ordering::Relaxed);
        self.0.epoch.fetch_add(1, Ordering::Relaxed);
    }

    fn epoch(&self) -> u64 {
        self.0.epoch.load(Ordering::Relaxed)
    }

    fn next_cleaned_id(&self) -> u64 {
        self.0.cleaned_counter.fetch_add(1, Ordering::Relaxed) + 1
    }

    fn set_latest_cleaned(&self, event: ClipboardCleanedEvent) {
        if let Ok(mut latest) = self.0.latest_cleaned.lock() {
            *latest = Some(event);
        }
    }

    fn get_latest_cleaned(&self) -> Option<ClipboardCleanedEvent> {
        let Ok(latest) = self.0.latest_cleaned.lock() else {
            return None;
        };
        latest.clone()
    }
}

#[tauri::command]
fn clean_text(input: String) -> String {
    link_cleaner_core::clean_text(&input)
}

#[tauri::command]
fn set_clipboard_monitor_enabled(enabled: bool, state: tauri::State<MonitorState>) -> bool {
    state.set_enabled(enabled);
    enabled
}

#[tauri::command]
fn get_clipboard_monitor_enabled(state: tauri::State<MonitorState>) -> bool {
    state.is_enabled()
}

#[tauri::command]
fn get_latest_clipboard_cleaned(
    state: tauri::State<MonitorState>,
) -> Option<ClipboardCleanedEvent> {
    state.get_latest_cleaned()
}

fn show_main_window(app: &tauri::AppHandle) {
    #[cfg(target_os = "macos")]
    {
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
        let _ = app.set_dock_visibility(true);
    }

    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn hide_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = window.hide();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
        let _ = app.set_dock_visibility(false);
    }
}

fn toggle_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        match window.is_visible() {
            Ok(true) => hide_main_window(app),
            Ok(false) | Err(_) => show_main_window(app),
        }
    }
}

fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let toggle_window = MenuItem::with_id(
        app,
        TRAY_SHOW_TOGGLE_ID,
        "Open / Hide CleanShare",
        true,
        None::<&str>,
    )?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, TRAY_QUIT_ID, "Quit CleanShare", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&toggle_window, &separator, &quit])?;

    let toggle_window_id = toggle_window.id().clone();
    let quit_id = quit.id().clone();

    let mut tray_builder = TrayIconBuilder::with_id("clean-share-tray")
        .tooltip("CleanShare")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            if event.id() == &toggle_window_id {
                toggle_main_window(app);
            } else if event.id() == &quit_id {
                app.exit(0);
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Down,
                ..
            } = event
            {
                toggle_main_window(tray.app_handle());
            }
        });

    if let Ok(icon) = tauri::image::Image::from_bytes(TRAY_ICON_MONO_BYTES) {
        tray_builder = tray_builder.icon(icon).icon_as_template(true);
    } else if let Some(icon) = app.default_window_icon().cloned() {
        tray_builder = tray_builder.icon(icon);
    }

    tray_builder.build(app)?;
    Ok(())
}

fn setup_main_window_behavior(app: &tauri::App) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let app_handle = app.handle().clone();
        window.on_window_event(move |event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                hide_main_window(&app_handle);
            }
        });

        hide_main_window(app.handle());
    }
}

fn start_clipboard_monitor(app_handle: tauri::AppHandle, monitor_state: MonitorState) {
    thread::spawn(move || {
        let mut clipboard = loop {
            match arboard::Clipboard::new() {
                Ok(clipboard) => break clipboard,
                Err(err) => {
                    eprintln!("clipboard monitor init failed: {err}");
                    thread::sleep(Duration::from_secs(2));
                }
            }
        };

        let mut last_seen_text = String::new();
        let mut known_epoch = monitor_state.epoch();

        loop {
            let current_epoch = monitor_state.epoch();
            if current_epoch != known_epoch {
                known_epoch = current_epoch;
                last_seen_text.clear();
            }

            if !monitor_state.is_enabled() {
                thread::sleep(Duration::from_millis(800));
                continue;
            }

            match clipboard.get_text() {
                Ok(current_text) if current_text != last_seen_text => {
                    let report = link_cleaner_core::clean_text_with_report(&current_text);

                    if report.output != current_text {
                        if clipboard.set_text(report.output.clone()).is_ok() {
                            let payload = ClipboardCleanedEvent {
                                id: monitor_state.next_cleaned_id(),
                                original_text: current_text,
                                cleaned_text: report.output.clone(),
                                urls_modified: report.urls_modified,
                                params_removed: report.params_removed,
                            };

                            monitor_state.set_latest_cleaned(payload.clone());
                            let _ = app_handle.emit("clipboard-cleaned", payload);

                            let body = format!(
                                "{} URL(s) cleaned, {} parameter(s) removed",
                                report.urls_modified, report.params_removed
                            );
                            let _ = app_handle
                                .notification()
                                .builder()
                                .title("CleanShare")
                                .body(&body)
                                .show();

                            last_seen_text = report.output;
                        } else {
                            last_seen_text = current_text;
                        }
                    } else {
                        last_seen_text = current_text;
                    }
                }
                Ok(_) => {}
                Err(_) => {}
            }

            thread::sleep(Duration::from_millis(800));
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn main() {
    let monitor_state = MonitorState::default();

    tauri::Builder::default()
        .manage(monitor_state)
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
                let _ = app.set_dock_visibility(false);
            }

            setup_tray(app)?;
            setup_main_window_behavior(app);

            let monitor_state = app.state::<MonitorState>().inner().clone();
            start_clipboard_monitor(app.handle().clone(), monitor_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            clean_text,
            set_clipboard_monitor_enabled,
            get_clipboard_monitor_enabled,
            get_latest_clipboard_cleaned
        ])
        .run(tauri::generate_context!())
        .expect("failed to run tauri app");
}
