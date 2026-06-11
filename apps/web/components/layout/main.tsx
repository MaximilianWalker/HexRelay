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
  closeWorkspaceTab as closeOpenTab,
  openWorkspaceTab as addOpenTab,
  readWorkspaceTabsSnapshot as readOpenTabsSnapshot,
  reorderWorkspaceTab as reorderOpenTab,
  routeToWorkspaceTab as routeToOpenTab,
  subscribeWorkspaceTabs as subscribeOpenTabs,
  syncWorkspaceTabsForRestoreMode as syncOpenTabsForRestoreMode,
  toggleWorkspaceTabPinned as toggleOpenTabPinned,
  type WorkspaceTab as OpenTab,
} from "@/lib/workspace-tabs";

import { BrandLockup } from "@/components/brand-lockup";
import { Bar as ContentTabs, type Item as ContentTab } from "@/components/content-tabs/bar";
import { Menu, type Item as MenuEntry } from "@/components/ui/navigation/menu";
import { Controls } from "@/components/profile/controls";
import { TabMenu } from "./tab-menu";
import { Root as Tabs } from "./tabs/root";
import { RealtimeClient } from "../realtime-client";
import styles from "./main.module.css";

type TabItem = ContentTab;

type TabMeta = {
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

const EMPTY_OPEN_TABS: OpenTab[] = [];
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

export function MainLayout({
  title,
  subtitle,
  tabs,
  activeTabId,
  tabActions,
  openTab,
  onTabChange,
  children,
}: {
  title: string;
  subtitle: string;
  tabs: TabItem[];
  activeTabId: string;
  tabActions?: React.ReactNode;
  openTab?: TabMeta;
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
  const openTabsRef = useRef<HTMLDivElement | null>(null);
  const contentTabOverflowUpdateRef = useRef<{
    frame: number | null;
    timeout: number | null;
    settledTimeout: number | null;
  }>({ frame: null, timeout: null, settledTimeout: null });
  const openTabOverflowUpdateRef = useRef<{
    frame: number | null;
    timeout: number | null;
    settledTimeout: number | null;
  }>({ frame: null, timeout: null, settledTimeout: null });
  const [contentTabScrollState, setContentTabScrollState] = useState<TabScrollState>(EMPTY_TAB_SCROLL_STATE);
  const [openTabScrollState, setOpenTabScrollState] = useState<TabScrollState>(EMPTY_TAB_SCROLL_STATE);
  const [draggedOpenTabId, setDraggedOpenTabId] = useState<string | null>(null);
  const [openTabMenu, setOpenTabMenu] = useState<{ tabId: string; x: number; y: number } | null>(null);
  const tabRestoreMode = useSyncExternalStore<TabRestoreMode>(
    subscribeWorkspacePreferences,
    readTabRestoreMode,
    () => "pinned",
  );
  const openTabs = useSyncExternalStore(subscribeOpenTabs, readOpenTabsSnapshot, () => EMPTY_OPEN_TABS);
  const routeTab = useMemo(() => {
    const tab = routeToOpenTab(pathname);
    if (!tab) {
      return null;
    }

    return {
      ...tab,
      label: openTab?.label ?? tab.label,
      imageLabel: openTab?.imageLabel ?? openTab?.label ?? tab.label,
      unread: normalizeUnread(openTab?.unread),
    };
  }, [pathname, openTab?.imageLabel, openTab?.label, openTab?.unread]);
  const openTabMenuTab = openTabMenu ? openTabs.find((tab) => tab.id === openTabMenu.tabId) : undefined;

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

    const activeTab = Array.from(element.querySelectorAll<HTMLElement>("[data-content-tab-id]")).find(
      (node) => node.dataset.contentTabId === activeTabId,
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
      ? Array.from(tabBar.querySelectorAll<HTMLElement>("[data-content-tab-scroll-button]"))
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

  const updateOpenTabOverflow = useCallback(() => {
    const element = openTabsRef.current;
    if (!element) {
      setOpenTabScrollState(EMPTY_TAB_SCROLL_STATE);
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

    setOpenTabScrollState((current) =>
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

  const clearScheduledOpenTabOverflowUpdate = useCallback(() => {
    const scheduled = openTabOverflowUpdateRef.current;
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

  const scheduleOpenTabOverflowUpdate = useCallback(() => {
    clearScheduledOpenTabOverflowUpdate();
    openTabOverflowUpdateRef.current.frame = window.requestAnimationFrame(updateOpenTabOverflow);
    openTabOverflowUpdateRef.current.timeout = window.setTimeout(updateOpenTabOverflow, 0);
    openTabOverflowUpdateRef.current.settledTimeout = window.setTimeout(updateOpenTabOverflow, 120);
  }, [clearScheduledOpenTabOverflowUpdate, updateOpenTabOverflow]);

  useEffect(() => {
    return () => {
      clearScheduledContentTabOverflowUpdate();
      clearScheduledOpenTabOverflowUpdate();
    };
  }, [clearScheduledContentTabOverflowUpdate, clearScheduledOpenTabOverflowUpdate]);

  const setContentTabsNode = useCallback((node: HTMLDivElement | null) => {
    contentTabsRef.current = node;
    if (!node) {
      return;
    }

    scheduleContentTabOverflowUpdate();
  }, [scheduleContentTabOverflowUpdate]);

  const setOpenTabsNode = useCallback((node: HTMLDivElement | null) => {
    openTabsRef.current = node;
    if (!node) {
      setOpenTabScrollState(EMPTY_TAB_SCROLL_STATE);
      return;
    }

    scheduleOpenTabOverflowUpdate();
  }, [scheduleOpenTabOverflowUpdate]);

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
    const element = openTabsRef.current;
    if (!element) {
      return;
    }

    const handleResize = (): void => {
      scheduleOpenTabOverflowUpdate();
    };
    const handleScroll = (): void => {
      updateOpenTabOverflow();
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
    scheduleOpenTabOverflowUpdate,
    updateOpenTabOverflow,
    openTabs.length,
  ]);

  useEffect(() => {
    const element = openTabsRef.current;
    if (!element || !routeTab?.id) {
      return;
    }

    const frame = window.requestAnimationFrame(() => {
      const activeTab = element.querySelector<HTMLElement>(`[data-open-tab-id="${CSS.escape(routeTab.id)}"]`);
      if (!activeTab) {
        updateOpenTabOverflow();
        return;
      }

      const centeredLeft = activeTab.offsetLeft + activeTab.offsetWidth / 2 - element.clientWidth / 2;
      const maxScrollLeft = Math.max(0, element.scrollWidth - element.clientWidth);
      element.scrollLeft = Math.min(maxScrollLeft, Math.max(0, centeredLeft));
      scheduleOpenTabOverflowUpdate();
    });

    return () => {
      window.cancelAnimationFrame(frame);
    };
  }, [
    collapsed,
    navLayout,
    routeTab?.id,
    scheduleOpenTabOverflowUpdate,
    updateOpenTabOverflow,
    openTabs.length,
  ]);

  useEffect(() => {
    if (routeTab) {
      addOpenTab(routeTab);
    }
  }, [routeTab]);

  useEffect(() => {
    syncOpenTabsForRestoreMode(tabRestoreMode);
  }, [tabRestoreMode]);

  useEffect(() => {
    if (!openTabMenu) {
      return;
    }

    function closeMenu(): void {
      setOpenTabMenu(null);
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
  }, [openTabMenu]);

  function handleCloseOpenTab(tab: OpenTab): void {
    const closingActiveTab = routeTab?.id === tab.id;
    const tabsBeforeClose = readOpenTabsSnapshot();
    const closedIndex = tabsBeforeClose.findIndex((item) => item.id === tab.id);
    const nextActiveTab =
      tabsBeforeClose[closedIndex + 1] ?? tabsBeforeClose[closedIndex - 1] ?? tabsBeforeClose.find((item) => item.id !== tab.id);

    closeOpenTab(tab.id);

    if (closingActiveTab) {
      router.push(nextActiveTab?.href ?? "/home");
    }
  }

  function handleOpenTabDrop(targetTab: OpenTab): void {
    if (!draggedOpenTabId) {
      return;
    }

    reorderOpenTab(draggedOpenTabId, targetTab.id);
    setDraggedOpenTabId(null);
  }

  function handleOpenTabDragStart(tab: OpenTab, event: DragEvent<HTMLElement>): void {
    setDraggedOpenTabId(tab.id);
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData("text/plain", tab.id);
  }

  function addOpenTabMenu(event: MouseEvent<HTMLElement>, tab: OpenTab): void {
    event.preventDefault();
    setOpenTabMenu({ tabId: tab.id, x: event.clientX, y: event.clientY });
  }

  function addOpenTabMenuFromKeyboard(event: KeyboardEvent<HTMLElement>, tab: OpenTab): void {
    if (event.key !== "ContextMenu" && !(event.shiftKey && event.key === "F10")) {
      return;
    }

    event.preventDefault();
    const rect = event.currentTarget.getBoundingClientRect();
    setOpenTabMenu({
      tabId: tab.id,
      x: Math.round(rect.left + Math.min(rect.width - 24, 48)),
      y: Math.round(rect.top + rect.height - 4),
    });
  }

  function handleOpenMenuPin(tab: OpenTab): void {
    toggleOpenTabPinned(tab.id);
    setOpenTabMenu(null);
  }

  function handleOpenMenuClose(tab: OpenTab): void {
    handleCloseOpenTab(tab);
    setOpenTabMenu(null);
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

  function scrollOpenTabs(direction: -1 | 1): void {
    const element = openTabsRef.current;
    if (!element) {
      return;
    }

    const maxScrollLeft = Math.max(0, element.scrollWidth - element.clientWidth);
    const distance = Math.max(180, Math.floor(element.clientWidth * 0.72));
    element.scrollLeft = Math.min(maxScrollLeft, Math.max(0, element.scrollLeft + direction * distance));
    scheduleOpenTabOverflowUpdate();
  }

  function handleOpenTabWheel(event: WheelEvent<HTMLElement>): void {
    const element = openTabsRef.current;
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
    scheduleOpenTabOverflowUpdate();
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

  const activeNavId = nav.find((item) => isPrimaryNavRoute(pathname, item.href))?.href;
  const sidebarNavItems: MenuEntry[] = nav.map((item) => {
    const NavIcon = item.icon;

    return {
      ariaLabel: item.label,
      href: item.href,
      icon: <NavIcon aria-hidden="true" />,
      id: item.href,
      name: item.label,
    };
  });
  const topbarNavLinks = nav.map((item) => {
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
    <Controls
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

  const pinnedOpenTabs = openTabs.filter((tab) => tab.pinned);
  const regularOpenTabs = openTabs.filter((tab) => !tab.pinned);
  const openTabContextMenu = openTabMenuTab ? (
    <TabMenu
      onCloseTab={handleOpenMenuClose}
      onTogglePinned={handleOpenMenuPin}
      position={{ x: openTabMenu?.x ?? 0, y: openTabMenu?.y ?? 0 }}
      tab={openTabMenuTab}
    />
  ) : null;
  const sidebarOpenTabSections = (
    <>
      <Tabs
        activeTabId={routeTab?.id}
        collapsed={collapsed}
        draggedTabId={draggedOpenTabId}
        emptyMessage="Open a server or conversation to create a tab."
        onCloseTab={handleCloseOpenTab}
        onContextMenu={addOpenTabMenu}
        onDragEnd={() => setDraggedOpenTabId(null)}
        onDragStart={handleOpenTabDragStart}
        onDrop={handleOpenTabDrop}
        onKeyboardContextMenu={addOpenTabMenuFromKeyboard}
        pinnedTabs={pinnedOpenTabs}
        regularTabs={regularOpenTabs}
        variant="sidebar"
      />
      {openTabContextMenu}
    </>
  );
  const brand = <BrandLockup className={styles.brandLockup} collapsed={collapsed} size="lg" />;
  const topbarOpenTabStrip = (
    <>
      <Tabs
        activeTabId={routeTab?.id}
        collapsed={collapsed}
        draggedTabId={draggedOpenTabId}
        emptyMessage="Open a server or conversation to create a tab."
        onCloseTab={handleCloseOpenTab}
        onContextMenu={addOpenTabMenu}
        onDragEnd={() => setDraggedOpenTabId(null)}
        onDragStart={handleOpenTabDragStart}
        onDrop={handleOpenTabDrop}
        onKeyboardContextMenu={addOpenTabMenuFromKeyboard}
        onScrollTabs={scrollOpenTabs}
        onWheel={handleOpenTabWheel}
        pinnedTabs={pinnedOpenTabs}
        regularTabs={regularOpenTabs}
        scrollState={openTabScrollState}
        tabListRef={setOpenTabsNode}
        variant="topbar"
      />
      {openTabContextMenu}
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
                {topbarNavLinks}
              </nav>
            </div>
            <div className={styles.openTabsStack} role="group" aria-label="Open tabs">
              {topbarOpenTabStrip}
            </div>
            <div className={styles.topbarControls}>
              {profileControls}
            </div>
          </header>
        ) : (
          <aside className={styles.sidebar}>
            <div className={styles.sidebarPrimary}>
              {brand}
              <Menu
                activeId={activeNavId}
                activeIndicator="rail"
                aria-label="Primary"
                as="nav"
                collapsed={collapsed}
                iconColor="accent"
                idleBorder={false}
                items={sidebarNavItems}
                panel
                skin="sidebar"
                spacing="sm"
              />
            </div>

            <div className={styles.openTabsStack} role="group" aria-label="Open tabs">
              {sidebarOpenTabSections}
            </div>
            <div className={styles.sidebarControls}>
              {profileControls}
            </div>
          </aside>
        )}

        <section
          aria-describedby="main-page-subtitle"
          aria-labelledby="main-page-title"
          className={`${styles.content} ${hasContentTabs || tabActions ? "" : styles.contentNoTabBar}`}
        >
          <header className={styles.visuallyHidden}>
            <h1 id="main-page-title">{title}</h1>
            <p id="main-page-subtitle">{subtitle}</p>
          </header>
          <ContentTabs
            activeId={activeTabId}
            activeRef={activeContentTabRef}
            actions={tabActions}
            canScrollLeft={contentTabScrollState.canScrollLeft}
            canScrollRight={contentTabScrollState.canScrollRight}
            items={tabs}
            label={`${title} sections`}
            listRef={setContentTabsNode}
            onChange={onTabChange}
            onScrollLeft={() => scrollContentTabs(-1)}
            onScrollRight={() => scrollContentTabs(1)}
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
