"use client";

import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { useCallback, useEffect, useMemo, useRef, useState, useSyncExternalStore } from "react";
import type { KeyboardEvent, MouseEvent, WheelEvent } from "react";
import {
  IconAddressBook,
  IconChevronLeft,
  IconChevronRight,
  IconHome,
  IconLayoutNavbar,
  IconLayoutNavbarCollapse,
  IconLayoutNavbarExpand,
  IconLayoutSidebar,
  IconLayoutSidebarLeftCollapse,
  IconLayoutSidebarLeftExpand,
  IconMicrophone,
  IconMicrophoneOff,
  IconPinned,
  IconPinnedOff,
  IconServer2,
  IconSettings,
  IconVolume,
  IconVolumeOff,
  IconX,
} from "@tabler/icons-react";

import { readActivePersonaId, readPersonas } from "@/lib/personas";
import {
  readMicrophoneMuted,
  readNavLayout,
  readSidebarCollapsed,
  readSoundMuted,
  readTabRestoreMode,
  setMicrophoneMuted,
  setNavLayout,
  setSidebarCollapsed,
  setSoundMuted,
  subscribeWorkspacePreferences,
  type NavLayout,
  type TabRestoreMode,
} from "@/lib/workspace-preferences";
import {
  closeWorkspaceTab,
  openWorkspaceTab,
  readWorkspaceTabsSnapshot,
  reorderWorkspaceTab,
  routeToWorkspaceTab,
  subscribeWorkspaceTabs,
  syncWorkspaceTabsForRestoreMode,
  toggleWorkspaceTabPinned,
  type WorkspaceTab,
} from "@/lib/workspace-tabs";

import { BrandLogo } from "@/components/brand-logo";
import { Avatar } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { RealtimeClient } from "./realtime-client";
import styles from "./workspace-shell.module.css";

type TabItem = {
  id: string;
  label: string;
  icon?: typeof IconServer2;
  onSelect?: () => void;
};

type WorkspaceTabMeta = {
  label?: string;
  imageLabel?: string;
  unread?: number;
};

type TabScrollState = {
  hasOverflow: boolean;
  canScrollLeft: boolean;
  canScrollRight: boolean;
};

const EMPTY_TAB_SCROLL_STATE: TabScrollState = {
  hasOverflow: false,
  canScrollLeft: false,
  canScrollRight: false,
};

const EMPTY_WORKSPACE_TABS: WorkspaceTab[] = [];
const DEFAULT_PROFILE = JSON.stringify({ name: "your profile", status: "No active profile" });

type ProfileSummary = {
  name: string;
  status: string;
};

function normalizeUnread(value: number | undefined): number {
  if (!Number.isFinite(value) || !value || value <= 0) {
    return 0;
  }

  return Math.floor(value);
}

function getInitials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) {
    return "?";
  }

  return parts
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");
}

function readProfileSnapshot(): string {
  try {
    const personas = readPersonas();
    const activeId = readActivePersonaId() ?? personas[0]?.id;
    const persona = personas.find((item) => item.id === activeId) ?? personas[0];

    if (!persona) {
      return DEFAULT_PROFILE;
    }

    return JSON.stringify({ name: persona.name, status: "Ready" });
  } catch {
    return DEFAULT_PROFILE;
  }
}

function parseProfileSnapshot(value: string): ProfileSummary {
  try {
    const parsed = JSON.parse(value) as Partial<ProfileSummary>;
    return {
      name: parsed.name || "your profile",
      status: parsed.status || "No active profile",
    };
  } catch {
    return { name: "your profile", status: "No active profile" };
  }
}

export function WorkspaceShell({
  title,
  subtitle,
  tabs,
  activeTabId,
  tabActions,
  workspaceTab,
  onTabChange,
  children,
}: {
  title: string;
  subtitle: string;
  tabs: TabItem[];
  activeTabId: string;
  tabActions?: React.ReactNode;
  workspaceTab?: WorkspaceTabMeta;
  onTabChange?: (tabId: string) => void;
  children: React.ReactNode;
}) {
  const pathname = usePathname();
  const router = useRouter();
  const navLayout = useSyncExternalStore<NavLayout>(subscribeWorkspacePreferences, readNavLayout, () => "sidebar");
  const collapsed = useSyncExternalStore(subscribeWorkspacePreferences, readSidebarCollapsed, () => false);
  const soundMuted = useSyncExternalStore(subscribeWorkspacePreferences, readSoundMuted, () => false);
  const microphoneMuted = useSyncExternalStore(subscribeWorkspacePreferences, readMicrophoneMuted, () => false);
  const profileSnapshot = useSyncExternalStore(subscribeWorkspacePreferences, readProfileSnapshot, () => DEFAULT_PROFILE);
  const contentTabsRef = useRef<HTMLDivElement | null>(null);
  const workspaceTabsRef = useRef<HTMLDivElement | null>(null);
  const contentTabOverflowUpdateRef = useRef<{
    frame: number | null;
    timeout: number | null;
    settledTimeout: number | null;
  }>({ frame: null, timeout: null, settledTimeout: null });
  const workspaceTabOverflowUpdateRef = useRef<{
    frame: number | null;
    timeout: number | null;
    settledTimeout: number | null;
  }>({ frame: null, timeout: null, settledTimeout: null });
  const [contentTabScrollState, setContentTabScrollState] = useState<TabScrollState>(EMPTY_TAB_SCROLL_STATE);
  const [workspaceTabScrollState, setWorkspaceTabScrollState] = useState<TabScrollState>(EMPTY_TAB_SCROLL_STATE);
  const [draggedWorkspaceTabId, setDraggedWorkspaceTabId] = useState<string | null>(null);
  const [workspaceTabMenu, setWorkspaceTabMenu] = useState<{ tabId: string; x: number; y: number } | null>(null);
  const tabRestoreMode = useSyncExternalStore<TabRestoreMode>(
    subscribeWorkspacePreferences,
    readTabRestoreMode,
    () => "pinned",
  );
  const workspaceTabs = useSyncExternalStore(subscribeWorkspaceTabs, readWorkspaceTabsSnapshot, () => EMPTY_WORKSPACE_TABS);
  const routeTab = useMemo(() => {
    const tab = routeToWorkspaceTab(pathname);
    if (!tab) {
      return null;
    }

    return {
      ...tab,
      label: workspaceTab?.label ?? tab.label,
      imageLabel: workspaceTab?.imageLabel ?? workspaceTab?.label ?? tab.label,
      unread: normalizeUnread(workspaceTab?.unread),
    };
  }, [pathname, workspaceTab?.imageLabel, workspaceTab?.label, workspaceTab?.unread]);
  const workspaceTabMenuTab = workspaceTabMenu ? workspaceTabs.find((tab) => tab.id === workspaceTabMenu.tabId) : undefined;

  const centerContentTabNode = useCallback((node: HTMLElement): void => {
    const element = contentTabsRef.current;
    if (!element) {
      return;
    }

    const centeredLeft = node.offsetLeft + node.offsetWidth / 2 - element.clientWidth / 2;
    const maxScrollLeft = Math.max(0, element.scrollWidth - element.clientWidth);
    element.scrollLeft = Math.min(maxScrollLeft, Math.max(0, centeredLeft));
  }, []);

  const centerActiveContentTab = useCallback(() => {
    const element = contentTabsRef.current;
    if (!element) {
      return;
    }

    const activeTab = Array.from(element.querySelectorAll<HTMLElement>("[data-tab-id]")).find(
      (node) => node.dataset.tabId === activeTabId,
    );
    if (activeTab) {
      centerContentTabNode(activeTab);
    }
  }, [activeTabId, centerContentTabNode]);

  const updateActiveContentTabMask = useCallback(() => {
    const element = contentTabsRef.current;
    const tabBar = element?.parentElement;
    if (!element || !tabBar) {
      return;
    }

    const activeTab = Array.from(element.querySelectorAll<HTMLElement>("[data-tab-id]")).find(
      (node) => node.dataset.tabId === activeTabId,
    );
    if (!activeTab) {
      tabBar.style.setProperty("--active-tab-mask-width", "0px");
      return;
    }

    const tabBarRect = tabBar.getBoundingClientRect();
    const activeRect = activeTab.getBoundingClientRect();
    const left = Math.max(0, activeRect.left - tabBarRect.left);
    const right = Math.min(tabBarRect.width, activeRect.right - tabBarRect.left);
    const width = Math.max(0, right - left - 2);

    tabBar.style.setProperty("--active-tab-mask-left", `${left}px`);
    tabBar.style.setProperty("--active-tab-mask-width", `${width}px`);
  }, [activeTabId]);

  const updateContentTabOverflow = useCallback(() => {
    const element = contentTabsRef.current;
    if (!element) {
      setContentTabScrollState(EMPTY_TAB_SCROLL_STATE);
      return;
    }

    const tabBar = element.parentElement;
    const scrollButtons = tabBar
      ? Array.from(tabBar.querySelectorAll<HTMLElement>("[data-tab-scroll-button]"))
      : [];
    const gap = tabBar ? Number.parseFloat(window.getComputedStyle(tabBar).columnGap || "0") || 0 : 0;
    const buttonWidth = scrollButtons.reduce((width, button) => width + button.offsetWidth, 0);
    const noButtonWidth = element.clientWidth + buttonWidth + gap * scrollButtons.length;
    const hasOverflow = element.scrollWidth > noButtonWidth + 1;
    const maxScrollLeft = Math.max(0, element.scrollWidth - element.clientWidth);
    const scrollLeft = Math.min(maxScrollLeft, Math.max(0, element.scrollLeft));
    const nextState: TabScrollState = {
      hasOverflow,
      canScrollLeft: hasOverflow && scrollLeft > 1,
      canScrollRight: hasOverflow && scrollLeft < maxScrollLeft - 1,
    };

    setContentTabScrollState((current) =>
      current.hasOverflow === nextState.hasOverflow &&
      current.canScrollLeft === nextState.canScrollLeft &&
      current.canScrollRight === nextState.canScrollRight
        ? current
        : nextState,
    );
  }, []);

  const updateWorkspaceTabOverflow = useCallback(() => {
    const element = workspaceTabsRef.current;
    if (!element) {
      setWorkspaceTabScrollState(EMPTY_TAB_SCROLL_STATE);
      return;
    }

    const maxScrollLeft = Math.max(0, element.scrollWidth - element.clientWidth);
    const hasOverflow = maxScrollLeft > 1;
    const scrollLeft = Math.min(maxScrollLeft, Math.max(0, element.scrollLeft));
    const nextState: TabScrollState = {
      hasOverflow,
      canScrollLeft: hasOverflow && scrollLeft > 1,
      canScrollRight: hasOverflow && scrollLeft < maxScrollLeft - 1,
    };

    setWorkspaceTabScrollState((current) =>
      current.hasOverflow === nextState.hasOverflow &&
      current.canScrollLeft === nextState.canScrollLeft &&
      current.canScrollRight === nextState.canScrollRight
        ? current
        : nextState,
    );
  }, []);

  const clearScheduledContentTabOverflowUpdate = useCallback(() => {
    const scheduled = contentTabOverflowUpdateRef.current;
    if (scheduled.frame !== null) {
      window.cancelAnimationFrame(scheduled.frame);
    }
    if (scheduled.timeout !== null) {
      window.clearTimeout(scheduled.timeout);
    }
    if (scheduled.settledTimeout !== null) {
      window.clearTimeout(scheduled.settledTimeout);
    }

    scheduled.frame = null;
    scheduled.timeout = null;
    scheduled.settledTimeout = null;
  }, []);

  const clearScheduledWorkspaceTabOverflowUpdate = useCallback(() => {
    const scheduled = workspaceTabOverflowUpdateRef.current;
    if (scheduled.frame !== null) {
      window.cancelAnimationFrame(scheduled.frame);
    }
    if (scheduled.timeout !== null) {
      window.clearTimeout(scheduled.timeout);
    }
    if (scheduled.settledTimeout !== null) {
      window.clearTimeout(scheduled.settledTimeout);
    }

    scheduled.frame = null;
    scheduled.timeout = null;
    scheduled.settledTimeout = null;
  }, []);

  const scheduleContentTabOverflowUpdate = useCallback(() => {
    clearScheduledContentTabOverflowUpdate();
    contentTabOverflowUpdateRef.current.frame = window.requestAnimationFrame(updateContentTabOverflow);
    contentTabOverflowUpdateRef.current.timeout = window.setTimeout(updateContentTabOverflow, 0);
    contentTabOverflowUpdateRef.current.settledTimeout = window.setTimeout(updateContentTabOverflow, 120);
  }, [clearScheduledContentTabOverflowUpdate, updateContentTabOverflow]);

  const scheduleWorkspaceTabOverflowUpdate = useCallback(() => {
    clearScheduledWorkspaceTabOverflowUpdate();
    workspaceTabOverflowUpdateRef.current.frame = window.requestAnimationFrame(updateWorkspaceTabOverflow);
    workspaceTabOverflowUpdateRef.current.timeout = window.setTimeout(updateWorkspaceTabOverflow, 0);
    workspaceTabOverflowUpdateRef.current.settledTimeout = window.setTimeout(updateWorkspaceTabOverflow, 120);
  }, [clearScheduledWorkspaceTabOverflowUpdate, updateWorkspaceTabOverflow]);

  useEffect(() => {
    return () => {
      clearScheduledContentTabOverflowUpdate();
      clearScheduledWorkspaceTabOverflowUpdate();
    };
  }, [clearScheduledContentTabOverflowUpdate, clearScheduledWorkspaceTabOverflowUpdate]);

  const setContentTabsNode = useCallback((node: HTMLDivElement | null) => {
    contentTabsRef.current = node;
    if (!node) {
      return;
    }

    scheduleContentTabOverflowUpdate();
  }, [scheduleContentTabOverflowUpdate]);

  const setWorkspaceTabsNode = useCallback((node: HTMLDivElement | null) => {
    workspaceTabsRef.current = node;
    if (!node) {
      setWorkspaceTabScrollState(EMPTY_TAB_SCROLL_STATE);
      return;
    }

    scheduleWorkspaceTabOverflowUpdate();
  }, [scheduleWorkspaceTabOverflowUpdate]);

  const activeContentTabRef = useCallback((node: HTMLElement | null) => {
    if (!node) {
      return;
    }

    window.requestAnimationFrame(() => {
      if (!node.isConnected) {
        return;
      }

      updateActiveContentTabMask();
      scheduleContentTabOverflowUpdate();
    });
  }, [scheduleContentTabOverflowUpdate, updateActiveContentTabMask]);

  useEffect(() => {
    const element = contentTabsRef.current;
    if (!element) {
      return;
    }

    const frame = window.requestAnimationFrame(() => {
      centerActiveContentTab();
      updateActiveContentTabMask();
      scheduleContentTabOverflowUpdate();
    });

    return () => {
      window.cancelAnimationFrame(frame);
    };
  }, [activeTabId, centerActiveContentTab, scheduleContentTabOverflowUpdate, tabs.length, updateActiveContentTabMask]);

  useEffect(() => {
    const frame = window.requestAnimationFrame(updateActiveContentTabMask);
    const timeout = window.setTimeout(updateActiveContentTabMask, 0);
    const settledTimeout = window.setTimeout(updateActiveContentTabMask, 120);
    return () => {
      window.cancelAnimationFrame(frame);
      window.clearTimeout(timeout);
      window.clearTimeout(settledTimeout);
    };
  }, [
    contentTabScrollState.canScrollLeft,
    contentTabScrollState.canScrollRight,
    updateActiveContentTabMask,
  ]);

  useEffect(() => {
    const element = contentTabsRef.current;
    if (!element) {
      return;
    }

    const handleResize = (): void => {
      updateActiveContentTabMask();
      scheduleContentTabOverflowUpdate();
    };
    const handleScroll = (): void => {
      updateActiveContentTabMask();
      updateContentTabOverflow();
    };
    const frame = window.requestAnimationFrame(handleResize);
    element.addEventListener("scroll", handleScroll, { passive: true });
    window.addEventListener("resize", handleResize);

    const observer = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(handleResize);
    observer?.observe(element);

    return () => {
      window.cancelAnimationFrame(frame);
      element.removeEventListener("scroll", handleScroll);
      window.removeEventListener("resize", handleResize);
      observer?.disconnect();
    };
  }, [scheduleContentTabOverflowUpdate, tabs.length, updateActiveContentTabMask, updateContentTabOverflow]);

  useEffect(() => {
    const element = workspaceTabsRef.current;
    if (!element) {
      return;
    }

    const handleResize = (): void => {
      scheduleWorkspaceTabOverflowUpdate();
    };
    const handleScroll = (): void => {
      updateWorkspaceTabOverflow();
    };
    const frame = window.requestAnimationFrame(handleResize);
    element.addEventListener("scroll", handleScroll, { passive: true });
    window.addEventListener("resize", handleResize);

    const observer = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(handleResize);
    observer?.observe(element);

    return () => {
      window.cancelAnimationFrame(frame);
      element.removeEventListener("scroll", handleScroll);
      window.removeEventListener("resize", handleResize);
      observer?.disconnect();
    };
  }, [
    collapsed,
    navLayout,
    routeTab?.id,
    scheduleWorkspaceTabOverflowUpdate,
    updateWorkspaceTabOverflow,
    workspaceTabs.length,
  ]);

  useEffect(() => {
    const element = workspaceTabsRef.current;
    if (!element || !routeTab?.id) {
      return;
    }

    const frame = window.requestAnimationFrame(() => {
      const activeTab = element.querySelector<HTMLElement>(`[data-workspace-tab-id="${CSS.escape(routeTab.id)}"]`);
      if (!activeTab) {
        updateWorkspaceTabOverflow();
        return;
      }

      const centeredLeft = activeTab.offsetLeft + activeTab.offsetWidth / 2 - element.clientWidth / 2;
      const maxScrollLeft = Math.max(0, element.scrollWidth - element.clientWidth);
      element.scrollLeft = Math.min(maxScrollLeft, Math.max(0, centeredLeft));
      scheduleWorkspaceTabOverflowUpdate();
    });

    return () => {
      window.cancelAnimationFrame(frame);
    };
  }, [
    collapsed,
    navLayout,
    routeTab?.id,
    scheduleWorkspaceTabOverflowUpdate,
    updateWorkspaceTabOverflow,
    workspaceTabs.length,
  ]);

  useEffect(() => {
    if (routeTab) {
      openWorkspaceTab(routeTab);
    }
  }, [routeTab]);

  useEffect(() => {
    syncWorkspaceTabsForRestoreMode(tabRestoreMode);
  }, [tabRestoreMode]);

  useEffect(() => {
    if (!workspaceTabMenu) {
      return;
    }

    function closeMenu(): void {
      setWorkspaceTabMenu(null);
    }

    function handleKeyDown(event: globalThis.KeyboardEvent): void {
      if (event.key === "Escape") {
        closeMenu();
      }
    }

    document.addEventListener("click", closeMenu);
    document.addEventListener("keydown", handleKeyDown);
    window.addEventListener("resize", closeMenu);
    window.addEventListener("scroll", closeMenu, true);

    return () => {
      document.removeEventListener("click", closeMenu);
      document.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("resize", closeMenu);
      window.removeEventListener("scroll", closeMenu, true);
    };
  }, [workspaceTabMenu]);

  function toggleNavLayout(): void {
    const next = navLayout === "sidebar" ? "topbar" : "sidebar";
    setNavLayout(next);
  }

  function toggleSidebar(): void {
    setSidebarCollapsed(!collapsed);
  }

  function handleCloseWorkspaceTab(tab: WorkspaceTab): void {
    const closingActiveTab = routeTab?.id === tab.id;
    const tabsBeforeClose = readWorkspaceTabsSnapshot();
    const closedIndex = tabsBeforeClose.findIndex((item) => item.id === tab.id);
    const nextActiveTab =
      tabsBeforeClose[closedIndex + 1] ?? tabsBeforeClose[closedIndex - 1] ?? tabsBeforeClose.find((item) => item.id !== tab.id);

    closeWorkspaceTab(tab.id);

    if (closingActiveTab) {
      router.push(nextActiveTab?.href ?? "/home");
    }
  }

  function handleWorkspaceTabDrop(targetTab: WorkspaceTab): void {
    if (!draggedWorkspaceTabId) {
      return;
    }

    reorderWorkspaceTab(draggedWorkspaceTabId, targetTab.id);
    setDraggedWorkspaceTabId(null);
  }

  function openWorkspaceTabMenu(event: MouseEvent<HTMLElement>, tab: WorkspaceTab): void {
    event.preventDefault();
    setWorkspaceTabMenu({ tabId: tab.id, x: event.clientX, y: event.clientY });
  }

  function openWorkspaceTabMenuFromKeyboard(event: KeyboardEvent<HTMLElement>, tab: WorkspaceTab): void {
    if (event.key !== "ContextMenu" && !(event.shiftKey && event.key === "F10")) {
      return;
    }

    event.preventDefault();
    const rect = event.currentTarget.getBoundingClientRect();
    setWorkspaceTabMenu({
      tabId: tab.id,
      x: Math.round(rect.left + Math.min(rect.width - 24, 48)),
      y: Math.round(rect.top + rect.height - 4),
    });
  }

  function handleWorkspaceMenuPin(tab: WorkspaceTab): void {
    toggleWorkspaceTabPinned(tab.id);
    setWorkspaceTabMenu(null);
  }

  function handleWorkspaceMenuClose(tab: WorkspaceTab): void {
    handleCloseWorkspaceTab(tab);
    setWorkspaceTabMenu(null);
  }

  function scrollContentTabs(direction: -1 | 1): void {
    const element = contentTabsRef.current;
    if (!element) {
      return;
    }

    const maxScrollLeft = Math.max(0, element.scrollWidth - element.clientWidth);
    const distance = Math.max(160, Math.floor(element.clientWidth * 0.72));
    element.scrollLeft = Math.min(maxScrollLeft, Math.max(0, element.scrollLeft + direction * distance));
    updateActiveContentTabMask();
    scheduleContentTabOverflowUpdate();
  }

  function scrollWorkspaceTabs(direction: -1 | 1): void {
    const element = workspaceTabsRef.current;
    if (!element) {
      return;
    }

    const maxScrollLeft = Math.max(0, element.scrollWidth - element.clientWidth);
    const distance = Math.max(180, Math.floor(element.clientWidth * 0.72));
    element.scrollLeft = Math.min(maxScrollLeft, Math.max(0, element.scrollLeft + direction * distance));
    scheduleWorkspaceTabOverflowUpdate();
  }

  function handleWorkspaceTabWheel(event: WheelEvent<HTMLElement>): void {
    const element = workspaceTabsRef.current;
    if (!element) {
      return;
    }

    const maxScrollLeft = Math.max(0, element.scrollWidth - element.clientWidth);
    if (maxScrollLeft <= 1) {
      return;
    }

    const delta = Math.abs(event.deltaX) > Math.abs(event.deltaY) ? event.deltaX : event.deltaY;
    if (delta === 0) {
      return;
    }

    event.preventDefault();
    element.scrollLeft = Math.min(maxScrollLeft, Math.max(0, element.scrollLeft + delta));
    scheduleWorkspaceTabOverflowUpdate();
  }

  const nav = useMemo(
    () => [
      { href: "/home", label: "Home", icon: IconHome },
      { href: "/servers", label: "Servers", icon: IconServer2 },
      { href: "/contacts", label: "Contacts", icon: IconAddressBook },
      { href: "/settings", label: "Settings", icon: IconSettings },
    ],
    [],
  );

  const isTopbar = navLayout === "topbar";
  const LayoutIcon = isTopbar ? IconLayoutSidebar : IconLayoutNavbar;
  const SidebarToggleIcon = collapsed ? IconLayoutSidebarLeftExpand : IconLayoutSidebarLeftCollapse;
  const TopbarToggleIcon = collapsed ? IconLayoutNavbarExpand : IconLayoutNavbarCollapse;
  const SoundIcon = soundMuted ? IconVolumeOff : IconVolume;
  const MicrophoneIcon = microphoneMuted ? IconMicrophoneOff : IconMicrophone;
  const profile = parseProfileSnapshot(profileSnapshot);
  const hasContentTabs = tabs.length > 0;

  function isActivePath(href: string): boolean {
    return pathname === href || pathname.startsWith(`${href}/`);
  }

  const navLinks = nav.map((item) => {
    const active = isActivePath(item.href);
    const NavIcon = item.icon;
    return (
      <Link
        aria-current={active ? "page" : undefined}
        aria-label={item.label}
        className={`${styles.navLink} ${active ? styles.navLinkActive : ""}`}
        href={item.href}
        key={item.href}
      >
        <NavIcon className={styles.navIcon} aria-hidden="true" />
        <span className={styles.navLabel}>{item.label}</span>
      </Link>
    );
  });

  const layoutSwitch = (
    <Button
      aria-label={isTopbar ? "Switch to sidebar layout" : "Switch to top bar layout"}
      className={styles.iconButton}
      onClick={toggleNavLayout}
      size="icon"
      title={isTopbar ? "Use sidebar" : "Use top bar"}
    >
      <LayoutIcon className={styles.controlIcon} aria-hidden="true" />
    </Button>
  );

  const profileControls = (
    <>
      <div className={styles.profileSummary} title={profile.name}>
        <Avatar className={styles.profileAvatar} kind="user" size="sm" text={getInitials(profile.name)} />
        <div className={styles.profileDetails}>
          <p className={styles.profileName}>{profile.name}</p>
          <p className={styles.profileStatus}>{profile.status}</p>
        </div>
      </div>
      <div className={styles.profileActions}>
        <Button
          aria-label={soundMuted ? "Unmute sound" : "Mute sound"}
          className={`${styles.iconButton} ${soundMuted ? styles.iconButtonActive : ""}`}
          onClick={() => setSoundMuted(!soundMuted)}
          pressed={soundMuted}
          size="icon"
          title={soundMuted ? "Unmute sound" : "Mute sound"}
        >
          <SoundIcon className={styles.controlIcon} aria-hidden="true" />
        </Button>
        <Button
          aria-label={microphoneMuted ? "Unmute microphone" : "Mute microphone"}
          className={`${styles.iconButton} ${microphoneMuted ? styles.iconButtonActive : ""}`}
          onClick={() => setMicrophoneMuted(!microphoneMuted)}
          pressed={microphoneMuted}
          size="icon"
          title={microphoneMuted ? "Unmute microphone" : "Mute microphone"}
        >
          <MicrophoneIcon className={styles.controlIcon} aria-hidden="true" />
        </Button>
        {layoutSwitch}
      </div>
    </>
  );

  function renderWorkspaceTab(tab: WorkspaceTab): React.ReactNode {
    const active = routeTab?.id === tab.id;
    const unread = normalizeUnread(tab.unread);
    const imageLabel = tab.imageLabel ?? tab.label;
    const isServer = tab.kind === "server";

    return (
      <div
        className={`${styles.workspaceTab} ${active ? styles.workspaceTabActive : ""} ${
          tab.pinned ? styles.workspaceTabPinned : ""
        }`}
        draggable
        key={tab.id}
        onDragEnd={() => setDraggedWorkspaceTabId(null)}
        onDragOver={(event) => {
          if (draggedWorkspaceTabId) {
            event.preventDefault();
          }
        }}
        onDragStart={(event) => {
          setDraggedWorkspaceTabId(tab.id);
          event.dataTransfer.effectAllowed = "move";
          event.dataTransfer.setData("text/plain", tab.id);
        }}
        onDrop={(event) => {
          event.preventDefault();
          handleWorkspaceTabDrop(tab);
        }}
        onContextMenu={(event) => openWorkspaceTabMenu(event, tab)}
        role="listitem"
        data-workspace-tab-id={tab.id}
      >
        <Link
          aria-current={active ? "page" : undefined}
          aria-label={`${tab.kind === "dm" ? "Conversation" : "Server"}: ${tab.label}`}
          className={styles.workspaceTabLink}
          href={tab.href}
          onKeyDown={(event) => openWorkspaceTabMenuFromKeyboard(event, tab)}
        >
          <Avatar
            className={`${styles.workspaceTabImage} ${
              isServer ? styles.workspaceTabImageServer : styles.workspaceTabImageContact
            }`}
            aria-hidden="true"
            kind={isServer ? "server" : "user"}
            text={getInitials(imageLabel)}
          />
          <span className={styles.workspaceTabLabel}>{tab.label}</span>
        </Link>
        <div className={styles.workspaceTabActions}>
          {isServer && unread > 0 ? (
            <Badge className={styles.workspaceTabBadge} aria-label={`${unread} unread notifications`} tone="accent">
              {unread}
            </Badge>
          ) : null}
          {!tab.pinned ? (
            <Button
              aria-label={`Close ${tab.label}`}
              className={styles.workspaceTabAction}
              onClick={() => handleCloseWorkspaceTab(tab)}
              size="icon"
              title="Close tab"
            >
              <IconX className={styles.workspaceTabIcon} aria-hidden="true" />
            </Button>
          ) : null}
        </div>
      </div>
    );
  }

  function renderWorkspaceTabs(tabsToRender: WorkspaceTab[], emptyMessage?: string): React.ReactNode {
    if (tabsToRender.length === 0) {
      return emptyMessage ? <p className={styles.emptyTabs}>{emptyMessage}</p> : null;
    }

    return (
      <div className={styles.workspaceTabs} role="list">
        {tabsToRender.map((tab) => renderWorkspaceTab(tab))}
      </div>
    );
  }

  const pinnedWorkspaceTabs = workspaceTabs.filter((tab) => tab.pinned);
  const regularWorkspaceTabs = workspaceTabs.filter((tab) => !tab.pinned);
  const showRegularWorkspaceTabs = regularWorkspaceTabs.length > 0 || !collapsed;
  const workspaceTabContextMenu = workspaceTabMenuTab ? (
    <div
      className={styles.workspaceContextMenu}
      onClick={(event) => event.stopPropagation()}
      role="menu"
      style={{ left: workspaceTabMenu?.x, top: workspaceTabMenu?.y }}
    >
      <button
        className={styles.workspaceContextMenuItem}
        onClick={() => handleWorkspaceMenuPin(workspaceTabMenuTab)}
        role="menuitem"
        type="button"
      >
        {workspaceTabMenuTab.pinned ? (
          <IconPinnedOff className={styles.workspaceTabIcon} aria-hidden="true" />
        ) : (
          <IconPinned className={styles.workspaceTabIcon} aria-hidden="true" />
        )}
        {workspaceTabMenuTab.pinned ? "Unpin tab" : "Pin tab"}
      </button>
      <button
        className={`${styles.workspaceContextMenuItem} ${styles.workspaceContextMenuDanger}`}
        onClick={() => handleWorkspaceMenuClose(workspaceTabMenuTab)}
        role="menuitem"
        type="button"
      >
        <IconX className={styles.workspaceTabIcon} aria-hidden="true" />
        Close tab
      </button>
    </div>
  ) : null;
  const sidebarWorkspaceTabSections = (
    <>
      {pinnedWorkspaceTabs.length > 0 ? (
        <div className={`${styles.workspaceSection} ${styles.workspaceSectionPinned}`} role="group" aria-label="Pinned tabs">
          {renderWorkspaceTabs(pinnedWorkspaceTabs)}
        </div>
      ) : null}
      {showRegularWorkspaceTabs ? (
        <div className={styles.workspaceSection} role="group" aria-label="Workspace tabs">
          {renderWorkspaceTabs(regularWorkspaceTabs, "Open a server or conversation to create a tab.")}
        </div>
      ) : null}
      {workspaceTabContextMenu}
    </>
  );
  const topbarWorkspaceTabs = [...pinnedWorkspaceTabs, ...regularWorkspaceTabs];
  const brand = (
    <div className={styles.brand} aria-label="HexRelay">
      <BrandLogo className={styles.brandMark} aria-hidden="true" />
      <span className={styles.brandText}>
        <span className={styles.brandName}>HEX RELAY</span>
        <span className={styles.brandTagline}>Connect. Communicate. Relay.</span>
      </span>
    </div>
  );
  const topbarWorkspaceTabStrip = (
    <>
      <div className={styles.workspaceRail} onWheel={handleWorkspaceTabWheel} role="group" aria-label="Workspace tabs">
        {topbarWorkspaceTabs.length === 0 ? (
          <p className={styles.emptyTabs}>Open a server or conversation to create a tab.</p>
        ) : (
          <>
            {workspaceTabScrollState.canScrollLeft ? (
              <button
                aria-label="Scroll workspace tabs left"
                className={styles.workspaceScrollButton}
                onClick={() => scrollWorkspaceTabs(-1)}
                type="button"
              >
                <IconChevronLeft className={styles.workspaceScrollIcon} aria-hidden="true" />
              </button>
            ) : null}
            <div className={styles.workspaceTabs} ref={setWorkspaceTabsNode} role="list">
              {topbarWorkspaceTabs.map((tab) => renderWorkspaceTab(tab))}
            </div>
            {workspaceTabScrollState.canScrollRight ? (
              <button
                aria-label="Scroll workspace tabs right"
                className={styles.workspaceScrollButton}
                onClick={() => scrollWorkspaceTabs(1)}
                type="button"
              >
                <IconChevronRight className={styles.workspaceScrollIcon} aria-hidden="true" />
              </button>
            ) : null}
          </>
        )}
      </div>
      {workspaceTabContextMenu}
    </>
  );

  return (
    <main className={`${styles.shell} ${isTopbar ? styles.topbarMode : ""} ${collapsed ? styles.collapsed : ""}`}>
      <RealtimeClient />
      <div className={styles.frame}>
        {isTopbar ? (
          <header className={styles.topbar}>
            <div className={styles.topbarPrimary}>
              {brand}
              <nav aria-label="Primary" className={styles.topbarNav}>
                {navLinks}
              </nav>
            </div>
            <div className={styles.workspaceStack} role="group" aria-label="Workspace tabs">
              {topbarWorkspaceTabStrip}
            </div>
            <div className={styles.topbarControls}>
              {profileControls}
              <Button
                aria-label={collapsed ? "Expand top bar" : "Collapse top bar"}
                className={styles.iconButton}
                onClick={toggleSidebar}
                size="icon"
                title={collapsed ? "Expand top bar" : "Collapse top bar"}
              >
                <TopbarToggleIcon className={styles.controlIcon} aria-hidden="true" />
              </Button>
            </div>
          </header>
        ) : (
          <aside className={styles.sidebar}>
            <div className={styles.sidebarPrimary}>
              {brand}
              <nav aria-label="Primary" className={styles.topNav}>
                {navLinks}
              </nav>
            </div>

            <div className={styles.workspaceStack} role="group" aria-label="Workspace tabs">
              {sidebarWorkspaceTabSections}
            </div>
            <div className={styles.sidebarControls}>
              {profileControls}
              <Button
                aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
                className={styles.iconButton}
                onClick={toggleSidebar}
                size="icon"
                title={collapsed ? "Expand sidebar" : "Collapse sidebar"}
              >
                <SidebarToggleIcon className={styles.controlIcon} aria-hidden="true" />
              </Button>
            </div>
          </aside>
        )}

        <section
          aria-describedby="workspace-page-subtitle"
          aria-labelledby="workspace-page-title"
          className={`${styles.content} ${hasContentTabs || tabActions ? "" : styles.contentNoTabBar}`}
        >
          <header className={styles.visuallyHidden}>
            <h1 id="workspace-page-title">{title}</h1>
            <p id="workspace-page-subtitle">{subtitle}</p>
          </header>
          {hasContentTabs || tabActions ? (
            <div className={`${styles.tabBar} ${hasContentTabs ? "" : styles.actionBarOnly}`}>
              {hasContentTabs && contentTabScrollState.canScrollLeft ? (
                <button
                  aria-label="Scroll tabs left"
                  className={styles.tabScrollButton}
                  data-tab-scroll-button="left"
                  onClick={() => scrollContentTabs(-1)}
                  type="button"
                >
                  <IconChevronLeft className={styles.tabScrollIcon} aria-hidden="true" />
                </button>
              ) : null}
              {hasContentTabs ? (
                <div aria-label={`${title} sections`} className={styles.tabs} ref={setContentTabsNode}>
                  {tabs.map((tab) => {
                    const TabIcon = tab.icon;
                    const active = tab.id === activeTabId;
                    const handleTabSelect = tab.onSelect ?? (onTabChange ? () => onTabChange(tab.id) : undefined);
                    const tabClassName = `${styles.tab} ${handleTabSelect ? styles.tabButton : ""} ${
                      active ? styles.tabActive : ""
                    }`;
                    const tabContent = (
                      <>
                        {TabIcon ? <TabIcon className={styles.tabIcon} aria-hidden="true" /> : null}
                        <span className={styles.tabLabel}>{tab.label}</span>
                      </>
                    );

                    return handleTabSelect ? (
                      <button
                        aria-pressed={active}
                        className={tabClassName}
                        data-tab-id={tab.id}
                        key={tab.id}
                        onClick={handleTabSelect}
                        ref={active ? activeContentTabRef : undefined}
                        type="button"
                      >
                        {tabContent}
                      </button>
                    ) : (
                      <div
                        className={tabClassName}
                        data-tab-id={tab.id}
                        key={tab.id}
                        ref={active ? activeContentTabRef : undefined}
                      >
                        {tabContent}
                      </div>
                    );
                  })}
                </div>
              ) : null}
              {hasContentTabs && contentTabScrollState.canScrollRight ? (
                <button
                  aria-label="Scroll tabs right"
                  className={styles.tabScrollButton}
                  data-tab-scroll-button="right"
                  onClick={() => scrollContentTabs(1)}
                  type="button"
                >
                  <IconChevronRight className={styles.tabScrollIcon} aria-hidden="true" />
                </button>
              ) : null}
              {tabActions ? <div className={styles.tabActions}>{tabActions}</div> : null}
            </div>
          ) : null}

          <section className={`${styles.body} ${hasContentTabs || tabActions ? "" : styles.bodyNoTabs}`}>{children}</section>
        </section>
      </div>

      <nav aria-label="Primary" className={styles.mobileTabs}>
        {nav.map((item) => {
          const active = isActivePath(item.href);
          return (
            <Link
              aria-current={active ? "page" : undefined}
              className={`${styles.mobileTab} ${active ? styles.mobileTabActive : ""}`}
              href={item.href}
              key={item.href}
            >
              <item.icon className={styles.mobileTabIcon} aria-hidden="true" />
              <span>{item.label}</span>
            </Link>
          );
        })}
      </nav>
    </main>
  );
}
