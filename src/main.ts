// Peekaboo frontend entry.
//
// Responsibility boundary (architecture.html §①): the Rust core owns all
// window properties, the global shortcut, cursor polling, the reveal state
// machine and persisted settings. The frontend renders content, reflects
// reveal state, and edits settings via the get_settings / save_settings IPC.

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

const DEFAULT_URL = "https://example.com";
const THIRD = 1 / 3;

function need<T extends HTMLElement>(id: string): T {
  const el = document.getElementById(id);
  if (!el) {
    throw new Error(`missing #${id}`);
  }
  return el as T;
}

const appWindow = getCurrentWindow();

const viewer = need<HTMLElement>("viewer");
const content = need<HTMLIFrameElement>("content");
const chrome = need<HTMLElement>("chrome");
const panel = need<HTMLElement>("settings");

const fUrl = need<HTMLInputElement>("f-url");
const fGhost = need<HTMLInputElement>("f-ghost");
const oGhost = need<HTMLOutputElement>("o-ghost");
const fRevealed = need<HTMLInputElement>("f-revealed");
const oRevealed = need<HTMLOutputElement>("o-revealed");
const fWidth = need<HTMLInputElement>("f-width");
const fHeight = need<HTMLInputElement>("f-height");
const fShortcut = need<HTMLInputElement>("f-shortcut");
const fAot = need<HTMLInputElement>("f-aot");
const fSpaces = need<HTMLInputElement>("f-spaces");
const fCp = need<HTMLInputElement>("f-cp");
const bookmarksEl = need<HTMLElement>("bookmarks");
const hzGrid = need<HTMLElement>("hotzone-grid");

// Local mirror of the core's settings; the core remains the source of truth.
let state: Settings = {
  url: DEFAULT_URL,
  ghost_opacity: 0.08,
  revealed_opacity: 1,
  width: 420,
  height: 720,
  hotzone: { fx: 0, fy: 0, fw: 1, fh: 1 },
  panic_shortcut: "CmdOrControl+Shift+H",
  bookmarks: [],
  content_protected: true,
  always_on_top: true,
  spaces_global: true,
};

// Set true only after the initial get_settings succeeds. Until then save() is
// suppressed so a failed load can't persist JS defaults over the user's prefs.
let loaded = false;

function pct(v: number): string {
  return `${Math.round(v * 100)}%`;
}

function applyView(): void {
  const root = document.documentElement;
  root.style.setProperty("--ghost-opacity", String(state.ghost_opacity));
  root.style.setProperty("--revealed-opacity", String(state.revealed_opacity));
}

// Only ever navigate the viewer to http(s). A javascript:/data:/file: URL would
// run in the iframe document context; we never hand arbitrary schemes to it.
function isHttpUrl(url: string): boolean {
  try {
    const proto = new URL(url).protocol;
    return proto === "http:" || proto === "https:";
  } catch {
    return false;
  }
}

function loadUrl(url: string): void {
  content.src = isHttpUrl(url) ? url : "about:blank";
}

async function save(): Promise<boolean> {
  // Never write back to the core before the initial load succeeded, or a failed
  // get_settings would let the JS defaults clobber the user's persisted prefs.
  if (!loaded) {
    return false;
  }
  try {
    await invoke("save_settings", { settings: state });
    return true;
  } catch (err) {
    console.warn("[peekaboo] save_settings failed:", err);
    return false;
  }
}

function hostLabel(url: string): string {
  try {
    return new URL(url).hostname.replace(/^www\./, "");
  } catch {
    return url;
  }
}

function addCurrentBookmark(): void {
  const url = state.url.trim();
  // Only persist http(s) bookmarks so a stored javascript:/data: URL can never
  // be re-applied to the viewer on a later session.
  if (isHttpUrl(url) && !state.bookmarks.includes(url)) {
    state.bookmarks = [...state.bookmarks, url];
    renderBookmarks();
    void save();
  }
}

function renderBookmarks(): void {
  bookmarksEl.replaceChildren();
  for (const url of state.bookmarks) {
    const chip = document.createElement("span");
    chip.className = "chip";

    const load = document.createElement("span");
    load.className = "load";
    load.textContent = hostLabel(url);
    load.title = url;
    load.addEventListener("click", () => {
      state.url = url;
      fUrl.value = url;
      loadUrl(url);
      void save();
    });

    const rm = document.createElement("button");
    rm.type = "button";
    rm.className = "rm";
    rm.textContent = "×";
    rm.title = "삭제";
    rm.addEventListener("click", () => {
      state.bookmarks = state.bookmarks.filter((b) => b !== url);
      renderBookmarks();
      void save();
    });

    chip.append(load, rm);
    bookmarksEl.append(chip);
  }

  const add = document.createElement("button");
  add.type = "button";
  add.className = "rm";
  add.textContent = "+ 추가";
  add.addEventListener("click", addCurrentBookmark);
  bookmarksEl.append(add);
}

function renderHotzone(): void {
  hzGrid.replaceChildren();
  const isThird = Math.abs(state.hotzone.fw - THIRD) < 0.05;
  const onCol = isThird ? Math.round(state.hotzone.fx * 3) : -1;
  const onRow = isThird ? Math.round(state.hotzone.fy * 3) : -1;
  for (let r = 0; r < 3; r++) {
    for (let c = 0; c < 3; c++) {
      const b = document.createElement("button");
      b.type = "button";
      if (r === onRow && c === onCol) {
        b.classList.add("on");
      }
      b.addEventListener("click", () => {
        state.hotzone = { fx: c * THIRD, fy: r * THIRD, fw: THIRD, fh: THIRD };
        renderHotzone();
        void save();
      });
      hzGrid.append(b);
    }
  }
}

function populateForm(): void {
  fUrl.value = state.url;
  fGhost.value = String(Math.round(state.ghost_opacity * 100));
  oGhost.value = pct(state.ghost_opacity);
  fRevealed.value = String(Math.round(state.revealed_opacity * 100));
  oRevealed.value = pct(state.revealed_opacity);
  fWidth.value = String(Math.round(state.width));
  fHeight.value = String(Math.round(state.height));
  fShortcut.value = state.panic_shortcut;
  fAot.checked = state.always_on_top;
  fSpaces.checked = state.spaces_global;
  fCp.checked = state.content_protected;
  renderBookmarks();
  renderHotzone();
}

// Build a Tauri accelerator from a key event. Requires at least one modifier
// and a non-media key (media keys need a CGEventTap → Accessibility perm).
function keyToAccel(e: KeyboardEvent): string | null {
  const mods: string[] = [];
  if (e.metaKey) {
    mods.push("CmdOrControl");
  } else if (e.ctrlKey) {
    mods.push("Control");
  }
  if (e.altKey) {
    mods.push("Alt");
  }
  if (e.shiftKey) {
    mods.push("Shift");
  }
  let key: string | null = null;
  if (/^Key[A-Z]$/.test(e.code)) {
    key = e.code.slice(3);
  } else if (/^Digit[0-9]$/.test(e.code)) {
    key = e.code.slice(5);
  } else if (/^F([1-9]|1[0-9]|2[0-4])$/.test(e.code)) {
    key = e.code;
  }
  if (!key || mods.length === 0) {
    return null;
  }
  return [...mods, key].join("+");
}

// Briefly show a message in the (readonly) shortcut field, then restore it to
// the current bound accelerator. Gives feedback for rejected / invalid combos.
let shortcutHintTimer: number | undefined;
function flashShortcutHint(msg: string): void {
  fShortcut.value = msg;
  if (shortcutHintTimer !== undefined) {
    clearTimeout(shortcutHintTimer);
  }
  shortcutHintTimer = setTimeout(() => {
    fShortcut.value = state.panic_shortcut;
    shortcutHintTimer = undefined;
  }, 1400);
}

// Open/close the settings panel and tell the core, so the hover loop ignores
// the hotzone while it's open (keeps the panel revealed + clickable anywhere).
function setSettingsOpen(open: boolean): void {
  panel.hidden = !open;
  void invoke("set_settings_open", { open });
  if (open) {
    populateForm();
  }
}

// ---- wiring ----
need<HTMLButtonElement>("btn-settings").addEventListener("click", () => {
  setSettingsOpen(panel.hidden);
});
need<HTMLButtonElement>("btn-close").addEventListener("click", () => {
  setSettingsOpen(false);
});
need<HTMLButtonElement>("btn-bookmark").addEventListener("click", addCurrentBookmark);
need<HTMLButtonElement>("btn-panic").addEventListener("click", () => {
  void appWindow.hide();
});
need<HTMLButtonElement>("hz-full").addEventListener("click", () => {
  state.hotzone = { fx: 0, fy: 0, fw: 1, fh: 1 };
  renderHotzone();
  void save();
});

fUrl.addEventListener("change", () => {
  state.url = fUrl.value.trim();
  loadUrl(state.url);
  void save();
});
fGhost.addEventListener("input", () => {
  state.ghost_opacity = Number(fGhost.value) / 100;
  oGhost.value = pct(state.ghost_opacity);
  applyView();
});
fGhost.addEventListener("change", () => void save());
fRevealed.addEventListener("input", () => {
  state.revealed_opacity = Number(fRevealed.value) / 100;
  oRevealed.value = pct(state.revealed_opacity);
  applyView();
});
fRevealed.addEventListener("change", () => void save());
fWidth.addEventListener("change", () => {
  state.width = Math.max(120, Number(fWidth.value) || state.width);
  void save();
});
fHeight.addEventListener("change", () => {
  state.height = Math.max(120, Number(fHeight.value) || state.height);
  void save();
});
fAot.addEventListener("change", () => {
  state.always_on_top = fAot.checked;
  void save();
});
fSpaces.addEventListener("change", () => {
  state.spaces_global = fSpaces.checked;
  void save();
});
fCp.addEventListener("change", () => {
  state.content_protected = fCp.checked;
  void save();
});
fShortcut.addEventListener("keydown", (e) => {
  e.preventDefault();
  const accel = keyToAccel(e);
  if (!accel) {
    flashShortcutHint("수정자 + 키 조합이 필요합니다");
    return;
  }
  const previous = state.panic_shortcut;
  state.panic_shortcut = accel;
  fShortcut.value = accel;
  void (async () => {
    // If the backend rejects the accelerator (or persistence fails), revert so
    // the field never claims a panic key that isn't actually registered.
    if (!(await save())) {
      state.panic_shortcut = previous;
      fShortcut.value = previous;
      flashShortcutHint("단축키를 등록할 수 없습니다");
    }
  })();
});

document.addEventListener("keydown", (e) => {
  if (e.key === "Escape" && !panel.hidden) {
    setSettingsOpen(false);
  }
});

// On-demand focus (architecture §⑥): Accessory apps don't focus on show()
// alone, and we avoid stealing focus on hover. Interacting with the chrome
// (incl. opening settings) focuses the overlay so keyboard input works.
chrome.addEventListener("pointerdown", () => {
  void appWindow.setFocus();
});

// ---- boot ----
// Do NOT point the iframe at any remote host until settings resolve: loading
// DEFAULT_URL eagerly would fire a request to a fixed third-party host on every
// launch (trace leak) and flash the placeholder before the user's real page.
viewer.dataset.state = "ghost";

void (async () => {
  try {
    state = await invoke<Settings>("get_settings");
    loaded = true;
    applyView();
    loadUrl(state.url || DEFAULT_URL);
    populateForm();
  } catch (err) {
    console.warn("[peekaboo] get_settings failed:", err);
    // Stay blank rather than leaking a request to a remote default on failure.
    content.src = "about:blank";
  }
})();

// Reflect the reveal state machine (owned by the core) onto the viewer.
void listen<string>("reveal-state-changed", (event) => {
  viewer.dataset.state = event.payload;
});

console.info("[peekaboo] frontend booted");
