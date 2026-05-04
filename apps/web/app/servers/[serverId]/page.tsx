"use client";

import { useParams } from "next/navigation";
import { IconMessageCircle, IconServer2 } from "@tabler/icons-react";

import { WorkspaceShell } from "@/components/workspace-shell";

import styles from "../../surfaces.module.css";

const CHANNELS = ["# general", "# announcements", "# design", "# support", "# random"];

export default function ServerWorkspacePage() {
  const params = useParams<{ serverId: string }>();
  const serverId = params.serverId;

  return (
    <WorkspaceShell
      activeTabId="servers"
      subtitle={`Server workspace for ${serverId}`}
      tabs={[
        { id: "servers", label: "Servers Hub", icon: IconServer2 },
        { id: "workspace", label: `Workspace: ${serverId}`, icon: IconMessageCircle },
      ]}
      title="Server Workspace"
    >
      <section className={styles.channelLayout}>
        <aside className={styles.channelRail}>
          {CHANNELS.map((channel) => (
            <div className={styles.channelItem} key={channel}>
              {channel}
            </div>
          ))}
        </aside>
        <article className={styles.channelMain}>
          <p className={styles.title}>Channel content area</p>
          <p className={styles.meta}>
            state progression: loading to channel_empty to reconnecting to error (screen-state spec).
          </p>
          <p className={styles.state}>
            Next wiring: fetch channel messages and membership policy guard from API.
          </p>
        </article>
      </section>
    </WorkspaceShell>
  );
}
