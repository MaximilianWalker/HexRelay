import { readActivePersonaId } from "@/lib/personas";
import { getPersonaPrivateKey } from "@/lib/sessions";

const RECOVERY_PREFIX = "hexrelay.recovery.phrase.v1";

const WORD_BANK = [
  "amber",
  "atlas",
  "aurora",
  "banyan",
  "basalt",
  "cedar",
  "cinder",
  "clover",
  "coral",
  "cypress",
  "delta",
  "dune",
  "ember",
  "fable",
  "fern",
  "flint",
  "gale",
  "garden",
  "glacier",
  "grain",
  "harbor",
  "hazel",
  "hollow",
  "ivory",
  "jade",
  "juniper",
  "lagoon",
  "lattice",
  "linen",
  "lumen",
  "marble",
  "meadow",
  "meridian",
  "mist",
  "nebula",
  "north",
  "oak",
  "olive",
  "onyx",
  "orbit",
  "pine",
  "quartz",
  "raven",
  "ridge",
  "river",
  "sage",
  "sand",
  "shell",
  "signal",
  "silver",
  "spruce",
  "stone",
  "summit",
  "tide",
  "topaz",
  "valley",
  "velvet",
  "violet",
  "wave",
  "whisper",
  "willow",
  "winter",
  "zephyr",
];

function getPersonaIdForRecovery(): string {
  return readActivePersonaId() ?? "onboarding";
}

function storageKey(personaId: string): string {
  return `${RECOVERY_PREFIX}.${personaId}`;
}

function generatePhrase(): string[] {
  const values = new Uint16Array(12);
  crypto.getRandomValues(values);
  return Array.from(values, (value) => WORD_BANK[value % WORD_BANK.length]);
}

async function sha256Hex(value: string): Promise<string> {
  const encoded = new TextEncoder().encode(value);
  const hashBuffer = await crypto.subtle.digest("SHA-256", encoded);
  return Array.from(new Uint8Array(hashBuffer), (byte) => byte.toString(16).padStart(2, "0")).join("");
}

function phraseFromSeed(seedHex: string): string[] {
  const bytes = seedHex.match(/.{1,2}/g)?.map((chunk) => Number.parseInt(chunk, 16)) ?? [];
  if (bytes.length === 0) {
    return generatePhrase();
  }

  return Array.from({ length: 12 }, (_, index) => {
    const byte = bytes[index % bytes.length] ?? 0;
    return WORD_BANK[byte % WORD_BANK.length] ?? WORD_BANK[0]!;
  });
}

export function getOrCreateRecoveryPhrase(): string[] {
  if (typeof window === "undefined") {
    return [];
  }

  const personaId = getPersonaIdForRecovery();
  const existing = window.sessionStorage.getItem(storageKey(personaId));
  if (existing) {
    const words = existing.split(" ").filter((word) => word.trim().length > 0);
    if (words.length === 12) {
      return words;
    }
  }

  const phrase = generatePhrase();
  window.sessionStorage.setItem(storageKey(personaId), phrase.join(" "));
  return phrase;
}

export async function getOrCreateRecoveryPhraseForPersona(personaId: string): Promise<string[]> {
  if (typeof window === "undefined") {
    return [];
  }

  const existing = window.sessionStorage.getItem(storageKey(personaId));
  if (existing) {
    const words = existing.split(" ").filter((word) => word.trim().length > 0);
    if (words.length === 12) {
      return words;
    }
  }

  const privateKeyHex = await getPersonaPrivateKey(personaId);
  if (!privateKeyHex) {
    const fallback = generatePhrase();
    window.sessionStorage.setItem(storageKey(personaId), fallback.join(" "));
    return fallback;
  }

  const seed = await sha256Hex(privateKeyHex);
  const phrase = phraseFromSeed(seed);
  window.sessionStorage.setItem(storageKey(personaId), phrase.join(" "));
  return phrase;
}
