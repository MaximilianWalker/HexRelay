import { afterEach, describe, expect, it } from "vitest";

import nextConfig from "./next.config";

const originalEnv = {
  NODE_ENV: process.env.NODE_ENV,
  NEXT_PUBLIC_API_BASE_URL: process.env.NEXT_PUBLIC_API_BASE_URL,
  NEXT_PUBLIC_REALTIME_WS_URL: process.env.NEXT_PUBLIC_REALTIME_WS_URL,
};

function setEnv(name: keyof typeof originalEnv, value: string | undefined) {
  if (value === undefined) {
    delete process.env[name];
  } else {
    process.env[name] = value;
  }
}

function restoreEnv() {
  setEnv("NODE_ENV", originalEnv.NODE_ENV);
  setEnv("NEXT_PUBLIC_API_BASE_URL", originalEnv.NEXT_PUBLIC_API_BASE_URL);
  setEnv("NEXT_PUBLIC_REALTIME_WS_URL", originalEnv.NEXT_PUBLIC_REALTIME_WS_URL);
}

async function connectSources() {
  const headerGroups = await nextConfig.headers?.();
  const csp = headerGroups
    ?.flatMap((group) => group.headers)
    .find((header) => header.key === "Content-Security-Policy")?.value;

  if (!csp) {
    throw new Error("Content-Security-Policy header was not configured");
  }

  const connectDirective = csp
    .split(";")
    .map((directive) => directive.trim())
    .find((directive) => directive.startsWith("connect-src "));

  if (!connectDirective) {
    throw new Error("connect-src directive was not configured");
  }

  return connectDirective.split(/\s+/).slice(1);
}

describe("web security headers", () => {
  afterEach(() => {
    restoreEnv();
  });

  it("limits production connect-src to self and configured endpoint origins", async () => {
    setEnv("NODE_ENV", "production");
    setEnv("NEXT_PUBLIC_API_BASE_URL", "https://api.example.test/api");
    setEnv("NEXT_PUBLIC_REALTIME_WS_URL", "wss://realtime.example.test/ws");

    await expect(connectSources()).resolves.toEqual([
      "'self'",
      "https://api.example.test",
      "wss://realtime.example.test",
    ]);
  });

  it("keeps development loopback allowances without broad network schemes", async () => {
    setEnv("NODE_ENV", "development");
    setEnv("NEXT_PUBLIC_API_BASE_URL", "http://127.0.0.1:8080");
    setEnv("NEXT_PUBLIC_REALTIME_WS_URL", "ws://127.0.0.1:8081/ws");

    const sources = await connectSources();

    expect(sources).toContain("'self'");
    expect(sources).toContain("http://127.0.0.1:8080");
    expect(sources).toContain("ws://127.0.0.1:8081");
    expect(sources).toContain("http://127.0.0.1:*");
    expect(sources).toContain("ws://127.0.0.1:*");
    expect(sources).not.toContain("http:");
    expect(sources).not.toContain("https:");
    expect(sources).not.toContain("ws:");
    expect(sources).not.toContain("wss:");
  });

  it("rejects non-loopback plaintext endpoints outside development", async () => {
    setEnv("NODE_ENV", "production");
    setEnv("NEXT_PUBLIC_API_BASE_URL", "http://api.example.test");
    setEnv("NEXT_PUBLIC_REALTIME_WS_URL", "wss://realtime.example.test/ws");

    await expect(connectSources()).rejects.toThrow(
      "NEXT_PUBLIC_API_BASE_URL must not use non-loopback http: endpoints outside development.",
    );
  });
});
