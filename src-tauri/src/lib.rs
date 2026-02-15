use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconEvent},
    Manager,
};

#[cfg(target_os = "linux")]
use gtk::prelude::{GtkWindowExt, WidgetExt};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct NoteData {
    content: String,
    color: String,
    pinned: bool,
    #[serde(default = "default_opacity")]
    opacity: f64,
    #[serde(default)]
    border_radius: u32,
}

fn default_opacity() -> f64 {
    1.0
}

fn get_data_dir() -> PathBuf {
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("sticky-md");
    path
}

fn get_note_path() -> PathBuf {
    let mut path = get_data_dir();
    path.push("note.json");
    path
}

/// Bypass Tauri's setAlwaysOnTop (which is unreliable on some Linux WMs/compositors)
/// by calling gtk_window_set_keep_above directly on the underlying GTK window.
#[tauri::command]
fn set_pinned(window: tauri::WebviewWindow, pinned: bool) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let gtk_window = window.gtk_window().map_err(|e| e.to_string())?;
        gtk_window.set_keep_above(pinned);
        return Ok(());
    }
    #[cfg(not(target_os = "linux"))]
    {
        window
            .set_always_on_top(pinned)
            .map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn set_opacity(window: tauri::WebviewWindow, opacity: f64) -> Result<(), String> {
    let clamped = opacity.clamp(0.2, 1.0);
    #[cfg(target_os = "linux")]
    {
        let gtk_window = window.gtk_window().map_err(|e| e.to_string())?;
        gtk_window.set_opacity(clamped);
        return Ok(());
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = clamped;
        Ok(())
    }
}

#[tauri::command]
fn save_note(data: NoteData) -> Result<(), String> {
    let dir = get_data_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = get_note_path();
    let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

#[tauri::command]
fn load_note() -> Result<NoteData, String> {
    let path = get_note_path();
    if path.exists() {
        let json = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    } else {
        Ok(NoteData {
            content: String::from(
                "# Welcome to Sticky MD\n\nStart typing your **markdown** here...\n\n- [x] Always on top\n- [x] Draggable\n- [ ] Your notes here",
            ),
            color: String::from("yellow"),
            pinned: true,
            opacity: 1.0,
            border_radius: 0,
        })
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // Apply always-on-top via GTK directly on Linux since Tauri's
            // built-in setAlwaysOnTop is unreliable under some compositors.
            #[cfg(target_os = "linux")]
            {
                if let Some(window) = app.get_webview_window("main") {
                    if let Ok(gtk_window) = window.gtk_window() {
                        gtk_window.set_keep_above(true);
                    }
                }
            }

            // Set up tray icon click to toggle window visibility
            let app_handle = app.handle().clone();
            if let Some(tray) = app.tray_by_id("main") {
                tray.on_tray_icon_event(move |_tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                });
            }

            // Global shortcut: Super+Shift+N to toggle visibility
            use tauri_plugin_global_shortcut::GlobalShortcutExt;
            let app_handle2 = app.handle().clone();
            app.global_shortcut().on_shortcut("Super+Shift+N", move |_app, _shortcut, _event| {
                if let Some(window) = app_handle2.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            })?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![save_note, load_note, set_pinned, set_opacity])
        .run(tauri::generate_context!())
        .expect("error while running Sticky MD");
}
