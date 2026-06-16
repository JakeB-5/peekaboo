// Peekaboo frontend entry.
//
// Responsibility boundary (architecture.html §①): the Rust core owns all
// window properties, the global shortcut, cursor polling and the reveal state
// machine. The frontend only renders content and *reflects* state it receives
// via events. Phase 0 just boots; event wiring is added in later phases.

const viewer = document.getElementById("viewer");

if (viewer) {
  // Default state on launch is Ghost (screen-design control table:
  // "고스트(평소) = 앱 실행 후 기본").
  viewer.dataset.state = "ghost";
}

console.info("[peekaboo] frontend booted");
