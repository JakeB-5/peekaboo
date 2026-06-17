// Lightweight i18n for the settings/chrome UI. No dependencies.
//
// The UI language is auto-detected from the system locale (navigator.language in
// the WebView) and resolved to the supported set declared in the READMEs
// (en/ko/ja/zh). Static DOM is translated via data-i18n* attributes; dynamic
// nodes (bookmark chips, shortcut hints) call t() at creation time.
//
// Manual language selection (persisted via the Rust core) is intentionally left
// for a later step — this phase ships automatic detection only.

export const SUPPORTED_LOCALES = ["en", "ko", "ja", "zh"] as const;
export type Locale = (typeof SUPPORTED_LOCALES)[number];

// Neutral fallback when the system locale isn't in the supported set; matches
// README.md being the canonical English entry point.
const DEFAULT_LOCALE: Locale = "en";

// Stored language preference: a supported locale, or "auto" to follow the system.
export const LOCALE_PREFS = ["auto", ...SUPPORTED_LOCALES] as const;
export type LocalePref = (typeof LOCALE_PREFS)[number];

export function isLocalePref(value: string): value is LocalePref {
  return (LOCALE_PREFS as readonly string[]).includes(value);
}

export type MessageKey =
  | "settings.open"
  | "bookmark.add"
  | "panic.hide"
  | "settings.title"
  | "close"
  | "section.content"
  | "field.bookmarks"
  | "section.display"
  | "field.ghostOpacity"
  | "field.hoverOpacity"
  | "field.windowSize"
  | "field.hotzone"
  | "hotzone.aria"
  | "hotzone.full"
  | "section.escape"
  | "field.panicShortcut"
  | "shortcut.placeholder"
  | "field.alwaysOnTop"
  | "field.allSpaces"
  | "field.screenShareHide"
  | "field.dockHide"
  | "field.language"
  | "locale.auto"
  | "bookmark.remove"
  | "bookmark.addChip"
  | "shortcut.needModifier"
  | "shortcut.cannotRegister";

// Record<MessageKey, string> makes every locale prove it defines every key.
type Catalog = Record<MessageKey, string>;

const en: Catalog = {
  "settings.open": "Settings",
  "bookmark.add": "Bookmark this page",
  "panic.hide": "Hide (panic)",
  "settings.title": "Peekaboo · Settings",
  close: "Close",
  "section.content": "Content",
  "field.bookmarks": "Bookmarks",
  "section.display": "Display",
  "field.ghostOpacity": "Idle opacity",
  "field.hoverOpacity": "Hover opacity",
  "field.windowSize": "Window size",
  "field.hotzone": "Hot-zone",
  "hotzone.aria": "Hot-zone position",
  "hotzone.full": "Full",
  "section.escape": "Escape · Concealment",
  "field.panicShortcut": "Panic hotkey",
  "shortcut.placeholder": "Click, then press a key combo",
  "field.alwaysOnTop": "Always on top",
  "field.allSpaces": "All Spaces",
  "field.screenShareHide": "Screen-share hiding",
  "field.dockHide": "Hide from Dock",
  "field.language": "Language",
  "locale.auto": "Auto (System)",
  "bookmark.remove": "Remove",
  "bookmark.addChip": "+ Add",
  "shortcut.needModifier": "Need a modifier + key",
  "shortcut.cannotRegister": "Couldn't register the hotkey",
};

const ko: Catalog = {
  "settings.open": "설정",
  "bookmark.add": "현재 페이지 북마크",
  "panic.hide": "숨기기 (패닉)",
  "settings.title": "Peekaboo · 설정",
  close: "닫기",
  "section.content": "콘텐츠",
  "field.bookmarks": "북마크",
  "section.display": "표시",
  "field.ghostOpacity": "평소 투명도",
  "field.hoverOpacity": "호버 투명도",
  "field.windowSize": "창 크기",
  "field.hotzone": "핫존",
  "hotzone.aria": "핫존 위치",
  "hotzone.full": "전체",
  "section.escape": "탈출 · 은폐",
  "field.panicShortcut": "패닉 단축키",
  "shortcut.placeholder": "클릭 후 키 조합 입력",
  "field.alwaysOnTop": "항상 위",
  "field.allSpaces": "모든 Spaces",
  "field.screenShareHide": "화면공유 비노출",
  "field.dockHide": "Dock 숨김",
  "field.language": "언어",
  "locale.auto": "자동 (시스템)",
  "bookmark.remove": "삭제",
  "bookmark.addChip": "+ 추가",
  "shortcut.needModifier": "수정자 + 키 조합이 필요합니다",
  "shortcut.cannotRegister": "단축키를 등록할 수 없습니다",
};

const ja: Catalog = {
  "settings.open": "設定",
  "bookmark.add": "現在のページをブックマーク",
  "panic.hide": "隠す（パニック）",
  "settings.title": "Peekaboo · 設定",
  close: "閉じる",
  "section.content": "コンテンツ",
  "field.bookmarks": "ブックマーク",
  "section.display": "表示",
  "field.ghostOpacity": "通常の不透明度",
  "field.hoverOpacity": "ホバー時の不透明度",
  "field.windowSize": "ウィンドウサイズ",
  "field.hotzone": "ホットゾーン",
  "hotzone.aria": "ホットゾーンの位置",
  "hotzone.full": "全体",
  "section.escape": "脱出 · 隠蔽",
  "field.panicShortcut": "パニックホットキー",
  "shortcut.placeholder": "クリックしてキーの組み合わせを入力",
  "field.alwaysOnTop": "常に最前面",
  "field.allSpaces": "すべての Spaces",
  "field.screenShareHide": "画面共有での非表示",
  "field.dockHide": "Dock から隠す",
  "field.language": "言語",
  "locale.auto": "自動（システム）",
  "bookmark.remove": "削除",
  "bookmark.addChip": "+ 追加",
  "shortcut.needModifier": "修飾キー + キーの組み合わせが必要です",
  "shortcut.cannotRegister": "ホットキーを登録できません",
};

const zh: Catalog = {
  "settings.open": "设置",
  "bookmark.add": "收藏当前页面",
  "panic.hide": "隐藏（紧急）",
  "settings.title": "Peekaboo · 设置",
  close: "关闭",
  "section.content": "内容",
  "field.bookmarks": "书签",
  "section.display": "显示",
  "field.ghostOpacity": "平时不透明度",
  "field.hoverOpacity": "悬停不透明度",
  "field.windowSize": "窗口大小",
  "field.hotzone": "热区",
  "hotzone.aria": "热区位置",
  "hotzone.full": "全部",
  "section.escape": "撤离 · 隐匿",
  "field.panicShortcut": "紧急快捷键",
  "shortcut.placeholder": "点击后按下组合键",
  "field.alwaysOnTop": "始终置顶",
  "field.allSpaces": "所有 Spaces",
  "field.screenShareHide": "屏幕共享隐藏",
  "field.dockHide": "从程序坞隐藏",
  "field.language": "语言",
  "locale.auto": "自动（系统）",
  "bookmark.remove": "删除",
  "bookmark.addChip": "+ 添加",
  "shortcut.needModifier": "需要修饰键 + 按键组合",
  "shortcut.cannotRegister": "无法注册快捷键",
};

const CATALOGS: Record<Locale, Catalog> = { en, ko, ja, zh };

let activeLocale: Locale = DEFAULT_LOCALE;

// Ordered locale preferences reported by the WebView/system.
function systemLocales(): string[] {
  const langs = navigator.languages;
  if (langs && langs.length > 0) {
    return [...langs];
  }
  return navigator.language ? [navigator.language] : [];
}

// Match one BCP 47 tag (e.g. "ko-KR", "zh-Hans") to a supported locale by its
// primary subtag.
function matchSupported(tag: string): Locale | undefined {
  const base = tag.toLowerCase().split("-")[0];
  return SUPPORTED_LOCALES.find((l) => l === base);
}

// Resolve a language preference to a concrete locale. An explicit, supported
// preference wins; "auto"/unknown/unset falls through to system detection, then
// to DEFAULT_LOCALE.
export function resolveLocale(preferred?: string): Locale {
  if (preferred && preferred !== "auto") {
    const forced = matchSupported(preferred);
    if (forced) {
      return forced;
    }
  }
  for (const tag of systemLocales()) {
    const match = matchSupported(tag);
    if (match) {
      return match;
    }
  }
  return DEFAULT_LOCALE;
}

export function getLocale(): Locale {
  return activeLocale;
}

export function setLocale(locale: Locale): void {
  activeLocale = locale;
  document.documentElement.lang = locale;
}

// Resolve + apply the active locale; returns the chosen locale.
export function initLocale(preferred?: string): Locale {
  setLocale(resolveLocale(preferred));
  return activeLocale;
}

export function t(key: MessageKey): string {
  return CATALOGS[activeLocale][key];
}

function translateAttr(
  selector: string,
  read: (el: HTMLElement) => string | undefined,
  write: (el: HTMLElement, value: string) => void,
  root: ParentNode,
): void {
  root.querySelectorAll<HTMLElement>(selector).forEach((el) => {
    const key = read(el);
    if (key) {
      write(el, t(key as MessageKey));
    }
  });
}

// Translate static DOM in `root`: textContent for [data-i18n], and the title /
// placeholder / aria-label attributes for their data-i18n-* variants.
export function applyTranslations(root: ParentNode = document): void {
  translateAttr(
    "[data-i18n]",
    (el) => el.dataset.i18n,
    (el, v) => {
      el.textContent = v;
    },
    root,
  );
  translateAttr(
    "[data-i18n-title]",
    (el) => el.dataset.i18nTitle,
    (el, v) => {
      el.title = v;
    },
    root,
  );
  translateAttr(
    "[data-i18n-placeholder]",
    (el) => el.dataset.i18nPlaceholder,
    (el, v) => {
      (el as HTMLInputElement).placeholder = v;
    },
    root,
  );
  translateAttr(
    "[data-i18n-aria-label]",
    (el) => el.dataset.i18nAriaLabel,
    (el, v) => {
      el.setAttribute("aria-label", v);
    },
    root,
  );
}
