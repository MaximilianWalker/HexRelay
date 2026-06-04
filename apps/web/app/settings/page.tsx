"use client";

import { useEffect, useMemo, useState, useSyncExternalStore } from "react";
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

import { Panel } from "@/components/settings/panel";
import { Button } from "@/components/settings/button";
import { Row } from "@/components/settings/row";
import { Select } from "@/components/settings/select";
import { Toggle } from "@/components/settings/toggle";
import { Value } from "@/components/settings/value";
import { MainLayout } from "@/components/layout/main";
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
import {
  EMPTY_PERSONA_SNAPSHOT,
  parsePersonaSnapshot,
  readPersonaSnapshot,
  switchPersona,
  upsertPersona,
} from "@/lib/personas";
import { getPersonaSession, setPersonaSession, subscribePersonaSession } from "@/lib/sessions";
import {
  readThemePreference,
  setThemePreference,
  subscribeThemePreference,
  type ThemePreference,
} from "@/lib/ui/theme";
import {
  readMessageAlignment,
  readMessageBubbleSize,
  readMicrophoneMuted,
  readMessageLayout,
  readNavLayout,
  readSidebarCollapsed,
  readSoundMuted,
  readTabRestoreMode,
  setMicrophoneMuted,
  setMessageAlignment,
  setMessageBubbleSize,
  setMessageLayout,
  setNavLayout,
  setSidebarCollapsed,
  setSoundMuted,
  setTabRestoreMode,
  subscribeWorkspacePreferences,
  type MessageAlignment,
  type MessageBubbleSize,
  type MessageLayout,
  type NavLayout,
  type TabRestoreMode,
} from "@/lib/workspace-preferences";
import { syncWorkspaceTabsForRestoreMode } from "@/lib/workspace-tabs";

import settingsStyles from "./styles.module.css";

const DM_POLICY_KEY = "hexrelay.settings.dm-policy";
const DM_POLICY_EVENT = "hexrelay-dm-policy-changed";
const SHOW_DEV_TESTING = process.env.NODE_ENV === "development";

type DmPolicy = DmInboundPolicy;
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
    summary: "Navigation layout, tab restore behavior, and message display preferences.",
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
    description: "Shared server fixture with Alice's pinned and unread state.",
    profileId: "alice.primary",
    href: "/servers/hexrelay-local-server",
    scenarioId: "server-chat",
  },
  {
    id: "bob-atlas-server",
    label: "Bob Atlas server",
    description: "Shared server fixture with Bob's member state.",
    profileId: "bob.primary",
    href: "/servers/hexrelay-local-server",
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

export default function SettingsPage() {
  const router = useRouter();
  const navLayout = useSyncExternalStore<NavLayout>(subscribeWorkspacePreferences, readNavLayout, () => "sidebar");
  const sidebarCollapsed = useSyncExternalStore(subscribeWorkspacePreferences, readSidebarCollapsed, () => false);
  const soundMuted = useSyncExternalStore(subscribeWorkspacePreferences, readSoundMuted, () => false);
  const microphoneMuted = useSyncExternalStore(subscribeWorkspacePreferences, readMicrophoneMuted, () => false);
  const messageLayout = useSyncExternalStore<MessageLayout>(
    subscribeWorkspacePreferences,
    readMessageLayout,
    () => "bubble-cards",
  );
  const messageBubbleSize = useSyncExternalStore<MessageBubbleSize>(
    subscribeWorkspacePreferences,
    readMessageBubbleSize,
    () => "comfortable",
  );
  const messageAlignment = useSyncExternalStore<MessageAlignment>(
    subscribeWorkspacePreferences,
    readMessageAlignment,
    () => "conversation-sides",
  );
  const tabRestoreMode = useSyncExternalStore<TabRestoreMode>(
    subscribeWorkspacePreferences,
    readTabRestoreMode,
    () => "pinned",
  );
  const dmPolicy = useSyncExternalStore<DmPolicy>(subscribeDmPolicy, readDmPolicy, () => "friends_only");
  const themePreference = useSyncExternalStore<ThemePreference>(
    subscribeThemePreference,
    readThemePreference,
    () => "system",
  );
  const personaSnapshot = useSyncExternalStore(
    subscribeWorkspacePreferences,
    readPersonaSnapshot,
    () => EMPTY_PERSONA_SNAPSHOT,
  );
  const { activePersonaId, personas } = parsePersonaSnapshot(personaSnapshot);
  const identityId = useMemo(() => activePersonaId ?? personas[0]?.id ?? null, [activePersonaId, personas]);
  const activePersona = useMemo(
    () => personas.find((persona) => persona.id === identityId) ?? personas[0] ?? null,
    [identityId, personas],
  );
  const hasSession = useSyncExternalStore(
    subscribePersonaSession,
    () => (identityId ? getPersonaSession(identityId) !== null : false),
    () => false,
  );
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

    switchPersona(personaId);
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
    <MainLayout
      activeTabId={activeSettingsTab}
      onTabChange={selectSettingsTab}
      subtitle="Profile, privacy, device, workspace, and developer settings"
      tabs={shellTabs}
      title="Settings"
    >
      <section className={settingsStyles.page}>
        {activeSettingsTab === "profile" ? (
          <Panel category={activeSettingsCategory}>
          <Row
            description="The profile used for local sessions, privacy policy updates, contacts, and fixture validation."
            label="Active profile"
            status="Live"
          >
            <Select
              aria-label="Active profile"
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
            </Select>
          </Row>
          <Row
            description="Shows whether the selected profile has a browser runtime session."
            label="Session status"
            status="Live"
          >
            <Value>{hasSession ? "Active" : "No active session"}</Value>
          </Row>
          <Row
            description="Session revoke on profile switch or removal is mandatory for the current security model."
            label="Session revoke on switch/remove"
            status="Locked"
          >
            <Toggle checked disabled label="Session revoke on switch or remove" />
          </Row>
          <Row
            description="Recovery material remains device-local; export flow still needs final product approval."
            label="Recovery phrase export"
            status="Review"
          >
            <Button disabled>
              Review flow
            </Button>
          </Row>
          </Panel>
        ) : null}

        {activeSettingsTab === "privacy" ? (
          <Panel category={activeSettingsCategory}>
          <Row
            description={policyMessage ?? "Controls who can start an inbound private-message conversation."}
            label="DM inbound policy"
            status="Live"
          >
            <Select
              aria-label="DM inbound policy"
              disabled={!hasSession || policyBusy}
              onChange={(event) => void updatePolicy(event.target.value as DmPolicy)}
              value={dmPolicy}
            >
              <option value="friends_only">Friends only</option>
              <option value="same_server">Same server</option>
              <option value="anyone">Anyone</option>
            </Select>
          </Row>
          <Row
            description="Current product behavior requires explicit approval before contacts can message."
            label="Contact request approval"
            status="Review"
          >
            <Select aria-label="Contact request approval" disabled value="manual">
              <option value="manual">Manual approval</option>
              <option value="auto">Auto-accept trusted requests</option>
            </Select>
          </Row>
          <Row
            description="Discovery is not part of the current MVP surface."
            label="Profile discoverability"
            status="Review"
          >
            <Select aria-label="Profile discoverability" disabled value="off">
              <option value="off">Off</option>
              <option value="contacts">Contacts only</option>
              <option value="server">Shared servers</option>
            </Select>
          </Row>
          </Panel>
        ) : null}

        {activeSettingsTab === "devices" ? (
          <Panel category={activeSettingsCategory}>
          <Row
            description="Runtime sessions are isolated to browser session storage per persona."
            label="Current session storage"
            status="Locked"
          >
            <Value>Session storage</Value>
          </Row>
          <Row
            description="Private keys stay client/device-only and are not uploaded to servers."
            label="Private key storage"
            status="Locked"
          >
            <Value>Device-only</Value>
          </Row>
          <Row
            description="Profile-device heartbeat exists in the API; user-facing device management is not wired yet."
            label="Device heartbeat mode"
            status="Review"
          >
            <Select aria-label="Device heartbeat mode" disabled value="runtime">
              <option value="runtime">Runtime managed</option>
              <option value="manual">Manual</option>
            </Select>
          </Row>
          <Row
            description="Future control for revoking stale device sessions from the selected profile."
            label="Revoke other devices"
            status="Review"
          >
            <Button disabled>
              Review action
            </Button>
          </Row>
          </Panel>
        ) : null}

        {activeSettingsTab === "notifications" ? (
          <Panel category={activeSettingsCategory}>
          <Row
            description="Desktop notification permission and delivery are not implemented in the web client yet."
            label="Desktop notifications"
            status="Review"
          >
            <Toggle checked={false} disabled label="Desktop notifications" />
          </Row>
          <Row
            description="Notification rules for encrypted DMs still need product copy and runtime delivery hooks."
            label="DM notifications"
            status="Review"
          >
            <Select aria-label="DM notifications" disabled value="mentions_dms">
              <option value="mentions_dms">Mentions and DMs</option>
              <option value="mentions">Mentions only</option>
              <option value="off">Off</option>
            </Select>
          </Row>
          <Row
            description="Friend/contact request notification behavior still needs approval."
            label="Contact request notifications"
            status="Review"
          >
            <Toggle checked={false} disabled label="Contact request notifications" />
          </Row>
          <Row
            description="Server-channel notification policy is future work for server-channel surfaces."
            label="Server channel notifications"
            status="Review"
          >
            <Select aria-label="Server channel notifications" disabled value="mentions">
              <option value="mentions">Mentions only</option>
              <option value="all">All messages</option>
              <option value="muted">Muted</option>
            </Select>
          </Row>
          </Panel>
        ) : null}

        {activeSettingsTab === "voice-video" ? (
          <Panel category={activeSettingsCategory}>
          <Row
            description="Mirrors the current workspace microphone quick control."
            label="Microphone muted"
            status="Live"
          >
            <Toggle checked={microphoneMuted} label="Microphone muted" onChange={setMicrophoneMuted} />
          </Row>
          <Row
            description="Mirrors the current workspace sound quick control."
            label="Sound muted"
            status="Live"
          >
            <Toggle checked={soundMuted} label="Sound muted" onChange={setSoundMuted} />
          </Row>
          <Row
            description="Voice device selection should stay local to the desktop/browser runtime."
            label="Input device"
            status="Review"
          >
            <Select aria-label="Input device" disabled value="system">
              <option value="system">System default</option>
            </Select>
          </Row>
          <Row
            description="Output routing belongs with voice readiness, not the current MVP settings behavior."
            label="Output device"
            status="Review"
          >
            <Select aria-label="Output device" disabled value="system">
              <option value="system">System default</option>
            </Select>
          </Row>
          <Row
            description="Audio processing controls should be added only with the actual voice stack."
            label="Noise suppression"
            status="Review"
          >
            <Toggle checked={false} disabled label="Noise suppression" />
          </Row>
          </Panel>
        ) : null}

        {activeSettingsTab === "appearance" ? (
          <Panel category={activeSettingsCategory}>
          <Row
            description="Controls the app color theme across all shared UI surfaces."
            label="Theme"
            status="Live"
          >
            <Select
              aria-label="Theme"
              onChange={(event) => setThemePreference(event.target.value as ThemePreference)}
              value={themePreference}
            >
              <option value="system">System</option>
              <option value="light">Light</option>
              <option value="dark">Dark</option>
            </Select>
          </Row>
          <Row
            description="Switches the main app navigation between sidebar and top bar layouts."
            label="Navigation layout"
            status="Live"
          >
            <Select
              aria-label="Navigation layout"
              onChange={(event) => setNavLayout(event.target.value as NavLayout)}
              value={navLayout}
            >
              <option value="sidebar">Sidebar</option>
              <option value="topbar">Top bar</option>
            </Select>
          </Row>
          <Row
            description="Controls whether the current sidebar/top bar chrome is collapsed."
            label="Navigation collapsed"
            status="Live"
          >
            <Toggle checked={sidebarCollapsed} label="Navigation collapsed" onChange={setSidebarCollapsed} />
          </Row>
          <Row
            description="Controls whether normal workspace tabs reopen across app sessions."
            label="Workspace tab restore"
            status="Live"
          >
            <Select
              aria-label="Workspace tab restore"
              onChange={(event) => updateTabRestoreMode(event.target.value as TabRestoreMode)}
              value={tabRestoreMode}
            >
              <option value="pinned">Pinned tabs only</option>
              <option value="all">Pinned and normal tabs</option>
            </Select>
          </Row>
          <Row
            description="Choose whether channel messages render as separated bubble cards or a continuous feed."
            label="Message layout"
            status="Live"
          >
            <Select
              aria-label="Message layout"
              onChange={(event) => setMessageLayout(event.target.value as MessageLayout)}
              value={messageLayout}
            >
              <option value="bubble-cards">Bubble cards</option>
              <option value="continuous-feed">Continuous feed</option>
            </Select>
          </Row>
          <Row
            description="Controls message padding and spacing for people who want a tighter chat view."
            label="Message bubble size"
            status="Live"
          >
            <Select
              aria-label="Message bubble size"
              onChange={(event) => setMessageBubbleSize(event.target.value as MessageBubbleSize)}
              value={messageBubbleSize}
            >
              <option value="comfortable">Comfortable</option>
              <option value="compact">Compact</option>
            </Select>
          </Row>
          <Row
            description="Controls whether your messages sit on the right while everyone else stays on the left."
            label="Message alignment"
            status="Live"
          >
            <Select
              aria-label="Message alignment"
              onChange={(event) => setMessageAlignment(event.target.value as MessageAlignment)}
              value={messageAlignment}
            >
              <option value="conversation-sides">Mine right, others left</option>
              <option value="single-column">Single column</option>
            </Select>
          </Row>
          </Panel>
        ) : null}

        {activeSettingsTab === "advanced" ? (
          <Panel category={activeSettingsCategory}>
          <Row
            description="Public API endpoint used by this web client."
            label="API base URL"
            status="Live"
          >
            <Value>{env.NEXT_PUBLIC_API_BASE_URL}</Value>
          </Row>
          <Row
            description="Realtime websocket endpoint used by this web client."
            label="Realtime URL"
            status="Live"
          >
            <Value>{env.NEXT_PUBLIC_REALTIME_WS_URL}</Value>
          </Row>
          <Row
            description="User-owned export/import is a product guardrail but not wired into the web client yet."
            label="Export profile data"
            status="Review"
          >
            <Button disabled>
              Review export
            </Button>
          </Row>
          <Row
            description="Import flow should be specified with the same portability/export model."
            label="Import profile data"
            status="Review"
          >
            <Button disabled>
              Review import
            </Button>
          </Row>
          <Row
            description="Diagnostics should be added after runtime health states are normalized."
            label="Diagnostics mode"
            status="Review"
          >
            <Toggle checked={false} disabled label="Diagnostics mode" />
          </Row>
          </Panel>
        ) : null}

        {SHOW_DEV_TESTING && activeSettingsTab === "developer" ? (
          <Panel category={activeSettingsCategory}>
            <div className={devTestingAvailable ? settingsStyles.alert : settingsStyles.warningAlert}>
              <IconFlask className={settingsStyles.icon} aria-hidden="true" />
              <div>
                <p className={settingsStyles.alertTitle}>
                  {devTestingAvailable ? "Dev testing API available" : "Dev testing API unavailable"}
                </p>
                <p className={settingsStyles.alertText}>
                  {devTestingAvailable
                    ? "Seeded profiles can be activated and fixture routes can be opened from this tab."
                    : "Start the local API with dev testing enabled, then seed dm-basic, contacts-edge, or server-chat."}
                </p>
              </div>
            </div>
            <Row
              description="Creates a real local dev session for the selected seeded profile."
              label="Testing profile"
              status="Dev only"
            >
              <div className={settingsStyles.controlStack}>
                <Select
                  aria-label="Testing profile"
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
                </Select>
                <Button
                  disabled={!selectedTestingProfile || testingBusy !== null}
                  onClick={() => void activateSelectedTestingProfile()}
                  variant="primary"
                >
                  {testingBusy === "profile" ? "Activating..." : "Activate profile"}
                </Button>
              </div>
            </Row>
            <Row
              description={
                selectedShortcut
                  ? `${selectedShortcut.description} Seed: ${selectedShortcut.scenarioId}; profile: ${selectedShortcut.profileId}.`
                  : "Choose a seeded fixture route to open."
              }
              label="Fixture shortcut"
              status="Dev only"
            >
              <div className={settingsStyles.controlStack}>
                <Select
                  aria-label="Fixture shortcut"
                  disabled={!devTestingAvailable || testingBusy !== null}
                  onChange={(event) => setSelectedShortcutId(event.target.value)}
                  value={selectedShortcut?.id ?? ""}
                >
                  {DEV_TESTING_SHORTCUTS.map((shortcut) => (
                    <option key={shortcut.id} value={shortcut.id}>
                      {shortcut.label}
                    </option>
                  ))}
                </Select>
                <Button
                  disabled={!selectedShortcutProfile || testingBusy !== null}
                  onClick={() => void openSelectedTestingShortcut()}
                  variant="primary"
                >
                  {testingBusy === "shortcut" ? "Opening..." : "Open shortcut"}
                </Button>
              </div>
            </Row>
            <Row
              description="Known fixture scenarios available in fixtures/dev-seed/scenarios."
              label="Seed scenarios"
              status="Dev only"
            >
              <Value>dm-basic, contacts-edge, server-chat</Value>
            </Row>
            {testingMessage ? (
              <div className={settingsStyles.inlineMessage}>
                <IconRoute className={settingsStyles.icon} aria-hidden="true" />
                {testingMessage}
              </div>
            ) : null}
          </Panel>
        ) : null}
      </section>
    </MainLayout>
  );
}
