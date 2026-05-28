"use client";

import Link from "next/link";
import { useState, useSyncExternalStore } from "react";
import { IconClock, IconHome, IconInfoCircle } from "@tabler/icons-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Notice } from "@/components/ui/notice";
import { Panel } from "@/components/ui/panel";
import { TextInput } from "@/components/ui/text-input";
import { WorkspaceShell } from "@/components/workspace-shell";
import { revokeSession } from "@/lib/api";
import {
  ensurePersona,
  EMPTY_PERSONA_SNAPSHOT,
  parsePersonaSnapshot,
  readPersonaSnapshot,
  removePersona,
  switchPersona,
  type PersonaRecord,
} from "@/lib/personas";
import {
  clearPersonaPrivateKey,
  clearPersonaSession,
  getPersonaSession,
} from "@/lib/sessions";
import { trackEvent } from "@/lib/telemetry";
import { subscribeWorkspacePreferences } from "@/lib/workspace-preferences";

import styles from "./home.module.css";

export default function HomePage() {
  const [, forceRefresh] = useState(0);
  const [newPersonaName, setNewPersonaName] = useState("");
  const [actionMessage, setActionMessage] = useState<string | null>(null);
  const [busyPersonaId, setBusyPersonaId] = useState<string | null>(null);

  const personaSnapshot = useSyncExternalStore(
    subscribeWorkspacePreferences,
    readPersonaSnapshot,
    () => EMPTY_PERSONA_SNAPSHOT,
  );
  const parsedPersonaSnapshot = parsePersonaSnapshot(personaSnapshot);
  const personas: PersonaRecord[] = parsedPersonaSnapshot.personas;
  const activePersonaId = parsedPersonaSnapshot.activePersonaId ?? personas[0]?.id ?? null;

  const activePersona = personas.find((persona) => persona.id === activePersonaId) ?? null;

  function handleCreatePersona() {
    if (newPersonaName.trim().length < 2) {
      return;
    }

    const created = ensurePersona(newPersonaName);
    forceRefresh((value) => value + 1);
    setNewPersonaName("");
    trackEvent("persona_created", { personaId: created.id });
    setActionMessage(`Created persona: ${created.name}`);
    void created;
  }

  async function revokePersonaSession(personaId: string): Promise<void> {
    const existingSession = getPersonaSession(personaId);
    if (!existingSession) {
      return;
    }

    const result = await revokeSession({
      sessionId: existingSession.sessionId,
    });
    clearPersonaSession(personaId);

    if (!result.ok) {
      setActionMessage(
        `Session revoke warning for ${personaId}: ${result.code} (${result.message})`,
      );
      trackEvent("persona_session_revoke_warning", {
        personaId,
        code: result.code,
      });
      return;
    }

    trackEvent("persona_session_revoked", { personaId });
  }

  async function handleSwitchPersona(personaId: string) {
    if (activePersonaId === personaId) {
      return;
    }

    setBusyPersonaId(personaId);
    if (activePersonaId) {
      await revokePersonaSession(activePersonaId);
    }

    switchPersona(personaId);
    forceRefresh((value) => value + 1);
    trackEvent("persona_switched", { personaId });
    setActionMessage(`Switched active persona.`);
    setBusyPersonaId(null);
  }

  async function handleRemovePersona(persona: PersonaRecord) {
    setBusyPersonaId(persona.id);
    await revokePersonaSession(persona.id);
    clearPersonaPrivateKey(persona.id);
    clearPersonaSession(persona.id);
    removePersona(persona.id);
    forceRefresh((value) => value + 1);
    trackEvent("persona_removed", { personaId: persona.id });
    setActionMessage(`Removed persona: ${persona.name}`);
    setBusyPersonaId(null);
  }

  return (
    <WorkspaceShell
      activeTabId="home"
      subtitle="Recent activity and persona-scoped session control"
      tabs={[
        { id: "home", label: "Home", icon: IconHome },
        { id: "alerts", label: "Alerts", icon: IconInfoCircle },
        { id: "resume", label: "Resume", icon: IconClock },
      ]}
      title="Home"
    >
      <section>
        <p className={styles.eyebrow}>HexRelay workspace</p>
        <h1 className={styles.title}>Onboarding complete</h1>
        <p className={styles.subtitle}>
          Persona sessions are isolated locally. Switch between personas to keep
          settings and active sessions scoped.
        </p>

        <div className={styles.statusRow}>
          <Badge tone={activePersona ? "success" : "muted"}>Active persona: {activePersona?.name ?? "none selected"}</Badge>
          <Badge tone="neutral">Persona count: {personas.length}</Badge>
          <Badge tone={activePersonaId && getPersonaSession(activePersonaId) ? "success" : "muted"}>
            Session: {activePersonaId ? (getPersonaSession(activePersonaId) ? "active" : "none") : "none"}
          </Badge>
        </div>

        {actionMessage ? <Notice className={styles.message}>{actionMessage}</Notice> : null}

        <div className={styles.createRow}>
          <TextInput
            aria-label="New persona name"
            className={styles.createInput}
            onChange={(event) => setNewPersonaName(event.target.value)}
            placeholder="Create persona (e.g. Max - work)"
            value={newPersonaName}
          />
          <Button onClick={handleCreatePersona} variant="primary">
            Create persona
          </Button>
        </div>

        <div className={styles.list}>
          {personas.length === 0 ? (
            <Panel className={styles.item} padding="sm">
              <div>
                <p className={styles.itemName}>No personas yet</p>
                <p className={styles.itemMeta}>
                  Create one above or restart onboarding.
                </p>
              </div>
            </Panel>
          ) : (
            personas.map((persona) => {
              const isActive = activePersonaId === persona.id;
              return (
                <Panel className={styles.item} key={persona.id} padding="sm">
                  <div>
                    <p className={styles.itemName}>{persona.name}</p>
                    <p className={styles.itemMeta}>
                      last selected: {new Date(persona.lastSelectedAt).toLocaleString()}
                    </p>
                  </div>
                  <div className={styles.itemActions}>
                    <Button
                      disabled={busyPersonaId === persona.id}
                      onClick={() => handleSwitchPersona(persona.id)}
                      pressed={isActive}
                      size="sm"
                      variant={isActive ? "primary" : "secondary"}
                    >
                      {isActive ? "Active" : "Switch"}
                    </Button>
                    <Button
                      disabled={busyPersonaId === persona.id}
                      onClick={() => handleRemovePersona(persona)}
                      size="sm"
                      variant="danger"
                    >
                      Remove
                    </Button>
                  </div>
                </Panel>
              );
            })
          )}
        </div>

        <div className={styles.links}>
          <Link className={styles.linkGhost} href="/onboarding/identity">
            Restart onboarding
          </Link>
        </div>
      </section>
    </WorkspaceShell>
  );
}
