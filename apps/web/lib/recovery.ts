import { readActivePersonaId } from "@/lib/personas";

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
