"use client";

import { useEffect, useMemo, useState, useSyncExternalStore } from "react";
import { IconDeviceDesktop, IconLock, IconSettings } from "@tabler/icons-react";

import { WorkspaceShell } from "@/components/workspace-shell";
import {
  fetchDmPolicy,
  updateDmPolicy,
  type DmInboundPolicy,
} from "@/lib/api";
import { readActivePersonaId, readPersonas } from "@/lib/personas";
import { getPersonaSession } from "@/lib/sessions";
import {
  readTabRestoreMode,
  setTabRestoreMode,
  subscribeWorkspacePreferences,
  type TabRestoreMode,
} from "@/lib/workspace-preferences";
import { syncWorkspaceTabsForRestoreMode } from "@/lib/workspace-tabs";

import styles from "../surfaces.module.css";

const DM_POLICY_KEY = "hexrelay.settings.dm-policy.v1";
const DM_POLICY_EVENT = "hexrelay-dm-policy-changed";

type DmPolicy = DmInboundPolicy;

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
  const tabRestoreMode = useSyncExternalStore<TabRestoreMode>(
    subscribeWorkspacePreferences,
    readTabRestoreMode,
    () => "pinned",
  );
  const dmPolicy = useSyncExternalStore<DmPolicy>(subscribeDmPolicy, readDmPolicy, () => "friends_only");
  const personas = useMemo(() => readPersonas(), []);
  const identityId = useMemo(() => readActivePersonaId() ?? personas[0]?.id ?? null, [personas]);
  const hasSession = useMemo(() => (identityId ? getPersonaSession(identityId) !== null : false), [identityId]);
  const [policyBusy, setPolicyBusy] = useState<DmPolicy | null>(null);
  const [policyMessage, setPolicyMessage] = useState<string | null>(null);

  useEffect(() => {
    let active = true;

    const run = async (): Promise<void> => {
      if (!hasSession) {
        setPolicyMessage("Create or select a profile before changing DM policy.");
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
  }, [hasSession]);

  async function updatePolicy(next: DmPolicy): Promise<void> {
    if (!hasSession) {
      setPolicyMessage("Create or select a profile before changing DM policy.");
      return;
    }

    setPolicyBusy(next);
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
      setPolicyBusy(null);
    }
  }

  function updateTabRestoreMode(next: TabRestoreMode): void {
    syncWorkspaceTabsForRestoreMode(next);
    setTabRestoreMode(next);
  }

  return (
    <WorkspaceShell
      activeTabId="settings"
      subtitle="Policy and device-level preferences"
      tabs={[
        { id: "settings", label: "General", icon: IconSettings },
        { id: "privacy", label: "Privacy", icon: IconLock },
        { id: "devices", label: "Devices", icon: IconDeviceDesktop },
      ]}
      title="Settings"
    >
      <section>
        <div className={styles.grid}>
          <article className={styles.card}>
            <p className={styles.title}>DM inbound policy</p>
            <p className={styles.meta}>Default and current: friends-only.</p>
            <div className={styles.row}>
              <button
                aria-pressed={dmPolicy === "friends_only"}
                className={`${styles.pill} ${dmPolicy === "friends_only" ? styles.pillActive : ""}`}
                disabled={!hasSession || policyBusy !== null}
                onClick={() => void updatePolicy("friends_only")}
                type="button"
              >
                {policyBusy === "friends_only" ? "Saving..." : "Friends only"}
              </button>
              <button
                aria-pressed={dmPolicy === "same_server"}
                className={`${styles.pill} ${dmPolicy === "same_server" ? styles.pillActive : ""}`}
                disabled={!hasSession || policyBusy !== null}
                onClick={() => void updatePolicy("same_server")}
                type="button"
              >
                {policyBusy === "same_server" ? "Saving..." : "Same server"}
              </button>
              <button
                aria-pressed={dmPolicy === "anyone"}
                className={`${styles.pill} ${dmPolicy === "anyone" ? styles.pillActive : ""}`}
                disabled={!hasSession || policyBusy !== null}
                onClick={() => void updatePolicy("anyone")}
                type="button"
              >
                {policyBusy === "anyone" ? "Saving..." : "Anyone"}
              </button>
            </div>
            {policyMessage ? <p className={styles.meta}>{policyMessage}</p> : null}
          </article>
          <article className={styles.card}>
            <p className={styles.title}>Workspace tabs</p>
            <p className={styles.meta}>
              Pinned tabs always reopen. Choose whether normal tabs reopen too, like a browser.
            </p>
            <div className={styles.row}>
              <button
                aria-pressed={tabRestoreMode === "pinned"}
                className={`${styles.pill} ${tabRestoreMode === "pinned" ? styles.pillActive : ""}`}
                onClick={() => updateTabRestoreMode("pinned")}
                type="button"
              >
                Restore pinned only
              </button>
              <button
                aria-pressed={tabRestoreMode === "all"}
                className={`${styles.pill} ${tabRestoreMode === "all" ? styles.pillActive : ""}`}
                onClick={() => updateTabRestoreMode("all")}
                type="button"
              >
                Restore all tabs
              </button>
            </div>
          </article>
          <article className={styles.card}>
            <p className={styles.title}>Session hardening</p>
            <p className={styles.meta}>Persona session revoke on switch/remove enabled.</p>
          </article>
        </div>
        <p className={styles.state}>state: ready</p>
      </section>
    </WorkspaceShell>
  );
}
