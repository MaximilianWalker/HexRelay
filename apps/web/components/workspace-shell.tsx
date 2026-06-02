"use client";

import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { useCallback, useEffect, useMemo, useRef, useState, useSyncExternalStore } from "react";
import type { DragEvent, KeyboardEvent, MouseEvent, WheelEvent } from "react";
import { IconAddressBook, IconHome, IconServer2, IconSettings } from "@tabler/icons-react";

import { readActivePersonaId, readPersonas } from "@/lib/personas";
import { isPrimaryNavRoute } from "@/lib/navigation-routes";
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

import { BrandLockup } from "@/components/brand-lockup";
import { ContentTabBar, type ContentTabItem } from "@/components/content-tab-bar";
import { WorkspaceContextMenu } from "@/components/workspace-context-menu";
import { WorkspaceProfileControls } from "@/components/workspace-profile-controls";
import { WorkspaceTabs } from "@/components/workspace-tabs";
import { RealtimeClient } from "./realtime-client";
import styles from "./workspace-shell.module.css";

type TabItem = ContentTabItem;

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
const DEFAULT_PROFILE = JSON.stringify({ active: false, name: "your profile", status: "No active profile" });

type ProfileSummary = {
  active: boolean;
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

    return JSON.stringify({ active: true, name: persona.name, status: "Ready" });
  } catch {
    return DEFAULT_PROFILE;
  }
}

function parseProfileSnapshot(value: string): ProfileSummary {
  try {
    const parsed = JSON.parse(value) as Partial<ProfileSummary>;
    return {
      active: parsed.active === true,
      name: parsed.name || "your profile",
      status: parsed.status || "No active profile",
    };
  } catch {
    return { active: false, name: "your profile", status: "No active profile" };
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

      scheduleContentTabOverflowUpdate();
    });
  }, [scheduleContentTabOverflowUpdate]);

  useEffect(() => {
    const element = contentTabsRef.current;
    if (!element) {
      return;
    }

    const frame = window.requestAnimationFrame(() => {
      centerActiveContentTab();
      scheduleContentTabOverflowUpdate();
    });

    return () => {
      window.cancelAnimationFrame(frame);
    };
  }, [activeTabId, centerActiveContentTab, scheduleContentTabOverflowUpdate, tabs.length]);

  useEffect(() => {
    const element = contentTabsRef.current;
    if (!element) {
      return;
    }

    const handleResize = (): void => {
      scheduleContentTabOverflowUpdate();
    };
    const handleScroll = (): void => {
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
  }, [scheduleContentTabOverflowUpdate, tabs.length, updateContentTabOverflow]);

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

  function handleWorkspaceTabDragStart(tab: WorkspaceTab, event: DragEvent<HTMLElement>): void {
    setDraggedWorkspaceTabId(tab.id);
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData("text/plain", tab.id);
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
  const profile = parseProfileSnapshot(profileSnapshot);
  const hasContentTabs = tabs.length > 0;
  const voiceActionsAvailable = routeTab?.kind === "server" && activeTabId === "voice";

  const navLinks = nav.map((item) => {
    const active = isPrimaryNavRoute(pathname, item.href);
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

  const profileControls = (
    <WorkspaceProfileControls
      collapsed={collapsed}
      microphoneMuted={microphoneMuted}
      navLayout={navLayout}
      onOpenAudioDevices={() => router.push("/settings#voice-video")}
      onSetCollapsed={setSidebarCollapsed}
      onSetMicrophoneMuted={setMicrophoneMuted}
      onSetNavLayout={setNavLayout}
      onSetSoundMuted={setSoundMuted}
      placement={isTopbar ? "topbar" : "sidebar"}
      profile={{
        active: profile.active,
        initials: getInitials(profile.name),
        name: profile.name,
        status: profile.status,
      }}
      soundMuted={soundMuted}
      voiceActionsAvailable={voiceActionsAvailable}
    />
  );

  const pinnedWorkspaceTabs = workspaceTabs.filter((tab) => tab.pinned);
  const regularWorkspaceTabs = workspaceTabs.filter((tab) => !tab.pinned);
  const workspaceTabContextMenu = workspaceTabMenuTab ? (
    <WorkspaceContextMenu
      onCloseTab={handleWorkspaceMenuClose}
      onTogglePinned={handleWorkspaceMenuPin}
      position={{ x: workspaceTabMenu?.x ?? 0, y: workspaceTabMenu?.y ?? 0 }}
      tab={workspaceTabMenuTab}
    />
  ) : null;
  const sidebarWorkspaceTabSections = (
    <>
      <WorkspaceTabs
        activeTabId={routeTab?.id}
        collapsed={collapsed}
        draggedTabId={draggedWorkspaceTabId}
        emptyMessage="Open a server or conversation to create a tab."
        onCloseTab={handleCloseWorkspaceTab}
        onContextMenu={openWorkspaceTabMenu}
        onDragEnd={() => setDraggedWorkspaceTabId(null)}
        onDragStart={handleWorkspaceTabDragStart}
        onDrop={handleWorkspaceTabDrop}
        onKeyboardContextMenu={openWorkspaceTabMenuFromKeyboard}
        pinnedTabs={pinnedWorkspaceTabs}
        regularTabs={regularWorkspaceTabs}
        variant="sidebar"
      />
      {workspaceTabContextMenu}
    </>
  );
  const brand = <BrandLockup className={styles.brandLockup} collapsed={collapsed} size="lg" />;
  const topbarWorkspaceTabStrip = (
    <>
      <WorkspaceTabs
        activeTabId={routeTab?.id}
        collapsed={collapsed}
        draggedTabId={draggedWorkspaceTabId}
        emptyMessage="Open a server or conversation to create a tab."
        onCloseTab={handleCloseWorkspaceTab}
        onContextMenu={openWorkspaceTabMenu}
        onDragEnd={() => setDraggedWorkspaceTabId(null)}
        onDragStart={handleWorkspaceTabDragStart}
        onDrop={handleWorkspaceTabDrop}
        onKeyboardContextMenu={openWorkspaceTabMenuFromKeyboard}
        onScrollTabs={scrollWorkspaceTabs}
        onWheel={handleWorkspaceTabWheel}
        pinnedTabs={pinnedWorkspaceTabs}
        regularTabs={regularWorkspaceTabs}
        scrollState={workspaceTabScrollState}
        tabListRef={setWorkspaceTabsNode}
        variant="topbar"
      />
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
          <ContentTabBar
            activeTabId={activeTabId}
            activeTabRef={activeContentTabRef}
            canScrollLeft={contentTabScrollState.canScrollLeft}
            canScrollRight={contentTabScrollState.canScrollRight}
            label={`${title} sections`}
            onScrollLeft={() => scrollContentTabs(-1)}
            onScrollRight={() => scrollContentTabs(1)}
            onTabChange={onTabChange}
            tabActions={tabActions}
            tabListRef={setContentTabsNode}
            tabs={tabs}
          />

          <section className={`${styles.body} ${hasContentTabs || tabActions ? "" : styles.bodyNoTabs}`}>{children}</section>
        </section>
      </div>

      <nav aria-label="Primary" className={styles.mobileTabs}>
        {nav.map((item) => {
          const active = isPrimaryNavRoute(pathname, item.href);
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
