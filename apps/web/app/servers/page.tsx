"use client";

import { useCallback, useEffect, useMemo, useState, useSyncExternalStore } from "react";
import { useRouter } from "next/navigation";
import {
  IconLayoutGrid,
  IconList,
  IconMessageCircle,
  IconPinned,
  IconPinnedOff,
  IconPlus,
  IconServer2,
  IconTrash,
  IconVolume,
  IconVolumeOff,
} from "@tabler/icons-react";

import { HubSurface } from "@/components/hub-surface";
import { WorkspaceShell } from "@/components/workspace-shell";
import {
  createServer,
  fetchServers,
  joinServer,
  leaveServer,
  updateServerPreferences,
  type ServerSummary,
} from "@/lib/api";
import type { HubLayout } from "@/lib/hub-state";
import { serverWorkspaceRoute } from "@/lib/navigation-routes";
import { readActivePersonaId, readPersonas } from "@/lib/personas";
import { getPersonaSession } from "@/lib/sessions";
import { readHubLayout, setHubLayout, subscribeWorkspacePreferences } from "@/lib/workspace-preferences";
import { closeWorkspaceTabsForServer } from "@/lib/workspace-tabs";

import styles from "../surfaces.module.css";

type Panel = "create" | "join" | null;

export default function ServersPage() {
  const router = useRouter();
  const [servers, setServers] = useState<ServerSummary[]>([]);
  const [search, setSearch] = useState("");
  const [pinnedOnly, setPinnedOnly] = useState(false);
  const [unreadOnly, setUnreadOnly] = useState(false);
  const [mutedOnly, setMutedOnly] = useState(false);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [selecting, setSelecting] = useState(false);
  const [activePanel, setActivePanel] = useState<Panel>(null);
  const [createName, setCreateName] = useState("");
  const [createDescription, setCreateDescription] = useState("");
  const [manualBootstrap, setManualBootstrap] = useState(false);
  const [bootstrapCredential, setBootstrapCredential] = useState("");
  const [inviteLink, setInviteLink] = useState("");
  const [joinEndpoint, setJoinEndpoint] = useState("");
  const [joinServerId, setJoinServerId] = useState("");
  const [joinToken, setJoinToken] = useState("");
  const [showJoinAdvanced, setShowJoinAdvanced] = useState(false);
  const [leaveTargets, setLeaveTargets] = useState<ServerSummary[]>([]);
  const [deleteLocalData, setDeleteLocalData] = useState(true);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [hasError, setHasError] = useState(false);
  const [actionMessage, setActionMessage] = useState<string | null>(null);
  const layout = useSyncExternalStore<HubLayout>(
    subscribeWorkspacePreferences,
    () => readHubLayout("servers"),
    () => "cards",
  );

  const identityId = useMemo(() => {
    const active = readActivePersonaId();
    if (active) {
      return active;
    }

    return readPersonas()[0]?.id ?? "usr-nora-k";
  }, []);

  const hasSession = useMemo(() => Boolean(getPersonaSession(identityId)), [identityId]);

  const refreshServers = useCallback(async (): Promise<void> => {
    if (!hasSession) {
      setServers([]);
      setLoading(false);
      return;
    }

    try {
      const result = await fetchServers({
        search,
        pinnedOnly,
        unreadOnly,
        mutedOnly,
      });

      if (!result.ok) {
        setHasError(true);
        setServers([]);
        setLoading(false);
        setActionMessage("Could not load servers. Try again in a moment.");
        return;
      }

      setServers(result.data.items);
      setHasError(false);
      setLoading(false);
    } catch {
      setHasError(true);
      setServers([]);
      setLoading(false);
      setActionMessage("Could not reach servers. Check your connection and try again.");
    }
  }, [hasSession, mutedOnly, pinnedOnly, search, unreadOnly]);

  useEffect(() => {
    let active = true;

    queueMicrotask(() => {
      if (!active) {
        return;
      }

      void refreshServers();
    });

    return () => {
      active = false;
    };
  }, [refreshServers]);

  function setFilterState(update: () => void): void {
    setLoading(true);
    setHasError(false);
    update();
  }

  function toggleSelected(itemId: string): void {
    setSelecting(true);
    setSelectedIds((current) => {
      const next = new Set(current);
      if (next.has(itemId)) {
        next.delete(itemId);
      } else {
        next.add(itemId);
      }
      return next;
    });
  }

  function selectedServers(): ServerSummary[] {
    return servers.filter((server) => selectedIds.has(server.id));
  }

  async function updateOneServer(server: ServerSummary, update: { pinned?: boolean; muted?: boolean }): Promise<void> {
    setBusy(true);
    setActionMessage(null);
    const result = await updateServerPreferences({ serverId: server.id, ...update });
    setBusy(false);

    if (!result.ok) {
      setActionMessage(formatApiError(result.code, result.message));
      return;
    }

    setServers((items) => items.map((item) => (item.id === server.id ? result.data.item : item)));
  }

  async function updateSelectedServers(update: "pin" | "unpin" | "mute" | "unmute"): Promise<void> {
    const targets = selectedServers();
    if (targets.length === 0) {
      return;
    }

    setBusy(true);
    setActionMessage(null);
    for (const server of targets) {
      const result = await updateServerPreferences({
        serverId: server.id,
        pinned: update === "pin" ? true : update === "unpin" ? false : undefined,
        muted: update === "mute" ? true : update === "unmute" ? false : undefined,
      });
      if (!result.ok) {
        setActionMessage(formatApiError(result.code, result.message));
        setBusy(false);
        return;
      }
      setServers((items) => items.map((item) => (item.id === server.id ? result.data.item : item)));
    }
    setBusy(false);
  }

  async function handleCreateServer(): Promise<void> {
    if (!createName.trim()) {
      setActionMessage("Enter a server name first.");
      return;
    }

    setBusy(true);
    setActionMessage(null);
    const result = await createServer({
      name: createName,
      description: createDescription,
      bootstrapCredential: manualBootstrap ? bootstrapCredential : undefined,
    });
    setBusy(false);

    if (!result.ok) {
      setActionMessage(formatApiError(result.code, result.message));
      return;
    }

    setCreateName("");
    setCreateDescription("");
    setBootstrapCredential("");
    setActivePanel(null);
    setActionMessage(`Server created. Bootstrap credential: ${result.data.bootstrap_credential}`);
    await refreshServers();
  }

  async function handleJoinServer(): Promise<void> {
    if (!inviteLink.trim() && !joinToken.trim()) {
      setActionMessage("Enter an invite link or invite token first.");
      return;
    }

    setBusy(true);
    setActionMessage(null);
    const result = await joinServer({
      inviteLink,
      endpoint: showJoinAdvanced ? joinEndpoint : undefined,
      serverId: showJoinAdvanced ? joinServerId : undefined,
      inviteToken: showJoinAdvanced ? joinToken : undefined,
    });
    setBusy(false);

    if (!result.ok) {
      setActionMessage(formatApiError(result.code, result.message));
      return;
    }

    setInviteLink("");
    setJoinEndpoint("");
    setJoinServerId("");
    setJoinToken("");
    setActivePanel(null);
    setActionMessage("Server joined.");
    await refreshServers();
  }

  async function confirmLeave(): Promise<void> {
    setBusy(true);
    setActionMessage(null);
    for (const server of leaveTargets) {
      const result = await leaveServer({ serverId: server.id, deleteLocalData });
      if (!result.ok) {
        setActionMessage(formatApiError(result.code, result.message));
        setBusy(false);
        return;
      }
      closeWorkspaceTabsForServer(server.id);
      setServers((items) => items.filter((item) => item.id !== server.id));
    }
    setSelectedIds(new Set());
    setSelecting(false);
    setLeaveTargets([]);
    setBusy(false);
  }

  const visibleServers = hasSession ? servers : [];
  const pageState = !hasSession
    ? "error"
    : loading
      ? "loading"
      : hasError
        ? "error"
        : visibleServers.length === 0
          ? search.trim() || pinnedOnly || unreadOnly || mutedOnly
            ? "search_no_results"
            : "empty"
          : "ready";

  const selectedCount = selectedIds.size;

  return (
    <WorkspaceShell
      activeTabId="servers"
      subtitle="Global servers hub with shared card and list controls"
      tabs={[
        { id: "servers", label: "Servers Hub", icon: IconServer2 },
        { id: "pinned", label: "Pinned", icon: IconPinned },
        { id: "unread", label: "Unread", icon: IconMessageCircle },
      ]}
      tabActions={
        <div className={styles.row}>
          <button className={styles.pill} disabled={!hasSession || busy} onClick={() => setActivePanel("create")} type="button">
            <IconPlus className={styles.icon} aria-hidden="true" />
            Create
          </button>
          <button className={styles.pill} disabled={!hasSession || busy} onClick={() => setActivePanel("join")} type="button">
            <IconServer2 className={styles.icon} aria-hidden="true" />
            Join
          </button>
        </div>
      }
      title="Servers"
    >
      <section>
        {activePanel === "create" ? (
          <section className={styles.state} aria-label="Create server">
            <p className={styles.title}>Create server</p>
            <input
              className={styles.search}
              onChange={(event) => setCreateName(event.target.value)}
              placeholder="Server name"
              value={createName}
            />
            <input
              className={styles.search}
              onChange={(event) => setCreateDescription(event.target.value)}
              placeholder="Description"
              value={createDescription}
            />
            <label className={styles.meta}>
              <input
                checked={manualBootstrap}
                onChange={(event) => setManualBootstrap(event.target.checked)}
                type="checkbox"
              />{" "}
              Supply bootstrap credential manually
            </label>
            {manualBootstrap ? (
              <input
                className={styles.search}
                onChange={(event) => setBootstrapCredential(event.target.value)}
                placeholder="Bootstrap credential"
                value={bootstrapCredential}
              />
            ) : null}
            <div className={styles.row}>
              <button className={styles.pill} disabled={busy} onClick={() => void handleCreateServer()} type="button">
                Create server
              </button>
              <button className={styles.pill} onClick={() => setActivePanel(null)} type="button">
                Close
              </button>
            </div>
          </section>
        ) : null}

        {activePanel === "join" ? (
          <section className={styles.state} aria-label="Join server">
            <p className={styles.title}>Join server</p>
            <input
              className={styles.search}
              onChange={(event) => setInviteLink(event.target.value)}
              placeholder="Invite link"
              value={inviteLink}
            />
            <label className={styles.meta}>
              <input
                checked={showJoinAdvanced}
                onChange={(event) => setShowJoinAdvanced(event.target.checked)}
                type="checkbox"
              />{" "}
              Show advanced fields
            </label>
            {showJoinAdvanced ? (
              <>
                <input className={styles.search} onChange={(event) => setJoinEndpoint(event.target.value)} placeholder="Endpoint" value={joinEndpoint} />
                <input className={styles.search} onChange={(event) => setJoinServerId(event.target.value)} placeholder="Server id" value={joinServerId} />
                <input className={styles.search} onChange={(event) => setJoinToken(event.target.value)} placeholder="Invite token" value={joinToken} />
              </>
            ) : null}
            <div className={styles.row}>
              <button className={styles.pill} disabled={busy} onClick={() => void handleJoinServer()} type="button">
                Join server
              </button>
              <button className={styles.pill} onClick={() => setActivePanel(null)} type="button">
                Close
              </button>
            </div>
          </section>
        ) : null}

        {actionMessage ? <p className={styles.state}>{actionMessage}</p> : null}

        <div className={styles.row}>
          <button
            aria-pressed={pinnedOnly}
            className={`${styles.pill} ${pinnedOnly ? styles.pillActive : ""}`}
            onClick={() => setFilterState(() => setPinnedOnly((value) => !value))}
            type="button"
          >
            <IconPinned className={styles.icon} aria-hidden="true" />
            Pinned
          </button>
          <button
            aria-pressed={unreadOnly}
            className={`${styles.pill} ${unreadOnly ? styles.pillActive : ""}`}
            onClick={() => setFilterState(() => setUnreadOnly((value) => !value))}
            type="button"
          >
            <IconMessageCircle className={styles.icon} aria-hidden="true" />
            Unread
          </button>
          <button
            aria-pressed={mutedOnly}
            className={`${styles.pill} ${mutedOnly ? styles.pillActive : ""}`}
            onClick={() => setFilterState(() => setMutedOnly((value) => !value))}
            type="button"
          >
            <IconVolumeOff className={styles.icon} aria-hidden="true" />
            Muted
          </button>
          <button className={styles.pill} onClick={() => setHubLayout("servers", layout === "cards" ? "list" : "cards")} type="button">
            {layout === "cards" ? <IconList className={styles.icon} aria-hidden="true" /> : <IconLayoutGrid className={styles.icon} aria-hidden="true" />}
            {layout === "cards" ? "List" : "Cards"}
          </button>
          <button
            aria-pressed={selecting}
            className={`${styles.pill} ${selecting ? styles.pillActive : ""}`}
            onClick={() => {
              setSelecting((value) => !value);
              setSelectedIds(new Set());
            }}
            type="button"
          >
            Select{selectedCount > 0 ? ` (${selectedCount})` : ""}
          </button>
        </div>

        {selecting ? (
          <div className={styles.row}>
            <button className={styles.pill} disabled={busy || selectedCount === 0} onClick={() => void updateSelectedServers("pin")} type="button">
              <IconPinned className={styles.icon} aria-hidden="true" />
              Pin
            </button>
            <button className={styles.pill} disabled={busy || selectedCount === 0} onClick={() => void updateSelectedServers("unpin")} type="button">
              <IconPinnedOff className={styles.icon} aria-hidden="true" />
              Unpin
            </button>
            <button className={styles.pill} disabled={busy || selectedCount === 0} onClick={() => void updateSelectedServers("mute")} type="button">
              <IconVolumeOff className={styles.icon} aria-hidden="true" />
              Mute
            </button>
            <button className={styles.pill} disabled={busy || selectedCount === 0} onClick={() => void updateSelectedServers("unmute")} type="button">
              <IconVolume className={styles.icon} aria-hidden="true" />
              Unmute
            </button>
            <button className={`${styles.pill} ${styles.dangerButton}`} disabled={busy || selectedCount === 0} onClick={() => setLeaveTargets(selectedServers())} type="button">
              <IconTrash className={styles.icon} aria-hidden="true" />
              Leave
            </button>
          </div>
        ) : null}

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

        {pageState === "loading" ? <p className={styles.state}>Loading servers...</p> : null}
        {pageState === "error" ? (
          <p className={styles.state}>
            {hasSession ? "Could not load servers. Try again in a moment." : "Create or select a profile before managing servers."}
          </p>
        ) : null}
        {pageState === "search_no_results" ? <p className={styles.state}>No servers match your search.</p> : null}
        {pageState === "empty" ? <p className={styles.state}>No servers yet. Join or create a server to get started.</p> : null}

        {visibleServers.length > 0 ? (
          <HubSurface
            items={visibleServers}
            layout={layout}
            noun="server"
            onOpen={(server) => router.push(serverWorkspaceRoute(server.id))}
            onToggleSelected={toggleSelected}
            renderActions={(server) => (
              <>
                <button className={styles.pill} disabled={busy} onClick={() => void updateOneServer(server, { pinned: !server.pinned })} type="button">
                  {server.pinned ? <IconPinnedOff className={styles.icon} aria-hidden="true" /> : <IconPinned className={styles.icon} aria-hidden="true" />}
                  {server.pinned ? "Unpin" : "Pin"}
                </button>
                <button className={styles.pill} disabled={busy} onClick={() => void updateOneServer(server, { muted: !server.muted })} type="button">
                  {server.muted ? <IconVolume className={styles.icon} aria-hidden="true" /> : <IconVolumeOff className={styles.icon} aria-hidden="true" />}
                  {server.muted ? "Unmute" : "Mute"}
                </button>
                <button className={`${styles.pill} ${styles.dangerButton}`} disabled={busy} onClick={() => setLeaveTargets([server])} type="button">
                  <IconTrash className={styles.icon} aria-hidden="true" />
                  Leave
                </button>
              </>
            )}
            selectedIds={selectedIds}
            selecting={selecting}
          />
        ) : null}

        {leaveTargets.length > 0 ? (
          <div className={styles.dialogBackdrop} role="presentation">
            <section aria-label="Leave server confirmation" className={styles.dialog}>
              <p className={styles.title}>Leave {leaveTargets.length === 1 ? leaveTargets[0]?.name : `${leaveTargets.length} servers`}?</p>
              <p className={styles.meta}>Leaving removes the server from this hub and closes related workspace tabs.</p>
              <label className={styles.meta}>
                <input checked={deleteLocalData} onChange={(event) => setDeleteLocalData(event.target.checked)} type="checkbox" />{" "}
                Delete local data for this server
              </label>
              <div className={styles.row}>
                <button className={`${styles.pill} ${styles.dangerButton}`} disabled={busy} onClick={() => void confirmLeave()} type="button">
                  Leave server
                </button>
                <button className={styles.pill} disabled={busy} onClick={() => setLeaveTargets([])} type="button">
                  Cancel
                </button>
              </div>
            </section>
          </div>
        ) : null}
      </section>
    </WorkspaceShell>
  );
}

function formatApiError(code: string, message: string): string {
  const normalized = `${code} ${message}`.toLowerCase();
  if (normalized.includes("invite")) {
    return "That invite could not be used.";
  }
  if (normalized.includes("session")) {
    return "Your session is not ready. Try signing in again.";
  }
  if (normalized.includes("storage") || normalized.includes("unavailable")) {
    return "Could not reach the server runtime. Try again in a moment.";
  }

  return message || "Something went wrong. Try again in a moment.";
}
