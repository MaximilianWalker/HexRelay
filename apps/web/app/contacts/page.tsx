"use client";

import { useMemo, useState } from "react";

import { WorkspaceShell } from "@/components/workspace-shell";

import styles from "../surfaces.module.css";

type Contact = {
  name: string;
  status: "online" | "offline" | "away";
  unread: number;
  favorite: boolean;
  inboundRequest?: boolean;
  pendingRequest?: boolean;
};

const CONTACTS: Contact[] = [
  { name: "Nora K", status: "online", unread: 1, favorite: true },
  { name: "Alex R", status: "offline", unread: 0, favorite: false, pendingRequest: true },
  { name: "Mina S", status: "online", unread: 3, favorite: true },
  { name: "Jules P", status: "away", unread: 0, favorite: false, inboundRequest: true },
];

export default function ContactsPage() {
  const [search, setSearch] = useState("");
  const [onlineOnly, setOnlineOnly] = useState(false);
  const [unreadOnly, setUnreadOnly] = useState(false);
  const [favoritesOnly, setFavoritesOnly] = useState(false);

  const filtered = useMemo(() => {
    return CONTACTS.filter((contact) => {
      if (onlineOnly && contact.status !== "online") {
        return false;
      }
      if (unreadOnly && contact.unread === 0) {
        return false;
      }
      if (favoritesOnly && !contact.favorite) {
        return false;
      }
      if (search.trim() && !contact.name.toLowerCase().includes(search.toLowerCase())) {
        return false;
      }
      return true;
    });
  }, [favoritesOnly, onlineOnly, search, unreadOnly]);

  const state =
    CONTACTS.length === 0
      ? "empty"
      : filtered.length === 0
        ? "search_no_results"
        : filtered.some((item) => item.inboundRequest)
          ? "friend_request_inbound"
          : filtered.some((item) => item.pendingRequest)
            ? "friend_request_pending"
            : "ready";

  return (
    <WorkspaceShell
      activeTabId="contacts"
      subtitle="Contacts hub with friend-request and direct-message entrypoints"
      tabs={[
        { id: "contacts", label: "Contacts Hub" },
        { id: "friends", label: "Friends" },
        { id: "requests", label: "Requests" },
      ]}
      title="Contacts"
    >
      <section>
        <div className={styles.row}>
          <button className={styles.pill} onClick={() => setOnlineOnly((v) => !v)} type="button">
            online {onlineOnly ? "on" : "off"}
          </button>
          <button className={styles.pill} onClick={() => setUnreadOnly((v) => !v)} type="button">
            unread {unreadOnly ? "on" : "off"}
          </button>
          <button
            className={styles.pill}
            onClick={() => setFavoritesOnly((v) => !v)}
            type="button"
          >
            favorites {favoritesOnly ? "on" : "off"}
          </button>
        </div>
        <input
          className={styles.search}
          onChange={(event) => setSearch(event.target.value)}
          placeholder="Search contacts"
          value={search}
        />

        {filtered.length > 0 ? (
          <div className={styles.grid}>
            {filtered.map((contact) => (
              <article className={styles.card} key={contact.name}>
                <p className={styles.title}>{contact.name}</p>
                <p className={styles.meta}>
                  {contact.status} · unread {contact.unread} · {contact.favorite ? "favorite" : "normal"}
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
