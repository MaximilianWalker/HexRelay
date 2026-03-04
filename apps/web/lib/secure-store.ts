type SecureStoreProvider = {
  getItem(key: string): Promise<string | null>;
  removeItem(key: string): Promise<void>;
  setItem(key: string, value: string): Promise<void>;
};

declare global {
  interface Window {
    __HEXRELAY_SECURE_STORE__?: SecureStoreProvider;
  }
}

const FALLBACK_PREFIX = "hexrelay.secure.fallback.v1";

function resolveProvider(): SecureStoreProvider | null {
  if (typeof window === "undefined") {
    return null;
  }

  return window.__HEXRELAY_SECURE_STORE__ ?? null;
}

function fallbackKey(key: string): string {
  return `${FALLBACK_PREFIX}.${key}`;
}

export async function secureGetItem(key: string): Promise<string | null> {
  if (typeof window === "undefined") {
    return null;
  }

  const provider = resolveProvider();
  if (provider) {
    return provider.getItem(key);
  }

  return window.sessionStorage.getItem(fallbackKey(key));
}

export async function secureSetItem(key: string, value: string): Promise<void> {
  if (typeof window === "undefined") {
    return;
  }

  const provider = resolveProvider();
  if (provider) {
    await provider.setItem(key, value);
    return;
  }

  window.sessionStorage.setItem(fallbackKey(key), value);
}

export async function secureRemoveItem(key: string): Promise<void> {
  if (typeof window === "undefined") {
    return;
  }

  const provider = resolveProvider();
  if (provider) {
    await provider.removeItem(key);
    return;
  }

  window.sessionStorage.removeItem(fallbackKey(key));
}
