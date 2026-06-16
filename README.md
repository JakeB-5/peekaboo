# 🫣 Peekaboo

> A macOS stealth browser overlay — a transparent, always-on-top web window that reveals on hover and vanishes the instant you need it gone.

**English** · [한국어](README.ko.md) · [日本語](README.ja.md) · [中文](README.zh.md)

---

> [!NOTE]
> **Status: planning / design stage.** This repository currently holds the planning & design document set only — there is no application code yet. The stack is decided (Tauri v2, macOS) and the architecture is specified.

## What is Peekaboo?

Peekaboo floats any web page as a transparent, resizable window that sits above whatever you're doing. In its default **ghost** state it's nearly invisible and clicks pass straight through to the app underneath. Move your cursor into its hot-zone and the content sharpens into view; press the global hotkey and it **disappears instantly**, without a trace.

The whole product fits in one line: **"visible only when you want it, gone the instant you need it."**

## Features (planned)

| | Feature | What it does |
|:--:|---|---|
| 👻 | **Ghost overlay** | Transparent, frameless, always-on-top window floating above everything else |
| 🫥 | **Hover-reveal** | Dim + click-through until your cursor enters the hot-zone, then it sharpens |
| 🚨 | **Panic hotkey** | One system-wide shortcut hides the window instantly — even when unfocused |
| 🙈 | **App concealment** | Hidden from the Dock, Cmd-Tab, and menu bar (Accessory app) |
| 🎚️ | **Opacity control** | Separate "idle" and "hover" opacity levels you tune yourself |
| 🖥️ | **Screen-share hiding** | Best-effort only — see the honest caveat below |

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

Planning is complete; implementation has not started. Planned path: **scaffold → MVP (overlay + panic) → stealth core (hover-reveal) → concealment → polish & distribution.** See the [roadmap](docs/roadmap.html).

---

<sub>A planning exercise for a personal tool. Use responsibly. 🫣</sub>
