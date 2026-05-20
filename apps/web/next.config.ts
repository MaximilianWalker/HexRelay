import type { NextConfig } from "next";

const isDev = process.env.NODE_ENV === "development";
const runtimeInstance = process.env.HEXRELAY_RUNTIME_INSTANCE?.replace(/[^a-zA-Z0-9_-]/g, "-");
const runtimeDistId = (process.env.HEXRELAY_RUNTIME_DIST_ID ?? runtimeInstance)?.replace(/[^a-zA-Z0-9_-]/g, "-");
const runtimeDistDir = runtimeDistId ? `.next-${runtimeDistId}` : undefined;

const nextConfig: NextConfig = {
  ...(runtimeDistDir
    ? {
        distDir: runtimeDistDir,
        typescript: {
          tsconfigPath: `.runtime-tsconfig/${runtimeDistId}.json`,
        },
      }
    : {}),
  async headers() {
    const csp = [
      "default-src 'self'",
      "base-uri 'self'",
      "frame-ancestors 'none'",
      "object-src 'none'",
      `script-src 'self'${isDev ? " 'unsafe-inline' 'unsafe-eval'" : ""}`,
      "style-src 'self' 'unsafe-inline'",
      "img-src 'self' data: blob:",
      "font-src 'self' https://fonts.gstatic.com",
      "connect-src 'self' http: https: ws: wss:",
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
