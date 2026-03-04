"use client";

import Link from "next/link";
import { useState } from "react";

import { createInvite, redeemInvite } from "@/lib/api";
import { trackEvent } from "@/lib/telemetry";
import styles from "../onboarding.module.css";

type RedeemState = "idle" | "loading" | "success" | "error";

export default function AccessOnboardingPage() {
  const [entry, setEntry] = useState<"server" | "contact" | "skip">("server");
  const [token, setToken] = useState("");
  const [nodeFingerprint, setNodeFingerprint] = useState("hexrelay-local-fingerprint");
  const [redeemState, setRedeemState] = useState<RedeemState>("idle");
  const [createState, setCreateState] = useState<"idle" | "loading" | "ok" | "error">("idle");
  const [apiCode, setApiCode] = useState<string | null>(null);
  const [apiMessage, setApiMessage] = useState<string | null>(null);

  const canRedeem = token.trim().length > 0 && nodeFingerprint.trim().length > 0;
  const canFinish = entry === "skip" || redeemState === "success";

  async function handleRedeem() {
    if (!canRedeem || redeemState === "loading") {
      return;
    }

    setRedeemState("loading");
    setApiCode(null);
    setApiMessage(null);
    trackEvent("onboarding_access_redeem_started", { entry });

    const result = await redeemInvite({
      token,
      nodeFingerprint,
    });

    if (result.ok) {
      setRedeemState("success");
      trackEvent("onboarding_access_redeem_success", { entry });
      return;
    }

    setRedeemState("error");
    setApiCode(result.code);
    setApiMessage(result.message);
    trackEvent("onboarding_access_redeem_failed", {
      code: result.code,
      entry,
    });
  }

  async function handleCreateTestInvite() {
    if (createState === "loading") {
      return;
    }

    setCreateState("loading");
    setApiCode(null);
    setApiMessage(null);
    trackEvent("onboarding_access_create_started", {});

    const result = await createInvite({
      mode: "multi_use",
      maxUses: 5,
    });

    if (!result.ok) {
      setCreateState("error");
      setApiCode(result.code);
      setApiMessage(result.message);
      trackEvent("onboarding_access_create_failed", { code: result.code });
      return;
    }

    setToken(result.data.token);
    setCreateState("ok");
    trackEvent("onboarding_access_create_success", {});
  }

  return (
    <div className={styles.shell}>
      <div className={styles.content}>
        <aside className={styles.panel}>
          <p className={styles.brandEyebrow}>HexRelay onboarding</p>
          <h1 className={styles.leftTitle}>Choose your access path</h1>
          <p className={styles.leftBody}>
            You can join a server, add a direct contact, or continue to Home and
            do this later.
          </p>
          <ul className={styles.promiseList}>
            <li>Server joins use invite token policy.</li>
            <li>Direct contact flow supports link and QR.</li>
            <li>You can start with an empty workspace.</li>
          </ul>
        </aside>

        <main className={styles.panel}>
          <div className={styles.steps}>
            <div className={styles.step}>1. Identity</div>
            <div className={styles.step}>2. Recovery</div>
            <div className={`${styles.step} ${styles.activeStep}`}>3. Access</div>
          </div>
          <h2 className={styles.wizardTitle}>Entry options</h2>
          <p className={styles.wizardSubtitle}>
            Pick how you want to start. You can change this immediately after
            onboarding.
          </p>

          <div className={styles.choiceGrid}>
            <button
              className={styles.choice}
              type="button"
              onClick={() => {
                setEntry("server");
                setRedeemState("idle");
                setApiCode(null);
                setApiMessage(null);
              }}
            >
              <p className={styles.choiceTitle}>Join server via invite token</p>
              <p className={styles.choiceText}>
                Uses mode, expiration, and max-use policy checks.
              </p>
            </button>
            <button
              className={styles.choice}
              type="button"
              onClick={() => {
                setEntry("contact");
                setRedeemState("idle");
                setApiCode(null);
                setApiMessage(null);
              }}
            >
              <p className={styles.choiceTitle}>Add direct contact invite</p>
              <p className={styles.choiceText}>
                Redeem personal link/QR for privacy-first contact bootstrap.
              </p>
            </button>
            <button
              className={styles.choice}
              type="button"
              onClick={() => {
                setEntry("skip");
                setRedeemState("idle");
                setApiCode(null);
                setApiMessage(null);
              }}
            >
              <p className={styles.choiceTitle}>Continue without joining</p>
              <p className={styles.choiceText}>
                Land in Home with onboarding-complete empty-state.
              </p>
            </button>
          </div>

          {entry === "skip" ? (
            <div className={`${styles.status} ${styles.warn}`}>
              You can finish onboarding now and connect later.
            </div>
          ) : (
            <>
              <div className={styles.fieldGroup}>
                <label className={styles.label} htmlFor="tokenInput">
                  {entry === "server" ? "Server invite token" : "Contact invite token"}
                </label>
                <input
                  id="tokenInput"
                  className={styles.input}
                  value={token}
                  onChange={(event) => setToken(event.target.value)}
                  placeholder="Paste invite token"
                />
              </div>
              <div className={styles.fieldGroup}>
                <label className={styles.label} htmlFor="fingerprintInput">
                  Node fingerprint
                </label>
                <input
                  id="fingerprintInput"
                  className={styles.input}
                  value={nodeFingerprint}
                  onChange={(event) => setNodeFingerprint(event.target.value)}
                  placeholder="hexrelay-local-fingerprint"
                />
                <p className={styles.helper}>
                  Join flow fails closed when fingerprint does not match invite metadata.
                </p>
              </div>

              <div className={styles.ctaRow}>
                <button
                  className={styles.buttonGhost}
                  onClick={handleCreateTestInvite}
                  type="button"
                >
                  {createState === "loading" ? "Creating..." : "Create test invite"}
                </button>
                <button
                  className={styles.button}
                  disabled={!canRedeem || redeemState === "loading"}
                  onClick={handleRedeem}
                  type="button"
                >
                  {redeemState === "loading" ? "Redeeming..." : "Redeem invite"}
                </button>
              </div>

              {redeemState === "success" && (
                <div className={`${styles.status} ${styles.ok}`}>
                  Invite redeemed. You can finish onboarding now.
                </div>
              )}
              {redeemState === "error" && apiCode === "invite_invalid" && (
                <div className={`${styles.status} ${styles.error}`}>
                  invite_invalid: {apiMessage}
                </div>
              )}
              {redeemState === "error" && apiCode === "invite_expired" && (
                <div className={`${styles.status} ${styles.error}`}>
                  invite_expired: {apiMessage}
                </div>
              )}
              {redeemState === "error" && apiCode === "invite_exhausted" && (
                <div className={`${styles.status} ${styles.error}`}>
                  invite_exhausted: {apiMessage}
                </div>
              )}
              {redeemState === "error" && apiCode === "fingerprint_mismatch" && (
                <div className={`${styles.status} ${styles.error}`}>
                  fingerprint_mismatch: {apiMessage}
                </div>
              )}
              {redeemState === "error" && apiCode === "error" && (
                <div className={`${styles.status} ${styles.error}`}>
                  error: unable to redeem invite.
                </div>
              )}
              {createState === "ok" && (
                <div className={`${styles.status} ${styles.ok}`}>
                  Test invite created and token field prefilled.
                </div>
              )}
            </>
          )}

          <div className={styles.ctaRow}>
            <Link className={styles.buttonGhost} href="/onboarding/recovery">
              Back to recovery
            </Link>
            <Link
              aria-disabled={!canFinish}
              className={styles.button}
              href={canFinish ? "/home" : "#"}
              onClick={(event) => {
                if (!canFinish) {
                  event.preventDefault();
                }
              }}
            >
              Finish onboarding
            </Link>
          </div>
        </main>

        <aside className={styles.panel}>
          <h3 className={styles.wizardTitle}>Setup status</h3>
          <div className={styles.asideList}>
            <div className={styles.asideItem}>Identity: ready</div>
            <div className={styles.asideItem}>Recovery: confirmed</div>
            <div className={styles.asideItem}>Access: {entry}</div>
          </div>
        </aside>
      </div>
    </div>
  );
}
