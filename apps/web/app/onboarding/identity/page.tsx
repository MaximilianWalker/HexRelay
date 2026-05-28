"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useMemo, useState } from "react";

import {
  issueAuthChallenge,
  registerIdentityKey,
  verifyAuthChallenge,
} from "@/lib/api";
import { Button } from "@/components/ui/button";
import { Field } from "@/components/ui/field";
import { Notice } from "@/components/ui/notice";
import { SegmentedControl, type SegmentedControlOption } from "@/components/ui/segmented-control";
import { TextArea } from "@/components/ui/text-area";
import { TextInput } from "@/components/ui/text-input";
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
import { OnboardingShell } from "@/components/onboarding/onboarding-shell";
import styles from "../onboarding.module.css";

const SAMPLE_PUBLIC_KEY = "7f:31:9c:4a:22:09:11:ab:c4:17:59:82:1d:ef:4b:10";
const IDENTITY_MODE_OPTIONS: SegmentedControlOption<"create" | "import">[] = [
  { id: "create", label: "Create identity" },
  { id: "import", label: "Import identity" },
];

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
    <OnboardingShell
      activeStep="identity"
      introBody="Your identity keys stay on your device. Server interactions only use the public key and signed proofs."
      introTitle="Set up your local identity"
      promises={[
        "Client-controlled key material and persona isolation.",
        "DM inbound policy defaults to friends-only.",
        "No server relay for direct-message payloads.",
      ]}
      statusItems={[
        `Identity: ${canContinue ? "ready" : "needs input"}`,
        "Recovery: pending",
        "Access path: pending",
      ]}
      wizardSubtitle="Choose how you want to start and set a persona label."
      wizardTitle="Identity bootstrap"
    >
      <Field helper="Persona sessions and settings are kept isolated." label="Persona name">
        <TextInput
          value={personaName}
          onChange={(event) => setPersonaName(event.target.value)}
          placeholder="e.g. Max - main"
        />
      </Field>

      <SegmentedControl label="Identity setup mode" onChange={setMode} options={IDENTITY_MODE_OPTIONS} value={mode} />

      {mode === "create" ? (
        <>
          <Notice className={styles.notice} tone="success">
            New ed25519 keypair will be generated locally on continue.
          </Notice>
          <Field label="Public key preview">
            <TextInput value={SAMPLE_PUBLIC_KEY} readOnly />
          </Field>
        </>
      ) : (
        <>
          <Field label="Private key">
            <TextArea
              value={importKey}
              onChange={(event) => setImportKey(event.target.value)}
              placeholder="Paste hex/base64 private key"
            />
          </Field>
          {importKey.length > 0 && !isLikelyKey(importKey) ? (
            <Notice className={styles.notice} tone="danger">
              identity_key_invalid: unsupported key format.
            </Notice>
          ) : (
            <Notice className={styles.notice} tone="warning">
              Imported keys are encrypted locally before persistence.
            </Notice>
          )}
        </>
      )}

      {error ? (
        <Notice className={styles.notice} tone="danger">
          {error}
        </Notice>
      ) : null}

      <div className={styles.ctaRow}>
        <Link className={styles.buttonGhost} href="/">
          Back
        </Link>
        <Button disabled={!canContinue || loading} onClick={handleContinue} variant="primary">
          {loading ? "Creating identity..." : "Continue to recovery"}
        </Button>
      </div>
    </OnboardingShell>
  );
}
