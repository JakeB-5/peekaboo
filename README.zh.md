# 🫣 Peekaboo

> 一款 macOS 隐身浏览器悬浮窗——半透明、始终置顶，悬停时才清晰显示，需要时立刻消失。

[English](README.md) · [한국어](README.ko.md) · [日本語](README.ja.md) · **中文**

---

> [!NOTE]
> **状态：v0.1.0 — 已为 macOS 实现。** 应用已可构建并通过单元测试。可用 `npm run tauri build` 自行构建（见 [BUILD.md](BUILD.md)）。真实的隐身行为（屏幕共享隐藏、全局紧急隐藏、点击穿透、应用隐藏）需在你自己的 Mac 上手动验证，步骤见 [BUILD.md](BUILD.md) 与 [PLAN.md](PLAN.md)。

## Peekaboo 是什么？

Peekaboo 把任意网页作为一个半透明、可调整大小的窗口，悬浮在你正在做的任何事情之上。在默认的**幽灵（ghost）**状态下它几乎不可见，点击会直接穿透到下方的应用。把光标移入热区，内容随即变清晰；按下全局快捷键，它便**不留痕迹地立刻消失。**

整个产品可以浓缩成一句话——**“想看时才看得见，需要时立刻消失。”**

## 功能

| | 功能 | 说明 |
|:--:|---|---|
| 👻 | **幽灵悬浮窗** | 浮于一切之上的半透明、无边框、始终置顶窗口 |
| 🫥 | **悬停显示** | 平时低不透明度 + 点击穿透，光标进入热区后变清晰 |
| 🚨 | **紧急快捷键** | 一个全局快捷键立即隐藏窗口——即使未获焦点也有效 |
| 🙈 | **应用隐藏** | 从程序坞（Dock）、Cmd-Tab、菜单栏中隐藏（Accessory 应用） |
| 🎚️ | **不透明度控制** | 分别设置“平时”和“悬停”的不透明度 |
| 🖥️ | **屏幕共享隐藏** | 仅尽力而为（best-effort）——见下方诚实说明 |

## 使用方法

> 下方图片是用 Peekaboo 真实界面（应用自身的 HTML/CSS）生成的 **UI 渲染图**。若要替换为真实截图，请在 macOS 上捕获——由于 content protection 在 macOS 14 及以下可能让窗口无法被截图，请先在设置中临时关闭**屏幕共享隐藏**。

**1 · 浮起网页。** 启动 Peekaboo，它会以一个淡淡的、始终置顶的**幽灵**悬浮窗打开。在平时的不透明度下几乎不可见，点击会直接穿透到下方的应用。

![淡淡浮于桌面之上、处于幽灵状态的 Peekaboo](docs/images/usage-ghost.png)

**2 · 悬停显示。** 将光标移入悬浮窗的热区，内容便会清晰显示（完全不透明）以便阅读；移开后又淡回幽灵状态。

![光标进入热区后以完全不透明显示的 Peekaboo](docs/images/usage-reveal.png)

**3 · 调整设置。** 点击 ☰ 打开设置——设置 URL 与书签、平时/悬停不透明度、窗口大小、3×3 热区、紧急快捷键以及隐藏开关。

![Peekaboo 的设置面板](docs/images/usage-settings.png)

**4 · 立即消失。** 按下紧急快捷键（默认 ⌘⇧H）或 ✕ 按钮，即使焦点在其他应用，窗口也会立刻消失；再按一次即可回到你离开时的位置。

![按下紧急快捷键隐藏 Peekaboo 之后的桌面](docs/images/usage-panic.png)

## 工作原理

这是一个职责清晰划分的 Tauri v2 应用：**所有隐身能力都由 Rust 内核掌控**——窗口透明、始终置顶、点击穿透、全局紧急快捷键、用于悬停检测的光标轮询，以及显示/紧急隐藏的状态机；而 **WebView 只负责渲染**网页内容和设置界面。

难点不在 UI，而在 macOS 的窗口控制。由于悬浮窗处于点击穿透状态，常规的 DOM 与原生悬停事件都不会触发，因此悬停显示通过轮询全局光标坐标并与热区矩形比较来实现。

## ⚠️ 诚实说明：屏幕共享

通过 macOS content protection 实现的屏幕共享隐藏，**在 macOS 15（Sequoia）及更高版本上无效。** Zoom、Meet、QuickTime 所使用的 ScreenCaptureKit 会捕获合成后的帧缓冲区，并忽略 `NSWindowSharingNone` 标志（[Tauri #14200](https://github.com/tauri-apps/tauri/issues/14200)，暂无已知绕过方法）。它仅在旧版 macOS（≤ 14）和传统截图 API 上有效。

因此，屏幕共享隐藏只作为**辅助手段（best-effort）**，而**屏幕共享时真正的防线是紧急快捷键**（在共享前或共享的瞬间隐藏）。

## 文档

完整的规划与设计文档位于 [`docs/`](docs/)——一套含 SVG/HTML 原型的深色主题 6 页文档：

1. [概览](docs/index.html) — 愿景、威胁→防御模型、关键决策
2. [功能规格](docs/feature-spec.html) — 按优先级划分的功能 + 已核实的 Tauri API
3. [界面设计](docs/screen-design.html) — **可视化原型**：桌面三状态、悬浮窗解剖图、设置面板
4. [架构与行为](docs/architecture.html) — 进程模型、状态机、悬停检测设计
5. [技术栈与结构](docs/tech-stack.html) — 配置、权限、代码骨架、构建
6. [路线图](docs/roadmap.html) — 阶段、验证关卡、风险清单

> 文档为 HTML。可用 `open docs/index.html` 在本地打开，或启用 GitHub Pages 在线浏览。

## 技术栈

- **框架：** Tauri v2（Rust 内核 + 系统 WebView）
- **目标平台：** 仅 macOS
- **前端：** TypeScript + Vite
- **核心插件：** `global-shortcut`（紧急快捷键）

## 状态与路线图

**v0.1.0 — 已实现。** 阶段 0–4 已完成：脚手架 → MVP（悬浮窗 + 紧急隐藏）→ 隐身核心（悬停显示）→ 隐藏 → 设置、持久化与分发。自动化关卡（构建、clippy、eslint、tsc、单元测试）均已通过；真实的隐身行为需在你的 Mac 上手动验证。详见[路线图](docs/roadmap.html)与 [PLAN.md](PLAN.md)。

## 支持的语言

Peekaboo 文档以下列语言提供——这是未来应用内本地化（i18n）将依托的受支持语言集。语言代码遵循 BCP 47。

| 语言 | 代码 | README |
|---|---|---|
| English（英语） | `en` | [README.md](README.md) |
| 한국어（韩语） | `ko` | [README.ko.md](README.ko.md) |
| 日本語（日语） | `ja` | [README.ja.md](README.ja.md) |
| 中文（简体） | `zh` | [README.zh.md](README.zh.md) |

---

<sub>这是一个个人工具。请合理使用。🫣</sub>
