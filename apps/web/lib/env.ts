import { z } from "zod";

const envSchema = z.object({
  NEXT_PUBLIC_API_BASE_URL: z.string().url(),
  NEXT_PUBLIC_REALTIME_WS_URL: z.string().url(),
});

const parsed = envSchema.safeParse({
  NEXT_PUBLIC_API_BASE_URL:
    process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8080",
  NEXT_PUBLIC_REALTIME_WS_URL:
    process.env.NEXT_PUBLIC_REALTIME_WS_URL ?? "ws://127.0.0.1:8081/ws",
});

if (!parsed.success) {
  const issues = parsed.error.issues
    .map((issue) => `${issue.path.join(".")}: ${issue.message}`)
    .join("; ");
  throw new Error(`Invalid web environment configuration. ${issues}`);
}

export const env = parsed.data;
