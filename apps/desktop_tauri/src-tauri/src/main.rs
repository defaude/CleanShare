use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use tauri::{Emitter, Manager};
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
                                "{} URL(s) bereinigt, {} Parameter entfernt",
                                report.urls_modified, report.params_removed
                            );
                            let _ = app_handle
                                .notification()
                                .builder()
                                .title("Clean Share")
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
