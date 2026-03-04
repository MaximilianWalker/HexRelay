import nacl from "tweetnacl";

function byteToHex(byte: number): string {
  return byte.toString(16).padStart(2, "0");
}

export function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes).map(byteToHex).join("");
}

export function hexToBytes(value: string): Uint8Array {
  const trimmed = value.trim();
  if (!/^[a-fA-F0-9]+$/.test(trimmed) || trimmed.length % 2 !== 0) {
    throw new Error("Invalid hex value");
  }

  const output = new Uint8Array(trimmed.length / 2);
  for (let index = 0; index < output.length; index += 1) {
    output[index] = Number.parseInt(trimmed.slice(index * 2, index * 2 + 2), 16);
  }
  return output;
}

export function base64ToBytes(value: string): Uint8Array {
  const normalized = value.trim();
  const raw = atob(normalized);
  const output = new Uint8Array(raw.length);
  for (let index = 0; index < raw.length; index += 1) {
    output[index] = raw.charCodeAt(index);
  }
  return output;
}

export function parsePrivateKey(value: string): Uint8Array {
  const trimmed = value.trim();
  const bytes = /^[a-fA-F0-9]+$/.test(trimmed) ? hexToBytes(trimmed) : base64ToBytes(trimmed);

  if (bytes.length === 64) {
    return bytes;
  }

  if (bytes.length === 32) {
    return nacl.sign.keyPair.fromSeed(bytes).secretKey;
  }

  throw new Error("Private key must be 32-byte seed or 64-byte secret key");
}

export function generateIdentityKeypair(): { publicKeyHex: string; privateKeyHex: string } {
  const pair = nacl.sign.keyPair();
  return {
    publicKeyHex: bytesToHex(pair.publicKey),
    privateKeyHex: bytesToHex(pair.secretKey),
  };
}

export function derivePublicKey(privateKey: Uint8Array): Uint8Array {
  return nacl.sign.keyPair.fromSecretKey(privateKey).publicKey;
}

export function signNonce(privateKey: Uint8Array, nonce: string): string {
  const nonceBytes = new TextEncoder().encode(nonce);
  const signature = nacl.sign.detached(nonceBytes, privateKey);
  return bytesToHex(signature);
}
