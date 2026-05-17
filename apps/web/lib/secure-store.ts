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

function resolveProvider(): SecureStoreProvider | null {
  if (typeof window === "undefined") {
    return null;
  }

  return window.__HEXRELAY_SECURE_STORE__ ?? null;
}

export async function secureGetItem(key: string): Promise<string | null> {
  const provider = resolveProvider();
  if (!provider) {
    return null;
  }

  try {
    return await provider.getItem(key);
  } catch {
    return null;
  }
}

export async function secureSetItem(key: string, value: string): Promise<void> {
  const provider = resolveProvider();
  if (!provider) {
    throw new Error("Secure storage provider unavailable");
  }

  try {
    await provider.setItem(key, value);
  } catch {
    throw new Error("Secure storage provider write failed");
  }
}

export async function secureRemoveItem(key: string): Promise<void> {
  const provider = resolveProvider();
  if (!provider) {
    return;
  }

  try {
    await provider.removeItem(key);
  } catch {
    // Removing key handles is best-effort when the platform store is unavailable.
  }
}
