"use client";

import Link from "next/link";

import { OnboardingShell } from "@/components/onboarding/onboarding-shell";
import styles from "../onboarding.module.css";

export default function AccessOnboardingPage() {
  return (
    <OnboardingShell
      activeStep="access"
      finalStepLabel="3. Complete"
      introBody="Server join and friend-request actions now happen inside the main app to keep onboarding focused and predictable."
      introTitle="You are ready to enter HexRelay"
      promises={[
        "Onboarding stays focused on identity and recovery only.",
        "Join server and add contact flows are in app surfaces.",
        "You can start testing immediately from Home and Hubs.",
      ]}
      statusItems={["Identity: ready", "Recovery: confirmed", "Access flows: moved to app hubs"]}
      wizardSubtitle="Continue to the app. Use Servers and Contacts hubs for join and request workflows."
      wizardTitle="Onboarding complete"
    >
      <div className={`${styles.status} ${styles.ok}`}>Identity and recovery checkpoints passed.</div>

      <div className={styles.choiceGrid}>
        <div className={styles.choice}>
          <p className={styles.choiceTitle}>Join server in-app</p>
          <p className={styles.choiceText}>Use the Servers hub after onboarding.</p>
        </div>
        <div className={styles.choice}>
          <p className={styles.choiceTitle}>Manage contacts in-app</p>
          <p className={styles.choiceText}>Use the Contacts hub for friend requests.</p>
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
    </OnboardingShell>
  );
}
