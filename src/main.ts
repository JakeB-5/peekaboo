// Peekaboo frontend entry.
//
// Responsibility boundary (architecture.html §①): the Rust core owns all
// window properties, the global shortcut, cursor polling and the reveal state
// machine. The frontend only renders content and *reflects* state it receives
// via events.

import { listen } from "@tauri-apps/api/event";

// Default URL loaded into the viewer. Becomes a persisted setting in Phase 4.
const DEFAULT_URL = "https://example.com";

const viewer = document.getElementById("viewer");
const content = document.getElementById("content");

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

console.info("[peekaboo] frontend booted");
