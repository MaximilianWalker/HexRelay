import { WorkspaceShell } from "@/components/workspace-shell";

import styles from "../surfaces.module.css";

const SERVERS = [
  { name: "Atlas Core", unread: 2, mode: "favorite" },
  { name: "Relay Lab", unread: 0, mode: "muted" },
  { name: "Dev Signals", unread: 5, mode: "active" },
  { name: "Ops Watch", unread: 0, mode: "active" },
];

export default function ServersPage() {
  return (
    <WorkspaceShell
      activeTabId="servers"
      subtitle="Global servers hub with searchable cards and filters"
      tabs={[
        { id: "servers", label: "Servers Hub" },
        { id: "favorites", label: "Pinned" },
        { id: "unread", label: "Unread" },
      ]}
      title="Servers"
    >
      <section>
        <div className={styles.row}>
          <span className={styles.pill}>filter: favorites</span>
          <span className={styles.pill}>filter: unread</span>
          <span className={styles.pill}>filter: muted</span>
        </div>
        <input className={styles.search} placeholder="Search servers" readOnly value="" />
        <div className={styles.grid}>
          {SERVERS.map((server) => (
            <article className={styles.card} key={server.name}>
              <p className={styles.title}>{server.name}</p>
              <p className={styles.meta}>
                unread {server.unread} · {server.mode}
              </p>
            </article>
          ))}
        </div>
        <p className={styles.state}>
          Empty-state contract: include Join server and Create server actions when list is empty.
        </p>
      </section>
    </WorkspaceShell>
  );
}
