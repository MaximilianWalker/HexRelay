export function buildDmPairingLink(envelope: string): string {
  return `hexrelay://dm-pairing/${envelope}`;
}

export function buildDmPairingManualCode(envelope: string): string {
  const chunks = envelope.match(/.{1,8}/g) ?? [envelope];
  return `DM1 ${chunks.join(" ")}`;
}

export function parseDmPairingInput(rawInput: string): string | null {
  const raw = rawInput.trim();
  if (!raw) {
    return null;
  }

  const manualCode = raw.match(/^DM1[:\s]+([\s\S]+)$/i);
  if (manualCode?.[1]) {
    const envelope = manualCode[1].replace(/\s+/g, "");
    return envelope || null;
  }

  try {
    const maybeUrl = new URL(raw);
    if (maybeUrl.protocol === "hexrelay:" && maybeUrl.hostname === "dm-pairing") {
      const pathSegment = maybeUrl.pathname.replace(/^\/+/, "").split("/").filter(Boolean)[0];
      return pathSegment ?? null;
    }
  } catch {
    // Fall through to raw-envelope parsing.
  }

  const withoutQueryOrFragment = raw.split(/[?#]/)[0];
  if (withoutQueryOrFragment.includes("/")) {
    const segments = withoutQueryOrFragment.split("/").filter(Boolean);
    return segments.length > 0 ? segments[segments.length - 1] : null;
  }

  return withoutQueryOrFragment || null;
}
