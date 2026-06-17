# Usage screenshots

The "How to use / 사용법 / 使い方 / 使用方法" sections in the README files reference
the files below.

> **The current images are UI renders**, generated from the app's own
> `src/index.html` + `src/styles.css` with headless Chrome — there were no live
> captures yet. They faithfully show the real interface, but are not photos of
> the running app over a live macOS desktop. To replace any of them with a real
> screenshot, capture on macOS and overwrite the file with the **same name**;
> the READMEs update automatically.

To (re)capture for real, use the table below.

| File | Shows | How to capture |
|---|---|---|
| `usage-ghost.png` | **Ghost state** — the overlay floating faintly over a normal app/desktop; clicks pass through. | Launch Peekaboo over some app. The ghost is ~8% opacity, so temporarily raise **idle opacity (평소 투명도)** in Settings until it's visible, then capture a region with ⌘⇧4. |
| `usage-reveal.png` | **Hover-revealed** — content sharp at full opacity, with the top chrome (☰ ★ ⠿ ✕) visible. | Move the cursor into the hot-zone so the overlay reveals, then capture the window region. |
| `usage-settings.png` | **Settings panel** — URL, bookmarks, idle/hover opacity, window size, 3×3 hot-zone, panic shortcut, concealment toggles. | Click ☰ to open Settings, then capture the window. |
| `usage-panic.png` | **After the panic hotkey** — the overlay gone, leaving the clean desktop. | Capture the desktop with the overlay visible, press ⌘⇧H to hide, capture again; use the "after" shot. (An animated `usage-panic.gif` works too.) |

> [!IMPORTANT]
> **Content-protection caveat.** Peekaboo enables **Screen-share hiding**
> (`NSWindowSharingNone`) by default, which can exclude the window from
> screenshots/recordings on **macOS ≤ 14**. Turn **Screen-share hiding off** in
> Settings before capturing, then turn it back on. On **macOS 15+ (Sequoia)**
> this protection is already ineffective (see the README caveat), so capture
> works regardless.

Recommended: PNG, retina (2×), cropped tight to the overlay/region, each under ~500 KB.
