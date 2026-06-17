# Changelog

본 프로젝트의 주요 변경 사항을 기록한다. 형식은 [Keep a Changelog](https://keepachangelog.com/ko/1.1.0/),
버전 체계는 [Semantic Versioning](https://semver.org/lang/ko/)을 따른다.

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

[0.1.0]: https://github.com/JakeB-5/peekaboo/releases/tag/v0.1.0
