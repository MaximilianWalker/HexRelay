"use client";

import { useMemo, useState } from "react";
import { useEffect } from "react";
import Link from "next/link";

import { WorkspaceShell } from "@/components/workspace-shell";
import { fetchServers } from "@/lib/api";

import styles from "../surfaces.module.css";

type Server = {
  id: string;
  name: string;
  unread: number;
  favorite: boolean;
  muted: boolean;
};

export default function ServersPage() {
  const [servers, setServers] = useState<Server[]>([]);
  const [search, setSearch] = useState("");
  const [favoritesOnly, setFavoritesOnly] = useState(false);
  const [unreadOnly, setUnreadOnly] = useState(false);
  const [mutedOnly, setMutedOnly] = useState(false);
  const [loading, setLoading] = useState(true);
  const [hasError, setHasError] = useState(false);

  useEffect(() => {
    let active = true;

    void fetchServers({
      search,
      favoritesOnly,
      unreadOnly,
      mutedOnly,
    }).then((result) => {
      if (!active) {
        return;
      }

      if (!result.ok) {
        setHasError(true);
        setServers([]);
        setLoading(false);
        return;
      }

      setServers(result.data.items);
      setLoading(false);
    });

    return () => {
      active = false;
    };
  }, [favoritesOnly, mutedOnly, search, unreadOnly]);

  function setFilterState(update: () => void): void {
    setLoading(true);
    setHasError(false);
    update();
  }

  const filtered = useMemo(() => {
    return servers;
  }, [servers]);

  const state = loading
    ? "loading"
    : hasError
      ? "error"
      : filtered.length === 0
        ? search.trim() || favoritesOnly || unreadOnly || mutedOnly
          ? "search_no_results"
          : "empty"
        : "ready";

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
          <button
            className={styles.pill}
            onClick={() => setFilterState(() => setFavoritesOnly((value) => !value))}
            type="button"
          >
            favorites {favoritesOnly ? "on" : "off"}
          </button>
          <button
            className={styles.pill}
            onClick={() => setFilterState(() => setUnreadOnly((value) => !value))}
            type="button"
          >
            unread {unreadOnly ? "on" : "off"}
          </button>
          <button
            className={styles.pill}
            onClick={() => setFilterState(() => setMutedOnly((value) => !value))}
            type="button"
          >
            muted {mutedOnly ? "on" : "off"}
          </button>
        </div>
        <input
          className={styles.search}
          onChange={(event) =>
            setFilterState(() => {
              setSearch(event.target.value);
            })
          }
          placeholder="Search servers"
          value={search}
        />

        {filtered.length > 0 ? (
          <div className={styles.grid}>
            {filtered.map((server) => (
              <article className={styles.card} key={server.id}>
                <p className={styles.title}>
                  <Link href={`/servers/${server.id}`}>{server.name}</Link>
                </p>
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
