"use client";

import { useCallback, useEffect, useMemo, useState, useSyncExternalStore } from "react";
import { useRouter } from "next/navigation";
import {
  IconPinnedOff,
  IconPinned,
  IconPlus,
  IconServer2,
  IconTrash,
  IconVolume,
  IconVolumeOff,
} from "@tabler/icons-react";

import { HubBulkActions } from "@/components/hubs/hub-bulk-actions";
import { HubSurface } from "@/components/hubs/hub-surface";
import { HubToolbar } from "@/components/hubs/hub-toolbar";
import { Button } from "@/components/ui/button";
import { Dialog, DialogActions } from "@/components/ui/dialog";
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
import { EMPTY_PERSONA_SNAPSHOT, parsePersonaSnapshot, readPersonaSnapshot } from "@/lib/personas";
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
  const personaSnapshot = useSyncExternalStore(
    subscribeWorkspacePreferences,
    readPersonaSnapshot,
    () => EMPTY_PERSONA_SNAPSHOT,
  );
  const { activePersonaId, personas } = parsePersonaSnapshot(personaSnapshot);
  const identityId = activePersonaId ?? personas[0]?.id ?? "usr-nora-k";

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
        return;
      }

      setServers(result.data.items);
      setHasError(false);
      setLoading(false);
    } catch {
      setHasError(true);
      setServers([]);
      setLoading(false);
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

  function closePanel(): void {
    setActivePanel(null);
    setActionMessage(null);
  }

  function toggleSelected(itemId: string): void {
    setSelectedIds((current) => {
      const next = new Set(current);
      if (next.has(itemId)) {
        next.delete(itemId);
      } else {
        next.add(itemId);
      }
      setSelecting(next.size > 0);
      return next;
    });
  }

  function clearSelection(): void {
    setSelectedIds(new Set());
    setSelecting(false);
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
      tabs={[]}
      title="Servers"
    >
      <section>
        {actionMessage && activePanel === null ? <p className={styles.state}>{actionMessage}</p> : null}

        <HubToolbar
          actions={
            <>
              <Button
                disabled={!hasSession || busy}
                icon={<IconPlus className={styles.icon} aria-hidden="true" />}
                onClick={() => setActivePanel("create")}
              >
                Create
              </Button>
              <Button
                disabled={!hasSession || busy}
                icon={<IconServer2 className={styles.icon} aria-hidden="true" />}
                onClick={() => setActivePanel("join")}
              >
                Join
              </Button>
            </>
          }
          layout={layout}
          mutedOnly={mutedOnly}
          onLayoutChange={(nextLayout) => setHubLayout("servers", nextLayout)}
          onMutedChange={() => setFilterState(() => setMutedOnly((value) => !value))}
          onPinnedChange={() => setFilterState(() => setPinnedOnly((value) => !value))}
          onSearchChange={(value) =>
            setFilterState(() => {
              setSearch(value);
            })
          }
          onUnreadChange={() => setFilterState(() => setUnreadOnly((value) => !value))}
          pinnedOnly={pinnedOnly}
          search={search}
          searchLabel="Search servers"
          unreadOnly={unreadOnly}
        />

        {selecting ? (
          <HubBulkActions
            busy={busy}
            destructiveLabel="Leave"
            onDestructive={() => setLeaveTargets(selectedServers())}
            onDone={clearSelection}
            onMute={() => void updateSelectedServers("mute")}
            onPin={() => void updateSelectedServers("pin")}
            onUnmute={() => void updateSelectedServers("unmute")}
            onUnpin={() => void updateSelectedServers("unpin")}
            selectedCount={selectedCount}
          />
        ) : null}

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

        {activePanel === "create" ? (
          <Dialog
            description="Create a local test server and choose whether to provide the bootstrap credential yourself."
            onClose={closePanel}
            title="Create server"
          >
            <form
              className={styles.dialogForm}
              onSubmit={(event) => {
                event.preventDefault();
                void handleCreateServer();
              }}
            >
              <label className={styles.dialogField}>
                Server name
                <input
                  autoComplete="off"
                  className={styles.search}
                  data-autofocus
                  onChange={(event) => setCreateName(event.target.value)}
                  placeholder="Atlas Team"
                  value={createName}
                />
              </label>
              <label className={styles.dialogField}>
                Description
                <input
                  autoComplete="off"
                  className={styles.search}
                  onChange={(event) => setCreateDescription(event.target.value)}
                  placeholder="Shared workspace for a team or community"
                  value={createDescription}
                />
              </label>
              <label className={styles.checkboxRow}>
                <input
                  checked={manualBootstrap}
                  onChange={(event) => setManualBootstrap(event.target.checked)}
                  type="checkbox"
                />
                Supply bootstrap credential manually
              </label>
              {manualBootstrap ? (
                <label className={styles.dialogField}>
                  Bootstrap credential
                  <input
                    autoComplete="off"
                    className={styles.search}
                    onChange={(event) => setBootstrapCredential(event.target.value)}
                    placeholder="Credential"
                    value={bootstrapCredential}
                  />
                </label>
              ) : null}
              {actionMessage ? <p className={styles.dialogMessage}>{actionMessage}</p> : null}
              <DialogActions>
                <button className={styles.pill} disabled={busy} onClick={closePanel} type="button">
                  Cancel
                </button>
                <button className={`${styles.pill} ${styles.primaryPill}`} disabled={busy} type="submit">
                  Create server
                </button>
              </DialogActions>
            </form>
          </Dialog>
        ) : null}

        {activePanel === "join" ? (
          <Dialog
            description="Paste an invite link, or use advanced fields when you have a server endpoint and invite token."
            onClose={closePanel}
            title="Join server"
          >
            <form
              className={styles.dialogForm}
              onSubmit={(event) => {
                event.preventDefault();
                void handleJoinServer();
              }}
            >
              <label className={styles.dialogField}>
                Invite link
                <input
                  autoComplete="off"
                  className={styles.search}
                  data-autofocus
                  onChange={(event) => setInviteLink(event.target.value)}
                  placeholder="hexrelay://invite/..."
                  value={inviteLink}
                />
              </label>
              <label className={styles.checkboxRow}>
                <input
                  checked={showJoinAdvanced}
                  onChange={(event) => setShowJoinAdvanced(event.target.checked)}
                  type="checkbox"
                />
                Show advanced fields
              </label>
              {showJoinAdvanced ? (
                <>
                  <label className={styles.dialogField}>
                    Endpoint
                    <input className={styles.search} onChange={(event) => setJoinEndpoint(event.target.value)} placeholder="https://server.example" value={joinEndpoint} />
                  </label>
                  <label className={styles.dialogField}>
                    Server id
                    <input className={styles.search} onChange={(event) => setJoinServerId(event.target.value)} placeholder="srv_..." value={joinServerId} />
                  </label>
                  <label className={styles.dialogField}>
                    Invite token
                    <input className={styles.search} onChange={(event) => setJoinToken(event.target.value)} placeholder="Invite token" value={joinToken} />
                  </label>
                </>
              ) : null}
              {actionMessage ? <p className={styles.dialogMessage}>{actionMessage}</p> : null}
              <DialogActions>
                <button className={styles.pill} disabled={busy} onClick={closePanel} type="button">
                  Cancel
                </button>
                <button className={`${styles.pill} ${styles.primaryPill}`} disabled={busy} type="submit">
                  Join server
                </button>
              </DialogActions>
            </form>
          </Dialog>
        ) : null}

        {leaveTargets.length > 0 ? (
          <Dialog
            description="Leaving removes the server from this hub and closes related workspace tabs."
            onClose={() => setLeaveTargets([])}
            title={`Leave ${leaveTargets.length === 1 ? leaveTargets[0]?.name : `${leaveTargets.length} servers`}?`}
          >
            <div className={styles.dialogStack}>
              <label className={styles.checkboxRow}>
                <input checked={deleteLocalData} onChange={(event) => setDeleteLocalData(event.target.checked)} type="checkbox" />{" "}
                Delete local data for this server
              </label>
              <DialogActions>
                <button className={styles.pill} disabled={busy} onClick={() => setLeaveTargets([])} type="button">
                  Cancel
                </button>
                <button className={`${styles.pill} ${styles.dangerButton}`} disabled={busy} onClick={() => void confirmLeave()} type="button">
                  Leave server
                </button>
              </DialogActions>
            </div>
          </Dialog>
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
