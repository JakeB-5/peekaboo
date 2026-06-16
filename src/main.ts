// Peekaboo frontend entry.
//
// Responsibility boundary (architecture.html §①): the Rust core owns all
// window properties, the global shortcut, cursor polling and the reveal state
// machine. The frontend only renders content and *reflects* state it receives
// via events.

import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

// Default URL loaded into the viewer. Becomes a persisted setting in Phase 4.
const DEFAULT_URL = "https://example.com";

const viewer = document.getElementById("viewer");
const content = document.getElementById("content");
const chrome = document.getElementById("chrome");

if (viewer) {
  // Default state on launch is Ghost (screen-design control table:
  // "고스트(평소) = 앱 실행 후 기본").
  viewer.dataset.state = "ghost";
}

if (content instanceof HTMLIFrameElement) {
  content.src = DEFAULT_URL;
}

// Reflect the reveal state machine (owned by the Rust core) onto the viewer.
// Payload is a plain string: "ghost" | "revealed" | "hidden".
void listen<string>("reveal-state-changed", (event) => {
  if (viewer) {
    viewer.dataset.state = event.payload;
  }
});

// On-demand focus (architecture §⑥). The app runs as Accessory, so show()
// alone doesn't grab focus, and we deliberately avoid stealing focus on mere
// hover. Clicking the drag strip focuses the overlay so keyboard input works
// only when the user actually wants it.
if (chrome) {
  const appWindow = getCurrentWindow();
  chrome.addEventListener("pointerdown", () => {
    void appWindow.setFocus();
  });
}

console.info("[peekaboo] frontend booted");
