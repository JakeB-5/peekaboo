# Peekaboo — 빌드 · 실행 · 배포

> 본 문서는 개발/빌드/개인용 실행 경로를 정리한다. 기획·설계 근거는 `docs/`, 단계별 완료 기준은 `PLAN.md` 참조.

## 사전 준비

- **Rust 툴체인** — `rustup`(stable). 미설치 시 `https://rustup.rs`.
- **Node.js** — LTS 이상 (`npm` 동봉).
- **Xcode Command Line Tools** — `xcode-select --install`.

## 의존성 설치

```bash
npm install
```

## 개발 실행

```bash
npm run tauri dev
```

Vite 개발 서버(`localhost:1420`)가 뜨고 Tauri가 투명·무테 오버레이 창을 띄운다. 기본 상태는 **고스트**(저투명·클릭통과). 마우스를 오버레이 위로 올리면 또렷해지고(Revealed), 패닉 단축키(기본 `⌘⇧H`)로 즉시 숨김/복귀한다.

## 프로덕션 빌드

```bash
npm run tauri build
```

산출물: `src-tauri/target/release/bundle/` 아래 `.app` / `.dmg`.

## 개인용(미서명) 실행

Peekaboo는 본질적으로 개인 도구이므로 미서명/ad-hoc 서명으로 충분하다. Gatekeeper 격리 경고는 다음으로 우회한다.

```bash
xattr -dr com.apple.quarantine "Peekaboo.app"
```

또는 Finder에서 우클릭 → 열기.

> **주의** `transparent` 창은 `macos-private-api` feature를 쓰므로 **App Store 배포 불가**(개인 배포는 무관). Developer ID 서명·공증이 필요하면 Tauri 공식 문서의 최신 환경변수/필드를 재확인한다(버전에 따라 변동 — 확인필요).

## 검증 명령

```bash
# 프론트엔드
npx tsc --noEmit
npx eslint .
npm run build            # tsc --noEmit + vite build

# Rust 코어
cargo fmt   --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets
cargo test  --manifest-path src-tauri/Cargo.toml
cargo build --manifest-path src-tauri/Cargo.toml
```

## 권한 · 한계 메모

- **전역(패닉) 단축키** — 표준 modifier+key 조합은 macOS `RegisterEventHotKey`로 중개되어 **손쉬운 사용(Accessibility) 권한 없이** 비포커스에서도 동작한다. 미디어키는 금지(CGEventTap → 권한 유발).
- **화면 공유 비노출** — `contentProtected`는 **macOS 15(Sequoia)+에서 무효**(ScreenCaptureKit가 `NSWindowSharingNone` 무시, Tauri #14200). best-effort 보조 수단이며, 실질 방어선은 **패닉 단축키로 사전/즉시 숨기기**다.
- **앱 은폐** — `ActivationPolicy::Accessory`로 Dock·Cmd-Tab·메뉴바에서 숨긴다. Accessory 앱은 `show()`만으론 포커스가 안 오므로, 입력이 필요할 때 드래그 스트립 클릭으로 포커스를 잡는다.

## 영속화(흔적) 메모

설정·북마크·마지막 위치는 앱 config 디렉터리의 `prefs.json`에 저장된다(macOS: `~/Library/Application Support/<bundle id>/prefs.json`). 흔적 최소화 관점에서 번들 식별자/파일명은 추후 더 평범하게 바꿀 수 있다.
