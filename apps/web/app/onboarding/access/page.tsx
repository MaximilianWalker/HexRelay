"use client";

import Link from "next/link";

import styles from "../onboarding.module.css";

export default function AccessOnboardingPage() {
  return (
    <div className={styles.shell}>
      <div className={styles.content}>
        <aside className={styles.panel}>
          <p className={styles.brandEyebrow}>HexRelay onboarding</p>
          <h1 className={styles.leftTitle}>You are ready to enter HexRelay</h1>
          <p className={styles.leftBody}>
            Server join and contact invite actions now happen inside the main app
            to keep onboarding focused and predictable.
          </p>
          <ul className={styles.promiseList}>
            <li>Onboarding stays focused on identity and recovery only.</li>
            <li>Join server and add contact flows are in app surfaces.</li>
            <li>You can start testing immediately from Home and Hubs.</li>
          </ul>
        </aside>

        <main className={styles.panel}>
          <div className={styles.steps}>
            <div className={styles.step}>1. Identity</div>
            <div className={styles.step}>2. Recovery</div>
            <div className={`${styles.step} ${styles.activeStep}`}>3. Complete</div>
          </div>
          <h2 className={styles.wizardTitle}>Onboarding complete</h2>
          <p className={styles.wizardSubtitle}>
            Continue to the app. Use Servers and Contacts hubs for join and invite
            workflows.
          </p>

          <div className={`${styles.status} ${styles.ok}`}>
            Identity and recovery checkpoints passed.
          </div>

          <div className={styles.choiceGrid}>
            <div className={styles.choice}>
              <p className={styles.choiceTitle}>Join server in-app</p>
              <p className={styles.choiceText}>
                Use the Servers hub after onboarding.
              </p>
            </div>
            <div className={styles.choice}>
              <p className={styles.choiceTitle}>Manage contacts in-app</p>
              <p className={styles.choiceText}>
                Use the Contacts hub for requests and invite actions.
              </p>
            </div>
          </div>

          <div className={styles.ctaRow}>
            <Link className={styles.buttonGhost} href="/onboarding/recovery">
              Back to recovery
            </Link>
            <Link className={styles.button} href="/home">
              Enter HexRelay
            </Link>
          </div>
        </main>

        <aside className={styles.panel}>
          <h3 className={styles.wizardTitle}>Setup status</h3>
          <div className={styles.asideList}>
            <div className={styles.asideItem}>Identity: ready</div>
            <div className={styles.asideItem}>Recovery: confirmed</div>
            <div className={styles.asideItem}>Access flows: moved to app hubs</div>
          </div>
        </aside>
      </div>
    </div>
  );
}
