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

function fallbackStorage(): Storage | null {
  if (typeof window === "undefined") {
    return null;
  }

  return window.sessionStorage;
}

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
  const storage = fallbackStorage();
  if (!storage) {
    return null;
  }

  const provider = resolveProvider();
  if (provider) {
    const fallbackValue = storage.getItem(fallbackKey(key));

    try {
      const providerValue = await provider.getItem(key);

      if (fallbackValue !== null) {
        let synced = providerValue === fallbackValue;

        if (!synced) {
          try {
            await provider.setItem(key, fallbackValue);
            synced = true;
          } catch {
            // keep fallback value as source of truth when provider update fails
          }
        }

        if (synced) {
          storage.removeItem(fallbackKey(key));
        }

        return fallbackValue;
      }

      return providerValue;
    } catch {
      return fallbackValue;
    }
  }

  return storage.getItem(fallbackKey(key));
}

export async function secureSetItem(key: string, value: string): Promise<void> {
  const storage = fallbackStorage();
  if (!storage) {
    return;
  }

  const provider = resolveProvider();
  if (provider) {
    try {
      await provider.setItem(key, value);
      storage.removeItem(fallbackKey(key));
      return;
    } catch {
      storage.setItem(fallbackKey(key), value);
      return;
    }
  }

  storage.setItem(fallbackKey(key), value);
}

export async function secureRemoveItem(key: string): Promise<void> {
  const storage = fallbackStorage();
  if (!storage) {
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

  storage.removeItem(fallbackKey(key));
}
