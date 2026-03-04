"use client";

import { useEffect, useMemo, useState } from "react";

import { WorkspaceShell } from "@/components/workspace-shell";
import {
    acceptFriendRequest,
    createFriendRequest,
    declineFriendRequest,
    fetchContacts,
    fetchFriendRequests,
} from "@/lib/api";
import { readActivePersonaId, readPersonas } from "@/lib/personas";
import { getPersonaSession } from "@/lib/sessions";

import styles from "../surfaces.module.css";

type Contact = {
  id: string;
  name: string;
  status: "online" | "offline" | "away";
  unread: number;
  favorite: boolean;
};

type FriendRequest = {
  request_id: string;
  requester_identity_id: string;
  target_identity_id: string;
  status: string;
};

export default function ContactsPage() {
  const [contacts, setContacts] = useState<Contact[]>([]);
  const [friendRequests, setFriendRequests] = useState<FriendRequest[]>([]);
  const [search, setSearch] = useState("");
  const [onlineOnly, setOnlineOnly] = useState(false);
  const [unreadOnly, setUnreadOnly] = useState(false);
  const [favoritesOnly, setFavoritesOnly] = useState(false);
  const [loading, setLoading] = useState(true);
  const [hasError, setHasError] = useState(false);

  const identityId = useMemo(() => {
    const active = readActivePersonaId();
    if (active) {
      return active;
    }
    return readPersonas()[0]?.id ?? "usr-nora-k";
  }, []);

  const sessionId = useMemo(() => getPersonaSession(identityId)?.sessionId ?? null, [identityId]);

  useEffect(() => {
    let active = true;

    if (!sessionId) {
      return () => {
        active = false;
      };
    }

    void Promise.all([
      fetchContacts({ search, onlineOnly, unreadOnly, favoritesOnly }),
      fetchFriendRequests({ identityId, sessionId }),
    ]).then(([contactsResult, requestsResult]) => {
      if (!active) {
        return;
      }

      if (!contactsResult.ok || !requestsResult.ok) {
        setContacts([]);
        setFriendRequests([]);
        setHasError(true);
        setLoading(false);
        return;
      }

      setContacts(
        contactsResult.data.items.map((item) => ({
          id: item.id,
          name: item.name,
          status: item.status as "online" | "offline" | "away",
          unread: item.unread,
          favorite: item.favorite,
        })),
      );
      setFriendRequests(requestsResult.data.items);
      setLoading(false);
    });

    return () => {
      active = false;
    };
  }, [favoritesOnly, identityId, onlineOnly, search, sessionId, unreadOnly]);

  function setFilterState(update: () => void): void {
    setLoading(true);
    setHasError(false);
    update();
  }

  async function refreshRequests(): Promise<void> {
    if (!sessionId) {
      return;
    }

    const result = await fetchFriendRequests({ identityId, sessionId });
    if (result.ok) {
      setFriendRequests(result.data.items);
    }
  }

  async function handleCreateRequest(targetIdentityId: string): Promise<void> {
    if (!sessionId) {
      return;
    }

    const result = await createFriendRequest({
      requesterIdentityId: identityId,
      targetIdentityId,
      sessionId,
    });
    if (result.ok) {
      await refreshRequests();
    }
  }

  async function handleAcceptRequest(requestId: string): Promise<void> {
    if (!sessionId) {
      return;
    }

    const result = await acceptFriendRequest({ requestId, sessionId });
    if (result.ok) {
      await refreshRequests();
    }
  }

  async function handleDeclineRequest(requestId: string): Promise<void> {
    if (!sessionId) {
      return;
    }

    const result = await declineFriendRequest({ requestId, sessionId });
    if (result.ok) {
      await refreshRequests();
    }
  }

  const inboundPending = friendRequests.filter(
    (item) => item.target_identity_id === identityId && item.status === "pending",
  );
  const outboundPending = friendRequests.filter(
    (item) => item.requester_identity_id === identityId && item.status === "pending",
  );

  const visibleContacts = sessionId ? contacts : [];

  const state = !sessionId
      ? "error"
    : loading
      ? "loading"
    : hasError
      ? "error"
      : visibleContacts.length === 0
        ? search.trim() || onlineOnly || unreadOnly || favoritesOnly
          ? "search_no_results"
          : "empty"
        : inboundPending.length > 0
          ? "friend_request_inbound"
          : outboundPending.length > 0
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
          <button
            className={styles.pill}
            onClick={() => setFilterState(() => setOnlineOnly((value) => !value))}
            type="button"
          >
            online {onlineOnly ? "on" : "off"}
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
            onClick={() => setFilterState(() => setFavoritesOnly((value) => !value))}
            type="button"
          >
            favorites {favoritesOnly ? "on" : "off"}
          </button>
        </div>

        <input
          className={styles.search}
          onChange={(event) =>
            setFilterState(() => {
              setSearch(event.target.value);
            })
          }
          placeholder="Search contacts"
          value={search}
        />

        {visibleContacts.length > 0 ? (
          <div className={styles.grid}>
            {visibleContacts.map((contact) => (
              <article className={styles.card} key={contact.id}>
                <p className={styles.title}>{contact.name}</p>
                <p className={styles.meta}>
                  {contact.status} · unread {contact.unread} · {contact.favorite ? "favorite" : "normal"}
                </p>
                <div className={styles.row}>
                  <button
                    className={styles.pill}
                    onClick={() => void handleCreateRequest(contact.id)}
                    type="button"
                  >
                    request
                  </button>
                </div>
              </article>
            ))}
          </div>
        ) : null}

        {inboundPending.length > 0 ? (
          <div className={styles.state}>
            inbound requests: {inboundPending.length}
            <div className={styles.row}>
              {inboundPending.map((request) => (
                <span key={request.request_id}>
                  <button
                    className={styles.pill}
                    onClick={() => void handleAcceptRequest(request.request_id)}
                    type="button"
                  >
                    accept
                  </button>
                  <button
                    className={styles.pill}
                    onClick={() => void handleDeclineRequest(request.request_id)}
                    type="button"
                  >
                    decline
                  </button>
                </span>
              ))}
            </div>
          </div>
        ) : null}

        <p className={styles.state}>state: {state}</p>
      </section>
    </WorkspaceShell>
  );
}
