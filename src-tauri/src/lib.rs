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
use std::sync::atomic::{AtomicBool, Ordering};
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
        Self {
            fx: 0.0,
            fy: 0.0,
            fw: 1.0,
            fh: 1.0,
        }
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

/// Whether the settings panel is open. While true, the hover loop ignores the
/// hotzone and keeps the overlay revealed + click-catching so the panel stays
/// usable wherever the cursor is.
type SettingsOpen = Arc<AtomicBool>;

/// Cursor polling interval (~25 fps). Trade-off between reveal responsiveness
/// and idle CPU (roadmap risk #4). Edge-triggered.
const HOVER_POLL: Duration = Duration::from_millis(40);

// ---- persistence -------------------------------------------------------

fn settings_path(app: &AppHandle) -> Option<PathBuf> {
    // A plain filename keeps the on-disk trace low-key (the dir still carries
    // the bundle identifier — acceptable for a personal tool).
    app.path()
        .app_config_dir()
        .ok()
        .map(|d| d.join("prefs.json"))
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
    // Log a failure rather than swallow it: a silently lost content-protection
    // call means the overlay runs exposed, which manual verification must catch.
    if let Err(e) = win.set_content_protected(s.content_protected) {
        eprintln!("[peekaboo] set_content_protected failed: {e}");
    }
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
            // Re-assert stealth-critical window properties on every re-show:
            // macOS can drop content protection / always-on-top across some
            // hide/show or display-reconfiguration transitions, and the panic
            // re-show is the most safety-critical moment to guarantee them.
            if let Some(state) = app.try_state::<SharedSettings>() {
                if let Ok(s) = state.lock() {
                    apply_window_settings(&win, &s);
                }
            }
            // Start from a safe ghost state: click-through on and content
            // ghosted, so the overlay never reappears at the previous revealed
            // opacity (visible flash) or swallows clicks until the hover loop
            // re-evaluates the cursor on its next poll.
            if let Err(e) = win.set_ignore_cursor_events(true) {
                eprintln!("[peekaboo] set_ignore_cursor_events failed: {e}");
            }
            let _ = win.emit("reveal-state-changed", "ghost");
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

/// Is the point (physical px) on any currently-attached monitor? Used to avoid
/// restoring the overlay off-screen after a monitor / display-layout change.
fn point_on_any_monitor(win: &WebviewWindow, x: i32, y: i32) -> bool {
    match win.available_monitors() {
        Ok(monitors) => monitors.iter().any(|m| {
            let p = m.position();
            let s = m.size();
            x >= p.x && y >= p.y && x < p.x + s.width as i32 && y < p.y + s.height as i32
        }),
        // If monitors can't be enumerated, don't block the restore.
        Err(_) => true,
    }
}

/// Reveal state machine driver (architecture.html §④): poll the global cursor,
/// hit-test the (configurable) hotzone, flip Ghost/Revealed on edges only.
/// Suspended while hidden (panic); re-evaluated on the next show.
fn spawn_hover_loop(win: WebviewWindow, settings: SharedSettings, settings_open: SettingsOpen) {
    thread::spawn(move || {
        let mut inside_prev: Option<bool> = None;
        loop {
            thread::sleep(HOVER_POLL);

            if !win.is_visible().unwrap_or(false) {
                inside_prev = None;
                continue;
            }

            // While the settings panel is open, ignore the hotzone: keep the
            // overlay revealed and click-catching so every control is reachable
            // regardless of cursor position.
            if settings_open.load(Ordering::Relaxed) {
                if inside_prev != Some(true) {
                    if let Err(e) = win.set_ignore_cursor_events(false) {
                        eprintln!("[peekaboo] set_ignore_cursor_events failed: {e}");
                    }
                    let _ = win.emit("reveal-state-changed", "revealed");
                    inside_prev = Some(true);
                }
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
                if let Err(e) = win.set_ignore_cursor_events(!inside) {
                    eprintln!("[peekaboo] set_ignore_cursor_events failed: {e}");
                }
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
    // A read must never crash the command layer: recover the guard if the mutex
    // was poisoned, mirroring save_settings' graceful handling.
    state.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

/// The WebView reports settings-panel open/close so the hover loop can ignore
/// the hotzone while it is open. Applies immediately on open for instant
/// feedback (no wait for the next poll).
#[tauri::command]
fn set_settings_open(app: AppHandle, state: State<'_, SettingsOpen>, open: bool) {
    state.store(open, Ordering::Relaxed);
    if open {
        if let Some(win) = app.get_webview_window("main") {
            let _ = win.set_ignore_cursor_events(false);
            let _ = win.emit("reveal-state-changed", "revealed");
        }
    }
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
        // Register the NEW accelerator first; only drop the old one once the new
        // one is live. If registration fails we keep the old panic key intact
        // (never leave the user with no working escape) and abort before
        // mutating state, so get_settings never reports an unregistered key.
        gs.register(settings.panic_shortcut.as_str())
            .map_err(|e| e.to_string())?;
        let _ = gs.unregister(old.as_str());
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
    let settings_open: SettingsOpen = Arc::new(AtomicBool::new(false));
    let settings_open_for_loop = settings_open.clone();

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
        .manage(settings_open)
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            set_settings_open
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            let loaded = load_settings(&handle);
            if let Ok(mut guard) = settings_for_setup.lock() {
                *guard = loaded.clone();
            }

            // Register the panic shortcut exactly once (os error 22 on double).
            // If the persisted accelerator is invalid (e.g. a hand-edited or
            // corrupt prefs.json), fall back to the known-good default instead
            // of aborting launch — a stealth overlay that refuses to start is
            // worse than one running with the default panic key.
            if app
                .global_shortcut()
                .register(loaded.panic_shortcut.as_str())
                .is_err()
            {
                let def = Settings::default().panic_shortcut;
                let _ = app.global_shortcut().register(def.as_str());
                if let Ok(mut guard) = settings_for_setup.lock() {
                    guard.panic_shortcut = def;
                }
                eprintln!(
                    "[peekaboo] persisted panic shortcut '{}' could not be registered; \
                     fell back to default",
                    loaded.panic_shortcut
                );
            }

            // Hide from Dock / Cmd-Tab. Must be on `App`, not `AppHandle` (#9244).
            #[cfg(target_os = "macos")]
            app.set_activation_policy(ActivationPolicy::Accessory);

            if let Some(win) = app.get_webview_window("main") {
                apply_window_settings(&win, &loaded);

                // Restore the last window position (physical pixels) only if it
                // still lands on an attached monitor — a display change could
                // otherwise strand the decorationless overlay off-screen with no
                // titlebar or Dock entry to recover it.
                if let (Some(x), Some(y)) = (loaded.x, loaded.y) {
                    if point_on_any_monitor(&win, x, y) {
                        let _ = win.set_position(PhysicalPosition::new(x, y));
                    }
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
                    tauri::WindowEvent::Resized(size) => {
                        // Reflect a manual (edge-drag) resize into the source-of-
                        // truth settings and the frontend, so the panel shows the
                        // live size and the next save doesn't snap the window back.
                        if let Some(w) = handle_for_events.get_webview_window("main") {
                            let scale = w.scale_factor().unwrap_or(1.0);
                            let lw = size.width as f64 / scale;
                            let lh = size.height as f64 / scale;
                            if let Ok(mut s) = settings_for_events.lock() {
                                s.width = lw;
                                s.height = lh;
                            }
                            let _ = w.emit("window-resized", (lw, lh));
                        }
                    }
                    tauri::WindowEvent::CloseRequested { .. } => {
                        if let Ok(s) = settings_for_events.lock() {
                            let _ = persist_settings(&handle_for_events, &s);
                        }
                    }
                    _ => {}
                });

                spawn_hover_loop(
                    win,
                    settings_for_setup.clone(),
                    settings_open_for_loop.clone(),
                );
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
        let s = Settings {
            x: Some(123),
            y: Some(-45),
            ..Settings::default()
        };
        let back: Settings =
            serde_json::from_str(&serde_json::to_string(&s).expect("ser")).expect("de");
        assert_eq!(back.x, Some(123));
        assert_eq!(back.y, Some(-45));
    }
}
