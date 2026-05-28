export type NavLayout = "sidebar" | "topbar";
export type TabRestoreMode = "pinned" | "all";
export type MessageLayout = "bubble-cards" | "continuous-feed";
export type MessageBubbleSize = "comfortable" | "compact";
export type MessageAlignment = "conversation-sides" | "single-column";
export type HubKind = "servers" | "contacts";
export type HubLayout = "cards" | "list";

const SIDEBAR_MODE_KEY = "hexrelay.ui.sidebar-mode";
const NAV_LAYOUT_KEY = "hexrelay.ui.nav-layout";
const TAB_RESTORE_MODE_KEY = "hexrelay.ui.tab-restore-mode";
const MESSAGE_LAYOUT_KEY = "hexrelay.ui.message-layout";
const MESSAGE_BUBBLE_SIZE_KEY = "hexrelay.ui.message-bubble-size";
const MESSAGE_ALIGNMENT_KEY = "hexrelay.ui.message-alignment";
const SERVERS_HUB_LAYOUT_KEY = "hexrelay.ui.servers-hub-layout";
const CONTACTS_HUB_LAYOUT_KEY = "hexrelay.ui.contacts-hub-layout";
const SOUND_MUTED_KEY = "hexrelay.ui.sound-muted";
const MICROPHONE_MUTED_KEY = "hexrelay.ui.microphone-muted";
const PERSONAS_KEY = "hexrelay.personas";
const ACTIVE_PERSONA_KEY = "hexrelay.active-persona";
const UI_PREFS_EVENT = "hexrelay-ui-preferences-changed";

let fallbackNavLayout: NavLayout = "sidebar";
let fallbackSidebarCollapsed = false;
let fallbackTabRestoreMode: TabRestoreMode = "pinned";
let fallbackMessageLayout: MessageLayout = "bubble-cards";
let fallbackMessageBubbleSize: MessageBubbleSize = "comfortable";
let fallbackMessageAlignment: MessageAlignment = "conversation-sides";
const fallbackHubLayouts = new Map<HubKind, HubLayout>();
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
        MESSAGE_BUBBLE_SIZE_KEY,
        MESSAGE_ALIGNMENT_KEY,
        SERVERS_HUB_LAYOUT_KEY,
        CONTACTS_HUB_LAYOUT_KEY,
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

function hubLayoutKey(kind: HubKind): string {
  return kind === "servers" ? SERVERS_HUB_LAYOUT_KEY : CONTACTS_HUB_LAYOUT_KEY;
}

function defaultHubLayout(): HubLayout {
  if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
    return "cards";
  }

  return window.matchMedia("(max-width: 700px)").matches ? "list" : "cards";
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

export function readMessageBubbleSize(): MessageBubbleSize {
  const value = readStorageItem(MESSAGE_BUBBLE_SIZE_KEY);
  if (value === undefined) {
    return fallbackMessageBubbleSize;
  }

  return value === "compact" ? "compact" : "comfortable";
}

export function setMessageBubbleSize(value: MessageBubbleSize): void {
  fallbackMessageBubbleSize = value;
  writeStorageItem(MESSAGE_BUBBLE_SIZE_KEY, value);
  notifyWorkspacePreferenceChange();
}

export function readMessageAlignment(): MessageAlignment {
  const value = readStorageItem(MESSAGE_ALIGNMENT_KEY);
  if (value === undefined) {
    return fallbackMessageAlignment;
  }

  return value === "single-column" ? "single-column" : "conversation-sides";
}

export function setMessageAlignment(value: MessageAlignment): void {
  fallbackMessageAlignment = value;
  writeStorageItem(MESSAGE_ALIGNMENT_KEY, value);
  notifyWorkspacePreferenceChange();
}

export function readHubLayout(kind: HubKind): HubLayout {
  const value = readStorageItem(hubLayoutKey(kind));
  if (value === undefined) {
    return fallbackHubLayouts.get(kind) ?? defaultHubLayout();
  }

  return value === "list" || value === "cards" ? value : defaultHubLayout();
}

export function setHubLayout(kind: HubKind, value: HubLayout): void {
  fallbackHubLayouts.set(kind, value);
  writeStorageItem(hubLayoutKey(kind), value);
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
