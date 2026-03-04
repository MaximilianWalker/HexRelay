import nacl from "tweetnacl";
import WebSocket from "ws";

const apiBase = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8080";
const realtimeWs = process.env.NEXT_PUBLIC_REALTIME_WS_URL ?? "ws://127.0.0.1:8081/ws";
const realtimeHealth = `${realtimeWs.replace(/^ws/, "http").replace(/\/ws$/, "")}/health`;

function toHex(bytes) {
  return Buffer.from(bytes).toString("hex");
}

async function waitForHealth(url, timeoutMs = 15000) {
  const started = Date.now();

  while (Date.now() - started < timeoutMs) {
    try {
      const response = await fetch(url);
      if (response.ok) {
        return;
      }
    } catch {
      // retry until timeout
    }

    await new Promise((resolve) => setTimeout(resolve, 300));
  }

  throw new Error(`Timeout waiting for ${url}`);
}

async function connectWebSocket(accessToken) {
  await new Promise((resolve, reject) => {
    const socket = new WebSocket(realtimeWs, {
      headers: {
        authorization: `Bearer ${accessToken}`,
      },
    });

    const timer = setTimeout(() => {
      socket.close();
      reject(new Error("Timed out waiting for websocket open"));
    }, 8000);

    socket.on("open", () => {
      clearTimeout(timer);
      socket.close();
      resolve();
    });

    socket.on("error", (error) => {
      clearTimeout(timer);
      reject(error);
    });
  });
}

async function run() {
  await waitForHealth(`${apiBase}/health`);
  await waitForHealth(realtimeHealth);

  const identityId = `smoke-${Date.now()}`;
  const keypair = nacl.sign.keyPair();
  const publicKeyHex = toHex(keypair.publicKey);

  const registerResponse = await fetch(`${apiBase}/v1/identity/keys/register`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      identity_id: identityId,
      public_key: publicKeyHex,
      algorithm: "ed25519",
    }),
  });

  if (registerResponse.status !== 201) {
    throw new Error(`register failed (${registerResponse.status})`);
  }

  const challengeResponse = await fetch(`${apiBase}/v1/auth/challenge`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ identity_id: identityId }),
  });
  if (!challengeResponse.ok) {
    throw new Error(`challenge failed (${challengeResponse.status})`);
  }

  const challenge = await challengeResponse.json();
  const signatureHex = toHex(nacl.sign.detached(new TextEncoder().encode(challenge.nonce), keypair.secretKey));

  const verifyResponse = await fetch(`${apiBase}/v1/auth/verify`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      identity_id: identityId,
      challenge_id: challenge.challenge_id,
      signature: signatureHex,
    }),
  });

  if (!verifyResponse.ok) {
    throw new Error(`verify failed (${verifyResponse.status})`);
  }

  const verified = await verifyResponse.json();
  if (!verified.access_token) {
    throw new Error("verify response missing access_token");
  }

  await connectWebSocket(verified.access_token);

  process.stdout.write("Smoke e2e passed\n");
}

run().catch((error) => {
  process.stderr.write(`Smoke e2e failed: ${error.message}\n`);
  process.exit(1);
});
