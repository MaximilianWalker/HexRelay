export type PersonaRecord = {
  id: string;
  name: string;
  createdAt: string;
  lastSelectedAt: string;
};

export type PersonaSnapshot = {
  activePersonaId: string | null;
  personas: PersonaRecord[];
};

const PERSONAS_KEY = "hexrelay.personas";
const ACTIVE_PERSONA_KEY = "hexrelay.active-persona";
const UI_PREFS_EVENT = "hexrelay-ui-preferences-changed";
export const EMPTY_PERSONA_SNAPSHOT = JSON.stringify({ activePersonaId: null, personas: [] });

function notifyPersonaChange(): void {
  if (typeof window.dispatchEvent !== "function") {
    return;
  }

  window.dispatchEvent(new Event(UI_PREFS_EVENT));
}

function safeParse<T>(value: string | null, fallback: T): T {
  if (!value) {
    return fallback;
  }

  try {
    return JSON.parse(value) as T;
  } catch {
    return fallback;
  }
}

function sanitizePersonaRecord(value: unknown): PersonaRecord | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const candidate = value as Partial<Record<keyof PersonaRecord, unknown>>;
  if (
    typeof candidate.id !== "string" ||
    typeof candidate.name !== "string" ||
    typeof candidate.createdAt !== "string" ||
    typeof candidate.lastSelectedAt !== "string"
  ) {
    return null;
  }

  const id = candidate.id.trim();
  const name = candidate.name.trim();
  if (!id || !name) {
    return null;
  }

  return {
    id,
    name,
    createdAt: candidate.createdAt,
    lastSelectedAt: candidate.lastSelectedAt,
  };
}

function sanitizePersonaRecords(value: unknown): PersonaRecord[] {
  if (!Array.isArray(value)) {
    return [];
  }

  return value.flatMap((item) => {
    const record = sanitizePersonaRecord(item);
    return record ? [record] : [];
  });
}

export function readPersonas(): PersonaRecord[] {
  if (typeof window === "undefined") {
    return [];
  }

  try {
    return sanitizePersonaRecords(safeParse<unknown>(window.localStorage.getItem(PERSONAS_KEY), []));
  } catch {
    return [];
  }
}

export function readActivePersonaId(): string | null {
  if (typeof window === "undefined") {
    return null;
  }

  try {
    return window.localStorage.getItem(ACTIVE_PERSONA_KEY);
  } catch {
    return null;
  }
}

export function readPersonaSnapshot(): string {
  try {
    return JSON.stringify({
      activePersonaId: readActivePersonaId(),
      personas: readPersonas(),
    });
  } catch {
    return EMPTY_PERSONA_SNAPSHOT;
  }
}

export function parsePersonaSnapshot(value: string): PersonaSnapshot {
  try {
    const parsed = JSON.parse(value) as Partial<PersonaSnapshot>;
    return {
      activePersonaId: typeof parsed.activePersonaId === "string" ? parsed.activePersonaId : null,
      personas: sanitizePersonaRecords(parsed.personas),
    };
  } catch {
    return { activePersonaId: null, personas: [] };
  }
}

export function ensurePersona(name: string): PersonaRecord {
  const trimmedName = name.trim();
  const now = new Date().toISOString();
  const personas = readPersonas();

  const existing = personas.find(
    (persona) => persona.name.toLowerCase() === trimmedName.toLowerCase(),
  );

  if (existing) {
    const updated = {
      ...existing,
      lastSelectedAt: now,
    };
    const next = personas.map((persona) =>
      persona.id === existing.id ? updated : persona,
    );
    window.localStorage.setItem(PERSONAS_KEY, JSON.stringify(next));
    window.localStorage.setItem(ACTIVE_PERSONA_KEY, updated.id);
    notifyPersonaChange();
    return updated;
  }

  const created: PersonaRecord = {
    id: crypto.randomUUID(),
    name: trimmedName,
    createdAt: now,
    lastSelectedAt: now,
  };

  window.localStorage.setItem(PERSONAS_KEY, JSON.stringify([created, ...personas]));
  window.localStorage.setItem(ACTIVE_PERSONA_KEY, created.id);
  notifyPersonaChange();

  return created;
}

export function upsertPersona(input: { id: string; name: string }): PersonaRecord {
  const id = input.id.trim();
  const name = input.name.trim() || id;
  const now = new Date().toISOString();
  const personas = readPersonas();
  const existing = personas.find((persona) => persona.id === id);

  const nextRecord: PersonaRecord = existing
    ? {
        ...existing,
        name,
        lastSelectedAt: now,
      }
    : {
        id,
        name,
        createdAt: now,
        lastSelectedAt: now,
      };

  const next = existing
    ? personas.map((persona) => (persona.id === id ? nextRecord : persona))
    : [nextRecord, ...personas];

  window.localStorage.setItem(PERSONAS_KEY, JSON.stringify(next));
  window.localStorage.setItem(ACTIVE_PERSONA_KEY, nextRecord.id);
  notifyPersonaChange();

  return nextRecord;
}

export function switchPersona(personaId: string): PersonaRecord[] {
  const now = new Date().toISOString();
  const personas = readPersonas().map((persona) =>
    persona.id === personaId ? { ...persona, lastSelectedAt: now } : persona,
  );

  window.localStorage.setItem(PERSONAS_KEY, JSON.stringify(personas));
  window.localStorage.setItem(ACTIVE_PERSONA_KEY, personaId);
  notifyPersonaChange();

  return personas;
}

export function removePersona(personaId: string): PersonaRecord[] {
  const personas = readPersonas().filter((persona) => persona.id !== personaId);
  window.localStorage.setItem(PERSONAS_KEY, JSON.stringify(personas));

  const currentActive = readActivePersonaId();
  if (currentActive === personaId) {
    const nextActive = personas[0]?.id ?? null;
    if (nextActive) {
      window.localStorage.setItem(ACTIVE_PERSONA_KEY, nextActive);
    } else {
      window.localStorage.removeItem(ACTIVE_PERSONA_KEY);
    }
  }

  notifyPersonaChange();

  return personas;
}
