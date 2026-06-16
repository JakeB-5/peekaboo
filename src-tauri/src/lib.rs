//! Peekaboo — stealth browser overlay core.
//!
//! Responsibility boundary (docs/architecture.html §①): the Rust core owns
//! every stealth-relevant window property, the panic global shortcut, cursor
//! polling and the reveal state machine. The WebView only renders content.
//!
//! Phase 0 (scaffold): the transparent / borderless / always-on-top window is
//! created from `tauri.conf.json`. The global-shortcut plugin is initialised
//! here so it is ready for the panic shortcut wired up in Phase 1.

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .run(tauri::generate_context!())
        .expect("error while running Peekaboo");
}
