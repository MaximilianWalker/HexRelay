import { secureGetItem, secureRemoveItem, secureSetItem } from "@/lib/secure-store";

const SESSION_PREFIX = "hexrelay.session.v1";
const SESSION_TOKEN_PREFIX = "hexrelay.session.token.v1";
const PRIVATE_KEY_PREFIX = "hexrelay.identity.private.v1";
const MASTER_KEY_STORAGE = "hexrelay.identity.master-key.v1";

function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  bytes.forEach((value) => {
    binary += String.fromCharCode(value);
  });
  return btoa(binary);
}

function base64ToBytes(value: string): Uint8Array {
  const decoded = atob(value);
  const output = new Uint8Array(decoded.length);
  for (let index = 0; index < decoded.length; index += 1) {
    output[index] = decoded.charCodeAt(index);
  }
  return output;
}

async function getOrCreateMasterKeyMaterial(): Promise<Uint8Array> {
  const existing = await secureGetItem(MASTER_KEY_STORAGE);
  if (existing) {
    return Uint8Array.from(base64ToBytes(existing));
  }

  const material = new Uint8Array(32);
  crypto.getRandomValues(material);
  await secureSetItem(MASTER_KEY_STORAGE, bytesToBase64(material));
  return Uint8Array.from(material);
}

async function derivePersonaAesKey(personaId: string): Promise<CryptoKey> {
  const masterMaterial = await getOrCreateMasterKeyMaterial();
  const rawMaster = masterMaterial.buffer.slice(0) as ArrayBuffer;
  const keyMaterial = await crypto.subtle.importKey(
    "raw",
    rawMaster,
    "PBKDF2",
    false,
    ["deriveKey"],
  );

  return crypto.subtle.deriveKey(
    {
      name: "PBKDF2",
      salt: new TextEncoder().encode(`persona:${personaId}`),
      iterations: 120_000,
      hash: "SHA-256",
    },
    keyMaterial,
    {
      name: "AES-GCM",
      length: 256,
    },
    false,
    ["encrypt", "decrypt"],
  );
}

async function encryptText(personaId: string, plainText: string): Promise<string> {
  const key = await derivePersonaAesKey(personaId);
  const ivRaw = new Uint8Array(12);
  crypto.getRandomValues(ivRaw);
  const iv = Uint8Array.from(ivRaw);
  const encoded = new TextEncoder().encode(plainText);
  const cipherBuffer = await crypto.subtle.encrypt(
    {
      name: "AES-GCM",
      iv,
    },
    key,
    encoded,
  );

  return `${bytesToBase64(iv)}.${bytesToBase64(new Uint8Array(cipherBuffer))}`;
}

async function decryptText(personaId: string, cipherText: string): Promise<string> {
  const [ivPart, dataPart] = cipherText.split(".");
  if (!ivPart || !dataPart) {
    throw new Error("Invalid encrypted payload");
  }

  const key = await derivePersonaAesKey(personaId);
  const iv = Uint8Array.from(base64ToBytes(ivPart));
  const data = Uint8Array.from(base64ToBytes(dataPart));
  const plainBuffer = await crypto.subtle.decrypt(
    {
      name: "AES-GCM",
      iv,
    },
    key,
    data,
  );

  return new TextDecoder().decode(plainBuffer);
}

export function setPersonaSession(
  personaId: string,
  value: { sessionId: string; accessToken: string; expiresAt: string },
): void {
  if (typeof window === "undefined") {
    return;
  }

  window.localStorage.setItem(
    `${SESSION_PREFIX}.${personaId}`,
    JSON.stringify({
      sessionId: value.sessionId,
      expiresAt: value.expiresAt,
      updatedAt: new Date().toISOString(),
    }),
  );

  window.sessionStorage.setItem(`${SESSION_TOKEN_PREFIX}.${personaId}`, value.accessToken);
}

export function getPersonaSession(
  personaId: string,
): { sessionId: string; accessToken: string; expiresAt: string } | null {
  if (typeof window === "undefined") {
    return null;
  }

  const raw = window.localStorage.getItem(`${SESSION_PREFIX}.${personaId}`);
  if (!raw) {
    return null;
  }

  try {
    const parsed = JSON.parse(raw) as {
      sessionId?: string;
      accessToken?: string;
      expiresAt?: string;
      updatedAt?: string;
    };
    if (!parsed.sessionId || !parsed.expiresAt) {
      return null;
    }

    let accessToken = window.sessionStorage.getItem(`${SESSION_TOKEN_PREFIX}.${personaId}`);
    if (parsed.accessToken) {
      if (!accessToken) {
        accessToken = parsed.accessToken;
        window.sessionStorage.setItem(`${SESSION_TOKEN_PREFIX}.${personaId}`, accessToken);
      }

      window.localStorage.setItem(
        `${SESSION_PREFIX}.${personaId}`,
        JSON.stringify({
          sessionId: parsed.sessionId,
          expiresAt: parsed.expiresAt,
          updatedAt: parsed.updatedAt ?? new Date().toISOString(),
        }),
      );
    }

    if (!accessToken) {
      return null;
    }

    return {
      sessionId: parsed.sessionId,
      accessToken,
      expiresAt: parsed.expiresAt,
    };
  } catch {
    return null;
  }
}

export async function setPersonaPrivateKey(
  personaId: string,
  privateKeyHex: string,
): Promise<void> {
  if (typeof window === "undefined") {
    return;
  }

  const encrypted = await encryptText(personaId, privateKeyHex);
  await secureSetItem(`${PRIVATE_KEY_PREFIX}.${personaId}`, encrypted);
}

export async function getPersonaPrivateKey(personaId: string): Promise<string | null> {
  if (typeof window === "undefined") {
    return null;
  }

  const encrypted = await secureGetItem(`${PRIVATE_KEY_PREFIX}.${personaId}`);
  if (!encrypted) {
    return null;
  }

  try {
    return await decryptText(personaId, encrypted);
  } catch {
    return null;
  }
}

export function clearPersonaSession(personaId: string): void {
  if (typeof window === "undefined") {
    return;
  }

  window.localStorage.removeItem(`${SESSION_PREFIX}.${personaId}`);
  window.sessionStorage.removeItem(`${SESSION_TOKEN_PREFIX}.${personaId}`);
}

export function clearPersonaPrivateKey(personaId: string): void {
  if (typeof window === "undefined") {
    return;
  }

  void secureRemoveItem(`${PRIVATE_KEY_PREFIX}.${personaId}`);
}
