"use client";

import Link from "next/link";
import { useState } from "react";

import styles from "./home.module.css";
import { WorkspaceShell } from "@/components/workspace-shell";
import { revokeSession } from "@/lib/api";
import {
  ensurePersona,
  readActivePersonaId,
  readPersonas,
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

export default function HomePage() {
  const [, forceRefresh] = useState(0);
  const [newPersonaName, setNewPersonaName] = useState("");
  const [actionMessage, setActionMessage] = useState<string | null>(null);
  const [busyPersonaId, setBusyPersonaId] = useState<string | null>(null);

  const personas: PersonaRecord[] = readPersonas();
  const activePersonaId = readActivePersonaId() ?? personas[0]?.id ?? null;

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
        { id: "home", label: "Home" },
        { id: "alerts", label: "Alerts" },
        { id: "resume", label: "Resume" },
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
          <div className={styles.badge}>
            Active persona: {activePersona?.name ?? "none selected"}
          </div>
          <div className={styles.badge}>Persona count: {personas.length}</div>
          <div className={styles.badge}>
            Session: {activePersonaId ? (getPersonaSession(activePersonaId) ? "active" : "none") : "none"}
          </div>
        </div>

        {actionMessage ? <div className={styles.message}>{actionMessage}</div> : null}

        <div className={styles.createRow}>
          <input
            className={styles.input}
            value={newPersonaName}
            onChange={(event) => setNewPersonaName(event.target.value)}
            placeholder="Create persona (e.g. Max - work)"
          />
          <button className={styles.button} type="button" onClick={handleCreatePersona}>
            Create persona
          </button>
        </div>

        <div className={styles.list}>
          {personas.length === 0 ? (
            <div className={styles.item}>
              <div>
                <p className={styles.itemName}>No personas yet</p>
                <p className={styles.itemMeta}>
                  Create one above or restart onboarding.
                </p>
              </div>
            </div>
          ) : (
            personas.map((persona) => {
              const isActive = activePersonaId === persona.id;
              return (
                <div className={styles.item} key={persona.id}>
                  <div>
                    <p className={styles.itemName}>{persona.name}</p>
                    <p className={styles.itemMeta}>
                      last selected: {new Date(persona.lastSelectedAt).toLocaleString()}
                    </p>
                  </div>
                  <div className={styles.itemActions}>
                    <button
                      className={`${styles.switchButton} ${isActive ? styles.switchActive : ""}`}
                      type="button"
                      onClick={() => handleSwitchPersona(persona.id)}
                      disabled={busyPersonaId === persona.id}
                    >
                      {isActive ? "Active" : "Switch"}
                    </button>
                    <button
                      className={styles.removeButton}
                      type="button"
                      onClick={() => handleRemovePersona(persona)}
                      disabled={busyPersonaId === persona.id}
                    >
                      Remove
                    </button>
                  </div>
                </div>
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
