import type { NextConfig } from "next";

import { DEFAULT_API_BASE_URL, DEFAULT_REALTIME_WS_URL } from "./lib/env-defaults";

const runtimeInstance = process.env.HEXRELAY_RUNTIME_INSTANCE?.replace(/[^a-zA-Z0-9_-]/g, "-");
const runtimeDistDir = runtimeInstance ? `.next-${runtimeInstance}` : undefined;

const developmentLoopbackConnectSources = [
  "http://127.0.0.1:*",
  "http://localhost:*",
  "ws://127.0.0.1:*",
  "ws://localhost:*",
] as const;

const loopbackHosts = new Set(["localhost", "::1", "[::1]"]);

function isDevelopment() {
  return process.env.NODE_ENV === "development";
}

function isLoopbackHost(hostname: string) {
  const normalized = hostname.toLowerCase();

  return loopbackHosts.has(normalized) || normalized.startsWith("127.");
}

function endpointOrigin(
  envName: "NEXT_PUBLIC_API_BASE_URL" | "NEXT_PUBLIC_REALTIME_WS_URL",
  fallback: string,
  allowedProtocols: readonly string[],
) {
  const rawValue = process.env[envName] ?? fallback;
  const url = new URL(rawValue);

  if (!allowedProtocols.includes(url.protocol)) {
    throw new Error(
      `${envName} must use one of ${allowedProtocols.join(", ")} for web CSP connect-src.`,
    );
  }

  if (!isDevelopment() && ["http:", "ws:"].includes(url.protocol) && !isLoopbackHost(url.hostname)) {
    throw new Error(
      `${envName} must not use non-loopback ${url.protocol} endpoints outside development.`,
    );
  }

  return url.origin;
}

function connectSrcSources() {
  const sources = [
    "'self'",
    endpointOrigin("NEXT_PUBLIC_API_BASE_URL", DEFAULT_API_BASE_URL, ["http:", "https:"]),
    endpointOrigin("NEXT_PUBLIC_REALTIME_WS_URL", DEFAULT_REALTIME_WS_URL, ["ws:", "wss:"]),
    ...(isDevelopment() ? developmentLoopbackConnectSources : []),
  ];

  return Array.from(new Set(sources));
}

const nextConfig: NextConfig = {
  ...(runtimeDistDir
    ? {
        distDir: runtimeDistDir,
        typescript: {
          tsconfigPath: `.runtime-tsconfig/${runtimeInstance}.json`,
        },
      }
    : {}),
  async headers() {
    const csp = [
      "default-src 'self'",
      "base-uri 'self'",
      "frame-ancestors 'none'",
      "object-src 'none'",
      `script-src 'self'${isDevelopment() ? " 'unsafe-inline' 'unsafe-eval'" : ""}`,
      "style-src 'self' 'unsafe-inline'",
      "img-src 'self' data: blob:",
      "font-src 'self' https://fonts.gstatic.com",
      `connect-src ${connectSrcSources().join(" ")}`,
      "form-action 'self'",
      "upgrade-insecure-requests",
    ].join("; ");

    return [
      {
        source: "/:path*",
        headers: [
          { key: "Content-Security-Policy", value: csp },
          { key: "Referrer-Policy", value: "strict-origin-when-cross-origin" },
          { key: "X-Content-Type-Options", value: "nosniff" },
          { key: "X-Frame-Options", value: "DENY" },
          { key: "Permissions-Policy", value: "camera=(), microphone=(), geolocation=()" },
        ],
      },
    ];
  },
};

export default nextConfig;
