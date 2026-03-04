export type PersonaRecord = {
  id: string;
  name: string;
  createdAt: string;
  lastSelectedAt: string;
};

const PERSONAS_KEY = "hexrelay.personas.v1";
const ACTIVE_PERSONA_KEY = "hexrelay.active-persona.v1";

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

export function readPersonas(): PersonaRecord[] {
  if (typeof window === "undefined") {
    return [];
  }

  return safeParse<PersonaRecord[]>(window.localStorage.getItem(PERSONAS_KEY), []);
}

export function readActivePersonaId(): string | null {
  if (typeof window === "undefined") {
    return null;
  }

  return window.localStorage.getItem(ACTIVE_PERSONA_KEY);
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

  return created;
}

export function switchPersona(personaId: string): PersonaRecord[] {
  const now = new Date().toISOString();
  const personas = readPersonas().map((persona) =>
    persona.id === personaId ? { ...persona, lastSelectedAt: now } : persona,
  );

  window.localStorage.setItem(PERSONAS_KEY, JSON.stringify(personas));
  window.localStorage.setItem(ACTIVE_PERSONA_KEY, personaId);

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

  return personas;
}
