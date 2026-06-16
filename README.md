# 🫣 Peekaboo

> A macOS stealth browser overlay — a transparent, always-on-top web window that reveals on hover and vanishes the instant you need it gone.

**English** · [한국어](README.ko.md) · [日本語](README.ja.md) · [中文](README.zh.md)

---

> [!NOTE]
> **Status: v0.1.0 — implemented for macOS.** The app builds and is unit-tested; build it yourself with `npm run tauri build` (see [BUILD.md](BUILD.md)). The real-world stealth behaviors (screen-share hiding, global panic, click-through, concealment) are verified manually on your own Mac — see the checklists in [BUILD.md](BUILD.md) and [PLAN.md](PLAN.md).

## What is Peekaboo?

Peekaboo floats any web page as a transparent, resizable window that sits above whatever you're doing. In its default **ghost** state it's nearly invisible and clicks pass straight through to the app underneath. Move your cursor into its hot-zone and the content sharpens into view; press the global hotkey and it **disappears instantly**, without a trace.

The whole product fits in one line: **"visible only when you want it, gone the instant you need it."**

## Features

| | Feature | What it does |
|:--:|---|---|
| 👻 | **Ghost overlay** | Transparent, frameless, always-on-top window floating above everything else |
| 🫥 | **Hover-reveal** | Dim + click-through until your cursor enters the hot-zone, then it sharpens |
| 🚨 | **Panic hotkey** | One system-wide shortcut hides the window instantly — even when unfocused |
| 🙈 | **App concealment** | Hidden from the Dock, Cmd-Tab, and menu bar (Accessory app) |
| 🎚️ | **Opacity control** | Separate "idle" and "hover" opacity levels you tune yourself |
| 🖥️ | **Screen-share hiding** | Best-effort only — see the honest caveat below |

## How to use

> The images below are UI renders of Peekaboo's real interface, generated from its own HTML/CSS. To replace them with live screenshots, capture on macOS — first turn **Screen-share hiding** off in Settings, since content protection can exclude the window from screenshots on macOS ≤ 14.

**1 · Float a page.** Launch Peekaboo and it opens as a faint, always-on-top *ghost* overlay. At its idle opacity it's barely there, and clicks pass straight through to the app underneath.

![Peekaboo in its ghost state, floating faintly above the desktop](docs/images/usage-ghost.png)

**2 · Reveal on hover.** Move your cursor into the overlay's hot-zone and the content sharpens to full opacity so you can read it. Move away and it fades back to a ghost.

![Peekaboo at full opacity once the cursor enters the hot-zone](docs/images/usage-reveal.png)

**3 · Tune it.** Click ☰ to open Settings — set the URL and bookmarks, the idle/hover opacity, window size, the 3×3 hot-zone, the panic hotkey, and the concealment toggles.

![Peekaboo's settings panel](docs/images/usage-settings.png)

**4 · Vanish.** Press the panic hotkey (default ⌘⇧H) — or the ✕ button — and the window disappears instantly, even when another app is focused. Press it again to bring it back exactly where you left off.

![The desktop after the panic hotkey has hidden Peekaboo](docs/images/usage-panic.png)

## How it works

A Tauri v2 app with a clear split: a **Rust core owns all the stealth** — window transparency, always-on-top, click-through, the global panic hotkey, cursor-polling for hover detection, and the reveal/panic state machine — while the **WebView only renders** the web content and the settings UI.

The hard part isn't the UI; it's macOS window control. Because the overlay is click-through, normal DOM/native hover events don't fire, so hover-reveal is driven by polling the global cursor position against a hot-zone rectangle.

## ⚠️ Honest caveat: screen sharing

Hiding from screen sharing via macOS content protection **does not work on macOS 15 (Sequoia) and later.** ScreenCaptureKit — used by Zoom, Meet, and QuickTime — captures the composited framebuffer and ignores the `NSWindowSharingNone` flag ([Tauri #14200](https://github.com/tauri-apps/tauri/issues/14200), no known workaround). It only helps on macOS ≤ 14 and against legacy screenshot APIs.

So screen-share hiding is treated as **best-effort**, and the **real defense during a screen share is the panic hotkey** — hide before or the moment you start sharing.

## Documentation

The full planning & design docs live in [`docs/`](docs/) — a 6-page, dark-themed set with working SVG/HTML mockups:

1. [Overview](docs/index.html) — vision, threat→defense model, key decisions
2. [Feature spec](docs/feature-spec.html) — features by priority + verified Tauri APIs
3. [Screen design](docs/screen-design.html) — **visual mockups**: 3-state desktop scenes, overlay anatomy, settings panel
4. [Architecture & behavior](docs/architecture.html) — process model, state machine, hover-detection design
5. [Tech stack & structure](docs/tech-stack.html) — config, capabilities, code skeletons, build
6. [Roadmap](docs/roadmap.html) — phases, verification gates, risk register

> The docs are HTML. Open them locally with `open docs/index.html`, or enable GitHub Pages to browse online.

## Tech stack

- **Framework:** Tauri v2 (Rust core + system WebView)
- **Target:** macOS only
- **Frontend:** TypeScript + Vite
- **Key plugin:** `global-shortcut` (panic hotkey)

## Status & roadmap

**v0.1.0 — implemented.** Phases 0–4 are done: scaffold → MVP (overlay + panic) → stealth core (hover-reveal) → concealment → settings, persistence & distribution. The automated gates (build, clippy, eslint, tsc, unit tests) pass; the real-world stealth behaviors are verified manually on your Mac. See the [roadmap](docs/roadmap.html) and [PLAN.md](PLAN.md).

---

<sub>A personal tool. Use responsibly. 🫣</sub>
