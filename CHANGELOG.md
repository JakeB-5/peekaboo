# Changelog

본 프로젝트의 주요 변경 사항을 기록한다. 형식은 [Keep a Changelog](https://keepachangelog.com/ko/1.1.0/),
버전 체계는 [Semantic Versioning](https://semver.org/lang/ko/)을 따른다.

## [0.1.2] - 2026-06-17

주요 사이트 로딩(아키텍처 변경) + 설정 UX 수정 + 배포 서명.

### Added
- 페이지를 iframe이 아닌 **네이티브 webview(최상위 로드)** 로 표시 — X-Frame-Options로 임베드를 막는 사이트(Google·Naver 등)도 로드된다(Tauri `unstable` 멀티-webview).

### Fixed
- 설정 패널이 열려 있는 동안 핫존 무시 — 커서 위치와 무관하게 패널을 조작할 수 있다.
- 스킴 없는 주소(`example.com`)도 로드 — 자동으로 `https://`를 붙여 정규화.
- 드래그로 창 크기 변경 시 설정에 반영되고, 다음 저장에 옛 크기로 되돌려지지 않는다.

### Changed
- 콘텐츠가 네이티브 webview이므로 고스트/노출 투명도를 **NSWindow alpha**(네이티브)로 적용.
- 릴리스 `.app`을 ad-hoc 서명(`signingIdentity: "-"`)해 다운로드한 빌드가 "손상"으로 차단되지 않는다(첫 실행은 우클릭 → 열기; 여전히 미공증).

## [0.1.1] - 2026-06-17

0.1.0 이후의 안전성 수정과 하드닝, 배포 자동화. 사용자 기능 변경은 없다(스텔스 동작 동일).

### Fixed
- 패닉 단축키 재등록 안전성 — 새 단축키를 먼저 등록한 뒤 옛 것을 해제하므로, 등록 실패 시에도 작동하는 패닉 키가 사라지지 않는다. 손상된 `prefs.json`의 단축키는 앱을 중단시키지 않고 기본값으로 폴백.
- 패닉 재표시 시 content protection 재적용 + ghost·클릭 통과 상태로 시작(풀 불투명 플래시·클릭 흡수·보호 유실 방지).
- 마지막 창 위치는 연결된 모니터 안에 있을 때만 복원(화면 밖 실종 방지). poisoned lock에서 `get_settings`가 패닉 대신 복구. 스텔스 핵심 호출 실패를 로깅.
- Frontend — 초기 로드 성공 전 저장 차단(기본값이 prefs를 덮어쓰는 사고 방지), 패닉 단축키 저장 실패 시 롤백·안내, 뷰어·북마크를 http(s)로 제한, 설정 해석 전 기본 URL 선로딩 제거.

### Security
- `csp: null` → 제한 CSP, 뷰어 iframe `sandbox`(top-navigation·popup 차단).
- capability ACL을 최소권한으로 트림.
- 안정 bundle identifier `com.jakeb5.peekaboo`로 변경(스캐폴드 placeholder 제거).

### Changed
- 릴리스 번들 minify + sourcemap 미배포.

### Added
- 4개 언어 README에 "사용법" 섹션 + UI 렌더 이미지.
- GitHub Actions — CI(typecheck·lint·build·clippy·test) + 릴리스 워크플로(태그 푸시 시 macOS `.dmg` 빌드·발행).
- `CLAUDE.md`에 Git 워크플로 + 릴리스(버전·태그) 규칙.

## [0.1.0] - 2026-06-16

최초 구현. `docs/` 기획 세트(Peekaboo — macOS 스텔스 브라우저 오버레이)를 Phase 0~4로 구현.

### Added
- 투명·무테·항상-위 오버레이 창 (Tauri v2 + Vite, macOS, `macos-private-api`)
- 패닉 전역 단축키(기본 `⌘⇧H`): 비포커스에서도 즉시 hide/show 토글 (Rust 핸들러 직접 처리, 저지연)
- Hover-Reveal: 커서 폴링(40ms, 엣지 트리거) + 클릭 통과 토글 + Ghost↔Revealed 상태 머신 (Rust 단일 소스)
- 앱 은폐: Accessory(Dock·Cmd-Tab·메뉴바 숨김) · Spaces 전역 부유 · 온디맨드 포커스
- 화면공유 비노출(content protection) — best-effort
- 설정 UI: URL·북마크·평소/호버 투명도·창 크기·핫존(3×3)·패닉 단축키 캡처·은폐 토글
- 설정·북마크·마지막 창 위치 영속화(`prefs.json`) 및 재실행 시 복원
- 문서: `BUILD.md`(빌드/실행/배포), `PLAN.md`(단계별 계획·완료 기준)
- Rust 단위 테스트 4종(설정 serde 라운드트립·부분 로드·기본 핫존·위치 라운드트립)

### Known limitations
- content protection은 macOS 15(Sequoia)+ 화면 공유를 막지 못함(ScreenCaptureKit가 `NSWindowSharingNone` 무시, Tauri #14200). 실질 방어선은 패닉 단축키(사전/즉시 숨김).
- 일부 사이트는 iframe 임베드 차단(X-Frame-Options / CSP).

[0.1.2]: https://github.com/JakeB-5/peekaboo/releases/tag/v0.1.2
[0.1.1]: https://github.com/JakeB-5/peekaboo/releases/tag/v0.1.1
[0.1.0]: https://github.com/JakeB-5/peekaboo/releases/tag/v0.1.0
