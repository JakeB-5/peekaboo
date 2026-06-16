// Peekaboo frontend entry.
//
// Responsibility boundary (architecture.html §①): the Rust core owns all
// window properties, the global shortcut, cursor polling and the reveal state
// machine. The frontend only renders content and *reflects* state it receives
// via events. Phase 0 just boots; event wiring is added in later phases.

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

console.info("[peekaboo] frontend booted");
