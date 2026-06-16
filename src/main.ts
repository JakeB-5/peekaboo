// Peekaboo frontend entry.
//
// Responsibility boundary (architecture.html §①): the Rust core owns all
// window properties, the global shortcut, cursor polling, the reveal state
// machine and persisted settings. The frontend renders content and reflects
// state it receives via events / commands.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface Hotzone {
  fx: number;
  fy: number;
  fw: number;
  fh: number;
}

interface Settings {
  url: string;
  ghost_opacity: number;
  revealed_opacity: number;
  width: number;
  height: number;
  hotzone: Hotzone;
  panic_shortcut: string;
  bookmarks: string[];
  content_protected: boolean;
  always_on_top: boolean;
  spaces_global: boolean;
}

// Fallback URL shown before settings load (and if loading fails).
const DEFAULT_URL = "https://example.com";

const viewer = document.getElementById("viewer");
const content = document.getElementById("content");
const chrome = document.getElementById("chrome");

if (viewer) {
  // Default state on launch is Ghost (screen-design: "고스트(평소) = 기본").
  viewer.dataset.state = "ghost";
}
if (content instanceof HTMLIFrameElement) {
  content.src = DEFAULT_URL;
}

/** Apply persisted settings to the view (opacity vars + loaded URL). */
function applySettings(s: Settings): void {
  const root = document.documentElement;
  root.style.setProperty("--ghost-opacity", String(s.ghost_opacity));
  root.style.setProperty("--revealed-opacity", String(s.revealed_opacity));
  if (content instanceof HTMLIFrameElement && s.url) {
    content.src = s.url;
  }
}

// Pull persisted settings from the Rust core on boot.
void (async () => {
  try {
    const s = await invoke<Settings>("get_settings");
    applySettings(s);
  } catch (err) {
    console.warn("[peekaboo] get_settings failed:", err);
  }
})();

// Reflect the reveal state machine (owned by the core) onto the viewer.
// Payload is a plain string: "ghost" | "revealed" | "hidden".
void listen<string>("reveal-state-changed", (event) => {
  if (viewer) {
    viewer.dataset.state = event.payload;
  }
});

// On-demand focus (architecture §⑥): Accessory apps don't focus on show()
// alone, and we avoid stealing focus on hover. Clicking the drag strip focuses
// the overlay so keyboard input works only when the user wants it.
if (chrome) {
  const appWindow = getCurrentWindow();
  chrome.addEventListener("pointerdown", () => {
    void appWindow.setFocus();
  });
}

console.info("[peekaboo] frontend booted");
