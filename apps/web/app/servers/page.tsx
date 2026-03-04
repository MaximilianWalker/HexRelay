"use client";

import { useMemo, useState } from "react";

import { WorkspaceShell } from "@/components/workspace-shell";

import styles from "../surfaces.module.css";

type Server = {
  name: string;
  unread: number;
  favorite: boolean;
  muted: boolean;
};

const SERVERS: Server[] = [
  { name: "Atlas Core", unread: 2, favorite: true, muted: false },
  { name: "Relay Lab", unread: 0, favorite: false, muted: true },
  { name: "Dev Signals", unread: 5, favorite: true, muted: false },
  { name: "Ops Watch", unread: 0, favorite: false, muted: false },
];

export default function ServersPage() {
  const [search, setSearch] = useState("");
  const [favoritesOnly, setFavoritesOnly] = useState(false);
  const [unreadOnly, setUnreadOnly] = useState(false);
  const [mutedOnly, setMutedOnly] = useState(false);

  const filtered = useMemo(() => {
    return SERVERS.filter((server) => {
      if (favoritesOnly && !server.favorite) {
        return false;
      }
      if (unreadOnly && server.unread === 0) {
        return false;
      }
      if (mutedOnly && !server.muted) {
        return false;
      }
      if (search.trim() && !server.name.toLowerCase().includes(search.toLowerCase())) {
        return false;
      }
      return true;
    });
  }, [favoritesOnly, mutedOnly, search, unreadOnly]);

  const state =
    SERVERS.length === 0 ? "empty" : filtered.length === 0 ? "search_no_results" : "ready";

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
          <button className={styles.pill} onClick={() => setFavoritesOnly((v) => !v)} type="button">
            favorites {favoritesOnly ? "on" : "off"}
          </button>
          <button className={styles.pill} onClick={() => setUnreadOnly((v) => !v)} type="button">
            unread {unreadOnly ? "on" : "off"}
          </button>
          <button className={styles.pill} onClick={() => setMutedOnly((v) => !v)} type="button">
            muted {mutedOnly ? "on" : "off"}
          </button>
        </div>
        <input
          className={styles.search}
          onChange={(event) => setSearch(event.target.value)}
          placeholder="Search servers"
          value={search}
        />

        {filtered.length > 0 ? (
          <div className={styles.grid}>
            {filtered.map((server) => (
              <article className={styles.card} key={server.name}>
                <p className={styles.title}>{server.name}</p>
                <p className={styles.meta}>
                  unread {server.unread} · {server.favorite ? "favorite" : "standard"} ·{" "}
                  {server.muted ? "muted" : "audible"}
                </p>
              </article>
            ))}
          </div>
        ) : null}

        <p className={styles.state}>state: {state}</p>
      </section>
    </WorkspaceShell>
  );
}
