"use client";

import Link from "next/link";
import { useMemo, useState } from "react";

import styles from "../onboarding.module.css";

const PHRASE = [
  "amber",
  "violet",
  "atlas",
  "linen",
  "shell",
  "ridge",
  "orbit",
  "grain",
  "harbor",
  "whisper",
  "delta",
  "cedar",
];

export default function RecoveryOnboardingPage() {
  const [word3, setWord3] = useState("");
  const [word7, setWord7] = useState("");
  const [word11, setWord11] = useState("");

  const confirmed = useMemo(
    () =>
      word3.trim().toLowerCase() === PHRASE[2] &&
      word7.trim().toLowerCase() === PHRASE[6] &&
      word11.trim().toLowerCase() === PHRASE[10],
    [word11, word3, word7],
  );

  return (
    <div className={styles.shell}>
      <div className={styles.content}>
        <aside className={styles.panel}>
          <p className={styles.brandEyebrow}>HexRelay onboarding</p>
          <h1 className={styles.leftTitle}>Confirm recovery phrase</h1>
          <p className={styles.leftBody}>
            Recovery confirmation is mandatory. Onboarding cannot finish without
            this step.
          </p>
          <ul className={styles.promiseList}>
            <li>Phrase is displayed once for backup.</li>
            <li>Never send this phrase through chat channels.</li>
            <li>Losing phrase means no key recovery.</li>
          </ul>
        </aside>

        <main className={styles.panel}>
          <div className={styles.steps}>
            <div className={styles.step}>1. Identity</div>
            <div className={`${styles.step} ${styles.activeStep}`}>2. Recovery</div>
            <div className={styles.step}>3. Access</div>
          </div>
          <h2 className={styles.wizardTitle}>Recovery checkpoint</h2>
          <p className={styles.wizardSubtitle}>
            Write this phrase down offline, then prove backup with selected words.
          </p>

          <div className={`${styles.status} ${styles.warn}`}>
            {PHRASE.join(" ")}
          </div>

          <div className={styles.fieldGroup}>
            <label className={styles.label} htmlFor="word3">
              Enter word 3
            </label>
            <input
              id="word3"
              className={styles.input}
              value={word3}
              onChange={(event) => setWord3(event.target.value)}
            />
          </div>
          <div className={styles.fieldGroup}>
            <label className={styles.label} htmlFor="word7">
              Enter word 7
            </label>
            <input
              id="word7"
              className={styles.input}
              value={word7}
              onChange={(event) => setWord7(event.target.value)}
            />
          </div>
          <div className={styles.fieldGroup}>
            <label className={styles.label} htmlFor="word11">
              Enter word 11
            </label>
            <input
              id="word11"
              className={styles.input}
              value={word11}
              onChange={(event) => setWord11(event.target.value)}
            />
          </div>

          {confirmed ? (
            <div className={`${styles.status} ${styles.ok}`}>
              Recovery backup status: confirmed.
            </div>
          ) : (
            <div className={`${styles.status} ${styles.error}`}>
              recovery_unconfirmed: words do not match required positions.
            </div>
          )}

          <div className={styles.ctaRow}>
            <Link className={styles.buttonGhost} href="/onboarding/identity">
              Back to identity
            </Link>
            <Link
              aria-disabled={!confirmed}
              className={styles.button}
              href={confirmed ? "/onboarding/access" : "#"}
              onClick={(event) => {
                if (!confirmed) {
                  event.preventDefault();
                }
              }}
            >
              Continue to access
            </Link>
          </div>
        </main>

        <aside className={styles.panel}>
          <h3 className={styles.wizardTitle}>Setup status</h3>
          <div className={styles.asideList}>
            <div className={styles.asideItem}>Identity: ready</div>
            <div className={styles.asideItem}>
              Recovery: {confirmed ? "confirmed" : "pending"}
            </div>
            <div className={styles.asideItem}>Access path: pending</div>
          </div>
        </aside>
      </div>
    </div>
  );
}
