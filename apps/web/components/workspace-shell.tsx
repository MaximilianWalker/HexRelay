"use client";

import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { useCallback, useEffect, useMemo, useRef, useState, useSyncExternalStore } from "react";
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
  IconMessageCircle,
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
  routeToWorkspaceTab,
  subscribeWorkspaceTabs,
  syncWorkspaceTabsForRestoreMode,
  toggleWorkspaceTabPinned,
  type WorkspaceTab,
} from "@/lib/workspace-tabs";

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

type ContentTabScrollState = {
  hasOverflow: boolean;
  canScrollLeft: boolean;
  canScrollRight: boolean;
};

const EMPTY_CONTENT_TAB_SCROLL_STATE: ContentTabScrollState = {
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

function getTabIcon(tab: WorkspaceTab): typeof IconServer2 {
  return tab.kind === "dm" ? IconMessageCircle : IconServer2;
}

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
  const [contentTabScrollState, setContentTabScrollState] = useState<ContentTabScrollState>(
    EMPTY_CONTENT_TAB_SCROLL_STATE,
  );
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
      setContentTabScrollState(EMPTY_CONTENT_TAB_SCROLL_STATE);
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
    const nextState: ContentTabScrollState = {
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

  const scheduleContentTabOverflowUpdate = useCallback(() => {
    window.requestAnimationFrame(updateContentTabOverflow);
    window.setTimeout(updateContentTabOverflow, 0);
    window.setTimeout(updateContentTabOverflow, 120);
  }, [updateContentTabOverflow]);

  const setContentTabsNode = useCallback((node: HTMLDivElement | null) => {
    contentTabsRef.current = node;
    if (!node) {
      return;
    }

    scheduleContentTabOverflowUpdate();
  }, [scheduleContentTabOverflowUpdate]);

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
    if (routeTab) {
      openWorkspaceTab(routeTab);
    }
  }, [routeTab]);

  useEffect(() => {
    syncWorkspaceTabsForRestoreMode(tabRestoreMode);
  }, [tabRestoreMode]);

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
    <button
      aria-label={isTopbar ? "Switch to sidebar layout" : "Switch to top bar layout"}
      className={styles.iconButton}
      onClick={toggleNavLayout}
      title={isTopbar ? "Use sidebar" : "Use top bar"}
      type="button"
    >
      <LayoutIcon className={styles.controlIcon} aria-hidden="true" />
    </button>
  );

  const profileControls = (
    <>
      <div className={styles.profileSummary} title={profile.name}>
        <div className={styles.profileAvatar}>{getInitials(profile.name)}</div>
        <div className={styles.profileDetails}>
          <p className={styles.profileName}>{profile.name}</p>
          <p className={styles.profileStatus}>{profile.status}</p>
        </div>
      </div>
      <div className={styles.profileActions}>
        <button
          aria-label={soundMuted ? "Unmute sound" : "Mute sound"}
          aria-pressed={soundMuted}
          className={`${styles.iconButton} ${soundMuted ? styles.iconButtonActive : ""}`}
          onClick={() => setSoundMuted(!soundMuted)}
          title={soundMuted ? "Unmute sound" : "Mute sound"}
          type="button"
        >
          <SoundIcon className={styles.controlIcon} aria-hidden="true" />
        </button>
        <button
          aria-label={microphoneMuted ? "Unmute microphone" : "Mute microphone"}
          aria-pressed={microphoneMuted}
          className={`${styles.iconButton} ${microphoneMuted ? styles.iconButtonActive : ""}`}
          onClick={() => setMicrophoneMuted(!microphoneMuted)}
          title={microphoneMuted ? "Unmute microphone" : "Mute microphone"}
          type="button"
        >
          <MicrophoneIcon className={styles.controlIcon} aria-hidden="true" />
        </button>
        {layoutSwitch}
      </div>
    </>
  );

  function renderWorkspaceTabs(tabsToRender: WorkspaceTab[], emptyMessage?: string): React.ReactNode {
    if (tabsToRender.length === 0) {
      return emptyMessage ? <p className={styles.emptyTabs}>{emptyMessage}</p> : null;
    }

    return (
      <div className={styles.workspaceTabs} role="list">
        {tabsToRender.map((tab) => {
          const TabIcon = getTabIcon(tab);
          const active = routeTab?.id === tab.id;
          const unread = normalizeUnread(tab.unread);
          const imageLabel = tab.imageLabel ?? tab.label;
          const isServer = tab.kind === "server";

          return (
            <div
              className={`${styles.workspaceTab} ${active ? styles.workspaceTabActive : ""} ${
                tab.pinned ? styles.workspaceTabPinned : ""
              }`}
              key={tab.id}
              role="listitem"
            >
              <Link
                aria-current={active ? "page" : undefined}
                aria-label={`${tab.kind === "dm" ? "Conversation" : "Server"}: ${tab.label}`}
                className={styles.workspaceTabLink}
                href={tab.href}
              >
                {isServer ? (
                  <span className={styles.workspaceTabImage} aria-hidden="true">
                    {getInitials(imageLabel)}
                  </span>
                ) : (
                  <TabIcon className={styles.workspaceTabIcon} aria-hidden="true" />
                )}
                <span className={styles.workspaceTabLabel}>{tab.label}</span>
              </Link>
              <div className={styles.workspaceTabActions}>
                {isServer && unread > 0 ? (
                  <span className={styles.workspaceTabBadge} aria-label={`${unread} unread notifications`}>
                    {unread}
                  </span>
                ) : null}
                <button
                  aria-label={tab.pinned ? `Unpin ${tab.label}` : `Pin ${tab.label}`}
                  className={styles.workspaceTabAction}
                  onClick={() => toggleWorkspaceTabPinned(tab.id)}
                  title={tab.pinned ? "Unpin tab" : "Pin tab"}
                  type="button"
                >
                  {tab.pinned ? (
                    <IconPinnedOff className={styles.workspaceTabIcon} aria-hidden="true" />
                  ) : (
                    <IconPinned className={styles.workspaceTabIcon} aria-hidden="true" />
                  )}
                </button>
                {!tab.pinned ? (
                  <button
                    aria-label={`Close ${tab.label}`}
                    className={styles.workspaceTabAction}
                    onClick={() => handleCloseWorkspaceTab(tab)}
                    title="Close tab"
                    type="button"
                  >
                    <IconX className={styles.workspaceTabIcon} aria-hidden="true" />
                  </button>
                ) : null}
              </div>
            </div>
          );
        })}
      </div>
    );
  }

  const pinnedWorkspaceTabs = workspaceTabs.filter((tab) => tab.pinned);
  const regularWorkspaceTabs = workspaceTabs.filter((tab) => !tab.pinned);
  const showRegularWorkspaceTabs = regularWorkspaceTabs.length > 0 || !collapsed;
  const workspaceTabSections = (
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
    </>
  );

  return (
    <main className={`${styles.shell} ${isTopbar ? styles.topbarMode : ""} ${collapsed ? styles.collapsed : ""}`}>
      <RealtimeClient />
      <div className={styles.frame}>
        {isTopbar ? (
          <header className={styles.topbar}>
            <div className={styles.topbarPrimary}>
              <p className={styles.brand}>HexRelay</p>
              <nav aria-label="Primary" className={styles.topbarNav}>
                {navLinks}
              </nav>
            </div>
            <div className={styles.workspaceStack} role="group" aria-label="Workspace tabs">
              {workspaceTabSections}
            </div>
            <div className={styles.topbarControls}>
              {profileControls}
              <button
                aria-label={collapsed ? "Expand top bar" : "Collapse top bar"}
                className={styles.iconButton}
                onClick={toggleSidebar}
                title={collapsed ? "Expand top bar" : "Collapse top bar"}
                type="button"
              >
                <TopbarToggleIcon className={styles.controlIcon} aria-hidden="true" />
              </button>
            </div>
          </header>
        ) : (
          <aside className={styles.sidebar}>
            <div className={styles.sidebarPrimary}>
              <p className={styles.brand}>HexRelay</p>
              <nav aria-label="Primary" className={styles.topNav}>
                {navLinks}
              </nav>
            </div>

            <div className={styles.workspaceStack} role="group" aria-label="Workspace tabs">
              {workspaceTabSections}
            </div>
            <div className={styles.sidebarControls}>
              {profileControls}
              <button
                aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
                className={styles.iconButton}
                onClick={toggleSidebar}
                title={collapsed ? "Expand sidebar" : "Collapse sidebar"}
                type="button"
              >
                <SidebarToggleIcon className={styles.controlIcon} aria-hidden="true" />
              </button>
            </div>
          </aside>
        )}

        <section
          aria-describedby="workspace-page-subtitle"
          aria-labelledby="workspace-page-title"
          className={styles.content}
        >
          <header className={styles.visuallyHidden}>
            <h1 id="workspace-page-title">{title}</h1>
            <p id="workspace-page-subtitle">{subtitle}</p>
          </header>
          <div className={styles.tabBar}>
            {contentTabScrollState.canScrollLeft ? (
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
            <div aria-label={`${title} tabs`} className={styles.tabs} ref={setContentTabsNode} role="tablist">
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
                    aria-selected={active}
                    className={tabClassName}
                    data-tab-id={tab.id}
                    key={tab.id}
                    onClick={handleTabSelect}
                    ref={active ? activeContentTabRef : undefined}
                    role="tab"
                    type="button"
                  >
                    {tabContent}
                  </button>
                ) : (
                  <div
                    aria-selected={active}
                    className={tabClassName}
                    data-tab-id={tab.id}
                    key={tab.id}
                    ref={active ? activeContentTabRef : undefined}
                    role="tab"
                  >
                    {tabContent}
                  </div>
                );
              })}
            </div>
            {contentTabScrollState.canScrollRight ? (
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

          <section className={styles.body}>{children}</section>
        </section>
      </div>

      <nav aria-label="Primary" className={styles.mobileTabs}>
        <button
          aria-label="Switch to top bar layout"
          className={styles.mobileLayoutButton}
          onClick={toggleNavLayout}
          title="Use top bar"
          type="button"
        >
          <IconLayoutNavbar className={styles.mobileTabIcon} aria-hidden="true" />
          <span>Layout</span>
        </button>
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
