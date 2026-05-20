export type NavLayout = "sidebar" | "topbar";
export type TabRestoreMode = "pinned" | "all";
export type MessageLayout = "bubble-cards" | "continuous-feed";

const SIDEBAR_MODE_KEY = "hexrelay.ui.sidebar-mode";
const NAV_LAYOUT_KEY = "hexrelay.ui.nav-layout";
const TAB_RESTORE_MODE_KEY = "hexrelay.ui.tab-restore-mode";
const MESSAGE_LAYOUT_KEY = "hexrelay.ui.message-layout";
const SOUND_MUTED_KEY = "hexrelay.ui.sound-muted";
const MICROPHONE_MUTED_KEY = "hexrelay.ui.microphone-muted";
const PERSONAS_KEY = "hexrelay.personas";
const ACTIVE_PERSONA_KEY = "hexrelay.active-persona";
const UI_PREFS_EVENT = "hexrelay-ui-preferences-changed";

let fallbackNavLayout: NavLayout = "sidebar";
let fallbackSidebarCollapsed = false;
let fallbackTabRestoreMode: TabRestoreMode = "pinned";
let fallbackMessageLayout: MessageLayout = "bubble-cards";
let fallbackSoundMuted = false;
let fallbackMicrophoneMuted = false;

function readStorageItem(key: string): string | null | undefined {
  if (typeof window === "undefined") {
    return null;
  }

  try {
    return window.localStorage.getItem(key);
  } catch {
    return undefined;
  }
}

function writeStorageItem(key: string, value: string): void {
  if (typeof window === "undefined") {
    return;
  }

  try {
    window.localStorage.setItem(key, value);
  } catch {
    // In-memory fallbacks still update the current page when storage is blocked.
  }
}

export function notifyWorkspacePreferenceChange(): void {
  if (typeof window === "undefined") {
    return;
  }

  window.dispatchEvent(new Event(UI_PREFS_EVENT));
}

export function subscribeWorkspacePreferences(onChange: () => void): () => void {
  if (typeof window === "undefined") {
    return () => {};
  }

  function handleStorage(event: StorageEvent): void {
    if (
      [
        SIDEBAR_MODE_KEY,
        NAV_LAYOUT_KEY,
        TAB_RESTORE_MODE_KEY,
        MESSAGE_LAYOUT_KEY,
        SOUND_MUTED_KEY,
        MICROPHONE_MUTED_KEY,
        PERSONAS_KEY,
        ACTIVE_PERSONA_KEY,
      ].includes(event.key ?? "")
    ) {
      onChange();
    }
  }

  window.addEventListener("storage", handleStorage);
  window.addEventListener(UI_PREFS_EVENT, onChange);

  return () => {
    window.removeEventListener("storage", handleStorage);
    window.removeEventListener(UI_PREFS_EVENT, onChange);
  };
}

export function readNavLayout(): NavLayout {
  const value = readStorageItem(NAV_LAYOUT_KEY);
  if (value === undefined) {
    return fallbackNavLayout;
  }

  return value === "topbar" ? "topbar" : "sidebar";
}

export function setNavLayout(value: NavLayout): void {
  fallbackNavLayout = value;
  writeStorageItem(NAV_LAYOUT_KEY, value);
  notifyWorkspacePreferenceChange();
}

export function readSidebarCollapsed(): boolean {
  const value = readStorageItem(SIDEBAR_MODE_KEY);
  return value === undefined ? fallbackSidebarCollapsed : value === "collapsed";
}

export function setSidebarCollapsed(value: boolean): void {
  fallbackSidebarCollapsed = value;
  writeStorageItem(SIDEBAR_MODE_KEY, value ? "collapsed" : "expanded");
  notifyWorkspacePreferenceChange();
}

export function readTabRestoreMode(): TabRestoreMode {
  const value = readStorageItem(TAB_RESTORE_MODE_KEY);
  if (value === undefined) {
    return fallbackTabRestoreMode;
  }

  return value === "all" ? "all" : "pinned";
}

export function setTabRestoreMode(value: TabRestoreMode): void {
  fallbackTabRestoreMode = value;
  writeStorageItem(TAB_RESTORE_MODE_KEY, value);
  notifyWorkspacePreferenceChange();
}

export function readMessageLayout(): MessageLayout {
  const value = readStorageItem(MESSAGE_LAYOUT_KEY);
  if (value === undefined) {
    return fallbackMessageLayout;
  }

  return value === "continuous-feed" ? "continuous-feed" : "bubble-cards";
}

export function setMessageLayout(value: MessageLayout): void {
  fallbackMessageLayout = value;
  writeStorageItem(MESSAGE_LAYOUT_KEY, value);
  notifyWorkspacePreferenceChange();
}

export function readSoundMuted(): boolean {
  const value = readStorageItem(SOUND_MUTED_KEY);
  return value === undefined ? fallbackSoundMuted : value === "true";
}

export function setSoundMuted(value: boolean): void {
  fallbackSoundMuted = value;
  writeStorageItem(SOUND_MUTED_KEY, value ? "true" : "false");
  notifyWorkspacePreferenceChange();
}

export function readMicrophoneMuted(): boolean {
  const value = readStorageItem(MICROPHONE_MUTED_KEY);
  return value === undefined ? fallbackMicrophoneMuted : value === "true";
}

export function setMicrophoneMuted(value: boolean): void {
  fallbackMicrophoneMuted = value;
  writeStorageItem(MICROPHONE_MUTED_KEY, value ? "true" : "false");
  notifyWorkspacePreferenceChange();
}
