"use client";

import { useCallback, useEffect, useMemo, useState, useSyncExternalStore } from "react";
import { useRouter } from "next/navigation";
import { IconPlus, IconServer2 } from "@tabler/icons-react";

import { BulkActions } from "@/components/hubs/bulk-actions";
import { ItemActions } from "@/components/hubs/item-actions";
import { Surface } from "@/components/hubs/surface";
import { Toolbar } from "@/components/hubs/toolbar";
import { ServerCreateDialog } from "@/components/hubs/server-create-dialog";
import { ServerJoinDialog } from "@/components/hubs/server-join-dialog";
import { ServerLeaveDialog } from "@/components/hubs/server-leave-dialog";
import { Button } from "@/components/ui/button";
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

        <Toolbar
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
          <BulkActions
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
          <Surface
            items={visibleServers}
            layout={layout}
            noun="server"
            onOpen={(server) => router.push(serverWorkspaceRoute(server.id))}
            onToggleSelected={toggleSelected}
            renderActions={(server) => (
              <ItemActions
                busy={busy}
                destructiveLabel="Leave"
                muted={server.muted}
                onDestructive={() => setLeaveTargets([server])}
                onToggleMuted={() => void updateOneServer(server, { muted: !server.muted })}
                onTogglePinned={() => void updateOneServer(server, { pinned: !server.pinned })}
                pinned={server.pinned}
              />
            )}
            selectedIds={selectedIds}
            selecting={selecting}
          />
        ) : null}

        {activePanel === "create" ? (
          <ServerCreateDialog
            actionMessage={actionMessage}
            bootstrapCredential={bootstrapCredential}
            busy={busy}
            description={createDescription}
            manualBootstrap={manualBootstrap}
            name={createName}
            onBootstrapCredentialChange={setBootstrapCredential}
            onClose={closePanel}
            onDescriptionChange={setCreateDescription}
            onManualBootstrapChange={setManualBootstrap}
            onNameChange={setCreateName}
            onSubmit={() => void handleCreateServer()}
          />
        ) : null}

        {activePanel === "join" ? (
          <ServerJoinDialog
            actionMessage={actionMessage}
            busy={busy}
            endpoint={joinEndpoint}
            inviteLink={inviteLink}
            inviteToken={joinToken}
            onClose={closePanel}
            onEndpointChange={setJoinEndpoint}
            onInviteLinkChange={setInviteLink}
            onInviteTokenChange={setJoinToken}
            onServerIdChange={setJoinServerId}
            onShowAdvancedChange={setShowJoinAdvanced}
            onSubmit={() => void handleJoinServer()}
            serverId={joinServerId}
            showAdvanced={showJoinAdvanced}
          />
        ) : null}

        {leaveTargets.length > 0 ? (
          <ServerLeaveDialog
            busy={busy}
            deleteLocalData={deleteLocalData}
            onClose={() => setLeaveTargets([])}
            onConfirm={() => void confirmLeave()}
            onDeleteLocalDataChange={setDeleteLocalData}
            targetLabel={leaveTargets.length === 1 ? (leaveTargets[0]?.name ?? "server") : `${leaveTargets.length} servers`}
          />
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
