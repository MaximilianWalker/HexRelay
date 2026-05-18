"use client";

import { useEffect, useMemo, useState, useSyncExternalStore, type ReactNode } from "react";
import { useRouter } from "next/navigation";
import {
  IconBell,
  IconDeviceLaptop,
  IconFlask,
  IconMicrophone,
  IconPalette,
  IconRoute,
  IconSettings,
  IconShieldLock,
  IconUserCircle,
} from "@tabler/icons-react";

import { WorkspaceShell } from "@/components/workspace-shell";
import {
  activateTestingSession,
  fetchDmPolicy,
  fetchTestingProfiles,
  storeCsrfToken,
  updateDmPolicy,
  type DmInboundPolicy,
  type TestingProfileSummary,
} from "@/lib/api";
import { env } from "@/lib/env";
import { readActivePersonaId, readPersonas, switchPersona, upsertPersona } from "@/lib/personas";
import { getPersonaSession, setPersonaSession } from "@/lib/sessions";
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
  setTabRestoreMode,
  subscribeWorkspacePreferences,
  type NavLayout,
  type TabRestoreMode,
} from "@/lib/workspace-preferences";
import { syncWorkspaceTabsForRestoreMode } from "@/lib/workspace-tabs";

import settingsStyles from "./settings.module.css";

const DM_POLICY_KEY = "hexrelay.settings.dm-policy";
const DM_POLICY_EVENT = "hexrelay-dm-policy-changed";
const SHOW_DEV_TESTING = process.env.NODE_ENV === "development";

type DmPolicy = DmInboundPolicy;
type SettingStatus = "Live" | "Review" | "Locked" | "Dev only";
type SettingsTabId =
  | "profile"
  | "privacy"
  | "notifications"
  | "voice-video"
  | "appearance"
  | "devices"
  | "advanced"
  | "developer";

type SettingsCategory = {
  id: SettingsTabId;
  label: string;
  summary: string;
  icon: typeof IconSettings;
};

type TestingShortcut = {
  id: string;
  label: string;
  description: string;
  profileId: string;
  href: string;
  scenarioId: string;
};

const SETTINGS_TABS: SettingsCategory[] = [
  {
    id: "profile",
    label: "Profile",
    summary: "Active identity, session state, and account recovery surfaces.",
    icon: IconUserCircle,
  },
  {
    id: "privacy",
    label: "Privacy",
    summary: "Inbound DM policy, contact approval, and discoverability defaults.",
    icon: IconShieldLock,
  },
  {
    id: "notifications",
    label: "Notifications",
    summary: "Future desktop, DM, request, and server mention notification controls.",
    icon: IconBell,
  },
  {
    id: "voice-video",
    label: "Voice & Video",
    summary: "Local voice controls now, richer device selection later.",
    icon: IconMicrophone,
  },
  {
    id: "appearance",
    label: "Appearance",
    summary: "Navigation layout, tab restore behavior, and future display preferences.",
    icon: IconPalette,
  },
  {
    id: "devices",
    label: "Devices",
    summary: "Local device state, private key boundaries, and session revocation.",
    icon: IconDeviceLaptop,
  },
  {
    id: "advanced",
    label: "Advanced",
    summary: "Runtime endpoints, portability tools, and diagnostics.",
    icon: IconSettings,
  },
  {
    id: "developer",
    label: "Developer",
    summary: "Development-only seeded profiles and manual validation shortcuts.",
    icon: IconFlask,
  },
];

function isSettingsTabId(value: string): value is SettingsTabId {
  return SETTINGS_TABS.some((tab) => tab.id === value);
}

const DEV_TESTING_SHORTCUTS: TestingShortcut[] = [
  {
    id: "alice-contacts",
    label: "Alice contacts",
    description: "Contact list with accepted and pending fixtures.",
    profileId: "alice.primary",
    href: "/contacts",
    scenarioId: "dm-basic / contacts-edge",
  },
  {
    id: "alice-to-bob-dm",
    label: "Alice -> Bob DM",
    description: "Accepted private-message fixture as Alice.",
    profileId: "alice.primary",
    href: "/contacts/usr-test-bob/messages",
    scenarioId: "dm-basic",
  },
  {
    id: "bob-to-alice-dm",
    label: "Bob -> Alice DM",
    description: "Accepted private-message fixture as Bob.",
    profileId: "bob.primary",
    href: "/contacts/usr-test-alice/messages",
    scenarioId: "dm-basic",
  },
  {
    id: "alice-to-carol-pending",
    label: "Alice -> Carol pending",
    description: "Outbound pending contact guard.",
    profileId: "alice.primary",
    href: "/contacts/usr-test-carol/messages",
    scenarioId: "contacts-edge",
  },
  {
    id: "alice-to-dave-inbound",
    label: "Alice -> Dave inbound",
    description: "Inbound contact-request guard.",
    profileId: "alice.primary",
    href: "/contacts/usr-test-dave/messages",
    scenarioId: "contacts-edge",
  },
  {
    id: "alice-atlas-server",
    label: "Alice Atlas server",
    description: "Shared server fixture with Alice's favorite and unread state.",
    profileId: "alice.primary",
    href: "/servers/fixture-server-atlas",
    scenarioId: "server-chat",
  },
  {
    id: "bob-atlas-server",
    label: "Bob Atlas server",
    description: "Shared server fixture with Bob's member state.",
    profileId: "bob.primary",
    href: "/servers/fixture-server-atlas",
    scenarioId: "server-chat",
  },
];

let fallbackDmPolicy: DmPolicy = "friends_only";

function readDmPolicy(): DmPolicy {
  if (typeof window === "undefined") {
    return "friends_only";
  }

  try {
    const stored = window.localStorage.getItem(DM_POLICY_KEY);
    return stored === "same_server" || stored === "anyone" || stored === "friends_only"
      ? stored
      : "friends_only";
  } catch {
    return fallbackDmPolicy;
  }
}

function writeDmPolicy(next: DmPolicy): void {
  fallbackDmPolicy = next;

  try {
    window.localStorage.setItem(DM_POLICY_KEY, next);
  } catch {
    // Keep the in-memory value so settings remain usable without localStorage.
  }

  if (typeof window !== "undefined") {
    window.dispatchEvent(new Event(DM_POLICY_EVENT));
  }
}

function subscribeDmPolicy(onChange: () => void): () => void {
  if (typeof window === "undefined") {
    return () => {};
  }

  function handleStorage(event: StorageEvent): void {
    if (event.key === DM_POLICY_KEY) {
      onChange();
    }
  }

  window.addEventListener("storage", handleStorage);
  window.addEventListener(DM_POLICY_EVENT, onChange);

  return () => {
    window.removeEventListener("storage", handleStorage);
    window.removeEventListener(DM_POLICY_EVENT, onChange);
  };
}

function statusClass(status: SettingStatus): string {
  if (status === "Live") {
    return settingsStyles.statusLive;
  }
  if (status === "Dev only") {
    return settingsStyles.statusDev;
  }
  if (status === "Locked") {
    return settingsStyles.statusLocked;
  }

  return settingsStyles.statusReview;
}

function booleanLabel(value: boolean): string {
  return value ? "On" : "Off";
}

function SettingPanel({
  category,
  children,
}: {
  category: SettingsCategory;
  children: ReactNode;
}) {
  return (
    <section aria-label={category.label} className={settingsStyles.panel} role="tabpanel">
      <div className={settingsStyles.settingList}>{children}</div>
    </section>
  );
}

function SettingRow({
  label,
  description,
  status,
  children,
}: {
  label: string;
  description: string;
  status: SettingStatus;
  children: ReactNode;
}) {
  return (
    <div className={settingsStyles.settingRow}>
      <div className={settingsStyles.settingCopy}>
        <div className={settingsStyles.settingHeading}>
          <p className={settingsStyles.settingLabel}>{label}</p>
          <span className={`${settingsStyles.status} ${statusClass(status)}`}>{status}</span>
        </div>
        <p className={settingsStyles.settingDescription}>{description}</p>
      </div>
      <div className={settingsStyles.settingControl}>{children}</div>
    </div>
  );
}

function ToggleControl({
  checked,
  disabled,
  label,
  onChange,
}: {
  checked: boolean;
  disabled?: boolean;
  label: string;
  onChange?: (next: boolean) => void;
}) {
  return (
    <button
      aria-checked={checked}
      aria-label={label}
      className={`${settingsStyles.toggle} ${checked ? settingsStyles.toggleOn : ""}`}
      disabled={disabled}
      onClick={() => onChange?.(!checked)}
      role="switch"
      type="button"
    >
      <span className={settingsStyles.toggleTrack}>
        <span className={settingsStyles.toggleThumb} />
      </span>
      <span>{booleanLabel(checked)}</span>
    </button>
  );
}

function ReadOnlyValue({ children }: { children: ReactNode }) {
  return <span className={settingsStyles.readOnlyValue}>{children}</span>;
}

export default function SettingsPage() {
  const router = useRouter();
  const navLayout = useSyncExternalStore<NavLayout>(subscribeWorkspacePreferences, readNavLayout, () => "sidebar");
  const sidebarCollapsed = useSyncExternalStore(subscribeWorkspacePreferences, readSidebarCollapsed, () => false);
  const soundMuted = useSyncExternalStore(subscribeWorkspacePreferences, readSoundMuted, () => false);
  const microphoneMuted = useSyncExternalStore(subscribeWorkspacePreferences, readMicrophoneMuted, () => false);
  const tabRestoreMode = useSyncExternalStore<TabRestoreMode>(
    subscribeWorkspacePreferences,
    readTabRestoreMode,
    () => "pinned",
  );
  const dmPolicy = useSyncExternalStore<DmPolicy>(subscribeDmPolicy, readDmPolicy, () => "friends_only");
  const [personas, setPersonas] = useState(() => readPersonas());
  const identityId = useMemo(() => readActivePersonaId() ?? personas[0]?.id ?? null, [personas]);
  const activePersona = useMemo(
    () => personas.find((persona) => persona.id === identityId) ?? personas[0] ?? null,
    [identityId, personas],
  );
  const hasSession = useMemo(() => (identityId ? getPersonaSession(identityId) !== null : false), [identityId]);
  const [policyBusy, setPolicyBusy] = useState(false);
  const [policyMessage, setPolicyMessage] = useState<string | null>(null);
  const [testingProfiles, setTestingProfiles] = useState<TestingProfileSummary[]>([]);
  const [selectedTestingProfileId, setSelectedTestingProfileId] = useState("");
  const [selectedShortcutId, setSelectedShortcutId] = useState(DEV_TESTING_SHORTCUTS[0]?.id ?? "");
  const [testingBusy, setTestingBusy] = useState<string | null>(null);
  const [testingMessage, setTestingMessage] = useState<string | null>(null);

  const testingProfilesById = useMemo(
    () => new Map(testingProfiles.map((profile) => [profile.profile_id, profile])),
    [testingProfiles],
  );
  const selectedTestingProfile =
    testingProfilesById.get(selectedTestingProfileId) ?? testingProfiles[0] ?? null;
  const selectedShortcut =
    DEV_TESTING_SHORTCUTS.find((shortcut) => shortcut.id === selectedShortcutId) ?? DEV_TESTING_SHORTCUTS[0];
  const selectedShortcutProfile = selectedShortcut
    ? testingProfilesById.get(selectedShortcut.profileId) ?? null
    : null;
  const devTestingAvailable = testingProfiles.length > 0;
  const [activeSettingsTab, setActiveSettingsTab] = useState<SettingsTabId>("profile");
  const visibleSettingsTabs = useMemo(
    () => (SHOW_DEV_TESTING ? SETTINGS_TABS : SETTINGS_TABS.filter((tab) => tab.id !== "developer")),
    [],
  );
  const activeSettingsCategory =
    visibleSettingsTabs.find((tab) => tab.id === activeSettingsTab) ?? visibleSettingsTabs[0] ?? SETTINGS_TABS[0];
  const shellTabs = useMemo(
    () => visibleSettingsTabs.map(({ icon, id, label }) => ({ icon, id, label })),
    [visibleSettingsTabs],
  );

  useEffect(() => {
    if (visibleSettingsTabs.some((tab) => tab.id === activeSettingsTab)) {
      return;
    }

    setActiveSettingsTab(visibleSettingsTabs[0]?.id ?? "profile");
  }, [activeSettingsTab, visibleSettingsTabs]);

  useEffect(() => {
    function syncTabFromHash(): void {
      const tabId = window.location.hash.replace("#", "");
      if (isSettingsTabId(tabId) && visibleSettingsTabs.some((tab) => tab.id === tabId)) {
        setActiveSettingsTab(tabId);
      }
    }

    syncTabFromHash();
    window.addEventListener("hashchange", syncTabFromHash);

    return () => window.removeEventListener("hashchange", syncTabFromHash);
  }, [visibleSettingsTabs]);

  useEffect(() => {
    let active = true;

    const run = async (): Promise<void> => {
      if (!hasSession) {
        setPolicyMessage("Select or create a profile before changing DM policy.");
        return;
      }

      const result = await fetchDmPolicy();
      if (!active) {
        return;
      }

      if (result.ok) {
        writeDmPolicy(result.data.inbound_policy);
        setPolicyMessage(null);
        return;
      }

      setPolicyMessage(result.message);
    };

    void run();

    return () => {
      active = false;
    };
  }, [hasSession, identityId]);

  useEffect(() => {
    if (!SHOW_DEV_TESTING) {
      return;
    }

    let active = true;

    const run = async (): Promise<void> => {
      const result = await fetchTestingProfiles().catch(() => null);
      if (!active) {
        return;
      }

      if (!result?.ok) {
        setTestingProfiles([]);
        setTestingMessage("Dev testing API unavailable. Start the local API with dev testing enabled.");
        return;
      }

      setTestingProfiles(result.data.items);
      setTestingMessage(null);
    };

    void run();

    return () => {
      active = false;
    };
  }, []);

  async function updatePolicy(next: DmPolicy): Promise<void> {
    if (!hasSession) {
      setPolicyMessage("Select or create a profile before changing DM policy.");
      return;
    }

    setPolicyBusy(true);
    setPolicyMessage(null);

    try {
      const result = await updateDmPolicy({ inboundPolicy: next });
      if (result.ok) {
        writeDmPolicy(result.data.inbound_policy);
        setPolicyMessage("DM inbound policy saved.");
        return;
      }

      setPolicyMessage(result.message);
    } finally {
      setPolicyBusy(false);
    }
  }

  function updateProfile(personaId: string): void {
    if (!personaId) {
      return;
    }

    setPersonas(switchPersona(personaId));
  }

  function updateTabRestoreMode(next: TabRestoreMode): void {
    syncWorkspaceTabsForRestoreMode(next);
    setTabRestoreMode(next);
  }

  async function activateProfileSession(
    profile: TestingProfileSummary,
    busyId: string,
    successMessage: string,
  ): Promise<boolean> {
    setTestingBusy(busyId);
    setTestingMessage(null);

    try {
      const result = await activateTestingSession({ profileId: profile.profile_id }).catch(() => null);
      if (!result) {
        setTestingMessage("Testing session unavailable. Start the local API with dev testing enabled.");
        return false;
      }

      if (!result.ok) {
        setTestingMessage(result.message);
        return false;
      }

      upsertPersona({
        id: result.data.identity_id,
        name: result.data.profile_id,
      });
      setPersonaSession(result.data.identity_id, {
        sessionId: result.data.session_id,
        expiresAt: result.data.expires_at,
      });
      storeCsrfToken(result.data.csrf_token);
      setPersonas(readPersonas());
      setTestingMessage(successMessage);
      return true;
    } finally {
      setTestingBusy(null);
    }
  }

  async function activateSelectedTestingProfile(): Promise<void> {
    if (!selectedTestingProfile) {
      setTestingMessage("Choose an available testing profile first.");
      return;
    }

    await activateProfileSession(
      selectedTestingProfile,
      "profile",
      `Activated ${selectedTestingProfile.profile_id}.`,
    );
  }

  async function openSelectedTestingShortcut(): Promise<void> {
    if (!selectedShortcut) {
      return;
    }
    if (!selectedShortcutProfile) {
      setTestingMessage(`Profile ${selectedShortcut.profileId} unavailable for this shortcut.`);
      return;
    }

    const activated = await activateProfileSession(
      selectedShortcutProfile,
      "shortcut",
      `Activated ${selectedShortcut.profileId}; opening ${selectedShortcut.label}.`,
    );
    if (activated) {
      router.push(selectedShortcut.href);
    }
  }

  function selectSettingsTab(tabId: string): void {
    if (!isSettingsTabId(tabId) || !visibleSettingsTabs.some((tab) => tab.id === tabId)) {
      return;
    }

    setActiveSettingsTab(tabId);
    window.history.replaceState(null, "", `#${tabId}`);
  }

  return (
    <WorkspaceShell
      activeTabId={activeSettingsTab}
      onTabChange={selectSettingsTab}
      subtitle="Profile, privacy, device, workspace, and developer settings"
      tabs={shellTabs}
      title="Settings"
    >
      <section className={settingsStyles.page}>
        {activeSettingsTab === "profile" ? (
          <SettingPanel category={activeSettingsCategory}>
          <SettingRow
            description="The profile used for local sessions, privacy policy updates, contacts, and fixture validation."
            label="Active profile"
            status="Live"
          >
            <select
              aria-label="Active profile"
              className={settingsStyles.select}
              disabled={personas.length === 0}
              onChange={(event) => updateProfile(event.target.value)}
              value={activePersona?.id ?? ""}
            >
              {personas.length === 0 ? <option value="">No profile</option> : null}
              {personas.map((persona) => (
                <option key={persona.id} value={persona.id}>
                  {persona.name}
                </option>
              ))}
            </select>
          </SettingRow>
          <SettingRow
            description="Shows whether the selected profile has a browser runtime session."
            label="Session status"
            status="Live"
          >
            <ReadOnlyValue>{hasSession ? "Active" : "No active session"}</ReadOnlyValue>
          </SettingRow>
          <SettingRow
            description="Session revoke on profile switch or removal is mandatory for the current security model."
            label="Session revoke on switch/remove"
            status="Locked"
          >
            <ToggleControl checked disabled label="Session revoke on switch or remove" />
          </SettingRow>
          <SettingRow
            description="Recovery material remains device-local; export flow still needs final product approval."
            label="Recovery phrase export"
            status="Review"
          >
            <button className={settingsStyles.secondaryButton} disabled type="button">
              Review flow
            </button>
          </SettingRow>
          </SettingPanel>
        ) : null}

        {activeSettingsTab === "privacy" ? (
          <SettingPanel category={activeSettingsCategory}>
          <SettingRow
            description={policyMessage ?? "Controls who can start an inbound private-message conversation."}
            label="DM inbound policy"
            status="Live"
          >
            <select
              aria-label="DM inbound policy"
              className={settingsStyles.select}
              disabled={!hasSession || policyBusy}
              onChange={(event) => void updatePolicy(event.target.value as DmPolicy)}
              value={dmPolicy}
            >
              <option value="friends_only">Friends only</option>
              <option value="same_server">Same server</option>
              <option value="anyone">Anyone</option>
            </select>
          </SettingRow>
          <SettingRow
            description="Contact invite creation and redemption already live in the Contacts surface."
            label="Contact invite links"
            status="Review"
          >
            <select aria-label="Contact invite links" className={settingsStyles.select} disabled value="enabled">
              <option value="enabled">Enabled</option>
              <option value="disabled">Disabled</option>
            </select>
          </SettingRow>
          <SettingRow
            description="Current product behavior requires explicit approval before contacts can message."
            label="Contact request approval"
            status="Review"
          >
            <select aria-label="Contact request approval" className={settingsStyles.select} disabled value="manual">
              <option value="manual">Manual approval</option>
              <option value="auto">Auto-accept trusted invites</option>
            </select>
          </SettingRow>
          <SettingRow
            description="Discovery is not part of the current MVP surface."
            label="Profile discoverability"
            status="Review"
          >
            <select aria-label="Profile discoverability" className={settingsStyles.select} disabled value="off">
              <option value="off">Off</option>
              <option value="contacts">Contacts only</option>
              <option value="server">Shared servers</option>
            </select>
          </SettingRow>
          </SettingPanel>
        ) : null}

        {activeSettingsTab === "devices" ? (
          <SettingPanel category={activeSettingsCategory}>
          <SettingRow
            description="Runtime sessions are isolated to browser session storage per persona."
            label="Current session storage"
            status="Locked"
          >
            <ReadOnlyValue>Session storage</ReadOnlyValue>
          </SettingRow>
          <SettingRow
            description="Private keys stay client/device-only and are not uploaded to server nodes."
            label="Private key storage"
            status="Locked"
          >
            <ReadOnlyValue>Device-only</ReadOnlyValue>
          </SettingRow>
          <SettingRow
            description="Profile-device heartbeat exists in the API; user-facing device management is not wired yet."
            label="Device heartbeat mode"
            status="Review"
          >
            <select aria-label="Device heartbeat mode" className={settingsStyles.select} disabled value="runtime">
              <option value="runtime">Runtime managed</option>
              <option value="manual">Manual</option>
            </select>
          </SettingRow>
          <SettingRow
            description="Future control for revoking stale device sessions from the selected profile."
            label="Revoke other devices"
            status="Review"
          >
            <button className={settingsStyles.secondaryButton} disabled type="button">
              Review action
            </button>
          </SettingRow>
          </SettingPanel>
        ) : null}

        {activeSettingsTab === "notifications" ? (
          <SettingPanel category={activeSettingsCategory}>
          <SettingRow
            description="Desktop notification permission and delivery are not implemented in the web client yet."
            label="Desktop notifications"
            status="Review"
          >
            <ToggleControl checked={false} disabled label="Desktop notifications" />
          </SettingRow>
          <SettingRow
            description="Notification rules for encrypted DMs still need product copy and runtime delivery hooks."
            label="DM notifications"
            status="Review"
          >
            <select aria-label="DM notifications" className={settingsStyles.select} disabled value="mentions_dms">
              <option value="mentions_dms">Mentions and DMs</option>
              <option value="mentions">Mentions only</option>
              <option value="off">Off</option>
            </select>
          </SettingRow>
          <SettingRow
            description="Friend/contact request notification behavior still needs approval."
            label="Contact request notifications"
            status="Review"
          >
            <ToggleControl checked={false} disabled label="Contact request notifications" />
          </SettingRow>
          <SettingRow
            description="Server-channel notification policy is future work for guild/channel surfaces."
            label="Server channel notifications"
            status="Review"
          >
            <select aria-label="Server channel notifications" className={settingsStyles.select} disabled value="mentions">
              <option value="mentions">Mentions only</option>
              <option value="all">All messages</option>
              <option value="muted">Muted</option>
            </select>
          </SettingRow>
          </SettingPanel>
        ) : null}

        {activeSettingsTab === "voice-video" ? (
          <SettingPanel category={activeSettingsCategory}>
          <SettingRow
            description="Mirrors the current workspace microphone quick control."
            label="Microphone muted"
            status="Live"
          >
            <ToggleControl checked={microphoneMuted} label="Microphone muted" onChange={setMicrophoneMuted} />
          </SettingRow>
          <SettingRow
            description="Mirrors the current workspace sound quick control."
            label="Sound muted"
            status="Live"
          >
            <ToggleControl checked={soundMuted} label="Sound muted" onChange={setSoundMuted} />
          </SettingRow>
          <SettingRow
            description="Voice device selection should stay local to the desktop/browser runtime."
            label="Input device"
            status="Review"
          >
            <select aria-label="Input device" className={settingsStyles.select} disabled value="system">
              <option value="system">System default</option>
            </select>
          </SettingRow>
          <SettingRow
            description="Output routing belongs with voice readiness, not the current MVP settings behavior."
            label="Output device"
            status="Review"
          >
            <select aria-label="Output device" className={settingsStyles.select} disabled value="system">
              <option value="system">System default</option>
            </select>
          </SettingRow>
          <SettingRow
            description="Audio processing controls should be added only with the actual voice stack."
            label="Noise suppression"
            status="Review"
          >
            <ToggleControl checked={false} disabled label="Noise suppression" />
          </SettingRow>
          </SettingPanel>
        ) : null}

        {activeSettingsTab === "appearance" ? (
          <SettingPanel category={activeSettingsCategory}>
          <SettingRow
            description="Switches the main app navigation between sidebar and top bar layouts."
            label="Navigation layout"
            status="Live"
          >
            <select
              aria-label="Navigation layout"
              className={settingsStyles.select}
              onChange={(event) => setNavLayout(event.target.value as NavLayout)}
              value={navLayout}
            >
              <option value="sidebar">Sidebar</option>
              <option value="topbar">Top bar</option>
            </select>
          </SettingRow>
          <SettingRow
            description="Controls whether the current sidebar/top bar chrome is collapsed."
            label="Navigation collapsed"
            status="Live"
          >
            <ToggleControl checked={sidebarCollapsed} label="Navigation collapsed" onChange={setSidebarCollapsed} />
          </SettingRow>
          <SettingRow
            description="Controls whether normal workspace tabs reopen across app sessions."
            label="Workspace tab restore"
            status="Live"
          >
            <select
              aria-label="Workspace tab restore"
              className={settingsStyles.select}
              onChange={(event) => updateTabRestoreMode(event.target.value as TabRestoreMode)}
              value={tabRestoreMode}
            >
              <option value="pinned">Pinned tabs only</option>
              <option value="all">Pinned and normal tabs</option>
            </select>
          </SettingRow>
          <SettingRow
            description="Theme support is not implemented yet."
            label="Theme"
            status="Review"
          >
            <select aria-label="Theme" className={settingsStyles.select} disabled value="system">
              <option value="system">System</option>
              <option value="light">Light</option>
              <option value="dark">Dark</option>
            </select>
          </SettingRow>
          <SettingRow
            description="Message density belongs with the final chat/channel surfaces."
            label="Message density"
            status="Review"
          >
            <select aria-label="Message density" className={settingsStyles.select} disabled value="comfortable">
              <option value="comfortable">Comfortable</option>
              <option value="compact">Compact</option>
            </select>
          </SettingRow>
          </SettingPanel>
        ) : null}

        {activeSettingsTab === "advanced" ? (
          <SettingPanel category={activeSettingsCategory}>
          <SettingRow
            description="Public API endpoint used by this web client."
            label="API base URL"
            status="Live"
          >
            <ReadOnlyValue>{env.NEXT_PUBLIC_API_BASE_URL}</ReadOnlyValue>
          </SettingRow>
          <SettingRow
            description="Realtime websocket endpoint used by this web client."
            label="Realtime URL"
            status="Live"
          >
            <ReadOnlyValue>{env.NEXT_PUBLIC_REALTIME_WS_URL}</ReadOnlyValue>
          </SettingRow>
          <SettingRow
            description="User-owned export/import is a product guardrail but not wired into the web client yet."
            label="Export profile data"
            status="Review"
          >
            <button className={settingsStyles.secondaryButton} disabled type="button">
              Review export
            </button>
          </SettingRow>
          <SettingRow
            description="Import flow should be specified with the same portability/export model."
            label="Import profile data"
            status="Review"
          >
            <button className={settingsStyles.secondaryButton} disabled type="button">
              Review import
            </button>
          </SettingRow>
          <SettingRow
            description="Diagnostics should be added after runtime health states are normalized."
            label="Diagnostics mode"
            status="Review"
          >
            <ToggleControl checked={false} disabled label="Diagnostics mode" />
          </SettingRow>
          </SettingPanel>
        ) : null}

        {SHOW_DEV_TESTING && activeSettingsTab === "developer" ? (
          <SettingPanel category={activeSettingsCategory}>
            <div className={devTestingAvailable ? settingsStyles.notice : settingsStyles.warningNotice}>
              <IconFlask className={settingsStyles.icon} aria-hidden="true" />
              <div>
                <p className={settingsStyles.noticeTitle}>
                  {devTestingAvailable ? "Dev testing API available" : "Dev testing API unavailable"}
                </p>
                <p className={settingsStyles.noticeText}>
                  {devTestingAvailable
                    ? "Seeded profiles can be activated and fixture routes can be opened from this tab."
                    : "Start the local API with dev testing enabled, then seed dm-basic, contacts-edge, or server-chat."}
                </p>
              </div>
            </div>
            <SettingRow
              description="Creates a real local dev session for the selected seeded profile."
              label="Testing profile"
              status="Dev only"
            >
              <div className={settingsStyles.controlStack}>
                <select
                  aria-label="Testing profile"
                  className={settingsStyles.select}
                  disabled={!devTestingAvailable || testingBusy !== null}
                  onChange={(event) => setSelectedTestingProfileId(event.target.value)}
                  value={selectedTestingProfile?.profile_id ?? ""}
                >
                  {!devTestingAvailable ? <option value="">No profiles available</option> : null}
                  {testingProfiles.map((profile) => (
                    <option key={profile.profile_id} value={profile.profile_id}>
                      {profile.profile_id}
                    </option>
                  ))}
                </select>
                <button
                  className={settingsStyles.primaryButton}
                  disabled={!selectedTestingProfile || testingBusy !== null}
                  onClick={() => void activateSelectedTestingProfile()}
                  type="button"
                >
                  {testingBusy === "profile" ? "Activating..." : "Activate profile"}
                </button>
              </div>
            </SettingRow>
            <SettingRow
              description={
                selectedShortcut
                  ? `${selectedShortcut.description} Seed: ${selectedShortcut.scenarioId}; profile: ${selectedShortcut.profileId}.`
                  : "Choose a seeded fixture route to open."
              }
              label="Fixture shortcut"
              status="Dev only"
            >
              <div className={settingsStyles.controlStack}>
                <select
                  aria-label="Fixture shortcut"
                  className={settingsStyles.select}
                  disabled={!devTestingAvailable || testingBusy !== null}
                  onChange={(event) => setSelectedShortcutId(event.target.value)}
                  value={selectedShortcut?.id ?? ""}
                >
                  {DEV_TESTING_SHORTCUTS.map((shortcut) => (
                    <option key={shortcut.id} value={shortcut.id}>
                      {shortcut.label}
                    </option>
                  ))}
                </select>
                <button
                  className={settingsStyles.primaryButton}
                  disabled={!selectedShortcutProfile || testingBusy !== null}
                  onClick={() => void openSelectedTestingShortcut()}
                  type="button"
                >
                  {testingBusy === "shortcut" ? "Opening..." : "Open shortcut"}
                </button>
              </div>
            </SettingRow>
            <SettingRow
              description="Known fixture scenarios available in scripts/fixtures/scenarios."
              label="Seed scenarios"
              status="Dev only"
            >
              <ReadOnlyValue>dm-basic, contacts-edge, server-chat</ReadOnlyValue>
            </SettingRow>
            {testingMessage ? (
              <div className={settingsStyles.inlineMessage}>
                <IconRoute className={settingsStyles.icon} aria-hidden="true" />
                {testingMessage}
              </div>
            ) : null}
          </SettingPanel>
        ) : null}
      </section>
    </WorkspaceShell>
  );
}
