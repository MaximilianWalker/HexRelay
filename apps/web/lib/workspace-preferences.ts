export type NavLayout = "sidebar" | "topbar";
export type TabRestoreMode = "pinned" | "all";

const SIDEBAR_MODE_KEY = "hexrelay.ui.sidebar-mode.v1";
const NAV_LAYOUT_KEY = "hexrelay.ui.nav-layout.v1";
const TAB_RESTORE_MODE_KEY = "hexrelay.ui.tab-restore-mode.v1";
const UI_PREFS_EVENT = "hexrelay-ui-preferences-changed";

let fallbackNavLayout: NavLayout = "sidebar";
let fallbackSidebarCollapsed = false;
let fallbackTabRestoreMode: TabRestoreMode = "pinned";

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
    if ([SIDEBAR_MODE_KEY, NAV_LAYOUT_KEY, TAB_RESTORE_MODE_KEY].includes(event.key ?? "")) {
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
