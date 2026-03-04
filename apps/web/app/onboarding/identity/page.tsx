"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useMemo, useState } from "react";

import {
  issueAuthChallenge,
  registerIdentityKey,
  verifyAuthChallenge,
} from "@/lib/api";
import {
  bytesToHex,
  derivePublicKey,
  generateIdentityKeypair,
  parsePrivateKey,
  signNonce,
} from "@/lib/crypto";
import { ensurePersona } from "@/lib/personas";
import { setPersonaPrivateKey, setPersonaSession } from "@/lib/sessions";
import { trackEvent } from "@/lib/telemetry";
import styles from "../onboarding.module.css";

const SAMPLE_FINGERPRINT = "7f:31:9c:4a:22:09:11:ab:c4:17:59:82:1d:ef:4b:10";

function isLikelyKey(value: string): boolean {
  const trimmed = value.trim();
  if (/^[a-fA-F0-9]{64}$/.test(trimmed)) {
    return true;
  }

  return /^[A-Za-z0-9+/=]+$/.test(trimmed) && trimmed.length >= 40;
}

export default function IdentityOnboardingPage() {
  const router = useRouter();
  const [mode, setMode] = useState<"create" | "import">("create");
  const [personaName, setPersonaName] = useState("");
  const [importKey, setImportKey] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const canContinue = useMemo(() => {
    if (personaName.trim().length < 2) {
      return false;
    }

    if (mode === "create") {
      return true;
    }

    return isLikelyKey(importKey);
  }, [importKey, mode, personaName]);

  async function handleContinue() {
    if (!canContinue) {
      return;
    }

    setLoading(true);
    setError(null);
    trackEvent("onboarding_identity_submit", { mode });

    try {
      const persona = ensurePersona(personaName);
      const identityId = persona.id;

      const privateKeyHex =
        mode === "create"
          ? generateIdentityKeypair().privateKeyHex
          : bytesToHex(parsePrivateKey(importKey));

      const privateKey = parsePrivateKey(privateKeyHex);
      const publicKey = bytesToHex(derivePublicKey(privateKey));

      await setPersonaPrivateKey(identityId, privateKeyHex);

      const registerResult = await registerIdentityKey({
        identityId,
        publicKey,
      });
      if (!registerResult.ok) {
        trackEvent("onboarding_identity_register_failed", {
          code: registerResult.code,
        });
        setError(`${registerResult.code}: ${registerResult.message}`);
        return;
      }

      const challengeResult = await issueAuthChallenge({ identityId });
      if (!challengeResult.ok) {
        trackEvent("onboarding_identity_challenge_failed", {
          code: challengeResult.code,
        });
        setError(`${challengeResult.code}: ${challengeResult.message}`);
        return;
      }

      const signature = signNonce(privateKey, challengeResult.data.nonce);
      const verifyResult = await verifyAuthChallenge({
        identityId,
        challengeId: challengeResult.data.challenge_id,
        signature,
      });
      if (!verifyResult.ok) {
        trackEvent("onboarding_identity_verify_failed", {
          code: verifyResult.code,
        });
        setError(`${verifyResult.code}: ${verifyResult.message}`);
        return;
      }

      setPersonaSession(identityId, {
        sessionId: verifyResult.data.session_id,
        accessToken: verifyResult.data.access_token,
        expiresAt: verifyResult.data.expires_at,
      });

      trackEvent("onboarding_identity_complete", {
        personaId: identityId,
      });

      router.push("/onboarding/recovery");
    } catch (caughtError) {
      trackEvent("onboarding_identity_exception", {});
      if (caughtError instanceof Error) {
        setError(`identity_key_invalid: ${caughtError.message}`);
      } else {
        setError("identity_key_invalid: unknown key parsing failure");
      }
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className={styles.shell}>
      <div className={styles.content}>
        <aside className={styles.panel}>
          <p className={styles.brandEyebrow}>HexRelay onboarding</p>
          <h1 className={styles.leftTitle}>Set up your local identity</h1>
          <p className={styles.leftBody}>
            Your identity keys stay on your device. Server interactions only use
            the public key and signed proofs.
          </p>
          <ul className={styles.promiseList}>
            <li>Client-controlled key material and persona isolation.</li>
            <li>DM inbound policy defaults to friends-only.</li>
            <li>No guild relay for direct-message payloads.</li>
          </ul>
        </aside>

        <main className={styles.panel}>
          <div className={styles.steps}>
            <div className={`${styles.step} ${styles.activeStep}`}>1. Identity</div>
            <div className={styles.step}>2. Recovery</div>
            <div className={styles.step}>3. Access</div>
          </div>
          <h2 className={styles.wizardTitle}>Identity bootstrap</h2>
          <p className={styles.wizardSubtitle}>
            Choose how you want to start and set a persona label.
          </p>

          <div className={styles.fieldGroup}>
            <label className={styles.label} htmlFor="personaName">
              Persona name
            </label>
            <input
              id="personaName"
              className={styles.input}
              value={personaName}
              onChange={(event) => setPersonaName(event.target.value)}
              placeholder="e.g. Max - main"
            />
            <p className={styles.helper}>
              Persona sessions and settings are kept isolated.
            </p>
          </div>

          <div className={styles.tabRow}>
            <button
              className={`${styles.tab} ${mode === "create" ? styles.activeTab : ""}`}
              type="button"
              onClick={() => setMode("create")}
            >
              Create identity
            </button>
            <button
              className={`${styles.tab} ${mode === "import" ? styles.activeTab : ""}`}
              type="button"
              onClick={() => setMode("import")}
            >
              Import identity
            </button>
          </div>

          {mode === "create" ? (
            <>
              <div className={`${styles.status} ${styles.ok}`}>
                New ed25519 keypair will be generated locally on continue.
              </div>
              <div className={styles.fieldGroup}>
                <label className={styles.label}>Public fingerprint preview</label>
                <input className={styles.input} value={SAMPLE_FINGERPRINT} readOnly />
              </div>
            </>
          ) : (
            <>
              <div className={styles.fieldGroup}>
                <label className={styles.label} htmlFor="importKey">
                  Private key
                </label>
                <textarea
                  id="importKey"
                  className={styles.textarea}
                  value={importKey}
                  onChange={(event) => setImportKey(event.target.value)}
                  placeholder="Paste hex/base64 private key"
                />
              </div>
              {importKey.length > 0 && !isLikelyKey(importKey) ? (
                <div className={`${styles.status} ${styles.error}`}>
                  identity_key_invalid: unsupported key format.
                </div>
              ) : (
                <div className={`${styles.status} ${styles.warn}`}>
                  Imported keys are encrypted locally before persistence.
                </div>
              )}
            </>
          )}

          {error ? <div className={`${styles.status} ${styles.error}`}>{error}</div> : null}

          <div className={styles.ctaRow}>
            <Link className={styles.buttonGhost} href="/">
              Back
            </Link>
            <button
              className={styles.button}
              disabled={!canContinue || loading}
              onClick={handleContinue}
              type="button"
            >
              {loading ? "Creating identity..." : "Continue to recovery"}
            </button>
          </div>
        </main>

        <aside className={styles.panel}>
          <h3 className={styles.wizardTitle}>Setup status</h3>
          <div className={styles.asideList}>
            <div className={styles.asideItem}>
              Identity: {canContinue ? "ready" : "needs input"}
            </div>
            <div className={styles.asideItem}>Recovery: pending</div>
            <div className={styles.asideItem}>Access path: pending</div>
          </div>
        </aside>
      </div>
    </div>
  );
}
