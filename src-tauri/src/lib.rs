//! Peekaboo — stealth browser overlay core.
//!
//! Responsibility boundary (docs/architecture.html §①): the Rust core owns
//! every stealth-relevant window property, the panic global shortcut, cursor
//! polling, the reveal state machine and persisted settings. The WebView only
//! renders content and reflects state.
//!
//! Phases: 0 scaffold · 1 panic shortcut · 2 hover-reveal · 3 concealment
//! (Accessory / Spaces / content protection) · 4 settings + persistence.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};

use serde::{Deserialize, Serialize};
use tauri::{
    ActivationPolicy, AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, State,
    WebviewWindow,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// Hover hotzone as fractions (0..1) of the window. Default = whole window.
#[derive(Clone, Serialize, Deserialize)]
struct Hotzone {
    fx: f64,
    fy: f64,
    fw: f64,
    fh: f64,
}

impl Default for Hotzone {
    fn default() -> Self {
        Self { fx: 0.0, fy: 0.0, fw: 1.0, fh: 1.0 }
    }
}

/// Persisted user settings — the single source of truth, owned by the core.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
struct Settings {
    url: String,
    ghost_opacity: f64,
    revealed_opacity: f64,
    width: f64,
    height: f64,
    hotzone: Hotzone,
    /// Accelerator string, e.g. "CmdOrControl+Shift+H".
    panic_shortcut: String,
    bookmarks: Vec<String>,
    content_protected: bool,
    always_on_top: bool,
    spaces_global: bool,
    /// Last window position in physical pixels (restored on launch).
    x: Option<i32>,
    y: Option<i32>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            url: "https://example.com".to_string(),
            ghost_opacity: 0.08,
            revealed_opacity: 1.0,
            width: 420.0,
            height: 720.0,
            hotzone: Hotzone::default(),
            panic_shortcut: "CmdOrControl+Shift+H".to_string(),
            bookmarks: Vec::new(),
            content_protected: true,
            always_on_top: true,
            spaces_global: true,
            x: None,
            y: None,
        }
    }
}

type SharedSettings = Arc<Mutex<Settings>>;

/// Cursor polling interval (~25 fps). Trade-off between reveal responsiveness
/// and idle CPU (roadmap risk #4). Edge-triggered.
const HOVER_POLL: Duration = Duration::from_millis(40);

// ---- persistence -------------------------------------------------------

fn settings_path(app: &AppHandle) -> Option<PathBuf> {
    // A plain filename keeps the on-disk trace low-key (the dir still carries
    // the bundle identifier — acceptable for a personal tool).
    app.path().app_config_dir().ok().map(|d| d.join("prefs.json"))
}

fn load_settings(app: &AppHandle) -> Settings {
    settings_path(app)
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|txt| serde_json::from_str::<Settings>(&txt).ok())
        .unwrap_or_default()
}

fn persist_settings(app: &AppHandle, s: &Settings) -> Result<(), String> {
    let path = settings_path(app).ok_or("no config dir")?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let txt = serde_json::to_string_pretty(s).map_err(|e| e.to_string())?;
    std::fs::write(path, txt).map_err(|e| e.to_string())
}

// ---- window / hover ----------------------------------------------------

fn apply_window_settings(win: &WebviewWindow, s: &Settings) {
    let _ = win.set_size(LogicalSize::new(s.width, s.height));
    let _ = win.set_always_on_top(s.always_on_top);
    let _ = win.set_visible_on_all_workspaces(s.spaces_global);
    // Best-effort only — ineffective on macOS 15+ (ScreenCaptureKit, #14200).
    let _ = win.set_content_protected(s.content_protected);
}

fn toggle_overlay(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        if win.is_visible().unwrap_or(false) {
            let _ = win.hide();
            // Checkpoint on hide: the panic-hide is a natural, infrequent moment
            // to persist the latest position/settings.
            if let Some(state) = app.try_state::<SharedSettings>() {
                if let Ok(s) = state.lock() {
                    let _ = persist_settings(app, &s);
                }
            }
        } else {
            let _ = win.show();
            let _ = win.set_focus();
        }
    }
}

/// Is the cursor (desktop physical coords) inside the hotzone rectangle?
fn hotzone_contains(win: &WebviewWindow, cursor: PhysicalPosition<f64>, hz: &Hotzone) -> bool {
    let (Ok(origin), Ok(size)) = (win.outer_position(), win.inner_size()) else {
        return false;
    };
    let w = size.width as f64;
    let h = size.height as f64;
    let left = origin.x as f64 + hz.fx * w;
    let top = origin.y as f64 + hz.fy * h;
    cursor.x >= left
        && cursor.x <= left + hz.fw * w
        && cursor.y >= top
        && cursor.y <= top + hz.fh * h
}

/// Reveal state machine driver (architecture.html §④): poll the global cursor,
/// hit-test the (configurable) hotzone, flip Ghost/Revealed on edges only.
/// Suspended while hidden (panic); re-evaluated on the next show.
fn spawn_hover_loop(win: WebviewWindow, settings: SharedSettings) {
    thread::spawn(move || {
        let mut inside_prev: Option<bool> = None;
        loop {
            thread::sleep(HOVER_POLL);

            if !win.is_visible().unwrap_or(false) {
                inside_prev = None;
                continue;
            }

            let Ok(cursor) = win.cursor_position() else {
                continue;
            };
            let hz = settings
                .lock()
                .map(|s| s.hotzone.clone())
                .unwrap_or_default();
            let inside = hotzone_contains(&win, cursor, &hz);

            if Some(inside) != inside_prev {
                let _ = win.set_ignore_cursor_events(!inside);
                let _ = win.emit(
                    "reveal-state-changed",
                    if inside { "revealed" } else { "ghost" },
                );
                inside_prev = Some(inside);
            }
        }
    });
}

// ---- commands ----------------------------------------------------------

#[tauri::command]
fn get_settings(state: State<'_, SharedSettings>) -> Settings {
    state.lock().expect("settings lock poisoned").clone()
}

#[tauri::command]
fn save_settings(
    app: AppHandle,
    state: State<'_, SharedSettings>,
    settings: Settings,
) -> Result<(), String> {
    // Re-register the panic shortcut only if it changed.
    let old = state
        .lock()
        .map_err(|_| "settings lock poisoned".to_string())?
        .panic_shortcut
        .clone();
    if old != settings.panic_shortcut {
        let gs = app.global_shortcut();
        let _ = gs.unregister(old.as_str());
        gs.register(settings.panic_shortcut.as_str())
            .map_err(|e| e.to_string())?;
    }

    if let Some(win) = app.get_webview_window("main") {
        apply_window_settings(&win, &settings);
    }

    *state
        .lock()
        .map_err(|_| "settings lock poisoned".to_string())? = settings.clone();
    persist_settings(&app, &settings)
}

// ---- entry -------------------------------------------------------------

pub fn run() {
    let settings: SharedSettings = Arc::new(Mutex::new(Settings::default()));
    let settings_for_setup = settings.clone();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    // Only the panic shortcut is ever registered, so any Pressed
                    // event is the panic key. Toggle straight from the Rust
                    // handler (no IPC round-trip) for minimal latency.
                    if event.state() == ShortcutState::Pressed {
                        toggle_overlay(app);
                    }
                })
                .build(),
        )
        .manage(settings)
        .invoke_handler(tauri::generate_handler![get_settings, save_settings])
        .setup(move |app| {
            let handle = app.handle().clone();
            let loaded = load_settings(&handle);
            if let Ok(mut guard) = settings_for_setup.lock() {
                *guard = loaded.clone();
            }

            // Register the panic shortcut exactly once (os error 22 on double).
            app.global_shortcut()
                .register(loaded.panic_shortcut.as_str())?;

            // Hide from Dock / Cmd-Tab. Must be on `App`, not `AppHandle` (#9244).
            #[cfg(target_os = "macos")]
            app.set_activation_policy(ActivationPolicy::Accessory);

            if let Some(win) = app.get_webview_window("main") {
                apply_window_settings(&win, &loaded);

                // Restore the last window position (physical pixels).
                if let (Some(x), Some(y)) = (loaded.x, loaded.y) {
                    let _ = win.set_position(PhysicalPosition::new(x, y));
                }

                // Track moves in memory; persist on close.
                let settings_for_events = settings_for_setup.clone();
                let handle_for_events = handle.clone();
                win.on_window_event(move |event| match event {
                    tauri::WindowEvent::Moved(pos) => {
                        if let Ok(mut s) = settings_for_events.lock() {
                            s.x = Some(pos.x);
                            s.y = Some(pos.y);
                        }
                    }
                    tauri::WindowEvent::CloseRequested { .. } => {
                        if let Ok(s) = settings_for_events.lock() {
                            let _ = persist_settings(&handle_for_events, &s);
                        }
                    }
                    _ => {}
                });

                spawn_hover_loop(win, settings_for_setup.clone());
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Peekaboo");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip() {
        let s = Settings::default();
        let json = serde_json::to_string(&s).expect("serialize");
        let back: Settings = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.url, s.url);
        assert_eq!(back.panic_shortcut, s.panic_shortcut);
        assert!((back.ghost_opacity - s.ghost_opacity).abs() < 1e-9);
        assert!((back.hotzone.fw - s.hotzone.fw).abs() < 1e-9);
        assert_eq!(back.bookmarks.len(), 0);
    }

    #[test]
    fn settings_tolerates_missing_fields() {
        // #[serde(default)] lets partial/older prefs files load without error.
        let partial = r#"{"url":"https://x.test"}"#;
        let s: Settings = serde_json::from_str(partial).expect("partial load");
        assert_eq!(s.url, "https://x.test");
        assert_eq!(s.panic_shortcut, "CmdOrControl+Shift+H");
        assert!((s.ghost_opacity - 0.08).abs() < 1e-9);
    }

    #[test]
    fn default_hotzone_covers_whole_window() {
        let hz = Hotzone::default();
        assert!(hz.fx.abs() < 1e-9 && hz.fy.abs() < 1e-9);
        assert!((hz.fw - 1.0).abs() < 1e-9 && (hz.fh - 1.0).abs() < 1e-9);
    }

    #[test]
    fn position_round_trip() {
        // Last-position persistence (Phase 4c).
        let s = Settings { x: Some(123), y: Some(-45), ..Settings::default() };
        let back: Settings =
            serde_json::from_str(&serde_json::to_string(&s).expect("ser")).expect("de");
        assert_eq!(back.x, Some(123));
        assert_eq!(back.y, Some(-45));
    }
}
