type TelemetryEvent = {
  name: string;
  timestamp: string;
  payload?: Record<string, string | number | boolean | null>;
};

const TELEMETRY_KEY = "hexrelay.telemetry.v1";

function safeReadEvents(): TelemetryEvent[] {
  if (typeof window === "undefined") {
    return [];
  }

  const raw = window.sessionStorage.getItem(TELEMETRY_KEY);
  if (!raw) {
    return [];
  }

  try {
    return JSON.parse(raw) as TelemetryEvent[];
  } catch {
    return [];
  }
}

export function trackEvent(
  name: string,
  payload?: Record<string, string | number | boolean | null>,
): void {
  if (typeof window === "undefined") {
    return;
  }

  const event: TelemetryEvent = {
    name,
    timestamp: new Date().toISOString(),
    payload,
  };

  const next = [...safeReadEvents(), event].slice(-200);
  window.sessionStorage.setItem(TELEMETRY_KEY, JSON.stringify(next));
  console.info("[telemetry]", event);
}
