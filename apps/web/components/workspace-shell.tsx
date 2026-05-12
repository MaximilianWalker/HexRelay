"use client";

import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { useEffect, useMemo, useSyncExternalStore } from "react";
import {
  IconAddressBook,
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
  children,
}: {
  title: string;
  subtitle: string;
  tabs: TabItem[];
  activeTabId: string;
  tabActions?: React.ReactNode;
  workspaceTab?: WorkspaceTabMeta;
  children: React.ReactNode;
}) {
  const pathname = usePathname();
  const router = useRouter();
  const navLayout = useSyncExternalStore<NavLayout>(subscribeWorkspacePreferences, readNavLayout, () => "sidebar");
  const collapsed = useSyncExternalStore(subscribeWorkspacePreferences, readSidebarCollapsed, () => false);
  const soundMuted = useSyncExternalStore(subscribeWorkspacePreferences, readSoundMuted, () => false);
  const microphoneMuted = useSyncExternalStore(subscribeWorkspacePreferences, readMicrophoneMuted, () => false);
  const profileSnapshot = useSyncExternalStore(subscribeWorkspacePreferences, readProfileSnapshot, () => DEFAULT_PROFILE);
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
                {!tab.pinned && !isServer ? (
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
  const workspaceTabSections = (
    <>
      {pinnedWorkspaceTabs.length > 0 ? (
        <div className={`${styles.workspaceSection} ${styles.workspaceSectionPinned}`} role="group" aria-label="Pinned tabs">
          {renderWorkspaceTabs(pinnedWorkspaceTabs)}
        </div>
      ) : null}
      <div className={styles.workspaceSection} role="group" aria-label="Workspace tabs">
        {renderWorkspaceTabs(regularWorkspaceTabs, "Open a server or conversation to create a tab.")}
      </div>
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
            <div className={styles.tabs}>
              {tabs.map((tab) => {
                const TabIcon = tab.icon;
                const tabClassName = `${styles.tab} ${tab.id === activeTabId ? styles.tabActive : ""}`;

                if (tab.onSelect) {
                  return (
                    <button
                      aria-pressed={tab.id === activeTabId}
                      className={tabClassName}
                      key={tab.id}
                      onClick={tab.onSelect}
                      type="button"
                    >
                      {TabIcon ? <TabIcon className={styles.tabIcon} aria-hidden="true" /> : null}
                      <span className={styles.tabLabel}>{tab.label}</span>
                    </button>
                  );
                }

                return (
                  <div className={tabClassName} key={tab.id}>
                    {TabIcon ? <TabIcon className={styles.tabIcon} aria-hidden="true" /> : null}
                    <span className={styles.tabLabel}>{tab.label}</span>
                  </div>
                );
              })}
            </div>
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
