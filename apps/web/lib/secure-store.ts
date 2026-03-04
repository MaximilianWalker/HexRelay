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
    try {
      const providerValue = await provider.getItem(key);
      if (providerValue !== null) {
        return providerValue;
      }

      return window.localStorage.getItem(fallbackKey(key));
    } catch {
      return window.localStorage.getItem(fallbackKey(key));
    }
  }

  return window.localStorage.getItem(fallbackKey(key));
}

export async function secureSetItem(key: string, value: string): Promise<void> {
  if (typeof window === "undefined") {
    return;
  }

  const provider = resolveProvider();
  if (provider) {
    try {
      await provider.setItem(key, value);
      return;
    } catch {
      window.localStorage.setItem(fallbackKey(key), value);
      return;
    }
  }

  window.localStorage.setItem(fallbackKey(key), value);
}

export async function secureRemoveItem(key: string): Promise<void> {
  if (typeof window === "undefined") {
    return;
  }

  const provider = resolveProvider();
  if (provider) {
    try {
      await provider.removeItem(key);
    } catch {
      // continue and clear fallback storage below
    }
  }

  window.localStorage.removeItem(fallbackKey(key));
}
