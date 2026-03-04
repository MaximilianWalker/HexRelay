"use client";

import { useState } from "react";

import { WorkspaceShell } from "@/components/workspace-shell";

import styles from "../surfaces.module.css";

const DM_POLICY_KEY = "hexrelay.settings.dm-policy.v1";

export default function SettingsPage() {
  const [dmPolicy, setDmPolicy] = useState<"friends_only" | "same_server" | "anyone">(() => {
    if (typeof window === "undefined") {
      return "friends_only";
    }

    const stored = window.localStorage.getItem(DM_POLICY_KEY);
    return stored === "same_server" || stored === "anyone" ? stored : "friends_only";
  });

  function updatePolicy(next: "friends_only" | "same_server" | "anyone"): void {
    setDmPolicy(next);
    window.localStorage.setItem(DM_POLICY_KEY, next);
  }

  return (
    <WorkspaceShell
      activeTabId="settings"
      subtitle="Policy and device-level preferences"
      tabs={[
        { id: "settings", label: "General" },
        { id: "privacy", label: "Privacy" },
        { id: "devices", label: "Devices" },
      ]}
      title="Settings"
    >
      <section>
        <div className={styles.grid}>
          <article className={styles.card}>
            <p className={styles.title}>DM inbound policy</p>
            <p className={styles.meta}>Default and current: friends-only.</p>
            <div className={styles.row}>
              <button className={styles.pill} onClick={() => updatePolicy("friends_only")} type="button">
                friends_only {dmPolicy === "friends_only" ? "active" : ""}
              </button>
              <button className={styles.pill} onClick={() => updatePolicy("same_server")} type="button">
                same_server {dmPolicy === "same_server" ? "active" : ""}
              </button>
              <button className={styles.pill} onClick={() => updatePolicy("anyone")} type="button">
                anyone {dmPolicy === "anyone" ? "active" : ""}
              </button>
            </div>
          </article>
          <article className={styles.card}>
            <p className={styles.title}>Sidebar mode preference</p>
            <p className={styles.meta}>Persisted per device (expanded/collapsed).</p>
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
