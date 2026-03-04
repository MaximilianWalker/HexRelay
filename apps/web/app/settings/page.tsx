import { WorkspaceShell } from "@/components/workspace-shell";

import styles from "../surfaces.module.css";

export default function SettingsPage() {
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
            <p className={styles.meta}>Default: friends-only (user-overridable).</p>
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
        <p className={styles.state}>
          Additional settings views (export/import, observability) are tracked in later iterations.
        </p>
      </section>
    </WorkspaceShell>
  );
}
