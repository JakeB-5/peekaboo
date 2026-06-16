# Peekaboo 구현 계획 · 단계별 완료 기준

> 본 문서는 `docs/` 기획 세트(특히 `docs/roadmap.html`)를 실제 구현으로 옮기기 위한 **실행 계획**이자 **완료 체크리스트**다.
> 각 Phase는 `산출물`, `자동 검증`(에이전트가 실행 가능), `수동 검증`(사용자 머신에서 GUI 육안·상호작용 필요)으로 나눈다.
> 스텔스 앱의 합격선은 "기능이 된다"가 아니라 "들키지 않는다"이므로, 수동 검증 항목은 실제 위협 시나리오로 기술한다.

## 검증 환경의 한계 (정직한 고지)

- 빌드·컴파일·린트·dev 서버 기동·설정 스키마 유효성은 **자동 검증**으로 확인한다.
- 투명창의 실제 비침, 다른 앱 포커스 상태에서의 패닉 키 발화, 클릭 통과, 화면 공유 노출 여부 등 **GUI 육안/전역 입력 상호작용**은 헤드리스 환경에서 확인 불가하므로 **수동 검증**으로 분리하고, 재현 절차를 명시한다.
- 환경: macOS 26.x(Sequoia 이후) → `contentProtected`는 화면 공유를 **막지 못함**(문서에 명시된 확정 한계). 따라서 Phase 3의 화면공유 게이트는 "통과"가 아니라 "노출됨을 기록"이 목표다.

## 기술 베이스라인 (검증된 버전)

- Rust: stable (rustup), Tauri 코어 `tauri = "2"`, `tauri-build = "2"`
- 플러그인: `tauri-plugin-global-shortcut = "2"` (+ JS `@tauri-apps/plugin-global-shortcut` 2.3.x)
- CLI: `@tauri-apps/cli` 2.11.x, API: `@tauri-apps/api` 2.11.x
- 프론트: TypeScript + Vite(vanilla, 최소 의존성 원칙) — Vite root = `src/`
- feature: `macos-private-api`(투명창 필수, App Store 부적격 — 개인 도구라 무관)

---

## Phase 0 · 스캐폴딩

**목표** 빈 투명·무테 창이 떠서 `tauri dev`가 도는 상태.

**산출물**
- [x] `package.json` / `tsconfig.json` / `vite.config.ts` (Vite root=src, port 1420) + `eslint.config.js`
- [x] `src/index.html` · `src/main.ts` · `src/styles.css`
- [x] `src-tauri/Cargo.toml` (`macos-private-api` feature, global-shortcut 플러그인)
- [x] `src-tauri/tauri.conf.json` (투명·무테·항상위·skipTaskbar·`visible:true`=고스트 기본)
- [x] `src-tauri/build.rs` · `src-tauri/src/main.rs` · `src-tauri/src/lib.rs`
- [x] `src-tauri/capabilities/default.json` (window·global-shortcut 권한 선언)
- [x] `src-tauri/icons/` (PIL로 생성한 아이콘 세트 — macOS만)

**자동 검증**
- [x] `npm install` 성공 (exit 0)
- [x] `npx tsc --noEmit` 에러 0 (`npm run build`)
- [x] `eslint .` 0 errors / 0 warnings
- [x] `cargo build --manifest-path src-tauri/Cargo.toml` 성공 (exit 0, 40.6s) — 의존성·feature·capabilities 권한 식별자 검증됨
- [x] `tauri.conf.json` 스키마 유효(빌드가 곧 검증)

**수동 검증 (사용자)**
- [ ] `npm run tauri dev` 실행 시 투명·무테 창이 뜨고 배경이 비침, 콘솔 에러 없음 (헤드리스 환경 확인 불가 — 사용자 머신 필요)

---

## Phase 1 · MVP — 오버레이 + 패닉

**목표** "띄우고 → 즉시 숨긴다" 수직 경로 완성.

**산출물**
- [x] 웹 URL 로드(설정 가능한 기본 URL, iframe), 항상-위, 기본 크기/위치
- [x] **패닉 전역 단축키**(기본 `Cmd+Shift+H`) — Rust 핸들러에서 직접 토글 hide/show (저지연)
- [x] 복귀 시 보던 위치/스크롤 유지(창을 destroy하지 않고 hide/show)
- [x] 기본 투명도(고스트 opacity) CSS 적용
- [x] 드래그 영역(`data-tauri-drag-region`) 정의(무테 창 이동 — 상단 chrome 스트립)

**자동 검증**
- [x] `cargo build` 성공, `cargo clippy` 경고 0 (Shortcut `Copy` clone 제거)
- [x] 단축키 단일 등록 보장(setup에서 1회만 register — os error 22 회피)
- [x] `tsc --noEmit` / eslint 통과
- [x] **런타임 스모크**: `tauri dev`로 앱 기동 확인 — Vite ready → cargo Finished → `Running target/debug/peekaboo`, 패닉/초기화 에러 없이 실행됨(초기화 경로 검증)

**수동 검증 (사용자 — 위협 시나리오)** *(헤드리스 확인 불가 — 전역 입력·육안 필요)*
- [ ] **즉시 탈출**: 다른 앱이 포커스인 상태에서 패닉 키(`⌘⇧H`) → 창이 체감 지연 없이 사라짐
- [ ] **복귀 무결성**: 다시 패닉 키 → 보던 위치·스크롤 유지된 채 복귀

---

## Phase 2 · 스텔스 본체 — Hover-Reveal

**목표** 평소 고스트(클릭 통과) ↔ 핫존 호버 시 노출. 상태 머신은 Rust 단일 소스.

**산출물**
- [x] 클릭 통과 토글 `set_ignore_cursor_events(bool)` (Ghost=true / Revealed=false)
- [x] 커서 폴링 루프(`cursor_position`, 40ms, 엣지 트리거)로 핫존 히트테스트
- [x] 핫존 rect 계산(`outer_position` + `inner_size`; 기본 핫존=창 전체, 물리좌표 일치로 scale 불필요 — 서브핫존은 P4)
- [x] 노출 상태 머신(Ghost↔Revealed; Hidden은 패닉, Focused는 P3) — Rust 폴링 스레드 보유, 숨김 중 hover 전이 중단
- [x] `reveal-state-changed` 이벤트 emit → 프론트 `data-state` opacity 동기화
- [x] 평소/호버 투명도 분리(CSS, `.12s` 트랜지션)

**자동 검증**
- [x] `cargo build` / `clippy`(무경고) / `tsc` / eslint 통과 — `cursor_position`·`set_ignore_cursor_events`·`outer_position`·`inner_size`·`emit` API 검증됨
- [x] 엣지 트리거(상태 변화 시에만 토글) + `is_visible` 게이트로 폴링 비용·thread 안전성 확보
- [x] **런타임 스모크**: `tauri dev`로 앱 기동 + hover 루프 ~6초 가동, 스레드 패닉 없음

**수동 검증 (사용자 — 위협 시나리오)** *(헤드리스 확인 불가 — 커서 이동·육안 필요)*
- [ ] **클릭 통과**: 오버레이 밖(고스트)에서 아래 앱 클릭이 통과됨
- [ ] **호버 노출**: 오버레이 위로 커서 진입 시 또렷↔이탈 시 고스트 전환
- [ ] 폴링 CPU 사용이 수용 범위(체감)

---

## Phase 3 · 은폐 강화

**목표** Dock·Cmd-Tab 은폐 + 온디맨드 포커스 + Spaces 전역 + content protection(한계 실측).

**산출물**
- [ ] `ActivationPolicy::Accessory`(Dock·Cmd-Tab·메뉴바 숨김) — `App`에서 호출
- [ ] 입력 필요 시 `set_focus()` 온디맨드 활성화
- [ ] `set_visible_on_all_workspaces(true)`(Spaces 전역)
- [ ] `set_content_protected(true)` 적용 + 코드/문서에 macOS 15+ 무효 명시

**자동 검증**
- [ ] `cargo build` / `clippy` / `tsc` / eslint 통과
- [ ] activation policy가 `App`(AppHandle 아님) 시점에서 호출되는지 리뷰

**수동 검증 (사용자 — 위협 시나리오)**
- [ ] **앱 은폐**: Dock·Cmd-Tab·메뉴바 어디에도 안 보임
- [ ] 입력 필요 시 포커스가 잡힘(스크롤/클릭)
- [ ] **화면공유(기록)**: Zoom/Meet/QuickTime 공유 시 창 노출 여부를 실측·기록(현 macOS는 노출 예상)

---

## Phase 4 · 폴리시 + 배포

**목표** 설정 UI·상태 영속화·마지막 위치 복원·빌드 경로 정리.

**산출물**
- [ ] 설정 UI(크기·평소/호버 투명도·핫존 위치·패닉 단축키·북마크·은폐 토글)
- [ ] 상태 영속화(`tauri-plugin-store`) — 설정·북마크·마지막 URL
- [ ] 재실행 시 설정·북마크·마지막 위치 복원
- [ ] 명령·이벤트 계약 구현(`set_opacity`·`set_hotzone`·`set_panic_shortcut`·`load_url`·`save_bookmark`)
- [ ] macOS 빌드 경로 정리(`tauri build`, 미서명 실행 안내: `xattr -dr com.apple.quarantine`)

**자동 검증**
- [ ] `cargo build` / `clippy` / `tsc` / eslint 통과
- [ ] `npm run tauri build`로 `.app`/`.dmg` 산출(시간 허용 시) 또는 `cargo build --release` 성공

**수동 검증 (사용자)**
- [ ] 재실행 시 설정·북마크·마지막 위치 복원
- [ ] 빌드 산출물이 본인 머신에서 정상 실행

---

## 진행 로그

- (작성 시작) Phase 0 착수 전 — 계획 수립 및 Rust 툴체인 설치 완료.
- **Phase 0 완료** — 스캐폴딩 전체 작성, 자동 검증 5종 통과(npm/tsc/eslint/vite/cargo build). 아키텍처 결정: 메인 창 = 우리 프론트엔드(`src/`), 원격 콘텐츠는 컨테이너(iframe)로 로드해 opacity 제어. 기본 상태 = 고스트(`visible:true`).
- **Phase 1 완료** — 패닉 전역 단축키(`⌘⇧H`, Rust 핸들러 직접 토글), iframe 뷰어, 드래그 핸들. 자동 검증 + `tauri dev` 런타임 스모크(앱 기동·무패닉) 통과. 수동 검증(전역 입력 즉시 탈출/복귀)은 사용자 머신 필요.
- **Phase 2 완료** — Hover-Reveal: 커서 폴링 스레드(40ms, 엣지 트리거) + 클릭통과 토글 + Ghost↔Revealed 상태 머신(Rust) + `reveal-state-changed` 이벤트→프론트 opacity. 자동 검증 + hover 루프 런타임 스모크(무패닉) 통과. 수동(클릭통과·호버전환)은 사용자 머신 필요.
