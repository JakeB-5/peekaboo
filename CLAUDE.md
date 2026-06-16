# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

> 본 문서의 본문은 한글로 작성하되, 코드/명령/식별자는 영어로 표기한다(전역 언어 정책).

## 프로젝트 개요

**Peekaboo**는 회사에서 웹툰을 몰래 보기 위한 데스크톱 스텔스 오버레이 앱이다. 핵심은 "보고 싶을 때만 보이고, 위험할 때 즉시 사라진다"는 한 줄로 요약된다.

요구되는 동작:
- 임의의 웹 페이지(웹툰 뷰어)를 **원하는 크기 / 원하는 투명도**의 떠 있는 창으로 띄운다.
- 평소에는 내용이 거의 드러나지 않다가, **지정한 영역에 마우스가 올라가 있을 때만** 또렷하게 보인다(hover-reveal).
- 들킬 위기에 **전역 단축키(패닉 키) 한 번으로 즉시 숨긴다** — 창에 포커스가 없어도 동작해야 한다.
- 화면 공유 / 화면 녹화에 **잡히지 않아야** 한다(스텔스의 핵심).

## 현재 상태 (중요)

이 저장소는 **아직 비어 있는 greenfield 프로젝트**다. 소스 코드, 빌드 설정, 의존성이 전혀 없다. 따라서 아래의 "기술 스택 / 명령 / 아키텍처"는 *구현된 사실이 아니라 확정된 방향과 예정 설계*다. 첫 작업은 보통 스캐폴딩이며, 그 전까지는 아래 명령들이 동작하지 않는다.

## 확정된 기술 방향

- **Framework**: Tauri v2 (Rust 백엔드 + WebView 프론트엔드). 가벼운 바이너리/메모리 = 스텔스에 유리.
- **Target OS**: macOS 단일 타깃. macOS 전용 API(투명 창, content protection, dock 숨김)에 의존해도 된다.
- **Frontend**: TypeScript + Vite (프레임워크는 스캐폴드 시 확정; 단순 오버레이라 vanilla/React 모두 가능, 과한 의존성 지양).
- **패키지 매니저**: npm (전역 워크플로와 일관).

## 아키텍처 핵심 — "어디서 무엇을 하는가"

이 앱의 난이도는 UI가 아니라 **OS 레벨 창 동작 제어**에 있다. 기능별로 책임이 Rust(Tauri) 측인지 프론트 측인지가 분명히 갈리므로, 이 경계를 먼저 이해해야 생산적이다.

**Rust / `src-tauri` 측 (OS 레벨, 스텔스의 본체)**
- 창 생성 옵션: `transparent`, `decorations: false`, `alwaysOnTop`, `skipTaskbar` 등은 `tauri.conf.json`의 window 설정에서 출발한다. macOS 투명 창은 `macOSPrivateApi` 플래그가 필요하다.
- **패닉/탈출 단축키**: global shortcut 플러그인(Rust `tauri-plugin-global-shortcut` + JS `@tauri-apps/plugin-global-shortcut`)으로 등록. 포커스가 없어도 동작해야 하므로 앱 단축키가 아니라 *전역* 단축키여야 한다. macOS는 이를 위해 **손쉬운 사용(Accessibility) 권한**이 필요할 수 있다 — 권한 미부여 시 단축키가 조용히 실패하는 점을 항상 의심하라.
- **화면 공유 비노출**: 창에 content protection을 켠다(macOS의 `NSWindowSharingNone` 매핑). Tauri의 `set_content_protected(true)` 계열 API를 사용. 이 기능이 빠지면 스텔스가 무의미하므로 1순위로 검증한다.
- **Dock/Cmd-Tab 숨김**: `ActivationPolicy::Accessory`로 dock 아이콘과 앱 스위처 노출을 없앤다.
- **클릭 통과(click-through)**: hover 영역 밖에서는 마우스 이벤트를 아래 창으로 흘려보내도록 `set_ignore_cursor_events(true)`를 토글한다.

**Frontend 측 (표현/상호작용)**
- hover-reveal: 마우스 진입/이탈에 따라 불투명도를 전환. 단, "마우스가 위에 있는지" 판정과 click-through 토글은 결국 Rust의 ignore-cursor-events와 맞물리므로, *프론트의 opacity 전환과 OS의 클릭 통과 상태를 한 곳에서 일관되게 관리*해야 어긋나지 않는다.
- 투명도/크기 조절 UI, 표시할 URL 입력, 패닉 키 매핑 설정.
- 상태(현재 opacity, 표시/숨김, 패닉 여부)는 단일 소스에서 관리하고 Tauri command/event로 Rust와 동기화한다.

**비-자명한 함정 (먼저 확인할 것)**
- 위 macOS 기능(투명·content protection·activation policy·ignore cursor events)은 Tauri 버전에 따라 정확한 API 이름/위치가 다르다. 구현 전 **반드시 Tauri v2 공식 문서로 확인**한다(context7 또는 `document-specialist` 경유). 메모리에 의존하지 말 것.
- 패닉 키는 "숨김"뿐 아니라 **흔적 최소화**(창 위치/스크롤 복원, 직전 상태 비노출)까지 고려해야 진짜 탈출이 된다.

## 개발 명령 (스캐폴딩 완료 후 적용)

아직 동작하지 않는다. Tauri v2 표준 흐름 기준:

```bash
# 개발 실행 (Vite dev server + Tauri 창)
npm run tauri dev

# 프로덕션 빌드 (.app / .dmg)
npm run tauri build
```

프론트엔드 검증(전역 정책의 cache-busting 플래그 사용):
```bash
npx tsc --noEmit
npx eslint --quiet .
npx vitest run --no-cache
```

Rust(`src-tauri`) 측:
```bash
cargo fmt   --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml
cargo test  --manifest-path src-tauri/Cargo.toml
```

## 검증 우선순위

스텔스 앱은 "기능이 된다"가 아니라 "들키지 않는다"가 합격선이다. 변경 후 다음을 실제로 확인한다: 화면 공유 시 창이 보이지 않는가 / 다른 앱이 포커스를 가진 상태에서 패닉 키가 즉시 듣는가 / dock·Cmd-Tab에 노출되지 않는가 / hover 밖에서 클릭이 아래 창으로 통과되는가.

## 작업 방식

프로젝트 고유 사항만 적는다(전역 CLAUDE.md의 단계별·승인 우선·검증 정책이 그대로 적용된다). 스캐폴딩처럼 디렉터리 구조를 한 번에 만드는 작업도 접근 방식을 먼저 합의한 뒤 진행한다.
