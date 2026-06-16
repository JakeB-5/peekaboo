//! Peekaboo — stealth browser overlay core.
//!
//! Responsibility boundary (docs/architecture.html §①): the Rust core owns
//! every stealth-relevant window property, the panic global shortcut, cursor
//! polling and the reveal state machine. The WebView only renders content.
//!
//! Phase 2 (stealth core): adds the hover-reveal state machine. A cursor
//! polling loop hit-tests the global cursor against the hotzone and flips the
//! window between Ghost (faint, click-through) and Revealed (opaque, cursor
//! events captured) — the only robust approach, since click-through suppresses
//! every WebView/native mouse event.

use std::{thread, time::Duration};

use tauri::{Emitter, Manager, PhysicalPosition, WebviewWindow};
use tauri_plugin_global_shortcut::{
    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
};

/// Default panic shortcut: Cmd+Shift+H.
///
/// A standard modifier+key combo is routed by macOS via `RegisterEventHotKey`,
/// so it fires even when Peekaboo is unfocused and needs no Accessibility
/// permission. Media keys are avoided on purpose (they require a CGEventTap and
/// therefore the Accessibility permission).
fn panic_shortcut() -> Shortcut {
    Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyH)
}

/// Toggle the overlay between hidden and visible.
///
/// The window is hidden (not destroyed), so scroll position and loaded content
/// are preserved and re-showing restores the exact prior view ("복귀 무결성").
fn toggle_overlay(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        if win.is_visible().unwrap_or(false) {
            let _ = win.hide();
        } else {
            let _ = win.show();
            let _ = win.set_focus();
        }
    }
}

/// Cursor polling interval (~25 fps). Trade-off between reveal responsiveness
/// and idle CPU (roadmap risk #4). Edge-triggered: native APIs are touched only
/// when the inside/outside result actually changes.
const HOVER_POLL: Duration = Duration::from_millis(40);

/// Is the cursor (desktop physical coords) within the reveal hotzone?
///
/// Phase 2 default hotzone = the whole window. `outer_position` and
/// `inner_size` are both physical, matching `cursor_position`, so no scale
/// conversion is needed here; sub-window hotzones (Phase 4) will fold in
/// `scale_factor`.
fn hotzone_contains(win: &WebviewWindow, cursor: PhysicalPosition<f64>) -> bool {
    let (Ok(origin), Ok(size)) = (win.outer_position(), win.inner_size()) else {
        return false;
    };
    let left = origin.x as f64;
    let top = origin.y as f64;
    cursor.x >= left
        && cursor.x <= left + size.width as f64
        && cursor.y >= top
        && cursor.y <= top + size.height as f64
}

/// Reveal state machine driver (architecture.html §④).
///
/// Click-through and hover detection are intrinsically in conflict: once
/// `set_ignore_cursor_events(true)` is on, the WebView receives no mouse events
/// at all. So we poll the global cursor and hit-test the hotzone, flipping
/// native state only on edges:
///   inside  → Revealed (opaque, cursor events captured)
///   outside → Ghost    (faint, click-through)
/// While the window is hidden (panic), hover transitions are suspended and
/// re-evaluated cleanly on the next show.
fn spawn_hover_loop(win: WebviewWindow) {
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
            let inside = hotzone_contains(&win, cursor);

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

pub fn run() {
    // `Shortcut` is `Copy`, so the value is copied into each `move` closure.
    let panic = panic_shortcut();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    // Run the panic toggle straight from the Rust handler — no
                    // IPC round-trip — so the overlay hides with minimal delay.
                    if event.state() == ShortcutState::Pressed && shortcut == &panic {
                        toggle_overlay(app);
                    }
                })
                .build(),
        )
        .setup(move |app| {
            // Register exactly once; double registration panics with
            // `Invalid argument (os error 22)`.
            app.global_shortcut().register(panic)?;

            // Start the cursor-polling reveal loop on the main window.
            if let Some(win) = app.get_webview_window("main") {
                spawn_hover_loop(win);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Peekaboo");
}
