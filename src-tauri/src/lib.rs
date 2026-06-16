//! Peekaboo — stealth browser overlay core.
//!
//! Responsibility boundary (docs/architecture.html §①): the Rust core owns
//! every stealth-relevant window property, the panic global shortcut, cursor
//! polling and the reveal state machine. The WebView only renders content.
//!
//! Phase 1 (MVP): the transparent / borderless / always-on-top window is
//! created from `tauri.conf.json`; a panic global shortcut toggles the overlay
//! hide/show directly in the Rust handler for minimal latency.

use tauri::Manager;
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
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Peekaboo");
}
